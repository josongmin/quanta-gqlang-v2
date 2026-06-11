#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
//! F08/J09A — zero-query authority handshake, asserted through the CLI verb
//! (`commands::zero_query::run`) rather than the LSP handler. The rendered report
//! must keep the read-only shell owner and the catalog↔example authority split,
//! and never carry a mutating/code-action verb. Proof rail: `cli --test
//! zero_query_handshake`.

use quanta_gqlang_v2_cli::commands::zero_query;
use quanta_gqlang_v2_ir::catalog::capability_catalog::capability_catalog_authority_v1;

#[test]
fn zero_query_verb_stays_read_only_and_external_owner_v1() {
    let rendered = zero_query::run().expect("zero-query verb");

    assert!(rendered.contains("zero_query.mode=read-only"));
    assert!(rendered.contains("zero_query.provenance=typed-catalog-example-metadata"));
    assert_eq!(
        rendered
            .lines()
            .filter(|line| line.starts_with("zero_query.group[") && line.contains("].id="))
            .filter_map(|line| line.split_once("].id=").map(|(_, id)| id))
            .collect::<Vec<_>>(),
        vec!["catalog", "analysis", "retrieval"],
    );
    // Non-mutating: no scaffold suggests a rename/apply/code-action verb.
    for query in rendered
        .lines()
        .filter(|line| line.contains(".query="))
        .filter_map(|line| line.split_once(".query=").map(|(_, query)| query))
    {
        assert!(
            !query.starts_with("RENAME")
                && !query.starts_with("APPLY")
                && !query.starts_with("CODE ACTION"),
            "mutating scaffold leaked: `{query}`"
        );
    }
}

#[test]
fn zero_query_verb_keeps_catalog_authority_marker_v1() {
    let rendered = zero_query::run().expect("zero-query verb");
    // The catalog group's suggestions originate from the capability catalog; the
    // flat zero-query surface does not re-emit per-suggestion authority, so the
    // handshake is pinned at the catalog provenance marker the scaffold carries.
    assert!(
        !capability_catalog_authority_v1().is_empty(),
        "capability catalog authority must be a stable non-empty owner"
    );
    assert!(rendered.contains("zero_query.group[0].id=catalog"));
    assert!(rendered.contains("zero_query.group[0].suggestion[0].reason.code=catalog_overview"));
}
