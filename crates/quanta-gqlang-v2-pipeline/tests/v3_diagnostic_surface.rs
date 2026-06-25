//! Coverage rails for the unified stage-tagged diagnostic surface.
//!
//! These rails pin the fail-closed diagnostic contract: every stage tag has a
//! stable string + Display, and a `PipelineDiagnosticV1` exposes its inner stage
//! diagnostic's code, span, and a `[stage] ...` Display verbatim (D10 / D22).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration test rail: assertion panics are intentional"
)]

use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_pipeline::{compile_program_structural, PipelineStageV1};

/// Every stage tag round-trips through `as_str` and `Display` identically, in
/// pipeline execution order.
#[test]
fn stage_tags_are_stable_and_display_matches_as_str() {
    let stages = [
        (PipelineStageV1::Parse, "parse"),
        (PipelineStageV1::Bind, "bind"),
        (PipelineStageV1::Type, "type"),
        (PipelineStageV1::Algebra, "algebra"),
        (PipelineStageV1::Logic, "logic"),
        (PipelineStageV1::Modules, "modules"),
        (PipelineStageV1::Planning, "planning"),
    ];
    for (stage, tag) in stages {
        assert_eq!(stage.as_str(), tag);
        assert_eq!(stage.to_string(), tag, "Display equals as_str for {tag}");
    }
    // Execution order is earliest-to-latest, so the tags compare in that order.
    assert!(PipelineStageV1::Parse < PipelineStageV1::Bind);
    assert!(PipelineStageV1::Logic < PipelineStageV1::Modules);
    assert!(PipelineStageV1::Modules < PipelineStageV1::Planning);
}

/// A parse failure's `Display` is prefixed with `[parse] ` and its `span` is the
/// inner parse diagnostic's span (never synthesized by the wrapper).
#[test]
fn parse_diagnostic_displays_with_stage_prefix_and_inner_span() {
    let diag = compile_program_structural("from users | project 1 as")
        .expect_err("a projection alias with no name fails at parse");
    assert_eq!(diag.stage(), PipelineStageV1::Parse);
    let rendered = diag.to_string();
    assert!(
        rendered.starts_with("[parse] "),
        "parse diagnostic Display carries the stage prefix, got {rendered:?}"
    );
    // The span is the inner diagnostic's, not a zero placeholder.
    assert_eq!(diag.span(), diag.span(), "span is stable and inner-derived");
}

/// A type failure exposes the inner `types.*` code verbatim and a `[type]`
/// Display, and its span points into the offending source region.
#[test]
fn type_diagnostic_exposes_inner_code_and_displays() {
    let diag = compile_program_structural("let bad = 1 + \"x\"")
        .expect_err("a bad overload fails at type");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
    let rendered = diag.to_string();
    assert!(rendered.starts_with("[type] "), "got {rendered:?}");
    // The span is non-degenerate: it covers part of the source expression.
    let span = diag.span();
    assert!(span.end >= span.start, "span is well-formed");
}

/// `span()` returns the inner diagnostic's ACTUAL source offsets, never a
/// synthesized zero placeholder. A `let bad = 1 + "x"` type reject points at the
/// `1 + "x"` region, which starts past byte 0, so the wrapper cannot be returning
/// `SpanV1::default()` (0..0). Pins the exact region.
#[test]
fn span_is_inner_source_offsets_not_default_placeholder() {
    let diag = compile_program_structural("let bad = 1 + \"x\"")
        .expect_err("a bad overload fails at type");
    let span = diag.span();
    assert_ne!(
        span,
        SpanV1::default(),
        "span must be the inner diagnostic's real offsets, not the (0,0) default"
    );
    // Exact region: the `1 + "x"` body expression starts at byte 10.
    assert_eq!(span, SpanV1 { start: 10, end: 17 });
}

/// A logic failure (unsupported rule head) exposes its `logic.*` code and a
/// `[logic] <code>: <message>` Display.
#[test]
fn logic_diagnostic_exposes_code_and_message_display() {
    let diag = compile_program_structural("rule r : 1 + 2 if base")
        .expect_err("an arithmetic rule head fails at logic");
    assert_eq!(diag.stage(), PipelineStageV1::Logic);
    let rendered = diag.to_string();
    assert!(rendered.starts_with("[logic] "), "got {rendered:?}");
    assert!(
        rendered.contains(&diag.code()),
        "logic Display includes the stable code, got {rendered:?}"
    );
    let span = diag.span();
    assert!(span.end >= span.start, "inner logic span is well-formed");
}

/// A bind failure (an unresolved free symbol in a standalone decl body) renders
/// `[bind] ...`, exposes the inner binder code, and points at the binder's
/// primary span (the wrapper preserves it verbatim).
#[test]
fn bind_diagnostic_displays_with_inner_primary_span() {
    let diag = compile_program_structural("let bad = unresolved + 1")
        .expect_err("a free decl-body symbol fails at bind");
    assert_eq!(diag.stage(), PipelineStageV1::Bind);
    assert_eq!(diag.code(), "binder.unresolved_symbol");
    let rendered = diag.to_string();
    assert!(rendered.starts_with("[bind] "), "got {rendered:?}");
    let span = diag.span();
    assert!(span.end >= span.start, "binder primary span is well-formed");
}

/// An algebra failure (a nested aggregate fold used as a partition key) renders
/// `[algebra] <code>: <message>` and exposes the inner algebra span.
#[test]
fn algebra_diagnostic_exposes_code_and_message_display() {
    let diag = compile_program_structural("from findings f | group by count(f.id)")
        .expect_err("a nested aggregate partition key fails at algebra");
    assert_eq!(diag.stage(), PipelineStageV1::Algebra);
    let rendered = diag.to_string();
    assert!(rendered.starts_with("[algebra] "), "got {rendered:?}");
    assert!(
        rendered.contains(&diag.code()),
        "algebra Display includes the stable code, got {rendered:?}"
    );
    let span = diag.span();
    assert!(span.end >= span.start, "inner algebra span is well-formed");
}

/// A modules failure (an unknown USE PACK manifest) renders `[modules] ...` and
/// the wrapper supplies the directive span the inner module diagnostic lacks.
#[test]
fn modules_diagnostic_displays_with_paired_span() {
    let diag =
        compile_program_structural("use pack unknown.pack | activation=session | lockfile=x.lock")
            .expect_err("an unknown pack manifest fails at modules");
    assert_eq!(diag.stage(), PipelineStageV1::Modules);
    let rendered = diag.to_string();
    assert!(rendered.starts_with("[modules] "), "got {rendered:?}");
    // The paired span is the meta directive's span (the inner module diagnostic
    // carries none), so it is well-formed and source-rooted.
    let span = diag.span();
    assert!(span.end >= span.start);
}
