//! v3 binder fail-closed hellgate rail.
//!
//! Each test asserts the ACTUAL typed diagnostic code and Display text, not
//! just that an error occurred. Covers the five reject classes the binder owns
//! (`D3`/`D25`/`D28`/`D30`): unresolved, duplicate, shadowed, illegal-capture,
//! ambiguous-import. HIR is built directly (parser is ticket 06).

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(clippy::panic, reason = "tests assert by panicking")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected variant and panic on the rest"
)]

use quanta_gqlang_v2_binder::{BindDiagnosticV1, Binder, CaptureV1, ModuleGraphV1};
use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};

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
    binder.declare_program(&[d1, d2], &[]);
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
        quanta_gqlang_v2_binder::ResolvedTargetV1::Local(_) => {}
        other => panic!("local must win over import, got {other:?}"),
    }
}
