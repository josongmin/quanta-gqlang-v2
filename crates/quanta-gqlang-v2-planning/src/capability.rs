//! Capability requirement derivation from NORMALIZED query features (ticket 07,
//! decision `D6`).
//!
//! `D6`: planning attaches to the normalized query/program shape, not only the
//! root kind. This module derives the set of host-agnostic CAPABILITY
//! requirements a plan needs by inspecting which algebra operators and logic
//! constructs are actually present — never by re-inspecting syntax or root labels
//! (ticket 07 non-goal 3).
//!
//! A capability is a host-agnostic semantic requirement (e.g. "this plan needs
//! grouping", "this plan needs recursive closure"). It is NOT a runtime/provider
//! noun (`D7`).

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{AlgebraNodeV1, AlgebraOpV1};

/// A single host-agnostic capability the plan requires.
///
/// Capabilities are derived purely from the presence of normalized constructs;
/// the closed set below maps one-to-one onto algebra families and logic shapes
/// the planner can classify.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityV1 {
    /// Reading a base relation (any [`AlgebraOpV1::Scan`]).
    RelationScan,
    /// Row-level filtering (a [`AlgebraOpV1::Filter`]).
    Filtering,
    /// Joining independent inputs (a [`AlgebraOpV1::Join`]).
    Join,
    /// Correlated/lateral evaluation (a [`AlgebraOpV1::Apply`]).
    CorrelatedApply,
    /// Grouping / partitioning (a [`AlgebraOpV1::Group`]).
    Grouping,
    /// Aggregate folds (a [`AlgebraOpV1::Aggregate`]).
    Aggregation,
    /// Ordering output (a [`AlgebraOpV1::Sort`]).
    Ordering,
    /// Bounded truncation (a [`AlgebraOpV1::Limit`]).
    Limiting,
    /// Set/bag union (a [`AlgebraOpV1::Union`]).
    SetUnion,
    /// Existence testing (a [`AlgebraOpV1::SemiJoin`]).
    ExistenceCheck,
    /// Non-existence testing (a [`AlgebraOpV1::AntiJoin`]).
    NegationCheck,
    /// Recursive closure (a recursive logic program). Derived from the logic
    /// shape, attached by the producer rather than the algebra walk.
    RecursiveClosure,
    /// Witness/path materialization. Derived from a witness contract, attached by
    /// the producer.
    WitnessMaterialization,
}

impl CapabilityV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::RelationScan => "relation_scan",
            Self::Filtering => "filtering",
            Self::Join => "join",
            Self::CorrelatedApply => "correlated_apply",
            Self::Grouping => "grouping",
            Self::Aggregation => "aggregation",
            Self::Ordering => "ordering",
            Self::Limiting => "limiting",
            Self::SetUnion => "set_union",
            Self::ExistenceCheck => "existence_check",
            Self::NegationCheck => "negation_check",
            Self::RecursiveClosure => "recursive_closure",
            Self::WitnessMaterialization => "witness_materialization",
        }
    }
}

/// Walk a normalized algebra tree and collect every capability its operators
/// imply. Deterministic and ordered (a `BTreeSet`), so CLI/LSP/tests see a
/// stable requirement set. Logic-derived capabilities (recursive closure,
/// witness materialization) are added by the producer, not here.
#[must_use]
pub fn derive_algebra_capabilities(root: &AlgebraNodeV1) -> BTreeSet<CapabilityV1> {
    let mut caps = BTreeSet::new();
    collect(root, &mut caps);
    caps
}

fn collect(node: &AlgebraNodeV1, caps: &mut BTreeSet<CapabilityV1>) {
    match &node.op {
        AlgebraOpV1::Scan(_) => {
            caps.insert(CapabilityV1::RelationScan);
        }
        AlgebraOpV1::Project(project) => {
            collect(&project.input, caps);
        }
        AlgebraOpV1::Filter(filter) => {
            caps.insert(CapabilityV1::Filtering);
            collect(&filter.input, caps);
        }
        AlgebraOpV1::Join(join) => {
            caps.insert(CapabilityV1::Join);
            collect(&join.left, caps);
            collect(&join.right, caps);
        }
        AlgebraOpV1::Apply(apply) => {
            caps.insert(CapabilityV1::CorrelatedApply);
            collect(&apply.left, caps);
            collect(&apply.right, caps);
        }
        AlgebraOpV1::Group(group) => {
            caps.insert(CapabilityV1::Grouping);
            collect(&group.input, caps);
        }
        AlgebraOpV1::Aggregate(aggregate) => {
            caps.insert(CapabilityV1::Aggregation);
            collect(&aggregate.input, caps);
        }
        AlgebraOpV1::Sort(sort) => {
            caps.insert(CapabilityV1::Ordering);
            collect(&sort.input, caps);
        }
        AlgebraOpV1::Limit(limit) => {
            caps.insert(CapabilityV1::Limiting);
            collect(&limit.input, caps);
        }
        AlgebraOpV1::Union(union) => {
            caps.insert(CapabilityV1::SetUnion);
            collect(&union.left, caps);
            collect(&union.right, caps);
        }
        AlgebraOpV1::SemiJoin(semi) => {
            caps.insert(CapabilityV1::ExistenceCheck);
            collect(&semi.left, caps);
            collect(&semi.right, caps);
        }
        AlgebraOpV1::AntiJoin(anti) => {
            caps.insert(CapabilityV1::NegationCheck);
            collect(&anti.left, caps);
            collect(&anti.right, caps);
        }
    }
}
