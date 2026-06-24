//! v3 family hellgate: the 18 root families PARSE -> lower to the correct
//! `hir::ProgramItemHirV1` variant with exact spans, AND fail closed on
//! malformed/illegal input with the ACTUAL typed diagnostic code + span.
//!
//! These rails are written first (red) and drive the family grammar to green.
//! Binding/typing/legality stay downstream (D13): the parser is STRUCTURAL only,
//! so these rails assert structure + spans + fail-closed codes, never semantics.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::single_match,
    reason = "integration test rail: assertion panics, exhaustive/diagnostic match arms are intentional"
)]

use quanta_gqlang_v2_hir::{
    check_tree, ActivationV1, AdmissionV1, CatalogPayloadV1, CompareMetricV1, ConfirmModeV1,
    DetectionModeV1, EmitModeV1, EvidenceModeV1, ExportFormatV1, GateBindingV1, GraphQueryKindV1,
    HirBuilderV1, HirRootRefV1, MetaKindV1, PresentationKindV1, ProgramItemHirV1, SampleModeV1,
    ShowCategoryV1, TruthModeV1, ViewBodyV1,
};
use quanta_gqlang_v2_surface::{parse_program, ParseErrorCodeV1, SurfaceItemV1};

fn program(src: &str) -> ProgramItemHirV1 {
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("must parse");
    let item = match parsed.item {
        SurfaceItemV1::Program(p) => p,
        SurfaceItemV1::Query(q) => ProgramItemHirV1::from(q),
        SurfaceItemV1::Decl(d) => ProgramItemHirV1::from(d),
        SurfaceItemV1::Rule(r) => ProgramItemHirV1::from(r),
    };
    // every parsed family must pass the shared HIR integrity check.
    check_tree(HirRootRefV1::Item(&item), parsed.source_len).expect("tree integrity");
    item
}

fn err(src: &str) -> quanta_gqlang_v2_surface::ParseDiagnosticV1 {
    let builder = HirBuilderV1::new();
    parse_program(src, &builder).expect_err("must reject")
}

// ---- FIND ------------------------------------------------------------------

#[test]
fn find_taint_parses_to_graph_query() {
    let src = "find taint api.entry -> db.write | sanitizers=authz.clean | depth=3";
    let item = program(src);
    let gq = match item {
        ProgramItemHirV1::GraphQuery(g) => g,
        other => panic!("expected GraphQuery, got {other:?}"),
    };
    assert_eq!(gq.kind, GraphQueryKindV1::Taint);
    assert_eq!(gq.endpoints.source.as_ref().unwrap().name, "api.entry");
    assert_eq!(gq.endpoints.sink.as_ref().unwrap().name, "db.write");
    assert_eq!(gq.constraints.sanitizers[0].name, "authz.clean");
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

#[test]
fn find_missing_arrow_rejects() {
    // malformed FIND seed: no `->` between source and sink.
    let diag = err("find taint api.entry db.write");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}

#[test]
fn find_unknown_subkind_rejects() {
    let diag = err("find frobnicate a -> b");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    // points at the bad subkind token `frobnicate` at byte 5.
    assert_eq!(diag.span.start, 5);
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
fn check_missing_confirm_rejects() {
    // CHECK requires an explicit confirm clause: fail closed when absent.
    let diag = err("check rule.hybrid | scope=repo | emit=findings");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(diag.message.contains("confirm"), "msg: {}", diag.message);
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
    let item = program("view active | select=from users u | where u.active");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    match decl.payload {
        CatalogPayloadV1::View(ViewBodyV1::Inline(_)) => {}
        other => panic!("expected View Inline, got {other:?}"),
    }
}

#[test]
fn model_parses_to_catalog_decl() {
    let item = program("model embeddings.code | provider=openai | source=models.code");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
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
    match decl.payload {
        CatalogPayloadV1::Profile {
            capabilities,
            admission,
        } => {
            assert_eq!(capabilities.len(), 2);
            assert_eq!(capabilities[0].name, "graph.read");
            assert_eq!(admission, AdmissionV1::CertifiedOnly);
        }
        other => panic!("expected Profile, got {other:?}"),
    }
}

#[test]
fn profile_missing_capabilities_rejects() {
    // PROFILE must name at least one capability: an empty allowlist fails open
    // (admits everything), so reject fail-closed when 'capabilities' is omitted.
    let diag = err("profile p | admission=allowlist");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("capabilities"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn gate_parses_to_catalog_decl() {
    let item = program("gate release | checks=owasp.sql,authz.path | binding=all");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
    match decl.payload {
        CatalogPayloadV1::Gate { checks, binding } => {
            assert_eq!(checks.len(), 2);
            assert_eq!(binding, GateBindingV1::All);
        }
        other => panic!("expected Gate, got {other:?}"),
    }
}

#[test]
fn gate_missing_checks_rejects() {
    // GATE must name at least one check: an empty checks list binds over nothing
    // (vacuously passes), so reject fail-closed when 'checks' is omitted.
    let diag = err("gate g | binding=all");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(diag.message.contains("checks"), "msg: {}", diag.message);
}

#[test]
fn rule_binding_parses_to_catalog_decl() {
    // the `ruledef` keyword binds a catalog RULE to a check ref. (`rule` keeps
    // its existing horn-rule meaning; see DECISION note in parser.rs.)
    let item = program("ruledef shell.exec | check-ref=checks.shell-exec");
    let decl = match item {
        ProgramItemHirV1::CatalogDecl(d) => d,
        other => panic!("expected CatalogDecl, got {other:?}"),
    };
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
        MetaKindV1::UsePack {
            pack_id,
            activation,
            lockfile,
        } => {
            assert_eq!(pack_id, "secure-pack");
            assert_eq!(activation, ActivationV1::Session);
            assert_eq!(lockfile.as_deref(), Some("packs.secure-pack.lock"));
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
    assert!(matches!(
        *comp.bindings[0].producer,
        ProgramItemHirV1::Presentation(_)
    ));
    assert_eq!(comp.with_name, "suspects");
    assert!(matches!(*comp.body, ProgramItemHirV1::GraphQuery(_)));
}

#[test]
fn composition_unconsumed_let_rejects() {
    // `with` names a binding that was never declared -> fail closed.
    let src = "let a = from users u \
               find taint api.entry -> db.write with b";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("with") || diag.message.contains("binding"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn composition_duplicate_binding_rejects() {
    let src = "let a = sample query=\"x\" | mode=lexical | limit=1 \
               let a = sample query=\"y\" | mode=lexical | limit=1 \
               find taint p -> q with a";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
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
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("confirm"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn explain_duplicate_evidence_clause_rejects() {
    let diag = err("explain rule.x | evidence=trace | evidence=auto");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("evidence"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn sample_duplicate_mode_clause_rejects() {
    let diag = err("sample query=\"x\" | limit=1 | mode=lexical | mode=semantic");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("mode"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn compare_duplicate_metric_clause_rejects() {
    let diag = err("compare baseline=a | candidate=b | metric=exact | metric=semantic");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("metric"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn export_duplicate_kind_clause_rejects() {
    let diag = err("export source=a | kind=json | kind=csv");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("kind"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn model_duplicate_provider_clause_rejects() {
    let diag = err("model m | provider=p1 | source=s | provider=p2");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("provider"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn pack_duplicate_source_clause_rejects() {
    let diag = err("pack p | source=s1 | activation=session | source=s2");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("source"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn profile_duplicate_admission_clause_rejects() {
    let diag = err("profile p | admission=allowlist | admission=certified-only");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("admission"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn gate_duplicate_binding_clause_rejects() {
    let diag = err("gate g | binding=all | binding=any");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("binding"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn use_pack_duplicate_activation_clause_rejects() {
    // USE PACK is also clause-bearing; a duplicate activation must fail closed.
    let diag = err("use pack p | activation=session | lockfile=p.lock | activation=workspace");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("activation"),
        "msg: {}",
        diag.message
    );
}

#[test]
fn find_duplicate_depth_clause_rejects() {
    // D28: conflicts fail closed. A repeated FIND clause key is rejected with a
    // typed UnsupportedConstruct rather than silently last-wins overwriting.
    let src = "find taint a -> b | depth=2 | depth=5";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(
        diag.message.contains("duplicate") && diag.message.contains("depth"),
        "msg: {}",
        diag.message
    );
    // points at the second `depth` key, at byte 30.
    assert_eq!(diag.span.start, 30);
}

#[test]
fn find_duplicate_endpoint_clause_rejects() {
    // Repeated endpoint clause (anchor) also fails closed.
    let diag = err("find slice | anchor=a | anchor=b");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(diag.message.contains("duplicate"), "msg: {}", diag.message);
}

#[test]
fn find_duplicate_list_clause_rejects() {
    // Repeated list-valued clause (via) also fails closed.
    let diag = err("find taint a -> b | via=m1 | via=m2");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert!(diag.message.contains("duplicate"), "msg: {}", diag.message);
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
fn find_summary_empty_facets_rejects() {
    // PUNCH-LIST sample_facets_empty_list: a summary facet list given an EMPTY
    // list (the `facets=` key with no following facet) must reject fail-closed
    // with a typed UnsupportedConstruct, not silently mint an empty facet vec.
    let src = "find summary | facets=";
    let diag = err(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
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

#[test]
fn composition_binding_names_can_shadow_keywords() {
    // REPRO: let find = sample ... find ... with find
    // The binding name matches the family keyword `find`.
    // This should either PARSE (bug) or REJECT (correct).
    let src = "let find = sample query=\"auth\" | mode=semantic | limit=3 find taint api.entry -> db.write with find";
    let result = parse_program(src, &quanta_gqlang_v2_hir::HirBuilderV1::new());
    match result {
        Ok(_) => {
            // Parser accepted it - keyword shadowing is allowed
            println!("Parser accepted: let find = ... find ... with find");
        }
        Err(diag) => {
            // Parser rejected it
            println!("Parser rejected: {}", diag.message);
            println!("Code: {:?}", diag.code);
        }
    }
}
#[cfg(test)]
mod test_shadowing {
    use quanta_gqlang_v2_hir::HirBuilderV1;
    use quanta_gqlang_v2_surface::parse_program;

    #[test]
    fn test_keyword_shadowing() {
        let builder = HirBuilderV1::new();

        // Try to parse: let find = sample ... find ... with find
        let src = "let find = sample query=\"auth\" | mode=semantic | limit=3 find taint api.entry -> db.write with find";

        match parse_program(src, &builder) {
            Ok(result) => {
                eprintln!("PARSED SUCCESSFULLY (bug confirmed!)");
                eprintln!(
                    "Parsed item type: {}",
                    match result.item {
                        quanta_gqlang_v2_surface::SurfaceItemV1::Program(_) => "Program",
                        _ => "Other",
                    }
                );
            }
            Err(diag) => {
                eprintln!("REJECTED (correct!)");
                eprintln!("Message: {}", diag.message);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use quanta_gqlang_v2_hir::{check_tree, HirBuilderV1, HirRootRefV1};
    use quanta_gqlang_v2_surface::{parse_program, SurfaceItemV1};

    #[test]
    fn composition_with_keyword_shadowing_binding() {
        let builder = HirBuilderV1::new();

        // Test case: let find = sample ... find ... with find
        // The binding name `find` matches a family keyword
        let src = "let find = sample query=\"auth\" | mode=semantic | limit=3 find taint api.entry -> db.write with find";

        match parse_program(src, &builder) {
            Ok(parsed) => {
                eprintln!("✓ PARSER ACCEPTED the keyword-shadowing binding");

                // Verify tree integrity
                match check_tree(
                    HirRootRefV1::Item(&match &parsed.item {
                        SurfaceItemV1::Program(p) => p.clone(),
                        other => panic!("unexpected surface item type: {other:?}"),
                    }),
                    parsed.source_len,
                ) {
                    Ok(_) => eprintln!("✓ HIR tree integrity check passed"),
                    Err(e) => eprintln!("✗ HIR tree integrity check failed: {}", e),
                }

                match &parsed.item {
                    SurfaceItemV1::Program(program_item) => {
                        eprintln!(
                            "Program item type: {:?}",
                            std::mem::discriminant(program_item)
                        );
                    }
                    _ => {}
                }
            }
            Err(diag) => {
                eprintln!("✗ PARSER REJECTED: {}", diag.message);
            }
        }
    }
}
