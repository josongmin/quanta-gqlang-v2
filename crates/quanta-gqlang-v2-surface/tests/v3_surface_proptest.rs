//! Structural property rails for the v3 surface public string-entry API.
//!
//! These are PROPERTY tests (not examples): proptest generates arbitrary and
//! adversarial inputs and asserts the three SOTA++ structural guarantees that
//! the crate-level contract promises:
//!
//! 1. TOTALITY / panic-freedom — every public `&str`-entry API
//!    ([`lex`], [`parse_program`], [`parse_document`], [`parse_expr`],
//!    [`parse_predicate`]) returns `Ok(..)` or a typed [`ParseDiagnosticV1`] for
//!    ARBITRARY input. It NEVER unwinds. The ~45 `expect`/indexing sites in the
//!    surface lib are required to be panic-free on adversarial input; that is
//!    exactly what these rails prove.
//! 2. DETERMINISM — the same input parsed twice yields an identical result
//!    (same `Ok` structure, or the same typed diagnostic).
//! 3. ROUND-TRIP + IDEMPOTENCE — for any input that parses `Ok`, formatting the
//!    lowered item then re-parsing must succeed and be FORMAT-STABLE
//!    (`format(reparse) == format(item)`), and formatting is a fixed point
//!    (`format(format(x)) == format(x)`). The same holds for `;`-joined
//!    documents via [`format_document`].
//!
//! Structural equality is asserted on the CANONICAL TEXT (not the HIR value),
//! because HIR nodes carry per-parse node ids and spans that legitimately differ
//! between two parses; the canonical text is the stable structural witness the
//! formatter contract guarantees.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration test rail: assertion helpers use expect intentionally"
)]

use proptest::prelude::*;
use quanta_gqlang_v2_hir::HirBuilderV1;
use quanta_gqlang_v2_surface::{
    format_document, format_item, lex, parse_document, parse_expr, parse_predicate, parse_program,
    ParseDiagnosticV1, ParsedDocumentV1, ParsedProgramV1, SurfaceItemV1,
};

/// Bounded config so the adversarial rails stay fast in CI.
fn cfg() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        ..ProptestConfig::default()
    }
}

// ---- generators -------------------------------------------------------------

/// Fully unconstrained Strings: ascii, unicode, control characters, the empty
/// string, and arbitrarily long runs. This is the broadest totality stressor.
fn arb_any_string() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..256).prop_map(|cs| cs.into_iter().collect())
}

/// A small alphabet of "structured-ish" fragments biased toward the tokens that
/// most exercise the parser's recursive-descent recovery: brackets, quotes,
/// separators, arrows, operators, and keyword fragments. Joined into nasty
/// strings with unbalanced nesting, `;`-storms, and keyword soup.
fn arb_fragment() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "(", ")", "{", "}", "[", "]", "(", ")", "(", ")", ";", ";;", "|", "=>", "->", ",", ".",
        ":", "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "<>", "!=", "\"", "\"x\"", "\"unterm",
        "let", "rule", "from", "where", "project", "join", "on", "group", "order", "by", "with",
        "find", "check", "explain", "show", "view", "model", "pack", "profile", "gate", "use",
        "and", "or", "not", "if", "as", "in", "true", "false", "null", "reachability", "flow",
        "SELECT", "FROM", "WHERE", "x", "y", "a1", "_z", "42", "3.14", " ", "\n", "\t", "#c",
    ])
    .prop_map(str::to_owned)
}

/// Concatenate many fragments (no separators) to manufacture deeply unbalanced,
/// keyword-dense, separator-storm inputs.
fn arb_structured_string() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_fragment(), 0..160).prop_map(|fs| fs.join(""))
}

/// Deeply nested balanced parens/brackets/braces around a tiny core: stresses
/// recursion depth without tripping a stack overflow on reasonable bounds.
fn arb_deep_nesting() -> impl Strategy<Value = String> {
    (1usize..600, prop::sample::select(vec!['(', '[', '{']), prop::sample::select(vec!['x', '1'])).prop_map(
        |(n, open, core)| {
            let close = match open {
                '(' => ')',
                '[' => ']',
                _ => '}',
            };
            let mut s = String::new();
            for _ in 0..n {
                s.push(open);
            }
            s.push(core);
            for _ in 0..n {
                s.push(close);
            }
            s
        },
    )
}

/// Syntactically plausible v3 items, so the round-trip path is actually
/// exercised (a generator that only emits garbage would make round-trip
/// vacuously true). Mixed with the adversarial strategies above.
fn arb_plausible_item() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "from users".to_string(),
        "from users | where a > 1".to_string(),
        "from users | project name, age as a".to_string(),
        "from a | join b on a = b".to_string(),
        "from a | group by k".to_string(),
        "from a | order by k desc".to_string(),
        "from u => { name: name, n: count }".to_string(),
        "let x = 1 + 2 * 3".to_string(),
        "let s = \"hi\"".to_string(),
        "let r = f(a, b, c)".to_string(),
        "let l = [1, 2, 3]".to_string(),
        "let rec = { a: 1, b: 2 }".to_string(),
        "let p = a.b.c".to_string(),
        "rule r : x if a > 1 and b < 2".to_string(),
        "rule q : y if not (a or b)".to_string(),
        "find reachability src -> sink".to_string(),
        "find flow src -> sink | via v".to_string(),
        "check myrule | confirm=require | emit=findings | mode=hybrid".to_string(),
        "show rules".to_string(),
        "explain t | evidence=auto | truth=inspect".to_string(),
        "use pack mypack | activation=session".to_string(),
        "require certified".to_string(),
    ])
}

/// Inputs likely to be Ok, blended with all adversarial families. Each strategy
/// gets exercised, so the rails hold over both garbage and grammar.
fn arb_program_source() -> impl Strategy<Value = String> {
    prop_oneof![
        4 => arb_plausible_item(),
        3 => arb_structured_string(),
        2 => arb_any_string(),
        1 => arb_deep_nesting(),
    ]
}

/// Document sources: `;`-join several plausible/adversarial items.
fn arb_document_source() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_program_source(), 0..6).prop_map(|items| items.join(";\n"))
}

// ---- comparable views of results (id/span-free) -----------------------------

/// Project a parse result to a stable, comparable witness for determinism:
/// either the canonical formatted text (Ok) or the typed diagnostic (Err).
fn program_witness(r: &Result<ParsedProgramV1, ParseDiagnosticV1>) -> Result<String, ParseDiagnosticV1> {
    match r {
        Ok(p) => Ok(format_item(&p.item.to_program_item())),
        Err(e) => Err(e.clone()),
    }
}

fn document_witness(
    r: &Result<ParsedDocumentV1, ParseDiagnosticV1>,
) -> Result<String, ParseDiagnosticV1> {
    match r {
        Ok(d) => {
            let items: Vec<_> = d.items.iter().map(SurfaceItemV1::to_program_item).collect();
            Ok(format_document(&items))
        }
        Err(e) => Err(e.clone()),
    }
}

// ---- 1. TOTALITY / panic-freedom -------------------------------------------

proptest! {
    #![proptest_config(cfg())]

    /// `lex` never panics on arbitrary input.
    #[test]
    fn lex_never_panics_any(s in arb_any_string()) {
        let _ = lex(&s);
    }

    #[test]
    fn lex_never_panics_structured(s in arb_structured_string()) {
        let _ = lex(&s);
    }

    #[test]
    fn lex_never_panics_deep(s in arb_deep_nesting()) {
        let _ = lex(&s);
    }

    /// `parse_program` never panics — every input is Ok or a typed diagnostic.
    #[test]
    fn parse_program_never_panics(s in arb_program_source()) {
        let builder = HirBuilderV1::new();
        let _ = parse_program(&s, &builder);
    }

    #[test]
    fn parse_program_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _ = parse_program(&s, &builder);
    }

    #[test]
    fn parse_program_never_panics_deep(s in arb_deep_nesting()) {
        let builder = HirBuilderV1::new();
        let _ = parse_program(&s, &builder);
    }

    /// `parse_document` never panics.
    #[test]
    fn parse_document_never_panics(s in arb_document_source()) {
        let builder = HirBuilderV1::new();
        let _ = parse_document(&s, &builder);
    }

    #[test]
    fn parse_document_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _ = parse_document(&s, &builder);
    }

    /// The bare-expression entry points never panic either.
    #[test]
    fn parse_expr_never_panics(s in arb_program_source()) {
        let builder = HirBuilderV1::new();
        let _ = parse_expr(&s, &builder);
    }

    #[test]
    fn parse_predicate_never_panics(s in arb_program_source()) {
        let builder = HirBuilderV1::new();
        let _ = parse_predicate(&s, &builder);
    }

    #[test]
    fn parse_expr_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _ = parse_expr(&s, &builder);
    }

    #[test]
    fn parse_predicate_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _ = parse_predicate(&s, &builder);
    }
}

// ---- 2. DETERMINISM ---------------------------------------------------------

proptest! {
    #![proptest_config(cfg())]

    /// `parse_program(s)` twice yields an identical witness for any `s`.
    #[test]
    fn parse_program_is_deterministic(s in arb_program_source()) {
        let a = parse_program(&s, &HirBuilderV1::new());
        let b = parse_program(&s, &HirBuilderV1::new());
        prop_assert_eq!(program_witness(&a), program_witness(&b));
    }

    #[test]
    fn parse_program_is_deterministic_any(s in arb_any_string()) {
        let a = parse_program(&s, &HirBuilderV1::new());
        let b = parse_program(&s, &HirBuilderV1::new());
        prop_assert_eq!(program_witness(&a), program_witness(&b));
    }

    #[test]
    fn parse_document_is_deterministic(s in arb_document_source()) {
        let a = parse_document(&s, &HirBuilderV1::new());
        let b = parse_document(&s, &HirBuilderV1::new());
        prop_assert_eq!(document_witness(&a), document_witness(&b));
    }

    /// `lex` is deterministic for arbitrary input.
    #[test]
    fn lex_is_deterministic(s in arb_any_string()) {
        prop_assert_eq!(lex(&s), lex(&s));
    }
}

// ---- 3. ROUND-TRIP + IDEMPOTENCE -------------------------------------------

proptest! {
    #![proptest_config(cfg())]

    /// For any `s` that parses Ok: format the item, re-parse the canonical text
    /// (must be Ok), and the re-formatted text must EQUAL the first canonical
    /// text. Also format is idempotent: format(format(x)) == format(x).
    #[test]
    fn parse_program_round_trips_and_is_idempotent(s in arb_program_source()) {
        let Ok(parsed) = parse_program(&s, &HirBuilderV1::new()) else {
            return Ok(());
        };
        let canon = format_item(&parsed.item.to_program_item());

        // Idempotence at the text level: re-parse canonical text, re-format.
        let reparsed = parse_program(&canon, &HirBuilderV1::new());
        prop_assert!(
            reparsed.is_ok(),
            "canonical text must re-parse Ok; canon={canon:?} err={:?}",
            reparsed.err()
        );
        let reparsed = reparsed.expect("checked Ok above");
        let canon2 = format_item(&reparsed.item.to_program_item());
        prop_assert_eq!(&canon, &canon2, "format must be a fixed point (round-trip stable)");

        // format(format(x)) == format(x) — a third pass changes nothing.
        let third = parse_program(&canon2, &HirBuilderV1::new())
            .expect("canonical text re-parses");
        let canon3 = format_item(&third.item.to_program_item());
        prop_assert_eq!(canon2, canon3, "format idempotence");
    }

    /// Document-level round-trip: `;`-joined items survive format -> re-parse
    /// -> format unchanged.
    #[test]
    fn parse_document_round_trips_and_is_idempotent(s in arb_document_source()) {
        let Ok(parsed) = parse_document(&s, &HirBuilderV1::new()) else {
            return Ok(());
        };
        let items: Vec<_> = parsed.items.iter().map(SurfaceItemV1::to_program_item).collect();
        let canon = format_document(&items);

        let reparsed = parse_document(&canon, &HirBuilderV1::new());
        prop_assert!(
            reparsed.is_ok(),
            "canonical document must re-parse Ok; canon={canon:?} err={:?}",
            reparsed.err()
        );
        let reparsed = reparsed.expect("checked Ok above");
        let items2: Vec<_> =
            reparsed.items.iter().map(SurfaceItemV1::to_program_item).collect();
        let canon2 = format_document(&items2);
        prop_assert_eq!(canon, canon2, "document format must be a fixed point");
    }
}
