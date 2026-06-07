use quanta_gqlang_v2_sema::ai::exploration::{
    AiExplorationPlanV1, AiExportTargetV1, AiObjectiveV1, CandidateBasisV1, DiscoveryBasisV1,
    DiscoveryPlanV1, ExplorationBudgetV1, ExportPlanV1, ReconfirmationRequirementV1,
    WorkingSetBasisV1,
};
use quanta_gqlang_v2_sema::interactive::show_describe::{
    TypedDescribeTargetKindV1, TypedShowCategoryV1, TypedShowDescribeSpecV1,
};
use quanta_gqlang_v2_sema::profile::ExecutionProfileClassV1;
use quanta_gqlang_v2_sema::roots::check::{
    CheckConfirmIntentV1, CheckExecutionModeV1, CheckRootIntentV1,
};
use quanta_gqlang_v2_sema::roots::explain::{
    TypedExplainEvidenceModeV1, TypedExplainTruthExpectationV1,
};
use quanta_gqlang_v2_sema::{
    admissibility::validate_ai_plan_v1, analyze_document_v1, analyze_root_v1,
    TypedCertifiedAdmissionPolicyV1, TypedCertifiedTruthRequirementV1, TypedMetaCommandV1,
    TypedPackActivationModeV1, TypedRootV1,
};
use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::show::{DescribeRootAstV1, ShowRootAstV1};
use quanta_gqlang_v2_syntax::parse_document_v1;
use quanta_gqlang_v2_syntax::roots::check::CheckRootAstV1;
use quanta_gqlang_v2_syntax::SyntaxDiagnosticV1;

fn analyze_root_from_query_v1(input: &str) -> TypedRootV1 {
    analyze_document_v1(
        &parse_document_v1(input)
            .unwrap_or_else(|page| panic!("expected parse success for `{input}`: {page:?}")),
    )
    .unwrap_or_else(|page| panic!("expected sema success for `{input}`: {page:?}"))
    .roots
    .into_iter()
    .next()
    .expect("one root")
}

fn parse_error_v1(input: &str) -> SyntaxDiagnosticV1 {
    parse_document_v1(input)
        .expect_err("query must fail closed")
        .diagnostics
        .into_iter()
        .next()
        .expect("syntax diagnostic")
}

fn valid_budget_v1() -> ExplorationBudgetV1 {
    ExplorationBudgetV1 {
        max_candidates: 25,
        max_working_set_items: 10,
        max_reconfirmation_runs: 2,
    }
}

fn valid_reconfirmation_v1() -> ReconfirmationRequirementV1 {
    ReconfirmationRequirementV1 {
        replay_profile: ExecutionProfileClassV1::Lint,
        same_intent_replay: true,
    }
}

#[test]
fn select_and_find_roots_admit_shipped_surface_v1() {
    let select = analyze_root_from_query_v1(
        "SELECT id, path FROM catalog.files WHERE severity = 'high' LIMIT 10",
    );
    let TypedRootV1::Select(select) = select else {
        panic!("expected SELECT typed root");
    };
    assert_eq!(
        select.from_entity,
        "SELECT id, path FROM catalog.files WHERE severity = 'high' LIMIT 10"
    );

    let find_inputs = [
        "FIND reachability api.entry -> db.write | via=auth | depth=3",
        "FIND flow ingress -> egress | through=ssa | window=function",
        "FIND slice callsite.foo | direction=backward | frontier=cfg",
        "FIND taint user.input -> shell.exec | sanitizers=escape | depth=4",
        "FIND path handler -> sink | through=router | depth=6",
        "FIND points_to ptr => heap.obj | scope=fn:main",
        "FIND summary pkg::open | include=params,returns | depth=2",
    ];

    for input in find_inputs {
        let typed = analyze_root_from_query_v1(input);
        let TypedRootV1::Find(spec) = typed else {
            panic!("expected FIND typed root for `{input}`");
        };
        assert!(
            spec.typed_spec().is_ok(),
            "expected typed FIND spec for `{input}`"
        );
    }
}

#[test]
fn check_modes_and_explain_shapes_stay_explicit_v1() {
    let cases = [
        (
            CheckRootIntentV1::new(
                "rule.pattern/path-filter",
                "pattern",
                None,
                Some("path ~= \"src/**/*.rs\"".to_string()),
                CheckConfirmIntentV1::Prompt,
            )
            .expect("pattern-only intent"),
            CheckExecutionModeV1::PatternOnly,
        ),
        (
            CheckRootIntentV1::new(
                "rule.semantic/ownership",
                "policy",
                Some("repo://frontend".to_string()),
                None,
                CheckConfirmIntentV1::Skip,
            )
            .expect("semantic-only intent"),
            CheckExecutionModeV1::SemanticOnly,
        ),
        (
            CheckRootIntentV1::new(
                "rule.hybrid/shell-audit",
                "security",
                Some("repo://runtime".to_string()),
                Some("call = \"std::process::Command\"".to_string()),
                CheckConfirmIntentV1::Require,
            )
            .expect("hybrid intent"),
            CheckExecutionModeV1::Hybrid,
        ),
    ];

    for (intent, expected_mode) in cases {
        let typed = analyze_root_v1(&AstRootV1::Check(CheckRootAstV1::from_intent(
            intent.clone(),
        )))
        .expect("typed CHECK");
        let TypedRootV1::Check(spec) = typed else {
            panic!("expected CHECK typed root");
        };
        assert_eq!(spec.intent().expect("intent"), intent);
        assert_eq!(spec.execution_mode().expect("mode"), expected_mode);
    }

    let explain = analyze_root_from_query_v1(
        "EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm",
    );
    let TypedRootV1::Explain(explain) = explain else {
        panic!("expected EXPLAIN typed root");
    };
    let shape = explain.shape().expect("typed EXPLAIN shape");
    assert_eq!(shape.target, "rule::owasp.sql-injection");
    assert_eq!(shape.evidence, TypedExplainEvidenceModeV1::Trace);
    assert_eq!(
        shape.truth_expectation,
        TypedExplainTruthExpectationV1::Confirm
    );
}

#[test]
fn interactive_declaration_and_meta_roots_are_admissible_v1() {
    let show = analyze_root_from_query_v1("SHOW capabilities");
    let TypedRootV1::ShowDescribe(show) = show else {
        panic!("expected SHOW typed root");
    };
    assert_eq!(
        show.typed_spec(),
        Ok(TypedShowDescribeSpecV1::Show(
            quanta_gqlang_v2_sema::interactive::show_describe::TypedShowRootV1 {
                category: TypedShowCategoryV1::Capabilities,
            },
        ))
    );

    let describe = analyze_root_from_query_v1("DESCRIBE rule::owasp.sql-injection");
    let TypedRootV1::ShowDescribe(describe) = describe else {
        panic!("expected DESCRIBE typed root");
    };
    let TypedShowDescribeSpecV1::Describe(spec) = describe.typed_spec().expect("describe spec")
    else {
        panic!("expected DESCRIBE typed spec");
    };
    assert_eq!(spec.target.kind, TypedDescribeTargetKindV1::Rule);
    assert_eq!(
        spec.target.path,
        vec!["owasp".to_string(), "sql-injection".to_string()]
    );

    let profile = analyze_root_from_query_v1(
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
    );
    assert!(matches!(profile, TypedRootV1::Declaration(_)));

    let pack = analyze_root_from_query_v1(
        "PACK secure-pack ;; source=registry::security ;; activation=workspace",
    );
    assert!(matches!(pack, TypedRootV1::Declaration(_)));

    let gate = analyze_root_from_query_v1(
        "GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all",
    );
    assert!(matches!(gate, TypedRootV1::Declaration(_)));

    let use_pack = analyze_root_from_query_v1(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock",
    );
    assert_eq!(
        use_pack,
        TypedRootV1::Meta(TypedMetaCommandV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: TypedPackActivationModeV1::Workspace,
            lockfile_path: "packs/secure-pack.lock".to_string(),
        })
    );

    let require = analyze_root_from_query_v1("REQUIRE CERTIFIED");
    assert_eq!(
        require,
        TypedRootV1::Meta(TypedMetaCommandV1::RequireCertified {
            truth_requirement: TypedCertifiedTruthRequirementV1::CertifiedOnly,
            admission_policy: TypedCertifiedAdmissionPolicyV1::FailClosed,
        })
    );
}

#[test]
fn use_pack_and_require_certified_stay_explicit_and_fail_closed_v1() {
    let missing_lockfile = parse_error_v1("USE PACK secure-pack ;; activation=workspace");
    assert_eq!(missing_lockfile.code, "use_pack_missing_lockfile");

    let invalid_activation = parse_error_v1(
        "USE PACK secure-pack ;; activation=manual ;; lockfile=packs/secure-pack.lock",
    );
    assert_eq!(invalid_activation.code, "use_pack_invalid_activation");

    let traversal_lockfile = parse_error_v1(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=../secure-pack.lock",
    );
    assert_eq!(traversal_lockfile.code, "use_pack_invalid_lockfile");

    let ambient_clause = parse_error_v1(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock ;; source=ambient",
    );
    assert_eq!(ambient_clause.code, "unsupported_use_pack_clause");

    let typed = analyze_root_from_query_v1(
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
    );
    assert_eq!(
        typed,
        TypedRootV1::Meta(TypedMetaCommandV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: TypedPackActivationModeV1::Session,
            lockfile_path: "packs/secure-pack.lock".to_string(),
        })
    );

    let require = analyze_root_from_query_v1("REQUIRE CERTIFIED");
    assert_eq!(
        require,
        TypedRootV1::Meta(TypedMetaCommandV1::RequireCertified {
            truth_requirement: TypedCertifiedTruthRequirementV1::CertifiedOnly,
            admission_policy: TypedCertifiedAdmissionPolicyV1::FailClosed,
        })
    );
}

#[test]
fn show_and_describe_do_not_fallback_to_generic_targets_v1() {
    let invalid_show = analyze_root_v1(&AstRootV1::Show(ShowRootAstV1 {
        category: "catalog".to_string(),
    }))
    .expect_err("generic SHOW category must fail closed");
    assert!(invalid_show
        .diagnostics
        .iter()
        .any(|diag| diag.contains("unsupported_show_category")));

    let invalid_describe = analyze_root_v1(&AstRootV1::Describe(DescribeRootAstV1 {
        target: "catalog::overview".to_string(),
    }))
    .expect_err("generic DESCRIBE target must fail closed");
    assert!(invalid_describe
        .diagnostics
        .iter()
        .any(|diag| diag.contains("unsupported_describe_kind")));
}

#[test]
fn ai_profile_admissibility_stays_explicit_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::FindCandidates,
        discovery_plan: Some(DiscoveryPlanV1 {
            basis: DiscoveryBasisV1::Candidate(CandidateBasisV1::RankedSelect),
        }),
        narrowing_plan: None,
        proof_plan: None,
        export_plan: Some(ExportPlanV1 {
            working_set_basis: WorkingSetBasisV1::CandidateSet,
            target: AiExportTargetV1::CandidateSet,
        }),
        budget: valid_budget_v1(),
        reconfirmation: valid_reconfirmation_v1(),
    };

    let err = validate_ai_plan_v1(ExecutionProfileClassV1::Query, &plan).unwrap_err();
    assert_eq!(err.code, "AI_PROFILE_REQUIRED");
}
