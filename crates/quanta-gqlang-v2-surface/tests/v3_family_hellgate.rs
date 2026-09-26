#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, ActivationV1, AdmissionV1, BoolExprKindV1, CatalogPayloadV1, CompareMetricV1,
    CompositionBodyV1, CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExportFormatV1, GateBindingV1, GraphQueryKindV1, HirBuilderV1, HirRootRefV1,
    MetaKindV1, PresentationKindV1, ProgramItemHirV1, RelationExprKindV1, SampleModeV1,
    ShowCategoryV1, SpanV1, TruthModeV1, UsePackActivationV1, ValueExprKindV1, ViewBodyV1,
};
use quanta_gqlang_v2_surface::{parse_program, ParseErrorCodeV1};

fn program(src: &str) -> ProgramItemHirV1 {
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("must parse");
    let item = parsed.item.into_program_item();
    // every parsed family must pass the shared HIR integrity check.
    check_tree(HirRootRefV1::Item(&item), parsed.source_len).expect("tree integrity");
    item
}

fn err(src: &str) -> quanta_gqlang_v2_surface::ParseDiagnosticV1 {
    let builder = HirBuilderV1::new();
    parse_program(src, &builder).expect_err("must reject")
}

fn assert_empty_find_list_rejects(src: &str, key: &str) {
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, format!("FIND {key} list must not be empty"));
    let start = src
        .find(key)
        .expect("list key must be present in the test input");
    assert_eq!(diag.span.start, start);
    assert_eq!(diag.span.end, start + key.len());
}

// ---- FIND ------------------------------------------------------------------

#[test]
fn find_taint_parses_to_graph_query() {
    let src = "find taint api.entry -> db.write | sanitizers=authz.clean,authz.safe | depth=3";
    let item = program(src);
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(gq.kind, GraphQueryKindV1::Taint);
    assert_eq!(gq.endpoints.source.as_ref().unwrap().name, "api.entry");
    assert_eq!(gq.endpoints.sink.as_ref().unwrap().name, "db.write");
    let sanitizers: Vec<&str> = gq
        .constraints
        .sanitizers
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(sanitizers, ["authz.clean", "authz.safe"]);
    assert_eq!(gq.constraints.depth, Some(3));
    // span starts at `find` (byte 0) and ends at the last clause value.
    assert_eq!(gq.span.start, 0);
    assert_eq!(gq.span.end, src.len());
    // source ref span is exact: `api.entry` at byte 11.
    assert_eq!(gq.endpoints.source.as_ref().unwrap().span.start, 11);
    assert_eq!(gq.endpoints.source.as_ref().unwrap().span.end, 20);
}

#[test]
fn find_reachability_subkind_parses() {
    let item = program("find reachability a.b -> c.d");
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(gq.kind, GraphQueryKindV1::Reachability);
}

/// Every admitted FIND subkind keyword lowers to its EXACT `GraphQueryKindV1`.
///
/// The parser owns the keyword→kind mapping; the format rails are
/// stability-only (`format(parse(x)) == format(parse(format(parse(x))))`), so a
/// mismapped arm such as `"flow" => GraphQueryKindV1::Taint` is self-consistent
/// and survives them, and the pipeline rails assert only "is a GraphQuery".
/// This rail pins the mapping itself, so drift on any single arm fails closed.
#[test]
fn find_subkind_keywords_map_to_exact_kinds() {
    let cases = [
        (
            "find reachability a.b -> c.d",
            GraphQueryKindV1::Reachability,
        ),
        ("find flow a.b -> c.d", GraphQueryKindV1::Flow),
        ("find slice | anchor=var | depth=2", GraphQueryKindV1::Slice),
        ("find taint a.b -> c.d", GraphQueryKindV1::Taint),
        ("find path a.b -> c.d", GraphQueryKindV1::Path),
        (
            "find points_to | reference=ptr | pointee=obj | depth=2",
            GraphQueryKindV1::PointsTo,
        ),
        ("find summary | facets=calls", GraphQueryKindV1::Summary),
    ];
    for (src, expected) in cases {
        let item = program(src);
        let gq = match item {
            ProgramItemHirV1::GraphQuery(g) => g,
            other => panic!("{src:?}: expected GraphQuery, got {other:?}"),
        };
        assert_eq!(
            gq.kind, expected,
            "src {src:?} lowered to the wrong GraphQueryKindV1"
        );
    }
}

#[test]
fn find_missing_arrow_rejects() {
    // malformed FIND seed: no `->` between source and sink.
    let diag = err("find taint api.entry db.write");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(
        diag.message,
        "expected '->' between the FIND source and sink"
    );
    assert_eq!(diag.span, SpanV1::new(21, 23));
}

#[test]
fn find_unknown_subkind_rejects() {
    let diag = err("find frobnicate a -> b");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, "unknown FIND subkind 'frobnicate'");
    // points at the bad subkind token `frobnicate` at bytes 5..15.
    assert_eq!(diag.span, SpanV1::new(5, 15));
}

// ---- CHECK -----------------------------------------------------------------

#[test]
fn check_parses_to_legality() {
    let src = "check rule.hybrid | scope=repo | confirm=require | emit=sarif | mode=hybrid";
    let item = program(src);
    let chk = match item {
        ProgramItemHirV1::Legality(c) => c,
        other => panic!("expected Legality, got {other:?}"),
    };
    assert_eq!(chk.rule_ref.name, "rule.hybrid");
    assert_eq!(chk.confirm, ConfirmModeV1::Require);
    assert_eq!(chk.emit, EmitModeV1::Sarif);
    assert_eq!(chk.mode, DetectionModeV1::Hybrid);
    assert_eq!(chk.scope.as_ref().unwrap().name, "repo");
    assert_eq!(chk.span.start, 0);
    assert_eq!(chk.span.end, src.len());
}

#[test]
fn check_owns_explicit_semantic_confirm_query() {
    let src = "check rule.hybrid | pattern=p | confirm=require | emit=findings | mode=hybrid using find reachability src.entry -> sink.write | depth=8";
    let item = program(src);
    let chk = match item {
        ProgramItemHirV1::Legality(c) => c,
        other => panic!("expected Legality, got {other:?}"),
    };
    let confirm = chk
        .semantic_confirm
        .as_ref()
        .expect("an explicit semantic confirm query is retained in CHECK HIR");
    assert_eq!(confirm.kind, GraphQueryKindV1::Reachability);
    assert_eq!(
        confirm.endpoints.source.as_ref().map(|r| r.name.as_str()),
        Some("src.entry")
    );
    assert_eq!(
        confirm.endpoints.sink.as_ref().map(|r| r.name.as_str()),
        Some("sink.write")
    );
    assert_eq!(confirm.constraints.depth, Some(8));
    assert_eq!(chk.span.end, src.len());
}

#[test]
fn check_using_requires_a_find_query() {
    let src = "check rule.hybrid | confirm=require | mode=hybrid using check other | confirm=skip";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    let invalid_keyword = src.find("using check").expect("authored using clause") + "using ".len();
    assert_eq!(
        diag.span,
        SpanV1::new(invalid_keyword, invalid_keyword + "check".len())
    );
    assert!(diag.message.contains("find"), "msg: {}", diag.message);
}

#[test]
fn check_missing_confirm_rejects() {
    // CHECK requires an explicit confirm clause: fail closed when absent.
    let diag = err("check rule.hybrid | scope=repo | emit=findings");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.span, SpanV1::new(0, 5));
    assert!(diag.message.contains("confirm"), "msg: {}", diag.message);
}

#[test]
fn check_ref_double_colon_separator_is_atomic() {
    // REF-01: `::` is exactly two colons. The positive case retains the full ref.
    let src = "check repo::auth | confirm=skip";
    let item = program(src);
    let chk = match item {
        ProgramItemHirV1::Legality(c) => c,
        other => panic!("expected Legality, got {other:?}"),
    };
    assert_eq!(chk.rule_ref.name, "repo::auth");
    let ref_at = src.find("repo::auth").expect("ref in source");
    assert_eq!(chk.rule_ref.span.start, ref_at);
    assert_eq!(chk.rule_ref.span.end, ref_at + "repo::auth".len());
}

#[test]
fn check_ref_double_colon_rejects_whitespace_at_each_boundary() {
    for (src, message, token) in [
        (
            "check repo ::auth | confirm=skip",
            "expected a contiguous '::' separator",
            ":",
        ),
        (
            "check repo: :auth | confirm=skip",
            "expected a second ':' in '::'",
            ":",
        ),
        (
            "check repo:: auth | confirm=skip",
            "expected a path segment after '::'",
            "auth",
        ),
    ] {
        let diag = err(src);
        assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken, "{src}");
        assert_eq!(diag.message, message, "{src}");
        let start = match message {
            "expected a contiguous '::' separator" => src.find("::").expect("separator"),
            "expected a second ':' in '::'" => src.find(":auth").expect("second colon"),
            _ => src.find(" auth").expect("spaced segment") + " ".len(),
        };
        assert_eq!(diag.span.start, start, "{src}");
        assert_eq!(diag.span.end, start + token.len(), "{src}");
    }
}

#[test]
fn check_ref_path_separators_reject_gaps_and_comments() {
    for (src, message, token) in [
        (
            "check repo .auth | confirm=skip",
            "expected a contiguous '.' separator",
            ".",
        ),
        (
            "check repo. auth | confirm=skip",
            "expected a path segment after '.'",
            "auth",
        ),
        (
            "check repo#note\n.auth | confirm=skip",
            "expected a contiguous '.' separator",
            ".",
        ),
        (
            "check repo - auth | confirm=skip",
            "expected a contiguous path separator",
            "-",
        ),
        (
            "check repo- auth | confirm=skip",
            "expected a path segment after separator",
            "auth",
        ),
        (
            "check repo / auth | confirm=skip",
            "expected a contiguous path separator",
            "/",
        ),
        (
            "check repo/ auth | confirm=skip",
            "expected a path segment after separator",
            "auth",
        ),
    ] {
        let diagnostic = err(src);
        assert_eq!(diagnostic.code, ParseErrorCodeV1::UnexpectedToken, "{src}");
        assert_eq!(diagnostic.message, message, "{src}");
        let start = src.find(token).expect("ref fragment in source");
        assert_eq!(diagnostic.span.start, start, "{src}");
        assert_eq!(diagnostic.span.end, start + token.len(), "{src}");
    }
    for name in ["repo.auth", "repo-auth", "repo/auth"] {
        let src = format!("check {name} | confirm=skip");
        let ProgramItemHirV1::Legality(check) = program(&src) else {
            panic!("check source must parse to a legality item");
        };
        assert_eq!(check.rule_ref.name, name);
    }
}

#[test]
fn check_ref_single_colon_rejects_at_second_colon_expectation() {
    // REF-01: a single `:` is not a lenient `::`. The diagnostic points at the
    // token immediately following the first colon with the stable expectation.
    let src = "check repo:auth | confirm=skip";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(diag.message, "expected a second ':' in '::'");
    let ident_at = src.find(":auth").expect("single-colon input") + 1;
    assert_eq!(diag.span.start, ident_at);
    assert_eq!(diag.span.end, ident_at + "auth".len());
}

#[test]
fn check_ref_missing_segment_after_double_colon_rejects() {
    // REF-01: `::` with no following segment rejects at the missing segment.
    let src = "check repo:: | confirm=skip";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(diag.message, "expected a path segment after '::'");
    let pipe_at = src.find('|').expect("pipe input");
    assert_eq!(diag.span.start, pipe_at);
    assert_eq!(diag.span.end, pipe_at + 1);
}

// ---- EXPLAIN ---------------------------------------------------------------

#[test]
fn explain_parses_to_presentation() {
    let src = "explain rule.sqli | evidence=trace | truth=confirm";
    let item = program(src);
    let pres = match item {
        ProgramItemHirV1::Presentation(p) => p,
        other => panic!("expected Presentation, got {other:?}"),
    };
    match pres.kind {
        PresentationKindV1::Explain {
            target_ref,
            evidence,
            truth,
        } => {
            assert_eq!(target_ref.name, "rule.sqli");
            assert_eq!(evidence, EvidenceModeV1::Trace);
            assert_eq!(truth, TruthModeV1::Confirm);
        }
        other => panic!("expected Explain, got {other:?}"),
    }
}

// ---- SHOW ------------------------------------------------------------------

#[test]
fn show_parses_to_presentation() {
    let item = program("show capabilities");
    let pres = match item {
        ProgramItemHirV1::Presentation(p) => p,
        other => panic!("expected Presentation, got {other:?}"),
    };
    match pres.kind {
        PresentationKindV1::Show { category } => {
            assert_eq!(category, ShowCategoryV1::Capabilities)
        }
        other => panic!("expected Show, got {other:?}"),
    }
}

// ---- DESCRIBE --------------------------------------------------------------

#[test]
fn describe_parses_to_presentation() {
    let item = program("describe rule::owasp.sqli");
    let pres = match item {
        ProgramItemHirV1::Presentation(p) => p,
        other => panic!("expected Presentation, got {other:?}"),
    };
    match pres.kind {
        PresentationKindV1::Describe { kind, path } => {
            assert_eq!(kind, "rule");
            assert_eq!(path.name, "owasp.sqli");
        }
        other => panic!("expected Describe, got {other:?}"),
    }
}

// ---- SAMPLE ----------------------------------------------------------------

#[test]
fn sample_parses_to_presentation() {
    let src = "sample query=\"callgraph\" | mode=semantic | limit=5 | scope=repo";
    let item = program(src);
    let pres = match item {
        ProgramItemHirV1::Presentation(p) => p,
        other => panic!("expected Presentation, got {other:?}"),
    };
    match pres.kind {
        PresentationKindV1::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            assert_eq!(query_text, "callgraph");
            assert_eq!(mode, SampleModeV1::Semantic);
            assert_eq!(limit, 5);
            assert_eq!(scope.as_ref().unwrap().name, "repo");
        }
        other => panic!("expected Sample, got {other:?}"),
    }
}

#[test]
fn sample_zero_limit_rejects() {
    let diag = err("sample query=\"x\" | mode=lexical | limit=0");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, "SAMPLE limit must be > 0");
    assert_eq!(diag.span, SpanV1::new(40, 41));
}

#[test]
fn sample_empty_query_rejects() {
    let diag = err("sample query=\"\" | limit=1");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, "SAMPLE query_text must not be empty");
    assert_eq!(diag.span, SpanV1::new(13, 15));
}

// ---- COMPARE ---------------------------------------------------------------

#[test]
fn compare_parses_to_presentation() {
    let src = "compare baseline=run.a | candidate=run.b | metric=semantic";
    let item = program(src);
    let pres = match item {
        ProgramItemHirV1::Presentation(p) => p,
        other => panic!("expected Presentation, got {other:?}"),
    };
    match pres.kind {
        PresentationKindV1::Compare {
            baseline_ref,
            candidate_ref,
            metric,
        } => {
            assert_eq!(baseline_ref.name, "run.a");
            assert_eq!(candidate_ref.name, "run.b");
            assert_eq!(metric, CompareMetricV1::Semantic);
        }
        other => panic!("expected Compare, got {other:?}"),
    }
}

#[test]
fn compare_missing_candidate_rejects() {
    let diag = err("compare baseline=run.a | metric=exact");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.span, SpanV1::new(0, 7));
    assert!(diag.message.contains("candidate"), "msg: {}", diag.message);
}

// ---- EXPORT ----------------------------------------------------------------

#[test]
fn export_parses_to_presentation() {
    let src = "export source=result.latest | kind=sarif | profile=strict";
    let item = program(src);
    let pres = match item {
        ProgramItemHirV1::Presentation(p) => p,
        other => panic!("expected Presentation, got {other:?}"),
    };
    match pres.kind {
        PresentationKindV1::Export {
            source_ref,
            format,
            profile,
        } => {
            assert_eq!(source_ref.name, "result.latest");
            assert_eq!(format, ExportFormatV1::Sarif);
            assert_eq!(profile.as_ref().unwrap().name, "strict");
        }
        other => panic!("expected Export, got {other:?}"),
    }
}

// ---- VIEW / MODEL / PACK / PROFILE / GATE / RULE (catalog decls) ------------

#[test]
fn view_query_ref_parses_to_catalog_decl() {
    let item = program("view catalog.users | query-ref=queries.catalog.users");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    assert_eq!(decl.name, "catalog.users");
    match decl.payload {
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(r)) => {
            assert_eq!(r.name, "queries.catalog.users")
        }
        other => panic!("expected View QueryRef, got {other:?}"),
    }
}

#[test]
fn view_inline_select_parses_to_catalog_decl() {
    // VIEW-01: the inline query body must survive structurally — root filter,
    // scan/source identity, alias, and the `u.active` predicate path. Matching
    // `Inline(_)` alone would let a parser that drops the `where` stage pass.
    let src = "view active | select=from users u | where u.active";
    let item = program(src);
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    let query = match decl.payload {
        CatalogPayloadV1::View(ViewBodyV1::Inline(query)) => *query,
        other => panic!("expected View Inline, got {other:?}"),
    };
    let filter = match &query.body.kind {
        RelationExprKindV1::Filter(filter) => filter,
        other => panic!("expected root Filter, got {other:?}"),
    };
    let source = match &filter.input.kind {
        RelationExprKindV1::Source(source) => source,
        other => panic!("expected Source input, got {other:?}"),
    };
    assert_eq!(source.name, "users");
    assert_eq!(
        source.alias.as_ref().expect("alias u").name,
        "u",
        "binding alias must survive"
    );
    let predicate = match &filter.predicate.kind {
        BoolExprKindV1::Value(value) => value,
        other => panic!("expected Value predicate, got {other:?}"),
    };
    let path = match &predicate.kind {
        ValueExprKindV1::Path(path) => path,
        other => panic!("expected Path predicate, got {other:?}"),
    };
    let base = match &path.base.kind {
        ValueExprKindV1::Symbol(symbol) => symbol,
        other => panic!("expected Symbol base, got {other:?}"),
    };
    assert_eq!(base.name, "u");
    let segments = path.segments.as_slice();
    assert_eq!(segments.len(), 1, "exactly one path segment");
    assert_eq!(segments[0].name, "active");
    // span proof: the predicate covers `u.active` in the source text.
    let pred_at = src.find("u.active").expect("predicate in source");
    assert_eq!(predicate.span.start, pred_at);
    assert_eq!(predicate.span.end, pred_at + "u.active".len());
}

#[test]
fn model_parses_to_catalog_decl() {
    let item = program("model embeddings.code | provider=openai | source=models.code");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    assert_eq!(decl.name, "embeddings.code");
    match decl.payload {
        CatalogPayloadV1::Model { provider, source } => {
            assert_eq!(provider, "openai");
            assert_eq!(source.name, "models.code");
        }
        other => panic!("expected Model, got {other:?}"),
    }
}

#[test]
fn pack_parses_to_catalog_decl() {
    let item = program("pack secure-pack | source=registry.security | activation=workspace");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    assert_eq!(decl.name, "secure-pack");
    match decl.payload {
        CatalogPayloadV1::Pack { source, activation } => {
            assert_eq!(source.name, "registry.security");
            assert_eq!(activation, ActivationV1::Workspace);
        }
        other => panic!("expected Pack, got {other:?}"),
    }
}

#[test]
fn profile_parses_to_catalog_decl() {
    let item =
        program("profile strict | capabilities=graph.read,graph.trace | admission=certified-only");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    assert_eq!(decl.name, "strict");
    match decl.payload {
        CatalogPayloadV1::Profile {
            capabilities,
            admission,
        } => {
            // Full list, not just arity and the first element: a dropped or
            // reordered capability must fail this rail.
            let names: Vec<&str> = capabilities.iter().map(|r| r.name.as_str()).collect();
            assert_eq!(names, ["graph.read", "graph.trace"]);
            assert_eq!(admission, AdmissionV1::CertifiedOnly);
        }
        other => panic!("expected Profile, got {other:?}"),
    }
}

// DEDUPE-01: `profile_missing_capabilities_rejects` removed. The
// `v3_family_edges::profile_admission_and_clause_reject` rail asserts the same
// input with the exact message; the weaker `contains` oracle added executions
// without an independent oracle.

#[test]
fn gate_parses_to_catalog_decl() {
    let item = program("gate release | checks=owasp.sql,authz.path | binding=all");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    assert_eq!(decl.name, "release");
    match decl.payload {
        CatalogPayloadV1::Gate { checks, binding } => {
            // Full ordered list; arity alone would let a dropped check-name pass.
            let names: Vec<&str> = checks.iter().map(|r| r.name.as_str()).collect();
            assert_eq!(names, ["owasp.sql", "authz.path"]);
            assert_eq!(binding, GateBindingV1::All);
        }
        other => panic!("expected Gate, got {other:?}"),
    }
}

// DEDUPE-01: `gate_missing_checks_rejects` removed. The
// `v3_family_edges::gate_binding_and_clause_reject` rail asserts the same input
// with the exact message; the weaker `contains` oracle added executions without
// an independent oracle.

#[test]
fn rule_binding_parses_to_catalog_decl() {
    // the `ruledef` keyword binds a catalog RULE to a check ref. (`rule` keeps
    // its existing horn-rule meaning; see DECISION note in parser.rs.)
    let item = program("ruledef shell.exec | check-ref=checks.shell-exec");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    assert_eq!(decl.name, "shell.exec");
    match decl.payload {
        CatalogPayloadV1::RuleBinding { check_ref } => {
            assert_eq!(check_ref.name, "checks.shell-exec")
        }
        other => panic!("expected RuleBinding, got {other:?}"),
    }
}

// ---- META ------------------------------------------------------------------

#[test]
fn use_pack_parses_to_meta() {
    let src = "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock";
    let item = program(src);
    let meta = match item {
        ProgramItemHirV1::Meta(m) => m,
        other => panic!("expected Meta, got {other:?}"),
    };
    match meta.kind {
        MetaKindV1::UsePack(use_pack) => {
            assert_eq!(use_pack.pack_id, "secure-pack");
            assert_eq!(use_pack.activation, UsePackActivationV1::Session);
            assert_eq!(use_pack.lockfile, "packs.secure-pack.lock");
        }
        other => panic!("expected UsePack, got {other:?}"),
    }
}

#[test]
fn use_pack_without_lockfile_rejects() {
    let diag = err("use pack secure-pack | activation=session");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(diag.message.contains("lockfile"), "msg: {}", diag.message);
}

#[test]
fn require_certified_parses_to_meta() {
    let item = program("require certified");
    let meta = match item {
        ProgramItemHirV1::Meta(m) => m,
        other => panic!("expected Meta, got {other:?}"),
    };
    assert_eq!(meta.kind, MetaKindV1::RequireCertified);
}

// ---- COMPOSITION -----------------------------------------------------------

#[test]
fn composition_let_with_parses() {
    let src = "let suspects = sample query=\"auth\" | mode=semantic | limit=3 \
               find taint api.entry -> db.write with suspects";
    let item = program(src);
    let comp = match item {
        ProgramItemHirV1::Composition(c) => c,
        other => panic!("expected Composition, got {other:?}"),
    };
    assert_eq!(comp.bindings.len(), 1);
    assert_eq!(comp.bindings[0].name, "suspects");
    let CompositionProducerV1::Sample(sample) = &comp.bindings[0].producer else {
        panic!(
            "binding must be a SAMPLE producer, got {:?}",
            comp.bindings[0].producer
        );
    };
    // The SAMPLE payload is part of the producer oracle: a producer that loses
    // its query text, mode, or limit must fail here, not just a non-SAMPLE one.
    assert_eq!(sample.query_text, "auth");
    assert_eq!(sample.mode, SampleModeV1::Semantic);
    assert_eq!(sample.limit, 3);
    assert_eq!(comp.with_name, "suspects");
    let CompositionBodyV1::GraphQuery(body) = &comp.body else {
        panic!("body must be a FIND graph query, got {:?}", comp.body);
    };
    assert_eq!(body.kind, GraphQueryKindV1::Taint);
    assert_eq!(body.endpoints.source.as_ref().unwrap().name, "api.entry");
    assert_eq!(body.endpoints.sink.as_ref().unwrap().name, "db.write");
}

#[test]
fn composition_unconsumed_let_rejects() {
    // `with` names a binding that was never declared -> fail closed.
    let src = "let a = from users u \
               find taint api.entry -> db.write with b";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    // Exact message: `contains("with") || contains("binding")` would accept a
    // wrong branch (for example the `with` keyword-missing reject).
    assert_eq!(
        diag.message, "'with b' names an undefined composition binding",
        "msg: {}",
        diag.message
    );
    let with_at = src.find("with b").expect("with clause in source") + "with ".len();
    assert_eq!(diag.span.start, with_at);
    assert_eq!(diag.span.end, with_at + "b".len());
}

#[test]
fn composition_duplicate_binding_rejects() {
    let src = "let a = sample query=\"x\" | mode=lexical | limit=1 \
               let a = sample query=\"y\" | mode=lexical | limit=1 \
               find taint p -> q with a";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    let duplicate_name = src.rfind("let a").expect("second authored binding") + "let ".len();
    assert_eq!(diag.span, SpanV1::new(duplicate_name, duplicate_name + 1));
    assert!(diag.message.contains("duplicate"), "msg: {}", diag.message);
}

// ---- D28 duplicate-clause fail-closed across the clause-bearing families ---
//
// SURFACE-DUPLICATE-CLAUSES-D28: a repeated clause key in CHECK / EXPLAIN /
// SAMPLE / COMPARE / EXPORT / MODEL / PACK / PROFILE / GATE must fail closed
// with a typed UnsupportedConstruct that names the duplicate key, rather than
// silently overwriting (last-wins). These mirror the FIND duplicate rails.

#[test]
fn check_duplicate_confirm_clause_rejects() {
    // The reported repro: confirm=require then confirm=skip must NOT last-wins.
    let diag = err("check rule.x | confirm=require | confirm=skip");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate CHECK clause 'confirm'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(33, 40));
}

#[test]
fn explain_duplicate_evidence_clause_rejects() {
    let diag = err("explain rule.x | evidence=trace | evidence=auto");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate EXPLAIN clause 'evidence'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(34, 42));
}

#[test]
fn sample_duplicate_mode_clause_rejects() {
    let diag = err("sample query=\"x\" | limit=1 | mode=lexical | mode=semantic");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate SAMPLE clause 'mode'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(44, 48));
}

#[test]
fn compare_duplicate_metric_clause_rejects() {
    let diag = err("compare baseline=a | candidate=b | metric=exact | metric=semantic");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate COMPARE clause 'metric'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(50, 56));
}

#[test]
fn export_duplicate_kind_clause_rejects() {
    let diag = err("export source=a | kind=json | kind=csv");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate EXPORT clause 'kind'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(30, 34));
}

#[test]
fn model_duplicate_provider_clause_rejects() {
    let diag = err("model m | provider=p1 | source=s | provider=p2");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate MODEL clause 'provider'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(35, 43));
}

#[test]
fn pack_duplicate_source_clause_rejects() {
    let diag = err("pack p | source=s1 | activation=session | source=s2");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate PACK clause 'source'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(42, 48));
}

#[test]
fn profile_duplicate_admission_clause_rejects() {
    let diag = err("profile p | admission=allowlist | admission=certified-only");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate PROFILE clause 'admission'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(34, 43));
}

#[test]
fn gate_duplicate_binding_clause_rejects() {
    let diag = err("gate g | binding=all | binding=any");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate GATE clause 'binding'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(23, 30));
}

#[test]
fn use_pack_duplicate_activation_clause_rejects() {
    // USE PACK is also clause-bearing; a duplicate activation must fail closed.
    let diag = err("use pack p | activation=session | lockfile=p.lock | activation=workspace");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate USE PACK clause 'activation'",
        "msg: {}",
        diag.message
    );
    assert_eq!(diag.span, SpanV1::new(52, 62));
}

#[test]
fn find_duplicate_depth_clause_rejects() {
    // D28: conflicts fail closed. A repeated FIND clause key is rejected with a
    // typed UnsupportedConstruct rather than silently last-wins overwriting.
    let src = "find taint a -> b | depth=2 | depth=5";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(
        diag.message, "duplicate FIND clause 'depth'",
        "msg: {}",
        diag.message
    );
    // points at the second `depth` key, at bytes 30..35.
    assert_eq!(diag.span, SpanV1::new(30, 35));
}

#[test]
fn find_duplicate_endpoint_clause_rejects() {
    // Repeated endpoint clause (anchor) also fails closed.
    let diag = err("find slice | anchor=a | anchor=b");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, "duplicate FIND clause 'anchor'");
    assert_eq!(diag.span, SpanV1::new(24, 30));
}

#[test]
fn find_duplicate_list_clause_rejects() {
    // Repeated list-valued clause (via) also fails closed.
    let diag = err("find taint a -> b | via=m1 | via=m2");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, "duplicate FIND clause 'via'");
    assert_eq!(diag.span, SpanV1::new(29, 32));
}

#[test]
fn find_empty_list_clauses_reject() {
    for (src, key) in [
        ("find taint a -> b | via=", "via"),
        ("find taint a -> b | through=", "through"),
        ("find taint a -> b | sanitizers=", "sanitizers"),
    ] {
        assert_empty_find_list_rejects(src, key);
    }
}

#[test]
fn find_empty_list_clauses_before_next_clause_reject() {
    for (src, key) in [
        ("find taint a -> b | via= | scope=x", "via"),
        ("find taint a -> b | through= | scope=x", "through"),
        ("find taint a -> b | sanitizers= | scope=x", "sanitizers"),
    ] {
        assert_empty_find_list_rejects(src, key);
    }
}

#[test]
fn find_depth_zero_rejects() {
    // PUNCH-LIST find_depth_zero_boundary: a `depth=0` bound is meaningless
    // (a zero-hop graph walk). The parser must reject it fail-closed with a
    // typed UnsupportedConstruct, mirroring the SAMPLE `limit > 0` policy,
    // rather than minting a GraphQuery carrying depth=Some(0).
    let src = "find taint a -> b | depth=0";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("depth") && diag.message.contains("> 0"),
        "msg: {}",
        diag.message
    );
    // points at the `0` literal at byte 26.
    assert_eq!(diag.span.start, 26);
}

#[test]
fn find_depth_one_is_accepted() {
    // Boundary above zero stays legal: depth=1 is the smallest meaningful bound.
    let item = program("find taint a -> b | depth=1");
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(gq.constraints.depth, Some(1));
}

#[test]
fn find_direction_unknown_keyword_rejects() {
    // PUNCH-LIST direction_keyword_validation: the FIND `direction` clause must
    // not silently mint an unvalidated keyword. An unknown direction value must
    // reject fail-closed with a typed UnsupportedConstruct pointed at the bad
    // keyword span, mirroring the confirm/emit/mode open-tag validation policy,
    // rather than building direction=Some(GraphDirectionV1{ keyword: <any> }).
    let src = "find taint a -> b | direction=invalid_keyword";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("direction") && diag.message.contains("invalid_keyword"),
        "msg: {}",
        diag.message
    );
    // span covers the offending keyword exactly: `invalid_keyword` at byte 29.
    let start = src
        .find("invalid_keyword")
        .expect("keyword present in input");
    assert_eq!(diag.span.start, start);
    assert_eq!(diag.span.end, start + "invalid_keyword".len());
}

#[test]
fn find_direction_known_keywords_parse() {
    // Each well-attested traversal direction stays legal and is carried through
    // verbatim into the open GraphDirectionV1 tag.
    for kw in ["forward", "backward", "both", "undirected"] {
        let src = format!("find taint a -> b | direction={kw}");
        let item = program(&src);
        let gq = match item {
            ProgramItemHirV1::GraphQuery(g) => g,
            other => panic!("expected GraphQuery, got {other:?}"),
        };
        assert_eq!(
            gq.constraints
                .direction
                .as_ref()
                .map(|d| d.keyword.as_str()),
            Some(kw),
            "direction {kw} must parse"
        );
    }
}

#[test]
fn find_summary_empty_facets_rejects() {
    // PUNCH-LIST sample_facets_empty_list: a summary facet list given an EMPTY
    // list (the `facets=` key with no following facet) must reject fail-closed
    // with a typed UnsupportedConstruct, not silently mint an empty facet vec.
    let src = "find summary | facets=";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.span, SpanV1::new(15, 21));
    assert!(diag.message.contains("facet"), "msg: {}", diag.message);
}

#[test]
fn find_summary_facets_list_parses() {
    // A non-empty facet list is accepted and captured verbatim.
    let item = program("find summary | facets=calls,reads,writes");
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(
        gq.constraints.summary_facets,
        vec![
            "calls".to_string(),
            "reads".to_string(),
            "writes".to_string()
        ]
    );
}

#[test]
fn find_via_list_preserves_written_order() {
    let item = program("find taint a -> b | via=m1,m2,m3");
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(gq.constraints.via.len(), 3);
    assert_eq!(gq.constraints.via[0].name, "m1");
    assert_eq!(gq.constraints.via[1].name, "m2");
    assert_eq!(gq.constraints.via[2].name, "m3");
}

#[test]
fn find_slice_seed_endpoints_are_not_dropped() {
    // PUNCH-LIST find_conflicting_endpoint_modifiers: the parser is STRUCTURAL
    // only (D13) and must NOT silently drop a `source -> sink` seed on a slice
    // (whose endpoint legality is rejected downstream at classify). The seed
    // endpoints must survive parse so the downstream conflict reject can see
    // them; the parser does not pre-empt or discard them.
    let item = program("find slice api.entry -> db.write");
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(gq.kind, GraphQueryKindV1::Slice);
    assert_eq!(gq.endpoints.source.as_ref().unwrap().name, "api.entry");
    assert_eq!(gq.endpoints.sink.as_ref().unwrap().name, "db.write");
}

// ---- COMPOSITION-PRODUCER-BODY-MISMATCH-VALIDATION -------------------------
//
// The composition grammar admits a strict (producer, body) family matrix:
//   producer ::= from | sample | find        (parse_producer)
//   body     ::= from | find | check | explain (parse_body)
// A family outside its column rejects fail-closed at PARSE time with a typed
// diagnostic, so the boundary cannot silently widen if a new family parser is
// added later. (The reported repro `find` producer + `check` body is a VALID
// pairing per this matrix; CHECK is deliberately an admitted body. These rails
// pin both the accepted cells AND the rejected cells so the boundary is fixed.)

#[test]
fn composition_find_producer_check_body_is_accepted() {
    // The reported repro: a FIND producer with a CHECK body is a LEGAL pairing
    // (CHECK is an admitted composition body). It must parse to a Composition
    // whose producer is the graph query and whose body is the legality check.
    let src = "let q = find taint a -> b | depth=1 check rule.x | confirm=skip with q";
    let item = program(src);
    let comp = match item {
        ProgramItemHirV1::Composition(c) => c,
        other => panic!("expected Composition, got {other:?}"),
    };
    assert_eq!(comp.bindings.len(), 1);
    let CompositionProducerV1::GraphQuery(producer) = &comp.bindings[0].producer else {
        panic!(
            "producer must be a FIND graph query, got {:?}",
            comp.bindings[0].producer
        );
    };
    assert_eq!(producer.kind, GraphQueryKindV1::Taint);
    assert_eq!(producer.constraints.depth, Some(1));
    let CompositionBodyV1::Legality(check) = &comp.body else {
        panic!("body must be a CHECK legality item, got {:?}", comp.body);
    };
    assert_eq!(check.rule_ref.name, "rule.x");
    assert_eq!(check.confirm, ConfirmModeV1::Skip);
    assert_eq!(comp.with_name, "q");
}

fn composition_body(src: &str) -> CompositionBodyV1 {
    match program(src) {
        ProgramItemHirV1::Composition(c) => c.body,
        other => panic!("expected Composition for {src:?}, got {other:?}"),
    }
}

#[test]
fn composition_admitted_body_families_parse() {
    // Every body column member parses: from / find / check / explain.
    let prefix = "let q = sample query=\"x\" | mode=lexical | limit=1 ";
    assert!(matches!(
        composition_body(&format!("{prefix}from users u with q")),
        CompositionBodyV1::Query(_)
    ));
    assert!(matches!(
        composition_body(&format!("{prefix}find taint a -> b with q")),
        CompositionBodyV1::GraphQuery(_)
    ));
    assert!(matches!(
        composition_body(&format!("{prefix}check rule.x | confirm=skip with q")),
        CompositionBodyV1::Legality(_)
    ));
    assert!(matches!(
        composition_body(&format!("{prefix}explain rule.x with q")),
        CompositionBodyV1::Explain(_)
    ));
}

#[test]
fn composition_non_body_family_rejects() {
    // A family OUTSIDE the body column (e.g. show / compare / sample / export /
    // view) must reject fail-closed where the body is expected, rather than the
    // boundary silently widening. `with` never appears, so the failure is the
    // body family rejection, not the binding hygiene guard.
    for src in [
        "let q = sample query=\"x\" | mode=lexical | limit=1 show capabilities with q",
        "let q = sample query=\"x\" | mode=lexical | limit=1 compare baseline=a | candidate=b with q",
        "let q = sample query=\"x\" | mode=lexical | limit=1 export source=s | kind=json with q",
        "let q = sample query=\"x\" | mode=lexical | limit=1 view v | query-ref=r with q",
    ] {
        let diag = err(src);
        assert_eq!(
            diag.code,
            ParseErrorCodeV1::UnexpectedToken,
            "body family must reject for {src:?}"
        );
    }
}

#[test]
fn composition_explain_producer_rejects() {
    // PRODUCER-01: EXPLAIN is an admitted BODY but NOT an admitted PRODUCER.
    // A valid first binding forces composition recognition, so the invalid
    // second `let bad = explain ...` reaches `parse_producer` itself (the old
    // source was rejected by composition lookahead and never pinned the
    // producer-column allowlist it named).
    let prefix = "let q = sample query=\"x\" | mode=lexical | limit=1 ";
    let src = format!("{prefix}let bad = explain rule.x find taint a -> b with q");
    let diag = err(&src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(
        diag.message,
        "expected a composition producer (from / sample / find)"
    );
    let explain_at = src.find("explain rule.x").expect("explain producer");
    assert_eq!(diag.span.start, explain_at);
    assert_eq!(diag.span.end, explain_at + "explain".len());
}

#[test]
fn composition_second_producer_from_parses() {
    // PRODUCER-01 positive neighbor: a valid second `from` producer parses, so
    // the reject above pins the producer allowlist rather than second bindings.
    let prefix = "let q = sample query=\"x\" | mode=lexical | limit=1 ";
    let src = format!("{prefix}let bad = from users u find taint a -> b with q");
    let item = program(&src);
    let comp = match item {
        ProgramItemHirV1::Composition(c) => c,
        other => panic!("expected Composition, got {other:?}"),
    };
    assert_eq!(comp.bindings.len(), 2);
    assert_eq!(comp.bindings[0].name, "q");
    assert!(
        matches!(&comp.bindings[0].producer, CompositionProducerV1::Sample(_)),
        "first binding must stay the SAMPLE producer: {:?}",
        comp.bindings[0].producer
    );
    assert_eq!(comp.bindings[1].name, "bad");
    assert!(
        matches!(&comp.bindings[1].producer, CompositionProducerV1::Query(_)),
        "second binding must be the `from` query producer: {:?}",
        comp.bindings[1].producer
    );
}

#[test]
fn composition_binding_names_may_shadow_family_keywords() {
    // A composition binding name may equal a later family keyword. The parser
    // must still preserve the composition structure instead of mistokenizing
    // the body or rejecting on keyword shadowing.
    let src = "let find = sample query=\"auth\" | mode=semantic | limit=3 find taint api.entry -> db.write with find";
    let item = program(src);
    let comp = match item {
        ProgramItemHirV1::Composition(c) => c,
        other => panic!("expected Composition, got {other:?}"),
    };
    assert_eq!(comp.bindings.len(), 1);
    assert_eq!(comp.bindings[0].name, "find");
    assert!(matches!(
        &comp.bindings[0].producer,
        CompositionProducerV1::Sample(_)
    ));
    assert!(matches!(&comp.body, CompositionBodyV1::GraphQuery(_)));
    assert_eq!(comp.with_name, "find");
}

#[test]
fn find_seed_and_clause_endpoints_coexist_unvalidated() {
    // SEED-CLAUSE-ENDPOINT-CONFLICT: the parser is STRUCTURAL only (D13) and
    // admits seed endpoints (source -> sink) and clause-based endpoints
    // (anchor=, reference=, pointee=, callable=) to coexist in a single FIND
    // without validation or rejection. This is ambiguous syntax that should be
    // rejected downstream during classification (ticket 03), not here.

    // Test 1: find slice api -> db | anchor=x
    // seed (source/sink) + clause (anchor) coexist without validation
    let src1 = "find slice api -> db | anchor=x";
    let item1 = program(src1);
    let gq1 = match item1 {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    // Both seed AND clause-based endpoints are present
    assert!(
        gq1.endpoints.source.is_some(),
        "seed source should be present"
    );
    assert!(gq1.endpoints.sink.is_some(), "seed sink should be present");
    assert!(
        gq1.endpoints.anchor.is_some(),
        "clause anchor should be present"
    );
    assert_eq!(
        gq1.endpoints.anchor.as_ref().map(|r| r.name.as_str()),
        Some("x"),
        "clause anchor must retain its authored ref"
    );

    // Test 2: find points_to a -> b | reference=p | pointee=q
    // seed (source/sink) + clause (reference/pointee) coexist
    let src2 = "find points_to a -> b | reference=p | pointee=q";
    let item2 = program(src2);
    let gq2 = match item2 {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert!(
        gq2.endpoints.source.is_some(),
        "seed source should be present"
    );
    assert!(gq2.endpoints.sink.is_some(), "seed sink should be present");
    assert!(
        gq2.endpoints.reference.is_some(),
        "clause reference should be present"
    );
    assert!(
        gq2.endpoints.pointee.is_some(),
        "clause pointee should be present"
    );
    assert_eq!(
        gq2.endpoints.reference.as_ref().map(|r| r.name.as_str()),
        Some("p"),
        "clause reference must retain its authored ref"
    );
    assert_eq!(
        gq2.endpoints.pointee.as_ref().map(|r| r.name.as_str()),
        Some("q"),
        "clause pointee must retain its authored ref"
    );
}
