#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Declaration-family negative conformance (fail-closed parser contract).
//!
//! Pins the fail-closed branches of the declaration root family
//! (VIEW / RULE / MODEL / PACK / PROFILE / GATE) at the `parse_document_v1`
//! boundary. The parser dispatch assigns each family two diagnostic codes: a
//! `<family>_missing_name` code when the bare keyword carries no payload, and a
//! single `invalid_<family>_declaration` code into which every shape-decoding
//! failure funnels. Sub-branches within `invalid_<family>_declaration` are then
//! distinguished by a stable `.message` substring (asserted alongside `.code`).
//!
//! Coverage ledger (negative branches pinned here, by family):
//!
//! ```text
//! family   | missing payload          | missing required clause      | unsupported clause       | malformed carrier (empty `;;` / non key=value) | malformed carrier (identifier / enum value)
//! ---------+--------------------------+------------------------------+--------------------------+------------------------------------------------+--------------------------------------------
//! VIEW     | view_missing_name        | invalid_view_declaration     | invalid_view_declaration | invalid_view_declaration                       | invalid_view_declaration (multi-token name)
//! RULE     | rule_missing_name        | invalid_rule_declaration     | invalid_rule_declaration | invalid_rule_declaration                       | -
//! MODEL    | model_missing_name       | invalid_model_declaration    | invalid_model_declaration| invalid_model_declaration                      | -
//! PACK     | pack_missing_name        | invalid_pack_declaration     | invalid_pack_declaration | invalid_pack_declaration                       | invalid_pack_declaration (activation enum)
//! PROFILE  | profile_missing_name     | invalid_profile_declaration  | invalid_profile_declaration | invalid_profile_declaration                 | invalid_profile_declaration (admission enum / whitespace id)
//! GATE     | gate_missing_name        | invalid_gate_declaration     | invalid_gate_declaration | invalid_gate_declaration                       | invalid_gate_declaration (binding enum / whitespace id)
//! ```
//!
//! Notes:
//! - VIEW/RULE/MODEL decode via `split_decl_payload_v1` ("must use key=value
//!   form", "must be a single identifier token"); PACK/PROFILE/GATE decode via
//!   `decode_declaration_carrier_v1` ("must be key=value", "must not contain
//!   whitespace"). The codes are identical per family; messages differ by source.
//! - RULE/MODEL carry no enum or identifier-shape validation on their carrier
//!   beyond the shared name/clause checks, so the rightmost ledger column is `-`.

use quanta_gqlang_v2_syntax::{parse_document_v1, SyntaxDiagnosticV1};

fn parse_error_v1(input: &str) -> SyntaxDiagnosticV1 {
    parse_document_v1(input)
        .expect_err("expected parse failure")
        .diagnostics
        .into_iter()
        .next()
        .expect("one diagnostic")
}

/// One fail-closed declaration case: a malformed input plus the diagnostic
/// contract it must pin (the exact `.code`, and a stable `.message` substring
/// that distinguishes the sub-branch when several share one code).
struct DeclNegativeCaseV1 {
    label: &'static str,
    input: &'static str,
    expect_code: &'static str,
    expect_message_contains: &'static str,
}

fn assert_decl_negatives_v1(cases: &[DeclNegativeCaseV1]) {
    for case in cases {
        let diagnostic = parse_error_v1(case.input);
        assert_eq!(
            diagnostic.code, case.expect_code,
            "[{}] expected code `{}` for input `{}`, got `{}` ({})",
            case.label, case.expect_code, case.input, diagnostic.code, diagnostic.message
        );
        assert!(
            diagnostic.message.contains(case.expect_message_contains),
            "[{}] expected message to contain `{}` for input `{}`, got `{}`",
            case.label,
            case.expect_message_contains,
            case.input,
            diagnostic.message
        );
    }
}

#[test]
fn view_declaration_fails_closed_v1() {
    assert_decl_negatives_v1(&[
        DeclNegativeCaseV1 {
            label: "view_missing_payload",
            input: "VIEW",
            expect_code: "view_missing_name",
            expect_message_contains: "VIEW requires payload",
        },
        DeclNegativeCaseV1 {
            label: "view_missing_required_query",
            input: "VIEW catalog.users",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "requires `query-ref` or `body`",
        },
        DeclNegativeCaseV1 {
            label: "view_unsupported_clause",
            input: "VIEW catalog.users ;; query-ref=queries/catalog/users ;; format=pretty",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "clause `format` is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "view_carrier_empty_clause",
            input: "VIEW catalog.users ;;",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "clause 2 is empty",
        },
        DeclNegativeCaseV1 {
            label: "view_carrier_not_key_value",
            input: "VIEW catalog.users ;; query-ref",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "must use key=value form",
        },
        DeclNegativeCaseV1 {
            label: "view_malformed_identifier",
            input: "VIEW name with spaces ;; query-ref=queries/catalog/users",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "name must be a single identifier token",
        },
    ]);
}

#[test]
fn rule_declaration_fails_closed_v1() {
    assert_decl_negatives_v1(&[
        DeclNegativeCaseV1 {
            label: "rule_missing_payload",
            input: "RULE",
            expect_code: "rule_missing_name",
            expect_message_contains: "RULE requires payload",
        },
        DeclNegativeCaseV1 {
            label: "rule_missing_required_check_ref",
            input: "RULE shell.exec",
            expect_code: "invalid_rule_declaration",
            expect_message_contains: "requires `check-ref`",
        },
        DeclNegativeCaseV1 {
            label: "rule_unsupported_clause",
            input: "RULE shell.exec ;; check-ref=checks/shell/exec ;; severity=high",
            expect_code: "invalid_rule_declaration",
            expect_message_contains: "clause `severity` is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "rule_carrier_empty_clause",
            input: "RULE shell.exec ;;",
            expect_code: "invalid_rule_declaration",
            expect_message_contains: "clause 2 is empty",
        },
        DeclNegativeCaseV1 {
            label: "rule_carrier_not_key_value",
            input: "RULE shell.exec ;; check-ref",
            expect_code: "invalid_rule_declaration",
            expect_message_contains: "must use key=value form",
        },
    ]);
}

#[test]
fn model_declaration_fails_closed_v1() {
    assert_decl_negatives_v1(&[
        DeclNegativeCaseV1 {
            label: "model_missing_payload",
            input: "MODEL",
            expect_code: "model_missing_name",
            expect_message_contains: "MODEL requires payload",
        },
        DeclNegativeCaseV1 {
            label: "model_missing_required_source",
            input: "MODEL embeddings.code ;; provider=openai",
            expect_code: "invalid_model_declaration",
            expect_message_contains: "requires `source`",
        },
        DeclNegativeCaseV1 {
            label: "model_missing_required_provider",
            input: "MODEL embeddings.code ;; source=models/code-search",
            expect_code: "invalid_model_declaration",
            expect_message_contains: "requires `provider`",
        },
        DeclNegativeCaseV1 {
            label: "model_unsupported_clause",
            input:
                "MODEL embeddings.code ;; provider=openai ;; source=models/code-search ;; region=us",
            expect_code: "invalid_model_declaration",
            expect_message_contains: "clause `region` is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "model_carrier_empty_clause",
            input: "MODEL embeddings.code ;;",
            expect_code: "invalid_model_declaration",
            expect_message_contains: "clause 2 is empty",
        },
    ]);
}

#[test]
fn pack_declaration_fails_closed_v1() {
    assert_decl_negatives_v1(&[
        DeclNegativeCaseV1 {
            label: "pack_missing_payload",
            input: "PACK",
            expect_code: "pack_missing_name",
            expect_message_contains: "PACK requires payload",
        },
        DeclNegativeCaseV1 {
            label: "pack_missing_required_activation",
            input: "PACK secure-pack ;; source=registry::security",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "missing `activation` clause",
        },
        DeclNegativeCaseV1 {
            label: "pack_missing_required_source",
            input: "PACK secure-pack ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "missing `source` clause",
        },
        DeclNegativeCaseV1 {
            label: "pack_unsupported_clause",
            input: "PACK secure-pack ;; source=registry::security ;; activation=workspace ;; scope=ambient",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "clause 'scope' is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "pack_carrier_empty_clause",
            input: "PACK secure-pack ;;",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "clause 2 is empty",
        },
        DeclNegativeCaseV1 {
            label: "pack_carrier_not_key_value",
            input: "PACK secure-pack ;; source=registry::security ;; activation",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "must be key=value",
        },
        DeclNegativeCaseV1 {
            label: "pack_carrier_empty_name",
            input: "PACK ;; source=registry::security ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "clause 1 is empty",
        },
        DeclNegativeCaseV1 {
            label: "pack_invalid_activation_enum",
            input: "PACK secure-pack ;; source=registry::security ;; activation=manual-only",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "activation mode 'manual-only' is unsupported",
        },
    ]);
}

#[test]
fn profile_declaration_fails_closed_v1() {
    assert_decl_negatives_v1(&[
        DeclNegativeCaseV1 {
            label: "profile_missing_payload",
            input: "PROFILE",
            expect_code: "profile_missing_name",
            expect_message_contains: "PROFILE requires payload",
        },
        DeclNegativeCaseV1 {
            label: "profile_missing_required_capabilities",
            input: "PROFILE strict-review ;; admission=certified-only",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "missing `capabilities` clause",
        },
        DeclNegativeCaseV1 {
            label: "profile_missing_required_admission",
            input: "PROFILE strict-review ;; capabilities=graph.read,graph.trace",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "missing `admission` clause",
        },
        DeclNegativeCaseV1 {
            label: "profile_unsupported_clause",
            input: "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only ;; scope=repo",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "clause 'scope' is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "profile_invalid_admission_enum",
            input: "PROFILE strict-review ;; capabilities=graph.read ;; admission=maybe",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "admission policy 'maybe' is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "profile_malformed_identifier",
            input: "PROFILE strict-review ;; capabilities=graph read ;; admission=certified-only",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "must not contain whitespace",
        },
    ]);
}

#[test]
fn gate_declaration_fails_closed_v1() {
    assert_decl_negatives_v1(&[
        DeclNegativeCaseV1 {
            label: "gate_missing_payload",
            input: "GATE",
            expect_code: "gate_missing_name",
            expect_message_contains: "GATE requires payload",
        },
        DeclNegativeCaseV1 {
            label: "gate_missing_required_binding",
            input: "GATE release-gate ;; checks=owasp.sql,authz.path",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "missing `binding` clause",
        },
        DeclNegativeCaseV1 {
            label: "gate_missing_required_checks",
            input: "GATE release-gate ;; binding=all",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "missing `checks` clause",
        },
        DeclNegativeCaseV1 {
            label: "gate_unsupported_clause",
            input:
                "GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all ;; window=daily",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "clause 'window' is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "gate_invalid_binding_enum",
            input: "GATE release-gate ;; checks=owasp.sql ;; binding=some",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "binding mode 'some' is unsupported",
        },
        DeclNegativeCaseV1 {
            label: "gate_malformed_identifier",
            input: "GATE release-gate ;; checks=owasp sql ;; binding=all",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "must not contain whitespace",
        },
    ]);
}
