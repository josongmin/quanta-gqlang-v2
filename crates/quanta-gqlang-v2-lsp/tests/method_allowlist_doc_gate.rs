#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "doc gate panics with rich context on method-allowlist / README drift"
)]

//! Doc gate: the language README's LSP method list, the
//! `READ_ONLY_METHOD_NAMES_V1` const, and the real dispatch allowlist cannot
//! drift apart. This is the structural guard for the `explain`-omission the
//! audit caught (R7-fix had to add it by hand).

mod support;

use std::fs;
use std::path::PathBuf;

use quanta_gqlang_v2_lsp::server::READ_ONLY_METHOD_NAMES_V1;
use support::run_binary_line_v1;

fn language_readme_v1() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/language/README.md");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

#[test]
fn every_allowlisted_method_is_dispatched_and_documented_v1() {
    let readme = language_readme_v1();
    for method in READ_ONLY_METHOD_NAMES_V1 {
        // const <-> dispatch: a real allowlisted method must NOT fall through to
        // `method_not_allowed` (a missing required field yields `missing_field`,
        // not the not-allowed code — still proof the method is recognized).
        let request = format!(r#"{{"method":"{method}"}}"#);
        let (_code, line) = run_binary_line_v1(&request);
        assert!(
            !line.contains("method_not_allowed"),
            "`{method}` is in the allowlist const but the dispatch rejected it as \
             not-allowed: {line}"
        );

        // const <-> README: every allowlisted method must be documented.
        assert!(
            readme.contains(method),
            "language README does not list the allowlisted LSP method `{method}`"
        );
    }
}
