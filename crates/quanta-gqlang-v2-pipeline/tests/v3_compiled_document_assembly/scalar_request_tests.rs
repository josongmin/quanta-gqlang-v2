use super::request_body_tests::BodyRequestSinkV3;
use super::*;
use quanta_gqlang_v2_pipeline::{
    compile_scalar_source_with_request_owner_v3, ExecutionCardinalityV3,
    ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFieldV3, ExecutionItemV3,
    ExecutionRecordFieldV3, ExecutionRelationalOperatorV3, PlannedSourceBodyRequestSourceV3,
    ScalarExecutionRequestErrorV3, ScalarSourceCompileRequestErrorV3, SourceQualifiedNodeIdV2,
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

fn planned_nine_request<'a>(
    source: &str,
    authority: &'a ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> PlannedSourceBodyRequestSourceV3<'a> {
    use quanta_gqlang_v2_pipeline::{
        begin_bind_source_document_with_request_owner_v3,
        begin_prepare_source_body_with_request_owner_v3,
        begin_resolve_source_document_with_request_owner_v3,
        check_source_document_with_request_owner_v3, declare_source_document_with_request_owner_v3,
        lower_source_body_with_request_owner_v3, materialize_source_body_with_request_owner_v3,
        parse_source_document_with_request_owner_v3, plan_source_body_with_request_owner_v3,
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
    let lowered = lower_source_body_with_request_owner_v3(source, root, catalog).expect("algebra");
    plan_source_body_with_request_owner_v3(source, root, lowered).expect("plan")
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
