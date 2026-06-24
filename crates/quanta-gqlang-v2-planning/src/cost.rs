//! The FIXED, host-agnostic semantic-cost schema (ticket 07, decision `D32`).
//!
//! `D32` requires semantic cost/profile output to use a fixed schema with stable
//! field names for CLI/LSP/tests. This module is the single owner of that
//! schema. The struct [`SemanticCostV1`] carries EXACTLY these five fields and no
//! ad-hoc string cost knobs:
//!
//! - `estimated_cardinality_class`
//! - `recursion_risk_class`
//! - `witness_explosion_risk_class`
//! - `ordering_contract`
//! - `unsupported_reason_code`
//!
//! Each field is a TYPED class, not a free-form string (`D24`: cost signals are
//! frontend-owned host-agnostic outputs; no runtime jargon). The field names are
//! guarded against drift by [`SemanticCostV1::FIELD_NAMES`] and a stability rail
//! in the crate tests.

/// How big the result is expected to get, as a coarse host-agnostic class.
///
/// This is a SEMANTIC class derived from the normalized algebra shape (scans,
/// joins, grouping), never a runtime row estimate. It does not pretend to be a
/// physical cardinality number — that belongs downstream (`D24`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardinalityClassV1 {
    /// At most one row (a scalar aggregate with no grouping, or a `limit 1`).
    ScalarOrSingleton,
    /// Bounded by an explicit `limit` clause.
    Bounded,
    /// Proportional to a single base relation (filters/projections of one scan).
    LinearInInput,
    /// A join / cartesian product can multiply input sizes.
    Multiplicative,
    /// Recursive closure: size is a fixpoint, potentially much larger than any
    /// single input.
    Closure,
}

impl CardinalityClassV1 {
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
}

/// How much recursion risk the plan carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecursionRiskClassV1 {
    /// No recursion in the plan.
    None,
    /// Positive recursion admitted under a least-fixpoint stratification proof:
    /// terminates, but iterates to a fixpoint.
    BoundedFixpoint,
    /// Recursion whose termination depends on an explicit witness/path bound
    /// (a cyclic relation made finite only by the bound).
    BoundDependent,
}

impl RecursionRiskClassV1 {
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
pub enum WitnessExplosionRiskClassV1 {
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

impl WitnessExplosionRiskClassV1 {
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
/// pretend an order exists. This class makes that distinction explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderingContractV1 {
    /// Output order is NOT semantic (a set or bag); any rendered order is a
    /// tooling canonicalization, not a language guarantee.
    Unordered,
    /// Output order IS semantic: the plan ends in an explicit sort, so the row
    /// order is part of the result's meaning.
    Ordered,
}

impl OrderingContractV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Unordered => "unordered",
            Self::Ordered => "ordered",
        }
    }
}

/// The FIXED semantic-cost schema (`D32`).
///
/// Exactly five typed fields, named verbatim. The `unsupported_reason_code` is
/// `Some(code)` only when the plan is non-servable due to an unsupported
/// construct mix (it carries the same stable code as the planning diagnostic);
/// for a servable plan it is `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticCostV1 {
    /// Coarse expected-size class of the result.
    pub estimated_cardinality_class: CardinalityClassV1,
    /// Recursion-risk class of the plan.
    pub recursion_risk_class: RecursionRiskClassV1,
    /// Witness/path explosion-risk class.
    pub witness_explosion_risk_class: WitnessExplosionRiskClassV1,
    /// Order contract of the rendered result.
    pub ordering_contract: OrderingContractV1,
    /// `Some(code)` when the plan is non-servable due to an unsupported
    /// construct, carrying the stable reason code; `None` when servable.
    pub unsupported_reason_code: Option<String>,
}

impl SemanticCostV1 {
    /// The five stable field names, in declaration order. This is the drift
    /// guard `D32` requires: a test asserts these verbatim so a rename cannot
    /// pass silently.
    pub const FIELD_NAMES: [&'static str; 5] = [
        "estimated_cardinality_class",
        "recursion_risk_class",
        "witness_explosion_risk_class",
        "ordering_contract",
        "unsupported_reason_code",
    ];
}
