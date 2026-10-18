#![expect(
    clippy::unwrap_used,
    reason = "owner assertions inspect exact Composition semantic contracts"
)]

use quanta_gqlang_v2_hir::{
    AliasV1, CatalogDeclHirV1, CatalogPayloadV1, CompositionBindingV1, CompositionBodyV1,
    CompositionHirV1, CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExplainHirV1, GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1,
    GraphQueryKindV1, GraphRefV1, HirBuilderV1, HirDocumentV1, HirNodeIdV1, LegalityHirV1,
    LocatedSpanV2, PathSegmentV1, ProgramItemHirV1, ProjectionItemV1, SampleHirV1, SampleModeV1,
    SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1, TruthModeV1,
};
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2, ExternalCatalogPayloadInputV2,
    ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, NamespaceKindV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    FrontendBudgetV2, FrontendTransactionErrorV2, GraphReferenceCatalogV2, GraphReferenceInputV2,
    GraphReferenceKindV2, OutputColumnOriginV2, ResolvedCompositionBodyV2,
    ResolvedCompositionProducerV2, ResolvedExplainTargetV2, ResolvedGraphTargetV2,
    ResolvedPresentationKindV2, ResolvedRelationSourceKindV2, SemanticItemIdV2, SemanticItemV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2,
    RelationSourceInputV2, ScalarTyV1, ValueTypeV1,
};

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
    modules
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "check1".into(),
            kind_token: "check".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Check,
            provenance: "host".into(),
            location: location.clone(),
            dependencies: Vec::new(),
        })
        .unwrap();
    modules
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "capability1".into(),
            kind_token: "capability".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Capability,
            provenance: "host".into(),
            location: location.clone(),
            dependencies: Vec::new(),
        })
        .unwrap();
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
            authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
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
            host_binding_identity: None,
        })
        .collect();
    let relations = RelationCatalogV2::try_new(relation_inputs, &host, profile.clone()).unwrap();
    let graphs = GraphReferenceCatalogV2::try_new(
        vec![
            GraphReferenceInputV2 {
                spelling: "scope".into(),
                kind: GraphReferenceKindV2::Scope,
                entity_subtype: None,
                location: location.clone(),
            },
            GraphReferenceInputV2 {
                spelling: "a".into(),
                kind: GraphReferenceKindV2::Entity,
                entity_subtype: None,
                location: location.clone(),
            },
            GraphReferenceInputV2 {
                spelling: "b".into(),
                kind: GraphReferenceKindV2::Entity,
                entity_subtype: None,
                location: location.clone(),
            },
        ],
        &host,
        profile,
    )
    .unwrap();
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget).unwrap()
}

fn resolve(
    composition: CompositionHirV1,
    source_len: usize,
) -> Result<quanta_gqlang_v2_sema::SemanticDocumentV2, FrontendTransactionErrorV2> {
    resolve_items(
        vec![ProgramItemHirV1::Composition(Box::new(composition))],
        source_len,
    )
}

fn resolve_items(
    items: Vec<ProgramItemHirV1>,
    source_len: usize,
) -> Result<quanta_gqlang_v2_sema::SemanticDocumentV2, FrontendTransactionErrorV2> {
    resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len,
            items,
        })
        .unwrap(),
        &authority(),
    )
}

#[expect(
    clippy::panic,
    reason = "owner test rejects a non-Composition semantic source"
)]
fn assert_composition_source(
    source: Option<&ResolvedRelationSourceKindV2>,
    source_id: SourceIdV2,
    owner: HirNodeIdV1,
    ordinal: usize,
) {
    let Some(ResolvedRelationSourceKindV2::Composition(identity)) = source else {
        panic!("Composition source expected");
    };
    let binding = identity.binding();
    assert_eq!(binding.source_id(), source_id);
    assert_eq!(binding.target_node(), owner);
    assert_eq!(binding.ordinal(), ordinal);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn sequential_query_lets_retain_exact_owner_ordinal_rows_and_scan_sources() {
    let builder = HirBuilderV1::new();
    let first_root =
        builder.relation_source("records", SpanV1::new(12, 19), None, SpanV1::new(12, 19));
    let first_query = builder.query(first_root, None, SpanV1::new(8, 19));
    let first_query_id = first_query.id;
    let second_root =
        builder.relation_source("first", SpanV1::new(34, 39), None, SpanV1::new(34, 39));
    let second_root_id = second_root.id;
    let second_query = builder.query(second_root, None, SpanV1::new(30, 39));
    let second_query_id = second_query.id;
    let body_root =
        builder.relation_source("second", SpanV1::new(48, 54), None, SpanV1::new(48, 54));
    let body_root_id = body_root.id;
    let body_query = builder.query(body_root, None, SpanV1::new(44, 54));
    let body_query_id = body_query.id;
    let composition_id = HirNodeIdV1::from_raw(100);
    let composition = CompositionHirV1::new(
        composition_id,
        SpanV1::new(0, 66),
        vec![
            CompositionBindingV1::new(
                "first",
                SpanV1::new(4, 9),
                CompositionProducerV1::Query(Box::new(first_query)),
            ),
            CompositionBindingV1::new(
                "second",
                SpanV1::new(23, 29),
                CompositionProducerV1::Query(Box::new(second_query)),
            ),
        ],
        "second",
        SpanV1::new(60, 66),
        CompositionBodyV1::Query(Box::new(body_query)),
    );

    let document = resolve(composition, 66).unwrap();
    let item_id = SemanticItemIdV2::new(SourceIdV2::from_raw(9), composition_id);
    let SemanticItemV2::Composition(composition) = document.item(item_id).unwrap() else {
        panic!("typed Composition expected");
    };
    assert_eq!(composition.bindings().len(), 2);
    for (ordinal, binding) in composition.bindings().iter().enumerate() {
        assert_eq!(binding.binding().target_node(), composition_id);
        assert_eq!(binding.binding().ordinal(), ordinal);
        assert_eq!(
            binding.name_location().location().source_id,
            SourceIdV2::from_raw(9)
        );
        assert_eq!(
            binding
                .relation_output()
                .unwrap()
                .row()
                .fields()
                .first()
                .unwrap()
                .name(),
            "id"
        );
    }
    let first_binding = composition.bindings().first().unwrap();
    let second_binding = composition.bindings().get(1).unwrap();
    assert_eq!(composition.with_target(), second_binding.binding());
    let ResolvedCompositionProducerV2::Query(first) = first_binding.producer() else {
        panic!("first LET must retain its Query producer");
    };
    assert_eq!(first.item_id().hir_item_id(), first_query_id);
    let ResolvedCompositionProducerV2::Query(second) = second_binding.producer() else {
        panic!("second LET must retain its Query producer");
    };
    assert_eq!(second.item_id().hir_item_id(), second_query_id);
    assert_eq!(second.output().row(), first.output().row());
    assert_composition_source(
        second.root().source(),
        SourceIdV2::from_raw(9),
        composition_id,
        0,
    );
    assert_eq!(second.root().key().node_id, second_root_id);
    assert_eq!(second.output().provenance().iter().count(), 1);
    let ResolvedCompositionBodyV2::Query(body) = composition.body() else {
        panic!("Composition body must retain its Query");
    };
    assert_eq!(body.item_id().hir_item_id(), body_query_id);
    assert_eq!(body.output().row(), second.output().row());
    assert_composition_source(
        body.root().source(),
        SourceIdV2::from_raw(9),
        composition_id,
        1,
    );
    assert_eq!(body.root().key().node_id, body_root_id);
}

#[test]
#[expect(
    clippy::panic,
    reason = "transaction error variant is part of the oracle"
)]
fn self_and_forward_query_sources_never_fall_back_to_same_named_host_relations() {
    for (binding_name, source_name, source_span) in [
        ("records", "records", SpanV1::new(12, 19)),
        ("first", "later", SpanV1::new(12, 17)),
    ] {
        let builder = HirBuilderV1::new();
        let first_query = builder.query(
            builder.relation_source(source_name, source_span, None, source_span),
            None,
            SpanV1::new(8, 19),
        );
        let later_query = builder.query(
            builder.relation_source("records", SpanV1::new(30, 37), None, SpanV1::new(30, 37)),
            None,
            SpanV1::new(26, 37),
        );
        let body_query = builder.query(
            builder.relation_source("later", SpanV1::new(44, 49), None, SpanV1::new(44, 49)),
            None,
            SpanV1::new(40, 49),
        );
        let composition = CompositionHirV1::new(
            HirNodeIdV1::from_raw(100),
            SpanV1::new(0, 60),
            vec![
                CompositionBindingV1::new(
                    binding_name,
                    SpanV1::new(4, 11),
                    CompositionProducerV1::Query(Box::new(first_query)),
                ),
                CompositionBindingV1::new(
                    "later",
                    SpanV1::new(21, 25),
                    CompositionProducerV1::Query(Box::new(later_query)),
                ),
            ],
            "later",
            SpanV1::new(54, 59),
            CompositionBodyV1::Query(Box::new(body_query)),
        );
        let FrontendTransactionErrorV2::Document(error) = resolve(composition, 60).unwrap_err()
        else {
            panic!("self/forward source must be a document diagnostic");
        };
        let diagnostic = error.diagnostics().first().unwrap();
        assert_eq!(diagnostic.code(), "binder.unresolved_symbol");
        assert_eq!(diagnostic.primary().span, source_span);
    }
}

#[test]
#[expect(
    clippy::panic,
    reason = "transaction error variant is part of the oracle"
)]
fn non_row_local_source_rejects_without_same_named_host_fallback() {
    let builder = HirBuilderV1::new();
    let graph = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(50),
        SpanV1::new(8, 18),
        GraphQueryKindV1::Flow,
        GraphEndpointsV1::default(),
        GraphConstraintsV1::default(),
    );
    let body_source_span = SpanV1::new(30, 40);
    let body = builder.query(
        builder.relation_source("graph_rows", body_source_span, None, body_source_span),
        None,
        SpanV1::new(26, 40),
    );
    let composition = CompositionHirV1::new(
        HirNodeIdV1::from_raw(100),
        SpanV1::new(0, 55),
        vec![CompositionBindingV1::new(
            "graph_rows",
            SpanV1::new(4, 14),
            CompositionProducerV1::GraphQuery(Box::new(graph)),
        )],
        "graph_rows",
        SpanV1::new(44, 54),
        CompositionBodyV1::Query(Box::new(body)),
    );
    let FrontendTransactionErrorV2::Document(error) = resolve(composition, 55).unwrap_err() else {
        panic!("non-row local source must be a document diagnostic");
    };
    let diagnostic = error.diagnostics().first().unwrap();
    assert_eq!(diagnostic.code(), "sema.non_relation_composition_source");
    assert_eq!(diagnostic.primary().span, body_source_span);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong resolved Composition body variant must fail the source oracle"
)]
fn composition_graph_body_materializes_with_its_exact_nested_identity() {
    let builder = HirBuilderV1::new();
    let producer = builder.query(
        builder.relation_source("records", SpanV1::new(8, 15), None, SpanV1::new(8, 15)),
        None,
        SpanV1::new(4, 15),
    );
    let graph_id = HirNodeIdV1::from_raw(77);
    let graph = GraphQueryHirV1::new(
        graph_id,
        SpanV1::new(24, 32),
        GraphQueryKindV1::Flow,
        GraphEndpointsV1::default(),
        GraphConstraintsV1::default(),
    );
    let composition_id = HirNodeIdV1::from_raw(100);
    let document = resolve(
        CompositionHirV1::new(
            composition_id,
            SpanV1::new(0, 45),
            vec![CompositionBindingV1::new(
                "rows",
                SpanV1::new(0, 4),
                CompositionProducerV1::Query(Box::new(producer)),
            )],
            "rows",
            SpanV1::new(40, 44),
            CompositionBodyV1::GraphQuery(Box::new(graph)),
        ),
        45,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let ResolvedCompositionBodyV2::Graph(body) = composition.body() else {
        panic!("Graph body expected");
    };
    assert_eq!(body.item_id().hir_item_id(), graph_id);
    assert_eq!(composition.with_target().canonical_name(), "rows");
}

#[test]
#[expect(
    clippy::panic,
    reason = "transaction error variant is part of the oracle"
)]
fn path_graph_producer_exports_its_generated_function_row_to_the_body() {
    let builder = HirBuilderV1::new();
    let graph_id = HirNodeIdV1::from_raw(50);
    let body_source = builder.relation_source(
        "functions",
        SpanV1::new(38, 47),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(48, 49),
        }),
        SpanV1::new(38, 49),
    );
    let function = builder
        .try_path(
            builder.symbol("f", SpanV1::new(60, 61)),
            vec![PathSegmentV1 {
                name: "function".into(),
                span: SpanV1::new(62, 70),
            }],
            SpanV1::new(60, 70),
        )
        .unwrap();
    let body = builder.query(
        builder
            .try_project(
                body_source,
                vec![ProjectionItemV1 {
                    value: function,
                    alias: Some(AliasV1 {
                        name: "function".into(),
                        span: SpanV1::new(74, 82),
                    }),
                }],
                SpanV1::new(38, 82),
            )
            .unwrap(),
        None,
        SpanV1::new(34, 82),
    );
    let composition_id = HirNodeIdV1::from_raw(100);
    let document = resolve(
        CompositionHirV1::new(
            composition_id,
            SpanV1::new(0, 92),
            vec![CompositionBindingV1::new(
                "functions",
                SpanV1::new(4, 13),
                CompositionProducerV1::GraphQuery(Box::new(GraphQueryHirV1::new(
                    graph_id,
                    SpanV1::new(16, 31),
                    GraphQueryKindV1::Path,
                    GraphEndpointsV1::default(),
                    GraphConstraintsV1::default(),
                ))),
            )],
            "functions",
            SpanV1::new(83, 92),
            CompositionBodyV1::Query(Box::new(body)),
        ),
        92,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let binding = composition.bindings().first().unwrap();
    let ResolvedCompositionProducerV2::Graph {
        graph,
        output: Some(output),
    } = binding.producer()
    else {
        panic!("PATH FIND must be a row-capable Composition producer");
    };
    assert_eq!(graph.item_id().hir_item_id(), graph_id);
    assert_eq!(output.cardinality(), CardinalityV1::Set);
    assert_eq!(output.row().fields().len(), 1);
    assert_eq!(output.row().fields().first().unwrap().name(), "function");
    assert_eq!(
        output.row().fields().first().unwrap().value_type(),
        &ValueTypeV1::scalar(ScalarTyV1::EntityRef)
    );
    assert!(matches!(
        output.columns(),
        [column] if matches!(column.origin(), OutputColumnOriginV2::Generated(origin) if *origin == graph.item_id())
    ));
    let ResolvedCompositionBodyV2::Query(body) = composition.body() else {
        panic!("query body expected");
    };
    assert_eq!(body.output().row(), output.row());
    let scan = body.root().inputs().next().unwrap();
    assert_composition_source(scan.source(), SourceIdV2::from_raw(9), composition_id, 0);
}

#[test]
#[expect(
    clippy::panic,
    reason = "SAMPLE must remain a sealed typed local relation producer"
)]
fn sample_producer_exports_only_its_generated_path_row_to_the_body() {
    let builder = HirBuilderV1::new();
    let body_source = builder.relation_source(
        "sampled",
        SpanV1::new(38, 45),
        Some(AliasV1 {
            name: "s".into(),
            span: SpanV1::new(46, 47),
        }),
        SpanV1::new(38, 47),
    );
    let path = builder
        .try_path(
            builder.symbol("s", SpanV1::new(58, 59)),
            vec![PathSegmentV1 {
                name: "path".into(),
                span: SpanV1::new(60, 64),
            }],
            SpanV1::new(58, 64),
        )
        .unwrap();
    let body_root = builder
        .try_project(
            body_source,
            vec![ProjectionItemV1 {
                value: path,
                alias: Some(AliasV1 {
                    name: "path".into(),
                    span: SpanV1::new(68, 72),
                }),
            }],
            SpanV1::new(38, 72),
        )
        .unwrap();
    let body = builder.query(body_root, None, SpanV1::new(34, 72));
    let composition_id = HirNodeIdV1::from_raw(100);
    let sample_id = HirNodeIdV1::from_raw(50);
    let document = resolve(
        CompositionHirV1::new(
            composition_id,
            SpanV1::new(0, 82),
            vec![CompositionBindingV1::new(
                "sampled",
                SpanV1::new(4, 11),
                CompositionProducerV1::Sample(Box::new(SampleHirV1 {
                    id: sample_id,
                    span: SpanV1::new(14, 31),
                    query_text: "auth".into(),
                    mode: SampleModeV1::Lexical,
                    limit: 2,
                    scope: None,
                })),
            )],
            "sampled",
            SpanV1::new(74, 81),
            CompositionBodyV1::Query(Box::new(body)),
        ),
        82,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let binding = composition.bindings().first().unwrap();
    let ResolvedCompositionProducerV2::Sample {
        presentation,
        output,
    } = binding.producer()
    else {
        panic!("SAMPLE producer expected");
    };
    let sample_item = SemanticItemIdV2::new(SourceIdV2::from_raw(9), sample_id);
    assert_eq!(presentation.item_id(), sample_item);
    assert_eq!(output.cardinality(), CardinalityV1::Set);
    assert!(output.provenance().is_empty());
    assert_eq!(output.row().fields().len(), 1);
    assert_eq!(output.row().fields().first().unwrap().name(), "path");
    assert_eq!(
        output.row().fields().first().unwrap().value_type(),
        &ValueTypeV1::scalar(ScalarTyV1::RepoPath)
    );
    assert!(matches!(
        output.columns(),
        [column] if matches!(column.origin(), OutputColumnOriginV2::Generated(origin) if *origin == sample_item)
    ));
    let ResolvedCompositionBodyV2::Query(body) = composition.body() else {
        panic!("query body expected");
    };
    assert_eq!(body.output().row().fields().len(), 1);
    assert_eq!(body.output().row().fields().first().unwrap().name(), "path");
    let scan = body.root().inputs().next().unwrap();
    assert_composition_source(scan.source(), SourceIdV2::from_raw(9), composition_id, 0);
}

#[test]
#[expect(
    clippy::panic,
    reason = "the SAMPLE producer row must not be widened by a caller-side assumption"
)]
fn sample_producer_rejects_body_fields_outside_its_closed_generated_row() {
    let builder = HirBuilderV1::new();
    let body_source_span = SpanV1::new(38, 45);
    let body_source = builder.relation_source(
        "sampled",
        body_source_span,
        Some(AliasV1 {
            name: "s".into(),
            span: SpanV1::new(46, 47),
        }),
        SpanV1::new(38, 47),
    );
    let function = builder
        .try_path(
            builder.symbol("s", SpanV1::new(58, 59)),
            vec![PathSegmentV1 {
                name: "function".into(),
                span: SpanV1::new(60, 68),
            }],
            SpanV1::new(58, 68),
        )
        .unwrap();
    let body = builder.query(
        builder
            .try_project(
                body_source,
                vec![ProjectionItemV1 {
                    value: function,
                    alias: Some(AliasV1 {
                        name: "function".into(),
                        span: SpanV1::new(72, 80),
                    }),
                }],
                SpanV1::new(38, 80),
            )
            .unwrap(),
        None,
        SpanV1::new(34, 80),
    );
    let composition = CompositionHirV1::new(
        HirNodeIdV1::from_raw(100),
        SpanV1::new(0, 90),
        vec![CompositionBindingV1::new(
            "sampled",
            SpanV1::new(4, 11),
            CompositionProducerV1::Sample(Box::new(SampleHirV1 {
                id: HirNodeIdV1::from_raw(50),
                span: SpanV1::new(14, 31),
                query_text: "auth".into(),
                mode: SampleModeV1::Lexical,
                limit: 2,
                scope: None,
            })),
        )],
        "sampled",
        SpanV1::new(82, 89),
        CompositionBodyV1::Query(Box::new(body)),
    );
    let FrontendTransactionErrorV2::Document(error) = resolve(composition, 90).unwrap_err() else {
        panic!("SAMPLE must reject an undeclared generated field");
    };
    let diagnostic = error.diagnostics().first().unwrap();
    assert_eq!(diagnostic.code(), "types.bad_projection");
    assert_eq!(diagnostic.primary().span, SpanV1::new(60, 68));
}

#[test]
#[expect(
    clippy::panic,
    reason = "a prior PATH producer must be the only authority for a local Entity target"
)]
fn path_graph_producer_is_a_local_entity_target_without_host_selector_fallback() {
    let composition_id = HirNodeIdV1::from_raw(100);
    let graph_body_id = HirNodeIdV1::from_raw(60);
    let document = resolve(
        CompositionHirV1::new(
            composition_id,
            SpanV1::new(0, 62),
            vec![CompositionBindingV1::new(
                "paths",
                SpanV1::new(4, 9),
                CompositionProducerV1::GraphQuery(Box::new(GraphQueryHirV1::new(
                    HirNodeIdV1::from_raw(50),
                    SpanV1::new(12, 27),
                    GraphQueryKindV1::Path,
                    GraphEndpointsV1::default(),
                    GraphConstraintsV1::default(),
                ))),
            )],
            "paths",
            SpanV1::new(57, 62),
            CompositionBodyV1::GraphQuery(Box::new(GraphQueryHirV1::new(
                graph_body_id,
                SpanV1::new(30, 54),
                GraphQueryKindV1::Flow,
                GraphEndpointsV1 {
                    source: Some(GraphRefV1 {
                        name: "paths".into(),
                        span: SpanV1::new(42, 47),
                    }),
                    ..GraphEndpointsV1::default()
                },
                GraphConstraintsV1::default(),
            ))),
        ),
        62,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let ResolvedCompositionBodyV2::Graph(body) = composition.body() else {
        panic!("graph body expected");
    };
    assert_eq!(body.item_id().hir_item_id(), graph_body_id);
    let target = body.references().first().unwrap().target();
    assert!(target.host_selector().is_none());
    assert!(matches!(target, ResolvedGraphTargetV2::Composition(_)));
}

#[test]
#[expect(
    clippy::panic,
    reason = "SAMPLE scope must resolve a prior local Scope capability before host lookup"
)]
fn sample_producer_can_scope_to_a_prior_local_sample_without_host_fallback() {
    let builder = HirBuilderV1::new();
    let body = builder.query(
        builder.relation_source("second", SpanV1::new(54, 60), None, SpanV1::new(54, 60)),
        None,
        SpanV1::new(50, 60),
    );
    let composition_id = HirNodeIdV1::from_raw(100);
    let document = resolve(
        CompositionHirV1::new(
            composition_id,
            SpanV1::new(0, 70),
            vec![
                CompositionBindingV1::new(
                    "first",
                    SpanV1::new(4, 9),
                    CompositionProducerV1::Sample(Box::new(SampleHirV1 {
                        id: HirNodeIdV1::from_raw(50),
                        span: SpanV1::new(12, 25),
                        query_text: "first".into(),
                        mode: SampleModeV1::Lexical,
                        limit: 1,
                        scope: None,
                    })),
                ),
                CompositionBindingV1::new(
                    "second",
                    SpanV1::new(28, 34),
                    CompositionProducerV1::Sample(Box::new(SampleHirV1 {
                        id: HirNodeIdV1::from_raw(51),
                        span: SpanV1::new(37, 49),
                        query_text: "second".into(),
                        mode: SampleModeV1::Semantic,
                        limit: 1,
                        scope: Some(GraphRefV1 {
                            name: "first".into(),
                            span: SpanV1::new(43, 48),
                        }),
                    })),
                ),
            ],
            "second",
            SpanV1::new(64, 70),
            CompositionBodyV1::Query(Box::new(body)),
        ),
        70,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let ResolvedCompositionProducerV2::Sample { presentation, .. } =
        composition.bindings().get(1).unwrap().producer()
    else {
        panic!("second binding must remain SAMPLE");
    };
    let ResolvedPresentationKindV2::Sample {
        scope: Some(scope), ..
    } = presentation.kind()
    else {
        panic!("SAMPLE scope expected");
    };
    assert!(scope.target().host_selector().is_none());
    assert!(matches!(
        scope.target(),
        ResolvedGraphTargetV2::Composition(_)
    ));
}

#[test]
#[expect(
    clippy::panic,
    reason = "Composition EXPLAIN must retain a Binder-first local Entity target"
)]
fn composition_explain_resolves_a_prior_path_let_without_host_fallback() {
    let composition_id = HirNodeIdV1::from_raw(100);
    let explain_id = HirNodeIdV1::from_raw(60);
    let document = resolve(
        CompositionHirV1::new(
            composition_id,
            SpanV1::new(0, 62),
            vec![CompositionBindingV1::new(
                "paths",
                SpanV1::new(4, 9),
                CompositionProducerV1::GraphQuery(Box::new(GraphQueryHirV1::new(
                    HirNodeIdV1::from_raw(50),
                    SpanV1::new(12, 27),
                    GraphQueryKindV1::Path,
                    GraphEndpointsV1::default(),
                    GraphConstraintsV1::default(),
                ))),
            )],
            "paths",
            SpanV1::new(57, 62),
            CompositionBodyV1::Explain(Box::new(ExplainHirV1 {
                id: explain_id,
                span: SpanV1::new(30, 54),
                target_ref: GraphRefV1 {
                    name: "paths".into(),
                    span: SpanV1::new(42, 47),
                },
                evidence: EvidenceModeV1::Auto,
                truth: TruthModeV1::Inspect,
            })),
        ),
        62,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let ResolvedCompositionBodyV2::Explain(explain) = composition.body() else {
        panic!("EXPLAIN body expected");
    };
    assert_eq!(explain.item_id().hir_item_id(), explain_id);
    let ResolvedPresentationKindV2::Explain {
        target: ResolvedExplainTargetV2::Entity(target),
        ..
    } = explain.kind()
    else {
        panic!("local Entity EXPLAIN target expected");
    };
    assert!(target.target().host_selector().is_none());
}

#[test]
#[expect(
    clippy::panic,
    reason = "Composition CHECK must combine the Modules Rule proof with a Binder-first local Scope"
)]
fn composition_check_preserves_modules_rule_and_prior_local_sample_scope() {
    let composition_id = HirNodeIdV1::from_raw(100);
    let check_id = HirNodeIdV1::from_raw(60);
    let catalog_rule = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(8),
        SpanV1::new(0, 20),
        "rule1",
        SpanV1::new(5, 10),
        CatalogPayloadV1::RuleBinding {
            check_ref: GraphRefV1 {
                name: "check1".into(),
                span: SpanV1::new(12, 18),
            },
        },
    );
    let composition = CompositionHirV1::new(
        composition_id,
        SpanV1::new(21, 90),
        vec![CompositionBindingV1::new(
            "scope",
            SpanV1::new(25, 30),
            CompositionProducerV1::Sample(Box::new(SampleHirV1 {
                id: HirNodeIdV1::from_raw(50),
                span: SpanV1::new(33, 47),
                query_text: "scope".into(),
                mode: SampleModeV1::Lexical,
                limit: 1,
                scope: None,
            })),
        )],
        "scope",
        SpanV1::new(84, 89),
        CompositionBodyV1::Legality(Box::new(LegalityHirV1::new(
            check_id,
            SpanV1::new(50, 80),
            GraphRefV1 {
                name: "rule1".into(),
                span: SpanV1::new(56, 61),
            },
            Some(GraphRefV1 {
                name: "scope".into(),
                span: SpanV1::new(70, 75),
            }),
            None,
            ConfirmModeV1::Skip,
            EmitModeV1::Findings,
            DetectionModeV1::PatternOnly,
        ))),
    );
    let document = resolve_items(
        vec![
            ProgramItemHirV1::CatalogDecl(Box::new(catalog_rule)),
            ProgramItemHirV1::Composition(Box::new(composition)),
        ],
        90,
    )
    .unwrap();
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let ResolvedCompositionBodyV2::Check(check) = composition.body() else {
        panic!("CHECK body expected");
    };
    assert_eq!(check.item_id().hir_item_id(), check_id);
    assert_eq!(check.rule().kind(), NamespaceKindV2::Rule);
    let scope = check.scope().unwrap();
    assert!(scope.target().host_selector().is_none());
    assert!(matches!(
        scope.target(),
        ResolvedGraphTargetV2::Composition(_)
    ));
}

#[test]
#[expect(
    clippy::panic,
    reason = "Composition CHECK must preserve independently proven Rule, Scope, and nested Entity paths"
)]
fn composition_check_uses_modules_rule_local_scope_and_local_nested_find_entity() {
    let composition_id = HirNodeIdV1::from_raw(100);
    let check_id = HirNodeIdV1::from_raw(60);
    let catalog_id = HirNodeIdV1::from_raw(90);
    let catalog_rule = CatalogDeclHirV1::new(
        catalog_id,
        SpanV1::new(0, 20),
        "rule1",
        SpanV1::new(5, 10),
        CatalogPayloadV1::RuleBinding {
            check_ref: GraphRefV1 {
                name: "check1".into(),
                span: SpanV1::new(12, 18),
            },
        },
    );
    let check = LegalityHirV1::new(
        check_id,
        SpanV1::new(72, 110),
        GraphRefV1 {
            name: "rule1".into(),
            span: SpanV1::new(78, 83),
        },
        Some(GraphRefV1 {
            name: "scoped".into(),
            span: SpanV1::new(90, 96),
        }),
        None,
        ConfirmModeV1::Require,
        EmitModeV1::Findings,
        DetectionModeV1::SemanticOnly,
    )
    .with_semantic_confirm(GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(61),
        SpanV1::new(97, 109),
        GraphQueryKindV1::Flow,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "paths".into(),
                span: SpanV1::new(103, 108),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
    ));
    let document = resolve_items(
        vec![
            ProgramItemHirV1::CatalogDecl(Box::new(catalog_rule)),
            ProgramItemHirV1::Composition(Box::new(CompositionHirV1::new(
                composition_id,
                SpanV1::new(21, 118),
                vec![
                    CompositionBindingV1::new(
                        "scoped",
                        SpanV1::new(25, 31),
                        CompositionProducerV1::Sample(Box::new(SampleHirV1 {
                            id: HirNodeIdV1::from_raw(50),
                            span: SpanV1::new(34, 46),
                            query_text: "scope".into(),
                            mode: SampleModeV1::Lexical,
                            limit: 1,
                            scope: None,
                        })),
                    ),
                    CompositionBindingV1::new(
                        "paths",
                        SpanV1::new(49, 54),
                        CompositionProducerV1::GraphQuery(Box::new(GraphQueryHirV1::new(
                            HirNodeIdV1::from_raw(51),
                            SpanV1::new(57, 69),
                            GraphQueryKindV1::Path,
                            GraphEndpointsV1::default(),
                            GraphConstraintsV1::default(),
                        ))),
                    ),
                ],
                "paths",
                SpanV1::new(112, 117),
                CompositionBodyV1::Legality(Box::new(check)),
            ))),
        ],
        118,
    )
    .unwrap();
    let SemanticItemV2::Catalog(rule) = document
        .item(SemanticItemIdV2::new(SourceIdV2::from_raw(9), catalog_id))
        .unwrap()
    else {
        panic!("Rule catalog item expected");
    };
    let SemanticItemV2::Composition(composition) = document
        .item(SemanticItemIdV2::new(
            SourceIdV2::from_raw(9),
            composition_id,
        ))
        .unwrap()
    else {
        panic!("Composition item expected");
    };
    let ResolvedCompositionBodyV2::Check(check) = composition.body() else {
        panic!("CHECK body expected");
    };
    assert_eq!(check.item_id().hir_item_id(), check_id);
    assert_eq!(check.rule().id(), rule.asset_id());
    assert!(check.scope().unwrap().target().host_selector().is_none());
    let nested = check.nested_graph().unwrap();
    assert_eq!(nested.item_id().hir_item_id(), HirNodeIdV1::from_raw(61));
    let target = nested.references().first().unwrap().target();
    assert!(target.host_selector().is_none());
    assert!(matches!(target, ResolvedGraphTargetV2::Composition(_)));
}

#[test]
#[expect(
    clippy::panic,
    reason = "a local Entity binding must reject CHECK Scope without consulting host authority"
)]
fn composition_check_rejects_local_path_as_scope_even_when_host_scope_has_the_same_name() {
    let catalog_rule = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(90),
        SpanV1::new(0, 20),
        "rule1",
        SpanV1::new(5, 10),
        CatalogPayloadV1::RuleBinding {
            check_ref: GraphRefV1 {
                name: "check1".into(),
                span: SpanV1::new(12, 18),
            },
        },
    );
    let scope_span = SpanV1::new(70, 75);
    let composition = CompositionHirV1::new(
        HirNodeIdV1::from_raw(100),
        SpanV1::new(21, 90),
        vec![CompositionBindingV1::new(
            "scope",
            SpanV1::new(25, 30),
            CompositionProducerV1::GraphQuery(Box::new(GraphQueryHirV1::new(
                HirNodeIdV1::from_raw(50),
                SpanV1::new(33, 47),
                GraphQueryKindV1::Path,
                GraphEndpointsV1::default(),
                GraphConstraintsV1::default(),
            ))),
        )],
        "scope",
        SpanV1::new(84, 89),
        CompositionBodyV1::Legality(Box::new(LegalityHirV1::new(
            HirNodeIdV1::from_raw(60),
            SpanV1::new(50, 80),
            GraphRefV1 {
                name: "rule1".into(),
                span: SpanV1::new(56, 61),
            },
            Some(GraphRefV1 {
                name: "scope".into(),
                span: scope_span,
            }),
            None,
            ConfirmModeV1::Skip,
            EmitModeV1::Findings,
            DetectionModeV1::PatternOnly,
        ))),
    );
    let FrontendTransactionErrorV2::Document(error) = resolve_items(
        vec![
            ProgramItemHirV1::CatalogDecl(Box::new(catalog_rule)),
            ProgramItemHirV1::Composition(Box::new(composition)),
        ],
        90,
    )
    .unwrap_err() else {
        panic!("local PATH must not become a CHECK Scope through same-name host fallback");
    };
    let diagnostic = error.diagnostics().first().unwrap();
    assert_eq!(diagnostic.code(), "sema.invalid_composition_graph_target");
    assert_eq!(diagnostic.primary().span, scope_span);
}
