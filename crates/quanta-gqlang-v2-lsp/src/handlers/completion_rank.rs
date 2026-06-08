use quanta_gqlang_v2_ir::catalog::autocomplete_surface::{
    AutocompleteSuggestionCategoryV1, AutocompleteSuggestionRowV1,
};
use quanta_gqlang_v2_ir::catalog::capability_catalog::CatalogScopeMarkerV1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MatchReasonV1 {
    ExactScope,
    RootFallback,
}

impl MatchReasonV1 {
    #[must_use]
    pub(crate) const fn label_v1(self) -> &'static str {
        match self {
            Self::ExactScope => "exact-scope",
            Self::RootFallback => "root-fallback",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RankedAutocompleteMatchV1<'a> {
    pub(crate) row: &'a AutocompleteSuggestionRowV1,
    pub(crate) match_reason: MatchReasonV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CompletionContextV1 {
    Root { fragment: String },
    ShowCategoryTail { fragment: String },
    DescribeTargetTail { fragment: String },
    None,
}

pub(crate) fn rank_matches_v1<'a>(
    prefix: &str,
    rows: &'a [AutocompleteSuggestionRowV1],
) -> Result<Vec<RankedAutocompleteMatchV1<'a>>, String> {
    let context = parse_completion_context_v1(prefix)?;
    let mut ranked = rows
        .iter()
        .filter_map(|row| ranked_match_v1(row, &context))
        .collect::<Vec<_>>();
    ranked.sort_by_key(|match_row| {
        (
            match_reason_rank_v1(match_row.match_reason),
            scope_rank_v1(match_row.row.scope),
            category_rank_v1(match_row.row.category),
            match_row.row.label.len(),
            match_row.row.label,
        )
    });
    Ok(ranked)
}

fn ranked_match_v1<'a>(
    row: &'a AutocompleteSuggestionRowV1,
    context: &CompletionContextV1,
) -> Option<RankedAutocompleteMatchV1<'a>> {
    match context {
        CompletionContextV1::Root { fragment } => {
            if !label_matches_v1(row.label, fragment) {
                return None;
            }
            let match_reason = if row.scope == CatalogScopeMarkerV1::DocumentRoot {
                MatchReasonV1::ExactScope
            } else {
                MatchReasonV1::RootFallback
            };
            Some(RankedAutocompleteMatchV1 { row, match_reason })
        }
        CompletionContextV1::ShowCategoryTail { fragment } => {
            if row.scope != CatalogScopeMarkerV1::ShowCategoryTail
                || !label_matches_v1(row.label, fragment)
            {
                return None;
            }
            Some(RankedAutocompleteMatchV1 {
                row,
                match_reason: MatchReasonV1::ExactScope,
            })
        }
        CompletionContextV1::DescribeTargetTail { fragment } => {
            if row.scope != CatalogScopeMarkerV1::DescribeTargetTail
                || !label_matches_v1(row.label, fragment)
            {
                return None;
            }
            Some(RankedAutocompleteMatchV1 {
                row,
                match_reason: MatchReasonV1::ExactScope,
            })
        }
        CompletionContextV1::None => None,
    }
}

fn category_rank_v1(category: AutocompleteSuggestionCategoryV1) -> u8 {
    match category {
        AutocompleteSuggestionCategoryV1::QueryKeyword => 0,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword => 1,
        AutocompleteSuggestionCategoryV1::InteractiveKeyword => 2,
        AutocompleteSuggestionCategoryV1::MetaKeyword => 3,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword => 4,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix => 5,
    }
}

fn parse_completion_context_v1(prefix: &str) -> Result<CompletionContextV1, String> {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return Ok(CompletionContextV1::Root {
            fragment: String::new(),
        });
    }

    // A trailing space means the root keyword is complete and the cursor is in
    // argument position, so `SHOW ` / `DESCRIBE ` must enter their tail contexts
    // with an empty fragment (full category/target list) rather than re-matching
    // the keyword the user just finished typing.
    let (head, tail): (&str, &str) = match split_head_v1(trimmed) {
        Some((head, tail)) => (head, tail),
        None if prefix.ends_with(char::is_whitespace) => (trimmed, ""),
        None => {
            return Ok(CompletionContextV1::Root {
                fragment: trimmed.to_string(),
            });
        }
    };

    if head.eq_ignore_ascii_case("SHOW") {
        return Ok(CompletionContextV1::ShowCategoryTail {
            fragment: tail.trim().to_string(),
        });
    }

    if head.eq_ignore_ascii_case("DESCRIBE") {
        let fragment = tail.trim();
        if fragment.ends_with("::") {
            return Err(
                "class=syntax code=invalid_describe_path message=DESCRIBE target path cannot be empty"
                    .to_string(),
            );
        }
        if fragment.contains("::") {
            return Ok(CompletionContextV1::None);
        }
        return Ok(CompletionContextV1::DescribeTargetTail {
            fragment: fragment.to_string(),
        });
    }

    if matches!(head.to_ascii_uppercase().as_str(), "USE" | "REQUIRE") {
        return Ok(CompletionContextV1::Root {
            fragment: trimmed.to_string(),
        });
    }

    if exact_root_v1(head) && !tail.trim().is_empty() {
        return Ok(CompletionContextV1::None);
    }

    Ok(CompletionContextV1::Root {
        fragment: trimmed.to_string(),
    })
}

fn exact_root_v1(head: &str) -> bool {
    matches!(
        head.to_ascii_uppercase().as_str(),
        "SELECT"
            | "FIND"
            | "CHECK"
            | "EXPLAIN"
            | "SHOW"
            | "DESCRIBE"
            | "SAMPLE"
            | "COMPARE"
            | "EXPORT"
            | "VIEW"
            | "RULE"
            | "MODEL"
            | "PACK"
            | "PROFILE"
            | "GATE"
            | "USE"
            | "REQUIRE"
    )
}

fn split_head_v1(input: &str) -> Option<(&str, &str)> {
    let head_end = input.find(char::is_whitespace)?;
    let (head, tail) = input.split_at(head_end);
    Some((head, tail))
}

fn label_matches_v1(label: &str, fragment: &str) -> bool {
    let normalized_fragment = normalized_v1(fragment);
    if normalized_fragment.is_empty() {
        return true;
    }
    normalized_v1(label).starts_with(&normalized_fragment)
}

fn match_reason_rank_v1(reason: MatchReasonV1) -> u8 {
    match reason {
        MatchReasonV1::ExactScope => 0,
        MatchReasonV1::RootFallback => 1,
    }
}

fn scope_rank_v1(scope: CatalogScopeMarkerV1) -> u8 {
    match scope {
        CatalogScopeMarkerV1::DocumentRoot => 0,
        CatalogScopeMarkerV1::ShowCategoryTail => 1,
        CatalogScopeMarkerV1::DescribeTargetTail => 2,
        CatalogScopeMarkerV1::ReadOnlyShell => 3,
    }
}

fn normalized_v1(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "")
}

#[cfg(test)]
mod tests {
    use super::rank_matches_v1;
    use quanta_gqlang_v2_ir::catalog::autocomplete_surface::autocomplete_suggestion_rows_v1;
    use quanta_gqlang_v2_ir::catalog::capability_catalog::CatalogScopeMarkerV1;

    #[test]
    fn show_with_trailing_space_lists_categories_not_the_keyword() {
        let matches =
            rank_matches_v1("SHOW ", autocomplete_suggestion_rows_v1()).unwrap_or_default();
        let labels = matches
            .iter()
            .map(|entry| entry.row.label)
            .collect::<Vec<_>>();
        assert!(
            labels.contains(&"capabilities"),
            "`SHOW ` should list catalog categories, got {labels:?}"
        );
    }

    #[test]
    fn describe_with_trailing_space_lists_targets_not_the_keyword() {
        let matches =
            rank_matches_v1("DESCRIBE ", autocomplete_suggestion_rows_v1()).unwrap_or_default();
        assert!(
            !matches.is_empty()
                && matches
                    .iter()
                    .all(|entry| entry.row.scope == CatalogScopeMarkerV1::DescribeTargetTail),
            "`DESCRIBE ` should list describe targets, got {:?}",
            matches
                .iter()
                .map(|entry| entry.row.label)
                .collect::<Vec<_>>()
        );
    }
}
