#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "conformance parity guard intentionally panics with rich context on any registry/pipeline drift"
)]

//! v3 PARITY GUARD hellgate for the family registry.
//!
//! This is the rail that PROVES the one v3 compiler
//! ([`quanta_gqlang_v2_pipeline::compile_program`]) covers ALL 18 DSL root
//! families (the 17 root families plus the LET/WITH composition family). It
//! drives every spec's representative v3 surface example through the real
//! `surface parse -> bind -> type -> algebra/logic -> plan` pipeline and proves:
//!   1. the registry covers each family exactly once (no drop, no duplicate);
//!   2. every example compiles end to end and lands on the declared
//!      [`FamilyKindV1`] (compiled-item kind), with the declared `result_kind` /
//!      `effect` for plan-producing families;
//!   3. an unsupported construct still fails closed (no silent classification).

use std::collections::BTreeSet;

use quanta_gqlang_v2_conformance::{
    root_family_specs_v1, spec_for_v1, supplemental_surface_unit_specs_v1, FamilyKindV1,
    FamilyOutcomeV1, PlanOutcomeV1, ROOT_FAMILIES_V1, SUPPLEMENTAL_SURFACE_UNITS_V1,
    UNSUPPORTED_CONSTRUCT_V1,
};
use quanta_gqlang_v2_pipeline::{compile_program, CompiledItemV1, CompiledProgramV1};

#[test]
fn registry_covers_each_family_exactly_once_v1() {
    assert_eq!(ROOT_FAMILIES_V1.len(), 18, "expected 18 root families");

    let unique: BTreeSet<_> = ROOT_FAMILIES_V1
        .iter()
        .map(|family| format!("{family:?}"))
        .collect();
    assert_eq!(
        unique.len(),
        ROOT_FAMILIES_V1.len(),
        "ROOT_FAMILIES_V1 has a duplicate variant"
    );

    for &family in ROOT_FAMILIES_V1 {
        assert_eq!(
            spec_for_v1(family).family,
            family,
            "spec_for_v1 must echo the requested family"
        );
    }
}

#[test]
fn keyword_leads_every_example_v1() {
    // Pin that `keyword` is the real leading token of the surface example, so a
    // wrong keyword (e.g. a typo) cannot sit unvalidated in the registry.
    for spec in root_family_specs_v1() {
        assert!(
            spec.example
                .trim_start()
                .to_ascii_lowercase()
                .starts_with(spec.keyword),
            "{:?}: example `{}` does not lead with keyword `{}`",
            spec.family,
            spec.example,
            spec.keyword
        );
    }
}

#[test]
fn every_family_example_matches_declared_v3_outcome_v1() {
    for spec in root_family_specs_v1() {
        match spec.outcome {
            FamilyOutcomeV1::Compiles { kind, plan } => {
                let compiled = compile_program(spec.example).unwrap_or_else(|diag| {
                    panic!(
                        "{:?}: example `{}` must compile end to end through the v3 pipeline, got {diag:?}",
                        spec.family, spec.example
                    )
                });

                let actual_kind = family_kind_of(compiled.compiled());
                assert_eq!(
                    actual_kind,
                    kind,
                    "{:?}: example `{}` compiled to kind `{}`, registry declares `{}`",
                    spec.family,
                    spec.example,
                    actual_kind.keyword(),
                    kind.keyword()
                );

                assert_plan_outcome(spec.family, plan, &compiled);
            }
            FamilyOutcomeV1::FailsClosed { code } => {
                let diag = compile_program(spec.example).expect_err(&format!(
                    "{:?}: example `{}` must fail closed at its routed stage",
                    spec.family, spec.example
                ));
                assert_eq!(
                    diag.code(),
                    code,
                    "{:?}: example `{}`: fail-closed code drifted from the registry",
                    spec.family,
                    spec.example
                );
            }
        }
    }
}

#[test]
fn unsupported_construct_fails_closed_v1() {
    // The parity guard is only honest if an UNSUPPORTED construct does not slip
    // through as some family: the v3 pipeline must reject it fail-closed.
    let result = compile_program(UNSUPPORTED_CONSTRUCT_V1);
    assert!(
        result.is_err(),
        "an unsupported construct must fail closed, but it compiled to {:?}",
        result.ok().map(|c| family_kind_of(c.compiled()))
    );
}

#[test]
fn supplemental_surface_units_are_pinned_separately_v1() {
    assert_eq!(
        SUPPLEMENTAL_SURFACE_UNITS_V1.len(),
        2,
        "expected the standalone decl + horn-rule units to stay pinned explicitly"
    );

    for spec in supplemental_surface_unit_specs_v1() {
        assert!(
            spec.example
                .trim_start()
                .to_ascii_lowercase()
                .starts_with(spec.keyword),
            "{:?}: example `{}` does not lead with keyword `{}`",
            spec.unit,
            spec.example,
            spec.keyword
        );

        let compiled = compile_program(spec.example).unwrap_or_else(|diag| {
            panic!(
                "{:?}: example `{}` must compile to an explicit non-servable artifact, got {diag:?}",
                spec.unit, spec.example
            )
        });
        assert_eq!(
            family_kind_of(compiled.compiled()),
            spec.outcome.kind,
            "{:?}: compiled-item kind drifted from the pinned public-pipeline truth",
            spec.unit
        );
        assert_eq!(
            compiled.plan().is_some(),
            spec.outcome.has_plan,
            "{:?}: no-plan/plan boundary drifted from the pinned public-pipeline truth",
            spec.unit
        );
    }
}

/// Assert the compiled program's plan outcome matches the registry: for a
/// plan-producing family the `result_kind`/`effect` must match; for a no-plan
/// family there must be no plan.
fn assert_plan_outcome(
    family: quanta_gqlang_v2_conformance::RootFamilyV1,
    expected: Option<PlanOutcomeV1>,
    compiled: &CompiledProgramV1,
) {
    match (expected, compiled.plan()) {
        (Some(expected), Some(plan)) => {
            assert_eq!(
                plan.result_kind, expected.result_kind,
                "{family:?}: result_kind drifted from the registry"
            );
            assert_eq!(
                plan.effect, expected.effect,
                "{family:?}: effect drifted from the registry"
            );
        }
        (Some(_), None) => {
            panic!("{family:?}: registry declares a plan but the compile produced none")
        }
        (None, Some(_)) => {
            panic!("{family:?}: registry declares no plan but the compile produced one")
        }
        (None, None) => {}
    }
}

/// Map a compiled item onto its registry [`FamilyKindV1`] discriminant.
fn family_kind_of(item: &CompiledItemV1) -> FamilyKindV1 {
    match item {
        CompiledItemV1::RelationalQuery { .. } => FamilyKindV1::RelationalQuery,
        CompiledItemV1::ValueArtifact { .. } => FamilyKindV1::ValueArtifact,
        CompiledItemV1::GraphQuery { .. } => FamilyKindV1::GraphQuery,
        CompiledItemV1::Legality { .. } => FamilyKindV1::Legality,
        CompiledItemV1::Presentation { .. } => FamilyKindV1::Presentation,
        CompiledItemV1::RuleArtifact { .. } => FamilyKindV1::RuleArtifact,
        CompiledItemV1::CatalogRegistration { .. } => FamilyKindV1::CatalogRegistration,
        CompiledItemV1::Composition { .. } => FamilyKindV1::Composition,
        CompiledItemV1::Meta { .. } => FamilyKindV1::Meta,
    }
}
