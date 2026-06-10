#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "acceptance-test assertions intentionally use expect, exercise fail-closed parse rejects, and bind-all match arms that panic on unexpected variants"
)]

//! F09 hellgate (jun-10-gqlnag-v2-followup) — `CHECK` root acceptance/reject
//! contract (E02C `check_root_acceptance_*`).
//!
//! NOTE TO OTHER AGENTS: this file pins the E02C `CHECK` grammar contract. Two
//! hard rules must never regress:
//!   1. IMPLICIT CLASS INFERENCE IS REJECTED. A `CHECK <rule_id> <bare-scope>`
//!      with no explicit `check:v1|class=...` payload must fail closed — it must
//!      NOT be silently stamped `class="legacy"`. (The old `decode_legacy`
//!      fallback was the audited E02C line-48 violation; it is gone.)
//!   2. The typed shape is `{ rule_id, class, scope, pattern, confirm, emit }`.
//!      `emit` is a typed read-only emission target that defaults to `Findings`
//!      when the clause is absent, and a direct command-bridge `apply` emission
//!      is rejected (E02C line 48 "direct command-bridge apply payloads").
//!
//! Do not relax these into positive-only checks.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::parse_document_v1;
use quanta_gqlang_v2_syntax::roots::check::{CheckConfirmIntentV1, CheckEmitIntentV1};

fn check_intent_v1(input: &str) -> quanta_gqlang_v2_syntax::roots::check::CheckRootIntentV1 {
    let doc = parse_document_v1(input).expect("parse");
    assert_eq!(doc.roots.len(), 1, "expected one root: {input}");
    match doc.roots.into_iter().next().expect("root") {
        AstRootV1::Check(spec) => spec.intent().expect("decode intent"),
        other => panic!("expected CHECK root, got {other:?}"),
    }
}

fn reject_code_and_message_v1(input: &str) -> (Vec<&'static str>, String) {
    let page = parse_document_v1(input).expect_err("expected CHECK parse rejection");
    let codes = page.diagnostics.iter().map(|d| d.code).collect();
    let message = page
        .diagnostics
        .iter()
        .map(|d| d.message.clone())
        .collect::<Vec<_>>()
        .join(" | ");
    (codes, message)
}

#[test]
fn check_root_acceptance_explicit_class_and_emit_v1() {
    let intent = check_intent_v1(
        "CHECK rule.security/no-shell check:v1|class=security|scope=repo%3A%2F%2Fruntime|pattern=call%20%3D%20%22std%3A%3AProcess%22|confirm=require|emit=findings",
    );
    assert_eq!(intent.class, "security");
    assert_eq!(intent.confirm, CheckConfirmIntentV1::Require);
    assert_eq!(intent.emit, CheckEmitIntentV1::Findings);
    assert_eq!(intent.scope.as_deref(), Some("repo://runtime"));
}

#[test]
fn check_root_acceptance_emit_defaults_to_findings_when_absent_v1() {
    // Backward-compatible: existing `check:v1|...` payloads that pre-date `emit`
    // still parse, defaulting to the read-only Findings emission.
    let intent = check_intent_v1(
        "CHECK rule.audit/stale check:v1|class=audit|scope=repo%3A%2F%2Ffrontend|pattern=~|confirm=skip",
    );
    assert_eq!(intent.class, "audit");
    assert_eq!(intent.emit, CheckEmitIntentV1::Findings);
}

#[test]
fn check_root_acceptance_emit_sarif_v1() {
    let intent = check_intent_v1(
        "CHECK rule.audit/stale check:v1|class=audit|scope=repo%3A%2F%2Ffrontend|pattern=~|confirm=skip|emit=sarif",
    );
    assert_eq!(intent.emit, CheckEmitIntentV1::Sarif);
}

#[test]
fn check_root_rejects_implicit_class_inference_v1() {
    // Bare scope, no `check:v1|class=...` — must fail closed, NOT become legacy.
    let (codes, message) = reject_code_and_message_v1("CHECK rule.audit/stale repo://frontend");
    assert!(
        codes.contains(&"invalid_check_payload"),
        "expected invalid_check_payload, got {codes:?}"
    );
    assert!(
        message.contains("implicit class"),
        "reject message must name implicit class inference, got: {message}"
    );
}

#[test]
fn check_root_rejects_direct_command_bridge_apply_emit_v1() {
    let (codes, message) = reject_code_and_message_v1(
        "CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fruntime|pattern=~|confirm=skip|emit=apply",
    );
    assert!(
        codes.contains(&"invalid_check_payload"),
        "expected invalid_check_payload, got {codes:?}"
    );
    assert!(
        message.contains("command-bridge") || message.contains("apply"),
        "reject message must name the command-bridge apply reject, got: {message}"
    );
}
