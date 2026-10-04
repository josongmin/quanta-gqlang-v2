//! Deterministic lexical + relation scope model with EXPLICIT capture edges.
//!
//! Cites `D25`: nested/path forms use lexical, binder-visible scoping only; no
//! hidden path-scoping or implicit capture algorithm may become semantic
//! authority. The model therefore makes capture an explicit property of a scope
//! frame: a frame is either *capturing* (it can see bindings in its enclosing
//! frame) or *non-capturing* (a correlation barrier). Resolving a name that
//! lives beyond a non-capturing boundary is an explicit illegal-capture reject,
//! never a silent success.
//!
//! Two binder namespaces are modelled and never collapsed:
//!
//! - the *value* namespace: aliases, parameters, and `let` bindings that a
//!   bare symbol or a path BASE resolves against,
//! - the *declaration* namespace: top-level declaration/rule/view names, kept
//!   in a separate flat table ([`crate::resolve`]).
//!
//! Path access (`u.id`) resolves only the base symbol `u` here; segment names
//! (`id`) are field selectors typed later (ticket 03), honouring the `D25` rule
//! that no hidden path-scoping resolves them.

use quanta_gqlang_v2_hir::{HirNodeIdV1, SpanV1};

/// What a value-namespace name binds to. Open enough for ticket 03 to type, but
/// the binder fixes the *origin kind* so explain/diagnostics are precise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinderKindV1 {
    /// A relation source alias (`... AS u`).
    Alias,
    /// A declaration/rule/view formal parameter.
    Param,
    /// A local `let`-style binding.
    Let,
    /// A quantifier-introduced bound variable (`exists x ...`).
    Quantified,
    /// A composition `LET <name> = <producer>` working-set binding. Visible to
    /// later producers, the composition body, and the `WITH` consumer.
    CompositionLet,
}

/// A single name binding introduced into a scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingV1 {
    /// The bound name as written.
    pub name: String,
    /// The origin kind of the binding.
    pub kind: BinderKindV1,
    /// Span of the binding site (the alias/param/let name).
    pub span: SpanV1,
    /// The HIR node id of the binding site, so downstream side-tables can key
    /// resolutions back to the introducing node.
    pub node: HirNodeIdV1,
    /// Zero-based ordinal within the owning HIR node. The pair `(node,
    /// ordinal)` is the stable binding identity; owner node alone is not
    /// sufficient for multi-binder quantifiers or projections.
    ordinal: usize,
}

impl BindingV1 {
    /// Construct a binding.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        kind: BinderKindV1,
        span: SpanV1,
        node: HirNodeIdV1,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            span,
            node,
            ordinal: 0,
        }
    }

    /// Construct a binding with an explicit ordinal within its owner node.
    #[must_use]
    pub fn new_indexed(
        name: impl Into<String>,
        kind: BinderKindV1,
        span: SpanV1,
        node: HirNodeIdV1,
        ordinal: usize,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            span,
            node,
            ordinal,
        }
    }

    /// Stable `(owner node, ordinal)` identity.
    #[must_use]
    pub const fn identity(&self) -> (HirNodeIdV1, usize) {
        (self.node, self.ordinal)
    }
}

/// Whether a scope frame may see bindings in its enclosing frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureV1 {
    /// The frame can resolve names against enclosing frames (lexical capture is
    /// allowed across this edge).
    Capturing,
    /// The frame is a correlation barrier: names that only exist in an
    /// enclosing frame are NOT visible and resolving them is an explicit
    /// illegal-capture reject (`D25`).
    NonCapturing,
}

/// One lexical scope frame: its own bindings plus the capture discipline of the
/// edge to its parent.
#[derive(Clone, Debug, PartialEq, Eq)]
struct FrameV1 {
    bindings: Vec<BindingV1>,
    /// Capture discipline of the edge from this frame UP to its parent.
    parent_edge: CaptureV1,
}

/// The outcome of resolving a name against the lexical scope stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexicalResolutionV1 {
    /// The name resolved to a visible binding in a reachable frame.
    Bound(BindingV1),
    /// The name exists only beyond a non-capturing boundary; capturing it is
    /// illegal. Carries the binding it would have resolved to so the caller can
    /// build a precise [`crate::diagnostic::BindDiagnosticV1::IllegalCapture`].
    IllegalCapture(BindingV1),
    /// The name is not bound in any frame on the stack.
    Unbound,
}

/// A deterministic lexical scope stack with explicit capture edges.
///
/// Frames are pushed on scope entry and popped on exit. Resolution walks from
/// the innermost frame outward; the first frame whose edge to its parent is
/// non-capturing acts as a barrier: bindings strictly beyond it are reported as
/// illegal captures rather than resolved.
#[derive(Clone, Debug, Default)]
pub struct LexicalScopeV1 {
    frames: Vec<FrameV1>,
}

impl LexicalScopeV1 {
    /// An empty scope stack.
    #[must_use]
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub(crate) const fn one_frame_storage_bytes_v3() -> usize {
        std::mem::size_of::<FrameV1>()
    }

    pub(crate) fn try_reserve_one_frame_v3(
        &mut self,
    ) -> Result<(), std::collections::TryReserveError> {
        self.frames.try_reserve_exact(1)
    }

    pub(crate) fn one_frame_capacity_v3(&self) -> bool {
        self.frames.capacity() == 1
    }

    pub(crate) const fn two_frame_storage_bytes_v3() -> usize {
        2 * std::mem::size_of::<FrameV1>()
    }

    pub(crate) fn try_reserve_two_frames_v3(
        &mut self,
    ) -> Result<(), std::collections::TryReserveError> {
        self.frames.try_reserve_exact(2)
    }

    pub(crate) fn two_frame_capacity_v3(&self) -> bool {
        self.frames.capacity() == 2
    }

    pub(crate) fn push_prepared_frame_v3(
        &mut self,
        parent_edge: CaptureV1,
        bindings: Vec<BindingV1>,
    ) -> bool {
        if self.frames.len() >= self.frames.capacity() {
            return false;
        }
        self.frames.push(FrameV1 {
            bindings,
            parent_edge,
        });
        true
    }

    pub(crate) fn is_unborn_v3(&self) -> bool {
        self.frames.is_empty() && self.frames.capacity() == 0
    }

    /// Push a new innermost frame whose edge to the current top has the given
    /// capture discipline.
    pub fn push(&mut self, parent_edge: CaptureV1) {
        self.frames.push(FrameV1 {
            bindings: Vec::new(),
            parent_edge,
        });
    }

    /// Pop the innermost frame. No-op on an empty stack (callers balance their
    /// own push/pop; a no-op pop can never under/over-resolve).
    pub fn pop(&mut self) {
        let _ = self.frames.pop();
    }

    /// Add a binding to the innermost frame.
    ///
    /// Returns the span of an existing binding of the same name in the SAME
    /// frame (a duplicate in one scope) or, failing that, the span of a binding
    /// in an enclosing frame that this one shadows. Returning `None` means the
    /// name is fresh on the visible stack. The caller decides whether a shadow
    /// is a reject; the scope model only reports it.
    pub fn bind(&mut self, binding: BindingV1) -> Option<ShadowReportV1> {
        let report = self.same_frame_or_outer(&binding.name);
        if let Some(frame) = self.frames.last_mut() {
            frame.bindings.push(binding);
        }
        report
    }

    fn same_frame_or_outer(&self, name: &str) -> Option<ShadowReportV1> {
        let top = self.frames.last()?;
        for existing in &top.bindings {
            if existing.name == name {
                return Some(ShadowReportV1 {
                    kind: ShadowReportKindV1::SameFrameDuplicate,
                    existing: existing.clone(),
                });
            }
        }
        // A binding only "shadows" an enclosing one if that enclosing binding is
        // actually VISIBLE from the new binding's frame. A non-capturing edge is
        // a correlation barrier: bindings strictly beyond it are invisible (a
        // reference to them is an illegal capture, not a resolution), so the new
        // binding hides nothing across it. Stop searching once the edge UP from
        // the top frame is non-capturing — anything beyond it is out of scope.
        if top.parent_edge == CaptureV1::NonCapturing {
            return None;
        }
        // Search enclosing frames (everything but the top), innermost-out, and
        // stop at the first non-capturing edge (the visibility barrier).
        let upper = self.frames.len().saturating_sub(1);
        for frame in self.frames.iter().take(upper).rev() {
            for existing in &frame.bindings {
                if existing.name == name {
                    return Some(ShadowReportV1 {
                        kind: ShadowReportKindV1::OuterShadow,
                        existing: existing.clone(),
                    });
                }
            }
            if frame.parent_edge == CaptureV1::NonCapturing {
                break;
            }
        }
        None
    }

    /// Resolve `name` against ONLY the innermost (current) frame, never any
    /// enclosing frame.
    ///
    /// A composition `WITH <name>` must consume a `LET` defined in the SAME
    /// composition frame, not an enclosing composition's `LET` (`D25`: only
    /// binder-visible, lexical scoping — and the working set a `WITH` consumes
    /// is frame-local, so an inner `WITH` naming an outer `LET` must fail closed
    /// rather than capture it). This resolution therefore deliberately does not
    /// walk parents: a name found only in an enclosing frame is [`Unbound`] here.
    ///
    /// [`Unbound`]: LexicalResolutionV1::Unbound
    #[must_use]
    pub fn resolve_current_frame(&self, name: &str) -> LexicalResolutionV1 {
        if let Some(frame) = self.frames.last() {
            for binding in frame.bindings.iter().rev() {
                if binding.name == name {
                    return LexicalResolutionV1::Bound(binding.clone());
                }
            }
        }
        LexicalResolutionV1::Unbound
    }

    /// Resolve `name` against the visible scope stack, honouring capture edges.
    #[must_use]
    pub fn resolve(&self, name: &str) -> LexicalResolutionV1 {
        self.resolve_with_capture(name).0
    }

    /// Resolve a name and report the exact frame index containing its binding.
    /// An inaccessible binding beyond a non-capturing edge remains an
    /// [`LexicalResolutionV1::IllegalCapture`] with no usable frame index.
    pub(crate) fn resolve_with_capture(&self, name: &str) -> (LexicalResolutionV1, Option<usize>) {
        // `crossed_barrier` flips true once we walk OUT through a non-capturing
        // edge; any binding found after that point is an illegal capture.
        let mut crossed_barrier = false;
        for (frame_index, frame) in self.frames.iter().enumerate().rev() {
            for binding in frame.bindings.iter().rev() {
                if binding.name == name {
                    return if crossed_barrier {
                        (LexicalResolutionV1::IllegalCapture(binding.clone()), None)
                    } else {
                        (
                            LexicalResolutionV1::Bound(binding.clone()),
                            Some(frame_index),
                        )
                    };
                }
            }
            // Walking from this frame to its parent crosses `parent_edge`.
            if frame.parent_edge == CaptureV1::NonCapturing {
                crossed_barrier = true;
            }
        }
        (LexicalResolutionV1::Unbound, None)
    }

    /// Internal frame position for explicit relational capture ownership.
    pub(crate) fn current_frame_index(&self) -> Option<usize> {
        self.frames.len().checked_sub(1)
    }

    /// Replace the bindings in the current frame with one complete relation
    /// stage output. This is the row-frame transition primitive: projection
    /// drops its input aliases, while join installs the merged left/right
    /// output only after both inputs have been resolved independently.
    pub(crate) fn replace_current_bindings(&mut self, bindings: Vec<BindingV1>) {
        if let Some(frame) = self.frames.last_mut() {
            frame.bindings = bindings;
        }
    }

    /// First visible binding outside the current frame. Relation-stage frame
    /// replacement uses this to preserve the shadowing rule without treating a
    /// projected output as a duplicate of the input frame it replaces.
    pub(crate) fn visible_outer_binding(&self, name: &str) -> Option<&BindingV1> {
        let top = self.frames.last()?;
        if top.parent_edge == CaptureV1::NonCapturing {
            return None;
        }
        let upper = self.frames.len().saturating_sub(1);
        for frame in self.frames.iter().take(upper).rev() {
            if let Some(binding) = frame
                .bindings
                .iter()
                .rev()
                .find(|binding| binding.name == name)
            {
                return Some(binding);
            }
            if frame.parent_edge == CaptureV1::NonCapturing {
                break;
            }
        }
        None
    }
}

/// A name collision report from [`LexicalScopeV1::bind`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowReportV1 {
    /// Whether the collision is a same-frame duplicate or an outer shadow.
    pub kind: ShadowReportKindV1,
    /// The pre-existing binding that collided.
    pub existing: BindingV1,
}

/// Classification of a name collision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowReportKindV1 {
    /// Two bindings of the same name in one frame (a duplicate).
    SameFrameDuplicate,
    /// A binding shadows one in an enclosing frame.
    OuterShadow,
}
