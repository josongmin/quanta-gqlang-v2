//! Result-kind, witness-selection policy, and multiplicity/order contract
//! (ticket 07, decisions `D20` / `D26`).
//!
//! `D20`: result kinds, source locations, and witness/path payloads are SEMANTIC
//! outputs of the language stack, not CLI-only presentation. This module owns the
//! typed result-kind model and the render-requirement contract derived from the
//! normalized algebra/logic shape.
//!
//! `D26`/pinned-semantics 6: multiplicity, duplicate-elimination, and output
//! order are explicit and testable; an unordered result stays unordered. The
//! [`MultiplicityOrderContractV1`] carries the algebra [`CardinalityV1`] forward
//! so tooling cannot invent an order the language does not guarantee.

use quanta_gqlang_v2_types::CardinalityV1;

/// The KIND of result the plan renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResultKindV1 {
    /// A relational table of rows.
    Table,
    /// An alert: rows that must carry a source location (file/span) payload.
    Alert,
    /// A path/witness result: enumerated concrete witnesses over a (recursive)
    /// relation.
    PathWitness,
    /// A proof/explain result: rule/path provenance without execution, surfacing
    /// WHY a result holds rather than the result rows themselves.
    ProofExplain,
}

impl ResultKindV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Alert => "alert",
            Self::PathWitness => "path_witness",
            Self::ProofExplain => "proof_explain",
        }
    }
}

/// The witness-selection policy, made EXPLICIT rather than backend-accidental
/// (pinned-semantics 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WitnessSelectionPolicyV1 {
    /// Materialize exactly one canonical witness per result (the tie-break in the
    /// witness contract decides which).
    CanonicalOne,
    /// Materialize at most the top `k` witnesses under the contract's tie-break.
    TopK {
        /// The finite cap `k`.
        k: u64,
    },
    /// Materialize all witnesses up to the contract's max-witness cap.
    AllWitness,
}

impl WitnessSelectionPolicyV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering. `TopK`
    /// renders its keyword without the count so the keyword set stays closed;
    /// the count is available via the field.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::CanonicalOne => "canonical_one",
            Self::TopK { .. } => "top_k",
            Self::AllWitness => "all_witness",
        }
    }
}

/// The multiplicity / duplicate / order contract of the rendered result, derived
/// from the algebra [`CardinalityV1`] of the plan root (`D26`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiplicityOrderContractV1 {
    /// The algebra cardinality of the result root: the single source of truth
    /// for whether duplicates are eliminated and whether order is semantic.
    pub cardinality: CardinalityV1,
}

impl MultiplicityOrderContractV1 {
    /// Build the contract from an algebra root cardinality.
    #[must_use]
    pub const fn from_cardinality(cardinality: CardinalityV1) -> Self {
        Self { cardinality }
    }

    /// Whether duplicate rows are eliminated (a `set`), as opposed to preserved
    /// (`bag`/`sequence`).
    #[must_use]
    pub const fn eliminates_duplicates(self) -> bool {
        matches!(self.cardinality, CardinalityV1::Set)
    }

    /// Whether output ORDER is semantic. Only a `Sequence` orders rows; a set or
    /// bag does NOT, and tooling must not pretend otherwise (pinned-semantics 6).
    #[must_use]
    pub const fn is_ordered(self) -> bool {
        matches!(self.cardinality, CardinalityV1::Sequence)
    }
}

/// What each rendered result row must carry for the result kind to be
/// well-formed (`D20`). Distinct from the truth-lane evidence requirement: this
/// is the RENDER payload contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderRequirementV1 {
    /// Nothing beyond the result columns.
    RowsOnly,
    /// Each row must carry a source location (file/span).
    SourceLocation,
    /// Each result must carry a concrete witness/path payload.
    WitnessPayload,
    /// The result is a provenance/proof tree rather than rows.
    ProvenanceTree,
}

impl RenderRequirementV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::RowsOnly => "rows_only",
            Self::SourceLocation => "source_location",
            Self::WitnessPayload => "witness_payload",
            Self::ProvenanceTree => "provenance_tree",
        }
    }
}
