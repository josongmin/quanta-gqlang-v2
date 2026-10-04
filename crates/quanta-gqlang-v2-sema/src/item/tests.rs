#![expect(
    clippy::unwrap_used,
    reason = "owner contract tests inspect exact invalid-state rejects"
)]

use super::*;
use crate::{OutputColumnContractV2, OutputColumnOriginV2, OutputProvenanceV2};
use quanta_gqlang_v2_hir::{
    HirNodeIdV1, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ExpressionEffectsV2, RecordFieldTypeV1, RecordTypeV1, ScalarTyV1,
    SourceQualifiedNodeIdV2, ValueTypeV1,
};

fn checked(source_id: SourceIdV2, span: SpanV1) -> CheckedLocatedSpanV2 {
    SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id,
        source_len: 128,
    }])
    .unwrap()
    .try_check_location(LocatedSpanV2 { source_id, span })
    .unwrap()
}

fn single_field_output(
    source_id: SourceIdV2,
    producer: SemanticItemIdV2,
    name: &str,
    scalar: ScalarTyV1,
    cardinality: CardinalityV1,
) -> RelationOutputContractV2 {
    let location = checked(source_id, SpanV1::new(10, 20));
    let field =
        RecordFieldTypeV1::try_new(name, ValueTypeV1::scalar(scalar), location.clone()).unwrap();
    let row = RecordTypeV1::try_new(vec![field]).unwrap();
    let column = OutputColumnContractV2::new(
        name.to_owned(),
        OutputColumnOriginV2::Generated(producer),
        location,
    );
    RelationOutputContractV2::try_new(
        row,
        OutputProvenanceV2::default(),
        vec![column],
        cardinality,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .unwrap()
}

fn sample(source_id: SourceIdV2, node: u32) -> ResolvedPresentationV2 {
    ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(node)),
        checked(source_id, SpanV1::new(60, 70)),
        ResolvedPresentationKindV2::Sample {
            query_text: "needle".to_owned(),
            mode: quanta_gqlang_v2_hir::SampleModeV1::Semantic,
            limit: 1,
            scope: None,
        },
    )
    .unwrap()
}

fn typed_relation(source_id: SourceIdV2, item_node: u32, root_node: u32) -> TypedRelationV2 {
    let location = checked(source_id, SpanV1::new(40, 50));
    let key = SourceQualifiedNodeIdV2 {
        source_id,
        node_id: HirNodeIdV1::from_raw(root_node),
    };
    let source = crate::ResolvedRelationSourceKindV2::Composition(
        crate::ResolvedCompositionSourceIdV2::try_from_binding(
            ResolvedValueBindingV2::try_new(
                source_id,
                HirNodeIdV1::from_raw(item_node),
                0,
                ResolvedBindingKindV2::Composition,
                "prior",
            )
            .unwrap(),
        )
        .unwrap(),
    );
    let field = RecordFieldTypeV1::try_new("value", ValueTypeV1::int(), location.clone()).unwrap();
    let row = RecordTypeV1::try_new(vec![field]).unwrap();
    let column = OutputColumnContractV2::new(
        "value".to_owned(),
        OutputColumnOriginV2::SourceField {
            source: source.clone(),
            scan_stage: key,
            field_name: "value".to_owned(),
        },
        location.clone(),
    );
    let output = RelationOutputContractV2::try_new(
        row,
        OutputProvenanceV2::new([source.clone()]),
        vec![column],
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .unwrap();
    let root =
        crate::TypedRelationExprV2::scan(key, location.clone(), source, output.clone()).unwrap();
    TypedRelationV2::try_new(
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(item_node)),
        location,
        key,
        root,
        output,
    )
    .unwrap()
}

fn graph(source_id: SourceIdV2, node: u32, kind: GraphQueryKindV1) -> ResolvedGraphQueryV2 {
    ResolvedGraphQueryV2::try_new(
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(node)),
        checked(source_id, SpanV1::new(30, 40)),
        kind,
        Vec::new(),
        None,
        None,
        Vec::new(),
        GraphWitnessBoundsV2::new(32, 64),
    )
    .unwrap()
}

#[test]
fn composition_carrier_preserves_exact_binder_ordinal_and_closed_families() {
    let source_id = SourceIdV2::from_raw(41);
    let composition_node = HirNodeIdV1::from_raw(7);
    let item_id = SemanticItemIdV2::new(source_id, composition_node);
    let binding = ResolvedValueBindingV2::try_new(
        source_id,
        composition_node,
        0,
        ResolvedBindingKindV2::Composition,
        "sampled",
    )
    .unwrap();
    let producer = ResolvedCompositionProducerV2::try_sample(
        sample(source_id, 8),
        single_field_output(
            source_id,
            SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8)),
            "path",
            ScalarTyV1::RepoPath,
            CardinalityV1::Set,
        ),
    )
    .unwrap();
    let resolved_binding = ResolvedCompositionBindingV2::try_new(
        item_id,
        0,
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8)),
        binding.clone(),
        checked(source_id, SpanV1::new(4, 11)),
        producer,
    )
    .unwrap();
    let body = ResolvedCompositionBodyV2::Graph(graph(source_id, 9, GraphQueryKindV1::Flow));
    let composition = ResolvedCompositionV2::try_new(
        item_id,
        checked(source_id, SpanV1::new(0, 64)),
        vec![resolved_binding],
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(9)),
        body,
        binding.clone(),
        checked(source_id, SpanV1::new(57, 64)),
    )
    .unwrap();

    assert_eq!(composition.with_target().ordinal(), 0);
    assert_eq!(composition.bindings().first().unwrap().name(), "sampled");
    assert_eq!(
        composition
            .bindings()
            .first()
            .unwrap()
            .relation_output()
            .unwrap()
            .row()
            .fields()
            .first()
            .unwrap()
            .name(),
        "path"
    );
    assert!(matches!(
        composition.body(),
        ResolvedCompositionBodyV2::Graph(_)
    ));
}

#[test]
fn composition_carrier_rejects_schema_kind_owner_and_with_transplants() {
    let source_id = SourceIdV2::from_raw(51);
    let other_source = SourceIdV2::from_raw(52);
    let composition_node = HirNodeIdV1::from_raw(17);
    let item_id = SemanticItemIdV2::new(source_id, composition_node);
    let canonical = ResolvedValueBindingV2::try_new(
        source_id,
        composition_node,
        0,
        ResolvedBindingKindV2::Composition,
        "sampled",
    )
    .unwrap();
    assert!(ResolvedCompositionProducerV2::try_sample(
        sample(source_id, 18),
        single_field_output(
            source_id,
            SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(18)),
            "path",
            ScalarTyV1::RepoPath,
            CardinalityV1::Bag,
        ),
    )
    .is_err());
    assert!(ResolvedCompositionProducerV2::try_sample(
        sample(source_id, 18),
        single_field_output(
            source_id,
            SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(19)),
            "path",
            ScalarTyV1::RepoPath,
            CardinalityV1::Set,
        ),
    )
    .is_err());
    assert!(ResolvedCompositionProducerV2::try_graph(
        graph(source_id, 19, GraphQueryKindV1::Flow),
        Some(single_field_output(
            source_id,
            SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(19)),
            "function",
            ScalarTyV1::EntityRef,
            CardinalityV1::Set,
        )),
    )
    .is_err());
    assert!(ResolvedCompositionProducerV2::try_graph(
        graph(source_id, 19, GraphQueryKindV1::Flow),
        Some(single_field_output(
            source_id,
            SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(19)),
            "arbitrary",
            ScalarTyV1::String,
            CardinalityV1::Bag,
        )),
    )
    .is_err());
    assert!(ResolvedCompositionProducerV2::try_graph(
        graph(source_id, 20, GraphQueryKindV1::Path),
        None,
    )
    .is_err());
    assert!(ResolvedCompositionProducerV2::try_graph(
        graph(source_id, 20, GraphQueryKindV1::Path),
        Some(single_field_output(
            source_id,
            SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(21)),
            "function",
            ScalarTyV1::EntityRef,
            CardinalityV1::Set,
        )),
    )
    .is_err());

    let wrong_kind = ResolvedValueBindingV2::try_new(
        source_id,
        composition_node,
        0,
        ResolvedBindingKindV2::Local,
        "sampled",
    )
    .unwrap();
    assert!(ResolvedCompositionBindingV2::try_new(
        item_id,
        0,
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(21)),
        wrong_kind,
        checked(source_id, SpanV1::new(4, 11)),
        ResolvedCompositionProducerV2::try_sample(
            sample(source_id, 21),
            single_field_output(
                source_id,
                SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(21)),
                "path",
                ScalarTyV1::RepoPath,
                CardinalityV1::Set,
            ),
        )
        .unwrap(),
    )
    .is_err());
    assert!(ResolvedCompositionBindingV2::try_new(
        item_id,
        1,
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(22)),
        canonical.clone(),
        checked(source_id, SpanV1::new(4, 11)),
        ResolvedCompositionProducerV2::try_sample(
            sample(source_id, 22),
            single_field_output(
                source_id,
                SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(22)),
                "path",
                ScalarTyV1::RepoPath,
                CardinalityV1::Set,
            ),
        )
        .unwrap(),
    )
    .is_err());
    assert!(ResolvedCompositionBindingV2::try_new(
        item_id,
        0,
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(99)),
        canonical.clone(),
        checked(source_id, SpanV1::new(4, 11)),
        ResolvedCompositionProducerV2::try_sample(
            sample(source_id, 22),
            single_field_output(
                source_id,
                SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(22)),
                "path",
                ScalarTyV1::RepoPath,
                CardinalityV1::Set,
            ),
        )
        .unwrap(),
    )
    .is_err());

    let valid = ResolvedCompositionBindingV2::try_new(
        item_id,
        0,
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(23)),
        canonical.clone(),
        checked(source_id, SpanV1::new(4, 11)),
        ResolvedCompositionProducerV2::try_sample(
            sample(source_id, 23),
            single_field_output(
                source_id,
                SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(23)),
                "path",
                ScalarTyV1::RepoPath,
                CardinalityV1::Set,
            ),
        )
        .unwrap(),
    )
    .unwrap();
    let foreign_with = ResolvedValueBindingV2::try_new(
        other_source,
        composition_node,
        0,
        ResolvedBindingKindV2::Composition,
        "sampled",
    )
    .unwrap();
    assert!(ResolvedCompositionV2::try_new(
        item_id,
        checked(source_id, SpanV1::new(0, 64)),
        vec![valid.clone()],
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(24)),
        ResolvedCompositionBodyV2::Graph(graph(source_id, 24, GraphQueryKindV1::Flow)),
        foreign_with,
        checked(source_id, SpanV1::new(57, 64)),
    )
    .is_err());
    assert!(ResolvedCompositionV2::try_new(
        item_id,
        checked(source_id, SpanV1::new(0, 64)),
        vec![valid.clone()],
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(99)),
        ResolvedCompositionBodyV2::Graph(graph(source_id, 24, GraphQueryKindV1::Flow)),
        canonical.clone(),
        checked(source_id, SpanV1::new(57, 64)),
    )
    .is_err());
    assert!(ResolvedCompositionV2::try_new(
        item_id,
        checked(source_id, SpanV1::new(0, 64)),
        vec![valid],
        SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(25)),
        ResolvedCompositionBodyV2::Explain(sample(source_id, 25)),
        canonical,
        checked(source_id, SpanV1::new(57, 64)),
    )
    .is_err());
}

#[test]
fn composition_graph_target_is_capability_checked_without_host_selector_reminting() {
    let source_id = SourceIdV2::from_raw(71);
    let binding = ResolvedValueBindingV2::try_new(
        source_id,
        HirNodeIdV1::from_raw(31),
        0,
        ResolvedBindingKindV2::Composition,
        "paths",
    )
    .unwrap();
    let entity = ResolvedGraphTargetV2::Composition(ResolvedCompositionGraphTargetV2 {
        source: crate::ResolvedCompositionSourceIdV2::try_from_binding(binding.clone()).unwrap(),
        capability: CompositionGraphCapabilityV2::Entity,
    });
    let scope = ResolvedGraphTargetV2::Composition(ResolvedCompositionGraphTargetV2 {
        source: crate::ResolvedCompositionSourceIdV2::try_from_binding(binding).unwrap(),
        capability: CompositionGraphCapabilityV2::Scope,
    });
    let location = checked(source_id, SpanV1::new(12, 17));

    assert!(ResolvedGraphReferenceV2::try_new(
        GraphReferenceRoleV2::Source,
        entity,
        location.clone(),
    )
    .is_ok());
    assert!(ResolvedGraphReferenceV2::try_new(
        GraphReferenceRoleV2::Scope,
        scope,
        location.clone(),
    )
    .is_ok());
    assert!(ResolvedGraphReferenceV2::try_new(
        GraphReferenceRoleV2::Scope,
        ResolvedGraphTargetV2::Composition(ResolvedCompositionGraphTargetV2 {
            source: crate::ResolvedCompositionSourceIdV2::try_from_binding(
                ResolvedValueBindingV2::try_new(
                    source_id,
                    HirNodeIdV1::from_raw(31),
                    0,
                    ResolvedBindingKindV2::Composition,
                    "paths",
                )
                .unwrap(),
            )
            .unwrap(),
            capability: CompositionGraphCapabilityV2::Entity,
        }),
        location,
    )
    .is_err());
}

#[test]
fn inline_view_requires_the_exact_nested_query_identity() {
    let source_id = SourceIdV2::from_raw(61);
    let catalog_id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(1));
    let query_id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(2));
    let exact = typed_relation(source_id, 2, 10);
    let same_document_other_item = typed_relation(source_id, 3, 20);

    assert!(inline_view_identity_matches(
        catalog_id,
        Some(query_id),
        Some(&exact)
    ));
    assert!(!inline_view_identity_matches(
        catalog_id,
        Some(query_id),
        Some(&same_document_other_item)
    ));
    assert!(!inline_view_identity_matches(
        catalog_id,
        Some(catalog_id),
        Some(&exact)
    ));
    assert!(!inline_view_identity_matches(
        catalog_id,
        None,
        Some(&exact)
    ));
    assert!(!inline_view_identity_matches(
        catalog_id,
        Some(query_id),
        None
    ));
    assert!(inline_view_identity_matches(catalog_id, None, None));
}
