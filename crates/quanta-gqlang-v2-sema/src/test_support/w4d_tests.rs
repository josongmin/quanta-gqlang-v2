#![expect(
    clippy::indexing_slicing,
    clippy::panic_in_result_fn,
    clippy::wildcard_enum_match_arm,
    reason = "owner assertions pin exact fixture shapes, index pinned producer vectors, and \
              name the payload vocabulary they reject"
)]

use super::*;

// ---- meta ----------------------------------------------------------

#[test]
fn require_certified_meta_document_seals_exactly_one_directive() -> Result<(), String> {
    let document = reference_meta_document_v2(ReferenceMetaFixtureV2::RequireCertified)?;
    assert_eq!(document.document().items().count(), 1);
    assert!(matches!(
        document.kind(),
        ResolvedMetaKindV2::RequireCertified
    ));
    assert!(matches!(
        document.document().item(document.meta_item_id()),
        Some(SemanticItemV2::Meta(_))
    ));
    assert_eq!(document.document().order(), [document.meta_item_id()]);
    Ok(())
}

#[test]
fn use_pack_meta_document_seals_through_the_real_transaction() -> Result<(), String> {
    let document = reference_meta_document_v2(ReferenceMetaFixtureV2::UsePack)?;
    match document.kind() {
        ResolvedMetaKindV2::UsePack { pack, activation } => {
            assert_eq!(pack.as_str(), REFERENCE_USE_PACK_ID_V2);
            assert_eq!(*activation, UsePackActivationV1::Session);
        }
        ResolvedMetaKindV2::RequireCertified => {
            return Err("the fixture must retain USE PACK".to_owned())
        }
    }
    Ok(())
}

#[test]
fn forged_use_pack_meta_document_seals_the_resolved_use_pack_kind() -> Result<(), String> {
    let document = forged_use_pack_meta_document_v2()?;
    match document.kind() {
        ResolvedMetaKindV2::UsePack { pack, activation } => {
            assert_eq!(pack.as_str(), REFERENCE_USE_PACK_ID_V2);
            assert_eq!(*activation, UsePackActivationV1::Session);
        }
        ResolvedMetaKindV2::RequireCertified => {
            return Err("the forged fixture seals a USE PACK payload".to_owned())
        }
    }
    assert_eq!(document.document().items().count(), 1);
    assert_eq!(document.document().order(), [document.meta_item_id()]);
    Ok(())
}

// ---- presentation --------------------------------------------------

#[test]
fn every_presentation_fixture_resolves() -> Result<(), String> {
    for fixture in [
        ReferencePresentationFixtureV2::Show,
        ReferencePresentationFixtureV2::Describe,
        ReferencePresentationFixtureV2::Sample,
        ReferencePresentationFixtureV2::SampleWithScope,
        ReferencePresentationFixtureV2::Compare,
        ReferencePresentationFixtureV2::Export,
    ] {
        let document = reference_presentation_document_v2(fixture)?;
        assert_eq!(document.document().items().count(), 1);
        assert_eq!(
            document.presentation().item_id(),
            document.presentation_item_id()
        );
    }
    Ok(())
}

#[test]
fn every_explain_evidence_truth_permutation_resolves() -> Result<(), String> {
    const EVIDENCE: [EvidenceModeV1; 4] = [
        EvidenceModeV1::Auto,
        EvidenceModeV1::Trace,
        EvidenceModeV1::Citation,
        EvidenceModeV1::Counterexample,
    ];
    const TRUTH: [TruthModeV1; 3] = [
        TruthModeV1::Inspect,
        TruthModeV1::Confirm,
        TruthModeV1::Refute,
    ];
    let mut seen = 0_u8;
    for evidence in EVIDENCE {
        for truth in TRUTH {
            let document =
                reference_presentation_document_v2(ReferencePresentationFixtureV2::Explain {
                    evidence,
                    truth,
                })?;
            match document.presentation().kind() {
                crate::ResolvedPresentationKindV2::Explain {
                    target,
                    evidence: sealed_evidence,
                    truth: sealed_truth,
                } => {
                    assert_eq!(*sealed_evidence, evidence);
                    assert_eq!(*sealed_truth, truth);
                    let crate::ResolvedExplainTargetV2::Entity(target) = target else {
                        return Err(
                            "EXPLAIN must resolve the authored host entity selector".to_owned()
                        );
                    };
                    let selector = target
                        .target()
                        .host_selector()
                        .ok_or("EXPLAIN target must stay host-owned")?;
                    assert_eq!(
                        selector.spelling(),
                        REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2
                    );
                    assert_eq!(selector.kind(), GraphReferenceKindV2::Entity);
                    assert_eq!(
                        selector.entity_subtype(),
                        Some(REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2)
                    );
                    assert_eq!(target.location().location().span, SpanV1::new(8, 15));
                }
                other => return Err(format!("EXPLAIN payload expected, got {other:?}")),
            }
            seen += 1;
        }
    }
    assert_eq!(seen, 12, "every evidence/truth permutation is reachable");
    Ok(())
}

#[test]
fn presentation_payloads_retain_their_exact_facts() -> Result<(), String> {
    let assert_use = |reference: &crate::ResolvedGraphUseV2,
                      spelling: &str,
                      kind: GraphReferenceKindV2,
                      entity_subtype: Option<&str>,
                      span: SpanV1|
     -> Result<(), String> {
        let target = reference
            .target()
            .host_selector()
            .ok_or("expected a host presentation selector")?;
        assert_eq!(target.spelling(), spelling);
        assert_eq!(target.kind(), kind);
        assert_eq!(target.entity_subtype(), entity_subtype);
        assert_eq!(
            reference.location().location(),
            LocatedSpanV2 {
                source_id: REFERENCE_SOURCE_V2,
                span,
            }
        );
        Ok(())
    };
    let show = reference_presentation_document_v2(ReferencePresentationFixtureV2::Show)?;
    assert!(matches!(
        show.presentation().kind(),
        crate::ResolvedPresentationKindV2::Show {
            category: ShowCategoryV1::Capabilities
        }
    ));

    let describe = reference_presentation_document_v2(ReferencePresentationFixtureV2::Describe)?;
    let crate::ResolvedPresentationKindV2::Describe { target } = describe.presentation().kind()
    else {
        return Err("expected DESCRIBE presentation".to_owned());
    };
    assert_use(
        target,
        REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2,
        GraphReferenceKindV2::Entity,
        Some(REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2),
        SpanV1::new(17, 24),
    )?;

    let sample = reference_presentation_document_v2(ReferencePresentationFixtureV2::Sample)?;
    let crate::ResolvedPresentationKindV2::Sample {
        query_text,
        mode: SampleModeV1::Lexical,
        limit,
        scope: None,
    } = sample.presentation().kind()
    else {
        return Err("expected an unscoped LEXICAL SAMPLE presentation".to_owned());
    };
    assert_eq!(query_text.as_str(), "auth");
    assert_eq!(*limit, REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2);
    let sample_with_scope =
        reference_presentation_document_v2(ReferencePresentationFixtureV2::SampleWithScope)?;
    let crate::ResolvedPresentationKindV2::Sample {
        query_text,
        mode: SampleModeV1::Lexical,
        scope: Some(scope),
        limit: REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2,
    } = sample_with_scope.presentation().kind()
    else {
        return Err("expected scoped SAMPLE presentation".to_owned());
    };
    assert_eq!(query_text.as_str(), "auth");
    assert_use(
        scope,
        REFERENCE_PRESENTATION_SCOPE_SELECTOR_V2,
        GraphReferenceKindV2::Scope,
        None,
        SpanV1::new(20, 27),
    )?;

    let compare = reference_presentation_document_v2(ReferencePresentationFixtureV2::Compare)?;
    let crate::ResolvedPresentationKindV2::Compare {
        baseline,
        candidate,
        metric: CompareMetricV1::Structural,
    } = compare.presentation().kind()
    else {
        return Err("expected structural COMPARE presentation".to_owned());
    };
    assert_use(
        baseline,
        REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2,
        GraphReferenceKindV2::Snapshot,
        None,
        SpanV1::new(17, 27),
    )?;
    assert_use(
        candidate,
        REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2,
        GraphReferenceKindV2::Snapshot,
        None,
        SpanV1::new(30, 41),
    )?;

    let export = reference_presentation_document_v2(ReferencePresentationFixtureV2::Export)?;
    let crate::ResolvedPresentationKindV2::Export {
        source,
        format: ExportFormatV1::Sarif,
        profile: None,
    } = export.presentation().kind()
    else {
        return Err("expected SARIF EXPORT presentation".to_owned());
    };
    assert_use(
        source,
        REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2,
        GraphReferenceKindV2::ExportSource,
        None,
        SpanV1::new(14, 27),
    )?;
    Ok(())
}

// ---- composition ---------------------------------------------------

#[test]
fn every_composition_body_fixture_resolves_with_ordered_producers() -> Result<(), String> {
    for body in [
        ReferenceCompositionBodyV2::Relational,
        ReferenceCompositionBodyV2::Graph,
        ReferenceCompositionBodyV2::Check,
        ReferenceCompositionBodyV2::Explain,
    ] {
        let document = reference_composition_document_v2(body)?;
        assert_eq!(
            document.document().order(),
            [document.composition_item_id()]
        );
        assert!(!document.producer_item_ids().is_empty());
        assert_ne!(document.body_item_id(), document.composition_item_id());
        let composition = document.composition();
        assert_eq!(composition.item_id(), document.composition_item_id());
        assert_eq!(composition.body().item_id(), document.body_item_id());
        assert_eq!(composition.with_target(), document.with_binding());
        let selected = composition
            .bindings()
            .iter()
            .filter(|binding| binding.binding() == document.with_binding())
            .count();
        assert_eq!(selected, 1, "WITH must select exactly one ordered producer");
        for (ordinal, binding) in composition.bindings().iter().enumerate() {
            assert_eq!(binding.binding().ordinal(), ordinal);
            assert_eq!(
                binding.producer().item_id(),
                document.producer_item_ids()[ordinal]
            );
            assert!(binding.relation_output().is_some());
        }
    }
    Ok(())
}

#[test]
fn relational_composition_body_consumes_the_working_set() -> Result<(), String> {
    let document = reference_composition_document_v2(ReferenceCompositionBodyV2::Relational)?;
    assert_eq!(document.producer_item_ids().len(), 2);
    let composition = document.composition();
    let body = match composition.body() {
        crate::ResolvedCompositionBodyV2::Query(query) => query,
        other => {
            return Err(format!(
                "relational composition body expected, got {other:?}"
            ))
        }
    };
    let scan = body
        .root()
        .inputs()
        .next()
        .ok_or("the body projection must consume a working set")?;
    let source = scan
        .source()
        .ok_or("the working-set scan must retain its resolved source")?;
    assert!(matches!(source, ResolvedScanSourceV2::Composition(_)));
    assert_eq!(
        composition.bindings()[0].name(),
        composition.with_target().canonical_name()
    );
    Ok(())
}

#[test]
fn composition_bodies_keep_their_closed_family() -> Result<(), String> {
    let graph = reference_composition_document_v2(ReferenceCompositionBodyV2::Graph)?;
    assert!(matches!(
        graph.composition().body(),
        crate::ResolvedCompositionBodyV2::Graph(_)
    ));
    let check = reference_composition_document_v2(ReferenceCompositionBodyV2::Check)?;
    assert!(matches!(
        check.composition().body(),
        crate::ResolvedCompositionBodyV2::Check(_)
    ));
    let explain = reference_composition_document_v2(ReferenceCompositionBodyV2::Explain)?;
    match explain.composition().body() {
        crate::ResolvedCompositionBodyV2::Explain(presentation) => {
            assert!(matches!(
                presentation.kind(),
                crate::ResolvedPresentationKindV2::Explain { .. }
            ));
        }
        other => return Err(format!("EXPLAIN composition body expected, got {other:?}")),
    }
    Ok(())
}

// ---- relation shapes -----------------------------------------------

#[test]
fn multiplicative_right_semi_input_fixture_resolves() -> Result<(), String> {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight)?;
    let root = exists_root(reference.document())?;
    let nested = reference
        .document()
        .nested_relation(root)
        .ok_or("the EXISTS fixture must register its nested relation")?;
    assert!(matches!(nested, TypedRelationExprV2::Join(_)));
    assert_eq!(nested.join_mode(), Some(JoinModeV2::Cross));
    assert_eq!(nested.inputs().count(), 2);
    Ok(())
}

// ---- small-receipt rule documents ----------------------------------

#[test]
fn forged_small_receipt_rule_document_carries_the_small_budget() -> Result<(), String> {
    let document =
        forged_small_receipt_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)?;
    assert_eq!(
        document.budget().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );
    assert_eq!(document.item_ids().len(), 2);
    assert_eq!(
        reference_rule_program_charge_v2(2),
        3,
        "two members charge one program plus two member steps"
    );
    Ok(())
}

#[test]
fn rule_program_receipt_is_exact_at_the_sealed_limit_and_one_over() -> Result<(), String> {
    let exact =
        forged_small_receipt_rule_program_document_v2(SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS)?;
    assert_eq!(
        exact.budget().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );
    assert_eq!(
        u32::try_from(exact.item_ids().len())
            .map_err(|_error| "member count exceeds u32".to_owned())?,
        SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS
    );
    assert_eq!(
        reference_rule_program_charge_v2(SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );

    let over =
        forged_small_receipt_rule_program_document_v2(SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS)?;
    assert_eq!(
        over.budget().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS,
        "the one-over document must still carry the sealed small receipt"
    );
    assert_eq!(
        reference_rule_program_charge_v2(SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS + 1
    );
    assert_eq!(
        u32::try_from(over.item_ids().len())
            .map_err(|_error| "member count exceeds u32".to_owned())?,
        SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS
    );
    assert!(forged_small_receipt_rule_program_document_v2(0).is_err());
    Ok(())
}

// ---- narrow witness and alternate graph identity -------------------

#[test]
fn narrow_witness_document_seals_a_one_witness_cap() -> Result<(), String> {
    let document = reference_narrow_witness_graph_document_v2(GraphQueryKindV1::Path, 4)?;
    assert_eq!(
        document.graph().witness_bounds().max_witnesses(),
        NARROW_WITNESS_MAX_WITNESSES_V2
    );
    assert_eq!(document.graph().kind(), GraphQueryKindV1::Path);
    assert_eq!(document.graph().depth(), Some(4));
    Ok(())
}

#[test]
fn alternate_graph_document_seals_a_distinct_node_identity() -> Result<(), String> {
    let request = |kind| ReferenceGraphRequestV2 {
        kind,
        references: reference_witness_graph_references_v2(kind),
        depth: Some(4),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    };
    let primary = reference_graph_document_v2(request(GraphQueryKindV1::Taint))?;
    let alternate = reference_alternate_graph_document_v2(request(GraphQueryKindV1::Taint))?;
    assert_ne!(alternate.graph_item_id(), primary.graph_item_id());
    assert_eq!(
        alternate.graph_item_id().hir_item_id(),
        HirNodeIdV1::from_raw(ALTERNATE_GRAPH_NODE_ID_V2)
    );
    assert_eq!(
        primary.graph_item_id().hir_item_id(),
        HirNodeIdV1::from_raw(2)
    );
    Ok(())
}

#[test]
fn second_source_graph_document_has_a_foreign_source_qualified_identity() -> Result<(), String> {
    let primary = reference_graph_document_v2(ReferenceGraphRequestV2 {
        kind: GraphQueryKindV1::Taint,
        references: reference_witness_graph_references_v2(GraphQueryKindV1::Taint),
        depth: Some(4),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    })?;
    let secondary = second_source_graph_document_v2()?;
    assert_eq!(
        secondary.document().source_id(),
        SECONDARY_REFERENCE_SOURCE_V2
    );
    assert_eq!(
        secondary.graph_item_id().source_id(),
        SECONDARY_REFERENCE_SOURCE_V2
    );
    assert_ne!(secondary.graph_item_id(), primary.graph_item_id());
    assert_eq!(
        secondary.graph_item_id().hir_item_id(),
        primary.graph_item_id().hir_item_id()
    );
    assert_eq!(secondary.graph().kind(), primary.graph().kind());
    assert_eq!(secondary.graph().depth(), primary.graph().depth());
    Ok(())
}

#[test]
fn configurable_check_fixture_seals_the_full_mode_option_matrix() -> Result<(), String> {
    for mode in [
        DetectionModeV1::PatternOnly,
        DetectionModeV1::SemanticOnly,
        DetectionModeV1::Hybrid,
    ] {
        for with_pattern in [false, true] {
            for with_nested_graph in [false, true] {
                let request = ReferenceGraphRequestV2 {
                    kind: GraphQueryKindV1::Taint,
                    references: reference_witness_graph_references_v2(GraphQueryKindV1::Taint),
                    depth: Some(4),
                    direction: None,
                    summary_facets: Vec::new(),
                    item_span: (0, 80),
                };
                let fixture = reference_check_document_v2(
                    request,
                    ReferenceCheckFixtureV2 {
                        mode,
                        with_pattern,
                        with_nested_graph,
                    },
                )?;
                let check = fixture.check();
                assert_eq!(check.mode(), mode);
                assert_eq!(check.pattern().is_some(), with_pattern);
                assert_eq!(check.nested_graph().is_some(), with_nested_graph);
                assert_eq!(
                    check
                        .nested_graph()
                        .map(crate::ResolvedGraphQueryV2::item_id),
                    fixture.nested_graph_item_id()
                );
                assert_eq!(check.item_id(), fixture.check_item_id());
            }
        }
    }
    Ok(())
}
