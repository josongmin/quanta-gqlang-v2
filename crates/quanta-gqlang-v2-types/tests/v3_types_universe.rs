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

use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2,
    SpanV1,
};
use quanta_gqlang_v2_types::*;

fn loc(source: u32, start: usize) -> LocatedSpanV2 {
    LocatedSpanV2 {
        source_id: SourceIdV2::from_raw(source),
        span: SpanV1::new(start, start + 1),
    }
}

fn checked(registry: &SourceRegistryViewV2, source: u32, start: usize) -> CheckedLocatedSpanV2 {
    registry.try_check_location(loc(source, start)).unwrap()
}

fn type_registry() -> SourceRegistryViewV2 {
    SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 32,
        },
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(2),
            source_len: 32,
        },
    ])
    .unwrap()
}

fn authority_budget() -> AuthorityBudgetProfileV2 {
    AuthorityBudgetProfileV2::try_new(128, 256, 1024, 64 * 1024, 1024).unwrap()
}

#[test]
fn checked_record_union_and_relation_carriers_reject_invalid_states() {
    let registry = type_registry();
    let first =
        RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), checked(&registry, 1, 0)).unwrap();
    let duplicate =
        RecordFieldTypeV1::try_new("id", ValueTypeV1::string(), checked(&registry, 1, 2)).unwrap();
    let error = RecordTypeV1::try_new(vec![first.clone(), duplicate]).unwrap_err();
    assert_eq!(error.code(), "types.duplicate_record_field");
    let same_type_duplicate =
        RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), checked(&registry, 1, 3)).unwrap();
    assert_eq!(
        RecordTypeV1::try_new(vec![first.clone(), same_type_duplicate])
            .unwrap_err()
            .code(),
        "types.duplicate_record_field"
    );

    let row = RecordTypeV1::try_new(vec![first]).unwrap();
    let relation = RelationTypeV1::try_new(row.clone(), CardinalityV1::Bag).unwrap();
    assert_eq!(relation.row(), &row);
    assert_eq!(relation.cardinality(), CardinalityV1::Bag);

    let a =
        UnionVariantTypeV1::try_new("ok", ValueTypeV1::int(), checked(&registry, 1, 4)).unwrap();
    let b =
        UnionVariantTypeV1::try_new("ok", ValueTypeV1::string(), checked(&registry, 1, 6)).unwrap();
    assert_eq!(
        UnionTypeV1::try_new(vec![a, b]).unwrap_err().code(),
        "types.duplicate_union_tag"
    );
}

#[test]
fn finite_float_normalizes_signed_zero_and_orders_deterministically() {
    let positive = FiniteF64V2::try_new(0.0, loc(1, 0)).unwrap();
    let negative = FiniteF64V2::try_new(-0.0, loc(1, 1)).unwrap();
    assert_eq!(positive, negative);
    assert!(FiniteF64V2::try_new(f64::INFINITY, loc(1, 2)).is_err());
    assert_eq!(
        FiniteF64V2::try_new(f64::NAN, loc(1, 2))
            .unwrap_err()
            .code(),
        "types.float_non_finite"
    );
    assert!(FiniteF64V2::try_new(1.0, loc(1, 3)).unwrap() > negative);
}

#[test]
fn relation_catalog_rejects_duplicate_name_and_structured_id() {
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 10,
    }])
    .unwrap();
    let namespace = ProvenanceNamespaceV2::try_new("host", checked(&registry, 1, 0)).unwrap();
    let source = RelationSourceInputV2 {
        provenance_namespace: namespace.clone(),
        qualified_name: "pkg::users".into(),
        lookup_name: "users".into(),
        row: row.clone(),
        cardinality: CardinalityV1::Set,
        provenance: "catalog".into(),
        location: checked(&registry, 1, 0),
    };
    let duplicate = RelationSourceInputV2 {
        provenance_namespace: namespace,
        qualified_name: "pkg::users".into(),
        lookup_name: "users".into(),
        row,
        cardinality: CardinalityV1::Set,
        provenance: "catalog".into(),
        location: checked(&registry, 1, 2),
    };
    assert_eq!(
        RelationCatalogV2::try_new(vec![source, duplicate], &registry, authority_budget())
            .unwrap_err()
            .code(),
        "types.duplicate_relation_source"
    );
}

#[test]
fn relation_identity_and_diagnostic_rendering_are_globally_bounded() {
    let registry = type_registry();
    let location = checked(&registry, 1, 0);
    let namespace = ProvenanceNamespaceV2::try_new("host", location.clone()).unwrap();
    let error = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            provenance_namespace: namespace,
            qualified_name: format!("{}tail", "segment::".repeat(40)),
            lookup_name: "users".into(),
            row: RecordTypeV1::try_new(vec![]).unwrap(),
            cardinality: CardinalityV1::Set,
            provenance: "host".into(),
            location,
        }],
        &registry,
        authority_budget(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "types.invalid_canonical_name");
    assert!(error.to_string().len() <= 1024);
}

#[test]
fn type_identity_is_location_independent() {
    let registry = type_registry();
    let left = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
        "id",
        ValueTypeV1::int(),
        checked(&registry, 1, 0),
    )
    .unwrap()])
    .unwrap();
    let right = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
        "id",
        ValueTypeV1::int(),
        checked(&registry, 2, 5),
    )
    .unwrap()])
    .unwrap();
    assert_eq!(left, right);
    assert_eq!(TypeV1::record(left).to_string(), "{ id: int }");
}

#[test]
fn relation_catalog_retains_exact_source_registry_identity_and_rejects_oob() {
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 2,
    }])
    .unwrap();
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    let input = RelationSourceInputV2 {
        provenance_namespace: ProvenanceNamespaceV2::try_new("Host", checked(&registry, 1, 0))
            .unwrap(),
        qualified_name: "Pkg::Users".into(),
        lookup_name: "Users".into(),
        row,
        cardinality: CardinalityV1::Set,
        provenance: "host".into(),
        location: checked(&registry, 1, 0),
    };
    let profile = authority_budget();
    let catalog = RelationCatalogV2::try_new(vec![input], &registry, profile.clone()).unwrap();
    assert_eq!(catalog.source_registry_identity(), &registry.identity());
    assert_eq!(catalog.budget_profile(), &profile);
    let substitute = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 8,
    }])
    .unwrap();
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    let invalid = RelationSourceInputV2 {
        provenance_namespace: ProvenanceNamespaceV2::try_new("Host", checked(&substitute, 1, 5))
            .unwrap(),
        qualified_name: "Pkg::Other".into(),
        lookup_name: "Other".into(),
        row,
        cardinality: CardinalityV1::Set,
        provenance: "host".into(),
        location: checked(&substitute, 1, 5),
    };
    assert_eq!(
        RelationCatalogV2::try_new(vec![invalid], &registry, authority_budget())
            .unwrap_err()
            .code(),
        "types.source_registry_mismatch"
    );
}

#[test]
fn relation_catalog_enforces_profile_string_and_aggregate_byte_limits() {
    let registry = type_registry();
    let make_input = || RelationSourceInputV2 {
        provenance_namespace: ProvenanceNamespaceV2::try_new("host", checked(&registry, 1, 0))
            .unwrap(),
        qualified_name: "pkg::users".into(),
        lookup_name: "users".into(),
        row: RecordTypeV1::try_new(vec![]).unwrap(),
        cardinality: CardinalityV1::Set,
        provenance: "host".into(),
        location: checked(&registry, 1, 0),
    };
    let exact = AuthorityBudgetProfileV2::try_new(1, 1, 1, 23, 10).unwrap();
    assert!(RelationCatalogV2::try_new(vec![make_input()], &registry, exact).is_ok());
    let aggregate_short = AuthorityBudgetProfileV2::try_new(1, 1, 1, 22, 10).unwrap();
    let error =
        RelationCatalogV2::try_new(vec![make_input()], &registry, aggregate_short).unwrap_err();
    assert_eq!(error.code(), "types.resource_exhausted");
    let string_short = AuthorityBudgetProfileV2::try_new(1, 1, 1, 23, 9).unwrap();
    let error =
        RelationCatalogV2::try_new(vec![make_input()], &registry, string_short).unwrap_err();
    assert_eq!(error.code(), "types.resource_exhausted");
    assert_eq!(
        AuthorityBudgetProfileV2::try_new(0, 1, 1, 1, 1)
            .unwrap_err()
            .code(),
        "types.invalid_authority_budget"
    );
}

#[test]
fn type_names_are_canonical_ascii_and_bounded() {
    let registry = type_registry();
    for name in ["", "two words", "dot.name", "naïve"] {
        assert_eq!(
            RecordFieldTypeV1::try_new(name, ValueTypeV1::int(), checked(&registry, 1, 0))
                .unwrap_err()
                .code(),
            "types.invalid_type_name"
        );
    }
    assert!(
        RecordFieldTypeV1::try_new("dash-name", ValueTypeV1::int(), checked(&registry, 1, 0),)
            .is_ok()
    );
    assert_eq!(
        UnionVariantTypeV1::try_new(
            &"x".repeat(MAX_TYPE_NAME_BYTES + 1),
            ValueTypeV1::int(),
            checked(&registry, 1, 0),
        )
        .unwrap_err()
        .code(),
        "types.invalid_type_name"
    );
}

#[test]
fn public_recursive_carriers_enforce_shape_and_optional_invariants() {
    let empty = TupleTypeV1::try_new(vec![]).unwrap();
    assert!(empty.items().is_empty());
    let optional = OptionalTypeV1::try_new(ValueTypeV1::int()).unwrap();
    assert_eq!(optional.inner(), &ValueTypeV1::int());
    assert_eq!(
        OptionalTypeV1::try_new(ValueTypeV1::try_optional(ValueTypeV1::int()).unwrap())
            .unwrap_err()
            .code(),
        "types.redundant_optional"
    );
    let list = CollectionTypeV1::try_new(ValueTypeV1::string(), CardinalityV1::Sequence).unwrap();
    assert_eq!(list.element(), &ValueTypeV1::string());
    assert_eq!(list.cardinality(), CardinalityV1::Sequence);
}

#[test]
fn relation_types_cannot_cross_the_value_boundary_directly_or_nested() {
    let relation =
        RelationTypeV1::try_new(RecordTypeV1::try_new(vec![]).unwrap(), CardinalityV1::Set)
            .unwrap();
    assert_eq!(
        ValueTypeV1::try_from_type(TypeV1::relation(relation))
            .unwrap_err()
            .code(),
        "types.relation_used_as_value"
    );
}

#[test]
fn recursive_type_depth_and_node_budgets_fail_closed() {
    let mut nested = ValueTypeV1::int();
    for _ in 1..MAX_TYPE_DEPTH {
        nested = ValueTypeV1::try_list(nested).unwrap();
    }
    assert_eq!(nested.depth(), MAX_TYPE_DEPTH);
    assert_eq!(
        ValueTypeV1::try_list(nested).unwrap_err().code(),
        "types.type_shape_limit"
    );

    let exact_wide = ValueTypeV1::try_tuple(vec![ValueTypeV1::int(); MAX_TYPE_NODES - 1]).unwrap();
    assert_eq!(exact_wide.node_count(), MAX_TYPE_NODES);
    let wide = vec![ValueTypeV1::int(); MAX_TYPE_NODES];
    assert_eq!(
        TupleTypeV1::try_new(wide).unwrap_err().code(),
        "types.type_shape_limit"
    );

    let registry = type_registry();
    let duplicate_field =
        RecordFieldTypeV1::try_new("same", ValueTypeV1::int(), checked(&registry, 1, 0)).unwrap();
    assert_eq!(
        RecordTypeV1::try_new(vec![duplicate_field; MAX_TYPE_NODES])
            .unwrap_err()
            .code(),
        "types.type_shape_limit",
        "shape admission must reject before duplicate-map insertion"
    );
    let duplicate_variant =
        UnionVariantTypeV1::try_new("same", ValueTypeV1::int(), checked(&registry, 1, 0)).unwrap();
    assert_eq!(
        UnionTypeV1::try_new(vec![duplicate_variant; MAX_TYPE_NODES])
            .unwrap_err()
            .code(),
        "types.type_shape_limit",
        "shape admission must reject before duplicate-map insertion"
    );

    let optional_exact = ValueTypeV1::try_optional(
        ValueTypeV1::try_tuple(vec![ValueTypeV1::int(); MAX_TYPE_NODES - 2]).unwrap(),
    )
    .unwrap();
    assert_eq!(optional_exact.node_count(), MAX_TYPE_NODES);
    assert_eq!(
        ValueTypeV1::try_list(optional_exact).unwrap_err().code(),
        "types.type_shape_limit"
    );
}

#[test]
fn semantic_type_order_and_hash_ignore_construction_locations() {
    use std::collections::BTreeSet;
    let registry = type_registry();
    let left = ValueTypeV1::try_record(
        RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
            "id",
            ValueTypeV1::int(),
            checked(&registry, 1, 0),
        )
        .unwrap()])
        .unwrap(),
    )
    .unwrap();
    let right = ValueTypeV1::try_record(
        RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
            "id",
            ValueTypeV1::int(),
            checked(&registry, 2, 8),
        )
        .unwrap()])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(left, right);
    assert_eq!(BTreeSet::from([left, right]).len(), 1);
}

#[test]
fn relation_catalog_distinguishes_duplicate_id_from_duplicate_lookup_name() {
    let registry = type_registry();
    let namespace = ProvenanceNamespaceV2::try_new("host", checked(&registry, 1, 0)).unwrap();
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    let first = RelationSourceInputV2 {
        provenance_namespace: namespace.clone(),
        qualified_name: "pkg::users".into(),
        lookup_name: "users".into(),
        row: row.clone(),
        cardinality: CardinalityV1::Set,
        provenance: "one".into(),
        location: checked(&registry, 1, 0),
    };
    let same_id = RelationSourceInputV2 {
        provenance_namespace: namespace,
        qualified_name: "pkg::users".into(),
        lookup_name: "people".into(),
        row,
        cardinality: CardinalityV1::Set,
        provenance: "two".into(),
        location: checked(&registry, 1, 2),
    };
    assert_eq!(
        RelationCatalogV2::try_new(vec![first, same_id], &registry, authority_budget())
            .unwrap_err()
            .code(),
        "types.duplicate_relation_source_id"
    );
}

#[test]
fn resolved_predicate_and_capture_identities_are_canonical() {
    assert_eq!(
        ResolvedPredicateIdV2::try_new("bad name")
            .unwrap_err()
            .code(),
        "types.invalid_resolved_identity"
    );
    assert!(ResolvedPredicateIdV2::try_new("pkg::predicate_1").is_ok());
    assert_eq!(
        ResolvedPredicateIdV2::try_new(&"a".repeat(MAX_TYPE_NAME_BYTES + 1))
            .unwrap_err()
            .code(),
        "types.invalid_resolved_identity"
    );
}
