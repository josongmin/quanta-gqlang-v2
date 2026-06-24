#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration hellgate rail: assertion panics and intentional indexing"
)]

//! CLI/LSP parity hellgate over the ONE shared pipeline (ticket 08, D22, D10).
//!
//! syntax->sema->ir path pinned: a SINGLE failing advanced source -> both
//! `pipeline::compile_program` (what the CLI worker surfaces from its side) and
//! the LSP `v3_diagnostics` stdio method -> the SAME stage-tagged code. If the
//! LSP rewrote, downgraded, or reclassified the diagnostic, this fails closed.
//!
//! The reference code asserted here is
//! `algebra.ill_formed_aggregate.nested_aggregate` at the `algebra` stage for
//! `from findings f | group by count(f.id)` (a `group by` keyed on an aggregate
//! fold cannot lower — an aggregate is not a partition key); the CLI worker
//! asserts the same code from the other side, so the two surfaces are pinned to
//! one pipeline truth and can never drift.

mod support;

use quanta_gqlang_v2_pipeline::{compile_program, PipelineStageV1};
use support::run_binary_line_v1;

/// The single failing advanced fixture: `group by count(f.id)` binds (alias `f`
/// resolves, `count` is a known aggregate) but cannot lower — an aggregate fold
/// is not a valid partition key — so it rejects at the ALGEBRA stage.
const ADVANCED_FAILING_SRC_V1: &str = "from findings f | group by count(f.id)";

/// The exact pipeline diagnostic code the parity is anchored to. Pasted into the
/// receipt so the CLI worker can cross-check the identical string.
const EXPECTED_PARITY_CODE_V1: &str = "algebra.ill_formed_aggregate.nested_aggregate";
const EXPECTED_PARITY_STAGE_V1: &str = "algebra";

#[test]
fn pipeline_side_emits_the_expected_stage_and_code_v1() {
    // The pipeline truth (CLI side) for the fixture.
    let diag = compile_program(ADVANCED_FAILING_SRC_V1).expect_err("advanced query must reject");
    assert_eq!(
        diag.stage(),
        PipelineStageV1::Algebra,
        "tagged algebra stage"
    );
    assert_eq!(diag.stage().as_str(), EXPECTED_PARITY_STAGE_V1);
    assert_eq!(
        diag.code(),
        EXPECTED_PARITY_CODE_V1,
        "pipeline diagnostic code drifted from the parity anchor"
    );
}

#[test]
fn lsp_v3_diagnostics_code_equals_pipeline_code_v1() {
    // Compute the pipeline code directly (no LSP-local reasoning).
    let pipeline_code = compile_program(ADVANCED_FAILING_SRC_V1)
        .expect_err("advanced query must reject")
        .code();

    // Drive the SAME source through the spawned LSP binary.
    let escaped = ADVANCED_FAILING_SRC_V1
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let request = format!(r#"{{"method":"v3_diagnostics","query":"{escaped}"}}"#);
    let (status, line) = run_binary_line_v1(&request);
    assert_eq!(status, Some(0), "v3_diagnostics must exit clean: {line}");

    // The LSP surface must carry the EXACT pipeline stage and code, verbatim.
    assert!(
        line.contains(&format!(r#""stage":"{EXPECTED_PARITY_STAGE_V1}""#)),
        "LSP must tag the pipeline stage: {line}"
    );
    assert!(
        line.contains(&format!(r#""code":"{pipeline_code}""#)),
        "LSP v3_diagnostics code must equal the pipeline code `{pipeline_code}`: {line}"
    );
    // And that pipeline code is the parity anchor the CLI worker also asserts.
    assert_eq!(pipeline_code, EXPECTED_PARITY_CODE_V1);
}

#[test]
fn lsp_v3_diagnostics_preserves_parse_stage_code_too_v1() {
    // A second stage to prove the LSP path is genuinely stage-faithful, not a
    // hard-coded single-stage shim: an unclosed paren rejects at PARSE.
    let src = "from users | where (a + 1";
    let pipeline = compile_program(src).expect_err("must reject at parse");
    assert_eq!(pipeline.stage(), PipelineStageV1::Parse);
    let pipeline_code = pipeline.code();

    let escaped = src.replace('\\', "\\\\").replace('"', "\\\"");
    let request = format!(r#"{{"method":"v3_diagnostics","query":"{escaped}"}}"#);
    let (status, line) = run_binary_line_v1(&request);
    assert_eq!(status, Some(0), "v3_diagnostics must exit clean: {line}");
    assert!(
        line.contains(r#""stage":"parse""#),
        "LSP must tag the parse stage: {line}"
    );
    assert!(
        line.contains(&format!(r#""code":"{pipeline_code}""#)),
        "LSP parse-stage code must equal pipeline code `{pipeline_code}`: {line}"
    );
}

/// Drive `src` through BOTH the in-process pipeline (the CLI's side of the one
/// shared compiler) and the spawned LSP binary, and assert the LSP surface
/// carries the EXACT stage tag, code, and primary span the pipeline emits. This
/// is the generalized parity assertion the per-family rails below reuse so each
/// family proves identical, verbatim, span-faithful parity — never an LSP-local
/// rewrite or downgrade.
fn assert_lsp_matches_pipeline_v1(src: &str, expected_stage: PipelineStageV1, expected_code: &str) {
    // Pipeline truth (CLI side): the family rejects at the named stage with the
    // anchored code and a concrete primary span.
    let pipeline = compile_program(src).expect_err("advanced family source must reject");
    assert_eq!(
        pipeline.stage(),
        expected_stage,
        "pipeline stage drifted for `{src}`"
    );
    assert_eq!(
        pipeline.code(),
        expected_code,
        "pipeline code drifted from the parity anchor for `{src}`"
    );
    let span = pipeline.span();

    // LSP side: the SAME source through the spawned binary front door.
    let escaped = src.replace('\\', "\\\\").replace('"', "\\\"");
    let request = format!(r#"{{"method":"v3_diagnostics","query":"{escaped}"}}"#);
    let (status, line) = run_binary_line_v1(&request);
    assert_eq!(status, Some(0), "v3_diagnostics must exit clean: {line}");
    assert!(
        line.contains(&format!(r#""stage":"{}""#, expected_stage.as_str())),
        "LSP must tag the `{}` stage for `{src}`: {line}",
        expected_stage.as_str()
    );
    assert!(
        line.contains(&format!(r#""code":"{expected_code}""#)),
        "LSP code must equal pipeline code `{expected_code}` for `{src}`: {line}"
    );
    // Span parity: a stage tag with a drifted span would still mislocate the
    // squiggle, so pin both ends to the pipeline's primary span verbatim.
    assert!(
        line.contains(&format!(
            r#""span_start":{},"span_end":{}"#,
            span.start, span.end
        )),
        "LSP span must equal pipeline span {}..{} for `{src}`: {line}",
        span.start,
        span.end
    );
}

/// FIND family (graph-query / logic contract): a cyclic reachability witness with
/// no path-length bound binds and types, but the logic stage refuses to
/// materialize an unbounded witness. The LSP must surface that `logic`-stage code
/// verbatim — a family whose rejection lives entirely past the relational stages.
#[test]
fn lsp_v3_diagnostics_parity_for_find_logic_stage_v1() {
    assert_lsp_matches_pipeline_v1(
        "find reachability a.b -> c.d",
        PipelineStageV1::Logic,
        "logic.unbounded_witness.missing_path_length_bound",
    );
}

/// META family (`USE PACK` directive / modules contract): activating a pack with
/// no resolvable manifest rejects at the `modules` stage. Proves the LSP path
/// reaches the catalog/registration stage too, with the module diagnostic paired
/// to the directive span.
#[test]
fn lsp_v3_diagnostics_parity_for_meta_modules_stage_v1() {
    assert_lsp_matches_pipeline_v1(
        "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock",
        PipelineStageV1::Modules,
        "modules.unknown_pack_manifest",
    );
}

/// COMPOSITION family (`LET .. WITH`): a `with` clause naming a binding that was
/// never declared is a fail-closed composition error caught at the PARSE stage.
/// The LSP must surface the same `surface.parse.unsupported_construct` code the
/// pipeline emits, pointing at the offending `with` reference.
#[test]
fn lsp_v3_diagnostics_parity_for_composition_undefined_binding_v1() {
    assert_lsp_matches_pipeline_v1(
        "let a = from users u find taint api.entry -> db.write with b",
        PipelineStageV1::Parse,
        "surface.parse.unsupported_construct",
    );
}

/// COMPOSITION family, deeper stage: a well-formed `LET .. WITH` whose BODY is an
/// unbounded FIND rejects at the `logic` stage. Proves the LSP surfaces a
/// diagnostic originating inside a composition body verbatim, with the span
/// pointing into the body rather than the composition header.
#[test]
fn lsp_v3_diagnostics_parity_for_composition_body_logic_stage_v1() {
    assert_lsp_matches_pipeline_v1(
        "let suspects = sample query=\"auth\" | mode=semantic | limit=3 \
         find reachability a.b -> c.d with suspects",
        PipelineStageV1::Logic,
        "logic.unbounded_witness.missing_path_length_bound",
    );
}
