#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing, exercise fail-closed panics, and bind-all match arms that panic on unexpected variants"
)]

//! E03C ticket 02 — `LET` / `WITH` composition typing conformance.
//!
//! Grammar: `[LET <name> = <producer>]+` then `<consumer-body> WITH <binding>`.
//! Pins alias→index resolution, the D3 producer allow-set, basis mapping, the
//! consumer-body rule, and that a producer's own sema error passes through
//! unmasked.

use quanta_gqlang_v2_sema::interactive::composition::{TypedCompositionV1, TypedWorkingSetBasisV1};
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1, TypedRootV1};

fn typed_composition_v1(input: &str) -> TypedCompositionV1 {
    let doc = parse_document_v1(input).expect("parse");
    let typed = analyze_document_v1(&doc).expect("analyze");
    assert_eq!(typed.roots.len(), 1, "expected one typed root: {input}");
    match typed.roots.into_iter().next().expect("root") {
        TypedRootV1::Composition(spec) => spec,
        other => panic!("expected composition, got {other:?}"),
    }
}

fn sema_error_v1(input: &str) -> String {
    let doc = parse_document_v1(input).expect("parse");
    let page = analyze_document_v1(&doc).expect_err("expected sema failure");
    page.diagnostics.into_iter().next().expect("a diagnostic")
}

#[test]
fn with_resolves_to_the_correct_binding_index_v1() {
    let spec = typed_composition_v1(concat!(
        "LET a = SELECT id FROM catalog.users LIMIT 5\n",
        "LET b = SAMPLE query=auth ;; scope=repo::auth ;; mode=semantic ;; limit=3\n",
        "FIND taint api.entry -> db.write WITH b",
    ));
    assert_eq!(spec.bindings.len(), 2);
    assert_eq!(spec.with_index, 1, "WITH b must resolve to binding index 1");
    assert!(matches!(*spec.body, TypedRootV1::Find(_)));
}

#[test]
fn producer_basis_is_mapped_per_d3_v1() {
    let select_basis = typed_composition_v1(concat!(
        "LET a = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH a",
    ));
    assert_eq!(
        select_basis.bindings[0].basis,
        TypedWorkingSetBasisV1::SearchQuery
    );

    let find_basis = typed_composition_v1(concat!(
        "LET a = FIND taint api.entry -> db.write\n",
        "SELECT id FROM catalog.users LIMIT 5 WITH a",
    ));
    assert_eq!(
        find_basis.bindings[0].basis,
        TypedWorkingSetBasisV1::AiDiscovery
    );
}

#[test]
fn undefined_alias_fails_closed_v1() {
    let message = sema_error_v1("FIND taint api.entry -> db.write WITH ghost");
    assert!(
        message.contains("composition_undefined_binding"),
        "{message}"
    );
}

#[test]
fn invalid_producer_is_a_positive_allow_set_v1() {
    for producer in [
        "LET x = SHOW models",
        "LET x = DESCRIBE rule::owasp.sql-injection",
        "LET x = EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm",
    ] {
        let doc = format!("{producer}\nFIND taint api.entry -> db.write WITH x");
        let message = sema_error_v1(&doc);
        assert!(
            message.contains("composition_invalid_producer"),
            "producer `{producer}` should be rejected: {message}"
        );
    }
}

#[test]
fn body_must_be_a_working_set_consumer_v1() {
    // Consumer bodies (analysis/query plane) are accepted.
    for body in [
        "FIND taint api.entry -> db.write",
        "SELECT id FROM catalog.users LIMIT 5",
        "EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm",
    ] {
        let doc = format!("LET x = SELECT id FROM catalog.users LIMIT 5\n{body} WITH x");
        let _ = typed_composition_v1(&doc);
    }
    // Non-consumer bodies cannot scope to a working set -> fail closed.
    for body in [
        "SHOW models",
        "SAMPLE query=a ;; scope=repo::a ;; mode=semantic ;; limit=3",
        "COMPARE baseline=run::a ;; candidate=run::b ;; metric=semantic",
        "DESCRIBE rule::owasp.sql-injection",
    ] {
        let doc = format!("LET x = SELECT id FROM catalog.users LIMIT 5\n{body} WITH x");
        let message = sema_error_v1(&doc);
        assert!(
            message.contains("composition_invalid_body"),
            "non-consumer body `{body}` must be rejected, got: {message}"
        );
    }
}

#[test]
fn producer_own_sema_error_is_not_masked_v1() {
    let message = sema_error_v1(concat!(
        "LET x = FIND path api.entry -> db.write | depth=0\n",
        "FIND taint api.entry -> db.write WITH x",
    ));
    assert!(message.contains("greater than zero"), "{message}");
    assert!(
        !message.contains("composition_invalid_producer"),
        "inner producer error wrongly rewrapped: {message}"
    );
}
