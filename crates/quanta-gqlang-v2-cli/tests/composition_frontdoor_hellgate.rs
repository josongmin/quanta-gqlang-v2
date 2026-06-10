#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! ============================================================================
//! E03C `LET` / `WITH` composition — CLI binary front-door hellgate.
//!
//! Grammar: `[LET <name> = <producer>]+` then `<consumer-body> WITH <binding>`
//! (the body explicitly names the working set it consumes). Single-degraded
//! -per-root model (D6): a composition document surfaces the composition's own
//! degraded lane; the body's lane is preserved in the lowered IR, not flattened.
//! ============================================================================

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

/// S1 happy pipeline fixture (multi-line, so `--file`-only by construction).
const S1_DOC_V1: &str = concat!(
    "LET suspects = SAMPLE query=AuthMiddleware ;; scope=repo::auth ;; mode=semantic ;; limit=20\n",
    "FIND taint api.entry -> db.write WITH suspects\n",
);

fn run_file_v1(command: &str, name: &str, doc: &str) -> std::process::Output {
    let path = write_target_tmp_input_v1(name, doc);
    run_cli_argv_v1(&[command, "--file", path.to_str().expect("utf8 path")])
}

// ---------------------------------------------------------------- S1 happy
#[test]
fn s1_composition_parses_types_lowers_through_cli_v1() {
    let parse = run_file_v1("parse", "s1_parse.qglang", S1_DOC_V1);
    assert_eq!(
        parse.status.code(),
        Some(0),
        "stderr={}",
        cli_stderr_v1(&parse)
    );
    let parse_out = cli_stdout_v1(&parse);
    assert!(parse_out.contains("parse.root_count=1"), "{parse_out}");
    assert!(
        parse_out.contains("parse.root[0].kind=composition"),
        "{parse_out}"
    );
    assert!(
        parse_out.contains("parse.root[0].summary=lets=1 with=suspects body=find"),
        "composition summary contract drifted:\n{parse_out}"
    );

    let sema = run_file_v1("sema", "s1_sema.qglang", S1_DOC_V1);
    assert_eq!(sema.status.code(), Some(0));
    assert!(cli_stdout_v1(&sema).contains("sema.root[0].kind=composition"));

    let query = run_file_v1("query", "s1_query.qglang", S1_DOC_V1);
    assert_eq!(query.status.code(), Some(0));
    let query_out = cli_stdout_v1(&query);
    assert!(
        query_out.contains("query.root[0].ir.kind=composition"),
        "{query_out}"
    );
    assert!(
        query_out.contains("suspects"),
        "working-set ref id absent:\n{query_out}"
    );
    assert!(
        query_out.contains("find"),
        "lowered body kind absent:\n{query_out}"
    );
}

#[test]
fn s1_headless_and_doctor_surface_composition_degraded_v1() {
    let headless = run_file_v1("headless", "s1_headless.qglang", S1_DOC_V1);
    assert_eq!(headless.status.code(), Some(0));
    let h = cli_stdout_v1(&headless);
    assert!(h.contains("headless.mode=read-only"), "{h}");
    assert!(h.contains("headless.command_plane=forbidden"), "{h}");
    assert!(
        h.contains("headless.no_result_reason=read-only-shell-has-no-execution-backend"),
        "{h}"
    );
    assert!(
        h.contains("composition-materialization-unavailable"),
        "degraded missing:\n{h}"
    );

    let doctor = run_file_v1("doctor", "s1_doctor.qglang", S1_DOC_V1);
    assert_eq!(doctor.status.code(), Some(0));
    assert!(
        cli_stderr_v1(&doctor).is_empty(),
        "classified outcome must not write stderr"
    );
    let d = cli_stdout_v1(&doctor);
    assert!(d.contains("doctor.status=no-result"), "{d}");
    assert!(d.contains("composition-materialization-unavailable"), "{d}");
}

#[test]
fn s1_fmt_is_canonical_and_idempotent_v1() {
    let once = run_file_v1("fmt", "s1_fmt_a.qglang", S1_DOC_V1);
    assert_eq!(
        once.status.code(),
        Some(0),
        "stderr={}",
        cli_stderr_v1(&once)
    );
    let formatted = cli_stdout_v1(&once);
    assert!(formatted.contains("LET suspects = SAMPLE"), "{formatted}");
    assert!(formatted.contains("WITH suspects"), "{formatted}");
    let twice = run_file_v1("fmt", "s1_fmt_b.qglang", &formatted);
    assert_eq!(
        cli_stdout_v1(&twice),
        formatted,
        "composition fmt must be idempotent"
    );
}

// ---------------------------------------------------------------- S2 alias faults
#[test]
fn s2_undefined_alias_and_standalone_with_fail_closed_v1() {
    // Body names a binding that was never declared -> sema-error.
    let undefined = "FIND taint api.entry -> db.write WITH ghost\n";
    let out = run_file_v1("doctor", "s2_undef.qglang", undefined);
    assert_eq!(out.status.code(), Some(1));
    let o = cli_stdout_v1(&out);
    assert!(o.contains("doctor.status=sema-error"), "{o}");
    assert!(o.contains("composition_undefined_binding"), "{o}");

    // The old `WITH x` on its own line is now a parse fault: WITH must trail a
    // body as `<consumer-root> WITH <binding>`.
    let standalone = concat!(
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "WITH x\n",
        "FIND taint api.entry -> db.write\n",
    );
    let sout = run_file_v1("doctor", "s2_standalone.qglang", standalone);
    assert_eq!(sout.status.code(), Some(1));
    assert!(cli_stdout_v1(&sout).contains("composition_with_must_trail_body"));
}

// ---------------------------------------------------------------- S3 invalid producer
#[test]
fn s3_invalid_producer_is_a_positive_allow_set_v1() {
    for (name, producer) in [
        ("s3_show", "LET x = SHOW models"),
        (
            "s3_meta",
            "LET x = USE PACK secure-pack ;; activation=session ;; lockfile=packs/p.lock",
        ),
    ] {
        let doc = format!("{producer}\nFIND taint api.entry -> db.write WITH x\n");
        let out = run_file_v1("doctor", &format!("{name}.qglang"), &doc);
        assert_eq!(out.status.code(), Some(1), "{name}");
        assert!(
            cli_stdout_v1(&out).contains("composition_invalid_producer"),
            "{name}: {}",
            cli_stdout_v1(&out)
        );
    }
}

// ---------------------------------------------------------------- S4 structural faults
#[test]
fn s4_structural_parse_faults_each_have_a_distinct_code_v1() {
    let body = "FIND taint api.entry -> db.write";
    let cases: &[(&str, &str, &str)] = &[
        ("s4_eq", "LET x SAMPLE query=a ;; scope=repo::a ;; mode=semantic ;; limit=5", "composition_let_missing_equals"),
        ("s4_name", "LET 9bad = SELECT id FROM catalog.users LIMIT 5", "composition_invalid_binding_name"),
        ("s4_nested", "LET x = LET y = SELECT id FROM catalog.users LIMIT 5", "composition_nested_binding"),
        ("s4_dup", "LET x = SELECT id FROM catalog.users LIMIT 5\nLET x = SELECT id FROM catalog.users LIMIT 6", "composition_duplicate_binding"),
    ];
    for (name, head, code) in cases {
        let doc = format!("{head}\n{body} WITH x\n");
        let out = run_file_v1("doctor", &format!("{name}.qglang"), &doc);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{name}: stderr={}",
            cli_stderr_v1(&out)
        );
        assert!(
            cli_stdout_v1(&out).contains(code),
            "{name}: expected `{code}` in:\n{}",
            cli_stdout_v1(&out)
        );
    }

    // A LET with no consuming body.
    let unconsumed = run_file_v1(
        "doctor",
        "s4_unconsumed.qglang",
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
    );
    assert_eq!(unconsumed.status.code(), Some(1));
    assert!(cli_stdout_v1(&unconsumed).contains("composition_unconsumed_let"));
}

// ---------------------------------------------------------------- S5 producer error pass-through
#[test]
fn s5_producer_own_error_is_not_masked_by_composition_v1() {
    let doc = concat!(
        "LET x = FIND path api.entry -> db.write | depth=0\n",
        "FIND taint api.entry -> db.write WITH x\n",
    );
    let out = run_file_v1("doctor", "s5.qglang", doc);
    assert_eq!(out.status.code(), Some(1));
    let o = cli_stdout_v1(&out);
    assert!(o.contains("doctor.status=sema-error"), "{o}");
    assert!(
        o.contains("greater than zero"),
        "producer error masked:\n{o}"
    );
    assert!(
        !o.contains("composition_invalid_producer"),
        "inner error wrongly rewrapped:\n{o}"
    );
}

// ---------------------------------------------------------------- S6 body-consumer
#[test]
fn s6_non_consumer_body_fails_closed_v1() {
    // SHOW cannot consume a working set, so it is not an admissible body.
    let doc = "LET x = SELECT id FROM catalog.users LIMIT 5\nSHOW models WITH x\n";
    let out = run_file_v1("doctor", "s6_show_body.qglang", doc);
    assert_eq!(out.status.code(), Some(1));
    let o = cli_stdout_v1(&out);
    assert!(o.contains("doctor.status=sema-error"), "{o}");
    assert!(o.contains("composition_invalid_body"), "{o}");
}

// ---------------------------------------------------------------- S7 backward compat
#[test]
fn s7_plain_root_above_composition_does_not_greedily_group_v1() {
    let doc = concat!(
        "SHOW models\n",
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH x\n",
    );
    let out = run_file_v1("parse", "s7.qglang", doc);
    assert_eq!(out.status.code(), Some(0), "stderr={}", cli_stderr_v1(&out));
    let o = cli_stdout_v1(&out);
    assert!(
        o.contains("parse.root_count=2"),
        "expected 2 roots (show + composition):\n{o}"
    );
    assert!(o.contains("parse.root[0].kind=show"), "{o}");
    assert!(o.contains("parse.root[1].kind=composition"), "{o}");
}

// ---------------------------------------------------------------- S8 inline transport
#[test]
fn s8_inline_argv_cannot_express_a_multiline_composition_v1() {
    // Inline argv joins with single spaces; the LET producer then swallows the
    // rest of the line and fails closed as a classified parse-error — never a
    // half-composed root.
    let out = run_cli_argv_v1(&[
        "doctor",
        "LET",
        "x",
        "=",
        "SELECT",
        "id",
        "FROM",
        "catalog.users",
        "LIMIT",
        "5",
        "FIND",
        "taint",
        "api.entry",
        "->",
        "db.write",
        "WITH",
        "x",
    ]);
    assert_eq!(out.status.code(), Some(1), "stderr={}", cli_stderr_v1(&out));
    assert!(
        cli_stdout_v1(&out).contains("doctor.status=parse-error"),
        "inline composition must fail closed as parse-error:\n{}",
        cli_stdout_v1(&out)
    );
}
