#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "conformance gate panics with rich context on registry/corpus drift"
)]

//! Ties the CLI test corpus to the conformance registry so the hand-maintained
//! `SHIPPED_EXAMPLE_DOC_PATHS_V1` list can never silently drift from the single
//! source of truth. The registry covers 18 front-door families (17 roots + the
//! clause-level composition family); the shipped 17-root example corpus is
//! exactly the registry's root example paths (composition is excluded from the
//! 17-root corpus by the E03C taxonomy decision and shipped via its own rails).

mod support;

use quanta_gqlang_v2_conformance::{root_surface_specs_v1, RootFamilyV1};
use support::SHIPPED_EXAMPLE_DOC_PATHS_V1;

#[test]
fn shipped_corpus_equals_registry_root_example_paths_v1() {
    let mut registry_root_paths: Vec<&str> = root_surface_specs_v1()
        .iter()
        .filter(|spec| spec.family != RootFamilyV1::Composition)
        .map(|spec| spec.example_path)
        .collect();
    registry_root_paths.sort_unstable();

    let mut shipped: Vec<&str> = SHIPPED_EXAMPLE_DOC_PATHS_V1.to_vec();
    shipped.sort_unstable();

    assert_eq!(
        shipped, registry_root_paths,
        "SHIPPED_EXAMPLE_DOC_PATHS_V1 drifted from the conformance registry's root \
         example paths — keep the registry as the single source of truth"
    );
}
