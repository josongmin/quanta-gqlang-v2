use super::request_body_tests::BodyRequestSinkV3;
use super::*;
use quanta_gqlang_v2_pipeline::{
    compile_scalar_source_with_request_owner_v3, ScalarSourceCompileRequestErrorV3,
};
use quanta_gqlang_v2_sema::{
    FrontendRelationCardinalityV2, FrontendRelationFieldInputV2, SourceBodyTypesRequestErrorV3,
};
use quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const EFFECT_SOURCE: &str = "from effect e | project e.row_kind as kind";
const FILTERED_EFFECT_SOURCE: &str =
    "from effect e | where e.scope_id = \"selected\" | project e.row_kind as kind";
const EFFECT_FIELDS: [(&str, bool, bool); 9] = [
    ("row_kind", false, false),
    ("scope_id", false, false),
    ("summary_symbol_id", true, false),
    ("summary_source_scope", true, false),
    ("summary_effect_axes", false, true),
    ("edge_caller_symbol", true, false),
    ("edge_callee_symbol", true, false),
    ("edge_effect_axis", true, false),
    ("degraded_scope", true, false),
];

fn effect_authority_variant(nested_sequence: bool) -> Option<ClosedFrontendAuthorityV2> {
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST_SOURCE,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("closed Host builder");
    builder.add_relation(FrontendRelationInputV2 {
        authority: FrontendRelationAuthorityV2::Host,
        provenance_namespace: "host".to_owned(),
        qualified_name: "app::effect".to_owned(),
        lookup_name: "effect".to_owned(),
        fields: EFFECT_FIELDS
            .iter()
            .map(|(name, optional, sequence)| FrontendRelationFieldInputV2 {
                name: (*name).to_owned(),
                scalar_type: FrontendScalarTypeV2::String,
                sequence: *sequence,
                optional: *optional || (nested_sequence && *name == "summary_effect_axes"),
                location,
            })
            .collect(),
        cardinality: FrontendRelationCardinalityV2::Bag,
        provenance: "test".to_owned(),
        location,
        host_binding_identity: None,
    });
    builder.finish().ok()
}

fn effect_authority() -> ClosedFrontendAuthorityV2 {
    effect_authority_variant(false).expect("nine-field Effect authority")
}

fn sink(live: &Arc<AtomicU64>) -> BodyRequestSinkV3 {
    BodyRequestSinkV3 {
        live: Arc::clone(live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    }
}

#[test]
fn mixed_schema_projection_exact_and_one_short_refunds() {
    super::filter_request_tests::assert_projection_exact_and_one_short(
        EFFECT_SOURCE,
        &effect_authority(),
    );
}

fn assert_effect_schema(scan: &ExecutionRelationalOperatorV3) {
    assert_eq!(scan.family(), AlgebraFamilyV2::Scan);
    let fields = scan.schema().fields();
    assert_eq!(
        fields.len(),
        9,
        "nine *Scan fields*, not nine Project columns"
    );
    for (field, (name, optional, sequence)) in fields.iter().zip(EFFECT_FIELDS) {
        assert_eq!(field.name(), name);
        let expected = if optional {
            ExecutionValueTypeV3::Optional(Box::new([ExecutionValueTypeV3::Scalar(
                ExecutionScalarTypeV3::String,
            )]))
        } else if sequence {
            ExecutionValueTypeV3::Collection {
                element: Box::new([ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String)]),
                cardinality: ExecutionCardinalityV3::Sequence,
            }
        } else {
            ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String)
        };
        assert_eq!(field.value_type(), &expected, "exact field {name}");
    }
    assert_eq!(fields[4].name(), "summary_effect_axes");
}

#[test]
fn mixed_nine_field_scan_keeps_optional_and_sequence_with_scalar_project() {
    let authority = effect_authority();
    let live = Arc::new(AtomicU64::new(0));
    let sink = sink(&live);
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, EFFECT_SOURCE, &sink);
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, EFFECT_SOURCE, &authority, &root)
            .expect("issued mixed Scan + scalar Project");
    let ordinary = compile_source_document_v3(DOC_SOURCE, EFFECT_SOURCE, &authority)
        .expect("ordinary independent compiled document");
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
    let [item] = ordinary.execution().items() else {
        panic!("one Effect Query")
    };
    let project = item.relation().expect("relation").root();
    assert_eq!(project.family(), AlgebraFamilyV2::Project);
    assert_eq!(project.schema().fields().len(), 1);
    assert_eq!(project.schema().fields()[0].name(), "kind");
    assert_eq!(
        project.schema().fields()[0].value_type(),
        &ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String)
    );
    let [scan] = project.children() else {
        panic!("Project→Scan")
    };
    assert_effect_schema(scan);
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, EFFECT_SOURCE)
        .expect("retained mixed row owner");
    let held = shared.clone();
    assert!(shared.shares_payload_v3(&held));
    let borrowed = held.relation_read_v3().expect("borrowed mixed relation");
    let project_read = borrowed.root();
    assert_eq!(project_read.family(), AlgebraFamilyV2::Project);
    assert_eq!(
        project_read
            .schema()
            .fields()
            .first()
            .expect("scalar output")
            .name(),
        "kind"
    );
    let scan_read = project_read.children().first().expect("nine-field Scan");
    assert_eq!(scan_read.family(), AlgebraFamilyV2::Scan);
    let borrowed_fields = scan_read.schema().fields();
    assert_eq!(borrowed_fields.len(), 9);
    for (field, (name, optional, sequence)) in borrowed_fields.iter().zip(EFFECT_FIELDS) {
        assert_eq!(field.name(), name);
        let kind = field.value_type().kind();
        assert_eq!(
            kind,
            if optional {
                "optional"
            } else if sequence {
                "collection"
            } else {
                "scalar"
            }
        );
    }
    assert_eq!(
        borrowed_fields.get(4).expect("Sequence field").name(),
        "summary_effect_axes"
    );
    drop(shared);
    assert!(live.load(Ordering::Acquire) > 0);
    drop(held);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn filtered_mixed_scan_keeps_filter_and_nine_input_fields() {
    let authority = effect_authority();
    let live = Arc::new(AtomicU64::new(0));
    let sink = sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        FILTERED_EFFECT_SOURCE,
        &sink,
    );
    let admitted = compile_scalar_source_with_request_owner_v3(
        DOC_SOURCE,
        FILTERED_EFFECT_SOURCE,
        &authority,
        &root,
    )
    .expect("issued mixed Scan→Filter→scalar Project");
    let ordinary = compile_source_document_v3(DOC_SOURCE, FILTERED_EFFECT_SOURCE, &authority)
        .expect("ordinary mixed Filter producer");
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
    let [item] = ordinary.execution().items() else {
        panic!("one Query")
    };
    let relation = item.relation().expect("relation");
    let [filter] = relation.root().children() else {
        panic!("Project→Filter")
    };
    assert_eq!(filter.family(), AlgebraFamilyV2::Filter);
    assert_eq!(filter.predicates().len(), 1);
    let [scan] = filter.children() else {
        panic!("Filter→Scan")
    };
    assert_effect_schema(scan);
    assert_eq!(
        filter.schema(),
        scan.schema(),
        "Filter retains the complete input schema"
    );
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, FILTERED_EFFECT_SOURCE)
        .expect("retained combined owner");
    let borrowed = shared
        .relation_read_v3()
        .expect("combined borrowed relation");
    let filtered = borrowed.root().children().first().expect("Filter");
    assert_eq!(filtered.family(), AlgebraFamilyV2::Filter);
    assert_eq!(filtered.predicates().len(), 1);
    let borrowed_scan = filtered.children().first().expect("Scan");
    assert_eq!(borrowed_scan.schema().fields().len(), 9);
    assert_eq!(
        borrowed_scan
            .schema()
            .fields()
            .get(4)
            .expect("Sequence")
            .value_type()
            .kind(),
        "collection"
    );
    drop(shared);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn selected_optional_and_sequence_stay_rejected_as_non_scalar_project() {
    let authority = effect_authority();
    for source in [
        "from effect e | project e.summary_effect_axes as axes",
        "from effect e | project e.summary_symbol_id as symbol",
    ] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = sink(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
        assert!(matches!(
            compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root),
            Err(ScalarSourceCompileRequestErrorV3::Type(
                SourceBodyTypesRequestErrorV3::UnsupportedShape
            ))
        ));
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn another_scalar_field_from_mixed_scan_is_not_hardcoded_to_row_kind() {
    let source = "from effect e | project e.scope_id as scope";
    let authority = effect_authority();
    let live = Arc::new(AtomicU64::new(0));
    let sink = sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
            .expect("another scalar field in the same mixed row");
    let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("ordinary scope field route");
    assert_eq!(
        admitted.contract_digest_v3(),
        ordinary.execution().contract_digest()
    );
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, source)
        .expect("retained owner");
    let read = shared.relation_read_v3().expect("borrowed relation");
    assert_eq!(
        read.root()
            .schema()
            .fields()
            .first()
            .expect("output")
            .name(),
        "scope"
    );
    assert_eq!(
        read.root()
            .children()
            .first()
            .expect("Scan")
            .schema()
            .fields()
            .len(),
        9
    );
    drop(shared);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn nested_optional_sequence_is_refused_by_authority_or_request_owner() {
    // This adjacent shape is not one of the nine admitted Effect fields.
    let Some(authority) = effect_authority_variant(true) else {
        return; // Closed authority rejected it before compilation.
    };
    let live = Arc::new(AtomicU64::new(0));
    let sink = sink(&live);
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, EFFECT_SOURCE, &sink);
    assert!(
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, EFFECT_SOURCE, &authority, &root,)
            .is_err(),
        "nested Optional(Sequence(String)) must remain unsupported"
    );
    assert_eq!(live.load(Ordering::Acquire), 0);
}
