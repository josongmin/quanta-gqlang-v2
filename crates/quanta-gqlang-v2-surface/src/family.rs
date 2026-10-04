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
use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};
use crate::lexer::TokenKindV1;
use crate::materialization::FrontendRetainedStoragePermitV3;
use crate::query::parse_query;
use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_hir::{
    CompositionBodyV1, CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExplainHirV1, GraphConstraintsV1, GraphDirectionV1, GraphEndpointsV1,
    GraphQueryHirV1, GraphQueryKindV1, GraphRefV1, PresentationKindV1, ProgramItemHirV1,
    RenamePhaseV3, SampleHirV1, TruthModeV1,
};

mod catalog;
mod presentation;

use catalog::{
    parse_gate, parse_model, parse_pack, parse_profile, parse_require_certified, parse_ruledef,
    parse_use_pack, parse_view,
};
use presentation::{parse_compare, parse_describe, parse_export, parse_sample, parse_show};

fn boxed_program_item_v3<T>(
    cur: &mut CursorV1<'_>,
    node: T,
    span: SpanV1,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1>
where
    ProgramItemHirV1: From<T>,
{
    cur.reserve_nested_bytes_v3(std::mem::size_of::<T>(), span)?;
    Ok(ProgramItemHirV1::from(node))
}

struct RequestBindingNodeV3 {
    binding: Option<quanta_gqlang_v2_hir::CompositionBindingV1>,
    next: Option<Box<Self>>,
    _scratch_permit: FrontendRetainedStoragePermitV3,
}

impl Drop for RequestBindingNodeV3 {
    fn drop(&mut self) {
        // Error paths may drop a long partial composition. Detach successors
        // iteratively so the linked scratch never recurses on the Rust stack.
        let mut next = self.next.take();
        while let Some(mut node) = next {
            next = node.next.take();
        }
    }
}

enum CompositionBindingsV3 {
    Raw(Vec<quanta_gqlang_v2_hir::CompositionBindingV1>),
    Request {
        newest: Option<Box<RequestBindingNodeV3>>,
        len: usize,
    },
}

impl CompositionBindingsV3 {
    fn new(request_owner: bool) -> Self {
        if request_owner {
            Self::Request {
                newest: None,
                len: 0,
            }
        } else {
            Self::Raw(Vec::new())
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Raw(bindings) => bindings.len(),
            Self::Request { len, .. } => *len,
        }
    }

    fn contains_name(&self, name: &str) -> bool {
        match self {
            Self::Raw(bindings) => bindings.iter().any(|binding| binding.name == name),
            Self::Request { newest, .. } => {
                let mut node = newest.as_deref();
                while let Some(binding) = node {
                    if binding
                        .binding
                        .as_ref()
                        .is_some_and(|value| value.name == name)
                    {
                        return true;
                    }
                    node = binding.next.as_deref();
                }
                false
            }
        }
    }

    fn push(
        &mut self,
        cur: &mut CursorV1<'_>,
        binding: quanta_gqlang_v2_hir::CompositionBindingV1,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        match self {
            Self::Raw(bindings) => bindings.push(binding),
            Self::Request { newest, len } => {
                let new_len = len.checked_add(1).ok_or_else(|| {
                    ParseDiagnosticV1::new(
                        ParseErrorCodeV1::ResourceExhausted,
                        span,
                        "composition binding count overflow",
                    )
                })?;
                // This linked node is temporary parser scratch. Give it its
                // own live permit so conversion to the final admitted Vec
                // releases the scratch reservation with the last node.
                let scratch_permit = cur
                    .reserve_temporary_scratch_v3(
                        std::mem::size_of::<RequestBindingNodeV3>(),
                        span,
                    )?
                    .ok_or_else(|| {
                        ParseDiagnosticV1::new(
                            ParseErrorCodeV1::ResourceExhausted,
                            span,
                            "request composition scratch has no storage owner",
                        )
                    })?;
                let previous = newest.take();
                *newest = Some(Box::new(RequestBindingNodeV3 {
                    binding: Some(binding),
                    next: previous,
                    _scratch_permit: scratch_permit,
                }));
                *len = new_len;
            }
        }
        Ok(())
    }

    fn into_vec(
        self,
        cur: &mut CursorV1<'_>,
        span: SpanV1,
    ) -> Result<Vec<quanta_gqlang_v2_hir::CompositionBindingV1>, ParseDiagnosticV1> {
        match self {
            Self::Raw(bindings) => Ok(bindings),
            Self::Request { mut newest, len } => {
                let bytes = len
                    .checked_mul(std::mem::size_of::<
                        quanta_gqlang_v2_hir::CompositionBindingV1,
                    >())
                    .ok_or_else(|| {
                        ParseDiagnosticV1::new(
                            ParseErrorCodeV1::ResourceExhausted,
                            span,
                            "composition binding vector size overflow",
                        )
                    })?;
                cur.reserve_nested_bytes_v3(bytes, span)?;
                let mut ordered = Vec::new();
                ordered
                    .try_reserve_exact(len)
                    .map_err(|_allocation_error| {
                        ParseDiagnosticV1::new(
                            ParseErrorCodeV1::ResourceExhausted,
                            span,
                            "composition binding vector allocation failed",
                        )
                    })?;
                if ordered.capacity() > len {
                    return Err(ParseDiagnosticV1::new(
                        ParseErrorCodeV1::ResourceExhausted,
                        span,
                        "composition binding allocator exceeded admitted slots",
                    ));
                }
                while let Some(mut node) = newest {
                    let binding = node.binding.take().ok_or_else(|| {
                        ParseDiagnosticV1::new(
                            ParseErrorCodeV1::ResourceExhausted,
                            span,
                            "composition binding scratch lost its value",
                        )
                    })?;
                    ordered.push(binding);
                    newest = node.next.take();
                }
                ordered.reverse();
                Ok(ordered)
            }
        }
    }
}

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
            | "rename"
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
        return parse_show(cur, source);
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
    if cur.is_keyword("rename") {
        return parse_rename(cur, source);
    }
    Err(cur.unexpected("a v3 family keyword"))
}

// ---- RENAME COMMAND -------------------------------------------------------

/// One canonical, fail-closed command surface. Snapshot and preview token are
/// authored preconditions; the host still owns authority to preview or mutate.
fn parse_rename(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let start = cur
        .expect_keyword("rename", "'rename' to start a command")?
        .span
        .start;
    let target = parse_ref(cur, source, "a closed symbol selector")?;
    cur.expect_keyword("to", "'to' after a rename target")?;
    let (new_name, name_span) = expect_string(cur, "a quoted replacement name")?;
    if new_name.is_empty() {
        return Err(reject(
            cur,
            name_span,
            "rename replacement must not be empty",
        ));
    }
    let mut end = name_span.end;
    let mut phase = None;
    let mut snapshot = None;
    let mut token = None;
    let mut seen = 0u64;
    let mut seen_count = 0usize;
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        let expected = match seen_count {
            0 => "phase",
            1 => "snapshot",
            2 if phase == Some(RenamePhaseV3::Apply) => "preview_token",
            _ => return Err(reject(cur, key_span, "unexpected RENAME clause")),
        };
        if key != expected {
            return Err(reject(
                cur,
                key_span,
                format!("expected RENAME clause '{expected}'"),
            ));
        }
        note_clause(cur, &mut seen, "RENAME", &key, key_span)?;
        seen_count += 1;
        match key.as_str() {
            "phase" => {
                let (value, span) = cur.expect_ident("'preview' or 'apply'")?;
                phase = Some(match value.as_str() {
                    "preview" => RenamePhaseV3::Preview,
                    "apply" => RenamePhaseV3::Apply,
                    _ => return Err(reject(cur, span, "unsupported RENAME phase")),
                });
                end = span.end;
            }
            "snapshot" => {
                let (value, span) = expect_string(cur, "a quoted expected snapshot")?;
                if value.is_empty() {
                    return Err(reject(cur, span, "RENAME snapshot must not be empty"));
                }
                snapshot = Some(value);
                end = span.end;
            }
            "preview_token" => {
                let (value, span) = expect_string(cur, "a quoted issued preview token")?;
                if value.is_empty() {
                    return Err(reject(cur, span, "RENAME preview token must not be empty"));
                }
                token = Some(value);
                end = span.end;
            }
            _ => return Err(reject(cur, key_span, "unsupported RENAME clause")),
        }
    }
    let phase =
        phase.ok_or_else(|| reject(cur, SpanV1::new(start, end), "RENAME phase is required"))?;
    let snapshot = snapshot
        .ok_or_else(|| reject(cur, SpanV1::new(start, end), "RENAME snapshot is required"))?;
    if matches!(phase, RenamePhaseV3::Preview) && token.is_some() {
        return Err(reject(
            cur,
            SpanV1::new(start, end),
            "preview cannot carry a preview token",
        ));
    }
    if matches!(phase, RenamePhaseV3::Apply) && token.is_none() {
        return Err(reject(
            cur,
            SpanV1::new(start, end),
            "apply requires an issued preview token",
        ));
    }
    let command = cur.builder.rename_command(
        target,
        new_name,
        phase,
        snapshot,
        token,
        SpanV1::new(start, end),
    );
    boxed_program_item_v3(cur, command, SpanV1::new(start, end))
}

// ---- COMPOSITION -----------------------------------------------------------

/// Whether the cursor is at a composition: `let <name> = <producer> ...`, where
/// the producer is a SELECT (`from`), SAMPLE (`sample`), or FIND (`find`). A
/// leading unparenthesized `from`, `sample` or `find` selects this grammar even
/// when that spelling could be a value symbol elsewhere. Use `let x = (from)`
/// (and likewise for sample/find) for a declaration referring to such a symbol.
/// Raw HIR whose canonical rendering is ambiguous is rejected by try_format_item.
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
    let mut bindings = CompositionBindingsV3::new(cur.has_request_owner_v3());
    let start = cur.here_span().start;

    while cur.is_keyword("let") {
        let _let = cur.expect_keyword("let", "'let' to start a composition binding")?;
        let (name, name_span) = cur.expect_ident("a composition binding name")?;
        // duplicate-name reject (fail closed).
        if bindings.contains_name(&name) {
            return Err(reject(
                cur,
                name_span,
                format!("duplicate composition binding '{name}'"),
            ));
        }
        cur.expect_op("=", "'=' after a binding name")?;
        let next_binding_count = bindings.len().checked_add(1).ok_or_else(|| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                name_span,
                "composition binding count overflow",
            )
        })?;
        cur.budget
            .charge_collection_element(next_binding_count, name_span)?;
        let producer = parse_producer(cur, source)?;
        let binding = cur.builder.composition_binding(name, name_span, producer);
        bindings.push(cur, binding, name_span)?;
    }

    // body: a FIND / SELECT / CHECK / EXPLAIN family (legality is downstream).
    let body = parse_body(cur, source)?;

    // trailing `with <name>`
    let with_kw = cur.expect_keyword("with", "'with' to consume a composition binding")?;
    let (with_name, with_name_span) =
        cur.expect_ident("a composition binding name after 'with'")?;

    // the consumed name must be one of the declared bindings (fail closed).
    if !bindings.contains_name(&with_name) {
        return Err(reject(
            cur,
            with_name_span,
            format!("'with {with_name}' names an undefined composition binding"),
        ));
    }

    let span = SpanV1::new(start, with_name_span.end);
    let _ = with_kw;
    let bindings = bindings.into_vec(cur, span)?;
    let comp = cur
        .builder
        .composition(bindings, with_name, with_name_span, body, span);
    boxed_program_item_v3(cur, comp, span)
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
        let query = parse_query(cur)?;
        cur.reserve_nested_bytes_v3(std::mem::size_of_val(&query), query.span)?;
        return Ok(CompositionProducerV1::Query(Box::new(query)));
    }
    if cur.is_keyword("sample") {
        let ProgramItemHirV1::Presentation(presentation) = parse_sample(cur, source)? else {
            unreachable!("sample parser returns presentation")
        };
        let sample = SampleHirV1::try_from(*presentation)
            .map_err(|_wrong_kind| cur.unexpected("a SAMPLE composition producer"))?;
        cur.reserve_nested_bytes_v3(std::mem::size_of_val(&sample), sample.span)?;
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
        let query = parse_query(cur)?;
        cur.reserve_nested_bytes_v3(std::mem::size_of_val(&query), query.span)?;
        return Ok(CompositionBodyV1::Query(Box::new(query)));
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
        cur.reserve_nested_bytes_v3(std::mem::size_of_val(&explain), explain.span)?;
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
            if cur.here_span().start != end {
                return Err(cur.unexpected("a contiguous '.' separator"));
            }
            let separator = cur.bump();
            if cur.here_span().start != separator.span.end {
                return Err(cur.unexpected("a path segment after '.'"));
            }
            let (_s, sp) = cur.expect_ident("a path segment after '.'")?;
            end = sp.end;
            continue;
        }
        if matches!(cur.peek().kind, TokenKindV1::Colon) {
            // `::` is atomic: exactly two consecutive colons. A single colon
            // rejects at the token following it (REF-01); lenient recovery
            // would silently widen the reference grammar.
            if cur.here_span().start != end {
                return Err(cur.unexpected("a contiguous '::' separator"));
            }
            let first_colon = cur.bump();
            if cur.here_span().start != first_colon.span.end {
                return Err(cur.unexpected("a second ':' in '::'"));
            }
            let Some(second_colon) = cur.eat(&TokenKindV1::Colon) else {
                return Err(cur.unexpected("a second ':' in '::'"));
            };
            if cur.here_span().start != second_colon.span.end {
                return Err(cur.unexpected("a path segment after '::'"));
            }
            let (_s, sp) = cur.expect_ident("a path segment after '::'")?;
            end = sp.end;
            continue;
        }
        if matches!(&cur.peek().kind, TokenKindV1::Op(sym) if sym == "-" || sym == "/") {
            if cur.here_span().start != end {
                return Err(cur.unexpected("a contiguous path separator"));
            }
            let separator = cur.bump();
            if cur.here_span().start != separator.span.end {
                return Err(cur.unexpected("a path segment after separator"));
            }
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
    let name = cur.materialize_reference_v1(source, span)?;
    Ok(GraphRefV1 { name, span })
}

/// Parse a comma-separated list of references.
fn parse_ref_list(
    cur: &mut CursorV1<'_>,
    source: &str,
    what: &str,
) -> Result<Vec<GraphRefV1>, ParseDiagnosticV1> {
    let mut refs = cur.preallocate_clause_items_v3::<GraphRefV1>()?;
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
    cur.expect_op("=", "'=' after a clause key")?;
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
    seen: &mut u64,
    family: &str,
    key: &str,
    key_span: SpanV1,
) -> Result<(), ParseDiagnosticV1> {
    let Some(bit) = match key {
        "phase" => Some(0),
        "snapshot" => Some(1),
        "preview_token" => Some(2),
        "provider" => Some(3),
        "source" => Some(4),
        "activation" => Some(5),
        "capabilities" => Some(6),
        "admission" => Some(7),
        "checks" => Some(8),
        "binding" => Some(9),
        "lockfile" => Some(10),
        "scope" => Some(11),
        "pattern" => Some(12),
        "confirm" => Some(13),
        "emit" => Some(14),
        "mode" => Some(15),
        "evidence" => Some(16),
        "truth" => Some(17),
        "limit" => Some(18),
        "candidate" => Some(19),
        "metric" => Some(20),
        "kind" => Some(21),
        "profile" => Some(22),
        "via" => Some(23),
        "through" => Some(24),
        "sanitizers" => Some(25),
        "window" => Some(26),
        "frontier" => Some(27),
        "direction" => Some(28),
        "depth" => Some(29),
        "anchor" => Some(30),
        "reference" => Some(31),
        "pointee" => Some(32),
        "callable" => Some(33),
        "facets" => Some(34),
        _ => None,
    }
    .map(|position| 1u64 << position) else {
        return Err(reject(
            cur,
            key_span,
            format!("unsupported {family} clause '{key}'"),
        ));
    };
    if *seen & bit != 0 {
        return Err(reject(
            cur,
            key_span,
            format!("duplicate {family} clause '{key}'"),
        ));
    }
    *seen |= bit;
    Ok(())
}

// ---- FIND ------------------------------------------------------------------

fn parse_find(cur: &mut CursorV1<'_>, source: &str) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let find_kw = cur.expect_keyword("find", "'find' to start a graph query")?;
    let graph = parse_find_body(cur, source, find_kw.span)?;
    let span = graph.span;
    boxed_program_item_v3(cur, graph, span)
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
    let mut seen = 0u64;
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "FIND", &key, key_span)?;
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
                let mut facets = cur.preallocate_clause_items_v3::<String>()?;
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

    let mut seen = 0u64;
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
        cur.reserve_nested_bytes_v3(std::mem::size_of::<GraphQueryHirV1>(), query.span)?;
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
    let span = chk.span;
    boxed_program_item_v3(cur, chk, span)
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

    let mut seen = 0u64;
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
    let presentation = cur.builder.presentation(kind, span);
    boxed_program_item_v3(cur, presentation, span)
}

// ---- small value parsers ---------------------------------------------------

/// Parse a non-negative integer literal into a `u32` plus its span.
fn parse_u32(cur: &mut CursorV1<'_>, what: &str) -> Result<(u32, SpanV1), ParseDiagnosticV1> {
    if matches!(&cur.peek().kind, TokenKindV1::Int(_)) {
        let token = cur.bump();
        let TokenKindV1::Int(raw) = token.kind else {
            return Err(cur.unexpected(what));
        };
        let span = token.span;
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
    if matches!(&cur.peek().kind, TokenKindV1::Str(_)) {
        let token = cur.bump();
        if let TokenKindV1::Str(content) = token.kind {
            return Ok((content, token.span));
        }
    }
    Err(cur.unexpected(what))
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
        // Every keyword the family dispatcher owns must be recognized; a single
        // dropped arm would send that family down the non-family parse path.
        for keyword in [
            "find", "check", "explain", "show", "describe", "sample", "compare", "export", "view",
            "model", "pack", "profile", "gate", "ruledef", "use", "require",
        ] {
            assert!(
                is_family_keyword(keyword),
                "{keyword} must be a family keyword"
            );
        }
        // Sibling root keywords owned elsewhere must not be claimed here.
        for keyword in ["from", "let", "rule", "select"] {
            assert!(
                !is_family_keyword(keyword),
                "{keyword} must not be a family keyword"
            );
        }
        // arbitrary non-family words return false (the `-> true` mutant fails).
        assert!(!is_family_keyword("zzz_not_a_family"));
        assert!(!is_family_keyword(""));
    }
}
