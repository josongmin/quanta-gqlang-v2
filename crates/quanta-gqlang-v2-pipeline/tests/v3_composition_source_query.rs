#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "cross-owner integration assertions inspect exact sealed contracts"
)]

use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2, ExternalCatalogPayloadInputV2,
    ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CardinalityV1, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    FrontendBudgetV2, GraphReferenceCatalogV2, GraphReferenceInputV2, GraphReferenceKindV2,
    LocatedSpanV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2,
    RelationSourceInputV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    SemanticItemV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1, ValueTypeV1,
};
use quanta_gqlang_v2_surface::{parse_source_document_v2, ParseBudgetV2};

fn authority() -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 16,
    }])
    .unwrap();
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .unwrap();
    let module = ModuleIdV2::try_new("app", location.clone()).unwrap();
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let profile = budget.authority_profile().unwrap();
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).unwrap();
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .unwrap();
    for (local_name, kind_token, typed_payload) in [
        ("check1", "check", ExternalCatalogPayloadInputV2::Check),
        (
            "capability1",
            "capability",
            ExternalCatalogPayloadInputV2::Capability,
        ),
    ] {
        modules
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".into(),
                local_name: local_name.into(),
                kind_token: kind_token.into(),
                typed_payload,
                provenance: "host".into(),
                location: location.clone(),
                dependencies: Vec::new(),
            })
            .unwrap();
    }
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let relation_inputs = ["records", "later", "graph_rows"]
        .into_iter()
        .map(|name| RelationSourceInputV2 {
            provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone()).unwrap(),
            qualified_name: format!("app::{name}"),
            lookup_name: name.into(),
            row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                "id",
                ValueTypeV1::int(),
                location.clone(),
            )
            .unwrap()])
            .unwrap(),
            cardinality: CardinalityV1::Bag,
            provenance: "host".into(),
            location: location.clone(),
        })
        .collect();
    let relations = RelationCatalogV2::try_new(relation_inputs, &host, profile.clone()).unwrap();
    let graphs = GraphReferenceCatalogV2::try_new(
        [
            ("scope", GraphReferenceKindV2::Scope),
            ("a", GraphReferenceKindV2::Entity),
            ("b", GraphReferenceKindV2::Entity),
        ]
        .into_iter()
        .map(|(spelling, kind)| GraphReferenceInputV2 {
            spelling: spelling.into(),
            kind,
            entity_subtype: None,
            location: location.clone(),
        })
        .collect(),
        &host,
        profile,
    )
    .unwrap();
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget).unwrap()
}

fn resolve(source: &str) -> quanta_gqlang_v2_sema::SemanticDocumentV2 {
    let parsed = parse_source_document_v2(
        SourceIdV2::from_raw(9),
        source,
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap();
    resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).unwrap(),
        &authority(),
    )
    .unwrap()
}

fn composition(
    document: &quanta_gqlang_v2_sema::SemanticDocumentV2,
) -> &quanta_gqlang_v2_sema::ResolvedCompositionV2 {
    document
        .items()
        .find_map(|(_, item)| match item {
            SemanticItemV2::Composition(composition) => Some(composition),
            SemanticItemV2::Relation(_)
            | SemanticItemV2::Rule(_)
            | SemanticItemV2::Graph(_)
            | SemanticItemV2::Check(_)
            | SemanticItemV2::Presentation(_)
            | SemanticItemV2::Catalog(_)
            | SemanticItemV2::Declaration(_)
            | SemanticItemV2::Meta(_) => None,
        })
        .unwrap()
}

#[test]
fn surface_to_hir_to_sema_materializes_a_sample_composition_body() {
    let document = resolve(
        "let sampled = sample query=\"auth\" | mode=lexical | limit=2 \
         from sampled s | project s.path as path with sampled",
    );
    let composition = composition(&document);
    let ResolvedCompositionProducerV2::Sample { output, .. } =
        composition.bindings().first().unwrap().producer()
    else {
        panic!("Surface SAMPLE must retain the closed producer contract");
    };
    assert_eq!(output.row().fields().first().unwrap().name(), "path");
    let ResolvedCompositionBodyV2::Query(body) = composition.body() else {
        panic!("Surface Composition body must reach typed Query materialization");
    };
    assert_eq!(body.output().row(), output.row());
}

#[test]
fn surface_to_hir_to_sema_reaches_every_composition_body_family() {
    for (source, expected) in [
        (
            "let rows = from records r from rows x | project x.id as id with rows",
            "query",
        ),
        (
            "let sampled = sample query=\"auth\" | mode=lexical | limit=2 from sampled s | project s.path as path with sampled",
            "sample-query",
        ),
        (
            "let paths = find path a -> b | depth=1 find flow paths -> b | depth=1 with paths",
            "graph",
        ),
        (
            "ruledef rule1 | check-ref=check1; let scoped = sample query=\"auth\" | mode=lexical | limit=1 check rule1 | scope=scoped | confirm=skip with scoped",
            "check",
        ),
        (
            "let paths = find path a -> b | depth=1 explain paths | evidence=auto | truth=inspect with paths",
            "explain",
        ),
    ] {
        let document = resolve(source);
        let body = composition(&document).body();
        match expected {
            "query" | "sample-query" => assert!(matches!(body, ResolvedCompositionBodyV2::Query(_))),
            "graph" => assert!(matches!(body, ResolvedCompositionBodyV2::Graph(_))),
            "check" => assert!(matches!(body, ResolvedCompositionBodyV2::Check(_))),
            "explain" => assert!(matches!(body, ResolvedCompositionBodyV2::Explain(_))),
            _ => panic!("test vector has an unknown expected Composition body"),
        }
    }
}

#[test]
fn surface_to_hir_to_sema_mixed_document_reaches_all_semantic_item_variants() {
    let document = resolve(
        "let answer = 1; \
         rule local_rule : x if x = 1; \
         from records r | project r.id as id; \
         find path a -> b | depth=1; \
         ruledef policy | check-ref=check1; \
         check policy | confirm=skip; \
         show capabilities; \
         profile cert | capabilities=capability1 | admission=certified-only; \
         let sampled = sample query=\"auth\" | mode=lexical | limit=1 \
             from sampled s | project s.path as path with sampled; \
         require certified",
    );
    let counts = document.items().fold([0_u8; 9], |mut counts, (_, item)| {
        let index = match item {
            SemanticItemV2::Declaration(_) => 0,
            SemanticItemV2::Rule(_) => 1,
            SemanticItemV2::Relation(_) => 2,
            SemanticItemV2::Graph(_) => 3,
            SemanticItemV2::Check(_) => 4,
            SemanticItemV2::Presentation(_) => 5,
            SemanticItemV2::Catalog(_) => 6,
            SemanticItemV2::Composition(_) => 7,
            SemanticItemV2::Meta(_) => 8,
        };
        counts[index] += 1;
        counts
    });
    assert_eq!(counts, [1, 1, 1, 1, 1, 1, 2, 1, 1]);
}
