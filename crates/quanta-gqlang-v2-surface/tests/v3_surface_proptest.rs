#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use proptest::prelude::*;
mod support;
use quanta_gqlang_v2_hir::{
    check_document_v2, check_tree, HirBuilderV1, HirDocumentV1, HirRootRefV1, SourceIdV2,
};
use quanta_gqlang_v2_surface::{
    format_document, format_item, lex, parse_document, parse_expr, parse_predicate, parse_program,
    try_format_item, ParseDiagnosticV1, ParsedDocumentV1, ParsedProgramV1, SurfaceItemV1,
    TokenKindV1, TokenV1,
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
        "(",
        ")",
        "{",
        "}",
        "[",
        "]",
        "(",
        ")",
        "(",
        ")",
        ";",
        ";;",
        "|",
        "=>",
        "->",
        ",",
        ".",
        ":",
        "+",
        "-",
        "*",
        "/",
        "=",
        "<",
        ">",
        "<=",
        ">=",
        "<>",
        "!=",
        "\"",
        "\"x\"",
        "\"unterm",
        "let",
        "rule",
        "from",
        "where",
        "project",
        "join",
        "on",
        "group",
        "order",
        "by",
        "with",
        "find",
        "check",
        "explain",
        "show",
        "view",
        "model",
        "pack",
        "profile",
        "gate",
        "use",
        "and",
        "or",
        "not",
        "if",
        "as",
        "in",
        "true",
        "false",
        "null",
        "reachability",
        "flow",
        "SELECT",
        "FROM",
        "WHERE",
        "x",
        "y",
        "a1",
        "_z",
        "42",
        "3.14",
        " ",
        "\n",
        "\t",
        "#c",
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
    (
        1usize..3000,
        prop::sample::select(vec!['(', '[', '{']),
        prop::sample::select(vec!['x', '1']),
    )
        .prop_map(|(n, open, core)| {
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
        })
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
fn program_witness(
    r: &Result<ParsedProgramV1, ParseDiagnosticV1>,
) -> Result<String, ParseDiagnosticV1> {
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

fn diagnostic_invariants(
    source: &str,
    diagnostic: &ParseDiagnosticV1,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let span = diagnostic.span;
    prop_assert!(
        span.start <= span.end && span.end <= source.len(),
        "diagnostic span {:?} exceeds source byte length {}",
        span,
        source.len()
    );
    prop_assert!(source.is_char_boundary(span.start));
    prop_assert!(source.is_char_boundary(span.end));
    prop_assert_eq!(
        diagnostic.resource.is_some(),
        diagnostic.code == ParseErrorCodeV1::ResourceExhausted,
        "resource code and payload disagree"
    );
    // Total, fail-closed option handling: a `ResourceExhausted` diagnostic must
    // carry a payload whose observed count exceeds the limit, and no other
    // diagnostic may masquerade as one. The `None` arm is explicit so the
    // payload check cannot be skipped by a silently absent resource.
    match diagnostic.resource {
        Some(resource) => prop_assert!(
            resource.observed > resource.limit,
            "resource payload must exceed its limit: {resource:?}"
        ),
        None => prop_assert_ne!(
            diagnostic.code,
            ParseErrorCodeV1::ResourceExhausted,
            "resource code and payload disagree"
        ),
    }
    Ok(())
}

/// Accepted token streams carry the same source-bounded, ordered invariants the
/// diagnostics do, so a successful `lex` is verified rather than merely observed.
fn lexed_token_invariants(
    source: &str,
    tokens: &[TokenV1],
) -> Result<(), proptest::test_runner::TestCaseError> {
    let mut previous_end = 0usize;
    for token in tokens {
        prop_assert!(
            token.span.start <= token.span.end && token.span.end <= source.len(),
            "lexed token span {:?} exceeds source byte length {}",
            token.span,
            source.len()
        );
        prop_assert!(source.is_char_boundary(token.span.start));
        prop_assert!(source.is_char_boundary(token.span.end));
        prop_assert!(
            token.span.start >= previous_end,
            "lexed token spans must stay ordered and non-overlapping: {:?}",
            token.span
        );
        previous_end = token.span.end;
    }
    prop_assert!(
        matches!(
            tokens.last().map(|token| &token.kind),
            Some(TokenKindV1::Eof)
        ),
        "a successful lex must terminate in Eof"
    );
    Ok(())
}

#[test]
fn structural_witness_detects_a_dropped_query_stage() {
    let original = parse_program("from users | where active = true", &HirBuilderV1::new())
        .expect("filtered query parses");
    let dropped =
        parse_program("from users", &HirBuilderV1::new()).expect("unfiltered query parses");
    assert_ne!(
        support::structural_witness(&original.item.to_program_item()),
        support::structural_witness(&dropped.item.to_program_item()),
        "the fuzz round-trip oracle must notice a lost relation stage"
    );
}

// ---- 0. regression: deep nesting fails CLOSED (no stack-overflow abort) -----

use quanta_gqlang_v2_surface::ParseErrorCodeV1;

/// Pinned regression for the totality bug proptest surfaced: an adversarially
/// deep paren/bracket/brace nest used to overflow the native stack and ABORT
/// the process (a fail-open totality violation). It must now fail CLOSED with a
/// typed diagnostic from every recursive entry point.
#[test]
fn deeply_nested_input_fails_closed_not_abort() {
    // `(` and `[` are value/list openers that recurse per level, so the depth
    // budget is the gate: the typed reject must be ResourceExhausted. (`{` is a record opener that needs `field:` after the
    // brace, so it rejects shallowly with UnexpectedToken before recursing — also
    // fail-closed, asserted separately below.)
    for (open, close) in [('(', ')'), ('[', ']')] {
        // 5000 levels deep: far past the recursion budget, and the size that
        // previously aborted the process via stack overflow.
        let depth = 5000;
        let mut s = String::with_capacity(depth * 2 + 1);
        s.extend(std::iter::repeat(open).take(depth));
        s.push('x');
        s.extend(std::iter::repeat(close).take(depth));

        let prog = parse_program(&s, &HirBuilderV1::new());
        assert!(prog.is_err(), "deep {open}..{close} program must reject");

        let expr_err = parse_expr(&s, &HirBuilderV1::new()).expect_err("deep expr must reject");
        assert_eq!(
            expr_err.code,
            ParseErrorCodeV1::ResourceExhausted,
            "deep {open}..{close} nest exhausts the depth budget"
        );

        let pred = parse_predicate(&s, &HirBuilderV1::new());
        assert!(pred.is_err(), "deep {open}..{close} predicate must reject");
    }

    // Deeply nested braces must also fail closed (no abort), via the record path.
    let depth = 5000;
    let s = "{".repeat(depth) + "x" + &"}".repeat(depth);
    assert!(
        parse_expr(&s, &HirBuilderV1::new()).is_err(),
        "deep brace nest must reject, not abort"
    );
}

/// `not not not ...` (boolean prefix self-recursion) also fails closed.
#[test]
fn deeply_nested_not_fails_closed() {
    let s = "not ".repeat(5000) + "x";
    let error = parse_predicate(&s, &HirBuilderV1::new()).expect_err("deep `not` chain rejects");
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
}

/// `- - - ...` (unary value prefix self-recursion) also fails closed.
#[test]
fn deeply_nested_unary_fails_closed() {
    let s = "-".repeat(5000) + "1";
    let error = parse_expr(&s, &HirBuilderV1::new()).expect_err("deep unary chain rejects");
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
}

// ---- 1. TOTALITY / panic-freedom -------------------------------------------

proptest! {
    #![proptest_config(cfg())]

    /// `lex` never panics on arbitrary input.
    #[test]
    fn lex_never_panics_any(s in arb_any_string()) {
        let _result = lex(&s);
    }

    #[test]
    fn lex_never_panics_structured(s in arb_structured_string()) {
        let _result = lex(&s);
    }

    #[test]
    fn lex_never_panics_deep(s in arb_deep_nesting()) {
        let _result = lex(&s);
    }

    /// `parse_program` never panics — every input is Ok or a typed diagnostic.
    #[test]
    fn parse_program_never_panics(s in arb_program_source()) {
        let builder = HirBuilderV1::new();
        let _result = parse_program(&s, &builder);
    }

    #[test]
    fn parse_program_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _result = parse_program(&s, &builder);
    }

    #[test]
    fn parse_program_never_panics_deep(s in arb_deep_nesting()) {
        let builder = HirBuilderV1::new();
        let _result = parse_program(&s, &builder);
    }

    /// `parse_document` never panics.
    #[test]
    fn parse_document_never_panics(s in arb_document_source()) {
        let builder = HirBuilderV1::new();
        let _result = parse_document(&s, &builder);
    }

    #[test]
    fn parse_document_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _result = parse_document(&s, &builder);
    }

    /// The bare-expression entry points never panic either.
    #[test]
    fn parse_expr_never_panics(s in arb_program_source()) {
        let builder = HirBuilderV1::new();
        let _result = parse_expr(&s, &builder);
    }

    #[test]
    fn parse_predicate_never_panics(s in arb_program_source()) {
        let builder = HirBuilderV1::new();
        let _result = parse_predicate(&s, &builder);
    }

    #[test]
    fn parse_expr_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _result = parse_expr(&s, &builder);
    }

    #[test]
    fn parse_predicate_never_panics_any(s in arb_any_string()) {
        let builder = HirBuilderV1::new();
        let _result = parse_predicate(&s, &builder);
    }
}

// ---- 4. STRUCTURED RESULT INVARIANTS --------------------------------------

proptest! {
    #![proptest_config(cfg())]

    #[test]
    fn arbitrary_program_has_bounded_diagnostic_or_integrity_checked_hir(s in arb_program_source()) {
        match parse_program(&s, &HirBuilderV1::new()) {
            Ok(parsed) => {
                prop_assert_eq!(parsed.source_len, s.len());
                let item = parsed.item.to_program_item();
                let checked = check_tree(HirRootRefV1::Item(&item), s.len());
                prop_assert!(checked.is_ok(), "accepted source has invalid HIR: {s:?}: {checked:?}");
            }
            Err(diagnostic) => diagnostic_invariants(&s, &diagnostic)?,
        }
    }

    #[test]
    fn arbitrary_document_has_bounded_diagnostic_or_integrity_checked_hir(s in arb_document_source()) {
        match parse_document(&s, &HirBuilderV1::new()) {
            Ok(parsed) => {
                // The caller-visible source length is part of the accepted result,
                // not just an input we re-derive.
                prop_assert_eq!(parsed.source_len, s.len());
                let document = HirDocumentV1 {
                    source_id: SourceIdV2::from_raw(0),
                    source_len: parsed.source_len,
                    items: parsed.items.iter().map(SurfaceItemV1::to_program_item).collect(),
                };
                let checked = check_document_v2(&document);
                prop_assert!(checked.is_ok(), "accepted document has invalid HIR: {s:?}: {checked:?}");
            }
            Err(diagnostic) => diagnostic_invariants(&s, &diagnostic)?,
        }
    }

    #[test]
    fn arbitrary_expression_and_predicate_diagnostics_are_source_bounded(s in arb_program_source()) {
        // Fixed positive controls: if any entry point regressed to rejecting
        // unconditionally, the Ok branches below would go permanently unexercised.
        prop_assert!(parse_expr("a + 1", &HirBuilderV1::new()).is_ok(), "parse_expr must accept");
        prop_assert!(
            parse_predicate("a > 1", &HirBuilderV1::new()).is_ok(),
            "parse_predicate must accept"
        );
        prop_assert!(lex("let x = 1").is_ok(), "lex must accept");
        // Every branch asserts: an accepted result must carry integrity-checked
        // HIR, and a rejected one must carry a source-bounded diagnostic. No
        // entry point can pass this case by producing `Ok` silently.
        match parse_expr(&s, &HirBuilderV1::new()) {
            Ok(value) => prop_assert!(
                check_tree(HirRootRefV1::Value(&value), s.len()).is_ok(),
                "accepted expression has invalid HIR: {s:?}"
            ),
            Err(diagnostic) => diagnostic_invariants(&s, &diagnostic)?,
        }
        match parse_predicate(&s, &HirBuilderV1::new()) {
            Ok(predicate) => prop_assert!(
                check_tree(HirRootRefV1::Bool(&predicate), s.len()).is_ok(),
                "accepted predicate has invalid HIR: {s:?}"
            ),
            Err(diagnostic) => diagnostic_invariants(&s, &diagnostic)?,
        }
        match lex(&s) {
            Ok(tokens) => lexed_token_invariants(&s, &tokens)?,
            Err(diagnostic) => diagnostic_invariants(&s, &diagnostic)?,
        }
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
        // Full structural equality, not only the id/span-free witness: a reused
        // builder that leaks node ids between parses must fail here.
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(program_witness(&a), program_witness(&b));
    }

    #[test]
    fn parse_program_is_deterministic_any(s in arb_any_string()) {
        let a = parse_program(&s, &HirBuilderV1::new());
        let b = parse_program(&s, &HirBuilderV1::new());
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(program_witness(&a), program_witness(&b));
    }

    #[test]
    fn parse_document_is_deterministic(s in arb_document_source()) {
        let a = parse_document(&s, &HirBuilderV1::new());
        let b = parse_document(&s, &HirBuilderV1::new());
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(document_witness(&a), document_witness(&b));
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
        // Rejected inputs still assert the diagnostic bound, so a case is never a
        // silent no-op; accepted inputs assert the full round-trip.
        let parsed = match parse_program(&s, &HirBuilderV1::new()) {
            Ok(parsed) => parsed,
            Err(diagnostic) => {
                diagnostic_invariants(&s, &diagnostic)?;
                return Ok(());
            }
        };
        let item = parsed.item.to_program_item();
        let canon = format_item(&item);
        prop_assert_eq!(
            try_format_item(&item),
            Ok(canon.clone()),
            "every parser-produced HIR item must pass the checked formatter"
        );

        // Idempotence at the text level: re-parse canonical text, re-format.
        let reparsed = parse_program(&canon, &HirBuilderV1::new());
        prop_assert!(
            reparsed.is_ok(),
            "canonical text must re-parse Ok; canon={canon:?} err={:?}",
            reparsed.err()
        );
        let reparsed = reparsed.expect("checked Ok above");
        prop_assert_eq!(
            support::structural_witness(&parsed.item.to_program_item()),
            support::structural_witness(&reparsed.item.to_program_item()),
            "visitor-derived structure changed"
        );
        let canon2 = format_item(&reparsed.item.to_program_item());
        prop_assert_eq!(&canon, &canon2, "format must be a fixed point (round-trip stable)");
    }

    /// Document-level round-trip: `;`-joined items survive format -> re-parse
    /// -> format unchanged.
    #[test]
    fn parse_document_round_trips_and_is_idempotent(s in arb_document_source()) {
        let parsed = match parse_document(&s, &HirBuilderV1::new()) {
            Ok(parsed) => parsed,
            Err(diagnostic) => {
                diagnostic_invariants(&s, &diagnostic)?;
                return Ok(());
            }
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
        let witnesses = items.iter().map(support::structural_witness).collect::<Vec<_>>();
        let witnesses2 = items2.iter().map(support::structural_witness).collect::<Vec<_>>();
        prop_assert_eq!(witnesses, witnesses2, "document structure changed");
        let canon2 = format_document(&items2);
        prop_assert_eq!(canon, canon2, "document format must be a fixed point");
    }
}
