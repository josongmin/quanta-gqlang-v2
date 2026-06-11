#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_lsp::handlers::completion::handle_completion;
use quanta_gqlang_v2_lsp::handlers::quickfix::handle_quickfix;

#[test]
fn autocomplete_and_quickfix_stay_suggestion_only_v1() {
    let completion = handle_completion("pro").expect("completion should succeed");
    assert_eq!(completion[0].label, "PROFILE");
    assert!(completion[0]
        .example_query
        .as_deref()
        .is_some_and(|query| query.starts_with("PROFILE strict-review")));

    let fixes = handle_quickfix("VIEW catalog.users").expect("quickfix should succeed");
    assert_eq!(fixes.len(), 1);
    for fix in fixes {
        assert_eq!(fix.authority_v1(), "external-read-only-suggestion");
        assert!(fix.apply_command_v1().is_none());
        assert!(fix.workspace_edit_v1().is_none());
        fix.validate_metadata_only_v1()
            .expect("quickfix must remain metadata-only");
    }
}

#[test]
fn malformed_quickfix_input_fails_closed_v1() {
    let err =
        handle_quickfix("DESCRIBE model::").expect_err("malformed target must stay fail-closed");
    assert_eq!(
        err,
        "class=syntax code=describe_target_invalid_shape message=DESCRIBE target must use `<kind>::<identifier>`"
    );
}

#[test]
fn unsupported_quickfix_describe_kind_fails_closed_v1() {
    // Second negative: a well-shaped target whose KIND is outside the closed
    // catalog vocabulary still fails closed with its own typed code, not the
    // shape code above — proving the reject is decoded, not pattern-matched.
    let err = handle_quickfix("DESCRIBE bogus::thing")
        .expect_err("an unsupported describe kind must stay fail-closed");
    assert!(
        err.contains("code=unsupported_describe_kind"),
        "expected unsupported-kind reject, got: {err}"
    );
}
