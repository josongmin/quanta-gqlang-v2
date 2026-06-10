#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! ============================================================================
//! RED RAIL — E03C `LET` / `WITH` composition, LSP stdio front door.
//!
//! NOTE TO OTHER AGENTS: written RED-FIRST as the executable spec for
//! `docs/plans/jun-10-e03c-let-with-composition`. The tests were
//! `#[ignore]`d until tickets 01-05 landed; ticket 05 (the LSP read-only
//! surface) removed the ignores as the flip-to-green gate, so this rail is now
//! GREEN. Do NOT weaken assertions — match the implementation to this contract.
//! ============================================================================

mod support;

use support::run_binary_line_v1;

/// The S1 composition document encoded as a single JSON request `query` value.
/// `\n` are JSON escapes the server decodes into statement boundaries.
const S1_QUERY_V1: &str = "LET suspects = SAMPLE query=AuthMiddleware ;; scope=repo::auth ;; mode=semantic ;; limit=20\\nFIND taint api.entry -> db.write WITH suspects";

// ---------------------------------------------------------------- hover (first-statement card)
#[test]
fn hover_summarizes_the_composition_block_as_first_statement_v1() {
    let request = format!(r#"{{"method":"hover","query":"{S1_QUERY_V1}"}}"#);
    let (code, line) = run_binary_line_v1(&request);
    assert_eq!(code, Some(0));
    assert!(line.contains(r#""ok":true"#), "{line}");
    // The block IS the first statement: card keyword LET, category composition.
    assert!(line.contains(r#""root_keyword":"LET""#), "{line}");
    assert!(line.contains(r#""category":"composition""#), "{line}");
    assert!(line.contains(r#""typed_target":"suspects""#), "{line}");
}

// ---------------------------------------------------------------- symbols
#[test]
fn symbols_expose_bindings_with_and_body_v1() {
    let request = format!(r#"{{"method":"symbols","query":"{S1_QUERY_V1}"}}"#);
    let (code, line) = run_binary_line_v1(&request);
    assert_eq!(code, Some(0));
    assert!(line.contains(r#""ok":true"#), "{line}");
    assert!(line.contains(r#""name":"document""#), "{line}");
    assert!(line.contains(r#""name":"suspects""#), "{line}");
    assert!(line.contains(r#""kind":"binding""#), "{line}");
    // the composed body's own root symbol is present
    assert!(
        line.contains(r#""name":"FIND""#) || line.contains("find"),
        "{line}"
    );
}

// ---------------------------------------------------------------- diagnostics (clean + faults)
#[test]
fn diagnostics_are_clean_for_a_valid_composition_v1() {
    let request = format!(r#"{{"method":"diagnostics","query":"{S1_QUERY_V1}"}}"#);
    let (code, line) = run_binary_line_v1(&request);
    assert_eq!(code, Some(0));
    assert!(line.contains(r#""ok":true"#), "{line}");
    assert!(
        line.contains(r#""result":[]"#),
        "valid composition must be clean: {line}"
    );
}

#[test]
fn diagnostics_surface_undefined_binding_fail_closed_v1() {
    let q = "FIND taint api.entry -> db.write WITH ghost";
    let request = format!(r#"{{"method":"diagnostics","query":"{q}"}}"#);
    let (code, line) = run_binary_line_v1(&request);
    assert_eq!(code, Some(0)); // diagnostics returns Ok with an item, exit 0
    assert!(line.contains(r#""ok":true"#), "{line}");
    assert!(line.contains(r#""class":"sema""#), "{line}");
    assert!(line.contains("composition_undefined_binding"), "{line}");
}

#[test]
fn diagnostics_surface_invalid_producer_fail_closed_v1() {
    let q = "LET x = SHOW models\\nFIND taint api.entry -> db.write WITH x";
    let request = format!(r#"{{"method":"diagnostics","query":"{q}"}}"#);
    let (code, line) = run_binary_line_v1(&request);
    assert_eq!(code, Some(0));
    assert!(line.contains("composition_invalid_producer"), "{line}");
}

// ---------------------------------------------------------------- command-plane stays closed
#[test]
fn composition_adds_no_mutating_surface_v1() {
    // A quickfix request on a composition document must still carry only
    // read-only metadata (apply_command/workspace_edit null) — composition
    // never widens the command plane.
    let q =
        "LET x = SELECT id FROM catalog.users LIMIT 5\\nFIND taint api.entry -> db.write WITH x";
    let request = format!(r#"{{"method":"quickfix","input":"{q}"}}"#);
    let (_code, line) = run_binary_line_v1(&request);
    // Whatever quickfix decides, it must not emit a non-null apply/edit.
    assert!(
        !line.contains(r#""apply_command":""#),
        "composition leaked an apply command: {line}"
    );
    assert!(
        !line.contains(r#""workspace_edit":""#),
        "composition leaked a workspace edit: {line}"
    );
}
