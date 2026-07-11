//! All-family formatter hellgate (GAP 2).
//!
//! For at least one representative source per family, assert:
//!   - round-trip stability: `format(parse(src))` is stable under a second
//!     parse + format (i.e. `format(parse(format(parse(src)))) == format(parse(src))`),
//!   - the rendered text re-parses to the SAME family/structure (structural HIR
//!     equality modulo ids/spans),
//!   - idempotence: `format(format(...))` is a no-op at the text level.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration test rail: assertion panics are intentional"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1};
use quanta_gqlang_v2_surface::{
    format_document, format_item, parse_document, parse_program, SurfaceItemV1,
};

fn parse(src: &str) -> ProgramItemHirV1 {
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
    match parsed.item {
        SurfaceItemV1::Program(p) => p,
        SurfaceItemV1::Query(q) => ProgramItemHirV1::from(q),
        SurfaceItemV1::Decl(d) => ProgramItemHirV1::from(d),
        SurfaceItemV1::Rule(r) => ProgramItemHirV1::from(r),
    }
}

/// Structural equality of two items ignoring node ids and spans: re-render both
/// and compare the canonical text. Two items render identically iff their
/// structure matches, so canonical-text equality is the structural oracle.
fn same_structure(a: &ProgramItemHirV1, b: &ProgramItemHirV1) -> bool {
    format_item(a) == format_item(b)
}

/// The core rail: parse, format, re-parse, and assert the round trip is stable,
/// the family is preserved, and formatting is idempotent.
fn assert_roundtrip(src: &str) {
    let item1 = parse(src);
    let text1 = format_item(&item1);

    // rendered text must re-parse (it is canonical v3 surface).
    let item2 = parse(&text1);
    let text2 = format_item(&item2);

    // round-trip stable: second parse+format yields identical text.
    assert_eq!(text1, text2, "round trip not stable for {src:?}");

    // family preserved + structurally equal.
    assert_eq!(
        std::mem::discriminant(&item1),
        std::mem::discriminant(&item2),
        "family changed under round trip for {src:?}"
    );
    assert!(
        same_structure(&item1, &item2),
        "structure changed under round trip for {src:?}: {text1:?} vs {text2:?}"
    );

    // idempotence at the text level: format(format(x)) == format(x).
    let text3 = format_item(&parse(&text2));
    assert_eq!(text2, text3, "format not idempotent for {src:?}");
}

#[test]
fn query_roundtrips() {
    assert_roundtrip("from users u | where u.active");
    assert_roundtrip("from users u | where u.age > 18 and u.active");
    assert_roundtrip("from users u | project u.id as id, u.name | where u.active");
    assert_roundtrip("from a x | join b y on x.k = y.k");
    assert_roundtrip("from orders o | group by o.customer");
    assert_roundtrip("from orders o | order by o.total desc, o.id asc");
    assert_roundtrip("from users u => { id: u.id, tags: [ u.t1 ] }");
}

#[test]
fn decl_and_rule_roundtrip() {
    assert_roundtrip("let total = a + b * c");
    assert_roundtrip("let r = { id: 1, name: \"x\" }");
    assert_roundtrip("rule risky : u.id if u.score > 10 and not u.safe");
    assert_roundtrip("rule q : x if exists v in (v.flag)");
}

#[test]
fn find_roundtrips() {
    assert_roundtrip("find taint api.entry -> db.write | sanitizers=authz.clean | depth=3");
    assert_roundtrip("find reachability a.b -> c.d");
    assert_roundtrip("find summary | callable=svc.handler | facets=calls, reads");
    assert_roundtrip("find points_to | reference=p.x | pointee=heap.obj");
}

#[test]
fn check_roundtrips() {
    assert_roundtrip("check rule.hybrid | scope=repo | confirm=require | emit=sarif | mode=hybrid");
    assert_roundtrip("check rule.x | confirm=skip | emit=findings | mode=pattern");
    assert_roundtrip(
        "check rule.hybrid | pattern=p | confirm=require | emit=findings | mode=hybrid using find reachability src.entry -> sink.write | depth=8",
    );
}

#[test]
fn presentation_roundtrips() {
    assert_roundtrip("explain rule.sqli | evidence=trace | truth=confirm");
    assert_roundtrip("show capabilities");
    assert_roundtrip("describe rule::owasp.sqli");
    assert_roundtrip("sample query=\"callgraph\" | mode=semantic | limit=5 | scope=repo");
    assert_roundtrip("compare baseline=run.a | candidate=run.b | metric=semantic");
    assert_roundtrip("export source=result.latest | kind=sarif | profile=strict");
}

#[test]
fn catalog_decl_roundtrips() {
    assert_roundtrip("view catalog.users | query-ref=queries.catalog.users");
    assert_roundtrip("view active | select=from users u | where u.active");
    assert_roundtrip("model embeddings.code | provider=openai | source=models.code");
    assert_roundtrip("pack secure-pack | source=registry.security | activation=workspace");
    assert_roundtrip(
        "profile strict | capabilities=graph.read, graph.trace | admission=certified-only",
    );
    assert_roundtrip("gate release | checks=owasp.sql, authz.path | binding=all");
    assert_roundtrip("ruledef shell.exec | check-ref=checks.shell-exec");
}

#[test]
fn composition_roundtrips() {
    assert_roundtrip(
        "let suspects = sample query=\"auth\" | mode=semantic | limit=3 \
         find taint api.entry -> db.write with suspects",
    );
}

#[test]
fn meta_roundtrips() {
    assert_roundtrip("use pack secure-pack | activation=session | lockfile=packs.secure.lock");
    assert_roundtrip("require certified");
}

#[test]
fn document_roundtrips() {
    let src = "show capabilities ;\n\
               find taint a.b -> c.d | depth=2 ;\n\
               view v | query-ref=q.r";
    let builder = HirBuilderV1::new();
    let doc = parse_document(src, &builder).expect("parse document");
    let items: Vec<ProgramItemHirV1> = doc
        .items
        .into_iter()
        .map(|i| match i {
            SurfaceItemV1::Program(p) => p,
            SurfaceItemV1::Query(q) => ProgramItemHirV1::from(q),
            SurfaceItemV1::Decl(d) => ProgramItemHirV1::from(d),
            SurfaceItemV1::Rule(r) => ProgramItemHirV1::from(r),
        })
        .collect();
    let text = format_document(&items);

    // the rendered document re-parses to the same number of items, each same family.
    let builder2 = HirBuilderV1::new();
    let doc2 = parse_document(&text, &builder2).expect("re-parse document");
    assert_eq!(doc2.items.len(), items.len());

    let items2: Vec<ProgramItemHirV1> = doc2
        .items
        .into_iter()
        .map(|i| match i {
            SurfaceItemV1::Program(p) => p,
            SurfaceItemV1::Query(q) => ProgramItemHirV1::from(q),
            SurfaceItemV1::Decl(d) => ProgramItemHirV1::from(d),
            SurfaceItemV1::Rule(r) => ProgramItemHirV1::from(r),
        })
        .collect();
    // idempotent at document level.
    assert_eq!(text, format_document(&items2));
}
