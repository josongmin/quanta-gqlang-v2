#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

mod support;

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1, SpanV1};
use quanta_gqlang_v2_surface::{
    format_document, format_item, parse_document, parse_program, SurfaceItemV1,
};

fn parse(src: &str) -> ProgramItemHirV1 {
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
    parsed.item.into_program_item()
}

/// Structural equality of two items ignoring node ids and spans: re-render both
/// and compare the canonical text. Two items render identically iff their
/// structure matches, so canonical-text equality is the structural oracle.
fn same_structure(a: &ProgramItemHirV1, b: &ProgramItemHirV1) -> bool {
    support::structural_witness(a) == support::structural_witness(b)
}

#[test]
fn deep_shape_formatter_child() {
    if std::env::var_os("QGLANG_DEEP_SHAPE_FORMAT_CHILD").is_none() {
        return;
    }

    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let mut shape = builder.shape_leaf(builder.int("1", span), span);
    const DEPTH: usize = 100_000;
    for _ in 0..DEPTH {
        shape = builder.shape_list(shape, span);
    }
    let item = ProgramItemHirV1::from(builder.query(
        builder.relation_source("r", span, None, span),
        Some(shape),
        span,
    ));

    let rendered = format_item(&item);
    assert_eq!(rendered.len(), "from r => ".len() + DEPTH * 4 + 1);
    assert!(rendered.starts_with("from r => [ [ [ "));
    assert!(rendered.ends_with(" ] ] ]"));

    // Keep this rail scoped to formatter stack behavior: recursively dropping
    // an adversarial raw-HIR shape is a separate ownership concern.
    std::mem::forget(item);
}

#[test]
fn deep_shape_formatter_is_stack_safe() {
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "deep_shape_formatter_child", "--nocapture"])
        .env("QGLANG_DEEP_SHAPE_FORMAT_CHILD", "1")
        .output()
        .expect("child process");
    assert!(
        output.status.success(),
        "deep shape formatter child status: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;"),
        "deep shape formatter child selected zero or multiple tests: {stdout}"
    );
}

#[test]
fn programmatic_precedence_is_preserved_by_reparse() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let add = builder.binary(
        "+",
        builder.symbol("a", span),
        builder.symbol("b", span),
        span,
    );
    let mul = builder.binary("*", add, builder.symbol("c", span), span);
    let item = ProgramItemHirV1::from(builder.decl("x", span, mul, span));
    let reparsed = parse(&format_item(&item));
    assert_eq!(
        support::structural_witness(&item),
        support::structural_witness(&reparsed)
    );

    let and = builder.and(
        builder.bool_value(builder.symbol("a", span), span),
        builder.bool_value(builder.symbol("b", span), span),
        span,
    );
    let not = builder.not(and, span);
    let rule =
        ProgramItemHirV1::from(builder.rule("r", span, builder.symbol("x", span), not, span));
    let reparsed = parse(&format_item(&rule));
    assert_eq!(
        support::structural_witness(&rule),
        support::structural_witness(&reparsed)
    );
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

fn assert_witness_diff(left: &str, right: &str) {
    assert_ne!(
        support::structural_witness(&parse(left)),
        support::structural_witness(&parse(right)),
        "witness collapsed semantic mutation: {left:?} vs {right:?}"
    );
}

#[test]
fn structural_witness_distinguishes_every_semantic_payload_category() {
    for (left, right) in [
        ("from a", "from b"),
        ("let x = 1", "let y = 1"),
        ("rule r : x if true", "rule s : x if true"),
        ("find flow a -> b", "find taint a -> b"),
        ("check r | confirm=skip", "check r | confirm=prompt"),
        ("show capabilities", "show models"),
        (
            "model m | provider=a | source=s",
            "model m | provider=b | source=s",
        ),
        (
            "let a = sample query=\"x\" | mode=lexical | limit=1 find flow x -> y with a",
            "let b = sample query=\"x\" | mode=lexical | limit=1 find flow x -> y with b",
        ),
        (
            "use pack p | activation=session | lockfile=a.lock",
            "use pack p | activation=session | lockfile=b.lock",
        ),
        ("let x = a + b", "let x = a * b"),
        ("rule r : x if a and b", "rule r : x if a or b"),
        (
            "from a | where exists (from b)",
            "from a | where not exists (from b)",
        ),
        ("from a | group by x", "from a | project x"),
        ("from a => { x: a }", "from a => { y: a }"),
        ("from a x", "from a y"),
        ("from a | order by x asc", "from a | order by x desc"),
        (
            "rule r : x if exists a in (a)",
            "rule r : x if exists b in (b)",
        ),
        ("let x = f(a)", "let x = f(*)"),
    ] {
        assert_witness_diff(left, right);
    }
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
        .map(SurfaceItemV1::into_program_item)
        .collect();
    let text = format_document(&items);

    // the rendered document re-parses to the same number of items, each same family.
    let builder2 = HirBuilderV1::new();
    let doc2 = parse_document(&text, &builder2).expect("re-parse document");
    assert_eq!(doc2.items.len(), items.len());

    let items2: Vec<ProgramItemHirV1> = doc2
        .items
        .into_iter()
        .map(SurfaceItemV1::into_program_item)
        .collect();
    // idempotent at document level.
    assert_eq!(text, format_document(&items2));
}
