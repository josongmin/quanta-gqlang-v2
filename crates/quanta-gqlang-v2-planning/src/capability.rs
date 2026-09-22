//! Capability requirement vocabulary over checked algebra families (ticket 07).
//!
//! A capability is a host-agnostic semantic requirement ("this plan needs
//! grouping", "this plan needs recursive closure"). It is NOT a runtime/provider
//! noun (`D7`). The V2 cut deletes the V1 recursive walk
//! (`derive_algebra_capabilities`/`collect`): capabilities are now collected by
//! the single iterative, deterministic, budgeted traversal run by
//! [`crate::plan_outcome`], which keys every capability off the checked algebra
//! family of a node it has actually charged a semantic step for.

use quanta_gqlang_v2_algebra::AlgebraFamilyV2;

/// A single host-agnostic capability the plan requires.
///
/// Capabilities are derived purely from the presence of checked constructs; the
/// closed set below maps one-to-one onto algebra families and sealed logic
/// shapes the planner can classify.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapabilityV2 {
    /// Reading a base relation or a composition working set (a `Scan`).
    RelationScan,
    /// Row-level filtering (a `Filter`).
    Filtering,
    /// Joining independent inputs (a `Join`).
    Join,
    /// Correlated/lateral evaluation (an `Apply`).
    CorrelatedApply,
    /// Grouping / partitioning (a `Group`).
    Grouping,
    /// Aggregate folds (an `Aggregate`).
    Aggregation,
    /// Ordering output (a `Sort`).
    Ordering,
    /// Bounded truncation (a `Limit`).
    Limiting,
    /// Set/bag union (a `Union`).
    SetUnion,
    /// Existence testing (a `SemiJoin`).
    ExistenceCheck,
    /// Non-existence testing (an `AntiJoin`).
    NegationCheck,
    /// Recursive closure (a document rule program with a recursive member).
    /// Derived from the sealed logic proof, never from the algebra walk.
    RecursiveClosure,
    /// Witness/path materialization. Derived from a sealed witness contract
    /// (FIND or a witness-bearing presentation), never from the algebra walk.
    WitnessMaterialization,
}

impl CapabilityV2 {
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

/// The capability a checked algebra family requires, if any.
///
/// `Scan`, `Filter`, `Join`, `Apply`, `Group`, `Aggregate`, `Sort`, `Limit`,
/// `Union`, `SemiJoin`, and `AntiJoin` each require exactly one capability;
/// `Project` is a pure frame re-shape and requires none.
pub(crate) const fn capability_for_family(family: AlgebraFamilyV2) -> Option<CapabilityV2> {
    match family {
        AlgebraFamilyV2::Scan => Some(CapabilityV2::RelationScan),
        AlgebraFamilyV2::Filter => Some(CapabilityV2::Filtering),
        AlgebraFamilyV2::Join => Some(CapabilityV2::Join),
        AlgebraFamilyV2::Apply => Some(CapabilityV2::CorrelatedApply),
        AlgebraFamilyV2::Group => Some(CapabilityV2::Grouping),
        AlgebraFamilyV2::Aggregate => Some(CapabilityV2::Aggregation),
        AlgebraFamilyV2::Sort => Some(CapabilityV2::Ordering),
        AlgebraFamilyV2::Limit => Some(CapabilityV2::Limiting),
        AlgebraFamilyV2::Union => Some(CapabilityV2::SetUnion),
        AlgebraFamilyV2::SemiJoin => Some(CapabilityV2::ExistenceCheck),
        AlgebraFamilyV2::AntiJoin => Some(CapabilityV2::NegationCheck),
        AlgebraFamilyV2::Project => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_FAMILIES: [AlgebraFamilyV2; 12] = [
        AlgebraFamilyV2::Scan,
        AlgebraFamilyV2::Filter,
        AlgebraFamilyV2::Project,
        AlgebraFamilyV2::Join,
        AlgebraFamilyV2::Apply,
        AlgebraFamilyV2::Group,
        AlgebraFamilyV2::Aggregate,
        AlgebraFamilyV2::Sort,
        AlgebraFamilyV2::Limit,
        AlgebraFamilyV2::Union,
        AlgebraFamilyV2::SemiJoin,
        AlgebraFamilyV2::AntiJoin,
    ];

    #[test]
    fn every_family_has_a_stable_capability_except_project() {
        for family in ALL_FAMILIES {
            let capability = capability_for_family(family);
            if family == AlgebraFamilyV2::Project {
                assert_eq!(capability, None, "Project is a pure frame re-shape");
            } else {
                assert!(
                    capability.is_some(),
                    "{family:?} must require one capability"
                );
            }
        }
    }

    #[test]
    fn capability_keywords_are_stable_verbatim() {
        let cases = [
            (CapabilityV2::RelationScan, "relation_scan"),
            (CapabilityV2::Filtering, "filtering"),
            (CapabilityV2::Join, "join"),
            (CapabilityV2::CorrelatedApply, "correlated_apply"),
            (CapabilityV2::Grouping, "grouping"),
            (CapabilityV2::Aggregation, "aggregation"),
            (CapabilityV2::Ordering, "ordering"),
            (CapabilityV2::Limiting, "limiting"),
            (CapabilityV2::SetUnion, "set_union"),
            (CapabilityV2::ExistenceCheck, "existence_check"),
            (CapabilityV2::NegationCheck, "negation_check"),
            (CapabilityV2::RecursiveClosure, "recursive_closure"),
            (
                CapabilityV2::WitnessMaterialization,
                "witness_materialization",
            ),
        ];
        for (capability, keyword) in cases {
            assert_eq!(capability.keyword(), keyword);
        }
    }
}
