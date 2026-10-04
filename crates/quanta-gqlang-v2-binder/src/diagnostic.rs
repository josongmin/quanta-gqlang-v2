//! Typed binder diagnostics with stable machine-readable codes.
//!
//! Every resolution failure is a typed [`BindDiagnosticV1`] value carrying a
//! stable [`BindDiagnosticV1::code`] string and a human `Display` rendering.
//! Cites packet decisions:
//!
//! - `D3`: binding/scope resolution is mandatory before algebra lowering, so
//!   these are first-class fail-closed errors, not warnings.
//! - `D28`: import conflicts fail closed deterministically; there is no
//!   last-wins or import-order-sensitive override, so an ambiguous import is a
//!   hard typed reject ([`BindDiagnosticV1::AmbiguousImport`]).
//! - `D25`: nested/path forms use lexical, binder-visible scoping only; an
//!   attempt to capture across a non-capturing scope boundary is an explicit
//!   typed reject ([`BindDiagnosticV1::IllegalCapture`]) rather than a silent
//!   hidden-scope success.

use quanta_gqlang_v2_hir::SpanV1;
use std::cmp::Ordering;
use std::fmt;

fn decimal_render_order(mut left: usize, mut right: usize) -> Ordering {
    fn highest_place(mut value: usize) -> usize {
        let mut place = 1usize;
        while value >= 10 {
            value /= 10;
            place *= 10;
        }
        place
    }
    let mut left_place = highest_place(left);
    let mut right_place = highest_place(right);
    loop {
        let digit_order = (left / left_place).cmp(&(right / right_place));
        if digit_order != Ordering::Equal {
            return digit_order;
        }
        if left_place == 1 || right_place == 1 {
            return (left_place != 1).cmp(&(right_place != 1));
        }
        left %= left_place;
        right %= right_place;
        left_place /= 10;
        right_place /= 10;
    }
}

fn span_render_order(left: SpanV1, right: SpanV1) -> Ordering {
    decimal_render_order(left.start, right.start)
        .then_with(|| decimal_render_order(left.end, right.end))
}

fn joined_candidates_render_order(left: &[String], right: &[String]) -> Ordering {
    fn bytes(candidates: &[String]) -> impl Iterator<Item = u8> + '_ {
        candidates.iter().enumerate().flat_map(|(index, value)| {
            let separator: &[u8] = if index == 0 { b"" } else { b", " };
            separator.iter().copied().chain(value.bytes())
        })
    }
    bytes(left).cmp(bytes(right))
}

/// A typed binder diagnostic.
///
/// Each variant carries the offending name, the source span to point at, and
/// any companion spans needed to explain the conflict. Construction is the only
/// way the binder reports a resolution failure; there is no untyped error path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindDiagnosticV1 {
    /// A referenced symbol does not resolve to any visible binder.
    Unresolved {
        /// The unresolved name as written.
        name: String,
        /// Span of the offending reference.
        span: SpanV1,
    },
    /// Two declarations in the same symbol namespace share a name.
    Duplicate {
        /// The duplicated declared name.
        name: String,
        /// Span of the first declaration seen with this name.
        first_span: SpanV1,
        /// Span of the second (colliding) declaration.
        second_span: SpanV1,
    },
    /// A binding shadows an outer binding of the same name. Reported as a
    /// fail-closed reject (deterministic, not order-sensitive) so shadowing is
    /// never silent.
    Shadowed {
        /// The shadowed name.
        name: String,
        /// Span of the outer (shadowed) binding.
        outer_span: SpanV1,
        /// Span of the inner binding that shadows it.
        inner_span: SpanV1,
    },
    /// An imported name resolves to more than one module-graph entry. Per `D28`
    /// this is a hard reject with no import-order tiebreak.
    AmbiguousImport {
        /// The ambiguous imported name.
        name: String,
        /// Span of the reference that triggered the ambiguous lookup.
        span: SpanV1,
        /// The distinct fully-qualified module paths that collided, in the
        /// deterministic sorted order the binder reports them.
        candidates: Vec<String>,
    },
    /// An aggregate argument could name both a private pre-group input and a
    /// top-level declaration/import. Neither authority may silently win.
    PartitionAuthorityCollision {
        /// Conflicting authored name.
        name: String,
        /// Exact reference in the post-group expression.
        span: SpanV1,
        /// Original pre-group binding location.
        partition_span: SpanV1,
    },
    /// A relation source names a local value rather than a composition LET
    /// working-set producer. A same-spelled host relation cannot override it.
    WrongRelationSourceKind {
        /// Authored source name.
        name: String,
        /// Exact source-name token span, excluding any alias.
        span: SpanV1,
        /// Location of the incompatible local binding.
        binding_span: SpanV1,
    },
    /// Two resolved uses reuse the same node, span, and spelling. Node-only
    /// lookup is insufficient for multi-reference graph items.
    DuplicateReferenceUse {
        /// Repeated authored spelling.
        name: String,
        /// Repeated exact use span.
        span: SpanV1,
    },
    /// A reference reaches across a non-capturing scope boundary without an
    /// explicit allowed capture edge. Per `D25` correlated capture must be an
    /// explicit scope edge, never an implicit hidden-scope success.
    IllegalCapture {
        /// The captured name.
        name: String,
        /// Span of the offending reference.
        span: SpanV1,
        /// Span of the binding it illegally tried to capture.
        binding_span: SpanV1,
    },
}

impl BindDiagnosticV1 {
    /// Stable machine-readable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Unresolved { .. } => "binder.unresolved_symbol",
            Self::Duplicate { .. } => "binder.duplicate_declaration",
            Self::Shadowed { .. } => "binder.shadowed_binding",
            Self::AmbiguousImport { .. } => "binder.ambiguous_import",
            Self::PartitionAuthorityCollision { .. } => "binder.partition_authority_collision",
            Self::WrongRelationSourceKind { .. } => "binder.wrong_relation_source_kind",
            Self::DuplicateReferenceUse { .. } => "binder.duplicate_reference_use",
            Self::IllegalCapture { .. } => "binder.illegal_capture",
        }
    }

    /// The primary name the diagnostic is about.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Unresolved { name, .. }
            | Self::Duplicate { name, .. }
            | Self::Shadowed { name, .. }
            | Self::AmbiguousImport { name, .. }
            | Self::PartitionAuthorityCollision { name, .. }
            | Self::WrongRelationSourceKind { name, .. }
            | Self::DuplicateReferenceUse { name, .. }
            | Self::IllegalCapture { name, .. } => name,
        }
    }

    /// The primary source span the diagnostic points at.
    #[must_use]
    pub fn primary_span(&self) -> SpanV1 {
        match self {
            Self::Unresolved { span, .. }
            | Self::AmbiguousImport { span, .. }
            | Self::PartitionAuthorityCollision { span, .. }
            | Self::WrongRelationSourceKind { span, .. }
            | Self::DuplicateReferenceUse { span, .. }
            | Self::IllegalCapture { span, .. } => *span,
            Self::Duplicate { second_span, .. } => *second_span,
            Self::Shadowed { inner_span, .. } => *inner_span,
        }
    }

    /// Exact Display tie-break without allocating rendered diagnostics inside
    /// the sort comparator. Call only after primary span, code, and name tie.
    #[must_use]
    pub(crate) fn render_tie_break_order(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::Duplicate {
                    first_span: left, ..
                },
                Self::Duplicate {
                    first_span: right, ..
                },
            )
            | (
                Self::Shadowed {
                    outer_span: left, ..
                },
                Self::Shadowed {
                    outer_span: right, ..
                },
            )
            | (
                Self::PartitionAuthorityCollision {
                    partition_span: left,
                    ..
                },
                Self::PartitionAuthorityCollision {
                    partition_span: right,
                    ..
                },
            )
            | (
                Self::WrongRelationSourceKind {
                    binding_span: left, ..
                },
                Self::WrongRelationSourceKind {
                    binding_span: right,
                    ..
                },
            )
            | (
                Self::IllegalCapture {
                    binding_span: left, ..
                },
                Self::IllegalCapture {
                    binding_span: right,
                    ..
                },
            ) => span_render_order(*left, *right),
            (
                Self::AmbiguousImport {
                    candidates: left, ..
                },
                Self::AmbiguousImport {
                    candidates: right, ..
                },
            ) => joined_candidates_render_order(left, right),
            _ => Ordering::Equal,
        }
    }
}

impl fmt::Display for BindDiagnosticV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unresolved { name, span } => write!(
                f,
                "{}: unresolved symbol `{}` at {}..{}",
                self.code(),
                name,
                span.start,
                span.end,
            ),
            Self::Duplicate {
                name,
                first_span,
                second_span,
            } => write!(
                f,
                "{}: duplicate declaration `{}` (first at {}..{}, again at {}..{})",
                self.code(),
                name,
                first_span.start,
                first_span.end,
                second_span.start,
                second_span.end,
            ),
            Self::Shadowed {
                name,
                outer_span,
                inner_span,
            } => write!(
                f,
                "{}: binding `{}` at {}..{} shadows outer binding at {}..{}",
                self.code(),
                name,
                inner_span.start,
                inner_span.end,
                outer_span.start,
                outer_span.end,
            ),
            Self::AmbiguousImport {
                name,
                span,
                candidates,
            } => {
                write!(
                    f,
                    "{}: imported symbol `{}` at {}..{} is ambiguous between ",
                    self.code(),
                    name,
                    span.start,
                    span.end,
                )?;
                for (index, candidate) in candidates.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_str(candidate)?;
                }
                Ok(())
            }
            Self::PartitionAuthorityCollision {
                name,
                span,
                partition_span,
            } => write!(
                f,
                "{}: `{}` at {}..{} also names a private pre-group binding at {}..{}",
                self.code(),
                name,
                span.start,
                span.end,
                partition_span.start,
                partition_span.end,
            ),
            Self::WrongRelationSourceKind {
                name,
                span,
                binding_span,
            } => {
                write!(
                f,
                "{}: relation source `{}` at {}..{} names a non-relation local binding at {}..{}",
                self.code(), name, span.start, span.end, binding_span.start, binding_span.end,
            )
            }
            Self::DuplicateReferenceUse { name, span } => write!(
                f,
                "{}: duplicate reference use `{}` at {}..{}",
                self.code(),
                name,
                span.start,
                span.end,
            ),
            Self::IllegalCapture {
                name,
                span,
                binding_span,
            } => write!(
                f,
                "{}: reference `{}` at {}..{} illegally captures binding at {}..{} \
                 across a non-capturing boundary",
                self.code(),
                name,
                span.start,
                span.end,
                binding_span.start,
                binding_span.end,
            ),
        }
    }
}

impl std::error::Error for BindDiagnosticV1 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_tie_break_matches_display_at_digit_boundaries() {
        let values = [0, 1, 9, 10, 11, 99, 100, usize::MAX];
        for left in values {
            for right in values {
                assert_eq!(
                    decimal_render_order(left, right),
                    left.to_string().cmp(&right.to_string())
                );
            }
        }
    }

    #[test]
    fn typed_tie_break_matches_rendered_diagnostic_order() {
        let cases = [
            (
                BindDiagnosticV1::Duplicate {
                    name: "x".into(),
                    first_span: SpanV1::new(9, 10),
                    second_span: SpanV1::new(30, 31),
                },
                BindDiagnosticV1::Duplicate {
                    name: "x".into(),
                    first_span: SpanV1::new(10, 11),
                    second_span: SpanV1::new(30, 31),
                },
            ),
            (
                BindDiagnosticV1::AmbiguousImport {
                    name: "x".into(),
                    span: SpanV1::new(30, 31),
                    candidates: vec!["a".into(), "z".into()],
                },
                BindDiagnosticV1::AmbiguousImport {
                    name: "x".into(),
                    span: SpanV1::new(30, 31),
                    candidates: vec!["a+".into()],
                },
            ),
            (
                BindDiagnosticV1::IllegalCapture {
                    name: "x".into(),
                    span: SpanV1::new(30, 31),
                    binding_span: SpanV1::new(9, 10),
                },
                BindDiagnosticV1::IllegalCapture {
                    name: "x".into(),
                    span: SpanV1::new(30, 31),
                    binding_span: SpanV1::new(10, 11),
                },
            ),
        ];
        for (left, right) in cases {
            assert_eq!(
                left.render_tie_break_order(&right),
                left.to_string().cmp(&right.to_string())
            );
        }
    }
}
