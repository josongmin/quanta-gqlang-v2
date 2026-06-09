use super::super::{ReadOnlyRootSummaryV1, SymbolItemV1};
use super::common::{document_symbol_v1, root_summary_v1, root_symbol_v1};
use quanta_gqlang_v2_ir::frontend::{SelectFieldRefAstV1, SelectProjectionAstV1, SelectRootAstV1};

pub(super) fn select_summary_v1(spec: &SelectRootAstV1) -> ReadOnlyRootSummaryV1 {
    // `shape_v1` re-parses the canonical SELECT carrier; the root already validated
    // during parse_document_v1, so this only fails on an internal invariant break,
    // which we surface as a best-effort raw summary rather than panicking.
    let Ok(shape) = spec.shape_v1() else {
        return root_summary_v1(
            "SELECT",
            "query",
            spec.raw_query_v1().to_string(),
            vec![format!("query={}", spec.raw_query_v1())],
        );
    };

    let projection_fields = match &shape.projection {
        SelectProjectionAstV1::Wildcard => vec!["*".to_string()],
        SelectProjectionAstV1::Fields(fields) => fields
            .iter()
            .map(SelectFieldRefAstV1::dotted_name_v1)
            .collect(),
    };
    let table = shape.table.dotted_name_v1();

    let mut summary = vec![format!("projection={}", projection_fields.join(", "))];
    let mut symbols = vec![
        document_symbol_v1(),
        root_symbol_v1("SELECT", "query"),
        SymbolItemV1 {
            name: table.clone(),
            kind: "table".to_string(),
            detail: Some("select.target".to_string()),
        },
    ];
    symbols.extend(projection_fields.iter().map(|field| SymbolItemV1 {
        name: field.clone(),
        kind: "field".to_string(),
        detail: Some("projection".to_string()),
    }));

    if let Some(filter) = &shape.filter {
        let rendered = format!(
            "{} {} {}",
            filter.field.dotted_name_v1(),
            filter.op.as_str_v1(),
            filter.value.to_query_fragment_v1()
        );
        summary.push(format!("filter={rendered}"));
        symbols.push(SymbolItemV1 {
            name: rendered,
            kind: "filter".to_string(),
            detail: Some("select.where".to_string()),
        });
    }
    if let Some(limit_value) = shape.limit {
        summary.push(format!("limit={limit_value}"));
    }

    ReadOnlyRootSummaryV1 {
        root_keyword: "SELECT",
        category: "query",
        typed_target: table,
        summary,
        symbols,
    }
}
