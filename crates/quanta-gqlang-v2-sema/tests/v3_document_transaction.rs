#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner assertions inspect exact transaction diagnostics and semantic variants"
)]

use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, AliasV1, CallArgHirV1, CallArgsHirV1, CatalogDeclHirV1,
    CatalogPayloadV1, ConfirmModeV1, DeclHirV1, DetectionModeV1, EmitModeV1, ExportFormatV1,
    GateBindingV1, GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1,
    GraphRefV1, HirBuilderV1, HirDocumentV1, HirNodeIdV1, LegalityHirV1, LiteralV1, LocatedSpanV2,
    MetaHirV1, MetaKindV1, OrderKeyHirV1, PathSegmentV1, PresentationHirV1, PresentationKindV1,
    ProgramItemHirV1, ProjectionItemV1, RecordFieldV1, RelationExistsPolarityV1, ShowCategoryV1,
    SortDirectionHirV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
    SymbolRefV1, ValueExprHirV1, ValueExprKindV1, ViewBodyV1,
};
use quanta_gqlang_v2_modules::{
    CallableKindV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2,
    ExternalCatalogPayloadInputV2, LocatedParameterInputV2, ModuleAuthorityBuilderV2, ModuleIdV2,
    ModuleNodeInputV2, SignatureInputV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, FrontendBudgetV2, FrontendTransactionErrorV2, GraphReferenceCatalogV2,
    GraphReferenceInputV2, GraphReferenceKindV2, SemanticItemIdV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ExpressionEffectsV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1,
    RelationCatalogV2, RelationSourceInputV2, RelationTypeV1, ValueTypeV1,
};

fn host() -> (SourceRegistryViewV2, ClosedFrontendAuthorityV2) {
    host_with_budget(FrontendBudgetV2::standard_v2().unwrap())
}

#[expect(
    clippy::too_many_lines,
    reason = "legacy integration fixture awaits decomposition"
)]
fn host_with_budget(budget: FrontendBudgetV2) -> (SourceRegistryViewV2, ClosedFrontendAuthorityV2) {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 8,
    }])
    .unwrap();
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .unwrap();
    let module = ModuleIdV2::try_new("app", location.clone()).unwrap();
    let profile = budget.authority_profile().unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "check1".into(),
            kind_token: "check".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Check,
            provenance: "host".into(),
            location: host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(3, 4),
                })
                .unwrap(),
            dependencies: Vec::new(),
        })
        .unwrap();
    for (kind_token, local_name, typed_payload) in [
        ("query", "query1", ExternalCatalogPayloadInputV2::Query),
        (
            "model_source",
            "model_source1",
            ExternalCatalogPayloadInputV2::ModelSource,
        ),
        (
            "pack_source",
            "pack_source1",
            ExternalCatalogPayloadInputV2::PackSource,
        ),
        (
            "capability",
            "capability1",
            ExternalCatalogPayloadInputV2::Capability,
        ),
    ] {
        builder
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".into(),
                local_name: local_name.into(),
                kind_token: kind_token.into(),
                typed_payload,
                provenance: "host".into(),
                location: host
                    .try_check_location(LocatedSpanV2 {
                        source_id: SourceIdV2::from_raw(1),
                        span: SpanV1::new(3, 4),
                    })
                    .unwrap(),
                dependencies: Vec::new(),
            })
            .unwrap();
    }
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "host_view".into(),
            kind_token: "view".into(),
            typed_payload: ExternalCatalogPayloadInputV2::View(
                RelationTypeV1::try_new(
                    RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                        "id",
                        ValueTypeV1::int(),
                        location.clone(),
                    )
                    .unwrap()])
                    .unwrap(),
                    CardinalityV1::Bag,
                )
                .unwrap(),
            ),
            provenance: "host".into(),
            location: location.clone(),
            dependencies: Vec::new(),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "host_pred".into(),
            kind_token: "predicate".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                parameters: vec![LocatedParameterInputV2 {
                    name: "value".into(),
                    value_type: ValueTypeV1::int(),
                    location: location.clone(),
                }],
                result: ValueTypeV1::bool(),
                callable_kind: CallableKindV2::Predicate,
                effects: ExpressionEffectsV2::pure_scalar(),
                location: location.clone(),
            }),
            provenance: "host".into(),
            location: location.clone(),
            dependencies: Vec::new(),
        })
        .unwrap();
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location,
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let relation_location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(4, 5),
        })
        .unwrap();
    let relations = RelationCatalogV2::try_new(
        vec![
            RelationSourceInputV2 {
                authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
                provenance_namespace: ProvenanceNamespaceV2::try_new(
                    "host",
                    relation_location.clone(),
                )
                .unwrap(),
                qualified_name: "records".into(),
                lookup_name: "records".into(),
                row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                    "id",
                    ValueTypeV1::int(),
                    relation_location.clone(),
                )
                .unwrap()])
                .unwrap(),
                cardinality: CardinalityV1::Bag,
                provenance: "host".into(),
                location: relation_location.clone(),
                host_binding_identity: None,
            },
            RelationSourceInputV2 {
                authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
                provenance_namespace: ProvenanceNamespaceV2::try_new(
                    "host",
                    relation_location.clone(),
                )
                .unwrap(),
                qualified_name: "records2".into(),
                lookup_name: "records2".into(),
                row: RecordTypeV1::try_new(vec![
                    RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), relation_location.clone())
                        .unwrap(),
                    RecordFieldTypeV1::try_new(
                        "value",
                        ValueTypeV1::int(),
                        relation_location.clone(),
                    )
                    .unwrap(),
                ])
                .unwrap(),
                cardinality: CardinalityV1::Bag,
                provenance: "host".into(),
                location: relation_location,
                host_binding_identity: None,
            },
        ],
        &host,
        profile,
    )
    .unwrap();
    let graph = GraphReferenceCatalogV2::try_new(
        [
            ("repo::src", SpanV1::new(1, 2), GraphReferenceKindV2::Entity),
            (
                "repo::sink",
                SpanV1::new(2, 3),
                GraphReferenceKindV2::Entity,
            ),
            (
                "result.latest",
                SpanV1::new(5, 6),
                GraphReferenceKindV2::ExportSource,
            ),
        ]
        .into_iter()
        .map(|(spelling, span, kind)| GraphReferenceInputV2 {
            spelling: spelling.into(),
            kind,
            entity_subtype: None,
            location: host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .unwrap(),
        })
        .collect(),
        &host,
        budget.authority_profile().unwrap(),
    )
    .unwrap();
    (
        host,
        ClosedFrontendAuthorityV2::try_new(modules, relations, graph, budget).unwrap(),
    )
}

fn meta_document(source: u32, node: u32) -> HirDocumentV1 {
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(source),
        source_len: 17,
        items: vec![ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
            HirNodeIdV1::from_raw(node),
            SpanV1::new(0, 17),
            MetaKindV1::RequireCertified,
        )))],
    }
}

#[path = "v3_document_transaction/document_rule_tests.rs"]
mod document_rule_tests;

#[path = "v3_document_transaction/authority_view_tests.rs"]
mod authority_view_tests;

#[path = "v3_document_transaction/relation_query_tests.rs"]
mod relation_query_tests;

#[path = "v3_document_transaction/aggregation_tests.rs"]
mod aggregation_tests;
