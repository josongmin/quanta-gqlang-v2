#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "doc gate panics with rich context on method-allowlist / README drift"
)]

//! Doc gate: the language README's diagnostics method list, the
//! `READ_ONLY_METHOD_NAMES_V1` const, and the real dispatch allowlist cannot
//! drift apart. This is the structural guard for the `explain`-omission the
//! audit caught (R7-fix had to add it by hand).

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use quanta_gqlang_v2_diagnostics_stdio::server::READ_ONLY_METHOD_NAMES_V1;
use support::run_binary_line_v1;

fn repository_root_v1() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_document_v1(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn documented_transport_section_v1<'a>(document: &'a str, heading: &str) -> &'a str {
    let start = document
        .find(heading)
        .unwrap_or_else(|| panic!("missing transport heading `{heading}`"));
    let tail = document.get(start + heading.len()..).expect("heading end");
    let end = tail.find("\n## ").unwrap_or(tail.len());
    tail.get(..end).expect("section bounds")
}

#[test]
fn every_allowlisted_method_is_dispatched_and_documented_v1() {
    assert_eq!(READ_ONLY_METHOD_NAMES_V1, ["v3_diagnostics"]);
    let root = repository_root_v1();
    let root_readme = read_document_v1(&root.join("README.md"));
    let language_readme = read_document_v1(&root.join("docs/language/README.md"));
    let root_transport =
        documented_transport_section_v1(&root_readme, "Diagnostics stdio front door:");
    let language_transport =
        documented_transport_section_v1(&language_readme, "## CLI And Diagnostics Stdio");
    let retired_package = ["quanta-gqlang-v2-", "lsp"].concat();
    for document in [&root_readme, &language_readme] {
        assert!(
            document.contains("quanta-gqlang-v2-diagnostics-stdio"),
            "documentation must name the diagnostics-stdio package"
        );
        assert!(
            !document.contains(&retired_package),
            "documentation must not retain the deleted LSP package identity"
        );
    }
    for section in [root_transport, language_transport] {
        assert!(
            section.contains("v3_diagnostics"),
            "transport section must list v3_diagnostics"
        );
        assert!(
            !section.contains("LSP front door") && !section.contains("\nLSP:"),
            "transport section must use diagnostics-stdio identity"
        );
        assert!(
            section.contains("private flat-JSON protocol")
                && section.contains("not JSON-RPC or LSP")
                && section.contains("does not implement")
                && section.contains("`initialize`"),
            "transport section must explicitly delimit the non-LSP protocol"
        );
    }
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

        assert!(root_transport.contains(method));
        assert!(language_transport.contains(method));
    }
}
