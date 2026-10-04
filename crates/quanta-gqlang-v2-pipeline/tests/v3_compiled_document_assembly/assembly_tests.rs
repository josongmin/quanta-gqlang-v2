use super::*;

#[test]
fn relation_scan_compiles_to_one_exact_planned_item() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("scan compiles");
    assert_eq!(compiled.source_id(), fixture.document().source_id());
    assert_eq!(compiled.items().len(), 1);
    assert!(compiled.diagnostics().is_empty());
    assert!(compiled.rule_program().is_none());
    let item = compiled
        .item(fixture.item_id())
        .expect("the authored relation identity is compiled");
    assert_eq!(item.item_id(), fixture.item_id());
    let CompiledItemV2::Planned(planned) = item else {
        panic!("a relational item must compile to a planned relation item");
    };
    let PlannedItemV2::Relation { plan, .. } = planned.as_ref() else {
        panic!("a relational item retains its relation plan");
    };
    assert!(item.plan().is_some());
    assert!(item.no_plan_reason().is_none());
    assert_eq!(
        plan,
        item.plan().expect("plan accessor agrees with the variant")
    );
}

#[test]
fn parenthesized_exists_and_not_parenthesized_exists_compile_as_literals() {
    for (source, family) in [
        (
            "from records r | where (exists (from records s | where s.id = r.id))",
            AlgebraFamilyV2::SemiJoin,
        ),
        (
            "from records r | where not (exists (from records s | where s.id = r.id))",
            AlgebraFamilyV2::AntiJoin,
        ),
    ] {
        let compiled = compile_source_document_v3(
            DOC_SOURCE,
            source,
            &authority_with_records_type(ValueTypeV1::int()),
        )
        .expect("transparent wrappers preserve a direct existential literal");
        let root = compiled.execution().items()[0]
            .relation()
            .expect("relation")
            .root();
        assert_eq!(root.family(), family);
        assert_eq!(root.children().len(), 2);
        assert_eq!(root.correlations().len(), 1);
    }
}

#[test]
fn many_exists_conditions_have_constant_operator_depth_and_closed_terms() {
    let source = format!(
        "from records r | where {}",
        std::iter::repeat("exists (from records s | where s.id = r.id)")
            .take(48)
            .collect::<Vec<_>>()
            .join(" and ")
    );
    let compiled = compile_source_document_v3(
        DOC_SOURCE,
        &source,
        &authority_with_records_type(ValueTypeV1::int()),
    )
    .expect("48 existential conjuncts fit the source depth budget");
    let relation = compiled.execution().items()[0]
        .relation()
        .expect("relation");
    let root = relation.root();
    assert_eq!(root.family(), AlgebraFamilyV2::ExistsFilter);
    assert_eq!(root.existential_terms().len(), 48);
    assert_eq!(root.children().len(), 49);
    assert_eq!(root.children()[0].family(), AlgebraFamilyV2::Scan);
    let table: BTreeSet<_> = relation
        .expressions()
        .iter()
        .map(|expr| expr.id())
        .collect();
    for (term, child) in root.existential_terms().iter().zip(&root.children()[1..]) {
        assert_eq!(child.family(), AlgebraFamilyV2::Filter);
        assert!(table.contains(&term.expression()));
        assert_eq!(term.correlations().len(), 1);
        assert!(table.contains(&term.correlations()[0].reference()));
    }
    assert_eq!(
        root.children()
            .iter()
            .map(|child| child.children().len())
            .max(),
        Some(1)
    );
}

#[test]
fn flat_exists_filter_retains_mixed_polarity_and_both_capabilities() {
    let source = concat!(
        "from records r | where exists (from records s | where s.id = r.id) ",
        "and not (exists (from records t | where t.id = r.id))"
    );
    let compiled = compile_source_document_v3(
        DOC_SOURCE,
        source,
        &authority_with_records_type(ValueTypeV1::int()),
    )
    .expect("mixed existential polarities compile");
    let item = &compiled.execution().items()[0];
    let root = item.relation().expect("relation").root();
    assert_eq!(root.family(), AlgebraFamilyV2::ExistsFilter);
    assert_eq!(root.existential_terms().len(), 2);
    assert_eq!(
        root.existential_terms()
            .iter()
            .map(|term| term.polarity())
            .collect::<Vec<_>>(),
        vec![
            quanta_gqlang_v2_pipeline::ExecutionExistsPolarityV3::Exists,
            quanta_gqlang_v2_pipeline::ExecutionExistsPolarityV3::NotExists,
        ]
    );
    let relation = item.relation().expect("relation");
    for (term, child) in root.existential_terms().iter().zip(&root.children()[1..]) {
        let expression = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == term.expression())
            .expect("each term names a closed expression");
        let ExecutionExpressionShapeV3::RelationExists {
            polarity,
            relation_root,
            correlations,
        } = expression.shape()
        else {
            panic!("each term names a relational EXISTS shape");
        };
        assert_eq!(
            *polarity,
            quanta_gqlang_v2_pipeline::ExecutionExistsPolarityV3::Exists,
            "an enclosing NOT changes the term, not its inner EXISTS fact"
        );
        assert_eq!(*relation_root, child.stage());
        assert_eq!(correlations.as_ref(), term.correlations());
    }
    assert_eq!(
        relation
            .expressions()
            .iter()
            .map(|expression| expression.id().node_id.raw())
            .collect::<Vec<_>>(),
        vec![2, 3, 4, 5, 6, 9, 11, 12, 13, 14, 15, 18],
        "the flat relation contains exactly its source-bound expression closure"
    );
    let plan = compiled.items()[0]
        .plan()
        .expect("relation must be planned");
    assert!(plan.capabilities().contains(&CapabilityV2::ExistenceCheck));
    assert!(plan.capabilities().contains(&CapabilityV2::NegationCheck));
}

#[test]
fn residual_filter_and_flat_exists_share_source_stage_provenance() {
    let source = concat!(
        "from records r | where true and exists (from records s) ",
        "and exists (from records t)"
    );
    let compiled = compile_source_document_v3(
        DOC_SOURCE,
        source,
        &authority_with_records_type(ValueTypeV1::int()),
    )
    .expect("ordinary and existential conjuncts compile");
    let residual = compiled.execution().items()[0]
        .relation()
        .expect("relation")
        .root();
    assert_eq!(residual.family(), AlgebraFamilyV2::Filter);
    let [flat] = residual.children() else {
        panic!("one flat existential child remains below the residual filter");
    };
    assert_eq!(flat.family(), AlgebraFamilyV2::ExistsFilter);
    assert_eq!(flat.existential_terms().len(), 2);
    assert_eq!(residual.stage(), flat.stage());
    assert_ne!(residual.family(), flat.family());
}

#[test]
fn exists_nested_under_or_or_scalar_wrapper_remains_unplannable() {
    for (source, expected_code) in [
        (
            "from records r | where true or exists (from records s)",
            "algebra.invalid_stage_contract",
        ),
        (
            "from records r | where exists (from records s) = true",
            "surface.parse.unexpected_token",
        ),
        (
            "from records r | where length(exists (from records s)) > 0",
            "surface.parse.unexpected_token",
        ),
    ] {
        let error = compile_source_document_v3(
            DOC_SOURCE,
            source,
            &authority_with_records_type(ValueTypeV1::int()),
        )
        .expect_err("nested existential must not produce an execution document");
        let FrontendTransactionErrorV2::Document(failure) = error else {
            panic!("nested existential must fail with a source diagnostic: {source}");
        };
        assert_eq!(failure.diagnostics().len(), 1, "{source}");
        assert_eq!(
            failure.diagnostics()[0].code(),
            expected_code,
            "nested existential must reject at its actual owner: {source}"
        );
    }
}

#[test]
fn compiled_item_order_is_the_authored_source_order() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::CrossJoin)
        .expect("join resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("join compiles");
    let order: Vec<SemanticItemIdV2> = fixture.document().order().to_vec();
    let assembled: Vec<SemanticItemIdV2> = compiled
        .items()
        .iter()
        .map(CompiledItemV2::item_id)
        .collect();
    assert_eq!(assembled, order);

    // A multi-item source discriminates ORDER, not just membership: with three
    // authored items the compiled sequence must match the document order
    // element-by-element.
    let multi = compile_source_document_v3(DOC_SOURCE, MINIMAL_SOURCE, &minimal_authority())
        .expect("the minimal three-item document compiles");
    assert_eq!(multi.items().len(), 3);
    let authored: Vec<SemanticItemIdV2> = resolve_minimal_document(&minimal_authority())
        .order()
        .to_vec();
    let assembled: Vec<SemanticItemIdV2> =
        multi.items().iter().map(CompiledItemV2::item_id).collect();
    assert_eq!(assembled, authored);
}

#[test]
fn group_partition_identity_and_sort_policy_change_the_assembled_plan() {
    let one_key = reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)
        .expect("one-key group resolves");
    let two_keys = reference_relation_document_v2(
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
    )
    .expect("two-key group resolves");
    let one = compile_resolved_document_v2(one_key.document()).expect("one-key group compiles");
    let two = compile_resolved_document_v2(two_keys.document()).expect("two-key group compiles");
    assert_ne!(
        one.items(),
        two.items(),
        "a swapped Group partition identity must change the assembled plan"
    );

    let ascending =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SortAsc).expect("sort resolves");
    let nulls_last =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SortNullsLastNullableField)
            .expect("nulls-last sort resolves");
    let asc = compile_resolved_document_v2(ascending.document()).expect("ascending sort compiles");
    let last = compile_resolved_document_v2(nulls_last.document()).expect("nulls-last compiles");
    assert_ne!(
        asc.items(),
        last.items(),
        "a dropped sort direction / null policy must change the assembled plan"
    );
}

#[test]
fn find_witness_retention_changes_the_assembled_plan() {
    let shallow = compile_graph(bounded_query(GraphQueryKindV1::Taint, 1));
    let deep = compile_graph(bounded_query(GraphQueryKindV1::Taint, 4));
    let shallow_plan =
        compile_resolved_document_v2(shallow.document()).expect("shallow FIND compiles");
    let deep_plan = compile_resolved_document_v2(deep.document()).expect("deep FIND compiles");
    assert_ne!(
        shallow_plan.items(),
        deep_plan.items(),
        "a lost FIND witness cap/path-length bound must change the assembled plan"
    );
    let CompiledItemV2::Planned(planned) = &shallow_plan.items()[0] else {
        panic!("a FIND item must compile to a planned graph item");
    };
    let PlannedItemV2::Graph { item_id, .. } = planned.as_ref() else {
        panic!("a FIND item retains its graph plan");
    };
    assert_eq!(*item_id, shallow.graph_item_id());
}

#[test]
fn nested_check_retains_its_own_graph_identity() {
    let fixture = reference_nested_check_document_v2(bounded_query(GraphQueryKindV1::Path, 2))
        .expect("nested CHECK fixture resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("CHECK compiles");
    assert_eq!(compiled.items().len(), 1);
    let CompiledItemV2::Planned(planned) = &compiled.items()[0] else {
        panic!("a CHECK item must compile to a planned CHECK item");
    };
    let PlannedItemV2::Check { item_id, plan } = planned.as_ref() else {
        panic!("a CHECK item retains its CHECK plan");
    };
    assert_eq!(*item_id, fixture.check_item_id());
    assert_ne!(*item_id, fixture.nested_graph_item_id());

    // The document's OWN sealed CHECK and the nested FIND it retains. Every
    // assertion below reads one of those facts (or an exact class derived from
    // them), never the plan against a payload destructured out of that same
    // plan.
    let Some(SemanticItemV2::Check(check)) = fixture.document().item(*item_id) else {
        panic!("the CHECK identity is the document's own item");
    };
    let nested = check
        .nested_graph()
        .expect("the sealed CHECK retains its nested confirm FIND");
    assert_eq!(nested.item_id(), fixture.nested_graph_item_id());

    // Derived lanes: `confirm=skip` keeps the source-location evidence and
    // `emit=findings` keeps the read-only effect and the alert render.
    assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(
        plan.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    assert_eq!(plan.evidence(), EvidenceRequirementV2::SourceLocation);
    assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
    assert_eq!(plan.result().result_kind(), ResultKindV2::Alert);
    assert_eq!(
        plan.result().render_requirement(),
        RenderRequirementV2::SourceLocation
    );
    assert_eq!(
        plan.result().witness_selection(),
        Some(WitnessSelectionPolicyV2::AllWitness),
        "the standard witness cap is wider than one, so every witness is materialized"
    );
    assert!(plan.result().multiplicity_order().eliminates_duplicates());
    assert!(!plan.result().multiplicity_order().is_ordered());

    // The visited nested FIND is a cyclic witness contract: witness
    // materialization and recursive closure, and the bound-dependent cost.
    assert_eq!(
        plan.capabilities(),
        &BTreeSet::from([
            CapabilityV2::WitnessMaterialization,
            CapabilityV2::RecursiveClosure,
        ])
    );
    assert_eq!(
        plan.semantic_cost(),
        &SemanticCostV2 {
            estimated_cardinality_class: CardinalityClassV2::Closure,
            recursion_risk_class: RecursionRiskClassV2::BoundDependent,
            witness_explosion_risk_class: WitnessExplosionRiskClassV2::CapBounded,
            ordering_contract: OrderingContractV2::Unordered,
        }
    );

    // Attribution is the CHECK item itself; a CHECK plan carries no algebra
    // family, but it IS recursive through its nested confirm FIND.
    assert_eq!(plan.explain().root_location(), check.location());
    assert!(plan.explain().families().is_empty());
    assert!(plan.explain().recursive());

    // The retained CHECK policy is the sealed CHECK's own payload, field for
    // field, plus the nested witness provenance of the sealed FIND.
    let PlanPolicyV2::Check(policy) = plan.policy() else {
        panic!("a CHECK plan must retain the CHECK policy");
    };
    assert_eq!(policy.mode(), check.mode());
    assert_eq!(policy.confirm(), check.confirm());
    assert_eq!(policy.emit(), check.emit());
    assert_eq!(policy.rule(), check.rule());
    assert_eq!(policy.scope(), check.scope());
    assert_eq!(policy.pattern(), check.pattern());
    assert_eq!(
        policy.rule().location().location(),
        LocatedSpanV2 {
            source_id: fixture.check_item_id().source_id(),
            span: SpanV1::new(100, 105),
        },
        "the retained rule reference keeps its authored `rule1` span"
    );
    let retention = policy
        .nested_graph()
        .expect("the nested semantic-confirm FIND is retained");
    assert_eq!(retention.item_id(), fixture.nested_graph_item_id());
    assert_ne!(retention.item_id(), *item_id);
    assert_eq!(retention.location(), nested.location());
    assert_eq!(retention.kind(), nested.kind());
    assert_eq!(retention.depth(), 2);
    assert_eq!(retention.shape(), RelationShapeV1::Cyclic);
    assert_eq!(
        retention.max_witnesses(),
        nested.witness_bounds().max_witnesses()
    );
    assert_eq!(retention.max_path_length(), Some(2));
}

#[test]
fn rule_program_receives_one_plan_and_exact_members() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("rules compile");
    let program = compiled
        .rule_program()
        .expect("a rule document has one program");
    let members: Vec<SemanticItemIdV2> = program
        .members()
        .iter()
        .map(|member| member.member_item_id())
        .collect();
    assert_eq!(members, fixture.item_ids());
    assert_eq!(program.program_id(), fixture.item_ids()[0]);
    assert!(program.stratification().stratum_count() > 0);
    assert_eq!(
        compiled.items().len(),
        fixture.item_ids().len(),
        "every authored rule item compiles to one item result"
    );
    for member in program.members() {
        assert_eq!(member.program_id(), program.program_id());
    }
    for item in compiled.items() {
        let CompiledItemV2::Planned(planned) = item else {
            panic!("every source rule compiles to a rule-program member reference");
        };
        let PlannedItemV2::RuleMember(member) = planned.as_ref() else {
            panic!("every source rule retains its member reference");
        };
        assert!(item.plan().is_none());
        assert_eq!(member.program_id(), program.program_id());
    }
}

/// F-06: the document rule graph is stratified exactly ONCE, at the program's
/// sole ingress, and every consumer reads THAT proof.
///
/// Before this rail's subject change the pipeline stratified the program itself
/// (charging the document's rule-graph allowance) and planning stratified it
/// again from the same receipt with a fresh counter, producing two independent
/// proofs of one member set. The assertions below pin the seam: the compiled
/// program's proof, the plan's retained rule policy, and the ingress contract's
/// proof are one and the same admission of the FULL projected program under the
/// document's own receipt (a re-derived proof over a subset, or under a
/// different budget, would not compare equal).
#[test]
fn the_rule_program_proof_is_the_single_ingress_stratification() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let document = fixture.document();
    let compiled = compile_resolved_document_v2(document).expect("rules compile");
    let program = compiled.rule_program().expect("one program");

    // The ONE admitted proof: `from_document` charged this document's own
    // rule-graph receipt, and the mutual pair is one recursive SCC.
    let contract = quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(document)
        .expect("the mutual pair is stratifiable");
    assert_eq!(contract.budget(), document.budget_receipt());
    assert_eq!(contract.budget().max_rule_graph_nodes(), 50_000);
    assert_eq!(contract.stratification().stratum_count(), 1);
    assert_eq!(contract.stratification().recursive().len(), 2);

    // The compiled DTO carries THAT proof, byte for byte.
    assert_eq!(program.stratification(), contract.stratification());

    // The plan's rule policy retains the same proof summary: the ordered member
    // identities, the stratum count, and the recursive member set of the ONE
    // ingress proof — not a second classifier's.
    let PlanPolicyV2::Rule(policy) = program.plan().policy() else {
        panic!("a rule program must retain the rule-program policy");
    };
    assert_eq!(policy.item_ids(), fixture.item_ids());
    assert_eq!(
        policy.stratum_count(),
        contract.stratification().stratum_count()
    );
    assert_eq!(
        policy.recursive_members(),
        contract
            .stratification()
            .recursive()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .as_slice()
    );

    // Non-vacuity: recomputing the stratification over the program the contract
    // projects, under the DOCUMENT's own receipt, reproduces the retained proof
    // exactly. The retained proof is therefore the full-program proof under the
    // document's single allowance.
    let recomputed = quanta_gqlang_v2_logic::stratify(
        &quanta_gqlang_v2_logic::RecursiveProgramV1::from_rules(contract.to_program()),
        &document.budget_receipt(),
    )
    .expect("the admitted member set re-stratifies identically");
    assert_eq!(contract.stratification(), &recomputed);
    assert_eq!(program.stratification(), &recomputed);
}

// ---------------------------------------------------------------------------
// Exact-key membership kills
// ---------------------------------------------------------------------------

#[test]
fn forged_rule_membership_rejects_with_the_logic_code() {
    for mutation in [
        ForgedRuleMembershipMutationV2::DroppedItem,
        ForgedRuleMembershipMutationV2::OrphanItem,
        ForgedRuleMembershipMutationV2::DuplicateOrderId,
    ] {
        let forged =
            forged_rule_membership_document_v2(mutation).expect("forged rule document builds");
        let error = compile_resolved_document_v2(forged.document())
            .expect_err("a forged rule membership must reject");
        let code = failure_code(&error);
        assert!(
            [
                DocumentAssemblyCodeV2::MissingItemResult.as_str(),
                DocumentAssemblyCodeV2::ExtraItemResult.as_str(),
                DocumentAssemblyCodeV2::DuplicateItemResult.as_str(),
                DocumentAssemblyCodeV2::RuleMembershipMismatch.as_str(),
                "logic.rule_program.missing_member",
                "logic.rule_program.duplicate_member",
                "logic.rule_program.membership_mismatch",
            ]
            .contains(&code.as_str()),
            "expected a stable membership code, got {code}"
        );
    }
}

#[test]
fn duplicate_authored_identity_rejects() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, _items| {
            let first = order[0];
            order.push(first);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("duplicate order rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::DuplicateItemResult.as_str()
    );
}

#[test]
fn missing_produced_item_rejects() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, items| {
            let first = order[0];
            items.remove(&first);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("missing item rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::MissingItemResult.as_str()
    );
}

#[test]
fn extra_produced_item_rejects() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, _items| {
            order.remove(0);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("extra item rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::ExtraItemResult.as_str()
    );
}

#[test]
fn foreign_authored_identity_rejects() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, _items| {
            let first = order[0];
            order[0] = SemanticItemIdV2::new(SourceIdV2::from_raw(77), first.hir_item_id());
        });
    let error = compile_resolved_document_v2(&forged).expect_err("foreign identity rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::ForeignItemResult.as_str()
    );
}

#[test]
fn same_name_wrong_id_substitution_rejects() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, items| {
            let first = order[0];
            let second = order[1];
            let first_item = items.remove(&first).expect("first item present");
            let second_item = items.remove(&second).expect("second item present");
            items.insert(first, second_item);
            items.insert(second, first_item);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("wrong-ID substitution rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::WrongKindItemResult.as_str()
    );
}

#[test]
fn check_graph_identity_substitution_rejects() {
    let fixture = reference_nested_check_document_v2(bounded_query(GraphQueryKindV1::Path, 2))
        .expect("nested CHECK fixture resolves");
    let check = fixture.check_item_id();
    let nested = fixture.nested_graph_item_id();
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, items| {
            let item = items.remove(&check).expect("check item present");
            items.insert(nested, item);
            order[0] = nested;
        });
    let error = compile_resolved_document_v2(&forged).expect_err("CHECK id substitution rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::WrongKindItemResult.as_str()
    );
}

// ---------------------------------------------------------------------------
// Budget: the sealed document receipt is the only authority
// ---------------------------------------------------------------------------

#[test]
fn sealed_receipt_admits_the_exact_limit_and_rejects_limit_plus_one() {
    // The forged document carries the small receipt while its chain was
    // resolved under the standard budget, so the algebra relation walk's own
    // charging is the only limit: a chain of depth d lowers d + 1 stages.
    let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let exact = forged_small_receipt_relation_chain_document_v2(exact_depth)
        .expect("exact-limit chain builds");
    compile_resolved_document_v2(exact.document()).expect("the exact sealed limit admits");

    let over = forged_small_receipt_relation_chain_document_v2(exact_depth + 1)
        .expect("over-limit builds");
    let error = compile_resolved_document_v2(over.document())
        .expect_err("one visit past the sealed receipt rejects");
    assert_eq!(failure_code(&error), "algebra.traversal_budget_exceeded");

    // The same depth resolved under the standard budget is NOT a limit: the
    // pipeline reads the document's receipt, never a caller-selected budget.
    let standard = deep_relation_chain_document_v2(exact_depth + 1, ReferenceBudgetV2::Standard)
        .expect("the standard budget admits the deeper chain");
    compile_resolved_document_v2(standard.document())
        .expect("the standard receipt admits what the small receipt refuses");
}

#[test]
fn sealed_expression_node_receipt_governs_the_expression_walk() {
    // Depth 1 charges seven of the eight small-receipt expression nodes; depth
    // 2 charges eleven and must reject one over.
    let exact =
        forged_small_receipt_bool_tree_document_v2(1).expect("exact-limit bool tree builds");
    compile_resolved_document_v2(exact.document()).expect("the exact node limit admits");

    let over = forged_small_receipt_bool_tree_document_v2(2).expect("over-limit bool tree builds");
    let error = compile_resolved_document_v2(over.document())
        .expect_err("the sealed expression-node receipt rejects the oversized walk");
    assert_eq!(failure_code(&error), "algebra.traversal_budget_exceeded");
}

// ---------------------------------------------------------------------------
// Public source entries
// ---------------------------------------------------------------------------

#[test]
fn closed_source_entry_assembles_the_exact_item_dispositions() {
    let authority = minimal_authority();
    let compiled = compile_source_document_v3(DOC_SOURCE, MINIMAL_SOURCE, &authority)
        .expect("the minimal document compiles");
    assert_eq!(compiled.source_id(), DOC_SOURCE);
    assert_eq!(compiled.items().len(), 3);
    assert!(compiled.rule_program().is_some());

    let dispositions: Vec<&'static str> = compiled
        .items()
        .iter()
        .map(|item| match item {
            CompiledItemV2::NoPlan { .. } => "no_plan",
            CompiledItemV2::Planned(planned) => match planned.as_ref() {
                PlannedItemV2::RuleMember(_) => "rule_member",
                PlannedItemV2::Presentation { .. } => "presentation",
                PlannedItemV2::Relation { .. }
                | PlannedItemV2::Graph { .. }
                | PlannedItemV2::Command { .. }
                | PlannedItemV2::Check { .. }
                | PlannedItemV2::Composition { .. }
                | PlannedItemV2::MetaRequireCertified { .. } => "planned",
            },
        })
        .collect();
    assert_eq!(dispositions, ["no_plan", "rule_member", "presentation"]);
    let first = &compiled.items()[0];
    assert_eq!(
        first.no_plan_reason(),
        Some(&NoPlanReasonV2::ValueDeclaration)
    );
    assert!(first.plan().is_none());
}

#[test]
fn authored_presentation_options_reach_the_exact_planned_policy() {
    let source = concat!(
        "sample query=\"callgraph\" | mode=semantic | limit=5 | scope=repo; ",
        "compare baseline=run.a | candidate=run.b | metric=semantic; ",
        "export source=result.latest | kind=sarif; ",
        "show capabilities; ",
        "describe rule::owasp.sqli;",
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("all five authored presentation requests compile");
    assert_eq!(compiled.items().len(), 5);
    assert_eq!(
        compiled
            .execution()
            .items()
            .iter()
            .map(|item| item.item_id())
            .collect::<Vec<_>>(),
        compiled
            .items()
            .iter()
            .map(CompiledItemV2::item_id)
            .collect::<Vec<_>>(),
        "V3 items keep the authored multi-item order",
    );
    assert!(compiled.execution().items().iter().all(|item| {
        item.family() == ExecutionFamilyV3::Presentation
            && item.relation().is_none()
            && item.composition().is_none()
    }));

    let policies: Vec<&ResolvedPresentationKindV2> = compiled
        .items()
        .iter()
        .map(|item| {
            let CompiledItemV2::Planned(planned) = item else {
                panic!("presentation source became a no-plan item: {item:?}");
            };
            let PlannedItemV2::Presentation { plan, .. } = planned.as_ref() else {
                panic!("presentation source became a different plan: {planned:?}");
            };
            let PlanPolicyV2::Presentation(kind) = plan.policy() else {
                panic!("presentation plan lost its policy: {:?}", plan.policy());
            };
            kind
        })
        .collect();

    let ResolvedPresentationKindV2::Sample {
        query_text,
        mode,
        limit,
        scope: Some(scope),
    } = policies[0]
    else {
        panic!(
            "authored SAMPLE fields were not retained: {:?}",
            policies[0]
        );
    };
    assert_eq!(query_text, "callgraph");
    assert_eq!(*mode, SampleModeV1::Semantic);
    assert_eq!(*limit, 5);
    assert_eq!(
        scope
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("repo")
    );
    let ResolvedPresentationKindV2::Compare {
        baseline,
        candidate,
        metric,
    } = policies[1]
    else {
        panic!(
            "authored COMPARE fields were not retained: {:?}",
            policies[1]
        );
    };
    assert_eq!(
        baseline
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("run.a")
    );
    assert_eq!(
        candidate
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("run.b")
    );
    assert_eq!(*metric, CompareMetricV1::Semantic);
    let ResolvedPresentationKindV2::Export {
        source,
        format,
        profile,
    } = policies[2]
    else {
        panic!(
            "authored EXPORT fields were not retained: {:?}",
            policies[2]
        );
    };
    assert_eq!(
        source
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("result.latest")
    );
    assert_eq!(*format, ExportFormatV1::Sarif);
    assert!(profile.is_none());
    assert!(matches!(
        policies[3],
        ResolvedPresentationKindV2::Show {
            category: ShowCategoryV1::Capabilities
        }
    ));
    let ResolvedPresentationKindV2::Describe { target } = policies[4] else {
        panic!("authored DESCRIBE target was not retained");
    };
    assert_eq!(
        target
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("owasp.sqli")
    );
    let issued = compiled.execution().items();
    assert!(
        matches!(issued[0].presentation(), Some(ExecutionPresentationKindV3::Sample {
        query_text, mode: ExecutionSampleModeV3::Semantic, limit: 5, scope: Some(_),
    }) if query_text == "callgraph")
    );
    assert!(
        matches!(issued[1].presentation(), Some(ExecutionPresentationKindV3::Compare {
        baseline: ExecutionGraphTargetV3::Host { spelling: baseline, .. },
        candidate: ExecutionGraphTargetV3::Host { spelling: candidate, .. },
        metric: ExecutionCompareMetricV3::Semantic,
    }) if baseline == "run.a" && candidate == "run.b")
    );
    assert!(matches!(
        issued[2].presentation(),
        Some(ExecutionPresentationKindV3::Export {
            format: ExecutionExportFormatV3::Sarif,
            profile: None,
            ..
        })
    ));
    assert!(matches!(
        issued[3].presentation(),
        Some(ExecutionPresentationKindV3::Show {
            category: ExecutionShowCategoryV3::Capabilities,
        })
    ));
    assert!(
        matches!(issued[4].presentation(), Some(ExecutionPresentationKindV3::Describe {
        target: ExecutionGraphTargetV3::Host { spelling, .. },
    }) if spelling == "owasp.sqli")
    );
}

#[test]
fn authored_nondefault_execution_plan_facts_survive_projection() {
    use quanta_gqlang_v2_pipeline::{
        ExecutionEffectV3, ExecutionEvidenceV3, ExecutionPolicyFamilyV3,
    };

    let source = concat!(
        "export source=result.latest | kind=sarif; ",
        "explain rule.sqli | evidence=trace | truth=confirm; ",
        "find taint api.entry -> db.write | sanitizers=authz.clean | direction=backward | depth=3; ",
        "check rule.hybrid | scope=repo | pattern=pattern.hybrid | confirm=require ",
        "| emit=sarif | mode=hybrid using find reachability src.entry -> sink.write | depth=8;",
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("existing source policy fixtures compile together");
    let expected = [
        (
            ExecutionFamilyV3::Presentation,
            ExecutionEffectV3::ExportOnly,
            ExecutionEvidenceV3::None,
            ExecutionPolicyFamilyV3::Presentation,
        ),
        (
            ExecutionFamilyV3::Presentation,
            ExecutionEffectV3::ReadOnly,
            ExecutionEvidenceV3::ProofTree,
            ExecutionPolicyFamilyV3::Presentation,
        ),
        (
            ExecutionFamilyV3::Graph,
            ExecutionEffectV3::ProofEvidenceOnly,
            ExecutionEvidenceV3::WitnessPayload,
            ExecutionPolicyFamilyV3::Graph,
        ),
        (
            ExecutionFamilyV3::Check,
            ExecutionEffectV3::ExportOnly,
            ExecutionEvidenceV3::ProofTree,
            ExecutionPolicyFamilyV3::Check,
        ),
    ];
    assert_eq!(compiled.items().len(), expected.len());
    assert_eq!(compiled.execution().items().len(), expected.len());
    for (index, (family, effect, evidence, policy_family)) in expected.into_iter().enumerate() {
        let item = &compiled.execution().items()[index];
        assert_eq!(item.item_id(), compiled.items()[index].item_id());
        assert_eq!(item.family(), family, "authored item {index}");
        assert_eq!(item.plan().effect(), effect, "authored item {index}");
        assert_eq!(item.plan().evidence(), evidence, "authored item {index}");
        assert_eq!(
            item.plan().policy_family(),
            policy_family,
            "authored item {index}"
        );
    }
    let items = compiled.execution().items();
    assert!(matches!(
        items[0].presentation(),
        Some(ExecutionPresentationKindV3::Export {
            format: ExecutionExportFormatV3::Sarif,
            ..
        })
    ));
    assert!(matches!(
        items[1].presentation(),
        Some(ExecutionPresentationKindV3::Explain {
            target: ExecutionExplainTargetV3::Entity(_),
            evidence: ExecutionEvidenceModeV3::Trace,
            truth: ExecutionTruthModeV3::Confirm,
        })
    ));
    let graph = items[2].graph().expect("standalone graph payload");
    assert_eq!(graph.kind(), ExecutionGraphKindV3::Taint);
    assert_eq!(graph.depth(), 3);
    assert_eq!(graph.direction(), Some(ExecutionGraphDirectionV3::Backward));
    assert_eq!(
        graph.references()[0].role(),
        ExecutionGraphReferenceRoleV3::Source
    );
    assert_eq!(
        graph.references()[1].role(),
        ExecutionGraphReferenceRoleV3::Sink
    );
    assert_eq!(
        graph.references()[2].role(),
        ExecutionGraphReferenceRoleV3::Sanitizer
    );
    let check = items[3].check().expect("standalone CHECK payload");
    assert_eq!(check.rule().module(), "app");
    assert_eq!(check.rule().name(), "rule.hybrid");
    assert!(
        matches!(check.scope(), Some(ExecutionGraphTargetV3::Host { spelling, .. }) if spelling == "repo")
    );
    assert!(
        matches!(check.pattern(), Some(ExecutionGraphTargetV3::Host { spelling, .. }) if spelling == "pattern.hybrid")
    );
    assert_eq!(check.confirm(), ExecutionConfirmModeV3::Require);
    assert_eq!(check.emit(), ExecutionEmitModeV3::Sarif);
    assert_eq!(check.mode(), ExecutionDetectionModeV3::Hybrid);
    let nested = check.nested_graph().expect("checked nested FIND");
    assert_ne!(nested.item_id(), items[3].item_id());
    assert_eq!(nested.query().kind(), ExecutionGraphKindV3::Reachability);
    assert_eq!(nested.query().depth(), 8);
    assert!(items[0].graph().is_none() && items[0].check().is_none());
    assert!(items[2].presentation().is_none() && items[3].graph().is_none());
}

#[test]
fn authored_find_check_and_explain_options_reach_the_exact_planned_policy() {
    let source = concat!(
        "find taint api.entry -> db.write | sanitizers=authz.clean | direction=backward | depth=3; ",
        "check rule.hybrid | scope=repo | pattern=pattern.hybrid | confirm=require ",
        "| emit=sarif | mode=hybrid using find reachability src.entry -> sink.write | depth=8; ",
        "explain rule.sqli | evidence=trace | truth=confirm;",
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("authored graph, check, and explain requests compile");
    assert_eq!(compiled.items().len(), 3);

    let CompiledItemV2::Planned(find) = &compiled.items()[0] else {
        panic!("FIND must be planned");
    };
    let PlannedItemV2::Graph { plan, .. } = find.as_ref() else {
        panic!("FIND must keep its graph plan");
    };
    let PlanPolicyV2::Graph(witness) = plan.policy() else {
        panic!("FIND must keep its graph witness policy");
    };
    assert_eq!(witness.kind(), GraphQueryKindV1::Taint);
    assert_eq!(witness.depth(), 3);
    assert_eq!(witness.direction(), Some(GraphDirectionV2::Backward));
    assert_eq!(witness.references().len(), 3);
    assert!(witness
        .references()
        .iter()
        .any(|reference| reference.role() == GraphReferenceRoleV2::Sanitizer));

    let CompiledItemV2::Planned(check) = &compiled.items()[1] else {
        panic!("CHECK must be planned");
    };
    let PlannedItemV2::Check { plan, .. } = check.as_ref() else {
        panic!("CHECK must keep its check plan");
    };
    let PlanPolicyV2::Check(policy) = plan.policy() else {
        panic!("CHECK must keep its option policy");
    };
    assert_eq!(policy.mode(), DetectionModeV1::Hybrid);
    assert_eq!(policy.confirm(), ConfirmModeV1::Require);
    assert_eq!(policy.emit(), EmitModeV1::Sarif);
    assert_eq!(policy.rule().id().local_name(), "rule.hybrid");
    assert_eq!(
        policy
            .scope()
            .and_then(|use_site| use_site.target().host_selector())
            .map(|target| target.spelling()),
        Some("repo")
    );
    assert_eq!(
        policy
            .pattern()
            .and_then(|use_site| use_site.target().host_selector())
            .map(|target| target.spelling()),
        Some("pattern.hybrid")
    );
    let nested = policy
        .nested_graph()
        .expect("semantic confirm FIND retained");
    assert_eq!(nested.kind(), GraphQueryKindV1::Reachability);
    assert_eq!(nested.depth(), 8);

    let CompiledItemV2::Planned(explain) = &compiled.items()[2] else {
        panic!("EXPLAIN must be planned");
    };
    let PlannedItemV2::Presentation { plan, .. } = explain.as_ref() else {
        panic!("EXPLAIN must keep its presentation plan");
    };
    let PlanPolicyV2::Presentation(ResolvedPresentationKindV2::Explain {
        target,
        evidence,
        truth,
    }) = plan.policy()
    else {
        panic!("EXPLAIN must keep its evidence/truth policy");
    };
    assert_eq!(*evidence, EvidenceModeV1::Trace);
    assert_eq!(*truth, TruthModeV1::Confirm);
    let ResolvedExplainTargetV2::Entity(target) = target else {
        panic!("EXPLAIN must keep its resolved entity target");
    };
    assert_eq!(
        target
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("rule.sqli")
    );
}

#[test]
fn find_selectors_and_summary_facets_survive_source_to_final_plan() {
    let taint = compile_source_document_v3(
        DOC_SOURCE,
        concat!(
            "find taint api.entry -> db.write | via=mid.one,mid.two ",
            "| through=must.pass | sanitizers=authz.clean | scope=repo ",
            "| window=recent | direction=backward | frontier=edge | depth=3",
        ),
        &source_policy_authority(),
    )
    .expect("fully selected FIND compiles");
    let CompiledItemV2::Planned(item) = &taint.items()[0] else {
        panic!("FIND must be planned");
    };
    let PlanPolicyV2::Graph(retention) = item.plan().expect("FIND plan").policy() else {
        panic!("FIND must retain graph policy");
    };
    assert_eq!(retention.direction(), Some(GraphDirectionV2::Backward));
    assert!(retention.summary_facets().is_empty());
    let references = retention
        .references()
        .iter()
        .map(|reference| {
            (
                reference.role(),
                reference
                    .target()
                    .host_selector()
                    .expect("source fixture uses host selectors")
                    .spelling(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        [
            (GraphReferenceRoleV2::Source, "api.entry"),
            (GraphReferenceRoleV2::Sink, "db.write"),
            (GraphReferenceRoleV2::Scope, "repo"),
            (GraphReferenceRoleV2::Window, "recent"),
            (GraphReferenceRoleV2::Frontier, "edge"),
            (GraphReferenceRoleV2::Via, "mid.one"),
            (GraphReferenceRoleV2::Via, "mid.two"),
            (GraphReferenceRoleV2::Through, "must.pass"),
            (GraphReferenceRoleV2::Sanitizer, "authz.clean"),
        ]
    );

    let summary = compile_source_document_v3(
        DOC_SOURCE,
        "find summary | callable=svc.handler | facets=calls,reads,writes | direction=forward | depth=4",
        &source_policy_authority(),
    )
    .expect("summary FIND compiles");
    let CompiledItemV2::Planned(item) = &summary.items()[0] else {
        panic!("summary FIND must be planned");
    };
    let PlanPolicyV2::Graph(retention) = item.plan().expect("summary plan").policy() else {
        panic!("summary FIND must retain graph policy");
    };
    assert_eq!(retention.direction(), Some(GraphDirectionV2::Forward));
    assert_eq!(retention.summary_facets(), ["calls", "reads", "writes"]);
    assert_eq!(retention.references().len(), 1);
    assert_eq!(
        retention.references()[0].role(),
        GraphReferenceRoleV2::Callable
    );
}

#[test]
fn inline_view_is_algebra_checked_before_its_catalog_no_plan_outcome() {
    let source = "view active | select=from records r | project r.id as id";
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect("inline View traverses Sema and Algebra before catalog disposition");
    assert!(matches!(
        compiled.items(),
        [CompiledItemV2::NoPlan {
            reason: NoPlanReasonV2::CatalogDeclaration { .. },
            ..
        }]
    ));

    // Negative control: an inline View whose `select` body names an unknown
    // relation must be rejected by Sema before any catalog non-plan outcome, so
    // the positive case cannot stay green if the body traversal is dropped.
    let unresolved = "view active | select=from missing_relation r | project r.id as id";
    let error = compile_source_document_v3(DOC_SOURCE, unresolved, &authority_with_records())
        .expect_err("an unresolvable view body must not reach catalog disposition");
    assert_eq!(failure_code(&error), "sema.unresolved_reference");
}

#[test]
fn duplicate_authored_projection_alias_rejects_before_a_plan_is_issued() {
    let source = "from records r | project r.id as id, r.id as id";
    let error = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect_err("duplicate authored projection names cannot produce a checked plan");
    assert_eq!(failure_code(&error), "binder.duplicate_declaration");
}

#[test]
fn analyze_and_compile_share_the_same_stage_payloads() {
    let authority = minimal_authority();
    let document = resolve_minimal_document(&authority);
    let compiled = compile_resolved_document_v2(&document).expect("the minimal document compiles");
    let analyzed = quanta_gqlang_v2_pipeline::analyze_resolved_document_v2(&document)
        .expect("sealed analysis succeeds");
    assert_eq!(analyzed.source_id(), compiled.source_id());
    assert_eq!(analyzed.order(), document.order());
    assert_eq!(analyzed.items().len(), compiled.items().len());
    assert!(
        !compiled.items().is_empty(),
        "the minimal document must compile at least one item to compare"
    );
    for index in 0..compiled.items().len() {
        match (&compiled.items()[index], &analyzed.items()[index]) {
            (CompiledItemV2::Planned(compiled_item), AnalysisItemV2::Complete(analyzed_item)) => {
                assert_eq!(compiled_item, analyzed_item);
            }
            (
                CompiledItemV2::NoPlan { reason, .. },
                AnalysisItemV2::NotApplicable(analyzed_reason),
            ) => {
                assert_eq!(reason, analyzed_reason);
            }
            _ => panic!("compile and analysis item dispositions must agree"),
        }
        assert!(
            analyzed.item(analyzed.order()[index]).is_some(),
            "every authored identity is addressable in the analysis document"
        );
    }
}
