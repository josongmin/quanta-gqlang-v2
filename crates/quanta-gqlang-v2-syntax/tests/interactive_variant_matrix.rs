#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Contract: exhaustive EXPORT-kind / SAMPLE-mode / COMPARE-metric matrix.
//!
//! `parser_conformance.rs` only drove `kind=sarif`, `mode=semantic`, and
//! `metric=semantic`. That left `EXPORT` json/csv/markdown, `SAMPLE` lexical,
//! and `COMPARE` exact/structural parser-supported but unpinned — a regression
//! in any of those decode arms would pass CI silently.
//!
//! This rail drives every value of `ExportKindV1`, `SampleRetrievalModeV1`, and
//! `CompareMetricV1` through both the low-level `decode_*_intent_v1` API and the
//! full `parse_document_v1` front door, asserts the decoded variant, and proves
//! each is a `format_document_v1` fixed point. The negative half pins the
//! unknown-value fail-closed branches to their parser diagnostic codes.
//!
//! Frontend-only: typed decoding and formatting determinism, never runtime
//! export materialization, retrieval results, or comparison truth.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::sample::{
    decode_compare_intent_v1, decode_export_intent_v1, decode_sample_intent_v1, CompareMetricV1,
    ExportKindV1, SampleRetrievalModeV1,
};
use quanta_gqlang_v2_syntax::{format_document_v1, parse_document_v1};

/// Format a single-root document once through the public formatter.
fn fmt_once_v1(input: &str) -> String {
    let doc = parse_document_v1(input)
        .unwrap_or_else(|page| panic!("expected `{input}` to parse for fmt: {page:?}"));
    format_document_v1(&doc)
}

/// Assert `input` is a `format_document_v1` fixed point and that its canonical
/// form carries `fragment`.
fn assert_fmt_fixed_point_with_fragment_v1(input: &str, fragment: &str) {
    let formatted = fmt_once_v1(input);
    assert!(
        formatted.contains(fragment),
        "fmt `{formatted}` missing `{fragment}` for `{input}`"
    );
    assert_eq!(
        formatted,
        fmt_once_v1(formatted.trim()),
        "fmt not idempotent for `{input}`"
    );
}

/// First diagnostic code from a document expected to fail closed.
fn error_code_v1(input: &str) -> &'static str {
    parse_document_v1(input)
        .expect_err(&format!("expected `{input}` to fail closed"))
        .diagnostics
        .into_iter()
        .next()
        .expect("one diagnostic")
        .code
}

struct ExportCaseV1 {
    written: &'static str,
    kind: ExportKindV1,
    canonical_fragment: &'static str,
}

#[test]
fn every_export_kind_decodes_and_round_trips_v1() {
    let cases = [
        ExportCaseV1 {
            written: "json",
            kind: ExportKindV1::Json,
            canonical_fragment: "kind=json",
        },
        ExportCaseV1 {
            written: "csv",
            kind: ExportKindV1::Csv,
            canonical_fragment: "kind=csv",
        },
        ExportCaseV1 {
            written: "sarif",
            kind: ExportKindV1::Sarif,
            canonical_fragment: "kind=sarif",
        },
        ExportCaseV1 {
            written: "markdown",
            kind: ExportKindV1::Markdown,
            canonical_fragment: "kind=markdown",
        },
        ExportCaseV1 {
            written: "graphml",
            kind: ExportKindV1::GraphMl,
            // graphml and graph-ml both decode; the canonical emission is graph-ml.
            canonical_fragment: "kind=graph-ml",
        },
    ];
    assert_eq!(cases.len(), 5, "export matrix must cover all five kinds");

    for case in &cases {
        let tail = format!("source=result::latest ;; kind={}", case.written);

        // Low-level decode API.
        let decoded = decode_export_intent_v1(&tail)
            .unwrap_or_else(|err| panic!("decode export `{}`: {err:?}", case.written));
        assert_eq!(
            decoded.kind, case.kind,
            "decode kind for `{}`",
            case.written
        );

        // Full front-door parse + lazy intent decode.
        let document = format!("EXPORT {tail}");
        match parse_document_v1(&document)
            .unwrap_or_else(|page| panic!("parse export `{}`: {page:?}", case.written))
            .roots
            .into_iter()
            .next()
            .expect("one root")
        {
            AstRootV1::Export(root) => assert_eq!(
                root.intent().expect("export intent").kind,
                case.kind,
                "front-door kind for `{}`",
                case.written
            ),
            other => panic!("expected EXPORT root for `{}`, got {other:?}", case.written),
        }

        assert_fmt_fixed_point_with_fragment_v1(&document, case.canonical_fragment);
    }

    // The `graph-ml` spelling is the second accepted alias for the same variant.
    assert_eq!(
        decode_export_intent_v1("source=result::latest ;; kind=graph-ml")
            .expect("graph-ml decode")
            .kind,
        ExportKindV1::GraphMl
    );

    // Unknown kind fails closed at the front door.
    assert_eq!(
        error_code_v1("EXPORT source=result::latest ;; kind=pdf"),
        "invalid_export_payload",
        "unknown EXPORT kind must fail closed"
    );
}

struct SampleCaseV1 {
    written: &'static str,
    mode: SampleRetrievalModeV1,
    canonical_fragment: &'static str,
}

#[test]
fn every_sample_mode_decodes_and_round_trips_v1() {
    let cases = [
        SampleCaseV1 {
            written: "lexical",
            mode: SampleRetrievalModeV1::Lexical,
            canonical_fragment: "mode=lexical",
        },
        SampleCaseV1 {
            written: "semantic",
            mode: SampleRetrievalModeV1::Semantic,
            canonical_fragment: "mode=semantic",
        },
        SampleCaseV1 {
            written: "hybrid",
            mode: SampleRetrievalModeV1::Hybrid,
            canonical_fragment: "mode=hybrid",
        },
    ];
    assert_eq!(cases.len(), 3, "sample matrix must cover all three modes");

    for case in &cases {
        let tail = format!(
            "query=callgraph entrypoints ;; mode={} ;; limit=5",
            case.written
        );

        let decoded = decode_sample_intent_v1(&tail)
            .unwrap_or_else(|err| panic!("decode sample `{}`: {err:?}", case.written));
        assert_eq!(
            decoded.mode, case.mode,
            "decode mode for `{}`",
            case.written
        );

        let document = format!("SAMPLE {tail}");
        match parse_document_v1(&document)
            .unwrap_or_else(|page| panic!("parse sample `{}`: {page:?}", case.written))
            .roots
            .into_iter()
            .next()
            .expect("one root")
        {
            AstRootV1::Sample(root) => assert_eq!(
                root.intent().expect("sample intent").mode,
                case.mode,
                "front-door mode for `{}`",
                case.written
            ),
            other => panic!("expected SAMPLE root for `{}`, got {other:?}", case.written),
        }

        assert_fmt_fixed_point_with_fragment_v1(&document, case.canonical_fragment);
    }

    // Unknown mode fails closed at the front door.
    assert_eq!(
        error_code_v1("SAMPLE query=callgraph ;; mode=fuzzy ;; limit=5"),
        "invalid_sample_payload",
        "unknown SAMPLE mode must fail closed"
    );
}

struct CompareCaseV1 {
    written: &'static str,
    metric: CompareMetricV1,
    canonical_fragment: &'static str,
}

#[test]
fn every_compare_metric_decodes_and_round_trips_v1() {
    let cases = [
        CompareCaseV1 {
            written: "exact",
            metric: CompareMetricV1::Exact,
            canonical_fragment: "metric=exact",
        },
        CompareCaseV1 {
            written: "structural",
            metric: CompareMetricV1::Structural,
            canonical_fragment: "metric=structural",
        },
        CompareCaseV1 {
            written: "semantic",
            metric: CompareMetricV1::Semantic,
            canonical_fragment: "metric=semantic",
        },
    ];
    assert_eq!(
        cases.len(),
        3,
        "compare matrix must cover all three metrics"
    );

    for case in &cases {
        let tail = format!(
            "baseline=baseline::nightly ;; candidate=result::latest ;; metric={}",
            case.written
        );

        let decoded = decode_compare_intent_v1(&tail)
            .unwrap_or_else(|err| panic!("decode compare `{}`: {err:?}", case.written));
        assert_eq!(
            decoded.metric, case.metric,
            "decode metric for `{}`",
            case.written
        );

        let document = format!("COMPARE {tail}");
        match parse_document_v1(&document)
            .unwrap_or_else(|page| panic!("parse compare `{}`: {page:?}", case.written))
            .roots
            .into_iter()
            .next()
            .expect("one root")
        {
            AstRootV1::Compare(root) => assert_eq!(
                root.intent().expect("compare intent").metric,
                case.metric,
                "front-door metric for `{}`",
                case.written
            ),
            other => panic!(
                "expected COMPARE root for `{}`, got {other:?}",
                case.written
            ),
        }

        assert_fmt_fixed_point_with_fragment_v1(&document, case.canonical_fragment);
    }

    // Unknown metric fails closed at the front door.
    assert_eq!(
        error_code_v1("COMPARE baseline=baseline::a ;; candidate=result::b ;; metric=cosine"),
        "invalid_compare_payload",
        "unknown COMPARE metric must fail closed"
    );
}
