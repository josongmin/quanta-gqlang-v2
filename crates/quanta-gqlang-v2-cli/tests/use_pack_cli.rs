#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use std::path::PathBuf;
use std::process::{Command, Output};

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

fn stdout_success_v1(output: Output) -> String {
    assert!(
        output.status.success(),
        "status={:?}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout utf-8")
}

fn stderr_string_v1(output: Output) -> String {
    String::from_utf8(output.stderr).expect("stderr utf-8")
}

#[test]
fn use_pack_and_require_certified_examples_surface_deterministic_read_only_projection() {
    let use_pack_query = stdout_success_v1(run_cli_file_v1(
        "query",
        "examples/use_pack/secure_pack.qglang",
    ));
    assert!(use_pack_query.contains("query.root[0].syntax.kind=meta"));
    assert!(use_pack_query.contains("query.root[0].typed.kind=meta"));
    assert!(use_pack_query.contains("query.root[0].ir.kind=meta"));
    assert!(use_pack_query.contains(
        "query.root[0].syntax.summary=use-pack id=secure-pack activation=session lockfile=packs/secure-pack.lock",
    ));
    assert!(use_pack_query.contains(
        "query.root[0].typed.summary=use-pack id=secure-pack activation=session lockfile=packs/secure-pack.lock",
    ));
    assert!(use_pack_query.contains(
        "query.root[0].ir.summary=use-pack id=secure-pack activation=session lockfile=packs/secure-pack.lock",
    ));

    let use_pack_headless = stdout_success_v1(run_cli_file_v1(
        "headless",
        "examples/use_pack/secure_pack.qglang",
    ));
    assert!(use_pack_headless.contains("headless.mode=read-only"));
    assert!(use_pack_headless.contains("headless.command_plane=forbidden"));
    assert!(use_pack_headless.contains(
        "query.root[0].syntax.summary=use-pack id=secure-pack activation=session lockfile=packs/secure-pack.lock",
    ));
    assert!(!use_pack_headless.contains("headless.degraded[0]="));

    let require_query = stdout_success_v1(run_cli_file_v1(
        "query",
        "examples/require_certified/strict_only.qglang",
    ));
    assert!(require_query.contains("query.root[0].syntax.kind=meta"));
    assert!(require_query.contains(
        "query.root[0].syntax.summary=require-certified truth_requirement=certified-only admission_policy=fail-closed",
    ));
    assert!(require_query.contains(
        "query.root[0].typed.summary=require-certified truth_requirement=certified-only admission_policy=fail-closed",
    ));
    assert!(require_query.contains(
        "query.root[0].ir.summary=require-certified truth_requirement=certified-only admission_policy=fail-closed",
    ));

    let require_headless = stdout_success_v1(run_cli_file_v1(
        "headless",
        "examples/require_certified/strict_only.qglang",
    ));
    assert!(require_headless.contains("headless.mode=read-only"));
    assert!(require_headless.contains(
        "query.root[0].syntax.summary=require-certified truth_requirement=certified-only admission_policy=fail-closed",
    ));
    assert!(!require_headless.contains("headless.degraded[0]="));
}

#[test]
fn malformed_use_pack_surfaces_fail_closed_on_cli() {
    let missing_lockfile = run_cli_v1(&[
        "query",
        "USE",
        "PACK",
        "secure-pack",
        ";;",
        "activation=session",
    ]);
    assert_eq!(missing_lockfile.status.code(), Some(1));
    let missing_lockfile_stderr = stderr_string_v1(missing_lockfile);
    assert!(missing_lockfile_stderr.contains("use_pack_missing_lockfile"));

    let invalid_activation = run_cli_v1(&[
        "query",
        "USE",
        "PACK",
        "secure-pack",
        ";;",
        "activation=manual",
        ";;",
        "lockfile=packs/secure-pack.lock",
    ]);
    assert_eq!(invalid_activation.status.code(), Some(1));
    let invalid_activation_stderr = stderr_string_v1(invalid_activation);
    assert!(invalid_activation_stderr.contains("use_pack_invalid_activation"));

    let invalid_lockfile = run_cli_v1(&[
        "query",
        "USE",
        "PACK",
        "secure-pack",
        ";;",
        "activation=session",
        ";;",
        "lockfile=packs/secure-pack.txt",
    ]);
    assert_eq!(invalid_lockfile.status.code(), Some(1));
    let invalid_lockfile_stderr = stderr_string_v1(invalid_lockfile);
    assert!(invalid_lockfile_stderr.contains("use_pack_invalid_lockfile"));

    let duplicate_activation = run_cli_v1(&[
        "query",
        "USE",
        "PACK",
        "secure-pack",
        ";;",
        "activation=session",
        ";;",
        "activation=workspace",
        ";;",
        "lockfile=packs/secure-pack.lock",
    ]);
    assert_eq!(duplicate_activation.status.code(), Some(1));
    let duplicate_activation_stderr = stderr_string_v1(duplicate_activation);
    assert!(duplicate_activation_stderr.contains("duplicate_use_pack_activation"));
}
