//! Byte-exact tokenizer for the v3 surface language.
//!
//! The lexer turns source text into a flat [`Vec`] of [`TokenV1`], each carrying
//! a `hir::SpanV1` byte range into the ORIGINAL source. Spans are load-bearing:
//! every HIR node the parser mints copies a token (or merged token) span, so
//! diagnostics and the formatter map back to exact source offsets (ticket 06
//! "exact span behavior").
//!
//! The lexer is fail-closed: an unexpected byte or an unterminated string yields
//! a typed [`LexErrorV1`] with a stable code and an exact span, never a panic
//! and never a silent skip. This upholds D10 (no silent downgrade) at the
//! lexical layer.

use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};
use quanta_gqlang_v2_hir::SpanV1;

/// A lexical token kind. Operator and keyword text is retained verbatim so the
/// parser can copy operator symbols into the open HIR operator tags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKindV1 {
    /// An identifier or keyword (kept as raw text; keyword classification is the
    /// parser's job so the lexer needs no keyword table).
    Ident(String),
    /// An integer literal (raw source text).
    Int(String),
    /// A floating/decimal literal (raw source text).
    Float(String),
    /// A string literal (already-unescaped content).
    Str(String),
    /// `(`.
    LParen,
    /// `)`.
    RParen,
    /// `{`.
    LBrace,
    /// `}`.
    RBrace,
    /// `[`.
    LBracket,
    /// `]`.
    RBracket,
    /// `,`.
    Comma,
    /// `.`.
    Dot,
    /// `:`.
    Colon,
    /// `;`, the document-level statement separator (multi-root documents).
    Semicolon,
    /// `|`, the pipeline-stage separator.
    Pipe,
    /// `=>`, the result-shape arrow.
    Arrow,
    /// `->`, the FIND source-to-sink seed arrow.
    ThinArrow,
    /// An operator symbol (`+ - * / = <> < <= > >=`). The exact symbol text is
    /// kept so it lowers into the open HIR operator tag verbatim.
    Op(String),
    /// End of input. Always the final token; carries a zero-width span at the
    /// source end so "unexpected EOF" diagnostics still have an exact offset.
    Eof,
}

/// A token with its exact source span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenV1 {
    /// Token classification and payload.
    pub kind: TokenKindV1,
    /// Exact byte range into the original source.
    pub span: SpanV1,
}

/// A typed lexical error carrying an exact span. Re-uses the shared parse
/// diagnostic taxonomy so callers see one error type across lex and parse.
pub type LexErrorV1 = ParseDiagnosticV1;

/// Tokenize `source` into tokens terminated by a single [`TokenKindV1::Eof`].
///
/// Returns the first typed [`LexErrorV1`] on an illegal byte or unterminated
/// string. Whitespace and `# line comments` are skipped; their spans never
/// surface in tokens.
pub fn lex(source: &str) -> Result<Vec<TokenV1>, LexErrorV1> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut i = 0usize;

    while i < len {
        let b = byte_at(bytes, i);
        // whitespace
        if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' {
            i += 1;
            continue;
        }
        // line comment
        if b == b'#' {
            while i < len && byte_at(bytes, i) != b'\n' {
                i += 1;
            }
            continue;
        }
        // string literal
        if b == b'"' {
            let (tok, next) = lex_string(bytes, i)?;
            tokens.push(tok);
            i = next;
            continue;
        }
        // number
        if b.is_ascii_digit() {
            let (tok, next) = lex_number(source, bytes, i);
            tokens.push(tok);
            i = next;
            continue;
        }
        // identifier / keyword
        if is_ident_start(b) {
            let (tok, next) = lex_ident(source, bytes, i);
            tokens.push(tok);
            i = next;
            continue;
        }
        // punctuation / operators
        if let Some((tok, next)) = lex_symbol(bytes, i) {
            tokens.push(tok);
            i = next;
            continue;
        }
        // unknown byte: fail closed with exact span
        return Err(ParseDiagnosticV1::new(
            ParseErrorCodeV1::UnexpectedChar,
            SpanV1::new(i, i + 1),
            format!("unexpected character '{}'", char_at(source, i)),
        ));
    }

    tokens.push(TokenV1 {
        kind: TokenKindV1::Eof,
        span: SpanV1::point(len),
    });
    Ok(tokens)
}

fn lex_string(bytes: &[u8], start: usize) -> Result<(TokenV1, usize), LexErrorV1> {
    let len = bytes.len();
    let mut i = start + 1; // skip opening quote
    let mut content = String::new();
    while i < len {
        let b = byte_at(bytes, i);
        if b == b'"' {
            let span = SpanV1::new(start, i + 1);
            return Ok((
                TokenV1 {
                    kind: TokenKindV1::Str(content),
                    span,
                },
                i + 1,
            ));
        }
        if b == b'\\' {
            // minimal escape handling: \" \\ \n \t
            i += 1;
            if i >= len {
                break;
            }
            let e = byte_at(bytes, i);
            match e {
                b'"' => content.push('"'),
                b'\\' => content.push('\\'),
                b'n' => content.push('\n'),
                b't' => content.push('\t'),
                other => content.push(char::from(other)),
            }
            i += 1;
            continue;
        }
        content.push(char::from(b));
        i += 1;
    }
    Err(ParseDiagnosticV1::new(
        ParseErrorCodeV1::UnterminatedString,
        SpanV1::new(start, len),
        "unterminated string literal".to_string(),
    ))
}

fn lex_number(source: &str, bytes: &[u8], start: usize) -> (TokenV1, usize) {
    let len = bytes.len();
    let mut i = start;
    while i < len && byte_at(bytes, i).is_ascii_digit() {
        i += 1;
    }
    let mut is_float = false;
    // fractional part, only when a digit follows the dot (so `a.b` paths and
    // `1.field` never get eaten as floats by accident).
    if i + 1 < len && byte_at(bytes, i) == b'.' && byte_at(bytes, i + 1).is_ascii_digit() {
        is_float = true;
        i += 1;
        while i < len && byte_at(bytes, i).is_ascii_digit() {
            i += 1;
        }
    }
    let text = slice(source, start, i);
    let kind = if is_float {
        TokenKindV1::Float(text)
    } else {
        TokenKindV1::Int(text)
    };
    (
        TokenV1 {
            kind,
            span: SpanV1::new(start, i),
        },
        i,
    )
}

fn lex_ident(source: &str, bytes: &[u8], start: usize) -> (TokenV1, usize) {
    let len = bytes.len();
    let mut i = start;
    while i < len && is_ident_continue(byte_at(bytes, i)) {
        i += 1;
    }
    let text = slice(source, start, i);
    (
        TokenV1 {
            kind: TokenKindV1::Ident(text),
            span: SpanV1::new(start, i),
        },
        i,
    )
}

fn lex_symbol(bytes: &[u8], start: usize) -> Option<(TokenV1, usize)> {
    let len = bytes.len();
    let b = byte_at(bytes, start);
    let two = if start + 1 < len {
        Some(byte_at(bytes, start + 1))
    } else {
        None
    };
    // two-byte operators first
    if let Some(second) = two {
        let pair = [b, second];
        let twostr: Option<&str> = match &pair {
            b"=>" => return Some(make(TokenKindV1::Arrow, start, 2)),
            b"->" => return Some(make(TokenKindV1::ThinArrow, start, 2)),
            b"<=" => Some("<="),
            b">=" => Some(">="),
            b"<>" => Some("<>"),
            b"!=" => Some("!="),
            _ => None,
        };
        if let Some(sym) = twostr {
            return Some(make(TokenKindV1::Op(sym.to_string()), start, 2));
        }
    }
    let single = match b {
        b'(' => TokenKindV1::LParen,
        b')' => TokenKindV1::RParen,
        b'{' => TokenKindV1::LBrace,
        b'}' => TokenKindV1::RBrace,
        b'[' => TokenKindV1::LBracket,
        b']' => TokenKindV1::RBracket,
        b',' => TokenKindV1::Comma,
        b'.' => TokenKindV1::Dot,
        b':' => TokenKindV1::Colon,
        b';' => TokenKindV1::Semicolon,
        b'|' => TokenKindV1::Pipe,
        b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' => {
            TokenKindV1::Op(char::from(b).to_string())
        }
        _ => return None,
    };
    Some(make(single, start, 1))
}

fn make(kind: TokenKindV1, start: usize, width: usize) -> (TokenV1, usize) {
    (
        TokenV1 {
            kind,
            span: SpanV1::new(start, start + width),
        },
        start + width,
    )
}

fn is_ident_start(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphabetic()
}

fn is_ident_continue(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

fn byte_at(bytes: &[u8], i: usize) -> u8 {
    // Bounds are guaranteed by callers; fall back to 0 instead of indexing to
    // keep `indexing_slicing` clean without changing behavior.
    bytes.get(i).copied().unwrap_or(0)
}

fn slice(source: &str, start: usize, end: usize) -> String {
    source.get(start..end).unwrap_or_default().to_string()
}

fn char_at(source: &str, i: usize) -> char {
    source
        .get(i..)
        .and_then(|s| s.chars().next())
        .unwrap_or('?')
}
