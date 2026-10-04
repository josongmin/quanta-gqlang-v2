//! Exact-key semantic document assembly and retained proof validation.

use super::{
    classify_rule_body_calls, document_failure, BTreeMap, BTreeSet, CatalogAssetIdV2,
    CheckedLocatedSpanV2, DeclaredDocumentV2, DocumentCatalogAuthorityV2,
    DocumentCatalogReferenceFamilyV2, DownstreamBudgetReceiptV2, FrontendFailureV2,
    GraphReferenceCatalogV2, NamespaceKindV2, ProfileAdmissionV2, ResolvedCatalogPayloadV2,
    ResolvedCatalogReferenceV2, ResolvedExplainTargetV2, ResolvedGraphTargetV2, ResolvedMetaKindV2,
    ResolvedPresentationKindV2, ResolvedRuleCallV2, ResolvedRuleV2, RuleBodyClassificationErrorV2,
    SemanticDocumentV2, SemanticItemIdV2, SemanticItemKindV2, SemanticItemV2, SourceIdV2,
    SourceQualifiedNodeIdV2, SourceRegistryIdentityV2, SpanV1, TypedExprShapeV2, TypedExprV2,
    TypedRelationExprV2,
};

fn graph_target_is_admitted(
    target: &ResolvedGraphTargetV2,
    authority: &GraphReferenceCatalogV2,
) -> bool {
    match target {
        ResolvedGraphTargetV2::Host(target) => authority.admits(target),
        // Composition targets are sealed from a prior Binder-derived local
        // producer, not minted by the host graph catalog. Their producer
        // membership is checked by the Composition semantic carrier.
        ResolvedGraphTargetV2::Composition(_) => true,
    }
}

impl SemanticDocumentV2 {
    /// Test-only minting seam: mutate the item map and document order of an
    /// already-sealed document so adversarial-document rails can forge
    /// membership mismatches (dropped item, orphan item, duplicate order id)
    /// against the real Logic validator. Compiled for unit tests or under
    /// the `test` feature; never part of a normal build.
    #[cfg(any(test, feature = "test"))]
    #[doc(hidden)]
    pub fn forged_membership<F>(mut self, mutate: F) -> Self
    where
        F: FnOnce(&mut Vec<SemanticItemIdV2>, &mut BTreeMap<SemanticItemIdV2, SemanticItemV2>),
    {
        mutate(&mut self.order, &mut self.items);
        self
    }

    /// Test-only minting seam for high-level forged expression-tree fixtures.
    /// The seam stays crate-private; downstream tests receive only a complete
    /// fixture document from `test_support`.
    #[cfg(any(test, feature = "test"))]
    pub(crate) fn forged_expressions<F>(mut self, mutate: F) -> Self
    where
        F: FnOnce(&mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>),
    {
        mutate(&mut self.expressions);
        self
    }

    /// Assemble exact-key semantic outputs; no positional zip or partial
    /// success path is available.
    #[expect(
        clippy::too_many_lines,
        reason = "legacy semantic assembly awaits phase extraction"
    )]
    pub(crate) fn try_assemble(
        declared: &DeclaredDocumentV2,
        produced: Vec<SemanticItemV2>,
        expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
        nested_relations: BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
        type_fact_count: usize,
        catalog: DocumentCatalogAuthorityV2,
        graph_authority: &GraphReferenceCatalogV2,
        budget_receipt: DownstreamBudgetReceiptV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = declared.checked.source_id();
        if catalog.source_registry_identity() != &declared.registry_identity {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "catalog proof belongs to another document authority registry",
            ));
        }
        let expected: BTreeSet<_> = declared.expected.keys().copied().collect();
        let mut items = BTreeMap::new();
        for item in produced {
            let id = item.item_id();
            if id.source_id() != source_id || !expected.contains(&id) {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    SpanV1::point(0),
                    "foreign or extra semantic item identity",
                ));
            }
            if declared
                .expected
                .get(&id)
                .is_some_and(|(_, kind)| *kind != SemanticItemKindV2::from_semantic(&item))
            {
                let span = declared
                    .expected
                    .get(&id)
                    .map_or(SpanV1::point(0), |(span, _)| *span);
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "semantic item kind differs from checked syntax",
                ));
            }
            if items.insert(id, item).is_some() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    SpanV1::point(0),
                    "duplicate semantic item identity",
                ));
            }
        }
        if items.keys().copied().collect::<BTreeSet<_>>() != expected {
            let missing = declared
                .expected
                .iter()
                .find(|(id, _)| !items.contains_key(id))
                .map_or(SpanV1::point(0), |(_, (span, _))| *span);
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                missing,
                "missing semantic item identity",
            ));
        }
        let semantic_assets = items
            .values()
            .filter_map(|item| match item {
                SemanticItemV2::Catalog(catalog) => Some((
                    catalog.asset_id().clone(),
                    catalog.asset_id().namespace_kind(),
                )),
                SemanticItemV2::Relation(_)
                | SemanticItemV2::Rule(_)
                | SemanticItemV2::Graph(_)
                | SemanticItemV2::Check(_)
                | SemanticItemV2::Presentation(_)
                | SemanticItemV2::Declaration(_)
                | SemanticItemV2::Composition(_)
                | SemanticItemV2::Meta(_)
                | SemanticItemV2::Command(_) => None,
            })
            .collect::<BTreeSet<_>>();
        let validated_assets = catalog
            .assets()
            .map(|(id, kind)| (id.clone(), kind))
            .collect::<BTreeSet<_>>();
        if semantic_assets != validated_assets {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "semantic catalog assets differ from modules-validated document assets",
            ));
        }
        let semantic_profiles = items
            .values()
            .filter_map(|item| {
                let SemanticItemV2::Catalog(catalog) = item else {
                    return None;
                };
                let ResolvedCatalogPayloadV2::Profile { admission, .. } = catalog.payload() else {
                    return None;
                };
                let admission = match admission {
                    quanta_gqlang_v2_hir::AdmissionV1::Allowlist => ProfileAdmissionV2::Allowlist,
                    quanta_gqlang_v2_hir::AdmissionV1::ReviewRequired => {
                        ProfileAdmissionV2::ReviewRequired
                    }
                    quanta_gqlang_v2_hir::AdmissionV1::CertifiedOnly => {
                        ProfileAdmissionV2::CertifiedOnly
                    }
                };
                Some((catalog.asset_id().clone(), admission))
            })
            .collect::<BTreeSet<_>>();
        let validated_profiles = catalog
            .profiles()
            .map(|(id, admission)| (id.clone(), admission))
            .collect::<BTreeSet<_>>();
        if semantic_profiles != validated_profiles {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "semantic profile admissions differ from modules-validated policy",
            ));
        }
        let certified = items
            .values()
            .filter_map(|item| match item {
                SemanticItemV2::Meta(meta)
                    if matches!(meta.kind(), ResolvedMetaKindV2::RequireCertified) =>
                {
                    Some(meta.location().location())
                }
                SemanticItemV2::Relation(_)
                | SemanticItemV2::Rule(_)
                | SemanticItemV2::Graph(_)
                | SemanticItemV2::Check(_)
                | SemanticItemV2::Presentation(_)
                | SemanticItemV2::Catalog(_)
                | SemanticItemV2::Declaration(_)
                | SemanticItemV2::Composition(_)
                | SemanticItemV2::Meta(_)
                | SemanticItemV2::Command(_) => None,
            })
            .collect::<Vec<_>>();
        if certified.len() > 1 || certified.first().copied() != catalog.require_certified_location()
        {
            return Err(document_failure("sema.document_validation_failed", source_id,
                certified.first().map_or(SpanV1::point(0), |location| location.span),
                "semantic certification directive differs from modules-validated exact location/count"));
        }
        // Modules seals one dependency per (source, target), retaining the
        // earliest checked location for repeated identical references. Mirror
        // that exact canonicalization before comparing the semantic payload.
        let mut semantic_edges: BTreeMap<
            (CatalogAssetIdV2, CatalogAssetIdV2),
            (
                NamespaceKindV2,
                Option<DocumentCatalogReferenceFamilyV2>,
                quanta_gqlang_v2_hir::LocatedSpanV2,
            ),
        > = BTreeMap::new();
        let mut semantic_conflict = false;
        for item in items.values() {
            let SemanticItemV2::Catalog(catalog) = item else {
                continue;
            };
            let mut add = |reference: &ResolvedCatalogReferenceV2, family| {
                let key = (catalog.asset_id().clone(), reference.id().clone());
                let candidate = (reference.kind(), family, reference.location().location());
                if let Some(existing) = semantic_edges.get_mut(&key) {
                    if existing.0 != candidate.0 || existing.1 != candidate.1 {
                        semantic_conflict = true;
                    } else {
                        let candidate_location = candidate.2;
                        let existing_location = existing.2;
                        if (
                            candidate_location.source_id.raw(),
                            candidate_location.span.start,
                            candidate_location.span.end,
                        ) < (
                            existing_location.source_id.raw(),
                            existing_location.span.start,
                            existing_location.span.end,
                        ) {
                            *existing = candidate;
                        }
                    }
                } else {
                    semantic_edges.insert(key, candidate);
                }
            };
            match catalog.payload() {
                ResolvedCatalogPayloadV2::View { query_ref } => {
                    if let Some(reference) = query_ref {
                        add(reference, Some(DocumentCatalogReferenceFamilyV2::ViewQuery));
                    }
                    for reference in catalog.inline_view_dependencies() {
                        add(reference, None);
                    }
                }
                ResolvedCatalogPayloadV2::Model { source } => {
                    add(source, Some(DocumentCatalogReferenceFamilyV2::ModelSource))
                }
                ResolvedCatalogPayloadV2::Pack { source, .. } => {
                    add(source, Some(DocumentCatalogReferenceFamilyV2::PackSource))
                }
                ResolvedCatalogPayloadV2::Profile { capabilities, .. } => {
                    for reference in capabilities {
                        add(
                            reference,
                            Some(DocumentCatalogReferenceFamilyV2::ProfileCapability),
                        );
                    }
                }
                ResolvedCatalogPayloadV2::Gate { checks, .. } => {
                    for reference in checks {
                        add(reference, Some(DocumentCatalogReferenceFamilyV2::GateCheck));
                    }
                }
                ResolvedCatalogPayloadV2::RuleBinding { check_ref } => add(
                    check_ref,
                    Some(DocumentCatalogReferenceFamilyV2::RuleBindingCheck),
                ),
            }
        }
        let validated_edges = catalog
            .dependencies()
            .iter()
            .map(|edge| {
                (
                    (
                        edge.source_asset_id().clone(),
                        edge.target_asset_id().clone(),
                    ),
                    (
                        edge.expected_target_kind(),
                        edge.family(),
                        edge.location().location(),
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if semantic_conflict
            || validated_edges.len() != catalog.dependencies().len()
            || semantic_edges != validated_edges
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "semantic catalog references differ from modules-validated dependency graph",
            ));
        }
        if expressions
            .iter()
            .any(|(key, fact)| key.source_id != source_id || *key != fact.key())
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "foreign or mismatched typed expression identity",
            ));
        }
        if type_fact_count != declared.expected_expressions.len()
            || expressions.keys().copied().collect::<BTreeSet<_>>() != declared.expected_expressions
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "typed expression set differs from checked HIR and types owner facts",
            ));
        }
        for item in items.values() {
            if let SemanticItemV2::Declaration(decl) = item {
                if expressions.get(&decl.body().key()) != Some(decl.body()) {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        SpanV1::point(0),
                        "declaration body does not match document expression proof",
                    ));
                }
            }
            if let SemanticItemV2::Rule(rule) = item {
                if expressions.get(&rule.head_term().key()) != Some(rule.head_term())
                    || expressions.get(&rule.body().expression().key())
                        != Some(rule.body().expression())
                {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        SpanV1::point(0),
                        "rule head/body disagree with the document expression proof",
                    ));
                }
            }
            if let SemanticItemV2::Graph(graph) = item {
                for reference in graph.references() {
                    if !graph_target_is_admitted(reference.target(), graph_authority)
                        || reference.use_location().registry_identity()
                            != &declared.registry_identity
                        || reference.use_location().location().source_id != source_id
                    {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.use_location().location().span, "graph reference does not belong to this document and sealed graph authority"));
                    }
                }
            }
            if let SemanticItemV2::Presentation(presentation) = item {
                let mut uses = Vec::new();
                match presentation.kind() {
                    ResolvedPresentationKindV2::Explain {
                        target: ResolvedExplainTargetV2::Entity(graph),
                        ..
                    }
                    | ResolvedPresentationKindV2::Describe { target: graph } => uses.push(graph),
                    ResolvedPresentationKindV2::Sample { scope, .. } => uses.extend(scope.as_ref()),
                    ResolvedPresentationKindV2::Compare {
                        baseline,
                        candidate,
                        ..
                    } => {
                        uses.push(baseline);
                        uses.push(candidate);
                    }
                    ResolvedPresentationKindV2::Export { source, .. } => uses.push(source),
                    ResolvedPresentationKindV2::Explain {
                        target: ResolvedExplainTargetV2::Rule(_),
                        ..
                    }
                    | ResolvedPresentationKindV2::Show { .. } => {}
                }
                for reference in uses {
                    if !graph_target_is_admitted(reference.target(), graph_authority)
                        || reference.location().registry_identity() != &declared.registry_identity
                        || reference.location().location().source_id != source_id
                    {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.location().location().span, "presentation selector differs from the sealed graph/document authority"));
                    }
                }
            }
            if let SemanticItemV2::Check(check) = item {
                let uses = check.scope().into_iter().chain(check.pattern());
                for reference in uses {
                    if !graph_target_is_admitted(reference.target(), graph_authority)
                        || reference.location().registry_identity() != &declared.registry_identity
                        || reference.location().location().source_id != source_id
                    {
                        return Err(document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            reference.location().location().span,
                            "CHECK selector differs from sealed graph/document authority",
                        ));
                    }
                }
                if let Some(nested) = check.nested_graph() {
                    for reference in nested.references() {
                        if !graph_target_is_admitted(reference.target(), graph_authority)
                            || reference.use_location().registry_identity()
                                != &declared.registry_identity
                            || reference.use_location().location().source_id != source_id
                        {
                            return Err(document_failure("sema.document_validation_failed", source_id, reference.use_location().location().span, "nested CHECK/FIND selector differs from sealed graph/document authority"));
                        }
                    }
                }
            }
        }
        check_nested_relation_finalizer(
            source_id,
            &declared.registry_identity,
            &expressions,
            &nested_relations,
        )
        .map_err(|message| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                message,
            )
        })?;
        Ok(Self {
            source_id,
            order: declared.order.clone(),
            source_anchor: declared.source_anchor.clone(),
            items,
            expressions,
            nested_relations,
            catalog,
            budget_receipt,
        })
    }

    /// Source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }
    /// Original item order.
    #[must_use]
    pub fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }
    /// Byte-zero diagnostic anchor checked against this transaction's original
    /// extended source registry, retained even when the document has no items.
    #[must_use]
    pub const fn source_anchor(&self) -> &CheckedLocatedSpanV2 {
        &self.source_anchor
    }
    /// Exact semantic item by ID.
    #[must_use]
    pub fn item(&self, id: SemanticItemIdV2) -> Option<&SemanticItemV2> {
        self.items.get(&id)
    }
    /// All semantic items by canonical identity.
    pub fn items(&self) -> impl Iterator<Item = (&SemanticItemIdV2, &SemanticItemV2)> {
        self.items.iter()
    }
    /// Complete source-rule set from the same sealed document transaction.
    pub fn rules(&self) -> impl Iterator<Item = &ResolvedRuleV2> {
        self.items.values().filter_map(|item| match item {
            SemanticItemV2::Rule(rule) => Some(rule),
            SemanticItemV2::Relation(_)
            | SemanticItemV2::Graph(_)
            | SemanticItemV2::Check(_)
            | SemanticItemV2::Presentation(_)
            | SemanticItemV2::Catalog(_)
            | SemanticItemV2::Declaration(_)
            | SemanticItemV2::Composition(_)
            | SemanticItemV2::Meta(_)
            | SemanticItemV2::Command(_) => None,
        })
    }
    /// Closed typed expression by exact source-qualified node ID.
    #[must_use]
    pub fn expression(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExprV2> {
        self.expressions.get(&key)
    }
    /// Every typed expression fact, including child nodes, in stable key order.
    pub fn expressions(&self) -> impl Iterator<Item = (&SourceQualifiedNodeIdV2, &TypedExprV2)> {
        self.expressions.iter()
    }
    /// Exact typed nested relation owned by one relational EXISTS root.
    #[must_use]
    pub fn nested_relation(&self, root: SourceQualifiedNodeIdV2) -> Option<&TypedRelationExprV2> {
        self.nested_relations.get(&root)
    }
    /// Every nested relation in stable root-identity order.
    pub fn nested_relations(
        &self,
    ) -> impl Iterator<Item = (&SourceQualifiedNodeIdV2, &TypedRelationExprV2)> {
        self.nested_relations.iter()
    }
    /// Independently classify this document's typed rule-body source calls.
    /// Logic uses the same shape proof as the semantic producer.
    pub fn classify_rule_body_calls(
        &self,
        root: SourceQualifiedNodeIdV2,
    ) -> Result<Vec<ResolvedRuleCallV2>, RuleBodyClassificationErrorV2> {
        classify_rule_body_calls(
            root,
            |key| self.expression(key),
            |key| self.nested_relation(key),
        )
    }
    /// Document catalog proof.
    #[must_use]
    pub const fn catalog(&self) -> &DocumentCatalogAuthorityV2 {
        &self.catalog
    }
    /// Immutable downstream budget receipt projected once from the closed
    /// authority's `FrontendBudgetV2`. Downstream stages read every traversal
    /// and output cap from this receipt; they never accept an independent
    /// budget argument.
    #[must_use]
    pub const fn budget_receipt(&self) -> DownstreamBudgetReceiptV2 {
        self.budget_receipt
    }
}

/// Finalizer of the nested-relation registry and every retained correlation
/// edge. It requires exact equality between the relational-EXISTS expression
/// roots and the registry keys, document/registry identity of each stored
/// relation, and — for every correlation reference — membership in the sealed
/// expression map, uniqueness, and a checked location minted by this
/// document's registry. A dropped or extra nested root, a dropped, duplicate,
/// swapped, foreign, or cross-registry correlation reference rejects before
/// the document is assembled.
pub(super) fn check_nested_relation_finalizer(
    source_id: SourceIdV2,
    registry_identity: &SourceRegistryIdentityV2,
    expressions: &BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
) -> Result<(), &'static str> {
    let expected_nested_roots = expressions
        .values()
        .filter_map(|expression| match expression.shape() {
            TypedExprShapeV2::RelationExists { relation_root, .. } => Some(*relation_root),
            TypedExprShapeV2::Literal(_)
            | TypedExprShapeV2::Binding(_)
            | TypedExprShapeV2::Path { .. }
            | TypedExprShapeV2::Unary { .. }
            | TypedExprShapeV2::Binary { .. }
            | TypedExprShapeV2::Call { .. }
            | TypedExprShapeV2::Paren { .. }
            | TypedExprShapeV2::Record(_)
            | TypedExprShapeV2::List(_)
            | TypedExprShapeV2::BoolValue { .. }
            | TypedExprShapeV2::Compare { .. }
            | TypedExprShapeV2::And { .. }
            | TypedExprShapeV2::Or { .. }
            | TypedExprShapeV2::Not { .. }
            | TypedExprShapeV2::LogicalQuantifier { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    if nested_relations.keys().copied().collect::<BTreeSet<_>>() != expected_nested_roots {
        return Err("nested relation registry differs from relational EXISTS expression facts");
    }
    if nested_relations.iter().any(|(key, relation)| {
        key.source_id != source_id
            || relation.key() != *key
            || relation.location().location().source_id != source_id
            || relation.location().registry_identity() != registry_identity
    }) {
        return Err("nested relation registry identity differs from this document");
    }
    let expression_keys = expressions.keys().copied().collect::<BTreeSet<_>>();
    for expression in expressions.values() {
        let TypedExprShapeV2::RelationExists { correlations, .. } = expression.shape() else {
            continue;
        };
        let witnesses = correlations
            .iter()
            .map(|edge| CorrelationWitnessV2 {
                reference: edge.reference(),
                binding_source: edge.binding().source_id(),
                registry: edge.location().registry_identity().clone(),
            })
            .collect::<Vec<_>>();
        check_correlation_witnesses(source_id, registry_identity, &expression_keys, &witnesses)?;
    }
    Ok(())
}

/// Read-only correlation witness compared by the document finalizer. This is
/// comparison data only; Binder and the sealing path remain the authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CorrelationWitnessV2 {
    pub(super) reference: quanta_gqlang_v2_types::SourceQualifiedNodeIdV2,
    pub(super) binding_source: SourceIdV2,
    pub(super) registry: SourceRegistryIdentityV2,
}

/// Private per-nested-relation correlation finalizer: every retained capture
/// reference must be a member of the sealed expression set of this document,
/// appear at most once in one vector, and carry a location minted by this
/// document's extended source registry.
pub(super) fn check_correlation_witnesses(
    source_id: SourceIdV2,
    registry_identity: &SourceRegistryIdentityV2,
    expression_keys: &BTreeSet<SourceQualifiedNodeIdV2>,
    witnesses: &[CorrelationWitnessV2],
) -> Result<(), &'static str> {
    let mut references = BTreeSet::new();
    for witness in witnesses {
        if witness.reference.source_id != source_id
            || witness.binding_source != source_id
            || !expression_keys.contains(&witness.reference)
        {
            return Err("correlation reference is foreign to this document's expressions");
        }
        if !references.insert(witness.reference) {
            return Err("duplicate correlation reference in one nested relation");
        }
        if &witness.registry != registry_identity {
            return Err("correlation reference location belongs to another source registry");
        }
    }
    Ok(())
}
