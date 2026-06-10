#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing, exercise fail-closed panics, and bind-all match arms that panic on unexpected variants"
)]

//! E03C ticket 01 — `LET` / `WITH` composition syntax conformance.
//!
//! Grammar: `[LET <name> = <producer>]+` then a body root that names the working
//! set it consumes via a trailing `WITH <binding>` clause (SQL-CTE style — the
//! binding is referenced once, at the point of consumption). Deterministic
//! document grouping + canonical formatting. Every fail-closed branch is pinned
//! to its exact diagnostic code (no heuristic repair); the happy path pins the
//! grouped AST; the formatter is proven idempotent; a no-composition document
//! parses unchanged.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::composition::CompositionRootAstV1;
use quanta_gqlang_v2_syntax::{format_document_v1, parse_document_v1, SyntaxDiagnosticV1};

fn parse_error_v1(input: &str) -> SyntaxDiagnosticV1 {
    let page = parse_document_v1(input).expect_err("expected parse failure");
    page.diagnostics
        .into_iter()
        .next()
        .expect("a diagnostic must be present")
}

fn sole_root_v1(input: &str) -> AstRootV1 {
    let doc = parse_document_v1(input).expect("parse");
    assert_eq!(doc.roots.len(), 1, "expected exactly one root: {input}");
    doc.roots.into_iter().next().expect("root")
}

fn composition_v1(input: &str) -> CompositionRootAstV1 {
    match sole_root_v1(input) {
        AstRootV1::Composition(spec) => spec,
        other => panic!("expected composition root, got {other:?}"),
    }
}

// ---------------------------------------------------------------- happy paths
#[test]
fn single_binding_body_with_ref_groups_into_one_composition_v1() {
    let spec = composition_v1(concat!(
        "LET suspects = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH suspects",
    ));
    assert_eq!(spec.lets.len(), 1);
    assert_eq!(spec.lets[0].name, "suspects");
    assert!(matches!(*spec.lets[0].producer, AstRootV1::Select(_)));
    assert_eq!(spec.with_name, "suspects");
    assert!(matches!(*spec.body, AstRootV1::Find(_)));
}

#[test]
fn multiple_lets_accumulate_in_declaration_order_v1() {
    let spec = composition_v1(concat!(
        "LET a = SELECT id FROM catalog.users LIMIT 5\n",
        "LET b = SAMPLE query=auth ;; scope=repo::auth ;; mode=semantic ;; limit=3\n",
        "FIND taint api.entry -> db.write WITH b",
    ));
    assert_eq!(spec.lets.len(), 2);
    assert_eq!(spec.lets[0].name, "a");
    assert_eq!(spec.lets[1].name, "b");
    assert_eq!(spec.with_name, "b");
}

#[test]
fn let_with_no_spaces_around_equals_parses_v1() {
    let spec = composition_v1(concat!(
        "LET x=SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH x",
    ));
    assert_eq!(spec.lets[0].name, "x");
    assert_eq!(spec.with_name, "x");
}

#[test]
fn let_and_with_are_case_insensitive_v1() {
    let spec = composition_v1(concat!(
        "let suspects = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write with suspects",
    ));
    assert_eq!(spec.with_name, "suspects");
}

#[test]
fn with_ref_inside_a_quoted_value_is_not_a_scope_clause_v1() {
    // A standalone SELECT whose WHERE value contains "WITH" is NOT a composition
    // body — the suffix after the last ` WITH ` is not a bare identifier.
    let doc = parse_document_v1("SELECT id FROM catalog.users WHERE name = 'a WITH b'")
        .expect("quoted WITH must parse as a plain SELECT");
    assert_eq!(doc.roots.len(), 1);
    assert!(matches!(doc.roots[0], AstRootV1::Select(_)));
}

// ---------------------------------------------------------------- fail-closed codes
#[test]
fn structural_faults_each_map_to_a_distinct_code_v1() {
    let cases: &[(&str, &str)] = &[
        // missing `=`: the `=` inside the SAMPLE payload must not be mistaken
        // for the binding separator.
        (
            "LET x SAMPLE query=a ;; scope=repo::a ;; mode=semantic ;; limit=5\nFIND taint api.entry -> db.write WITH x",
            "composition_let_missing_equals",
        ),
        (
            "LET 9bad = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x",
            "composition_invalid_binding_name",
        ),
        (
            "LET CAP = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x",
            "composition_invalid_binding_name",
        ),
        (
            "LET x = LET y = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x",
            "composition_nested_binding",
        ),
    ];
    for (doc, code) in cases {
        assert_eq!(parse_error_v1(doc).code, *code, "input: {doc}");
    }
}

#[test]
fn standalone_with_line_must_trail_a_body_v1() {
    // The old `WITH x` on its own line is rejected: WITH is now a trailing
    // consumption clause on the body.
    let doc = concat!(
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "WITH x\n",
        "FIND taint api.entry -> db.write",
    );
    assert_eq!(parse_error_v1(doc).code, "composition_with_must_trail_body");
}

#[test]
fn dangling_let_without_a_consuming_body_fails_closed_v1() {
    // No body references the binding.
    assert_eq!(
        parse_error_v1("LET x = SELECT id FROM catalog.users LIMIT 5").code,
        "composition_unconsumed_let"
    );
    // A plain root with no WITH ref cannot consume the binding.
    let orphan = "LET x = SELECT id FROM catalog.users LIMIT 5\nSHOW models";
    assert_eq!(parse_error_v1(orphan).code, "composition_unconsumed_let");
}

#[test]
fn duplicate_binding_name_fails_closed_v1() {
    let doc = concat!(
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "LET x = SELECT id FROM catalog.users LIMIT 6\n",
        "FIND taint api.entry -> db.write WITH x",
    );
    assert_eq!(parse_error_v1(doc).code, "composition_duplicate_binding");
}

#[test]
fn producer_parse_error_is_not_masked_by_composition_v1() {
    // A syntactically invalid producer (bare SELECT) surfaces the SELECT's own
    // exact parse diagnostic, never a composition_* code.
    let doc = "LET x = SELECT\nFIND taint api.entry -> db.write WITH x";
    assert_eq!(parse_error_v1(doc).code, "expected_projection");
}

// ---------------------------------------------------------------- formatter
#[test]
fn composition_formats_canonically_and_idempotently_v1() {
    let input = concat!(
        "LET suspects = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH suspects",
    );
    let doc = parse_document_v1(input).expect("parse");
    let formatted = format_document_v1(&doc);
    assert!(formatted.contains("LET suspects = SELECT id FROM catalog.users LIMIT 5"));
    assert!(formatted.contains("FIND taint api.entry -> db.write WITH suspects"));
    let reparsed = parse_document_v1(&formatted).expect("reparse");
    assert_eq!(
        formatted,
        format_document_v1(&reparsed),
        "fmt must be idempotent"
    );
}

// ---------------------------------------------------------------- backward compat
#[test]
fn plain_documents_are_unaffected_by_grouping_v1() {
    let doc = parse_document_v1("SHOW models\nSHOW views").expect("parse");
    assert_eq!(doc.roots.len(), 2);
    assert!(matches!(doc.roots[0], AstRootV1::Show(_)));
    assert!(matches!(doc.roots[1], AstRootV1::Show(_)));
}

#[test]
fn plain_root_above_composition_does_not_group_backward_v1() {
    let doc = parse_document_v1(concat!(
        "SHOW models\n",
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH x",
    ))
    .expect("parse");
    assert_eq!(
        doc.roots.len(),
        2,
        "show stays standalone, composition is second"
    );
    assert!(matches!(doc.roots[0], AstRootV1::Show(_)));
    assert!(matches!(doc.roots[1], AstRootV1::Composition(_)));
}
