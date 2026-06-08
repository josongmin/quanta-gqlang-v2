#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::check_ir::CheckExecutionModeV1;
use quanta_gqlang_v2_ir::explain_ir::{ExplainEvidenceModeIrV1, ExplainTruthExpectationIrV1};
use quanta_gqlang_v2_ir::find_ir::{FindExecutionPlanIrV1, FindFlowWindowIrV1};
use quanta_gqlang_v2_ir::root_ir::{
    CatalogCategoryIrV1, CompareMetricIrV1, DeclarationIrV1, ExportKindIrV1, InteractiveIrV1,
    InteractiveTargetKindIrV1, MetaIrV1, PackActivationIrV1, ProfileAdmissionIrV1, PublicIrRootV1,
    ViewQueryIrV1,
};
use quanta_gqlang_v2_ir::{lower_document_to_ir_v1, select_ir::SelectProjectionIrV1};
use quanta_gqlang_v2_sema::analyze_document_v1;
use quanta_gqlang_v2_syntax::parse_document_v1;

fn lower_root_v1(input: &str) -> PublicIrRootV1 {
    lower_document_to_ir_v1(
        &analyze_document_v1(
            &parse_document_v1(input)
                .unwrap_or_else(|page| panic!("expected parse success for `{input}`: {page:?}")),
        )
        .unwrap_or_else(|page| panic!("expected sema success for `{input}`: {page:?}")),
    )
    .unwrap_or_else(|err| panic!("expected IR lowering success for `{input}`: {err:?}"))
    .into_iter()
    .next()
    .expect("one root")
}

#[test]
fn core_roots_lower_to_expected_public_ir_v1() {
    let select =
        lower_root_v1("SELECT id, path FROM catalog.files WHERE severity = 'high' LIMIT 10");
    let PublicIrRootV1::Select(select) = select else {
        panic!("expected SELECT IR root");
    };
    let shape = select.shape_v1().expect("SELECT shape");
    assert_eq!(shape.table.dotted_name_v1(), "catalog.files");
    assert!(matches!(shape.projection, SelectProjectionIrV1::Fields(_)));
    assert_eq!(shape.limit, Some(10));

    let find = lower_root_v1("FIND flow ingress -> egress | through=ssa | window=function");
    let PublicIrRootV1::Find(find) = find else {
        panic!("expected FIND IR root");
    };
    assert_eq!(
        find.execution_plan(),
        Ok(FindExecutionPlanIrV1::Flow(
            quanta_gqlang_v2_ir::find_ir::FlowPlanIrV1 {
                source: "ingress".to_string(),
                sink: "egress".to_string(),
                stages: vec!["ssa".to_string()],
                window: FindFlowWindowIrV1::Function,
            },
        ))
    );

    let check = lower_root_v1(
        "CHECK rule.hybrid check:v1|class=security|scope=repo%3A%2F%2Fruntime|pattern=call%20%3D%20%22std%3A%3Aprocess%3A%3ACommand%22|confirm=require",
    );
    let PublicIrRootV1::Check(check) = check else {
        panic!("expected CHECK IR root");
    };
    assert_eq!(
        check.execution_mode().expect("CHECK mode"),
        CheckExecutionModeV1::Hybrid
    );

    let explain =
        lower_root_v1("EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm");
    let PublicIrRootV1::Explain(explain) = explain else {
        panic!("expected EXPLAIN IR root");
    };
    let shape = explain.shape().expect("EXPLAIN shape");
    assert_eq!(shape.target, "rule::owasp.sql-injection");
    assert_eq!(shape.evidence, ExplainEvidenceModeIrV1::Trace);
    assert_eq!(
        shape.truth_expectation,
        ExplainTruthExpectationIrV1::Confirm
    );
}

#[test]
fn interactive_roots_lower_to_public_frontend_shapes_v1() {
    let show = lower_root_v1("SHOW capabilities");
    assert_eq!(
        show,
        PublicIrRootV1::Interactive(InteractiveIrV1::Show {
            category: CatalogCategoryIrV1::Capabilities,
        })
    );

    let describe = lower_root_v1("DESCRIBE rule::owasp.sql-injection");
    assert_eq!(
        describe,
        PublicIrRootV1::Interactive(InteractiveIrV1::Describe {
            target: quanta_gqlang_v2_ir::root_ir::InteractiveTargetIrV1 {
                kind: InteractiveTargetKindIrV1::Rule,
                path: vec!["owasp".to_string(), "sql-injection".to_string()],
            },
        })
    );

    let sample = lower_root_v1(
        "SAMPLE query=callgraph entrypoints ;; scope=repo::frontend ;; mode=semantic ;; limit=5",
    );
    assert!(matches!(sample, PublicIrRootV1::Retrieval(_)));

    let compare = lower_root_v1(
        "COMPARE baseline=run::baseline-1 ;; candidate=run::candidate-2 ;; metric=semantic",
    );
    assert_eq!(
        compare,
        PublicIrRootV1::Interactive(InteractiveIrV1::Compare {
            baseline: quanta_gqlang_v2_ir::root_ir::InteractiveRefIrV1 {
                kind: "run".to_string(),
                locator: "baseline-1".to_string(),
            },
            candidate: quanta_gqlang_v2_ir::root_ir::InteractiveRefIrV1 {
                kind: "run".to_string(),
                locator: "candidate-2".to_string(),
            },
            metric: CompareMetricIrV1::Semantic,
        })
    );

    let export = lower_root_v1("EXPORT source=result::latest ;; kind=sarif ;; profile=strict");
    assert_eq!(
        export,
        PublicIrRootV1::Interactive(InteractiveIrV1::Export {
            source: quanta_gqlang_v2_ir::root_ir::InteractiveRefIrV1 {
                kind: "result".to_string(),
                locator: "latest".to_string(),
            },
            kind: ExportKindIrV1::Sarif,
            profile: Some("strict".to_string()),
        })
    );
}

#[test]
fn declaration_and_meta_roots_stay_host_agnostic_v1() {
    let view = lower_root_v1("VIEW catalog.users ;; query-ref=queries/catalog/users");
    assert_eq!(
        view,
        PublicIrRootV1::Declaration(DeclarationIrV1::View {
            name: "catalog.users".to_string(),
            query: ViewQueryIrV1::QueryRef("queries/catalog/users".to_string()),
        })
    );

    let pack =
        lower_root_v1("PACK secure-pack ;; source=registry::security ;; activation=workspace");
    assert_eq!(
        pack,
        PublicIrRootV1::Declaration(DeclarationIrV1::Pack {
            name: "secure-pack".to_string(),
            source: "registry::security".to_string(),
            activation: PackActivationIrV1::Workspace,
        })
    );

    let profile = lower_root_v1(
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
    );
    assert_eq!(
        profile,
        PublicIrRootV1::Declaration(DeclarationIrV1::Profile {
            name: "strict-review".to_string(),
            capabilities: vec!["graph.read".to_string(), "graph.trace".to_string()],
            admission: ProfileAdmissionIrV1::CertifiedOnly,
        })
    );

    let require = lower_root_v1("REQUIRE CERTIFIED");
    assert_eq!(
        require,
        PublicIrRootV1::Meta(MetaIrV1::RequireCertified {
            truth_requirement:
                quanta_gqlang_v2_ir::root_ir::CertifiedTruthRequirementIrV1::CertifiedOnly,
            admission_policy:
                quanta_gqlang_v2_ir::root_ir::CertifiedAdmissionPolicyIrV1::FailClosed,
        })
    );
}
