use std::collections::BTreeMap;

use crate::server::StdioLspErrorV1;

pub(crate) struct JsonCursorV1<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> JsonCursorV1<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub(crate) fn parse_flat_object_v1(
        mut self,
    ) -> Result<BTreeMap<String, String>, StdioLspErrorV1> {
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

pub(crate) fn json_object_v1<const N: usize>(fields: [(&str, String); N]) -> String {
    let entries = fields
        .into_iter()
        .map(|(key, value)| format!("{}:{value}", json_string_v1(key)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{entries}}}")
}

pub(crate) fn json_array_v1<I>(items: I) -> String
where
    I: Iterator<Item = String>,
{
    let joined = items.collect::<Vec<_>>().join(",");
    format!("[{joined}]")
}

pub(crate) fn json_string_v1(value: &str) -> String {
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

/// Render a `usize` as a JSON number. Used for byte-offset spans, which are
/// non-negative integers, so the decimal form is a valid JSON number with no
/// quoting or lossy numeric coercion.
pub(crate) fn json_usize_v1(value: usize) -> String {
    value.to_string()
}

pub(crate) fn json_bool_v1(value: bool) -> String {
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "unit tests intentionally use expect/expect_err to assert exact parse results and reject envelopes"
)]
mod tests {
    use super::{
        json_array_v1, json_bool_v1, json_object_v1, json_string_v1, json_usize_v1, JsonCursorV1,
    };

    /// Parse and, on error, surface the fail-closed envelope JSON the front door
    /// would emit. `StdioLspErrorV1`'s fields are private to `server`, so we pin
    /// the public `render_json_v1` line instead — exactly what a client sees.
    fn parse_v1(input: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
        JsonCursorV1::new(input)
            .parse_flat_object_v1()
            .map_err(|err| err.render_json_v1())
    }

    // --- branches the brace-balanced frame extractor strips before the parser
    // ever runs, so they are unreachable from the stdio front door and are
    // pinned here against the parser directly. ---

    #[test]
    fn empty_object_parses_to_empty_map_v1() {
        let fields = parse_v1("{}").expect("empty object parses");
        assert!(fields.is_empty(), "empty object yields no fields");
    }

    fn err_line_v1(code: &str, message: &str) -> String {
        format!(
            r#"{{"ok":false,"error":{{"code":"{code}","message":{}}}}}"#,
            json_string_v1(message)
        )
    }

    #[test]
    fn empty_object_then_trailing_content_is_rejected_v1() {
        let err = parse_v1("{}x").expect_err("trailing after empty object rejects");
        assert_eq!(
            err,
            err_line_v1("trailing_content", "trailing content after JSON object")
        );
    }

    #[test]
    fn populated_object_then_trailing_content_is_rejected_v1() {
        let err = parse_v1(r#"{"a":"b"} x"#).expect_err("trailing after object rejects");
        assert_eq!(
            err,
            err_line_v1("trailing_content", "trailing content after JSON object")
        );
    }

    #[test]
    fn unterminated_string_is_rejected_v1() {
        let err = parse_v1(r#"{"a"#).expect_err("unterminated string rejects");
        assert_eq!(
            err,
            err_line_v1("malformed_json", "unterminated JSON string")
        );
    }

    #[test]
    fn unterminated_escape_at_end_of_input_is_rejected_v1() {
        // A backslash with nothing after it inside the string.
        let err = parse_v1("{\"a\\").expect_err("unterminated escape rejects");
        assert_eq!(
            err,
            err_line_v1("malformed_json", "unterminated JSON escape")
        );
    }

    #[test]
    fn unterminated_unicode_escape_mid_hex_is_rejected_v1() {
        let err = parse_v1(r#"{"a":"\u12"#).expect_err("truncated \\u rejects");
        assert_eq!(
            err,
            err_line_v1("malformed_json", "unterminated unicode escape")
        );
    }

    #[test]
    fn unterminated_surrogate_pair_escape_is_rejected_v1() {
        // A high surrogate, then the `\` separator, then end of input.
        let err = parse_v1(r#"{"a":"\uD83D\"#).expect_err("unterminated surrogate pair rejects");
        assert_eq!(
            err,
            err_line_v1("malformed_json", "unterminated surrogate pair escape")
        );
    }

    #[test]
    fn expect_char_at_end_of_input_is_rejected_v1() {
        // Whitespace-trimmed empty input: `expect_char_v1('{')` hits end of input.
        let err = parse_v1("   ").expect_err("empty input rejects");
        assert_eq!(
            err,
            err_line_v1("malformed_json", "expected `{`, found end of input")
        );
    }

    #[test]
    fn missing_opening_brace_reports_found_char_v1() {
        let err = parse_v1("x").expect_err("non-brace start rejects");
        assert_eq!(
            err,
            err_line_v1("malformed_json", "expected `{`, found `x`")
        );
    }

    // --- positive decode paths. ---

    #[test]
    fn all_simple_escapes_decode_to_their_control_chars_v1() {
        let fields = parse_v1(r#"{"k":"\"\\\/\b\f\n\r\t"}"#).expect("simple escapes decode");
        assert_eq!(
            fields.get("k").map(String::as_str),
            Some("\"\\/\u{8}\u{c}\n\r\t")
        );
    }

    #[test]
    fn valid_surrogate_pair_decodes_to_astral_scalar_v1() {
        // `😀` is the grinning-face emoji `😀` (U+1F600).
        let fields = parse_v1(r#"{"k":"😀"}"#).expect("surrogate pair decodes");
        assert_eq!(fields.get("k").map(String::as_str), Some("😀"));
    }

    #[test]
    fn bmp_unicode_escape_decodes_v1() {
        // A non-surrogate BMP scalar via `\u`.
        let fields = parse_v1(r#"{"k":"é"}"#).expect("bmp escape decodes");
        assert_eq!(fields.get("k").map(String::as_str), Some("é"));
    }

    #[test]
    fn whitespace_around_tokens_is_skipped_v1() {
        let fields = parse_v1("  {  \"a\"  :  \"b\"  ,  \"c\" : \"d\"  }  ")
            .expect("interior whitespace is skipped");
        assert_eq!(fields.get("a").map(String::as_str), Some("b"));
        assert_eq!(fields.get("c").map(String::as_str), Some("d"));
    }

    // --- encoders. ---

    #[test]
    fn json_string_escapes_every_special_char_v1() {
        let encoded = json_string_v1("\"\\\u{8}\u{c}\n\r\t");
        assert_eq!(encoded, r#""\"\\\b\f\n\r\t""#);
    }

    #[test]
    fn json_string_escapes_other_control_chars_as_u_hex_v1() {
        // A control char with no dedicated escape (U+0001) becomes its \uXXXX form.
        let encoded = json_string_v1("\u{1}");
        assert_eq!(encoded, "\"\\u0001\"");
    }

    #[test]
    fn json_string_passes_through_non_control_chars_v1() {
        let encoded = json_string_v1("héllo 😀");
        assert_eq!(encoded, "\"héllo 😀\"");
    }

    #[test]
    fn json_object_joins_fields_with_quoted_keys_v1() {
        let rendered = json_object_v1([("a", "1".to_string()), ("b", "2".to_string())]);
        assert_eq!(rendered, r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn json_object_with_zero_fields_is_empty_braces_v1() {
        let rendered = json_object_v1::<0>([]);
        assert_eq!(rendered, "{}");
    }

    #[test]
    fn json_array_joins_items_with_commas_v1() {
        let rendered = json_array_v1(["1".to_string(), "2".to_string()].into_iter());
        assert_eq!(rendered, "[1,2]");
    }

    #[test]
    fn json_array_empty_is_empty_brackets_v1() {
        let rendered = json_array_v1(std::iter::empty());
        assert_eq!(rendered, "[]");
    }

    #[test]
    fn json_usize_renders_decimal_v1() {
        assert_eq!(json_usize_v1(0), "0");
        assert_eq!(json_usize_v1(42), "42");
    }

    #[test]
    fn json_bool_renders_keywords_v1() {
        assert_eq!(json_bool_v1(true), "true");
        assert_eq!(json_bool_v1(false), "false");
    }
}
