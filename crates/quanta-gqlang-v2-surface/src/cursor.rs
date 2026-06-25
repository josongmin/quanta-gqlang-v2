//! A token cursor shared by every sub-parser.
//!
//! Holds the token stream plus a position, and threads the single
//! `hir::HirBuilderV1` that mints node ids so the whole parse produces one
//! id-unique tree. All lookahead/consume helpers are fail-closed: requesting a
//! token past EOF returns the EOF token (never panics), and `expect_*` helpers
//! return typed [`ParseDiagnosticV1`] on mismatch.

use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};
use crate::lexer::{TokenKindV1, TokenV1};
use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};
use std::cell::Cell;

/// Maximum recursive-descent nesting depth.
///
/// The expression grammar is recursive (`(`, `[`, `{` each recurse into the
/// value/bool grammar), so adversarial input with thousands of nested brackets
/// would otherwise overflow the native stack and ABORT the process — a
/// fail-open totality violation. Every recursion entry checks this budget and
/// fails CLOSED with a typed [`ParseErrorCodeV1::UnsupportedConstruct`]
/// diagnostic instead, so the public API stays total (`Ok` or typed `Err`,
/// never an unwind/abort). The bound is far above any hand-written program yet
/// well under the stack frames available, so legal input is unaffected.
const MAX_RECURSION_DEPTH: usize = 256;

/// Shared parsing state: token stream + position + the id-allocating builder.
pub(crate) struct CursorV1<'b> {
    tokens: Vec<TokenV1>,
    pos: usize,
    /// Current recursive-descent nesting depth (fail-closed against stack
    /// exhaustion on adversarially-nested input).
    depth: Cell<usize>,
    /// The single builder for the whole parse; borrowed by every sub-parser.
    pub(crate) builder: &'b HirBuilderV1,
}

impl<'b> CursorV1<'b> {
    /// Construct over a non-empty token stream (must end in `Eof`).
    pub(crate) fn new(tokens: Vec<TokenV1>, builder: &'b HirBuilderV1) -> Self {
        Self {
            tokens,
            pos: 0,
            depth: Cell::new(0),
            builder,
        }
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
        let next = self.depth.get().saturating_add(1);
        if next > MAX_RECURSION_DEPTH {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnsupportedConstruct,
                self.here_span(),
                format!(
                    "expression nesting exceeds the maximum supported depth of \
                     {MAX_RECURSION_DEPTH}"
                ),
            ));
        }
        self.depth.set(next);
        let out = body(self);
        self.depth.set(self.depth.get().saturating_sub(1));
        out
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
        let tok = self.at(self.pos).clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
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

    /// Require an identifier token, returning its `(text, span)`.
    pub(crate) fn expect_ident(
        &mut self,
        what: &str,
    ) -> Result<(String, SpanV1), ParseDiagnosticV1> {
        if let TokenKindV1::Ident(s) = &self.peek().kind {
            let s = s.clone();
            let span = self.peek().span;
            let _ = self.bump();
            return Ok((s, span));
        }
        Err(self.unexpected(what))
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
