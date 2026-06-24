//! Logical explain / provenance contract over normalized semantics (ticket 07,
//! decision `D24`).
//!
//! `D24`: logical explain/provenance and semantic cost signals are FRONTEND-owned
//! host-agnostic outputs; runtime physical plans and substrate profiling stay
//! downstream. This module owns the LOGICAL explain note: an ordered list of the
//! normalized algebra families the result is built from, plus whether the program
//! is recursive, anchored to the root's source span. It says WHAT normalized
//! constructs produce the result, never HOW a runtime would execute it.

use quanta_gqlang_v2_algebra::{AlgebraFamilyV1, AlgebraNodeV1, AlgebraOpV1};
use quanta_gqlang_v2_hir::SpanV1;

use crate::plan::RecursionShapeV1;

/// A logical explain note: a host-agnostic provenance summary of the normalized
/// plan. Ordered top-down (root family first) so CLI/LSP/tests see a stable
/// rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainNoteV1 {
    /// Source span of the plan root (`D20`).
    pub root_span: SpanV1,
    /// The algebra families present, root-first, in stable pre-order.
    pub families: Vec<AlgebraFamilyV1>,
    /// Whether the program is recursive (a least-fixpoint closure).
    pub recursive: bool,
}

impl ExplainNoteV1 {
    /// Build a logical explain note from the normalized root and recursion shape.
    #[must_use]
    pub fn from_root(root: &AlgebraNodeV1, recursion: &RecursionShapeV1) -> Self {
        let mut families = Vec::new();
        collect_families(root, &mut families);
        Self {
            root_span: root.span,
            families,
            recursive: matches!(recursion, RecursionShapeV1::Fixpoint(_)),
        }
    }

    /// Build a logical explain note for a NON-relational program item (FIND /
    /// CHECK / EXPLAIN / SHOW / DESCRIBE / SAMPLE / COMPARE / EXPORT / META).
    ///
    /// Such an item has no normalized algebra tree, so the family list is empty;
    /// its provenance is carried by the plan's result-kind / effect / capability
    /// classification, anchored to the item span. `recursive` is set when the
    /// item's underlying reasoning is a (bounded) recursive closure, e.g. a FIND
    /// over a cyclic call/flow graph.
    #[must_use]
    pub const fn for_item(item_span: SpanV1, recursive: bool) -> Self {
        Self {
            root_span: item_span,
            families: Vec::new(),
            recursive,
        }
    }
}

fn collect_families(node: &AlgebraNodeV1, out: &mut Vec<AlgebraFamilyV1>) {
    out.push(node.family());
    match &node.op {
        AlgebraOpV1::Scan(_) => {}
        AlgebraOpV1::Project(project) => collect_families(&project.input, out),
        AlgebraOpV1::Filter(filter) => collect_families(&filter.input, out),
        AlgebraOpV1::Group(group) => collect_families(&group.input, out),
        AlgebraOpV1::Aggregate(aggregate) => collect_families(&aggregate.input, out),
        AlgebraOpV1::Sort(sort) => collect_families(&sort.input, out),
        AlgebraOpV1::Limit(limit) => collect_families(&limit.input, out),
        AlgebraOpV1::Join(join) => {
            collect_families(&join.left, out);
            collect_families(&join.right, out);
        }
        AlgebraOpV1::Apply(apply) => {
            collect_families(&apply.left, out);
            collect_families(&apply.right, out);
        }
        AlgebraOpV1::Union(union) => {
            collect_families(&union.left, out);
            collect_families(&union.right, out);
        }
        AlgebraOpV1::SemiJoin(semi) => {
            collect_families(&semi.left, out);
            collect_families(&semi.right, out);
        }
        AlgebraOpV1::AntiJoin(anti) => {
            collect_families(&anti.left, out);
            collect_families(&anti.right, out);
        }
    }
}
