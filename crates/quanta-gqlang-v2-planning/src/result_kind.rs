//! Result-kind, witness-selection, multiplicity/order, and the assembled result
//! contract (ticket 07, decisions `D20` / `D26`).
//!
//! `D20`: result kinds, source locations, and witness/path payloads are SEMANTIC
//! outputs of the language stack, not CLI-only presentation.
//!
//! `D26`/pinned-semantics 6: multiplicity, duplicate-elimination, and output
//! order are explicit and testable; an unordered result stays unordered. The
//! [`MultiplicityOrderContractV2`] carries relational cardinality forward, or
//! records the planner-owned family rule for a non-relational result. FIND,
//! CHECK and presentation results are unordered sets as specified in
//! `docs/contracts/planning-results.md`; they do not have a relational root
//! whose cardinality could be copied. Tooling cannot invent a semantic order.
//!
//! [`ResultContractV2`] is not an empty result-kind wrapper: it retains the
//! closed [`ResultKindV2`], the per-row [`RenderRequirementV2`], the optional
//! bounded [`WitnessSelectionPolicyV2`], and the
//! [`MultiplicityOrderContractV2`].

use quanta_gqlang_v2_types::CardinalityV1;

/// The KIND of result the plan renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResultKindV2 {
    /// A relational table of rows.
    Table,
    /// An alert: rows that must carry a source location (file/span) payload.
    Alert,
    /// A path/witness result: enumerated concrete witnesses over a relation.
    PathWitness,
    /// A proof/explain result: provenance without execution.
    ProofExplain,
}

impl ResultKindV2 {
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
pub enum WitnessSelectionPolicyV2 {
    /// Materialize exactly one canonical witness per result (the sealed
    /// tie-break decides which).
    CanonicalOne,
    /// Materialize at most the top `k` witnesses under the sealed tie-break.
    TopK {
        /// The finite cap `k`.
        k: u64,
    },
    /// Materialize every witness up to the sealed max-witness cap.
    AllWitness,
}

impl WitnessSelectionPolicyV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering. `TopK`
    /// renders its keyword without the count so the keyword set stays closed;
    /// the count is read from the variant.
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
/// from the sealed relational root or the planner-owned non-relational family
/// rule (`D26`; see `docs/contracts/planning-results.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiplicityOrderContractV2 {
    /// The planner's result cardinality: the single source of truth for
    /// whether duplicates are eliminated and whether order is semantic.
    pub cardinality: CardinalityV1,
}

impl MultiplicityOrderContractV2 {
    /// Retain a cardinality supplied by the planning owner. A standalone
    /// value of this carrier does not confer checked-plan authority.
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
    /// bag does NOT, and tooling must not pretend otherwise.
    #[must_use]
    pub const fn is_ordered(self) -> bool {
        matches!(self.cardinality, CardinalityV1::Sequence)
    }
}

/// What each rendered result row must carry for the result kind to be
/// well-formed (`D20`). Distinct from the authority-lane evidence requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderRequirementV2 {
    /// Nothing beyond the result columns.
    RowsOnly,
    /// Each row must carry a source location (file/span).
    SourceLocation,
    /// Each finding must be rendered in the SARIF interchange shape, so the
    /// export contract is part of the plan rather than a renderer choice.
    SarifFindings,
    /// Each result must carry a concrete witness/path payload.
    WitnessPayload,
    /// The result is a provenance/proof tree rather than rows.
    ProvenanceTree,
}

impl RenderRequirementV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::RowsOnly => "rows_only",
            Self::SourceLocation => "source_location",
            Self::SarifFindings => "sarif_findings",
            Self::WitnessPayload => "witness_payload",
            Self::ProvenanceTree => "provenance_tree",
        }
    }
}

/// The assembled result contract of one semantic plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultContractV2 {
    result_kind: ResultKindV2,
    render_requirement: RenderRequirementV2,
    witness_selection: Option<WitnessSelectionPolicyV2>,
    multiplicity_order: MultiplicityOrderContractV2,
}

impl ResultContractV2 {
    /// Assemble one result contract from its four sealed-derived parts.
    #[must_use]
    pub const fn new(
        result_kind: ResultKindV2,
        render_requirement: RenderRequirementV2,
        witness_selection: Option<WitnessSelectionPolicyV2>,
        multiplicity_order: MultiplicityOrderContractV2,
    ) -> Self {
        Self {
            result_kind,
            render_requirement,
            witness_selection,
            multiplicity_order,
        }
    }

    /// The closed result kind.
    #[must_use]
    pub const fn result_kind(&self) -> ResultKindV2 {
        self.result_kind
    }

    /// The per-row render-payload requirement.
    #[must_use]
    pub const fn render_requirement(&self) -> RenderRequirementV2 {
        self.render_requirement
    }

    /// The bounded witness-selection policy, when witnesses are materialized.
    #[must_use]
    pub const fn witness_selection(&self) -> Option<WitnessSelectionPolicyV2> {
        self.witness_selection
    }

    /// The multiplicity/order contract of the rendered result.
    #[must_use]
    pub const fn multiplicity_order(&self) -> MultiplicityOrderContractV2 {
        self.multiplicity_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_kind_and_render_keywords_are_stable() {
        for (kind, keyword) in [
            (ResultKindV2::Table, "table"),
            (ResultKindV2::Alert, "alert"),
            (ResultKindV2::PathWitness, "path_witness"),
            (ResultKindV2::ProofExplain, "proof_explain"),
        ] {
            assert_eq!(kind.keyword(), keyword);
        }
        for (render, keyword) in [
            (RenderRequirementV2::RowsOnly, "rows_only"),
            (RenderRequirementV2::SourceLocation, "source_location"),
            (RenderRequirementV2::SarifFindings, "sarif_findings"),
            (RenderRequirementV2::WitnessPayload, "witness_payload"),
            (RenderRequirementV2::ProvenanceTree, "provenance_tree"),
        ] {
            assert_eq!(render.keyword(), keyword);
        }
        for (selection, keyword) in [
            (WitnessSelectionPolicyV2::CanonicalOne, "canonical_one"),
            (WitnessSelectionPolicyV2::TopK { k: 3 }, "top_k"),
            (WitnessSelectionPolicyV2::AllWitness, "all_witness"),
        ] {
            assert_eq!(selection.keyword(), keyword);
        }
    }

    #[test]
    fn multiplicity_order_reads_the_sealed_cardinality() {
        assert!(
            MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Set)
                .eliminates_duplicates()
        );
        assert!(
            !MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Bag)
                .eliminates_duplicates()
        );
        assert!(
            MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Sequence).is_ordered()
        );
        assert!(!MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Set).is_ordered());
    }

    #[test]
    fn result_contract_retains_every_part() {
        let contract = ResultContractV2::new(
            ResultKindV2::PathWitness,
            RenderRequirementV2::WitnessPayload,
            Some(WitnessSelectionPolicyV2::AllWitness),
            MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Set),
        );
        assert_eq!(contract.result_kind(), ResultKindV2::PathWitness);
        assert_eq!(
            contract.render_requirement(),
            RenderRequirementV2::WitnessPayload
        );
        assert_eq!(
            contract.witness_selection(),
            Some(WitnessSelectionPolicyV2::AllWitness)
        );
        assert_eq!(
            contract.multiplicity_order().cardinality,
            CardinalityV1::Set
        );
    }
}
