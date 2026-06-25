#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! Logical explain-note provenance rails (ticket 07, decision `D24`).
//!
//! [`ExplainNoteV1`] is the host-agnostic provenance summary: the ordered,
//! root-first list of algebra families a result is built from, anchored to the
//! root span, plus a recursion flag. These rails drive the per-family arms of
//! `explain::collect_families` (every binary and unary op) and both note
//! constructors (`from_root`, `for_item`), asserting the EXACT family sequence
//! and span, not merely that a note was produced.

use quanta_gqlang_v2_algebra::{
    AlgebraFamilyV1, AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, ApplyV1, ColumnProvenanceV1, ColumnV1,
    FilterV1, GroupV1, JoinV1, LimitV1, ProjectV1, ScanV1, SemiJoinV1, SortDirectionV1, SortKeyV1,
    SortV1, UnionV1,
};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, HirNodeIdV1, LiteralV1, SpanV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_logic::{BodyLiteralV1, RecursiveProgramV1, RuleDefV1};
use quanta_gqlang_v2_planning::{
    produce_plan, EffectClassV1, ExplainNoteV1, PlanRequestV1, RecursionShapeV1,
    RequestedResultKindV1, TruthLaneV1,
};
use quanta_gqlang_v2_types::CardinalityV1;

fn id(n: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(n)
}

fn span_at(lo: usize, hi: usize) -> SpanV1 {
    SpanV1::new(lo, hi)
}

fn span() -> SpanV1 {
    span_at(0, 10)
}

fn col(name: &str) -> ColumnV1 {
    ColumnV1::new(
        name,
        ColumnProvenanceV1::Source {
            relation: "rel".to_owned(),
            alias: None,
            field: Some(name.to_owned()),
        },
    )
}

fn lit(n: u32) -> ValueExprHirV1 {
    ValueExprHirV1::new(
        id(n),
        span(),
        ValueExprKindV1::Literal(LiteralV1::Int("1".to_owned())),
    )
}

fn pred() -> Box<BoolExprHirV1> {
    Box::new(BoolExprHirV1::new(
        id(99),
        span(),
        BoolExprKindV1::Literal(true),
    ))
}

fn node_at(span: SpanV1, card: CardinalityV1, op: AlgebraOpV1) -> AlgebraNodeV1 {
    AlgebraNodeV1::new(id(1), span, vec![col("x")], card, op)
}

fn node(card: CardinalityV1, op: AlgebraOpV1) -> AlgebraNodeV1 {
    node_at(span(), card, op)
}

fn scan(name: &str) -> AlgebraNodeV1 {
    node(
        CardinalityV1::Bag,
        AlgebraOpV1::Scan(ScanV1 {
            relation: name.to_owned(),
            alias: None,
        }),
    )
}

fn fixpoint() -> RecursionShapeV1 {
    let program = RecursiveProgramV1::from_rules(vec![
        RuleDefV1::new("reach", vec![BodyLiteralV1::positive("edge")], span()),
        RuleDefV1::new(
            "reach",
            vec![
                BodyLiteralV1::positive("edge"),
                BodyLiteralV1::positive("reach"),
            ],
            span(),
        ),
    ]);
    RecursionShapeV1::Fixpoint(program.stratify().expect("positive recursion admits"))
}

/// `from_root` over a bare scan: a single-family note, anchored to the root
/// span, not recursive.
#[test]
fn from_root_scan_is_single_family_at_root_span() {
    let root = node_at(
        span_at(3, 9),
        CardinalityV1::Bag,
        AlgebraOpV1::Scan(ScanV1 {
            relation: "users".to_owned(),
            alias: None,
        }),
    );
    let note = ExplainNoteV1::from_root(&root, &RecursionShapeV1::None);
    assert_eq!(note.families, vec![AlgebraFamilyV1::Scan]);
    assert_eq!(note.root_span, span_at(3, 9));
    assert!(!note.recursive);
}

/// A unary chain (project over filter over scan) lists families root-first.
#[test]
fn from_root_unary_chain_is_root_first() {
    let filtered = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Filter(FilterV1 {
            input: Box::new(scan("users")),
            predicate: pred(),
        }),
    );
    let projected = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Project(ProjectV1 {
            input: Box::new(filtered),
            items: vec![lit(2)],
        }),
    );
    let note = ExplainNoteV1::from_root(&projected, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::Project,
            AlgebraFamilyV1::Filter,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// A group + aggregate chain.
#[test]
fn from_root_group_aggregate_chain() {
    use quanta_gqlang_v2_algebra::{AggregateExprV1, AggregateV1};
    let group = node(
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(scan("users")),
            keys: vec![lit(2)],
        }),
    );
    let aggregate = node(
        CardinalityV1::Set,
        AlgebraOpV1::Aggregate(AggregateV1 {
            input: Box::new(group),
            aggregates: vec![AggregateExprV1 {
                function: "count".to_owned(),
                origin: id(11),
                args: vec![],
                distinct: false,
            }],
        }),
    );
    let note = ExplainNoteV1::from_root(&aggregate, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::Aggregate,
            AlgebraFamilyV1::Group,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// A limit over a sort over a scan.
#[test]
fn from_root_limit_sort_chain() {
    let sorted = node(
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(scan("users")),
            keys: vec![SortKeyV1 {
                expr: lit(2),
                direction: SortDirectionV1::Ascending,
            }],
        }),
    );
    let limited = node(
        CardinalityV1::Sequence,
        AlgebraOpV1::Limit(LimitV1 {
            input: Box::new(sorted),
            count: 5,
            offset: 0,
        }),
    );
    let note = ExplainNoteV1::from_root(&limited, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::Limit,
            AlgebraFamilyV1::Sort,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// A join lists the join family then left subtree then right subtree, in
/// pre-order.
#[test]
fn from_root_join_is_preorder_left_then_right() {
    let joined = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: Some(pred()),
        }),
    );
    let note = ExplainNoteV1::from_root(&joined, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::Join,
            AlgebraFamilyV1::Scan,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// An apply node lists both correlated sides.
#[test]
fn from_root_apply_lists_both_sides() {
    let applied = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Apply(ApplyV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            correlation: vec![],
        }),
    );
    let note = ExplainNoteV1::from_root(&applied, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::Apply,
            AlgebraFamilyV1::Scan,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// A union lists both arms.
#[test]
fn from_root_union_lists_both_arms() {
    let unioned = node(
        CardinalityV1::Set,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            distinct: true,
        }),
    );
    let note = ExplainNoteV1::from_root(&unioned, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::Union,
            AlgebraFamilyV1::Scan,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// A semijoin lists both arms.
#[test]
fn from_root_semijoin_lists_both_arms() {
    let semi = node(
        CardinalityV1::Bag,
        AlgebraOpV1::SemiJoin(SemiJoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: None,
        }),
    );
    let note = ExplainNoteV1::from_root(&semi, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::SemiJoin,
            AlgebraFamilyV1::Scan,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// An antijoin lists both arms.
#[test]
fn from_root_antijoin_lists_both_arms() {
    let anti = node(
        CardinalityV1::Bag,
        AlgebraOpV1::AntiJoin(AntiJoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: None,
        }),
    );
    let note = ExplainNoteV1::from_root(&anti, &RecursionShapeV1::None);
    assert_eq!(
        note.families,
        vec![
            AlgebraFamilyV1::AntiJoin,
            AlgebraFamilyV1::Scan,
            AlgebraFamilyV1::Scan
        ]
    );
}

/// A fixpoint recursion sets the `recursive` flag on the note.
#[test]
fn from_root_fixpoint_sets_recursive_flag() {
    let note = ExplainNoteV1::from_root(&scan("reach"), &fixpoint());
    assert!(note.recursive);
    assert_eq!(note.families, vec![AlgebraFamilyV1::Scan]);
}

/// `for_item` produces an empty family list anchored to the item span; the
/// recursive flag is carried through verbatim.
#[test]
fn for_item_has_empty_families_and_carries_recursive_flag() {
    let recursive = ExplainNoteV1::for_item(span_at(5, 12), true);
    assert!(recursive.families.is_empty());
    assert_eq!(recursive.root_span, span_at(5, 12));
    assert!(recursive.recursive);

    let non_recursive = ExplainNoteV1::for_item(span_at(1, 2), false);
    assert!(non_recursive.families.is_empty());
    assert!(!non_recursive.recursive);
}

/// The plan produced by `produce_plan` carries the explain note built from the
/// root: a recursive program (closure) surfaces `recursive = true` on the plan's
/// explain note.
#[test]
fn produced_plan_carries_recursive_explain_note() {
    let request = PlanRequestV1 {
        root: scan("reach"),
        recursion: fixpoint(),
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    // A bag-cardinality scan under a fixpoint is not ordered, so it survives the
    // ordered-recursive-closure reject.
    let plan = produce_plan(&request).expect("recursive bag plan");
    assert!(plan.explain.recursive);
    assert_eq!(plan.explain.families, vec![AlgebraFamilyV1::Scan]);
    assert_eq!(plan.explain.root_span, span());
}
