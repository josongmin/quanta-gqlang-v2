#[cfg(test)]
mod test_cardinality_soundness {
    use quanta_gqlang_v2_algebra::{
        AggregateExprV1, AggregateV1, AlgebraNodeV1, AlgebraOpV1, ColumnProvenanceV1, ColumnV1,
        ScanV1,
    };
    use quanta_gqlang_v2_hir::{HirNodeIdV1, SpanV1};
    use quanta_gqlang_v2_planning::{
        produce_plan, CardinalityClassV1, EffectClassV1, PlanRequestV1, RecursionShapeV1,
        RequestedResultKindV1, TruthLaneV1,
    };
    use quanta_gqlang_v2_types::CardinalityV1;

    fn id(n: u32) -> HirNodeIdV1 {
        HirNodeIdV1::from_raw(n)
    }

    fn span() -> SpanV1 {
        SpanV1::new(0, 10)
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

    // A scalar aggregate: SELECT COUNT(*) FROM big_table
    // Input: Bag (preserves duplicates), Output: ScalarOrSingleton (exactly 1 row)
    #[test]
    fn scalar_aggregate_should_be_singleton_not_linear() {
        let scan = AlgebraNodeV1::new(
            id(1),
            span(),
            vec![col("count")],
            CardinalityV1::Bag,  // Input is large bag from scan
            AlgebraOpV1::Scan(ScanV1 {
                relation: "big_table".to_owned(),
                alias: None,
            }),
        );

        let aggregate = AlgebraNodeV1::new(
            id(2),
            span(),
            vec![col("count")],
            CardinalityV1::Set,  // WRONG: should be ScalarOrSingleton, but the algebra normalization
                                 // may produce Set for a scalar agg
            AlgebraOpV1::Aggregate(AggregateV1 {
                input: Box::new(scan),
                aggregates: vec![AggregateExprV1 {
                    function: "count".to_owned(),
                    origin: id(10),
                    args: vec![],
                    distinct: false,
                }],
            }),
        );

        let request = PlanRequestV1 {
            root: aggregate,
            recursion: RecursionShapeV1::None,
            witness: None,
            result_kind: RequestedResultKindV1::Table,
            effect: EffectClassV1::ReadOnly,
            truth_lane: TruthLaneV1::Deterministic,
        };

        let plan = produce_plan(&request).expect("scalar aggregate plans");
        
        // EXPECTED: ScalarOrSingleton (one row result)
        // ACTUAL: The classify_node function will return Bag's cardinality class
        // which is LinearInInput, not ScalarOrSingleton
        println!(
            "Cardinality class: {:?}",
            plan.cost.estimated_cardinality_class
        );
        
        // This assertion SHOULD pass but will FAIL due to the bug
        assert_eq!(
            plan.cost.estimated_cardinality_class,
            CardinalityClassV1::ScalarOrSingleton,
            "A scalar aggregate with no grouping should produce exactly one row"
        );
    }
}
