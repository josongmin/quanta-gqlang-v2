// Edge case audit: decl/rule rejection, required clause presence
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration test audit: panic on failure is intentional"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1, SpanV1};
use quanta_gqlang_v2_pipeline::{
    compile_item_structural, compile_program_structural, CompiledItemV1, PipelineStageV1,
};

// Test 1: Decl compiles to an explicit non-relational artifact
#[test]
fn decl_compiles_to_value_artifact() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);

    // A let decl: `let x = 1`
    let decl = builder.decl("x", span, builder.int("1", span), span);
    let item = ProgramItemHirV1::from(decl);

    let result = compile_item_structural(&item);
    let compiled = result.expect("let decl must compile to artifact");
    assert!(
        matches!(compiled, CompiledItemV1::ValueArtifact { .. }),
        "should surface ValueArtifact"
    );
}

// Test 2: Rule compiles to an explicit logic artifact
#[test]
fn rule_compiles_to_logic_artifact() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);

    // A rule: `rule p if true` - "p" is an unresolved symbol
    let head = builder.symbol("p", span);
    let body = builder.bool_literal(true, span);
    let rule = builder.rule("p", span, head, body, span);
    let item = ProgramItemHirV1::from(rule);

    let result = compile_item_structural(&item);
    let compiled = result.expect("rule must compile to artifact");
    assert!(
        matches!(compiled, CompiledItemV1::RuleArtifact { .. }),
        "should surface RuleArtifact"
    );
}

// Test 3: Unknown FIND subkind should parse-reject
#[test]
fn find_unknown_subkind_rejects() {
    let src = "find unknown src -> dst | depth=1";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "unknown FIND subkind must reject");
}

// Test 4: Unbounded FIND rejects logic stage (not planning)
#[test]
fn unbounded_find_rejects_logic_not_planning() {
    let src = "find taint api.entry -> db.write";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "unbounded taint must reject");
    let err = result.unwrap_err();
    assert_eq!(
        err.stage(),
        PipelineStageV1::Logic,
        "unbounded witness should reject at logic"
    );
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
}

// Test 5: CHECK with missing required confirm should parse-reject
#[test]
fn check_missing_confirm_rejects_parse() {
    let src = "check rule.x | scope=repo | emit=findings";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "CHECK without confirm must reject at parse"
    );
}

// Test 6: SAMPLE with zero limit should parse-reject
#[test]
fn sample_zero_limit_rejects_parse() {
    let src = "sample query=\"select 1\" | mode=lexical | limit=0";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "SAMPLE with limit=0 must reject at parse");
}

// Test 7: COMPARE missing required candidate should parse-reject
#[test]
fn compare_missing_candidate_rejects_parse() {
    let src = "compare baseline=v1 | metric=semantic";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "COMPARE without candidate must reject at parse"
    );
}

// Test 8: EXPORT missing required kind should parse-reject
#[test]
fn export_missing_kind_rejects_parse() {
    let src = "export source=q";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "EXPORT without kind must reject at parse");
}

// Test 9: Composition with undefined WITH binding is a parse-time guard
#[test]
fn composition_undefined_with_parse_guard() {
    // The surface parser guards this at parse time as a structural constraint
    let src = "let q = find taint a -> b | depth=1 find taint x -> y | depth=2 with undefined";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "undefined WITH name must reject at parse");
}

// Test 10: Composition with duplicate LET name should parse-reject
#[test]
fn composition_duplicate_let_rejects_parse() {
    let src = "let q = find taint a -> b | depth=1 let q = find taint x -> y | depth=2 find taint p -> q | depth=1 with q";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "duplicate LET name must reject at parse");
}

// Test 11: Composition with invalid producer kind should parse-reject
#[test]
fn composition_invalid_producer_kind_rejects_parse() {
    let src = "let q = check rule.x | confirm=skip from users with q";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "invalid composition producer must reject");
}

// Test 12: Composition with invalid body kind should parse-reject
#[test]
fn composition_invalid_body_kind_rejects_parse() {
    let src = "let q = find taint a -> b | depth=1 gate g | checks=c | binding=all with q";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "invalid composition body must reject");
}

// Test 13: PROFILE with missing admission should parse-reject
#[test]
fn profile_missing_admission_rejects_parse() {
    let src = "profile p | capabilities=c1,c2";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "PROFILE without admission must reject at parse"
    );
}

// Test 14: GATE with missing binding should parse-reject
#[test]
fn gate_missing_binding_rejects_parse() {
    let src = "gate g | checks=c1,c2";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "GATE without binding must reject at parse");
}

// Test 15: REQUIRE CERTIFIED works without arguments
#[test]
fn require_certified_no_args() {
    let src = "require certified";
    let compiled = compile_program_structural(src).expect("REQUIRE CERTIFIED compiles");
    assert!(compiled.plan().is_some(), "REQUIRE CERTIFIED carries plan");
}

// Test 16: USE PACK missing lockfile should parse-reject
#[test]
fn use_pack_missing_lockfile_rejects_parse() {
    let src = "use pack my.pack | activation=session";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "USE PACK without lockfile must reject at parse"
    );
}

// Test 17: PACK missing activation should parse-reject
#[test]
fn pack_missing_activation_rejects_parse() {
    let src = "pack p | source=src";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "PACK without activation must reject at parse"
    );
}

// Test 18: MODEL missing provider should parse-reject
#[test]
fn model_missing_provider_rejects_parse() {
    let src = "model m | source=src";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "MODEL without provider must reject at parse"
    );
}

// Test 19: MODEL missing source should parse-reject
#[test]
fn model_missing_source_rejects_parse() {
    let src = "model m | provider=openai";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "MODEL without source must reject at parse");
}

// Test 20: RULEDEF missing check-ref should parse-reject
#[test]
fn ruledef_missing_check_ref_rejects_parse() {
    let src = "ruledef r | source=x";
    let result = compile_program_structural(src);
    assert!(
        result.is_err(),
        "RULEDEF without check-ref must reject at parse"
    );
}

// Test 21: SAMPLE missing limit should parse-reject
#[test]
fn sample_missing_limit_rejects_parse() {
    let src = "sample query=\"select 1\" | mode=lexical";
    let result = compile_program_structural(src);
    assert!(result.is_err(), "SAMPLE without limit must reject at parse");
}
