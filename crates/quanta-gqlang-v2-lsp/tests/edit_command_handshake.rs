#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_lsp::server::LspServerV1;

#[test]
fn edit_command_bridge_is_not_allowed_v1() {
    let server = LspServerV1::new();

    for method in [
        "rename",
        "textDocument/codeAction",
        "workspace/applyEdit",
        "workspace/executeCommand",
    ] {
        let request = format!(r#"{{"method":"{method}","input":"MODEL shell"}}"#);
        let rendered = server
            .dispatch_json_v1(&request)
            .expect_err("non-allowlisted method must fail closed")
            .render_json_v1();
        assert!(
            rendered.contains(r#""code":"method_not_allowed""#),
            "{method}: {rendered}"
        );
        assert!(rendered.contains(method), "{method}: {rendered}");
    }
}

/// The old forbidden-vs-unsupported distinction is gone: an unknown non-mutating
/// method gets the SAME `method_not_allowed` code as a mutating one. The front
/// door is a closed allowlist and never classifies *why* a method is refused.
#[test]
fn unknown_method_is_not_allowed_v1() {
    let server = LspServerV1::new();
    let rendered = server
        .dispatch_json_v1(r#"{"method":"workspace/index","query":"SELECT id FROM catalog.files"}"#)
        .expect_err("unknown method must fail closed")
        .render_json_v1();

    assert!(rendered.contains(r#""code":"method_not_allowed""#));
    assert!(rendered.contains("workspace/index"));
}
