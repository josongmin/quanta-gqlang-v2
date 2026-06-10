#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! No-runtime-leakage cross-command binary sweep.
//!
//! This is the marquee "No runtime leakage" hellgate for the read-only CLI
//! shell. The shell parses, type-checks, lowers, formats and reports on
//! queries but has *no execution backend*: it must never emit runtime-truth
//! (real results, materialized rows, host routing, applied edits, etc.).
//!
//! The sweep drives every shipped example through every universally-applicable
//! command (`parse`, `sema`, `query`, `fmt`, `doctor`, `headless`) plus
//! `explain` on the FIND/EXPLAIN-rooted examples, then proves each real binary
//! output is honest-frontend-only via three layers:
//!
//! 1. a denylist scan that no runtime-truth token leaks into stdout,
//! 2. positive honest-no-result invariants on `headless` and `doctor`, and
//! 3. a strong degraded-reason authority invariant — every degraded reason
//!    value emitted is a single honest-absence enum that ends with
//!    `-unavailable`, never a positive runtime claim.

mod support;

use support::{
    cli_stdout_v1, example_path_v1, run_cli_argv_v1, write_target_tmp_input_v1,
    SHIPPED_EXAMPLE_DOC_PATHS_V1,
};

/// Commands that apply to *every* shipped example regardless of root family.
const UNIVERSAL_COMMANDS_V1: &[&str] = &["parse", "sema", "query", "fmt", "doctor", "headless"];

/// Example root families that the `explain` front door accepts. `explain`
/// rejects every other root with a command error, so it is swept only here.
const EXPLAIN_EXAMPLE_PREFIXES_V1: &[&str] = &["examples/find/", "examples/explain/"];

/// Runtime-truth tokens that imply real execution / materialization / routing /
/// host DTO / applied edit. None of these may appear in any read-only output.
///
/// These are compound tokens deliberately chosen so they do not collide with
/// the *honest* markers the shell legitimately emits (e.g. `no_result_reason`,
/// `why_no_result`, `root_count`, `step_count`, `degraded_count`,
/// `diagnostic_count`, and the `*-unavailable` degraded-reason enum values).
const FORBIDDEN_RUNTIME_TOKENS_V1: &[&str] = &[
    "result_count",
    "row_count",
    "rows_returned",
    "rows=",
    "match_count",
    "matches_found",
    "results_found",
    "num_results",
    "execution_time",
    "elapsed_ms",
    "executed_at",
    "ran_at",
    "provider_response",
    "provider_endpoint",
    "routed_to",
    "host_dto",
    "host_response",
    "backend_result",
    "edit_applied",
    "applied_edit",
    "workspace_edit_applied",
    "materialized_result",
    "result_materialized",
    "runtime_truth",
    "confirmed_true",
    "verdict=",
    "actual_result",
];

/// Honest absence authority suffix. Every degraded-reason value emitted by any
/// front door must end with this; it is the single honest-absence enum, never a
/// positive runtime claim.
const DEGRADED_REASON_SUFFIX_V1: &str = "-unavailable";

/// Run `<command> --file <example>` through the real binary, assert exit 0, and
/// return stdout. The read-only shell always succeeds on shipped examples (its
/// honest "no result" is exit 0, not a failure).
fn run_command_file_stdout_v1(command: &str, example_relative: &str) -> String {
    let path = example_path_v1(example_relative);
    let path_str = path.to_str().expect("example path is valid UTF-8");
    let output = run_cli_argv_v1(&[command, "--file", path_str]);
    assert!(
        output.status.success(),
        "command `{command}` unexpectedly failed for `{example_relative}` \
         (status={:?})\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    cli_stdout_v1(&output)
}

/// Assert no runtime-truth token leaks into `stdout`. On failure the message
/// names the offending (command, example, token).
fn assert_no_forbidden_tokens_v1(command: &str, example_relative: &str, stdout: &str) {
    for token in FORBIDDEN_RUNTIME_TOKENS_V1 {
        assert!(
            !stdout.contains(token),
            "runtime-leakage detected: command=`{command}` example=`{example_relative}` \
             leaked forbidden token=`{token}`\n--- stdout ---\n{stdout}"
        );
    }
}

/// Extract every degraded-reason *value* emitted by a front door, keyed off the
/// three honest line shapes:
/// - `headless.degraded[<i>]=<reason>`
/// - `doctor.degraded_reason[<i>]=<reason>`
/// - `explain.degraded[<i>].code=<reason>`
fn extract_degraded_reasons_v1(stdout: &str) -> Vec<String> {
    let mut reasons = Vec::new();
    for line in stdout.lines() {
        if let Some(reason) = degraded_reason_value_v1(line) {
            reasons.push(reason.to_string());
        }
    }
    reasons
}

/// Pull the reason value out of a single line if it is one of the three honest
/// degraded-reason line shapes; otherwise `None`.
fn degraded_reason_value_v1(line: &str) -> Option<&str> {
    let value = |key_open: &str| -> Option<&str> {
        // Lines look like `headless.degraded[0]=<reason>`; we want the suffix
        // after the closing-bracket-and-`=` of the index, matching whatever
        // numeric index is present without hardcoding it.
        if !line.starts_with(key_open) {
            return None;
        }
        let after_open = &line[key_open.len()..];
        let close = after_open.find(']')?;
        let rest = &after_open[close + 1..];
        rest.strip_prefix("=")
    };

    if let Some(reason) = value("headless.degraded[") {
        return Some(reason);
    }
    if let Some(reason) = value("doctor.degraded_reason[") {
        return Some(reason);
    }
    // explain shape: `explain.degraded[<i>].code=<reason>`
    if let Some(stripped) = line.strip_prefix("explain.degraded[") {
        let close = stripped.find(']')?;
        let rest = &stripped[close + 1..];
        return rest.strip_prefix(".code=");
    }
    None
}

/// The full set of (command, example) pairs swept by this test: every universal
/// command on every shipped example, plus `explain` on FIND/EXPLAIN examples.
fn sweep_matrix_v1() -> Vec<(&'static str, &'static str)> {
    let mut matrix = Vec::new();
    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        for command in UNIVERSAL_COMMANDS_V1 {
            matrix.push((*command, *example_relative));
        }
        if is_explain_example_v1(example_relative) {
            matrix.push(("explain", *example_relative));
        }
    }
    matrix
}

fn is_explain_example_v1(example_relative: &str) -> bool {
    EXPLAIN_EXAMPLE_PREFIXES_V1
        .iter()
        .any(|prefix| example_relative.starts_with(prefix))
}

/// Layer 1: denylist scan across the whole matrix. Not a single runtime-truth
/// token may appear in any read-only output.
#[test]
fn no_runtime_leakage_denylist_sweep_v1() {
    let matrix = sweep_matrix_v1();
    assert!(!matrix.is_empty(), "sweep matrix must be non-empty");

    for (command, example_relative) in matrix {
        let stdout = run_command_file_stdout_v1(command, example_relative);
        assert_no_forbidden_tokens_v1(command, example_relative, &stdout);
    }
}

/// Layer 2: positive honest no-result invariants for `headless` on every
/// shipped example.
#[test]
fn headless_honest_no_result_invariant_v1() {
    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        let stdout = run_command_file_stdout_v1("headless", example_relative);
        for required in [
            "headless.mode=read-only",
            "headless.command_plane=forbidden",
            "headless.no_result_reason=read-only-shell-has-no-execution-backend",
        ] {
            assert!(
                stdout.contains(required),
                "headless missing honest marker `{required}` for `{example_relative}`\n\
                 --- stdout ---\n{stdout}"
            );
        }
    }
}

/// Layer 2: `doctor.status` must be one of the honest no/partial-result statuses
/// and never a positive runtime claim.
#[test]
fn doctor_status_is_honest_invariant_v1() {
    const HONEST_STATUSES_V1: &[&str] = &["no-result", "parse-error", "sema-error"];
    const DISHONEST_STATUSES_V1: &[&str] = &["success", "executed", "ok"];

    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        let stdout = run_command_file_stdout_v1("doctor", example_relative);
        let status_line = stdout
            .lines()
            .find_map(|line| line.strip_prefix("doctor.status="))
            .unwrap_or_else(|| {
                panic!("doctor output missing `doctor.status=` for `{example_relative}`\n{stdout}")
            });
        assert!(
            HONEST_STATUSES_V1.contains(&status_line),
            "doctor.status=`{status_line}` is not an honest status for `{example_relative}`\n{stdout}"
        );
        assert!(
            !DISHONEST_STATUSES_V1.contains(&status_line),
            "doctor.status=`{status_line}` is a dishonest runtime claim for `{example_relative}`"
        );
    }
}

/// Layer 3: degraded-reason authority invariant. Every degraded reason emitted
/// by `headless`, `doctor`, or `explain` across the sweep must be a single
/// honest-absence enum ending with `-unavailable`. Also asserts the sweep is
/// non-vacuous: it observed a meaningful number of degraded reasons.
#[test]
fn degraded_reasons_are_honest_absence_authority_v1() {
    let mut observed_total = 0_usize;

    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        let mut command_outputs = vec![
            (
                "headless",
                run_command_file_stdout_v1("headless", example_relative),
            ),
            (
                "doctor",
                run_command_file_stdout_v1("doctor", example_relative),
            ),
        ];
        if is_explain_example_v1(example_relative) {
            command_outputs.push((
                "explain",
                run_command_file_stdout_v1("explain", example_relative),
            ));
        }

        for (command, stdout) in command_outputs {
            for reason in extract_degraded_reasons_v1(&stdout) {
                assert!(
                    reason.ends_with(DEGRADED_REASON_SUFFIX_V1),
                    "degraded reason `{reason}` from command=`{command}` example=`{example_relative}` \
                     does not end with `{DEGRADED_REASON_SUFFIX_V1}` (positive runtime claim leaked!)"
                );
                observed_total += 1;
            }
        }
    }

    assert!(
        observed_total >= 3,
        "expected several degraded reasons across the sweep (non-vacuous invariant), \
         observed only {observed_total}"
    );
}

/// E03C (S6): a `LET` / `WITH` composition is a multi-line, `--file`-only
/// document outside the shipped 17-root example corpus, so it is swept
/// separately. It must obey the same no-runtime-leakage contract: no forbidden
/// token in any command's output, and every degraded reason it surfaces (the
/// composition lane) is an honest `-unavailable` absence — never a materialized
/// working set or runtime count.
#[test]
fn composition_document_obeys_no_runtime_leakage_v1() {
    let doc = concat!(
        "LET suspects = SAMPLE query=AuthMiddleware ;; scope=repo::auth ;; mode=semantic ;; limit=20\n",
        "FIND taint api.entry -> db.write WITH suspects\n",
    );
    let path = write_target_tmp_input_v1("no_leak_composition.qglang", doc);
    let path_str = path.to_str().expect("temp path is valid UTF-8");
    for command in UNIVERSAL_COMMANDS_V1 {
        let output = run_cli_argv_v1(&[command, "--file", path_str]);
        assert!(
            output.status.success(),
            "command `{command}` unexpectedly failed for the composition document \
             (status={:?})\nstderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = cli_stdout_v1(&output);
        assert_no_forbidden_tokens_v1(command, "composition", &stdout);
        for reason in extract_degraded_reasons_v1(&stdout) {
            assert!(
                reason.ends_with(DEGRADED_REASON_SUFFIX_V1),
                "command `{command}`: composition degraded reason `{reason}` must end with \
                 `{DEGRADED_REASON_SUFFIX_V1}`"
            );
        }
    }
}
