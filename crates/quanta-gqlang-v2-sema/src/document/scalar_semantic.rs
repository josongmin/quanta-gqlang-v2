//! Opaque request-owned scalar semantic item for downstream read-only lowering.

use super::{
    AdmittedSourceInlineViewBodyV3, AdmittedSourceOneLetBodyV3, AdmittedSourceViewTopologyV3,
    CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, FrontendRequestMaterializationRootV3,
    FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3, OutputColumnContractV2,
    ProgramItemHirV1, RelationOutputContractV2, ResolvedScanSourceV2, ResolvedValueBindingV2,
    ResolvingDocumentSessionV3, SemanticItemIdV2, SourceIdV2, SourceQualifiedNodeIdV2, TypedExprV2,
    TypedRelationExprV2, TypedRelationV2,
};
use quanta_gqlang_v2_modules::{
    AdmittedDocumentCatalogAuthorityV3, DocumentCatalogFinalInputV3, DocumentCatalogRequestErrorV3,
};

/// One admitted checked scalar item and the final catalog from its Modules
/// session. All cloneable semantic and catalog values remain private.
pub struct AdmittedScalarSemanticDocumentV3<'authority> {
    body: AdmittedSourceInlineViewBodyV3<'authority>,
    catalog: AdmittedDocumentCatalogAuthorityV3,
    kind: AdmittedScalarSemanticKindV3,
    authored_item_id: SemanticItemIdV2,
    authored_item_location: CheckedLocatedSpanV2,
}

/// One checked LET and its two retained Query relations under the issuing
/// catalog session. No cloneable ordinary semantic document escapes.
pub struct AdmittedOneLetSemanticDocumentV3<'authority> {
    body: AdmittedSourceOneLetBodyV3<'authority>,
    catalog: AdmittedDocumentCatalogAuthorityV3,
    authored_item_id: SemanticItemIdV2,
    authored_item_location: CheckedLocatedSpanV2,
    binding_name_location: CheckedLocatedSpanV2,
}

impl AdmittedOneLetSemanticDocumentV3<'_> {
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        let source_id = self.body.source_id_v3();
        self.body
            ._storage
            .validate_owner_v3(root, source_id, source)?;
        self.body
            ._types
            .facts
            .validate_owner_v3(root, source_id, source)?;
        self.catalog.validate_owner_v3(root, source_id, source)
    }

    #[must_use]
    pub fn authored_item_id_v3(&self) -> SemanticItemIdV2 {
        self.authored_item_id
    }

    #[must_use]
    pub fn authored_item_location_v3(&self) -> &CheckedLocatedSpanV2 {
        &self.authored_item_location
    }

    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        self.body.body.output().columns().len()
    }

    #[must_use]
    pub fn expression_count_v3(&self) -> usize {
        self.body.producer_expressions.len() + self.body.body_expressions.len()
    }

    #[must_use]
    pub fn producer_item_id_v3(&self) -> SemanticItemIdV2 {
        self.body.producer.item_id()
    }

    #[must_use]
    pub fn producer_output_v3(&self) -> &RelationOutputContractV2 {
        self.body.producer.output()
    }

    #[must_use]
    pub fn binding_name_location_v3(&self) -> &CheckedLocatedSpanV2 {
        &self.binding_name_location
    }

    #[must_use]
    pub fn body_item_id_v3(&self) -> SemanticItemIdV2 {
        self.body.body.item_id()
    }

    #[must_use]
    pub fn with_binding_v3(&self) -> Option<&ResolvedValueBindingV2> {
        let mut inputs = self.body.body.root().inputs();
        let source = inputs.next()?.source()?;
        if inputs.next().is_some() {
            return None;
        }
        let ResolvedScanSourceV2::Composition(local) = source else {
            return None;
        };
        Some(local.binding())
    }
}

/// The source owner is shape-specific; both shapes retain their complete
/// producer and catalog custody until the next stage consumes this enum.
pub enum AdmittedSourceSemanticDocumentV3<'authority> {
    Scalar(AdmittedScalarSemanticDocumentV3<'authority>),
    OneLet(AdmittedOneLetSemanticDocumentV3<'authority>),
}

impl AdmittedSourceSemanticDocumentV3<'_> {
    #[must_use]
    pub fn authored_item_id_v3(&self) -> SemanticItemIdV2 {
        match self {
            Self::Scalar(value) => value.authored_item_id_v3(),
            Self::OneLet(value) => value.authored_item_id_v3(),
        }
    }

    #[must_use]
    pub fn kind_v3(&self) -> AdmittedScalarSemanticKindV3 {
        match self {
            Self::Scalar(value) => value.kind_v3(),
            Self::OneLet(_) => AdmittedScalarSemanticKindV3::OneLetComposition,
        }
    }

    #[must_use]
    pub fn catalog_asset_count_v3(&self) -> usize {
        match self {
            Self::Scalar(value) => value.catalog_asset_count_v3(),
            Self::OneLet(_) => 0,
        }
    }

    #[must_use]
    pub fn catalog_dependency_count_v3(&self) -> usize {
        match self {
            Self::Scalar(value) => value.catalog_dependency_count_v3(),
            Self::OneLet(_) => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmittedScalarSemanticKindV3 {
    Query,
    InlineView,
    OneLetComposition,
}

#[derive(Debug)]
pub enum ScalarSemanticAssemblyErrorV3 {
    Storage(FrontendStorageRefusalV3),
    ForeignCatalogSession,
    Membership,
}

#[derive(Debug)]
pub enum SourceCatalogSealErrorV3 {
    Storage(FrontendStorageRefusalV3),
    Modules(DocumentCatalogRequestErrorV3),
    Semantic(ScalarSemanticAssemblyErrorV3),
}

/// Immutable checked Scan and Project observations. This carries no permit
/// separately from its owner and exposes no detachable raw semantic document.
pub struct ScalarSourceProjectReadV3<'read, 'authority> {
    relation: &'read TypedRelationV2,
    item_id: SemanticItemIdV2,
    expressions: &'read [(SourceQualifiedNodeIdV2, TypedExprV2)],
    authored_item_location: &'read CheckedLocatedSpanV2,
    budget_receipt: DownstreamBudgetReceiptV2,
    scan: &'read TypedRelationExprV2,
    project: &'read TypedRelationExprV2,
    _authority: std::marker::PhantomData<&'authority ()>,
}

impl ScalarSourceProjectReadV3<'_, '_> {
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    #[must_use]
    pub fn authored_item_location_v3(&self) -> &CheckedLocatedSpanV2 {
        self.authored_item_location
    }

    #[must_use]
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self.relation.item_id().source_id()
    }

    #[must_use]
    pub fn budget_receipt_v3(&self) -> DownstreamBudgetReceiptV2 {
        self.budget_receipt
    }

    #[must_use]
    pub fn scan_key_v3(&self) -> SourceQualifiedNodeIdV2 {
        self.scan.key()
    }

    #[must_use]
    pub fn project_key_v3(&self) -> SourceQualifiedNodeIdV2 {
        self.project.key()
    }

    #[must_use]
    pub fn scan_location_v3(&self) -> &CheckedLocatedSpanV2 {
        self.scan.location()
    }

    #[must_use]
    pub fn project_location_v3(&self) -> &CheckedLocatedSpanV2 {
        self.project.location()
    }

    #[must_use]
    pub fn scan_output_v3(&self) -> &RelationOutputContractV2 {
        self.scan.output()
    }

    #[must_use]
    pub fn project_output_v3(&self) -> &RelationOutputContractV2 {
        self.project.output()
    }

    #[must_use]
    pub fn scan_source_v3(&self) -> Option<&ResolvedScanSourceV2> {
        self.scan.source()
    }

    pub fn project_expression_keys_v3(
        &self,
    ) -> impl ExactSizeIterator<Item = SourceQualifiedNodeIdV2> + '_ {
        self.project.expressions().iter().copied()
    }

    #[must_use]
    pub fn expression_v3(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExprV2> {
        self.expressions
            .binary_search_by_key(&key, |(candidate, _value)| *candidate)
            .ok()
            .and_then(|index| self.expressions.get(index))
            .map(|(_key, value)| value)
    }
}

fn source_project_read_v3<'read, 'authority>(
    relation: &'read TypedRelationV2,
    item_id: SemanticItemIdV2,
    expressions: &'read [(SourceQualifiedNodeIdV2, TypedExprV2)],
    authored_item_location: &'read CheckedLocatedSpanV2,
    budget_receipt: DownstreamBudgetReceiptV2,
) -> Option<ScalarSourceProjectReadV3<'read, 'authority>> {
    let project = relation.root();
    if !matches!(project, TypedRelationExprV2::Project(_)) {
        return None;
    }
    let mut inputs = project.inputs();
    let scan = inputs.next()?;
    if inputs.next().is_some() || !matches!(scan, TypedRelationExprV2::Scan(_)) {
        return None;
    }
    Some(ScalarSourceProjectReadV3 {
        relation,
        item_id,
        expressions,
        authored_item_location,
        budget_receipt,
        scan,
        project,
        _authority: std::marker::PhantomData,
    })
}

impl AdmittedOneLetSemanticDocumentV3<'_> {
    #[must_use]
    pub fn source_project_reads_v3(&self) -> Option<[ScalarSourceProjectReadV3<'_, '_>; 2]> {
        let receipt = self.body._types._authority.budget().downstream_receipt();
        Some([
            source_project_read_v3(
                &self.body.producer,
                self.body.producer.item_id(),
                &self.body.producer_expressions,
                &self.authored_item_location,
                receipt,
            )?,
            source_project_read_v3(
                &self.body.body,
                self.body.body.item_id(),
                &self.body.body_expressions,
                &self.authored_item_location,
                receipt,
            )?,
        ])
    }
}

impl AdmittedScalarSemanticDocumentV3<'_> {
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        let source_id = self.body.source_id_v3();
        self.body
            ._storage
            .validate_owner_v3(root, source_id, source)?;
        self.body
            ._types
            .facts
            .validate_owner_v3(root, source_id, source)?;
        self.catalog.validate_owner_v3(root, source_id, source)
    }

    #[must_use]
    pub fn scalar_source_project_read_v3(&self) -> Option<ScalarSourceProjectReadV3<'_, '_>> {
        source_project_read_v3(
            &self.body.relation,
            self.authored_item_id,
            &self.body.expressions,
            &self.authored_item_location,
            self.body._types._authority.budget().downstream_receipt(),
        )
    }

    #[must_use]
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self.body.source_id_v3()
    }

    #[must_use]
    pub const fn kind_v3(&self) -> AdmittedScalarSemanticKindV3 {
        self.kind
    }

    #[must_use]
    pub const fn authored_item_id_v3(&self) -> SemanticItemIdV2 {
        self.authored_item_id
    }

    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        self.body.output_field_count_v3()
    }

    #[must_use]
    pub fn expression_count_v3(&self) -> usize {
        self.body.expression_count_v3()
    }

    #[must_use]
    pub fn catalog_asset_count_v3(&self) -> usize {
        self.catalog.asset_count()
    }

    #[must_use]
    pub fn catalog_dependency_count_v3(&self) -> usize {
        self.catalog.dependency_count()
    }

    #[must_use]
    pub fn output_columns_v3(&self) -> &[OutputColumnContractV2] {
        self.body.output_columns_v3()
    }
}

/// Join body, authored item, and final catalog only when all three belong to
/// the same source and Modules session. The Vec is sorted at body seal, so
/// exact expression membership needs no temporary tree or scratch allocation.
pub(super) fn assemble_scalar_semantic_with_request_owner_v3<'authority>(
    body: AdmittedSourceInlineViewBodyV3<'authority>,
    catalog: AdmittedDocumentCatalogAuthorityV3,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
) -> Result<AdmittedScalarSemanticDocumentV3<'authority>, ScalarSemanticAssemblyErrorV3> {
    let source_id = body.source_id_v3();
    body._storage
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarSemanticAssemblyErrorV3::Storage)?;
    body._types
        .facts
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarSemanticAssemblyErrorV3::Storage)?;
    catalog
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarSemanticAssemblyErrorV3::Storage)?;
    if !body.catalog_session.matches_catalog_v3(&catalog) {
        return Err(ScalarSemanticAssemblyErrorV3::ForeignCatalogSession);
    }
    let declared = &body._types._declared;
    if !catalog.has_source_registry_v3(&declared.registry_identity)
        || declared.checked().source_len() != source.len()
        || body.expressions.len() != declared.expected_expressions.len()
        || !body
            .expressions
            .iter()
            .map(|(key, _value)| key)
            .eq(declared.expected_expressions.iter())
    {
        return Err(ScalarSemanticAssemblyErrorV3::Membership);
    }
    let [authored_item_id] = declared.order.as_slice() else {
        return Err(ScalarSemanticAssemblyErrorV3::Membership);
    };
    let authored_item_id = *authored_item_id;
    let (kind, item_span) = match declared.checked().checked().document().items.as_slice() {
        [ProgramItemHirV1::Query(query)]
            if catalog.asset_count() == 0
                && authored_item_id == body.relation.item_id()
                && body.relation.root().key().node_id == query.body.id =>
        {
            (AdmittedScalarSemanticKindV3::Query, query.span)
        }
        [ProgramItemHirV1::CatalogDecl(view)]
            if catalog.asset_count() == 1
                && authored_item_id == SemanticItemIdV2::new(source_id, view.id)
                && matches!(
                    &view.payload,
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::Inline(query)
                    ) if body.relation.item_id() == SemanticItemIdV2::new(source_id, query.id)
                        && body.relation.root().key().node_id == query.body.id
                ) =>
        {
            (AdmittedScalarSemanticKindV3::InlineView, view.span)
        }
        _ => return Err(ScalarSemanticAssemblyErrorV3::Membership),
    };
    let authored_item_location = body
        ._types
        ._extended
        .combined()
        .try_check_location(super::location(source_id, item_span))
        .map_err(|_error| ScalarSemanticAssemblyErrorV3::Membership)?;
    Ok(AdmittedScalarSemanticDocumentV3 {
        body,
        catalog,
        kind,
        authored_item_id,
        authored_item_location,
    })
}

/// Finish the one-item catalog in its original Modules session and atomically
/// bind it to the body. No raw Modules input or catalog escapes to pipeline.
pub fn seal_source_scalar_catalog_with_request_owner_v3<'authority>(
    mut session: ResolvingDocumentSessionV3<'authority>,
    topology: AdmittedSourceViewTopologyV3,
    body: AdmittedSourceInlineViewBodyV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedScalarSemanticDocumentV3<'authority>, SourceCatalogSealErrorV3> {
    let source_id = body.source_id_v3();
    parent
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    body._storage
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    body._types
        .facts
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    topology
        .storage
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    session
        .validator
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    if !std::ptr::eq(body._types._authority, session.authority) {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::Membership,
        ));
    }
    if !body.catalog_session.matches_topology_v3(&topology.proof) {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::ForeignCatalogSession,
        ));
    }
    let catalog = session
        .validator
        .validate_predeclared_with_request_owner_v3(
            topology.proof,
            DocumentCatalogFinalInputV3 {
                profile_admissions: Vec::new(),
                activations: Vec::new(),
                require_certified_directives: Vec::new(),
                family_references: Vec::new(),
            },
            root,
            source_id,
            source,
            parent,
        )
        .map_err(SourceCatalogSealErrorV3::Modules)?;
    assemble_scalar_semantic_with_request_owner_v3(body, catalog, root, source)
        .map_err(SourceCatalogSealErrorV3::Semantic)
}

/// Seal the same Modules session for the one-LET body. Both expression tables
/// must exactly cover the declared membership before Algebra may observe them.
pub fn seal_source_one_let_catalog_with_request_owner_v3<'authority>(
    mut session: ResolvingDocumentSessionV3<'authority>,
    topology: AdmittedSourceViewTopologyV3,
    body: AdmittedSourceOneLetBodyV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedOneLetSemanticDocumentV3<'authority>, SourceCatalogSealErrorV3> {
    let source_id = body.source_id_v3();
    for permit in [parent, &body._storage, &topology.storage] {
        permit
            .validate_owner_v3(root, source_id, source)
            .map_err(SourceCatalogSealErrorV3::Storage)?;
    }
    body._types
        .facts
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    session
        .validator
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    if !std::ptr::eq(body._types._authority, session.authority)
        || !body.catalog_session.matches_topology_v3(&topology.proof)
    {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::ForeignCatalogSession,
        ));
    }
    let catalog = session
        .validator
        .validate_predeclared_with_request_owner_v3(
            topology.proof,
            DocumentCatalogFinalInputV3 {
                profile_admissions: Vec::new(),
                activations: Vec::new(),
                require_certified_directives: Vec::new(),
                family_references: Vec::new(),
            },
            root,
            source_id,
            source,
            parent,
        )
        .map_err(SourceCatalogSealErrorV3::Modules)?;
    catalog
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogSealErrorV3::Storage)?;
    let declared = &body._types._declared;
    let [ProgramItemHirV1::Composition(composition)] =
        declared.checked().checked().document().items.as_slice()
    else {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::Membership,
        ));
    };
    let [authored_item_id] = declared.order.as_slice() else {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::Membership,
        ));
    };
    let expression_count = body
        .producer_expressions
        .len()
        .checked_add(body.body_expressions.len())
        .ok_or(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::Membership,
        ))?;
    if catalog.asset_count() != 0
        || !catalog.has_source_registry_v3(&declared.registry_identity)
        || declared.checked().source_len() != source.len()
        || *authored_item_id != SemanticItemIdV2::new(source_id, composition.id)
        || expression_count != declared.expected_expressions.len()
        || declared.expected_expressions.iter().any(|key| {
            let producer = body
                .producer_expressions
                .binary_search_by_key(key, |(found, _)| *found)
                .is_ok();
            let consumer = body
                .body_expressions
                .binary_search_by_key(key, |(found, _)| *found)
                .is_ok();
            producer == consumer
        })
    {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::Membership,
        ));
    }
    let authored_item_location = body
        ._types
        ._extended
        .combined()
        .try_check_location(super::location(source_id, composition.span))
        .map_err(|_error| {
            SourceCatalogSealErrorV3::Semantic(ScalarSemanticAssemblyErrorV3::Membership)
        })?;
    let [authored] = composition.bindings.as_slice() else {
        return Err(SourceCatalogSealErrorV3::Semantic(
            ScalarSemanticAssemblyErrorV3::Membership,
        ));
    };
    let binding_name_location = body
        ._types
        ._extended
        .combined()
        .try_check_location(super::location(source_id, authored.name_span))
        .map_err(|_error| {
            SourceCatalogSealErrorV3::Semantic(ScalarSemanticAssemblyErrorV3::Membership)
        })?;
    let authored_item_id = *authored_item_id;
    Ok(AdmittedOneLetSemanticDocumentV3 {
        body,
        catalog,
        authored_item_id,
        authored_item_location,
        binding_name_location,
    })
}
