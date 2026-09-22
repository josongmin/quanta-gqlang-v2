#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Shipped example corpus rail.
//!
//! `examples/**/*.qglang` is the frontend-owned corpus the docs call
//! "parser-stable". Before this rail it was referenced by **no test and no
//! `src` file at all**, and the three docs that list it had already drifted
//! (17 of 18 entries). This is the smallest owner-local rail that closes both
//! holes: `surface` owns the grammar, so it owns the guarantee that every
//! shipped example parses and formats to a fixed point, and that the corpus on
//! disk is exactly what the docs advertise.
//!
//! Scope is deliberately parse+format only. Whether an example compiles end to
//! end is a different claim owned by the pipeline/conformance rails, and one
//! example (`use_pack/secure_pack.qglang`) is schema-bound and fails closed
//! downstream by design.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use quanta_gqlang_v2_hir::HirBuilderV1;
use quanta_gqlang_v2_surface::{format_document, parse_document};

/// Repo root = this crate's manifest dir walked up out of `crates/<crate>`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("surface crate lives at <repo>/crates/<crate>")
        .to_path_buf()
}

/// Every `*.qglang` under `examples/`, as repo-relative slash paths, sorted.
fn corpus_paths() -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<String>, root: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out, root);
            } else if path.extension().is_some_and(|e| e == "qglang") {
                let rel = path
                    .strip_prefix(root)
                    .expect("corpus file is under the repo root")
                    .to_string_lossy()
                    .into_owned();
                out.push(rel);
            }
        }
    }
    let root = repo_root();
    let mut out = Vec::new();
    walk(&root.join("examples"), &mut out, &root);
    out.sort();
    out
}

/// Parse one example and return its canonical text.
fn canonical_text_v1(src: &str, path: &str) -> String {
    let builder = HirBuilderV1::new();
    let doc = parse_document(src, &builder)
        .unwrap_or_else(|diag| panic!("{path}: shipped example must parse, got {diag:?}"));
    let items: Vec<_> = doc
        .items
        .iter()
        .map(quanta_gqlang_v2_surface::SurfaceItemV1::to_program_item)
        .collect();
    format_document(&items)
}

#[test]
fn every_shipped_example_parses_and_round_trips() {
    let paths = corpus_paths();
    assert!(
        !paths.is_empty(),
        "the shipped example corpus must not be empty"
    );
    for path in &paths {
        let src = std::fs::read_to_string(repo_root().join(path))
            .unwrap_or_else(|err| panic!("read {path}: {err}"));
        let text = canonical_text_v1(&src, path);
        assert!(!text.is_empty(), "{path}: canonical text must not be empty");

        // Fixed point: the canonical text parses again and re-renders identically.
        let text2 = canonical_text_v1(&text, path);
        assert_eq!(
            text2, text,
            "{path}: format -> parse -> format must be a fixed point"
        );
    }
}

#[test]
fn every_example_is_listed_in_all_three_docs() {
    let on_disk: BTreeSet<String> = corpus_paths().into_iter().collect();
    let docs = [
        "README.md",
        "docs/language/README.md",
        "docs/language/examples_index.md",
    ];
    for doc in docs {
        let root = repo_root();
        let text = std::fs::read_to_string(root.join(doc))
            .unwrap_or_else(|err| panic!("read {doc}: {err}"));
        let listed: BTreeSet<String> = text
            .split(|c: char| c.is_whitespace() || c == '`' || c == ',' || c == '(' || c == ')')
            .filter(|tok| tok.starts_with("examples/") && tok.ends_with(".qglang"))
            .map(str::to_owned)
            .collect();

        let missing: Vec<&String> = on_disk.difference(&listed).collect();
        let dangling: Vec<&String> = listed.difference(&on_disk).collect();
        assert!(
            missing.is_empty(),
            "{doc}: shipped examples missing from this list: {missing:?}"
        );
        assert!(
            dangling.is_empty(),
            "{doc}: this list names examples that do not exist: {dangling:?}"
        );
    }
}

/// `parse_program` must agree with `parse_document` on a single-root example, so
/// the two public entry points cannot classify the corpus differently.
#[test]
fn single_root_examples_agree_across_entry_points() {
    let paths = corpus_paths();
    let mut checked = 0usize;
    for path in &paths {
        let src = std::fs::read_to_string(repo_root().join(path))
            .unwrap_or_else(|err| panic!("read {path}: {err}"));
        let builder = HirBuilderV1::new();
        let Ok(doc) = parse_document(&src, &builder) else {
            continue;
        };
        if doc.items.len() != 1 {
            continue;
        }
        let builder = HirBuilderV1::new();
        let single =
            quanta_gqlang_v2_surface::parse_program(&src, &builder).unwrap_or_else(|diag| {
                panic!("{path}: single-root example must parse via parse_program, got {diag:?}")
            });
        let doc_item = doc.items[0].to_program_item();
        let single_item = single.item.to_program_item();
        assert_eq!(
            format_document(&[single_item]),
            format_document(&[doc_item]),
            "{path}: parse_program and parse_document disagree on the single root"
        );
        checked += 1;
    }
    assert!(checked > 0, "expected at least one single-root example");
}
