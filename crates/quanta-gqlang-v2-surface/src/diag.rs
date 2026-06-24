//! Typed parse diagnostics for the v3 surface parser.
//!
//! Every reject path in this crate (lex or parse) produces a
//! [`ParseDiagnosticV1`] carrying a stable machine code ([`ParseErrorCodeV1`])
//! plus an EXACT `hir::SpanV1` byte range. This is the fail-closed contract from
//! ticket 06 scenario 4 and packet decision D10: a malformed surface program
//! rejects with a typed diagnostic and an exact span, and the parser never
//! silently downgrades to a narrower v2-style subset.

use quanta_gqlang_v2_hir::SpanV1;
use std::fmt;

/// Stable, machine-readable parse/lex error codes.
///
/// The `code()` string of each variant is part of this crate's public contract:
/// downstream LSP/CLI/tests match on these without re-reading parser internals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseErrorCodeV1 {
    /// The lexer hit a byte that begins no legal token.
    UnexpectedChar,
    /// A string literal was opened but never closed.
    UnterminatedString,
    /// A required token was missing (e.g. a closing paren, a clause keyword).
    UnexpectedToken,
    /// Input ended while a construct was still open.
    UnexpectedEof,
    /// A construct legal in some other position is not legal here, OR a
    /// recognized-but-rejected v2-ism. Used by the no-silent-downgrade guard so
    /// rejects are explicit rather than a lenient reparse.
    UnsupportedConstruct,
}

impl ParseErrorCodeV1 {
    /// The stable diagnostic code string.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnexpectedChar => "surface.parse.unexpected_char",
            Self::UnterminatedString => "surface.parse.unterminated_string",
            Self::UnexpectedToken => "surface.parse.unexpected_token",
            Self::UnexpectedEof => "surface.parse.unexpected_eof",
            Self::UnsupportedConstruct => "surface.parse.unsupported_construct",
        }
    }
}

/// A typed parse/lex diagnostic with an exact span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseDiagnosticV1 {
    /// Stable error classification.
    pub code: ParseErrorCodeV1,
    /// Exact byte range the diagnostic points at.
    pub span: SpanV1,
    /// Human-readable message (the code is the stable contract, not this text).
    pub message: String,
}

impl ParseDiagnosticV1 {
    /// Construct a diagnostic.
    #[must_use]
    pub fn new(code: ParseErrorCodeV1, span: SpanV1, message: impl Into<String>) -> Self {
        Self {
            code,
            span,
            message: message.into(),
        }
    }

    /// The stable diagnostic code string.
    #[must_use]
    pub const fn code_str(&self) -> &'static str {
        self.code.code()
    }
}

impl fmt::Display for ParseDiagnosticV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} (at {}..{})",
            self.code.code(),
            self.message,
            self.span.start,
            self.span.end,
        )
    }
}

impl std::error::Error for ParseDiagnosticV1 {}
