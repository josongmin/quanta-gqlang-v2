#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

mod support;

use quanta_gqlang_v2_hir::{
    AdmissionV1, BoolExprKindV1, CatalogPayloadV1, GraphConstraintsV1, GraphEndpointsV1,
    GraphQueryKindV1, GraphRefV1, HirBuilderV1, PathSegmentV1, ProgramItemHirV1,
    RelationExistsPolarityV1, SpanV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{
    format_document, format_item, parse_document, parse_program, try_format_item,
    FormatErrorCodeV1, SurfaceItemV1,
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

fn assert_deep_relation_child(test_name: &str, child_env: &str) {
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test_name, "--nocapture"])
        .env(child_env, "1")
        .output()
        .expect("child process");
    assert!(
        output.status.success(),
        "{test_name} status: {}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("running 1 test")
            && stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;"),
        "{test_name} selected zero or multiple tests: {stdout}"
    );
}

fn assert_deep_relation_render(join: bool) {
    const DEPTH: usize = 3_000;
    const FILTER_PREFIX: &str = "from f | where exists (";
    const JOIN_PREFIX: &str = "from l | join r on not exists (";

    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let mut relation = builder.relation_source("leaf", span, None, span);
    for _ in 0..DEPTH {
        let polarity = if join {
            RelationExistsPolarityV1::NotExists
        } else {
            RelationExistsPolarityV1::Exists
        };
        let predicate = builder.relation_exists(polarity, relation, span);
        relation = if join {
            builder.inner_join(
                builder.relation_source("l", span, None, span),
                builder.relation_source("r", span, None, span),
                predicate,
                span,
            )
        } else {
            builder.relation_filter(
                builder.relation_source("f", span, None, span),
                predicate,
                span,
            )
        };
    }
    let item = ProgramItemHirV1::from(builder.query(relation, None, span));
    let rendered = format_item(&item);
    // Raw HIR depth, rather than parser input, is the formatter's public boundary.
    // Deep recursive destruction is owned by HIR and must not obscure this rail.
    std::mem::forget(item);

    let prefix = if join { JOIN_PREFIX } else { FILTER_PREFIX };
    let suffix = format!("from leaf{}", ")".repeat(DEPTH));
    let expected = format!("{}{suffix}", prefix.repeat(DEPTH));
    assert_eq!(rendered.len(), expected.len());
    assert!(rendered.starts_with(prefix));
    assert!(rendered.ends_with(&suffix));
    assert_eq!(rendered, expected);
}

#[test]
fn deep_filter_exists_formatter_child() {
    if std::env::var_os("QGLANG_DEEP_FILTER_FORMAT_CHILD").is_none() {
        return;
    }
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| assert_deep_relation_render(false))
        .expect("small-stack formatter thread")
        .join()
        .expect("small-stack formatter completed");
}

#[test]
fn deep_filter_exists_formatter_is_stack_safe() {
    assert_deep_relation_child(
        "deep_filter_exists_formatter_child",
        "QGLANG_DEEP_FILTER_FORMAT_CHILD",
    );
}

#[test]
fn deep_join_exists_formatter_child() {
    if std::env::var_os("QGLANG_DEEP_JOIN_FORMAT_CHILD").is_none() {
        return;
    }
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| assert_deep_relation_render(true))
        .expect("small-stack formatter thread")
        .join()
        .expect("small-stack formatter completed");
}

#[test]
fn deep_join_exists_formatter_is_stack_safe() {
    assert_deep_relation_child(
        "deep_join_exists_formatter_child",
        "QGLANG_DEEP_JOIN_FORMAT_CHILD",
    );
}

#[test]
fn deep_mixed_relation_formatter_child() {
    if std::env::var_os("QGLANG_DEEP_MIXED_FORMAT_CHILD").is_none() {
        return;
    }
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            const DEPTH: usize = 3_000;
            const FILTER_PREFIX: &str = "from f | where exists (";
            const JOIN_PREFIX: &str = "from l | join r on not exists (";

            let builder = HirBuilderV1::new();
            let span = SpanV1::new(0, 1);
            let mut relation = builder.relation_source("leaf", span, None, span);
            for level in 0..DEPTH {
                let is_join = level % 2 == 1;
                let polarity = if is_join {
                    RelationExistsPolarityV1::NotExists
                } else {
                    RelationExistsPolarityV1::Exists
                };
                let predicate = builder.relation_exists(polarity, relation, span);
                relation = if is_join {
                    builder.inner_join(
                        builder.relation_source("l", span, None, span),
                        builder.relation_source("r", span, None, span),
                        predicate,
                        span,
                    )
                } else {
                    builder.relation_filter(
                        builder.relation_source("f", span, None, span),
                        predicate,
                        span,
                    )
                };
            }
            let item = ProgramItemHirV1::from(builder.query(relation, None, span));
            let rendered = format_item(&item);
            std::mem::forget(item);

            // The HIR is wrapped from the leaf outward; surface text emits in
            // the opposite order. This oracle does not call a formatter helper.
            let mut expected = String::new();
            for level in (0..DEPTH).rev() {
                expected.push_str(if level % 2 == 1 {
                    JOIN_PREFIX
                } else {
                    FILTER_PREFIX
                });
            }
            expected.push_str("from leaf");
            expected.push_str(&")".repeat(DEPTH));
            assert_eq!(rendered.len(), expected.len());
            assert!(rendered.starts_with(JOIN_PREFIX));
            assert!(rendered.ends_with(&format!("from leaf{}", ")".repeat(DEPTH))));
            assert_eq!(rendered, expected);
        })
        .expect("small-stack formatter thread")
        .join()
        .expect("small-stack formatter completed");
}

#[test]
fn deep_mixed_relation_formatter_is_stack_safe() {
    assert_deep_relation_child(
        "deep_mixed_relation_formatter_child",
        "QGLANG_DEEP_MIXED_FORMAT_CHILD",
    );
}

#[test]
fn relation_exists_precedence_and_polarity_are_exact() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let inner = builder.relation_filter(
        builder.relation_source("b", span, None, span),
        builder.bool_literal(true, span),
        span,
    );
    let exists = builder.relation_exists(RelationExistsPolarityV1::Exists, inner, span);
    let not_exists = builder.relation_exists(
        RelationExistsPolarityV1::NotExists,
        builder.relation_source("c", span, None, span),
        span,
    );
    let predicate = builder.or(
        builder.and(
            exists,
            builder.bool_value(builder.symbol("x", span), span),
            span,
        ),
        builder.not(not_exists, span),
        span,
    );
    let item = ProgramItemHirV1::from(builder.query(
        builder.relation_filter(
            builder.relation_source("a", span, None, span),
            predicate,
            span,
        ),
        None,
        span,
    ));
    assert_eq!(
        format_item(&item),
        "from a | where exists (from b | where true) and x or not (not exists (from c))"
    );
}

#[test]
fn raw_unary_path_base_keeps_the_unary_inside_the_path() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let base = builder.unary("-", builder.symbol("a", span), span);
    let path = builder
        .try_path(
            base,
            vec![PathSegmentV1 {
                name: "b".into(),
                span,
            }],
            span,
        )
        .expect("one path segment");
    let item = ProgramItemHirV1::from(builder.decl("x", span, path, span));
    let formatted = format_item(&item);
    assert_eq!(formatted, "let x = (-a).b");
    let ProgramItemHirV1::Decl(reparsed) = parse(&formatted) else {
        panic!("declaration must reparse");
    };
    let ValueExprKindV1::Path(path) = &reparsed.body.kind else {
        panic!("path must remain the outer value node");
    };
    let ValueExprKindV1::Paren(inner) = &path.base.kind else {
        panic!("path base must carry protective parentheses");
    };
    assert!(matches!(&inner.kind, ValueExprKindV1::Unary(_)));
}

#[test]
fn raw_not_exists_keeps_not_distinct_from_not_exists_polarity() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        builder.relation_source("r", span, None, span),
        span,
    );
    let item = ProgramItemHirV1::from(builder.rule(
        "x",
        span,
        builder.symbol("x", span),
        builder.not(exists, span),
        span,
    ));
    let formatted = format_item(&item);
    assert_eq!(formatted, "rule x : x if not (exists (from r))");
    let ProgramItemHirV1::Rule(reparsed) = parse(&formatted) else {
        panic!("rule must reparse");
    };
    let BoolExprKindV1::Not(inner) = &reparsed.body.kind else {
        panic!("outer Not must survive parsing");
    };
    let BoolExprKindV1::Paren(inner) = &inner.kind else {
        panic!("inner Exists must have protective parentheses");
    };
    assert!(matches!(
        &inner.kind,
        BoolExprKindV1::RelationExists(exists)
            if exists.polarity == RelationExistsPolarityV1::Exists
    ));
}

#[test]
fn checked_formatter_rejects_unrepresentable_raw_hir_before_silent_loss() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let predicate = || builder.bool_value(builder.symbol("ok", span), span);
    let staged_right = builder.relation_filter(
        builder.relation_source("b", span, None, span),
        predicate(),
        span,
    );
    let join = ProgramItemHirV1::from(builder.query(
        builder.inner_join(
            builder.relation_source("a", span, None, span),
            staged_right,
            predicate(),
            span,
        ),
        None,
        span,
    ));
    assert_eq!(
        try_format_item(&join).unwrap_err().code,
        FormatErrorCodeV1::JoinRightNotSource
    );

    let empty_shape = ProgramItemHirV1::from(builder.query(
        builder.relation_source("a", span, None, span),
        Some(builder.shape_record(Vec::new(), span)),
        span,
    ));
    assert_eq!(
        try_format_item(&empty_shape).unwrap_err().code,
        FormatErrorCodeV1::EmptyResultShape
    );

    let partial_endpoints = ProgramItemHirV1::from(builder.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "source".into(),
                span,
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        span,
    ));
    assert_eq!(
        try_format_item(&partial_endpoints).unwrap_err().code,
        FormatErrorCodeV1::PartialGraphEndpoints
    );

    let raw_bool_literal = ProgramItemHirV1::from(builder.rule(
        "r",
        span,
        builder.symbol("x", span),
        builder.bool_literal(true, span),
        span,
    ));
    assert_eq!(
        try_format_item(&raw_bool_literal).unwrap_err().code,
        FormatErrorCodeV1::RawBoolLiteral
    );

    let ambiguous_bool_paren = ProgramItemHirV1::from(builder.rule(
        "r",
        span,
        builder.symbol("x", span),
        builder.paren_bool(predicate(), span),
        span,
    ));
    assert_eq!(
        try_format_item(&ambiguous_bool_paren).unwrap_err().code,
        FormatErrorCodeV1::AmbiguousBoolParen
    );
}

#[test]
fn checked_formatter_requires_parser_admission_and_preserves_positive_cases() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let malformed =
        ProgramItemHirV1::from(builder.decl("bad name", span, builder.int("1", span), span));
    let error = try_format_item(&malformed).expect_err("invalid raw identifier rejects");
    assert_eq!(error.code, FormatErrorCodeV1::ParseRejected);
    assert!(
        error.parse.is_some(),
        "retain the exact rendered parse error"
    );

    for source in [
        "let x = (-a).b",
        "rule r : x if not (exists (from a))",
        "from a | join b on a.id = b.id => { id: a.id }",
        "find reachability source -> sink | depth=2",
    ] {
        let item = parse(source);
        assert_eq!(
            try_format_item(&item).expect("parser-produced HIR is representable"),
            format_item(&item),
            "{source}"
        );
    }
}

#[test]
fn checked_formatter_detects_canonical_text_with_split_raw_list_payloads() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let split_ref = GraphRefV1 {
        name: "a, b".into(),
        span,
    };
    for constraints in [
        GraphConstraintsV1 {
            via: vec![split_ref.clone()],
            ..GraphConstraintsV1::default()
        },
        GraphConstraintsV1 {
            summary_facets: vec!["calls, reads".into()],
            ..GraphConstraintsV1::default()
        },
    ] {
        let graph = ProgramItemHirV1::from(builder.graph_query(
            GraphQueryKindV1::Summary,
            GraphEndpointsV1::default(),
            constraints,
            span,
        ));
        assert_eq!(
            try_format_item(&graph).unwrap_err().code,
            FormatErrorCodeV1::NonCanonical,
            "parse/format fixed point alone cannot detect a split list element"
        );
    }

    let catalog = ProgramItemHirV1::from(builder.catalog_decl(
        "profile_name",
        span,
        CatalogPayloadV1::Profile {
            capabilities: vec![split_ref],
            admission: AdmissionV1::Allowlist,
        },
        span,
    ));
    assert_eq!(
        try_format_item(&catalog).unwrap_err().code,
        FormatErrorCodeV1::NonCanonical
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
    assert_eq!(
        try_format_item(&item1).expect("parser-produced item passes checked formatter"),
        text1,
        "checked formatting diverged for {src:?}"
    );

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
