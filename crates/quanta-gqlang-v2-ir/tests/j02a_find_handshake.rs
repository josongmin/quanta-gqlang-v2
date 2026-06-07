use quanta_gqlang_v2_ir::find_ir::{
    FindExecutionPlanIrV1, FindFlowWindowIrV1, FindIrV1, FindSliceDirectionIrV1,
    FindSummaryFacetIrV1,
};
use quanta_gqlang_v2_ir::lower_document_to_ir_v1;
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;
use quanta_gqlang_v2_sema::analyze_document_v1;
use quanta_gqlang_v2_syntax::parse_document_v1;

fn lower_find_root_v1(input: &str) -> quanta_gqlang_v2_ir::find_ir::FindIrV1 {
    let root = lower_document_to_ir_v1(
        &analyze_document_v1(
            &parse_document_v1(input)
                .unwrap_or_else(|page| panic!("expected parse success for `{input}`: {page:?}")),
        )
        .unwrap_or_else(|page| panic!("expected sema success for `{input}`: {page:?}")),
    )
    .unwrap_or_else(|err| panic!("expected IR lowering success for `{input}`: {err:?}"))
    .into_iter()
    .next()
    .expect("one root");

    let PublicIrRootV1::Find(find) = root else {
        panic!("expected FIND IR root");
    };
    find
}

#[test]
fn find_handshake_keeps_subkind_matrix_explicit() {
    let cases = vec![
        (
            "FIND reachability api.entry -> db.write | via=auth,repo | depth=3",
            FindExecutionPlanIrV1::Reachability(
                quanta_gqlang_v2_ir::find_ir::ReachabilityPlanIrV1 {
                    source: "api.entry".to_string(),
                    sink: "db.write".to_string(),
                    required_waypoints: vec!["auth".to_string(), "repo".to_string()],
                    max_hops: Some(3),
                },
            ),
        ),
        (
            "FIND flow ingress -> egress | through=ssa | window=function",
            FindExecutionPlanIrV1::Flow(quanta_gqlang_v2_ir::find_ir::FlowPlanIrV1 {
                source: "ingress".to_string(),
                sink: "egress".to_string(),
                stages: vec!["ssa".to_string()],
                window: FindFlowWindowIrV1::Function,
            }),
        ),
        (
            "FIND slice callsite.foo | direction=forward | frontier=cfg",
            FindExecutionPlanIrV1::Slice(quanta_gqlang_v2_ir::find_ir::SlicePlanIrV1 {
                anchor: "callsite.foo".to_string(),
                direction: FindSliceDirectionIrV1::Forward,
                frontier: Some("cfg".to_string()),
            }),
        ),
        (
            "FIND taint user.input -> shell.exec | sanitizers=escape | depth=4",
            FindExecutionPlanIrV1::Taint(quanta_gqlang_v2_ir::find_ir::TaintPlanIrV1 {
                source: "user.input".to_string(),
                sink: "shell.exec".to_string(),
                blocked_by: vec!["escape".to_string()],
                max_hops: Some(4),
            }),
        ),
        (
            "FIND path handler -> sink | through=router,middleware | depth=6",
            FindExecutionPlanIrV1::Path(quanta_gqlang_v2_ir::find_ir::PathPlanIrV1 {
                source: "handler".to_string(),
                sink: "sink".to_string(),
                required_waypoints: vec!["router".to_string(), "middleware".to_string()],
                max_hops: Some(6),
            }),
        ),
        (
            "FIND points_to ptr => heap.obj | scope=fn:main",
            FindExecutionPlanIrV1::PointsTo(quanta_gqlang_v2_ir::find_ir::PointsToPlanIrV1 {
                reference: "ptr".to_string(),
                expected_pointee: "heap.obj".to_string(),
                scope: Some("fn:main".to_string()),
            }),
        ),
        (
            "FIND summary auth.check | include=params,returns,effects | depth=2",
            FindExecutionPlanIrV1::Summary(quanta_gqlang_v2_ir::find_ir::SummaryPlanIrV1 {
                callable: "auth.check".to_string(),
                requested_facets: vec![
                    FindSummaryFacetIrV1::Parameters,
                    FindSummaryFacetIrV1::Returns,
                    FindSummaryFacetIrV1::Effects,
                ],
                expansion_depth: Some(2),
            }),
        ),
    ];

    for (input, expected) in cases {
        let find = lower_find_root_v1(input);
        assert_eq!(find.execution_plan(), Ok(expected), "{input}");
    }
}

#[test]
fn find_handshake_rejects_unknown_subkind_fail_closed() {
    let error = FindIrV1 {
        subkind: "mystery".to_string(),
        seed: "anything".to_string(),
    }
    .execution_plan()
    .expect_err("unknown FIND subkind must fail closed");

    assert_eq!(error.message, "unknown FIND subkind `mystery`");
}
