//! Typed semantic contracts for non-relational document item families.

use crate::{
    GraphReferenceIdV2, GraphReferenceKindV2, OutputColumnOriginV2, RelationOutputContractV2,
    ResolvedBindingKindV2, ResolvedCatalogReferenceV2, ResolvedCompositionSourceIdV2,
    ResolvedRuleV2, ResolvedValueBindingV2, SemanticItemIdV2, TypedExprV2, TypedRelationV2,
};
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, CompareMetricV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExportFormatV1, GraphQueryKindV1, SampleModeV1, ShowCategoryV1, TruthModeV1,
};
use quanta_gqlang_v2_modules::CatalogAssetIdV2;
use std::sync::Arc;

/// Sealed rename intent. Target identity comes from the closed host symbol
/// catalog; snapshot/token remain source preconditions, not grants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRenameCommandV3 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    target: crate::ResolvedSymbolTargetV3,
    new_name: String,
    phase: quanta_gqlang_v2_hir::RenamePhaseV3,
    expected_snapshot: String,
    preview_token: Option<String>,
}

impl ResolvedRenameCommandV3 {
    pub(crate) fn new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        target: crate::ResolvedSymbolTargetV3,
        new_name: String,
        phase: quanta_gqlang_v2_hir::RenamePhaseV3,
        expected_snapshot: String,
        preview_token: Option<String>,
    ) -> Self {
        Self {
            item_id,
            location,
            target,
            new_name,
            phase,
            expected_snapshot,
            preview_token,
        }
    }
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    #[must_use]
    pub const fn target(&self) -> &crate::ResolvedSymbolTargetV3 {
        &self.target
    }
    #[must_use]
    pub fn new_name(&self) -> &str {
        &self.new_name
    }
    #[must_use]
    pub const fn phase(&self) -> quanta_gqlang_v2_hir::RenamePhaseV3 {
        self.phase
    }
    #[must_use]
    pub fn expected_snapshot(&self) -> &str {
        &self.expected_snapshot
    }
    #[must_use]
    pub fn preview_token(&self) -> Option<&str> {
        self.preview_token.as_deref()
    }
}

/// Semantic role of one resolved FIND reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GraphReferenceRoleV2 {
    /// Source endpoint.
    Source,
    /// Sink endpoint.
    Sink,
    /// Anchor endpoint.
    Anchor,
    /// Reference endpoint.
    Reference,
    /// Pointee endpoint.
    Pointee,
    /// Callable endpoint.
    Callable,
    /// Required waypoint.
    Via,
    /// Required intermediate.
    Through,
    /// Sanitizer.
    Sanitizer,
    /// Scope.
    Scope,
    /// Window.
    Window,
    /// Frontier.
    Frontier,
}

/// Closed local graph capability derived from a prior Composition producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompositionGraphCapabilityV2 {
    /// A PATH or REACHABILITY producer exposes an Entity target.
    Entity,
    /// A SAMPLE producer exposes a Scope target.
    Scope,
}

impl CompositionGraphCapabilityV2 {
    const fn satisfies(self, required: GraphReferenceKindV2) -> bool {
        matches!(
            (self, required),
            (Self::Entity, GraphReferenceKindV2::Entity)
                | (Self::Scope, GraphReferenceKindV2::Scope)
        )
    }
}

/// Opaque Composition-local graph target.
///
/// Its binding and capability are private so no public API can relabel a raw
/// Composition coordinate as a graph selector. The transaction derives it
/// only from a prior sealed producer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedCompositionGraphTargetV2 {
    source: ResolvedCompositionSourceIdV2,
    capability: CompositionGraphCapabilityV2,
}

impl ResolvedCompositionGraphTargetV2 {
    /// Exact retained Binder-derived Composition source.
    #[must_use]
    pub const fn source(&self) -> &ResolvedCompositionSourceIdV2 {
        &self.source
    }

    /// Closed capability derived from the prior producer.
    #[must_use]
    pub const fn capability(&self) -> CompositionGraphCapabilityV2 {
        self.capability
    }
}

/// One resolved graph target. Host and Composition targets are disjoint.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResolvedGraphTargetV2 {
    /// Immutable host graph authority selector.
    Host(GraphReferenceIdV2),
    /// Prior Composition producer with an explicit frontend capability.
    Composition(ResolvedCompositionGraphTargetV2),
}

impl ResolvedGraphTargetV2 {
    pub(crate) const fn host(target: GraphReferenceIdV2) -> Self {
        Self::Host(target)
    }

    fn satisfies(&self, required: GraphReferenceKindV2) -> bool {
        match self {
            Self::Host(target) => target.kind() == required,
            Self::Composition(target) => target.capability.satisfies(required),
        }
    }

    /// Borrow the host selector when this target is host-owned.
    #[must_use]
    pub const fn host_selector(&self) -> Option<&GraphReferenceIdV2> {
        match self {
            Self::Host(target) => Some(target),
            Self::Composition(_) => None,
        }
    }
}

/// One role-qualified, kind-checked FIND reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphReferenceV2 {
    role: GraphReferenceRoleV2,
    target: ResolvedGraphTargetV2,
    use_location: CheckedLocatedSpanV2,
}

/// One host-or-Composition target at its exact checked document use site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphUseV2 {
    target: ResolvedGraphTargetV2,
    location: CheckedLocatedSpanV2,
}

impl ResolvedGraphUseV2 {
    pub(crate) fn new(target: ResolvedGraphTargetV2, location: CheckedLocatedSpanV2) -> Self {
        Self { target, location }
    }
    /// Opaque host-or-Composition target.
    #[must_use]
    pub const fn target(&self) -> &ResolvedGraphTargetV2 {
        &self.target
    }
    /// Checked source use site.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

impl ResolvedGraphReferenceV2 {
    /// Only a target with the role's required capability may fill a FIND role.
    pub(crate) fn try_new(
        role: GraphReferenceRoleV2,
        target: ResolvedGraphTargetV2,
        use_location: CheckedLocatedSpanV2,
    ) -> Result<Self, &'static str> {
        let required = match role {
            GraphReferenceRoleV2::Source
            | GraphReferenceRoleV2::Sink
            | GraphReferenceRoleV2::Anchor
            | GraphReferenceRoleV2::Reference
            | GraphReferenceRoleV2::Pointee
            | GraphReferenceRoleV2::Via
            | GraphReferenceRoleV2::Through => GraphReferenceKindV2::Entity,
            GraphReferenceRoleV2::Callable => GraphReferenceKindV2::Callable,
            GraphReferenceRoleV2::Sanitizer => GraphReferenceKindV2::Sanitizer,
            GraphReferenceRoleV2::Scope => GraphReferenceKindV2::Scope,
            GraphReferenceRoleV2::Window => GraphReferenceKindV2::Window,
            GraphReferenceRoleV2::Frontier => GraphReferenceKindV2::Frontier,
        };
        if !target.satisfies(required) {
            return Err("FIND reference role/selector kind mismatch");
        }
        Ok(Self {
            role,
            target,
            use_location,
        })
    }
    /// Semantic role.
    #[must_use]
    pub const fn role(&self) -> GraphReferenceRoleV2 {
        self.role
    }
    /// Host-or-Composition graph target identity.
    #[must_use]
    pub const fn target(&self) -> &ResolvedGraphTargetV2 {
        &self.target
    }
    /// Checked authored reference location.
    #[must_use]
    pub const fn use_location(&self) -> &CheckedLocatedSpanV2 {
        &self.use_location
    }
}

/// Fully reference-resolved FIND input for the logic owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphQueryV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    kind: GraphQueryKindV1,
    references: Vec<ResolvedGraphReferenceV2>,
    depth: Option<u32>,
    direction: Option<GraphDirectionV2>,
    summary_facets: Vec<String>,
    witness_bounds: GraphWitnessBoundsV2,
}

/// Exact frontend limits passed to the graph-logic owner. The logic owner
/// retains responsibility for subkind-specific dedup and witness tie-break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphWitnessBoundsV2 {
    max_depth: u32,
    max_witnesses: u32,
}

impl GraphWitnessBoundsV2 {
    pub(crate) const fn new(max_depth: u32, max_witnesses: u32) -> Self {
        Self {
            max_depth,
            max_witnesses,
        }
    }
    #[must_use]
    pub const fn max_depth(self) -> u32 {
        self.max_depth
    }
    #[must_use]
    pub const fn max_witnesses(self) -> u32 {
        self.max_witnesses
    }
}

/// Closed traversal directions admitted by the surface parser and rechecked
/// here for programmatic HIR construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphDirectionV2 {
    Forward,
    Backward,
    Both,
    Undirected,
}

impl GraphDirectionV2 {
    pub(crate) fn try_from_keyword(keyword: &str) -> Result<Self, &'static str> {
        match keyword {
            "forward" => Ok(Self::Forward),
            "backward" => Ok(Self::Backward),
            "both" => Ok(Self::Both),
            "undirected" => Ok(Self::Undirected),
            _ => Err("unsupported FIND direction"),
        }
    }
}

impl ResolvedGraphQueryV2 {
    /// Seal one graph query. Depth zero is not a valid authored traversal bound.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        kind: GraphQueryKindV1,
        references: Vec<ResolvedGraphReferenceV2>,
        depth: Option<u32>,
        direction: Option<GraphDirectionV2>,
        summary_facets: Vec<String>,
        witness_bounds: GraphWitnessBoundsV2,
    ) -> Result<Self, &'static str> {
        if location.location().source_id != item_id.source_id()
            || witness_bounds.max_depth == 0
            || witness_bounds.max_witnesses == 0
        {
            return Err("graph query location or witness budget is invalid");
        }
        if depth.is_some_and(|depth| depth == 0 || depth > witness_bounds.max_depth) {
            return Err("graph traversal depth exceeds the checked frontend bound");
        }
        let singular = references
            .iter()
            .filter(|reference| {
                !matches!(
                    reference.role(),
                    GraphReferenceRoleV2::Via
                        | GraphReferenceRoleV2::Through
                        | GraphReferenceRoleV2::Sanitizer
                )
            })
            .map(ResolvedGraphReferenceV2::role)
            .collect::<Vec<_>>();
        if singular
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != singular.len()
        {
            return Err("duplicate singular FIND reference role");
        }
        if references.iter().any(|reference| {
            reference.use_location().registry_identity() != location.registry_identity()
                || reference.use_location().location().source_id != item_id.source_id()
        }) {
            return Err("graph reference belongs to another checked document");
        }
        Ok(Self {
            item_id,
            location,
            kind,
            references,
            depth,
            direction,
            summary_facets,
            witness_bounds,
        })
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Checked source span of the complete FIND item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// FIND kind.
    #[must_use]
    pub const fn kind(&self) -> GraphQueryKindV1 {
        self.kind
    }
    /// Ordered resolved references.
    #[must_use]
    pub fn references(&self) -> &[ResolvedGraphReferenceV2] {
        &self.references
    }
    /// Explicit authored traversal depth.
    #[must_use]
    pub const fn depth(&self) -> Option<u32> {
        self.depth
    }
    /// Authored direction, checked by graph logic in ticket 06.
    #[must_use]
    pub const fn direction(&self) -> Option<GraphDirectionV2> {
        self.direction
    }
    /// Summary facets.
    #[must_use]
    pub fn summary_facets(&self) -> &[String] {
        &self.summary_facets
    }
    /// Exact frontend witness limits; no independent default in logic.
    #[must_use]
    pub const fn witness_bounds(&self) -> GraphWitnessBoundsV2 {
        self.witness_bounds
    }
}

/// Fully resolved CHECK semantics, including exact nested FIND identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCheckV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    rule: ResolvedCatalogReferenceV2,
    scope: Option<ResolvedGraphUseV2>,
    pattern: Option<ResolvedGraphUseV2>,
    nested_graph: Option<ResolvedGraphQueryV2>,
    confirm: ConfirmModeV1,
    emit: EmitModeV1,
    mode: DetectionModeV1,
}

impl ResolvedCheckV2 {
    /// Construct a CHECK and reject a nested graph from another document.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        expected_nested_id: Option<SemanticItemIdV2>,
        rule: ResolvedCatalogReferenceV2,
        scope: Option<ResolvedGraphUseV2>,
        pattern: Option<ResolvedGraphUseV2>,
        nested_graph: Option<ResolvedGraphQueryV2>,
        confirm: ConfirmModeV1,
        emit: EmitModeV1,
        mode: DetectionModeV1,
    ) -> Result<Self, &'static str> {
        if nested_graph.as_ref().map(ResolvedGraphQueryV2::item_id) != expected_nested_id
            || expected_nested_id
                .is_some_and(|id| id.source_id() != item_id.source_id() || id == item_id)
        {
            return Err("nested CHECK/FIND identity mismatch");
        }
        if rule.kind() != quanta_gqlang_v2_modules::NamespaceKindV2::Rule
            || scope
                .as_ref()
                .is_some_and(|id| !id.target().satisfies(GraphReferenceKindV2::Scope))
            || pattern
                .as_ref()
                .is_some_and(|id| !id.target().satisfies(GraphReferenceKindV2::Pattern))
        {
            return Err("CHECK reference kind mismatch");
        }
        Ok(Self {
            item_id,
            location,
            rule,
            scope,
            pattern,
            nested_graph,
            confirm,
            emit,
            mode,
        })
    }
    /// CHECK item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Registry-proven location of this CHECK item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Resolved rule catalog reference.
    #[must_use]
    pub const fn rule(&self) -> &ResolvedCatalogReferenceV2 {
        &self.rule
    }
    /// Optional scope reference.
    #[must_use]
    pub const fn scope(&self) -> Option<&ResolvedGraphUseV2> {
        self.scope.as_ref()
    }
    /// Optional pattern reference.
    #[must_use]
    pub const fn pattern(&self) -> Option<&ResolvedGraphUseV2> {
        self.pattern.as_ref()
    }
    /// Exact nested FIND semantic contract.
    #[must_use]
    pub const fn nested_graph(&self) -> Option<&ResolvedGraphQueryV2> {
        self.nested_graph.as_ref()
    }
    /// Confirmation policy.
    #[must_use]
    pub const fn confirm(&self) -> ConfirmModeV1 {
        self.confirm
    }
    /// Emission target.
    #[must_use]
    pub const fn emit(&self) -> EmitModeV1 {
        self.emit
    }
    /// Detection mode.
    #[must_use]
    pub const fn mode(&self) -> DetectionModeV1 {
        self.mode
    }
}

/// Resolved presentation payload. All reference-bearing forms retain semantic
/// identities rather than raw names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedPresentationKindV2 {
    /// EXPLAIN target and truth/evidence policy.
    Explain {
        target: ResolvedExplainTargetV2,
        evidence: EvidenceModeV1,
        truth: TruthModeV1,
    },
    /// SHOW a closed category.
    Show { category: ShowCategoryV1 },
    /// DESCRIBE a resolved entity.
    Describe { target: ResolvedGraphUseV2 },
    /// SAMPLE retains bounded query metadata and optional resolved scope.
    Sample {
        query_text: String,
        mode: SampleModeV1,
        limit: u32,
        scope: Option<ResolvedGraphUseV2>,
    },
    /// COMPARE two exact targets.
    Compare {
        baseline: ResolvedGraphUseV2,
        candidate: ResolvedGraphUseV2,
        metric: CompareMetricV1,
    },
    /// EXPORT exact source/profile references.
    Export {
        source: ResolvedGraphUseV2,
        format: ExportFormatV1,
        profile: Option<ResolvedCatalogReferenceV2>,
    },
}

/// Disjoint, resolved EXPLAIN target namespaces. A name present in both
/// namespaces must be rejected as ambiguous before this value is constructed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedExplainTargetV2 {
    /// Catalog/source rule.
    Rule(ResolvedCatalogReferenceV2),
    /// Graph entity selector.
    Entity(ResolvedGraphUseV2),
}

/// Presentation item retaining its canonical item identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPresentationV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    kind: ResolvedPresentationKindV2,
}

impl ResolvedPresentationV2 {
    /// Construct a presentation contract. SAMPLE must retain a positive limit.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        kind: ResolvedPresentationKindV2,
    ) -> Result<Self, &'static str> {
        if matches!(&kind, ResolvedPresentationKindV2::Sample { limit: 0, .. }) {
            return Err("sample limit must be positive");
        }
        Ok(Self {
            item_id,
            location,
            kind,
        })
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Registry-proven location of this presentation item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Resolved payload.
    #[must_use]
    pub const fn kind(&self) -> &ResolvedPresentationKindV2 {
        &self.kind
    }
}

/// Resolved document catalog declaration. Provider text is retained as opaque
/// bounded frontend metadata and is never looked up by sema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCatalogItemV2 {
    item_id: SemanticItemIdV2,
    asset_id: CatalogAssetIdV2,
    provider_metadata: Option<String>,
    inline_view: Option<TypedRelationV2>,
    inline_view_dependencies: Vec<ResolvedCatalogReferenceV2>,
    payload: ResolvedCatalogPayloadV2,
}

/// Checked declaration payload retaining authored semantic fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCatalogPayloadV2 {
    /// View alias or typed inline body.
    View {
        query_ref: Option<ResolvedCatalogReferenceV2>,
    },
    /// Opaque provider metadata and exact source reference.
    Model { source: ResolvedCatalogReferenceV2 },
    /// Pack source and activation.
    Pack {
        source: ResolvedCatalogReferenceV2,
        activation: quanta_gqlang_v2_hir::ActivationV1,
    },
    /// Profile capabilities and admission.
    Profile {
        capabilities: Vec<ResolvedCatalogReferenceV2>,
        admission: quanta_gqlang_v2_hir::AdmissionV1,
    },
    /// Gate checks and binding policy.
    Gate {
        checks: Vec<ResolvedCatalogReferenceV2>,
        binding: quanta_gqlang_v2_hir::GateBindingV1,
    },
    /// Rule binding to an exact CHECK reference.
    RuleBinding {
        check_ref: ResolvedCatalogReferenceV2,
    },
}

impl ResolvedCatalogItemV2 {
    /// Construct a catalog item and retain provider bytes verbatim.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        asset_id: CatalogAssetIdV2,
        provider_metadata: Option<String>,
        expected_inline_view_id: Option<SemanticItemIdV2>,
        inline_view: Option<TypedRelationV2>,
        inline_view_dependencies: Vec<ResolvedCatalogReferenceV2>,
        payload: ResolvedCatalogPayloadV2,
    ) -> Result<Self, &'static str> {
        if !inline_view_identity_matches(item_id, expected_inline_view_id, inline_view.as_ref()) {
            return Err("inline VIEW relation identity differs from its checked nested query");
        }
        let valid = match &payload {
            ResolvedCatalogPayloadV2::View { query_ref } => {
                provider_metadata.is_none()
                    && (query_ref.is_some() ^ inline_view.is_some())
                    && match (query_ref, inline_view.as_ref()) {
                        (None, Some(_)) => inline_view_dependencies.iter().all(|reference| {
                            reference.kind() == quanta_gqlang_v2_modules::NamespaceKindV2::View
                        }),
                        (Some(_), None) => inline_view_dependencies.is_empty(),
                        (None, None) | (Some(_), Some(_)) => false,
                    }
            }
            ResolvedCatalogPayloadV2::Model { .. } => {
                provider_metadata.is_some()
                    && inline_view.is_none()
                    && inline_view_dependencies.is_empty()
            }
            ResolvedCatalogPayloadV2::Pack { .. }
            | ResolvedCatalogPayloadV2::Profile { .. }
            | ResolvedCatalogPayloadV2::Gate { .. }
            | ResolvedCatalogPayloadV2::RuleBinding { .. } => {
                provider_metadata.is_none()
                    && inline_view.is_none()
                    && inline_view_dependencies.is_empty()
            }
        };
        if !valid {
            return Err("catalog semantic payload does not match declaration kind");
        }
        Ok(Self {
            item_id,
            asset_id,
            provider_metadata,
            inline_view,
            inline_view_dependencies,
            payload,
        })
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Modules-owned asset identity.
    #[must_use]
    pub const fn asset_id(&self) -> &CatalogAssetIdV2 {
        &self.asset_id
    }
    /// Opaque provider metadata, byte-for-byte.
    #[must_use]
    pub fn provider_metadata(&self) -> Option<&str> {
        self.provider_metadata.as_deref()
    }
    /// Inline VIEW typed through the canonical relation path.
    #[must_use]
    pub const fn inline_view(&self) -> Option<&TypedRelationV2> {
        self.inline_view.as_ref()
    }
    /// Canonical direct inline-View dependencies. These are retained with
    /// `family=None` and must exactly match the modules topology proof.
    #[must_use]
    pub fn inline_view_dependencies(&self) -> &[ResolvedCatalogReferenceV2] {
        &self.inline_view_dependencies
    }
    /// Resolved family-specific declaration content.
    #[must_use]
    pub const fn payload(&self) -> &ResolvedCatalogPayloadV2 {
        &self.payload
    }
}

fn inline_view_identity_matches(
    catalog_item_id: SemanticItemIdV2,
    expected_inline_view_id: Option<SemanticItemIdV2>,
    inline_view: Option<&TypedRelationV2>,
) -> bool {
    match (expected_inline_view_id, inline_view) {
        (None, None) => true,
        (Some(expected), Some(view)) => {
            expected.source_id() == catalog_item_id.source_id()
                && expected != catalog_item_id
                && view.item_id() == expected
        }
        (None, Some(_)) | (Some(_), None) => false,
    }
}

/// Typed value declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDeclarationV2 {
    item_id: SemanticItemIdV2,
    name: String,
    body: TypedExprV2,
}

impl ResolvedDeclarationV2 {
    /// Construct a checked value declaration.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        name: impl Into<String>,
        body: TypedExprV2,
    ) -> Result<Self, &'static str> {
        let name = name.into();
        if name.is_empty() || body.key().source_id != item_id.source_id() {
            return Err("invalid declaration identity or name");
        }
        Ok(Self {
            item_id,
            name,
            body,
        })
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Declared name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Typed value body.
    #[must_use]
    pub const fn body(&self) -> &TypedExprV2 {
        &self.body
    }
}

/// Closed semantic producer vocabulary for one Composition LET.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCompositionProducerV2 {
    /// SELECT/query producer with its sealed relation tree and output.
    Query(TypedRelationV2),
    /// SAMPLE producer and its exact `Set<{path: RepoPath}>` relation output.
    Sample {
        presentation: ResolvedPresentationV2,
        output: RelationOutputContractV2,
    },
    /// FIND producer. Only PATH/REACHABILITY may expose a relation output.
    Graph {
        graph: ResolvedGraphQueryV2,
        output: Option<RelationOutputContractV2>,
    },
}

impl ResolvedCompositionProducerV2 {
    /// Construct a SAMPLE producer with the canonical relation row.
    pub(crate) fn try_sample(
        presentation: ResolvedPresentationV2,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        if !matches!(
            presentation.kind(),
            ResolvedPresentationKindV2::Sample { .. }
        ) || !is_single_field_set_from_producer(
            &output,
            "path",
            quanta_gqlang_v2_types::ScalarTyV1::RepoPath,
            presentation.item_id(),
        ) {
            return Err("composition SAMPLE producer must expose exactly Set<{path: RepoPath}>");
        }
        Ok(Self::Sample {
            presentation,
            output,
        })
    }

    /// Construct a FIND producer and seal whether it has a relation row.
    pub(crate) fn try_graph(
        graph: ResolvedGraphQueryV2,
        output: Option<RelationOutputContractV2>,
    ) -> Result<Self, &'static str> {
        let row_capable = matches!(
            graph.kind(),
            GraphQueryKindV1::Path | GraphQueryKindV1::Reachability
        );
        let valid = match (row_capable, output.as_ref()) {
            (true, Some(output)) => is_single_field_set_from_producer(
                output,
                "function",
                quanta_gqlang_v2_types::ScalarTyV1::EntityRef,
                graph.item_id(),
            ),
            (false, None) => true,
            (true, None) | (false, Some(_)) => false,
        };
        if !valid {
            return Err(
                "composition FIND producer relation output does not match its closed subkind",
            );
        }
        Ok(Self::Graph { graph, output })
    }

    /// Nested producer identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Query(query) => query.item_id(),
            Self::Sample { presentation, .. } => presentation.item_id(),
            Self::Graph { graph, .. } => graph.item_id(),
        }
    }

    /// Relation row available to a later LET or the body.
    #[must_use]
    pub const fn relation_output(&self) -> Option<&RelationOutputContractV2> {
        match self {
            Self::Query(query) => Some(query.output()),
            Self::Sample { output, .. } => Some(output),
            Self::Graph { output, .. } => output.as_ref(),
        }
    }
}

/// Closed semantic body vocabulary for a Composition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCompositionBodyV2 {
    /// SELECT/query body.
    Query(Box<TypedRelationV2>),
    /// FIND body.
    Graph(ResolvedGraphQueryV2),
    /// CHECK body.
    Check(Box<ResolvedCheckV2>),
    /// EXPLAIN body.
    Explain(ResolvedPresentationV2),
}

impl ResolvedCompositionBodyV2 {
    /// Nested body identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Query(query) => query.item_id(),
            Self::Graph(graph) => graph.item_id(),
            Self::Check(check) => check.item_id(),
            Self::Explain(explain) => explain.item_id(),
        }
    }

    fn is_legal(&self) -> bool {
        !matches!(self, Self::Explain(explain) if !matches!(explain.kind(), ResolvedPresentationKindV2::Explain { .. }))
    }
}

fn is_single_field_set_from_producer(
    output: &RelationOutputContractV2,
    field_name: &str,
    scalar: quanta_gqlang_v2_types::ScalarTyV1,
    producer: SemanticItemIdV2,
) -> bool {
    let Some(field) = output.row().fields().first() else {
        return false;
    };
    output.cardinality() == quanta_gqlang_v2_types::CardinalityV1::Set
        && output.row().fields().len() == 1
        && field.name() == field_name
        && field.value_type() == &quanta_gqlang_v2_types::ValueTypeV1::scalar(scalar)
        && output.partition_keys().is_empty()
        && output.ordering().is_empty()
        && output.provenance().is_empty()
        && output.effects() == quanta_gqlang_v2_types::ExpressionEffectsV2::pure_scalar()
        && matches!(
            output.columns(),
            [column]
                if column.name() == field_name
                    && matches!(column.origin(), OutputColumnOriginV2::Generated(origin) if *origin == producer)
        )
}

/// One Binder-identified Composition LET with a sealed typed producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCompositionBindingV2 {
    binding: ResolvedValueBindingV2,
    name_location: CheckedLocatedSpanV2,
    producer: ResolvedCompositionProducerV2,
}

impl ResolvedCompositionBindingV2 {
    /// Preserve the exact Binder-owned owner/ordinal and producer.
    pub(crate) fn try_new(
        composition_id: SemanticItemIdV2,
        ordinal: usize,
        expected_producer_id: SemanticItemIdV2,
        binding: ResolvedValueBindingV2,
        name_location: CheckedLocatedSpanV2,
        producer: ResolvedCompositionProducerV2,
    ) -> Result<Self, &'static str> {
        if binding.kind() != ResolvedBindingKindV2::Composition
            || binding.source_id() != composition_id.source_id()
            || binding.target_node() != composition_id.hir_item_id()
            || binding.ordinal() != ordinal
            || name_location.location().source_id != composition_id.source_id()
            || producer.item_id() != expected_producer_id
            || expected_producer_id.source_id() != composition_id.source_id()
        {
            return Err(
                "composition binding identity, location, or producer belongs to another owner",
            );
        }
        Ok(Self {
            binding,
            name_location,
            producer,
        })
    }
    /// Exact Binder-owned working-set identity.
    #[must_use]
    pub const fn binding(&self) -> &ResolvedValueBindingV2 {
        &self.binding
    }

    /// Derive the only graph target this already-sealed producer may expose.
    pub(crate) fn try_graph_target(
        &self,
        required: GraphReferenceKindV2,
    ) -> Result<ResolvedGraphTargetV2, &'static str> {
        let capability = match self.producer() {
            ResolvedCompositionProducerV2::Sample { .. } => CompositionGraphCapabilityV2::Scope,
            ResolvedCompositionProducerV2::Graph { graph, .. }
                if matches!(
                    graph.kind(),
                    GraphQueryKindV1::Path | GraphQueryKindV1::Reachability
                ) =>
            {
                CompositionGraphCapabilityV2::Entity
            }
            ResolvedCompositionProducerV2::Query(_)
            | ResolvedCompositionProducerV2::Graph { .. } => {
                return Err("Composition producer has no graph target capability")
            }
        };
        if !capability.satisfies(required) {
            return Err("Composition graph target capability does not match use role");
        }
        Ok(ResolvedGraphTargetV2::Composition(
            ResolvedCompositionGraphTargetV2 {
                source: ResolvedCompositionSourceIdV2::try_from_binding(self.binding.clone())?,
                capability,
            },
        ))
    }
    /// Working-set name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.binding.canonical_name()
    }
    /// Checked authored name location.
    #[must_use]
    pub const fn name_location(&self) -> &CheckedLocatedSpanV2 {
        &self.name_location
    }
    /// Closed typed producer.
    #[must_use]
    pub const fn producer(&self) -> &ResolvedCompositionProducerV2 {
        &self.producer
    }
    /// Relation row available from this producer, if any.
    #[must_use]
    pub const fn relation_output(&self) -> Option<&RelationOutputContractV2> {
        self.producer.relation_output()
    }
}

/// Opaque identity of one sealed composition transaction.
/// This process-local seal binds lowerings to this exact resolved instance;
/// source-qualified IDs alone may be reused by another document transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionInstanceV2(Arc<()>);

impl CompositionInstanceV2 {
    /// Whether two references originate from the same resolved composition.
    #[must_use]
    pub fn same_instance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Typed composition item retaining sequential producers and exact body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCompositionV2 {
    instance: CompositionInstanceV2,
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    bindings: Vec<ResolvedCompositionBindingV2>,
    body: ResolvedCompositionBodyV2,
    with_target: ResolvedValueBindingV2,
    with_location: CheckedLocatedSpanV2,
}

impl ResolvedCompositionV2 {
    /// Check exact binding ownership, closed body family, and resolved WITH.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        bindings: Vec<ResolvedCompositionBindingV2>,
        expected_body_id: SemanticItemIdV2,
        body: ResolvedCompositionBodyV2,
        with_target: ResolvedValueBindingV2,
        with_location: CheckedLocatedSpanV2,
    ) -> Result<Self, &'static str> {
        let names = bindings
            .iter()
            .map(ResolvedCompositionBindingV2::name)
            .collect::<std::collections::BTreeSet<_>>();
        let exact_ordinals = bindings.iter().enumerate().all(|(ordinal, binding)| {
            binding.binding().source_id() == item_id.source_id()
                && binding.binding().target_node() == item_id.hir_item_id()
                && binding.binding().ordinal() == ordinal
                && binding.name_location().registry_identity() == location.registry_identity()
        });
        let with_matches = bindings
            .iter()
            .filter(|binding| binding.binding() == &with_target)
            .count()
            == 1;
        if bindings.is_empty()
            || names.len() != bindings.len()
            || !exact_ordinals
            || !with_matches
            || with_target.kind() != ResolvedBindingKindV2::Composition
            || location.location().source_id != item_id.source_id()
            || with_location.location().source_id != item_id.source_id()
            || with_location.registry_identity() != location.registry_identity()
            || body.item_id() != expected_body_id
            || expected_body_id.source_id() != item_id.source_id()
            || !body.is_legal()
        {
            return Err(
                "composition identities, closed body, or WITH target do not match the checked item",
            );
        }
        Ok(Self {
            instance: CompositionInstanceV2(Arc::new(())),
            item_id,
            location,
            bindings,
            body,
            with_target,
            with_location,
        })
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Opaque identity of this resolved composition transaction.
    #[must_use]
    pub const fn instance(&self) -> &CompositionInstanceV2 {
        &self.instance
    }
    /// Checked authored composition location.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Ordered typed producers.
    #[must_use]
    pub fn bindings(&self) -> &[ResolvedCompositionBindingV2] {
        &self.bindings
    }
    /// Typed consumer body.
    #[must_use]
    pub const fn body(&self) -> &ResolvedCompositionBodyV2 {
        &self.body
    }
    /// Exact checked WITH target.
    #[must_use]
    pub const fn with_target(&self) -> &ResolvedValueBindingV2 {
        &self.with_target
    }
    /// Checked WITH use location.
    #[must_use]
    pub const fn with_location(&self) -> &CheckedLocatedSpanV2 {
        &self.with_location
    }
}

/// Metadata directive retained for document-wide validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMetaV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    kind: ResolvedMetaKindV2,
}

/// Closed metadata payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedMetaKindV2 {
    /// USE PACK, resolved against validated pack authority.
    UsePack {
        pack: quanta_gqlang_v2_modules::PackIdV2,
        activation: quanta_gqlang_v2_hir::UsePackActivationV1,
    },
    /// REQUIRE CERTIFIED.
    RequireCertified,
}

impl ResolvedMetaV2 {
    /// Construct a resolved directive.
    #[must_use]
    pub(crate) const fn new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        kind: ResolvedMetaKindV2,
    ) -> Self {
        Self {
            item_id,
            location,
            kind,
        }
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Checked source span of the authored directive.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Directive payload.
    #[must_use]
    pub const fn kind(&self) -> &ResolvedMetaKindV2 {
        &self.kind
    }
}

/// Closed semantic document item vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticItemV2 {
    /// Sealed source-authored rename intent.
    Command(ResolvedRenameCommandV3),
    /// Relational query/composition result.
    Relation(TypedRelationV2),
    /// Resolved source rule.
    Rule(ResolvedRuleV2),
    /// FIND contract.
    Graph(ResolvedGraphQueryV2),
    /// CHECK contract.
    Check(Box<ResolvedCheckV2>),
    /// Presentation contract.
    Presentation(ResolvedPresentationV2),
    /// Catalog declaration.
    Catalog(ResolvedCatalogItemV2),
    /// Value declaration.
    Declaration(ResolvedDeclarationV2),
    /// Sequential LET composition.
    Composition(ResolvedCompositionV2),
    /// Document metadata directive.
    Meta(ResolvedMetaV2),
}

impl SemanticItemV2 {
    /// Canonical item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Command(item) => item.item_id(),
            Self::Relation(item) => item.item_id(),
            Self::Rule(item) => item.item_id(),
            Self::Graph(item) => item.item_id(),
            Self::Check(item) => item.item_id(),
            Self::Presentation(item) => item.item_id(),
            Self::Catalog(item) => item.item_id(),
            Self::Declaration(item) => item.item_id(),
            Self::Composition(item) => item.item_id(),
            Self::Meta(item) => item.item_id(),
        }
    }
}

#[cfg(test)]
#[path = "item/tests.rs"]
mod tests;
