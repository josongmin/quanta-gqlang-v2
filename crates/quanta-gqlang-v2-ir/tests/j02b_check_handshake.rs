#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::check_ir::{
    CheckConfirmIntentV1, CheckEmitIntentV1, CheckExecutionModeV1, CheckIrV1,
};
use quanta_gqlang_v2_ir::lower_root_to_ir_v1;
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;
use quanta_gqlang_v2_sema::analyze_root_v1;
use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::roots::check::{CheckRootAstV1, CheckRootIntentV1};

fn lower_check_from_intent_v1(intent: CheckRootIntentV1) -> CheckIrV1 {
    let typed = analyze_root_v1(&AstRootV1::Check(CheckRootAstV1::from_intent(
        intent.clone(),
    )))
    .unwrap_or_else(|page| panic!("expected sema success for `{intent:?}`: {page:?}"));
    let ir = lower_root_to_ir_v1(&typed)
        .unwrap_or_else(|err| panic!("expected IR lowering success for `{intent:?}`: {err:?}"));
    let PublicIrRootV1::Check(check) = ir else {
        panic!("expected CHECK IR root");
    };
    check
}

#[test]
fn check_handshake_preserves_pattern_semantic_and_hybrid_intent() {
    let pattern_only = CheckRootIntentV1::new(
        "rule.pattern/path-filter",
        "pattern",
        None,
        Some("path ~= \"src/**/*.rs\"".to_string()),
        CheckConfirmIntentV1::Prompt,
        CheckEmitIntentV1::Findings,
    )
    .expect("pattern intent");
    let pattern_ir = lower_check_from_intent_v1(pattern_only.clone());
    assert_eq!(pattern_ir.intent(), &pattern_only);
    assert_eq!(
        pattern_ir.execution_mode(),
        CheckExecutionModeV1::PatternOnly
    );

    let semantic_only = CheckRootIntentV1::new(
        "rule.semantic/ownership",
        "policy",
        Some("repo://frontend".to_string()),
        None,
        CheckConfirmIntentV1::Skip,
        CheckEmitIntentV1::Findings,
    )
    .expect("semantic intent");
    let semantic_ir = lower_check_from_intent_v1(semantic_only.clone());
    assert_eq!(semantic_ir.intent(), &semantic_only);
    assert_eq!(
        semantic_ir.execution_mode(),
        CheckExecutionModeV1::SemanticOnly
    );

    let hybrid = CheckRootIntentV1::new(
        "rule.hybrid/shell-audit",
        "security",
        Some("repo://runtime".to_string()),
        Some("call = \"std::process::Command\"".to_string()),
        CheckConfirmIntentV1::Require,
        CheckEmitIntentV1::Findings,
    )
    .expect("hybrid intent");
    let hybrid_ir = lower_check_from_intent_v1(hybrid.clone());
    assert_eq!(hybrid_ir.intent(), &hybrid);
    assert_eq!(hybrid_ir.execution_mode(), CheckExecutionModeV1::Hybrid);
}

#[test]
fn check_handshake_rejects_empty_semantic_and_pattern_intent_fail_closed() {
    // The malformed payload now fails closed at construction (lowering), where
    // the decode happens once — not on a later `intent()` read.
    let error = CheckIrV1::from_typed(&quanta_gqlang_v2_sema::roots::check::TypedCheckRootV1 {
        rule_id: "rule.invalid".to_string(),
        scope: "check:v1|class=security|scope=~|pattern=~|confirm=skip".to_string(),
    })
    .expect_err("CHECK without semantic scope or pattern must fail closed");

    assert_eq!(
        error.message,
        "CHECK intent requires semantic scope, pattern, or both"
    );
}
