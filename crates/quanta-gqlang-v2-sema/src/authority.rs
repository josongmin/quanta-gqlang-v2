//! Closed and open semantic authority plus the canonical frontend budget.

use crate::diagnostic::{
    bounded_identifier, FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2,
    FrontendStageV2,
};
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, ExtendedSourceRegistryViewV2, LocatedSpanV2, SourceIdV2,
    SourceRegistrationV2, SourceRegistryIdentityV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    CallableKindV2, ContentDigestV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2,
    ExternalCatalogPayloadInputV2, LocatedParameterInputV2, LockReferenceInputV2,
    ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, PackIdV2, PackManifestInputV2,
    SignatureInputV2, ValidatedModuleAuthorityPartsV2, VersionRangeV2, VersionV2,
};
use quanta_gqlang_v2_types::{
    AuthorityBudgetProfileV2, CardinalityV1, ExpressionEffectsV2, ExpressionTypeKernelV2,
    ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationAuthorityV2, RelationCatalogV2,
    RelationSourceInputV2, ScalarTyV1, ValueTypeV1, CANONICAL_WORKSPACE_FILES_FIELD_NAME_V2,
    CANONICAL_WORKSPACE_FILES_LOOKUP_NAME_V2, CANONICAL_WORKSPACE_FILES_PROVENANCE_NAMESPACE_V2,
    CANONICAL_WORKSPACE_FILES_PROVENANCE_V2, CANONICAL_WORKSPACE_FILES_QUALIFIED_NAME_V2,
};
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::sync::Arc;

/// Source-independent authority construction error. It never fabricates a
/// document ID or a byte span for host configuration failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendAuthorityErrorV2 {
    code: &'static str,
    message: String,
    primary: Option<LocatedSpanV2>,
    secondary: Option<LocatedSpanV2>,
}

impl FrontendAuthorityErrorV2 {
    /// Stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }
    /// Bounded explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Checked host source location, if this authority error has one.
    #[must_use]
    pub const fn primary(&self) -> Option<LocatedSpanV2> {
        self.primary
    }
    /// Conflicting declaration location, if present.
    #[must_use]
    pub const fn secondary(&self) -> Option<LocatedSpanV2> {
        self.secondary
    }

    pub(crate) fn from_external(code: &'static str, message: impl AsRef<str>) -> Self {
        configuration_failure(code, message)
    }
}

/// Closed kind vocabulary for host-admitted graph selectors. These selectors
/// are opaque frontend identities, not modules assets or provider handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GraphReferenceKindV2 {
    /// Graph entity/endpoint/waypoint.
    Entity,
    /// Callable endpoint.
    Callable,
    /// Analysis scope.
    Scope,
    /// Window selector.
    Window,
    /// Frontier selector.
    Frontier,
    /// Sanitizer selector.
    Sanitizer,
    /// CHECK pattern selector.
    Pattern,
    /// Comparison baseline/candidate snapshot.
    Snapshot,
    /// EXPORT source selector.
    ExportSource,
}

/// Deterministic identity in one sealed host graph-reference authority.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GraphReferenceIdV2 {
    kind: GraphReferenceKindV2,
    spelling: String,
    entity_subtype: Option<String>,
    location: CheckedLocatedSpanV2,
    catalog_identity: Arc<GraphReferenceCatalogIdentityV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct GraphReferenceCatalogIdentityV2 {
    registry: SourceRegistryIdentityV2,
    budget: AuthorityBudgetProfileV2,
    entries: Vec<(
        String,
        GraphReferenceKindV2,
        Option<String>,
        CheckedLocatedSpanV2,
    )>,
}

impl GraphReferenceIdV2 {
    /// Checked selector kind.
    #[must_use]
    pub const fn kind(&self) -> GraphReferenceKindV2 {
        self.kind
    }
    /// Exact host-admitted spelling; punctuation is never split/rewritten.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    /// Host-admitted subtype for DESCRIBE Entity selectors.
    #[must_use]
    pub fn entity_subtype(&self) -> Option<&str> {
        self.entity_subtype.as_deref()
    }
    /// Exact checked host declaration that admitted this selector.
    #[must_use]
    pub const fn declaration_location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// One host supplied graph selector and its checked source location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphReferenceInputV2 {
    /// Exact name as matched in source.
    pub spelling: String,
    /// Semantic selector kind.
    pub kind: GraphReferenceKindV2,
    /// Exact entity subtype for DESCRIBE, absent for other selector kinds.
    pub entity_subtype: Option<String>,
    /// Host-registry-checked declaration location.
    pub location: CheckedLocatedSpanV2,
}

/// Immutable graph selector authority. No runtime/provider lookup occurs.
#[derive(Clone, Debug)]
pub struct GraphReferenceCatalogV2 {
    by_spelling: BTreeMap<String, (GraphReferenceIdV2, CheckedLocatedSpanV2)>,
    identity: Arc<GraphReferenceCatalogIdentityV2>,
    registry_identity: SourceRegistryIdentityV2,
    budget_profile: AuthorityBudgetProfileV2,
}

impl GraphReferenceCatalogV2 {
    /// Admit every selector in one non-resettable authority-byte session.
    pub fn try_new(
        inputs: Vec<GraphReferenceInputV2>,
        host_sources: &SourceRegistryViewV2,
        budget_profile: AuthorityBudgetProfileV2,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        let identity = host_sources.identity();
        let mut admitted: BTreeMap<
            String,
            (GraphReferenceKindV2, Option<String>, CheckedLocatedSpanV2),
        > = BTreeMap::new();
        let mut bytes = 0usize;
        for input in inputs {
            if input.location.registry_identity() != &identity {
                return Err(authority_failure_at(
                    "sema.source_registry_mismatch",
                    "graph selector location belongs to another host registry",
                    input.location.location(),
                    None,
                ));
            }
            if input.spelling.is_empty() {
                return Err(authority_failure_at(
                    "sema.invalid_graph_reference",
                    "graph selector spelling cannot be empty",
                    input.location.location(),
                    None,
                ));
            }
            if input.entity_subtype.as_deref().is_some_and(str::is_empty)
                || (input.entity_subtype.is_some() && input.kind != GraphReferenceKindV2::Entity)
            {
                return Err(authority_failure_at(
                    "sema.invalid_graph_reference",
                    "entity subtype must be nonempty and only an Entity may carry it",
                    input.location.location(),
                    None,
                ));
            }
            if input.spelling.len() > budget_profile.authority_string_bytes() {
                return Err(authority_failure_at(
                    "frontend.resource_limit_exceeded",
                    "graph selector exceeds authority_string_bytes",
                    input.location.location(),
                    None,
                ));
            }
            bytes = bytes.saturating_add(input.spelling.len());
            if let Some(subtype) = &input.entity_subtype {
                if subtype.len() > budget_profile.authority_string_bytes() {
                    return Err(authority_failure_at(
                        "frontend.resource_limit_exceeded",
                        "entity subtype exceeds authority_string_bytes",
                        input.location.location(),
                        None,
                    ));
                }
                bytes = bytes.saturating_add(subtype.len());
            }
            if bytes > budget_profile.authority_bytes() {
                return Err(authority_failure_at(
                    "frontend.resource_limit_exceeded",
                    "graph selectors exceed authority_bytes",
                    input.location.location(),
                    None,
                ));
            }
            if let Some((_, _, previous)) = admitted.get(&input.spelling) {
                return Err(authority_failure_at(
                    "sema.ambiguous_reference",
                    "graph selector spelling is duplicated",
                    input.location.location(),
                    Some(previous.location()),
                ));
            }
            admitted.insert(
                input.spelling,
                (input.kind, input.entity_subtype, input.location),
            );
        }
        let catalog_identity = Arc::new(GraphReferenceCatalogIdentityV2 {
            registry: identity.clone(),
            budget: budget_profile.clone(),
            entries: admitted
                .iter()
                .map(|(name, (kind, subtype, location))| {
                    (name.clone(), *kind, subtype.clone(), location.clone())
                })
                .collect(),
        });
        let by_spelling = admitted
            .into_iter()
            .map(|(spelling, (kind, entity_subtype, location))| {
                let id = GraphReferenceIdV2 {
                    kind,
                    spelling: spelling.clone(),
                    entity_subtype,
                    location: location.clone(),
                    catalog_identity: Arc::clone(&catalog_identity),
                };
                (spelling, (id, location))
            })
            .collect();
        Ok(Self {
            by_spelling,
            identity: catalog_identity,
            registry_identity: identity,
            budget_profile,
        })
    }

    /// Resolve exact spelling with an expected semantic kind. No punctuation
    /// splitting, prefix search, or fallback namespace is allowed.
    pub fn resolve(
        &self,
        spelling: &str,
        expected: GraphReferenceKindV2,
        location: LocatedSpanV2,
    ) -> Result<GraphReferenceIdV2, FrontendFailureV2> {
        self.probe(spelling, expected, location)?.ok_or_else(|| {
            FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resolution,
                "sema.unresolved_reference",
                FrontendSeverityV2::Error,
                location,
                Vec::new(),
                format!("unknown graph selector `{}`", bounded_identifier(spelling)),
            ))
        })
    }

    /// Nonterminal exact candidate probe for a source form whose namespace is
    /// intentionally ambiguous until catalog and graph authorities are both
    /// queried (EXPLAIN). Wrong-kind is still a hard source diagnostic.
    pub fn probe(
        &self,
        spelling: &str,
        expected: GraphReferenceKindV2,
        location: LocatedSpanV2,
    ) -> Result<Option<GraphReferenceIdV2>, FrontendFailureV2> {
        match self.by_spelling.get(spelling) {
            Some((id, _)) if id.kind == expected => Ok(Some(id.clone())),
            Some((_, declaration)) => Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resolution,
                "sema.wrong_reference_kind",
                FrontendSeverityV2::Error,
                location,
                vec![declaration.location()],
                "graph selector has the wrong semantic kind",
            ))),
            None => Ok(None),
        }
    }

    /// Resolve a DESCRIBE path against an exact host-admitted entity subtype.
    pub fn resolve_entity_subtype(
        &self,
        spelling: &str,
        subtype: &str,
        location: LocatedSpanV2,
    ) -> Result<GraphReferenceIdV2, FrontendFailureV2> {
        let id = self.resolve(spelling, GraphReferenceKindV2::Entity, location)?;
        if id.entity_subtype() != Some(subtype) {
            return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resolution,
                "sema.wrong_reference_kind",
                FrontendSeverityV2::Error,
                location,
                Vec::new(),
                "DESCRIBE kind differs from the host-admitted entity subtype",
            )));
        }
        Ok(id)
    }

    /// Exact host registry identity.
    #[must_use]
    pub const fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
    /// Exact budget profile used at admission.
    #[must_use]
    pub const fn budget_profile(&self) -> &AuthorityBudgetProfileV2 {
        &self.budget_profile
    }
    /// Checked selector count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_spelling.len()
    }
    /// Whether no selectors were admitted by this explicit host proof.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_spelling.is_empty()
    }
    /// Verify an opaque selector was minted by this exact admitted catalog,
    /// including all records, host registry and budget profile.
    #[must_use]
    pub fn admits(&self, id: &GraphReferenceIdV2) -> bool {
        self.identity == id.catalog_identity
            && self
                .by_spelling
                .get(id.spelling())
                .is_some_and(|(owned, _)| owned == id)
    }
}

/// Immutable configuration for every bounded frontend stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendBudgetV2 {
    max_source_bytes: NonZeroU32,
    max_frame_bytes: NonZeroU32,
    max_tokens: NonZeroU32,
    max_document_items: NonZeroU32,
    max_expression_depth: NonZeroU32,
    max_expression_nodes: NonZeroU32,
    max_collection_width: NonZeroU32,
    max_projection_width: NonZeroU32,
    max_semantic_steps: NonZeroU32,
    max_module_graph_nodes: NonZeroU32,
    max_module_graph_edges: NonZeroU32,
    max_module_steps: NonZeroU32,
    max_authority_bytes: NonZeroU32,
    max_authority_string_bytes: NonZeroU32,
    max_rule_graph_nodes: NonZeroU32,
    max_rule_graph_edges: NonZeroU32,
    max_rule_steps: NonZeroU32,
    max_witness_depth: NonZeroU32,
    max_witnesses: NonZeroU32,
    max_diagnostics: NonZeroU32,
    max_response_bytes: NonZeroU32,
}

/// Immutable read-only projection of one `FrontendBudgetV2` retained by the
/// sealed semantic document. Downstream stages (algebra, logic, planning, and
/// the front doors) read their traversal and output caps from the document;
/// no consumer accepts an independent budget argument, a caller-selected
/// replacement, or a default. The receipt never extends the canonical budget:
/// every dimension is copied exactly once by
/// [`FrontendBudgetV2::downstream_receipt`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DownstreamBudgetReceiptV2 {
    max_source_bytes: NonZeroU32,
    max_frame_bytes: NonZeroU32,
    max_tokens: NonZeroU32,
    max_document_items: NonZeroU32,
    max_expression_depth: NonZeroU32,
    max_expression_nodes: NonZeroU32,
    max_collection_width: NonZeroU32,
    max_projection_width: NonZeroU32,
    max_semantic_steps: NonZeroU32,
    max_module_graph_nodes: NonZeroU32,
    max_module_graph_edges: NonZeroU32,
    max_module_steps: NonZeroU32,
    max_authority_bytes: NonZeroU32,
    max_authority_string_bytes: NonZeroU32,
    max_rule_graph_nodes: NonZeroU32,
    max_rule_graph_edges: NonZeroU32,
    max_rule_steps: NonZeroU32,
    max_witness_depth: NonZeroU32,
    max_witnesses: NonZeroU32,
    max_diagnostics: NonZeroU32,
    max_response_bytes: NonZeroU32,
}

impl DownstreamBudgetReceiptV2 {
    /// Maximum source bytes admitted to one document transaction.
    #[must_use]
    pub const fn max_source_bytes(&self) -> u32 {
        self.max_source_bytes.get()
    }
    /// Maximum transport frame bytes for one request.
    #[must_use]
    pub const fn max_frame_bytes(&self) -> u32 {
        self.max_frame_bytes.get()
    }
    /// Maximum surface parse tokens.
    #[must_use]
    pub const fn max_tokens(&self) -> u32 {
        self.max_tokens.get()
    }
    /// Maximum authored document items.
    #[must_use]
    pub const fn max_document_items(&self) -> u32 {
        self.max_document_items.get()
    }
    /// Maximum expression nesting depth.
    #[must_use]
    pub const fn max_expression_depth(&self) -> u32 {
        self.max_expression_depth.get()
    }
    /// Expression walk limit shared by every downstream expression traversal.
    #[must_use]
    pub const fn max_expression_nodes(&self) -> u32 {
        self.max_expression_nodes.get()
    }
    /// Maximum authored collection width.
    #[must_use]
    pub const fn max_collection_width(&self) -> u32 {
        self.max_collection_width.get()
    }
    /// Maximum authored projection width.
    #[must_use]
    pub const fn max_projection_width(&self) -> u32 {
        self.max_projection_width.get()
    }
    /// Relation walk and planning traversal step limit.
    #[must_use]
    pub const fn max_semantic_steps(&self) -> u32 {
        self.max_semantic_steps.get()
    }
    /// Module graph node limit.
    #[must_use]
    pub const fn max_module_graph_nodes(&self) -> u32 {
        self.max_module_graph_nodes.get()
    }
    /// Module graph edge limit.
    #[must_use]
    pub const fn max_module_graph_edges(&self) -> u32 {
        self.max_module_graph_edges.get()
    }
    /// Module admission step limit.
    #[must_use]
    pub const fn max_module_steps(&self) -> u32 {
        self.max_module_steps.get()
    }
    /// Authority byte session limit.
    #[must_use]
    pub const fn max_authority_bytes(&self) -> u32 {
        self.max_authority_bytes.get()
    }
    /// Authority string byte limit.
    #[must_use]
    pub const fn max_authority_string_bytes(&self) -> u32 {
        self.max_authority_string_bytes.get()
    }
    /// Logic rule graph node limit.
    #[must_use]
    pub const fn max_rule_graph_nodes(&self) -> u32 {
        self.max_rule_graph_nodes.get()
    }
    /// Logic rule graph edge limit.
    #[must_use]
    pub const fn max_rule_graph_edges(&self) -> u32 {
        self.max_rule_graph_edges.get()
    }
    /// Logic rule relaxation step limit.
    #[must_use]
    pub const fn max_rule_steps(&self) -> u32 {
        self.max_rule_steps.get()
    }
    /// Graph witness depth limit.
    #[must_use]
    pub const fn max_witness_depth(&self) -> u32 {
        self.max_witness_depth.get()
    }
    /// Graph witness count limit.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses.get()
    }
    /// Maximum diagnostics accumulated for one frontend outcome.
    #[must_use]
    pub const fn max_diagnostics(&self) -> u32 {
        self.max_diagnostics.get()
    }
    /// Maximum serialized response bytes.
    #[must_use]
    pub const fn max_response_bytes(&self) -> u32 {
        self.max_response_bytes.get()
    }
}

impl FrontendBudgetV2 {
    /// Construct one checked frontend budget. Every limit must be positive and
    /// the transport frame must be able to contain the admitted source.
    pub fn try_new(
        max_source_bytes: u32,
        max_frame_bytes: u32,
        max_tokens: u32,
        max_document_items: u32,
        max_expression_depth: u32,
        max_expression_nodes: u32,
        max_collection_width: u32,
        max_projection_width: u32,
        max_semantic_steps: u32,
        max_module_graph_nodes: u32,
        max_module_graph_edges: u32,
        max_module_steps: u32,
        max_authority_bytes: u32,
        max_authority_string_bytes: u32,
        max_rule_graph_nodes: u32,
        max_rule_graph_edges: u32,
        max_rule_steps: u32,
        max_witness_depth: u32,
        max_witnesses: u32,
        max_diagnostics: u32,
        max_response_bytes: u32,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        let raw = [
            max_source_bytes,
            max_frame_bytes,
            max_tokens,
            max_document_items,
            max_expression_depth,
            max_expression_nodes,
            max_collection_width,
            max_projection_width,
            max_semantic_steps,
            max_module_graph_nodes,
            max_module_graph_edges,
            max_module_steps,
            max_authority_bytes,
            max_authority_string_bytes,
            max_rule_graph_nodes,
            max_rule_graph_edges,
            max_rule_steps,
            max_witness_depth,
            max_witnesses,
            max_diagnostics,
            max_response_bytes,
        ];
        if raw.contains(&0) || max_frame_bytes < max_source_bytes {
            return Err(configuration_failure(
                "frontend.invalid_budget",
                "all frontend limits must be positive and frame bytes must cover source bytes",
            ));
        }
        let mut values = raw.into_iter().map(NonZeroU32::new);
        let mut field = || {
            values
                .next()
                .flatten()
                .ok_or_else(|| configuration_failure("frontend.invalid_budget", "invalid budget"))
        };
        Ok(Self {
            max_source_bytes: field()?,
            max_frame_bytes: field()?,
            max_tokens: field()?,
            max_document_items: field()?,
            max_expression_depth: field()?,
            max_expression_nodes: field()?,
            max_collection_width: field()?,
            max_projection_width: field()?,
            max_semantic_steps: field()?,
            max_module_graph_nodes: field()?,
            max_module_graph_edges: field()?,
            max_module_steps: field()?,
            max_authority_bytes: field()?,
            max_authority_string_bytes: field()?,
            max_rule_graph_nodes: field()?,
            max_rule_graph_edges: field()?,
            max_rule_steps: field()?,
            max_witness_depth: field()?,
            max_witnesses: field()?,
            max_diagnostics: field()?,
            max_response_bytes: field()?,
        })
    }

    /// Pinned CLI/service profile.
    pub fn standard_v2() -> Result<Self, FrontendAuthorityErrorV2> {
        Self::try_new(
            1_048_576, 4_194_304, 131_072, 4_096, 64, 262_144, 4_096, 4_096, 1_000_000, 50_000,
            200_000, 1_000_000, 16_777_216, 65_536, 50_000, 200_000, 1_000_000, 1_024, 10_000,
            1_024, 8_388_608,
        )
    }

    /// The one immutable downstream budget receipt. Every downstream traversal
    /// and output cap is read from this projection on the sealed semantic
    /// document; it is never extended, replaced, or accepted as an independent
    /// consumer argument.
    #[must_use]
    pub const fn downstream_receipt(&self) -> DownstreamBudgetReceiptV2 {
        DownstreamBudgetReceiptV2 {
            max_source_bytes: self.max_source_bytes,
            max_frame_bytes: self.max_frame_bytes,
            max_tokens: self.max_tokens,
            max_document_items: self.max_document_items,
            max_expression_depth: self.max_expression_depth,
            max_expression_nodes: self.max_expression_nodes,
            max_collection_width: self.max_collection_width,
            max_projection_width: self.max_projection_width,
            max_semantic_steps: self.max_semantic_steps,
            max_module_graph_nodes: self.max_module_graph_nodes,
            max_module_graph_edges: self.max_module_graph_edges,
            max_module_steps: self.max_module_steps,
            max_authority_bytes: self.max_authority_bytes,
            max_authority_string_bytes: self.max_authority_string_bytes,
            max_rule_graph_nodes: self.max_rule_graph_nodes,
            max_rule_graph_edges: self.max_rule_graph_edges,
            max_rule_steps: self.max_rule_steps,
            max_witness_depth: self.max_witness_depth,
            max_witnesses: self.max_witnesses,
            max_diagnostics: self.max_diagnostics,
            max_response_bytes: self.max_response_bytes,
        }
    }

    /// Canonical projection used by module, relation, and per-document catalog
    /// admission. Each admission owns its own non-resettable counter.
    pub fn authority_profile(&self) -> Result<AuthorityBudgetProfileV2, FrontendAuthorityErrorV2> {
        AuthorityBudgetProfileV2::try_new(
            usize::try_from(self.max_module_graph_nodes.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_module_graph_edges.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_module_steps.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_authority_bytes.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_authority_string_bytes.get()).unwrap_or(usize::MAX),
        )
        .map_err(|error| configuration_failure(error.code(), error.to_string()))
    }

    /// Maximum source bytes.
    #[must_use]
    pub const fn max_source_bytes(&self) -> u32 {
        self.max_source_bytes.get()
    }
    /// Maximum frame bytes.
    #[must_use]
    pub const fn max_frame_bytes(&self) -> u32 {
        self.max_frame_bytes.get()
    }
    /// Maximum tokens.
    #[must_use]
    pub const fn max_tokens(&self) -> u32 {
        self.max_tokens.get()
    }
    /// Maximum document items.
    #[must_use]
    pub const fn max_document_items(&self) -> u32 {
        self.max_document_items.get()
    }
    /// Maximum expression depth.
    #[must_use]
    pub const fn max_expression_depth(&self) -> u32 {
        self.max_expression_depth.get()
    }
    /// Maximum expression nodes.
    #[must_use]
    pub const fn max_expression_nodes(&self) -> u32 {
        self.max_expression_nodes.get()
    }
    /// Maximum collection width.
    #[must_use]
    pub const fn max_collection_width(&self) -> u32 {
        self.max_collection_width.get()
    }
    /// Maximum projection width.
    #[must_use]
    pub const fn max_projection_width(&self) -> u32 {
        self.max_projection_width.get()
    }
    /// Maximum semantic steps.
    #[must_use]
    pub const fn max_semantic_steps(&self) -> u32 {
        self.max_semantic_steps.get()
    }
    /// Maximum module graph nodes.
    #[must_use]
    pub const fn max_module_graph_nodes(&self) -> u32 {
        self.max_module_graph_nodes.get()
    }
    /// Maximum module graph edges.
    #[must_use]
    pub const fn max_module_graph_edges(&self) -> u32 {
        self.max_module_graph_edges.get()
    }
    /// Maximum module steps.
    #[must_use]
    pub const fn max_module_steps(&self) -> u32 {
        self.max_module_steps.get()
    }
    /// Maximum authority bytes per sealed admission session.
    #[must_use]
    pub const fn max_authority_bytes(&self) -> u32 {
        self.max_authority_bytes.get()
    }
    /// Maximum bytes in one authority string.
    #[must_use]
    pub const fn max_authority_string_bytes(&self) -> u32 {
        self.max_authority_string_bytes.get()
    }
    /// Maximum rule graph nodes.
    #[must_use]
    pub const fn max_rule_graph_nodes(&self) -> u32 {
        self.max_rule_graph_nodes.get()
    }
    /// Maximum rule graph edges.
    #[must_use]
    pub const fn max_rule_graph_edges(&self) -> u32 {
        self.max_rule_graph_edges.get()
    }
    /// Maximum rule steps.
    #[must_use]
    pub const fn max_rule_steps(&self) -> u32 {
        self.max_rule_steps.get()
    }
    /// Maximum witness depth.
    #[must_use]
    pub const fn max_witness_depth(&self) -> u32 {
        self.max_witness_depth.get()
    }
    /// Maximum witness count.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses.get()
    }
    /// Maximum diagnostics accumulated for one frontend outcome.
    #[must_use]
    pub const fn max_diagnostics(&self) -> u32 {
        self.max_diagnostics.get()
    }
    /// Maximum serialized response bytes.
    #[must_use]
    pub const fn max_response_bytes(&self) -> u32 {
        self.max_response_bytes.get()
    }
}

/// Complete immutable authority required for closed semantic compilation.
#[derive(Debug)]
pub struct ClosedFrontendAuthorityV2 {
    modules: ValidatedModuleAuthorityPartsV2,
    relations: RelationCatalogV2,
    graph_references: GraphReferenceCatalogV2,
    budget: FrontendBudgetV2,
}

/// Raw host source registration accepted by the curated authority builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendAuthoritySourceV2 {
    /// Stable source identity.
    pub source_id: SourceIdV2,
    /// Exact byte length of the registered host source.
    pub source_len: usize,
}

/// Raw host location checked against the builder's source registry at finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendAuthorityLocationV2 {
    /// Registered host source identity.
    pub source_id: SourceIdV2,
    /// Inclusive byte start.
    pub start: usize,
    /// Exclusive byte end.
    pub end: usize,
}

impl FrontendAuthorityLocationV2 {
    fn check(
        self,
        sources: &SourceRegistryViewV2,
    ) -> Result<CheckedLocatedSpanV2, FrontendAuthorityErrorV2> {
        sources
            .try_check_location(LocatedSpanV2 {
                source_id: self.source_id,
                span: SpanV1::new(self.start, self.end),
            })
            .map_err(|error| configuration_failure(error.code(), error.to_string()))
    }
}

/// Host-agnostic scalar field vocabulary admitted by relation authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendScalarTypeV2 {
    Bool,
    Int,
    Float,
    String,
    Duration,
    Location,
    Path,
    RepoPath,
    EntityRef,
    SymbolRef,
    RuleRef,
    SourceScope,
    ProofTree,
}

impl FrontendScalarTypeV2 {
    const fn owner(self) -> ScalarTyV1 {
        match self {
            Self::Bool => ScalarTyV1::Bool,
            Self::Int => ScalarTyV1::Int,
            Self::Float => ScalarTyV1::Float,
            Self::String => ScalarTyV1::String,
            Self::Duration => ScalarTyV1::Duration,
            Self::Location => ScalarTyV1::Location,
            Self::Path => ScalarTyV1::Path,
            Self::RepoPath => ScalarTyV1::RepoPath,
            Self::EntityRef => ScalarTyV1::EntityRef,
            Self::SymbolRef => ScalarTyV1::SymbolRef,
            Self::RuleRef => ScalarTyV1::RuleRef,
            Self::SourceScope => ScalarTyV1::SourceScope,
            Self::ProofTree => ScalarTyV1::ProofTree,
        }
    }
}

/// Relation multiplicity admitted by the curated authority builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendRelationCardinalityV2 {
    Set,
    Bag,
    Sequence,
}

/// Closed relation provenance admitted at the public frontend boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendRelationAuthorityV2 {
    /// An application-hosted relation with host-defined identity and schema.
    Host,
    /// The canonical workspace-file inventory relation.
    CanonicalWorkspaceFiles,
}

impl FrontendRelationAuthorityV2 {
    const fn owner(self) -> RelationAuthorityV2 {
        match self {
            Self::Host => RelationAuthorityV2::Host,
            Self::CanonicalWorkspaceFiles => RelationAuthorityV2::CanonicalWorkspaceFiles,
        }
    }
}

impl FrontendRelationCardinalityV2 {
    const fn owner(self) -> CardinalityV1 {
        match self {
            Self::Set => CardinalityV1::Set,
            Self::Bag => CardinalityV1::Bag,
            Self::Sequence => CardinalityV1::Sequence,
        }
    }
}

/// One relation field supplied through the public authority boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendRelationFieldInputV2 {
    pub name: String,
    pub scalar_type: FrontendScalarTypeV2,
    pub optional: bool,
    pub location: FrontendAuthorityLocationV2,
}

/// One host relation supplied through the public authority boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendRelationInputV2 {
    pub authority: FrontendRelationAuthorityV2,
    pub provenance_namespace: String,
    pub qualified_name: String,
    pub lookup_name: String,
    pub fields: Vec<FrontendRelationFieldInputV2>,
    pub cardinality: FrontendRelationCardinalityV2,
    pub provenance: String,
    pub location: FrontendAuthorityLocationV2,
}

impl FrontendRelationInputV2 {
    /// Construct the exact canonical workspace-file authority contract without
    /// requiring hosts to duplicate or infer its identity and row schema.
    #[must_use]
    pub fn canonical_workspace_files(
        location: FrontendAuthorityLocationV2,
        field_location: FrontendAuthorityLocationV2,
    ) -> Self {
        Self {
            authority: FrontendRelationAuthorityV2::CanonicalWorkspaceFiles,
            provenance_namespace: CANONICAL_WORKSPACE_FILES_PROVENANCE_NAMESPACE_V2.to_owned(),
            qualified_name: CANONICAL_WORKSPACE_FILES_QUALIFIED_NAME_V2.to_owned(),
            lookup_name: CANONICAL_WORKSPACE_FILES_LOOKUP_NAME_V2.to_owned(),
            fields: vec![FrontendRelationFieldInputV2 {
                name: CANONICAL_WORKSPACE_FILES_FIELD_NAME_V2.to_owned(),
                scalar_type: FrontendScalarTypeV2::RepoPath,
                optional: false,
                location: field_location,
            }],
            cardinality: FrontendRelationCardinalityV2::Set,
            provenance: CANONICAL_WORKSPACE_FILES_PROVENANCE_V2.to_owned(),
            location,
        }
    }
}

/// One graph selector supplied through the public authority boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendGraphReferenceInputV2 {
    pub spelling: String,
    pub kind: GraphReferenceKindV2,
    pub entity_subtype: Option<String>,
    pub location: FrontendAuthorityLocationV2,
}

/// Simple host catalog kinds admitted through the public authority builder.
/// Payload-bearing relation/predicate/model contracts remain owner-internal;
/// this vocabulary covers the opaque reference families authored by source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontendCatalogPayloadV2 {
    Query,
    Check,
    Capability,
    ModelSource,
    PackSource,
    Rule {
        parameters: Vec<FrontendCallableParameterInputV2>,
        result: FrontendScalarTypeV2,
    },
}

impl FrontendCatalogPayloadV2 {
    const fn tag(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Check => "check",
            Self::Capability => "capability",
            Self::ModelSource => "model_source",
            Self::PackSource => "pack_source",
            Self::Rule { .. } => "rule",
        }
    }

    fn owner_payload(
        self,
        sources: &SourceRegistryViewV2,
        location: &CheckedLocatedSpanV2,
    ) -> Result<ExternalCatalogPayloadInputV2, FrontendAuthorityErrorV2> {
        match self {
            Self::Query => Ok(ExternalCatalogPayloadInputV2::Query),
            Self::Check => Ok(ExternalCatalogPayloadInputV2::Check),
            Self::Capability => Ok(ExternalCatalogPayloadInputV2::Capability),
            Self::ModelSource => Ok(ExternalCatalogPayloadInputV2::ModelSource),
            Self::PackSource => Ok(ExternalCatalogPayloadInputV2::PackSource),
            Self::Rule { parameters, result } => {
                Ok(ExternalCatalogPayloadInputV2::Rule(SignatureInputV2 {
                    parameters: parameters
                        .into_iter()
                        .map(|parameter| {
                            let mut value_type = ValueTypeV1::scalar(parameter.scalar_type.owner());
                            if parameter.optional {
                                value_type =
                                    ValueTypeV1::try_optional(value_type).map_err(|error| {
                                        configuration_failure(
                                            "sema.invalid_catalog_signature",
                                            error.to_string(),
                                        )
                                    })?;
                            }
                            Ok(LocatedParameterInputV2 {
                                name: parameter.name,
                                value_type,
                                location: parameter.location.check(sources)?,
                            })
                        })
                        .collect::<Result<Vec<_>, FrontendAuthorityErrorV2>>()?,
                    result: ValueTypeV1::scalar(result.owner()),
                    callable_kind: CallableKindV2::Rule,
                    effects: ExpressionEffectsV2::pure_scalar(),
                    location: location.clone(),
                }))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendCallableParameterInputV2 {
    pub name: String,
    pub scalar_type: FrontendScalarTypeV2,
    pub optional: bool,
    pub location: FrontendAuthorityLocationV2,
}

/// One opaque external catalog asset supplied through the public authority
/// transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendCatalogAssetInputV2 {
    pub module_name: String,
    pub local_name: String,
    pub payload: FrontendCatalogPayloadV2,
    pub provenance: String,
    pub location: FrontendAuthorityLocationV2,
}

/// Semantic language version selected by the host authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendLanguageVersionV2 {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

/// One dependency-free pack manifest supplied through the public authority
/// boundary. Dependency/export/suite authoring remains an owner-level concern;
/// this DTO is the minimal closed input required for source-level `USE PACK`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendPackInputV2 {
    pub pack_id: String,
    pub version: FrontendLanguageVersionV2,
    pub content_digest: String,
    pub language_lower: FrontendLanguageVersionV2,
    pub language_upper: FrontendLanguageVersionV2,
    pub location: FrontendAuthorityLocationV2,
}

/// One canonical lockfile and the checked host declaration that names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendLockfileInputV2 {
    pub path: String,
    pub source_id: SourceIdV2,
    pub text: String,
    pub location: FrontendAuthorityLocationV2,
}

/// Curated external transaction for constructing closed frontend authority.
///
/// It accepts only host-agnostic declarations. Owner proofs and checked
/// locations are minted internally in one registry/budget session and cannot
/// be supplied or forged by an external caller.
pub struct ClosedFrontendAuthorityBuilderV2 {
    budget: FrontendBudgetV2,
    sources: Vec<FrontendAuthoritySourceV2>,
    current_module: String,
    current_module_location: FrontendAuthorityLocationV2,
    language_version: FrontendLanguageVersionV2,
    packs: Vec<FrontendPackInputV2>,
    lockfiles: Vec<FrontendLockfileInputV2>,
    relations: Vec<FrontendRelationInputV2>,
    graph_references: Vec<FrontendGraphReferenceInputV2>,
    catalog_assets: Vec<FrontendCatalogAssetInputV2>,
}

impl ClosedFrontendAuthorityBuilderV2 {
    /// Start an explicit authority transaction. No empty/default source or
    /// module identity is invented.
    pub fn try_new(
        budget: FrontendBudgetV2,
        sources: Vec<FrontendAuthoritySourceV2>,
        current_module: String,
        current_module_location: FrontendAuthorityLocationV2,
        language_version: FrontendLanguageVersionV2,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        if sources.is_empty() {
            return Err(configuration_failure(
                "sema.empty_authority_sources",
                "closed authority requires at least one registered host source",
            ));
        }
        if current_module.is_empty() {
            return Err(configuration_failure(
                "sema.invalid_current_module",
                "current module cannot be empty",
            ));
        }
        Ok(Self {
            budget,
            sources,
            current_module,
            current_module_location,
            language_version,
            packs: Vec::new(),
            lockfiles: Vec::new(),
            relations: Vec::new(),
            graph_references: Vec::new(),
            catalog_assets: Vec::new(),
        })
    }

    /// Add one relation declaration. Validation remains atomic until finish.
    pub fn add_relation(&mut self, relation: FrontendRelationInputV2) {
        self.relations.push(relation);
    }

    /// Add one graph selector declaration. Validation remains atomic until finish.
    pub fn add_graph_reference(&mut self, reference: FrontendGraphReferenceInputV2) {
        self.graph_references.push(reference);
    }

    /// Add one opaque external catalog asset. Validation remains atomic until
    /// `finish`; no caller can mint owner proof types.
    pub fn add_catalog_asset(&mut self, asset: FrontendCatalogAssetInputV2) {
        self.catalog_assets.push(asset);
    }

    /// Add one dependency-free pack manifest. Validation remains atomic until finish.
    pub fn add_pack(&mut self, pack: FrontendPackInputV2) {
        self.packs.push(pack);
    }

    /// Add one canonical lockfile. Exactly one is required for a nonempty pack graph.
    pub fn add_lockfile(&mut self, lockfile: FrontendLockfileInputV2) {
        self.lockfiles.push(lockfile);
    }

    /// Validate every input and seal one closed authority.
    pub fn finish(self) -> Result<ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2> {
        let host_sources = SourceRegistryViewV2::try_new(
            self.sources
                .into_iter()
                .map(|source| SourceRegistrationV2 {
                    source_id: source.source_id,
                    source_len: source.source_len,
                })
                .collect(),
        )
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let module_location = self.current_module_location.check(&host_sources)?;
        let profile = self.budget.authority_profile()?;
        let module = ModuleIdV2::try_new(&self.current_module, module_location.clone())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host_sources.clone())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        modules
            .try_add_module(ModuleNodeInputV2 {
                module_id: module.clone(),
                location: module_location.clone(),
            })
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        for pack in self.packs {
            let location = pack.location.check(&host_sources)?;
            let pack_id = PackIdV2::try_new(&pack.pack_id, location.clone())
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
            let content_digest = ContentDigestV2::try_parse(&pack.content_digest, location.clone())
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
            let language_range = VersionRangeV2::try_new(
                pack.language_lower.owner(),
                pack.language_upper.owner(),
                location.clone(),
            )
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
            modules
                .try_add_manifest(PackManifestInputV2 {
                    pack_id,
                    version: pack.version.owner(),
                    content_digest,
                    language_range,
                    dependencies: Vec::new(),
                    exports: Vec::new(),
                    suites: Vec::new(),
                    location,
                })
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        }
        for lockfile in self.lockfiles {
            modules
                .try_set_lockfile(
                    LockReferenceInputV2 {
                        path: lockfile.path,
                        location: lockfile.location.check(&host_sources)?,
                    },
                    lockfile.source_id,
                    &lockfile.text,
                )
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        }
        for asset in self.catalog_assets {
            let location = asset.location.check(&host_sources)?;
            let kind_token = asset.payload.tag().to_owned();
            let typed_payload = asset.payload.owner_payload(&host_sources, &location)?;
            modules
                .try_add_external(ExternalCatalogAssetInputV2 {
                    module_name: asset.module_name,
                    local_name: asset.local_name,
                    kind_token,
                    typed_payload,
                    provenance: asset.provenance,
                    location,
                    dependencies: Vec::new(),
                })
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        }
        let modules = modules
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: module,
                    location: module_location,
                },
                VersionV2::new(
                    self.language_version.major,
                    self.language_version.minor,
                    self.language_version.patch,
                ),
            )
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;

        let mut relation_inputs = Vec::with_capacity(self.relations.len());
        for relation in self.relations {
            let location = relation.location.check(&host_sources)?;
            let namespace =
                ProvenanceNamespaceV2::try_new(&relation.provenance_namespace, location.clone())
                    .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
            let mut fields = Vec::with_capacity(relation.fields.len());
            for field in relation.fields {
                let field_location = field.location.check(&host_sources)?;
                let mut value_type = ValueTypeV1::scalar(field.scalar_type.owner());
                if field.optional {
                    value_type = ValueTypeV1::try_optional(value_type)
                        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
                }
                fields.push(
                    RecordFieldTypeV1::try_new(&field.name, value_type, field_location)
                        .map_err(|error| configuration_failure(error.code(), error.to_string()))?,
                );
            }
            let row = RecordTypeV1::try_new(fields)
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
            relation_inputs.push(RelationSourceInputV2 {
                authority: relation.authority.owner(),
                provenance_namespace: namespace,
                qualified_name: relation.qualified_name,
                lookup_name: relation.lookup_name,
                row,
                cardinality: relation.cardinality.owner(),
                provenance: relation.provenance,
                location,
            });
        }
        let relations = RelationCatalogV2::try_new(relation_inputs, &host_sources, profile.clone())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;

        let graph_inputs = self
            .graph_references
            .into_iter()
            .map(|reference| {
                Ok(GraphReferenceInputV2 {
                    spelling: reference.spelling,
                    kind: reference.kind,
                    entity_subtype: reference.entity_subtype,
                    location: reference.location.check(&host_sources)?,
                })
            })
            .collect::<Result<Vec<_>, FrontendAuthorityErrorV2>>()?;
        let graph_references =
            GraphReferenceCatalogV2::try_new(graph_inputs, &host_sources, profile)?;
        ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, self.budget)
    }
}

impl FrontendLanguageVersionV2 {
    const fn owner(self) -> VersionV2 {
        VersionV2::new(self.major, self.minor, self.patch)
    }
}

impl ClosedFrontendAuthorityV2 {
    /// Seal authority proofs only when registry identity and budget profiles
    /// are exactly the ones selected by the frontend budget.
    pub fn try_new(
        modules: ValidatedModuleAuthorityPartsV2,
        relations: RelationCatalogV2,
        graph_references: GraphReferenceCatalogV2,
        budget: FrontendBudgetV2,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        let projected = budget.authority_profile()?;
        if modules.source_registry().identity() != *relations.source_registry_identity() {
            return Err(configuration_failure(
                "sema.source_registry_mismatch",
                "module and relation authority use different source registries",
            ));
        }
        if modules.source_registry().identity() != *graph_references.source_registry_identity() {
            return Err(configuration_failure(
                "sema.source_registry_mismatch",
                "graph references and modules use different host source registries",
            ));
        }
        if modules.budget_profile() != &projected
            || relations.budget_profile() != &projected
            || graph_references.budget_profile() != &projected
        {
            return Err(configuration_failure(
                "sema.authority_budget_mismatch",
                "authority proof budget differs from frontend budget projection",
            ));
        }
        Ok(Self {
            modules,
            relations,
            graph_references,
            budget,
        })
    }

    /// Sealed module authority.
    #[must_use]
    pub const fn modules(&self) -> &ValidatedModuleAuthorityPartsV2 {
        &self.modules
    }
    /// Sealed relation authority.
    #[must_use]
    pub const fn relations(&self) -> &RelationCatalogV2 {
        &self.relations
    }
    /// Host-admitted graph selector authority.
    #[must_use]
    pub const fn graph_references(&self) -> &GraphReferenceCatalogV2 {
        &self.graph_references
    }
    /// Frontend budget.
    #[must_use]
    pub const fn budget(&self) -> &FrontendBudgetV2 {
        &self.budget
    }
    pub(crate) fn try_extend_document(
        &self,
        source_id: SourceIdV2,
        source_len: usize,
    ) -> Result<ExtendedSourceRegistryViewV2, quanta_gqlang_v2_hir::SourceRegistryDiagnosticV2>
    {
        self.modules
            .source_registry()
            .try_extend_document(SourceRegistrationV2 {
                source_id,
                source_len,
            })
    }
}

/// Explicit inputs for open editor analysis. Missing authority stays missing.
#[derive(Debug)]
pub struct OpenAnalysisAuthorityInputV2 {
    optional_modules: Option<ValidatedModuleAuthorityPartsV2>,
    optional_relations: Option<RelationCatalogV2>,
    optional_graph_references: Option<GraphReferenceCatalogV2>,
    host_sources: SourceRegistryViewV2,
    budget: FrontendBudgetV2,
}

impl OpenAnalysisAuthorityInputV2 {
    /// Construct the explicit open-authority input carrier.
    #[must_use]
    pub fn new(
        optional_modules: Option<ValidatedModuleAuthorityPartsV2>,
        optional_relations: Option<RelationCatalogV2>,
        optional_graph_references: Option<GraphReferenceCatalogV2>,
        host_sources: SourceRegistryViewV2,
        budget: FrontendBudgetV2,
    ) -> Self {
        Self {
            optional_modules,
            optional_relations,
            optional_graph_references,
            host_sources,
            budget,
        }
    }
}

/// Checked authority for editor analysis. Absence is distinct from an empty
/// closed proof.
#[derive(Debug)]
pub struct OpenAnalysisAuthorityV2 {
    modules: Option<ValidatedModuleAuthorityPartsV2>,
    relations: Option<RelationCatalogV2>,
    graph_references: Option<GraphReferenceCatalogV2>,
    host_sources: SourceRegistryViewV2,
    budget: FrontendBudgetV2,
}

impl OpenAnalysisAuthorityV2 {
    /// Validate every supplied proof against the explicit host registry and
    /// canonical budget projection.
    pub fn try_new(input: OpenAnalysisAuthorityInputV2) -> Result<Self, FrontendAuthorityErrorV2> {
        let projected = input.budget.authority_profile()?;
        if input.optional_modules.as_ref().is_some_and(|modules| {
            modules.source_registry().identity() != input.host_sources.identity()
                || modules.budget_profile() != &projected
        }) || input.optional_relations.as_ref().is_some_and(|relations| {
            relations.source_registry_identity() != &input.host_sources.identity()
                || relations.budget_profile() != &projected
        }) || input
            .optional_graph_references
            .as_ref()
            .is_some_and(|graph| {
                graph.source_registry_identity() != &input.host_sources.identity()
                    || graph.budget_profile() != &projected
            })
        {
            return Err(configuration_failure(
                "sema.open_authority_mismatch",
                "open authority proof does not match the host registry and budget",
            ));
        }
        Ok(Self {
            modules: input.optional_modules,
            relations: input.optional_relations,
            graph_references: input.optional_graph_references,
            host_sources: input.host_sources,
            budget: input.budget,
        })
    }

    /// Construct the standard open editor authority without inventing module,
    /// relation, catalog, pack, or lock authority.
    pub fn try_standard_v2() -> Result<Self, FrontendAuthorityErrorV2> {
        Self::try_empty_v2(FrontendBudgetV2::standard_v2()?)
    }

    /// Construct an open editor authority with an explicit checked budget and
    /// no fabricated host authority.
    pub fn try_empty_v2(budget: FrontendBudgetV2) -> Result<Self, FrontendAuthorityErrorV2> {
        let _kernel = ExpressionTypeKernelV2::try_closed()
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let host_sources = SourceRegistryViewV2::try_new(Vec::new())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        Self::try_new(OpenAnalysisAuthorityInputV2::new(
            None,
            None,
            None,
            host_sources,
            budget,
        ))
    }

    /// Optional module authority.
    #[must_use]
    pub const fn modules(&self) -> Option<&ValidatedModuleAuthorityPartsV2> {
        self.modules.as_ref()
    }
    /// Optional relation catalog.
    #[must_use]
    pub const fn relations(&self) -> Option<&RelationCatalogV2> {
        self.relations.as_ref()
    }
    /// Optional host graph selectors.
    #[must_use]
    pub const fn graph_references(&self) -> Option<&GraphReferenceCatalogV2> {
        self.graph_references.as_ref()
    }
    /// Checked host sources.
    #[must_use]
    pub const fn host_sources(&self) -> &SourceRegistryViewV2 {
        &self.host_sources
    }
    /// Frontend budget.
    #[must_use]
    pub const fn budget(&self) -> &FrontendBudgetV2 {
        &self.budget
    }
}

fn configuration_failure(code: &'static str, message: impl AsRef<str>) -> FrontendAuthorityErrorV2 {
    let value = message.as_ref();
    let mut end = value.len().min(1_024);
    while !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    FrontendAuthorityErrorV2 {
        code,
        message: value[..end].to_owned(),
        primary: None,
        secondary: None,
    }
}

fn authority_failure_at(
    code: &'static str,
    message: &'static str,
    primary: LocatedSpanV2,
    secondary: Option<LocatedSpanV2>,
) -> FrontendAuthorityErrorV2 {
    FrontendAuthorityErrorV2 {
        code,
        message: message.to_owned(),
        primary: Some(primary),
        secondary,
    }
}
