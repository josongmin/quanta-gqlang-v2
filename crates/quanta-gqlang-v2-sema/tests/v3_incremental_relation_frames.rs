#![expect(
    clippy::unwrap_used,
    reason = "owner assertions inspect exact checked relation contracts"
)]

use quanta_gqlang_v2_hir::{
    LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_sema::{OutputProvenanceV2, ResolvedScanSourceV2};
use quanta_gqlang_v2_types::{
    AuthorityBudgetProfileV2, CardinalityV1, ProvenanceNamespaceV2, RecordTypeV1,
    RelationCatalogV2, RelationSourceInputV2,
};

fn host_source() -> quanta_gqlang_v2_types::RelationSourceIdV2 {
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
    }])
    .unwrap();
    let location = registry
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .unwrap();
    let namespace = ProvenanceNamespaceV2::try_new("host", location.clone()).unwrap();
    let budget = AuthorityBudgetProfileV2::try_new(10, 10, 10, 1024, 128).unwrap();
    let catalog = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            provenance_namespace: namespace,
            qualified_name: "records".into(),
            lookup_name: "records".into(),
            row: RecordTypeV1::try_new(Vec::new()).unwrap(),
            cardinality: CardinalityV1::Bag,
            provenance: "host".into(),
            location,
        }],
        &registry,
        budget,
    )
    .unwrap();
    catalog.get("records").unwrap().id().clone()
}

#[test]
fn provenance_retains_host_relation_identity_without_catalog_view_coercion() {
    let relation = ResolvedScanSourceV2::Relation(host_source());
    let provenance = OutputProvenanceV2::new([relation.clone(), relation.clone()]);
    assert_eq!(provenance.iter().collect::<Vec<_>>(), vec![&relation]);
}
