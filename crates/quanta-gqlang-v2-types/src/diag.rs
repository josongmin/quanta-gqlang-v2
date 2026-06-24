//! Typed type-checking diagnostics (ticket 03, decision `D4`/`D10`).
//!
//! Per `D4` type errors are typed, not stringly; per `D10` unsupported
//! constructs reject explicitly with no silent downgrade. Every diagnostic
//! carries a stable machine-readable [`TypeErrorV1::code`] plus a human
//! `Display`, and a [`SpanV1`] back to source so CLI/LSP can underline.

use crate::ty::TypeV1;
use quanta_gqlang_v2_hir::SpanV1;
use std::fmt;

/// A typed type-checking error.
///
/// The three load-bearing reject classes required by the ticket are
/// [`TypeErrorV1::Mismatch`], [`TypeErrorV1::BadOverload`], and
/// [`TypeErrorV1::IllegalAggregate`]; the remaining variants make the
/// fail-closed surface complete (unknown symbols/operators/builtins, arity,
/// bad projection) so nothing degrades to runtime guessing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeErrorV1 {
    /// A value's actual type did not match the type required by its context.
    Mismatch {
        /// Where the offending value sits.
        span: SpanV1,
        /// The type the context required.
        expected: TypeV1,
        /// The type that was actually produced.
        actual: TypeV1,
    },
    /// An operator was applied to operand types with no matching overload.
    BadOverload {
        /// Where the operator application sits.
        span: SpanV1,
        /// The operator symbol as written.
        operator: String,
        /// The operand types supplied, in order.
        operands: Vec<TypeV1>,
    },
    /// An aggregate builtin was used in a position that does not admit
    /// aggregation, or with a non-aggregable argument type.
    IllegalAggregate {
        /// Where the aggregate call sits.
        span: SpanV1,
        /// The aggregate builtin name.
        function: String,
        /// Why the aggregate is illegal here.
        reason: IllegalAggregateReasonV1,
    },
    /// A bare symbol was not present in the caller-provided type environment.
    UnknownSymbol {
        /// Where the symbol reference sits.
        span: SpanV1,
        /// The unresolved name.
        name: String,
    },
    /// A unary operator symbol is not in the closed operator registry.
    UnknownUnaryOperator {
        /// Where the operator sits.
        span: SpanV1,
        /// The unknown operator symbol.
        operator: String,
    },
    /// A binary operator symbol is not in the closed operator registry.
    UnknownBinaryOperator {
        /// Where the operator sits.
        span: SpanV1,
        /// The unknown operator symbol.
        operator: String,
    },
    /// A called builtin name is not in the closed builtin registry.
    UnknownBuiltin {
        /// Where the callee sits.
        span: SpanV1,
        /// The unknown builtin name.
        name: String,
    },
    /// A builtin was called with the wrong number of arguments.
    ArityMismatch {
        /// Where the call sits.
        span: SpanV1,
        /// The builtin name.
        function: String,
        /// Number of arguments the (overload set) expects.
        expected: usize,
        /// Number of arguments supplied.
        actual: usize,
    },
    /// A path/field projection segment is not valid for the base type.
    BadProjection {
        /// Where the projecting segment sits.
        span: SpanV1,
        /// The base type the projection was applied to.
        base: TypeV1,
        /// The segment name that did not resolve.
        segment: String,
    },
    /// A malformed numeric literal whose text does not denote its claimed type.
    MalformedLiteral {
        /// Where the literal sits.
        span: SpanV1,
        /// Which scalar the literal claimed to be.
        expected: TypeV1,
        /// The raw literal text.
        text: String,
    },
}

/// Why an aggregate usage is illegal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IllegalAggregateReasonV1 {
    /// The aggregate's argument type cannot be aggregated by this function.
    NonAggregableArgument {
        /// The argument type supplied.
        argument: TypeV1,
    },
    /// The aggregate received an unexpected argument count.
    BadArity {
        /// Arguments expected.
        expected: usize,
        /// Arguments supplied.
        actual: usize,
    },
}

impl TypeErrorV1 {
    /// Stable machine-readable diagnostic code, shared by CLI/LSP/tests.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Mismatch { .. } => "types.mismatch",
            Self::BadOverload { .. } => "types.bad_overload",
            Self::IllegalAggregate { .. } => "types.illegal_aggregate",
            Self::UnknownSymbol { .. } => "types.unknown_symbol",
            Self::UnknownUnaryOperator { .. } => "types.unknown_unary_operator",
            Self::UnknownBinaryOperator { .. } => "types.unknown_binary_operator",
            Self::UnknownBuiltin { .. } => "types.unknown_builtin",
            Self::ArityMismatch { .. } => "types.arity_mismatch",
            Self::BadProjection { .. } => "types.bad_projection",
            Self::MalformedLiteral { .. } => "types.malformed_literal",
        }
    }

    /// The source span this diagnostic points at.
    #[must_use]
    pub const fn span(&self) -> SpanV1 {
        match self {
            Self::Mismatch { span, .. }
            | Self::BadOverload { span, .. }
            | Self::IllegalAggregate { span, .. }
            | Self::UnknownSymbol { span, .. }
            | Self::UnknownUnaryOperator { span, .. }
            | Self::UnknownBinaryOperator { span, .. }
            | Self::UnknownBuiltin { span, .. }
            | Self::ArityMismatch { span, .. }
            | Self::BadProjection { span, .. }
            | Self::MalformedLiteral { span, .. } => *span,
        }
    }
}

fn write_type_list(f: &mut fmt::Formatter<'_>, types: &[TypeV1]) -> fmt::Result {
    for (idx, ty) in types.iter().enumerate() {
        if idx > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{ty}")?;
    }
    Ok(())
}

impl fmt::Display for TypeErrorV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mismatch {
                expected, actual, ..
            } => write!(
                f,
                "{}: expected type `{expected}`, found `{actual}`",
                self.code()
            ),
            Self::BadOverload {
                operator, operands, ..
            } => {
                write!(
                    f,
                    "{}: operator `{operator}` has no overload for (",
                    self.code()
                )?;
                write_type_list(f, operands)?;
                f.write_str(")")
            }
            Self::IllegalAggregate {
                function, reason, ..
            } => match reason {
                IllegalAggregateReasonV1::NonAggregableArgument { argument } => write!(
                    f,
                    "{}: aggregate `{function}` cannot aggregate argument of type `{argument}`",
                    self.code()
                ),
                IllegalAggregateReasonV1::BadArity { expected, actual } => write!(
                    f,
                    "{}: aggregate `{function}` expects {expected} argument(s), found {actual}",
                    self.code()
                ),
            },
            Self::UnknownSymbol { name, .. } => {
                write!(
                    f,
                    "{}: unknown symbol `{name}` in type environment",
                    self.code()
                )
            }
            Self::UnknownUnaryOperator { operator, .. } => {
                write!(f, "{}: unknown unary operator `{operator}`", self.code())
            }
            Self::UnknownBinaryOperator { operator, .. } => {
                write!(f, "{}: unknown binary operator `{operator}`", self.code())
            }
            Self::UnknownBuiltin { name, .. } => {
                write!(f, "{}: unknown builtin function `{name}`", self.code())
            }
            Self::ArityMismatch {
                function,
                expected,
                actual,
                ..
            } => write!(
                f,
                "{}: builtin `{function}` expects {expected} argument(s), found {actual}",
                self.code()
            ),
            Self::BadProjection { base, segment, .. } => {
                write!(f, "{}: type `{base}` has no field `{segment}`", self.code())
            }
            Self::MalformedLiteral { expected, text, .. } => write!(
                f,
                "{}: malformed `{expected}` literal `{text}`",
                self.code()
            ),
        }
    }
}

impl std::error::Error for TypeErrorV1 {}
