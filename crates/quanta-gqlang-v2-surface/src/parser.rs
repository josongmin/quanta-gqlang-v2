//! Top-level program dispatch and the fail-closed entry points.
//!
//! Discriminates a single surface item by its leading keyword (`let` -> decl,
//! `rule` -> rule, `from` -> query) and lowers it to the matching shared HIR
//! unit. After the item, input MUST be exhausted: trailing tokens are a typed
//! reject, never a silent truncation (D10).
//!
//! ## Compat-parsing policy (explicit, ticket 06 mandatory output)
//!
//! This parser is **v3-only**. It does NOT accept the legacy v2 surface and
//! does NOT down-convert to a v2 subset on error. Concretely:
//!
//! - The v2 root-keyword line surface (`SELECT ... FROM ...`, uppercase root
//!   dispatch) is rejected with [`ParseErrorCodeV1::UnsupportedConstruct`] at the
//!   offending token's exact span, rather than reparsed leniently. This is the
//!   no-silent-downgrade guarantee (packet D10): recovery must not encode v2
//!   subset semantics.
//! - No legacy parse path is retained; the legacy v2 parser stack has been
//!   removed entirely (ticket 09 cutover), so v3 is the sole frontend.

use crate::budget::{ParseBudgetCounterV2, ParseBudgetV2};
use crate::cursor::CursorV1;
use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1, SurfaceDiagnosticV2};
use crate::family::{at_composition, is_family_keyword, parse_composition, parse_family};
use crate::lexer::{lex_with_budget, TokenKindV1};
use crate::materialization::{
    FrontendLexicalMaterializationRootV1, FrontendLexicalPermitV1, ReferenceCloneAdmissionV1,
    ReferenceCloneAggregateV1,
};
use crate::query::parse_query;
use crate::unit::{parse_decl, parse_rule};
use quanta_gqlang_v2_hir::{
    DeclHirV1, HirBuilderV1, HirDocumentV1, PresentationKindV1, ProgramItemHirV1, QueryHirV1,
    RuleHirV1, SourceIdV2,
};

/// A parsed top-level surface item. Exactly one shared HIR unit per program.
///
/// The original `Query`/`Decl`/`Rule` carriers stay first-class so existing
/// callers and tree shapes are unchanged; the [`Self::Program`] variant carries
/// the remaining v3 root families (FIND / CHECK / presentation verbs / catalog
/// declarations / composition / meta) as a unified `hir::ProgramItemHirV1`.
#[derive(Clone, Debug, PartialEq)]
pub enum SurfaceItemV1 {
    /// A `from ...` query.
    Query(QueryHirV1),
    /// A `let ...` declaration.
    Decl(DeclHirV1),
    /// A `rule ...` rule.
    Rule(RuleHirV1),
    /// Any other v3 root family, lowered to the unified program item carrier.
    Program(ProgramItemHirV1),
}

impl SurfaceItemV1 {
    /// Borrowing conversion into the unified program-item view.
    ///
    /// The standalone `query` / `decl` / `rule` carriers stay first-class at the
    /// parse boundary, but downstream stages compile the one shared
    /// [`ProgramItemHirV1`] surface. Keeping the lift here prevents CLI /
    /// pipeline / tests from open-coding the same match and drifting.
    #[must_use]
    pub fn to_program_item(&self) -> ProgramItemHirV1 {
        match self {
            Self::Query(query) => ProgramItemHirV1::from(query.clone()),
            Self::Decl(decl) => ProgramItemHirV1::from(decl.clone()),
            Self::Rule(rule) => ProgramItemHirV1::from(rule.clone()),
            Self::Program(program) => program.clone(),
        }
    }

    /// Owning conversion into the unified program-item view.
    #[must_use]
    pub fn into_program_item(self) -> ProgramItemHirV1 {
        match self {
            Self::Query(query) => ProgramItemHirV1::from(query),
            Self::Decl(decl) => ProgramItemHirV1::from(decl),
            Self::Rule(rule) => ProgramItemHirV1::from(rule),
            Self::Program(program) => program,
        }
    }
}

/// The successful result of a parse: the lowered HIR item plus the source length
/// it was lowered from (so callers can `hir::check_tree` against the same span
/// bounds the parser used).
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedProgramV1 {
    /// The single lowered surface item.
    pub item: SurfaceItemV1,
    /// Byte length of the source the item was parsed from.
    pub source_len: usize,
}

/// Parse a single v3 surface item from `source` into shared HIR.
///
/// Fail-closed: returns the first typed [`ParseDiagnosticV1`] with an exact
/// span on any lex or parse error, on a rejected v2-ism, or on trailing tokens.
/// The caller passes a fresh [`HirBuilderV1`] so the returned tree's node ids
/// are unique by construction.
pub fn parse_program(
    source: &str,
    builder: &HirBuilderV1,
) -> Result<ParsedProgramV1, ParseDiagnosticV1> {
    parse_program_with_budget(source, builder, &ParseBudgetV2::standard_v2())
}

pub(crate) fn parse_program_with_budget(
    source: &str,
    builder: &HirBuilderV1,
    budget: &ParseBudgetV2,
) -> Result<ParsedProgramV1, ParseDiagnosticV1> {
    let counter = ParseBudgetCounterV2::new(budget);
    let tokens = lex_with_budget(source, &counter)?;
    let mut cur = CursorV1::new(tokens, builder, counter);

    cur.budget.charge_document_item(cur.here_span())?;
    reject_v2_isms(&cur)?;

    let item = parse_one_item(&mut cur, source)?;

    // An optional single trailing `;` terminator is tolerated so the single-item
    // and multi-item separator policies agree (a lone statement may be `;`-ended).
    let _ = cur.eat(&TokenKindV1::Semicolon);

    // Input must be fully consumed; trailing tokens fail closed.
    if !cur.at_eof() {
        return Err(ParseDiagnosticV1::new(
            ParseErrorCodeV1::UnexpectedToken,
            cur.here_span(),
            "unexpected trailing tokens after the surface item",
        ));
    }

    crate::depth::check_item(
        &item,
        cur.budget.max_expression_depth(),
        cur.budget.max_relation_depth(),
    )?;

    Ok(ParsedProgramV1 {
        item,
        source_len: source.len(),
    })
}

/// A parsed multi-root document: an ordered list of top-level surface items.
///
/// Every item is lowered through ONE shared [`HirBuilderV1`], so node ids are
/// globally unique across the WHOLE document by construction (the builder mints
/// monotonically increasing ids and never resets between items).
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDocumentV2 {
    /// Canonical HIR document and its single source-length authority.
    pub hir: HirDocumentV1,
}

/// Parse result retaining lexer, parser item, `parse_ref` copy, SHOW box, and
/// outer HIR vector debits while its HIR is owned. Other nested HIR and later stages
/// need separate permits.
pub struct AdmittedLexicalValueV1<'root, 'source, T> {
    value: T,
    // Value is dropped before all permits. Reservations share one live counter,
    // so concurrent parser strings and vectors are admitted together.
    _hir_item_permit: Option<FrontendLexicalPermitV1<'root, 'source>>,
    _reference_permit: Option<FrontendLexicalPermitV1<'root, 'source>>,
    _parser_item_permit: Option<FrontendLexicalPermitV1<'root, 'source>>,
    permit: FrontendLexicalPermitV1<'root, 'source>,
}

pub type AdmittedLexicalParsedDocumentV1<'root, 'source> =
    AdmittedLexicalValueV1<'root, 'source, HirDocumentV1>;

impl<'root, 'source, T> AdmittedLexicalValueV1<'root, 'source, T> {
    #[must_use]
    pub fn value_v1(&self) -> &T {
        &self.value
    }

    /// Reject a distinct request root even when it has the same source ID,
    /// source bytes, and numeric ceiling. Retained stage debits must share the
    /// first parser reservation's live counter.
    #[must_use]
    pub fn shares_root_v2(&self, root: &FrontendLexicalMaterializationRootV1<'_>) -> bool {
        self.permit.belongs_to_root_v2(root)
    }

    /// Consume one owner and carry its permit into the next owned stage.
    /// The permit remains live throughout the transformation.
    pub fn try_map_v1<U, E>(
        self,
        transform: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<AdmittedLexicalValueV1<'root, 'source, U>, E> {
        let value = transform(self.value)?;
        Ok(AdmittedLexicalValueV1 {
            value,
            _hir_item_permit: self._hir_item_permit,
            _reference_permit: self._reference_permit,
            _parser_item_permit: self._parser_item_permit,
            permit: self.permit,
        })
    }
}

impl<'root, 'source> AdmittedLexicalValueV1<'root, 'source, HirDocumentV1> {
    /// A typed grammar check for the zero-heap SHOW integrity path. No source
    /// name or span heuristic can turn an adjacent presentation into SHOW.
    #[must_use]
    pub fn is_show_only_v2(&self) -> bool {
        !self.value.items.is_empty()
            && self.value.items.iter().all(|item| {
                matches!(item, ProgramItemHirV1::Presentation(presentation)
                    if matches!(&presentation.kind, PresentationKindV1::Show { .. }))
            })
    }
}

/// Parse-derived top-level family, without semantic or execution authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceRootFamilyV3 {
    Relation,
    Declaration,
    Rule,
    Graph,
    Check,
    Presentation,
    Catalog,
    Composition,
    Meta,
    Command,
}

/// Classify one already-parsed HIR root at the parser-owner boundary.
///
/// The pipeline owns document cardinality and source-budget checks; this
/// function only maps canonical closed HIR variants.
#[must_use]
pub fn classify_parsed_root_v3(item: &ProgramItemHirV1) -> SourceRootFamilyV3 {
    match item {
        ProgramItemHirV1::Query(_) => SourceRootFamilyV3::Relation,
        ProgramItemHirV1::Decl(_) => SourceRootFamilyV3::Declaration,
        ProgramItemHirV1::Rule(_) => SourceRootFamilyV3::Rule,
        ProgramItemHirV1::GraphQuery(_) => SourceRootFamilyV3::Graph,
        ProgramItemHirV1::Legality(_) => SourceRootFamilyV3::Check,
        ProgramItemHirV1::Presentation(_) => SourceRootFamilyV3::Presentation,
        ProgramItemHirV1::CatalogDecl(_) => SourceRootFamilyV3::Catalog,
        ProgramItemHirV1::Composition(_) => SourceRootFamilyV3::Composition,
        ProgramItemHirV1::Meta(_) => SourceRootFamilyV3::Meta,
        ProgramItemHirV1::Command(_) => SourceRootFamilyV3::Command,
    }
}

/// Legacy single-source document result. It intentionally does not expose a
/// document identity; callers needing cross-document identity must use
/// [`parse_source_document_v2`].
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDocumentV1 {
    /// Ordered surface items.
    pub items: Vec<SurfaceItemV1>,
    /// Byte length of the parsed source.
    pub source_len: usize,
}

/// Parse a MULTI-ROOT document (several top-level items) into an ordered list.
///
/// ## Separator policy
///
/// Items are separated by an explicit `;` statement terminator. A trailing `;`
/// after the final item is optional. Whitespace and newlines are insignificant
/// (the lexer already skips them), so an item may freely span multiple lines —
/// this is exactly what lets a `let <name> = <producer> ... <body> with <name>`
/// COMPOSITION group into ONE item even though it spans many lines: composition
/// has no internal `;`, so the document loop never splits it. The `;` was an
/// illegal character in the single-item grammar, so adding it as the separator
/// introduces no ambiguity with any existing family clause.
///
/// Fail-closed: returns the first typed [`ParseDiagnosticV1`] (carrying the
/// offending item's exact span) on any lex/parse error, on a rejected v2-ism, or
/// on a malformed separator. Empty statements (`;;` or a leading `;`) reject.
pub fn parse_document(
    source: &str,
    builder: &HirBuilderV1,
) -> Result<ParsedDocumentV1, ParseDiagnosticV1> {
    parse_document_items_with_budget(source, builder, &ParseBudgetV2::standard_v2())
}

pub(crate) fn parse_document_items_with_budget(
    source: &str,
    builder: &HirBuilderV1,
    budget: &ParseBudgetV2,
) -> Result<ParsedDocumentV1, ParseDiagnosticV1> {
    let counter = ParseBudgetCounterV2::new(budget);
    let tokens = lex_with_budget(source, &counter)?;
    parse_document_items_from_tokens(source, builder, counter, tokens, None, None)
}

/// For a valid document every semicolon separates one item, except that the
/// final semicolon terminates the final item. This scan is over admitted lexer
/// tokens and allocates nothing before the parser's item vector is reserved.
fn parser_item_slots_v1(tokens: &[crate::lexer::TokenV1]) -> Result<usize, ParseDiagnosticV1> {
    let mut separators = 0usize;
    let mut has_item_tokens = false;
    let mut trailing_separator = false;
    for token in tokens {
        if matches!(&token.kind, TokenKindV1::Eof) {
            break;
        }
        if matches!(&token.kind, TokenKindV1::Semicolon) {
            separators = separators.checked_add(1).ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::LexicalMaterializationOverflow,
                    token.span,
                    "parser item count overflow",
                )
            })?;
            trailing_separator = true;
        } else {
            has_item_tokens = true;
            trailing_separator = false;
        }
    }
    if !has_item_tokens {
        return Ok(0);
    }
    separators
        .checked_add(usize::from(!trailing_separator))
        .ok_or_else(|| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                tokens
                    .last()
                    .map_or(quanta_gqlang_v2_hir::SpanV1::point(0), |token| token.span),
                "parser item count overflow",
            )
        })
}

fn parser_item_storage_bytes_v1(slots: usize) -> Result<usize, ParseDiagnosticV1> {
    slots
        .checked_mul(std::mem::size_of::<SurfaceItemV1>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<SurfaceItemV1>>()))
        .ok_or_else(|| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                quanta_gqlang_v2_hir::SpanV1::point(0),
                "parser item storage size overflow",
            )
        })
}

fn parse_document_items_from_tokens(
    source: &str,
    builder: &HirBuilderV1,
    counter: ParseBudgetCounterV2,
    tokens: Vec<crate::lexer::TokenV1>,
    admitted_slots: Option<usize>,
    reference_admission: Option<&mut dyn ReferenceCloneAdmissionV1>,
) -> Result<ParsedDocumentV1, ParseDiagnosticV1> {
    let mut cur = match reference_admission {
        Some(admission) => {
            CursorV1::new_with_reference_admission_v1(tokens, builder, counter, admission)
        }
        None => CursorV1::new(tokens, builder, counter),
    };

    let mut items = Vec::new();
    if let Some(slots) = admitted_slots {
        items.try_reserve_exact(slots).map_err(|_error| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                cur.here_span(),
                "parser item storage allocation failed",
            )
        })?;
        if items.capacity() > slots {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                cur.here_span(),
                "parser item allocator exceeded admitted slots",
            ));
        }
    }
    while !cur.at_eof() {
        // A `;` with no preceding item (leading or doubled separator) fails
        // closed rather than producing an empty item.
        if matches!(cur.peek().kind, TokenKindV1::Semicolon) {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedToken,
                cur.here_span(),
                "empty statement: a ';' must follow a top-level item",
            ));
        }

        cur.budget.charge_document_item(cur.here_span())?;
        reject_v2_isms(&cur)?;
        let item = parse_one_item(&mut cur, source)?;
        crate::depth::check_item(
            &item,
            cur.budget.max_expression_depth(),
            cur.budget.max_relation_depth(),
        )?;
        if admitted_slots.is_some_and(|slots| items.len() >= slots) {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                cur.here_span(),
                "parser item census is smaller than emitted item count",
            ));
        }
        items.push(item);

        // Each item is terminated by `;` or EOF. A non-separator, non-EOF token
        // here is trailing garbage on the item and fails closed at its span.
        if cur.at_eof() {
            break;
        }
        if cur.eat(&TokenKindV1::Semicolon).is_none() {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedToken,
                cur.here_span(),
                "expected ';' to separate top-level items",
            ));
        }
    }

    if admitted_slots.is_some_and(|slots| items.len() != slots) {
        return Err(ParseDiagnosticV1::new(
            ParseErrorCodeV1::LexicalMaterializationOverflow,
            cur.here_span(),
            "parser item census disagrees with emitted item count",
        ));
    }
    Ok(ParsedDocumentV1 {
        items,
        source_len: source.len(),
    })
}

/// Parse a source document with an explicit bounded profile.
pub fn parse_source_document_v2(
    source_id: SourceIdV2,
    source: &str,
    budget: &ParseBudgetV2,
) -> Result<ParsedDocumentV2, SurfaceDiagnosticV2> {
    let builder = HirBuilderV1::new();
    let parsed = parse_document_items_with_budget(source, &builder, budget)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    Ok(ParsedDocumentV2 {
        hir: HirDocumentV1 {
            source_id,
            source_len: parsed.source_len,
            items: parsed
                .items
                .into_iter()
                .map(SurfaceItemV1::into_program_item)
                .collect(),
        },
    })
}

/// Reserve the lexical demand before the first token or payload allocation,
/// then reserve source-derived item slots before creating the parser item Vec.
/// `parse_ref` debits each exact source span before cloning; the converted
/// outer HIR Vec is reserved from the parsed item count before allocation.
/// All permits remain attached to the HIR owner. Other nested HIR allocations
/// in general grammar branches remain unqualified.
pub fn parse_source_document_with_lexical_owner_v1<'root, 'source>(
    source_id: SourceIdV2,
    source: &'source str,
    budget: &ParseBudgetV2,
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedLexicalParsedDocumentV1<'root, 'source>, SurfaceDiagnosticV2> {
    let census = crate::lexer::census_lexical_materialization_v1(source, budget)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let permit = root
        .reserve_v1(source_id, source, census.logical_bytes())
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let builder = HirBuilderV1::new();
    let counter = ParseBudgetCounterV2::new(budget);
    let tokens = crate::lexer::lex_with_census_v1(source, &counter, census)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let item_slots = parser_item_slots_v1(&tokens)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let item_bytes = parser_item_storage_bytes_v1(item_slots)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let parser_item_permit = root
        .reserve_v1(source_id, source, item_bytes)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let mut reference_admission = ReferenceCloneAggregateV1::new(root, source_id);
    let parsed = parse_document_items_from_tokens(
        source,
        &builder,
        counter,
        tokens,
        Some(item_slots),
        Some(&mut reference_admission),
    )
    .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let hir_item_slots = parsed.items.len();
    let hir_item_bytes = hir_item_slots
        .checked_mul(std::mem::size_of::<ProgramItemHirV1>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<ProgramItemHirV1>>()))
        .ok_or_else(|| {
            SurfaceDiagnosticV2::from_parse(
                source_id,
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::LexicalMaterializationOverflow,
                    quanta_gqlang_v2_hir::SpanV1::point(0),
                    "HIR item vector storage size overflow",
                ),
            )
        })?;
    let hir_item_permit = root
        .reserve_v1(source_id, source, hir_item_bytes)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let mut hir_items = Vec::new();
    hir_items
        .try_reserve_exact(hir_item_slots)
        .map_err(|_error| {
            SurfaceDiagnosticV2::from_parse(
                source_id,
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::ResourceExhausted,
                    quanta_gqlang_v2_hir::SpanV1::point(0),
                    "HIR item vector storage allocation failed",
                ),
            )
        })?;
    if hir_items.capacity() > hir_item_slots {
        return Err(SurfaceDiagnosticV2::from_parse(
            source_id,
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                quanta_gqlang_v2_hir::SpanV1::point(0),
                "HIR item vector allocator exceeded admitted slots",
            ),
        ));
    }
    for item in parsed.items {
        hir_items.push(item.into_program_item());
    }
    Ok(AdmittedLexicalValueV1 {
        value: HirDocumentV1 {
            source_id,
            source_len: parsed.source_len,
            items: hir_items,
        },
        _hir_item_permit: Some(hir_item_permit),
        _reference_permit: reference_admission.into_permit_v1(),
        _parser_item_permit: Some(parser_item_permit),
        permit,
    })
}

/// Count lexical storage for one source without creating a token or HIR node.
/// The result deliberately excludes parser, semantic, and plan allocations.
pub fn census_source_lexical_materialization_v2(
    source_id: SourceIdV2,
    source: &str,
    budget: &ParseBudgetV2,
) -> Result<crate::lexer::LexicalMaterializationCensusV1, SurfaceDiagnosticV2> {
    crate::lexer::census_lexical_materialization_v1(source, budget)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))
}

/// Dispatch and parse exactly ONE top-level surface item from `cur`.
///
/// Shared by [`parse_program`] (single item) and [`parse_document`] (multi
/// item). Does NOT enforce EOF or consume any separator: the caller decides what
/// must follow. The leading-keyword dispatch is identical to the single-item
/// path so both entry points lower the same way.
fn parse_one_item(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<SurfaceItemV1, ParseDiagnosticV1> {
    let item = if at_composition(cur) {
        SurfaceItemV1::Program(parse_composition(cur, source)?)
    } else if cur.is_keyword("let") {
        SurfaceItemV1::Decl(parse_decl(cur)?)
    } else if cur.is_keyword("rule") {
        SurfaceItemV1::Rule(parse_rule(cur)?)
    } else if cur.is_keyword("from") {
        SurfaceItemV1::Query(parse_query(cur)?)
    } else if is_family_keyword(leading_word(cur)) {
        SurfaceItemV1::Program(parse_family(cur, source)?)
    } else {
        return Err(
            cur.unexpected("a top-level item ('let', 'rule', 'from', or a v3 family keyword)")
        );
    };
    Ok(item)
}

/// The leading identifier text, or `""` if the cursor is not on an identifier.
fn leading_word<'a>(cur: &'a CursorV1<'_>) -> &'a str {
    if let TokenKindV1::Ident(word) = &cur.peek().kind {
        word
    } else {
        ""
    }
}

/// Explicitly reject recognized v2 surface shapes instead of reparsing them.
///
/// The v2 surface dispatches on an uppercase root keyword such as `SELECT`. If
/// the program leads with one of those, we fail closed with a typed
/// `UnsupportedConstruct` diagnostic at its exact span. This is the
/// no-silent-downgrade guard (D10): the parser does not narrow to a v2 subset.
fn reject_v2_isms(cur: &CursorV1<'_>) -> Result<(), ParseDiagnosticV1> {
    if let TokenKindV1::Ident(word) = &cur.peek().kind {
        if is_v2_root_keyword(word) {
            return Err(cur.unsupported(
                cur.here_span(),
                format!(
                    "v2 surface keyword '{word}' is not accepted by the v3 parser; \
                     use the v3 'from ... | ...' pipeline form"
                ),
            ));
        }
    }
    Ok(())
}

/// Uppercase v2 root keywords whose presence marks a legacy (v2) program.
fn is_v2_root_keyword(word: &str) -> bool {
    matches!(
        word,
        "SELECT" | "FROM" | "WHERE" | "GROUP" | "ORDER" | "DEFINE" | "DECLARE" | "RULE"
    )
}

#[cfg(test)]
mod lexical_owner_tests {
    #![expect(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "unit tests assert exact lexical ownership outcomes"
    )]

    use super::*;
    use crate::materialization::FrontendLexicalHostCeilingV1;

    #[test]
    fn parsed_hir_retains_lexical_permit_until_owner_drop() {
        let source_id = SourceIdV2::from_raw(31);
        let source = String::from("show models");
        let budget = ParseBudgetV2::standard_v2();
        let lexical_demand = crate::lexer::census_lexical_materialization_v1(&source, &budget)
            .expect("source-derived census")
            .logical_bytes();
        let parser_demand =
            std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
        let hir_demand =
            std::mem::size_of::<Vec<ProgramItemHirV1>>() + std::mem::size_of::<ProgramItemHirV1>();
        let show_box_demand = std::mem::size_of::<quanta_gqlang_v2_hir::PresentationHirV1>();
        let demand = u64::try_from(lexical_demand + parser_demand + hir_demand + show_box_demand)
            .expect("logical demand fits u64");
        let host = FrontendLexicalHostCeilingV1::try_new(demand).expect("host ceiling");
        let root = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, demand, host)
            .expect("source-bound root");
        let admitted =
            parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
                .expect("admitted parse");
        assert_eq!(admitted.value_v1().items.len(), 1);
        assert_eq!(root.live_bytes_v1(), demand);
        let mapped = admitted
            .try_map_v1(|hir| Ok::<_, ()>(hir.items.len()))
            .expect("move-only owner transfer");
        assert_eq!(*mapped.value_v1(), 1);
        assert_eq!(root.live_bytes_v1(), demand);
        drop(mapped);
        assert_eq!(root.live_bytes_v1(), 0);
    }

    #[test]
    fn show_hir_vector_one_byte_short_rejects_and_releases_all_prior_permits() {
        let source_id = SourceIdV2::from_raw(33);
        let source = String::from("show models");
        let budget = ParseBudgetV2::standard_v2();
        let lexical_bytes = crate::lexer::census_lexical_materialization_v1(&source, &budget)
            .expect("source-derived lexical census")
            .logical_bytes();
        let parser_bytes =
            std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
        let hir_bytes =
            std::mem::size_of::<Vec<ProgramItemHirV1>>() + std::mem::size_of::<ProgramItemHirV1>();
        let show_box_bytes = std::mem::size_of::<quanta_gqlang_v2_hir::PresentationHirV1>();
        let total = u64::try_from(lexical_bytes + parser_bytes + hir_bytes + show_box_bytes)
            .expect("logical demand fits u64");
        let host = FrontendLexicalHostCeilingV1::try_new(total).expect("host ceiling");
        let root =
            FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, total - 1, host)
                .expect("one-byte-short request");
        let error = parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
            .err()
            .expect("HIR vector must reject before birth");
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        assert_eq!(root.live_bytes_v1(), 0);
    }

    #[test]
    fn show_box_one_byte_short_rejects_before_box_birth_and_releases_prior_permits() {
        let source_id = SourceIdV2::from_raw(37);
        let source = String::from("show models");
        let budget = ParseBudgetV2::standard_v2();
        let lexical_bytes = crate::lexer::census_lexical_materialization_v1(&source, &budget)
            .expect("source-derived lexical census")
            .logical_bytes();
        let parser_bytes =
            std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
        let show_box_bytes = std::mem::size_of::<quanta_gqlang_v2_hir::PresentationHirV1>();
        let short = u64::try_from(lexical_bytes + parser_bytes + show_box_bytes - 1)
            .expect("logical demand fits u64");
        let host = FrontendLexicalHostCeilingV1::try_new(short + 1).expect("host ceiling");
        let root = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, short, host)
            .expect("one-byte-short request");
        let error = parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
            .err()
            .expect("SHOW box must reject before birth");
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        assert_eq!(root.live_bytes_v1(), 0);
    }

    #[test]
    fn show_hir_vector_rejects_foreign_source_root() {
        let source_id = SourceIdV2::from_raw(34);
        let source = String::from("show models");
        let foreign_source = String::from("show models");
        let budget = ParseBudgetV2::standard_v2();
        let host = FrontendLexicalHostCeilingV1::try_new(4096).expect("host ceiling");
        let root =
            FrontendLexicalMaterializationRootV1::issue_v1(source_id, &foreign_source, 4096, host)
                .expect("foreign invocation root");
        assert!(
            parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
                .is_err()
        );
        assert_eq!(root.live_bytes_v1(), 0);
    }

    #[test]
    fn describe_reference_copy_is_charged_before_birth_and_retained_across_map() {
        let source_id = SourceIdV2::from_raw(35);
        let source = String::from("describe model::foo");
        let budget = ParseBudgetV2::standard_v2();
        let lexical_bytes = crate::lexer::census_lexical_materialization_v1(&source, &budget)
            .expect("source-derived lexical census")
            .logical_bytes();
        let parser_bytes =
            std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
        let hir_bytes =
            std::mem::size_of::<Vec<ProgramItemHirV1>>() + std::mem::size_of::<ProgramItemHirV1>();
        let base = u64::try_from(lexical_bytes + parser_bytes + hir_bytes)
            .expect("base logical demand fits u64");
        let exact = base + 3;
        let host = FrontendLexicalHostCeilingV1::try_new(exact).expect("host ceiling");
        let root = FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, exact, host)
            .expect("source-bound root");
        let admitted =
            parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
                .expect("exact reference copy debit");
        assert_eq!(admitted.value_v1().items.len(), 1);
        assert_eq!(root.live_bytes_v1(), exact);
        let mapped = admitted
            .try_map_v1(|hir| Ok::<_, ()>(hir.items.len()))
            .expect("all permits move with owner");
        assert_eq!(root.live_bytes_v1(), exact);
        drop(mapped);
        assert_eq!(root.live_bytes_v1(), 0);

        let short = FrontendLexicalMaterializationRootV1::issue_v1(
            source_id,
            &source,
            u64::try_from(lexical_bytes + parser_bytes).unwrap() + 2,
            host,
        )
        .expect("one-byte-short before reference copy");
        let error =
            parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &short)
                .err()
                .expect("reference copy must reject before String birth");
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        assert_eq!(short.live_bytes_v1(), 0);
    }

    #[test]
    fn parser_item_slot_one_byte_short_rejects_and_releases_lexical_permit() {
        let source_id = SourceIdV2::from_raw(32);
        let source = String::from("show models");
        let budget = ParseBudgetV2::standard_v2();
        let lexical_bytes = crate::lexer::census_lexical_materialization_v1(&source, &budget)
            .expect("source-derived lexical census")
            .logical_bytes();
        let item_bytes =
            std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
        let total = u64::try_from(lexical_bytes + item_bytes).expect("logical demand fits u64");
        let host = FrontendLexicalHostCeilingV1::try_new(total).expect("host ceiling");
        let root =
            FrontendLexicalMaterializationRootV1::issue_v1(source_id, &source, total - 1, host)
                .expect("one-byte-short request");
        let error = parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
            .err()
            .expect("parser item vector must reject before birth");
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        assert_eq!(root.live_bytes_v1(), 0);
    }
}
