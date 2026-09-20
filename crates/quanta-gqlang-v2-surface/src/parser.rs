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
use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};
use crate::family::{at_composition, is_family_keyword, parse_composition, parse_family};
use crate::lexer::{lex_with_budget, TokenKindV1};
use crate::query::parse_query;
use crate::unit::{parse_decl, parse_rule};
use quanta_gqlang_v2_hir::{
    DeclHirV1, HirBuilderV1, HirDocumentV1, ProgramItemHirV1, QueryHirV1, RuleHirV1, SourceIdV2,
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
    let mut cur = CursorV1::new(tokens, builder, counter);

    let mut items = Vec::new();
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
) -> Result<ParsedDocumentV2, ParseDiagnosticV1> {
    let builder = HirBuilderV1::new();
    let parsed = parse_document_items_with_budget(source, &builder, budget)?;
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
