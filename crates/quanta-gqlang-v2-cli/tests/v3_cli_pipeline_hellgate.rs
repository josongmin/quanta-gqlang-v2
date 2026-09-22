#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! v3 CLI fail-closed hellgate (ticket 08, D10 / D22).
//!
//! syntax->sema->ir path pinned: source text -> `pipeline::compile_program_structural` ->
//! the FIRST failing stage's typed diagnostic -> the real CLI binary's
//! stage-tagged error output. Each case asserts the ACTUAL stage tag and inner
//! stable code (not just that an error occurred), pinned against the real
//! binary, exit code 1, and silent stdout. An advanced construct never silently
//! downgrades — it rejects with the exact stage code (D10).

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// One fail-closed v3-query case: the exact stage tag and inner code the
/// pipeline must surface from the real binary.
struct RejectCaseV1 {
    label: &'static str,
    source: &'static str,
    stage: &'static str,
    code: &'static str,
}

/// Every advanced reject path rejects through the real binary with the exact
/// stage tag + inner stable code on stderr, exit 1, silent stdout.
#[test]
fn v3_query_fail_closed_with_exact_stage_tagged_code_v1() {
    let cases = [
        RejectCaseV1 {
            label: "parse-unterminated-paren",
            source: "from users | where (a + 1",
            stage: "parse",
            code: "surface.parse.unexpected_eof",
        },
        RejectCaseV1 {
            label: "type-bad-overload",
            source: "from users | project 1 + \"x\" as bad",
            stage: "type",
            code: "types.bad_overload",
        },
        RejectCaseV1 {
            label: "algebra-ill-formed-aggregate",
            source: "from findings f | group by count(f.id)",
            stage: "algebra",
            code: "algebra.ill_formed_aggregate.nested_aggregate",
        },
    ];

    for case in cases {
        let output = run_cli_argv_v1(&["v3-query", case.source]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "case `{}` must exit 1, got {:?}",
            case.label,
            output.status.code()
        );
        assert!(
            cli_stdout_v1(&output).is_empty(),
            "case `{}` must be silent on stdout",
            case.label
        );

        let stderr = cli_stderr_v1(&output);
        assert!(
            stderr.contains(&format!("pipeline.error.stage={}", case.stage)),
            "case `{}` missing stage tag `{}`:\n{stderr}",
            case.label,
            case.stage
        );
        assert!(
            stderr.contains(&format!("pipeline.error.code={}", case.code)),
            "case `{}` missing inner code `{}`:\n{stderr}",
            case.label,
            case.code
        );
        // The stage-prefixed Display line carries the inner code verbatim too.
        assert!(
            stderr.contains(&format!("[{}] {}", case.stage, case.code)),
            "case `{}` missing stage-prefixed message `[{}] {}`:\n{stderr}",
            case.label,
            case.stage,
            case.code
        );
    }
}

/// `v3-fmt` on a non-parsing input fails closed with the exact surface parse
/// code and exact span — never a partial format, exit 1, silent stdout.
#[test]
fn v3_fmt_fail_closed_with_exact_parse_code_v1() {
    let output = run_cli_argv_v1(&["v3-fmt", "from users | where (a + 1"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "v3-fmt parse fault must exit 1"
    );
    assert!(
        cli_stdout_v1(&output).is_empty(),
        "v3-fmt parse fault must be silent on stdout"
    );
    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("v3fmt.error.code=surface.parse.unexpected_eof"),
        "v3-fmt parse fault missing exact parse code:\n{stderr}"
    );
    assert!(
        stderr.contains("v3fmt.error.span=25..25"),
        "v3-fmt parse fault missing exact span:\n{stderr}"
    );
}
