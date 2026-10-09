use super::{
    assign_strata, canonical_return_path, scan_offending_edges, BodyLiteralV1, BodyPolarityV1,
    LogicDiagnosticV1, OffendingEdgeV1, RecursiveProgramV1, RuleComponentsV1, RuleDefV1,
    RuleGraphChargeV1, RuleGraphV1, RuleProgramInvalidReasonV1,
};
use quanta_gqlang_v2_sema::{test_support::reference_checked_span_v2, FrontendBudgetV2};
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

#[test]
fn missing_or_out_of_range_component_index_rejects_at_each_actual_consumer() {
    let rule_location = reference_checked_span_v2(40, 44).expect("registered rule");
    let literal_location = reference_checked_span_v2(44, 48).expect("registered literal");
    let budget = FrontendBudgetV2::standard_v2()
        .expect("standard")
        .downstream_receipt();
    let predicate = ResolvedPredicateIdV2::try_new("p").expect("predicate");
    let program = RecursiveProgramV1::from_rules(vec![RuleDefV1::new(
        predicate.clone(),
        vec![BodyLiteralV1::new(
            predicate,
            BodyPolarityV1::Negated,
            literal_location.clone(),
        )],
        rule_location.clone(),
    )]);
    let graph = RuleGraphV1::build(&program, &mut RuleGraphChargeV1::new(&budget)).expect("graph");
    let edge = graph.edge(0, 0).expect("self edge");
    let offending = OffendingEdgeV1 {
        source: 0,
        component: 0,
        edge,
    };
    // Internal fault injection. Valid construction always supplies a complete
    // dense index; this proves consumers cannot fabricate empty membership or
    // a stratum zero if that invariant regresses.
    for component_of in [vec![], vec![1]] {
        let components = RuleComponentsV1 {
            members: vec![vec![0]],
            component_of,
        };
        let scan = scan_offending_edges(&graph, &components, &mut RuleGraphChargeV1::new(&budget))
            .expect_err("scan fails closed");
        let path = canonical_return_path(
            &graph,
            &components,
            &offending,
            &mut RuleGraphChargeV1::new(&budget),
        )
        .expect_err("self-cycle cannot bypass membership");
        let strata = assign_strata(&graph, &components, &mut RuleGraphChargeV1::new(&budget))
            .expect_err("missing membership cannot become stratum zero");
        for (error, expected_span) in [
            (scan, &literal_location),
            (path, &literal_location),
            (strata, &rule_location),
        ] {
            assert!(matches!(error, LogicDiagnosticV1::RuleProgramInvalid {
                reason: RuleProgramInvalidReasonV1::MembershipMismatch, span
            } if &span == expected_span));
        }
    }
}
