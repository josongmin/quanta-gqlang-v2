#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "conformance gate panics with rich context on any registry/doc drift"
)]

//! Doc-conformance gate: the language README must stay a faithful mirror of the
//! conformance registry (the single source of truth for the root families).
//!
//! For every registry family this pins that the language README lists both the
//! family's keyword and its shipped example path. So a new root family added to
//! the registry cannot silently skip documentation — adding it makes this gate
//! fail until the README lists it too. (The CLI-command / LSP-method doc gates
//! belong in the cli test crate, which may legally reach both surfaces; this
//! gate only needs to read a doc file, so it has no crate dependency on them.)

use std::fs;
use std::path::PathBuf;

use quanta_gqlang_v2_conformance::root_surface_specs_v1;

fn language_readme_v1() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/language/README.md");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

#[test]
fn language_readme_lists_every_registry_family_keyword_and_example_v1() {
    let readme = language_readme_v1();
    for spec in root_surface_specs_v1() {
        assert!(
            readme.contains(spec.keyword),
            "{:?}: language README is missing the family keyword `{}`",
            spec.family,
            spec.keyword
        );
        assert!(
            readme.contains(spec.example_path),
            "{:?}: language README is missing the shipped example path `{}`",
            spec.family,
            spec.example_path
        );
    }
}
