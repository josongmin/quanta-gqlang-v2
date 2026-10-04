//! Inline View source-use collection and topology inputs.

use super::{
    document_failure, location, module_failure, CatalogAssetIdV2, CatalogAssetRefInputV2,
    ClosedFrontendAuthorityV2, DocumentCatalogDependencyInputV2, DocumentCatalogReferenceInputV2,
    DocumentCatalogReferenceOriginV2, DocumentCatalogValidatorV2, ExtendedSourceRegistryViewV2,
    ExternalCatalogPayloadV2, FrontendTransactionErrorV2, HirDocumentV1, HirNodeIdV1, HirVisitorV1,
    NamespaceKindV2, PredeclaredDocumentAssetV2, ProgramItemHirV1, RelationTypeV1,
    ResolvedCatalogReferenceV2, SemanticStepCounterV2, SourceCatalogAssetsV3, SourceIdV2,
    SourceQualifiedNodeIdV2, SpanV1,
};

/// Pre-topology facts for every inline document View. Modules remains the
/// topology authority; this plan only retains the exact source use and the
/// HIR body that must be materialized after its opaque proof is issued.
pub(super) struct InlineViewPlanV2 {
    /// Local inline View asset -> catalog declaration HIR identity.
    pub(super) declarations: Vec<(CatalogAssetIdV2, HirNodeIdV1)>,
    /// Local inline View asset -> root relation identity of its query body.
    pub(super) relation_roots: Vec<(CatalogAssetIdV2, SourceQualifiedNodeIdV2)>,
    /// Exact relation-source HIR node -> modules-owned local View asset.
    pub(super) source_targets: Vec<(HirNodeIdV1, ViewSourceTargetV2)>,
    /// Every direct source use retained by its owning inline View declaration.
    pub(super) direct_references: Vec<(HirNodeIdV1, Vec<ResolvedCatalogReferenceV2>)>,
    /// Modules input; only inline View -> View uses belong in this graph.
    pub(super) dependency_edges: Vec<DocumentCatalogDependencyInputV2>,
}

impl InlineViewPlanV2 {
    pub(super) fn declaration_for(&self, asset: &CatalogAssetIdV2) -> Option<HirNodeIdV1> {
        self.declarations
            .binary_search_by(|(candidate, _)| candidate.cmp(asset))
            .ok()
            .and_then(|index| self.declarations.get(index).map(|(_, node)| *node))
    }

    pub(super) fn relation_root_for(
        &self,
        asset: &CatalogAssetIdV2,
    ) -> Option<SourceQualifiedNodeIdV2> {
        self.relation_roots
            .binary_search_by(|(candidate, _)| candidate.cmp(asset))
            .ok()
            .and_then(|index| self.relation_roots.get(index).map(|(_, root)| *root))
    }

    pub(super) fn source_target_for(&self, node: HirNodeIdV1) -> Option<&ViewSourceTargetV2> {
        Self::source_target_in(&self.source_targets, node)
    }

    pub(super) fn source_target_in(
        source_targets: &[(HirNodeIdV1, ViewSourceTargetV2)],
        node: HirNodeIdV1,
    ) -> Option<&ViewSourceTargetV2> {
        source_targets
            .binary_search_by_key(&node, |(candidate, _)| *candidate)
            .ok()
            .and_then(|index| source_targets.get(index).map(|(_, target)| target))
    }
}

/// A relation source whose spelling resolved through the Modules View namespace.
/// Local Views are materialized by the document topology. External Views carry
/// a Modules-owned typed relation contract and require no local body.
#[derive(Clone)]
pub(super) enum ViewSourceTargetV2 {
    Inline(CatalogAssetIdV2),
    External {
        asset: CatalogAssetIdV2,
        relation_type: RelationTypeV1,
    },
}

struct ViewSourceUseCollectorV2<'a> {
    source_decl: Option<HirNodeIdV1>,
    uses: &'a mut Vec<ViewSourceUseV2>,
}

impl HirVisitorV1 for ViewSourceUseCollectorV2<'_> {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        if let quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) = &relation.kind {
            self.uses.push((
                self.source_decl,
                relation.id,
                source.name.clone(),
                source.name_span,
            ));
        }
        quanta_gqlang_v2_hir::walk_relation(self, relation);
    }
}

fn asset_ref_from_predeclared(asset: &PredeclaredDocumentAssetV2) -> CatalogAssetRefInputV2 {
    CatalogAssetRefInputV2 {
        module_id: asset.id().module_id().clone(),
        namespace_kind: asset.kind(),
        local_name: asset.id().local_name().to_owned(),
        location: asset.location().clone(),
    }
}

fn asset_ref_from_view_reference(
    reference: &quanta_gqlang_v2_modules::ResolvedDocumentCatalogReferenceV2,
) -> CatalogAssetRefInputV2 {
    CatalogAssetRefInputV2 {
        module_id: reference.id().module_id().clone(),
        namespace_kind: reference.kind(),
        local_name: reference.id().local_name().to_owned(),
        location: reference.location().clone(),
    }
}

pub(super) fn inline_query_for_decl(
    syntax: &HirDocumentV1,
    declaration: HirNodeIdV1,
) -> Option<&quanta_gqlang_v2_hir::QueryHirV1> {
    syntax.items.iter().find_map(|item| {
        let ProgramItemHirV1::CatalogDecl(decl) = item else {
            return None;
        };
        if decl.id != declaration {
            return None;
        }
        match &decl.payload {
            quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                quanta_gqlang_v2_hir::ViewBodyV1::Inline(query),
            ) => Some(query.as_ref()),
            quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(_),
            )
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => None,
        }
    })
}

type ViewSourceUseV2 = (Option<HirNodeIdV1>, HirNodeIdV1, String, SpanV1);

fn collect_view_declarations_and_uses(
    syntax: &HirDocumentV1,
    source_id: SourceIdV2,
    catalog_items: &SourceCatalogAssetsV3,
    plan: &mut InlineViewPlanV2,
) -> Result<Vec<ViewSourceUseV2>, FrontendTransactionErrorV2> {
    let mut uses = Vec::new();
    for item in &syntax.items {
        match item {
            ProgramItemHirV1::CatalogDecl(decl)
                if matches!(
                    decl.payload,
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::Inline(_)
                    )
                ) =>
            {
                let asset = catalog_items.get(&decl.id).ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        decl.span,
                        "inline View declaration lacks modules predeclaration",
                    )
                })?;
                let query = inline_query_for_decl(syntax, decl.id).ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        decl.span,
                        "inline View declaration lost its exact query body",
                    )
                })?;
                plan.declarations.push((asset.id().clone(), decl.id));
                plan.relation_roots.push((
                    asset.id().clone(),
                    SourceQualifiedNodeIdV2 {
                        source_id,
                        node_id: query.body.id,
                    },
                ));
                let mut collector = ViewSourceUseCollectorV2 {
                    source_decl: Some(decl.id),
                    uses: &mut uses,
                };
                collector.visit_query(query);
            }
            ProgramItemHirV1::CatalogDecl(_) => {}
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => {
                let mut collector = ViewSourceUseCollectorV2 {
                    source_decl: None,
                    uses: &mut uses,
                };
                collector.visit_item(item);
            }
        }
    }
    Ok(uses)
}

pub(super) fn collect_inline_view_plan(
    syntax: &HirDocumentV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &ExtendedSourceRegistryViewV2,
    catalog_items: &SourceCatalogAssetsV3,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<InlineViewPlanV2, FrontendTransactionErrorV2> {
    let inline_slots = syntax.items.iter().filter(|item| matches!(
        item,
        ProgramItemHirV1::CatalogDecl(decl) if matches!(decl.payload, quanta_gqlang_v2_hir::CatalogPayloadV1::View(quanta_gqlang_v2_hir::ViewBodyV1::Inline(_)))
    )).count();
    let mut plan = InlineViewPlanV2 {
        declarations: Vec::with_capacity(inline_slots),
        relation_roots: Vec::with_capacity(inline_slots),
        source_targets: Vec::new(),
        direct_references: Vec::with_capacity(inline_slots),
        dependency_edges: Vec::new(),
    };
    let uses = collect_view_declarations_and_uses(syntax, source_id, catalog_items, &mut plan)?;
    plan.declarations
        .sort_unstable_by(|left, right| left.0.cmp(&right.0));
    plan.relation_roots
        .sort_unstable_by(|left, right| left.0.cmp(&right.0));
    if plan
        .relation_roots
        .windows(2)
        .any(|pair| matches!(pair, [first, second] if first.0 == second.0))
    {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            SpanV1::point(0),
            "duplicate inline View materialization identity",
        )
        .into());
    }
    for (source_decl, node, name, span) in uses {
        steps.charge(source_id, span)?;
        let checked = extended
            .combined()
            .try_check_location(location(source_id, span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    error.to_string(),
                )
            })?;
        let Some(target) = validator
            .resolve_reference_if_present(DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: name,
                expected_kind: NamespaceKindV2::View,
                location: checked.clone(),
            })
            .map_err(module_failure)?
        else {
            continue;
        };
        let source_target = match target.origin() {
            DocumentCatalogReferenceOriginV2::Local
                if plan.declaration_for(target.id()).is_some() =>
            {
                ViewSourceTargetV2::Inline(target.id().clone())
            }
            DocumentCatalogReferenceOriginV2::External => {
                let Some(asset) = authority.modules().external_catalog().asset(target.id()) else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        span,
                        "Modules View reference lacks its external typed payload",
                    )
                    .into());
                };
                let ExternalCatalogPayloadV2::View(relation_type) = asset.payload() else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        span,
                        "Modules View reference payload differs from its namespace",
                    )
                    .into());
                };
                ViewSourceTargetV2::External {
                    asset: target.id().clone(),
                    relation_type: relation_type.clone(),
                }
            }
            DocumentCatalogReferenceOriginV2::Local
            | DocumentCatalogReferenceOriginV2::VisibleHost => return Err(document_failure(
                "sema.unmaterialized_view_source",
                source_id,
                span,
                "relation source resolves to a View without a local inline or external typed body",
            )
            .into()),
        };
        let insertion = match plan
            .source_targets
            .binary_search_by_key(&node, |(prior, _)| *prior)
        {
            Ok(_) => {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "duplicate inline View source-use identity",
                )
                .into())
            }
            Err(index) => index,
        };
        let local_inline = matches!(&source_target, ViewSourceTargetV2::Inline(_));
        plan.source_targets.insert(insertion, (node, source_target));
        if let (Some(source_decl), true) = (source_decl, local_inline) {
            let source = catalog_items.get(&source_decl).ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "inline View source lacks its modules predeclaration",
                )
            })?;
            plan.dependency_edges
                .push(DocumentCatalogDependencyInputV2 {
                    source: asset_ref_from_predeclared(source),
                    target: asset_ref_from_view_reference(&target),
                    expected_target_kind: NamespaceKindV2::View,
                    location: checked,
                });
            let reference = ResolvedCatalogReferenceV2::from_proof(&target);
            match plan
                .direct_references
                .binary_search_by_key(&source_decl, |(owner, _)| *owner)
            {
                Ok(index) => {
                    let Some((_, references)) = plan.direct_references.get_mut(index) else {
                        return Err(document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            span,
                            "inline View reference index lost its sorted owner",
                        )
                        .into());
                    };
                    references.push(reference);
                }
                Err(index) => plan
                    .direct_references
                    .insert(index, (source_decl, vec![reference])),
            }
        }
    }
    for (_, references) in &mut plan.direct_references {
        references.sort_by(|left, right| {
            (
                left.id(),
                left.location().location().source_id,
                left.location().location().span.start,
                left.location().location().span.end,
            )
                .cmp(&(
                    right.id(),
                    right.location().location().source_id,
                    right.location().location().span.start,
                    right.location().location().span.end,
                ))
        });
        references.dedup_by(|left, right| left.id() == right.id());
    }
    Ok(plan)
}
