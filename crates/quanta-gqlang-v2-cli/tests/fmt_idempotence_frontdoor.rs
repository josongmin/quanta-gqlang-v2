#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! `fmt` front-door idempotence.
//!
//! Proves the `fmt` command is a deterministic fixed point through the real
//! binary: formatting an already-formatted document is a no-op, and formatting
//! a messy-but-valid document converges to a canonical form that is itself a
//! fixed point. The canonical form is *discovered* by running the binary, not
//! guessed — the asserted property is idempotence (`fmt(fmt(x)) == fmt(x)`),
//! plus the invariant that the shipped examples are already in canonical form.

mod support;

use std::fs;
use std::path::PathBuf;

use support::{
    cli_stdout_v1, example_path_v1, example_source_v1, run_cli_argv_v1,
    SHIPPED_EXAMPLE_DOC_PATHS_V1,
};

/// Run `fmt --file <path>` through the real binary, assert exit 0, return stdout.
fn fmt_file_stdout_v1(path: &str) -> String {
    let output = run_cli_argv_v1(&["fmt", "--file", path]);
    assert!(
        output.status.success(),
        "fmt --file `{path}` failed (status={:?})\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    cli_stdout_v1(&output)
}

/// Run `fmt <inline...>` through the real binary, assert exit 0, return stdout.
fn fmt_inline_stdout_v1(inline: &str) -> String {
    let output = run_cli_argv_v1(&["fmt", inline]);
    assert!(
        output.status.success(),
        "fmt inline `{inline}` failed (status={:?})\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    cli_stdout_v1(&output)
}

/// Materialize `contents` into a uniquely-named temp file under the integration
/// test temp dir, so an in-memory `fmt` result can be fed back through the
/// `--file` front door.
fn write_tmp_v1(name: &str, contents: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, contents).expect("write temp fmt input");
    path
}

/// For every shipped example: prove `fmt` is idempotent and that the shipped
/// source is already canonical.
#[test]
fn fmt_is_fixed_point_on_shipped_examples_v1() {
    for (index, example_relative) in SHIPPED_EXAMPLE_DOC_PATHS_V1.iter().enumerate() {
        let path = example_path_v1(example_relative);
        let path_str = path.to_str().expect("example path is valid UTF-8");

        // First pass: format the shipped example.
        let out1 = fmt_file_stdout_v1(path_str);
        assert!(
            !out1.trim().is_empty(),
            "fmt produced empty output for `{example_relative}`"
        );

        // Canonical-equivalence: the shipped source is already formatted.
        let source = example_source_v1(example_relative);
        assert_eq!(
            out1.trim(),
            source.trim(),
            "fmt front door drifted from canonical for `{example_relative}`"
        );

        // Second pass: feed out1 back through fmt via a temp file.
        let tmp = write_tmp_v1(&format!("fmt_idem_example_{index}.qglang"), &out1);
        let tmp_str = tmp.to_str().expect("tmp path is valid UTF-8");
        let out2 = fmt_file_stdout_v1(tmp_str);

        assert_eq!(
            out1, out2,
            "fmt is not idempotent for `{example_relative}`:\n--- out1 ---\n{out1}\n--- out2 ---\n{out2}"
        );
    }
}

/// A valid-but-messily-spaced input paired with a descriptive label, used as a
/// non-canonical control to prove `fmt` converges to a fixed point.
struct MessyInputV1 {
    label: &'static str,
    source: &'static str,
}

/// Non-canonical control: at least two valid inputs with runs of extra spaces.
/// We don't assert a guessed canonical; we assert the *property* — `fmt` is an
/// idempotent fixed point and produces non-empty output with exit 0.
#[test]
fn fmt_converges_on_messy_valid_inputs_v1() {
    let messy_inputs = [
        MessyInputV1 {
            label: "messy-select",
            source: "SELECT    id,     profile.name    FROM     catalog.users   \
                     WHERE   active   =   true   LIMIT   7",
        },
        MessyInputV1 {
            label: "messy-use-pack",
            source: "USE    PACK    secure-pack   ;;   activation=session   ;;   \
                     lockfile=packs/secure-pack.lock",
        },
    ];

    for messy in messy_inputs {
        // First pass over the messy input.
        let f1 = fmt_inline_stdout_v1(messy.source);
        assert!(
            !f1.trim().is_empty(),
            "fmt produced empty output for messy input `{}`",
            messy.label
        );

        // Re-fmt the first-pass output as an inline argv: it must be a fixed
        // point. We trim the trailing newline the binary appends to println!
        // output so the inline argv carries just the canonical document.
        let f1_inline = f1.trim_end_matches('\n');
        let f2 = fmt_inline_stdout_v1(f1_inline);

        assert_eq!(
            f1, f2,
            "fmt is not a fixed point for messy input `{}`:\n--- f1 ---\n{f1}\n--- f2 ---\n{f2}",
            messy.label
        );
    }
}
