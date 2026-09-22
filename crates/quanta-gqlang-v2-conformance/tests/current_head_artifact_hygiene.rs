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

//! Ticket 16 hellgate: current-main artifact hygiene.
//!
//! A clean v3 workspace keeps Rust sources under `crates/<crate>/src` (or
//! `tests`) and compiled artifacts under `target/`. The repo ROOT must therefore
//! carry NO loose `.rs` source files and NO `.rlib` (or other compiled object)
//! artifacts. Scratch repro files (`test_*.rs`, `test_*.txt`) and stray
//! `libfoo.rlib` blobs committed at the root are exactly the hygiene drift
//! ticket 16 closes; this rail fails closed if any reappear, so the boundary
//! stays enforced rather than relying on a one-time manual sweep.

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
        // loose shell/scratch scripts at the root are repro debris too.
        let is_loose_script = name.ends_with(".sh") || name.ends_with(".bash");
        // scratch repro NOTES (`test_*.txt`) are the same debris class as
        // `test_*.rs`: the extension differs, the intent does not. Scoped to the
        // `test_` prefix so legitimate root text files stay allowed.
        let is_scratch_note = name.starts_with("test_") && name.ends_with(".txt");
        // editor / agent backup files never belong in the tree, anywhere.
        let is_backup = name.ends_with(".bak") || name.ends_with(".orig") || name.ends_with('~');
        if is_loose_source
            || is_compiled_artifact
            || is_loose_script
            || is_scratch_note
            || is_backup
        {
            offenders.push(name);
        }
    }

    // A backup file (`.bak` / `.orig` / `~`) is debris wherever it lands, so sweep
    // every crate's src+tests too, not just the repo root.
    for crate_dir in std::fs::read_dir(root.join("crates"))
        .into_iter()
        .flatten()
        .flatten()
    {
        for sub in ["src", "tests"] {
            let dir = crate_dir.path().join(sub);
            let Ok(walk) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in walk.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".bak") || name.ends_with(".orig") || name.ends_with('~') {
                    offenders.push(format!(
                        "crates/{}/{sub}/{name}",
                        crate_dir.file_name().to_string_lossy()
                    ));
                }
            }
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
