#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Documentation inventory guard.
//!
//! The hellgate ledger is the repo's curated index of test rails: it lists
//! concrete `crates/<crate>/tests/<file>.rs` paths. A stale path would
//! otherwise survive an ordinary source rename.
//!
//! This rail closes that class: every rail path named in the ledger must exist
//! on disk. It needs no crate API — it is a pure filesystem projection — so it
//! can live in any test target. It is hosted here because `surface` owns the
//! shipped example corpus and already carries the sibling corpus/doc-list rail
//! (`v3_examples_corpus.rs`). The assertion does not need a language API.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("surface crate lives at <repo>/crates/<crate>")
        .to_path_buf()
}

/// Extract every `crates/<crate>/tests/<file>.rs` path mentioned in `doc`.
fn rail_paths_v1(doc: &str) -> Vec<String> {
    let text = std::fs::read_to_string(repo_root().join(doc))
        .unwrap_or_else(|err| panic!("read {doc}: {err}"));
    let mut out: Vec<String> = Vec::new();
    for raw in text.split(|c: char| c.is_whitespace() || c == '`' || c == '(' || c == ')') {
        let token =
            raw.trim_matches(|c: char| matches!(c, ',' | ';' | '.' | ':' | '[' | ']' | '\'' | '"'));
        if token.starts_with("crates/") && token.contains("/tests/") && token.ends_with(".rs") {
            out.push(token.to_owned());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn assert_doc_paths_exist_v1(doc: &str) {
    let root = repo_root();
    let paths = rail_paths_v1(doc);
    assert!(
        !paths.is_empty(),
        "{doc}: expected this document to name at least one rail path"
    );
    let missing: Vec<&String> = paths
        .iter()
        .filter(|path| !root.join(path.as_str()).is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "{doc}: these rail paths no longer exist on disk (rename or delete them here too): {missing:?}"
    );
}

#[test]
fn hellgate_ledger_rail_paths_exist() {
    assert_doc_paths_exist_v1("docs/language/test_hellgate_ledger.md");
}
