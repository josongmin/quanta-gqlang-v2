#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! E03C — `LET` / `WITH` composition SCENARIO MATRIX (binary e2e hellgate).
//!
//! Grammar: `[LET <name> = <producer>]+` then `<consumer-body> WITH <binding>`
//! (the body names the working set it consumes, at the point of consumption).
//! Unfolds the contract across a wide, adversarial spread driven through the
//! real CLI binary's `doctor` command — the single front door that exercises all
//! three classification lanes (no-result / parse-error / sema-error) with their
//! exit codes and channel discipline.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

#[derive(Clone, Copy)]
enum ExpectV1 {
    NoResult,
    ParseError(&'static str),
    SemaError(&'static str),
}

struct ScenarioV1 {
    label: &'static str,
    doc: &'static str,
    expect: ExpectV1,
}

fn scenarios_v1() -> Vec<ScenarioV1> {
    use ExpectV1::{NoResult, ParseError, SemaError};
    vec![
        // ---- positive spread (valid -> no-result, one+ composition root)
        ScenarioV1 { label: "p_select_body_find", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x\n", expect: NoResult },
        ScenarioV1 { label: "p_sample_body_select", doc: "LET x = SAMPLE query=auth ;; scope=repo::auth ;; mode=semantic ;; limit=5\nSELECT id FROM catalog.users LIMIT 5 WITH x\n", expect: NoResult },
        ScenarioV1 { label: "p_find_body_select", doc: "LET x = FIND taint api.entry -> db.write\nSELECT id FROM catalog.users LIMIT 5 WITH x\n", expect: NoResult },
        ScenarioV1 { label: "p_body_explain", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nEXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm WITH x\n", expect: NoResult },
        ScenarioV1 { label: "p_multi_two_lets", doc: "LET a = SELECT id FROM catalog.users LIMIT 5\nLET b = SAMPLE query=auth ;; scope=repo::auth ;; mode=semantic ;; limit=3\nFIND taint api.entry -> db.write WITH b\n", expect: NoResult },
        ScenarioV1 { label: "p_lowercase_heads", doc: "let x = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write with x\n", expect: NoResult },
        ScenarioV1 { label: "p_blank_line_in_block", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\n\nFIND taint api.entry -> db.write WITH x\n", expect: NoResult },
        ScenarioV1 { label: "p_two_compositions", doc: "LET a = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH a\nLET b = SELECT id FROM catalog.users LIMIT 6\nSELECT id FROM catalog.users LIMIT 7 WITH b\n", expect: NoResult },
        ScenarioV1 { label: "p_plain_root_above_composition", doc: "SHOW models\nLET x = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x\n", expect: NoResult },

        // ---- structural parse faults (exit 1, parse-error, exact code)
        ScenarioV1 { label: "f_missing_equals", doc: "LET x SAMPLE query=a ;; scope=repo::a ;; mode=semantic ;; limit=5\nFIND taint api.entry -> db.write WITH x\n", expect: ParseError("composition_let_missing_equals") },
        ScenarioV1 { label: "f_name_leading_digit", doc: "LET 9bad = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x\n", expect: ParseError("composition_invalid_binding_name") },
        ScenarioV1 { label: "f_name_uppercase", doc: "LET CAP = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x\n", expect: ParseError("composition_invalid_binding_name") },
        ScenarioV1 { label: "f_nested_binding", doc: "LET x = LET y = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x\n", expect: ParseError("composition_nested_binding") },
        ScenarioV1 { label: "f_standalone_with_line", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nWITH x\nFIND taint api.entry -> db.write\n", expect: ParseError("composition_with_must_trail_body") },
        ScenarioV1 { label: "f_unconsumed_let_alone", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\n", expect: ParseError("composition_unconsumed_let") },
        ScenarioV1 { label: "f_unconsumed_let_then_plain_root", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nSHOW models\n", expect: ParseError("composition_unconsumed_let") },
        ScenarioV1 { label: "f_duplicate_binding", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nLET x = SELECT id FROM catalog.users LIMIT 6\nFIND taint api.entry -> db.write WITH x\n", expect: ParseError("composition_duplicate_binding") },

        // ---- sema faults (exit 1, sema-error)
        ScenarioV1 { label: "s_undefined_alias", doc: "FIND taint api.entry -> db.write WITH ghost\n", expect: SemaError("composition_undefined_binding") },
        ScenarioV1 { label: "s_producer_show", doc: "LET x = SHOW models\nFIND taint api.entry -> db.write WITH x\n", expect: SemaError("composition_invalid_producer") },
        ScenarioV1 { label: "s_producer_describe", doc: "LET x = DESCRIBE rule::owasp.sql-injection\nFIND taint api.entry -> db.write WITH x\n", expect: SemaError("composition_invalid_producer") },
        ScenarioV1 { label: "s_producer_compare", doc: "LET x = COMPARE baseline=run::a ;; candidate=run::b ;; metric=semantic\nFIND taint api.entry -> db.write WITH x\n", expect: SemaError("composition_invalid_producer") },
        ScenarioV1 { label: "s_producer_pack", doc: "LET x = PACK secure-pack ;; source=registry::security ;; activation=session\nFIND taint api.entry -> db.write WITH x\n", expect: SemaError("composition_invalid_producer") },
        ScenarioV1 { label: "s_body_non_consumer_show", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nSHOW models WITH x\n", expect: SemaError("composition_invalid_body") },
        ScenarioV1 { label: "s_body_non_consumer_compare", doc: "LET x = SELECT id FROM catalog.users LIMIT 5\nCOMPARE baseline=run::a ;; candidate=run::b ;; metric=semantic WITH x\n", expect: SemaError("composition_invalid_body") },
        // producer's own sema error must pass through unmasked, NOT be rewrapped
        ScenarioV1 { label: "s_producer_own_error_passthrough", doc: "LET x = FIND path api.entry -> db.write | depth=0\nFIND taint api.entry -> db.write WITH x\n", expect: SemaError("greater than zero") },
    ]
}

#[test]
fn composition_scenario_matrix_holds_through_the_doctor_front_door_v1() {
    for scenario in scenarios_v1() {
        let path =
            write_target_tmp_input_v1(&format!("matrix_{}.qglang", scenario.label), scenario.doc);
        let out = run_cli_argv_v1(&["doctor", "--file", path.to_str().expect("utf8 path")]);
        let stdout = cli_stdout_v1(&out);
        let stderr = cli_stderr_v1(&out);
        match scenario.expect {
            ExpectV1::NoResult => {
                assert_eq!(
                    out.status.code(),
                    Some(0),
                    "{}: expected no-result exit 0, stderr={stderr}",
                    scenario.label
                );
                assert!(
                    stderr.is_empty(),
                    "{}: classified outcome must not write stderr: {stderr}",
                    scenario.label
                );
                assert!(
                    stdout.contains("doctor.status=no-result"),
                    "{}: {stdout}",
                    scenario.label
                );
                assert!(
                    stdout.contains("kind=composition"),
                    "{}: a composition root must be present:\n{stdout}",
                    scenario.label
                );
            }
            ExpectV1::ParseError(code) => {
                assert_eq!(
                    out.status.code(),
                    Some(1),
                    "{}: expected parse-error exit 1, stdout={stdout}",
                    scenario.label
                );
                assert!(
                    stderr.is_empty(),
                    "{}: classified parse-error must not write stderr: {stderr}",
                    scenario.label
                );
                assert!(
                    stdout.contains("doctor.status=parse-error"),
                    "{}: {stdout}",
                    scenario.label
                );
                assert!(
                    stdout.contains(code),
                    "{}: expected parse code `{code}` in:\n{stdout}",
                    scenario.label
                );
            }
            ExpectV1::SemaError(token) => {
                assert_eq!(
                    out.status.code(),
                    Some(1),
                    "{}: expected sema-error exit 1, stdout={stdout}",
                    scenario.label
                );
                assert!(
                    stderr.is_empty(),
                    "{}: classified sema-error must not write stderr: {stderr}",
                    scenario.label
                );
                assert!(
                    stdout.contains("doctor.status=sema-error"),
                    "{}: {stdout}",
                    scenario.label
                );
                assert!(
                    stdout.contains(token),
                    "{}: expected sema token `{token}` in:\n{stdout}",
                    scenario.label
                );
                if scenario.label == "s_producer_own_error_passthrough" {
                    assert!(
                        !stdout.contains("composition_invalid_producer"),
                        "{}: inner error wrongly rewrapped:\n{stdout}",
                        scenario.label
                    );
                }
            }
        }
    }
}

#[test]
fn two_compositions_lower_to_two_roots_v1() {
    let doc = "LET a = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH a\nLET b = SELECT id FROM catalog.users LIMIT 6\nSELECT id FROM catalog.users LIMIT 7 WITH b\n";
    let path = write_target_tmp_input_v1("matrix_two_comp_count.qglang", doc);
    let out = run_cli_argv_v1(&["query", "--file", path.to_str().expect("utf8 path")]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = cli_stdout_v1(&out);
    assert!(stdout.contains("query.root_count=2"), "{stdout}");
    assert!(
        stdout.contains("query.root[0].ir.kind=composition"),
        "{stdout}"
    );
    assert!(
        stdout.contains("query.root[1].ir.kind=composition"),
        "{stdout}"
    );
}

#[test]
fn plain_root_above_composition_keeps_order_v1() {
    let doc = "SHOW models\nLET x = SELECT id FROM catalog.users LIMIT 5\nFIND taint api.entry -> db.write WITH x\n";
    let path = write_target_tmp_input_v1("matrix_plain_above.qglang", doc);
    let out = run_cli_argv_v1(&["query", "--file", path.to_str().expect("utf8 path")]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = cli_stdout_v1(&out);
    assert!(stdout.contains("query.root_count=2"), "{stdout}");
    assert!(
        stdout.contains("query.root[0].ir.kind=interactive"),
        "show lowers to interactive:\n{stdout}"
    );
    assert!(
        stdout.contains("query.root[1].ir.kind=composition"),
        "{stdout}"
    );
}
