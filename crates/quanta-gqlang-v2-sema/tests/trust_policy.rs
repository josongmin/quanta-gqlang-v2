#![allow(clippy::expect_used, reason = "trust policy tests assert typed reject codes")]
//! F07/E10B hellgate — `REQUIRE CERTIFIED` trust admission. Proof rail: `--test trust_policy`.
//! NOTE: rule #5 (no over-blocking) is load-bearing — a document WITHOUT
//! REQUIRE CERTIFIED must admit sampling/unconfirmed checks freely.
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1};

fn reject_code_v1(input: &str) -> Vec<&'static str> {
    analyze_document_v1(&parse_document_v1(input).expect("parse"))
        .expect_err("must reject")
        .diagnostics.iter().map(|d| d.code).collect()
}
fn admits_v1(input: &str) -> bool {
    analyze_document_v1(&parse_document_v1(input).expect("parse")).is_ok()
}

#[test]
fn require_certified_rejects_sampling_v1() {
    let codes = reject_code_v1(concat!(
        "REQUIRE CERTIFIED\n",
        "SAMPLE query=auth ;; mode=lexical ;; limit=5",
    ));
    assert!(codes.contains(&"require_certified_rejects_sampling"), "got {codes:?}");
}

#[test]
fn require_certified_rejects_unconfirmed_check_v1() {
    let codes = reject_code_v1(concat!(
        "REQUIRE CERTIFIED\n",
        "CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fr|pattern=~|confirm=skip",
    ));
    assert!(codes.contains(&"require_certified_rejects_unconfirmed_check"), "got {codes:?}");
}

#[test]
fn require_certified_rejects_weak_profile_admission_v1() {
    let codes = reject_code_v1(concat!(
        "REQUIRE CERTIFIED\n",
        "PROFILE lax ;; capabilities=graph.read ;; admission=allowlist",
    ));
    assert!(codes.contains(&"require_certified_profile_incompatible"), "got {codes:?}");
}

#[test]
fn require_certified_admits_compliant_document_v1() {
    assert!(admits_v1(concat!(
        "REQUIRE CERTIFIED\n",
        "CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fr|pattern=~|confirm=require\n",
        "PROFILE strict ;; capabilities=graph.read ;; admission=certified-only",
    )), "compliant require-certified document must admit");
}

#[test]
fn no_require_certified_does_not_over_block_v1() {
    // Without REQUIRE CERTIFIED, sampling + unconfirmed check admit freely.
    assert!(admits_v1("SAMPLE query=auth ;; mode=lexical ;; limit=5"), "sampling admits without require-certified");
    assert!(admits_v1("CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fr|pattern=~|confirm=skip"), "unconfirmed check admits without require-certified");
}
