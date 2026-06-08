#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_lsp::server::LspServerV1;

#[test]
fn read_only_surface_returns_metadata_without_edit_apply_v1() {
    let server = LspServerV1::new();

    let completion = server
        .dispatch_json_v1(r#"{"method":"completion","prefix":"pro"}"#)
        .expect("completion should stay on the read-only shell");
    assert!(completion.contains(r#""method":"completion""#));
    assert!(completion.contains(r#""label":"PROFILE""#));

    let quickfix = server
        .dispatch_json_v1(r#"{"method":"quickfix","input":"DESCRIBE model::billing_warehouse"}"#)
        .expect("quickfix should stay metadata-only");
    assert!(quickfix.contains(r#""method":"quickfix""#));
    assert!(quickfix.contains(r#""edits_document":false"#));
    assert!(quickfix.contains(r#""authority":"external-read-only-suggestion""#));
    assert!(quickfix.contains(r#""apply_command":null"#));
    assert!(quickfix.contains(r#""workspace_edit":null"#));

    let zero_query = server
        .dispatch_json_v1(r#"{"method":"zero_query"}"#)
        .expect("zero_query should stay read-only");
    assert!(zero_query.contains(r#""method":"zero_query""#));
    assert!(zero_query.contains(r#""mode":"read-only""#));
}
