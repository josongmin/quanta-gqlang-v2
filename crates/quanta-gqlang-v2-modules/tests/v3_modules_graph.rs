//! Positive rails for the canonical module/import graph (ticket 11, `D30`).
//!
//! syntax->sema->ir path pinned: NONE. These pin the four deterministic
//! resolution rules (shadowing, re-export precedence, ambiguity, cycle policy)
//! on their success paths plus the binder-view projection seam.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]

use quanta_gqlang_v2_modules::{ModuleGraphBuilderV1, VisibilityV1};

// Positive: a small import graph resolves deterministically.
#[test]
fn small_graph_resolves_deterministically() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("lib", "isUser", VisibilityV1::Public)
        .declare("lib", "isAdmin", VisibilityV1::Public)
        .import("query", "isUser", "lib", "isUser")
        .import("query", "isAdmin", "lib", "isAdmin");

    let resolved = graph.resolve().unwrap();
    assert_eq!(resolved.lookup("query", "isUser"), Some("lib::isUser"));
    assert_eq!(resolved.lookup("query", "isAdmin"), Some("lib::isAdmin"));
    // Deterministic, sorted bindings.
    assert_eq!(
        resolved.bindings("query"),
        vec![
            ("isAdmin".to_string(), "lib::isAdmin".to_string()),
            ("isUser".to_string(), "lib::isUser".to_string()),
        ]
    );
}

// Rule 1: a local declaration shadows an imported name of the same short name.
#[test]
fn local_declaration_shadows_import() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("lib", "helper", VisibilityV1::Public)
        // `query` both imports `helper` from lib AND declares its own.
        .import("query", "helper", "lib", "helper")
        .declare("query", "helper", VisibilityV1::Private);

    let resolved = graph.resolve().unwrap();
    // The local declaration wins; the import is shadowed, not ambiguous.
    assert_eq!(resolved.lookup("query", "helper"), Some("query::helper"));
}

// Rule 2: a direct import takes precedence over a re-export of the same name.
#[test]
fn direct_import_precedence_over_reexport() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("direct", "f", VisibilityV1::Public)
        .declare("indirect", "f", VisibilityV1::Public)
        // `facade` re-exports f from `indirect`, but `query` directly imports
        // f from `direct` while also re-exporting from `facade`.
        .reexport("facade", "f", "indirect", "f")
        .reexport("query", "f", "facade", "f")
        .import("query", "f", "direct", "f");

    let resolved = graph.resolve().unwrap();
    // Direct import precedence applied: `direct::f`, not the re-exported one.
    assert_eq!(resolved.lookup("query", "f"), Some("direct::f"));
}

// The frozen graph projects to the binder-consumed view with at most one target
// per name (binder never re-derives precedence).
#[test]
fn binder_view_projection_is_singleton() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("lib", "g", VisibilityV1::Public)
        .import("query", "g", "lib", "g");

    let resolved = graph.resolve().unwrap();
    let view = resolved.binder_view("query");
    assert_eq!(view.get("g"), Some(&vec!["lib::g".to_string()]));
    // Every projected candidate set is a singleton.
    assert!(view.values().all(|targets| targets.len() == 1));
}

// A self-import is a (trivial) cycle and rejects explicitly.
#[test]
fn self_cycle_rejects() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("m", "x", VisibilityV1::Public)
        .import("m", "x2", "m", "x");

    let err = graph.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.import_cycle");
}

// An acyclic diamond resolves: two parents RE-EXPORT a shared lower symbol, and
// the re-export chain resolves to the single declaring origin `base::b`. A plain
// import is module-private (deliverable 4) so transitive exposure must go through
// an explicit re-export.
#[test]
fn acyclic_diamond_resolves() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("base", "b", VisibilityV1::Public)
        .reexport("left", "b", "base", "b")
        .reexport("right", "b", "base", "b")
        .import("top", "bl", "left", "b")
        .import("top", "br", "right", "b");

    let resolved = graph.resolve().unwrap();
    // Both paths resolve to the same declaring origin.
    assert_eq!(resolved.lookup("top", "bl"), Some("base::b"));
    assert_eq!(resolved.lookup("top", "br"), Some("base::b"));
}
