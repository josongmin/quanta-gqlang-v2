#![forbid(unsafe_code)]

//! Non-vacuity rails for the facade-only conformance registry.

use quanta_gqlang_v2_conformance::{
    root_family_specs_v2, ROOT_FAMILIES_V2, SUPPLEMENTAL_SOURCES_V2,
};

#[test]
fn root_family_specs_cover_every_family_non_vacuously() {
    let specs = root_family_specs_v2().collect::<Vec<_>>();
    assert_eq!(specs.len(), ROOT_FAMILIES_V2.len());
    assert_eq!(specs.len(), 18);
    assert!(specs.iter().all(|spec| !spec.keyword.is_empty()));
    assert!(specs.iter().all(|spec| !spec.source.is_empty()));
}

#[test]
fn root_family_keywords_are_unique() {
    let mut keywords = root_family_specs_v2()
        .map(|spec| spec.keyword)
        .collect::<Vec<_>>();
    keywords.sort_unstable();
    keywords.dedup();
    assert_eq!(keywords.len(), ROOT_FAMILIES_V2.len());
}

#[test]
fn supplemental_sources_are_non_vacuous() {
    assert_eq!(SUPPLEMENTAL_SOURCES_V2.len(), 2);
    assert!(SUPPLEMENTAL_SOURCES_V2
        .iter()
        .all(|source| !source.is_empty()));
}
