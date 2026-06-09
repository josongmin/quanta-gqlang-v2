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
//! failure funnels. Because the funnel code does not discriminate sub-branches,
//! every negative case here asserts BOTH the `.code` AND a stable, branch-unique
//! `.message` substring discovered from the live parser (never invented). The
//! `span` field is structurally always `None` on this path (see
//! `parser::diagnostic_page_v1`), so it is pinned to `None` rather than guessed.
//!
//! Each family additionally carries a POSITIVE control (`assert_parses_v1`)
//! asserting its shipped/minimal valid form parses cleanly, so the negatives are
//! not merely "every input errors": a mutation that rejects valid declarations is
//! caught by the positive control, while a mutation that accepts a malformed one
//! is caught by the negative `expect_err`.
//!
//! Coverage ledger (negative branches pinned here, by family):
//!
//! ```text
//! family   | missing  | required clause(s)               | unsupported | empty `;;` | non key=value      | empty value (`key=`)              | duplicate clause                  | name-shape                         | enum / list value
//! ---------+----------+----------------------------------+-------------+------------+--------------------+-----------------------------------+-----------------------------------+------------------------------------+------------------------------------
//! VIEW     | name     | requires query-ref/body          | format      | clause 2   | must use key=value | non-empty key and value           | clause `query-ref` is duplicated  | single identifier / name-not-first | query-ref+body conflict / inline
//! RULE     | name     | requires check-ref               | severity    | clause 2   | must use key=value | non-empty key and value           | clause `check-ref` is duplicated  | -                                  | -
//! MODEL    | name     | requires source / requires provider | region   | clause 2   | -                  | non-empty key and value           | clause `provider` is duplicated   | -                                  | -
//! PACK     | name     | missing source / missing activation | scope    | clause 2   | clause 3 key=value | clause 'source' empty value       | source / activation / name dup    | empty name / whitespace in name    | activation enum
//! PROFILE  | name     | missing capabilities / admission | scope       | -          | -                  | clause 'capabilities' empty value | admission clause is duplicated    | whitespace in capability id        | admission enum / empty list
//! GATE     | name     | missing binding / missing checks | window      | -          | -                  | clause 'checks' empty value       | checks clause is duplicated       | whitespace in check id             | binding enum
//! ```
//!
//! Notes:
//! - VIEW/RULE/MODEL decode via `split_decl_payload_v1` ("must use key=value
//!   form", "must have non-empty key and value", "is duplicated", "must be a
//!   single identifier token"); PACK/PROFILE/GATE decode via
//!   `decode_declaration_carrier_v1` ("must be key=value", "has an empty value",
//!   "clause is duplicated", "must not contain whitespace"). The funnel codes are
//!   identical per family; the message source paths differ, hence the substrings.
//! - RULE/MODEL carry no enum or identifier-list validation on their carrier
//!   beyond the shared name/clause checks.

use quanta_gqlang_v2_syntax::{parse_document_v1, SyntaxDiagnosticV1};

/// Positive control: parse `input`, expecting a clean parse with at least one
/// root. A mutation that turns a valid declaration into a diagnostic (e.g. an
/// over-eager required-clause check) trips the `expect`.
fn assert_parses_v1(input: &str) {
    let document = parse_document_v1(input)
        .unwrap_or_else(|page| panic!("expected `{input}` to parse, got diagnostics: {page:?}"));
    assert!(
        !document.roots.is_empty(),
        "expected `{input}` to yield a root"
    );
}

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
/// that distinguishes the sub-branch when several share one funnel code).
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
        // The declaration diagnostic path never carries a span; pin that
        // structural invariant so a mutation that starts emitting one is caught.
        assert_eq!(
            diagnostic.span, None,
            "[{}] declaration diagnostics must not carry a span (input `{}`)",
            case.label, case.input
        );
    }
}

#[test]
fn view_declaration_fails_closed_v1() {
    // Positive controls: both accepted VIEW forms parse cleanly.
    assert_parses_v1("VIEW catalog.users ;; query-ref=queries/catalog/users");
    assert_parses_v1("VIEW catalog.users ;; body=SELECT id FROM catalog.users LIMIT 25");

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
            label: "view_query_ref_and_body_conflict",
            input: "VIEW catalog.users ;; query-ref=queries/catalog/users ;; body=SELECT id FROM t",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "must choose either `query-ref` or `body`, not both",
        },
        DeclNegativeCaseV1 {
            label: "view_inline_body_invalid",
            input: "VIEW catalog.users ;; body=NOTSELECT id FROM t",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "VIEW inline body is invalid",
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
            expect_message_contains: "clause 2 must use key=value form",
        },
        DeclNegativeCaseV1 {
            label: "view_carrier_empty_value",
            input: "VIEW catalog.users ;; query-ref=",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "clause 2 must have non-empty key and value",
        },
        DeclNegativeCaseV1 {
            label: "view_duplicate_clause",
            input: "VIEW catalog.users ;; query-ref=a ;; query-ref=b",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "clause `query-ref` is duplicated",
        },
        DeclNegativeCaseV1 {
            label: "view_name_not_first_clause",
            input: "VIEW name=catalog.users ;; query-ref=queries/catalog/users",
            expect_code: "invalid_view_declaration",
            expect_message_contains: "name must be the first clause before any key=value pairs",
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
    // Positive control: the shipped RULE form parses cleanly.
    assert_parses_v1("RULE shell.exec ;; check-ref=checks/shell/exec");

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
            expect_message_contains: "clause 2 must use key=value form",
        },
        DeclNegativeCaseV1 {
            label: "rule_carrier_empty_value",
            input: "RULE shell.exec ;; check-ref=",
            expect_code: "invalid_rule_declaration",
            expect_message_contains: "clause 2 must have non-empty key and value",
        },
        DeclNegativeCaseV1 {
            label: "rule_duplicate_clause",
            input: "RULE shell.exec ;; check-ref=a ;; check-ref=b",
            expect_code: "invalid_rule_declaration",
            expect_message_contains: "clause `check-ref` is duplicated",
        },
    ]);
}

#[test]
fn model_declaration_fails_closed_v1() {
    // Positive control: the shipped MODEL form parses cleanly.
    assert_parses_v1("MODEL embeddings.code ;; provider=openai ;; source=models/code-search");

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
        DeclNegativeCaseV1 {
            label: "model_carrier_empty_value",
            input: "MODEL embeddings.code ;; provider= ;; source=models/code-search",
            expect_code: "invalid_model_declaration",
            expect_message_contains: "clause 2 must have non-empty key and value",
        },
        DeclNegativeCaseV1 {
            label: "model_duplicate_clause",
            input: "MODEL embeddings.code ;; provider=a ;; provider=b ;; source=s",
            expect_code: "invalid_model_declaration",
            expect_message_contains: "clause `provider` is duplicated",
        },
    ]);
}

#[test]
fn pack_declaration_fails_closed_v1() {
    // Positive control: the shipped PACK form parses cleanly.
    assert_parses_v1(
        "PACK security-default ;; source=registry://packs/security-default ;; activation=workspace",
    );

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
            expect_message_contains: "clause 3 must be key=value",
        },
        DeclNegativeCaseV1 {
            label: "pack_carrier_empty_value",
            input: "PACK secure-pack ;; source= ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "clause 'source' has an empty value",
        },
        DeclNegativeCaseV1 {
            label: "pack_carrier_empty_name",
            input: "PACK ;; source=registry::security ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "clause 1 is empty",
        },
        DeclNegativeCaseV1 {
            label: "pack_name_whitespace",
            input: "PACK secure pack ;; source=registry::security ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "PACK name must not contain whitespace",
        },
        DeclNegativeCaseV1 {
            label: "pack_duplicate_source",
            input: "PACK secure-pack ;; source=a ;; source=b ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "PACK source clause is duplicated",
        },
        DeclNegativeCaseV1 {
            label: "pack_duplicate_activation",
            input: "PACK secure-pack ;; source=registry::security ;; activation=session ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "PACK activation clause is duplicated",
        },
        DeclNegativeCaseV1 {
            label: "pack_duplicate_name_clause",
            input: "PACK secure-pack ;; name=other ;; source=registry::security ;; activation=workspace",
            expect_code: "invalid_pack_declaration",
            expect_message_contains: "PACK name clause is duplicated",
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
    // Positive control: the shipped PROFILE form parses cleanly.
    assert_parses_v1(
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
    );

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
            label: "profile_carrier_empty_value",
            input: "PROFILE strict-review ;; capabilities= ;; admission=certified-only",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "clause 'capabilities' has an empty value",
        },
        DeclNegativeCaseV1 {
            label: "profile_empty_capability_list",
            input: "PROFILE strict-review ;; capabilities=, ;; admission=certified-only",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "PROFILE capabilities must not be empty",
        },
        DeclNegativeCaseV1 {
            label: "profile_duplicate_capability_entry",
            input: "PROFILE strict-review ;; capabilities=graph.read,graph_read ;; admission=certified-only",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "PROFILE capabilities contains duplicate entry 'graph_read'",
        },
        DeclNegativeCaseV1 {
            label: "profile_duplicate_admission",
            input: "PROFILE strict-review ;; capabilities=graph.read ;; admission=allowlist ;; admission=allowlist",
            expect_code: "invalid_profile_declaration",
            expect_message_contains: "PROFILE admission clause is duplicated",
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
            expect_message_contains: "PROFILE capabilities must not contain whitespace",
        },
    ]);
}

#[test]
fn gate_declaration_fails_closed_v1() {
    // Positive control: the shipped GATE form parses cleanly.
    assert_parses_v1("GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all");

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
            label: "gate_carrier_empty_value",
            input: "GATE release-gate ;; checks= ;; binding=all",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "clause 'checks' has an empty value",
        },
        DeclNegativeCaseV1 {
            label: "gate_duplicate_checks",
            input: "GATE release-gate ;; checks=owasp.sql ;; checks=authz.path ;; binding=all",
            expect_code: "invalid_gate_declaration",
            expect_message_contains: "GATE checks clause is duplicated",
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
            expect_message_contains: "GATE checks must not contain whitespace",
        },
    ]);
}
