#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "doc gate panics with rich context on command / usage / README drift"
)]

//! Doc gate: the language README's CLI command list, the `--help`/usage string
//! the binary emits, and the `CLI_COMMAND_TOKENS_V1` const cannot drift apart.
//! Structural guard for the `explain`-omission the audit caught.

mod support;

use std::fs;
use std::path::PathBuf;

use quanta_gqlang_v2_cli::CLI_COMMAND_TOKENS_V1;
use support::{cli_stderr_v1, run_cli_argv_v1};

fn language_readme_v1() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/language/README.md");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

#[test]
fn every_command_token_is_in_usage_and_documented_v1() {
    let readme = language_readme_v1();
    // No-args invocation fails closed with the usage string on stderr; that
    // string is the binary's own command list, so it ties the const to the
    // shipped binary.
    let usage = cli_stderr_v1(&run_cli_argv_v1(&[]));

    for token in CLI_COMMAND_TOKENS_V1 {
        assert!(
            usage.contains(token),
            "const command `{token}` is missing from the binary usage string:\n{usage}"
        );
        assert!(
            readme.contains(token),
            "language README does not list the CLI command `{token}`"
        );
    }
}
