//! The FIXED, host-agnostic semantic-cost schema (ticket 07, decision `D32`).
//!
//! `D32` requires semantic cost/profile output to use a fixed schema with stable
//! field names for CLI/LSP/tests. This module is the single owner of that
//! schema. [`SemanticCostV2`] carries EXACTLY these four fields and no ad-hoc
//! string cost knobs:
//!
//! - `estimated_cardinality_class`
//! - `recursion_risk_class`
//! - `witness_explosion_risk_class`
//! - `ordering_contract`
//!
//! There is deliberately no `unsupported_reason_code` success field: a
//! non-servable shape fails closed with a typed
//! [`crate::PlanningDiagnosticV1`] instead, so a successful plan can never carry
//! a dead `none` field. Each field is a TYPED class, not a free-form string
//! (`D24`), and no physical estimate (`cpu`, `memory`, `io`, `network`,
//! `coordination`) or numeric cardinality exists anywhere in the schema.

/// How big the result is expected to get, as a coarse host-agnostic class.
///
/// This is a SEMANTIC class derived from the checked shape (scans, joins,
/// grouping, sealed limits), never a runtime row estimate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardinalityClassV2 {
    /// At most one row (a scalar aggregate with no grouping, or a `limit 1`).
    ScalarOrSingleton,
    /// Bounded by an explicit sealed limit.
    Bounded,
    /// Proportional to a single base relation (filters/projections of one scan).
    LinearInInput,
    /// A join / cartesian product can multiply input sizes.
    Multiplicative,
    /// Recursive closure: size is a fixpoint, potentially much larger than any
    /// single input.
    Closure,
}

impl CardinalityClassV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::ScalarOrSingleton => "scalar_or_singleton",
            Self::Bounded => "bounded",
            Self::LinearInInput => "linear_in_input",
            Self::Multiplicative => "multiplicative",
            Self::Closure => "closure",
        }
    }

    /// Explicit risk rank, used to take the worst class of two union arms.
    pub(crate) const fn rank(self) -> u8 {
        match self {
            Self::ScalarOrSingleton => 0,
            Self::Bounded => 1,
            Self::LinearInInput => 2,
            Self::Multiplicative => 3,
            Self::Closure => 4,
        }
    }
}

/// How much recursion risk the plan carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecursionRiskClassV2 {
    /// No recursion in the plan.
    None,
    /// Positive recursion admitted under a least-fixpoint stratification proof:
    /// terminates, but iterates to a fixpoint.
    BoundedFixpoint,
    /// Recursion whose termination depends on an explicit witness/path bound
    /// (a cyclic relation made finite only by the bound).
    BoundDependent,
}

impl RecursionRiskClassV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BoundedFixpoint => "bounded_fixpoint",
            Self::BoundDependent => "bound_dependent",
        }
    }
}

/// How much witness/path enumeration could explode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WitnessExplosionRiskClassV2 {
    /// No witness/path materialization is requested.
    None,
    /// A single canonical witness is requested: enumeration is capped at one.
    SingleWitness,
    /// A bounded top-k / all-witness set with an explicit finite cap.
    BoundedSet,
    /// Witness enumeration over a cyclic relation; finite only because an
    /// explicit path-length bound caps it.
    CapBounded,
}

impl WitnessExplosionRiskClassV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SingleWitness => "single_witness",
            Self::BoundedSet => "bounded_set",
            Self::CapBounded => "cap_bounded",
        }
    }
}

/// The order contract of the rendered result.
///
/// Per `D26` and pinned-semantics 6, an unordered result stays semantically
/// unordered; tooling may canonicalize rendering, but the language must not
/// pretend an order exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderingContractV2 {
    /// Output order is NOT semantic (a set or bag); any rendered order is a
    /// tooling canonicalization, not a language guarantee.
    Unordered,
    /// Output order IS semantic: the plan ends in an explicit checked sort, so
    /// the row order is part of the result's meaning.
    Ordered,
}

impl OrderingContractV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Unordered => "unordered",
            Self::Ordered => "ordered",
        }
    }
}

/// The FIXED semantic-cost schema (`D32`): exactly four typed fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticCostV2 {
    /// Coarse expected-size class of the result.
    pub estimated_cardinality_class: CardinalityClassV2,
    /// Recursion-risk class of the plan.
    pub recursion_risk_class: RecursionRiskClassV2,
    /// Witness/path explosion-risk class.
    pub witness_explosion_risk_class: WitnessExplosionRiskClassV2,
    /// Order contract of the rendered result.
    pub ordering_contract: OrderingContractV2,
}

impl SemanticCostV2 {
    /// The four stable field names, in declaration order. This is the drift
    /// guard `D32` requires: a test asserts these verbatim so a rename cannot
    /// pass silently.
    pub const FIELD_NAMES: [&'static str; 4] = [
        "estimated_cardinality_class",
        "recursion_risk_class",
        "witness_explosion_risk_class",
        "ordering_contract",
    ];

    /// The canonical ordered field iterator: `(field name, keyword)` pairs in
    /// declaration order, for the CLI renderer to consume instead of hardcoding
    /// a second field list.
    #[must_use]
    pub fn fields(&self) -> [(&'static str, &'static str); 4] {
        [
            (
                "estimated_cardinality_class",
                self.estimated_cardinality_class.keyword(),
            ),
            ("recursion_risk_class", self.recursion_risk_class.keyword()),
            (
                "witness_explosion_risk_class",
                self.witness_explosion_risk_class.keyword(),
            ),
            ("ordering_contract", self.ordering_contract.keyword()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_field_names_are_stable_and_four() {
        assert_eq!(
            SemanticCostV2::FIELD_NAMES,
            [
                "estimated_cardinality_class",
                "recursion_risk_class",
                "witness_explosion_risk_class",
                "ordering_contract",
            ]
        );
        let cost = SemanticCostV2 {
            estimated_cardinality_class: CardinalityClassV2::Bounded,
            recursion_risk_class: RecursionRiskClassV2::None,
            witness_explosion_risk_class: WitnessExplosionRiskClassV2::None,
            ordering_contract: OrderingContractV2::Unordered,
        };
        let rendered = cost.fields();
        assert_eq!(rendered.len(), SemanticCostV2::FIELD_NAMES.len());
        for (index, (name, keyword)) in rendered.iter().enumerate() {
            assert_eq!(Some(*name), SemanticCostV2::FIELD_NAMES.get(index).copied());
            assert!(!keyword.is_empty());
        }
    }

    #[test]
    fn class_keywords_are_stable() {
        for (class, keyword) in [
            (CardinalityClassV2::ScalarOrSingleton, "scalar_or_singleton"),
            (CardinalityClassV2::Bounded, "bounded"),
            (CardinalityClassV2::LinearInInput, "linear_in_input"),
            (CardinalityClassV2::Multiplicative, "multiplicative"),
            (CardinalityClassV2::Closure, "closure"),
        ] {
            assert_eq!(class.keyword(), keyword);
        }
        for (class, keyword) in [
            (RecursionRiskClassV2::None, "none"),
            (RecursionRiskClassV2::BoundedFixpoint, "bounded_fixpoint"),
            (RecursionRiskClassV2::BoundDependent, "bound_dependent"),
        ] {
            assert_eq!(class.keyword(), keyword);
        }
        for (class, keyword) in [
            (WitnessExplosionRiskClassV2::None, "none"),
            (WitnessExplosionRiskClassV2::SingleWitness, "single_witness"),
            (WitnessExplosionRiskClassV2::BoundedSet, "bounded_set"),
            (WitnessExplosionRiskClassV2::CapBounded, "cap_bounded"),
        ] {
            assert_eq!(class.keyword(), keyword);
        }
        for (class, keyword) in [
            (OrderingContractV2::Unordered, "unordered"),
            (OrderingContractV2::Ordered, "ordered"),
        ] {
            assert_eq!(class.keyword(), keyword);
        }
    }
}
