#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "conformance self-validation intentionally panics with rich context on any registry/pipeline drift"
)]

//! Self-validation hellgate for the conformance registry.
//!
//! This is the rail that keeps the registry from rotting. It drives every one of
//! the 17 specs through the REAL `parse → sema → lower` pipeline and proves:
//!   1. the array covers each family exactly once (no drop, no duplicate);
//!   2. every shipped example parses, analyzes, lowers, and lands on the exact
//!      `ir_root_kind` the spec declares (the 17→8 mapping is real, not asserted);
//!   3. every canonical negative fails closed at the parser stage with its
//!      declared diagnostic code AND a distinctive message fragment (the fragment
//!      catches a same-code-wrong-reason regression the bare code cannot).

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use quanta_gqlang_v2_conformance::{
    root_surface_specs_v1, spec_for_v1, RootSurfaceSpecV1, ROOT_FAMILIES_V1,
};
use quanta_gqlang_v2_ir::frontend::{analyze_document_v1, parse_document_v1};
use quanta_gqlang_v2_ir::lower_document_to_ir_v1;

fn repo_relative_v1(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

#[test]
fn registry_covers_each_family_exactly_once_v1() {
    assert_eq!(ROOT_FAMILIES_V1.len(), 17, "expected 17 root families");

    let unique: BTreeSet<_> = ROOT_FAMILIES_V1
        .iter()
        .map(|family| format!("{family:?}"))
        .collect();
    assert_eq!(
        unique.len(),
        ROOT_FAMILIES_V1.len(),
        "ROOT_FAMILIES_V1 has a duplicate variant"
    );

    for &family in ROOT_FAMILIES_V1 {
        assert_eq!(
            spec_for_v1(family).family,
            family,
            "spec_for_v1 must echo the requested family"
        );
    }
}

#[test]
fn every_example_lowers_to_declared_ir_root_kind_v1() {
    for spec in root_surface_specs_v1() {
        let source = read_example_v1(&spec);

        let doc = parse_document_v1(&source)
            .unwrap_or_else(|page| panic!("{}: example must parse: {page:?}", spec.example_path));
        let typed = analyze_document_v1(&doc)
            .unwrap_or_else(|page| panic!("{}: example must analyze: {page:?}", spec.example_path));
        let ir_roots = lower_document_to_ir_v1(&typed)
            .unwrap_or_else(|err| panic!("{}: example must lower: {err:?}", spec.example_path));
        assert_eq!(ir_roots.len(), 1, "{}: one root", spec.example_path);
        assert_eq!(
            ir_roots[0].plan_root_kind_v1(),
            spec.ir_root_kind,
            "{}: lowered IR root kind drifted from the registry",
            spec.example_path
        );
    }
}

#[test]
fn every_negative_fails_closed_with_declared_code_and_fragment_v1() {
    // Every canonical negative is a parser-stage rejection (the canonical parser
    // is the single source of diagnostics; a few of its codes merely DISPLAY as
    // `sema`). So each must fail closed at `parse_document_v1` carrying the
    // declared code and a distinctive message fragment.
    for spec in root_surface_specs_v1() {
        let neg = spec.negative;
        let page = match parse_document_v1(neg.source) {
            Ok(_) => panic!(
                "{:?} negative `{}` must fail closed at parse, but parsed",
                spec.family, neg.source
            ),
            Err(page) => page,
        };
        let diag = page.diagnostics.first().expect("a syntax diagnostic");
        assert_eq!(
            diag.code, neg.code,
            "{:?} negative `{}`: diagnostic code drifted",
            spec.family, neg.source
        );
        assert!(
            diag.message.contains(neg.message_fragment),
            "{:?} negative `{}`: message `{}` missing fragment `{}`",
            spec.family,
            neg.source,
            diag.message,
            neg.message_fragment
        );
    }
}

#[test]
fn keyword_leads_every_example_and_negative_v1() {
    // Pin that `keyword` is the real leading token of both the shipped example
    // and the canonical negative, so a wrong keyword (e.g. a typo) cannot sit
    // unvalidated in the registry.
    for spec in root_surface_specs_v1() {
        let example = read_example_v1(&spec);
        assert!(
            example
                .trim_start()
                .to_uppercase()
                .starts_with(spec.keyword),
            "{:?}: example `{}` does not lead with keyword `{}`",
            spec.family,
            example.trim(),
            spec.keyword
        );
        assert!(
            spec.negative
                .source
                .trim_start()
                .to_uppercase()
                .starts_with(spec.keyword),
            "{:?}: negative `{}` does not lead with keyword `{}`",
            spec.family,
            spec.negative.source,
            spec.keyword
        );
    }
}

fn read_example_v1(spec: &RootSurfaceSpecV1) -> String {
    fs::read_to_string(repo_relative_v1(spec.example_path))
        .unwrap_or_else(|err| panic!("{}: read example: {err}", spec.example_path))
}
