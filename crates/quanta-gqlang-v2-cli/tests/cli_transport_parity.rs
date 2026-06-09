#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: CLI transport parity (inline argv ≡ `--file`).
//!
//! `main.rs` joins inline argv with single spaces and reads `--file` verbatim,
//! so for a single-spaced document both transports must produce byte-identical
//! stdout and the same exit code on every read-only command. This is the rail
//! that proves the front door's meaning is transport-invariant.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

const COMMANDS_V1: &[&str] = &["parse", "sema", "query", "fmt", "doctor", "headless"];

/// Single-spaced documents so that `argv.join(" ")` reconstructs the on-disk
/// bytes exactly — one query-family root and one meta-family root.
const PARITY_DOCS_V1: &[(&str, &str)] = &[
    (
        "parity_select.qglang",
        "SELECT id FROM catalog.users WHERE active = true LIMIT 7",
    ),
    (
        "parity_use_pack.qglang",
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
    ),
];

#[test]
fn inline_and_file_transports_are_byte_identical_v1() {
    for (file_name, document) in PARITY_DOCS_V1 {
        let path = write_target_tmp_input_v1(file_name, document);
        let path_str = path.to_str().expect("utf8 path");
        let inline_tokens: Vec<&str> = document.split(' ').collect();

        for command in COMMANDS_V1 {
            let mut inline_argv = Vec::with_capacity(inline_tokens.len() + 1);
            inline_argv.push(*command);
            inline_argv.extend(inline_tokens.iter().copied());

            let inline = run_cli_argv_v1(&inline_argv);
            let file = run_cli_argv_v1(&[command, "--file", path_str]);

            assert_eq!(
                inline.status.code(),
                file.status.code(),
                "{command} on `{document}`: exit-code parity (inline stderr={}, file stderr={})",
                cli_stderr_v1(&inline),
                cli_stderr_v1(&file)
            );
            assert_eq!(
                cli_stdout_v1(&inline),
                cli_stdout_v1(&file),
                "{command} on `{document}`: stdout transport parity"
            );
            assert!(
                inline.status.success(),
                "{command} on `{document}`: read-only command must succeed, stderr={}",
                cli_stderr_v1(&inline)
            );
        }
    }
}
