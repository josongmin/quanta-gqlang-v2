#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner test builds a sealed semantic document and inspects exact lowered shapes"
)]

//! Positive typed-stage lowering rails (ticket 05).
//!
//! Reconstruction note: the freeze-time overlay version of this file was lost
//! in an integrator incident; this content is the integrator-authorized
//! replacement. It keeps the Filter positive original (retained predicate
//! fact, checked Scan child, preserved output contract) and migrates the two
//! EXISTS-shape rails from the retired `Filter(EXISTS) over Semi/Anti` shape
//! to the pinned W2-A contract: a pure relational EXISTS/NOT EXISTS predicate
//! lowers to a Semi/Anti join rooted at the deepest join with no residual
//! Filter node, while an ordinary conjunction is retained by exactly one
//! Filter through `CheckedAlgebraNodeV2::filter_conjuncts`.

use quanta_gqlang_v2_algebra::{lower_relation_item, AlgebraFamilyV2, CheckedAlgebraNodeV2};
use quanta_gqlang_v2_sema::test_support::{
    reference_relation_document_v2, ReferenceRelationDocumentV2, ReferenceRelationFixtureV2,
};
use quanta_gqlang_v2_sema::{
    JoinModeV2, NullOrderV2, OutputColumnOriginV2, ResolvedScanSourceV2, SemanticItemV2,
    SortDirectionV2, TypedCallArgumentV2, TypedExprShapeV2, TypedRelationV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, SourceQualifiedNodeIdV2, ValueTypeV1,
};

#[test]
fn filter_retains_the_types_proven_predicate_and_checked_child() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();

    let root = lower_relation_item(reference.document(), reference.item_id())
        .unwrap()
        .root()
        .clone();
    assert_eq!(root.family(), AlgebraFamilyV2::Filter);
    let conjuncts = root.filter_conjuncts();
    assert_eq!(
        conjuncts.len(),
        1,
        "an ordinary predicate is retained as the full ordinary conjunction"
    );
    let Some(SemanticItemV2::Relation(relation)) = reference.document().item(reference.item_id())
    else {
        panic!("filter fixture must hold a relation item");
    };
    let Some(sealed_predicate) = relation.root().predicate() else {
        panic!("filter stage carries its typed predicate");
    };
    assert_eq!(
        conjuncts
            .first()
            .map(|conjunct| conjunct.expression().key()),
        Some(sealed_predicate.expression().key()),
        "the retained conjunct is the exact sealed typed Bool predicate fact"
    );
    let children = root.children();
    assert_eq!(children.len(), 1);
    assert_eq!(
        children.first().map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert!(children
        .first()
        .is_some_and(|child| root.output() == child.output()));
}

/// The one relational EXISTS expression of an EXISTS-family reference
/// fixture, together with its sealed nested relation root.
fn sole_relation_exists(
    reference: &ReferenceRelationDocumentV2,
) -> (SourceQualifiedNodeIdV2, SourceQualifiedNodeIdV2) {
    let document = reference.document();
    let Some(SemanticItemV2::Relation(relation)) = document.item(reference.item_id()) else {
        panic!("EXISTS fixture must hold a relation item");
    };
    assert!(
        relation.root().predicate().is_some(),
        "EXISTS fixture root is a Filter stage with a predicate"
    );
    let mut found: Option<SourceQualifiedNodeIdV2> = None;
    for (key, expression) in document.expressions() {
        if matches!(expression.shape(), TypedExprShapeV2::RelationExists { .. }) {
            assert!(
                found.is_none(),
                "fixture must carry exactly one relational EXISTS expression"
            );
            found = Some(*key);
        }
    }
    let Some(exists_key) = found else {
        panic!("fixture must carry one relational EXISTS expression");
    };
    let Some(exists_expression) = document.expression(exists_key) else {
        panic!("the EXISTS expression is present");
    };
    let TypedExprShapeV2::RelationExists { relation_root, .. } = exists_expression.shape() else {
        panic!("checked shape must still be a relational EXISTS");
    };
    (exists_key, *relation_root)
}

fn assert_pure_exists_tree_has_no_residual_filter(
    reference: &ReferenceRelationDocumentV2,
    expected_root_family: AlgebraFamilyV2,
) {
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();

    // A pure relational EXISTS predicate leaves no Filter node anywhere in
    // the lowered tree: the deepest Semi/Anti join is the stage root.
    let mut stack = vec![root];
    let mut visited = 0;
    while let Some(node) = stack.pop() {
        visited += 1;
        assert_ne!(
            node.family(),
            AlgebraFamilyV2::Filter,
            "a pure EXISTS predicate must not retain any residual Filter node"
        );
        let mut children = node.children();
        stack.extend(children.drain(..).rev());
    }
    assert_eq!(visited, 3, "outer scan + nested scan + one join");

    assert_eq!(root.family(), expected_root_family);
    assert!(
        root.filter_conjuncts().is_empty(),
        "no ordinary conjunct survives a pure EXISTS predicate"
    );
    assert!(
        root.predicate().is_none(),
        "a Semi/Anti join carries no join-ON predicate"
    );
    assert!(
        root.correlations()
            .is_some_and(|correlations| correlations.is_empty()),
        "the uncorrelated fixture lowers with an empty checked correlation set"
    );
    assert_eq!(root.item_id(), reference.item_id());

    // The Semi/Anti root is keyed by the EXISTS expression identity, not by
    // the authored Filter stage, and carries the stage's exact outer output.
    let (exists_key, nested_root_key) = sole_relation_exists(reference);
    assert_eq!(root.stage_key(), exists_key);
    let Some(SemanticItemV2::Relation(relation)) = reference.document().item(reference.item_id())
    else {
        panic!("EXISTS fixture must hold a relation item");
    };
    assert_eq!(root.output(), relation.output());

    // Children in evaluation order: the filtered outer relation first, then
    // the exact sealed nested relation root as the join's right input.
    let children = root.children();
    let (left, right) = (children.first(), children.get(1));
    assert_eq!(
        left.map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan),
        "the left input is the lowered outer relation"
    );
    assert_eq!(
        right.map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan),
        "the nested relation root of this fixture is a bare scan"
    );
    assert_eq!(
        right.map(|child| child.stage_key()),
        Some(nested_root_key),
        "the right input is the exact sealed nested relation root"
    );
}

#[test]
fn pure_exists_conjunct_lowers_to_semijoin_with_no_residual_filter() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Exists).unwrap();
    assert_pure_exists_tree_has_no_residual_filter(&reference, AlgebraFamilyV2::SemiJoin);
}

#[test]
fn pure_not_exists_conjunct_lowers_to_antijoin_with_no_residual_filter() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::NotExists).unwrap();
    assert_pure_exists_tree_has_no_residual_filter(&reference, AlgebraFamilyV2::AntiJoin);
}

// ---- typed stage lowering rails over the W2-D fixture matrix ---------------

/// The relation item of a reference fixture, for sealed-shape reads.
fn relation_of(reference: &ReferenceRelationDocumentV2) -> TypedRelationV2 {
    let Some(SemanticItemV2::Relation(relation)) = reference.document().item(reference.item_id())
    else {
        panic!("reference fixture must hold a relation item");
    };
    relation.clone()
}

/// Node families in authoring (pre-order) order.
fn families_in_authoring_order(root: &CheckedAlgebraNodeV2) -> Vec<AlgebraFamilyV2> {
    let mut stack = vec![root];
    let mut families = Vec::new();
    while let Some(node) = stack.pop() {
        families.push(node.family());
        let mut children = node.children();
        stack.extend(children.drain(..).rev());
    }
    families
}

/// The qualified host relation name of a scan node.
fn scan_qualified_name(node: &CheckedAlgebraNodeV2) -> &str {
    match node.scan_source().unwrap() {
        ResolvedScanSourceV2::Relation(source) => source.qualified_name(),
        ResolvedScanSourceV2::View(_) | ResolvedScanSourceV2::Composition(_) => {
            panic!("reference fixtures scan host relations only")
        }
    }
}

#[test]
fn project_lowering_retains_exact_typed_expressions_and_row_aligned_origins() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)
            .unwrap();
    let relation = relation_of(&reference);

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Project);

    // Exact stage identity: the node is keyed by the authored projection
    // stage, at its location, with its sealed output contract.
    assert_eq!(root.stage_key(), relation.root().key());
    assert_eq!(root.location(), relation.root().location());
    assert_eq!(root.output(), relation.output());

    // Two authored expressions, row-aligned: column i carries the Expression
    // origin of expression i, with the authored names and forwarded types.
    let expressions = root.expressions();
    assert_eq!(expressions.len(), 2);
    let columns = root.output().columns();
    assert_eq!(columns.len(), 2);
    for (expression, column) in expressions.iter().zip(columns) {
        assert!(matches!(
            column.origin(),
            OutputColumnOriginV2::Expression(origin) if *origin == expression.key()
        ));
    }
    assert_eq!(columns.first().unwrap().name(), "num");
    assert_eq!(columns.get(1).unwrap().name(), "label");
    let fields = root.output().row().fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields.first().unwrap().value_type(), &ValueTypeV1::int());
    assert_eq!(fields.get(1).unwrap().value_type(), &ValueTypeV1::string());

    // One checked Scan input; provenance passes through unchanged.
    let children = root.children();
    assert_eq!(children.len(), 1);
    assert_eq!(
        children.first().map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert_eq!(root.output().provenance().iter().count(), 1);
}

/// Every projection column carries the Expression origin of its own side's
/// authored expression (the anti-transplant invariant: a column never
/// names another side's scan or expression).
fn assert_expression_aligned_origins(project: &CheckedAlgebraNodeV2) {
    // `zip` truncates to the shorter side, so a missing or extra column would
    // otherwise leave the tail unverified; pin the one-to-one width first.
    assert_eq!(
        project.expressions().len(),
        project.output().columns().len(),
        "row-aligned origins require one output column per projection expression"
    );
    for (expression, column) in project.expressions().iter().zip(project.output().columns()) {
        assert!(matches!(
            column.origin(),
            OutputColumnOriginV2::Expression(origin) if *origin == expression.key()
        ));
    }
}

#[test]
fn inner_join_keeps_on_identity_and_both_scan_occurrences() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::InnerJoinOn).unwrap();
    let relation = relation_of(&reference);

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Join);
    assert_eq!(root.join_mode(), Some(JoinModeV2::Inner));

    // The retained ON predicate is the exact sealed typed Bool fact.
    let Some(sealed_on) = relation.root().predicate() else {
        panic!("the inner-join fixture seals an ON predicate");
    };
    let Some(on) = root.predicate() else {
        panic!("an Inner join retains its ON predicate");
    };
    assert_eq!(on.expression().key(), sealed_on.expression().key());

    // Two distinct scan occurrences, one per side, with their own host
    // relations; columns concatenate in child order and provenance unites.
    let children = root.children();
    assert_eq!(children.len(), 2);
    assert_eq!(
        children.first().map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert_eq!(
        children.get(1).map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert_ne!(
        children.first().unwrap().stage_key(),
        children.get(1).unwrap().stage_key()
    );
    assert_eq!(
        children.first().map(|child| scan_qualified_name(child)),
        Some("app::records")
    );
    assert_eq!(
        children.get(1).map(|child| scan_qualified_name(child)),
        Some("app::other")
    );
    let concatenated: Vec<_> = children
        .first()
        .unwrap()
        .output()
        .columns()
        .iter()
        .chain(children.get(1).unwrap().output().columns().iter())
        .cloned()
        .collect();
    assert_eq!(root.output().columns(), concatenated.as_slice());
    assert_eq!(root.output().provenance().iter().count(), 2);
    assert_eq!(root.output(), relation.output());
}

#[test]
fn self_join_keeps_two_distinct_scan_occurrences_row_aligned() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SelfJoinInner).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Join);
    assert_eq!(root.join_mode(), Some(JoinModeV2::Inner));
    assert!(root.predicate().is_some());

    // Each side is a projection over its own scan occurrence of the SAME
    // host relation: distinct stage keys and distinct authored path
    // expressions per side, with each side's columns carrying its own
    // side's Expression origin.
    let children = root.children();
    assert_eq!(children.len(), 2);
    for child in &children {
        assert_eq!(child.family(), AlgebraFamilyV2::Project);
        assert_expression_aligned_origins(child);
    }
    assert_ne!(
        children
            .first()
            .unwrap()
            .expressions()
            .first()
            .unwrap()
            .key(),
        children
            .get(1)
            .unwrap()
            .expressions()
            .first()
            .unwrap()
            .key(),
        "the two sides project their own scan occurrences' path facts"
    );
    let left_children = children.first().unwrap().children();
    let left_scan = left_children.first().unwrap();
    let right_children = children.get(1).unwrap().children();
    let right_scan = right_children.first().unwrap();
    assert_eq!(
        left_scan.stage_key().source_id,
        right_scan.stage_key().source_id
    );
    assert_ne!(left_scan.stage_key(), right_scan.stage_key());
    assert_eq!(scan_qualified_name(left_scan), "app::records");
    assert_eq!(scan_qualified_name(right_scan), "app::records");
}

#[test]
fn group_then_aggregate_preserves_partition_identity() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::Aggregate,
            AlgebraFamilyV2::Group,
            AlgebraFamilyV2::Scan
        ],
        "exactly one Group sits between the aggregate and its scan"
    );
    let root_children = root.children();
    let group = root_children.first().unwrap();

    // The authored Group: exactly one key, a key-only row, the partition
    // proof aligned with the pre-group frame, a Set output.
    let expressions = group.expressions();
    assert_eq!(expressions.len(), 1);
    assert_eq!(
        group.output().columns().len(),
        expressions.len(),
        "the Group projects one column per authored key"
    );
    assert_eq!(group.output().row().fields().len(), 1);
    assert_eq!(group.output().partition_keys().len(), 1);
    assert_eq!(
        group.output().partition_keys().first().unwrap(),
        &expressions.first().unwrap().key()
    );
    for (expression, column) in expressions.iter().zip(group.output().columns()) {
        assert!(matches!(
            column.origin(),
            OutputColumnOriginV2::Expression(origin) if *origin == expression.key()
        ));
    }
    assert_eq!(
        group.partition_input().len(),
        group
            .children()
            .first()
            .unwrap()
            .output()
            .row()
            .fields()
            .len()
    );
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert!(group.output().ordering().is_empty());

    // The aggregate consumes exactly that Group and retains its partition
    // identity verbatim.
    assert_eq!(
        root.output().partition_keys(),
        group.output().partition_keys()
    );
    assert_eq!(root.output().cardinality(), CardinalityV1::Set);
    assert_eq!(
        root.expressions()
            .first()
            .unwrap()
            .aggregate_context()
            .aggregation(),
        AggregationClassV1::Aggregate
    );
}

#[test]
fn two_key_group_preserves_authored_partition_order() {
    let reference = reference_relation_document_v2(
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
    )
    .unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::Aggregate,
            AlgebraFamilyV2::Group,
            AlgebraFamilyV2::Scan
        ]
    );
    let root_children = root.children();
    let group = root_children.first().unwrap();

    // Exactly the two authored keys, in authored order (r.id then
    // r.value): the partition identity, the key-only row, and the
    // row-aligned columns all follow that exact order.
    let expressions = group.expressions();
    assert_eq!(expressions.len(), 2);
    let keys = group.output().partition_keys();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys.first().unwrap(), &expressions.first().unwrap().key());
    assert_eq!(keys.get(1).unwrap(), &expressions.get(1).unwrap().key());
    assert_ne!(keys.first().unwrap(), keys.get(1).unwrap());
    // The AUTHORED order is part of the contract (r.id then r.value): a mutant
    // that emitted the same two keys in the opposite order is internally
    // consistent, so the key identities must be resolved to their field names.
    let authored: Vec<String> = keys
        .iter()
        .map(|key| {
            let expression = reference.document().expression(*key).unwrap();
            let TypedExprShapeV2::Path { segments, .. } = expression.shape() else {
                panic!("each authored group key is the checked field path");
            };
            segments.first().unwrap().name().to_owned()
        })
        .collect();
    assert_eq!(
        authored,
        vec!["id".to_owned(), "value".to_owned()],
        "the group keys keep their authored order"
    );
    for (expression, key) in expressions.iter().zip(keys) {
        assert!(
            matches!(
                reference
                    .document()
                    .expression(expression.key())
                    .unwrap()
                    .shape(),
                TypedExprShapeV2::Path { .. }
            ),
            "each authored group key is the checked field path"
        );
        assert!(expression.key() == *key);
    }
    for (expression, column) in expressions.iter().zip(group.output().columns()) {
        assert!(matches!(
            column.origin(),
            OutputColumnOriginV2::Expression(origin) if *origin == expression.key()
        ));
    }
    assert_eq!(
        group.output().columns().len(),
        expressions.len(),
        "the Group projects one column per authored key"
    );
    assert_eq!(group.output().row().fields().len(), 2);
    let bindings = group.partition_input();
    assert_eq!(
        bindings.len(),
        1,
        "the partition proof holds one binder binding per pre-group scan alias"
    );
    assert_eq!(bindings.first().unwrap().binding().canonical_name(), "r");

    // The aggregate retains the two-key partition identity verbatim, in
    // the same authored order.
    assert_eq!(
        root.output().partition_keys(),
        group.output().partition_keys()
    );
    assert_eq!(root.output().cardinality(), CardinalityV1::Set);
}

#[test]
fn scalar_aggregate_uses_exactly_one_implicit_empty_group() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::Aggregate,
            AlgebraFamilyV2::Group,
            AlgebraFamilyV2::Scan
        ]
    );
    let root_children = root.children();
    let group = root_children.first().unwrap();

    // The producer-sealed implicit Group is exactly empty: no keys, no
    // partition proof, no row, no columns, no ordering, Set output.
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert!(group.output().row().fields().is_empty());
    assert!(group.output().columns().is_empty());
    assert!(group.output().partition_keys().is_empty());
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert!(root.output().partition_keys().is_empty());
}

#[test]
fn count_star_keeps_the_sealed_star_argument() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Aggregate);

    // count(*): exactly one fold with exactly one Star argument — never a
    // zero-argument call spelling — classified as an aggregate using star.
    let expressions = root.expressions();
    assert_eq!(expressions.len(), 1);
    let fold = expressions.first().unwrap();
    let TypedExprShapeV2::Call { arguments, .. } = fold.shape() else {
        panic!("count(*) lowers as a checked call");
    };
    assert!(matches!(arguments.as_slice(), [TypedCallArgumentV2::Star]));
    assert!(fold.aggregate_context().uses_star());
    assert_eq!(
        fold.aggregate_context().aggregation(),
        AggregationClassV1::Aggregate
    );
    assert_eq!(
        root.output().columns().len(),
        1,
        "count(*) over the implicit empty group projects exactly one column"
    );
    assert!(matches!(
        root.output().columns().first().unwrap().origin(),
        OutputColumnOriginV2::Expression(origin) if *origin == fold.key()
    ));
}

fn assert_single_key_sort(
    fixture: ReferenceRelationFixtureV2,
    direction: SortDirectionV2,
    field_name: &str,
) {
    let reference = reference_relation_document_v2(fixture).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Sort);
    let keys = root.order_keys();
    assert_eq!(keys.len(), 1);

    // All four checked key fields stay sealed: expression identity,
    // direction, NULLS LAST policy, and value type; the output orders the
    // Sequence and preserves the input frame.
    let key = keys.first().unwrap();
    assert_eq!(key.direction(), direction);
    assert_eq!(key.null_order(), NullOrderV2::Last);
    // Expression identity: the key must resolve to the authored `field_name`
    // path, not merely to some column of the same value type.
    let key_expression = reference.document().expression(key.expression()).unwrap();
    let TypedExprShapeV2::Path { segments, .. } = key_expression.shape() else {
        panic!("sort keys are checked field paths");
    };
    assert_eq!(
        segments.first().unwrap().name(),
        field_name,
        "the sort key resolves to the authored field path"
    );
    let root_children = root.children();
    let input = root_children.first().unwrap();
    assert_eq!(
        input.family(),
        AlgebraFamilyV2::Scan,
        "the sort fixtures order a bare aliased scan"
    );
    let field = input
        .output()
        .row()
        .fields()
        .iter()
        .find(|field| field.name() == field_name)
        .unwrap();
    assert_eq!(key.value_type(), field.value_type());
    assert_eq!(root.output().ordering(), keys);
    assert_eq!(root.output().cardinality(), CardinalityV1::Sequence);
    assert_eq!(root.output().row(), input.output().row());
}

#[test]
fn sort_asc_preserves_all_checked_key_fields() {
    assert_single_key_sort(
        ReferenceRelationFixtureV2::SortAsc,
        SortDirectionV2::Asc,
        "id",
    );
}

#[test]
fn sort_desc_preserves_all_checked_key_fields() {
    assert_single_key_sort(
        ReferenceRelationFixtureV2::SortDesc,
        SortDirectionV2::Desc,
        "id",
    );
}

#[test]
fn nullable_key_sort_pins_the_nulls_last_payload() {
    // The nullable `label` authority field (Option<String>) carries the
    // full NULLS LAST payload proof on a genuinely nullable key.
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SortNullsLastNullableField)
            .unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(
        root.order_keys().len(),
        1,
        "a single-key sort fixture must lower to exactly one key"
    );
    let key = root.order_keys().first().unwrap();
    assert_eq!(key.direction(), SortDirectionV2::Desc);
    assert_eq!(key.null_order(), NullOrderV2::Last);
    let root_children = root.children();
    let input = root_children.first().unwrap();
    let label = input
        .output()
        .row()
        .fields()
        .iter()
        .find(|field| field.name() == "label")
        .unwrap();
    let Some(optional) = label.value_type().as_optional() else {
        panic!("the label authority field is nullable");
    };
    assert_eq!(optional.inner(), &ValueTypeV1::string());
    assert_eq!(key.value_type(), label.value_type());
}

#[test]
fn multi_key_sort_keeps_authored_order_and_directions() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SortMultiKeyMixedDirections)
            .unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    let keys = root.order_keys();
    assert_eq!(keys.len(), 2);

    // Authored order: r.value ascending first, r.id descending second.
    let first = keys.first().unwrap();
    let second = keys.get(1).unwrap();
    assert_eq!(first.direction(), SortDirectionV2::Asc);
    assert_eq!(second.direction(), SortDirectionV2::Desc);
    assert_ne!(first.expression(), second.expression());
    let document = reference.document();
    let segment_name = |key: &quanta_gqlang_v2_sema::OrderKeyContractV2| {
        let expression = document.expression(key.expression()).unwrap();
        let TypedExprShapeV2::Path { segments, .. } = expression.shape() else {
            panic!("sort keys are checked field paths");
        };
        segments.first().unwrap().name().to_owned()
    };
    assert_eq!(segment_name(first), "value");
    assert_eq!(segment_name(second), "id");
    assert_eq!(root.output().ordering(), keys);
    let root_children = root.children();
    let input = root_children.first().unwrap();
    assert_eq!(root.output().cardinality(), CardinalityV1::Sequence);
    assert_eq!(root.output().row(), input.output().row());
}

#[test]
fn sort_after_project_orders_by_projected_expression_identity() {
    for fixture in [
        ReferenceRelationFixtureV2::SortAliasKeyed,
        ReferenceRelationFixtureV2::SortByProjectedExpression,
    ] {
        let reference = reference_relation_document_v2(fixture).unwrap();

        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Sort);
        let root_children = root.children();
        let project = root_children.first().unwrap();
        assert_eq!(project.family(), AlgebraFamilyV2::Project);
        assert_eq!(project.expressions().len(), 1);
        assert_eq!(
            root.order_keys().len(),
            1,
            "the sort fixture authors exactly one key"
        );

        // The sort key resolves to the OUTPUT ALIAS binding fact — never a
        // fresh re-derivation of the projected path expression.
        // The sort key resolves to the OUTPUT ALIAS binding fact — bare for
        // the bare symbol, Paren-wrapped for the parenthesized spelling —
        // never a fresh re-derivation of the projected path expression.
        let key = root.order_keys().first().unwrap();
        let key_expression = reference.document().expression(key.expression()).unwrap();
        let resolves_to_score_alias =
            if let TypedExprShapeV2::Binding(binding) = key_expression.shape() {
                binding.canonical_name() == "score"
            } else if let TypedExprShapeV2::Paren { inner } = key_expression.shape() {
                reference
                .document()
                .expression(*inner)
                .is_some_and(|inner_expression| {
                    matches!(
                        inner_expression.shape(),
                        TypedExprShapeV2::Binding(binding) if binding.canonical_name() == "score"
                    )
                })
            } else {
                false
            };
        assert!(
            resolves_to_score_alias,
            "an output-alias sort key resolves to the alias binding fact"
        );
        assert_eq!(key.direction(), SortDirectionV2::Desc);
        assert_eq!(key.null_order(), NullOrderV2::Last);
        assert_eq!(root.output().ordering(), root.order_keys());
    }
}
