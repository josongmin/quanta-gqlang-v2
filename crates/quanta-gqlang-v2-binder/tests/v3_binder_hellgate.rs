#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_binder::{
    BindDiagnosticV1, Binder, BinderDeclarationRequestErrorV3, BinderKindV1, CaptureV1,
    ModuleGraphV1,
};
use quanta_gqlang_v2_hir::{
    AliasV1, CheckedHirDocumentOwnedV2, FrontendRequestMaterializationRootV3,
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryKindV1, GraphRefV1, HirBuilderV1,
    HirDocumentV1, LogicalBinderV1, LogicalQuantifierKindV1, NonEmptyBindersV1, PathSegmentV1,
    ProgramItemHirV1, ProjectionItemV1, RelationExistsPolarityV1, SourceIdV2, SpanV1,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct DeclarationTestSink {
    identity: Arc<()>,
    live: Arc<AtomicU64>,
    limit: u64,
}

struct DeclarationTestPermit {
    identity: Arc<()>,
    live: Arc<AtomicU64>,
    charged: u64,
}

impl Drop for DeclarationTestPermit {
    fn drop(&mut self) {
        self.live.fetch_sub(self.charged, Ordering::AcqRel);
    }
}

impl FrontendRequestStorageSinkV3 for DeclarationTestSink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let charged = logical_bytes
            .checked_add(u64::try_from(std::mem::size_of::<DeclarationTestPermit>()).unwrap())
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.live.load(Ordering::Acquire);
        loop {
            let next = live
                .checked_add(charged)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.limit {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self
                .live
                .compare_exchange_weak(live, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(observed) => live = observed,
            }
        }
        Ok(Box::new(DeclarationTestPermit {
            identity: Arc::clone(&self.identity),
            live: Arc::clone(&self.live),
            charged,
        }))
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<DeclarationTestPermit>()
            .is_some_and(|issued| Arc::ptr_eq(&issued.identity, &self.identity))
    }
}

fn alias(name: &str, start: usize) -> AliasV1 {
    AliasV1 {
        name: name.to_owned(),
        span: SpanV1::new(start, start + 1),
    }
}

// (2) unresolved symbol -> fail closed.
#[test]
fn unresolved_symbol_rejects() {
    let b = HirBuilderV1::new();
    let body = b.symbol("ghost", SpanV1::new(4, 9));
    let decl = b.decl("v", SpanV1::new(0, 1), body, SpanV1::new(0, 9));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_decl(&decl, &[]);
    let result = binder.finish();

    assert!(!result.is_ok());
    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.unresolved_symbol");
    assert_eq!(
        diag.to_string(),
        "binder.unresolved_symbol: unresolved symbol `ghost` at 4..9"
    );
}

// (3) duplicate declaration -> fail closed.
#[test]
fn duplicate_declaration_rejects() {
    let b = HirBuilderV1::new();
    let body1 = b.int("1", SpanV1::new(10, 11));
    let d1 = b.decl("dup", SpanV1::new(4, 7), body1, SpanV1::new(0, 11));
    let body2 = b.int("2", SpanV1::new(30, 31));
    let d2 = b.decl("dup", SpanV1::new(24, 27), body2, SpanV1::new(20, 31));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.declare_program(&[d1.clone(), d2.clone()], &[]);
    let result = binder.finish();

    assert!(!result.is_ok());
    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.duplicate_declaration");
    assert_eq!(
        diag.to_string(),
        "binder.duplicate_declaration: duplicate declaration `dup` \
         (first at 4..7, again at 24..27)"
    );

    let mut reversed = Binder::new(&graph);
    reversed.declare_program(&[d2.clone(), d1.clone()], &[]);
    assert_eq!(reversed.finish().diagnostics(), result.diagnostics());

    let borrowed = [&d2, &d1];
    let mut borrowed_binder = Binder::new(&graph);
    borrowed_binder.declare_borrowed_program(&borrowed);
    assert_eq!(borrowed[0].id, d2.id, "caller source order stays intact");
    assert_eq!(
        borrowed_binder.finish().diagnostics(),
        result.diagnostics(),
        "borrowed and owned paths retain one duplicate decision"
    );

    let first_body = b.int("3", SpanV1::new(40, 41));
    let later = b.decl("dup", SpanV1::new(44, 47), first_body, SpanV1::new(40, 47));
    let earlier_body = b.int("4", SpanV1::new(50, 51));
    let earlier = b.decl(
        "dup",
        SpanV1::new(14, 17),
        earlier_body,
        SpanV1::new(10, 51),
    );
    let mut shuffled = Binder::new(&graph);
    shuffled.declare_program(&[later, earlier], &[]);
    let shuffled = shuffled.finish();
    assert_eq!(shuffled.diagnostics().len(), 1);
    assert_eq!(
        shuffled.diagnostics()[0].primary_span(),
        SpanV1::new(44, 47)
    );
}

#[test]
fn request_declarations_preserve_source_first_duplicate_with_reverse_items() {
    let builder = HirBuilderV1::new();
    let first = builder.decl(
        "x",
        SpanV1::new(4, 5),
        builder.int("1", SpanV1::new(8, 9)),
        SpanV1::new(0, 9),
    );
    let other = builder.decl(
        "y",
        SpanV1::new(14, 15),
        builder.int("2", SpanV1::new(18, 19)),
        SpanV1::new(10, 19),
    );
    let second = builder.decl(
        "x",
        SpanV1::new(24, 25),
        builder.int("3", SpanV1::new(28, 29)),
        SpanV1::new(20, 29),
    );
    let third = builder.decl(
        "x",
        SpanV1::new(44, 45),
        builder.int("4", SpanV1::new(48, 49)),
        SpanV1::new(40, 49),
    );
    let source_id = SourceIdV2::from_raw(73);
    let source = " ".repeat(49);
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![
            ProgramItemHirV1::from(third.clone()),
            ProgramItemHirV1::from(second.clone()),
            ProgramItemHirV1::from(other.clone()),
            ProgramItemHirV1::from(first.clone()),
        ],
    })
    .unwrap();
    let sink = DeclarationTestSink {
        identity: Arc::new(()),
        live: Arc::new(AtomicU64::new(0)),
        limit: u64::MAX,
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    let parent = root.reserve_v3(source_id, &source, 0).unwrap();
    let retained = Binder::new(ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(&checked, &root, source_id, &source, &parent)
        .unwrap();
    assert_eq!(retained.declared_name_count_v3(), 2);
    let mut conventional = Binder::new(ModuleGraphV1::new());
    conventional.declare_program(&[third, second, other, first], &[]);
    assert_eq!(conventional.finish().diagnostics().len(), 2);
    assert!(matches!(
        retained.resolve_checked_source_bindings_with_request_owner_v3(
            &checked, &root, source_id, &source, &parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Phase)
    ));
    drop(parent);
    assert_eq!(sink.live.load(Ordering::Acquire), 0);
}

#[test]
fn request_declaration_refuses_scratch_budget_before_symbol_birth() {
    let builder = HirBuilderV1::new();
    let source_id = SourceIdV2::from_raw(74);
    let source = " ".repeat(9);
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![ProgramItemHirV1::from(builder.decl(
            "x",
            SpanV1::new(4, 5),
            builder.int("1", SpanV1::new(8, 9)),
            SpanV1::new(0, 9),
        ))],
    })
    .unwrap();
    // The parent permit requests zero logical bytes; the declaration's
    // nonempty scratch vector must make another request and be refused.
    let sink = DeclarationTestSink {
        identity: Arc::new(()),
        live: Arc::new(AtomicU64::new(0)),
        limit: u64::try_from(std::mem::size_of::<DeclarationTestPermit>()).unwrap(),
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    let parent = root.reserve_v3(source_id, &source, 0).unwrap();
    assert!(matches!(
        Binder::new(ModuleGraphV1::new()).declare_checked_program_with_request_owner_v3(
            &checked, &root, source_id, &source, &parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    drop(parent);
    assert_eq!(sink.live.load(Ordering::Acquire), 0);
}

// shadowed binding (a let shadows an outer let of the same name) -> fail closed.
#[test]
fn shadowed_binding_rejects() {
    let b = HirBuilderV1::new();
    let inner_ref = b.symbol("x", SpanV1::new(40, 41));
    let decl = b.decl("v", SpanV1::new(0, 1), inner_ref, SpanV1::new(0, 41));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("x", decl.id, SpanV1::new(10, 11));
    // nested scope shadows the outer `x`.
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("x", decl.id, SpanV1::new(25, 26));
    binder.close_scope();
    binder.close_scope();
    let result = binder.finish();

    assert!(!result.is_ok());
    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.shadowed_binding");
    assert_eq!(
        diag.to_string(),
        "binder.shadowed_binding: binding `x` at 25..26 shadows outer binding at 10..11"
    );
}

// (4) illegal capture across a non-capturing boundary -> fail closed.
#[test]
fn illegal_capture_rejects() {
    let b = HirBuilderV1::new();
    // reference to `outer` lives inside a non-capturing nested scope.
    let inner_ref = b.symbol("outer", SpanV1::new(50, 55));
    let decl = b.decl(
        "v",
        SpanV1::new(0, 1),
        inner_ref.clone(),
        SpanV1::new(0, 55),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("outer", decl.id, SpanV1::new(10, 15));
    // non-capturing barrier: `outer` is beyond it and cannot be captured.
    binder.open_scope(CaptureV1::NonCapturing);
    binder.resolve_decl(&decl, &[]);
    binder.close_scope();
    binder.close_scope();
    let result = binder.finish();

    assert!(!result.is_ok());
    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.illegal_capture");
    assert_eq!(
        diag.to_string(),
        "binder.illegal_capture: reference `outer` at 50..55 illegally captures \
         binding at 10..15 across a non-capturing boundary"
    );
}

/// A graph reference crossed out of a composition frame (`NonCapturing`) must
/// reject through `resolve_local_ref`, not silently leak the outer binding.
#[test]
fn graph_reference_across_composition_barrier_rejects_as_illegal_capture() {
    let b = HirBuilderV1::new();
    let producer = b.query(
        b.relation_source("catalog.users", SpanV1::new(0, 5), None, SpanV1::new(0, 5)),
        None,
        SpanV1::new(0, 5),
    );
    let binding = b.composition_binding("s", SpanV1::new(6, 7), producer.into());
    let find = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "outer".to_owned(),
                span: SpanV1::new(50, 55),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(45, 60),
    );
    let find_id = find.id;
    let composition = b.composition(
        vec![binding],
        "s",
        SpanV1::new(90, 91),
        find.into(),
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("outer", find_id, SpanV1::new(10, 15));
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.illegal_capture");
    assert_eq!(diag.name(), "outer");
    assert_eq!(diag.primary_span(), SpanV1::new(50, 55));
    match diag {
        BindDiagnosticV1::IllegalCapture { binding_span, .. } => {
            assert_eq!(*binding_span, SpanV1::new(10, 15));
        }
        other => panic!("expected illegal capture, got {other:?}"),
    }
}

/// A relation source crossed out of a non-capturing query frame must reject
/// through `resolve_relation`'s illegal-capture arm (not host fallback).
#[test]
fn relation_source_across_noncapturing_barrier_rejects_as_illegal_capture() {
    let b = HirBuilderV1::new();
    let outer = b.relation_source(
        "catalog.outer",
        SpanV1::new(0, 10),
        None,
        SpanV1::new(0, 10),
    );
    let inner = b.relation_source("outer", SpanV1::new(20, 25), None, SpanV1::new(20, 25));
    let query = b.query(inner, None, SpanV1::new(20, 25));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("outer", outer.id, SpanV1::new(0, 5));
    binder.open_scope(CaptureV1::NonCapturing);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.illegal_capture");
    assert_eq!(diag.name(), "outer");
    assert_eq!(diag.primary_span(), SpanV1::new(20, 25));
    match diag {
        BindDiagnosticV1::IllegalCapture { binding_span, .. } => {
            assert_eq!(*binding_span, SpanV1::new(0, 5));
        }
        other => panic!("expected illegal capture, got {other:?}"),
    }
}

// (5) ambiguous import (two frozen-graph entries for one name) -> fail closed,
// with deterministic sorted candidate list (no import-order tiebreak, D28/D30).
#[test]
fn ambiguous_import_rejects() {
    let b = HirBuilderV1::new();
    let body = b.symbol("shared", SpanV1::new(4, 10));
    let decl = b.decl("v", SpanV1::new(0, 1), body, SpanV1::new(0, 10));

    // two distinct targets for one imported name. Insertion order is the
    // opposite of sorted order to prove the binder sorts, not last-wins.
    let graph = ModuleGraphV1::new()
        .with_import("shared", "z.mod.shared")
        .with_import("shared", "a.mod.shared");
    let mut binder = Binder::new(&graph);
    binder.resolve_decl(&decl, &[]);
    let result = binder.finish();

    assert!(!result.is_ok());
    assert_eq!(result.diagnostics().len(), 1);
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.ambiguous_import");
    assert_eq!(
        diag.to_string(),
        "binder.ambiguous_import: imported symbol `shared` at 4..10 is ambiguous \
         between a.mod.shared, z.mod.shared"
    );
    // candidates are reported in deterministic sorted order.
    match diag {
        BindDiagnosticV1::AmbiguousImport { candidates, .. } => {
            assert_eq!(candidates, &["a.mod.shared", "z.mod.shared"]);
        }
        other => panic!("expected ambiguous import, got {other:?}"),
    }
}

// determinism: a local binding wins over an import of the same name; import
// order can never override a lexical binder (D25/D28).
#[test]
fn local_binding_shadows_import_deterministically() {
    let b = HirBuilderV1::new();
    let body = b.symbol("name", SpanV1::new(4, 8));
    let decl = b.decl("v", SpanV1::new(0, 1), body.clone(), SpanV1::new(0, 8));

    let graph = ModuleGraphV1::new().with_import("name", "std.name");
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("name", decl.id, SpanV1::new(2, 3));
    binder.resolve_decl(&decl, &[]);
    binder.close_scope();
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(body.id).unwrap() {
        quanta_gqlang_v2_binder::ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.kind, BinderKindV1::Let);
            assert_eq!(binding.name, "name");
        }
        other => panic!("local must win over import, got {other:?}"),
    }
}

#[test]
fn diagnostics_are_sorted_by_location_then_code_not_emission_order() {
    let b = HirBuilderV1::new();
    let unresolved = b.symbol("early", SpanV1::new(5, 10));
    let decl = b.decl("body", SpanV1::new(0, 4), unresolved, SpanV1::new(0, 10));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    // Emit the later duplicate first, then the earlier unresolved reference.
    binder.declare("dup", decl.id, SpanV1::new(70, 73));
    binder.declare("dup", decl.id, SpanV1::new(80, 83));
    binder.resolve_decl(&decl, &[]);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 2);
    assert_eq!(result.diagnostics()[0].code(), "binder.unresolved_symbol");
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(5, 10));
    assert_eq!(
        result.diagnostics()[1].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[1].primary_span(), SpanV1::new(80, 83));

    // Emit the same failures in the opposite order; finish must select the
    // same first diagnostic and preserve the same complete vector.
    let mut reversed = Binder::new(&graph);
    reversed.resolve_decl(&decl, &[]);
    reversed.declare("dup", decl.id, SpanV1::new(70, 73));
    reversed.declare("dup", decl.id, SpanV1::new(80, 83));
    let reversed = reversed.finish();
    assert_eq!(reversed.diagnostics(), result.diagnostics());
}

#[test]
fn relation_alias_shadowing_outer_row_binding_rejects() {
    let b = HirBuilderV1::new();
    let outer = b.relation_source(
        "catalog.outer",
        SpanV1::new(0, 10),
        Some(AliasV1 {
            name: "x".to_owned(),
            span: SpanV1::new(10, 11),
        }),
        SpanV1::new(0, 11),
    );
    let inner = b.relation_source(
        "catalog.inner",
        SpanV1::new(20, 30),
        Some(AliasV1 {
            name: "x".to_owned(),
            span: SpanV1::new(30, 31),
        }),
        SpanV1::new(20, 31),
    );
    let exists = b.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(18, 32));
    let query = b.query(
        b.relation_filter(outer, exists, SpanV1::new(0, 32)),
        None,
        SpanV1::new(0, 32),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), "binder.shadowed_binding");
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(30, 31));
    assert_eq!(result.diagnostics()[0].name(), "x");
    assert_eq!(
        result.diagnostics()[0].to_string(),
        "binder.shadowed_binding: binding `x` at 30..31 shadows outer binding at 10..11"
    );
}

#[test]
fn duplicate_default_path_names_reject_at_second_segment() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let first = b
        .try_path(
            b.symbol("u", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".to_owned(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let second = b
        .try_path(
            b.symbol("u", SpanV1::new(30, 31)),
            vec![PathSegmentV1 {
                name: "id".to_owned(),
                span: SpanV1::new(32, 34),
            }],
            SpanV1::new(30, 34),
        )
        .unwrap();
    let projected = b
        .try_project(
            source,
            vec![
                ProjectionItemV1 {
                    value: first,
                    alias: None,
                },
                ProjectionItemV1 {
                    value: second,
                    alias: None,
                },
            ],
            SpanV1::new(0, 34),
        )
        .unwrap();
    let query = b.query(projected, None, SpanV1::new(0, 34));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(32, 34));
    assert_eq!(
        result.diagnostics()[0].to_string(),
        "binder.duplicate_declaration: duplicate declaration `id` \
         (first at 22..24, again at 32..34)"
    );
}

#[test]
fn duplicate_canonical_col_names_reject_at_second_expression() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let projected = b
        .try_project(
            source,
            vec![
                ProjectionItemV1 {
                    value: b.int("1", SpanV1::new(20, 21)),
                    alias: None,
                },
                ProjectionItemV1 {
                    value: b.int("2", SpanV1::new(30, 31)),
                    alias: None,
                },
            ],
            SpanV1::new(0, 31),
        )
        .unwrap();
    let query = b.query(projected, None, SpanV1::new(0, 31));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(30, 31));
    assert_eq!(
        result.diagnostics()[0].to_string(),
        "binder.duplicate_declaration: duplicate declaration `col` \
         (first at 20..21, again at 30..31)"
    );
}

#[test]
fn bare_rule_name_is_not_a_value_binding() {
    let b = HirBuilderV1::new();
    let rule = b.rule(
        "is_valid",
        SpanV1::new(5, 13),
        b.symbol("x", SpanV1::new(15, 16)),
        b.bool_literal(true, SpanV1::new(20, 24)),
        SpanV1::new(0, 24),
    );
    let bare = b.symbol("is_valid", SpanV1::new(30, 38));
    let decl = b.decl("v", SpanV1::new(26, 27), bare, SpanV1::new(26, 38));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.declare_program(std::slice::from_ref(&decl), &[rule]);
    binder.resolve_decl(&decl, &[]);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), "binder.unresolved_symbol");
    assert_eq!(result.diagnostics()[0].name(), "is_valid");
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(30, 38));
}

#[test]
fn duplicate_quantifier_binders_reject_exact_second_name() {
    let b = HirBuilderV1::new();
    let binders = NonEmptyBindersV1::try_new(vec![
        LogicalBinderV1 {
            name: "x".to_owned(),
            span: SpanV1::new(20, 21),
        },
        LogicalBinderV1 {
            name: "x".to_owned(),
            span: SpanV1::new(24, 25),
        },
    ])
    .unwrap();
    let quant = b.logical_quantifier(
        LogicalQuantifierKindV1::Exists,
        binders,
        b.bool_literal(true, SpanV1::new(30, 34)),
        SpanV1::new(18, 34),
    );
    let rule = b.rule(
        "r",
        SpanV1::new(0, 1),
        b.int("1", SpanV1::new(10, 11)),
        quant,
        SpanV1::new(0, 34),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_rule(&rule, &[]);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(24, 25));
    assert_eq!(
        result.diagnostics()[0].to_string(),
        "binder.duplicate_declaration: duplicate declaration `x` \
         (first at 20..21, again at 24..25)"
    );
}
