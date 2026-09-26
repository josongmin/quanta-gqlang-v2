#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};
use quanta_gqlang_v2_surface::{parse_program, ParseDiagnosticV1, ParseErrorCodeV1};

fn err(src: &str) -> ParseDiagnosticV1 {
    let builder = HirBuilderV1::new();
    parse_program(src, &builder).expect_err("must reject")
}

fn assert_unsupported(src: &str, expected_message: &str) -> ParseDiagnosticV1 {
    let d = err(src);
    assert_eq!(
        d.code,
        ParseErrorCodeV1::UnsupportedConstruct,
        "src {src:?} message {}",
        d.message
    );
    assert_eq!(d.message, expected_message, "src {src:?}");
    d
}

// ---- FIND clause / mode rejects --------------------------------------------

/// An unknown FIND clause key fails closed naming the key.
#[test]
fn find_unsupported_clause_rejects() {
    assert_unsupported(
        "find reachability a.b -> c.d | bogus=x",
        "unsupported FIND clause 'bogus'",
    );
}

// ---- CHECK rejects ---------------------------------------------------------

/// An unsupported CHECK confirm/emit/mode keyword each reject at the keyword span.
#[test]
fn check_unsupported_mode_keywords_reject() {
    assert_unsupported(
        "check r.x | confirm=maybe",
        "unsupported confirm mode 'maybe'",
    );
    assert_unsupported(
        "check r.x | confirm=skip | emit=xml",
        "unsupported emit target 'xml'",
    );
    assert_unsupported(
        "check r.x | confirm=skip | mode=guess",
        "unsupported detection mode 'guess'",
    );
}

/// An unknown CHECK clause key rejects.
#[test]
fn check_unsupported_clause_rejects() {
    assert_unsupported(
        "check r.x | confirm=skip | bogus=y",
        "unsupported CHECK clause 'bogus'",
    );
}

// ---- EXPLAIN rejects -------------------------------------------------------

#[test]
fn explain_unsupported_modes_and_clause_reject() {
    assert_unsupported(
        "explain r.x | evidence=guess",
        "unsupported evidence mode 'guess'",
    );
    assert_unsupported(
        "explain r.x | truth=maybe",
        "unsupported truth mode 'maybe'",
    );
    assert_unsupported(
        "explain r.x | bogus=y",
        "unsupported EXPLAIN clause 'bogus'",
    );
}

// ---- SHOW / DESCRIBE rejects -----------------------------------------------

#[test]
fn show_unsupported_category_rejects() {
    assert_unsupported("show widgets", "unsupported SHOW category 'widgets'");
}

/// DESCRIBE without the `::` separator fails closed with a token error.
#[test]
fn describe_missing_double_colon_rejects() {
    let d = err("describe rule.path");
    assert_eq!(d.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(d.message, "expected ':' in a 'kind::path' target");
    assert_eq!(d.span, SpanV1::new(13, 14));
}

// ---- SAMPLE rejects --------------------------------------------------------

#[test]
fn sample_unsupported_mode_and_clause_reject() {
    assert_unsupported(
        "sample query=\"q\" | mode=fuzzy | limit=1",
        "unsupported sample mode 'fuzzy'",
    );
    assert_unsupported(
        "sample query=\"q\" | limit=1 | bogus=z",
        "unsupported SAMPLE clause 'bogus'",
    );
}

/// SAMPLE that does not start with `query` rejects.
#[test]
fn sample_wrong_first_key_rejects() {
    assert_unsupported(
        "sample mode=lexical | limit=1",
        "SAMPLE must start with a 'query' clause",
    );
}

/// A non-integer limit fails closed in `parse_u32`'s non-Int arm.
#[test]
fn sample_non_integer_limit_rejects() {
    let d = err("sample query=\"q\" | limit=many");
    assert_eq!(d.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(d.message, "expected a positive limit");
    assert_eq!(d.span, SpanV1::new(25, 29));
}

// ---- COMPARE / EXPORT rejects ----------------------------------------------

#[test]
fn compare_rejects() {
    assert_unsupported(
        "compare candidate=b",
        "COMPARE must start with a 'baseline' clause",
    );
    assert_unsupported(
        "compare baseline=a | metric=fuzzy | candidate=b",
        "unsupported metric 'fuzzy'",
    );
    assert_unsupported(
        "compare baseline=a | bogus=y",
        "unsupported COMPARE clause 'bogus'",
    );
}

#[test]
fn export_rejects() {
    assert_unsupported(
        "export kind=json",
        "EXPORT must start with a 'source' clause",
    );
    assert_unsupported(
        "export source=r.x | kind=xml",
        "unsupported export kind 'xml'",
    );
    assert_unsupported(
        "export source=r.x | bogus=y",
        "unsupported EXPORT clause 'bogus'",
    );
    assert_unsupported("export source=r.x", "EXPORT requires a 'kind' clause");
}

// ---- VIEW / MODEL / PACK / PROFILE / GATE rejects --------------------------

#[test]
fn view_unsupported_body_rejects() {
    assert_unsupported(
        "view v | body=x",
        "VIEW body must be 'query-ref' or 'select', got 'body'",
    );
}

#[test]
fn model_unsupported_clause_and_missing_reject() {
    assert_unsupported(
        "model m | provider=openai | source=s | bogus=y",
        "unsupported MODEL clause 'bogus'",
    );
    assert_unsupported("model m | provider=openai", "MODEL requires a 'source'");
    assert_unsupported("model m | source=s", "MODEL requires a 'provider'");
}

#[test]
fn pack_unsupported_clause_and_missing_reject() {
    assert_unsupported(
        "pack p | source=s | activation=manual | bogus=y",
        "unsupported PACK clause 'bogus'",
    );
    assert_unsupported("pack p | source=s", "PACK requires an 'activation'");
    assert_unsupported("pack p | activation=manual", "PACK requires a 'source'");
}

#[test]
fn profile_admission_and_clause_reject() {
    assert_unsupported(
        "profile p | capabilities=c.a | admission=open",
        "unsupported admission policy 'open'",
    );
    assert_unsupported(
        "profile p | capabilities=c.a | bogus=y",
        "unsupported PROFILE clause 'bogus'",
    );
    assert_unsupported(
        "profile p | admission=allowlist",
        "PROFILE requires a 'capabilities' clause",
    );
}

#[test]
fn gate_binding_and_clause_reject() {
    assert_unsupported(
        "gate g | checks=c.a | binding=most",
        "unsupported binding policy 'most'",
    );
    assert_unsupported(
        "gate g | checks=c.a | bogus=y",
        "unsupported GATE clause 'bogus'",
    );
    assert_unsupported("gate g | binding=all", "GATE requires a 'checks' clause");
}

#[test]
fn ruledef_wrong_key_rejects() {
    assert_unsupported(
        "ruledef r | other-key=c",
        "RULE binding requires a 'check-ref' clause, got 'other-key'",
    );
}

// ---- USE PACK rejects ------------------------------------------------------

#[test]
fn use_pack_rejects() {
    // manual activation is illegal for USE PACK.
    assert_unsupported(
        "use pack p | activation=manual | lockfile=l.lock",
        "USE PACK activation must be 'session' or 'workspace'",
    );
    // lockfile must end in .lock.
    assert_unsupported(
        "use pack p | activation=session | lockfile=notalock",
        "USE PACK lockfile must end with '.lock'",
    );
    // unsupported clause.
    assert_unsupported(
        "use pack p | activation=session | bogus=y",
        "unsupported USE PACK clause 'bogus'",
    );
    // unsupported activation keyword (activation_value's reject arm).
    assert_unsupported(
        "use pack p | activation=always | lockfile=l.lock",
        "unsupported activation 'always'",
    );
}

// ---- reference parsing rejects ---------------------------------------------

/// A reference that does not start with an identifier fails closed at the slot.
#[test]
fn reference_missing_ident_rejects() {
    // `check` with no rule reference: `parse_ref` hits the `|` instead of an ident.
    let d = err("check | confirm=skip");
    assert_eq!(d.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(d.message, "expected a rule reference");
    assert_eq!(d.span, SpanV1::new(6, 7));
}

/// A hyphenated reference with no segment after `-` fails closed asking for the
/// path segment.
#[test]
fn reference_dangling_hyphen_rejects() {
    let d = err("pack secure- | source=s | activation=session");
    assert_eq!(d.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(d.message, "expected a path segment after separator");
    assert_eq!(d.span, SpanV1::new(13, 14));
}

/// An integer literal too large for `u32` fails closed in `parse_u32`'s
/// overflow arm (a typed `UnsupportedConstruct`, not a panic).
#[test]
fn find_depth_overflow_rejects() {
    assert_unsupported(
        "find reachability a.b -> c.d | depth=99999999999",
        "'99999999999' is not a valid a depth bound",
    );
}
