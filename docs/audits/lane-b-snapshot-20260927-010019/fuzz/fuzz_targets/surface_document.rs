#![no_main]

#[path = "../../crates/quanta-gqlang-v2-surface/tests/support/mod.rs"]
mod support;

use libfuzzer_sys::fuzz_target;
use quanta_gqlang_v2_hir::{check_document_v2, HirBuilderV1, HirDocumentV1, SourceIdV2};
use quanta_gqlang_v2_surface::{format_document, parse_document, ParseErrorCodeV1, SurfaceItemV1};

fuzz_target!(|data: &[u8]| {
    // The surface parser receives UTF-8 text. Invalid transport bytes belong
    // to the CLI/stdio ingress contract; lossy decoding still lets every byte
    // sequence exercise parser and formatter totality.
    if data.len() > 4096 {
        return;
    }
    let source = String::from_utf8_lossy(data);
    match parse_document(&source, &HirBuilderV1::new()) {
        Ok(parsed) => {
            let items = parsed
                .items
                .iter()
                .map(SurfaceItemV1::to_program_item)
                .collect::<Vec<_>>();
            let document = HirDocumentV1 {
                source_id: SourceIdV2::from_raw(0),
                source_len: source.len(),
                items: items.clone(),
            };
            assert!(
                check_document_v2(&document).is_ok(),
                "parser accepted invalid HIR"
            );
            let canonical = format_document(&items);
            let reparsed = parse_document(&canonical, &HirBuilderV1::new())
                .expect("formatted accepted document must parse");
            let reparsed_items = reparsed
                .items
                .iter()
                .map(SurfaceItemV1::to_program_item)
                .collect::<Vec<_>>();
            let original_structure = items
                .iter()
                .map(support::structural_witness)
                .collect::<Vec<_>>();
            let reparsed_structure = reparsed_items
                .iter()
                .map(support::structural_witness)
                .collect::<Vec<_>>();
            assert_eq!(reparsed_structure, original_structure);
            assert_eq!(format_document(&reparsed_items), canonical);
        }
        Err(diagnostic) => {
            assert!(diagnostic.span.start <= diagnostic.span.end);
            assert!(diagnostic.span.end <= source.len());
            assert!(source.is_char_boundary(diagnostic.span.start));
            assert!(source.is_char_boundary(diagnostic.span.end));
            assert_eq!(
                diagnostic.resource.is_some(),
                diagnostic.code == ParseErrorCodeV1::ResourceExhausted
            );
            if let Some(resource) = diagnostic.resource {
                assert!(resource.observed > resource.limit);
            }
        }
    }
});
