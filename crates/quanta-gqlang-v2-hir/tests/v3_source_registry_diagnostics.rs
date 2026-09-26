#![expect(
    clippy::expect_used,
    reason = "exact diagnostic integration assertions"
)]

use quanta_gqlang_v2_hir::{
    LocatedSpanErrorV2, LocatedSpanV2, SourceIdV2, SourceRegistrationV2,
    SourceRegistryDiagnosticV2, SourceRegistryViewV2, SpanV1,
};

#[test]
fn registry_failures_retain_exact_payload_and_display() {
    let source = SourceIdV2::from_raw(17);
    let entry = SourceRegistrationV2 {
        source_id: source,
        source_len: 13,
    };
    let duplicate = SourceRegistryViewV2::try_new(vec![
        entry,
        SourceRegistrationV2 {
            source_id: source,
            source_len: 29,
        },
    ])
    .expect_err("duplicate source must reject");
    assert_eq!(
        duplicate,
        SourceRegistryDiagnosticV2::DuplicateSourceId {
            source_id: source,
            original_len: 13,
            duplicate_len: 29,
        }
    );
    assert_eq!(duplicate.code(), "hir.source_registry.duplicate_source_id");
    assert_eq!(
        duplicate.to_string(),
        "hir.source_registry.duplicate_source_id: source 17 registered twice with lengths 13 and 29"
    );

    let registry = SourceRegistryViewV2::try_new(vec![entry]).expect("registered source");
    let unknown = registry
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(99),
            span: SpanV1::new(2, 7),
        })
        .expect_err("unknown source must reject");
    assert_eq!(
        unknown,
        SourceRegistryDiagnosticV2::UnknownSourceId {
            source_id: SourceIdV2::from_raw(99),
        }
    );
    assert_eq!(unknown.code(), "hir.source_registry.unknown_source_id");
    assert_eq!(
        unknown.to_string(),
        "hir.source_registry.unknown_source_id: source 99 is not registered"
    );

    let inverted_location = LocatedSpanV2 {
        source_id: source,
        span: SpanV1::new(9, 4),
    };
    let inverted = registry
        .try_check_location(inverted_location)
        .expect_err("inverted span must reject");
    assert_eq!(
        inverted,
        SourceRegistryDiagnosticV2::InvalidLocation {
            location: inverted_location,
            error: LocatedSpanErrorV2::Inverted {
                span: SpanV1::new(9, 4),
            },
        }
    );
    assert_eq!(inverted.code(), "hir.source_registry.inverted_span");
    assert_eq!(
        inverted.to_string(),
        "hir.source_registry.inverted_span: span 9..4 is invalid for source 17"
    );

    let out_of_bounds_location = LocatedSpanV2 {
        source_id: source,
        span: SpanV1::new(3, 14),
    };
    let out_of_bounds = registry
        .try_check_location(out_of_bounds_location)
        .expect_err("span past the source must reject");
    assert_eq!(
        out_of_bounds,
        SourceRegistryDiagnosticV2::InvalidLocation {
            location: out_of_bounds_location,
            error: LocatedSpanErrorV2::OutOfBounds {
                span: SpanV1::new(3, 14),
                source_len: 13,
            },
        }
    );
    assert_eq!(
        out_of_bounds.code(),
        "hir.source_registry.location_out_of_bounds"
    );
    assert_eq!(
        out_of_bounds.to_string(),
        "hir.source_registry.location_out_of_bounds: span 3..14 is invalid for source 17"
    );

    // Exact-end and zero-width controls exclude an always-rejecting registry.
    for span in [SpanV1::new(3, 13), SpanV1::point(13)] {
        let location = LocatedSpanV2 {
            source_id: source,
            span,
        };
        assert_eq!(
            registry
                .try_check_location(location)
                .expect("in-bounds control")
                .location(),
            location
        );
    }
}
