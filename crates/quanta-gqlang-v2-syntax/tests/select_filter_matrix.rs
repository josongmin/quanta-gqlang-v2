#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing, exercise fail-closed panics, and bind-all match arms that panic on unexpected variants"
)]

//! Contract: exhaustive SELECT filter / literal / LIMIT decoding matrix.
//!
//! `parser_conformance.rs` proves a single SELECT shape (`WHERE active = true`)
//! decodes. That left every non-`=` comparison operator, the integer literal
//! kind (especially the negative branch), and the `LIMIT` non-negative guard
//! parser-supported but **unpinned**: a regression in `>=` parsing or in the
//! `LIMIT -1` rejection would pass CI silently.
//!
//! This rail closes that gap. It drives every value of `SelectFilterOpV1` and
//! `SelectLiteralAstV1` through the real `parse_document_v1` front door, asserts
//! the decoded shape, re-derives the canonical query, and proves the variant is
//! a `format_document_v1` fixed point. The negative half pins the `LIMIT`
//! fail-closed branches to their parser diagnostic codes.
//!
//! Frontend-only: this asserts parse/normalize/format determinism, never runtime
//! execution or result materialization.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::roots::select::{
    SelectFilterOpV1, SelectLiteralAstV1, SelectProjectionAstV1, SelectQueryShapeV1,
};
use quanta_gqlang_v2_syntax::{format_document_v1, parse_document_v1};

/// Parse a single-root document and return the decoded SELECT shape, panicking
/// with the offending input if the front door rejects it or yields a non-SELECT
/// root. The `shape_v1()` call re-parses the canonical carrier, so a passing
/// case has survived parse -> canonicalize -> re-parse.
fn select_shape_v1(input: &str) -> SelectQueryShapeV1 {
    let doc = parse_document_v1(input)
        .unwrap_or_else(|page| panic!("expected `{input}` to parse: {page:?}"));
    assert_eq!(
        doc.roots.len(),
        1,
        "expected exactly one root for `{input}`"
    );
    match doc.roots.into_iter().next().expect("one root") {
        AstRootV1::Select(select) => select
            .shape_v1()
            .unwrap_or_else(|err| panic!("expected `{input}` to yield a shape: {err:?}")),
        other => panic!("expected a SELECT root for `{input}`, got {other:?}"),
    }
}

/// Format a single-root document once through the public formatter.
fn fmt_once_v1(input: &str) -> String {
    let doc = parse_document_v1(input)
        .unwrap_or_else(|page| panic!("expected `{input}` to parse for fmt: {page:?}"));
    format_document_v1(&doc)
}

/// Parse a document expected to fail closed and return its first diagnostic code.
fn select_error_code_v1(input: &str) -> &'static str {
    parse_document_v1(input)
        .expect_err(&format!("expected `{input}` to fail closed"))
        .diagnostics
        .into_iter()
        .next()
        .expect("one diagnostic")
        .code
}

/// The decoded filter of `shape`, panicking if no WHERE clause survived.
fn filter_of_v1(
    shape: &SelectQueryShapeV1,
) -> &quanta_gqlang_v2_syntax::roots::select::SelectFilterAstV1 {
    shape
        .filter
        .as_ref()
        .expect("expected a WHERE filter to be present")
}

struct OperatorCaseV1 {
    input: &'static str,
    op: SelectFilterOpV1,
    canonical_fragment: &'static str,
}

#[test]
fn every_select_comparison_operator_decodes_and_round_trips_v1() {
    let cases = [
        OperatorCaseV1 {
            input: "SELECT id FROM catalog.users WHERE score = 5",
            op: SelectFilterOpV1::Eq,
            canonical_fragment: "score = 5",
        },
        OperatorCaseV1 {
            input: "SELECT id FROM catalog.users WHERE score != 5",
            op: SelectFilterOpV1::NotEq,
            canonical_fragment: "score != 5",
        },
        OperatorCaseV1 {
            input: "SELECT id FROM catalog.users WHERE score < 5",
            op: SelectFilterOpV1::Lt,
            canonical_fragment: "score < 5",
        },
        OperatorCaseV1 {
            input: "SELECT id FROM catalog.users WHERE score <= 5",
            op: SelectFilterOpV1::Lte,
            canonical_fragment: "score <= 5",
        },
        OperatorCaseV1 {
            input: "SELECT id FROM catalog.users WHERE score > 5",
            op: SelectFilterOpV1::Gt,
            canonical_fragment: "score > 5",
        },
        OperatorCaseV1 {
            input: "SELECT id FROM catalog.users WHERE score >= 5",
            op: SelectFilterOpV1::Gte,
            canonical_fragment: "score >= 5",
        },
    ];

    // Every operator variant must be exercised by the table — guard against a
    // silently dropped row if `SelectFilterOpV1` ever grows.
    assert_eq!(
        cases.len(),
        6,
        "operator matrix must cover all six operators"
    );

    for case in &cases {
        let shape = select_shape_v1(case.input);
        assert_eq!(
            filter_of_v1(&shape).op,
            case.op,
            "operator mismatch for `{}`",
            case.input
        );

        let canonical = shape.canonical_query_v1();
        assert!(
            canonical.contains(case.canonical_fragment),
            "canonical `{canonical}` missing `{}` for `{}`",
            case.canonical_fragment,
            case.input
        );

        // Format is a fixed point and preserves the operator token.
        let formatted = fmt_once_v1(case.input);
        assert!(
            formatted.contains(case.canonical_fragment),
            "fmt `{formatted}` dropped `{}` for `{}`",
            case.canonical_fragment,
            case.input
        );
        let reformatted = fmt_once_v1(formatted.trim());
        assert_eq!(
            formatted, reformatted,
            "fmt not idempotent for operator case `{}`",
            case.input
        );
    }
}

#[test]
fn every_select_literal_kind_decodes_including_negative_integer_v1() {
    // String literal with embedded `''` escape.
    let string_shape = select_shape_v1("SELECT id FROM catalog.users WHERE region = 'us-east'");
    assert_eq!(
        filter_of_v1(&string_shape).value,
        SelectLiteralAstV1::String("us-east".to_string())
    );

    // Positive integer literal — previously only seen in a malformed-order LSP
    // hover fixture, now pinned on the happy path.
    let positive_shape = select_shape_v1("SELECT id FROM catalog.users WHERE score = 42");
    assert_eq!(
        filter_of_v1(&positive_shape).value,
        SelectLiteralAstV1::Integer(42)
    );

    // Negative integer literal — exercises the `Minus` token branch of the
    // literal parser, which had zero coverage.
    let negative_shape = select_shape_v1("SELECT id FROM catalog.users WHERE delta = -17");
    assert_eq!(
        filter_of_v1(&negative_shape).value,
        SelectLiteralAstV1::Integer(-17)
    );
    assert!(
        negative_shape.canonical_query_v1().contains("delta = -17"),
        "negative integer dropped from canonical: {}",
        negative_shape.canonical_query_v1()
    );

    // Boolean literals, both values.
    let true_shape = select_shape_v1("SELECT id FROM catalog.users WHERE active = true");
    assert_eq!(
        filter_of_v1(&true_shape).value,
        SelectLiteralAstV1::Boolean(true)
    );
    let false_shape = select_shape_v1("SELECT id FROM catalog.users WHERE active = false");
    assert_eq!(
        filter_of_v1(&false_shape).value,
        SelectLiteralAstV1::Boolean(false)
    );

    // The negative-integer literal is also a format fixed point.
    let formatted = fmt_once_v1("SELECT id FROM catalog.users WHERE delta = -17");
    assert_eq!(formatted, fmt_once_v1(formatted.trim()));
}

#[test]
fn select_limit_accepts_non_negative_and_rejects_negative_v1() {
    // Non-negative bounds (including zero) are accepted.
    let zero = select_shape_v1("SELECT id FROM catalog.users LIMIT 0");
    assert_eq!(zero.limit, Some(0));
    let positive = select_shape_v1("SELECT id FROM catalog.users LIMIT 25");
    assert_eq!(positive.limit, Some(25));

    // A WHERE + LIMIT pair carrying a negative comparison literal AND a positive
    // bound must keep both — the filter sign must not leak into the bound.
    let combined = select_shape_v1("SELECT id FROM catalog.users WHERE delta >= -3 LIMIT 9");
    assert_eq!(combined.limit, Some(9));
    assert_eq!(filter_of_v1(&combined).op, SelectFilterOpV1::Gte);
    assert_eq!(
        filter_of_v1(&combined).value,
        SelectLiteralAstV1::Integer(-3)
    );

    // A negative LIMIT is the fail-closed branch: `LIMIT` is `u64`.
    assert_eq!(
        select_error_code_v1("SELECT id FROM catalog.users LIMIT -5"),
        "invalid_limit_value",
        "negative LIMIT must fail closed"
    );

    // A non-numeric LIMIT fails closed on a distinct branch.
    assert_eq!(
        select_error_code_v1("SELECT id FROM catalog.users LIMIT lots"),
        "expected_limit_value",
        "non-numeric LIMIT must fail closed"
    );

    // An overflowing LIMIT (> u64::MAX) fails closed rather than wrapping.
    assert_eq!(
        select_error_code_v1("SELECT id FROM catalog.users LIMIT 99999999999999999999999"),
        "invalid_limit_value",
        "overflowing LIMIT must fail closed"
    );
}

#[test]
fn select_wildcard_and_dotted_projection_decode_v1() {
    // Wildcard projection round-trips as `*`.
    let wildcard = select_shape_v1("SELECT * FROM catalog.users");
    assert_eq!(wildcard.projection, SelectProjectionAstV1::Wildcard);
    assert!(wildcard.canonical_query_v1().starts_with("SELECT * FROM"));

    // Dotted field projection survives with its full path.
    let dotted = select_shape_v1("SELECT id, profile.name FROM catalog.users");
    match dotted.projection {
        SelectProjectionAstV1::Fields(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[1].dotted_name_v1(), "profile.name");
        }
        SelectProjectionAstV1::Wildcard => panic!("expected explicit fields"),
    }
}
