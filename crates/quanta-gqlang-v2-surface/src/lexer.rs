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

use crate::budget::{ParseBudgetCounterV2, ParseBudgetV2};
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

/// Source-derived logical storage required by the lexer before it creates a
/// token vector or copies any token payload. Token storage includes the single
/// Vec header and its token slots. This covers the lexer only; HIR,
/// semantic, and plan allocations require their own admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LexicalMaterializationCensusV1 {
    token_count: usize,
    token_storage_bytes: usize,
    payload_bytes: usize,
    logical_bytes: usize,
}

impl LexicalMaterializationCensusV1 {
    #[must_use]
    pub const fn token_count(self) -> usize {
        self.token_count
    }

    #[must_use]
    pub const fn token_storage_bytes(self) -> usize {
        self.token_storage_bytes
    }

    #[must_use]
    pub const fn payload_bytes(self) -> usize {
        self.payload_bytes
    }

    #[must_use]
    pub const fn logical_bytes(self) -> usize {
        self.logical_bytes
    }
}

/// Validate and count the lexer's exact logical token and payload storage
/// without allocating a successful token or payload. Escape decoding uses the
/// same walker as the emitting lexer, including Unicode escape decoding.
pub fn census_lexical_materialization_v1(
    source: &str,
    budget: &ParseBudgetV2,
) -> Result<LexicalMaterializationCensusV1, LexErrorV1> {
    let counter = ParseBudgetCounterV2::new(budget);
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut token_count = 0usize;
    let mut payload_bytes = 0usize;
    while i < bytes.len() {
        let b = byte_at(bytes, i);
        if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
            i += 1;
            continue;
        }
        if b == b'#' {
            while i < bytes.len() && byte_at(bytes, i) != b'\n' {
                i += 1;
            }
            continue;
        }
        let (next, payload) = if b == b'"' {
            scan_string(source, i)?
        } else if b.is_ascii_digit() {
            let (next, _) = scan_number(bytes, i);
            (next, next - i)
        } else if is_ident_start(b) {
            let next = scan_ident(bytes, i);
            (next, next - i)
        } else if let Some((symbol, next)) = lex_symbol_spec(bytes, i) {
            (next, symbol.payload_bytes())
        } else {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::UnexpectedChar,
                SpanV1::new(i, i + char_at(source, i).len_utf8()),
                format!("unexpected character '{}'", char_at(source, i)),
            ));
        };
        let span = SpanV1::new(i, next);
        counter.charge_token(span)?;
        token_count = token_count
            .checked_add(1)
            .ok_or_else(|| census_overflow(span))?;
        payload_bytes = payload_bytes
            .checked_add(payload)
            .ok_or_else(|| census_overflow(span))?;
        i = next;
    }
    let eof = SpanV1::point(source.len());
    counter.charge_token(eof)?;
    token_count = token_count
        .checked_add(1)
        .ok_or_else(|| census_overflow(eof))?;
    let token_storage_bytes = token_count
        .checked_mul(std::mem::size_of::<TokenV1>())
        .and_then(|slots| slots.checked_add(std::mem::size_of::<Vec<TokenV1>>()))
        .ok_or_else(|| census_overflow(eof))?;
    let logical_bytes = token_storage_bytes
        .checked_add(payload_bytes)
        .ok_or_else(|| census_overflow(eof))?;
    Ok(LexicalMaterializationCensusV1 {
        token_count,
        token_storage_bytes,
        payload_bytes,
        logical_bytes,
    })
}

fn census_overflow(span: SpanV1) -> LexErrorV1 {
    ParseDiagnosticV1::new(
        ParseErrorCodeV1::LexicalMaterializationOverflow,
        span,
        "lexical materialization size overflow",
    )
}

/// Tokenize `source` into tokens terminated by a single [`TokenKindV1::Eof`].
///
/// Returns the first typed [`LexErrorV1`] on an illegal byte or unterminated
/// string. Whitespace and `# line comments` are skipped; their spans never
/// surface in tokens.
pub fn lex(source: &str) -> Result<Vec<TokenV1>, LexErrorV1> {
    let counter = ParseBudgetCounterV2::new(&ParseBudgetV2::standard_v2());
    lex_with_budget(source, &counter)
}

pub(crate) fn lex_with_budget(
    source: &str,
    counter: &ParseBudgetCounterV2,
) -> Result<Vec<TokenV1>, LexErrorV1> {
    lex_with_preallocated_slots_v1(source, counter, None)
}

/// The caller must hold a permit for this exact census before calling. The
/// token vector is allocated once before any payload is copied or decoded.
pub(crate) fn lex_with_census_v1(
    source: &str,
    counter: &ParseBudgetCounterV2,
    census: LexicalMaterializationCensusV1,
) -> Result<Vec<TokenV1>, LexErrorV1> {
    lex_with_preallocated_slots_v1(source, counter, Some(census))
}

fn lex_with_preallocated_slots_v1(
    source: &str,
    counter: &ParseBudgetCounterV2,
    admitted_census: Option<LexicalMaterializationCensusV1>,
) -> Result<Vec<TokenV1>, LexErrorV1> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    if let Some(slots) = admitted_census.map(LexicalMaterializationCensusV1::token_count) {
        tokens.try_reserve_exact(slots).map_err(|_error| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                SpanV1::new(0, len),
                "lexical token storage allocation failed",
            )
        })?;
        if tokens.capacity() > slots {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                SpanV1::new(0, len),
                "lexical token allocator exceeded admitted slots",
            ));
        }
    }
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
            let (next, decoded_bytes) = scan_string(source, i)?;
            let span = SpanV1::new(i, next);
            counter.charge_token(span)?;
            let content = decode_string(source, i, next, decoded_bytes)?;
            tokens.push(TokenV1 {
                kind: TokenKindV1::Str(content),
                span,
            });
            i = next;
            continue;
        }
        // number
        if b.is_ascii_digit() {
            let (next, is_float) = scan_number(bytes, i);
            let span = SpanV1::new(i, next);
            counter.charge_token(span)?;
            let text = copy_payload(source, i, next);
            tokens.push(TokenV1 {
                kind: if is_float {
                    TokenKindV1::Float(text)
                } else {
                    TokenKindV1::Int(text)
                },
                span,
            });
            i = next;
            continue;
        }
        // identifier / keyword
        if is_ident_start(b) {
            let next = scan_ident(bytes, i);
            let span = SpanV1::new(i, next);
            counter.charge_token(span)?;
            let text = copy_payload(source, i, next);
            tokens.push(TokenV1 {
                kind: TokenKindV1::Ident(text),
                span,
            });
            i = next;
            continue;
        }
        // punctuation / operators
        if let Some((symbol, next)) = lex_symbol_spec(bytes, i) {
            let span = SpanV1::new(i, next);
            counter.charge_token(span)?;
            tokens.push(materialize_symbol(symbol, i, next));
            i = next;
            continue;
        }
        // unknown byte: fail closed with exact span
        return Err(ParseDiagnosticV1::new(
            ParseErrorCodeV1::UnexpectedChar,
            SpanV1::new(i, i + char_at(source, i).len_utf8()),
            format!("unexpected character '{}'", char_at(source, i)),
        ));
    }

    let eof = TokenV1 {
        kind: TokenKindV1::Eof,
        span: SpanV1::point(len),
    };
    counter.charge_token(eof.span)?;
    tokens.push(eof);
    if let Some(census) = admitted_census {
        let emitted_payload = tokens.iter().try_fold(0usize, |bytes, token| {
            let next = if let TokenKindV1::Ident(text)
            | TokenKindV1::Int(text)
            | TokenKindV1::Float(text)
            | TokenKindV1::Str(text)
            | TokenKindV1::Op(text) = &token.kind
            {
                if text.capacity() != text.len() {
                    return None;
                }
                text.len()
            } else {
                0
            };
            bytes.checked_add(next)
        });
        if tokens.len() != census.token_count() || emitted_payload != Some(census.payload_bytes()) {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                SpanV1::point(len),
                "lexical census disagrees with emitted token or payload count",
            ));
        }
    }
    Ok(tokens)
}

fn scan_string(source: &str, start: usize) -> Result<(usize, usize), LexErrorV1> {
    let mut decoded_bytes = Some(0usize);
    let next = walk_string(source, start, |ch| {
        decoded_bytes = decoded_bytes.and_then(|bytes| bytes.checked_add(ch.len_utf8()));
    })?;
    Ok((
        next,
        decoded_bytes.ok_or_else(|| census_overflow(SpanV1::new(start, next)))?,
    ))
}

/// One escape grammar serves the allocation-free scan and admitted decoding.
fn walk_string(
    source: &str,
    start: usize,
    mut emit: impl FnMut(char),
) -> Result<usize, LexErrorV1> {
    let len = source.len();
    let mut i = start + 1; // skip opening quote
    while i < len {
        let Some(ch) = source.get(i..).and_then(|tail| tail.chars().next()) else {
            break;
        };
        if ch == '"' {
            return Ok(i + ch.len_utf8());
        }
        if ch == '\\' {
            let escape_start = i;
            i += ch.len_utf8();
            if i >= len {
                break;
            }
            let Some(escape) = source.get(i..).and_then(|tail| tail.chars().next()) else {
                break;
            };
            match escape {
                '"' => emit('"'),
                '\\' => emit('\\'),
                'n' => emit('\n'),
                'r' => emit('\r'),
                't' => emit('\t'),
                'b' => emit('\u{0008}'),
                'f' => emit('\u{000c}'),
                'u' => {
                    i += escape.len_utf8();
                    let open = source.get(i..).and_then(|tail| tail.chars().next());
                    if open != Some('{') {
                        return Err(invalid_escape(
                            source,
                            escape_start,
                            i,
                            "expected '{' after \\u",
                        ));
                    }
                    i += 1;
                    let digits_start = i;
                    while i < len {
                        let Some(digit) = source.get(i..).and_then(|tail| tail.chars().next())
                        else {
                            break;
                        };
                        if digit == '}' {
                            break;
                        }
                        if !digit.is_ascii_hexdigit() || i.saturating_sub(digits_start) >= 6 {
                            return Err(invalid_escape(
                                source,
                                escape_start,
                                i + digit.len_utf8(),
                                "invalid Unicode escape",
                            ));
                        }
                        i += digit.len_utf8();
                    }
                    if i == digits_start
                        || source.get(i..).and_then(|tail| tail.chars().next()) != Some('}')
                    {
                        return Err(invalid_escape(
                            source,
                            escape_start,
                            i,
                            "invalid Unicode escape",
                        ));
                    }
                    let digits = source.get(digits_start..i).unwrap_or_default();
                    let value = u32::from_str_radix(digits, 16).map_err(|_invalid_hex| {
                        invalid_escape(source, escape_start, i + 1, "invalid Unicode escape")
                    })?;
                    let scalar = char::from_u32(value).ok_or_else(|| {
                        invalid_escape(
                            source,
                            escape_start,
                            i + 1,
                            "Unicode escape is not a scalar value",
                        )
                    })?;
                    emit(scalar);
                    i += 1;
                    continue;
                }
                _ => {
                    return Err(invalid_escape(
                        source,
                        escape_start,
                        i + escape.len_utf8(),
                        "unknown string escape",
                    ));
                }
            }
            i += escape.len_utf8();
            continue;
        }
        emit(ch);
        i += ch.len_utf8();
    }
    Err(ParseDiagnosticV1::new(
        ParseErrorCodeV1::UnterminatedString,
        SpanV1::new(start, len),
        "unterminated string literal".to_string(),
    ))
}

fn decode_string(
    source: &str,
    start: usize,
    end: usize,
    decoded_bytes: usize,
) -> Result<String, LexErrorV1> {
    note_payload_allocation();
    // The admitted census counts decoded bytes; escaped source bytes may be
    // much longer. Allocate that exact logical demand before copying payload.
    let mut content = String::with_capacity(decoded_bytes);
    if content.capacity() > decoded_bytes {
        return Err(ParseDiagnosticV1::new(
            ParseErrorCodeV1::ResourceExhausted,
            SpanV1::new(start, end),
            "string allocator exceeded admitted payload bytes",
        ));
    }
    let observed_end = walk_string(source, start, |ch| content.push(ch))?;
    if observed_end != end || content.len() != decoded_bytes {
        return Err(census_overflow(SpanV1::new(start, end)));
    }
    Ok(content)
}

fn invalid_escape(source: &str, start: usize, end: usize, message: &str) -> LexErrorV1 {
    ParseDiagnosticV1::new(
        ParseErrorCodeV1::UnexpectedChar,
        SpanV1::new(start, end.min(source.len())),
        message,
    )
}

fn scan_number(bytes: &[u8], start: usize) -> (usize, bool) {
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
    (i, is_float)
}

fn scan_ident(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut i = start;
    while i < len && is_ident_continue(byte_at(bytes, i)) {
        i += 1;
    }
    i
}

enum LexSymbolSpecV1 {
    Punctuation(TokenKindV1),
    Operator(&'static str),
}

impl LexSymbolSpecV1 {
    fn payload_bytes(&self) -> usize {
        match self {
            Self::Punctuation(_) => 0,
            Self::Operator(symbol) => symbol.len(),
        }
    }
}

fn materialize_symbol(symbol: LexSymbolSpecV1, start: usize, next: usize) -> TokenV1 {
    let kind = match symbol {
        LexSymbolSpecV1::Punctuation(kind) => kind,
        LexSymbolSpecV1::Operator(text) => {
            note_payload_allocation();
            TokenKindV1::Op(text.to_owned())
        }
    };
    TokenV1 {
        kind,
        span: SpanV1::new(start, next),
    }
}

fn lex_symbol_spec(bytes: &[u8], start: usize) -> Option<(LexSymbolSpecV1, usize)> {
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
            b"=>" => return Some((LexSymbolSpecV1::Punctuation(TokenKindV1::Arrow), start + 2)),
            b"->" => {
                return Some((
                    LexSymbolSpecV1::Punctuation(TokenKindV1::ThinArrow),
                    start + 2,
                ))
            }
            b"<=" => Some("<="),
            b">=" => Some(">="),
            b"<>" => Some("<>"),
            b"!=" => Some("!="),
            _ => None,
        };
        if let Some(sym) = twostr {
            return Some((LexSymbolSpecV1::Operator(sym), start + 2));
        }
    }
    let single = match b {
        b'(' => LexSymbolSpecV1::Punctuation(TokenKindV1::LParen),
        b')' => LexSymbolSpecV1::Punctuation(TokenKindV1::RParen),
        b'{' => LexSymbolSpecV1::Punctuation(TokenKindV1::LBrace),
        b'}' => LexSymbolSpecV1::Punctuation(TokenKindV1::RBrace),
        b'[' => LexSymbolSpecV1::Punctuation(TokenKindV1::LBracket),
        b']' => LexSymbolSpecV1::Punctuation(TokenKindV1::RBracket),
        b',' => LexSymbolSpecV1::Punctuation(TokenKindV1::Comma),
        b'.' => LexSymbolSpecV1::Punctuation(TokenKindV1::Dot),
        b':' => LexSymbolSpecV1::Punctuation(TokenKindV1::Colon),
        b';' => LexSymbolSpecV1::Punctuation(TokenKindV1::Semicolon),
        b'|' => LexSymbolSpecV1::Punctuation(TokenKindV1::Pipe),
        b'+' => LexSymbolSpecV1::Operator("+"),
        b'-' => LexSymbolSpecV1::Operator("-"),
        b'*' => LexSymbolSpecV1::Operator("*"),
        b'/' => LexSymbolSpecV1::Operator("/"),
        b'=' => LexSymbolSpecV1::Operator("="),
        b'<' => LexSymbolSpecV1::Operator("<"),
        b'>' => LexSymbolSpecV1::Operator(">"),
        _ => return None,
    };
    Some((single, start + 1))
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

fn copy_payload(source: &str, start: usize, end: usize) -> String {
    note_payload_allocation();
    source.get(start..end).unwrap_or_default().to_string()
}

#[cfg(not(test))]
fn note_payload_allocation() {}

#[cfg(test)]
thread_local! {
    static PAYLOAD_ALLOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn note_payload_allocation() {
    PAYLOAD_ALLOCATIONS.with(|count| count.set(count.get() + 1));
}

fn char_at(source: &str, i: usize) -> char {
    source
        .get(i..)
        .and_then(|s| s.chars().next())
        .unwrap_or('?')
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unwrap_used,
        reason = "unit rail asserts exact budget construction and rejection"
    )]

    use super::{
        census_lexical_materialization_v1, decode_string, lex_with_budget, scan_string,
        TokenKindV1, TokenV1, PAYLOAD_ALLOCATIONS,
    };
    use crate::budget::ParseBudgetCounterV2;
    use crate::{ParseBudgetDimensionV2, ParseBudgetV2, ParseDiagnosticV1, ParseErrorCodeV1};
    use quanta_gqlang_v2_hir::SpanV1;

    #[test]
    fn string_walk_preserves_all_escape_payloads_and_utf8_offsets() {
        for (quoted, decoded) in [
            ("\"\"", ""),
            (r#""a\"b\\c\n\r\t\b\f""#, "a\"b\\c\n\r\t\u{0008}\u{000c}"),
            (
                r#""\u{0}\u{7f}\u{80}\u{d7ff}\u{e000}\u{10ffff}""#,
                "\0\u{7f}\u{80}\u{d7ff}\u{e000}\u{10ffff}",
            ),
            ("\"é🙂\"", "é🙂"),
        ] {
            let source = format!("é {quoted}");
            assert_eq!(scan_string(&source, 3), Ok((source.len(), decoded.len())));
            assert_eq!(
                decode_string(&source, 3, source.len(), decoded.len()),
                Ok(decoded.to_owned())
            );
        }
    }

    #[test]
    fn decoding_without_a_scanner_cannot_drop_invalid_escape_input() {
        for (quoted, code, message, start, end) in [
            (
                r#""\q""#,
                ParseErrorCodeV1::UnexpectedChar,
                "unknown string escape",
                1,
                3,
            ),
            (
                r#""\é""#,
                ParseErrorCodeV1::UnexpectedChar,
                "unknown string escape",
                1,
                4,
            ),
            (
                r#""\u110000""#,
                ParseErrorCodeV1::UnexpectedChar,
                "expected '{' after \\u",
                1,
                3,
            ),
            (
                r#""\u{}""#,
                ParseErrorCodeV1::UnexpectedChar,
                "invalid Unicode escape",
                1,
                4,
            ),
            (
                r#""\u{0000041}""#,
                ParseErrorCodeV1::UnexpectedChar,
                "invalid Unicode escape",
                1,
                11,
            ),
            (
                r#""\u{g}""#,
                ParseErrorCodeV1::UnexpectedChar,
                "invalid Unicode escape",
                1,
                5,
            ),
            (
                r#""\u{d800}""#,
                ParseErrorCodeV1::UnexpectedChar,
                "Unicode escape is not a scalar value",
                1,
                9,
            ),
            (
                r#""\u{110000}""#,
                ParseErrorCodeV1::UnexpectedChar,
                "Unicode escape is not a scalar value",
                1,
                11,
            ),
            (
                "\"\\u{41",
                ParseErrorCodeV1::UnexpectedChar,
                "invalid Unicode escape",
                1,
                6,
            ),
            (
                "\"x\\",
                ParseErrorCodeV1::UnterminatedString,
                "unterminated string literal",
                0,
                3,
            ),
            (
                "\"\\u{41}",
                ParseErrorCodeV1::UnterminatedString,
                "unterminated string literal",
                0,
                7,
            ),
        ] {
            let source = format!("é {quoted}");
            let expected = ParseDiagnosticV1::new(code, SpanV1::new(start + 3, end + 3), message);
            assert_eq!(scan_string(&source, 3), Err(expected.clone()), "{source}");
            assert_eq!(
                decode_string(&source, 3, source.len(), 0),
                Err(expected),
                "{source}"
            );
        }
    }

    #[test]
    fn token_limit_rejects_long_payload_before_copy_or_decode() {
        // Positive control first: a generous budget must actually allocate
        // payloads, otherwise the `== 0` assertions below could pass with a dead
        // counter rather than with "rejected before copy or decode".
        PAYLOAD_ALLOCATIONS.with(|count| count.set(0));
        let generous = ParseBudgetV2::try_new(100, 1, 100, 8, 8, 8, 8).unwrap();
        let counter = ParseBudgetCounterV2::new(&generous);
        lex_with_budget("x \"y\"", &counter).unwrap();
        PAYLOAD_ALLOCATIONS.with(|count| {
            assert!(
                count.get() >= 2,
                "payload allocations must be observable, got {}",
                count.get()
            )
        });

        let long = "x".repeat(32_768);
        let cases = [
            format!("({long}"),
            format!("({}", "7".repeat(32_768)),
            format!("({}.{}", "7".repeat(16_384), "8".repeat(16_384)),
            format!("(\"{long}\""),
        ];
        for source in cases {
            PAYLOAD_ALLOCATIONS.with(|count| count.set(0));
            let budget = ParseBudgetV2::try_new(1, 1, 1, 1, 1, 1, 1).unwrap();
            let counter = ParseBudgetCounterV2::new(&budget);
            let error = lex_with_budget(&source, &counter).unwrap_err();
            assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
            assert_eq!(
                error.resource.map(|resource| resource.dimension),
                Some(ParseBudgetDimensionV2::Tokens)
            );
            PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));
        }
    }

    #[test]
    fn lexical_census_matches_independent_token_and_decoded_payload_count() {
        let source = r#"name 42 3.5 "a\n" <= +"#;
        let budget = ParseBudgetV2::try_new(7, 1, 1, 1, 1, 1, 1).unwrap();
        PAYLOAD_ALLOCATIONS.with(|count| count.set(0));
        let census = census_lexical_materialization_v1(source, &budget).unwrap();
        assert_eq!(census.token_count(), 7);
        assert_eq!(census.payload_bytes(), 14);
        assert_eq!(
            census.token_storage_bytes(),
            std::mem::size_of::<Vec<TokenV1>>() + 7 * std::mem::size_of::<TokenV1>()
        );
        assert_eq!(
            census.logical_bytes(),
            std::mem::size_of::<Vec<TokenV1>>() + 7 * std::mem::size_of::<TokenV1>() + 14
        );
        PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));

        let tokens = lex_with_budget(source, &ParseBudgetCounterV2::new(&budget)).unwrap();
        let emitted_payload_bytes: usize = tokens
            .iter()
            .map(|token| {
                if let TokenKindV1::Ident(text)
                | TokenKindV1::Int(text)
                | TokenKindV1::Float(text)
                | TokenKindV1::Str(text)
                | TokenKindV1::Op(text) = &token.kind
                {
                    assert_eq!(text.capacity(), text.len());
                    text.len()
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(emitted_payload_bytes, census.payload_bytes());
    }

    #[test]
    fn admitted_escape_payload_allocates_only_its_decoded_bytes() {
        for (source, decoded) in [
            (r#""\u{41}""#, "A"),
            (r#""\u{1f642}""#, "🙂"),
            (r#""""#, ""),
        ] {
            let source_id = quanta_gqlang_v2_hir::SourceIdV2::from_raw(19);
            let budget = ParseBudgetV2::standard_v2();
            let census = census_lexical_materialization_v1(source, &budget).unwrap();
            assert_eq!(census.payload_bytes(), decoded.len());
            let admitted = u64::try_from(census.logical_bytes()).unwrap();
            let host = crate::FrontendLexicalHostCeilingV1::try_new(admitted).unwrap();
            let root = crate::FrontendLexicalMaterializationRootV1::issue_v1(
                source_id, source, admitted, host,
            )
            .unwrap();
            let permit = root
                .reserve_v1(source_id, source, census.logical_bytes())
                .unwrap();
            let tokens =
                super::lex_with_census_v1(source, &ParseBudgetCounterV2::new(&budget), census)
                    .unwrap();
            assert!(matches!(
                tokens.first(),
                Some(TokenV1 {
                    kind: TokenKindV1::Str(value),
                    ..
                }) if value == decoded && value.capacity() == census.payload_bytes()
            ));
            drop(tokens);
            drop(permit);
            assert_eq!(root.live_bytes_v1(), 0);
        }
    }

    #[test]
    fn census_rejects_token_limit_and_escape_before_payload_birth() {
        let budget = ParseBudgetV2::try_new(1, 1, 1, 1, 1, 1, 1).unwrap();
        PAYLOAD_ALLOCATIONS.with(|count| count.set(0));
        let exhausted = census_lexical_materialization_v1("(+", &budget).unwrap_err();
        assert_eq!(exhausted.code, ParseErrorCodeV1::ResourceExhausted);
        PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));

        let invalid = census_lexical_materialization_v1(r#""\q""#, &budget).unwrap_err();
        assert_eq!(invalid.code, ParseErrorCodeV1::UnexpectedChar);
        PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));

        let one_payload_budget = ParseBudgetV2::try_new(2, 1, 1, 1, 1, 1, 1).unwrap();
        let escaped =
            census_lexical_materialization_v1(r#""\u{41}""#, &one_payload_budget).unwrap();
        assert_eq!(escaped.payload_bytes(), 1);
        let unicode =
            census_lexical_materialization_v1(r#""\u{1f642}""#, &one_payload_budget).unwrap();
        assert_eq!(unicode.payload_bytes(), 4);
        PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));
    }

    #[test]
    fn rejected_operator_is_not_allocated_before_token_budget_admission() {
        let budget = ParseBudgetV2::try_new(1, 1, 1, 1, 1, 1, 1).unwrap();
        PAYLOAD_ALLOCATIONS.with(|count| count.set(0));
        let error = lex_with_budget("(+", &ParseBudgetCounterV2::new(&budget)).unwrap_err();
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));
    }

    #[test]
    fn size_overflow_has_distinct_diagnostic_without_fake_token_exhaustion() {
        let error = super::census_overflow(SpanV1::point(7));
        assert_eq!(error.code, ParseErrorCodeV1::LexicalMaterializationOverflow);
        assert_eq!(error.code_str(), "surface.lexical_materialization_overflow");
        assert_eq!(error.resource, None);
    }

    #[test]
    fn short_lexical_owner_rejects_before_payload_allocation() {
        let source = String::from("show models");
        let source_id = quanta_gqlang_v2_hir::SourceIdV2::from_raw(17);
        let budget = ParseBudgetV2::standard_v2();
        let demand = census_lexical_materialization_v1(&source, &budget)
            .unwrap()
            .logical_bytes();
        let demand = u64::try_from(demand).unwrap();
        let host = crate::FrontendLexicalHostCeilingV1::try_new(demand).unwrap();
        let root = crate::FrontendLexicalMaterializationRootV1::issue_v1(
            source_id,
            &source,
            demand - 1,
            host,
        )
        .unwrap();
        PAYLOAD_ALLOCATIONS.with(|count| count.set(0));
        let error =
            crate::parse_source_document_with_lexical_owner_v1(source_id, &source, &budget, &root)
                .err()
                .unwrap();
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        PAYLOAD_ALLOCATIONS.with(|count| assert_eq!(count.get(), 0));
    }
}
