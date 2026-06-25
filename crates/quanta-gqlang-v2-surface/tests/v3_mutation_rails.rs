//! Mutation-hardening rails: behavior-pinning tests that kill cargo-mutants
//! survivors in the lexer, expression-paren disambiguator, diagnostic Display,
//! and the canonical formatter. Each test asserts the EXACT observable behavior
//! (token kinds/spans, parse acceptance + kind, rendered string) so the
//! corresponding mutation flips a failing assertion.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "mutation rail: assertion panics are intentional"
)]

use quanta_gqlang_v2_hir::{BoolExprKindV1, HirBuilderV1, ProgramItemHirV1, SpanV1};
use quanta_gqlang_v2_surface::{
    format_item, lex, parse_predicate, parse_program, ParseErrorCodeV1, SurfaceItemV1, TokenKindV1,
};

// ---- lexer rails -----------------------------------------------------------

fn kinds(src: &str) -> Vec<TokenKindV1> {
    lex(src)
        .unwrap_or_else(|e| panic!("lex {src:?}: {e}"))
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

/// Whitespace bytes are each individually skipped. Kills lexer.rs:91 `||`->`&&`:
/// a lone space between idents must be consumed, not flagged as an unknown byte.
#[test]
fn space_separates_two_idents() {
    assert_eq!(
        kinds("a b"),
        vec![
            TokenKindV1::Ident("a".into()),
            TokenKindV1::Ident("b".into()),
            TokenKindV1::Eof,
        ]
    );
    // a tab also separates (and the lex must succeed, not error).
    assert_eq!(
        kinds("a\tb"),
        vec![
            TokenKindV1::Ident("a".into()),
            TokenKindV1::Ident("b".into()),
            TokenKindV1::Eof,
        ]
    );
}

/// A `#` line comment is skipped up to the newline, and a comment that runs to
/// EOF terminates cleanly. Kills lexer.rs:97 (`<`/`==`/`>`/`<=`, `&&`/`||`,
/// `!=`/`==`) and lexer.rs:98 (`+=`/`-=`/`*=`): every one of those mutations
/// either fails to consume the comment (wrong tokens) or loops forever (the
/// test would never return) on these inputs.
#[test]
fn line_comment_is_skipped() {
    // comment then a real token on the next line.
    assert_eq!(
        kinds("#hello\na"),
        vec![TokenKindV1::Ident("a".into()), TokenKindV1::Eof]
    );
    // comment running to EOF with no trailing newline must terminate.
    assert_eq!(kinds("# trailing comment"), vec![TokenKindV1::Eof]);
    // a token, then a comment to EOF.
    assert_eq!(
        kinds("x # tail"),
        vec![TokenKindV1::Ident("x".into()), TokenKindV1::Eof]
    );
}

/// A closing-quote string span ends exactly one byte past the closing quote.
/// Kills lexer.rs:151 `+`->`-`/`*` in `SpanV1::new(start, i + 1)`.
#[test]
fn string_span_covers_both_quotes() {
    let toks = lex("\"ab\"").unwrap();
    let first = toks.first().expect("at least the string token");
    assert_eq!(first.kind, TokenKindV1::Str("ab".into()));
    // bytes: " a b " -> span 0..4 (inclusive of both quote bytes).
    assert_eq!(first.span.start, 0);
    assert_eq!(first.span.end, 4);
}

/// A `.` is part of a float ONLY when a digit follows it; otherwise it is a
/// `Dot` and the number is an integer. Kills lexer.rs:196 col 49 (`&&`->`||`)
/// and col 69 (`+`->`-`): both mutations would mis-classify `1.x`/`1.` as a
/// float, eating the dot.
#[test]
fn dot_without_following_digit_is_not_a_float() {
    assert_eq!(
        kinds("1.x"),
        vec![
            TokenKindV1::Int("1".into()),
            TokenKindV1::Dot,
            TokenKindV1::Ident("x".into()),
            TokenKindV1::Eof,
        ]
    );
    // trailing dot at EOF: integer + dot, never a float.
    assert_eq!(
        kinds("1."),
        vec![
            TokenKindV1::Int("1".into()),
            TokenKindV1::Dot,
            TokenKindV1::Eof,
        ]
    );
    // sanity: a real fraction IS a float.
    assert_eq!(
        kinds("1.5"),
        vec![TokenKindV1::Float("1.5".into()), TokenKindV1::Eof]
    );
}

/// The fractional-digit scan stops at the first non-digit. Kills lexer.rs:199
/// col 23 (`&&`->`||`): with `||` the loop would swallow a following identifier
/// byte into the float literal.
#[test]
fn float_scan_stops_at_first_nondigit() {
    assert_eq!(
        kinds("1.5a"),
        vec![
            TokenKindV1::Float("1.5".into()),
            TokenKindV1::Ident("a".into()),
            TokenKindV1::Eof,
        ]
    );
}

// ---- paren_opens_bool rails (via parse_predicate) --------------------------

fn predicate_kind(src: &str) -> BoolExprKindV1 {
    let builder = HirBuilderV1::new();
    parse_predicate(src, &builder)
        .unwrap_or_else(|e| panic!("parse_predicate {src:?}: {e}"))
        .kind
}

/// The depth counter must increment on a nested `(`. Kills expr.rs:168
/// (`depth += 1` -> `*=`/`-=`).
///
/// `((a))` is a valid doubly-parenthesized VALUE: a compare token never appears,
/// so the outer paren is a value paren and lowers to `BoolExprKind::Value`. The
/// `-=` mutation drives the depth counter to underflow on the inner `)` (a
/// panic), so requiring this to parse to a `Value` kills it.
#[test]
fn nested_value_paren_keeps_value_kind() {
    assert!(matches!(predicate_kind("((a))"), BoolExprKindV1::Value(_)));
}

/// `((a > b))` REJECTS: `paren_opens_bool` only inspects depth-1 tokens of the
/// OUTER paren, so a compare buried at depth 2 leaves the outer paren a value
/// paren, and value-parsing `(a > b)` then fails on the `>`. The `*=` mutation
/// on expr.rs:168 pins the depth at 1, which would (wrongly) see the `>` at
/// "depth 1" and accept the input as a boolean paren. Requiring this to be an
/// error kills `depth += 1` -> `*=`.
#[test]
fn doubly_nested_compare_is_rejected() {
    let builder = HirBuilderV1::new();
    let err = parse_predicate("((a > b))", &builder).expect_err("must reject");
    assert_eq!(err.code, ParseErrorCodeV1::UnexpectedToken);
}

/// A boolean token at depth 1 AFTER a closed nested paren makes it a BOOL paren.
/// Kills expr.rs:170 (`depth -= 1` -> `+=`) and expr.rs:171 (`== 0` -> `!= 0`):
/// either mutation fails to return depth to 1 / returns false prematurely at the
/// inner `)`, so the trailing `or` is missed and the paren is mis-read as a
/// value paren (which then fails to parse).
#[test]
fn bool_token_after_nested_paren_opens_bool_paren() {
    // `((a) or b)`: inner `(a)` closes back to depth 1, then `or` at depth 1
    // marks a boolean group -> Paren wrapping an Or.
    assert!(matches!(
        predicate_kind("((a) or b)"),
        BoolExprKindV1::Paren(_)
    ));
}

// ---- diagnostic Display rail -----------------------------------------------

/// The `Display` for a parse diagnostic renders the exact `code: msg (at a..b)`
/// form. Kills diag.rs:78 (`fmt` -> `Ok(Default::default())`), which would emit
/// the empty string.
#[test]
fn diagnostic_display_is_exact() {
    let builder = HirBuilderV1::new();
    // an unterminated string is a lex error with a known code and span.
    let err = parse_program("let x = \"oops", &builder).expect_err("must reject");
    let rendered = format!("{err}");
    assert!(
        rendered.starts_with("surface.parse.unterminated_string: "),
        "rendered: {rendered:?}"
    );
    assert!(rendered.contains("(at "), "rendered: {rendered:?}");
    assert!(!rendered.is_empty());
    // also pin a fully exact render via a constructed diagnostic through the
    // public error path so the empty-string mutation is unambiguously dead.
    let exact = parse_program("@", &builder).expect_err("unknown char rejects");
    assert_eq!(
        format!("{exact}"),
        "surface.parse.unexpected_char: unexpected character '@' (at 0..1)"
    );
}

// ---- formatter rails -------------------------------------------------------

fn fmt(src: &str) -> String {
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
    let item: ProgramItemHirV1 = match parsed.item {
        SurfaceItemV1::Program(p) => p,
        SurfaceItemV1::Query(q) => ProgramItemHirV1::from(q),
        SurfaceItemV1::Decl(d) => ProgramItemHirV1::from(d),
        SurfaceItemV1::Rule(r) => ProgramItemHirV1::from(r),
    };
    format_item(&item)
}

/// `format_relation` renders the source and each pipeline stage verbatim. Kills
/// format.rs:91 (`format_relation` -> `"xyzzy"`) and format.rs:202
/// (`format_projection_item` -> `"xyzzy"`).
#[test]
fn relation_and_projection_render_exactly() {
    assert_eq!(
        fmt("from t | project a, b as c"),
        "from t | project a, b as c"
    );
}

/// A `join ... on ...` renders through the join-on recognizer, which gates on
/// the compose symbol being exactly `join`. Kills format.rs:108 (`==`->`!=`):
/// flipping the gate would drop the `on` form and render a `... | where ...`
/// over a bare join compose instead. Also exercises format_source_only
/// (format.rs:147 -> `"xyzzy"`).
#[test]
fn join_on_renders_canonically() {
    assert_eq!(fmt("from a | join b on a = b"), "from a | join b on a = b");
    // plain join (no `on`) exercises format_source_only on the right operand.
    assert_eq!(fmt("from a | join b"), "from a | join b");
}

/// `group by` / `order by` render through `format_keyed_compose`. Kills
/// format.rs:166 (`format_keyed_compose` -> `"xyzzy"`).
#[test]
fn keyed_compose_renders_canonically() {
    assert_eq!(fmt("from t | group by a"), "from t | group by a");
    assert_eq!(fmt("from t | order by a desc"), "from t | order by a desc");
}

/// String literals are rendered with escapes. Kills format.rs:325 (`escape_str`
/// -> `"xyzzy"`): a string with an embedded quote/newline must render its
/// escaped form, not a fixed sentinel.
#[test]
fn string_escape_renders_exactly() {
    // a SAMPLE query text carries a string literal through escape_str.
    assert_eq!(
        fmt("sample query=\"a\\\"b\" | limit=5"),
        "sample query=\"a\\\"b\" | mode=lexical | limit=5"
    );
}

/// The last-resort `format_compose_fallback` renders `<symbol> <operands>` for
/// an unrecognized compose tag. The v3 grammar never mints such a tag, but
/// `format_item` is public and total, so we build one directly through the HIR
/// builder. Kills format.rs:192 (`format_compose_fallback` -> `"xyzzy"` and
/// -> `String::new()`).
#[test]
fn unknown_compose_tag_uses_fallback_render() {
    let builder = HirBuilderV1::new();
    let sp = SpanV1::new(0, 1);
    let left = builder.relation_source("a", None, sp);
    let right = builder.relation_source("b", None, sp);
    // a compose tag the query grammar never produces -> hits the fallback arm.
    let comp = builder.compose("merge", vec![left, right], sp);
    let query = builder.query(comp, None, sp);
    let item = ProgramItemHirV1::from(query);
    assert_eq!(format_item(&item), "from merge a, b");
}

/// An optional reference clause renders only when present, with the exact key.
/// Kills format.rs:414 (`push_ref_opt` -> `()`), which would drop the clause.
#[test]
fn optional_ref_clause_is_emitted() {
    // CHECK renders `| scope=...` via push_ref_opt when scope is present.
    assert_eq!(
        fmt("check r | scope=s | confirm=skip"),
        "check r | scope=s | confirm=skip | emit=findings | mode=hybrid"
    );
    // and omits it when absent.
    assert_eq!(
        fmt("check r | confirm=skip"),
        "check r | confirm=skip | emit=findings | mode=hybrid"
    );
}
