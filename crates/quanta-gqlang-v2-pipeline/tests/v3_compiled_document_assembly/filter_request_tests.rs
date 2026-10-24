use super::request_body_tests::BodyRequestSinkV3;
use super::*;
use quanta_gqlang_v2_pipeline::{
    compile_scalar_parsed_source_with_request_owner_v3,
    compile_scalar_source_with_request_owner_v3, parse_source_document_with_request_owner_v3,
    project_source_body_with_request_owner_v3, FrontendRequestCheckErrorV3,
    ScalarExecutionRequestErrorV3, ScalarSourceCompileRequestErrorV3,
};
use quanta_gqlang_v2_sema::{FrontendRelationCardinalityV2, FrontendRelationFieldInputV2};
use quanta_gqlang_v2_surface::{FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const FILTER_SOURCE: &str = "from records r | where r.id > 3 | project r.id as id";

fn sink(live: &Arc<AtomicU64>) -> BodyRequestSinkV3 {
    BodyRequestSinkV3 {
        live: Arc::clone(live),
        limit: AtomicU64::new(u64::MAX),
        peak: AtomicU64::new(0),
        issuer: Arc::new(()),
    }
}

fn typed_comparison_authority() -> ClosedFrontendAuthorityV2 {
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
        qualified_name: "app::records".to_owned(),
        lookup_name: "records".to_owned(),
        fields: [
            ("id", FrontendScalarTypeV2::Int),
            ("active", FrontendScalarTypeV2::Bool),
            ("name", FrontendScalarTypeV2::String),
        ]
        .into_iter()
        .map(|(name, scalar_type)| FrontendRelationFieldInputV2 {
            name: name.to_owned(),
            scalar_type,
            sequence: false,
            optional: false,
            location,
        })
        .collect(),
        cardinality: FrontendRelationCardinalityV2::Bag,
        provenance: "test".to_owned(),
        location,
        host_binding_identity: None,
    });
    builder.finish().expect("three typed Host fields")
}

pub(super) fn assert_projection_exact_and_one_short(
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
) {
    let measured_live = Arc::new(AtomicU64::new(0));
    let measured_sink = sink(&measured_live);
    let measured_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        source,
        &measured_sink,
    );
    let planned =
        super::scalar_request_tests::planned_nine_request(source, authority, &measured_root);
    let before = measured_live.load(Ordering::Acquire);
    measured_sink.peak.store(before, Ordering::Release);
    let projected = project_source_body_with_request_owner_v3(source, &measured_root, planned)
        .expect("measure genuine execution projection charge");
    let exact_peak = measured_sink.peak.load(Ordering::Acquire);
    assert!(exact_peak > before);
    drop(projected);
    assert_eq!(measured_live.load(Ordering::Acquire), 0);
    for short in [false, true] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = sink(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
        let planned = super::scalar_request_tests::planned_nine_request(source, authority, &root);
        assert_eq!(
            live.load(Ordering::Acquire),
            before,
            "same-source stage input debit"
        );
        sink.limit
            .store(exact_peak - u64::from(short), Ordering::Release);
        let result = project_source_body_with_request_owner_v3(source, &root, planned);
        if short {
            assert!(matches!(
                result,
                Err(ScalarExecutionRequestErrorV3::Storage(
                    FrontendStorageRefusalV3::BudgetRejected
                ))
            ));
        } else {
            drop(result.expect("exact projection budget admits"));
        }
        assert_eq!(
            live.load(Ordering::Acquire),
            0,
            "failure/success releases every permit"
        );
    }
}

#[test]
fn filter_projection_exact_and_one_short_refunds() {
    assert_projection_exact_and_one_short(FILTER_SOURCE, &authority_with_records());
}

#[test]
fn issued_filter_project_retains_one_predicate_and_matches_ordinary_contract() {
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let sink = sink(&live);
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, FILTER_SOURCE, &sink);
    let admitted =
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, FILTER_SOURCE, &authority, &root)
            .expect("Filter→Project request compiles under one source/root");
    let ordinary = compile_source_document_v3(DOC_SOURCE, FILTER_SOURCE, &authority)
        .expect("ordinary source producer compiles the independent shape");
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
        panic!("one Query item")
    };
    let relation = item.relation().expect("one relation");
    let project = relation.root();
    assert_eq!(project.family(), AlgebraFamilyV2::Project);
    assert_eq!(project.schema().fields().len(), 1);
    assert_eq!(project.schema().fields()[0].name(), "id");
    let [filter] = project.children() else {
        panic!("Project has one Filter child")
    };
    assert_eq!(filter.family(), AlgebraFamilyV2::Filter);
    let [predicate] = filter.predicates() else {
        panic!("exactly one WHERE predicate")
    };
    let predicate_expr = relation
        .expressions()
        .iter()
        .find(|expression| expression.id() == *predicate)
        .expect("WHERE key remains in the closed expression table");
    let ExecutionExpressionShapeV3::Compare { operator, .. } = predicate_expr.shape() else {
        panic!("WHERE must remain a comparison");
    };
    assert_eq!(operator.symbol(), ">");
    assert!(relation
        .expressions()
        .iter()
        .any(|expression| expression.shape()
            == &ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(3))));
    let [scan] = filter.children() else {
        panic!("Filter has one Scan child")
    };
    assert_eq!(scan.family(), AlgebraFamilyV2::Scan);
    assert_eq!(scan.source().expect("host Source").name(), "app::records");
    // Independent row oracle for the authored predicate, not a second frontend invocation.
    let fixture = [-1_i64, 3, 4, 9];
    assert_eq!(
        fixture.into_iter().filter(|id| *id > 3).collect::<Vec<_>>(),
        vec![4, 9]
    );
    let shared = admitted
        .into_shared_with_request_owner_v3(&root, FILTER_SOURCE)
        .expect("retained shared Filter owner");
    let held = shared.clone();
    assert!(shared.shares_payload_v3(&held));
    let read = held
        .relation_read_v3()
        .expect("borrow retained Filter relation");
    let root_read = read.root();
    assert_eq!(root_read.family(), AlgebraFamilyV2::Project);
    assert_eq!(
        root_read.schema().fields().first().expect("output").name(),
        "id"
    );
    let filter_read = root_read.children().first().expect("Filter child");
    assert_eq!(filter_read.family(), AlgebraFamilyV2::Filter);
    let [predicate_read] = filter_read.predicates() else {
        panic!("one borrowed predicate")
    };
    assert!(read
        .expressions()
        .iter()
        .any(|expression| expression.id() == *predicate_read));
    let scan_read = filter_read.children().first().expect("Scan child");
    assert_eq!(scan_read.family(), AlgebraFamilyV2::Scan);
    assert_eq!(
        scan_read.source().expect("Host Source").name(),
        "app::records"
    );
    drop(shared);
    assert!(live.load(Ordering::Acquire) > 0);
    drop(held);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn wide_request_projection_preserves_closed_census_order_and_digest() {
    // The record itself consumes one of the 4,096 type-shape nodes.
    for (width, filtered) in [(257_usize, false), (4_095, true), (4_096, true)] {
        let names = (0..width)
            .map(|index| format!("field{index}"))
            .collect::<Vec<_>>();
        let projection = names
            .iter()
            .map(|name| format!("r.id as {name}"))
            .collect::<Vec<_>>()
            .join(", ");
        let predicate = if filtered { " | where r.id > 3" } else { "" };
        let source = format!("from records r{predicate} | project {projection}");
        let authority = authority_with_records();
        assert!(
            width <= usize::try_from(authority.budget().max_projection_width()).expect("width")
        );
        let live = Arc::new(AtomicU64::new(0));
        let storage = sink(&live);
        let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
            DOC_SOURCE, &source, &storage,
        );
        let result =
            compile_scalar_source_with_request_owner_v3(DOC_SOURCE, &source, &authority, &root);
        if width == 4_096 {
            assert!(matches!(
                result,
                Err(ScalarSourceCompileRequestErrorV3::Type(
                    quanta_gqlang_v2_sema::SourceBodyTypesRequestErrorV3::Diagnostic(_)
                ))
            ));
            assert!(
                live.load(Ordering::Acquire) > 0,
                "the diagnostic retains its permit"
            );
            drop(result);
            assert_eq!(live.load(Ordering::Acquire), 0);
            continue;
        }
        let admitted = result.expect("wide issued projection");
        let ordinary = compile_source_document_v3(DOC_SOURCE, &source, &authority)
            .expect("independent wide projection");
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
        assert_eq!(
            admitted.expression_count_v3(),
            width * 2 + usize::from(filtered) * 4
        );
        let shared = admitted
            .into_shared_with_request_owner_v3(&root, &source)
            .expect("wide shared owner");
        let read = shared.relation_read_v3().expect("wide borrowed relation");
        let fields = read.root().schema().fields();
        assert_eq!(fields.len(), names.len());
        for (field, expected) in fields.iter().zip(&names) {
            assert_eq!(field.name(), expected);
        }
        drop(shared);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn filter_request_refuses_foreign_root_and_source_before_rebinding() {
    let authority = authority_with_records();
    let live = Arc::new(AtomicU64::new(0));
    let owner_sink = sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        FILTER_SOURCE,
        &owner_sink,
    );
    let parsed = parse_source_document_with_request_owner_v3(
        DOC_SOURCE,
        FILTER_SOURCE,
        authority.budget(),
        &root,
    )
    .expect("parser-issued source");
    let foreign_live = Arc::new(AtomicU64::new(0));
    let foreign_sink = sink(&foreign_live);
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        DOC_SOURCE,
        FILTER_SOURCE,
        &foreign_sink,
    );
    assert!(matches!(
        compile_scalar_parsed_source_with_request_owner_v3(
            DOC_SOURCE,
            FILTER_SOURCE,
            &authority,
            &foreign_root,
            parsed,
        ),
        Err(ScalarSourceCompileRequestErrorV3::Check(
            FrontendRequestCheckErrorV3::Root(FrontendStorageRefusalV3::ForeignRequestRoot)
        ))
    ));
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);
    let parsed = parse_source_document_with_request_owner_v3(
        DOC_SOURCE,
        FILTER_SOURCE,
        authority.budget(),
        &root,
    )
    .expect("parser-issued source");
    assert!(matches!(
        compile_scalar_parsed_source_with_request_owner_v3(
            DOC_SOURCE,
            "from records r | where r.id > 4 | project r.id as id",
            &authority,
            &root,
            parsed,
        ),
        Err(ScalarSourceCompileRequestErrorV3::Check(
            FrontendRequestCheckErrorV3::Root(FrontendStorageRefusalV3::ForeignSource)
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn filter_request_rejects_non_boolean_wrong_alias_and_adjacent_filter() {
    let authority = authority_with_records();
    for source in [
        "from records r | where r.id | project r.id as id",
        "from records r | where x.id > 3 | project r.id as id",
        "from records r | where r.id > 3 | where r.id < 9 | project r.id as id",
    ] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = sink(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
        assert!(
            compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
                .is_err(),
            "unsupported or ill-typed Filter must not seal: {source}"
        );
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn boolean_literal_filter_is_valid_ordinary_but_refused_by_finite_request_contract() {
    let source = "from records r | where true | project r.id as id";
    let authority = authority_with_records();
    let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("ordinary frontend accepts a boolean literal Filter");
    let [item] = ordinary.execution().items() else {
        panic!("one Query item")
    };
    let relation = item.relation().expect("ordinary relation");
    let [filter] = relation.root().children() else {
        panic!("Project has one Filter child")
    };
    assert_eq!(filter.family(), AlgebraFamilyV2::Filter);
    assert_eq!(filter.predicates().len(), 1);
    let live = Arc::new(AtomicU64::new(0));
    let sink = sink(&live);
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
    // The bounded request route admits a single alias-qualified field/literal
    // Compare, not a bare Bool literal or arbitrary Boolean grammar.
    assert!(matches!(
        compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root),
        Err(ScalarSourceCompileRequestErrorV3::Binder(
            quanta_gqlang_v2_sema::BinderDeclarationRequestErrorV3::UnsupportedShape
        ))
    ));
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn filter_request_accepts_changed_threshold_and_comparison_without_hardcoding() {
    let authority = authority_with_records();
    for (source, symbol, literal, expected) in [
        (
            "from records r | where r.id > 8 | project r.id as id",
            ">",
            8_i128,
            vec![9],
        ),
        (
            "from records r | where r.id = 4 | project r.id as id",
            "=",
            4_i128,
            vec![4],
        ),
    ] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = sink(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
        let admitted =
            compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
                .expect("another admitted field/literal Compare");
        let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
            .expect("independent ordinary Filter producer");
        assert_eq!(
            admitted.contract_digest_v3(),
            ordinary.execution().contract_digest()
        );
        let [item] = ordinary.execution().items() else {
            panic!("one Query item")
        };
        let relation = item.relation().expect("relation");
        let [filter] = relation.root().children() else {
            panic!("Project→Filter")
        };
        assert_eq!(filter.family(), AlgebraFamilyV2::Filter);
        let [predicate] = filter.predicates() else {
            panic!("one predicate")
        };
        let expression = relation
            .expressions()
            .iter()
            .find(|entry| entry.id() == *predicate)
            .expect("closed predicate expression");
        let ExecutionExpressionShapeV3::Compare { operator, .. } = expression.shape() else {
            panic!("admitted predicate is a Compare");
        };
        assert_eq!(operator.symbol(), symbol);
        assert!(relation.expressions().iter().any(|entry| entry.shape()
            == &ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(literal))));
        let shared = admitted
            .into_shared_with_request_owner_v3(&root, source)
            .expect("retained Filter owner");
        let borrowed = shared
            .relation_read_v3()
            .expect("borrowed admitted relation");
        let filtered = borrowed.root().children().first().expect("Filter child");
        assert_eq!(filtered.family(), AlgebraFamilyV2::Filter);
        assert_eq!(filtered.predicates().len(), 1);
        assert_eq!(
            filtered
                .children()
                .first()
                .expect("Scan")
                .source()
                .expect("Host")
                .name(),
            "app::records"
        );
        let observed = [-1_i128, 3, 4, 9]
            .into_iter()
            .filter(|id| {
                if symbol == ">" {
                    *id > literal
                } else {
                    *id == literal
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(observed, expected, "independent row truth table");
        drop(shared);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn boolean_equality_and_ordered_string_filter_share_typed_request_and_ordinary_dto() {
    let authority = typed_comparison_authority();
    for (source, field, operator, literal, expected) in [
        (
            "from records r | where r.active = true | project r.id as id",
            "active",
            "=",
            ExecutionLiteralV3::Bool(true),
            vec![4_i128, 9],
        ),
        (
            "from records r | where r.name > \"beta\" | project r.id as id",
            "name",
            ">",
            ExecutionLiteralV3::String("beta".to_owned()),
            vec![4_i128, 9],
        ),
    ] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = sink(&live);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(DOC_SOURCE, source, &sink);
        let admitted =
            compile_scalar_source_with_request_owner_v3(DOC_SOURCE, source, &authority, &root)
                .expect("typed field/literal Compare admitted");
        let ordinary = compile_source_document_v3(DOC_SOURCE, source, &authority)
            .expect("ordinary typed comparison producer");
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
            panic!("one Query item")
        };
        let relation = item.relation().expect("one relation");
        let [filter] = relation.root().children() else {
            panic!("Project→Filter")
        };
        assert_eq!(filter.family(), AlgebraFamilyV2::Filter);
        let [predicate] = filter.predicates() else {
            panic!("one Compare predicate")
        };
        let expression = relation
            .expressions()
            .iter()
            .find(|entry| entry.id() == *predicate)
            .expect("closed Compare expression");
        let ExecutionExpressionShapeV3::Compare {
            operator: actual_operator,
            ..
        } = expression.shape()
        else {
            panic!("ordinary predicate is a Compare");
        };
        assert_eq!(actual_operator.symbol(), operator);
        assert!(relation
            .expressions()
            .iter()
            .any(|entry| entry.shape() == &ExecutionExpressionShapeV3::Literal(literal.clone())));
        let shared = admitted
            .into_shared_with_request_owner_v3(&root, source)
            .expect("retained typed Filter owner");
        let read = shared
            .relation_read_v3()
            .expect("borrowed admitted relation");
        let borrowed_filter = read.root().children().first().expect("Filter");
        assert_eq!(borrowed_filter.family(), AlgebraFamilyV2::Filter);
        assert_eq!(borrowed_filter.predicates().len(), 1);
        let scan = borrowed_filter.children().first().expect("Scan");
        assert_eq!(scan.source().expect("Host source").name(), "app::records");
        let fields = scan.schema().fields();
        assert_eq!(fields.len(), 3);
        assert!(fields.iter().any(|entry| entry.name() == field));
        // Fixture truth table is independent of the frontend producer.
        let rows = [
            (-1_i128, false, "alpha"),
            (3, false, "beta"),
            (4, true, "gamma"),
            (9, true, "omega"),
        ];
        let actual = rows
            .into_iter()
            .filter(|(_, active, name)| {
                if field == "active" {
                    *active
                } else {
                    *name > "beta"
                }
            })
            .map(|(id, _, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        drop(shared);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}
