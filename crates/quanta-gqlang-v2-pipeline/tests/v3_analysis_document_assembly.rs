#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "analysis-assembly rails assert exact stage statuses with unwrap/expect/indexing/panic; allowed for test targets only"
)]

//! Analysis-document stage statuses (ticket 07, lane W4-B).
//!
//! `AnalysisDocumentV2` stores every item stage as an explicit, non-optional
//! status: `Complete(payload)` / `NotApplicable(NoPlanReasonV2)` /
//! `BlockedByDiagnostics(ordered diagnostics)`. These rails prove:
//!
//! - a fully resolved document carries complete payloads and no unresolved fact;
//! - partial open authority blocks every authored item with explicit unresolved
//!   facts and named missing-authority warnings, never a fabricated default;
//! - a complete open authority resolves and carries the SAME payloads as a
//!   closed compile while remaining an analysis document;
//! - an owner stage that rejects blocks only its own item (diagnostic code
//!   preserved verbatim) instead of failing the whole analysis;
//! - a blocked analysis is not compilable, so no implicit promotion exists.

use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    VersionV2,
};
use quanta_gqlang_v2_pipeline::{
    analyze_resolved_document_v2, analyze_source_document_v2, compile_source_document_v3,
    AnalysisItemV2, CompiledItemV2, FrontendTransactionErrorV2, PlannedItemV2,
    OPEN_AUTHORITY_INCOMPLETE_CODE,
};
use quanta_gqlang_v2_sema::test_support::{
    forged_rule_fact_document_v2, forged_small_receipt_relation_chain_document_v2,
    reference_absent_item_id_v2, reference_relation_document_v2, ForgedRuleFactMutationV2,
    ReferenceRelationFixtureV2, ReferenceRuleFixtureV2, SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{
    CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2, FrontendBudgetV2, GraphReferenceCatalogV2,
    LocatedSpanV2, OpenAnalysisAuthorityInputV2, OpenAnalysisAuthorityV2, RelationCatalogV2,
    SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};

/// The host registry source identity used by the minimal authority.
const HOST_SOURCE: SourceIdV2 = SourceIdV2::from_raw(1);

/// The document source identity used by the source-text rails.
const DOC_SOURCE: SourceIdV2 = SourceIdV2::from_raw(9);

/// A source document that needs no relation or graph authority.
const MINIMAL_SOURCE: &str = "let answer = 1; rule local_rule : x if x = 1; show capabilities;";

fn budget_with_diagnostic_limit(limit: u32) -> FrontendBudgetV2 {
    FrontendBudgetV2::try_new(
        64, 256, 1_024, 64, 32, 32, 2_048, 64, 64, 10_000, 128, 256, 10_000, 4_096, 1_024, 128,
        256, 10_000, 64, 128, limit, 4_096,
    )
    .expect("checked diagnostic budget")
}

#[test]
fn public_open_analysis_enforces_its_diagnostic_cap() {
    let exact = OpenAnalysisAuthorityV2::try_empty_v2(budget_with_diagnostic_limit(3))
        .expect("exact open authority");
    let analyzed = analyze_source_document_v2(DOC_SOURCE, "show capabilities", &exact)
        .expect("exact diagnostic limit admits all three missing-authority facts");
    assert_eq!(analyzed.diagnostics().len(), 3);

    let limited = OpenAnalysisAuthorityV2::try_empty_v2(budget_with_diagnostic_limit(2))
        .expect("limited open authority");
    let error = analyze_source_document_v2(DOC_SOURCE, "show capabilities", &limited)
        .expect_err("a public analysis result must not exceed its own receipt");
    let FrontendTransactionErrorV2::Document(failure) = error else {
        panic!("diagnostic exhaustion must be source-anchored");
    };
    assert_eq!(failure.diagnostics().len(), 1);
    assert_eq!(
        failure.diagnostics()[0].code(),
        "pipeline.analysis.diagnostic_limit_exceeded"
    );
    assert_eq!(failure.diagnostics()[0].primary().source_id, DOC_SOURCE);
}

#[test]
fn incomplete_open_analysis_rejects_overdeep_relation_before_declaration() {
    let authority = OpenAnalysisAuthorityV2::try_standard_v2().expect("standard open authority");
    let source = format!("from records r{}", " | where true".repeat(4_096));
    let error = analyze_source_document_v2(DOC_SOURCE, &source, &authority)
        .expect_err("relation depth must be bounded even without closed authority");
    let FrontendTransactionErrorV2::Document(failure) = error else {
        panic!("relation depth must have a source-anchored diagnostic");
    };
    assert_eq!(
        failure.diagnostics()[0].code(),
        "surface.resource_exhausted"
    );
}

/// Build one closed authority with a module graph and no relation/graph
/// authority, solely from sema/modules re-exports.
fn minimal_authority() -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: HOST_SOURCE,
        source_len: 64,
    }])
    .expect("host registry admits one source");
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: HOST_SOURCE,
            span: SpanV1::new(0, 1),
        })
        .expect("checked host location");
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let profile = budget.authority_profile().expect("budget profile");
    let module = ModuleIdV2::try_new("app", location.clone()).expect("module identity");
    let mut builder =
        ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).expect("module builder");
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .expect("module node admits");
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location,
            },
            VersionV2::new(1, 0, 0),
        )
        .expect("module authority seals");
    let relations =
        RelationCatalogV2::try_new(Vec::new(), &host, profile.clone()).expect("relation catalog");
    let graph_references =
        GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile).expect("graph catalog");
    ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, budget)
        .expect("closed authority seals")
}

/// Build one complete open authority carrying every proof.
fn complete_open_authority() -> OpenAnalysisAuthorityV2 {
    open_authority_mask(0b111)
}

/// Build one of the eight module/relation/graph proof masks.
fn open_authority_mask(mask: u8) -> OpenAnalysisAuthorityV2 {
    let closed = minimal_authority();
    OpenAnalysisAuthorityV2::try_new(OpenAnalysisAuthorityInputV2::new(
        (mask & 0b001 != 0).then(|| closed.modules().clone()),
        (mask & 0b010 != 0).then(|| closed.relations().clone()),
        (mask & 0b100 != 0).then(|| closed.graph_references().clone()),
        closed.modules().source_registry().clone(),
        closed.budget().clone(),
    ))
    .expect("open authority admits complete proofs")
}

/// The diagnostics carried by one blocked analysis item.
fn blocked_diagnostics(item: &AnalysisItemV2) -> Vec<String> {
    let AnalysisItemV2::BlockedByDiagnostics(diagnostics) = item else {
        panic!("the item must be blocked");
    };
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code().to_owned())
        .collect()
}

#[test]
fn resolved_relation_analysis_is_complete() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let analyzed = analyze_resolved_document_v2(fixture.document()).expect("analysis resolves");
    assert_eq!(analyzed.source_id(), fixture.document().source_id());
    assert_eq!(analyzed.order(), [fixture.item_id()]);
    assert!(analyzed.unresolved().is_empty());
    assert!(analyzed.rule_program().is_none());
    let AnalysisItemV2::Complete(planned) = &analyzed.items()[0] else {
        panic!("a resolved relation is Complete with a planned payload");
    };
    let PlannedItemV2::Relation { item_id, .. } = planned.as_ref() else {
        panic!("a resolved relation retains its relation plan");
    };
    assert_eq!(*item_id, fixture.item_id());
    assert!(analyzed.item(fixture.item_id()).is_some());
    assert!(analyzed.item(reference_absent_item_id_v2()).is_none());
}

#[test]
fn partial_open_authority_blocks_every_item_with_unresolved_facts() {
    let authority = OpenAnalysisAuthorityV2::try_standard_v2().expect("standard open authority");
    let analyzed = analyze_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &authority)
        .expect("partial authority is not an error");
    assert_eq!(analyzed.order().len(), 3);
    assert_eq!(analyzed.items().len(), 3);
    assert!(analyzed.rule_program().is_none());
    let compiled = compile_source_document_v3(DOC_SOURCE, MINIMAL_SOURCE, &minimal_authority())
        .expect("the same source compiles");
    assert_eq!(
        analyzed.order(),
        compiled
            .items()
            .iter()
            .map(CompiledItemV2::item_id)
            .collect::<Vec<_>>(),
        "a blocked analysis lists authored items in exactly the compiled source order"
    );
    assert_eq!(analyzed.unresolved().len(), 3);
    for fact in analyzed.unresolved() {
        assert_eq!(fact.code(), OPEN_AUTHORITY_INCOMPLETE_CODE);
        assert_eq!(fact.location().source_id, DOC_SOURCE);
    }
    for (index, item) in analyzed.items().iter().enumerate() {
        let codes = blocked_diagnostics(item);
        for expected in [
            "pipeline.analysis.missing_module_authority",
            "pipeline.analysis.missing_relation_authority",
            "pipeline.analysis.missing_graph_authority",
        ] {
            assert!(
                codes.iter().any(|code| code == expected),
                "blocked item {index} must name {expected}: {codes:?}"
            );
        }
    }
}

#[test]
fn all_open_authority_masks_have_exact_status_and_missing_facts() {
    for mask in 0_u8..=0b111 {
        let authority = open_authority_mask(mask);
        let analyzed = analyze_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &authority)
            .expect("every proof mask is a valid open analysis authority");
        if mask == 0b111 {
            assert_eq!(analyzed.status().keyword(), "complete");
            assert!(analyzed.unresolved().is_empty());
            assert!(analyzed.diagnostics().is_empty());
            continue;
        }

        assert_eq!(analyzed.status().keyword(), "incomplete", "mask={mask:03b}");
        assert_eq!(analyzed.unresolved().len(), 3, "mask={mask:03b}");
        assert_eq!(
            analyzed.items().len(),
            3,
            "mask={mask:03b}: a partial authority must still author every item"
        );
        assert!(analyzed
            .items()
            .iter()
            .all(|item| matches!(item, AnalysisItemV2::BlockedByDiagnostics(_))));
        let actual = analyzed
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code())
            .collect::<Vec<_>>();
        let expected = [
            (0b100, "pipeline.analysis.missing_graph_authority"),
            (0b001, "pipeline.analysis.missing_module_authority"),
            (0b010, "pipeline.analysis.missing_relation_authority"),
        ]
        .into_iter()
        .filter_map(|(bit, code)| (mask & bit == 0).then_some(code))
        .collect::<Vec<_>>();
        assert_eq!(actual, expected, "mask={mask:03b}");
    }
}

#[test]
fn empty_documents_are_complete_for_every_open_authority_mask() {
    for source in ["", " \n\t# comment-only document"] {
        for mask in 0_u8..=0b111 {
            let analyzed =
                analyze_source_document_v2(DOC_SOURCE, source, &open_authority_mask(mask))
                    .expect("empty source is a valid open analysis document");
            assert_eq!(analyzed.status().keyword(), "complete", "mask={mask:03b}");
            assert!(analyzed.order().is_empty());
            assert!(analyzed.items().is_empty());
            assert!(analyzed.unresolved().is_empty());
            assert!(analyzed.diagnostics().is_empty());
        }
    }
}

#[test]
fn complete_open_authority_carries_complete_payloads() {
    let authority = complete_open_authority();
    let analyzed = analyze_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &authority)
        .expect("complete open authority resolves");
    assert!(analyzed.unresolved().is_empty());
    assert_eq!(analyzed.order().len(), 3);
    assert!(analyzed.rule_program().is_some());
    let dispositions: Vec<&'static str> = analyzed
        .items()
        .iter()
        .map(|item| match item {
            AnalysisItemV2::Complete(planned) => match planned.as_ref() {
                PlannedItemV2::RuleMember(_) => "rule_member",
                PlannedItemV2::Presentation { .. } => "presentation",
                PlannedItemV2::Relation { .. }
                | PlannedItemV2::Graph { .. }
                | PlannedItemV2::Command { .. }
                | PlannedItemV2::Check { .. }
                | PlannedItemV2::Composition { .. }
                | PlannedItemV2::MetaRequireCertified { .. } => "planned",
            },
            AnalysisItemV2::NotApplicable(_) => "not_applicable",
            AnalysisItemV2::BlockedByDiagnostics(_) => "blocked",
        })
        .collect();
    assert_eq!(
        dispositions,
        ["not_applicable", "rule_member", "presentation"]
    );
    // Three different dispositions make returning the first item for every
    // registered identity observable, independently of lookup membership.
    for (index, item_id) in analyzed.order().iter().enumerate() {
        assert_eq!(analyzed.item(*item_id), Some(&analyzed.items()[index]));
    }
    assert_eq!(analyzed.item(reference_absent_item_id_v2()), None);
    let AnalysisItemV2::NotApplicable(reason) = &analyzed.items()[0] else {
        panic!("a value declaration has no plan");
    };
    assert_eq!(
        reason,
        &quanta_gqlang_v2_planning::NoPlanReasonV2::ValueDeclaration
    );
}

#[test]
fn owner_stage_failure_blocks_the_item_without_failing_the_document() {
    let over = forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_MAX_SEMANTIC_STEPS)
        .expect("over-limit chain builds");
    let analyzed = analyze_resolved_document_v2(over.document())
        .expect("an owner stage failure stays inside the analysis document");
    let codes = blocked_diagnostics(&analyzed.items()[0]);
    assert_eq!(codes, ["algebra.traversal_budget_exceeded"]);
}

#[test]
fn a_blocked_rule_program_blocks_every_rule_member() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::MutualRecursionPair,
        ForgedRuleFactMutationV2::TargetTransplanted,
    )
    .expect("forged rule facts build");
    let analyzed = analyze_resolved_document_v2(forged.document())
        .expect("a rule-program failure stays inside the analysis document");
    assert!(analyzed.rule_program().is_none());
    assert_eq!(analyzed.items().len(), forged.item_ids().len());
    assert!(
        !analyzed.items().is_empty(),
        "the forged rule program must block at least one rule member"
    );
    for (index, item) in analyzed.items().iter().enumerate() {
        assert_eq!(
            blocked_diagnostics(item),
            ["logic.rule_program.target_transplanted"],
            "rule member {index} must report the exact logic code"
        );
    }
}

#[test]
fn blocked_analysis_is_not_compilable() {
    let over = forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_MAX_SEMANTIC_STEPS)
        .expect("over-limit chain builds");
    let error = quanta_gqlang_v2_pipeline::compile_resolved_document_v2(over.document())
        .expect_err("a closed compile fails closed on the same document");
    let FrontendTransactionErrorV2::Document(failure) = error else {
        panic!("a source-anchored stage failure is a document failure");
    };
    assert_eq!(
        failure
            .diagnostics()
            .first()
            .expect("one diagnostic")
            .code(),
        "algebra.traversal_budget_exceeded"
    );
    let blocked = analyze_resolved_document_v2(over.document())
        .expect("analysis still reports the blocked item");
    assert_eq!(
        blocked_diagnostics(&blocked.items()[0]),
        ["algebra.traversal_budget_exceeded"],
        "the analysis must still name the owner-stage failure, not merely succeed"
    );
}

#[test]
fn closed_and_open_entries_agree_on_the_same_source() {
    let closed = minimal_authority();
    let compiled = compile_source_document_v3(DOC_SOURCE, MINIMAL_SOURCE, &closed)
        .expect("the minimal document compiles");
    let analyzed =
        analyze_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &complete_open_authority())
            .expect("the same source analyzes");
    assert_eq!(compiled.items().len(), analyzed.items().len());
    assert!(
        !compiled.items().is_empty(),
        "the minimal document must compile at least one item to compare"
    );
    for index in 0..compiled.items().len() {
        match (&compiled.items()[index], &analyzed.items()[index]) {
            (CompiledItemV2::Planned(compiled_item), AnalysisItemV2::Complete(analyzed_item)) => {
                assert_eq!(compiled_item, analyzed_item);
            }
            (CompiledItemV2::NoPlan { reason, .. }, AnalysisItemV2::NotApplicable(analyzed)) => {
                assert_eq!(reason, analyzed);
            }
            _ => panic!("closed compile and open analysis must agree item by item"),
        }
    }
}

#[test]
fn sealed_analysis_reads_the_document_budget_receipt() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let analyzed = analyze_resolved_document_v2(fixture.document()).expect("analysis resolves");
    let AnalysisItemV2::Complete(planned) = &analyzed.items()[0] else {
        panic!("the sealed relation resolves");
    };
    let PlannedItemV2::Relation { item_id, .. } = planned.as_ref() else {
        panic!("the sealed relation retains its relation plan");
    };
    assert_eq!(*item_id, fixture.item_id());
    let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
        DOC_SOURCE,
        MINIMAL_SOURCE,
        &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
    )
    .expect("minimal source parses");
    let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("HIR integrity holds");
    let document =
        quanta_gqlang_v2_sema::resolve_checked_document_v2(checked, &minimal_authority())
            .expect("semantic transaction succeeds");
    assert_eq!(
        document.budget_receipt(),
        minimal_authority().budget().downstream_receipt()
    );
}
