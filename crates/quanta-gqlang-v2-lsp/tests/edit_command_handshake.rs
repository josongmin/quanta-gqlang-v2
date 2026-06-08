#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_lsp::server::LspServerV1;

#[test]
fn edit_command_bridge_remains_forbidden_v1() {
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
            .expect_err("mutating method must fail closed")
            .render_json_v1();
        assert!(
            rendered.contains(r#""code":"command_plane_forbidden""#),
            "{method}: {rendered}"
        );
        assert!(rendered.contains(method), "{method}: {rendered}");
    }
}

#[test]
fn non_command_unknown_method_stays_unsupported_v1() {
    let server = LspServerV1::new();
    let rendered = server
        .dispatch_json_v1(r#"{"method":"workspace/index","query":"SELECT id FROM catalog.files"}"#)
        .expect_err("unknown non-command method should remain unsupported")
        .render_json_v1();

    assert!(rendered.contains(r#""code":"unsupported_method""#));
    assert!(rendered.contains("workspace/index"));
}
