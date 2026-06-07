use quanta_gqlang_v2_ir::catalog::capability_catalog::capability_row_by_id_v1;
use quanta_gqlang_v2_ir::catalog::example_registry::example_row_by_id_v1;

use crate::handlers::hover::{analyze_query_v1, ReadOnlyLspErrorV1, ReadOnlyRootSummaryV1};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickfixSuggestionV1 {
    pub code: String,
    pub label: String,
    pub detail: String,
    pub scaffold: String,
    pub edits_document: bool,
}

pub fn handle_quickfix(input: &str) -> Result<Vec<QuickfixSuggestionV1>, String> {
    match analyze_query_v1(input) {
        Ok(summary) => quickfix_from_summary_v1(&summary),
        Err(err) => quickfix_from_error_v1(input, &err),
    }
}

fn quickfix_from_summary_v1(
    summary: &ReadOnlyRootSummaryV1,
) -> Result<Vec<QuickfixSuggestionV1>, String> {
    if let Some(model_name) = summary.typed_target.strip_prefix("model::") {
        return Ok(vec![missing_model_quickfix_v1(model_name)?]);
    }
    if let Some(profile_name) = summary.typed_target.strip_prefix("profile::") {
        return Ok(vec![missing_profile_quickfix_v1(profile_name)?]);
    }
    Ok(Vec::new())
}

fn quickfix_from_error_v1(
    input: &str,
    err: &ReadOnlyLspErrorV1,
) -> Result<Vec<QuickfixSuggestionV1>, String> {
    match detect_root_v1(input) {
        Some("MODEL") if matches!(err.code, "missing_named_target" | "missing_required_clause") => {
            Ok(vec![missing_model_quickfix_v1(
                extract_named_target_v1(input)
                    .as_deref()
                    .unwrap_or("new_model"),
            )?])
        }
        Some("PROFILE")
            if matches!(err.code, "missing_named_target" | "missing_required_clause") =>
        {
            Ok(vec![missing_profile_quickfix_v1(
                extract_named_target_v1(input)
                    .as_deref()
                    .unwrap_or("strict-review"),
            )?])
        }
        Some("VIEW") if err.code == "missing_view_query_shape" => {
            Ok(vec![missing_query_quickfix_v1(
                extract_named_target_v1(input)
                    .as_deref()
                    .unwrap_or("catalog.users"),
            )?])
        }
        Some("SELECT")
            if matches!(
                err.code,
                "missing_from_clause" | "missing_projection" | "missing_table_ref"
            ) =>
        {
            Ok(vec![starter_query_quickfix_v1(
                "missing-query-scaffold",
                "Add SELECT starter",
                "example.select.catalog_users",
            )?])
        }
        Some("DESCRIBE") | Some("SHOW") | Some("VIEW") | Some("MODEL") | Some("PROFILE") => {
            Err(err.render_v1())
        }
        _ => Ok(Vec::new()),
    }
}

fn missing_profile_quickfix_v1(profile_name: &str) -> Result<QuickfixSuggestionV1, String> {
    let capability = capability_row_by_id_v1("declaration.profile")
        .ok_or_else(|| "missing PROFILE capability catalog row".to_string())?;
    let scaffold_name = if profile_name.is_empty() {
        "strict-review"
    } else {
        profile_name
    };
    Ok(QuickfixSuggestionV1 {
        code: "missing-profile-scaffold".to_string(),
        label: format!("Declare PROFILE {scaffold_name}"),
        detail: capability.summary.to_string(),
        scaffold: format!(
            "PROFILE {scaffold_name} ;; capabilities=graph.read,graph.trace ;; admission=certified-only"
        ),
        edits_document: false,
    })
}

fn missing_query_quickfix_v1(view_name: &str) -> Result<QuickfixSuggestionV1, String> {
    let capability = capability_row_by_id_v1("declaration.view")
        .ok_or_else(|| "missing VIEW capability catalog row".to_string())?;
    Ok(QuickfixSuggestionV1 {
        code: "missing-query-scaffold".to_string(),
        label: format!("Add VIEW query scaffold for {view_name}"),
        detail: capability.summary.to_string(),
        scaffold: format!("VIEW {view_name} ;; query=SELECT id FROM catalog.users LIMIT 25"),
        edits_document: false,
    })
}

fn starter_query_quickfix_v1(
    code: &str,
    label: &str,
    example_id: &str,
) -> Result<QuickfixSuggestionV1, String> {
    let example = example_row_by_id_v1(example_id)
        .ok_or_else(|| format!("missing quickfix example `{example_id}`"))?;
    Ok(QuickfixSuggestionV1 {
        code: code.to_string(),
        label: label.to_string(),
        detail: example.reason.to_string(),
        scaffold: example.query_text.to_string(),
        edits_document: false,
    })
}

fn missing_model_quickfix_v1(model_name: &str) -> Result<QuickfixSuggestionV1, String> {
    let capability = capability_row_by_id_v1("declaration.model")
        .ok_or_else(|| "missing MODEL capability catalog row".to_string())?;
    let scaffold_name = if model_name.is_empty() {
        "new_model"
    } else {
        model_name
    };
    Ok(QuickfixSuggestionV1 {
        code: "missing-model-scaffold".to_string(),
        label: format!("Declare MODEL {scaffold_name}"),
        detail: capability.summary.to_string(),
        scaffold: format!(
            "MODEL {scaffold_name} ;; provider=provider.default ;; source=repo::{}",
            scaffold_name.replace('.', "_")
        ),
        edits_document: false,
    })
}

pub fn example_backed_quickfix_v1(example_id: &str) -> Option<QuickfixSuggestionV1> {
    let example = example_row_by_id_v1(example_id)?;
    Some(QuickfixSuggestionV1 {
        code: format!("example:{}", example.stable_id),
        label: example.title.to_string(),
        detail: example.reason.to_string(),
        scaffold: example.query_text.to_string(),
        edits_document: false,
    })
}

fn detect_root_v1(input: &str) -> Option<&str> {
    input.split_whitespace().next()
}

fn extract_named_target_v1(input: &str) -> Option<String> {
    let remainder = input.split_once(char::is_whitespace)?.1.trim();
    let target = remainder
        .split(";;")
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .trim_matches(':');
    if target.is_empty() {
        return None;
    }
    Some(target.to_string())
}
