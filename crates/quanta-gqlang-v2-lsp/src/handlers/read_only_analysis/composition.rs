//! Read-only presentation for an `LET` / `WITH` composition block (E03C
//! ticket 05).
//!
//! D2: the grouping pass folds a `[LET]* WITH <body>` run into ONE root, so the
//! composition block IS the first statement. Hover/symbols summarize that block
//! as the first-statement card (`root_keyword=LET`, `category=composition`,
//! `typed_target=<with name>`); the body's own root summary is reused verbatim
//! for the body symbol rather than re-deriving its label (no shadow grammar).

use super::super::{ReadOnlyRootSummaryV1, SymbolItemV1};
use super::common::document_symbol_v1;
use super::summary_from_root_v1;
use quanta_gqlang_v2_ir::frontend::AstRootV1;

/// Build the composition card + symbol set from the grouped composition root.
///
/// `lets`, `with_name` and `body` are the public fields of the composition AST
/// variant; the body summary is produced by the shared root dispatch so the body
/// label can never diverge from how that root is summarized standalone.
pub(super) fn composition_summary_v1(
    lets: &[(String, &AstRootV1)],
    with_name: &str,
    body: &AstRootV1,
) -> ReadOnlyRootSummaryV1 {
    let body_summary = summary_from_root_v1(body);

    let summary = vec![
        format!("lets={}", lets.len()),
        format!("with={with_name}"),
        format!("body={}", body_summary.root_keyword),
    ];

    let mut symbols = vec![document_symbol_v1()];
    // One `binding` symbol per LET, in declaration order (D4: resolution is
    // strictly declaration-order, so the surface mirrors that order).
    for (name, producer) in lets {
        let producer_summary = summary_from_root_v1(producer);
        symbols.push(SymbolItemV1 {
            name: name.clone(),
            kind: "binding".to_string(),
            detail: Some(format!("producer={}", producer_summary.root_keyword)),
        });
    }
    // The `WITH` narrowing operator names exactly one upstream binding.
    symbols.push(SymbolItemV1 {
        name: with_name.to_string(),
        kind: "with".to_string(),
        detail: Some("narrows=working-set".to_string()),
    });
    // The composed body root's own symbol, reused from its standalone summary.
    symbols.push(SymbolItemV1 {
        name: body_summary.root_keyword.to_string(),
        kind: "root".to_string(),
        detail: Some(format!("category={}", body_summary.category)),
    });

    ReadOnlyRootSummaryV1 {
        root_keyword: "LET",
        category: "composition",
        typed_target: with_name.to_string(),
        summary,
        symbols,
    }
}
