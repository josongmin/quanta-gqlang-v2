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

//! Mutation-kill rails for the conformance registry's NON-VACUITY.
//!
//! The self-validation rails iterate `root_family_specs_v1()` /
//! `supplemental_surface_unit_specs_v1()`; if either function returned an empty
//! vec, those loops would run zero times and pass vacuously — the parity guard
//! would silently cover NOTHING. These rails pin the spec counts and the exact
//! `FamilyKindV1::keyword` strings so emptying a registry fn (or blanking a
//! keyword) fails closed.

use quanta_gqlang_v2_conformance::{
    root_family_specs_v1, supplemental_surface_unit_specs_v1, FamilyKindV1, ROOT_FAMILIES_V1,
    SUPPLEMENTAL_SURFACE_UNITS_V1,
};

#[test]
fn root_family_specs_cover_every_family_non_vacuously() {
    let specs = root_family_specs_v1();
    assert_eq!(
        specs.len(),
        ROOT_FAMILIES_V1.len(),
        "root_family_specs_v1 must emit one spec per family, not an empty/short list"
    );
    assert_eq!(
        specs.len(),
        18,
        "the v3 parity registry covers exactly 18 families"
    );
}

#[test]
fn supplemental_surface_unit_specs_are_non_vacuous() {
    let specs = supplemental_surface_unit_specs_v1();
    assert_eq!(
        specs.len(),
        SUPPLEMENTAL_SURFACE_UNITS_V1.len(),
        "supplemental_surface_unit_specs_v1 must emit one spec per supplemental unit"
    );
    assert!(
        !specs.is_empty(),
        "the supplemental surface-unit registry must not be empty"
    );
}

#[test]
fn family_kind_keywords_are_exact_and_nonempty() {
    let cases = [
        (FamilyKindV1::RelationalQuery, "relational_query"),
        (FamilyKindV1::ValueArtifact, "value_artifact"),
        (FamilyKindV1::GraphQuery, "graph_query"),
        (FamilyKindV1::Legality, "legality"),
        (FamilyKindV1::Presentation, "presentation"),
        (FamilyKindV1::RuleArtifact, "rule_artifact"),
        (FamilyKindV1::CatalogRegistration, "catalog_registration"),
        (FamilyKindV1::Composition, "composition"),
        (FamilyKindV1::Meta, "meta"),
    ];
    for (kind, expected) in cases {
        let got = kind.keyword();
        assert!(!got.is_empty(), "{kind:?} keyword must not be empty");
        assert_eq!(got, expected, "{kind:?} keyword drifted");
    }
}
