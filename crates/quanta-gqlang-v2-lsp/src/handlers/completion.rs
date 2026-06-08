use quanta_gqlang_v2_ir::catalog::autocomplete_surface::{
    autocomplete_suggestion_rows_v1, autocomplete_surface_integrity_v1,
    AutocompleteSuggestionKindV1,
};
use quanta_gqlang_v2_ir::catalog::capability_catalog::CatalogScopeMarkerV1;
use quanta_gqlang_v2_ir::catalog::example_registry::example_row_by_id_v1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionItemV1 {
    pub label: String,
    pub kind: String,
    pub detail: String,
    pub reason: String,
    pub scope: String,
    pub match_reason: String,
    pub example_query: Option<String>,
}

pub fn handle_completion(prefix: &str) -> Result<Vec<CompletionItemV1>, String> {
    // Fail closed if the autocomplete surface has drifted (e.g. a row pointing at an
    // example for a different capability). `example_query` is surfaced verbatim to
    // the user, so the same guard the tests assert must run on the live path too.
    autocomplete_surface_integrity_v1().map_err(str::to_string)?;
    let ranked = crate::handlers::completion_rank::rank_matches_v1(
        prefix,
        autocomplete_suggestion_rows_v1(),
    )?;
    Ok(ranked.into_iter().map(completion_item_v1).collect())
}

fn completion_item_v1(
    ranked: crate::handlers::completion_rank::RankedAutocompleteMatchV1<'_>,
) -> CompletionItemV1 {
    let row = ranked.row;
    CompletionItemV1 {
        label: row.label.to_string(),
        kind: kind_label_v1(row.kind).to_string(),
        detail: row.detail.to_string(),
        reason: row.reason.to_string(),
        scope: scope_label_v1(row.scope).to_string(),
        match_reason: ranked.match_reason.label_v1().to_string(),
        example_query: row
            .example_ids
            .first()
            .and_then(|example_id| example_row_by_id_v1(example_id))
            .map(|example| example.query_text.to_string()),
    }
}

fn kind_label_v1(kind: AutocompleteSuggestionKindV1) -> &'static str {
    match kind {
        AutocompleteSuggestionKindV1::Root => "root",
        AutocompleteSuggestionKindV1::InteractiveRoot => "interactive-root",
        AutocompleteSuggestionKindV1::DeclarationRoot => "declaration-root",
        AutocompleteSuggestionKindV1::MetaRoot => "meta-root",
        AutocompleteSuggestionKindV1::ShowCategory => "show-category",
        AutocompleteSuggestionKindV1::DescribeKind => "describe-kind",
    }
}

fn scope_label_v1(scope: CatalogScopeMarkerV1) -> &'static str {
    match scope {
        CatalogScopeMarkerV1::DocumentRoot => "document-root",
        CatalogScopeMarkerV1::ShowCategoryTail => "show-category-tail",
        CatalogScopeMarkerV1::DescribeTargetTail => "describe-target-tail",
        CatalogScopeMarkerV1::ReadOnlyShell => "read-only-shell",
    }
}
