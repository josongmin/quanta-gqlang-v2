#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Doc gate: the language README's diagnostics method list, the
//! `READ_ONLY_METHOD_NAMES_V1` const, and the real dispatch allowlist cannot
//! drift apart. This is the structural guard for the `explain`-omission the
//! audit caught (R7-fix had to add it by hand).

mod support;

use std::collections::BTreeSet;
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

const SUPPORTED_METHODS_START_V1: &str = "<!-- supported-diagnostics-methods:start -->";
const SUPPORTED_METHODS_END_V1: &str = "<!-- supported-diagnostics-methods:end -->";

fn documented_supported_methods_v1(document: &str) -> BTreeSet<String> {
    let (_, after_start) = document
        .split_once(SUPPORTED_METHODS_START_V1)
        .expect("missing supported diagnostics methods start marker");
    let (body, _) = after_start
        .split_once(SUPPORTED_METHODS_END_V1)
        .expect("missing supported diagnostics methods end marker");
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            Some(
                line.strip_prefix("- `")
                    .and_then(|value| value.strip_suffix('`'))
                    .unwrap_or_else(|| panic!("invalid supported-method entry `{line}`"))
                    .to_owned(),
            )
        })
        .collect()
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
    let expected_methods = READ_ONLY_METHOD_NAMES_V1
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented_supported_methods_v1(root_transport),
        expected_methods
    );
    assert_eq!(
        documented_supported_methods_v1(language_transport),
        expected_methods
    );
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

#[test]
fn supported_method_parser_detects_an_extra_advertised_name_v1() {
    let document = format!(
        "{SUPPORTED_METHODS_START_V1}\n- `v3_diagnostics`\n- `invented_method`\n{SUPPORTED_METHODS_END_V1}"
    );
    let documented = documented_supported_methods_v1(&document);
    assert!(documented.contains("invented_method"));
    assert_ne!(
        documented,
        READ_ONLY_METHOD_NAMES_V1
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn hellgate_ledger_references_only_current_repository_files_v1() {
    let root = repository_root_v1();
    let ledger_path = root.join("docs/language/test_hellgate_ledger.md");
    let ledger = read_document_v1(&ledger_path);
    let retired_package = ["quanta-gqlang-v2-", "lsp/"].concat();

    assert!(
        !ledger.contains(&retired_package),
        "hellgate ledger must not reference the deleted LSP package"
    );

    let mut referenced = BTreeSet::new();
    for fragment in ledger.split('`').skip(1).step_by(2) {
        if fragment.starts_with("crates/") && fragment.ends_with(".rs") {
            referenced.insert(fragment);
        }
    }
    assert!(
        !referenced.is_empty(),
        "hellgate ledger must contain repository rail paths"
    );
    for relative in referenced {
        let path = root.join(relative);
        assert!(
            path.is_file(),
            "hellgate ledger references missing rail {}",
            path.display()
        );
    }
}
