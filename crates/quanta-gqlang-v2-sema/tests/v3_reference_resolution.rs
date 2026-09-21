#![expect(
    clippy::unwrap_used,
    reason = "owner assertions inspect exact sealed authority identities"
)]

use quanta_gqlang_v2_hir::{
    HirNodeIdV1, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    ValidatedModuleAuthorityPartsV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    ClosedFrontendAuthorityV2, FrontendBudgetV2, GraphReferenceCatalogV2, GraphReferenceInputV2,
    GraphReferenceKindV2, OpenAnalysisAuthorityInputV2, OpenAnalysisAuthorityV2,
    ResolvedBindingKindV2, ResolvedValueBindingV2,
};
use quanta_gqlang_v2_types::RelationCatalogV2;

fn host_registry(size: usize) -> SourceRegistryViewV2 {
    SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: size,
    }])
    .unwrap()
}

fn module_proof(
    host: &SourceRegistryViewV2,
    budget: &FrontendBudgetV2,
) -> ValidatedModuleAuthorityPartsV2 {
    let checked = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .unwrap();
    let current = ModuleIdV2::try_new("app", checked.clone()).unwrap();
    let mut builder =
        ModuleAuthorityBuilderV2::try_new(budget.authority_profile().unwrap(), host.clone())
            .unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: current.clone(),
            location: checked.clone(),
        })
        .unwrap();
    builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: current,
                location: checked,
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap()
}

fn relation_proof(host: &SourceRegistryViewV2, budget: &FrontendBudgetV2) -> RelationCatalogV2 {
    RelationCatalogV2::try_new(Vec::new(), host, budget.authority_profile().unwrap()).unwrap()
}

fn graph_proof(host: &SourceRegistryViewV2, budget: &FrontendBudgetV2) -> GraphReferenceCatalogV2 {
    GraphReferenceCatalogV2::try_new(Vec::new(), host, budget.authority_profile().unwrap()).unwrap()
}

#[test]
fn optional_authority_matrix_keeps_absence_explicit() {
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let host = host_registry(8);
    for (has_module, has_relation) in [(false, false), (true, false), (false, true), (true, true)] {
        let modules = has_module.then(|| module_proof(&host, &budget));
        let relations = has_relation.then(|| relation_proof(&host, &budget));
        let input = OpenAnalysisAuthorityInputV2::new(
            modules,
            relations,
            None,
            host.clone(),
            budget.clone(),
        );
        let authority = OpenAnalysisAuthorityV2::try_new(input).unwrap();
        assert_eq!(authority.modules().is_some(), has_module);
        assert_eq!(authority.relations().is_some(), has_relation);
        assert_eq!(authority.host_sources().identity(), host.identity());
    }
    let standard = OpenAnalysisAuthorityV2::try_standard_v2().unwrap();
    assert!(standard.modules().is_none());
    assert!(standard.relations().is_none());
    assert!(standard.graph_references().is_none());
}

#[test]
fn profile_and_registry_mismatch_reject_before_document_work() {
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let host = host_registry(8);
    let other = host_registry(9);
    let error = ClosedFrontendAuthorityV2::try_new(
        module_proof(&host, &budget),
        relation_proof(&other, &budget),
        graph_proof(&host, &budget),
        budget.clone(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "sema.source_registry_mismatch");
    let error = OpenAnalysisAuthorityV2::try_new(OpenAnalysisAuthorityInputV2::new(
        Some(module_proof(&host, &budget)),
        None,
        None,
        other.clone(),
        budget.clone(),
    ))
    .unwrap_err();
    assert_eq!(error.code(), "sema.open_authority_mismatch");
    let smaller = FrontendBudgetV2::try_new(
        1_048_576, 4_194_304, 131_072, 4_096, 64, 262_144, 4_096, 4_096, 1_000_000, 50_000,
        200_000, 1_000_000, 16_777_215, 65_536, 50_000, 200_000, 1_000_000, 1_024, 10_000, 1_024,
        8_388_608,
    )
    .unwrap();
    let error = ClosedFrontendAuthorityV2::try_new(
        module_proof(&host, &budget),
        relation_proof(&host, &budget),
        graph_proof(&host, &budget),
        smaller,
    )
    .unwrap_err();
    assert_eq!(error.code(), "sema.authority_budget_mismatch");
}

#[test]
fn source_qualified_binder_ordinal_is_part_of_semantic_identity() {
    let source = SourceIdV2::from_raw(7);
    let owner = HirNodeIdV1::from_raw(8);
    let first =
        ResolvedValueBindingV2::try_new(source, owner, 0, ResolvedBindingKindV2::Quantified, "x")
            .unwrap();
    let second =
        ResolvedValueBindingV2::try_new(source, owner, 1, ResolvedBindingKindV2::Quantified, "x")
            .unwrap();
    assert_ne!(first, second);
    assert_eq!((first.target_node(), first.ordinal()), (owner, 0));
    assert_eq!((second.target_node(), second.ordinal()), (owner, 1));
}

#[test]
fn graph_selector_preserves_punctuation_kind_subtype_and_host_location() {
    let host = host_registry(32);
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let host_loc = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(2, 8),
        })
        .unwrap();
    let input = GraphReferenceInputV2 {
        spelling: "repo::a/b.c".into(),
        kind: GraphReferenceKindV2::Entity,
        entity_subtype: Some("service".into()),
        location: host_loc.clone(),
    };
    let catalog = GraphReferenceCatalogV2::try_new(
        vec![input.clone()],
        &host,
        budget.authority_profile().unwrap(),
    )
    .unwrap();
    let use_site = LocatedSpanV2 {
        source_id: SourceIdV2::from_raw(9),
        span: SpanV1::new(10, 21),
    };
    let resolved = catalog
        .resolve_entity_subtype("repo::a/b.c", "service", use_site)
        .unwrap();
    assert_eq!(resolved.spelling(), "repo::a/b.c");
    assert_eq!(resolved.kind(), GraphReferenceKindV2::Entity);
    assert_eq!(resolved.entity_subtype(), Some("service"));
    assert_eq!(resolved.declaration_location(), &host_loc);
    assert!(catalog.admits(&resolved));
    let foreign_host = host_registry(33);
    let foreign_location = foreign_host
        .try_check_location(host_loc.location())
        .unwrap();
    let foreign = GraphReferenceCatalogV2::try_new(
        vec![GraphReferenceInputV2 {
            location: foreign_location,
            ..input.clone()
        }],
        &foreign_host,
        budget.authority_profile().unwrap(),
    )
    .unwrap();
    assert!(!foreign.admits(&resolved));
    let foreign_resolved = foreign
        .resolve("repo::a/b.c", GraphReferenceKindV2::Entity, use_site)
        .unwrap();
    assert!(!catalog.admits(&foreign_resolved));
    let wrong = catalog
        .resolve("repo::a/b.c", GraphReferenceKindV2::Scope, use_site)
        .unwrap_err();
    assert_eq!(
        wrong.diagnostics().first().unwrap().code(),
        "sema.wrong_reference_kind"
    );
    assert_eq!(wrong.diagnostics().first().unwrap().primary(), use_site);
    assert_eq!(
        wrong.diagnostics().first().unwrap().secondary(),
        &[host_loc.location()]
    );
    assert_eq!(
        catalog
            .resolve_entity_subtype("repo::a/b.c", "package", use_site)
            .unwrap_err()
            .diagnostics()
            .first()
            .unwrap()
            .code(),
        "sema.wrong_reference_kind"
    );
    let missing = catalog
        .resolve("repo::a/b", GraphReferenceKindV2::Entity, use_site)
        .unwrap_err();
    assert_eq!(
        missing.diagnostics().first().unwrap().code(),
        "sema.unresolved_reference"
    );
    let duplicate = GraphReferenceCatalogV2::try_new(
        vec![input.clone(), input],
        &host,
        budget.authority_profile().unwrap(),
    )
    .unwrap_err();
    assert_eq!(duplicate.code(), "sema.ambiguous_reference");
    assert_eq!(duplicate.primary(), Some(host_loc.location()));
    assert_eq!(duplicate.secondary(), Some(host_loc.location()));
}
