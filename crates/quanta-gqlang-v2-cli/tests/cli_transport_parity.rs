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
//! stdout and the same exit code on every read-only command. Byte-equality
//! alone is satisfiable by *two empty outputs*, so this rail also pins, per
//! command, a non-empty success marker that the shared stdout must contain —
//! killing the vacuous-empty false positive — and asserts BOTH transports exit
//! 0 with empty stderr. Covered for a `SELECT` (query-family) root and a
//! `USE PACK` (meta-family) root.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

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

/// A per-command, document-specific success marker the shared stdout must
/// contain. `marker_for_doc` selects the marker by the document's root family
/// (`select` vs `meta`) where the command's output is root-kind sensitive; for
/// `fmt` the marker is the round-tripped source itself.
struct CommandMarkerV1 {
    command: &'static str,
    /// Returns the substring that proves a real, non-empty result for the given
    /// `(file_name, document)` parity doc.
    marker_for_doc: fn(file_name: &str, document: &str) -> String,
}

fn root_kind_for_doc_v1(file_name: &str) -> &'static str {
    match file_name {
        "parity_select.qglang" => "select",
        "parity_use_pack.qglang" => "meta",
        other => panic!("unmapped parity doc {other}"),
    }
}

const COMMAND_MARKERS_V1: &[CommandMarkerV1] = &[
    CommandMarkerV1 {
        command: "parse",
        marker_for_doc: |file_name, _doc| {
            format!("parse.root[0].kind={}", root_kind_for_doc_v1(file_name))
        },
    },
    CommandMarkerV1 {
        command: "sema",
        marker_for_doc: |file_name, _doc| {
            format!("sema.root[0].kind={}", root_kind_for_doc_v1(file_name))
        },
    },
    CommandMarkerV1 {
        command: "query",
        marker_for_doc: |file_name, _doc| {
            format!("query.root[0].ir.kind={}", root_kind_for_doc_v1(file_name))
        },
    },
    CommandMarkerV1 {
        command: "fmt",
        // fmt round-trips the source: the formatted output must carry it back.
        marker_for_doc: |_file_name, document| document.to_string(),
    },
    CommandMarkerV1 {
        command: "doctor",
        marker_for_doc: |_file_name, _doc| "doctor.status=no-result".to_string(),
    },
    CommandMarkerV1 {
        command: "headless",
        marker_for_doc: |_file_name, _doc| "headless.mode=read-only".to_string(),
    },
];

#[test]
fn inline_and_file_transports_are_byte_identical_and_non_vacuous_v1() {
    for (file_name, document) in PARITY_DOCS_V1 {
        let path = write_target_tmp_input_v1(file_name, document);
        let path_str = path.to_str().expect("utf8 path");
        let inline_tokens: Vec<&str> = document.split(' ').collect();

        for entry in COMMAND_MARKERS_V1 {
            let command = entry.command;
            let mut inline_argv = Vec::with_capacity(inline_tokens.len() + 1);
            inline_argv.push(command);
            inline_argv.extend(inline_tokens.iter().copied());

            let inline = run_cli_argv_v1(&inline_argv);
            let file = run_cli_argv_v1(&[command, "--file", path_str]);

            // Both transports must succeed cleanly.
            assert_eq!(
                inline.status.code(),
                Some(0),
                "{command} on `{document}`: inline must exit 0, stderr={}",
                cli_stderr_v1(&inline)
            );
            assert_eq!(
                file.status.code(),
                Some(0),
                "{command} on `{document}`: file must exit 0, stderr={}",
                cli_stderr_v1(&file)
            );
            assert!(
                cli_stderr_v1(&inline).is_empty(),
                "{command} on `{document}`: inline stderr must be empty, got {}",
                cli_stderr_v1(&inline)
            );
            assert!(
                cli_stderr_v1(&file).is_empty(),
                "{command} on `{document}`: file stderr must be empty, got {}",
                cli_stderr_v1(&file)
            );

            // Byte-for-byte transport parity on stdout.
            let inline_stdout = cli_stdout_v1(&inline);
            let file_stdout = cli_stdout_v1(&file);
            assert_eq!(
                inline_stdout, file_stdout,
                "{command} on `{document}`: stdout transport parity"
            );

            // Non-vacuous: the shared stdout must be a real success payload, not
            // two coincidentally-empty buffers.
            assert!(
                !inline_stdout.trim().is_empty(),
                "{command} on `{document}`: shared stdout must be non-empty"
            );
            let marker = (entry.marker_for_doc)(file_name, document);
            assert!(
                inline_stdout.contains(&marker),
                "{command} on `{document}`: shared stdout must carry success marker `{marker}`:\n{inline_stdout}"
            );
        }
    }
}
