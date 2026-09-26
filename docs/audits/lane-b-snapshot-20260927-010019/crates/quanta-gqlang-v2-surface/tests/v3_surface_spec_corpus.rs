#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "the specification corpus rejects incomplete or drifting fixtures"
)]

//! File-backed, test-owned frontend language examples with independent expected
//! results. A source and its expectation must appear together. This is a
//! deliberately small specification corpus, not an assertion that every
//! shipped example has closed-authority semantics.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1};
use quanta_gqlang_v2_surface::{format_document, parse_document, SurfaceItemV1};

// Explicit registration prevents a source and expectation from disappearing
// together while the pairing check continues to pass.
const SPEC_CASES: [&str; 5] = [
    "declaration-precedence",
    "find-missing-arrow",
    "leading-separator",
    "presentation-show",
    "query-projection",
];

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("spec_cases")
}

fn kind(item: &ProgramItemHirV1) -> &'static str {
    match item {
        ProgramItemHirV1::Query(_) => "query",
        ProgramItemHirV1::Decl(_) => "decl",
        ProgramItemHirV1::Rule(_) => "rule",
        ProgramItemHirV1::GraphQuery(_) => "graph_query",
        ProgramItemHirV1::Legality(_) => "legality",
        ProgramItemHirV1::Presentation(_) => "presentation",
        ProgramItemHirV1::CatalogDecl(_) => "catalog_decl",
        ProgramItemHirV1::Composition(_) => "composition",
        ProgramItemHirV1::Meta(_) => "meta",
    }
}

fn case_names(root: &Path, extension: &str) -> BTreeSet<String> {
    std::fs::read_dir(root)
        .expect("spec corpus directory")
        .map(|entry| entry.expect("spec corpus entry").path())
        .filter(|path| path.extension().is_some_and(|actual| actual == extension))
        .map(|path| {
            path.file_stem()
                .expect("spec case name")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn every_spec_source_has_one_independent_expectation() {
    let root = corpus_root();
    let sources = case_names(&root, "qglang");
    let expectations = case_names(&root, "expect");
    let registered: BTreeSet<String> = SPEC_CASES.iter().map(|name| (*name).to_owned()).collect();
    assert_eq!(sources, registered, "spec corpus registration drift");
    assert_eq!(sources, expectations, "orphaned source or expectation");

    let mut accepted = 0;
    let mut rejected = 0;
    for name in sources {
        let source = std::fs::read_to_string(root.join(format!("{name}.qglang")))
            .unwrap_or_else(|error| panic!("{name}: read source: {error}"));
        let expected = std::fs::read_to_string(root.join(format!("{name}.expect")))
            .unwrap_or_else(|error| panic!("{name}: read expectation: {error}"));
        let (header, body) = expected
            .split_once('\n')
            .unwrap_or_else(|| panic!("{name}: expectation needs a header and newline"));
        let parts: Vec<&str> = header.split('|').collect();
        match parts.as_slice() {
            ["accept", expected_kind] => {
                accepted += 1;
                let parsed = parse_document(&source, &HirBuilderV1::new())
                    .unwrap_or_else(|error| panic!("{name}: expected accept: {error:?}"));
                assert_eq!(parsed.items.len(), 1, "{name}: expected one root");
                let items: Vec<_> = parsed
                    .items
                    .iter()
                    .map(SurfaceItemV1::to_program_item)
                    .collect();
                assert_eq!(
                    kind(items.first().expect("one parsed root")),
                    *expected_kind,
                    "{name}: root family"
                );
                assert_eq!(
                    format_document(&items),
                    body.trim_end_matches('\n'),
                    "{name}: canonical source"
                );
            }
            ["reject", expected_code, expected_span] => {
                rejected += 1;
                assert!(body.trim().is_empty(), "{name}: reject body must be empty");
                let diagnostic = parse_document(&source, &HirBuilderV1::new())
                    .expect_err("spec case must reject");
                let observed_span = format!("{}..{}", diagnostic.span.start, diagnostic.span.end);
                assert_eq!(
                    diagnostic.code.code(),
                    *expected_code,
                    "{name}: diagnostic code"
                );
                assert_eq!(observed_span, *expected_span, "{name}: diagnostic span");
            }
            _ => panic!("{name}: invalid expectation header {header:?}"),
        }
    }
    assert!(
        accepted > 0 && rejected > 0,
        "both acceptance and rejection must execute"
    );
}
