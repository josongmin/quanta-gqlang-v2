#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "exact formatter boundary regression assertions"
)]

use quanta_gqlang_v2_pipeline::{
    format_source_document_v2, FrontendBudgetV2, FrontendStageV2, SourceIdV2, SpanV1,
};

fn budget(bytes: u32, tokens: u32) -> FrontendBudgetV2 {
    FrontendBudgetV2::try_new(
        bytes, 4_194_304, tokens, 4_096, 64, 128, 262_144, 4_096, 4_096, 1_000_000, 50_000,
        200_000, 1_000_000, 16_777_216, 65_536, 50_000, 200_000, 1_000_000, 1_024, 10_000, 1_024,
        8_388_608,
    )
    .expect("checked budget")
}

#[test]
fn output_tokens_reject_at_cap_plus_one_with_original_source_location() {
    let id = SourceIdV2::from_raw(91);
    let source = "from t | order by a";
    let expected = "from t | order by a asc";
    let exact = budget(128, 8);
    let formatted = format_source_document_v2(id, source, &exact).expect("eight output tokens fit");
    assert_eq!(formatted, expected);
    assert_eq!(
        format_source_document_v2(id, &formatted, &exact).expect("same budget re-entry"),
        expected
    );
    let failure = format_source_document_v2(id, source, &budget(128, 7))
        .expect_err("input fits; canonical output does not");
    assert_eq!(failure.diagnostics().len(), 1);
    let error = &failure.diagnostics()[0];
    assert_eq!(error.stage(), FrontendStageV2::Resource);
    assert_eq!(error.code(), "surface.resource_exhausted");
    assert_eq!(error.primary().source_id, id);
    assert_eq!(error.primary().span, SpanV1::new(0, source.len()));
    assert_eq!(
        error.message(),
        "canonical output: tokens budget exhausted: limit 7, observed 8"
    );
}

#[test]
fn escaped_output_bytes_reject_at_cap_plus_one_with_original_source_location() {
    let id = SourceIdV2::from_raw(92);
    let source = "let x = \"\n\"";
    let expected = "let x = \"\\n\"";
    let exact = budget(12, 5);
    let formatted = format_source_document_v2(id, source, &exact).expect("twelve output bytes fit");
    assert_eq!(formatted, expected);
    assert_eq!(formatted.len(), 12);
    assert_eq!(
        format_source_document_v2(id, &formatted, &exact).expect("same budget re-entry"),
        expected
    );
    let failure = format_source_document_v2(id, source, &budget(11, 5))
        .expect_err("eleven input bytes fit; twelve output bytes do not");
    assert_eq!(failure.diagnostics().len(), 1);
    let error = &failure.diagnostics()[0];
    assert_eq!(error.stage(), FrontendStageV2::Resource);
    assert_eq!(error.code(), "frontend.resource_limit_exceeded");
    assert_eq!(error.primary().source_id, id);
    assert_eq!(error.primary().span, SpanV1::new(0, 11));
    assert_eq!(
        error.message(),
        "canonical output exceeds the frontend budget's max_source_bytes"
    );
}

#[test]
fn input_rejection_keeps_its_exact_span_and_reserved_alias_formats() {
    let id = SourceIdV2::from_raw(93);
    let budget = budget(128, 16);
    let failure = format_source_document_v2(id, "let x = @", &budget).expect_err("input rejects");
    let error = &failure.diagnostics()[0];
    assert_eq!(error.code(), "surface.parse.unexpected_char");
    assert_eq!(error.primary().source_id, id);
    assert_eq!(error.primary().span, SpanV1::new(8, 9));
    assert_eq!(error.message(), "unexpected character '@'");
    assert_eq!(
        format_source_document_v2(id, "from t as where", &budget).expect("reserved alias"),
        "from t as where"
    );
}
