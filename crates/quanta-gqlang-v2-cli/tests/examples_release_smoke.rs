#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

mod support;

use support::assert_shipped_examples_listed_v1;

const README_V1: &str = include_str!("../../../README.md");
const LANGUAGE_README_V1: &str = include_str!("../../../docs/language/README.md");
const LANGUAGE_RELEASE_DISCIPLINE_V1: &str =
    include_str!("../../../docs/language/release_discipline.md");
const LANGUAGE_EXAMPLES_INDEX_V1: &str = include_str!("../../../docs/language/examples_index.md");

fn example_path_v1(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn run_cli_v1(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .args(args)
        .output()
        .expect("spawn qglang cli")
}

fn run_cli_file_v1(command: &str, example_relative: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .arg(command)
        .arg("--file")
        .arg(example_path_v1(example_relative))
        .output()
        .expect("spawn qglang cli")
}

fn stdout_string_v1(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf-8")
}

fn stderr_string_v1(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf-8")
}

#[test]
fn release_docs_state_public_language_constraints_v1() {
    for required in [
        "parser-stable",
        "read-only",
        "no runtime truth",
        "experimental",
        "unsupported",
    ] {
        assert!(LANGUAGE_README_V1.contains(required));
        assert!(LANGUAGE_RELEASE_DISCIPLINE_V1.contains(required));
    }

    assert!(README_V1.contains("docs/language/README.md"));
    assert!(README_V1.contains("stdio JSON requests only"));
    assert_shipped_examples_listed_v1(README_V1, "README.md");
    assert_shipped_examples_listed_v1(
        LANGUAGE_EXAMPLES_INDEX_V1,
        "docs/language/examples_index.md",
    );
}

#[test]
fn release_smoke_uses_shipped_binary_front_door_v1() {
    let select_path = example_path_v1("examples/select/basic.qglang");
    let select_source = fs::read_to_string(&select_path).expect("read select example");
    let fmt_output = run_cli_file_v1("fmt", "examples/select/basic.qglang");
    assert!(
        fmt_output.status.success(),
        "stderr={}",
        stderr_string_v1(&fmt_output)
    );
    assert_eq!(stdout_string_v1(&fmt_output).trim(), select_source.trim());

    let doctor_output = run_cli_file_v1("doctor", "examples/show/capabilities.qglang");
    assert!(
        doctor_output.status.success(),
        "stderr={}",
        stderr_string_v1(&doctor_output)
    );
    let doctor_stdout = stdout_string_v1(&doctor_output);
    assert!(doctor_stdout.contains("doctor.status=no-result"));
    assert!(doctor_stdout.contains("doctor.root[0].kind=show"));

    let inline_query = run_cli_v1(&[
        "query",
        "SELECT",
        "id,",
        "profile.name",
        "FROM",
        "catalog.users",
        "WHERE",
        "active",
        "=",
        "true",
        "LIMIT",
        "7",
    ]);
    assert!(
        inline_query.status.success(),
        "stderr={}",
        stderr_string_v1(&inline_query)
    );
    let inline_stdout = stdout_string_v1(&inline_query);
    assert!(inline_stdout.contains("query.root[0].syntax.kind=select"));
    assert!(inline_stdout.contains("query.root[0].typed.kind=select"));
    assert!(inline_stdout.contains("query.root[0].ir.kind=select"));
}

#[test]
fn release_cli_fails_closed_on_bad_argv_and_bad_input_v1() {
    let malformed_argv = run_cli_v1(&["mutate", "SHOW", "capabilities"]);
    assert_eq!(malformed_argv.status.code(), Some(2));
    let malformed_argv_stderr = stderr_string_v1(&malformed_argv);
    assert!(malformed_argv_stderr.contains("cli.error.class=usage"));
    assert!(malformed_argv_stderr.contains("cli.usage="));

    let missing_file = run_cli_v1(&["query", "--file", "/tmp/qgv2-does-not-exist.qglang"]);
    assert_eq!(missing_file.status.code(), Some(1));
    let missing_file_stderr = stderr_string_v1(&missing_file);
    assert!(missing_file_stderr.contains("cli.error.class=io"));

    let parse_error = run_cli_v1(&["doctor", "SELECT"]);
    assert_eq!(parse_error.status.code(), Some(1));
    let parse_error_stdout = stdout_string_v1(&parse_error);
    assert!(parse_error_stdout.contains("doctor.status=parse-error"));
    assert!(parse_error_stdout.contains("doctor.parse_diagnostic[0]="));
}
