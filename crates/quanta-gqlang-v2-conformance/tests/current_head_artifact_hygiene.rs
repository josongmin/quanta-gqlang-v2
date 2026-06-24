#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "artifact-hygiene guard intentionally fails hard, listing every offending repo-root artifact"
)]

//! Ticket 16 hellgate: current-main artifact hygiene.
//!
//! A clean v3 workspace keeps Rust sources under `crates/<crate>/src` (or
//! `tests`) and compiled artifacts under `target/`. The repo ROOT must therefore
//! carry NO loose `.rs` source files and NO `.rlib` (or other compiled object)
//! artifacts. Scratch repro files (`test_*.rs`) and stray `libfoo.rlib` blobs
//! committed at the root are exactly the hygiene drift ticket 16 closes; this
//! rail fails closed if any reappear, so the boundary stays enforced rather than
//! relying on a one-time manual sweep.

use std::path::{Path, PathBuf};

/// Repo root = this crate's manifest dir walked up out of `crates/<crate>`.
fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent() // crates/
        .and_then(Path::parent) // repo root
        .expect("conformance crate lives at <repo>/crates/<crate>")
        .to_path_buf()
}

#[test]
fn repo_root_carries_no_loose_source_or_compiled_artifacts() {
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(&root).expect("repo root is readable");
    for entry in entries {
        let entry = entry.expect("dir entry readable");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_loose_source = name.ends_with(".rs");
        let is_compiled_artifact = name.ends_with(".rlib")
            || name.ends_with(".rmeta")
            || name.ends_with(".o")
            || name.ends_with(".d");
        if is_loose_source || is_compiled_artifact {
            offenders.push(name);
        }
    }

    offenders.sort();
    assert!(
        offenders.is_empty(),
        "repo-root artifact hygiene violated: stray source/compiled artifacts at \
         {} must live under crates/<crate> or target/, not the repo root: {:?}",
        root.display(),
        offenders
    );
}
