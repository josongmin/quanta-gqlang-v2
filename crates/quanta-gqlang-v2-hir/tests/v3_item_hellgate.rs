//! v3 unified program-item hellgate rails.
//!
//! Proves the unified [`ProgramItemHirV1`] model reaches structural parity with
//! the 18 v2 root families through the ONE shared HIR pipeline: every family is
//! built via [`HirBuilderV1`], wrapped as a `ProgramItemHirV1`, and validated by
//! the shared `check_tree` integrity gate (unique ids, in-bounds spans). It also
//! forces a fail-closed integrity reject on a malformed new node and proves
//! `ProgramItemHirV1` nesting via a Composition embedding a Query producer plus a
//! FIND body.

#![allow(clippy::unwrap_used, reason = "tests assert on Result directly")]
#![allow(clippy::panic, reason = "tests panic on unexpected variants")]
#![allow(
    clippy::expect_used,
    reason = "tests use expect for variant extraction"
)]
#![allow(clippy::indexing_slicing, reason = "tests index known-length vecs")]

use quanta_gqlang_v2_hir::{
    check_tree, ActivationV1, AdmissionV1, CatalogPayloadV1, CompareMetricV1, CompositionBodyV1,
    CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1, EvidenceModeV1,
    ExportFormatV1, GateBindingV1, GraphConstraintsV1, GraphDirectionV1, GraphEndpointsV1,
    GraphQueryKindV1, GraphRefV1, HirBuilderV1, HirIntegrityErrorV1, HirNodeIdV1, HirRootRefV1,
    MetaKindV1, PresentationKindV1, ProgramItemHirV1, SampleModeV1, ShowCategoryV1, SpanV1,
    TruthModeV1, UsePackActivationV1, UsePackHirV1, ViewBodyV1,
};

/// Walk `item` through the shared integrity gate, asserting it is sound against
/// `source_len` and reports exactly `expected_nodes` id-bearing nodes.
fn assert_sound(item: &ProgramItemHirV1, source_len: usize, expected_nodes: usize) {
    let checked = check_tree(HirRootRefV1::Item(item), source_len)
        .unwrap_or_else(|e| panic!("expected sound item, got {e}"));
    assert_eq!(
        checked.node_count(),
        expected_nodes,
        "node count mismatch for {item:?}"
    );
}

fn gref(name: &str, start: usize, end: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.into(),
        span: SpanV1::new(start, end),
    }
}

// ---- SELECT (Query) --------------------------------------------------------

#[test]
fn select_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, SpanV1::new(0, 5));
    let query = b.query(users, None, SpanV1::new(0, 5));
    let item: ProgramItemHirV1 = query.into();
    assert!(matches!(item, ProgramItemHirV1::Query(_)));
    // relation source + query unit = 2 id-bearing nodes.
    assert_sound(&item, 16, 2);
}

// ---- FIND (GraphQuery) -----------------------------------------------------

#[test]
fn find_family_taint_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("src", 5, 8)),
        sink: Some(gref("sink", 12, 16)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        sanitizers: vec![gref("clean", 20, 25)],
        direction: Some(GraphDirectionV1 {
            keyword: "forward".into(),
        }),
        depth: Some(7),
        summary_facets: vec!["calls".into()],
        ..GraphConstraintsV1::default()
    };
    let find = b.graph_query(
        GraphQueryKindV1::Taint,
        endpoints,
        constraints,
        SpanV1::new(0, 30),
    );
    assert_eq!(find.kind, GraphQueryKindV1::Taint);
    assert_eq!(find.constraints.depth, Some(7));
    let item: ProgramItemHirV1 = find.into();
    // a graph query is a single id-bearing node (endpoints/refs carry no ids).
    assert_sound(&item, 30, 1);
}

// ---- CHECK (Legality) ------------------------------------------------------

#[test]
fn check_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let legality = b.legality(
        gref("rules.noEval", 6, 18),
        Some(gref("scope.module", 25, 37)),
        None,
        ConfirmModeV1::Require,
        EmitModeV1::Sarif,
        DetectionModeV1::Hybrid,
        SpanV1::new(0, 40),
    );
    assert_eq!(legality.confirm, ConfirmModeV1::Require);
    assert_eq!(legality.emit, EmitModeV1::Sarif);
    assert_eq!(legality.mode, DetectionModeV1::Hybrid);
    let item: ProgramItemHirV1 = legality.into();
    assert_sound(&item, 40, 1);
}

// ---- EXPLAIN / SHOW / DESCRIBE / SAMPLE / COMPARE / EXPORT (Presentation) ---

#[test]
fn explain_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let pres = b.presentation(
        PresentationKindV1::Explain {
            target_ref: gref("finding.1", 8, 17),
            evidence: EvidenceModeV1::Counterexample,
            truth: TruthModeV1::Refute,
        },
        SpanV1::new(0, 40),
    );
    let item: ProgramItemHirV1 = pres.into();
    assert_sound(&item, 40, 1);
}

#[test]
fn show_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let pres = b.presentation(
        PresentationKindV1::Show {
            category: ShowCategoryV1::Capabilities,
        },
        SpanV1::new(0, 18),
    );
    let item: ProgramItemHirV1 = pres.into();
    assert_sound(&item, 18, 1);
}

#[test]
fn describe_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let pres = b.presentation(
        PresentationKindV1::Describe {
            kind: "function".into(),
            path: gref("app::main", 9, 18),
        },
        SpanV1::new(0, 20),
    );
    let item: ProgramItemHirV1 = pres.into();
    assert_sound(&item, 20, 1);
}

#[test]
fn sample_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let pres = b.presentation(
        PresentationKindV1::Sample {
            query_text: "sql injection".into(),
            mode: SampleModeV1::Hybrid,
            limit: 10,
            scope: Some(gref("repo", 30, 34)),
        },
        SpanV1::new(0, 40),
    );
    let item: ProgramItemHirV1 = pres.into();
    assert_sound(&item, 40, 1);
}

#[test]
fn compare_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let pres = b.presentation(
        PresentationKindV1::Compare {
            baseline_ref: gref("base", 8, 12),
            candidate_ref: gref("cand", 20, 24),
            metric: CompareMetricV1::Structural,
        },
        SpanV1::new(0, 30),
    );
    let item: ProgramItemHirV1 = pres.into();
    assert_sound(&item, 30, 1);
}

#[test]
fn export_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let pres = b.presentation(
        PresentationKindV1::Export {
            source_ref: gref("findings", 7, 15),
            format: ExportFormatV1::Sarif,
            profile: None,
        },
        SpanV1::new(0, 30),
    );
    let item: ProgramItemHirV1 = pres.into();
    assert_sound(&item, 30, 1);
}

// ---- VIEW / MODEL / PACK / PROFILE / GATE / RULE (CatalogDecl) --------------

#[test]
fn view_inline_select_is_sound_and_nests_query() {
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, SpanV1::new(20, 25));
    let inline = b.query(users, None, SpanV1::new(20, 25));
    let decl = b.catalog_decl(
        "active_users",
        SpanV1::new(5, 17),
        CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(inline))),
        SpanV1::new(0, 25),
    );
    assert_eq!(decl.name, "active_users");
    let item: ProgramItemHirV1 = decl.into();
    // catalog decl + inline query unit + relation source = 3 id-bearing nodes,
    // proving the integrity gate recurses into the embedded inline SELECT.
    assert_sound(&item, 25, 3);
}

#[test]
fn view_query_ref_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let decl = b.catalog_decl(
        "v",
        SpanV1::new(5, 6),
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("base_q", 10, 16))),
        SpanV1::new(0, 16),
    );
    let item: ProgramItemHirV1 = decl.into();
    assert_sound(&item, 16, 1);
}

#[test]
fn model_pack_profile_gate_rule_decls_are_sound() {
    let b = HirBuilderV1::new();

    let model = b.catalog_decl(
        "gpt",
        SpanV1::new(6, 9),
        CatalogPayloadV1::Model {
            provider: "openai".into(),
            source: gref("endpoint", 18, 26),
        },
        SpanV1::new(0, 30),
    );
    assert_sound(&model.into(), 30, 1);

    let pack = b.catalog_decl(
        "owasp",
        SpanV1::new(5, 10),
        CatalogPayloadV1::Pack {
            source: gref("registry", 15, 23),
            activation: ActivationV1::Session,
        },
        SpanV1::new(0, 30),
    );
    assert_sound(&pack.into(), 30, 1);

    let profile = b.catalog_decl(
        "strict",
        SpanV1::new(8, 14),
        CatalogPayloadV1::Profile {
            capabilities: vec![gref("read", 20, 24), gref("trace", 26, 31)],
            admission: AdmissionV1::CertifiedOnly,
        },
        SpanV1::new(0, 35),
    );
    assert_sound(&profile.into(), 35, 1);

    let gate = b.catalog_decl(
        "ci",
        SpanV1::new(5, 7),
        CatalogPayloadV1::Gate {
            checks: vec![gref("c1", 12, 14)],
            binding: GateBindingV1::All,
        },
        SpanV1::new(0, 20),
    );
    assert_sound(&gate.into(), 20, 1);

    let rule = b.catalog_decl(
        "r",
        SpanV1::new(5, 6),
        CatalogPayloadV1::RuleBinding {
            check_ref: gref("chk", 10, 13),
        },
        SpanV1::new(0, 13),
    );
    assert_sound(&rule.into(), 13, 1);
}

// ---- META (USE PACK / REQUIRE CERTIFIED) -----------------------------------

#[test]
fn meta_family_is_sound_program_item() {
    let b = HirBuilderV1::new();
    let use_pack = b.meta(
        MetaKindV1::UsePack(UsePackHirV1 {
            pack_id: "owasp".into(),
            activation: UsePackActivationV1::Workspace,
            lockfile: "owasp.lock".into(),
        }),
        SpanV1::new(0, 30),
    );
    assert_sound(&use_pack.into(), 30, 1);

    let require = b.meta(MetaKindV1::RequireCertified, SpanV1::new(0, 16));
    assert_sound(&require.into(), 16, 1);
}

// ---- COMPOSITION (LET ... WITH) nesting proof ------------------------------

#[test]
fn composition_nests_query_producer_and_find_body() {
    let b = HirBuilderV1::new();

    // LET candidates = SELECT users (a Query producer).
    let users = b.relation_source("users", None, SpanV1::new(18, 23));
    let producer_query = b.query(users, None, SpanV1::new(18, 23));
    let binding = b.composition_binding("candidates", SpanV1::new(4, 14), producer_query.into());

    // body: FIND taint src -> sink WITH candidates.
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("src", 30, 33)),
        sink: Some(gref("sink", 37, 41)),
        ..GraphEndpointsV1::default()
    };
    let body = b.graph_query(
        GraphQueryKindV1::Taint,
        endpoints,
        GraphConstraintsV1::default(),
        SpanV1::new(25, 45),
    );

    let comp = b.composition(
        vec![binding],
        "candidates",
        SpanV1::new(50, 60),
        body.into(),
        SpanV1::new(0, 60),
    );
    assert_eq!(comp.with_name, "candidates");
    assert_eq!(comp.bindings.len(), 1);

    // The producer is a Query item; the body is a GraphQuery item.
    assert!(matches!(
        &comp.bindings[0].producer,
        CompositionProducerV1::Query(_)
    ));
    assert!(matches!(&comp.body, CompositionBodyV1::GraphQuery(_)));

    let item: ProgramItemHirV1 = comp.into();
    // composition + producer query unit + relation source + find body = 4.
    assert_sound(&item, 60, 4);
}

// ---- fail-closed integrity reject on a malformed new node ------------------

/// A hand-built graph-query node with an inverted span must fail closed through
/// the SAME shared integrity gate the legacy nodes use.
#[test]
fn reject_malformed_graph_query_node_inverted_span() {
    use quanta_gqlang_v2_hir::GraphQueryHirV1;
    // node id 9 carries an inverted span 40..10.
    let find = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(9),
        SpanV1::new(40, 10),
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1::default(),
        GraphConstraintsV1::default(),
    );
    let item = ProgramItemHirV1::GraphQuery(Box::new(find));

    let err = check_tree(HirRootRefV1::Item(&item), 50).unwrap_err();
    assert_eq!(err.code(), "hir.integrity.inverted_span");
    let HirIntegrityErrorV1::InvertedSpan { id, span } = &err else {
        panic!("expected InvertedSpan, got {err:?}");
    };
    assert_eq!(id.raw(), 9);
    assert_eq!(*span, SpanV1::new(40, 10));
    assert_eq!(
        err.to_string(),
        "hir.integrity.inverted_span: HIR node id 9 has inverted span 40..10 (start > end)"
    );
}

#[test]
fn reject_duplicate_id_between_check_and_semantic_confirm() {
    use quanta_gqlang_v2_hir::{GraphQueryHirV1, LegalityHirV1};

    let confirm = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(9),
        SpanV1::new(60, 90),
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(gref("src", 70, 73)),
            sink: Some(gref("sink", 77, 81)),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1 {
            depth: Some(4),
            ..GraphConstraintsV1::default()
        },
    );
    let check = LegalityHirV1::new(
        HirNodeIdV1::from_raw(9),
        SpanV1::new(0, 90),
        gref("rule.hybrid", 6, 17),
        None,
        None,
        ConfirmModeV1::Require,
        EmitModeV1::Findings,
        DetectionModeV1::Hybrid,
    )
    .with_semantic_confirm(confirm);
    let item = ProgramItemHirV1::from(check);

    let err = check_tree(HirRootRefV1::Item(&item), 90).unwrap_err();
    assert_eq!(err.code(), "hir.integrity.duplicate_node_id");
    let HirIntegrityErrorV1::DuplicateNodeId { id, .. } = err else {
        panic!("expected DuplicateNodeId, got {err:?}");
    };
    assert_eq!(id.raw(), 9);
}

/// A duplicate id between a composition producer and its body must fail closed.
#[test]
fn reject_duplicate_id_across_composition_children() {
    use quanta_gqlang_v2_hir::{
        CompositionBindingV1, CompositionHirV1, LegalityHirV1, QueryHirV1, RelationExprHirV1,
        RelationExprKindV1, RelationSourceV1,
    };
    // Producer query and body legality deliberately share raw id 5.
    let source = RelationExprHirV1::new(
        HirNodeIdV1::from_raw(1),
        SpanV1::new(10, 15),
        RelationExprKindV1::Source(RelationSourceV1 {
            name: "users".into(),
            alias: None,
        }),
    );
    let producer = QueryHirV1::new(HirNodeIdV1::from_raw(5), SpanV1::new(10, 15), source, None);
    let binding = CompositionBindingV1::new(
        "c",
        SpanV1::new(4, 5),
        CompositionProducerV1::Query(Box::new(producer)),
    );
    let body = LegalityHirV1::new(
        HirNodeIdV1::from_raw(5),
        SpanV1::new(20, 30),
        gref("rules.x", 22, 29),
        None,
        None,
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        DetectionModeV1::PatternOnly,
    );
    let comp = CompositionHirV1::new(
        HirNodeIdV1::from_raw(2),
        SpanV1::new(0, 40),
        vec![binding],
        "c",
        SpanV1::new(33, 34),
        CompositionBodyV1::Legality(Box::new(body)),
    );
    let item = ProgramItemHirV1::Composition(Box::new(comp));

    let err = check_tree(HirRootRefV1::Item(&item), 40).unwrap_err();
    assert_eq!(err.code(), "hir.integrity.duplicate_node_id");
    let HirIntegrityErrorV1::DuplicateNodeId { id, .. } = &err else {
        panic!("expected DuplicateNodeId, got {err:?}");
    };
    assert_eq!(id.raw(), 5);
}

#[test]
fn reject_out_of_bounds_graph_reference_auxiliary_span() {
    let builder = HirBuilderV1::new();
    let item = ProgramItemHirV1::GraphQuery(Box::new(builder.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(gref("outside", 21, 28)),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(0, 20),
    )));

    assert!(matches!(
        check_tree(HirRootRefV1::Item(&item), 20),
        Err(HirIntegrityErrorV1::AuxiliarySpanOutOfBounds {
            span: SpanV1 { start: 21, end: 28 },
            source_len: 20,
        })
    ));
}
