#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner test builds a sealed semantic document and pins exact EXISTS-family trees"
)]

use quanta_gqlang_v2_algebra::{
    lower_relation_item, AlgebraFailureKindV2, AlgebraFamilyV2, CheckedAlgebraNodeV2,
};
use quanta_gqlang_v2_sema::{
    test_support::{
        reference_relation_document_v2, ReferenceRelationDocumentV2, ReferenceRelationFixtureV2,
        REFERENCE_SOURCE_V2,
    },
    JoinModeV2, ResolvedBindingKindV2, ResolvedScanSourceV2, SemanticItemV2, TypedBoolExprV2,
    TypedExprShapeV2, TypedRelationV2,
};

#[test]
fn explicit_cross_join_never_gains_a_synthetic_predicate() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::CrossJoin).unwrap();

    let root = lower_relation_item(reference.document(), reference.item_id())
        .unwrap()
        .root()
        .clone();
    assert_eq!(root.family(), AlgebraFamilyV2::Join);
    assert_eq!(root.join_mode(), Some(JoinModeV2::Cross));
    assert!(root.predicate().is_none());
    assert_eq!(root.children().len(), 2);
}

/// The relation item of a reference fixture, for sealed-shape reads.
fn relation_of(reference: &ReferenceRelationDocumentV2) -> TypedRelationV2 {
    let Some(SemanticItemV2::Relation(relation)) = reference.document().item(reference.item_id())
    else {
        panic!("reference fixture must hold a relation item");
    };
    relation.clone()
}

/// The single relational EXISTS expression fact of a fixture, keyed by the
/// filter stage's sealed predicate.
fn sole_exists_fact(reference: &ReferenceRelationDocumentV2) -> TypedBoolExprV2 {
    let relation = relation_of(reference);
    let Some(predicate) = relation.root().predicate() else {
        panic!("EXISTS fixture root is a Filter stage with a predicate");
    };
    predicate.clone()
}

#[test]
fn correlated_exists_retains_exact_captures_and_nested_root() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists).unwrap();
    let relation = relation_of(&reference);
    let exists = sole_exists_fact(&reference);

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::SemiJoin);
    assert_eq!(root.stage_key(), exists.expression().key());

    // Tree: SemiJoin(outer Scan, nested Filter(nested Scan)); the nested
    // right input is the exact sealed nested relation root.
    let children = root.children();
    assert_eq!(children.len(), 2);
    assert_eq!(
        children.first().map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    let nested = children.get(1).unwrap();
    assert_eq!(nested.family(), AlgebraFamilyV2::Filter);
    let TypedExprShapeV2::RelationExists { relation_root, .. } = reference
        .document()
        .expression(exists.expression().key())
        .unwrap()
        .shape()
    else {
        panic!("checked shape must still be a relational EXISTS");
    };
    assert_eq!(nested.stage_key(), *relation_root);
    assert_eq!(
        nested.children().first().map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert_eq!(nested.filter_conjuncts().len(), 1);

    // The one capture edge is the exact outer-alias binding captured at the
    // inner reference site.
    let correlations = root.correlations().unwrap();
    let edges: Vec<_> = correlations.iter().collect();
    assert_eq!(edges.len(), 1);
    let edge = edges.first().unwrap();
    assert_eq!(edge.binding().canonical_name(), "r");
    assert_eq!(edge.binding().kind(), ResolvedBindingKindV2::Alias);
    assert_eq!(edge.binding().ordinal(), 0);
    assert_eq!(edge.binding().source_id(), REFERENCE_SOURCE_V2);
    let captured = reference.document().expression(edge.reference()).unwrap();
    let TypedExprShapeV2::Binding(binding) = captured.shape() else {
        panic!("the capture reference is the inner r symbol fact");
    };
    assert_eq!(binding.canonical_name(), "r");
    assert_eq!(
        edge.location(),
        captured.location(),
        "the capture edge names the exact checked location of its reference"
    );
    assert_eq!(edge.location().location().span.start, 45);
    assert_eq!(edge.location().location().span.end, 46);

    // The SemiJoin retains the outer relation's exact output.
    assert_eq!(root.output(), relation.output());
}

#[test]
fn correlated_not_exists_keeps_the_same_exact_captures() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedNotExists).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::AntiJoin);
    let correlations = root.correlations().unwrap();
    let edges: Vec<_> = correlations.iter().collect();
    assert_eq!(edges.len(), 1);
    let edge = edges.first().unwrap();
    assert_eq!(edge.binding().canonical_name(), "r");
    assert_eq!(edge.binding().kind(), ResolvedBindingKindV2::Alias);
    assert_eq!(
        root.children().get(1).map(|child| child.family()),
        Some(AlgebraFamilyV2::Filter),
        "the AntiJoin right input is the nested filtered relation"
    );
}

#[test]
fn conjunctive_exists_attach_left_to_right_in_authored_order() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ConjunctiveExistsPair).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::SemiJoin);

    // Two conjunctive EXISTS lower to exactly two SemiJoins; the FIRST
    // authored conjunct attaches innermost.
    let children = root.children();
    let inner = children.first().unwrap();
    assert_eq!(inner.family(), AlgebraFamilyV2::SemiJoin);
    assert_eq!(
        root.children().get(1).map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert_eq!(inner.children().len(), 2);
    let scan_name = |node: &CheckedAlgebraNodeV2| match node.scan_source().unwrap() {
        ResolvedScanSourceV2::Relation(source) => source.qualified_name().to_owned(),
        ResolvedScanSourceV2::View(_) | ResolvedScanSourceV2::Composition(_) => {
            panic!("reference fixtures scan host relations only")
        }
    };
    assert_eq!(
        root.children().get(1).map(|child| scan_name(child)),
        Some("app::records2".to_owned()),
        "the SECOND authored EXISTS (records2) is the outer right input"
    );
    assert_eq!(
        inner.children().get(1).map(|child| scan_name(child)),
        Some("app::other".to_owned()),
        "the FIRST authored EXISTS (other) attaches innermost"
    );
    assert_eq!(
        inner.children().first().map(|child| scan_name(child)),
        Some("app::records".to_owned()),
        "the outer relation input is the leftmost input"
    );
    assert!(root.correlations().is_some_and(|set| set.is_empty()));
    assert!(inner.correlations().is_some_and(|set| set.is_empty()));
}

#[test]
fn ordinary_conjunct_survives_as_exactly_one_residual_filter() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ExistsWithBoolConjunct).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Filter);

    // Exactly ONE residual Filter holding the FULL ordinary conjunction
    // (here one BoolValue leaf with its document-owned literal child).
    let conjuncts = root.filter_conjuncts();
    assert_eq!(conjuncts.len(), 1);
    let TypedExprShapeV2::BoolValue { value } = conjuncts.first().unwrap().expression().shape()
    else {
        panic!("the residual conjunct is the authored ordinary Bool leaf");
    };
    let child = reference.document().expression(*value).unwrap();
    assert!(matches!(child.shape(), TypedExprShapeV2::Literal(_)));

    // Below it: exactly one SemiJoin over the outer scan and the nested
    // scan, with no second Filter anywhere on the spine.
    let children = root.children();
    let semi = children.first().unwrap();
    assert_eq!(semi.family(), AlgebraFamilyV2::SemiJoin);
    assert_eq!(semi.children().len(), 2);
    assert_eq!(
        semi.children().first().map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert_eq!(
        semi.children().get(1).map(|child| child.family()),
        Some(AlgebraFamilyV2::Scan)
    );
    assert!(semi.filter_conjuncts().is_empty());
    assert_eq!(root.output(), semi.output());
}

#[test]
fn nested_exists_keeps_boundary_separated_capture_sets() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels).unwrap();

    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::SemiJoin);

    // The outer EXISTS captures the outer binding at both reference sites;
    // its own capture set is otherwise unrelated to the nested EXISTS's.
    let outer_edges: Vec<_> = root.correlations().unwrap().iter().collect();
    assert_eq!(outer_edges.len(), 2);
    assert!(outer_edges
        .iter()
        .all(|edge| edge.binding().canonical_name() == "r"));
    assert_ne!(
        outer_edges.first().unwrap().reference(),
        outer_edges.get(1).unwrap().reference(),
        "the two outer captures are distinct reference sites"
    );

    // The middle relation keeps its own ordinary conjunct in exactly one
    // Filter, and the nested EXISTS carries its own single-edge set.
    let children = root.children();
    let middle = children.get(1).unwrap();
    assert_eq!(middle.family(), AlgebraFamilyV2::Filter);
    assert_eq!(middle.filter_conjuncts().len(), 1);
    let middle_children = middle.children();
    let nested_semi = middle_children.first().unwrap();
    assert_eq!(nested_semi.family(), AlgebraFamilyV2::SemiJoin);
    assert_eq!(nested_semi.correlations().unwrap().iter().count(), 1);
}

#[test]
fn exists_below_disjunction_rejects_as_a_conjunct_violation() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::DisjunctiveExistsPair).unwrap();

    let diagnostic = match lower_relation_item(reference.document(), reference.item_id()) {
        Ok(_) => panic!("a relational EXISTS below a disjunction must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::InvalidStageContract
    );
    assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
    assert_eq!(
        diagnostic.message(),
        "relational EXISTS must appear as a predicate conjunct"
    );
    // The rejection names the EXISTS expression found below the Or, at one
    // of the two authored EXISTS spans.
    assert!(
        diagnostic.location().is_some_and(|location| {
            let span = location.location().span;
            (span.start == 16 && span.end == 29) || (span.start == 36 && span.end == 52)
        }),
        "the rejection is pinned at the nested EXISTS expression's checked span"
    );
}

#[test]
fn negated_exists_rejects_as_a_conjunct_violation() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::NegatedExists).unwrap();

    let diagnostic = match lower_relation_item(reference.document(), reference.item_id()) {
        Ok(_) => panic!("a negated relational EXISTS must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::InvalidStageContract
    );
    assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
    assert_eq!(
        diagnostic.message(),
        "relational EXISTS must appear as a predicate conjunct"
    );
    assert!(
        diagnostic.location().is_some_and(|location| {
            let span = location.location().span;
            span.start == 18 && span.end == 31
        }),
        "the rejection is pinned at the EXISTS expression under the Not"
    );
}
