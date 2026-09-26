#![expect(
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

mod support;

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1, SpanV1};
use quanta_gqlang_v2_surface::{format_item, parse_program, ParseErrorCodeV1};

fn parse(src: &str) -> ProgramItemHirV1 {
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
    parsed.item.into_program_item()
}

/// parse -> format -> re-parse -> format must be a fixed point, and the rendered
/// text must re-parse to the identical structure.
fn assert_roundtrip(src: &str) {
    let item1 = parse(src);
    let text1 = format_item(&item1);
    let item2 = parse(&text1);
    let text2 = format_item(&item2);
    assert_eq!(
        support::structural_witness(&item1),
        support::structural_witness(&item2)
    );
    assert_eq!(text1, text2, "round-trip not stable for {src:?}");
}

/// The rendered text is also asserted to equal an EXACT expected canonical form,
/// so we pin what the formatter emits (not just that it is stable).
fn assert_renders(src: &str, expected: &str) {
    let item = parse(src);
    assert_eq!(format_item(&item), expected, "render of {src:?}");
    assert_roundtrip(src);
}

// ---- value-expr kinds ------------------------------------------------------

/// Unary, binary, call, paren, list, and path renders in a decl body.
#[test]
fn value_kinds_render_canonically() {
    assert_renders("let x = -a", "let x = -a");
    assert_renders("let x = a + b * c", "let x = a + b * c");
    assert_renders("let x = f(a, 1)", "let x = f(a, 1)");
    assert_renders("let x = (a + b)", "let x = (a + b)");
    assert_renders("let x = [a, b, c]", "let x = [a, b, c]");
    assert_renders("let x = u.profile.name", "let x = u.profile.name");
    // Unary under a tighter-binding binary must keep its operand binding.
    assert_renders("let x = -a * b", "let x = -a * b");
}

/// Every literal kind renders verbatim: int, float, string (escaped), bool, null.
#[test]
fn literal_kinds_render_canonically() {
    assert_renders("let x = 42", "let x = 42");
    assert_renders("let x = 3.14", "let x = 3.14");
    assert_renders("let x = true", "let x = true");
    assert_renders("let x = false", "let x = false");
    assert_renders("let x = null", "let x = null");
}

/// String escaping round-trips: quote, backslash, newline, tab.
#[test]
fn string_escapes_round_trip() {
    assert_renders("let x = \"a\\\"b\"", "let x = \"a\\\"b\"");
    assert_renders("let x = \"a\\\\b\"", "let x = \"a\\\\b\"");
    assert_renders("let x = \"a\\nb\"", "let x = \"a\\nb\"");
    assert_renders("let x = \"a\\tb\"", "let x = \"a\\tb\"");
    assert_renders("let x = \"a\\rb\\bc\\f\"", "let x = \"a\\rb\\bc\\f\"");
    // A `\u{..}` escape decodes and re-renders as the literal scalar, so this
    // pins the decoded value instead of only round-trip stability.
    assert_renders("let x = \"\\u{1f642}\"", "let x = \"🙂\"");
    assert_renders("let x = \"한글🙂é\"", "let x = \"한글🙂é\"");
}

#[test]
fn invalid_escape_forms_reject() {
    // Every branch pins the exact code, message, and span: `is_err()` alone would
    // accept any unrelated lexical rejection of these sources.
    for (source, message, start, end) in [
        (r#"let x = "\q""#, "unknown string escape", 9usize, 11usize),
        (r#"let x = "\u{}""#, "invalid Unicode escape", 9, 12),
        (r#"let x = "\u{0000041}""#, "invalid Unicode escape", 9, 19),
        (
            r#"let x = "\u{d800}""#,
            "Unicode escape is not a scalar value",
            9,
            17,
        ),
        (r#"let x = "\u110000""#, "expected '{' after \\u", 9, 11),
    ] {
        let builder = HirBuilderV1::new();
        let diag = match parse_program(source, &builder) {
            Err(diag) => diag,
            Ok(_) => panic!("accepted invalid escape {source:?}"),
        };
        assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedChar, "{source}");
        assert_eq!(diag.message, message, "{source}");
        assert_eq!(diag.span, SpanV1::new(start, end), "{source}");
    }
}

// ---- bool-expr connectives -------------------------------------------------

/// `or`, `not`, parenthesized bool, and quantifier-with-binders all render to
/// their exact canonical text. Each form is pinned absolutely: stability alone
/// cannot catch a dropped paren or a keyword aliased onto another variant.
#[test]
fn bool_connectives_render_canonically() {
    assert_renders("rule r : x if a or b", "rule r : x if a or b");
    assert_renders("rule r : x if not a", "rule r : x if not a");
    assert_renders(
        "rule r : x if (a or b) and c",
        "rule r : x if (a or b) and c",
    );
    assert_renders(
        "rule r : x if a and (b or c)",
        "rule r : x if a and (b or c)",
    );
    // a parenthesized value-comparison stays a value paren, not a bool paren.
    assert_renders("rule r : x if (a + b) > c", "rule r : x if (a + b) > c");
    assert_renders(
        "rule r : x if exists v in (v.flag)",
        "rule r : x if exists v in (v.flag)",
    );
    assert_renders(
        "rule r : x if forall v, w in (v.k = w.k)",
        "rule r : x if forall v, w in (v.k = w.k)",
    );
    // bare-value and comparison bool positions.
    assert_renders("rule r : x if a > b", "rule r : x if a > b");
    assert_renders("rule r : x if a", "rule r : x if a");
}

// ---- FIND subkinds + constraint clauses ------------------------------------

/// Every FIND subkind and constraint-bearing clause renders to its exact
/// canonical text (pins scope/callable presence, not just re-parse stability).
#[test]
fn find_all_subkinds_round_trip() {
    assert_renders(
        "find reachability a.b -> c.d",
        "find reachability a.b -> c.d",
    );
    assert_renders("find flow a.b -> c.d", "find flow a.b -> c.d");
    assert_renders("find taint a.b -> c.d", "find taint a.b -> c.d");
    assert_renders("find path a.b -> c.d", "find path a.b -> c.d");
    assert_renders("find slice | scope=mod", "find slice | scope=mod");
    assert_renders(
        "find points_to | reference=p.x | pointee=h.o",
        "find points_to | reference=p.x | pointee=h.o",
    );
    assert_renders(
        "find summary | callable=svc.h | facets=calls",
        "find summary | callable=svc.h | facets=calls",
    );
}

/// The constraint clauses the main hellgate skips: via/through, window,
/// frontier, direction, anchor, and a directioned walk.
#[test]
fn find_constraint_clauses_round_trip() {
    assert_renders(
        "find taint a.b -> c.d | via=x.y | through=z.w",
        "find taint a.b -> c.d | via=x.y | through=z.w",
    );
    assert_renders(
        "find reachability a.b -> c.d | window=win.scope | frontier=fr.edge",
        "find reachability a.b -> c.d | window=win.scope | frontier=fr.edge",
    );
    assert_renders(
        "find reachability a.b -> c.d | direction=backward",
        "find reachability a.b -> c.d | direction=backward",
    );
    assert_renders(
        "find reachability a.b -> c.d | direction=undirected",
        "find reachability a.b -> c.d | direction=undirected",
    );
    assert_renders(
        "find flow a.b -> c.d | anchor=an.node",
        "find flow a.b -> c.d | anchor=an.node",
    );
}

// ---- CHECK / presentation modes --------------------------------------------

/// Each CHECK enum value renders to its exact keyword. Exact renders stop a
/// keyword from being aliased onto an already-pinned variant.
#[test]
fn check_modes_render() {
    assert_renders(
        "check r.x | confirm=skip | emit=findings | mode=pattern",
        "check r.x | confirm=skip | emit=findings | mode=pattern",
    );
    assert_renders(
        "check r.x | confirm=prompt | emit=sarif | mode=semantic",
        "check r.x | confirm=prompt | emit=sarif | mode=semantic",
    );
    assert_renders(
        "check r.x | confirm=require | emit=findings | mode=hybrid",
        "check r.x | confirm=require | emit=findings | mode=hybrid",
    );
}

/// Each EXPLAIN evidence/truth value renders to its exact keyword.
#[test]
fn explain_modes_render() {
    assert_renders(
        "explain r.x | evidence=auto | truth=inspect",
        "explain r.x | evidence=auto | truth=inspect",
    );
    assert_renders(
        "explain r.x | evidence=citation | truth=refute",
        "explain r.x | evidence=citation | truth=refute",
    );
    assert_renders(
        "explain r.x | evidence=counterexample | truth=confirm",
        "explain r.x | evidence=counterexample | truth=confirm",
    );
}

/// Each SHOW category renders to its own keyword.
#[test]
fn show_categories_render() {
    for cat in [
        "capabilities",
        "models",
        "views",
        "rules",
        "packs",
        "profiles",
        "gates",
    ] {
        assert_renders(&format!("show {cat}"), &format!("show {cat}"));
    }
}

/// SAMPLE modes and COMPARE metrics render to their exact keywords.
#[test]
fn sample_and_compare_modes_render() {
    assert_renders(
        "sample query=\"q\" | mode=lexical | limit=3",
        "sample query=\"q\" | mode=lexical | limit=3",
    );
    assert_renders(
        "sample query=\"q\" | mode=hybrid | limit=3",
        "sample query=\"q\" | mode=hybrid | limit=3",
    );
    assert_renders(
        "compare baseline=a | candidate=b | metric=exact",
        "compare baseline=a | candidate=b | metric=exact",
    );
    assert_renders(
        "compare baseline=a | candidate=b | metric=structural",
        "compare baseline=a | candidate=b | metric=structural",
    );
}

/// Every EXPORT format keyword renders to its own keyword.
#[test]
fn export_formats_render() {
    for fmt in ["json", "csv", "sarif", "markdown", "graph-ml"] {
        assert_renders(
            &format!("export source=r.x | kind={fmt}"),
            &format!("export source=r.x | kind={fmt}"),
        );
    }
}

// ---- catalog enum-valued modes ---------------------------------------------

/// PACK / USE PACK activation policies and PROFILE admission / GATE binding all
/// render to their exact keyword forms.
#[test]
fn catalog_enum_modes_render() {
    assert_renders(
        "pack p | source=r.x | activation=manual",
        "pack p | source=r.x | activation=manual",
    );
    assert_renders(
        "pack p | source=r.x | activation=session",
        "pack p | source=r.x | activation=session",
    );
    assert_renders(
        "pack p | source=r.x | activation=workspace",
        "pack p | source=r.x | activation=workspace",
    );
    assert_renders(
        "profile p | capabilities=c.a | admission=allowlist",
        "profile p | capabilities=c.a | admission=allowlist",
    );
    assert_renders(
        "profile p | capabilities=c.a | admission=review-required",
        "profile p | capabilities=c.a | admission=review-required",
    );
    assert_renders(
        "profile p | capabilities=c.a | admission=certified-only",
        "profile p | capabilities=c.a | admission=certified-only",
    );
    assert_renders(
        "gate g | checks=c.a | binding=all",
        "gate g | checks=c.a | binding=all",
    );
    assert_renders(
        "gate g | checks=c.a | binding=any",
        "gate g | checks=c.a | binding=any",
    );
    assert_renders(
        "use pack p | activation=workspace | lockfile=l.lock",
        "use pack p | activation=workspace | lockfile=l.lock",
    );
}

// ---- query compose shapes --------------------------------------------------

/// `group by` with multiple keys and `order by` with mixed directions render
/// through the keyed-compose path.
#[test]
fn query_compose_shapes_render() {
    assert_renders(
        "from o x | group by x.a, x.b",
        "from o x | group by x.a, x.b",
    );
    assert_renders(
        "from o x | order by x.total desc, x.id asc",
        "from o x | order by x.total desc, x.id asc",
    );
    assert_renders("from a x | cross join b y", "from a x | cross join b y");
    assert_renders(
        "from a x | join b y on x.k = y.k",
        "from a x | join b y on x.k = y.k",
    );
    // Shape delimiters and spacing are pinned, not only round-trip stable.
    assert_renders(
        "from users u => { id: u.id, tags: [ u.t1 ] }",
        "from users u => { id: u.id, tags: [ u.t1 ] }",
    );
}
