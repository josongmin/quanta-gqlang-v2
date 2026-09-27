#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert sealed authority and compiled outcomes"
)]

use quanta_gqlang_v2_frontend_contract::{
    compile_source_document_v3, ClosedFrontendAuthorityBuilderV2, FrontendAuthorityLocationV2,
    FrontendAuthoritySourceV2, FrontendBudgetV2, FrontendLanguageVersionV2,
    FrontendTransactionErrorV2, SourceIdV2,
};

const HOST: SourceIdV2 = SourceIdV2::from_raw(1);
const DOCUMENT: SourceIdV2 = SourceIdV2::from_raw(2);

fn authority() -> quanta_gqlang_v2_frontend_contract::ClosedFrontendAuthorityV2 {
    ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST,
            source_len: 1,
        }],
        "app".to_owned(),
        FrontendAuthorityLocationV2 {
            source_id: HOST,
            start: 0,
            end: 1,
        },
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("authority builder")
    .finish()
    .expect("closed authority")
}

fn fanout_source(fields: usize) -> String {
    let mut source = "let x0 = 1;".to_owned();
    for index in 1..=11 {
        source.push_str(&format!(
            " let x{index} = {{a:x{}, b:x{}}};",
            index - 1,
            index - 1
        ));
    }
    source.push_str(" let out = {");
    for index in 0..fields {
        if index > 0 {
            source.push_str(", ");
        }
        source.push_str(&format!("g{index}:x11"));
    }
    source.push_str("};");
    source
}

#[test]
fn source_declaration_fanout_accepts_exact_parent_and_rejects_oversized_parent() {
    let authority = authority();
    let exact = compile_source_document_v3(DOCUMENT, &fanout_source(1), &authority)
        .expect("4095-node child and 4096-node parent are valid");
    assert_eq!(exact.items().len(), 13);

    let oversized_source = fanout_source(4096);
    let offending_start = oversized_source.find("g1:x11").expect("second field");
    let error = compile_source_document_v3(DOCUMENT, &oversized_source, &authority)
        .expect_err("a second 4095-node child exceeds the record limit");
    let FrontendTransactionErrorV2::Document(failure) = &error else {
        panic!("expected source typing diagnostic, got {error:?}");
    };
    let diagnostic = failure.diagnostics().first().expect("typing diagnostic");
    assert_eq!(diagnostic.code(), "types.type_shape_limit");
    assert_eq!(diagnostic.stage().keyword(), "typing");
    assert_eq!(diagnostic.primary().source_id, DOCUMENT);
    assert_eq!(
        diagnostic.primary().span,
        quanta_gqlang_v2_frontend_contract::SpanV1::new(offending_start, offending_start + 2)
    );
}

#[test]
fn later_invalid_field_name_precedes_parent_shape_limit() {
    let mut source = fanout_source(2);
    source.truncate(source.len() - 2);
    let invalid_name = "x".repeat(257);
    let invalid_start = source.len() + 2;
    source.push_str(&format!(", {invalid_name}:x11}};"));
    let error = compile_source_document_v3(DOCUMENT, &source, &authority())
        .expect_err("invalid field name wins over parent shape");
    let FrontendTransactionErrorV2::Document(failure) = &error else {
        panic!("expected source typing diagnostic, got {error:?}");
    };
    let diagnostic = failure.diagnostics().first().expect("typing diagnostic");
    assert_eq!(diagnostic.code(), "types.invalid_type_name");
    assert_eq!(diagnostic.stage().keyword(), "typing");
    assert_eq!(
        diagnostic.primary().span,
        quanta_gqlang_v2_frontend_contract::SpanV1::new(
            invalid_start,
            invalid_start + invalid_name.len(),
        )
    );
}

#[test]
fn nested_list_shape_failure_points_to_inner_expression() {
    let mut source = fanout_source(0);
    source.truncate(source.len() - 2);
    source.push_str("nested:[[x11]]};");
    let inner_start = source.find("[[x11]]").expect("nested list");
    let error = compile_source_document_v3(DOCUMENT, &source, &authority())
        .expect_err("nested list exceeds type shape");
    let FrontendTransactionErrorV2::Document(failure) = &error else {
        panic!("expected source typing diagnostic, got {error:?}");
    };
    let diagnostic = failure.diagnostics().first().expect("typing diagnostic");
    assert_eq!(diagnostic.code(), "types.type_shape_limit");
    assert_eq!(diagnostic.stage().keyword(), "typing");
    assert_eq!(diagnostic.primary().source_id, DOCUMENT);
    assert_eq!(
        diagnostic.primary().span,
        quanta_gqlang_v2_frontend_contract::SpanV1::new(inner_start, inner_start + 7)
    );
}
