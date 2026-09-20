//! Formatter edge rails: round-trips for the value/bool kinds, family modes,
//! and compose shapes the main formatter hellgate leaves uncovered.
//!
//! Each rail asserts that the canonical render re-parses to the same independent
//! visitor-derived structure and is idempotent. These lock the
//! formatter's per-kind arms (unary, call, paren, list, escaped string, every
//! literal kind, each bool connective, each find subkind, each enum-valued mode)
//! as the exact inverse of the grammar.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration test rail: assertion panics are intentional"
)]

mod support;

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1};
use quanta_gqlang_v2_surface::{format_item, parse_program};

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
    assert_roundtrip("let x = \"a\\\"b\"");
    assert_roundtrip("let x = \"a\\\\b\"");
    assert_roundtrip("let x = \"a\\nb\"");
    assert_roundtrip("let x = \"a\\tb\"");
    assert_roundtrip("let x = \"a\\rb\\bc\\f\"");
    assert_roundtrip("let x = \"\\u{1f642}\"");
    assert_renders("let x = \"한글🙂é\"", "let x = \"한글🙂é\"");
}

#[test]
fn invalid_escape_forms_reject() {
    for source in [
        r#"let x = "\q""#,
        r#"let x = "\u{}""#,
        r#"let x = "\u{d800}""#,
        r#"let x = "\u110000""#,
    ] {
        let builder = HirBuilderV1::new();
        assert!(
            parse_program(source, &builder).is_err(),
            "accepted {source:?}"
        );
    }
}

// ---- bool-expr connectives -------------------------------------------------

/// `or`, `not`, parenthesized bool, and quantifier-with-binders all round-trip
/// through a rule body (the bool formatter's uncovered arms).
#[test]
fn bool_connectives_render_canonically() {
    assert_roundtrip("rule r : x if a or b");
    assert_roundtrip("rule r : x if not a");
    assert_roundtrip("rule r : x if (a or b) and c");
    assert_roundtrip("rule r : x if a and (b or c)");
    // a parenthesized value-comparison stays a value paren, not a bool paren.
    assert_roundtrip("rule r : x if (a + b) > c");
    assert_roundtrip("rule r : x if exists v in (v.flag)");
    assert_roundtrip("rule r : x if forall v, w in (v.k = w.k)");
    // bare-value and comparison bool positions.
    assert_roundtrip("rule r : x if a > b");
    assert_roundtrip("rule r : x if a");
}

// ---- FIND subkinds + constraint clauses ------------------------------------

/// Every FIND subkind renders to its lowercase keyword.
#[test]
fn find_all_subkinds_round_trip() {
    assert_roundtrip("find reachability a.b -> c.d");
    assert_roundtrip("find flow a.b -> c.d");
    assert_roundtrip("find taint a.b -> c.d");
    assert_roundtrip("find path a.b -> c.d");
    assert_roundtrip("find slice | scope=mod");
    assert_roundtrip("find points_to | reference=p.x | pointee=h.o");
    assert_roundtrip("find summary | callable=svc.h | facets=calls");
}

/// The constraint clauses the main hellgate skips: via/through, window,
/// frontier, direction, anchor, and a directioned walk.
#[test]
fn find_constraint_clauses_round_trip() {
    assert_roundtrip("find taint a.b -> c.d | via=x.y | through=z.w");
    assert_roundtrip("find reachability a.b -> c.d | window=win.scope | frontier=fr.edge");
    assert_roundtrip("find reachability a.b -> c.d | direction=backward");
    assert_roundtrip("find reachability a.b -> c.d | direction=undirected");
    assert_roundtrip("find flow a.b -> c.d | anchor=an.node");
}

// ---- CHECK / presentation modes --------------------------------------------

/// Each CHECK enum value renders to its keyword.
#[test]
fn check_modes_render() {
    assert_roundtrip("check r.x | confirm=skip | emit=findings | mode=pattern");
    assert_roundtrip("check r.x | confirm=prompt | emit=sarif | mode=semantic");
    assert_roundtrip("check r.x | confirm=require | emit=findings | mode=hybrid");
}

/// Each EXPLAIN evidence/truth value renders to its keyword.
#[test]
fn explain_modes_render() {
    assert_roundtrip("explain r.x | evidence=auto | truth=inspect");
    assert_roundtrip("explain r.x | evidence=citation | truth=refute");
    assert_roundtrip("explain r.x | evidence=counterexample | truth=confirm");
}

/// Each SHOW category renders.
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
        assert_roundtrip(&format!("show {cat}"));
    }
}

/// SAMPLE modes and COMPARE metrics render.
#[test]
fn sample_and_compare_modes_render() {
    assert_roundtrip("sample query=\"q\" | mode=lexical | limit=3");
    assert_roundtrip("sample query=\"q\" | mode=hybrid | limit=3");
    assert_roundtrip("compare baseline=a | candidate=b | metric=exact");
    assert_roundtrip("compare baseline=a | candidate=b | metric=structural");
}

/// Every EXPORT format keyword renders.
#[test]
fn export_formats_render() {
    for fmt in ["json", "csv", "sarif", "markdown", "graph-ml"] {
        assert_roundtrip(&format!("export source=r.x | kind={fmt}"));
    }
}

// ---- catalog enum-valued modes ---------------------------------------------

/// PACK / USE PACK activation policies and PROFILE admission / GATE binding all
/// render to their keyword forms.
#[test]
fn catalog_enum_modes_render() {
    assert_roundtrip("pack p | source=r.x | activation=manual");
    assert_roundtrip("pack p | source=r.x | activation=session");
    assert_roundtrip("pack p | source=r.x | activation=workspace");
    assert_roundtrip("profile p | capabilities=c.a | admission=allowlist");
    assert_roundtrip("profile p | capabilities=c.a | admission=review-required");
    assert_roundtrip("profile p | capabilities=c.a | admission=certified-only");
    assert_roundtrip("gate g | checks=c.a | binding=all");
    assert_roundtrip("gate g | checks=c.a | binding=any");
    assert_roundtrip("use pack p | activation=workspace | lockfile=l.lock");
}

// ---- query compose shapes --------------------------------------------------

/// `group by` with multiple keys and `order by` with mixed directions render
/// through the keyed-compose path.
#[test]
fn query_compose_shapes_render() {
    assert_roundtrip("from o x | group by x.a, x.b");
    assert_roundtrip("from o x | order by x.total desc, x.id asc");
    assert_roundtrip("from a x | cross join b y");
    assert_roundtrip("from a x | join b y on x.k = y.k");
}
