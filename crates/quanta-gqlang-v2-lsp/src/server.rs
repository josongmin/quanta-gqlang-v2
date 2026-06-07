use std::collections::BTreeMap;

use crate::handlers::completion::CompletionItemV1;
use crate::handlers::diagnostics::DiagnosticItemV1;
use crate::handlers::hover::HoverCardV1;
use crate::handlers::quickfix::QuickfixSuggestionV1;
use crate::handlers::symbols::SymbolItemV1;
use crate::handlers::zero_query::{ZeroQueryGroupV1, ZeroQueryResponseV1, ZeroQuerySuggestionV1};

#[derive(Default)]
pub struct LspServerV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StdioLspErrorV1 {
    code: &'static str,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReadOnlyRequestV1 {
    Hover { query: String },
    Completion { prefix: String },
    Symbols { query: String },
    Diagnostics { query: String },
    ZeroQuery,
    Quickfix { input: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReadOnlyResponseV1 {
    Hover(HoverCardV1),
    Completion(Vec<CompletionItemV1>),
    Symbols(Vec<SymbolItemV1>),
    Diagnostics(Vec<DiagnosticItemV1>),
    ZeroQuery(ZeroQueryResponseV1),
    Quickfix(Vec<QuickfixSuggestionV1>),
}

struct JsonCursorV1<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl LspServerV1 {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn dispatch_json_v1(&self, input: &str) -> Result<String, StdioLspErrorV1> {
        let request = ReadOnlyRequestV1::parse_json_v1(input)?;
        let method = request.method_label_v1();
        let response = self.dispatch_request_v1(&request)?;
        Ok(render_success_json_v1(method, &response))
    }

    pub fn hover(&self, query: &str) -> Result<crate::handlers::hover::HoverCardV1, String> {
        crate::handlers::hover::handle_hover(query)
    }

    pub fn completion(
        &self,
        prefix: &str,
    ) -> Result<Vec<crate::handlers::completion::CompletionItemV1>, String> {
        crate::handlers::completion::handle_completion(prefix)
    }

    pub fn symbols(
        &self,
        query: &str,
    ) -> Result<Vec<crate::handlers::symbols::SymbolItemV1>, String> {
        crate::handlers::symbols::handle_symbols(query)
    }

    pub fn diagnostics(
        &self,
        query: &str,
    ) -> Result<Vec<crate::handlers::diagnostics::DiagnosticItemV1>, String> {
        crate::handlers::diagnostics::handle_diagnostics(query)
    }

    pub fn zero_query(&self) -> Result<crate::handlers::zero_query::ZeroQueryResponseV1, String> {
        crate::handlers::zero_query::handle_zero_query()
    }

    pub fn quickfix(
        &self,
        input: &str,
    ) -> Result<Vec<crate::handlers::quickfix::QuickfixSuggestionV1>, String> {
        crate::handlers::quickfix::handle_quickfix(input)
    }

    fn dispatch_request_v1(
        &self,
        request: &ReadOnlyRequestV1,
    ) -> Result<ReadOnlyResponseV1, StdioLspErrorV1> {
        match request {
            ReadOnlyRequestV1::Hover { query } => self
                .hover(query)
                .map(ReadOnlyResponseV1::Hover)
                .map_err(handler_rejected_v1),
            ReadOnlyRequestV1::Completion { prefix } => self
                .completion(prefix)
                .map(ReadOnlyResponseV1::Completion)
                .map_err(handler_rejected_v1),
            ReadOnlyRequestV1::Symbols { query } => self
                .symbols(query)
                .map(ReadOnlyResponseV1::Symbols)
                .map_err(handler_rejected_v1),
            ReadOnlyRequestV1::Diagnostics { query } => self
                .diagnostics(query)
                .map(ReadOnlyResponseV1::Diagnostics)
                .map_err(handler_rejected_v1),
            ReadOnlyRequestV1::ZeroQuery => self
                .zero_query()
                .map(ReadOnlyResponseV1::ZeroQuery)
                .map_err(handler_rejected_v1),
            ReadOnlyRequestV1::Quickfix { input } => self
                .quickfix(input)
                .map(ReadOnlyResponseV1::Quickfix)
                .map_err(handler_rejected_v1),
        }
    }
}

pub(crate) fn extract_next_request_frame_v1(
    buffer: &[u8],
    eof: bool,
) -> Result<Option<(usize, usize)>, StdioLspErrorV1> {
    let mut start = 0_usize;
    while buffer.get(start).is_some_and(u8::is_ascii_whitespace) {
        start += 1;
    }
    if start == buffer.len() {
        return Ok(None);
    }
    if buffer.get(start) != Some(&b'{') {
        return Err(StdioLspErrorV1::new(
            "malformed_json",
            "expected JSON object to start with `{`",
        ));
    }

    let mut cursor = start;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(byte) = buffer.get(cursor).copied() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Some((start, cursor + 1)));
                    }
                }
                _ => {}
            }
        }
        cursor += 1;
    }

    if eof {
        Err(StdioLspErrorV1::new(
            "malformed_json",
            "incomplete JSON object at end of input",
        ))
    } else {
        Ok(None)
    }
}

impl StdioLspErrorV1 {
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn io_failure_v1(stage: &'static str, message: impl Into<String>) -> Self {
        Self::new("io_failure", format!("{stage} failed: {}", message.into()))
    }

    #[must_use]
    pub fn render_json_v1(&self) -> String {
        json_object_v1([
            ("ok", json_bool_v1(false)),
            (
                "error",
                json_object_v1([
                    ("code", json_string_v1(self.code)),
                    ("message", json_string_v1(&self.message)),
                ]),
            ),
        ])
    }
}

impl ReadOnlyRequestV1 {
    fn parse_json_v1(input: &str) -> Result<Self, StdioLspErrorV1> {
        let mut fields = JsonCursorV1::new(input).parse_flat_object_v1()?;
        let raw_method = remove_required_field_v1(&mut fields, "method")?;
        let method = raw_method.trim();
        if method.is_empty() {
            return Err(StdioLspErrorV1::new(
                "empty_method",
                "method cannot be empty",
            ));
        }

        match method {
            "hover" => {
                let query = remove_required_field_v1(&mut fields, "query")?;
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::Hover { query })
            }
            "completion" => {
                let prefix = remove_required_field_v1(&mut fields, "prefix")?;
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::Completion { prefix })
            }
            "symbols" => {
                let query = remove_required_field_v1(&mut fields, "query")?;
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::Symbols { query })
            }
            "diagnostics" => {
                let query = remove_required_field_v1(&mut fields, "query")?;
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::Diagnostics { query })
            }
            "zero_query" => {
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::ZeroQuery)
            }
            "quickfix" => {
                let input = remove_required_field_v1(&mut fields, "input")?;
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::Quickfix { input })
            }
            _ if mutating_method_v1(method) => Err(StdioLspErrorV1::new(
                "command_plane_forbidden",
                format!("mutating method `{method}` is forbidden on the read-only front door"),
            )),
            _ => Err(StdioLspErrorV1::new(
                "unsupported_method",
                format!("unknown read-only method `{method}`"),
            )),
        }
    }

    const fn method_label_v1(&self) -> &'static str {
        match self {
            Self::Hover { .. } => "hover",
            Self::Completion { .. } => "completion",
            Self::Symbols { .. } => "symbols",
            Self::Diagnostics { .. } => "diagnostics",
            Self::ZeroQuery => "zero_query",
            Self::Quickfix { .. } => "quickfix",
        }
    }
}

impl ReadOnlyResponseV1 {
    fn render_json_v1(&self) -> String {
        match self {
            Self::Hover(card) => render_hover_card_json_v1(card),
            Self::Completion(items) => {
                json_array_v1(items.iter().map(render_completion_item_json_v1))
            }
            Self::Symbols(items) => json_array_v1(items.iter().map(render_symbol_item_json_v1)),
            Self::Diagnostics(items) => {
                json_array_v1(items.iter().map(render_diagnostic_item_json_v1))
            }
            Self::ZeroQuery(report) => render_zero_query_response_json_v1(report),
            Self::Quickfix(items) => json_array_v1(items.iter().map(render_quickfix_json_v1)),
        }
    }
}

impl<'a> JsonCursorV1<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn parse_flat_object_v1(mut self) -> Result<BTreeMap<String, String>, StdioLspErrorV1> {
        self.skip_ws_v1();
        self.expect_char_v1('{')?;
        self.skip_ws_v1();

        let mut fields = BTreeMap::new();
        if self.consume_char_v1('}') {
            self.skip_ws_v1();
            if self.chars.peek().is_some() {
                return Err(StdioLspErrorV1::new(
                    "trailing_content",
                    "trailing content after JSON object",
                ));
            }
            return Ok(fields);
        }

        loop {
            let key = self.parse_string_v1()?;
            self.skip_ws_v1();
            self.expect_char_v1(':')?;
            self.skip_ws_v1();
            if !matches!(self.chars.peek(), Some('"')) {
                return Err(StdioLspErrorV1::new(
                    "malformed_json",
                    "only string field values are supported",
                ));
            }
            let value = self.parse_string_v1()?;
            if fields.insert(key.clone(), value).is_some() {
                return Err(StdioLspErrorV1::new(
                    "duplicate_field",
                    format!("duplicate field `{key}`"),
                ));
            }

            self.skip_ws_v1();
            if self.consume_char_v1(',') {
                self.skip_ws_v1();
                continue;
            }
            if self.consume_char_v1('}') {
                break;
            }
            return Err(StdioLspErrorV1::new(
                "malformed_json",
                "expected `,` or `}` in JSON object",
            ));
        }

        self.skip_ws_v1();
        if self.chars.peek().is_some() {
            return Err(StdioLspErrorV1::new(
                "trailing_content",
                "trailing content after JSON object",
            ));
        }
        Ok(fields)
    }

    fn parse_string_v1(&mut self) -> Result<String, StdioLspErrorV1> {
        self.expect_char_v1('"')?;
        let mut output = String::new();
        loop {
            let Some(ch) = self.chars.next() else {
                return Err(StdioLspErrorV1::new(
                    "malformed_json",
                    "unterminated JSON string",
                ));
            };
            match ch {
                '"' => return Ok(output),
                '\\' => output.push(self.parse_escape_v1()?),
                c if c.is_control() => {
                    return Err(StdioLspErrorV1::new(
                        "malformed_json",
                        "control character in JSON string",
                    ))
                }
                c => output.push(c),
            }
        }
    }

    fn parse_escape_v1(&mut self) -> Result<char, StdioLspErrorV1> {
        let Some(ch) = self.chars.next() else {
            return Err(StdioLspErrorV1::new(
                "malformed_json",
                "unterminated JSON escape",
            ));
        };
        match ch {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => self.parse_unicode_escape_v1(),
            _ => Err(StdioLspErrorV1::new(
                "malformed_json",
                format!("unsupported JSON escape `\\{ch}`"),
            )),
        }
    }

    fn parse_unicode_escape_v1(&mut self) -> Result<char, StdioLspErrorV1> {
        let value = self.parse_unicode_hex_v1()?;
        if is_high_surrogate_v1(value) {
            self.expect_char_v1('\\')?;
            match self.chars.next() {
                Some('u') => {
                    let low = self.parse_unicode_hex_v1()?;
                    if !is_low_surrogate_v1(low) {
                        return Err(StdioLspErrorV1::new(
                            "malformed_json",
                            format!("expected low surrogate after `\\u{value:04X}`"),
                        ));
                    }
                    let scalar = 0x10000 + ((value - 0xD800) << 10) + (low - 0xDC00);
                    char::from_u32(scalar).ok_or_else(|| {
                        StdioLspErrorV1::new(
                            "malformed_json",
                            format!("invalid unicode scalar value `\\u{value:04X}\\u{low:04X}`"),
                        )
                    })
                }
                Some(ch) => Err(StdioLspErrorV1::new(
                    "malformed_json",
                    format!("expected `u` after surrogate pair separator, found `{ch}`"),
                )),
                None => Err(StdioLspErrorV1::new(
                    "malformed_json",
                    "unterminated surrogate pair escape",
                )),
            }
        } else if is_low_surrogate_v1(value) {
            Err(StdioLspErrorV1::new(
                "malformed_json",
                format!("unexpected low surrogate `\\u{value:04X}`"),
            ))
        } else {
            char::from_u32(value).ok_or_else(|| {
                StdioLspErrorV1::new(
                    "malformed_json",
                    format!("invalid unicode scalar value `\\u{value:04X}`"),
                )
            })
        }
    }

    fn parse_unicode_hex_v1(&mut self) -> Result<u32, StdioLspErrorV1> {
        let mut value = 0_u32;
        for _ in 0..4 {
            let Some(ch) = self.chars.next() else {
                return Err(StdioLspErrorV1::new(
                    "malformed_json",
                    "unterminated unicode escape",
                ));
            };
            let Some(digit) = ch.to_digit(16) else {
                return Err(StdioLspErrorV1::new(
                    "malformed_json",
                    format!("invalid unicode escape digit `{ch}`"),
                ));
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn expect_char_v1(&mut self, expected: char) -> Result<(), StdioLspErrorV1> {
        match self.chars.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(StdioLspErrorV1::new(
                "malformed_json",
                format!("expected `{expected}`, found `{ch}`"),
            )),
            None => Err(StdioLspErrorV1::new(
                "malformed_json",
                format!("expected `{expected}`, found end of input"),
            )),
        }
    }

    fn consume_char_v1(&mut self, expected: char) -> bool {
        match self.chars.peek() {
            Some(ch) if *ch == expected => {
                self.chars.next();
                true
            }
            _ => false,
        }
    }

    fn skip_ws_v1(&mut self) {
        while matches!(self.chars.peek(), Some(ch) if ch.is_ascii_whitespace()) {
            self.chars.next();
        }
    }
}

fn handler_rejected_v1(error: String) -> StdioLspErrorV1 {
    StdioLspErrorV1::new("handler_rejected", error)
}

fn remove_required_field_v1(
    fields: &mut BTreeMap<String, String>,
    key: &str,
) -> Result<String, StdioLspErrorV1> {
    fields.remove(key).ok_or_else(|| {
        StdioLspErrorV1::new("missing_field", format!("missing required field `{key}`"))
    })
}

fn reject_extra_fields_v1(
    method: &str,
    fields: &BTreeMap<String, String>,
) -> Result<(), StdioLspErrorV1> {
    if fields.is_empty() {
        return Ok(());
    }
    let extras = fields.keys().cloned().collect::<Vec<_>>().join(", ");
    Err(StdioLspErrorV1::new(
        "unexpected_field",
        format!("unexpected fields for `{method}`: {extras}"),
    ))
}

fn mutating_method_v1(method: &str) -> bool {
    let normalized = method
        .trim()
        .to_ascii_lowercase()
        .replace(['/', '-', '_'], "");
    normalized.contains("rename")
        || normalized.contains("applyedit")
        || normalized == "apply"
        || normalized.contains("codeaction")
        || normalized.contains("workspaceedit")
}

fn render_success_json_v1(method: &str, response: &ReadOnlyResponseV1) -> String {
    json_object_v1([
        ("ok", json_bool_v1(true)),
        ("method", json_string_v1(method)),
        ("result", response.render_json_v1()),
    ])
}

fn render_hover_card_json_v1(card: &HoverCardV1) -> String {
    json_object_v1([
        ("root_keyword", json_string_v1(&card.root_keyword)),
        ("category", json_string_v1(&card.category)),
        ("typed_target", json_string_v1(&card.typed_target)),
        (
            "summary",
            json_array_v1(card.summary.iter().map(|entry| json_string_v1(entry))),
        ),
    ])
}

fn render_completion_item_json_v1(item: &CompletionItemV1) -> String {
    json_object_v1([
        ("label", json_string_v1(&item.label)),
        ("kind", json_string_v1(&item.kind)),
        ("detail", json_string_v1(&item.detail)),
        ("reason", json_string_v1(&item.reason)),
        ("scope", json_string_v1(&item.scope)),
        ("match_reason", json_string_v1(&item.match_reason)),
        (
            "example_query",
            json_option_string_v1(item.example_query.as_deref()),
        ),
    ])
}

fn render_symbol_item_json_v1(item: &SymbolItemV1) -> String {
    json_object_v1([
        ("name", json_string_v1(&item.name)),
        ("kind", json_string_v1(&item.kind)),
        ("detail", json_option_string_v1(item.detail.as_deref())),
    ])
}

fn render_diagnostic_item_json_v1(item: &DiagnosticItemV1) -> String {
    json_object_v1([
        ("class", json_string_v1(&item.class)),
        ("code", json_string_v1(&item.code)),
        ("message", json_string_v1(&item.message)),
    ])
}

fn render_zero_query_response_json_v1(report: &ZeroQueryResponseV1) -> String {
    json_object_v1([
        ("mode", json_string_v1(report.mode)),
        ("provenance", json_string_v1(report.provenance)),
        (
            "groups",
            json_array_v1(report.groups.iter().map(render_zero_query_group_json_v1)),
        ),
    ])
}

fn render_zero_query_group_json_v1(group: &ZeroQueryGroupV1) -> String {
    json_object_v1([
        ("id", json_string_v1(group.id)),
        ("title", json_string_v1(group.title)),
        (
            "suggestions",
            json_array_v1(
                group
                    .suggestions
                    .iter()
                    .map(render_zero_query_suggestion_json_v1),
            ),
        ),
    ])
}

fn render_zero_query_suggestion_json_v1(suggestion: &ZeroQuerySuggestionV1) -> String {
    json_object_v1([
        ("label", json_string_v1(suggestion.label)),
        ("root_kind", json_string_v1(suggestion.root_kind)),
        ("query_scaffold", json_string_v1(suggestion.query_scaffold)),
        ("behavior", json_string_v1(suggestion.behavior)),
        ("reason_code", json_string_v1(suggestion.reason_code)),
        ("reason_detail", json_string_v1(suggestion.reason_detail)),
    ])
}

fn render_quickfix_json_v1(item: &QuickfixSuggestionV1) -> String {
    json_object_v1([
        ("code", json_string_v1(&item.code)),
        ("label", json_string_v1(&item.label)),
        ("detail", json_string_v1(&item.detail)),
        ("scaffold", json_string_v1(&item.scaffold)),
        ("edits_document", json_bool_v1(item.edits_document)),
    ])
}

fn json_object_v1<const N: usize>(fields: [(&str, String); N]) -> String {
    let entries = fields
        .into_iter()
        .map(|(key, value)| format!("{}:{value}", json_string_v1(key)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{entries}}}")
}

fn json_array_v1<I>(items: I) -> String
where
    I: Iterator<Item = String>,
{
    let joined = items.collect::<Vec<_>>().join(",");
    format!("[{joined}]")
}

fn json_string_v1(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000C}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() => output.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => output.push(c),
        }
    }
    output.push('"');
    output
}

fn json_option_string_v1(value: Option<&str>) -> String {
    value
        .map(json_string_v1)
        .unwrap_or_else(|| "null".to_string())
}

fn json_bool_v1(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

const fn is_high_surrogate_v1(value: u32) -> bool {
    value >= 0xD800 && value <= 0xDBFF
}

const fn is_low_surrogate_v1(value: u32) -> bool {
    value >= 0xDC00 && value <= 0xDFFF
}
