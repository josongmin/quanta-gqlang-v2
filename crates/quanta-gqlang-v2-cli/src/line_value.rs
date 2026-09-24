//! One physical-line encoding for untrusted CLI protocol values.

/// Escape characters that could split or visually rewrite a line-protocol
/// value. Ordinary ASCII and Unicode text retain their original bytes.
#[must_use]
pub fn encode_line_value_v1(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            '\u{2028}' | '\u{2029}' => encoded.extend(character.escape_unicode()),
            control if control.is_control() && u32::from(control) <= 0xff => {
                encoded.push_str(&format!("\\x{:02x}", u32::from(control)));
            }
            control if control.is_control() => encoded.extend(control.escape_unicode()),
            plain => encoded.push(plain),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::encode_line_value_v1;

    #[test]
    fn escapes_control_and_separator_chars_without_changing_benign_text() {
        assert_eq!(
            encode_line_value_v1("Aé=100%\\\n\r\t\u{1b}\u{85}\u{2028}\u{2029}終"),
            "Aé=100%\\\\\\n\\r\\t\\x1b\\x85\\u{2028}\\u{2029}終"
        );
        assert_eq!(encode_line_value_v1("plain é=100% 終"), "plain é=100% 終");
    }
}
