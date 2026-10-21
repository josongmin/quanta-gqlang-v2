use super::request_body_tests::BodyRequestSinkV3;
use super::*;
use quanta_gqlang_v2_pipeline::{
    compile_scalar_source_with_request_owner_v3, ExecutionCardinalityV3,
    ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFieldV3, ExecutionItemV3,
    ExecutionRecordFieldV3, ExecutionRelationalOperatorV3, LoweredSourceBodyRequestSourceV3,
    PlannedSourceBodyRequestSourceV3, ScalarExecutionRequestErrorV3,
    ScalarSourceCompileRequestErrorV3, SourceQualifiedNodeIdV2,
};
use quanta_gqlang_v2_surface::{
    FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
    FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

struct TracedScalarSinkV3 {
    live: Arc<AtomicU64>,
    limit: AtomicU64,
    peak: AtomicU64,
    issuer: Arc<()>,
    logical_reservations: Mutex<Vec<u64>>,
}

struct TracedScalarPermitV3 {
    live: Arc<AtomicU64>,
    bytes: u64,
    issuer: Arc<()>,
}

impl Drop for TracedScalarPermitV3 {
    fn drop(&mut self) {
        self.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

impl FrontendRequestStorageSinkV3 for TracedScalarSinkV3 {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let bytes = logical_bytes
            .checked_add(
                u64::try_from(std::mem::size_of::<TracedScalarPermitV3>())
                    .expect("permit size fits u64"),
            )
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.live.load(Ordering::Acquire);
        loop {
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.limit.load(Ordering::Acquire) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self
                .live
                .compare_exchange_weak(live, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.peak.fetch_max(next, Ordering::AcqRel);
                    self.logical_reservations
                        .lock()
                        .expect("trace lock")
                        .push(logical_bytes);
                    break;
                }
                Err(observed) => live = observed,
            }
        }
        Ok(Box::new(TracedScalarPermitV3 {
            live: Arc::clone(&self.live),
            bytes,
            issuer: Arc::clone(&self.issuer),
        }))
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<TracedScalarPermitV3>()
            .is_some_and(|value| Arc::ptr_eq(&value.issuer, &self.issuer))
    }
}

impl TracedScalarSinkV3 {
    fn new(live: &Arc<AtomicU64>) -> Self {
        Self {
            live: Arc::clone(live),
            limit: AtomicU64::new(u64::MAX),
            peak: AtomicU64::new(0),
            issuer: Arc::new(()),
            logical_reservations: Mutex::new(Vec::new()),
        }
    }

    fn reservations(&self) -> Vec<u64> {
        self.logical_reservations
            .lock()
            .expect("trace lock")
            .clone()
    }
}

fn nine_column_source() -> (String, Vec<String>) {
    let names: Vec<_> = (0..9).map(|index| format!("field{index}")).collect();
    let projection = names
        .iter()
        .map(|name| format!("r.id as {name}"))
        .collect::<Vec<_>>()
        .join(", ");
    (format!("from records r | project {projection}"), names)
}

/// Independent logical slot/string calculation for the authored nine-path
/// shape; this does not read the projection owner's demand helper or a peak.
fn nine_column_projection_bytes(names: &[String]) -> u64 {
    let fields = 2 * (1 + names.len()) * std::mem::size_of::<ExecutionFieldV3>();
    let field_names = "id".len() + names.iter().map(String::len).sum::<usize>();
    let project_ids = 2 * names.len() * std::mem::size_of::<SourceQualifiedNodeIdV2>();
    let closure_ids = 2 * names.len() * std::mem::size_of::<SourceQualifiedNodeIdV2>();
    let expressions = 2 * (2 * names.len()) * std::mem::size_of::<ExecutionExpressionV3>();
    let scan_child = 2 * std::mem::size_of::<ExecutionRelationalOperatorV3>();
    let source_names = "host".len() + "app::records".len();
    let per_path = 2 * std::mem::size_of::<ExecutionRecordFieldV3>()
        + "id".len()
        + "r".len()
        + 2 * std::mem::size_of::<String>()
        + "id".len();
    u64::try_from(
        fields
            + field_names
            + project_ids
            + closure_ids
            + expressions
            + scan_child
            + source_names
            + names.len() * per_path,
    )
    .expect("bounded N9 demand fits u64")
}

fn lowered_nine_request<'a>(
    source: &str,
    authority: &'a ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> LoweredSourceBodyRequestSourceV3<'a> {
    use quanta_gqlang_v2_pipeline::{
        begin_bind_source_document_with_request_owner_v3,
        begin_prepare_source_body_with_request_owner_v3,
        begin_resolve_source_document_with_request_owner_v3,
        check_source_document_with_request_owner_v3, declare_source_document_with_request_owner_v3,
        lower_source_body_with_request_owner_v3, materialize_source_body_with_request_owner_v3,
        parse_source_document_with_request_owner_v3,
        predeclare_source_catalog_with_request_owner_v3,
        resolve_source_bindings_with_request_owner_v3, seal_source_catalog_with_request_owner_v3,
        topologize_source_body_with_request_owner_v3, type_source_body_with_request_owner_v3,
    };
    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, authority.budget(), root)
            .expect("parsed source");
    let checked = check_source_document_with_request_owner_v3(DOC_SOURCE, source, root, parsed)
        .expect("checked HIR");
    let declared = declare_source_document_with_request_owner_v3(source, authority, root, checked)
        .expect("declared source");
    let session = begin_resolve_source_document_with_request_owner_v3(source, root, declared)
        .expect("Modules session");
    let predeclared = predeclare_source_catalog_with_request_owner_v3(source, root, session)
        .expect("predeclared catalog");
    let preparing = begin_prepare_source_body_with_request_owner_v3(source, root, predeclared)
        .expect("builtins");
    let declaring = begin_bind_source_document_with_request_owner_v3(source, root, preparing)
        .expect("Binder declarations");
    let resolved = resolve_source_bindings_with_request_owner_v3(source, root, declaring)
        .expect("Binder resolution");
    let typed = type_source_body_with_request_owner_v3(source, root, resolved).expect("Types");
    let topology =
        topologize_source_body_with_request_owner_v3(source, root, typed).expect("topology");
    let materialized =
        materialize_source_body_with_request_owner_v3(source, root, topology).expect("body");
    let catalog =
        seal_source_catalog_with_request_owner_v3(source, root, materialized).expect("catalog");
    lower_source_body_with_request_owner_v3(source, root, catalog).expect("algebra")
}

fn planned_nine_request<'a>(
    source: &str,
    authority: &'a ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> PlannedSourceBodyRequestSourceV3<'a> {
    quanta_gqlang_v2_pipeline::plan_source_body_with_request_owner_v3(
        source,
        root,
        lowered_nine_request(source, authority, root),
    )
    .expect("plan")
}

fn new_sink(live: &Arc<AtomicU64>) -> BodyRequestSinkV3 {
    BodyRequestSinkV3 {
        live: Arc::clone(live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    }
}

#[test]
fn scalar_request_execution_matches_independent_v2_document_and_source_bytes() {
    let source = RECORD_SOURCE;
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = new_sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
            .expect("one bare Query compiles through all request owners");
    let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("independent ordinary V2 semantic and execution route");
    assert_eq!(
        admitted.source_digest_v3().to_hex(),
        "08a0e064657b169fe72e81a96f3a75ba1ed43cf8d15be162a90265f9879ee701"
    );
    assert_eq!(
        admitted.contract_digest_v3().to_hex(),
        "d37d2ce066033a5b19fde37cae5cad3fa1a7bff55fdcf5add092b2a0f5bff98f"
    );
    assert_eq!(
        admitted.qualification_digest_v3().to_hex(),
        "f89812e2c1eb69010120219859924287072602c3b10b5b390aa16de00b8aeada"
    );
    assert_eq!(
        admitted.source_digest_v3(),
        ordinary.execution().source_digest()
    );
    assert_eq!(
        admitted.contract_digest_v3(),
        ordinary.execution().contract_digest()
    );
    assert_eq!(
        admitted.qualification_digest_v3(),
        ordinary.execution().qualification_digest()
    );
    assert_eq!(admitted.item_family_v3(), ExecutionFamilyV3::Relation);
    assert_eq!(admitted.expression_count_v3(), 2);
    assert!(admitted.matches_source_bytes_v3(source.as_bytes()));
    assert!(!admitted.matches_source_bytes_v3(b"from records r | project r.id as other"));
    assert!(live.load(Ordering::Acquire) > 0);
    drop(admitted);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn parsed_scalar_owner_classifies_and_continues_without_changing_n1_n9_execution() {
    use quanta_gqlang_v2_pipeline::{
        compile_scalar_parsed_source_with_request_owner_v3,
        parse_source_document_with_request_owner_v3, FrontendSourceRootFamilyV3,
    };

    let (nine_source, _) = nine_column_source();
    let authority = authority_with_records();
    for (source, expression_count) in [(RECORD_SOURCE, 2), (nine_source.as_str(), 18)] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = TracedScalarSinkV3::new(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
        let parsed = parse_source_document_with_request_owner_v3(
            DOC_SOURCE,
            source,
            authority.budget(),
            &root,
        )
        .expect("one parser-issued source owner");
        assert_eq!(parsed.item_count_v3(), 1);
        assert_eq!(
            parsed.root_family_v3().expect("one classified Query root"),
            FrontendSourceRootFamilyV3::Relation
        );
        let parsed_reservations = sink.reservations().len();
        let admitted = compile_scalar_parsed_source_with_request_owner_v3(
            DOC_SOURCE, source, &authority, &root, parsed,
        )
        .expect("continue the classified parser owner");
        let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
            .expect("ordinary producer is an independent DTO oracle");
        assert_eq!(admitted.expression_count_v3(), expression_count);
        assert_eq!(
            admitted.source_digest_v3(),
            ordinary.execution().source_digest()
        );
        assert_eq!(
            admitted.contract_digest_v3(),
            ordinary.execution().contract_digest()
        );
        assert_eq!(
            admitted.qualification_digest_v3(),
            ordinary.execution().qualification_digest()
        );
        assert!(admitted.matches_source_bytes_v3(source.as_bytes()));
        assert!(sink.reservations().len() > parsed_reservations);
        drop(admitted);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn parsed_onelet_composition_retains_one_request_root_and_refuses_foreign_or_short_funding() {
    use quanta_gqlang_v2_pipeline::{
        compile_source_parsed_with_request_owner_v3, parse_source_document_with_request_owner_v3,
        AdmittedSourceExecutionDocumentV3, FrontendSourceRootFamilyV3,
    };

    let source = "let rows = from records r | project r.id as id \
                  from rows x | project x.id as id with rows";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = TracedScalarSinkV3::new(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, authority.budget(), &root)
            .expect("one parser-issued Composition source");
    assert_eq!(parsed.item_count_v3(), 1);
    assert_eq!(
        parsed
            .root_family_v3()
            .expect("one classified Composition root"),
        FrontendSourceRootFamilyV3::Composition
    );
    let admitted = match compile_source_parsed_with_request_owner_v3(
        DOC_SOURCE, source, &authority, &root, parsed,
    )
    .expect("onelet follows the same request owner through execution seal")
    {
        AdmittedSourceExecutionDocumentV3::OneLet(value) => value,
        AdmittedSourceExecutionDocumentV3::Scalar(_) => {
            panic!("Composition cannot be published as a scalar capability")
        }
    };
    let compile_peak = sink.peak.load(Ordering::Acquire);
    let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("independent canonical execution DTO oracle");
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, source)
        .expect("share the genuine retained onelet owner");
    assert_eq!(
        shared.source_digest_v3(),
        ordinary.execution().source_digest()
    );
    assert_eq!(
        shared.contract_digest_v3(),
        ordinary.execution().contract_digest()
    );
    assert_eq!(
        shared.qualification_digest_v3(),
        ordinary.execution().qualification_digest()
    );
    let borrowed = shared
        .composition_read_v3()
        .expect("the admitted owner lends one Composition read");
    assert_eq!(borrowed.binding_count_v3(), 1);
    let binding = borrowed.binding_at_v3(0).expect("one retained LET");
    assert_eq!(binding.name_v3(), "rows");
    assert!(binding.matches_with_binding_v3());
    assert_eq!(binding.sample_v3(), None);

    let foreign_live = Arc::new(AtomicU64::new(0));
    let foreign_sink = TracedScalarSinkV3::new(&foreign_live);
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        source,
        &foreign_sink,
    );
    assert_eq!(
        shared.validate_owner_v3(&foreign_root, source),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    assert_eq!(
        shared.validate_issued_sink_v3(&foreign_sink),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    let held = shared.clone();
    drop(shared);
    assert!(live.load(Ordering::Acquire) > 0);
    drop(held);
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);

    let short_live = Arc::new(AtomicU64::new(0));
    let short_sink = TracedScalarSinkV3::new(&short_live);
    short_sink.limit.store(compile_peak - 1, Ordering::Release);
    let short_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &short_sink);
    let short_parsed = parse_source_document_with_request_owner_v3(
        DOC_SOURCE,
        source,
        authority.budget(),
        &short_root,
    )
    .expect("one-short projection budget still admits the same parser source");
    assert!(matches!(
        compile_source_parsed_with_request_owner_v3(
            DOC_SOURCE,
            source,
            &authority,
            &short_root,
            short_parsed,
        ),
        Err(ScalarSourceCompileRequestErrorV3::Execution(
            ScalarExecutionRequestErrorV3::Storage(FrontendStorageRefusalV3::BudgetRejected)
        ))
    ));
    assert_eq!(short_live.load(Ordering::Acquire), 0);
}

#[test]
fn admitted_onelet_explain_walk_matches_ordinary_body_plan() {
    let source = "let rows = from records r | project r.id as id \
                  from rows x | project x.id as id with rows";
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = new_sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let admitted = planned_nine_request(source, &authority, &root);
    let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("independent ordinary Composition plan");
    let [item] = ordinary.items() else {
        panic!("one Composition plan item");
    };
    let ordinary_plan = item.plan().expect("ordinary body plan");
    assert_eq!(admitted.item_id_v3(), item.item_id());
    assert_eq!(admitted.policy_v3(), ordinary_plan.policy());
    let PlanPolicyV2::Composition(admitted_policy) = admitted.policy_v3() else {
        panic!("one-LET root must retain a Composition policy");
    };
    let PlanPolicyV2::Composition(ordinary_policy) = ordinary_plan.policy() else {
        panic!("ordinary one-LET root has a Composition policy");
    };
    assert_eq!(admitted_policy.root_item_id(), item.item_id());
    assert_eq!(admitted_policy.location(), ordinary_policy.location());
    assert_eq!(admitted_policy.producers(), ordinary_policy.producers());
    assert_eq!(admitted_policy.producers().len(), 1);
    assert_eq!(
        admitted_policy.producers()[0].relation_output(),
        ordinary_policy.producers()[0].relation_output()
    );
    assert_eq!(
        admitted_policy.with_binding(),
        ordinary_policy.with_binding()
    );
    assert_eq!(
        admitted_policy.body_item_id(),
        ordinary_policy.body_item_id()
    );
    assert_eq!(admitted_policy.body_family(), ordinary_policy.body_family());
    assert_eq!(admitted_policy.body(), ordinary_policy.body());
    assert_eq!(admitted.result_v3(), ordinary_plan.result());
    assert_eq!(admitted.semantic_cost_v3(), *ordinary_plan.semantic_cost());
    assert_eq!(
        admitted.evaluation_determinism_v3(),
        ordinary_plan.evaluation_determinism()
    );
    assert_eq!(
        admitted.root_location_v3(),
        ordinary_plan.explain().root_location()
    );
    assert_eq!(
        admitted.explain_families_v3(),
        ordinary_plan.explain().families()
    );
    assert_eq!(
        admitted.explain_family_count_v3(),
        ordinary_plan.explain().families().len()
    );
    drop(admitted);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn admitted_onelet_policy_reserves_exact_new_owner_and_refuses_one_short() {
    use quanta_gqlang_v2_pipeline::plan_source_body_with_request_owner_v3;
    use quanta_gqlang_v2_planning::ScalarPlanningRequestErrorV3;

    let source = "let rows = from records r | project r.id as id \
                  from rows x | project x.id as id with rows";
    let authority = authority_with_records();
    // Independent fixed-shape calculation: one producer slot, one cloned
    // output column, one boxed body policy, and exact singleton source,
    // output-column and WITH name lengths. No BTree clone is admitted.
    let expected_demand = std::mem::size_of::<
        quanta_gqlang_v2_planning::CheckedCompositionProducerV2,
    >() + std::mem::size_of::<quanta_gqlang_v2_sema::OutputColumnContractV2>(
    ) + std::mem::size_of::<PlanPolicyV2>()
        + "host".len()
        + "app::records".len()
        + "id".len()
        + "rows".len();
    let live = Arc::new(AtomicU64::new(0));
    let sink = TracedScalarSinkV3::new(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let lowered = lowered_nine_request(source, &authority, &root);
    let before = sink.reservations().len();
    let pre_plan_live = live.load(Ordering::Acquire);
    let planned = plan_source_body_with_request_owner_v3(source, &root, lowered)
        .expect("onelet plan admits its policy storage");
    let reservations = sink.reservations();
    assert_eq!(&reservations[before..], &[expected_demand as u64]);
    let charge = expected_demand as u64 + std::mem::size_of::<TracedScalarPermitV3>() as u64;
    assert_eq!(live.load(Ordering::Acquire), pre_plan_live + charge);
    drop(planned);
    assert_eq!(live.load(Ordering::Acquire), 0);

    let short_live = Arc::new(AtomicU64::new(0));
    let short_sink = TracedScalarSinkV3::new(&short_live);
    let short_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &short_sink);
    let lowered = lowered_nine_request(source, &authority, &short_root);
    let short_pre_plan = short_live.load(Ordering::Acquire);
    let short_count = short_sink.reservations().len();
    short_sink
        .limit
        .store(short_pre_plan + charge - 1, Ordering::Release);
    assert!(matches!(
        plan_source_body_with_request_owner_v3(source, &short_root, lowered),
        Err(ScalarPlanningRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    assert_eq!(short_sink.reservations().len(), short_count);
    assert_eq!(short_live.load(Ordering::Acquire), 0);
}

#[test]
fn admitted_onelet_lends_project_scan_facts_without_nested_dto() {
    use quanta_gqlang_v2_pipeline::{
        compile_source_parsed_with_request_owner_v3, parse_source_document_with_request_owner_v3,
        AdmittedSourceExecutionDocumentV3, ExecutionRelationAuthorityV3,
    };
    let source = "let paths = from files f | project f.path as path \
                  from paths p | project p.path as path with paths";
    let authority =
        authority_with_workspace_files(FrontendRelationAuthorityV2::CanonicalWorkspaceFiles);
    let live = Arc::new(AtomicU64::new(0));
    let sink = new_sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, authority.budget(), &root)
            .expect("issued onelet parser owner");
    let admitted = match compile_source_parsed_with_request_owner_v3(
        DOC_SOURCE, source, &authority, &root, parsed,
    )
    .expect("same-owner onelet seal")
    {
        AdmittedSourceExecutionDocumentV3::OneLet(value) => value,
        AdmittedSourceExecutionDocumentV3::Scalar(_) => panic!("onelet is not scalar"),
    };
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, source)
        .expect("shared onelet owner");
    let read = shared
        .composition_read_v3()
        .expect("opaque Composition read");
    assert_eq!(read.binding_count_v3(), 1);
    let binding = read.binding_at_v3(0).expect("one LET producer");
    assert!(binding.matches_with_binding_v3());
    let producer = binding
        .producer_project_scan_v3()
        .expect("issued producer shape");
    assert_eq!(
        producer.source_authority_v3(),
        ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
    );
    assert!(producer.source_binding_v3().is_none());
    let body = read.body_project_scan_v3().expect("issued body shape");
    assert_eq!(
        body.source_authority_v3(),
        ExecutionRelationAuthorityV3::CompositionBinding
    );
    assert!(body
        .source_binding_v3()
        .expect("sealed WITH source")
        .matches_v3(read.with_binding_v3()));
    drop(shared);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn parsed_scalar_owner_refuses_budget_and_foreign_inputs_before_next_debit() {
    use quanta_gqlang_v2_pipeline::{
        compile_scalar_parsed_source_with_request_owner_v3,
        parse_source_document_with_request_owner_v3, FrontendRequestCheckErrorV3,
    };
    let source = RECORD_SOURCE;
    let authority = authority_with_records();

    let live = Arc::new(AtomicU64::new(0));
    let sink = TracedScalarSinkV3::new(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let alternate_budget = FrontendBudgetV2::standard_v2()
        .expect("valid standard budget")
        .try_with_rule_program_rules(1)
        .expect("valid different rule cap");
    assert_ne!(&alternate_budget, authority.budget());
    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, &alternate_budget, &root)
            .expect("parse-only budget admits this Query");
    let reservations = sink.reservations();
    let peak = sink.peak.load(Ordering::Acquire);
    assert!(matches!(
        compile_scalar_parsed_source_with_request_owner_v3(
            DOC_SOURCE, source, &authority, &root, parsed,
        ),
        Err(ScalarSourceCompileRequestErrorV3::ParsedBudgetMismatch)
    ));
    assert_eq!(sink.reservations(), reservations);
    assert_eq!(sink.peak.load(Ordering::Acquire), peak);
    assert_eq!(live.load(Ordering::Acquire), 0);

    let foreign_live = Arc::new(AtomicU64::new(0));
    let foreign_sink = TracedScalarSinkV3::new(&foreign_live);
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        source,
        &foreign_sink,
    );
    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, authority.budget(), &root)
            .expect("original root parses");
    let original_reservations = sink.reservations();
    assert!(matches!(
        compile_scalar_parsed_source_with_request_owner_v3(
            DOC_SOURCE,
            source,
            &authority,
            &foreign_root,
            parsed,
        ),
        Err(ScalarSourceCompileRequestErrorV3::Check(
            FrontendRequestCheckErrorV3::Root(FrontendStorageRefusalV3::ForeignRequestRoot)
        ))
    ));
    assert_eq!(sink.reservations(), original_reservations);
    assert!(foreign_sink.reservations().is_empty());
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);

    let parsed =
        parse_source_document_with_request_owner_v3(DOC_SOURCE, source, authority.budget(), &root)
            .expect("original source parses");
    let original_reservations = sink.reservations();
    let different_source = "from records r | project r.id as different";
    assert!(matches!(
        compile_scalar_parsed_source_with_request_owner_v3(
            DOC_SOURCE,
            different_source,
            &authority,
            &root,
            parsed,
        ),
        Err(ScalarSourceCompileRequestErrorV3::Check(
            FrontendRequestCheckErrorV3::Root(FrontendStorageRefusalV3::ForeignSource)
        ))
    ));
    assert_eq!(sink.reservations(), original_reservations);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn nine_projection_execution_preserves_order_digests_and_independent_stage_debits() {
    let (source, names) = nine_column_source();
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = TracedScalarSinkV3::new(&live);
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, &source, &sink);
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, &source, &authority, &root)
            .expect("nine authored scalar projections");
    let ordinary = compile_source_document_v3(DOC_SOURCE, &source, &authority)
        .expect("independent ordinary execution producer");
    assert_eq!(
        admitted.source_digest_v3(),
        ordinary.execution().source_digest()
    );
    assert_eq!(
        admitted.contract_digest_v3(),
        ordinary.execution().contract_digest()
    );
    assert_eq!(
        admitted.qualification_digest_v3(),
        ordinary.execution().qualification_digest()
    );
    assert_eq!(admitted.item_family_v3(), ExecutionFamilyV3::Relation);
    assert_eq!(admitted.expression_count_v3(), 18);
    assert!(admitted.matches_source_bytes_v3(source.as_bytes()));

    let [item] = ordinary.execution().items() else {
        panic!("one authored Query must produce one execution item");
    };
    let relation = item.relation().expect("one relation payload");
    let project = relation.root();
    let [scan] = project.children() else {
        panic!("Project must have one Scan child");
    };
    let scan_source = scan.source().expect("host Scan has a source");
    assert_eq!(scan_source.namespace(), Some("host"));
    assert_eq!(scan_source.name(), "app::records");
    assert_eq!(project.family(), AlgebraFamilyV2::Project);
    assert_eq!(scan.family(), AlgebraFamilyV2::Scan);
    assert_eq!(project.schema().fields().len(), names.len());
    assert_eq!(scan.schema().fields()[0].name(), "id");
    assert_eq!(relation.expressions().len(), 18);
    for (name, (field, key)) in names
        .iter()
        .zip(project.schema().fields().iter().zip(project.expressions()))
    {
        assert_eq!(field.name(), name);
        assert_eq!(
            field.value_type(),
            &ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int)
        );
        let expr = relation
            .expressions()
            .iter()
            .find(|expr| expr.id() == *key)
            .expect("authored Project ID has an expression row");
        let ExecutionExpressionShapeV3::Path { base, segments } = expr.shape() else {
            panic!("each projection is one field path");
        };
        assert_eq!(segments.as_ref(), &[String::from("id")]);
        assert!(relation
            .expressions()
            .iter()
            .any(|entry| entry.id() == *base));
    }
    assert!(relation
        .expressions()
        .windows(2)
        .all(|pair| pair[0].id() < pair[1].id()));
    let reservations = sink.reservations();
    assert!(reservations.len() >= 2);
    assert_eq!(
        reservations[reservations.len() - 2],
        nine_column_projection_bytes(&names)
    );
    assert_eq!(
        reservations[reservations.len() - 1],
        u64::try_from(2 * std::mem::size_of::<ExecutionItemV3>()).expect("item slots fit u64")
    );
    assert!(live.load(Ordering::Acquire) > 0);
    drop(admitted);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn nine_projection_and_document_seal_obey_exact_and_one_byte_short_debits() {
    use quanta_gqlang_v2_pipeline::{
        project_source_body_with_request_owner_v3,
        seal_scalar_execution_document_with_request_owner_v3,
    };
    let (source, names) = nine_column_source();
    let authority = authority_with_records();
    let projection_bytes = nine_column_projection_bytes(&names);
    let document_bytes =
        u64::try_from(2 * std::mem::size_of::<ExecutionItemV3>()).expect("item slots fit u64");
    let permit_bytes =
        u64::try_from(std::mem::size_of::<TracedScalarPermitV3>()).expect("permit size fits u64");
    for short in [false, true] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = TracedScalarSinkV3::new(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, &source, &sink);
        let planned = planned_nine_request(&source, &authority, &root);
        let before = live.load(Ordering::Acquire);
        let prior_trace = sink.reservations();
        sink.limit.store(
            before + projection_bytes + permit_bytes - u64::from(short),
            Ordering::Release,
        );
        let projected = project_source_body_with_request_owner_v3(&source, &root, planned);
        if short {
            assert!(matches!(
                projected,
                Err(ScalarExecutionRequestErrorV3::Storage(
                    FrontendStorageRefusalV3::BudgetRejected
                ))
            ));
            assert_eq!(sink.reservations(), prior_trace);
            assert_eq!(live.load(Ordering::Acquire), 0);
            continue;
        }
        let projected = projected.expect("exact projection debit admits");
        assert_eq!(sink.reservations().last(), Some(&projection_bytes));
        sink.limit.store(u64::MAX, Ordering::Release);
        drop(projected);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
    for short in [false, true] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = TracedScalarSinkV3::new(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, &source, &sink);
        let planned = planned_nine_request(&source, &authority, &root);
        let projected = project_source_body_with_request_owner_v3(&source, &root, planned)
            .expect("bounded projection");
        let before = live.load(Ordering::Acquire);
        let prior_trace = sink.reservations();
        sink.limit.store(
            before + document_bytes + permit_bytes - u64::from(short),
            Ordering::Release,
        );
        let sealed =
            seal_scalar_execution_document_with_request_owner_v3(&source, &root, projected);
        if short {
            assert!(matches!(
                sealed,
                Err(ScalarExecutionRequestErrorV3::Storage(
                    FrontendStorageRefusalV3::BudgetRejected
                ))
            ));
            assert_eq!(sink.reservations(), prior_trace);
            assert_eq!(live.load(Ordering::Acquire), 0);
            continue;
        }
        let sealed = sealed.expect("exact document Vec/Box debit admits");
        assert_eq!(sink.reservations().last(), Some(&document_bytes));
        assert_eq!(sealed.expression_count_v3(), 18);
        drop(sealed);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn canonical_files_read_is_borrowed_and_shared_owner_releases_at_last_drop() {
    let source = WORKSPACE_FILES_SOURCE;
    let authority =
        authority_with_workspace_files(FrontendRelationAuthorityV2::CanonicalWorkspaceFiles);
    let live = Arc::new(AtomicU64::new(0));
    let sink = new_sink(&live);
    let foreign_live = Arc::new(AtomicU64::new(0));
    let foreign_sink = new_sink(&foreign_live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        source,
        &foreign_sink,
    );
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
            .expect("canonical files Query compiles");
    assert_eq!(admitted.validate_issued_sink_v3(&sink), Ok(()));
    assert_eq!(
        admitted.validate_issued_sink_v3(&foreign_sink),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    assert_eq!(
        admitted.validate_owner_v3(&foreign_root, source),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    let before_share = live.load(Ordering::Acquire);
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, source)
        .expect("shared carrier retains the issued request");
    assert_eq!(shared.validate_issued_sink_v3(&sink), Ok(()));
    assert_eq!(
        shared.validate_issued_sink_v3(&foreign_sink),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    let share_charge = live.load(Ordering::Acquire) - before_share;
    assert!(share_charge > 0);
    let facts = shared
        .canonical_files_scalar_read_v3()
        .expect("closed canonical Scan Project");
    assert_eq!(facts.item_id().source_id(), DOC_SOURCE);
    assert_eq!(facts.source_namespace(), "builtin");
    assert_eq!(facts.source_name(), "workspace_files");
    assert_eq!(facts.scan_field_name(), "path");
    assert_eq!(facts.output_field_name(), "path");
    assert_eq!(facts.scan_cardinality(), ExecutionCardinalityV3::Set);
    assert_eq!(facts.output_cardinality(), ExecutionCardinalityV3::Set);
    assert_eq!(facts.binding_source_id(), DOC_SOURCE);
    assert_eq!(facts.binding_ordinal(), 0);
    let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("independent ordinary execution document");
    let ordinary_scan = &ordinary.execution().items()[0]
        .relation()
        .expect("relation item")
        .root()
        .children()[0];
    assert_eq!(
        facts.binding_owner_node(),
        ordinary_scan.stage().node_id.raw()
    );
    assert_eq!(
        shared.validate_owner_v3(&foreign_root, source),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    let clone = shared.clone();
    assert!(shared.shares_payload_v3(&clone));
    drop(shared);
    assert!(live.load(Ordering::Acquire) > 0);
    drop(clone);
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);

    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
            .expect("second exact owner");
    sink.limit.store(
        live.load(Ordering::Acquire) + share_charge - 1,
        Ordering::Release,
    );
    assert!(matches!(
        admitted.into_shared_with_request_owner_v3(&root, source),
        Err(ScalarExecutionRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn scalar_request_refuses_other_roots_and_one_byte_short_budget() {
    let source = RECORD_SOURCE;
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = new_sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    for unsupported in [
        "view active | select=from records r | project r.id as id",
        "from records r | project r.id as id; from records r | project r.id as id",
    ] {
        let unsupported_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
            DOC_SOURCE,
            unsupported,
            &sink,
        );
        assert!(matches!(
            compile_scalar_source_with_request_owner_v3(
                DOC_SOURCE,
                unsupported,
                &authority,
                &unsupported_root
            ),
            Err(ScalarSourceCompileRequestErrorV3::UnsupportedRoot)
        ));
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
            .expect("exact source admits");
    let peak = sink.peak.load(Ordering::Acquire);
    drop(admitted);
    assert_eq!(live.load(Ordering::Acquire), 0);
    sink.limit.store(peak - 1, Ordering::Release);
    assert!(matches!(
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root),
        Err(ScalarSourceCompileRequestErrorV3::Execution(
            ScalarExecutionRequestErrorV3::Storage(FrontendStorageRefusalV3::BudgetRejected)
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
}
