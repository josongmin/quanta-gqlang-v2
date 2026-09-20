//! Typed semantic contracts for non-relational document item families.

use crate::{
    GraphReferenceIdV2, GraphReferenceKindV2, ResolvedCatalogReferenceV2, ResolvedRuleV2, SemanticItemIdV2,
    TypedExprV2, TypedRelationV2,
};
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, CompareMetricV1, ConfirmModeV1, DetectionModeV1, EmitModeV1, EvidenceModeV1, ExportFormatV1,
    GraphQueryKindV1, SampleModeV1, ShowCategoryV1, TruthModeV1,
};
use quanta_gqlang_v2_modules::CatalogAssetIdV2;

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

/// One role-qualified, kind-checked FIND reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphReferenceV2 {
    role: GraphReferenceRoleV2,
    target: GraphReferenceIdV2,
    use_location: CheckedLocatedSpanV2,
}

/// One host-admitted selector at its exact checked document use site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphUseV2 {
    target: GraphReferenceIdV2,
    location: CheckedLocatedSpanV2,
}

impl ResolvedGraphUseV2 {
    pub(crate) fn new(target: GraphReferenceIdV2, location: CheckedLocatedSpanV2) -> Self { Self { target, location } }
    /// Opaque full-catalog-proven selector.
    #[must_use] pub const fn target(&self) -> &GraphReferenceIdV2 { &self.target }
    /// Checked source use site.
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.location }
}

impl ResolvedGraphReferenceV2 {
    /// Only matching host selector kinds may fill a FIND role.
    pub(crate) fn try_new(role: GraphReferenceRoleV2, target: GraphReferenceIdV2, use_location: CheckedLocatedSpanV2) -> Result<Self, &'static str> {
        let required = match role {
            GraphReferenceRoleV2::Source | GraphReferenceRoleV2::Sink | GraphReferenceRoleV2::Anchor
            | GraphReferenceRoleV2::Reference | GraphReferenceRoleV2::Pointee
            | GraphReferenceRoleV2::Via | GraphReferenceRoleV2::Through => GraphReferenceKindV2::Entity,
            GraphReferenceRoleV2::Callable => GraphReferenceKindV2::Callable,
            GraphReferenceRoleV2::Sanitizer => GraphReferenceKindV2::Sanitizer,
            GraphReferenceRoleV2::Scope => GraphReferenceKindV2::Scope,
            GraphReferenceRoleV2::Window => GraphReferenceKindV2::Window,
            GraphReferenceRoleV2::Frontier => GraphReferenceKindV2::Frontier,
        };
        if target.kind() != required { return Err("FIND reference role/selector kind mismatch"); }
        Ok(Self { role, target, use_location })
    }
    /// Semantic role.
    #[must_use] pub const fn role(&self) -> GraphReferenceRoleV2 { self.role }
    /// Host-admitted graph selector identity.
    #[must_use] pub const fn target(&self) -> &GraphReferenceIdV2 { &self.target }
    /// Checked authored reference location.
    #[must_use] pub const fn use_location(&self) -> &CheckedLocatedSpanV2 { &self.use_location }
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
    pub(crate) const fn new(max_depth: u32, max_witnesses: u32) -> Self { Self { max_depth, max_witnesses } }
    #[must_use] pub const fn max_depth(self) -> u32 { self.max_depth }
    #[must_use] pub const fn max_witnesses(self) -> u32 { self.max_witnesses }
}

/// Closed traversal directions admitted by the surface parser and rechecked
/// here for programmatic HIR construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphDirectionV2 { Forward, Backward, Both, Undirected }

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
        if location.location().source_id != item_id.source_id() || witness_bounds.max_depth == 0 || witness_bounds.max_witnesses == 0 {
            return Err("graph query location or witness budget is invalid");
        }
        if depth.is_some_and(|depth| depth == 0 || depth > witness_bounds.max_depth) {
            return Err("graph traversal depth exceeds the checked frontend bound");
        }
        let singular = references.iter().filter(|reference| !matches!(reference.role(), GraphReferenceRoleV2::Via | GraphReferenceRoleV2::Through | GraphReferenceRoleV2::Sanitizer)).map(ResolvedGraphReferenceV2::role).collect::<Vec<_>>();
        if singular.iter().copied().collect::<std::collections::BTreeSet<_>>().len() != singular.len() { return Err("duplicate singular FIND reference role"); }
        if references.iter().any(|reference| reference.use_location().registry_identity() != location.registry_identity()
            || reference.use_location().location().source_id != item_id.source_id()) {
            return Err("graph reference belongs to another checked document");
        }
        Ok(Self { item_id, location, kind, references, depth, direction, summary_facets, witness_bounds })
    }
    /// Item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Checked source span of the complete FIND item.
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.location }
    /// FIND kind.
    #[must_use] pub const fn kind(&self) -> GraphQueryKindV1 { self.kind }
    /// Ordered resolved references.
    #[must_use] pub fn references(&self) -> &[ResolvedGraphReferenceV2] { &self.references }
    /// Explicit authored traversal depth.
    #[must_use] pub const fn depth(&self) -> Option<u32> { self.depth }
    /// Authored direction, checked by graph logic in ticket 06.
    #[must_use] pub const fn direction(&self) -> Option<GraphDirectionV2> { self.direction }
    /// Summary facets.
    #[must_use] pub fn summary_facets(&self) -> &[String] { &self.summary_facets }
    /// Exact frontend witness limits; no independent default in logic.
    #[must_use] pub const fn witness_bounds(&self) -> GraphWitnessBoundsV2 { self.witness_bounds }
}

/// Fully resolved CHECK semantics, including exact nested FIND identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCheckV2 {
    item_id: SemanticItemIdV2,
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
            || expected_nested_id.is_some_and(|id| id.source_id() != item_id.source_id() || id == item_id) {
            return Err("nested CHECK/FIND identity mismatch");
        }
        if rule.kind() != quanta_gqlang_v2_modules::NamespaceKindV2::Rule
            || scope.as_ref().is_some_and(|id| id.target().kind() != GraphReferenceKindV2::Scope)
            || pattern.as_ref().is_some_and(|id| id.target().kind() != GraphReferenceKindV2::Pattern) {
            return Err("CHECK reference kind mismatch");
        }
        Ok(Self { item_id, rule, scope, pattern, nested_graph, confirm, emit, mode })
    }
    /// CHECK item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Resolved rule catalog reference.
    #[must_use] pub const fn rule(&self) -> &ResolvedCatalogReferenceV2 { &self.rule }
    /// Optional scope reference.
    #[must_use] pub const fn scope(&self) -> Option<&ResolvedGraphUseV2> { self.scope.as_ref() }
    /// Optional pattern reference.
    #[must_use] pub const fn pattern(&self) -> Option<&ResolvedGraphUseV2> { self.pattern.as_ref() }
    /// Exact nested FIND semantic contract.
    #[must_use] pub const fn nested_graph(&self) -> Option<&ResolvedGraphQueryV2> { self.nested_graph.as_ref() }
    /// Confirmation policy.
    #[must_use] pub const fn confirm(&self) -> ConfirmModeV1 { self.confirm }
    /// Emission target.
    #[must_use] pub const fn emit(&self) -> EmitModeV1 { self.emit }
    /// Detection mode.
    #[must_use] pub const fn mode(&self) -> DetectionModeV1 { self.mode }
}

/// Resolved presentation payload. All reference-bearing forms retain semantic
/// identities rather than raw names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedPresentationKindV2 {
    /// EXPLAIN target and truth/evidence policy.
    Explain { target: ResolvedExplainTargetV2, evidence: EvidenceModeV1, truth: TruthModeV1 },
    /// SHOW a closed category.
    Show { category: ShowCategoryV1 },
    /// DESCRIBE a resolved entity.
    Describe { target: ResolvedGraphUseV2 },
    /// SAMPLE retains bounded query metadata and optional resolved scope.
    Sample { query_text: String, mode: SampleModeV1, limit: u32, scope: Option<ResolvedGraphUseV2> },
    /// COMPARE two exact targets.
    Compare { baseline: ResolvedGraphUseV2, candidate: ResolvedGraphUseV2, metric: CompareMetricV1 },
    /// EXPORT exact source/profile references.
    Export { source: ResolvedGraphUseV2, format: ExportFormatV1, profile: Option<ResolvedCatalogReferenceV2> },
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
    kind: ResolvedPresentationKindV2,
}

impl ResolvedPresentationV2 {
    /// Construct a presentation contract. SAMPLE must retain a positive limit.
    pub(crate) fn try_new(item_id: SemanticItemIdV2, kind: ResolvedPresentationKindV2) -> Result<Self, &'static str> {
        if matches!(&kind, ResolvedPresentationKindV2::Sample { limit: 0, .. }) {
            return Err("sample limit must be positive");
        }
        Ok(Self { item_id, kind })
    }
    /// Item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Resolved payload.
    #[must_use] pub const fn kind(&self) -> &ResolvedPresentationKindV2 { &self.kind }
}

/// Resolved document catalog declaration. Provider text is retained as opaque
/// bounded frontend metadata and is never looked up by sema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCatalogItemV2 {
    item_id: SemanticItemIdV2,
    asset_id: CatalogAssetIdV2,
    provider_metadata: Option<String>,
    inline_view: Option<TypedRelationV2>,
    payload: ResolvedCatalogPayloadV2,
}

/// Checked declaration payload retaining authored semantic fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCatalogPayloadV2 {
    /// View alias or typed inline body.
    View { query_ref: Option<ResolvedCatalogReferenceV2> },
    /// Opaque provider metadata and exact source reference.
    Model { source: ResolvedCatalogReferenceV2 },
    /// Pack source and activation.
    Pack { source: ResolvedCatalogReferenceV2, activation: quanta_gqlang_v2_hir::ActivationV1 },
    /// Profile capabilities and admission.
    Profile { capabilities: Vec<ResolvedCatalogReferenceV2>, admission: quanta_gqlang_v2_hir::AdmissionV1 },
    /// Gate checks and binding policy.
    Gate { checks: Vec<ResolvedCatalogReferenceV2>, binding: quanta_gqlang_v2_hir::GateBindingV1 },
    /// Rule binding to an exact CHECK reference.
    RuleBinding { check_ref: ResolvedCatalogReferenceV2 },
}

impl ResolvedCatalogItemV2 {
    /// Construct a catalog item and retain provider bytes verbatim.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        asset_id: CatalogAssetIdV2,
        provider_metadata: Option<String>,
        inline_view: Option<TypedRelationV2>,
        payload: ResolvedCatalogPayloadV2,
    ) -> Result<Self, &'static str> {
        if inline_view.as_ref().is_some_and(|view| view.item_id().source_id() != item_id.source_id()) {
            return Err("inline VIEW relation belongs to another document");
        }
        let valid = match &payload {
            ResolvedCatalogPayloadV2::View { query_ref } => provider_metadata.is_none() && (query_ref.is_some() ^ inline_view.is_some()),
            ResolvedCatalogPayloadV2::Model { .. } => provider_metadata.is_some() && inline_view.is_none(),
            ResolvedCatalogPayloadV2::Pack { .. }
            | ResolvedCatalogPayloadV2::Profile { .. }
            | ResolvedCatalogPayloadV2::Gate { .. }
            | ResolvedCatalogPayloadV2::RuleBinding { .. } => provider_metadata.is_none() && inline_view.is_none(),
        };
        if !valid { return Err("catalog semantic payload does not match declaration kind"); }
        Ok(Self { item_id, asset_id, provider_metadata, inline_view, payload })
    }
    /// Item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Modules-owned asset identity.
    #[must_use] pub const fn asset_id(&self) -> &CatalogAssetIdV2 { &self.asset_id }
    /// Opaque provider metadata, byte-for-byte.
    #[must_use] pub fn provider_metadata(&self) -> Option<&str> { self.provider_metadata.as_deref() }
    /// Inline VIEW typed through the canonical relation path.
    #[must_use] pub const fn inline_view(&self) -> Option<&TypedRelationV2> { self.inline_view.as_ref() }
    /// Resolved family-specific declaration content.
    #[must_use] pub const fn payload(&self) -> &ResolvedCatalogPayloadV2 { &self.payload }
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
    pub(crate) fn try_new(item_id: SemanticItemIdV2, name: impl Into<String>, body: TypedExprV2) -> Result<Self, &'static str> {
        let name = name.into();
        if name.is_empty() || body.key().source_id != item_id.source_id() { return Err("invalid declaration identity or name"); }
        Ok(Self { item_id, name, body })
    }
    /// Item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Declared name.
    #[must_use] pub fn name(&self) -> &str { &self.name }
    /// Typed value body.
    #[must_use] pub const fn body(&self) -> &TypedExprV2 { &self.body }
}

/// One named composition producer with a sealed typed output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCompositionBindingV2 {
    name: String,
    item: Box<SemanticItemV2>,
}

impl ResolvedCompositionBindingV2 {
    /// Preserve a producer and its authored working-set name.
    pub(crate) fn try_new(name: impl Into<String>, item: SemanticItemV2) -> Result<Self, &'static str> {
        let name = name.into();
        if name.is_empty() { return Err("composition binding name cannot be empty"); }
        Ok(Self { name, item: Box::new(item) })
    }
    /// Working-set name.
    #[must_use] pub fn name(&self) -> &str { &self.name }
    /// Typed producer.
    #[must_use] pub const fn item(&self) -> &SemanticItemV2 { &self.item }
}

/// Typed composition item retaining sequential producers and exact body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCompositionV2 {
    item_id: SemanticItemIdV2,
    bindings: Vec<ResolvedCompositionBindingV2>,
    body: Box<SemanticItemV2>,
    with_name: String,
}

impl ResolvedCompositionV2 {
    /// Check unique working-set names and exact WITH reference.
    pub(crate) fn try_new(item_id: SemanticItemIdV2, bindings: Vec<ResolvedCompositionBindingV2>, body: SemanticItemV2, with_name: impl Into<String>) -> Result<Self, &'static str> {
        let with_name = with_name.into();
        let names: std::collections::BTreeSet<_> = bindings.iter().map(ResolvedCompositionBindingV2::name).collect();
        if names.len() != bindings.len() || !names.contains(with_name.as_str()) { return Err("composition WITH must name exactly one declared producer"); }
        Ok(Self { item_id, bindings, body: Box::new(body), with_name })
    }
    /// Item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Ordered typed producers.
    #[must_use] pub fn bindings(&self) -> &[ResolvedCompositionBindingV2] { &self.bindings }
    /// Typed consumer body.
    #[must_use] pub const fn body(&self) -> &SemanticItemV2 { &self.body }
    /// Checked WITH binding.
    #[must_use] pub fn with_name(&self) -> &str { &self.with_name }
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
    UsePack { pack: quanta_gqlang_v2_modules::PackIdV2, activation: quanta_gqlang_v2_hir::UsePackActivationV1 },
    /// REQUIRE CERTIFIED.
    RequireCertified,
}

impl ResolvedMetaV2 {
    /// Construct a resolved directive.
    #[must_use] pub(crate) const fn new(item_id: SemanticItemIdV2, location: CheckedLocatedSpanV2, kind: ResolvedMetaKindV2) -> Self { Self { item_id, location, kind } }
    /// Item identity.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Checked source span of the authored directive.
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.location }
    /// Directive payload.
    #[must_use] pub const fn kind(&self) -> &ResolvedMetaKindV2 { &self.kind }
}

/// Closed semantic document item vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticItemV2 {
    /// Relational query/composition result.
    Relation(TypedRelationV2),
    /// Resolved source rule.
    Rule(ResolvedRuleV2),
    /// FIND contract.
    Graph(ResolvedGraphQueryV2),
    /// CHECK contract.
    Check(ResolvedCheckV2),
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
