#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
//! F08/E12A hellgate — the CLI `zero-query` verb. This rail goes through the CLI
//! seam ONLY (`commands::zero_query::run` + the shipped binary); it no longer
//! imports the LSP handler, so a green here actually proves the CLI surface
//! exists (the prior masking the audit caught). The load-bearing guard is #2:
//! every suggested query must reparse through the CLI's own parser, so a starter
//! scaffold can never ship a query the grammar would reject.

mod support;

use quanta_gqlang_v2_cli::commands::{parse, zero_query};
use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// Pull the `...suggestion[N].query=<QUERY>` payloads out of the rendered report.
/// The query itself can contain `=` (e.g. `WHERE active = true`), so split once
/// on the `.query=` marker rather than the first `=`.
fn suggested_queries_v1(rendered: &str) -> Vec<String> {
    rendered
        .lines()
        .filter(|line| line.contains(".suggestion[") && line.contains(".query="))
        .filter_map(|line| {
            line.split_once(".query=")
                .map(|(_, query)| query.to_string())
        })
        .collect()
}

#[test]
fn zero_query_verb_renders_groups_and_suggestions_v1() {
    let rendered = zero_query::run().expect("zero-query verb");
    let rerun = zero_query::run().expect("zero-query rerun");
    assert_eq!(rendered, rerun, "zero-query must be deterministic");

    assert!(rendered.contains("zero_query.mode=read-only"));
    assert!(rendered.contains("zero_query.group[0].id=catalog"));
    assert!(
        !suggested_queries_v1(&rendered).is_empty(),
        "zero-query must surface at least one suggestion:\n{rendered}"
    );
    // Read-only: no mutating verbs, no fabricated runtime counts.
    assert!(!rendered.contains("result_count"));
    assert!(!rendered.contains("code_action"));
}

#[test]
fn every_suggested_query_reparses_through_the_cli_parser_v1() {
    let rendered = zero_query::run().expect("zero-query verb");
    let queries = suggested_queries_v1(&rendered);
    assert!(!queries.is_empty(), "no suggestions extracted");

    for query in &queries {
        let report = parse::run(query)
            .unwrap_or_else(|err| panic!("scaffolded query did not reparse: `{query}` -> {err}"));
        assert_eq!(
            report.root_count, 1,
            "scaffolded query must parse to exactly one root: `{query}`"
        );
    }
}

#[test]
fn zero_query_binary_is_nullary_and_read_only_v1() {
    // No args → exit 0 with the rendered scaffold on stdout.
    let ok = run_cli_argv_v1(&["zero-query"]);
    assert!(ok.status.success(), "zero-query should exit 0");
    assert!(cli_stdout_v1(&ok).contains("zero_query.group[0].id=catalog"));

    // Stray argument → usage fault (exit 2), nothing on stdout.
    let bad = run_cli_argv_v1(&["zero-query", "extra"]);
    assert_eq!(
        bad.status.code(),
        Some(2),
        "stray arg must be a usage fault"
    );
    assert!(cli_stdout_v1(&bad).is_empty());
    assert!(cli_stderr_v1(&bad).contains("zero-query takes no arguments"));
}
