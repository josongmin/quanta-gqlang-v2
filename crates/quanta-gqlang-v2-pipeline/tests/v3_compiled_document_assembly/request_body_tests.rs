use super::*;

pub(super) struct BodyRequestSinkV3 {
    pub(super) live: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub(super) limit: std::sync::atomic::AtomicU64,
    pub(super) peak: std::sync::atomic::AtomicU64,
    pub(super) issuer: std::sync::Arc<()>,
}

struct BodyRequestPermitV3 {
    live: std::sync::Arc<std::sync::atomic::AtomicU64>,
    bytes: u64,
    issuer: std::sync::Arc<()>,
}

impl Drop for BodyRequestPermitV3 {
    fn drop(&mut self) {
        self.live
            .fetch_sub(self.bytes, std::sync::atomic::Ordering::AcqRel);
    }
}

impl quanta_gqlang_v2_surface::FrontendRequestStorageSinkV3 for BodyRequestSinkV3 {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<
        Box<dyn quanta_gqlang_v2_surface::FrontendRequestStoragePermitV3>,
        quanta_gqlang_v2_surface::FrontendStorageRefusalV3,
    > {
        use quanta_gqlang_v2_surface::FrontendStorageRefusalV3;
        let bytes = logical_bytes
            .checked_add(
                u64::try_from(std::mem::size_of::<BodyRequestPermitV3>())
                    .map_err(|_size_overflow| FrontendStorageRefusalV3::DemandOverflow)?,
            )
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.live.load(std::sync::atomic::Ordering::Acquire);
        loop {
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.limit.load(std::sync::atomic::Ordering::Acquire) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self.live.compare_exchange_weak(
                live,
                next,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.peak
                        .fetch_max(next, std::sync::atomic::Ordering::AcqRel);
                    break;
                }
                Err(observed) => live = observed,
            }
        }
        Ok(Box::new(BodyRequestPermitV3 {
            live: std::sync::Arc::clone(&self.live),
            bytes,
            issuer: std::sync::Arc::clone(&self.issuer),
        }))
    }

    fn owns_permit_v3(
        &self,
        permit: &dyn quanta_gqlang_v2_surface::FrontendRequestStoragePermitV3,
    ) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<BodyRequestPermitV3>()
            .is_some_and(|value| std::sync::Arc::ptr_eq(&value.issuer, &self.issuer))
    }
}

fn bound_inline_view_request_v3<'a>(
    source: &str,
    authority: &'a ClosedFrontendAuthorityV2,
    root: &quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3<'_>,
) -> quanta_gqlang_v2_pipeline::ResolvedSourceBinderRequestSourceV3<'a> {
    use quanta_gqlang_v2_pipeline::{
        begin_bind_source_document_with_request_owner_v3,
        begin_prepare_source_body_with_request_owner_v3,
        begin_resolve_source_document_with_request_owner_v3,
        check_source_document_with_request_owner_v3, declare_source_document_with_request_owner_v3,
        parse_source_document_with_request_owner_v3,
        predeclare_source_catalog_with_request_owner_v3,
        resolve_source_bindings_with_request_owner_v3,
    };
    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, authority.budget(), root)
            .expect("source parser");
    let checked = check_source_document_with_request_owner_v3(DOC_SOURCE, source, root, parsed)
        .expect("checked HIR");
    let declared = declare_source_document_with_request_owner_v3(source, authority, root, checked)
        .expect("declared source");
    let resolving = begin_resolve_source_document_with_request_owner_v3(source, root, declared)
        .expect("Modules session");
    let predeclared = predeclare_source_catalog_with_request_owner_v3(source, root, resolving)
        .expect("View asset");
    let preparing = begin_prepare_source_body_with_request_owner_v3(source, root, predeclared)
        .expect("builtin inventory");
    let declaring = begin_bind_source_document_with_request_owner_v3(source, root, preparing)
        .expect("Binder declarations");
    resolve_source_bindings_with_request_owner_v3(source, root, declaring)
        .expect("Binder resolutions")
}

fn assert_scalar_body_semantics_v3(
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
    body: &quanta_gqlang_v2_pipeline::MaterializedSourceBodyRequestSourceV3<'_>,
    names: &[&str],
    expected_type: &ValueTypeV1,
) {
    assert_eq!(body.output_field_names_v3().collect::<Vec<_>>(), names);
    assert_eq!(body.output_row_v3().fields().len(), names.len());
    for (field, name) in body.output_row_v3().fields().iter().zip(names) {
        assert_eq!(field.name(), *name);
        assert_eq!(field.value_type(), expected_type);
    }
    let keys: BTreeSet<_> = body.expression_keys_v3().collect();
    assert_eq!(keys.len(), names.len() * 2);
    for (column, name) in body.output_columns_v3().iter().zip(names) {
        assert_eq!(column.name(), *name);
        assert!(matches!(
            column.origin(),
            quanta_gqlang_v2_sema::OutputColumnOriginV2::Expression(key)
                if key.source_id == DOC_SOURCE && keys.contains(key)
        ));
    }
    let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
        DOC_SOURCE,
        source,
        &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
    )
    .expect("independent legacy parse");
    let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("legacy checked syntax");
    let legacy = quanta_gqlang_v2_sema::resolve_checked_document_v2(checked, authority)
        .expect("legacy semantic transaction");
    let legacy_view = legacy
        .items()
        .find_map(|(_, item)| match item {
            SemanticItemV2::Catalog(catalog) => catalog.inline_view(),
            SemanticItemV2::Relation(relation) => Some(relation),
            SemanticItemV2::Command(_)
            | SemanticItemV2::Rule(_)
            | SemanticItemV2::Graph(_)
            | SemanticItemV2::Check(_)
            | SemanticItemV2::Presentation(_)
            | SemanticItemV2::Declaration(_)
            | SemanticItemV2::Composition(_)
            | SemanticItemV2::Meta(_) => None,
        })
        .expect("one legacy View body");
    assert_eq!(body.output_row_v3(), legacy_view.output().row());
    assert_eq!(body.output_columns_v3(), legacy_view.output().columns());
    assert_eq!(keys, legacy.expressions().map(|(key, _)| *key).collect());
}

#[test]
fn inline_view_request_types_retain_original_root_and_refuse_short_or_foreign() {
    use quanta_gqlang_v2_pipeline::{
        materialize_source_body_with_request_owner_v3,
        topologize_source_body_with_request_owner_v3, type_source_body_with_request_owner_v3,
    };
    use quanta_gqlang_v2_sema::SourceBodyTypesRequestErrorV3;
    use quanta_gqlang_v2_surface::{
        FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let source = "view active | select=from records r | project r.id as id";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = BodyRequestSinkV3 {
        live: Arc::clone(&live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let before_types = live.load(Ordering::Acquire);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound)
        .expect("source-backed View Types producer");
    assert_eq!(typed.typed_fact_count_v3(), 2);
    assert_eq!(typed.relation_row_count_v3(), 2);
    let types_birth = live.load(Ordering::Acquire) - before_types;
    assert!(types_birth > 0);
    let topology = topologize_source_body_with_request_owner_v3(source, &root, typed)
        .expect("Modules seals one source-backed View");
    assert_eq!(topology.dependency_first_view_count_v3(), 1);
    let body = materialize_source_body_with_request_owner_v3(source, &root, topology)
        .expect("Sema materializes one typed View body");
    assert_eq!(body.output_field_count_v3(), 1);
    assert_eq!(body.output_field_names_v3().collect::<Vec<_>>(), ["id"]);
    assert_eq!(body.expression_count_v3(), 2);
    assert_scalar_body_semantics_v3(source, &authority, &body, &["id"], &ValueTypeV1::int());
    drop(body);
    assert_eq!(live.load(Ordering::Acquire), 0);

    let bound = bound_inline_view_request_v3(source, &authority, &root);
    sink.limit.store(
        live.load(Ordering::Acquire) + types_birth - 1,
        Ordering::Release,
    );
    assert!(matches!(
        type_source_body_with_request_owner_v3(source, &root, bound),
        Err(SourceBodyTypesRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);

    sink.limit.store(u64::MAX, Ordering::Release);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound)
        .expect("Type owner before short Modules topology");
    sink.limit
        .store(live.load(Ordering::Acquire), Ordering::Release);
    assert!(matches!(
        topologize_source_body_with_request_owner_v3(source, &root, typed),
        Err(
            quanta_gqlang_v2_pipeline::SourceBodyTopologyRequestErrorV3::Storage(
                FrontendStorageRefusalV3::BudgetRejected
            )
        )
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);

    sink.limit.store(u64::MAX, Ordering::Release);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let foreign_live = Arc::new(AtomicU64::new(0));
    let foreign_sink = BodyRequestSinkV3 {
        live: Arc::clone(&foreign_live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        source,
        &foreign_sink,
    );
    assert!(matches!(
        type_source_body_with_request_owner_v3(source, &foreign_root, bound),
        Err(SourceBodyTypesRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);

    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound)
        .expect("Type owner before foreign Modules topology");
    assert!(matches!(
        topologize_source_body_with_request_owner_v3(source, &foreign_root, typed),
        Err(
            quanta_gqlang_v2_pipeline::SourceBodyTopologyRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            )
        )
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);

    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound)
        .expect("Type owner before foreign body materialization");
    let topology = topologize_source_body_with_request_owner_v3(source, &root, typed)
        .expect("Modules owner before foreign body materialization");
    assert!(matches!(
        materialize_source_body_with_request_owner_v3(source, &foreign_root, topology),
        Err(
            quanta_gqlang_v2_sema::SourceBodyMaterializationRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            )
        )
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);
}

#[test]
fn inline_view_nine_projection_types_admit_vec_growth_before_birth() {
    use quanta_gqlang_v2_pipeline::{
        materialize_source_body_with_request_owner_v3,
        topologize_source_body_with_request_owner_v3, type_source_body_with_request_owner_v3,
    };
    use quanta_gqlang_v2_sema::SourceBodyTypesRequestErrorV3;
    use quanta_gqlang_v2_surface::{
        FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let source = "view active | select=from records r | project \
        r.id as id0, r.id as id1, r.id as id2, r.id as id3, r.id as id4, \
        r.id as id5, r.id as id6, r.id as id7, r.id as id8";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = BodyRequestSinkV3 {
        live: Arc::clone(&live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let before_types = live.load(Ordering::Acquire);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound)
        .expect("nine source-issued projection aliases");
    assert_eq!(typed.typed_fact_count_v3(), 18);
    assert_eq!(typed.relation_row_count_v3(), 2);
    let type_charge = live.load(Ordering::Acquire) - before_types;
    assert!(type_charge > 0);
    let topology = topologize_source_body_with_request_owner_v3(source, &root, typed)
        .expect("nine-column View still has one Modules-issued topology node");
    assert_eq!(topology.dependency_first_view_count_v3(), 1);
    let before_body = live.load(Ordering::Acquire);
    let body = materialize_source_body_with_request_owner_v3(source, &root, topology)
        .expect("real nine-column Sema body");
    assert_eq!(body.output_field_count_v3(), 9);
    assert_eq!(
        body.output_field_names_v3().collect::<Vec<_>>(),
        ["id0", "id1", "id2", "id3", "id4", "id5", "id6", "id7", "id8"]
    );
    assert_eq!(body.expression_count_v3(), 18);
    assert_scalar_body_semantics_v3(
        source,
        &authority,
        &body,
        &[
            "id0", "id1", "id2", "id3", "id4", "id5", "id6", "id7", "id8",
        ],
        &ValueTypeV1::int(),
    );
    let body_charge = live.load(Ordering::Acquire) - before_body;
    assert!(body_charge > 0);
    drop(body);
    assert_eq!(live.load(Ordering::Acquire), 0);

    let bound = bound_inline_view_request_v3(source, &authority, &root);
    sink.limit.store(
        live.load(Ordering::Acquire) + type_charge - 1,
        Ordering::Release,
    );
    assert!(matches!(
        type_source_body_with_request_owner_v3(source, &root, bound),
        Err(SourceBodyTypesRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);

    sink.limit.store(u64::MAX, Ordering::Release);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound)
        .expect("Types before body short-budget refusal");
    let topology = topologize_source_body_with_request_owner_v3(source, &root, typed)
        .expect("Modules before body short-budget refusal");
    sink.limit.store(
        live.load(Ordering::Acquire) + body_charge - 1,
        Ordering::Release,
    );
    assert!(matches!(
        materialize_source_body_with_request_owner_v3(source, &root, topology),
        Err(
            quanta_gqlang_v2_sema::SourceBodyMaterializationRequestErrorV3::Storage(
                FrontendStorageRefusalV3::BudgetRejected
            )
        )
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn scalar_request_long_names_preserve_string_schema_and_budget_refusal() {
    use quanta_gqlang_v2_pipeline::{
        materialize_source_body_with_request_owner_v3,
        topologize_source_body_with_request_owner_v3, type_source_body_with_request_owner_v3,
    };
    use quanta_gqlang_v2_surface::{
        FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3,
    };
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    let host_id = format!("app::{}", "records".repeat(24));
    let authority = authority_with_records_type_and_host_id(ValueTypeV1::string(), &host_id);
    for width in [1, 9] {
        let names: Vec<_> = (0..width)
            .map(|n| format!("field_{n}_{}", "x".repeat(96)))
            .collect();
        let projection = names
            .iter()
            .map(|name| format!("r.id as {name}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!("view active | select=from records r | project {projection}");
        let live = Arc::new(AtomicU64::new(0));
        let sink = BodyRequestSinkV3 {
            live: Arc::clone(&live),
            limit: AtomicU64::new(u64::MAX),
            peak: AtomicU64::new(0),
            issuer: Arc::new(()),
        };
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, &source, &sink);
        let bound = bound_inline_view_request_v3(&source, &authority, &root);
        let typed =
            type_source_body_with_request_owner_v3(&source, &root, bound).expect("long-name Types");
        let topology = topologize_source_body_with_request_owner_v3(&source, &root, typed)
            .expect("long-name topology");
        let before = live.load(Ordering::Acquire);
        let body = materialize_source_body_with_request_owner_v3(&source, &root, topology)
            .expect("long-name body");
        let charge = live.load(Ordering::Acquire) - before;
        assert!(charge > 0);
        let borrowed_names: Vec<_> = names.iter().map(String::as_str).collect();
        assert_scalar_body_semantics_v3(
            &source,
            &authority,
            &body,
            &borrowed_names,
            &ValueTypeV1::string(),
        );
        drop(body);
        assert_eq!(live.load(Ordering::Acquire), 0);

        let bound = bound_inline_view_request_v3(&source, &authority, &root);
        let typed = type_source_body_with_request_owner_v3(&source, &root, bound)
            .expect("short-budget Types");
        let topology = topologize_source_body_with_request_owner_v3(&source, &root, typed)
            .expect("short-budget topology");
        sink.limit
            .store(live.load(Ordering::Acquire) + charge - 1, Ordering::Release);
        assert!(matches!(
            materialize_source_body_with_request_owner_v3(&source, &root, topology),
            Err(
                quanta_gqlang_v2_sema::SourceBodyMaterializationRequestErrorV3::Storage(
                    FrontendStorageRefusalV3::BudgetRejected
                )
            )
        ));
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn scalar_request_types_refuse_literal_projection_before_new_storage() {
    use quanta_gqlang_v2_pipeline::type_source_body_with_request_owner_v3;
    use quanta_gqlang_v2_sema::SourceBodyTypesRequestErrorV3;
    use quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let source = "from records r | project 1 as id";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = BodyRequestSinkV3 {
        live: Arc::clone(&live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let before = sink.peak.load(Ordering::Acquire);
    assert!(matches!(
        type_source_body_with_request_owner_v3(source, &root, bound),
        Err(SourceBodyTypesRequestErrorV3::UnsupportedShape)
    ));
    assert_eq!(sink.peak.load(Ordering::Acquire), before);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn scalar_request_body_preserves_authored_projection_order() {
    use quanta_gqlang_v2_pipeline::{
        materialize_source_body_with_request_owner_v3,
        topologize_source_body_with_request_owner_v3, type_source_body_with_request_owner_v3,
    };
    use quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let source = "view active | select=from records r | project r.id as zebra, r.id as alpha";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = BodyRequestSinkV3 {
        live: Arc::clone(&live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let bound = bound_inline_view_request_v3(source, &authority, &root);
    let typed = type_source_body_with_request_owner_v3(source, &root, bound).expect("scalar Types");
    let topology =
        topologize_source_body_with_request_owner_v3(source, &root, typed).expect("View topology");
    let body = materialize_source_body_with_request_owner_v3(source, &root, topology)
        .expect("materialized source order");
    assert_eq!(
        body.output_field_names_v3().collect::<Vec<_>>(),
        ["zebra", "alpha"]
    );
    assert_eq!(body.expression_count_v3(), 4);
    assert!(live.load(Ordering::Acquire) > 0);
    drop(body);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

fn materialized_scalar_request_v3<'authority>(
    source: &str,
    authority: &'authority ClosedFrontendAuthorityV2,
    root: &quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3<'_>,
) -> quanta_gqlang_v2_pipeline::MaterializedSourceBodyRequestSourceV3<'authority> {
    let bound = bound_inline_view_request_v3(source, authority, root);
    let typed =
        quanta_gqlang_v2_pipeline::type_source_body_with_request_owner_v3(source, root, bound)
            .expect("scalar Types");
    let topology = quanta_gqlang_v2_pipeline::topologize_source_body_with_request_owner_v3(
        source, root, typed,
    )
    .expect("same-session scalar topology");
    quanta_gqlang_v2_pipeline::materialize_source_body_with_request_owner_v3(source, root, topology)
        .expect("scalar body")
}

fn assert_request_plan_matches_compiled_v3(
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
    planned: &quanta_gqlang_v2_pipeline::PlannedSourceBodyRequestSourceV3<'_>,
) {
    let compiled = compile_source_document_v3(DOC_SOURCE, source, authority)
        .expect("independent ordinary closed compilation");
    let [item] = compiled.items() else {
        panic!("one authored Query compiles to one item");
    };
    let expected = item.plan().expect("ordinary Query requires a plan");
    assert_eq!(planned.item_id_v3(), item.item_id());
    assert_eq!(planned.result_v3(), expected.result());
    assert_eq!(planned.authority_lane_v3(), expected.authority_lane());
    assert_eq!(
        planned.evaluation_determinism_v3(),
        expected.evaluation_determinism()
    );
    assert_eq!(planned.evidence_v3(), expected.evidence());
    assert_eq!(planned.effect_v3(), expected.effect());
    assert_eq!(&planned.semantic_cost_v3(), expected.semantic_cost());
    assert_eq!(planned.policy_v3(), expected.policy());
    assert_eq!(planned.capability_count_v3(), expected.capabilities().len());
    assert!(expected
        .capabilities()
        .contains(&planned.scan_capability_v3()));
    assert_eq!(planned.explain_families_v3(), expected.explain().families());
    assert_eq!(
        planned.root_location_v3(),
        expected.explain().root_location()
    );
    assert_eq!(
        planned.result_kind_v3(),
        quanta_gqlang_v2_planning::ResultKindV2::Table
    );
    assert_eq!(
        planned.result_v3().render_requirement(),
        quanta_gqlang_v2_planning::RenderRequirementV2::RowsOnly
    );
    assert_eq!(
        planned.scan_capability_v3(),
        quanta_gqlang_v2_planning::CapabilityV2::RelationScan
    );
    assert_eq!(
        planned.explain_families_v3(),
        [
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Project,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan
        ]
    );
    assert!(
        matches!(planned.policy_v3(), quanta_gqlang_v2_planning::PlanPolicyV2::Relational { nodes: 2, effects } if *effects == planned.effect_summary_v3())
    );
}

#[test]
fn scalar_query_and_view_retain_original_request_through_planning() {
    use quanta_gqlang_v2_pipeline::{
        lower_source_body_with_request_owner_v3, plan_source_body_with_request_owner_v3,
        seal_source_catalog_with_request_owner_v3,
    };
    use quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    let authority = authority_with_records();
    for view in [false, true] {
        for width in [1, 9] {
            let names: Vec<_> = (0..width).map(|n| format!("field{n}")).collect();
            let projection = names
                .iter()
                .map(|name| format!("r.id as {name}"))
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!("from records r | project {projection}");
            let source = if view {
                format!("view active | select={query}")
            } else {
                query
            };
            let live = Arc::new(AtomicU64::new(0));
            let sink = BodyRequestSinkV3 {
                live: Arc::clone(&live),
                limit: AtomicU64::new(u64::MAX),
                peak: AtomicU64::new(0),
                issuer: Arc::new(()),
            };
            let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                DOC_SOURCE, &source, &sink,
            );
            let body = materialized_scalar_request_v3(&source, &authority, &root);
            let expected_names: Vec<_> = names.iter().map(String::as_str).collect();
            assert_scalar_body_semantics_v3(
                &source,
                &authority,
                &body,
                &expected_names,
                &ValueTypeV1::int(),
            );
            let catalog = seal_source_catalog_with_request_owner_v3(&source, &root, body)
                .expect("same-session catalog");
            assert_eq!(catalog.catalog_asset_count_v3(), usize::from(view));
            assert_eq!(catalog.catalog_dependency_count_v3(), 0);
            assert_eq!(
                catalog.semantic_kind_v3(),
                if view {
                    quanta_gqlang_v2_sema::AdmittedScalarSemanticKindV3::InlineView
                } else {
                    quanta_gqlang_v2_sema::AdmittedScalarSemanticKindV3::Query
                }
            );
            let retained = live.load(Ordering::Acquire);
            // Lowering and finite classification borrow retained semantic payloads.
            // A ledger limit with no spare bytes must still admit these stages.
            sink.limit.store(retained, Ordering::Release);
            let lowered = lower_source_body_with_request_owner_v3(&source, &root, catalog)
                .expect("checked allocation-free normalization");
            assert_eq!(
                lowered.root_family_v3(),
                quanta_gqlang_v2_algebra::AlgebraFamilyV2::Project
            );
            assert_eq!(lowered.output_field_count_v3(), width);
            assert_eq!(live.load(Ordering::Acquire), retained);
            let planned = plan_source_body_with_request_owner_v3(&source, &root, lowered)
                .expect("checked finite plan");
            assert_eq!(planned.capability_count_v3(), 1);
            assert_eq!(planned.explain_family_count_v3(), 2);
            assert_eq!(live.load(Ordering::Acquire), retained);
            if !view {
                assert_request_plan_matches_compiled_v3(&source, &authority, &planned);
            }
            drop(planned);
            assert_eq!(live.load(Ordering::Acquire), 0);
        }
    }
}

#[test]
fn scalar_continuation_rejects_foreign_root_at_each_owner() {
    use quanta_gqlang_v2_pipeline::{
        lower_source_body_with_request_owner_v3, plan_source_body_with_request_owner_v3,
        seal_source_catalog_with_request_owner_v3, SourceCatalogSealErrorV3,
    };
    use quanta_gqlang_v2_surface::{
        FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3,
    };
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
    let source = "from records r | project r.id as zebra, r.id as alpha";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let foreign_live = Arc::new(AtomicU64::new(0));
    let sink = BodyRequestSinkV3 {
        live: Arc::clone(&live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let foreign_sink = BodyRequestSinkV3 {
        live: Arc::clone(&foreign_live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let foreign = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        source,
        &foreign_sink,
    );
    let body = materialized_scalar_request_v3(source, &authority, &root);
    assert!(matches!(
        seal_source_catalog_with_request_owner_v3(source, &foreign, body),
        Err(SourceCatalogSealErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
    let body = materialized_scalar_request_v3(source, &authority, &root);
    let catalog = seal_source_catalog_with_request_owner_v3(source, &root, body).expect("catalog");
    assert!(matches!(
        lower_source_body_with_request_owner_v3(source, &foreign, catalog),
        Err(
            quanta_gqlang_v2_algebra::ScalarAlgebraRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            )
        )
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
    let body = materialized_scalar_request_v3(source, &authority, &root);
    let catalog = seal_source_catalog_with_request_owner_v3(source, &root, body).expect("catalog");
    let lowered = lower_source_body_with_request_owner_v3(source, &root, catalog).expect("algebra");
    assert!(matches!(
        plan_source_body_with_request_owner_v3(source, &foreign, lowered),
        Err(
            quanta_gqlang_v2_planning::ScalarPlanningRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            )
        )
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_sink.peak.load(Ordering::Acquire), 0);
}
