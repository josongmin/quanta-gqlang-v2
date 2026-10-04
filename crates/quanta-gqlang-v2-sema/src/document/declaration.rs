//! Checked syntax and source-issued declaration ownership.

use super::{
    check_relation_depth_impl, document_failure, location, CheckedHirDocumentOwnedV2,
    CheckedLocatedSpanV2, ClosedFrontendAuthorityV2, DepthScratchRequestV3,
    DownstreamBudgetReceiptV2, ExtendedSourceRegistryViewV2, FrontendBudgetV2, FrontendFailureV2,
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, HirVisitorV1, ProgramItemHirV1, ResolvedPresentationKindV2,
    ResolvedPresentationV2, SemanticItemIdV2, SemanticItemV2, SourceIdV2, SourceQualifiedNodeIdV2,
    SourceRegistryIdentityV2, SpanV1, ValueExprHirV1,
};

/// Owned proof that all HIR roots share one valid document-wide identity and
/// source-length bound. Fields are private; the HIR owner checks exactly once.
#[derive(Clone, Debug)]
pub struct CheckedSyntaxDocumentV2 {
    pub(super) checked: CheckedHirDocumentOwnedV2,
}

/// Checked declarations collected before any body resolution.
#[derive(Clone, Debug)]
pub struct DeclaredDocumentV2 {
    pub(super) checked: CheckedSyntaxDocumentV2,
    pub(super) source_anchor: CheckedLocatedSpanV2,
    pub(super) expected: Vec<(SemanticItemIdV2, (SpanV1, SemanticItemKindV2))>,
    pub(super) order: Vec<SemanticItemIdV2>,
    pub(super) expected_expressions: Vec<SourceQualifiedNodeIdV2>,
    pub(super) registry_identity: SourceRegistryIdentityV2,
}

/// Typed refusal from the request-owned declaration producer. Its storage
/// variants do not allocate diagnostics after a denied reservation.
#[derive(Debug)]
pub enum DeclaredRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    Registry(quanta_gqlang_v2_hir::SourceRegistryDiagnosticV2),
    DemandOverflow,
    FallibleAllocation,
    SourceMismatch,
    DocumentItemLimit,
    Validation(DeclarationBuildErrorV3),
}

/// Allocation-free structural refusal from declaration construction. The raw
/// entry formats diagnostics only after this typed decision; the request-owned
/// entry returns the decision without an unleased diagnostic payload.
#[derive(Debug)]
pub enum DeclarationBuildErrorV3 {
    SourceMismatch,
    DocumentItemLimit,
    RelationDepthLimit(SpanV1),
    ScratchAllocation,
    ItemCensusExceeded(SpanV1),
    DuplicateItem(SpanV1),
    ExpressionCensusExceeded(SpanV1),
    Anchor(quanta_gqlang_v2_hir::SourceRegistryDiagnosticV2),
}

impl DeclarationBuildErrorV3 {
    fn into_failure(self, source_id: SourceIdV2) -> FrontendFailureV2 {
        match self {
            Self::SourceMismatch => document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "checked syntax source differs from the extended authority registry",
            ),
            Self::DocumentItemLimit => document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                SpanV1::point(0),
                "document_items exceeds frontend limit",
            ),
            Self::RelationDepthLimit(span) => document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                span,
                "relation_depth exceeds frontend limit",
            ),
            Self::ScratchAllocation => document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                SpanV1::point(0),
                "relation depth scratch unavailable",
            ),
            Self::ItemCensusExceeded(span) => document_failure(
                "frontend.request_declaration_storage",
                source_id,
                span,
                "source-issued declaration item census exceeded",
            ),
            Self::DuplicateItem(span) => document_failure(
                "sema.document_validation_failed",
                source_id,
                span,
                "duplicate semantic item identity",
            ),
            Self::ExpressionCensusExceeded(span) => document_failure(
                "frontend.request_declaration_storage",
                source_id,
                span,
                "source-issued expression node census exceeded",
            ),
            Self::Anchor(error) => document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                error.to_string(),
            ),
        }
    }
}

struct DeclarationAdmissionV3<'a, 'root, 'source> {
    root: &'a FrontendRequestMaterializationRootV3<'root>,
    source_id: SourceIdV2,
    source: &'source str,
    permit: &'a mut FrontendRetainedStoragePermitV3,
}

impl DeclarationAdmissionV3<'_, '_, '_> {
    fn reserve_vec<T>(&mut self, slots: usize) -> Result<Vec<T>, DeclaredRequestErrorV3> {
        let bytes = slots
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(DeclaredRequestErrorV3::DemandOverflow)?;
        if bytes > 0 {
            self.permit
                .try_extend_v3(self.root, self.source_id, self.source, bytes)
                .map_err(DeclaredRequestErrorV3::Storage)?;
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(slots)
            .map_err(|_error| DeclaredRequestErrorV3::FallibleAllocation)?;
        if output.capacity() > slots {
            return Err(DeclaredRequestErrorV3::FallibleAllocation);
        }
        Ok(output)
    }
}

/// SHOW-only source declaration before semantic item construction.
///
/// The checked HIR supplies authored order and exact item IDs. The source
/// registry's first document extension retains the host owner and one inline
/// registration, so this stage does not create the generic declaration map or
/// order Vec. It is not a resolved semantic document or a plan.
#[derive(Debug)]
pub struct DeclaredShowSourceV2 {
    checked: CheckedSyntaxDocumentV2,
    extended: ExtendedSourceRegistryViewV2,
    source_anchor: CheckedLocatedSpanV2,
    budget_receipt: DownstreamBudgetReceiptV2,
}

impl DeclaredShowSourceV2 {
    /// Consume checked SHOW syntax and bind it to one exact closed authority.
    /// This standalone constructor has no lexical allocation owner; the
    /// pipeline's consuming wrapper must retain that earlier reservation.
    pub fn try_new(
        checked: CheckedSyntaxDocumentV2,
        authority: &ClosedFrontendAuthorityV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = checked.source_id();
        let source_len = checked.source_len();
        let document = checked.checked().document();
        quanta_gqlang_v2_hir::check_document_show_monotonic_v2(document).map_err(|_error| {
            document_failure(
                "sema.show_source_incomplete",
                source_id,
                SpanV1::point(0),
                "SHOW source does not satisfy the checked monotonic item contract",
            )
        })?;
        if document.items.len()
            > usize::try_from(authority.budget().max_document_items()).unwrap_or(usize::MAX)
        {
            return Err(document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                SpanV1::point(0),
                "document_items exceeds frontend limit",
            ));
        }
        let extended = authority
            .try_extend_document(source_id, source_len)
            .map_err(|error| {
                document_failure(error.code(), source_id, SpanV1::point(0), error.to_string())
            })?;
        if extended.source_len(source_id) != Some(source_len) {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "checked SHOW source differs from the extended authority registry",
            ));
        }
        let source_anchor = extended
            .combined()
            .try_check_location(location(source_id, SpanV1::point(0)))
            .map_err(|error| {
                document_failure(error.code(), source_id, SpanV1::point(0), error.to_string())
            })?;
        Ok(Self {
            checked,
            extended,
            source_anchor,
            budget_receipt: authority.budget().downstream_receipt(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.checked.source_id()
    }

    #[must_use]
    pub fn item_count(&self) -> usize {
        self.checked.checked().document().items.len()
    }

    #[must_use]
    pub fn source_anchor(&self) -> &CheckedLocatedSpanV2 {
        &self.source_anchor
    }

    #[must_use]
    pub fn source_len(&self) -> Option<usize> {
        self.extended.source_len(self.source_id())
    }

    /// Resolve one parser-issued SHOW item without constructing the generic
    /// semantic item map, catalog, or order vector. The checked document owns
    /// the exact item order and the extended registry owns its location proof.
    pub fn resolve_show_item_v2(
        &self,
        index: usize,
    ) -> Result<ResolvedPresentationV2, &'static str> {
        let source_id = self.source_id();
        let Some(ProgramItemHirV1::Presentation(presentation)) =
            self.checked.checked().document().items.get(index)
        else {
            return Err("sema.show_item_absent_or_other_family");
        };
        let quanta_gqlang_v2_hir::PresentationKindV1::Show { category } = &presentation.kind else {
            return Err("sema.presentation_is_not_show");
        };
        let location = self
            .extended
            .combined()
            .try_check_location(location(source_id, presentation.span))
            .map_err(|_error| "sema.show_location_registry_mismatch")?;
        ResolvedPresentationV2::try_new(
            SemanticItemIdV2::new(source_id, presentation.id),
            location,
            ResolvedPresentationKindV2::Show {
                category: *category,
            },
        )
        .map_err(|_error| "sema.invalid_show_item_shape")
    }

    /// The same closed authority budget that admitted the source declaration.
    #[must_use]
    pub const fn budget_receipt_v2(&self) -> DownstreamBudgetReceiptV2 {
        self.budget_receipt
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SemanticItemKindV2 {
    Relation,
    Declaration,
    Rule,
    Graph,
    Check,
    Presentation,
    Catalog,
    Composition,
    Meta,
    Command,
}

impl SemanticItemKindV2 {
    const fn from_hir(item: &ProgramItemHirV1) -> Self {
        match item {
            ProgramItemHirV1::Query(_) => Self::Relation,
            ProgramItemHirV1::Decl(_) => Self::Declaration,
            ProgramItemHirV1::Rule(_) => Self::Rule,
            ProgramItemHirV1::GraphQuery(_) => Self::Graph,
            ProgramItemHirV1::Legality(_) => Self::Check,
            ProgramItemHirV1::Presentation(_) => Self::Presentation,
            ProgramItemHirV1::CatalogDecl(_) => Self::Catalog,
            ProgramItemHirV1::Composition(_) => Self::Composition,
            ProgramItemHirV1::Meta(_) => Self::Meta,
            ProgramItemHirV1::Command(_) => Self::Command,
        }
    }

    pub(super) const fn from_semantic(item: &SemanticItemV2) -> Self {
        match item {
            SemanticItemV2::Relation(_) => Self::Relation,
            SemanticItemV2::Declaration(_) => Self::Declaration,
            SemanticItemV2::Rule(_) => Self::Rule,
            SemanticItemV2::Graph(_) => Self::Graph,
            SemanticItemV2::Check(_) => Self::Check,
            SemanticItemV2::Presentation(_) => Self::Presentation,
            SemanticItemV2::Catalog(_) => Self::Catalog,
            SemanticItemV2::Composition(_) => Self::Composition,
            SemanticItemV2::Meta(_) => Self::Meta,
            SemanticItemV2::Command(_) => Self::Command,
        }
    }
}

impl DeclaredDocumentV2 {
    /// Predeclare all authored items, preserving original order while
    /// rejecting duplicate semantic identities before any semantic output.
    pub fn try_new(
        checked: CheckedSyntaxDocumentV2,
        budget: &FrontendBudgetV2,
        extended: &ExtendedSourceRegistryViewV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = checked.source_id();
        Self::try_new_with_vectors(
            checked,
            budget,
            extended,
            true,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .map_err(|error| error.into_failure(source_id))
    }

    /// Admit source-issued declaration vectors and each live relation-depth
    /// stack on the original request root before allocating that stack.
    pub fn try_new_with_request_owner_v3(
        checked: CheckedSyntaxDocumentV2,
        budget: &FrontendBudgetV2,
        extended: &ExtendedSourceRegistryViewV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
        permit: &mut FrontendRetainedStoragePermitV3,
        issued_node_slots: usize,
    ) -> Result<Self, DeclaredRequestErrorV3> {
        let source_id = checked.source_id();
        permit
            .validate_owner_v3(root, source_id, source)
            .map_err(DeclaredRequestErrorV3::Storage)?;
        if checked.source_len() != source.len()
            || extended.source_len(source_id) != Some(checked.source_len())
        {
            return Err(DeclaredRequestErrorV3::SourceMismatch);
        }
        if checked.checked().document().items.len()
            > usize::try_from(budget.max_document_items()).unwrap_or(usize::MAX)
        {
            return Err(DeclaredRequestErrorV3::DocumentItemLimit);
        }
        check_relation_depth_impl(
            &checked,
            budget,
            Some(DepthScratchRequestV3 {
                root,
                source_id,
                source,
                slots: issued_node_slots,
            }),
        )?;
        let item_slots = checked.checked().document().items.len();
        let mut admission = DeclarationAdmissionV3 {
            root,
            source_id,
            source,
            permit,
        };
        let expected = admission
            .reserve_vec::<(SemanticItemIdV2, (SpanV1, SemanticItemKindV2))>(item_slots)?;
        let order = admission.reserve_vec::<SemanticItemIdV2>(item_slots)?;
        let expression_nodes =
            admission.reserve_vec::<SourceQualifiedNodeIdV2>(issued_node_slots)?;
        Self::try_new_with_vectors(
            checked,
            budget,
            extended,
            false,
            Some(issued_node_slots),
            expected,
            order,
            expression_nodes,
        )
        .map_err(DeclaredRequestErrorV3::Validation)
    }

    fn try_new_with_vectors(
        checked: CheckedSyntaxDocumentV2,
        budget: &FrontendBudgetV2,
        extended: &ExtendedSourceRegistryViewV2,
        check_depth: bool,
        admitted_node_slots: Option<usize>,
        mut expected: Vec<(SemanticItemIdV2, (SpanV1, SemanticItemKindV2))>,
        mut order: Vec<SemanticItemIdV2>,
        expression_nodes: Vec<SourceQualifiedNodeIdV2>,
    ) -> Result<Self, DeclarationBuildErrorV3> {
        let source_id = checked.source_id();
        if extended.source_len(source_id) != Some(checked.source_len()) {
            return Err(DeclarationBuildErrorV3::SourceMismatch);
        }
        if checked.checked.document().items.len()
            > usize::try_from(budget.max_document_items()).unwrap_or(usize::MAX)
        {
            return Err(DeclarationBuildErrorV3::DocumentItemLimit);
        }
        if check_depth {
            check_relation_depth_impl(&checked, budget, None).map_err(|error| match error {
                DeclaredRequestErrorV3::Validation(error) => error,
                DeclaredRequestErrorV3::Storage(_)
                | DeclaredRequestErrorV3::Registry(_)
                | DeclaredRequestErrorV3::DemandOverflow
                | DeclaredRequestErrorV3::FallibleAllocation
                | DeclaredRequestErrorV3::SourceMismatch
                | DeclaredRequestErrorV3::DocumentItemLimit => {
                    DeclarationBuildErrorV3::ScratchAllocation
                }
            })?;
        }
        struct ExpressionNodesV2 {
            nodes: Vec<SourceQualifiedNodeIdV2>,
            source_id: SourceIdV2,
            admitted_slots: Option<usize>,
            exceeded: bool,
        }
        impl HirVisitorV1 for ExpressionNodesV2 {
            fn visit_value(&mut self, value: &ValueExprHirV1) {
                if self
                    .admitted_slots
                    .is_some_and(|slots| self.nodes.len() >= slots)
                {
                    self.exceeded = true;
                    return;
                }
                self.nodes.push(SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: value.id,
                });
                quanta_gqlang_v2_hir::walk_value(self, value);
            }
            fn visit_bool(&mut self, value: &quanta_gqlang_v2_hir::BoolExprHirV1) {
                if self
                    .admitted_slots
                    .is_some_and(|slots| self.nodes.len() >= slots)
                {
                    self.exceeded = true;
                    return;
                }
                self.nodes.push(SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: value.id,
                });
                quanta_gqlang_v2_hir::walk_bool(self, value);
            }
        }
        let mut expression_nodes = ExpressionNodesV2 {
            nodes: expression_nodes,
            source_id,
            admitted_slots: admitted_node_slots,
            exceeded: false,
        };
        for item in &checked.checked.document().items {
            if admitted_node_slots.is_some_and(|_| {
                expected.len() >= expected.capacity() || order.len() >= order.capacity()
            }) {
                return Err(DeclarationBuildErrorV3::ItemCensusExceeded(item.span()));
            }
            let id = SemanticItemIdV2::new(source_id, item.id());
            match expected.binary_search_by_key(&id, |(candidate, _)| *candidate) {
                Ok(_) => {
                    return Err(DeclarationBuildErrorV3::DuplicateItem(item.span()));
                }
                Err(index) => expected.insert(
                    index,
                    (id, (item.span(), SemanticItemKindV2::from_hir(item))),
                ),
            }
            order.push(id);
            expression_nodes.visit_item(item);
            if expression_nodes.exceeded {
                return Err(DeclarationBuildErrorV3::ExpressionCensusExceeded(
                    item.span(),
                ));
            }
        }
        let mut expected_expressions = expression_nodes.nodes;
        expected_expressions.sort_unstable();
        expected_expressions.dedup();
        let source_anchor = extended
            .combined()
            .try_check_location(location(source_id, SpanV1::point(0)))
            .map_err(DeclarationBuildErrorV3::Anchor)?;
        Ok(Self {
            checked,
            source_anchor,
            expected,
            order,
            expected_expressions,
            registry_identity: extended.identity(),
        })
    }

    /// Checked syntax document.
    #[must_use]
    pub const fn checked(&self) -> &CheckedSyntaxDocumentV2 {
        &self.checked
    }
    /// Authored item order.
    #[must_use]
    pub fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }
    /// Expected item IDs and original spans.
    pub fn expected(&self) -> impl Iterator<Item = (SemanticItemIdV2, SpanV1)> + '_ {
        self.expected.iter().map(|(id, (span, _))| (*id, *span))
    }
}
