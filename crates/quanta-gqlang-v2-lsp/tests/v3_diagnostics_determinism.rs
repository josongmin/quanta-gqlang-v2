#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! v3 diagnostics determinism at the binary front door (ticket 08, D22).
//!
//! syntax->sema->ir path pinned: source text -> the `v3_diagnostics` stdio method
//! -> `pipeline::compile_program` -> the first failing stage's typed diagnostic,
//! flattened to a stage/code/message/span JSON row.
//!
//! Extends the `stdio_completion_determinism` pattern to the new read-only
//! surface: the v3 diagnostics method is a pure function of the source, so the
//! SAME advanced source must produce a BYTE-IDENTICAL ordered response across
//! separate process spawns. The pipeline short-circuits on the first stage, so
//! the ordered list is deterministic by construction; pinning the full line kills
//! any reorder, field-value, or span-drift mutation in one assertion.

mod support;

use support::run_binary_line_v1;

/// An advanced query that binds cleanly but is ill-formed at the ALGEBRA stage:
/// a `group by` keyed on an aggregate fold (`count(f.id)`) rejects with
/// `algebra.ill_formed_aggregate.nested_aggregate`, because an aggregate cannot
/// be a partition key. (`group by`/`order by` themselves now lower cleanly; the
/// fail-closed algebra-stage carrier is exercised via this still-genuine
/// rejection.) The full stage-tagged line is pinned, so any drift in
/// stage/code/message/span fails.
const ALGEBRA_DIAG_LINE_V1: &str = r#"{"ok":true,"method":"v3_diagnostics","result":[{"stage":"algebra","code":"algebra.ill_formed_aggregate.nested_aggregate","message":"[algebra] algebra.ill_formed_aggregate.nested_aggregate: aggregate call must be a top-level projection item, not nested in an expression","span_start":27,"span_end":38}]}"#;

/// A FIND-family (graph-query) source that binds and types cleanly but rejects
/// at the LOGIC stage: a cyclic reachability witness with no path-length bound
/// cannot be bounded-materialized. This pins a NON-relational family end to end
/// through the same stage-tagged surface, proving the determinism guarantee
/// holds for every item kind the generalized pipeline routes — not just the
/// relational/algebra path. The full line is pinned, so any stage/code/message/
/// span drift in the FIND path fails here.
const FIND_LOGIC_DIAG_LINE_V1: &str = r#"{"ok":true,"method":"v3_diagnostics","result":[{"stage":"logic","code":"logic.unbounded_witness.missing_path_length_bound","message":"[logic] logic.unbounded_witness.missing_path_length_bound: unbounded witness materialization: a cyclic relation requires a path-length bound","span_start":0,"span_end":28}]}"#;

/// A META-family (`USE PACK`) source that rejects at the MODULES stage: a pack
/// with no resolvable manifest. Pins the catalog/registration stage on the
/// read-only surface so the meta family's deterministic ordering is asserted too.
const META_MODULES_DIAG_LINE_V1: &str = r#"{"ok":true,"method":"v3_diagnostics","result":[{"stage":"modules","code":"modules.unknown_pack_manifest","message":"[modules] modules.unknown_pack_manifest: USE PACK `secure-pack` has no resolvable manifest","span_start":0,"span_end":75}]}"#;

fn v3_diagnostics_line_v1(query: &str) -> String {
    let escaped = query.replace('\\', "\\\\").replace('"', "\\\"");
    let request = format!(r#"{{"method":"v3_diagnostics","query":"{escaped}"}}"#);
    let (status, line) = run_binary_line_v1(&request);
    assert_eq!(
        status,
        Some(0),
        "v3_diagnostics must exit clean (a compile failure is data, not transport error): {line}"
    );
    line
}

#[test]
fn v3_diagnostics_advanced_query_is_byte_identical_across_spawns_v1() {
    let src = "from findings f | group by count(f.id)";
    let first = v3_diagnostics_line_v1(src);
    let second = v3_diagnostics_line_v1(src);
    assert_eq!(
        first, second,
        "two independent v3_diagnostics spawns must be byte-identical"
    );
    assert_eq!(
        first, ALGEBRA_DIAG_LINE_V1,
        "stage-tagged advanced-query diagnostic line drifted: {first}"
    );
}

#[test]
fn v3_diagnostics_find_logic_stage_is_byte_identical_across_spawns_v1() {
    // A FIND-family source: the diagnostic lives entirely past the relational
    // stages (LOGIC), so this proves the determinism guarantee generalizes to
    // every item kind, not just the relational/algebra path.
    let src = "find reachability a.b -> c.d";
    let first = v3_diagnostics_line_v1(src);
    let second = v3_diagnostics_line_v1(src);
    assert_eq!(
        first, second,
        "two independent FIND v3_diagnostics spawns must be byte-identical"
    );
    assert_eq!(
        first, FIND_LOGIC_DIAG_LINE_V1,
        "stage-tagged FIND diagnostic line drifted: {first}"
    );
}

#[test]
fn v3_diagnostics_meta_modules_stage_is_byte_identical_across_spawns_v1() {
    // A META (`USE PACK`) source rejecting at the MODULES stage: pins the
    // catalog/registration stage's deterministic ordered surface.
    let src = "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock";
    let first = v3_diagnostics_line_v1(src);
    let second = v3_diagnostics_line_v1(src);
    assert_eq!(
        first, second,
        "two independent META v3_diagnostics spawns must be byte-identical"
    );
    assert_eq!(
        first, META_MODULES_DIAG_LINE_V1,
        "stage-tagged META diagnostic line drifted: {first}"
    );
}

#[test]
fn v3_diagnostics_clean_query_yields_empty_ordered_list_v1() {
    // A query that compiles end-to-end emits no diagnostics: the ordered list is
    // empty, and that empty result is itself deterministic.
    let src = "from users u | where u.id > 1 | project u.id as id, u.name as name";
    let first = v3_diagnostics_line_v1(src);
    let second = v3_diagnostics_line_v1(src);
    assert_eq!(first, second, "clean-compile result must be deterministic");
    assert_eq!(
        first, r#"{"ok":true,"method":"v3_diagnostics","result":[]}"#,
        "a clean compile must surface an empty diagnostics list: {first}"
    );
}
