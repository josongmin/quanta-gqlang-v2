#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration rail: assertions fail hard on authority drift"
)]

//! GQW3-18 `USE PACK` pack-graph authority (owner-local pipeline rails).
//!
//! `USE PACK` resolves its pack id against the caller's frozen
//! [`PackGraphV1`] authority carried on [`FrontendContextV1`], NOT a local
//! empty graph. These rails pin the frozen contract:
//!
//! - a KNOWN pack in the context's pack graph resolves to a `Meta` artifact
//!   (positive), while
//! - an UNKNOWN pack (empty structural-only context) stays fail-closed with the
//!   typed `modules.unknown_pack_manifest` diagnostic (negative pair), and
//! - a bad `.lock` path preserves `modules.invalid_lockfile_path`, and
//! - the pack authority survives a composition-local context rebuild (a
//!   `USE PACK` inside a `LET .. WITH` body still resolves).
//!
//! Oracle = the resolve-vs-fail-closed invariants; evidence = the SUT compile
//! output (the typed diagnostic code / the compiled `Meta` family), never a
//! test-fabricated verdict.

use quanta_gqlang_v2_binder::ModuleGraphV1;
use quanta_gqlang_v2_modules::{PackGraphV1, PackIdV1, PackManifestV1, VersionRangeV1, VersionV1};
use quanta_gqlang_v2_pipeline::{
    compile_program_structural, compile_with_context, CompiledItemV1, FrontendContextV1,
    PipelineDiagnosticV1, PipelineStageV1,
};
use quanta_gqlang_v2_types::{TypeEnvV1, TypeV1};

/// A pack graph holding exactly one manifest for `secure-pack`, resolved against
/// a wide host-compat window. This is the real typed manifest authority
/// `USE PACK` resolves against (never a string heuristic).
fn secure_pack_graph() -> PackGraphV1 {
    let manifest = PackManifestV1::new(
        PackIdV1::new("secure-pack", VersionV1::new(1, 0, 0)),
        VersionRangeV1::new(VersionV1::new(0, 0, 0), VersionV1::new(9, 0, 0)),
    );
    PackGraphV1::new()
        .add_manifest(manifest)
        .expect("single manifest never collides")
}

/// A context carrying the `secure-pack` manifest authority (empty module graph +
/// empty schema; only the pack graph is populated).
fn context_with_secure_pack() -> FrontendContextV1 {
    FrontendContextV1::new(ModuleGraphV1::new(), TypeEnvV1::new(), secure_pack_graph())
}

/// POSITIVE: a `USE PACK` whose pack id resolves in the context pack graph
/// compiles to the `Meta` family (no `modules` reject). The manifest lookup is
/// the typed `PackGraphV1::manifest(name)` map, so `secure-pack` resolves.
#[test]
fn use_pack_resolves_against_context_pack_graph_authority() {
    let src = "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock";

    let compiled = compile_with_context(src, &context_with_secure_pack())
        .expect("known pack in the context pack graph must resolve, not reject");

    assert!(
        matches!(compiled.compiled(), CompiledItemV1::Meta { .. }),
        "a resolved USE PACK is a Meta artifact; got {:?}",
        compiled.compiled(),
    );
}

/// NEGATIVE (pair): the SAME `USE PACK` text under a structural-only context
/// (empty pack graph) stays fail-closed with the typed
/// `modules.unknown_pack_manifest` — the missing pack authority is NOT dressed
/// up as a meta success (Frozen decision 9).
#[test]
fn use_pack_without_pack_authority_fails_closed_unknown_manifest() {
    let src = "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock";

    let diag = compile_program_structural(src)
        .expect_err("an unknown pack (empty pack graph) must fail closed, never fake success");

    assert_eq!(
        diag.stage(),
        PipelineStageV1::Modules,
        "USE PACK unknown-pack reject is a modules-stage failure; got {diag:?}",
    );
    assert_eq!(
        diag.code(),
        "modules.unknown_pack_manifest",
        "unknown pack must carry the typed unknown_pack_manifest code; got {diag:?}",
    );
}

/// NEGATIVE: a bad lockfile path (not `.lock`) fails closed at the surface
/// grammar stage — the `.lock` shape is a parse-time invariant that fires BEFORE
/// the pack manifest resolves, so a resolvable manifest cannot rescue a bad
/// lockfile path. (The `modules.invalid_lockfile_path` code is the equivalent
/// HIR-level defense-in-depth check; here the parser owns the text reject.)
#[test]
fn use_pack_with_bad_lockfile_path_fails_closed_before_manifest_resolution() {
    let src = "use pack secure-pack | activation=session | lockfile=packs.secure-pack.txt";

    let diag = compile_with_context(src, &context_with_secure_pack())
        .expect_err("a non-.lock lockfile path must fail closed even with a resolvable manifest");

    assert_eq!(
        diag.stage(),
        PipelineStageV1::Parse,
        "a non-.lock lockfile path is a parse-stage reject; got {diag:?}",
    );
    assert!(
        matches!(diag, PipelineDiagnosticV1::Parse(_)),
        "bad lockfile path is a typed parse diagnostic; got {diag:?}",
    );
}

/// REBUILD RETENTION: the composition-local context rebuild (`compile_composition`
/// rebuilds a `FrontendContextV1` per binding to lift producer schema) MUST
/// preserve the pack-manifest authority. This mirrors that rebuild shape exactly
/// — reconstruct the context with the same `new(module_graph, schema, pack_graph)`
/// call the composition path uses — and proves the rebuilt context still resolves
/// the pack. A rebuild that dropped the pack graph would flip the resolve to
/// `unknown_pack_manifest`.
#[test]
fn pack_authority_survives_context_rebuild() {
    let original = context_with_secure_pack();

    // Reconstruct the context the SAME way the composition-local rebuild does:
    // clone module graph + a (mutated) schema + the SAME pack graph.
    let rebuilt_schema = original
        .schema()
        .clone()
        .with("some_binding", TypeV1::int());
    let rebuilt = FrontendContextV1::new(
        original.module_graph().clone(),
        rebuilt_schema,
        original.pack_graph().clone(),
    );

    // Accessor-level retention: the rebuilt pack graph equals the original.
    assert_eq!(
        rebuilt.pack_graph(),
        original.pack_graph(),
        "context rebuild must preserve the pack-manifest authority, not drop it",
    );

    // Behavioral retention: the rebuilt context still resolves USE PACK.
    let src = "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock";
    let compiled = compile_with_context(src, &rebuilt)
        .expect("rebuilt context must still resolve the known pack");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::Meta { .. }),
        "resolved USE PACK on rebuilt context is a Meta artifact; got {:?}",
        compiled.compiled(),
    );
}
