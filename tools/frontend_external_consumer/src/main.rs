//! External-consumer positive program for the pinned facade (lane W4-H).
//!
//! This crate is NOT a member of the repository workspace, and its only
//! dependency is `quanta-gqlang-v2-frontend-contract`. Everything below is
//! therefore exactly what a sibling repository can see. If a curated root
//! disappears, is renamed, or loses a public accessor or signature, this
//! program stops compiling — that is the point of the gate.
//!
//! Closed-authority status (recorded, not worked around): the facade exposes
//! `compile_source_document_v2` and `ClosedFrontendAuthorityV2`, but no
//! allowlisted type can build a closed authority yet — `try_new` takes
//! `ValidatedModuleAuthorityPartsV2` / `RelationCatalogV2` /
//! `GraphReferenceCatalogV2`, which are owner types the allowlist does not
//! carry (the facade documents this as pending Ticket 07 builder DTOs). The
//! closed entry point is therefore pinned at the exact external *signature*
//! level (`CLOSED_COMPILE_ENTRY_V2`) rather than called with a fabricated
//! authority. Nothing here reaches into an owner crate.

#![forbid(unsafe_code)]

use quanta_gqlang_v2_frontend_contract::{
    analyze_source_document_v2, compile_source_document_v2, format_source_document_v2,
    AlgebraFamilyV2, AnalysisDocumentV2, AnalysisItemV2, CheckedLocatedSpanV2,
    ClosedFrontendAuthorityV2, CompiledDocumentV2, CompiledItemV2, CompiledRuleProgramV2,
    FrontendAuthorityErrorV2, FrontendBudgetV2, FrontendDiagnosticV2, FrontendFailureV2,
    FrontendSeverityV2, FrontendStageV2, FrontendTransactionErrorV2, LocatedSpanV2, NoPlanReasonV2,
    OpenAnalysisAuthorityV2, PlannedItemV2, RuleProgramMemberV2, SemanticPlanV2, SourceIdV2,
    SpanV1, UnresolvedAnalysisFactV2,
};

/// Document identity of the probe source.
const DOCUMENT_SOURCE: SourceIdV2 = SourceIdV2::from_raw(9);

/// A source document that needs no relation, module, or graph authority.
const DOCUMENT_TEXT: &str = "let answer = 1; rule local_rule : x if x = 1; show capabilities;";

/// Code every unresolved fact carries while the open authority is partial.
///
/// The facade exports `UnresolvedAnalysisFactV2` but not the pipeline constant
/// that names this code, so an external consumer pins the literal.
const OPEN_AUTHORITY_INCOMPLETE_CODE_V2: &str = "pipeline.analysis.open_authority_incomplete";

/// The three authority stages an open editor session is missing here.
const MISSING_AUTHORITY_CODES_V2: [&str; 3] = [
    "pipeline.analysis.missing_module_authority",
    "pipeline.analysis.missing_relation_authority",
    "pipeline.analysis.missing_graph_authority",
];

/// Exact external signature of the closed-source entry point.
///
/// The facade must keep exporting `compile_source_document_v2` with this shape
/// even while no allowlisted builder can mint a `ClosedFrontendAuthorityV2`.
type ClosedCompileEntryV2 = fn(
    SourceIdV2,
    &str,
    &ClosedFrontendAuthorityV2,
) -> Result<CompiledDocumentV2, FrontendTransactionErrorV2>;

/// Signature pin: the closed-source entry point, named but never called.
const CLOSED_COMPILE_ENTRY_V2: ClosedCompileEntryV2 = compile_source_document_v2;

/// Name a facade item in a monomorphic position without invoking it.
///
/// Roots whose values only an owner crate (or a not-yet-public builder) can
/// mint are still proved *reachable*: the item must exist, and its exact
/// signature must hold.
fn pin<T>(_item: T) {}

/// Exhaustive read of the closed compiled-item vocabulary.
fn compiled_item_label(item: &CompiledItemV2) -> &'static str {
    match item {
        CompiledItemV2::Planned(planned) => match planned {
            PlannedItemV2::Relation { .. } => "planned-relation",
            PlannedItemV2::Graph { .. } => "planned-graph",
            PlannedItemV2::Check { .. } => "planned-check",
            PlannedItemV2::Presentation { .. } => "planned-presentation",
            PlannedItemV2::Composition { .. } => "planned-composition",
            PlannedItemV2::MetaRequireCertified { .. } => "planned-meta-require-certified",
            PlannedItemV2::RuleMember(_) => "planned-rule-member",
        },
        CompiledItemV2::NoPlan { .. } => "no-plan",
    }
}

/// A compiled item carries a plan or a non-plan reason, never both and never
/// neither, and its identity is stable across reads.
fn compiled_item_invariants_hold(item: &CompiledItemV2) -> bool {
    item.plan().is_some() == item.no_plan_reason().is_none() && item.item_id() == item.item_id()
}

/// Every accessor `CompiledDocumentV2` exposes to a sibling repository.
fn compiled_document_probe(document: &CompiledDocumentV2) -> (SourceIdV2, usize, usize, bool) {
    (
        document.source_id(),
        document.diagnostics().len(),
        document.items().len(),
        document.rule_program().is_some(),
    )
}

/// Every accessor `CompiledRuleProgramV2` exposes to a sibling repository.
fn compiled_rule_program_probe(program: &CompiledRuleProgramV2) -> (usize, bool) {
    let members = program.members();
    let stratified = std::mem::size_of_val(program.stratification()) > 0;
    (members.len(), stratified)
}

/// Every accessor `RuleProgramMemberV2` exposes to a sibling repository.
fn rule_member_probe(member: &RuleProgramMemberV2) -> bool {
    member.program_id() == member.program_id() && member.member_item_id() == member.member_item_id()
}

/// Every accessor `PlannedItemV2` exposes to a sibling repository.
fn planned_item_probe(item: &PlannedItemV2) -> (bool, bool) {
    let has_plan = item.plan().is_some();
    let has_rule_member = item.rule_member().is_some();
    let stable_identity = item.item_id() == item.item_id();
    (has_plan || has_rule_member, stable_identity)
}

/// Every accessor `SemanticPlanV2` exposes to a sibling repository.
fn semantic_plan_probe(plan: &SemanticPlanV2) -> usize {
    let lane = plan.authority_lane();
    let determinism = plan.evaluation_determinism();
    let evidence = plan.evidence();
    let effect = plan.effect();
    let result = plan.result();
    let cost = plan.semantic_cost();
    let explain = plan.explain();
    let policy = plan.policy();
    std::mem::size_of_val(&lane)
        + std::mem::size_of_val(&determinism)
        + std::mem::size_of_val(&evidence)
        + std::mem::size_of_val(&effect)
        + std::mem::size_of_val(result)
        + std::mem::size_of_val(cost)
        + std::mem::size_of_val(explain)
        + std::mem::size_of_val(policy)
        + plan.capabilities().len()
}

/// An unresolved open-analysis fact reports its code and exact location.
fn unresolved_fact_probe(fact: &UnresolvedAnalysisFactV2) -> (&'static str, LocatedSpanV2) {
    (fact.code(), fact.location())
}

/// The checked-location closure type reads back the location it was minted for.
fn checked_span_probe(span: &CheckedLocatedSpanV2) -> (LocatedSpanV2, usize) {
    let identity = span.registry_identity();
    (span.location(), std::mem::size_of_val(identity))
}

/// A sealed authority reports the exact budget that sealed it.
fn closed_authority_probe(authority: &ClosedFrontendAuthorityV2) -> &FrontendBudgetV2 {
    authority.budget()
}

/// Exhaustive read of the closed algebra-family vocabulary.
fn algebra_family_label(family: AlgebraFamilyV2) -> &'static str {
    match family {
        AlgebraFamilyV2::Scan => "scan",
        AlgebraFamilyV2::Filter => "filter",
        AlgebraFamilyV2::Project => "project",
        AlgebraFamilyV2::Join => "join",
        AlgebraFamilyV2::Apply => "apply",
        AlgebraFamilyV2::Group => "group",
        AlgebraFamilyV2::Aggregate => "aggregate",
        AlgebraFamilyV2::Sort => "sort",
        AlgebraFamilyV2::Limit => "limit",
        AlgebraFamilyV2::Union => "union",
        AlgebraFamilyV2::SemiJoin => "semi-join",
        AlgebraFamilyV2::AntiJoin => "anti-join",
    }
}

/// Exhaustive read of the closed non-plan vocabulary.
fn no_plan_reason_label(reason: NoPlanReasonV2) -> &'static str {
    match reason {
        NoPlanReasonV2::CatalogDeclaration { .. } => "catalog-declaration",
        NoPlanReasonV2::ValueDeclaration => "value-declaration",
        NoPlanReasonV2::MetaActivation => "meta-activation",
        NoPlanReasonV2::EmptyRuleProgram => "empty-rule-program",
    }
}

/// Pin every curated root that no allowlisted constructor can yet mint.
fn pin_unconstructible_roots() {
    pin(compiled_item_label as fn(&CompiledItemV2) -> &'static str);
    pin(compiled_item_invariants_hold as fn(&CompiledItemV2) -> bool);
    pin(compiled_document_probe as fn(&CompiledDocumentV2) -> (SourceIdV2, usize, usize, bool));
    pin(compiled_rule_program_probe as fn(&CompiledRuleProgramV2) -> (usize, bool));
    pin(rule_member_probe as fn(&RuleProgramMemberV2) -> bool);
    pin(planned_item_probe as fn(&PlannedItemV2) -> (bool, bool));
    pin(semantic_plan_probe as fn(&SemanticPlanV2) -> usize);
    pin(checked_span_probe as fn(&CheckedLocatedSpanV2) -> (LocatedSpanV2, usize));
    pin(closed_authority_probe as fn(&ClosedFrontendAuthorityV2) -> &FrontendBudgetV2);
    pin(unresolved_fact_probe as fn(&UnresolvedAnalysisFactV2) -> (&'static str, LocatedSpanV2));
    pin(CLOSED_COMPILE_ENTRY_V2);
}

fn main() {
    let budget: FrontendBudgetV2 = FrontendBudgetV2::standard_v2().expect("standard budget");
    assert!(
        budget.max_source_bytes() > 0,
        "the canonical budget is non-degenerate"
    );

    // ---- format_source_document_v2: twice, deterministic, idempotent -------
    let first: Result<String, FrontendFailureV2> =
        format_source_document_v2(DOCUMENT_SOURCE, DOCUMENT_TEXT, &budget);
    let first = first.expect("the probe source formats under the canonical budget");
    let second: Result<String, FrontendFailureV2> =
        format_source_document_v2(DOCUMENT_SOURCE, DOCUMENT_TEXT, &budget);
    let second = second.expect("formatting is repeatable");
    assert_eq!(first, second, "one input formats identically twice");
    assert!(!first.is_empty(), "the canonical form is not empty");
    let canonical = first.trim_end_matches('\n');
    let third = format_source_document_v2(DOCUMENT_SOURCE, canonical, &budget)
        .expect("the canonical form re-parses");
    assert_eq!(first, third, "fmt(fmt(x)) == fmt(x) through the facade");

    // ---- analyze_source_document_v2 under the open (partial) authority -----
    let authority: Result<OpenAnalysisAuthorityV2, FrontendAuthorityErrorV2> =
        OpenAnalysisAuthorityV2::try_standard_v2();
    let authority = authority.expect("the standard open authority seals");
    let analyzed: Result<AnalysisDocumentV2, FrontendTransactionErrorV2> =
        analyze_source_document_v2(DOCUMENT_SOURCE, DOCUMENT_TEXT, &authority);
    let analyzed = analyzed.expect("a partial authority is not an error");
    assert_eq!(
        analyzed.source_id(),
        DOCUMENT_SOURCE,
        "analysis keeps the source identity"
    );
    assert_eq!(
        analyzed.order().len(),
        3,
        "the probe source authors three items"
    );
    assert_eq!(
        analyzed.items().len(),
        analyzed.order().len(),
        "order names every item"
    );
    assert!(
        analyzed.rule_program().is_none(),
        "a partial authority seals no rule program"
    );

    let first_id = analyzed.order()[0];
    assert!(
        analyzed.item(first_id).is_some(),
        "an authored identity resolves to its status"
    );

    assert_eq!(
        analyzed.unresolved().len(),
        3,
        "every blocked item carries one unresolved fact"
    );
    for fact in analyzed.unresolved() {
        assert_eq!(fact.code(), OPEN_AUTHORITY_INCOMPLETE_CODE_V2);
        let location: LocatedSpanV2 = fact.location();
        assert_eq!(
            location.source_id, DOCUMENT_SOURCE,
            "unresolved facts are source-anchored"
        );
        assert!(
            location.span.start <= location.span.end,
            "spans are well formed"
        );
    }

    let mut blocked = 0_usize;
    for item in analyzed.items() {
        match item {
            AnalysisItemV2::BlockedByDiagnostics(diagnostics) => {
                assert!(
                    !diagnostics.is_empty(),
                    "a blocked item names at least one diagnostic"
                );
                blocked += 1;
            }
            AnalysisItemV2::Complete(_) | AnalysisItemV2::NotApplicable(_) => {}
        }
    }
    assert_eq!(blocked, 3, "a partial authority blocks every authored item");
    let AnalysisItemV2::BlockedByDiagnostics(first_diagnostics) = &analyzed.items()[0] else {
        panic!("the first authored item is blocked under a partial authority");
    };
    let first_codes: Vec<&'static str> = first_diagnostics
        .iter()
        .map(FrontendDiagnosticV2::code)
        .collect();
    for expected in MISSING_AUTHORITY_CODES_V2 {
        assert!(
            first_codes.contains(&expected),
            "the blocked item must name {expected}: {first_codes:?}"
        );
    }

    // ---- diagnostic / failure / transaction vocabulary ---------------------
    let diagnostic = FrontendDiagnosticV2::new(
        FrontendStageV2::Resource,
        "external.consumer.probe",
        FrontendSeverityV2::Warning,
        LocatedSpanV2 {
            source_id: DOCUMENT_SOURCE,
            span: SpanV1::new(0, 1),
        },
        vec![LocatedSpanV2 {
            source_id: DOCUMENT_SOURCE,
            span: SpanV1::new(1, 2),
        }],
        "external consumer diagnostic probe",
    );
    assert_eq!(diagnostic.stage(), FrontendStageV2::Resource);
    assert_eq!(diagnostic.code(), "external.consumer.probe");
    assert_eq!(diagnostic.severity(), FrontendSeverityV2::Warning);
    assert_eq!(diagnostic.primary().span, SpanV1::new(0, 1));
    assert_eq!(diagnostic.primary().source_id.raw(), DOCUMENT_SOURCE.raw());
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.message(), "external consumer diagnostic probe");

    let failure: FrontendFailureV2 = FrontendFailureV2::one(diagnostic.clone());
    assert_eq!(
        failure.diagnostics().len(),
        1,
        "one diagnostic in, one diagnostic out"
    );
    assert!(
        FrontendFailureV2::try_new(Vec::new()).is_err(),
        "a failure without a diagnostic is not constructible"
    );
    assert!(FrontendFailureV2::try_new(vec![diagnostic.clone()]).is_ok());

    let transaction_error: FrontendTransactionErrorV2 =
        FrontendTransactionErrorV2::Document(failure);
    match &transaction_error {
        FrontendTransactionErrorV2::Authority(error) => {
            assert!(
                !error.code().is_empty(),
                "an authority failure names its code"
            );
        }
        FrontendTransactionErrorV2::Document(failure) => {
            assert_eq!(failure.diagnostics().len(), 1);
        }
    }

    // ---- plan / no-plan vocabulary ----------------------------------------
    let reason: NoPlanReasonV2 = NoPlanReasonV2::ValueDeclaration;
    assert_eq!(no_plan_reason_label(reason), "value-declaration");
    assert_eq!(
        no_plan_reason_label(NoPlanReasonV2::MetaActivation),
        "meta-activation"
    );
    assert_eq!(
        no_plan_reason_label(NoPlanReasonV2::EmptyRuleProgram),
        "empty-rule-program"
    );
    let not_applicable: AnalysisItemV2 =
        AnalysisItemV2::NotApplicable(NoPlanReasonV2::ValueDeclaration);
    let blocked_item: AnalysisItemV2 =
        AnalysisItemV2::BlockedByDiagnostics(vec![diagnostic].into_boxed_slice());
    for item in [&not_applicable, &blocked_item] {
        let label = match item {
            AnalysisItemV2::Complete(_) => "complete",
            AnalysisItemV2::NotApplicable(_) => "not-applicable",
            AnalysisItemV2::BlockedByDiagnostics(diagnostics) => {
                assert!(!diagnostics.is_empty());
                "blocked"
            }
        };
        assert!(!label.is_empty());
    }

    // ---- closed algebra vocabulary ----------------------------------------
    assert_eq!(algebra_family_label(AlgebraFamilyV2::Scan), "scan");
    assert_eq!(algebra_family_label(AlgebraFamilyV2::AntiJoin), "anti-join");

    // ---- roots that stay type-reachable until the builder DTOs land -------
    pin_unconstructible_roots();

    println!("external-consumer positive: ok");
}
