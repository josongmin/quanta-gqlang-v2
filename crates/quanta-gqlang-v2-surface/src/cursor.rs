//! A token cursor shared by every sub-parser.
//!
//! Holds the token stream plus a position, and threads the single
//! `hir::HirBuilderV1` that mints node ids so the whole parse produces one
//! id-unique tree. All lookahead/consume helpers are fail-closed: requesting a
//! token past EOF returns the EOF token (never panics), and `expect_*` helpers
//! return typed [`ParseDiagnosticV1`] on mismatch.

use crate::budget::ParseBudgetCounterV2;
use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};
use crate::lexer::{TokenKindV1, TokenV1};
use crate::materialization::{FrontendRetainedStoragePermitV3, ReferenceCloneAdmissionV1};
use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};

/// Shared parsing state: token stream + position + the id-allocating builder.
pub(crate) struct CursorV1<'b> {
    tokens: Vec<TokenV1>,
    pos: usize,
    /// One shared non-resetting parse counter for the whole document.
    pub(crate) budget: ParseBudgetCounterV2,
    /// The single builder for the whole parse; borrowed by every sub-parser.
    pub(crate) builder: &'b HirBuilderV1,
    reference_admission: Option<&'b mut dyn ReferenceCloneAdmissionV1>,
}

impl<'b> CursorV1<'b> {
    /// Construct over a non-empty token stream (must end in `Eof`).
    pub(crate) fn new(
        tokens: Vec<TokenV1>,
        builder: &'b HirBuilderV1,
        budget: ParseBudgetCounterV2,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            budget,
            builder,
            reference_admission: None,
        }
    }

    pub(crate) fn new_with_reference_admission_v1(
        tokens: Vec<TokenV1>,
        builder: &'b HirBuilderV1,
        budget: ParseBudgetCounterV2,
        reference_admission: &'b mut dyn ReferenceCloneAdmissionV1,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            budget,
            builder,
            reference_admission: Some(reference_admission),
        }
    }

    /// Preserve the legacy producer while admitted parsing debits exact source
    /// bytes before the reference String's first allocation.
    pub(crate) fn materialize_reference_v1(
        &mut self,
        source: &str,
        span: SpanV1,
    ) -> Result<String, ParseDiagnosticV1> {
        let Some(admission) = self.reference_admission.as_mut() else {
            let text = source.get(span.start..span.end).ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::UnexpectedToken,
                    span,
                    "reference span is outside source",
                )
            })?;
            return Ok(text.to_string());
        };
        admission.reserve_reference_v1(source, span)?;
        let text = source.get(span.start..span.end).ok_or_else(|| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedToken,
                span,
                "reference span is outside source",
            )
        })?;
        let mut owned = String::new();
        owned.try_reserve_exact(text.len()).map_err(|_error| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                span,
                "reference string allocation failed",
            )
        })?;
        if owned.capacity() > text.len() {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                span,
                "reference allocator capacity exceeds admitted bytes",
            ));
        }
        owned.push_str(text);
        Ok(owned)
    }

    /// Debit the source-bound presentation payload before `ProgramItemHirV1`
    /// boxes it. The aggregate permit moves with the admitted parsed owner.
    pub(crate) fn reserve_show_box_v1(
        &mut self,
        source: &str,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        if let Some(admission) = self.reference_admission.as_mut() {
            admission.reserve_show_box_v1(source, span)?;
        }
        Ok(())
    }

    /// Admit one concrete nested HIR allocation before its constructor runs.
    /// Raw and lexical-only entries keep their existing partial contract.
    pub(crate) fn reserve_nested_bytes_v3(
        &mut self,
        bytes: usize,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        if let Some(admission) = self.reference_admission.as_mut() {
            admission.reserve_nested_bytes_v3(bytes, span)?;
        }
        Ok(())
    }

    pub(crate) fn has_request_owner_v3(&self) -> bool {
        self.reference_admission
            .as_ref()
            .is_some_and(|admission| admission.is_request_owner_v3())
    }

    pub(crate) fn reserve_temporary_scratch_v3(
        &mut self,
        bytes: usize,
        span: SpanV1,
    ) -> Result<Option<FrontendRetainedStoragePermitV3>, ParseDiagnosticV1> {
        self.reference_admission
            .as_mut()
            .map_or(Ok(None), |admission| {
                admission.reserve_temporary_scratch_v3(bytes, span)
            })
    }

    /// Count a comma list from the already lexed tokens before creating its
    /// Vec. The caller has consumed the opening delimiter and supplies its
    /// closing kind. Nested delimiters do not contribute separators.
    pub(crate) fn preallocate_delimited_items_v3<T>(
        &mut self,
        close: TokenKindV1,
    ) -> Result<Vec<T>, ParseDiagnosticV1> {
        if !self.has_request_owner_v3() {
            return Ok(Vec::new());
        }
        let span = self.here_span();
        let mut parens = 0usize;
        let mut braces = 0usize;
        let mut brackets = 0usize;
        let mut slots = usize::from(self.peek().kind != close);
        let mut offset = 0usize;
        loop {
            let kind = &self.peek_ahead(offset).kind;
            if matches!(kind, TokenKindV1::Eof) {
                return Err(self.unexpected("a closing list delimiter"));
            }
            let top = parens == 0 && braces == 0 && brackets == 0;
            if top && kind == &close {
                break;
            }
            if top && matches!(kind, TokenKindV1::Comma) {
                slots = slots.checked_add(1).ok_or_else(|| {
                    ParseDiagnosticV1::new(
                        ParseErrorCodeV1::LexicalMaterializationOverflow,
                        span,
                        "collection item count overflow",
                    )
                })?;
            }
            match kind {
                TokenKindV1::LParen => parens += 1,
                TokenKindV1::LBrace => braces += 1,
                TokenKindV1::LBracket => brackets += 1,
                TokenKindV1::RParen if parens > 0 => parens -= 1,
                TokenKindV1::RBrace if braces > 0 => braces -= 1,
                TokenKindV1::RBracket if brackets > 0 => brackets -= 1,
                TokenKindV1::RParen | TokenKindV1::RBrace | TokenKindV1::RBracket => {
                    return Err(self.unexpected("a matching collection delimiter"));
                }
                TokenKindV1::Ident(_)
                | TokenKindV1::Int(_)
                | TokenKindV1::Float(_)
                | TokenKindV1::Str(_)
                | TokenKindV1::Comma
                | TokenKindV1::Dot
                | TokenKindV1::Colon
                | TokenKindV1::Semicolon
                | TokenKindV1::Pipe
                | TokenKindV1::Arrow
                | TokenKindV1::ThinArrow
                | TokenKindV1::Op(_)
                | TokenKindV1::Eof => {}
            }
            offset = offset.checked_add(1).ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::LexicalMaterializationOverflow,
                    span,
                    "collection token offset overflow",
                )
            })?;
        }
        self.budget.charge_collection_element(slots, span)?;
        self.preallocate_exact_items_v3(slots, span)
    }

    pub(crate) fn preallocate_exact_items_v3<T>(
        &mut self,
        slots: usize,
        span: SpanV1,
    ) -> Result<Vec<T>, ParseDiagnosticV1> {
        if !self.has_request_owner_v3() {
            return Ok(Vec::new());
        }
        let bytes = slots.checked_mul(std::mem::size_of::<T>()).ok_or_else(|| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                span,
                "collection storage size overflow",
            )
        })?;
        if bytes > 0 {
            self.reserve_nested_bytes_v3(bytes, span)?;
        }
        let mut items = Vec::new();
        items
            .try_reserve_exact(slots)
            .map_err(|_allocation_error| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::ResourceExhausted,
                    span,
                    "collection storage allocation failed",
                )
            })?;
        if items.capacity() > slots {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                span,
                "collection allocator exceeded admitted slots",
            ));
        }
        Ok(items)
    }

    /// Count top-level projection/group/order items until the query stage
    /// boundary. Commas inside call, record, list, or grouped expressions do
    /// not grow the outer Vec. The parse budget checks projection width before
    /// any request-owned allocation.
    pub(crate) fn preallocate_stage_items_v3<T>(&mut self) -> Result<Vec<T>, ParseDiagnosticV1> {
        if !self.has_request_owner_v3() {
            return Ok(Vec::new());
        }
        let span = self.here_span();
        let mut parens = 0usize;
        let mut braces = 0usize;
        let mut brackets = 0usize;
        let mut slots = 1usize;
        let mut offset = 0usize;
        loop {
            let kind = &self.peek_ahead(offset).kind;
            let top = parens == 0 && braces == 0 && brackets == 0;
            if top
                && matches!(
                    kind,
                    TokenKindV1::Pipe
                        | TokenKindV1::Arrow
                        | TokenKindV1::Semicolon
                        | TokenKindV1::RParen
                        | TokenKindV1::RBrace
                        | TokenKindV1::RBracket
                        | TokenKindV1::Eof
                )
            {
                break;
            }
            if top && matches!(kind, TokenKindV1::Comma) {
                slots = slots.checked_add(1).ok_or_else(|| {
                    ParseDiagnosticV1::new(
                        ParseErrorCodeV1::LexicalMaterializationOverflow,
                        span,
                        "projection item count overflow",
                    )
                })?;
            }
            match kind {
                TokenKindV1::LParen => parens += 1,
                TokenKindV1::LBrace => braces += 1,
                TokenKindV1::LBracket => brackets += 1,
                TokenKindV1::RParen if parens > 0 => parens -= 1,
                TokenKindV1::RBrace if braces > 0 => braces -= 1,
                TokenKindV1::RBracket if brackets > 0 => brackets -= 1,
                TokenKindV1::Eof => break,
                TokenKindV1::Ident(_)
                | TokenKindV1::Int(_)
                | TokenKindV1::Float(_)
                | TokenKindV1::Str(_)
                | TokenKindV1::RParen
                | TokenKindV1::RBrace
                | TokenKindV1::RBracket
                | TokenKindV1::Comma
                | TokenKindV1::Dot
                | TokenKindV1::Colon
                | TokenKindV1::Semicolon
                | TokenKindV1::Pipe
                | TokenKindV1::Arrow
                | TokenKindV1::ThinArrow
                | TokenKindV1::Op(_) => {}
            }
            offset = offset.checked_add(1).ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::LexicalMaterializationOverflow,
                    span,
                    "projection token offset overflow",
                )
            })?;
        }
        self.budget.charge_projection_element(slots, span)?;
        self.preallocate_exact_items_v3(slots, span)
    }

    /// Catalog/FIND comma lists contain only flat references or identifiers.
    /// Count their slots through the next clause/document boundary before the
    /// first request-owned Vec allocation.
    pub(crate) fn preallocate_clause_items_v3<T>(&mut self) -> Result<Vec<T>, ParseDiagnosticV1> {
        if !self.has_request_owner_v3() {
            return Ok(Vec::new());
        }
        let span = self.here_span();
        let mut slots = 1usize;
        let mut offset = 0usize;
        loop {
            match &self.peek_ahead(offset).kind {
                TokenKindV1::Pipe
                | TokenKindV1::Semicolon
                | TokenKindV1::RParen
                | TokenKindV1::RBrace
                | TokenKindV1::RBracket
                | TokenKindV1::Eof => break,
                TokenKindV1::Comma => {
                    slots = slots.checked_add(1).ok_or_else(|| {
                        ParseDiagnosticV1::new(
                            ParseErrorCodeV1::LexicalMaterializationOverflow,
                            span,
                            "clause item count overflow",
                        )
                    })?;
                }
                TokenKindV1::Ident(_)
                | TokenKindV1::Int(_)
                | TokenKindV1::Float(_)
                | TokenKindV1::Str(_)
                | TokenKindV1::LParen
                | TokenKindV1::LBrace
                | TokenKindV1::LBracket
                | TokenKindV1::Dot
                | TokenKindV1::Colon
                | TokenKindV1::Arrow
                | TokenKindV1::ThinArrow
                | TokenKindV1::Op(_) => {}
            }
            offset = offset.checked_add(1).ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::LexicalMaterializationOverflow,
                    span,
                    "clause token offset overflow",
                )
            })?;
        }
        self.budget.charge_collection_element(slots, span)?;
        self.preallocate_exact_items_v3(slots, span)
    }

    /// Run `body` one level deeper in the recursive-descent grammar, failing
    /// CLOSED if the nesting budget is exhausted BEFORE recursing.
    ///
    /// This is the fail-closed conversion of an otherwise-fatal stack overflow
    /// (a process abort on adversarially-nested input) into a typed, span-exact
    /// diagnostic — upholding the totality contract (every input yields `Ok` or
    /// a typed `Err`, never an unwind/abort). The depth is always restored after
    /// `body` returns (success OR error), so sibling nesting at the same level
    /// is unaffected and only true stack depth is bounded.
    ///
    /// Uses interior mutability for the counter so it composes with the
    /// `&mut self` recursive descent inside `body` without a borrow conflict.
    pub(crate) fn with_recursion_depth<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> Result<T, ParseDiagnosticV1>,
    ) -> Result<T, ParseDiagnosticV1> {
        self.budget.enter_expression_depth(self.here_span())?;
        let out = body(self);
        self.budget.leave_expression_depth();
        out
    }

    /// Charge one value/Bool node immediately before minting it.
    pub(crate) fn charge_expression_node(&self, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.budget.charge_expression_node(span)
    }

    /// The current token (EOF token if past the end).
    pub(crate) fn peek(&self) -> &TokenV1 {
        self.at(self.pos)
    }

    /// The token `n` positions ahead (clamped to EOF). `peek_ahead(0) == peek()`.
    pub(crate) fn peek_ahead(&self, n: usize) -> &TokenV1 {
        self.at(self.pos.saturating_add(n))
    }

    /// Whether the token `n` ahead is the identifier `word`.
    pub(crate) fn is_keyword_ahead(&self, n: usize, word: &str) -> bool {
        matches!(&self.peek_ahead(n).kind, TokenKindV1::Ident(s) if s == word)
    }

    fn at(&self, idx: usize) -> &TokenV1 {
        // Streams always end with a single Eof token; clamp to it.
        let last = self.tokens.len().saturating_sub(1);
        self.tokens.get(idx).unwrap_or_else(|| {
            self.tokens
                .get(last)
                .unwrap_or_else(|| Self::eof_fallback())
        })
    }

    fn eof_fallback() -> &'static TokenV1 {
        // Only reachable if constructed with an empty vec, which `parse` never
        // does. Provide a stable static so this stays panic-free.
        static EOF: TokenV1 = TokenV1 {
            kind: TokenKindV1::Eof,
            span: SpanV1::point(0),
        };
        &EOF
    }

    /// Whether the current token is EOF.
    pub(crate) fn at_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKindV1::Eof)
    }

    /// Advance past and return the current token.
    pub(crate) fn bump(&mut self) -> TokenV1 {
        let Some(slot) = self.tokens.get_mut(self.pos) else {
            return TokenV1 {
                kind: TokenKindV1::Eof,
                span: self.at(self.pos).span,
            };
        };
        // A consumed slot is never revisited. Move its owned payload into the
        // parser instead of cloning a String before the HIR owner can use it.
        let span = slot.span;
        let token = std::mem::replace(
            slot,
            TokenV1 {
                kind: TokenKindV1::Eof,
                span,
            },
        );
        self.pos += 1;
        token
    }

    /// The span of the current token (for diagnostics).
    pub(crate) fn here_span(&self) -> SpanV1 {
        self.peek().span
    }

    /// Consume the current token if its kind matches `kind` exactly; else `None`.
    pub(crate) fn eat(&mut self, kind: &TokenKindV1) -> Option<TokenV1> {
        if &self.peek().kind == kind {
            Some(self.bump())
        } else {
            None
        }
    }

    /// Consume the current token if it is the identifier `word` (keyword match).
    pub(crate) fn eat_keyword(&mut self, word: &str) -> Option<TokenV1> {
        if self.is_keyword(word) {
            Some(self.bump())
        } else {
            None
        }
    }

    /// Whether the current token is the identifier `word`.
    pub(crate) fn is_keyword(&self, word: &str) -> bool {
        matches!(&self.peek().kind, TokenKindV1::Ident(s) if s == word)
    }

    /// Require the keyword identifier `word`, else a typed
    /// [`ParseDiagnosticV1`] at the current token span.
    pub(crate) fn expect_keyword(
        &mut self,
        word: &str,
        what: &str,
    ) -> Result<TokenV1, ParseDiagnosticV1> {
        if self.is_keyword(word) {
            Ok(self.bump())
        } else {
            Err(self.unexpected(what))
        }
    }

    /// Require an exact token kind, else a typed [`ParseDiagnosticV1`].
    pub(crate) fn expect(
        &mut self,
        kind: &TokenKindV1,
        what: &str,
    ) -> Result<TokenV1, ParseDiagnosticV1> {
        if &self.peek().kind == kind {
            Ok(self.bump())
        } else {
            Err(self.unexpected(what))
        }
    }

    /// Require an operator without allocating a temporary owned token kind.
    pub(crate) fn expect_op(
        &mut self,
        symbol: &str,
        what: &str,
    ) -> Result<TokenV1, ParseDiagnosticV1> {
        if matches!(&self.peek().kind, TokenKindV1::Op(value) if value == symbol) {
            Ok(self.bump())
        } else {
            Err(self.unexpected(what))
        }
    }

    /// Require an identifier token, returning its `(text, span)`.
    pub(crate) fn expect_ident(
        &mut self,
        what: &str,
    ) -> Result<(String, SpanV1), ParseDiagnosticV1> {
        if !matches!(&self.peek().kind, TokenKindV1::Ident(_)) {
            return Err(self.unexpected(what));
        }
        let token = self.bump();
        if let TokenKindV1::Ident(name) = token.kind {
            Ok((name, token.span))
        } else {
            Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedToken,
                token.span,
                "identifier lookahead changed during consumption",
            ))
        }
    }

    /// Build an `UnexpectedToken`/`UnexpectedEof` diagnostic at the cursor with
    /// the exact current-token span.
    pub(crate) fn unexpected(&self, what: &str) -> ParseDiagnosticV1 {
        let span = self.peek().span;
        if self.at_eof() {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedEof,
                span,
                format!("expected {what} but reached end of input"),
            )
        } else {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedToken,
                span,
                format!("expected {what}"),
            )
        }
    }

    /// Build an explicit `UnsupportedConstruct` reject at `span`. Used by the
    /// no-silent-downgrade guard so a recognized-but-rejected form fails closed
    /// rather than reparsing leniently (D10).
    pub(crate) fn unsupported(
        &self,
        span: SpanV1,
        message: impl Into<String>,
    ) -> ParseDiagnosticV1 {
        ParseDiagnosticV1::new(ParseErrorCodeV1::UnsupportedConstruct, span, message)
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "unit tests assert exact parser outcomes"
    )]

    use super::*;
    use crate::budget::ParseBudgetV2;

    #[test]
    fn consuming_identifier_moves_lexer_payload_without_copy() {
        let name = String::from("source_name");
        let original_payload = name.as_ptr();
        let span = SpanV1::new(0, name.len());
        let builder = HirBuilderV1::new();
        let mut cursor = CursorV1::new(
            vec![
                TokenV1 {
                    kind: TokenKindV1::Ident(name),
                    span,
                },
                TokenV1 {
                    kind: TokenKindV1::Eof,
                    span: SpanV1::point(span.end),
                },
            ],
            &builder,
            ParseBudgetCounterV2::new(&ParseBudgetV2::standard_v2()),
        );

        let (consumed, consumed_span) = cursor.expect_ident("source").expect("identifier");
        assert_eq!(consumed.as_ptr(), original_payload);
        assert_eq!(consumed_span, span);
        assert!(cursor.at_eof());
    }

    #[test]
    fn legacy_reference_materialization_rejects_invalid_span() {
        let builder = HirBuilderV1::new();
        let mut cursor = CursorV1::new(
            vec![TokenV1 {
                kind: TokenKindV1::Eof,
                span: SpanV1::point(0),
            }],
            &builder,
            ParseBudgetCounterV2::new(&ParseBudgetV2::standard_v2()),
        );
        let error = cursor
            .materialize_reference_v1("foo", SpanV1::new(0, 4))
            .expect_err("invalid source span cannot become an empty reference");
        assert_eq!(error.code, ParseErrorCodeV1::UnexpectedToken);
    }
}
