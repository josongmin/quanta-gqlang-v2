#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use std::io::{BufRead, BufReader, Cursor, Write};
use std::process::{Command, Stdio};

use quanta_gqlang_v2_lsp::run_stdio_session_v1;

fn run_request_v1(input: &str) -> (u8, String) {
    let mut reader = Cursor::new(input.as_bytes());
    let mut output = Vec::new();
    let status = run_stdio_session_v1(&mut reader, &mut output).expect("stdio run should work");
    let stdout = String::from_utf8(output).expect("stdout should be utf8");
    (status, stdout)
}

fn spawn_binary_v1() -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("binary should spawn")
}

#[test]
fn stdio_front_door_supports_all_read_only_methods_v1() {
    let (hover_status, hover_stdout) =
        run_request_v1(r#"{"method":"hover","query":"SELECT id FROM catalog.files"}"#);
    assert_eq!(hover_status, 0);
    assert_eq!(
        hover_stdout.trim(),
        r#"{"ok":true,"method":"hover","result":{"root_keyword":"SELECT","category":"query","typed_target":"catalog.files","summary":["projection=id"]}}"#
    );

    let (meta_hover_status, meta_hover_stdout) = run_request_v1(
        r#"{"method":"hover","query":"USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock"}"#,
    );
    assert_eq!(meta_hover_status, 0);
    assert_eq!(
        meta_hover_stdout.trim(),
        r#"{"ok":true,"method":"hover","result":{"root_keyword":"USE PACK","category":"meta","typed_target":"secure-pack","summary":["activation=session","lockfile=packs/secure-pack.lock"]}}"#
    );

    let (completion_status, completion_stdout) =
        run_request_v1(r#"{"method":"completion","prefix":"pro"}"#);
    assert_eq!(completion_status, 0);
    assert!(completion_stdout.contains(r#""method":"completion""#));
    assert!(completion_stdout.contains(r#""label":"PROFILE""#));
    assert!(completion_stdout.contains(r#""match_reason":"exact-scope""#));

    let (symbols_status, symbols_stdout) = run_request_v1(
        r#"{"method":"symbols","query":"PACK secure-pack ;; source=registry::security ;; activation=session"}"#,
    );
    assert_eq!(symbols_status, 0);
    assert!(symbols_stdout.contains(r#""method":"symbols""#));
    assert!(symbols_stdout.contains(r#""name":"document""#));
    assert!(symbols_stdout.contains(r#""name":"PACK""#));

    let (diagnostics_status, diagnostics_stdout) =
        run_request_v1(r#"{"method":"diagnostics","query":"SELECT id catalog.users"}"#);
    assert_eq!(diagnostics_status, 0);
    assert!(diagnostics_stdout.contains(r#""method":"diagnostics""#));
    assert!(diagnostics_stdout.contains(r#""code":"expected_from_keyword""#));

    let (zero_query_status, zero_query_stdout) = run_request_v1(r#"{"method":"zero_query"}"#);
    assert_eq!(zero_query_status, 0);
    assert!(zero_query_stdout.contains(r#""method":"zero_query""#));
    assert!(zero_query_stdout.contains(r#""mode":"read-only""#));
    assert!(zero_query_stdout.contains(r#""groups":["#));

    let (quickfix_status, quickfix_stdout) =
        run_request_v1(r#"{"method":"quickfix","input":"DESCRIBE model::billing_warehouse"}"#);
    assert_eq!(quickfix_status, 0);
    assert!(quickfix_stdout.contains(r#""method":"quickfix""#));
    assert!(quickfix_stdout.contains(r#""code":"missing-model-scaffold""#));
    assert!(quickfix_stdout.contains(r#""edits_document":false"#));
}

#[test]
fn stdio_front_door_rejects_unknown_or_mutating_methods_v1() {
    let (rename_status, rename_stdout) = run_request_v1(r#"{"method":"rename","input":"MODEL a"}"#);
    assert_eq!(rename_status, 1);
    assert_eq!(
        rename_stdout.trim(),
        r#"{"ok":false,"error":{"code":"command_plane_forbidden","message":"mutating method `rename` is forbidden on the read-only front door"}}"#
    );

    let (unknown_status, unknown_stdout) =
        run_request_v1(r#"{"method":"workspace/index","query":"SELECT id FROM catalog.files"}"#);
    assert_eq!(unknown_status, 1);
    assert_eq!(
        unknown_stdout.trim(),
        r#"{"ok":false,"error":{"code":"unsupported_method","message":"unknown read-only method `workspace/index`"}}"#
    );
}

#[test]
fn stdio_front_door_fails_closed_on_malformed_or_unexpected_input_v1() {
    let (malformed_status, malformed_stdout) = run_request_v1(r#"{"method":"hover","query":true}"#);
    assert_eq!(malformed_status, 1);
    assert_eq!(
        malformed_stdout.trim(),
        r#"{"ok":false,"error":{"code":"malformed_json","message":"only string field values are supported"}}"#
    );

    let (unexpected_status, unexpected_stdout) =
        run_request_v1(r#"{"method":"zero_query","query":"SELECT id FROM catalog.files"}"#);
    assert_eq!(unexpected_status, 1);
    assert_eq!(
        unexpected_stdout.trim(),
        r#"{"ok":false,"error":{"code":"unexpected_field","message":"unexpected fields for `zero_query`: query"}}"#
    );
}

#[test]
fn stdio_front_door_surfaces_handler_fail_closed_errors_v1() {
    let (status, stdout) = run_request_v1(r#"{"method":"quickfix","input":"DESCRIBE model::"}"#);
    assert_eq!(status, 1);
    assert_eq!(
        stdout.trim(),
        r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=describe_target_invalid_shape message=DESCRIBE target must use `<kind>::<identifier>`"}}"#
    );

    let (meta_status, meta_stdout) = run_request_v1(
        r#"{"method":"hover","query":"USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.txt"}"#,
    );
    assert_eq!(meta_status, 1);
    assert_eq!(
        meta_stdout.trim(),
        r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=use_pack_invalid_lockfile message=USE PACK lockfile `packs/secure-pack.txt` must end with `.lock` and include a non-empty file stem"}}"#
    );
}

#[test]
fn stdio_binary_keeps_session_open_across_multiple_requests_v1() {
    let mut child = spawn_binary_v1();
    let mut stdin = child.stdin.take().expect("stdin pipe");
    let stdout = child.stdout.take().expect("stdout pipe");
    let mut stdout_reader = BufReader::new(stdout);

    stdin
        .write_all(br#"{"method":"hover","query":"SELECT id FROM catalog.files"}"#)
        .expect("first request write");
    stdin.write_all(b"\n").expect("newline write");
    stdin.flush().expect("first request flush");

    let mut first_line = String::new();
    stdout_reader
        .read_line(&mut first_line)
        .expect("first response should arrive before EOF");
    assert!(first_line.contains(r#""ok":true"#));
    assert!(first_line.contains(r#""method":"hover""#));

    stdin
        .write_all(br#"{"method":"zero_query"}"#)
        .expect("second request write");
    stdin.write_all(b"\n").expect("second newline write");
    stdin.flush().expect("second request flush");

    let mut second_line = String::new();
    stdout_reader
        .read_line(&mut second_line)
        .expect("second response should arrive in the same session");
    assert!(second_line.contains(r#""ok":true"#));
    assert!(second_line.contains(r#""method":"zero_query""#));

    drop(stdin);
    let status = child.wait().expect("child should exit");
    assert!(status.success());
}

#[test]
fn stdio_binary_accepts_surrogate_pair_json_escape_v1() {
    let mut child = spawn_binary_v1();
    let mut stdin = child.stdin.take().expect("stdin pipe");
    let stdout = child.stdout.take().expect("stdout pipe");
    let mut stdout_reader = BufReader::new(stdout);

    stdin
        .write_all(br#"{"method":"completion","prefix":"\uD83D\uDE00"}"#)
        .expect("request write");
    stdin.write_all(b"\n").expect("newline write");
    stdin.flush().expect("request flush");
    drop(stdin);

    let mut response = String::new();
    stdout_reader
        .read_line(&mut response)
        .expect("response should be readable");

    let status = child.wait().expect("child should exit");
    assert!(status.success());
    assert!(response.contains(r#""ok":true"#));
    assert!(response.contains(r#""method":"completion""#));
}
