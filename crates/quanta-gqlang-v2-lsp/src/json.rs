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
