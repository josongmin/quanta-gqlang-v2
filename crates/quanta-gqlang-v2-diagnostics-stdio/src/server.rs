use std::collections::BTreeMap;

use crate::json::{
    json_array_v1, json_bool_v1, json_object_v1, json_string_v1, json_usize_v1, JsonCursorV1,
};

use crate::handlers::v3_diagnostics::V3DiagnosticItemV1;

/// The complete read-only stdio method allowlist, in dispatch order.
///
/// The diagnostics transport is v3-only. The old read-only methods
/// (`hover`/`completion`/`symbols`/`diagnostics`/`zero_query`/`quickfix`/
/// `explain`) were rooted in the legacy `ir`/`sema`/`syntax` stack and have been
/// removed; the single going-forward analysis surface is `v3_diagnostics`, which
/// runs over the ONE shared compiler pipeline
/// `quanta-gqlang-v2-pipeline::compile_program_structural` (`D22`).
///
/// This const is the single list the doc gate checks the README against (so a
/// shipped method can never go undocumented). A binary test pins that each name
/// is accepted by the dispatch (not `method_not_allowed`) and that the count
/// matches the dispatch arms, so the const cannot drift from the closed
/// allowlist in `parse_json_v1`.
pub const READ_ONLY_METHOD_NAMES_V1: [&str; 1] = ["v3_diagnostics"];

#[derive(Default)]
pub struct LspServerV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StdioLspErrorV1 {
    code: &'static str,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReadOnlyRequestV1 {
    V3Diagnostics { query: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReadOnlyResponseV1 {
    V3Diagnostics(Vec<V3DiagnosticItemV1>),
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

    /// v3 read-only diagnostics over the ONE shared compiler pipeline (D22).
    ///
    /// Infallible at the handler boundary: a compile failure is a diagnostic on
    /// the read-only surface (data), not a handler reject, so the SAME failing
    /// query yields the SAME stage-tagged pipeline code the CLI surfaces.
    pub fn v3_diagnostics(
        &self,
        query: &str,
    ) -> Vec<crate::handlers::v3_diagnostics::V3DiagnosticItemV1> {
        crate::handlers::v3_diagnostics::handle_v3_diagnostics_v1(query)
    }

    fn dispatch_request_v1(
        &self,
        request: &ReadOnlyRequestV1,
    ) -> Result<ReadOnlyResponseV1, StdioLspErrorV1> {
        match request {
            ReadOnlyRequestV1::V3Diagnostics { query } => Ok(ReadOnlyResponseV1::V3Diagnostics(
                self.v3_diagnostics(query),
            )),
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
            "v3_diagnostics" => {
                let query = remove_required_field_v1(&mut fields, "query")?;
                reject_extra_fields_v1(method, &fields)?;
                Ok(Self::V3Diagnostics { query })
            }
            _ => Err(StdioLspErrorV1::new(
                "method_not_allowed",
                format!("method `{method}` is not an allowed read-only method"),
            )),
        }
    }

    const fn method_label_v1(&self) -> &'static str {
        match self {
            Self::V3Diagnostics { .. } => "v3_diagnostics",
        }
    }
}

impl ReadOnlyResponseV1 {
    fn render_json_v1(&self) -> String {
        match self {
            Self::V3Diagnostics(items) => {
                json_array_v1(items.iter().map(render_v3_diagnostic_item_json_v1))
            }
        }
    }
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

fn render_success_json_v1(method: &str, response: &ReadOnlyResponseV1) -> String {
    json_object_v1([
        ("ok", json_bool_v1(true)),
        ("method", json_string_v1(method)),
        ("result", response.render_json_v1()),
    ])
}

fn render_v3_diagnostic_item_json_v1(item: &V3DiagnosticItemV1) -> String {
    json_object_v1([
        ("stage", json_string_v1(&item.stage)),
        ("code", json_string_v1(&item.code)),
        ("message", json_string_v1(&item.message)),
        ("span_start", json_usize_v1(item.span_start)),
        ("span_end", json_usize_v1(item.span_end)),
    ])
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "unit tests intentionally use expect to assert exact frame boundaries"
)]
mod tests {
    use super::extract_next_request_frame_v1;

    #[test]
    fn frame_extractor_treats_braces_inside_strings_as_literal_v1() {
        // The string value contains a `}` (index 7) and a `{` (index 9). The
        // extractor must enter string mode on the opening `"` and ignore those
        // braces, so the frame closes only at the FINAL `}` (index 11), giving
        // end == 12. If the `b'"' => in_string = true` arm is deleted, the inner
        // `}` is read as structural and prematurely closes the frame at index 7
        // (end == 8) — so pinning the exact end span kills that mutation.
        let buffer = b"{\"k\":\"a}{b\"}";
        assert_eq!(buffer.len(), 12, "buffer layout is what the test assumes");
        let frame = extract_next_request_frame_v1(buffer, true)
            .expect("balanced object extracts")
            .expect("a complete frame is present");
        assert_eq!(
            frame,
            (0, 12),
            "frame spans the whole brace-balanced object"
        );
    }

    #[test]
    fn frame_extractor_closing_brace_inside_string_does_not_terminate_early_v1() {
        // Two top-level objects back to back, the first carrying an embedded `}`
        // inside its string value. With string tracking, the first frame is the
        // full first object; a second call resumes after it. Without the `"`
        // arm, the first frame would be truncated at the embedded `}`.
        let buffer = b"{\"a\":\"}\"}{\"b\":\"c\"}";
        let frame = extract_next_request_frame_v1(buffer, false)
            .expect("first object extracts")
            .expect("first frame present");
        // First object is `{"a":"}"}` = 9 bytes, indices 0..=8, end == 9.
        assert_eq!(
            frame,
            (0, 9),
            "embedded brace is inside the string, not structural"
        );
    }
}
