//! v3 seam rail: surface(06) -> algebra(04) `group by` lowering.
//!
//! The surface parser emits `Compose("group_by", [input, keys])` where `keys`
//! is a `Project` carrying the group-key value exprs. Before the fix the algebra
//! normalizer had no lowering for the `group_by` symbol and rejected with
//! `algebra.unsupported_composition`. This rail pins the end-to-end behavior:
//! `from findings f | group by f.repo` must compile through every stage and
//! produce an explicit `Group` algebra root whose keys carry the grouping value
//! expressions end-to-end.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration test rail: assertion panics and indexing are intentional"
)]

use quanta_gqlang_v2_algebra::{AlgebraOpV1, ColumnProvenanceV1};
use quanta_gqlang_v2_hir::ValueExprKindV1;
use quanta_gqlang_v2_pipeline::compile_program_structural;

/// rail: a single-key `group by` compiles end to end into an explicit Group
/// whose key value expression is preserved (the `f.repo` path), the output
/// column carries Source provenance for the alias-resolved relation, and the
/// grouped output is a Set (one row per distinct key tuple).
#[test]
fn group_by_lowers_to_explicit_group_with_carried_key() {
    let src = "from findings f | group by f.repo";
    let compiled = compile_program_structural(src).expect("group by compiles end to end");

    let root = compiled
        .algebra()
        .expect("relational query has an algebra root");

    let AlgebraOpV1::Group(group) = &root.op else {
        panic!(
            "group by must lower to an explicit Group, got {:?}",
            root.op
        );
    };

    // The grouping key value expression is carried end-to-end: it is the
    // `f.repo` path written in the surface, not dropped or re-synthesized.
    assert_eq!(group.keys.len(), 1, "exactly one grouping key");
    let key = &group.keys[0];
    let ValueExprKindV1::Path(path) = &key.kind else {
        panic!("group key must be the `f.repo` path, got {:?}", key.kind);
    };
    assert_eq!(
        path.segments.last().map(|s| s.name.as_str()),
        Some("repo"),
        "key path resolves to the `repo` field"
    );

    // The Group input is the underlying Scan over `findings`, NOT the synthetic
    // `_keys` placeholder source the surface attaches to the keys projection.
    let AlgebraOpV1::Scan(scan) = &group.input.op else {
        panic!("group input must be the scan, got {:?}", group.input.op);
    };
    assert_eq!(scan.relation, "findings");

    // Output column is the alias-resolved grouping key with Source provenance.
    assert_eq!(root.columns.len(), 1, "output is exactly the key tuple");
    let col = &root.columns[0];
    let ColumnProvenanceV1::Source {
        relation, field, ..
    } = &col.provenance
    else {
        panic!(
            "key column must carry Source provenance, got {:?}",
            col.provenance
        );
    };
    assert_eq!(relation, "findings");
    assert_eq!(field.as_deref(), Some("repo"));
}
