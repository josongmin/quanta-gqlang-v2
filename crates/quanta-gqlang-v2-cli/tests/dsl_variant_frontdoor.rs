#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: DSL variant coverage promoted to the shipped CLI binary.
//!
//! The library matrices (`select_filter_matrix.rs`,
//! `interactive_variant_matrix.rs`) pin every SELECT operator / literal,
//! EXPORT kind, SAMPLE mode, and COMPARE metric at the parser. This rail proves
//! the headline variants survive the *full* front-door stack —
//! parse -> sema -> normalized IR -> render — through the real
//! `quanta-gqlang-v2-cli` binary, and that the `LIMIT` non-negative guard fails
//! closed there with the correct exit code and channel.
//!
//! Frontend-only: `query` proves the variant decodes through the typed/IR stack,
//! never that a result is materialized; `fmt` proves canonical round-trip.

mod support;

use std::path::Path;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

/// Run `<command> --file <path>`, assert exit 0, and return stdout.
fn cli_file_ok_v1(command: &str, path: &Path) -> String {
    let path_str = path.to_str().expect("temp path is valid UTF-8");
    let output = run_cli_argv_v1(&[command, "--file", path_str]);
    assert!(
        output.status.success(),
        "`{command} --file {path_str}` failed (status={:?})\nstderr={}",
        output.status.code(),
        cli_stderr_v1(&output)
    );
    cli_stdout_v1(&output)
}

/// SELECT comparison operators: every operator must round-trip through the real
/// `fmt` binary as a canonical fixed point and decode through the full `query`
/// stack.
#[test]
fn select_comparison_operators_round_trip_through_cli_v1() {
    let cases = [
        ("eq", "score = 5"),
        ("neq", "score != 5"),
        ("lt", "score < 5"),
        ("lte", "score <= 5"),
        ("gt", "score > 5"),
        ("gte", "score >= 5"),
    ];

    for (index, (label, predicate)) in cases.iter().enumerate() {
        let source = format!("SELECT id FROM catalog.users WHERE {predicate} LIMIT 3");
        let path = write_target_tmp_input_v1(&format!("variant_select_op_{index}.qglang"), &source);

        // Full stack accepts the operator.
        let query_stdout = cli_file_ok_v1("query", &path);
        assert!(
            query_stdout.contains("query.root[0].syntax.kind=select"),
            "case `{label}`: query did not classify SELECT:\n{query_stdout}"
        );

        // `fmt` preserves the operator token and is a fixed point.
        let fmt1 = cli_file_ok_v1("fmt", &path);
        assert!(
            fmt1.contains(predicate),
            "case `{label}`: fmt dropped `{predicate}`:\n{fmt1}"
        );
        let path2 =
            write_target_tmp_input_v1(&format!("variant_select_op_{index}_pass2.qglang"), &fmt1);
        let fmt2 = cli_file_ok_v1("fmt", &path2);
        assert_eq!(
            fmt1, fmt2,
            "case `{label}`: fmt is not idempotent for `{predicate}`"
        );
    }
}

/// Negative integer literal: the `Minus` branch survives the full binary stack
/// and round-trips canonically.
#[test]
fn select_negative_integer_literal_round_trips_through_cli_v1() {
    let source = "SELECT id FROM catalog.users WHERE delta = -8 LIMIT 4";
    let path = write_target_tmp_input_v1("variant_select_negative_int.qglang", source);

    let query_stdout = cli_file_ok_v1("query", &path);
    assert!(
        query_stdout.contains("query.root[0].syntax.kind=select"),
        "negative integer literal did not classify SELECT:\n{query_stdout}"
    );

    let fmt_stdout = cli_file_ok_v1("fmt", &path);
    assert!(
        fmt_stdout.contains("delta = -8"),
        "fmt dropped the negative integer literal:\n{fmt_stdout}"
    );
}

/// Negative `LIMIT`: the `u64` bound guard must fail closed at the binary with
/// exit 1, the command-error channel on stderr, the parser code, and no stdout.
#[test]
fn select_negative_limit_fails_closed_on_cli_v1() {
    let source = "SELECT id FROM catalog.users LIMIT -5";
    let path = write_target_tmp_input_v1("variant_select_negative_limit.qglang", source);
    let path_str = path.to_str().expect("temp path is valid UTF-8");

    let output = run_cli_argv_v1(&["parse", "--file", path_str]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "negative LIMIT must exit 1, got {:?}; stderr=\n{}",
        output.status.code(),
        cli_stderr_v1(&output)
    );

    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("cli.error.class=command"),
        "negative LIMIT missing command-error class on stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("invalid_limit_value"),
        "negative LIMIT missing parser code on stderr:\n{stderr}"
    );

    let stdout = cli_stdout_v1(&output);
    assert!(
        stdout.is_empty(),
        "negative LIMIT must not write stdout, got:\n{stdout}"
    );
}

/// EXPORT kinds (incl. the previously binary-untested json/csv/markdown) survive
/// the full stack and round-trip canonically.
#[test]
fn export_kinds_round_trip_through_cli_v1() {
    let cases = [
        ("json", "kind=json"),
        ("csv", "kind=csv"),
        ("sarif", "kind=sarif"),
        ("markdown", "kind=markdown"),
        ("graphml", "kind=graph-ml"),
    ];

    for (index, (written, canonical_fragment)) in cases.iter().enumerate() {
        let source = format!("EXPORT source=result::latest ;; kind={written}");
        let path = write_target_tmp_input_v1(&format!("variant_export_{index}.qglang"), &source);

        let query_stdout = cli_file_ok_v1("query", &path);
        assert!(
            query_stdout.contains("query.root[0].syntax.kind=export"),
            "kind `{written}`: query did not classify EXPORT:\n{query_stdout}"
        );

        let fmt_stdout = cli_file_ok_v1("fmt", &path);
        assert!(
            fmt_stdout.contains(canonical_fragment),
            "kind `{written}`: fmt missing `{canonical_fragment}`:\n{fmt_stdout}"
        );
    }
}

/// SAMPLE retrieval modes (incl. the previously untested `lexical`) survive the
/// full stack and round-trip canonically.
#[test]
fn sample_modes_round_trip_through_cli_v1() {
    let cases = [
        ("lexical", "mode=lexical"),
        ("semantic", "mode=semantic"),
        ("hybrid", "mode=hybrid"),
    ];

    for (index, (written, canonical_fragment)) in cases.iter().enumerate() {
        let source = format!("SAMPLE query=callgraph entrypoints ;; mode={written} ;; limit=5");
        let path = write_target_tmp_input_v1(&format!("variant_sample_{index}.qglang"), &source);

        let query_stdout = cli_file_ok_v1("query", &path);
        assert!(
            query_stdout.contains("query.root[0].syntax.kind=sample"),
            "mode `{written}`: query did not classify SAMPLE:\n{query_stdout}"
        );

        let fmt_stdout = cli_file_ok_v1("fmt", &path);
        assert!(
            fmt_stdout.contains(canonical_fragment),
            "mode `{written}`: fmt missing `{canonical_fragment}`:\n{fmt_stdout}"
        );
    }
}
