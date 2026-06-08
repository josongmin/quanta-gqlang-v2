#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::lower_document_to_ir_v1;
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;
use quanta_gqlang_v2_ir::select_ir::{
    SelectFieldRefIrV1, SelectFilterOpIrV1, SelectLiteralIrV1, SelectProjectionIrV1,
};
use quanta_gqlang_v2_sema::analyze_document_v1;
use quanta_gqlang_v2_syntax::parse_document_v1;

fn lower_root_v1(input: &str) -> PublicIrRootV1 {
    lower_document_to_ir_v1(
        &analyze_document_v1(
            &parse_document_v1(input)
                .unwrap_or_else(|page| panic!("expected parse success for `{input}`: {page:?}")),
        )
        .unwrap_or_else(|page| panic!("expected sema success for `{input}`: {page:?}")),
    )
    .unwrap_or_else(|err| panic!("expected IR lowering success for `{input}`: {err:?}"))
    .into_iter()
    .next()
    .expect("one root")
}

#[test]
fn select_vertical_slice_keeps_table_only_row_shape() {
    let root = lower_root_v1(
        "SELECT symbol.name FROM catalog.symbols WHERE symbol.kind = 'function' LIMIT 20",
    );
    let PublicIrRootV1::Select(select) = root else {
        panic!("expected SELECT IR root");
    };

    let shape = select.shape_v1().expect("SELECT shape");
    assert_eq!(shape.table.dotted_name_v1(), "catalog.symbols");
    assert_eq!(
        shape.projection,
        SelectProjectionIrV1::Fields(vec![SelectFieldRefIrV1 {
            field_path: vec!["symbol".to_string(), "name".to_string()],
        }]),
    );
    let filter = shape.filter.expect("filter must exist");
    assert_eq!(
        filter.field.field_path,
        vec!["symbol".to_string(), "kind".to_string()]
    );
    assert_eq!(filter.op, SelectFilterOpIrV1::Eq);
    assert_eq!(
        filter.value,
        SelectLiteralIrV1::String("function".to_string())
    );
    assert_eq!(shape.limit, Some(20));
}

#[test]
fn select_vertical_slice_rejects_non_table_widening_fail_closed() {
    let parsed = parse_document_v1("SELECT id FROM catalog.users JOIN catalog.teams");

    assert_eq!(
        parsed,
        Err(quanta_gqlang_v2_syntax::SyntaxDiagnosticPageV1 {
            diagnostics: vec![quanta_gqlang_v2_syntax::SyntaxDiagnosticV1 {
                code: "unsupported_table_shape",
                message: "JOIN is not supported; SELECT roots accept exactly one table in FROM"
                    .to_string(),
                span: None,
            }],
        }),
    );
}
