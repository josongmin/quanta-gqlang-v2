//! The 18 v3 root-family grammars producing shared `hir::ProgramItemHirV1`.
//!
//! syntax -> hir: lowers every non-`from`/`let`/`rule` v3 surface family into the
//! unified item carrier. STRUCTURE ONLY (D13): binding, classification, witness
//! legality, catalog registration, and admission gating are downstream
//! (tickets 02/03/05/07/11). This module decides only that a family PARSES into
//! the correct variant with EXACT spans, and fails closed with a typed
//! [`ParseDiagnosticV1`] on a malformed/illegal surface.
//!
//! ## Surface policy (ticket 06 mandatory output)
//!
//! v3 keeps the v2 family VOCABULARY (subkinds, modes, clause keys) but renders
//! it in the v3 lowercase, pipe-delimited style that the `from | where` query
//! already uses, rather than the v2 uppercase-root `;;`-delimited surface. So:
//!
//! - families lead with a lowercase keyword (`find`, `check`, `explain`, ...);
//! - clauses are `| key=value` segments (not `;; key=value`);
//! - references are dotted/hyphenated paths reconstructed verbatim from source.
//!
//! This is a DELIBERATE clean v3 surface (D1/D10): the v2 uppercase roots stay
//! rejected by the no-silent-downgrade guard, and the legacy v2 parser remains
//! the v2 authority until the ticket 09 cutover.
//!
//! ## RULE keyword DECISION
//!
//! v2 has both a horn `RULE name : head if body` and a catalog `RULE name ;;
//! check-ref=...`. The v3 surface already binds the lowercase `rule` keyword to
//! the horn rule. To keep both reachable without an ambiguous lookahead, the
//! catalog rule-binding family uses the distinct keyword `ruledef`.

use crate::cursor::CursorV1;
use crate::diag::ParseDiagnosticV1;
use crate::lexer::TokenKindV1;
use crate::query::parse_query;
use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, CatalogPayloadV1, CompareMetricV1, CompositionBodyV1,
    CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1, EvidenceModeV1,
    ExplainHirV1, ExportFormatV1, GateBindingV1, GraphConstraintsV1, GraphDirectionV1,
    GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1, MetaKindV1,
    PresentationKindV1, ProgramItemHirV1, SampleHirV1, SampleModeV1, ShowCategoryV1, TruthModeV1,
    UsePackActivationV1, UsePackHirV1, ViewBodyV1,
};

/// Whether `word` is a leading keyword owned by a v3 family parser here.
pub(crate) fn is_family_keyword(word: &str) -> bool {
    matches!(
        word,
        "find"
            | "check"
            | "explain"
            | "show"
            | "describe"
            | "sample"
            | "compare"
            | "export"
            | "view"
            | "model"
            | "pack"
            | "profile"
            | "gate"
            | "ruledef"
            | "use"
            | "require"
    )
}

/// Parse a single non-composition family item by its leading keyword.
///
/// `source` is the original program text, threaded so multi-token references
/// (dotted/hyphenated paths) can be reconstructed verbatim by span slice.
pub(crate) fn parse_family(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    if cur.is_keyword("find") {
        return parse_find(cur, source);
    }
    if cur.is_keyword("check") {
        return parse_check(cur, source);
    }
    if cur.is_keyword("explain") {
        return parse_explain(cur, source);
    }
    if cur.is_keyword("show") {
        return parse_show(cur);
    }
    if cur.is_keyword("describe") {
        return parse_describe(cur, source);
    }
    if cur.is_keyword("sample") {
        return parse_sample(cur, source);
    }
    if cur.is_keyword("compare") {
        return parse_compare(cur, source);
    }
    if cur.is_keyword("export") {
        return parse_export(cur, source);
    }
    if cur.is_keyword("view") {
        return parse_view(cur, source);
    }
    if cur.is_keyword("model") {
        return parse_model(cur, source);
    }
    if cur.is_keyword("pack") {
        return parse_pack(cur, source);
    }
    if cur.is_keyword("profile") {
        return parse_profile(cur, source);
    }
    if cur.is_keyword("gate") {
        return parse_gate(cur, source);
    }
    if cur.is_keyword("ruledef") {
        return parse_ruledef(cur, source);
    }
    if cur.is_keyword("use") {
        return parse_use_pack(cur, source);
    }
    if cur.is_keyword("require") {
        return parse_require_certified(cur);
    }
    Err(cur.unexpected("a v3 family keyword"))
}

// ---- COMPOSITION -----------------------------------------------------------

/// Whether the cursor is at a composition: `let <name> = <producer> ...`, where
/// the producer is a SELECT (`from`), SAMPLE (`sample`), or FIND (`find`). A
/// plain `let x = <value>` decl never starts its body with those keywords, so
/// this lookahead disambiguates composition from a value declaration.
pub(crate) fn at_composition(cur: &CursorV1<'_>) -> bool {
    if !cur.is_keyword("let") {
        return false;
    }
    // let <name> = <producer-keyword>
    cur.is_keyword_ahead(3, "from")
        || cur.is_keyword_ahead(3, "sample")
        || cur.is_keyword_ahead(3, "find")
}

/// Parse a `[let <name> = <producer>]+ <body> with <name>` composition.
///
/// STRUCTURE plus the fail-closed binding hygiene the ticket calls out: an
/// unconsumed/undefined `with` binding and a duplicate `let` name reject here
/// (typed `UnsupportedConstruct`); producer/body kind legality stays downstream.
pub(crate) fn parse_composition(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let mut bindings = Vec::new();
    let start = cur.here_span().start;

    while cur.is_keyword("let") {
        let _let = cur.expect_keyword("let", "'let' to start a composition binding")?;
        let (name, name_span) = cur.expect_ident("a composition binding name")?;
        // duplicate-name reject (fail closed).
        if bindings
            .iter()
            .any(|b: &quanta_gqlang_v2_hir::CompositionBindingV1| b.name == name)
        {
            return Err(reject(
                cur,
                name_span,
                format!("duplicate composition binding '{name}'"),
            ));
        }
        cur.expect(
            &TokenKindV1::Op("=".to_string()),
            "'=' after a binding name",
        )?;
        cur.budget
            .charge_collection_element(bindings.len() + 1, name_span)?;
        let producer = parse_producer(cur, source)?;
        bindings.push(cur.builder.composition_binding(name, name_span, producer));
    }

    // body: a FIND / SELECT / CHECK / EXPLAIN family (legality is downstream).
    let body = parse_body(cur, source)?;

    // trailing `with <name>`
    let with_kw = cur.expect_keyword("with", "'with' to consume a composition binding")?;
    let (with_name, with_name_span) =
        cur.expect_ident("a composition binding name after 'with'")?;

    // the consumed name must be one of the declared bindings (fail closed).
    if !bindings.iter().any(|b| b.name == with_name) {
        return Err(reject(
            cur,
            with_name_span,
            format!("'with {with_name}' names an undefined composition binding"),
        ));
    }

    let span = SpanV1::new(start, with_name_span.end);
    let _ = with_kw;
    let comp = cur
        .builder
        .composition(bindings, with_name, with_name_span, body, span);
    Ok(ProgramItemHirV1::from(comp))
}

// The composition (producer, body) family matrix.
//
// The composition grammar admits a STRICT, ASYMMETRIC matrix of which family a
// producer slot vs a body slot may hold. Any family outside its column rejects
// fail-closed at PARSE time (a typed `UnexpectedToken`), so the boundary cannot
// silently widen when a new family parser is added later. STRUCTURE ONLY (D13):
// this matrix is the parseability gate; producer/body KIND legality for a given
// composition (e.g. whether a CHECK body is meaningful for a FIND producer) is a
// downstream concern (tickets 02/03), not enforced here.
//
//   | family    | producer (parse_producer) | body (parse_body) |
//   |-----------|:-------------------------:|:-----------------:|
//   | from      | yes                       | yes               |
//   | sample    | yes                       | no                |
//   | find      | yes                       | yes               |
//   | check     | no                        | yes               |
//   | explain   | no                        | yes               |
//   | all other | no                        | no                |
//
// Note the asymmetry: `sample` is a producer but never a body; `check`/`explain`
// are bodies but never producers. The reported `find` producer + `check` body
// pairing is therefore a LEGAL composition, not a defect. (Rails pinning every
// accepted/rejected cell live in `tests/v3_family_hellgate.rs`.)

/// Parse a composition PRODUCER (SELECT / SAMPLE / FIND). Other forms reject.
///
/// See the composition (producer, body) matrix comment above for the full table.
fn parse_producer(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<CompositionProducerV1, ParseDiagnosticV1> {
    if cur.is_keyword("from") {
        return Ok(CompositionProducerV1::Query(Box::new(parse_query(cur)?)));
    }
    if cur.is_keyword("sample") {
        let ProgramItemHirV1::Presentation(presentation) = parse_sample(cur, source)? else {
            unreachable!("sample parser returns presentation")
        };
        let sample = SampleHirV1::try_from(*presentation)
            .map_err(|_wrong_kind| cur.unexpected("a SAMPLE composition producer"))?;
        return Ok(CompositionProducerV1::Sample(Box::new(sample)));
    }
    if cur.is_keyword("find") {
        let ProgramItemHirV1::GraphQuery(graph) = parse_find(cur, source)? else {
            unreachable!("find parser returns graph query")
        };
        return Ok(CompositionProducerV1::GraphQuery(graph));
    }
    Err(cur.unexpected("a composition producer (from / sample / find)"))
}

/// Parse a composition BODY (FIND / SELECT / CHECK / EXPLAIN). Other forms
/// reject. The body stops before a trailing `with`.
fn parse_body(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<CompositionBodyV1, ParseDiagnosticV1> {
    if cur.is_keyword("from") {
        return Ok(CompositionBodyV1::Query(Box::new(parse_query(cur)?)));
    }
    if cur.is_keyword("find") {
        let ProgramItemHirV1::GraphQuery(graph) = parse_find(cur, source)? else {
            unreachable!("find parser returns graph query")
        };
        return Ok(CompositionBodyV1::GraphQuery(graph));
    }
    if cur.is_keyword("check") {
        let ProgramItemHirV1::Legality(legality) = parse_check(cur, source)? else {
            unreachable!("check parser returns legality")
        };
        return Ok(CompositionBodyV1::Legality(legality));
    }
    if cur.is_keyword("explain") {
        let ProgramItemHirV1::Presentation(presentation) = parse_explain(cur, source)? else {
            unreachable!("explain parser returns presentation")
        };
        let explain = ExplainHirV1::try_from(*presentation)
            .map_err(|_wrong_kind| cur.unexpected("an EXPLAIN composition body"))?;
        return Ok(CompositionBodyV1::Explain(Box::new(explain)));
    }
    Err(cur.unexpected("a composition body (find / from / check / explain)"))
}

// ---- shared reference + clause primitives ----------------------------------

/// Reconstruct a reference (`a.b.c`, `secure-pack`, `repo::auth`, `models/code`)
/// from a contiguous run of ident/path tokens, returning its verbatim text and
/// merged span. Fails closed if the run does not start with an identifier.
fn parse_ref(
    cur: &mut CursorV1<'_>,
    source: &str,
    what: &str,
) -> Result<GraphRefV1, ParseDiagnosticV1> {
    if !matches!(cur.peek().kind, TokenKindV1::Ident(_) | TokenKindV1::Int(_)) {
        return Err(cur.unexpected(what));
    }
    let start = cur.here_span().start;
    let mut end = cur.here_span().end;
    // first segment
    let _ = cur.bump();
    // continuation: `.`, `-`, `/`, `::`, ident/int segments.
    loop {
        if matches!(cur.peek().kind, TokenKindV1::Dot) {
            let _ = cur.bump();
            let (_s, sp) = cur.expect_ident("a path segment after '.'")?;
            end = sp.end;
            continue;
        }
        if matches!(cur.peek().kind, TokenKindV1::Colon) {
            // accept `::` (two colons) as a path separator.
            let _ = cur.bump();
            if matches!(cur.peek().kind, TokenKindV1::Colon) {
                let _ = cur.bump();
            }
            let (_s, sp) = cur.expect_ident("a path segment after '::'")?;
            end = sp.end;
            continue;
        }
        if matches!(&cur.peek().kind, TokenKindV1::Op(sym) if sym == "-" || sym == "/") {
            let _ = cur.bump();
            if matches!(cur.peek().kind, TokenKindV1::Ident(_) | TokenKindV1::Int(_)) {
                end = cur.here_span().end;
                let _ = cur.bump();
                continue;
            }
            return Err(cur.unexpected("a path segment"));
        }
        break;
    }
    let span = SpanV1::new(start, end);
    let name = source.get(start..end).unwrap_or_default().to_string();
    Ok(GraphRefV1 { name, span })
}

/// Parse a comma-separated list of references.
fn parse_ref_list(
    cur: &mut CursorV1<'_>,
    source: &str,
    what: &str,
) -> Result<Vec<GraphRefV1>, ParseDiagnosticV1> {
    let mut refs = Vec::new();
    loop {
        cur.budget
            .charge_collection_element(refs.len() + 1, cur.here_span())?;
        refs.push(parse_ref(cur, source, what)?);
        if cur.eat(&TokenKindV1::Comma).is_some() {
            continue;
        }
        break;
    }
    Ok(refs)
}

/// Parse a comma-separated list of references whose empty form is illegal.
fn parse_required_ref_list(
    cur: &mut CursorV1<'_>,
    source: &str,
    what: &str,
    empty_span: SpanV1,
    empty_msg: impl Into<String>,
) -> Result<Vec<GraphRefV1>, ParseDiagnosticV1> {
    if !matches!(cur.peek().kind, TokenKindV1::Ident(_)) {
        return Err(reject(cur, empty_span, empty_msg));
    }
    parse_ref_list(cur, source, what)
}

/// Consume a `| key` clause head, returning `true` if a clause was present.
/// Leaves the cursor on the `=` (or the next token) for the value parser.
fn at_clause(cur: &CursorV1<'_>) -> bool {
    matches!(cur.peek().kind, TokenKindV1::Pipe)
}

/// Consume `| <key> =` and return the key text. Caller dispatches on the key.
/// Keys may be hyphenated (`query-ref`, `check-ref`), reassembled verbatim.
fn clause_key(cur: &mut CursorV1<'_>) -> Result<(String, SpanV1), ParseDiagnosticV1> {
    cur.expect(&TokenKindV1::Pipe, "'|' to start a clause")?;
    let (key, span) = parse_keyword_value(cur, "a clause key")?;
    cur.expect(&TokenKindV1::Op("=".to_string()), "'=' after a clause key")?;
    Ok((key, span))
}

/// Build an `UnsupportedConstruct` reject (illegal clause / missing required).
fn reject(cur: &CursorV1<'_>, span: SpanV1, msg: impl Into<String>) -> ParseDiagnosticV1 {
    cur.unsupported(span, msg)
}

/// Record a clause `key` (at `key_span`) in `seen`, failing closed on a repeat.
///
/// D28 (conflicts fail closed): a duplicate clause key is a typed
/// `UnsupportedConstruct` at the offending key's exact span rather than a silent
/// last-wins overwrite. `family` names the family for the diagnostic message.
/// This is the shared form of the FIND duplicate guard, applied uniformly across
/// every clause-bearing family loop.
fn note_clause(
    cur: &CursorV1<'_>,
    seen: &mut Vec<String>,
    family: &str,
    key: &str,
    key_span: SpanV1,
) -> Result<(), ParseDiagnosticV1> {
    if seen.iter().any(|k| k == key) {
        return Err(reject(
            cur,
            key_span,
            format!("duplicate {family} clause '{key}'"),
        ));
    }
    seen.push(key.to_string());
    Ok(())
}

// ---- FIND ------------------------------------------------------------------

fn parse_find(cur: &mut CursorV1<'_>, source: &str) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let find_kw = cur.expect_keyword("find", "'find' to start a graph query")?;
    let graph = parse_find_body(cur, source, find_kw.span)?;
    Ok(ProgramItemHirV1::from(graph))
}

fn parse_find_body(
    cur: &mut CursorV1<'_>,
    source: &str,
    find_span: SpanV1,
) -> Result<GraphQueryHirV1, ParseDiagnosticV1> {
    let (subkind_word, subkind_span) = cur.expect_ident("a FIND subkind")?;
    let kind = match subkind_word.as_str() {
        "reachability" => GraphQueryKindV1::Reachability,
        "flow" => GraphQueryKindV1::Flow,
        "slice" => GraphQueryKindV1::Slice,
        "taint" => GraphQueryKindV1::Taint,
        "path" => GraphQueryKindV1::Path,
        "points_to" => GraphQueryKindV1::PointsTo,
        "summary" => GraphQueryKindV1::Summary,
        other => {
            return Err(reject(
                cur,
                subkind_span,
                format!("unknown FIND subkind '{other}'"),
            ))
        }
    };

    let mut endpoints = GraphEndpointsV1::default();
    let mut constraints = GraphConstraintsV1::default();
    let mut end = subkind_span.end;

    // seed: `<source> -> <sink>` (the canonical reachability/flow/taint/path
    // shape). points_to/slice/summary supply endpoints via clauses instead, so
    // the seed is present only when a `->` follows the first reference.
    if matches!(cur.peek().kind, TokenKindV1::Ident(_)) {
        let src_ref = parse_ref(cur, source, "the FIND source endpoint")?;
        // require the arrow between source and sink.
        if cur.eat(&TokenKindV1::ThinArrow).is_none() {
            return Err(cur.unexpected("'->' between the FIND source and sink"));
        }
        let sink_ref = parse_ref(cur, source, "the FIND sink endpoint")?;
        end = sink_ref.span.end;
        endpoints.source = Some(src_ref);
        endpoints.sink = Some(sink_ref);
    }

    // clauses
    //
    // D28 (conflicts fail closed): a repeated clause key is rejected with a
    // typed `UnsupportedConstruct` rather than silently overwriting (last-wins).
    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        if seen.iter().any(|k| k == &key) {
            return Err(reject(
                cur,
                key_span,
                format!("duplicate FIND clause '{key}'"),
            ));
        }
        seen.push(key.clone());
        match key.as_str() {
            "via" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a via reference",
                    key_span,
                    "FIND via list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                constraints.via = refs;
            }
            "through" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a through reference",
                    key_span,
                    "FIND through list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                constraints.through = refs;
            }
            "sanitizers" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a sanitizer reference",
                    key_span,
                    "FIND sanitizers list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                constraints.sanitizers = refs;
            }
            "scope" => {
                let r = parse_ref(cur, source, "a scope reference")?;
                end = r.span.end;
                constraints.scope = Some(r);
            }
            "window" => {
                let r = parse_ref(cur, source, "a window reference")?;
                end = r.span.end;
                constraints.window = Some(r);
            }
            "frontier" => {
                let r = parse_ref(cur, source, "a frontier reference")?;
                end = r.span.end;
                constraints.frontier = Some(r);
            }
            "direction" => {
                let (kw, sp) = cur.expect_ident("a direction keyword")?;
                end = sp.end;
                // The `direction` tag is open in the HIR, but the surface must
                // fail closed: only the well-attested traversal directions are
                // admitted. An unknown keyword rejects with a typed
                // UnsupportedConstruct at the keyword span, mirroring the
                // confirm/emit/mode open-tag validation policy, rather than
                // silently minting an unvalidated GraphDirectionV1.
                match kw.as_str() {
                    "forward" | "backward" | "both" | "undirected" => {
                        constraints.direction = Some(GraphDirectionV1 { keyword: kw });
                    }
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported FIND direction '{other}'"),
                        ))
                    }
                }
            }
            "depth" => {
                let (n, sp) = parse_u32(cur, "a depth bound")?;
                end = sp.end;
                // depth=0 is a zero-hop walk and so meaningless: reject it
                // fail-closed rather than minting depth=Some(0), mirroring the
                // SAMPLE `limit > 0` policy.
                if n == 0 {
                    return Err(reject(cur, sp, "FIND depth must be > 0"));
                }
                constraints.depth = Some(n);
            }
            "anchor" => {
                let r = parse_ref(cur, source, "an anchor reference")?;
                end = r.span.end;
                endpoints.anchor = Some(r);
            }
            "reference" => {
                let r = parse_ref(cur, source, "a reference endpoint")?;
                end = r.span.end;
                endpoints.reference = Some(r);
            }
            "pointee" => {
                let r = parse_ref(cur, source, "a pointee endpoint")?;
                end = r.span.end;
                endpoints.pointee = Some(r);
            }
            "callable" => {
                let r = parse_ref(cur, source, "a callable endpoint")?;
                end = r.span.end;
                endpoints.callable = Some(r);
            }
            "facets" => {
                // The facet list is required to be non-empty: a bare `facets=`
                // (with no following facet identifier) rejects fail-closed with
                // a typed UnsupportedConstruct rather than relying on a generic
                // structural token error or minting an empty facet vec.
                if !matches!(cur.peek().kind, TokenKindV1::Ident(_)) {
                    return Err(reject(cur, key_span, "FIND facets list must not be empty"));
                }
                let mut facets = Vec::new();
                loop {
                    cur.budget
                        .charge_collection_element(facets.len() + 1, cur.here_span())?;
                    let (f, sp) = cur.expect_ident("a summary facet")?;
                    end = sp.end;
                    facets.push(f);
                    if cur.eat(&TokenKindV1::Comma).is_some() {
                        continue;
                    }
                    break;
                }
                constraints.summary_facets = facets;
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported FIND clause '{other}'"),
                ))
            }
        }
    }

    let span = find_span.merge(&SpanV1::point(end));
    Ok(cur.builder.graph_query(kind, endpoints, constraints, span))
}

// ---- CHECK -----------------------------------------------------------------

fn parse_check(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let check_kw = cur.expect_keyword("check", "'check' to start a legality check")?;
    let rule_ref = parse_ref(cur, source, "a rule reference")?;
    let mut end = rule_ref.span.end;

    let mut scope = None;
    let mut pattern = None;
    let mut confirm = None;
    let mut emit = EmitModeV1::Findings;
    let mut mode = DetectionModeV1::Hybrid;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "CHECK", &key, key_span)?;
        match key.as_str() {
            "scope" => {
                let r = parse_ref(cur, source, "a scope reference")?;
                end = r.span.end;
                scope = Some(r);
            }
            "pattern" => {
                let r = parse_ref(cur, source, "a pattern reference")?;
                end = r.span.end;
                pattern = Some(r);
            }
            "confirm" => {
                let (kw, sp) = cur.expect_ident("a confirm mode")?;
                end = sp.end;
                confirm = Some(match kw.as_str() {
                    "skip" => ConfirmModeV1::Skip,
                    "prompt" => ConfirmModeV1::Prompt,
                    "require" => ConfirmModeV1::Require,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported confirm mode '{other}'"),
                        ))
                    }
                });
            }
            "emit" => {
                let (kw, sp) = cur.expect_ident("an emit target")?;
                end = sp.end;
                emit = match kw.as_str() {
                    "findings" => EmitModeV1::Findings,
                    "sarif" => EmitModeV1::Sarif,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported emit target '{other}'"),
                        ))
                    }
                };
            }
            "mode" => {
                let (kw, sp) = cur.expect_ident("a detection mode")?;
                end = sp.end;
                mode = match kw.as_str() {
                    "pattern" => DetectionModeV1::PatternOnly,
                    "semantic" => DetectionModeV1::SemanticOnly,
                    "hybrid" => DetectionModeV1::Hybrid,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported detection mode '{other}'"),
                        ))
                    }
                };
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported CHECK clause '{other}'"),
                ))
            }
        }
    }

    let confirm = confirm.ok_or_else(|| {
        reject(
            cur,
            check_kw.span,
            "CHECK requires an explicit 'confirm' clause (skip|prompt|require)",
        )
    })?;

    let semantic_confirm = if cur.eat_keyword("using").is_some() {
        let find_kw = cur.expect_keyword("find", "'find' after CHECK 'using'")?;
        Some(parse_find_body(cur, source, find_kw.span)?)
    } else {
        None
    };

    if let Some(query) = &semantic_confirm {
        end = query.span.end;
    }
    let span = check_kw.span.merge(&SpanV1::point(end));
    let chk = match semantic_confirm {
        Some(query) => cur.builder.legality_with_semantic_confirm(
            rule_ref, scope, pattern, query, confirm, emit, mode, span,
        ),
        None => cur
            .builder
            .legality(rule_ref, scope, pattern, confirm, emit, mode, span),
    };
    Ok(ProgramItemHirV1::from(chk))
}

// ---- EXPLAIN ---------------------------------------------------------------

fn parse_explain(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("explain", "'explain' to start an explanation")?;
    let target_ref = parse_ref(cur, source, "a rule or graph reference")?;
    let mut end = target_ref.span.end;
    let mut evidence = EvidenceModeV1::Auto;
    let mut truth = TruthModeV1::Inspect;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "EXPLAIN", &key, key_span)?;
        match key.as_str() {
            "evidence" => {
                let (v, sp) = cur.expect_ident("an evidence mode")?;
                end = sp.end;
                evidence = match v.as_str() {
                    "auto" => EvidenceModeV1::Auto,
                    "trace" => EvidenceModeV1::Trace,
                    "citation" => EvidenceModeV1::Citation,
                    "counterexample" => EvidenceModeV1::Counterexample,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported evidence mode '{other}'"),
                        ))
                    }
                };
            }
            "truth" => {
                let (v, sp) = cur.expect_ident("a truth mode")?;
                end = sp.end;
                truth = match v.as_str() {
                    "inspect" => TruthModeV1::Inspect,
                    "confirm" => TruthModeV1::Confirm,
                    "refute" => TruthModeV1::Refute,
                    other => {
                        return Err(reject(cur, sp, format!("unsupported truth mode '{other}'")))
                    }
                };
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported EXPLAIN clause '{other}'"),
                ))
            }
        }
    }

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Explain {
        target_ref,
        evidence,
        truth,
    };
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- SHOW ------------------------------------------------------------------

fn parse_show(cur: &mut CursorV1<'_>) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("show", "'show' to start a catalog query")?;
    let (cat, cat_span) = cur.expect_ident("a SHOW category")?;
    let category = match cat.as_str() {
        "capabilities" => ShowCategoryV1::Capabilities,
        "models" => ShowCategoryV1::Models,
        "views" => ShowCategoryV1::Views,
        "rules" => ShowCategoryV1::Rules,
        "packs" => ShowCategoryV1::Packs,
        "profiles" => ShowCategoryV1::Profiles,
        "gates" => ShowCategoryV1::Gates,
        other => {
            return Err(reject(
                cur,
                cat_span,
                format!("unsupported SHOW category '{other}'"),
            ))
        }
    };
    let span = kw.span.merge(&cat_span);
    let kind = PresentationKindV1::Show { category };
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- DESCRIBE --------------------------------------------------------------

fn parse_describe(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("describe", "'describe' to start an entity lookup")?;
    let (entity_kind, _kind_span) = cur.expect_ident("an entity kind")?;
    // `kind::path`
    cur.expect(&TokenKindV1::Colon, "':' in a 'kind::path' target")?;
    cur.expect(&TokenKindV1::Colon, "'::' in a 'kind::path' target")?;
    let path = parse_ref(cur, source, "an entity path")?;
    let span = kw.span.merge(&path.span);
    let kind = PresentationKindV1::Describe {
        kind: entity_kind,
        path,
    };
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- SAMPLE ----------------------------------------------------------------

fn parse_sample(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("sample", "'sample' to start a retrieval")?;
    // `query="text"`
    let (q_key, _q_span) = cur.expect_ident("the 'query' key")?;
    if q_key != "query" {
        return Err(reject(
            cur,
            _q_span,
            "SAMPLE must start with a 'query' clause",
        ));
    }
    cur.expect(&TokenKindV1::Op("=".to_string()), "'=' after 'query'")?;
    let (query_text, q_text_span) = expect_string(cur, "a quoted query text")?;
    if query_text.is_empty() {
        return Err(reject(
            cur,
            q_text_span,
            "SAMPLE query_text must not be empty",
        ));
    }
    let mut end = q_text_span.end;

    let mut mode = SampleModeV1::Lexical;
    let mut limit: Option<(u32, SpanV1)> = None;
    let mut scope = None;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "SAMPLE", &key, key_span)?;
        match key.as_str() {
            "mode" => {
                let (v, sp) = cur.expect_ident("a sample mode")?;
                end = sp.end;
                mode = match v.as_str() {
                    "lexical" => SampleModeV1::Lexical,
                    "semantic" => SampleModeV1::Semantic,
                    "hybrid" => SampleModeV1::Hybrid,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported sample mode '{other}'"),
                        ))
                    }
                };
            }
            "limit" => {
                let (n, sp) = parse_u32(cur, "a positive limit")?;
                end = sp.end;
                limit = Some((n, sp));
            }
            "scope" => {
                let r = parse_ref(cur, source, "a scope reference")?;
                end = r.span.end;
                scope = Some(r);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported SAMPLE clause '{other}'"),
                ))
            }
        }
    }

    let limit = match limit {
        Some((n, _sp)) if n > 0 => n,
        Some((_, sp)) => return Err(reject(cur, sp, "SAMPLE limit must be > 0")),
        None => return Err(reject(cur, kw.span, "SAMPLE requires a 'limit' clause")),
    };

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Sample {
        query_text,
        mode,
        limit,
        scope,
    };
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- COMPARE ---------------------------------------------------------------

fn parse_compare(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("compare", "'compare' to start a delta")?;
    // `baseline=<ref>`
    let (b_key, b_span) = cur.expect_ident("the 'baseline' key")?;
    if b_key != "baseline" {
        return Err(reject(
            cur,
            b_span,
            "COMPARE must start with a 'baseline' clause",
        ));
    }
    cur.expect(&TokenKindV1::Op("=".to_string()), "'=' after 'baseline'")?;
    let baseline_ref = parse_ref(cur, source, "a baseline reference")?;
    let mut end = baseline_ref.span.end;

    let mut candidate_ref = None;
    let mut metric = CompareMetricV1::Exact;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "COMPARE", &key, key_span)?;
        match key.as_str() {
            "candidate" => {
                let r = parse_ref(cur, source, "a candidate reference")?;
                end = r.span.end;
                candidate_ref = Some(r);
            }
            "metric" => {
                let (v, sp) = cur.expect_ident("a comparison metric")?;
                end = sp.end;
                metric = match v.as_str() {
                    "exact" => CompareMetricV1::Exact,
                    "structural" => CompareMetricV1::Structural,
                    "semantic" => CompareMetricV1::Semantic,
                    other => return Err(reject(cur, sp, format!("unsupported metric '{other}'"))),
                };
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported COMPARE clause '{other}'"),
                ))
            }
        }
    }

    let candidate_ref = candidate_ref.ok_or_else(|| {
        reject(
            cur,
            kw.span,
            "COMPARE requires an explicit 'candidate' clause",
        )
    })?;

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Compare {
        baseline_ref,
        candidate_ref,
        metric,
    };
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- EXPORT ----------------------------------------------------------------

fn parse_export(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("export", "'export' to start a bridge receipt")?;
    let (s_key, s_span) = cur.expect_ident("the 'source' key")?;
    if s_key != "source" {
        return Err(reject(
            cur,
            s_span,
            "EXPORT must start with a 'source' clause",
        ));
    }
    cur.expect(&TokenKindV1::Op("=".to_string()), "'=' after 'source'")?;
    let source_ref = parse_ref(cur, source, "a source reference")?;
    let mut end = source_ref.span.end;

    let mut format = None;
    let mut profile = None;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "EXPORT", &key, key_span)?;
        match key.as_str() {
            "kind" => {
                let r = parse_ref(cur, source, "an export kind")?;
                end = r.span.end;
                format = Some(match r.name.as_str() {
                    "json" => ExportFormatV1::Json,
                    "csv" => ExportFormatV1::Csv,
                    "sarif" => ExportFormatV1::Sarif,
                    "markdown" => ExportFormatV1::Markdown,
                    "graph-ml" => ExportFormatV1::GraphMl,
                    other => {
                        return Err(reject(
                            cur,
                            r.span,
                            format!("unsupported export kind '{other}'"),
                        ))
                    }
                });
            }
            "profile" => {
                let r = parse_ref(cur, source, "a profile reference")?;
                end = r.span.end;
                profile = Some(r);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported EXPORT clause '{other}'"),
                ))
            }
        }
    }

    let format = format.ok_or_else(|| reject(cur, kw.span, "EXPORT requires a 'kind' clause"))?;

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Export {
        source_ref,
        format,
        profile,
    };
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- VIEW ------------------------------------------------------------------

fn parse_view(cur: &mut CursorV1<'_>, source: &str) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("view", "'view' to declare a view")?;
    let name_ref = parse_ref(cur, source, "a view name")?;
    let (key, key_span) = clause_key(cur)?;
    let (body, end) = match key.as_str() {
        "query-ref" => {
            let r = parse_ref(cur, source, "a query reference")?;
            (ViewBodyV1::QueryRef(r.clone()), r.span.end)
        }
        "select" => {
            // inline SELECT: the v3 `from | ...` pipe is the shared query carrier.
            let query = parse_query(cur)?;
            let e = query.span.end;
            (ViewBodyV1::Inline(Box::new(query)), e)
        }
        other => {
            return Err(reject(
                cur,
                key_span,
                format!("VIEW body must be 'query-ref' or 'select', got '{other}'"),
            ))
        }
    };
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::View(body),
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- MODEL -----------------------------------------------------------------

fn parse_model(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("model", "'model' to declare a model")?;
    let name_ref = parse_ref(cur, source, "a model name")?;
    let mut provider = None;
    let mut model_source = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "MODEL", &key, key_span)?;
        match key.as_str() {
            "provider" => {
                let r = parse_ref(cur, source, "a provider tag")?;
                end = r.span.end;
                provider = Some(r.name);
            }
            "source" => {
                let r = parse_ref(cur, source, "a source reference")?;
                end = r.span.end;
                model_source = Some(r);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported MODEL clause '{other}'"),
                ))
            }
        }
    }

    let provider = provider.ok_or_else(|| reject(cur, kw.span, "MODEL requires a 'provider'"))?;
    let model_source =
        model_source.ok_or_else(|| reject(cur, kw.span, "MODEL requires a 'source'"))?;
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Model {
            provider,
            source: model_source,
        },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- PACK ------------------------------------------------------------------

fn parse_pack(cur: &mut CursorV1<'_>, source: &str) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("pack", "'pack' to declare a pack")?;
    let name_ref = parse_ref(cur, source, "a pack name")?;
    let mut pack_source = None;
    let mut activation = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "PACK", &key, key_span)?;
        match key.as_str() {
            "source" => {
                let r = parse_ref(cur, source, "a source reference")?;
                end = r.span.end;
                pack_source = Some(r);
            }
            "activation" => {
                let (v, sp) = activation_value(cur)?;
                end = sp.end;
                activation = Some(v);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported PACK clause '{other}'"),
                ))
            }
        }
    }

    let pack_source =
        pack_source.ok_or_else(|| reject(cur, kw.span, "PACK requires a 'source'"))?;
    let activation =
        activation.ok_or_else(|| reject(cur, kw.span, "PACK requires an 'activation'"))?;
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Pack {
            source: pack_source,
            activation,
        },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- PROFILE ---------------------------------------------------------------

fn parse_profile(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("profile", "'profile' to declare a profile")?;
    let name_ref = parse_ref(cur, source, "a profile name")?;
    let mut capabilities = Vec::new();
    let mut admission = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "PROFILE", &key, key_span)?;
        match key.as_str() {
            "capabilities" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a capability reference",
                    key_span,
                    "PROFILE capabilities list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                capabilities = refs;
            }
            "admission" => {
                let (v, sp) = parse_keyword_value(cur, "an admission policy")?;
                end = sp.end;
                admission = Some(match v.as_str() {
                    "allowlist" => AdmissionV1::Allowlist,
                    "review-required" => AdmissionV1::ReviewRequired,
                    "certified-only" => AdmissionV1::CertifiedOnly,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported admission policy '{other}'"),
                        ))
                    }
                });
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported PROFILE clause '{other}'"),
                ))
            }
        }
    }

    let admission =
        admission.ok_or_else(|| reject(cur, kw.span, "PROFILE requires an 'admission'"))?;
    if capabilities.is_empty() {
        return Err(reject(
            cur,
            kw.span,
            "PROFILE requires a 'capabilities' clause",
        ));
    }
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Profile {
            capabilities,
            admission,
        },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- GATE ------------------------------------------------------------------

fn parse_gate(cur: &mut CursorV1<'_>, source: &str) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("gate", "'gate' to declare a gate")?;
    let name_ref = parse_ref(cur, source, "a gate name")?;
    let mut checks = Vec::new();
    let mut binding = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "GATE", &key, key_span)?;
        match key.as_str() {
            "checks" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a check reference",
                    key_span,
                    "GATE checks list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                checks = refs;
            }
            "binding" => {
                let (v, sp) = cur.expect_ident("a binding policy")?;
                end = sp.end;
                binding = Some(match v.as_str() {
                    "all" => GateBindingV1::All,
                    "any" => GateBindingV1::Any,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported binding policy '{other}'"),
                        ))
                    }
                });
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported GATE clause '{other}'"),
                ))
            }
        }
    }

    let binding = binding.ok_or_else(|| reject(cur, kw.span, "GATE requires a 'binding'"))?;
    if checks.is_empty() {
        return Err(reject(cur, kw.span, "GATE requires a 'checks' clause"));
    }
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Gate { checks, binding },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- RULE binding (ruledef) ------------------------------------------------

fn parse_ruledef(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("ruledef", "'ruledef' to bind a catalog rule")?;
    let name_ref = parse_ref(cur, source, "a rule name")?;
    let (key, key_span) = clause_key(cur)?;
    if key != "check-ref" {
        return Err(reject(
            cur,
            key_span,
            format!("RULE binding requires a 'check-ref' clause, got '{key}'"),
        ));
    }
    let check_ref = parse_ref(cur, source, "a check reference")?;
    let span = kw.span.merge(&check_ref.span);
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::RuleBinding { check_ref },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- META: USE PACK --------------------------------------------------------

fn parse_use_pack(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("use", "'use' to start a meta directive")?;
    let pack_word = cur.expect_keyword("pack", "'pack' after 'use'")?;
    let pack_ref = parse_ref(cur, source, "a pack id")?;
    let mut end = pack_ref.span.end;

    let mut activation = None;
    let mut lockfile = None;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "USE PACK", &key, key_span)?;
        match key.as_str() {
            "activation" => {
                let (v, sp) = activation_value(cur)?;
                end = sp.end;
                // USE PACK only admits session/workspace (manual is reject in v2).
                if matches!(v, ActivationV1::Manual) {
                    return Err(reject(
                        cur,
                        sp,
                        "USE PACK activation must be 'session' or 'workspace'",
                    ));
                }
                activation = Some(v);
            }
            "lockfile" => {
                let r = parse_ref(cur, source, "a lockfile path")?;
                end = r.span.end;
                if !r.name.ends_with(".lock") {
                    return Err(reject(
                        cur,
                        r.span,
                        "USE PACK lockfile must end with '.lock'",
                    ));
                }
                lockfile = Some(r.name);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported USE PACK clause '{other}'"),
                ))
            }
        }
    }

    let activation = activation
        .ok_or_else(|| reject(cur, pack_word.span, "USE PACK requires an 'activation'"))?;
    let lockfile = lockfile
        .ok_or_else(|| reject(cur, pack_word.span, "USE PACK requires a '.lock' lockfile"))?;
    let span = kw.span.merge(&SpanV1::point(end));
    let meta = cur.builder.meta(
        MetaKindV1::UsePack(UsePackHirV1 {
            pack_id: pack_ref.name,
            activation: match activation {
                ActivationV1::Session => UsePackActivationV1::Session,
                ActivationV1::Workspace => UsePackActivationV1::Workspace,
                ActivationV1::Manual => {
                    return Err(reject(
                        cur,
                        pack_word.span,
                        "USE PACK activation must be 'session' or 'workspace'",
                    ));
                }
            },
            lockfile,
        }),
        span,
    );
    Ok(ProgramItemHirV1::from(meta))
}

// ---- META: REQUIRE CERTIFIED -----------------------------------------------

fn parse_require_certified(cur: &mut CursorV1<'_>) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("require", "'require' to start a meta directive")?;
    let cert = cur.expect_keyword("certified", "'certified' after 'require'")?;
    let span = kw.span.merge(&cert.span);
    let meta = cur.builder.meta(MetaKindV1::RequireCertified, span);
    Ok(ProgramItemHirV1::from(meta))
}

// ---- small value parsers ---------------------------------------------------

/// Parse a non-negative integer literal into a `u32` plus its span.
fn parse_u32(cur: &mut CursorV1<'_>, what: &str) -> Result<(u32, SpanV1), ParseDiagnosticV1> {
    if let TokenKindV1::Int(raw) = &cur.peek().kind {
        let raw = raw.clone();
        let span = cur.here_span();
        let _ = cur.bump();
        let n = raw
            .parse::<u32>()
            .map_err(|_err| cur.unsupported(span, format!("'{raw}' is not a valid {what}")))?;
        return Ok((n, span));
    }
    Err(cur.unexpected(what))
}

/// Parse a quoted string literal into its content plus span.
fn expect_string(
    cur: &mut CursorV1<'_>,
    what: &str,
) -> Result<(String, SpanV1), ParseDiagnosticV1> {
    if let TokenKindV1::Str(content) = &cur.peek().kind {
        let content = content.clone();
        let span = cur.here_span();
        let _ = cur.bump();
        return Ok((content, span));
    }
    Err(cur.unexpected(what))
}

/// Parse an activation policy keyword (`manual`/`session`/`workspace`).
fn activation_value(cur: &mut CursorV1<'_>) -> Result<(ActivationV1, SpanV1), ParseDiagnosticV1> {
    let (v, sp) = cur.expect_ident("an activation policy")?;
    let act = match v.as_str() {
        "manual" => ActivationV1::Manual,
        "session" => ActivationV1::Session,
        "workspace" => ActivationV1::Workspace,
        other => return Err(cur.unsupported(sp, format!("unsupported activation '{other}'"))),
    };
    Ok((act, sp))
}

/// Parse a possibly-hyphenated keyword VALUE (`certified-only`,
/// `review-required`) into a single reassembled string plus its merged span.
/// The lexer splits `a-b` into `Ident "-" Ident`, so this stitches the run back
/// together verbatim from the token text.
fn parse_keyword_value(
    cur: &mut CursorV1<'_>,
    what: &str,
) -> Result<(String, SpanV1), ParseDiagnosticV1> {
    let (first, first_span) = cur.expect_ident(what)?;
    let mut text = first;
    let mut end = first_span.end;
    while matches!(&cur.peek().kind, TokenKindV1::Op(s) if s == "-") {
        let _ = cur.bump();
        let (seg, sp) = cur.expect_ident("the tail of a hyphenated keyword")?;
        text.push('-');
        text.push_str(&seg);
        end = sp.end;
    }
    Ok((text, SpanV1::new(first_span.start, end)))
}

#[cfg(test)]
mod tests {
    use super::is_family_keyword;

    /// `is_family_keyword` discriminates the leading keywords it owns from any
    /// other word. Kills family.rs:45 (`is_family_keyword` -> `true`): a
    /// non-family word must return `false`.
    #[test]
    fn family_keyword_discriminates() {
        // a representative member of the owned set returns true.
        assert!(is_family_keyword("find"));
        assert!(is_family_keyword("require"));
        // arbitrary non-family words return false (the `-> true` mutant fails).
        assert!(!is_family_keyword("from"));
        assert!(!is_family_keyword("let"));
        assert!(!is_family_keyword("zzz_not_a_family"));
        assert!(!is_family_keyword(""));
    }
}
