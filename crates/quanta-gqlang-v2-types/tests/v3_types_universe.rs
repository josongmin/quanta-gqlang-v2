#![allow(clippy::unwrap_used, reason = "owner assertions")]

use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1};
use quanta_gqlang_v2_types::*;

fn loc(source: u32, start: usize) -> LocatedSpanV2 {
    LocatedSpanV2 { source_id: SourceIdV2::from_raw(source), span: SpanV1::new(start, start + 1) }
}

fn checked(registry: &SourceRegistryViewV2, source: u32, start: usize) -> CheckedLocatedSpanV2 {
    registry.try_check_location(loc(source, start)).unwrap()
}

#[test]
fn checked_record_union_and_relation_carriers_reject_invalid_states() {
    let first = RecordFieldTypeV1::try_new("id", TypeV1::int(), loc(1, 0)).unwrap();
    let duplicate = RecordFieldTypeV1::try_new("id", TypeV1::string(), loc(1, 2)).unwrap();
    let error = RecordTypeV1::try_new(vec![first.clone(), duplicate]).unwrap_err();
    assert_eq!(error.code(), "types.duplicate_record_field");

    let row = RecordTypeV1::try_new(vec![first]).unwrap();
    let relation = RelationTypeV1::new(row.clone(), CardinalityV1::Bag);
    assert_eq!(relation.row(), &row);
    assert_eq!(relation.cardinality(), CardinalityV1::Bag);

    let a = UnionVariantTypeV1::try_new("ok", TypeV1::int(), loc(1, 4)).unwrap();
    let b = UnionVariantTypeV1::try_new("ok", TypeV1::string(), loc(1, 6)).unwrap();
    assert_eq!(UnionTypeV1::try_new(vec![a, b]).unwrap_err().code(), "types.duplicate_union_tag");
}

#[test]
fn finite_float_normalizes_signed_zero_and_orders_deterministically() {
    let positive = FiniteF64V2::try_new(0.0, loc(1, 0)).unwrap();
    let negative = FiniteF64V2::try_new(-0.0, loc(1, 1)).unwrap();
    assert_eq!(positive, negative);
    assert!(FiniteF64V2::try_new(f64::INFINITY, loc(1, 2)).is_err());
    assert!(FiniteF64V2::try_new(1.0, loc(1, 3)).unwrap() > negative);
}

#[test]
fn relation_catalog_rejects_duplicate_name_and_structured_id() {
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    let namespace = ProvenanceNamespaceV2::try_new("host").unwrap();
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 { source_id: SourceIdV2::from_raw(1), source_len: 10 }]).unwrap();
    let source = RelationSourceInputV2 { provenance_namespace: namespace.clone(), qualified_name: "pkg::users".into(), lookup_name: "users".into(), row: row.clone(), cardinality: CardinalityV1::Set, provenance: "catalog".into(), location: checked(&registry, 1, 0) };
    let duplicate = RelationSourceInputV2 { provenance_namespace: namespace, qualified_name: "pkg::users".into(), lookup_name: "users".into(), row, cardinality: CardinalityV1::Set, provenance: "catalog".into(), location: checked(&registry, 1, 2) };
    assert_eq!(RelationCatalogV2::try_new(vec![source, duplicate], &registry).unwrap_err().code(), "types.duplicate_relation_source");
}

#[test]
fn type_identity_is_location_independent() {
    let left=RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new("id",TypeV1::int(),loc(1,0)).unwrap()]).unwrap();
    let right=RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new("id",TypeV1::int(),loc(2,5)).unwrap()]).unwrap();
    assert_eq!(left,right);assert_eq!(TypeV1::record(left).to_string(),"{ id: int }");
}

#[test]
fn relation_catalog_retains_exact_source_registry_identity_and_rejects_oob() {
    let registry=SourceRegistryViewV2::try_new(vec![SourceRegistrationV2{source_id:SourceIdV2::from_raw(1),source_len:2}]).unwrap();
    let row=RecordTypeV1::try_new(vec![]).unwrap();let input=RelationSourceInputV2{provenance_namespace:ProvenanceNamespaceV2::try_new("Host").unwrap(),qualified_name:"Pkg::Users".into(),lookup_name:"Users".into(),row,cardinality:CardinalityV1::Set,provenance:"host".into(),location:checked(&registry,1,0)};
    let catalog=RelationCatalogV2::try_new(vec![input],&registry).unwrap();assert_eq!(catalog.source_registry_identity(),&registry.identity());
    let substitute=SourceRegistryViewV2::try_new(vec![SourceRegistrationV2{source_id:SourceIdV2::from_raw(1),source_len:8}]).unwrap();
    let row=RecordTypeV1::try_new(vec![]).unwrap();let invalid=RelationSourceInputV2{provenance_namespace:ProvenanceNamespaceV2::try_new("Host").unwrap(),qualified_name:"Pkg::Other".into(),lookup_name:"Other".into(),row,cardinality:CardinalityV1::Set,provenance:"host".into(),location:checked(&substitute,1,5)};
    assert_eq!(RelationCatalogV2::try_new(vec![invalid],&registry).unwrap_err().code(),"types.source_registry_mismatch");
}
