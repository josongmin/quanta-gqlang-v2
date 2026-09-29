//! The checked composition planning input and the composition policy a plan
//! retains (ticket 07, junction + composition consumer audit).
//!
//! A Composition is one root item whose plan is its BODY's plan: planning
//! dispatches to the body's existing plan policy through the SAME classifier and
//! never classifies a second time, never synthesizes a plan per `LET`, and never
//! turns a nested producer output into a top-level plan.
//!
//! [`CheckedCompositionPlanInputV2::new`] therefore retains, from Sema's sealed
//! [`ResolvedCompositionV2`]:
//!
//! - the composition root identity and its checked location;
//! - the ordered producers (authored LET ordinal, item identity, and the exact
//!   relation output each one exposes, if any);
//! - the exact `WITH` binding identity, which must be exactly one of those
//!   producers;
//! - the sealed body identity (closed family plus the body's own item identity),
//!   and the caller's ONE sealed body input, whose kind and identity must match
//!   that body.
//!
//! Producer ordinals, identities, and outputs are read from the sealed
//! composition rather than supplied by the caller, so a caller cannot transplant
//! them. [`CompositionRetentionV2`] is the ONE place those retained facts are
//! checked, and [`CheckedCompositionPlanInputV2::policy`] is the only way to
//! obtain a [`CompositionPolicyV2`]: it wraps the body policy the ONE classifier
//! derived from this input's own body, so a plan can neither drop the
//! composition identity nor attach a policy that describes another body.

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::CheckedAlgebraNodeV2;
use quanta_gqlang_v2_logic::GraphQueryContractV2;
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, RelationOutputContractV2, ResolvedCompositionBindingV2,
    ResolvedCompositionBodyV2, ResolvedCompositionV2, ResolvedValueBindingV2, SemanticItemIdV2,
    TypedRelationV2,
};

use crate::check::CheckContractV2;
use crate::diagnostic::{CompositionInvalidReasonV2, PlanningDiagnosticV1};
use crate::policy::PlanPolicyV2;
use crate::presentation::PresentationContractV2;

/// The closed family of one composition body.
///
/// The four variants mirror Sema's closed [`ResolvedCompositionBodyV2`] one to
/// one and the four families a composition body policy can retain, so the body a
/// carrier names and the body a policy was classified from are compared in ONE
/// vocabulary instead of a third hand-written matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompositionBodyFamilyV2 {
    /// A SELECT/query body.
    Query,
    /// A FIND body sealed by Logic.
    Graph,
    /// A CHECK body validated by this crate.
    Check,
    /// An EXPLAIN body validated by this crate.
    Explain,
}

impl CompositionBodyFamilyV2 {
    /// Stable family keyword for diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Graph => "graph",
            Self::Check => "check",
            Self::Explain => "explain",
        }
    }

    /// The family one classified carrier policy belongs to, when that carrier is
    /// a legal composition body.
    ///
    /// A document rule program, a meta directive, and a nested composition are
    /// not composition bodies, so their policies have no body family.
    const fn of_policy(policy: &PlanPolicyV2) -> Option<Self> {
        match policy {
            PlanPolicyV2::Relational { .. } => Some(Self::Query),
            PlanPolicyV2::Graph(_) => Some(Self::Graph),
            PlanPolicyV2::Check(_) => Some(Self::Check),
            PlanPolicyV2::Presentation(_) => Some(Self::Explain),
            PlanPolicyV2::Rule(_)
            | PlanPolicyV2::MetaRequireCertified
            | PlanPolicyV2::Composition(_)
            | PlanPolicyV2::Command(_) => None,
        }
    }
}

/// One nested relation body input: the body's own sealed identity plus the
/// checked algebra root lowered from that nested relation.
///
/// Both fields are private and reachable only through [`Self::try_new`], so a
/// body input can never be struct-literal-built; the exact relation the root was
/// lowered from is re-checked against the sealed composition body by
/// [`CheckedCompositionPlanInputV2::new`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionRelationalBodyPlanInputV2 {
    item_id: SemanticItemIdV2,
    root: CheckedAlgebraNodeV2,
}

impl CompositionRelationalBodyPlanInputV2 {
    /// Retain one nested relation body input.
    ///
    /// `root` is the checked algebra root lowered from this body's nested
    /// relation with the composition item as its owning item (algebra's
    /// [`quanta_gqlang_v2_algebra::lower_nested_relation_v2`]).
    ///
    /// # Errors
    ///
    /// Returns [`CompositionInvalidReasonV2::BodyIdentityMismatch`] when the body
    /// identity and the checked algebra root belong to different sources.
    pub fn try_new(
        item_id: SemanticItemIdV2,
        root: CheckedAlgebraNodeV2,
    ) -> Result<Self, CompositionInvalidReasonV2> {
        if item_id.source_id() != root.location().location().source_id {
            return Err(CompositionInvalidReasonV2::BodyIdentityMismatch);
        }
        Ok(Self { item_id, root })
    }

    /// The body's own sealed semantic identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The checked algebra root lowered from that nested body relation.
    #[must_use]
    pub const fn root(&self) -> &CheckedAlgebraNodeV2 {
        &self.root
    }
}

/// One sealed composition body input.
///
/// The four variants mirror Sema's closed [`ResolvedCompositionBodyV2`] one to
/// one: a composition body is a Query, FIND, CHECK, or EXPLAIN and can never be
/// another composition. The relational arm carries the body's own sealed
/// identity plus the checked algebra root lowered from that nested relation
/// (algebra's nested entry point), because a nested body relation is not a
/// top-level document item. Every variant payload is constructor-only, so a body
/// input always names a sealed upstream carrier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompositionBodyPlanInputV2 {
    /// A SELECT/query body lowered from the nested relation.
    Relational(CompositionRelationalBodyPlanInputV2),
    /// A FIND body sealed by Logic.
    Graph(GraphQueryContractV2),
    /// A CHECK body validated by this crate.
    Check(CheckContractV2),
    /// An EXPLAIN body validated by this crate.
    Explain(PresentationContractV2),
}

impl CompositionBodyPlanInputV2 {
    /// The body input's semantic identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Relational(body) => body.item_id(),
            Self::Graph(contract) => contract.item_id(),
            Self::Check(contract) => contract.item_id(),
            Self::Explain(contract) => contract.item_id(),
        }
    }

    /// The closed body family this input is.
    #[must_use]
    pub const fn family(&self) -> CompositionBodyFamilyV2 {
        match self {
            Self::Relational(_) => CompositionBodyFamilyV2::Query,
            Self::Graph(_) => CompositionBodyFamilyV2::Graph,
            Self::Check(_) => CompositionBodyFamilyV2::Check,
            Self::Explain(_) => CompositionBodyFamilyV2::Explain,
        }
    }

    /// Stable kind keyword of the body input, for diagnostics.
    #[must_use]
    pub const fn keyword(&self) -> &'static str {
        self.family().keyword()
    }

    /// The body's checked location.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        match self {
            Self::Relational(body) => body.root().location(),
            Self::Graph(contract) => contract.location(),
            Self::Check(contract) => contract.location(),
            Self::Explain(contract) => contract.location(),
        }
    }
}

/// Whether a nested relation body input is the canonical lowering of the sealed
/// body relation.
///
/// The lowered root must be minted from THIS resolved composition instance and
/// its exact original body root, and carry the sealed output contract. The
/// lowered stage KEY is deliberately NOT compared: canonical lowering may
/// re-anchor a pure relational `EXISTS` conjunct onto the EXISTS stage key
/// (`algebra::pure_exists_conjunct_lowers_to_semijoin_with_no_residual_filter`),
/// so comparing keys would make every pure-`EXISTS` composition body
/// unplannable.
fn nested_root_matches(
    composition: &ResolvedCompositionV2,
    sealed: &TypedRelationV2,
    body: &CompositionRelationalBodyPlanInputV2,
) -> bool {
    body.root().item_id() == composition.item_id()
        && body.root().output() == sealed.output()
        && body
            .root()
            .matches_composition_origin(composition.instance(), sealed.root().key())
}

/// One retained composition producer (a `LET` working set).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCompositionProducerV2 {
    ordinal: usize,
    item_id: SemanticItemIdV2,
    name_location: CheckedLocatedSpanV2,
    relation_output: Option<RelationOutputContractV2>,
}

impl CheckedCompositionProducerV2 {
    /// Retain one producer at its authored `LET` ordinal.
    ///
    /// # Errors
    ///
    /// Returns [`CompositionInvalidReasonV2::ProducerIdentityMismatch`] when the
    /// producer identity and its checked name location belong to different
    /// sources.
    pub(crate) fn try_new(
        ordinal: usize,
        item_id: SemanticItemIdV2,
        name_location: CheckedLocatedSpanV2,
        relation_output: Option<RelationOutputContractV2>,
    ) -> Result<Self, CompositionInvalidReasonV2> {
        if item_id.source_id() != name_location.location().source_id {
            return Err(CompositionInvalidReasonV2::ProducerIdentityMismatch);
        }
        Ok(Self {
            ordinal,
            item_id,
            name_location,
            relation_output,
        })
    }

    /// Read one producer from its sealed binding, at the ordinal that binding
    /// itself names.
    fn from_binding(
        binding: &ResolvedCompositionBindingV2,
    ) -> Result<Self, CompositionInvalidReasonV2> {
        Self::try_new(
            binding.binding().ordinal(),
            binding.producer().item_id(),
            binding.name_location().clone(),
            binding.relation_output().cloned(),
        )
    }

    /// The authored `LET` ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// The nested producer's semantic identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The checked location of the working-set name.
    #[must_use]
    pub const fn name_location(&self) -> &CheckedLocatedSpanV2 {
        &self.name_location
    }

    /// The relation output this producer exposes, when it exposes one.
    #[must_use]
    pub const fn relation_output(&self) -> Option<&RelationOutputContractV2> {
        self.relation_output.as_ref()
    }
}

/// The exact sealed identity of one composition body: its closed family and its
/// own item identity.
///
/// The two travel together because a body policy is only meaningful for the
/// exact body it was classified from; keeping them apart is what lets a policy
/// describe a different body than the one the retention names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompositionBodyIdentityV2 {
    family: CompositionBodyFamilyV2,
    item_id: SemanticItemIdV2,
}

impl CompositionBodyIdentityV2 {
    /// Retain one body identity.
    #[must_use]
    pub(crate) const fn new(family: CompositionBodyFamilyV2, item_id: SemanticItemIdV2) -> Self {
        Self { family, item_id }
    }

    /// The body identity of one sealed composition body.
    const fn of_sealed(body: &ResolvedCompositionBodyV2) -> Self {
        let family = match body {
            ResolvedCompositionBodyV2::Query(_) => CompositionBodyFamilyV2::Query,
            ResolvedCompositionBodyV2::Graph(_) => CompositionBodyFamilyV2::Graph,
            ResolvedCompositionBodyV2::Check(_) => CompositionBodyFamilyV2::Check,
            ResolvedCompositionBodyV2::Explain(_) => CompositionBodyFamilyV2::Explain,
        };
        Self::new(family, body.item_id())
    }

    /// The sealed body family.
    #[must_use]
    pub(crate) const fn family(self) -> CompositionBodyFamilyV2 {
        self.family
    }

    /// The sealed body's own item identity.
    #[must_use]
    pub(crate) const fn item_id(self) -> SemanticItemIdV2 {
        self.item_id
    }
}

/// The exact composition identity a plan policy retains: the root identity and
/// its checked location, the ordered producers, the exact `WITH` binding, and
/// the sealed body identity.
///
/// This is the ONE place those facts are checked, so the sealed-composition
/// reader ([`CheckedCompositionPlanInputV2::new`]) and the policy carrier read
/// the same rail instead of restating it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompositionRetentionV2 {
    root_item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    producers: Vec<CheckedCompositionProducerV2>,
    with_binding: ResolvedValueBindingV2,
    body: CompositionBodyIdentityV2,
}

impl CompositionRetentionV2 {
    /// Check one set of retained composition facts.
    ///
    /// The fixed rejection order is: producer presence, ordinal, duplicate
    /// identity, producer source membership, `WITH` target, body identity.
    ///
    /// # Errors
    ///
    /// Returns the first [`CompositionInvalidReasonV2`] defect.
    pub(crate) fn try_new(
        root_item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        producers: Vec<CheckedCompositionProducerV2>,
        with_binding: ResolvedValueBindingV2,
        body: CompositionBodyIdentityV2,
    ) -> Result<Self, CompositionInvalidReasonV2> {
        if producers.is_empty() {
            return Err(CompositionInvalidReasonV2::EmptyProducers);
        }
        let mut seen = BTreeSet::new();
        for (index, producer) in producers.iter().enumerate() {
            if producer.ordinal() != index {
                return Err(CompositionInvalidReasonV2::ProducerOrdinalMismatch);
            }
            if !seen.insert(producer.item_id()) {
                return Err(CompositionInvalidReasonV2::DuplicateProducer);
            }
            if producer.item_id().source_id() != root_item_id.source_id()
                || producer.name_location().location().source_id != root_item_id.source_id()
            {
                return Err(CompositionInvalidReasonV2::ProducerIdentityMismatch);
            }
        }

        // The WITH binding belongs to THIS composition root and names exactly one
        // of the ordered producers.
        let names_exactly_one_producer = producers
            .iter()
            .filter(|producer| producer.ordinal() == with_binding.ordinal())
            .count()
            == 1;
        if with_binding.source_id() != root_item_id.source_id()
            || with_binding.target_node() != root_item_id.hir_item_id()
            || !names_exactly_one_producer
        {
            return Err(CompositionInvalidReasonV2::WithTargetNotAProducer);
        }

        if body.item_id().source_id() != root_item_id.source_id() {
            return Err(CompositionInvalidReasonV2::BodyIdentityMismatch);
        }

        Ok(Self {
            root_item_id,
            location,
            producers,
            with_binding,
            body,
        })
    }

    /// Semantic item identity of the composition root.
    const fn root_item_id(&self) -> SemanticItemIdV2 {
        self.root_item_id
    }

    /// Checked location of the composition item.
    const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }

    /// The ordered producers, in authored `LET` order.
    fn producers(&self) -> &[CheckedCompositionProducerV2] {
        &self.producers
    }

    /// The exact `WITH` binding identity.
    const fn with_binding(&self) -> &ResolvedValueBindingV2 {
        &self.with_binding
    }

    /// The sealed body identity.
    const fn body(&self) -> CompositionBodyIdentityV2 {
        self.body
    }
}

/// The body policy a composition plan retains: the body's OWN classified plan
/// policy, narrowed to the four families a composition body can be.
///
/// The narrowing is checked, so a rule-program, meta, or nested-composition
/// policy can never be presented as a composition body policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionBodyPolicyV2 {
    family: CompositionBodyFamilyV2,
    /// Boxed because a [`PlanPolicyV2`] can itself retain a composition policy;
    /// the indirection keeps the two carriers from being mutually recursive.
    policy: Box<PlanPolicyV2>,
}

impl CompositionBodyPolicyV2 {
    /// Narrow one classified carrier policy to the composition-body families.
    ///
    /// # Errors
    ///
    /// Returns [`CompositionInvalidReasonV2::BodyKindMismatch`] when `policy` is
    /// not one of the four families a sealed composition body can be.
    pub(crate) fn try_new(policy: PlanPolicyV2) -> Result<Self, CompositionInvalidReasonV2> {
        let family = CompositionBodyFamilyV2::of_policy(&policy)
            .ok_or(CompositionInvalidReasonV2::BodyKindMismatch)?;
        Ok(Self {
            family,
            policy: Box::new(policy),
        })
    }

    /// The closed family of the body this policy was classified from.
    #[must_use]
    pub const fn family(&self) -> CompositionBodyFamilyV2 {
        self.family
    }

    /// The body's own classified policy, exactly as the ONE classifier derived
    /// it from the sealed body carrier.
    #[must_use]
    pub const fn policy(&self) -> &PlanPolicyV2 {
        &self.policy
    }

    /// Stable family keyword, shared with the body family vocabulary.
    #[must_use]
    pub const fn keyword(&self) -> &'static str {
        self.family.keyword()
    }
}

/// The exact composition policy a plan retains.
///
/// It is only obtainable from a checked composition input
/// ([`CheckedCompositionPlanInputV2::policy`]), so its root, producers, `WITH`
/// binding, and body identity are always the checked composition's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionPolicyV2 {
    retention: CompositionRetentionV2,
    body: CompositionBodyPolicyV2,
}

impl CompositionPolicyV2 {
    /// Check and retain one composition policy: the exact checked retention plus
    /// the body policy of the SAME body family that retention names.
    ///
    /// # Errors
    ///
    /// Returns [`CompositionInvalidReasonV2::BodyKindMismatch`] when the body
    /// policy's family is not the retained body identity's family.
    pub(crate) fn try_new(
        retention: CompositionRetentionV2,
        body: CompositionBodyPolicyV2,
    ) -> Result<Self, CompositionInvalidReasonV2> {
        if body.family() != retention.body().family() {
            return Err(CompositionInvalidReasonV2::BodyKindMismatch);
        }
        Ok(Self { retention, body })
    }

    /// Semantic item identity of the composition root this policy belongs to.
    #[must_use]
    pub const fn root_item_id(&self) -> SemanticItemIdV2 {
        self.retention.root_item_id()
    }

    /// Checked location of the composition item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        self.retention.location()
    }

    /// The ordered producers, in authored `LET` order.
    #[must_use]
    pub fn producers(&self) -> &[CheckedCompositionProducerV2] {
        self.retention.producers()
    }

    /// The exact `WITH` binding identity.
    #[must_use]
    pub const fn with_binding(&self) -> &ResolvedValueBindingV2 {
        self.retention.with_binding()
    }

    /// The sealed body identity this composition plans.
    #[must_use]
    pub const fn body_item_id(&self) -> SemanticItemIdV2 {
        self.retention.body().item_id()
    }

    /// The closed family of the sealed body this composition plans.
    #[must_use]
    pub const fn body_family(&self) -> CompositionBodyFamilyV2 {
        self.retention.body().family()
    }

    /// The body's retained policy.
    #[must_use]
    pub const fn body(&self) -> &CompositionBodyPolicyV2 {
        &self.body
    }
}

/// A checked composition planning input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCompositionPlanInputV2 {
    composition: ResolvedCompositionV2,
    retention: CompositionRetentionV2,
    body: CompositionBodyPlanInputV2,
}

impl CheckedCompositionPlanInputV2 {
    /// Check the sealed composition against the one caller-supplied body input.
    ///
    /// # Errors
    ///
    /// Returns [`PlanningDiagnosticV1::InvalidComposition`] with the first defect
    /// in the fixed order: producer membership/ordinals, `WITH` target, body
    /// kind, body identity.
    pub fn new(
        composition: &ResolvedCompositionV2,
        body: CompositionBodyPlanInputV2,
    ) -> Result<Self, PlanningDiagnosticV1> {
        let location = composition.location().clone();
        let reject =
            |reason: CompositionInvalidReasonV2| PlanningDiagnosticV1::InvalidComposition {
                reason,
                span: location.clone(),
            };

        // Ordered producers, read from the sealed composition; every retained
        // fact is then re-checked by the ONE retention rail.
        let mut producers = Vec::with_capacity(composition.bindings().len());
        for binding in composition.bindings() {
            producers.push(CheckedCompositionProducerV2::from_binding(binding).map_err(reject)?);
        }
        let body_identity = CompositionBodyIdentityV2::of_sealed(composition.body());
        let retention = CompositionRetentionV2::try_new(
            composition.item_id(),
            location.clone(),
            producers,
            composition.with_target().clone(),
            body_identity,
        )
        .map_err(reject)?;

        // The body input must be the sealed body's exact family and identity.
        if body.family() != body_identity.family() {
            return Err(reject(CompositionInvalidReasonV2::BodyKindMismatch));
        }
        if body.item_id() != body_identity.item_id() {
            return Err(reject(CompositionInvalidReasonV2::BodyIdentityMismatch));
        }
        // A relational body carries a lowered algebra root whose fields are
        // caller-supplied: the root must be the canonical lowering of the SEALED
        // body relation, not an arbitrary subtree carrying the sealed identity.
        if let (
            ResolvedCompositionBodyV2::Query(sealed),
            CompositionBodyPlanInputV2::Relational(input),
        ) = (composition.body(), &body)
        {
            if !nested_root_matches(composition, sealed, input) {
                return Err(reject(CompositionInvalidReasonV2::BodyIdentityMismatch));
            }
        }

        Ok(Self {
            composition: composition.clone(),
            retention,
            body,
        })
    }

    /// Semantic item identity of the composition root.
    #[must_use]
    pub const fn root_item_id(&self) -> SemanticItemIdV2 {
        self.retention.root_item_id()
    }

    /// Checked location of the composition item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        self.retention.location()
    }

    /// The composition's own checked `WITH` use location.
    #[must_use]
    pub const fn with_location(&self) -> &CheckedLocatedSpanV2 {
        self.composition.with_location()
    }

    /// The ordered producers, in authored `LET` order.
    #[must_use]
    pub fn producers(&self) -> &[CheckedCompositionProducerV2] {
        self.retention.producers()
    }

    /// The exact `WITH` binding identity.
    #[must_use]
    pub const fn with_binding(&self) -> &ResolvedValueBindingV2 {
        self.retention.with_binding()
    }

    /// The sealed body identity this composition plans.
    #[must_use]
    pub const fn body_item_id(&self) -> SemanticItemIdV2 {
        self.retention.body().item_id()
    }

    /// The closed family of the sealed body this composition plans.
    #[must_use]
    pub const fn body_family(&self) -> CompositionBodyFamilyV2 {
        self.retention.body().family()
    }

    /// The one sealed body input.
    #[must_use]
    pub const fn body(&self) -> &CompositionBodyPlanInputV2 {
        &self.body
    }

    /// The composition policy this input's plan retains: this input's checked
    /// root, ordered producers, `WITH` binding and body identity, plus the body
    /// policy the ONE classifier derived from THIS input's own body.
    ///
    /// # Errors
    ///
    /// Returns [`PlanningDiagnosticV1::InvalidComposition`] with
    /// [`CompositionInvalidReasonV2::BodyKindMismatch`] when `body` is not a
    /// composition-body family, or names a different body family than this
    /// input's own checked body. The anchor is the composition item's own checked
    /// location, never a fabricated one.
    pub fn policy(&self, body: PlanPolicyV2) -> Result<CompositionPolicyV2, PlanningDiagnosticV1> {
        CompositionBodyPolicyV2::try_new(body)
            .and_then(|body| CompositionPolicyV2::try_new(self.retention.clone(), body))
            .map_err(|reason| PlanningDiagnosticV1::InvalidComposition {
                reason,
                span: self.retention.location().clone(),
            })
    }

    /// The sealed composition this input was checked against, for exact
    /// document-membership comparison at plan time.
    #[must_use]
    pub const fn sealed_composition(&self) -> &ResolvedCompositionV2 {
        &self.composition
    }
}

#[cfg(test)]
pub(crate) mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "these rails assert exact retained facts and return a typed fixture error, so every assertion is a deliberate crash on drift"
    )]

    use crate::policy::{EffectSummaryV2, GraphWitnessRetentionV2};
    use quanta_gqlang_v2_algebra::{lower_nested_relation_v2, lower_relation_item};
    use quanta_gqlang_v2_logic::{GraphQueryContractV2, RuleProgramContractV2};
    use quanta_gqlang_v2_sema::test_support::{
        pristine_capture_edge_set_v2, reference_absent_item_id_v2, reference_checked_span_v2,
        reference_composition_document_v2, reference_graph_document_v2,
        reference_nested_check_document_v2, reference_relation_document_v2,
        reference_rule_document_v2, ReferenceCompositionBodyV2, ReferenceGraphDocumentV2,
        ReferenceGraphReferenceV2, ReferenceGraphRequestV2, ReferenceNestedCheckDocumentV2,
        ReferenceRelationFixtureV2, ReferenceRuleFixtureV2, REFERENCE_SOURCE_V2,
        SECONDARY_REFERENCE_SOURCE_V2,
    };
    use quanta_gqlang_v2_sema::{
        GraphQueryKindV1, GraphReferenceRoleV2, LocatedSpanV2, ResolvedBindingKindV2,
        ResolvedPresentationKindV2, ResolvedValueBindingV2, SourceRegistrationV2,
        SourceRegistryViewV2, SpanV1,
    };
    use quanta_gqlang_v2_types::{AggregationClassV1, DeterminismV1, MonotonicityV1, PurityV1};

    use super::*;

    /// The exact `WITH`-shaped binding the capture-edge fixture mints, plus the
    /// composition root identity that binding names as its owner.
    fn binding_and_root() -> Result<(ResolvedValueBindingV2, SemanticItemIdV2), String> {
        let edges = pristine_capture_edge_set_v2()?;
        let binding = edges
            .first()
            .map(|edge| edge.binding().clone())
            .ok_or_else(|| "the capture-edge fixture carries two edges".to_owned())?;
        let root = SemanticItemIdV2::new(binding.source_id(), binding.target_node());
        Ok((binding, root))
    }

    /// One distinct producer identity in the root's own source.
    fn producer_id(index: usize) -> Result<SemanticItemIdV2, String> {
        pristine_capture_edge_set_v2()?
            .get(index)
            .map(|edge| SemanticItemIdV2::new(REFERENCE_SOURCE_V2, edge.reference().node_id))
            .ok_or_else(|| format!("the capture-edge fixture carries no reference {index}"))
    }

    /// One producer in the root's own source at the requested ordinal.
    fn producer(
        ordinal: usize,
        index: usize,
        relation_output: Option<RelationOutputContractV2>,
    ) -> Result<CheckedCompositionProducerV2, String> {
        CheckedCompositionProducerV2::try_new(
            ordinal,
            producer_id(index)?,
            reference_checked_span_v2(0, 1)?,
            relation_output,
        )
        .map_err(|reason| format!("{reason:?}"))
    }

    /// One checked location in the SECONDARY fixture source, for
    /// transplanted-producer rails.
    fn secondary_span(start: usize, end: usize) -> Result<CheckedLocatedSpanV2, String> {
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: SECONDARY_REFERENCE_SOURCE_V2,
            source_len: 64,
        }])
        .map_err(|error| error.to_string())?;
        registry
            .try_check_location(LocatedSpanV2 {
                source_id: SECONDARY_REFERENCE_SOURCE_V2,
                span: SpanV1::new(start, end),
            })
            .map_err(|error| format!("{error:?}"))
    }

    /// The order-preserving producer vector of the fixture.
    fn producers() -> Result<Vec<CheckedCompositionProducerV2>, String> {
        Ok(vec![producer(0, 0, None)?, producer(1, 1, None)?])
    }

    /// The exact checked retention of the fixture: two ordered producers, the
    /// fixture `WITH` binding naming producer 0, and a query body identity.
    fn retention() -> Result<CompositionRetentionV2, String> {
        let (binding, root) = binding_and_root()?;
        CompositionRetentionV2::try_new(
            root,
            reference_checked_span_v2(0, 4)?,
            producers()?,
            binding,
            CompositionBodyIdentityV2::new(
                CompositionBodyFamilyV2::Query,
                reference_absent_item_id_v2(),
            ),
        )
        .map_err(|reason| format!("{reason:?}"))
    }

    /// One folded query body policy with distinct, non-default coefficients.
    fn query_body_policy() -> PlanPolicyV2 {
        PlanPolicyV2::Relational {
            nodes: 3,
            effects: EffectSummaryV2::new(
                PurityV1::Pure,
                DeterminismV1::NonDeterministic,
                MonotonicityV1::Monotone,
                AggregationClassV1::Aggregate,
            ),
        }
    }

    /// The exact composition policy of the fixture matrix: the checked retention
    /// above plus the fixture query body policy. It is `pub(crate)` for the
    /// plan-assembly rail in [`crate::plan`], which proves the policy reaches
    /// [`crate::SemanticPlanV2::policy`] unchanged.
    // nosemgrep: qglang-v2-no-string-typed-error-results-in-core -- cfg(test)-only fixture
    pub(crate) fn fixture_policy() -> Result<CompositionPolicyV2, String> {
        let body = CompositionBodyPolicyV2::try_new(query_body_policy())
            .map_err(|reason| format!("{reason:?}"))?;
        CompositionPolicyV2::try_new(retention()?, body).map_err(|reason| format!("{reason:?}"))
    }

    /// One sealed FIND document, for the graph body-policy and witness rails.
    fn graph_document() -> Result<ReferenceGraphDocumentV2, String> {
        reference_graph_document_v2(ReferenceGraphRequestV2 {
            kind: GraphQueryKindV1::Taint,
            references: vec![
                ReferenceGraphReferenceV2 {
                    role: GraphReferenceRoleV2::Source,
                    selector: "input_a",
                    span: (20, 27),
                },
                ReferenceGraphReferenceV2 {
                    role: GraphReferenceRoleV2::Sink,
                    selector: "exec_b",
                    span: (30, 36),
                },
            ],
            depth: Some(4),
            direction: None,
            summary_facets: Vec::new(),
            item_span: (0, 8),
        })
    }

    /// The one shared nested CHECK fixture document.
    fn check_document() -> Result<ReferenceNestedCheckDocumentV2, String> {
        reference_nested_check_document_v2(ReferenceGraphRequestV2 {
            kind: GraphQueryKindV1::Taint,
            references: vec![
                ReferenceGraphReferenceV2 {
                    role: GraphReferenceRoleV2::Source,
                    selector: "input_a",
                    span: (20, 27),
                },
                ReferenceGraphReferenceV2 {
                    role: GraphReferenceRoleV2::Sink,
                    selector: "exec_b",
                    span: (30, 36),
                },
            ],
            depth: Some(4),
            direction: None,
            summary_facets: Vec::new(),
            item_span: (0, 8),
        })
    }

    /// A relation output retained by one producer, read from a lowered item.
    fn relation_output() -> Result<RelationOutputContractV2, String> {
        let document = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)?;
        let item = lower_relation_item(document.document(), document.item_id())
            .map_err(|error| format!("{error:?}"))?;
        Ok(item.output().clone())
    }

    /// Every body family a composition body can be is narrowed with its exact
    /// payload, and the three non-body families are rejected.
    #[test]
    fn only_the_four_body_families_narrow_into_a_body_policy() -> Result<(), String> {
        let relational = query_body_policy();
        let narrowed = CompositionBodyPolicyV2::try_new(relational.clone())
            .map_err(|reason| format!("{reason:?}"))?;
        assert_eq!(narrowed.family(), CompositionBodyFamilyV2::Query);
        assert_eq!(narrowed.keyword(), "query");

        let graph = GraphQueryContractV2::from_resolved(graph_document()?.graph())
            .map_err(|error| format!("{error:?}"))?;
        let graph_policy = PlanPolicyV2::Graph(GraphWitnessRetentionV2::from_contract(&graph));
        let narrowed = CompositionBodyPolicyV2::try_new(graph_policy.clone())
            .map_err(|reason| format!("{reason:?}"))?;
        assert_eq!(narrowed.family(), CompositionBodyFamilyV2::Graph);
        assert_eq!(narrowed.policy(), &graph_policy);

        let check_fixture = check_document()?;
        let resolved = match check_fixture.document().item(check_fixture.check_item_id()) {
            Some(quanta_gqlang_v2_sema::SemanticItemV2::Check(check)) => check,
            other => {
                return Err(format!(
                    "the nested-check fixture is a CHECK, got {other:?}"
                ))
            }
        };
        let nested = resolved
            .nested_graph()
            .map(GraphQueryContractV2::from_resolved)
            .transpose()
            .map_err(|error| format!("{error:?}"))?;
        let contract = CheckContractV2::from_resolved(resolved, nested)
            .map_err(|error| format!("{error:?}"))?;
        let check_policy = PlanPolicyV2::Check(contract.policy().clone());
        let narrowed = CompositionBodyPolicyV2::try_new(check_policy.clone())
            .map_err(|reason| format!("{reason:?}"))?;
        assert_eq!(narrowed.family(), CompositionBodyFamilyV2::Check);
        assert_eq!(narrowed.policy(), &check_policy);

        let presentation = PlanPolicyV2::Presentation(ResolvedPresentationKindV2::Show {
            category: quanta_gqlang_v2_sema::ShowCategoryV1::Capabilities,
        });
        let narrowed = CompositionBodyPolicyV2::try_new(presentation.clone())
            .map_err(|reason| format!("{reason:?}"))?;
        assert_eq!(narrowed.family(), CompositionBodyFamilyV2::Explain);
        assert_eq!(narrowed.keyword(), "explain");
        assert_eq!(narrowed.policy(), &presentation);

        // The document rule program, the meta directive, and a nested composition
        // are not composition bodies.
        let rule_document =
            reference_rule_document_v2(ReferenceRuleFixtureV2::PositiveSelfRecursion)?;
        let program = RuleProgramContractV2::from_document(rule_document.document())
            .map_err(|error| format!("{error:?}"))?;
        for non_body in [
            PlanPolicyV2::Rule(crate::policy::RuleProgramPolicyV2::new(
                &program,
                1,
                Vec::new(),
            )),
            PlanPolicyV2::MetaRequireCertified,
            PlanPolicyV2::Composition(
                CompositionPolicyV2::try_new(
                    retention()?,
                    CompositionBodyPolicyV2::try_new(relational.clone())
                        .map_err(|reason| format!("{reason:?}"))?,
                )
                .map_err(|reason| format!("{reason:?}"))?,
            ),
        ] {
            assert_eq!(
                CompositionBodyPolicyV2::try_new(non_body),
                Err(CompositionInvalidReasonV2::BodyKindMismatch)
            );
        }
        Ok(())
    }

    /// The exact production order, `WITH` binding, body identity, and body
    /// policy survive the one checked retention into a composition policy.
    #[test]
    fn a_checked_retention_lands_every_composition_fact_exactly() -> Result<(), String> {
        let (binding, root) = binding_and_root()?;
        let output = relation_output()?;
        let ordered = vec![producer(0, 0, Some(output.clone()))?, producer(1, 1, None)?];
        let body = CompositionBodyIdentityV2::new(
            CompositionBodyFamilyV2::Query,
            reference_absent_item_id_v2(),
        );
        let retention = CompositionRetentionV2::try_new(
            root,
            reference_checked_span_v2(0, 4)?,
            ordered.clone(),
            binding.clone(),
            body,
        )
        .map_err(|reason| format!("{reason:?}"))?;
        let body_policy = CompositionBodyPolicyV2::try_new(query_body_policy())
            .map_err(|reason| format!("{reason:?}"))?;
        let policy = CompositionPolicyV2::try_new(retention, body_policy.clone())
            .map_err(|reason| format!("{reason:?}"))?;

        assert_eq!(policy.root_item_id(), root);
        assert_eq!(policy.location(), &reference_checked_span_v2(0, 4)?);
        // Producer order, ordinals, identities, name locations, and relation
        // outputs all survive exactly, in authored `LET` order.
        assert_eq!(policy.producers(), ordered.as_slice());
        let first = policy
            .producers()
            .first()
            .ok_or_else(|| "the retention keeps both producers".to_owned())?;
        let second = policy
            .producers()
            .get(1)
            .ok_or_else(|| "the retention keeps both producers".to_owned())?;
        assert_eq!(first.ordinal(), 0);
        assert_eq!(first.name_location(), &reference_checked_span_v2(0, 1)?);
        assert_eq!(first.relation_output(), Some(&output));
        assert_eq!(second.ordinal(), 1);
        assert_eq!(second.relation_output(), None);
        assert_eq!(policy.with_binding(), &binding);
        assert_eq!(policy.body_item_id(), reference_absent_item_id_v2());
        assert_eq!(policy.body_family(), CompositionBodyFamilyV2::Query);
        assert_eq!(policy.body(), &body_policy);
        assert_eq!(policy.body().policy(), &query_body_policy());
        Ok(())
    }

    /// A producer ordinal that is not its authored position is rejected, as are a
    /// repeated producer identity, a transplanted producer, and a missing
    /// producer.
    #[test]
    fn producer_defects_are_rejected_by_the_one_retention_rail() -> Result<(), String> {
        let (binding, root) = binding_and_root()?;
        let location = reference_checked_span_v2(0, 4)?;
        let name_location = reference_checked_span_v2(0, 1)?;
        let body = CompositionBodyIdentityV2::new(
            CompositionBodyFamilyV2::Query,
            reference_absent_item_id_v2(),
        );
        let retain = |producers: Vec<CheckedCompositionProducerV2>,
                      location: CheckedLocatedSpanV2| {
            CompositionRetentionV2::try_new(root, location, producers, binding.clone(), body)
        };

        assert_eq!(
            retain(Vec::new(), location.clone()),
            Err(CompositionInvalidReasonV2::EmptyProducers)
        );
        assert_eq!(
            retain(
                vec![producer(1, 0, None)?, producer(0, 1, None)?],
                location.clone()
            ),
            Err(CompositionInvalidReasonV2::ProducerOrdinalMismatch)
        );
        assert_eq!(
            retain(
                vec![producer(0, 0, None)?, producer(1, 0, None)?],
                location.clone()
            ),
            Err(CompositionInvalidReasonV2::DuplicateProducer)
        );

        // The same identity is admissible on its own, and the order-preserving
        // two-producer vector is the accepted baseline every defect above
        // deviates from.
        let one = CheckedCompositionProducerV2::try_new(0, producer_id(0)?, name_location, None)
            .map_err(|reason| format!("{reason:?}"))?;
        retain(vec![one], location.clone()).map_err(|reason| format!("{reason:?}"))?;
        retain(producers()?, location).map_err(|reason| format!("{reason:?}"))?;

        // A producer identity from another source cannot be transplanted into
        // this composition.
        let foreign = CheckedCompositionProducerV2::try_new(
            0,
            SemanticItemIdV2::new(SECONDARY_REFERENCE_SOURCE_V2, binding.target_node()),
            secondary_span(0, 1)?,
            None,
        )
        .map_err(|reason| format!("{reason:?}"))?;
        assert_eq!(
            CompositionRetentionV2::try_new(
                root,
                reference_checked_span_v2(0, 4)?,
                vec![foreign],
                binding.clone(),
                body
            ),
            Err(CompositionInvalidReasonV2::ProducerIdentityMismatch)
        );
        Ok(())
    }

    /// A `WITH` binding that names no ordered producer, or belongs to another
    /// source or owner, is rejected.
    #[test]
    fn a_with_binding_must_name_exactly_one_producer() -> Result<(), String> {
        let (binding, root) = binding_and_root()?;
        let body = CompositionBodyIdentityV2::new(
            CompositionBodyFamilyV2::Query,
            reference_absent_item_id_v2(),
        );
        let unnamed = ResolvedValueBindingV2::try_new(
            binding.source_id(),
            binding.target_node(),
            7,
            ResolvedBindingKindV2::Composition,
            "w",
        )
        .map_err(str::to_owned)?;
        assert_eq!(
            CompositionRetentionV2::try_new(
                root,
                reference_checked_span_v2(0, 4)?,
                producers()?,
                unnamed,
                body
            ),
            Err(CompositionInvalidReasonV2::WithTargetNotAProducer)
        );

        let foreign = ResolvedValueBindingV2::try_new(
            SECONDARY_REFERENCE_SOURCE_V2,
            binding.target_node(),
            0,
            ResolvedBindingKindV2::Composition,
            "w",
        )
        .map_err(str::to_owned)?;
        assert_eq!(
            CompositionRetentionV2::try_new(
                root,
                reference_checked_span_v2(0, 4)?,
                producers()?,
                foreign,
                body
            ),
            Err(CompositionInvalidReasonV2::WithTargetNotAProducer)
        );
        Ok(())
    }

    /// A body identity from another source is rejected, and a body policy of a
    /// different family can never be attached to it.
    #[test]
    fn a_swapped_body_identity_or_family_is_rejected() -> Result<(), String> {
        let (binding, root) = binding_and_root()?;
        let foreign_body = CompositionBodyIdentityV2::new(
            CompositionBodyFamilyV2::Query,
            SemanticItemIdV2::new(SECONDARY_REFERENCE_SOURCE_V2, binding.target_node()),
        );
        assert_eq!(
            CompositionRetentionV2::try_new(
                root,
                reference_checked_span_v2(0, 4)?,
                producers()?,
                binding,
                foreign_body
            ),
            Err(CompositionInvalidReasonV2::BodyIdentityMismatch)
        );

        let graph = GraphQueryContractV2::from_resolved(graph_document()?.graph())
            .map_err(|error| format!("{error:?}"))?;
        let graph_body = CompositionBodyPolicyV2::try_new(PlanPolicyV2::Graph(
            GraphWitnessRetentionV2::from_contract(&graph),
        ))
        .map_err(|reason| format!("{reason:?}"))?;
        assert_eq!(
            CompositionPolicyV2::try_new(retention()?, graph_body),
            Err(CompositionInvalidReasonV2::BodyKindMismatch)
        );
        Ok(())
    }

    /// A relational body root must be the canonical lowering of the SEALED body
    /// relation: another relation's lowering, or a root lowered for another
    /// owning item, is not this body.
    #[test]
    fn a_nested_body_root_must_be_the_sealed_body_lowering() -> Result<(), String> {
        let document = reference_composition_document_v2(ReferenceCompositionBodyV2::Relational)?;
        let composition = document.composition();
        let ResolvedCompositionBodyV2::Query(sealed) = composition.body() else {
            return Err("the composition fixture has no query body".to_owned());
        };
        let lowered =
            lower_nested_relation_v2(document.document(), composition.item_id(), sealed.root())
                .map_err(|error| format!("{error:?}"))?;
        let body = CompositionRelationalBodyPlanInputV2::try_new(sealed.item_id(), lowered.clone())
            .map_err(|reason| format!("{reason:?}"))?;
        assert!(nested_root_matches(composition, sealed, &body));

        // A DIFFERENT relation's lowering is not this body: the exact sealed
        // output contract differs.
        let other = reference_relation_document_v2(ReferenceRelationFixtureV2::SortAsc)?;
        let other_lowered = lower_relation_item(other.document(), other.item_id())
            .map_err(|error| format!("{error:?}"))?;
        let forged = CompositionRelationalBodyPlanInputV2::try_new(
            sealed.item_id(),
            other_lowered.root().clone(),
        )
        .map_err(|reason| format!("{reason:?}"))?;
        assert!(!nested_root_matches(composition, sealed, &forged));

        // The body identity and its root must belong to one source.
        let (binding, _root) = binding_and_root()?;
        assert_eq!(
            CompositionRelationalBodyPlanInputV2::try_new(
                SemanticItemIdV2::new(SECONDARY_REFERENCE_SOURCE_V2, binding.target_node()),
                lowered
            ),
            Err(CompositionInvalidReasonV2::BodyIdentityMismatch)
        );
        Ok(())
    }
}
