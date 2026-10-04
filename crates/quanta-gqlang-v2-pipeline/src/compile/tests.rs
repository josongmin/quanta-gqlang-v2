use super::assembly::{
    closure_error, item_location_of, record_produced_identity, validate_produced_membership,
    DocumentMembershipV2, DocumentRunV2, ItemRunV2, PlannedRunV2,
};
use super::*;
use quanta_gqlang_v2_sema::test_support::{
    reference_meta_document_v2, reference_relation_document_v2, ReferenceMetaFixtureV2,
    ReferenceRelationFixtureV2,
};
use quanta_gqlang_v2_sema::HirNodeIdV1;

fn failure_code(error: &FrontendTransactionErrorV2) -> &str {
    match error {
        FrontendTransactionErrorV2::Document(failure) => failure
            .diagnostics()
            .first()
            .expect("one assembly diagnostic")
            .code(),
        FrontendTransactionErrorV2::Authority(_) => {
            panic!("assembly test unexpectedly returned an authority error")
        }
    }
}

fn standard_budget_v2() -> FrontendBudgetV2 {
    FrontendBudgetV2::standard_v2().expect("standard frontend budget")
}

#[test]
fn parse_budget_projection_preserves_every_distinct_frontend_cap() {
    let budget = FrontendBudgetV2::try_new(
        151, 157, 31, 37, 11, 13, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107,
    )
    .expect("distinct positive frontend caps");
    let projected = project_parse_budget(SourceIdV2::from_raw(9), "", &budget)
        .expect("validated owner profile projects");
    assert_eq!(
        projected,
        ParseBudgetV2::try_new(31, 37, 41, 11, 13, 43, 47)
            .expect("independently specified parse limits")
    );
}

fn blocked_fixture_failure(code: &'static str) -> FrontendFailureV2 {
    let diagnostic = FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        code,
        FrontendSeverityV2::Error,
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(73),
            span: SpanV1::new(17, 31),
        },
        vec![LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(74),
            span: SpanV1::new(44, 49),
        }],
        "owner stage failed",
    );
    FrontendFailureV2::try_new(vec![diagnostic.clone(), diagnostic])
        .expect("nonempty owner failure")
}

#[test]
fn blocked_compile_retains_exact_owner_failure_for_rule_and_item_paths() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
        .expect("reference relation");
    let failure = blocked_fixture_failure("planning.test.item_blocked");
    let rule_failure = blocked_fixture_failure("planning.test.rule_blocked");
    for rule_path in [false, true] {
        let expected = if rule_path { &rule_failure } else { &failure }
            .diagnostics()
            .first()
            .expect("nonempty owner failure")
            .clone();
        let run = DocumentRunV2 {
            items: vec![ItemRunV2::Blocked {
                item_id: fixture.item_id(),
                failure: failure.clone(),
            }],
            rule_program: None,
            rule_blocked: rule_path.then(|| rule_failure.clone()),
        };
        assert_eq!(
            run.into_compiled()
                .expect_err("blocked stage must reject compilation"),
            FrontendTransactionErrorV2::Document(FrontendFailureV2::one(expected.clone())),
        );
    }
}

#[test]
fn blocked_analysis_retains_item_failure_and_counts_canonical_diagnostics() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
        .expect("reference relation");
    let failure = blocked_fixture_failure("planning.test.item_blocked");
    let make_run = || DocumentRunV2 {
        items: vec![ItemRunV2::Blocked {
            item_id: fixture.item_id(),
            failure: failure.clone(),
        }],
        rule_program: None,
        rule_blocked: None,
    };
    let analysis = make_run()
        .into_analysis(SourceIdV2::from_raw(73), 1)
        .expect("two identical diagnostics count once");
    assert_eq!(analysis.order.as_ref(), &[fixture.item_id()]);
    assert_eq!(
        analysis.items(),
        &[AnalysisItemV2::BlockedByDiagnostics(
            failure.diagnostics().into()
        )]
    );
    assert_eq!(
        analysis.diagnostics().as_ref(),
        &[failure
            .diagnostics()
            .first()
            .expect("nonempty owner failure")
            .clone()]
    );
    let error = make_run()
        .into_analysis(SourceIdV2::from_raw(73), 0)
        .expect_err("nonempty failure exceeds zero diagnostic budget");
    assert_eq!(
        failure_code(&error),
        "pipeline.analysis.diagnostic_limit_exceeded"
    );
}

#[test]
fn blocked_rule_analysis_projects_owner_failure_to_every_member_after_counting() {
    use quanta_gqlang_v2_sema::test_support::{reference_rule_document_v2, ReferenceRuleFixtureV2};
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::NonRecursiveDependency)
        .expect("reference rules");
    let membership = DocumentMembershipV2::of(fixture.document()).expect("membership");
    let program_id = membership.rule_program_id().expect("rule program");
    let members = membership.rule_member_ids();
    assert_eq!(members.len(), 2);
    let failure = blocked_fixture_failure("planning.test.item_blocked");
    let run = DocumentRunV2 {
        items: members
            .iter()
            .map(|id| {
                ItemRunV2::Planned(Box::new(PlannedRunV2 {
                    planned: Box::new(PlannedItemV2::RuleMember(RuleProgramMemberV2::new(
                        program_id, *id,
                    ))),
                    relation: None,
                    graph: None,
                    check: None,
                    presentation: None,
                    composition: None,
                }))
            })
            .collect(),
        rule_program: None,
        rule_blocked: Some(failure.clone()),
    };
    let analysis = run
        .into_analysis(SourceIdV2::from_raw(73), 1)
        .expect("one canonical failure is counted before member fanout");
    assert_eq!(analysis.order.as_ref(), members.as_slice());
    assert_eq!(
        analysis.items(),
        &[
            AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into()),
            AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into()),
        ]
    );
    assert_eq!(
        analysis.diagnostics().as_ref(),
        &[failure
            .diagnostics()
            .first()
            .expect("nonempty owner failure")
            .clone()]
    );
}

#[test]
fn source_root_classifier_maps_supported_and_refused_families_without_authority() {
    for (source_v1, expected_v1) in [
        (
            "find path src.marker -> src.marker | depth=8",
            FrontendSourceRootFamilyV3::Graph,
        ),
        (
            "show capabilities",
            FrontendSourceRootFamilyV3::Presentation,
        ),
        (
            "let q = sample query=\"x\" | mode=lexical | limit=1 from users u with q",
            FrontendSourceRootFamilyV3::Composition,
        ),
    ] {
        assert_eq!(
            classify_single_source_root_v3(
                SourceIdV2::from_raw(1),
                source_v1,
                &standard_budget_v2(),
            )
            .expect("valid root must classify"),
            expected_v1,
        );
    }
}

#[test]
fn frontend_lexical_census_is_source_derived_and_rejects_adjacent_invalid_input() {
    let source_id = SourceIdV2::from_raw(19);
    let budget = standard_budget_v2();
    let census = census_source_lexical_materialization_v3(source_id, "show capabilities", &budget)
        .expect("valid SHOW source has lexical storage");
    assert_eq!(census.token_count(), 3);
    assert_eq!(census.payload_bytes(), 16);
    assert_eq!(
        census.logical_bytes(),
        std::mem::size_of::<Vec<quanta_gqlang_v2_surface::TokenV1>>()
            + 3 * std::mem::size_of::<quanta_gqlang_v2_surface::TokenV1>()
            + 16,
    );

    let error = census_source_lexical_materialization_v3(source_id, "show capabilities @", &budget)
        .expect_err("invalid adjacent token must fail before allocation");
    assert_eq!(failure_code(&error), "surface.parse.unexpected_char");
}

#[test]
fn lexical_owner_classification_accepts_exact_and_rejects_short_or_foreign_root() {
    let source_id = SourceIdV2::from_raw(19);
    let source = String::from("show capabilities");
    let budget = standard_budget_v2();
    let lexical_bytes = u64::try_from(
        census_source_lexical_materialization_v3(source_id, &source, &budget)
            .expect("source census")
            .logical_bytes(),
    )
    .expect("logical demand fits u64");
    let parser_bytes = u64::try_from(
        std::mem::size_of::<Vec<quanta_gqlang_v2_surface::SurfaceItemV1>>()
            + std::mem::size_of::<quanta_gqlang_v2_surface::SurfaceItemV1>(),
    )
    .expect("parser item demand fits u64");
    let hir_bytes =
        u64::try_from(quanta_gqlang_v2_sema::test_support::show_hir_storage_bytes_v2(1))
            .expect("SHOW HIR storage fits u64");
    let bytes = lexical_bytes + parser_bytes + hir_bytes;
    let host = quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(bytes)
        .expect("finite installed host ceiling");
    let exact = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, bytes, host)
        .expect("source-bound exact root");
    assert_eq!(
        classify_single_source_root_with_lexical_owner_v1(source_id, &source, &budget, &exact,)
            .expect("exact lexical demand"),
        FrontendSourceRootFamilyV3::Presentation,
    );
    assert_eq!(exact.live_bytes_v1(), 0);

    let short = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, bytes - 1, host)
        .expect("one-byte-short root");
    let rejected =
        classify_single_source_root_with_lexical_owner_v1(source_id, &source, &budget, &short)
            .expect_err("one-byte-short root cannot create lexer payload");
    assert!(matches!(
        rejected,
        FrontendSourceRootClassificationErrorV3::Frontend(_)
    ));
    assert_eq!(short.live_bytes_v1(), 0);

    let foreign_source = String::from("show capabilities");
    let foreign =
        FrontendLexicalMaterializationRootV1::issue_v1(source_id, &foreign_source, bytes, host)
            .expect("distinct invocation");
    assert!(classify_single_source_root_with_lexical_owner_v1(
        source_id, &source, &budget, &foreign,
    )
    .is_err());
    assert_eq!(foreign.live_bytes_v1(), 0);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "exact peak and ownership cases share one fixture"
)]
fn show_monotonic_owner_accepts_exact_and_rejects_short_foreign_and_adjacent() {
    let source_id = SourceIdV2::from_raw(29);
    let source = String::from("show capabilities");
    let budget = standard_budget_v2();
    let lexical_bytes = census_source_lexical_materialization_v3(source_id, &source, &budget)
        .expect("source census")
        .logical_bytes();
    let parser_bytes = std::mem::size_of::<Vec<quanta_gqlang_v2_surface::SurfaceItemV1>>()
        + std::mem::size_of::<quanta_gqlang_v2_surface::SurfaceItemV1>();
    let hir_bytes = quanta_gqlang_v2_sema::test_support::show_hir_storage_bytes_v2(1);
    let exact = u64::try_from(lexical_bytes + parser_bytes + hir_bytes)
        .expect("SHOW logical demand fits u64");
    let host = quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(exact)
        .expect("finite installed ceiling");
    let root = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, exact, host)
        .expect("source-bound root");
    let admitted = admit_show_syntax_v2(source_id, &source, &budget, &root)
        .expect("exact SHOW parser payload and zero-heap integrity");
    assert_eq!(admitted.source_id_v2(), source_id);
    assert_eq!(admitted.item_count_v2(), 1);
    assert_eq!(admitted.checked_node_count_v2(), 1);
    let host_source = SourceIdV2::from_raw(1);
    let host_location = quanta_gqlang_v2_sema::FrontendAuthorityLocationV2 {
        source_id: host_source,
        start: 0,
        end: 1,
    };
    let authority = quanta_gqlang_v2_sema::ClosedFrontendAuthorityBuilderV2::try_new(
        budget.clone(),
        vec![quanta_gqlang_v2_sema::FrontendAuthoritySourceV2 {
            source_id: host_source,
            source_len: 64,
        }],
        "app".to_owned(),
        host_location,
        quanta_gqlang_v2_sema::FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("host authority builder")
    .finish()
    .expect("closed host authority");
    let declared = admit_show_source_declaration_v2(admitted, &authority)
        .expect("SHOW source declaration keeps the syntax root");
    assert_eq!(declared.source_id_v2(), source_id);
    assert_eq!(declared.item_count_v2(), 1);
    assert_eq!(root.live_bytes_v1(), exact);
    drop(declared);
    assert_eq!(root.live_bytes_v1(), 0);

    // Independent type/count oracle for the next first birth. The plan
    // slots are charged while every earlier parser debit is still live.
    let plan_bytes = u64::try_from(
        std::mem::size_of::<Vec<SemanticPlanV2>>() + std::mem::size_of::<SemanticPlanV2>(),
    )
    .expect("one SHOW plan slot fits u64");
    let complete_bytes = exact.checked_add(plan_bytes).expect("finite peak");
    let plan_host = quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(complete_bytes)
        .expect("installed peak ceiling");
    let plan_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        complete_bytes,
        plan_host,
    )
    .expect("same-source plan root");
    let plan_syntax = admit_show_syntax_v2(source_id, &source, &budget, &plan_root)
        .expect("already admitted syntax");
    let plan_declared =
        admit_show_source_declaration_v2(plan_syntax, &authority).expect("SHOW source declaration");
    let planned = admit_show_plans_v2(plan_declared, &source, &plan_root)
        .expect("exact source-derived SHOW plan peak");
    assert_eq!(planned.source_id_v2(), source_id);
    assert_eq!(planned.plan_count_v2(), 1);
    let plan = planned.plans.first().expect("one planned SHOW item");
    assert_eq!(
        plan.effect(),
        quanta_gqlang_v2_planning::EffectClassV2::ReadOnly
    );
    assert_eq!(
        plan.result().result_kind(),
        quanta_gqlang_v2_planning::ResultKindV2::Table
    );
    assert!(plan.capabilities().is_empty());
    assert!(matches!(
        plan.policy(),
        quanta_gqlang_v2_planning::PlanPolicyV2::Presentation(
            quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Show {
                category: quanta_gqlang_v2_sema::ShowCategoryV1::Capabilities,
            }
        )
    ));
    assert_eq!(plan_root.live_bytes_v1(), complete_bytes);
    drop(planned);
    assert_eq!(plan_root.live_bytes_v1(), 0);

    let execution_bytes = u64::try_from(
        std::mem::size_of::<Vec<ExecutionItemV3>>() + std::mem::size_of::<ExecutionItemV3>(),
    )
    .expect("one SHOW execution slot fits u64");
    let full_peak = complete_bytes
        .checked_add(execution_bytes)
        .expect("finite execution peak");
    let execution_host = quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(full_peak)
        .expect("installed execution peak");
    let execution_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        full_peak,
        execution_host,
    )
    .expect("exact execution root");
    let execution_syntax = admit_show_syntax_v2(source_id, &source, &budget, &execution_root)
        .expect("admitted SHOW syntax");
    let execution_declaration = admit_show_source_declaration_v2(execution_syntax, &authority)
        .expect("declared SHOW source");
    let execution_plans = admit_show_plans_v2(execution_declaration, &source, &execution_root)
        .expect("admitted SHOW plan slots");
    let execution = admit_show_execution_v2(execution_plans, &source, &execution_root)
        .expect("exact source-derived SHOW execution DTO");
    assert_eq!(execution.source_id_v2(), source_id);
    assert_eq!(execution.item_count_v2(), 1);
    assert_eq!(execution_root.live_bytes_v1(), full_peak);
    assert_ne!(execution.qualification_digest_v2().as_bytes(), &[0; 32]);
    drop(execution);
    assert_eq!(execution_root.live_bytes_v1(), 0);

    let execution_short = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        full_peak - 1,
        execution_host,
    )
    .expect("one-byte-short execution root");
    let short_syntax = admit_show_syntax_v2(source_id, &source, &budget, &execution_short)
        .expect("syntax fits before execution reservation");
    let short_declaration = admit_show_source_declaration_v2(short_syntax, &authority)
        .expect("declaration fits before execution reservation");
    let short_plans = admit_show_plans_v2(short_declaration, &source, &execution_short)
        .expect("plans fit before execution reservation");
    assert!(matches!(
        admit_show_execution_v2(short_plans, &source, &execution_short),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage)
    ));
    assert_eq!(execution_short.live_bytes_v1(), 0);

    let plan_owner_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        full_peak,
        execution_host,
    )
    .expect("plan owner root");
    let foreign_execution_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        full_peak,
        execution_host,
    )
    .expect("equal-policy foreign execution root");
    let owned_syntax =
        admit_show_syntax_v2(source_id, &source, &budget, &plan_owner_root).expect("owned syntax");
    let owned_declaration =
        admit_show_source_declaration_v2(owned_syntax, &authority).expect("owned declaration");
    let owned_plans =
        admit_show_plans_v2(owned_declaration, &source, &plan_owner_root).expect("owned plans");
    assert!(matches!(
        admit_show_execution_v2(owned_plans, &source, &foreign_execution_root),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteShowRoot)
    ));
    assert_eq!(plan_owner_root.live_bytes_v1(), 0);
    assert_eq!(foreign_execution_root.live_bytes_v1(), 0);

    let plan_short = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        complete_bytes - 1,
        plan_host,
    )
    .expect("one-byte-short plan root");
    let short_syntax = admit_show_syntax_v2(source_id, &source, &budget, &plan_short)
        .expect("parser fits before short plan reservation");
    let short_declared = admit_show_source_declaration_v2(short_syntax, &authority)
        .expect("source declaration fits before short plan reservation");
    assert!(matches!(
        admit_show_plans_v2(short_declared, &source, &plan_short),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage)
    ));
    assert_eq!(plan_short.live_bytes_v1(), 0);

    let syntax_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        complete_bytes,
        plan_host,
    )
    .expect("syntax root");
    let foreign_plan_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &source,
        complete_bytes,
        plan_host,
    )
    .expect("equal-policy foreign root");
    let syntax =
        admit_show_syntax_v2(source_id, &source, &budget, &syntax_root).expect("syntax owner");
    let source_declared =
        admit_show_source_declaration_v2(syntax, &authority).expect("source declaration");
    assert!(matches!(
        admit_show_plans_v2(source_declared, &source, &foreign_plan_root),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteShowRoot)
    ));
    assert_eq!(syntax_root.live_bytes_v1(), 0);
    assert_eq!(foreign_plan_root.live_bytes_v1(), 0);

    assert!(matches!(
        admit_show_planned_source_v2(source_id, &source, &authority, &plan_root),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteSourceDiagnostics)
    ));
    assert_eq!(plan_root.live_bytes_v1(), 0);
    let malformed = String::from("show @");
    let diagnostic_host =
        quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(1).unwrap();
    let malformed_root =
        FrontendLexicalMaterializationRootV1::issue_v1(source_id, &malformed, 1, diagnostic_host)
            .unwrap();
    assert!(matches!(
        admit_show_planned_source_v2(source_id, &malformed, &authority, &malformed_root),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteSourceDiagnostics)
    ));
    assert_eq!(malformed_root.live_bytes_v1(), 0);
    let overlong = "x".repeat(usize::try_from(budget.max_source_bytes()).unwrap() + 1);
    let overlong_host = quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(1).unwrap();
    let overlong_root =
        FrontendLexicalMaterializationRootV1::issue_v1(source_id, &overlong, 1, overlong_host)
            .unwrap();
    assert!(matches!(
        admit_show_planned_source_v2(source_id, &overlong, &authority, &overlong_root),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteShowSourceLimit)
    ));
    assert_eq!(overlong_root.live_bytes_v1(), 0);

    let different_budget = FrontendBudgetV2::try_new(
        1024, 1024, 1024, 1, 8, 8, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
        1024, 1024, 8, 1024, 1024, 1024,
    )
    .unwrap();
    let mismatched_authority = quanta_gqlang_v2_sema::ClosedFrontendAuthorityBuilderV2::try_new(
        different_budget,
        vec![quanta_gqlang_v2_sema::FrontendAuthoritySourceV2 {
            source_id: host_source,
            source_len: 64,
        }],
        "app".to_owned(),
        host_location,
        quanta_gqlang_v2_sema::FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .unwrap()
    .finish()
    .unwrap();
    let admitted = admit_show_syntax_v2(source_id, &source, &budget, &root).unwrap();
    assert!(matches!(
        admit_show_source_declaration_v2(admitted, &mismatched_authority),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteAuthorityBudget)
    ));
    assert_eq!(root.live_bytes_v1(), 0);

    let short = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, exact - 1, host)
        .expect("one-byte-short root");
    assert!(matches!(
        admit_show_syntax_v2(source_id, &source, &budget, &short),
        Err(ShowSyntaxAdmissionErrorV2::Frontend(_))
    ));
    assert_eq!(short.live_bytes_v1(), 0);

    let foreign_source = String::from("show capabilities");
    let foreign =
        FrontendLexicalMaterializationRootV1::issue_v1(source_id, &foreign_source, exact, host)
            .expect("foreign root");
    assert!(admit_show_syntax_v2(source_id, &source, &budget, &foreign).is_err());
    assert_eq!(foreign.live_bytes_v1(), 0);

    let adjacent_source = String::from("describe model::foo");
    let adjacent_host =
        quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(4096).unwrap();
    let adjacent_root = FrontendLexicalMaterializationRootV1::issue_v1(
        source_id,
        &adjacent_source,
        4096,
        adjacent_host,
    )
    .unwrap();
    assert!(matches!(
        admit_show_syntax_v2(source_id, &adjacent_source, &budget, &adjacent_root),
        Err(ShowSyntaxAdmissionErrorV2::IncompleteShowGrammar)
    ));
    assert_eq!(adjacent_root.live_bytes_v1(), 0);
}

#[test]
fn two_show_items_need_no_integrity_heap_and_retain_one_owner() {
    let source_id = SourceIdV2::from_raw(30);
    let source = String::from("show capabilities; show models");
    let budget = standard_budget_v2();
    let lexical_bytes = census_source_lexical_materialization_v3(source_id, &source, &budget)
        .expect("source census")
        .logical_bytes();
    let item_count = 2;
    let parser_bytes = std::mem::size_of::<Vec<quanta_gqlang_v2_surface::SurfaceItemV1>>()
        + item_count * std::mem::size_of::<quanta_gqlang_v2_surface::SurfaceItemV1>();
    let hir_bytes = quanta_gqlang_v2_sema::test_support::show_hir_storage_bytes_v2(item_count);
    let exact = u64::try_from(lexical_bytes + parser_bytes + hir_bytes)
        .expect("two-SHOW logical demand fits u64");
    let host = quanta_gqlang_v2_surface::FrontendLexicalHostCeilingV1::try_new(exact)
        .expect("finite ceiling");
    let root = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, exact, host)
        .expect("source-bound root");
    let admitted = admit_show_syntax_v2(source_id, &source, &budget, &root)
        .expect("two monotonic SHOW identities");
    assert_eq!(admitted.item_count_v2(), 2);
    assert_eq!(admitted.checked_node_count_v2(), 2);
    assert_eq!(root.live_bytes_v1(), exact);
    drop(admitted);
    assert_eq!(root.live_bytes_v1(), 0);
}

#[test]
fn source_root_classifier_rejects_zero_multiple_and_malformed_documents() {
    assert!(matches!(
        classify_single_source_root_v3(SourceIdV2::from_raw(1), "", &standard_budget_v2(),),
        Err(FrontendSourceRootClassificationErrorV3::NoItem)
    ));
    assert!(matches!(
        classify_single_source_root_v3(
            SourceIdV2::from_raw(1),
            "show capabilities; show models",
            &standard_budget_v2(),
        ),
        Err(FrontendSourceRootClassificationErrorV3::MultipleItems { observed: 2 })
    ));
    assert!(matches!(
        classify_single_source_root_v3(
            SourceIdV2::from_raw(1),
            "find path src.marker src.marker",
            &standard_budget_v2(),
        ),
        Err(FrontendSourceRootClassificationErrorV3::Frontend(_))
    ));
}

#[test]
fn duplicate_producer_identity_fails_closed_at_the_second_insert() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
        .expect("reference relation");
    let id = fixture.item_id();
    let location = item_location_of(fixture.document(), id);
    let mut produced = BTreeSet::new();
    record_produced_identity(&mut produced, id, location, "duplicate")
        .expect("first identity is admitted");
    let error = record_produced_identity(&mut produced, id, location, "duplicate")
        .expect_err("second identity rejects");
    assert_eq!(
        failure_code(&error),
        "pipeline.document.duplicate_item_result"
    );
}

#[test]
fn plan_membership_missing_extra_and_rule_mismatch_are_exact() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
        .expect("reference relation");
    let membership = DocumentMembershipV2::of(fixture.document()).expect("membership");
    let id = fixture.item_id();
    let expected_plan = BTreeSet::from([id]);
    validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &expected_plan,
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .expect("exact plan membership");

    let missing = validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .expect_err("missing plan rejects");
    assert_eq!(
        failure_code(&missing),
        "pipeline.document.missing_item_result"
    );

    let extra_id = SemanticItemIdV2::new(
        fixture.document().source_id(),
        HirNodeIdV1::from_raw(id.hir_item_id().raw() + 100),
    );
    let extra = validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &BTreeSet::from([id, extra_id]),
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .expect_err("extra plan rejects");
    assert_eq!(failure_code(&extra), "pipeline.document.extra_item_result");

    let rule_mismatch = validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &expected_plan,
        &BTreeSet::from([extra_id]),
        &BTreeSet::new(),
    )
    .expect_err("unexpected rule member rejects first");
    assert_eq!(
        failure_code(&rule_mismatch),
        "pipeline.document.rule_membership_mismatch"
    );
}

#[test]
fn checked_non_plan_membership_missing_and_extra_are_exact() {
    let fixture =
        reference_meta_document_v2(ReferenceMetaFixtureV2::UsePack).expect("reference USE PACK");
    let membership = DocumentMembershipV2::of(fixture.document()).expect("membership");
    let id = fixture.meta_item_id();
    validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::from([id]),
    )
    .expect("exact non-plan membership");

    let missing = validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .expect_err("missing non-plan rejects");
    assert_eq!(
        failure_code(&missing),
        "pipeline.document.missing_item_result"
    );

    let extra_id = SemanticItemIdV2::new(
        fixture.document().source_id(),
        HirNodeIdV1::from_raw(id.hir_item_id().raw() + 100),
    );
    let extra = validate_produced_membership(
        fixture.document().source_id(),
        &membership,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::from([id, extra_id]),
    )
    .expect_err("extra non-plan rejects");
    assert_eq!(failure_code(&extra), "pipeline.document.extra_item_result");
}

#[test]
fn execution_closure_failures_project_to_independent_pipeline_codes() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
        .expect("reference relation");
    for (failure, expected) in [
        (
            ExecutionClosureFailureV3::Dangling,
            "pipeline.document.dangling_execution_reference",
        ),
        (
            ExecutionClosureFailureV3::Foreign,
            "pipeline.document.foreign_item_result",
        ),
        (
            ExecutionClosureFailureV3::UnregisteredSymbol,
            "pipeline.document.unregistered_execution_symbol",
        ),
    ] {
        let error = closure_error(fixture.document(), fixture.item_id(), failure);
        assert_eq!(failure_code(&error), expected);
    }
}
