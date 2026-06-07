#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardV1 {
    pub root_keyword: String,
    pub category: String,
    pub typed_target: String,
    pub summary: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolItemV1 {
    pub name: String,
    pub kind: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticItemV1 {
    pub class: String,
    pub code: String,
    pub message: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum DiagnosticClassV1 {
    Syntax,
    Sema,
}

impl DiagnosticClassV1 {
    #[must_use]
    pub(crate) const fn label_v1(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Sema => "sema",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReadOnlyLspErrorV1 {
    pub class: DiagnosticClassV1,
    pub code: &'static str,
    pub message: String,
}

impl ReadOnlyLspErrorV1 {
    #[must_use]
    pub(crate) fn render_v1(&self) -> String {
        format!(
            "class={} code={} message={}",
            self.class.label_v1(),
            self.code,
            self.message
        )
    }

    #[must_use]
    pub(crate) fn diagnostic_item_v1(&self) -> DiagnosticItemV1 {
        DiagnosticItemV1 {
            class: self.class.label_v1().to_string(),
            code: self.code.to_string(),
            message: self.message.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReadOnlyRootSummaryV1 {
    pub root_keyword: &'static str,
    pub category: &'static str,
    pub typed_target: String,
    pub summary: Vec<String>,
    pub symbols: Vec<SymbolItemV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PayloadFieldV1 {
    key: String,
    value: String,
}

pub fn handle_hover(query: &str) -> Result<HoverCardV1, String> {
    analyze_query_v1(query)
        .map(|summary| HoverCardV1 {
            root_keyword: summary.root_keyword.to_string(),
            category: summary.category.to_string(),
            typed_target: summary.typed_target,
            summary: summary.summary,
        })
        .map_err(|err| err.render_v1())
}

pub(crate) fn analyze_query_v1(query: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(syntax_error_v1("empty_document", "document is empty"));
    }

    let upper = trimmed.to_ascii_uppercase();
    if upper.starts_with("USE ") {
        return analyze_use_meta_v1(trimmed);
    }
    if upper.starts_with("REQUIRE ") {
        return analyze_require_meta_v1(trimmed);
    }

    let (keyword, tail) = split_head_v1(trimmed)
        .ok_or_else(|| syntax_error_v1("empty_document", "document is empty"))?;
    match keyword.to_ascii_uppercase().as_str() {
        "SELECT" => analyze_select_v1(tail),
        "FIND" => analyze_find_v1(tail),
        "CHECK" => analyze_check_v1(tail),
        "EXPLAIN" => analyze_explain_v1(tail),
        "SHOW" => analyze_show_v1(tail),
        "DESCRIBE" => analyze_describe_v1(tail),
        "SAMPLE" => analyze_sample_v1(tail),
        "COMPARE" => analyze_compare_v1(tail),
        "EXPORT" => analyze_export_v1(tail),
        "VIEW" => analyze_view_v1(tail),
        "RULE" => analyze_rule_v1(tail),
        "MODEL" => analyze_model_v1(tail),
        "PACK" => analyze_pack_v1(tail),
        "PROFILE" => analyze_profile_v1(tail),
        "GATE" => analyze_gate_v1(tail),
        _ => Err(syntax_error_v1(
            "unsupported_root_keyword",
            format!("unsupported root keyword `{keyword}`"),
        )),
    }
}

fn analyze_select_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let Some((projection_raw, after_from)) = split_once_case_insensitive_v1(tail, " FROM ") else {
        return Err(syntax_error_v1(
            "missing_from_clause",
            "SELECT root requires a FROM clause",
        ));
    };
    let projection_raw = projection_raw.trim();
    if projection_raw.is_empty() {
        return Err(syntax_error_v1(
            "missing_projection",
            "SELECT projection cannot be empty",
        ));
    }

    let projection_fields = parse_projection_fields_v1(projection_raw)?;
    let after_from = after_from.trim();
    let (table_raw, remainder) = split_select_table_and_remainder_v1(after_from);
    if table_raw.is_empty() {
        return Err(syntax_error_v1(
            "missing_table_ref",
            "SELECT table reference cannot be empty",
        ));
    }
    if table_raw.contains(char::is_whitespace) || table_raw.contains(',') {
        return Err(sema_error_v1(
            "unsupported_table_shape",
            format!("SELECT table shape `{table_raw}` is unsupported"),
        ));
    }

    let mut summary = vec![format!("projection={}", projection_fields.join(", "))];
    let mut symbols = vec![
        SymbolItemV1 {
            name: "document".to_string(),
            kind: "document".to_string(),
            detail: Some("roots=1".to_string()),
        },
        SymbolItemV1 {
            name: "SELECT".to_string(),
            kind: "root".to_string(),
            detail: Some("category=query".to_string()),
        },
        SymbolItemV1 {
            name: table_raw.to_string(),
            kind: "table".to_string(),
            detail: Some("select.target".to_string()),
        },
    ];
    symbols.extend(projection_fields.iter().map(|field| SymbolItemV1 {
        name: field.clone(),
        kind: "field".to_string(),
        detail: Some("projection".to_string()),
    }));

    let mut limit = None;
    if !remainder.is_empty() {
        let (where_clause, limit_clause) = split_select_remainder_v1(remainder)?;
        if let Some(where_clause) = where_clause {
            summary.push(format!("filter={where_clause}"));
            symbols.push(SymbolItemV1 {
                name: where_clause.clone(),
                kind: "filter".to_string(),
                detail: Some("select.where".to_string()),
            });
        }
        if let Some(limit_value) = limit_clause {
            limit = Some(limit_value);
        }
    }
    if let Some(limit_value) = limit {
        summary.push(format!("limit={limit_value}"));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "SELECT",
        category: "query",
        typed_target: table_raw.to_string(),
        summary,
        symbols,
    })
}

fn analyze_find_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (subkind, seed) = split_head_v1(tail).ok_or_else(|| {
        syntax_error_v1(
            "missing_find_subkind",
            "FIND root requires a subkind and seed",
        )
    })?;
    if seed.trim().is_empty() {
        return Err(syntax_error_v1(
            "missing_find_seed",
            "FIND root requires a non-empty seed",
        ));
    }
    let normalized = normalize_token_v1(subkind);
    let typed_subkind = match normalized.as_str() {
        "reachability" => "reachability",
        "flow" => "flow",
        "slice" => "slice",
        "taint" => "taint",
        "path" => "path",
        "pointsto" => "points_to",
        "summary" => "summary",
        _ => {
            return Err(sema_error_v1(
                "unknown_find_subkind",
                format!("unknown FIND subkind `{subkind}`"),
            ));
        }
    };

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "FIND",
        category: "query",
        typed_target: format!("find::{typed_subkind}"),
        summary: vec![format!("seed={}", seed.trim())],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "FIND".to_string(),
                kind: "root".to_string(),
                detail: Some("category=query".to_string()),
            },
            SymbolItemV1 {
                name: typed_subkind.to_string(),
                kind: "subkind".to_string(),
                detail: Some(seed.trim().to_string()),
            },
        ],
    })
}

fn analyze_check_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (rule_id, scope) = split_head_v1(tail).ok_or_else(|| {
        syntax_error_v1(
            "missing_check_rule",
            "CHECK root requires rule id and scope",
        )
    })?;
    if scope.trim().is_empty() {
        return Err(syntax_error_v1(
            "missing_check_scope",
            "CHECK root requires a non-empty scope payload",
        ));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "CHECK",
        category: "query",
        typed_target: rule_id.to_string(),
        summary: vec![format!("scope={}", scope.trim())],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "CHECK".to_string(),
                kind: "root".to_string(),
                detail: Some("category=query".to_string()),
            },
            SymbolItemV1 {
                name: rule_id.to_string(),
                kind: "rule".to_string(),
                detail: Some(scope.trim().to_string()),
            },
        ],
    })
}

fn analyze_explain_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let target = tail.trim();
    if target.is_empty() {
        return Err(syntax_error_v1(
            "missing_explain_target",
            "EXPLAIN root requires a target payload",
        ));
    }
    let (subject, fields) = split_subject_and_fields_v1(target);
    let subject = subject.trim();
    if subject.is_empty() {
        return Err(syntax_error_v1(
            "missing_explain_subject",
            "EXPLAIN subject cannot be empty",
        ));
    }

    let evidence = optional_unique_field_v1(&fields, "evidence", DiagnosticClassV1::Sema)?
        .unwrap_or_else(|| "auto".to_string());
    if !matches!(
        normalize_token_v1(&evidence).as_str(),
        "auto" | "trace" | "citation" | "counterexample"
    ) {
        return Err(sema_error_v1(
            "unsupported_explain_evidence",
            format!("unsupported EXPLAIN evidence `{evidence}`"),
        ));
    }
    let truth = optional_unique_field_v1(&fields, "truth", DiagnosticClassV1::Sema)?
        .unwrap_or_else(|| "inspect".to_string());
    if !matches!(
        normalize_token_v1(&truth).as_str(),
        "inspect" | "confirm" | "refute"
    ) {
        return Err(sema_error_v1(
            "unsupported_explain_truth",
            format!("unsupported EXPLAIN truth expectation `{truth}`"),
        ));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "EXPLAIN",
        category: "query",
        typed_target: subject.to_string(),
        summary: vec![format!("evidence={evidence}"), format!("truth={truth}")],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "EXPLAIN".to_string(),
                kind: "root".to_string(),
                detail: Some("category=query".to_string()),
            },
            SymbolItemV1 {
                name: subject.to_string(),
                kind: "target".to_string(),
                detail: Some(format!("evidence={evidence}, truth={truth}")),
            },
        ],
    })
}

fn analyze_show_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let category = tail.trim();
    if category.is_empty() {
        return Err(syntax_error_v1(
            "missing_show_category",
            "SHOW root requires a category",
        ));
    }
    let normalized = normalize_token_v1(category);
    let typed_category = match normalized.as_str() {
        "capabilities" => "capabilities",
        "models" => "models",
        "views" => "views",
        "rules" => "rules",
        "packs" => "packs",
        "profiles" => "profiles",
        "gates" => "gates",
        _ => {
            return Err(sema_error_v1(
                "unsupported_show_category",
                format!("unsupported SHOW category `{category}`"),
            ));
        }
    };

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "SHOW",
        category: "interactive",
        typed_target: typed_category.to_string(),
        summary: vec![format!("catalog_category={typed_category}")],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "SHOW".to_string(),
                kind: "root".to_string(),
                detail: Some("category=interactive".to_string()),
            },
            SymbolItemV1 {
                name: typed_category.to_string(),
                kind: "category".to_string(),
                detail: Some("show.target".to_string()),
            },
        ],
    })
}

fn analyze_describe_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let target = tail.trim();
    if target.is_empty() {
        return Err(syntax_error_v1(
            "missing_describe_target",
            "DESCRIBE root requires a target",
        ));
    }
    let Some((kind_raw, path_raw)) = target.split_once("::") else {
        return Err(syntax_error_v1(
            "invalid_describe_target",
            "DESCRIBE target must be `<kind>::<path>`",
        ));
    };
    let kind = match normalize_token_v1(kind_raw).as_str() {
        "capability" => "capability",
        "model" => "model",
        "view" => "view",
        "rule" => "rule",
        "pack" => "pack",
        "profile" => "profile",
        "gate" => "gate",
        _ => {
            return Err(sema_error_v1(
                "unsupported_describe_kind",
                format!("unsupported DESCRIBE kind `{kind_raw}`"),
            ));
        }
    };
    let path = path_raw.trim();
    if path.is_empty() || path.split('.').any(|segment| segment.trim().is_empty()) {
        return Err(syntax_error_v1(
            "invalid_describe_path",
            "DESCRIBE target path cannot be empty",
        ));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "DESCRIBE",
        category: "interactive",
        typed_target: format!("{kind}::{path}"),
        summary: vec![format!("kind={kind}"), format!("path={path}")],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "DESCRIBE".to_string(),
                kind: "root".to_string(),
                detail: Some("category=interactive".to_string()),
            },
            SymbolItemV1 {
                name: format!("{kind}::{path}"),
                kind: "target".to_string(),
                detail: Some("describe.target".to_string()),
            },
        ],
    })
}

fn analyze_sample_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let fields = parse_payload_fields_v1(tail)?;
    let query = required_field_v1(&fields, "query", DiagnosticClassV1::Syntax)?;
    let mode = optional_unique_field_v1(&fields, "mode", DiagnosticClassV1::Sema)?
        .unwrap_or_else(|| "hybrid".to_string());
    if !matches!(
        normalize_token_v1(&mode).as_str(),
        "lexical" | "semantic" | "hybrid"
    ) {
        return Err(sema_error_v1(
            "unsupported_sample_mode",
            format!("unsupported SAMPLE mode `{mode}`"),
        ));
    }
    let limit = optional_unique_field_v1(&fields, "limit", DiagnosticClassV1::Syntax)?
        .map(|raw| parse_positive_u32_v1(&raw, "limit"))
        .transpose()?
        .unwrap_or(10);
    let scope = optional_unique_field_v1(&fields, "scope", DiagnosticClassV1::Syntax)?;

    let mut summary = vec![format!("mode={mode}"), format!("limit={limit}")];
    if let Some(scope) = scope {
        summary.push(format!("scope={scope}"));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "SAMPLE",
        category: "interactive",
        typed_target: query.clone(),
        summary,
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "SAMPLE".to_string(),
                kind: "root".to_string(),
                detail: Some("category=interactive".to_string()),
            },
            SymbolItemV1 {
                name: query,
                kind: "query".to_string(),
                detail: Some(format!("mode={mode}")),
            },
        ],
    })
}

fn analyze_compare_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let fields = parse_payload_fields_v1(tail)?;
    let baseline = required_field_v1(&fields, "baseline", DiagnosticClassV1::Syntax)?;
    let candidate = required_field_v1(&fields, "candidate", DiagnosticClassV1::Syntax)?;
    let metric = required_field_v1(&fields, "metric", DiagnosticClassV1::Sema)?;
    if !matches!(
        normalize_token_v1(&metric).as_str(),
        "exact" | "structural" | "semantic"
    ) {
        return Err(sema_error_v1(
            "unsupported_compare_metric",
            format!("unsupported COMPARE metric `{metric}`"),
        ));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "COMPARE",
        category: "interactive",
        typed_target: baseline.clone(),
        summary: vec![format!("candidate={candidate}"), format!("metric={metric}")],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "COMPARE".to_string(),
                kind: "root".to_string(),
                detail: Some("category=interactive".to_string()),
            },
            SymbolItemV1 {
                name: baseline,
                kind: "baseline".to_string(),
                detail: Some("compare.source".to_string()),
            },
            SymbolItemV1 {
                name: candidate,
                kind: "candidate".to_string(),
                detail: Some(format!("metric={metric}")),
            },
        ],
    })
}

fn analyze_export_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let fields = parse_payload_fields_v1(tail)?;
    let source = required_field_v1(&fields, "source", DiagnosticClassV1::Syntax)?;
    let kind = required_field_v1(&fields, "kind", DiagnosticClassV1::Sema)?;
    let normalized_kind = normalize_token_v1(&kind);
    if !matches!(
        normalized_kind.as_str(),
        "json" | "csv" | "sarif" | "markdown" | "graphml"
    ) {
        return Err(sema_error_v1(
            "unsupported_export_kind",
            format!("EXPORT kind `{kind}` is unsupported"),
        ));
    }
    let profile = optional_unique_field_v1(&fields, "profile", DiagnosticClassV1::Syntax)?;

    let mut summary = vec![format!("kind={kind}")];
    if let Some(profile) = profile {
        summary.push(format!("profile={profile}"));
    }

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "EXPORT",
        category: "interactive",
        typed_target: source.clone(),
        summary,
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "EXPORT".to_string(),
                kind: "root".to_string(),
                detail: Some("category=interactive".to_string()),
            },
            SymbolItemV1 {
                name: source,
                kind: "source".to_string(),
                detail: Some(format!("kind={kind}")),
            },
        ],
    })
}

fn analyze_view_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (name, fields) = parse_named_payload_v1(tail, "VIEW")?;
    let query_ref = optional_unique_field_v1(&fields, "query-ref", DiagnosticClassV1::Syntax)?;
    let inline_query = optional_unique_field_v1(&fields, "query", DiagnosticClassV1::Syntax)?;
    let query_shape = match (query_ref, inline_query) {
        (Some(query_ref), None) => format!("query-ref={query_ref}"),
        (None, Some(inline_query)) => format!("query={inline_query}"),
        (Some(_), Some(_)) => {
            return Err(sema_error_v1(
                "duplicate_view_query_shape",
                "VIEW declaration cannot carry both `query-ref` and `query`",
            ));
        }
        (None, None) => {
            return Err(syntax_error_v1(
                "missing_view_query_shape",
                "VIEW declaration requires `query-ref` or `query`",
            ));
        }
    };

    Ok(declaration_summary_v1("VIEW", &name, vec![query_shape]))
}

fn analyze_rule_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (name, fields) = parse_named_payload_v1(tail, "RULE")?;
    let check_ref = required_field_v1(&fields, "check-ref", DiagnosticClassV1::Syntax)?;
    Ok(declaration_summary_v1(
        "RULE",
        &name,
        vec![format!("check-ref={check_ref}")],
    ))
}

fn analyze_model_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (name, fields) = parse_named_payload_v1(tail, "MODEL")?;
    let provider = required_field_v1(&fields, "provider", DiagnosticClassV1::Syntax)?;
    let source = required_field_v1(&fields, "source", DiagnosticClassV1::Syntax)?;
    Ok(declaration_summary_v1(
        "MODEL",
        &name,
        vec![format!("provider={provider}"), format!("source={source}")],
    ))
}

fn analyze_pack_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (name, fields) = parse_named_payload_v1(tail, "PACK")?;
    let source = required_field_v1(&fields, "source", DiagnosticClassV1::Syntax)?;
    let activation = required_field_v1(&fields, "activation", DiagnosticClassV1::Sema)?;
    if !matches!(
        normalize_token_v1(&activation).as_str(),
        "manual" | "session" | "workspace"
    ) {
        return Err(sema_error_v1(
            "unsupported_pack_activation",
            format!("unsupported PACK activation `{activation}`"),
        ));
    }
    Ok(declaration_summary_v1(
        "PACK",
        &name,
        vec![
            format!("source={source}"),
            format!("activation={activation}"),
        ],
    ))
}

fn analyze_profile_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (name, fields) = parse_named_payload_v1(tail, "PROFILE")?;
    let capabilities = required_field_v1(&fields, "capabilities", DiagnosticClassV1::Syntax)?;
    let admission = required_field_v1(&fields, "admission", DiagnosticClassV1::Sema)?;
    if !matches!(
        normalize_token_v1(&admission).as_str(),
        "allowlist" | "reviewrequired" | "certifiedonly"
    ) {
        return Err(sema_error_v1(
            "unsupported_profile_admission",
            format!("unsupported PROFILE admission `{admission}`"),
        ));
    }
    Ok(declaration_summary_v1(
        "PROFILE",
        &name,
        vec![
            format!("capabilities={capabilities}"),
            format!("admission={admission}"),
        ],
    ))
}

fn analyze_gate_v1(tail: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let (name, fields) = parse_named_payload_v1(tail, "GATE")?;
    let checks = required_field_v1(&fields, "checks", DiagnosticClassV1::Syntax)?;
    let binding = required_field_v1(&fields, "binding", DiagnosticClassV1::Sema)?;
    if !matches!(normalize_token_v1(&binding).as_str(), "all" | "any") {
        return Err(sema_error_v1(
            "unsupported_gate_binding",
            format!("unsupported GATE binding `{binding}`"),
        ));
    }
    Ok(declaration_summary_v1(
        "GATE",
        &name,
        vec![format!("checks={checks}"), format!("binding={binding}")],
    ))
}

fn analyze_use_meta_v1(query: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let command = parse_meta_command_v1(query)?;
    let shape = command.shape().map_err(meta_shape_error_to_lsp_v1)?;
    let MetaCommandShapeV1::UsePack(spec) = shape else {
        return Err(syntax_error_v1(
            "unsupported_use_meta",
            "USE meta command currently supports only `PACK`",
        ));
    };
    let activation = pack_activation_mode_summary_v1(spec.activation_mode);
    let summary = vec![
        format!("activation={activation}"),
        format!("lockfile={}", spec.lockfile_path),
    ];

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "USE PACK",
        category: "meta",
        typed_target: spec.pack_id.to_string(),
        summary,
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "USE PACK".to_string(),
                kind: "root".to_string(),
                detail: Some("category=meta".to_string()),
            },
            SymbolItemV1 {
                name: spec.pack_id.to_string(),
                kind: "pack".to_string(),
                detail: Some(format!(
                    "activation={activation} lockfile={}",
                    spec.lockfile_path
                )),
            },
        ],
    })
}

fn analyze_require_meta_v1(query: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    let command = parse_meta_command_v1(query)?;
    let shape = command.shape().map_err(meta_shape_error_to_lsp_v1)?;
    let MetaCommandShapeV1::RequireCertified(spec) = shape else {
        return Err(syntax_error_v1(
            "unsupported_require_meta",
            "REQUIRE meta command currently supports only `CERTIFIED`",
        ));
    };
    let truth = certified_truth_summary_v1(spec.truth_requirement);
    let admission = certified_admission_summary_v1(spec.admission_policy);

    Ok(ReadOnlyRootSummaryV1 {
        root_keyword: "REQUIRE CERTIFIED",
        category: "meta",
        typed_target: "certified-only".to_string(),
        summary: vec![format!("truth={truth}"), format!("admission={admission}")],
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: "REQUIRE CERTIFIED".to_string(),
                kind: "root".to_string(),
                detail: Some("category=meta".to_string()),
            },
        ],
    })
}

fn parse_meta_command_v1(query: &str) -> Result<MetaCommandAstV1, ReadOnlyLspErrorV1> {
    let document = parse_document_v1(query).map_err(syntax_page_to_lsp_error_v1)?;
    let Some(root) = document.roots.into_iter().next() else {
        return Err(syntax_error_v1("empty_document", "document is empty"));
    };
    match root {
        AstRootV1::Meta(command) => Ok(command),
        _ => Err(syntax_error_v1(
            "unsupported_root_keyword",
            "expected meta root on meta query path",
        )),
    }
}

fn declaration_summary_v1(
    keyword: &'static str,
    name: &str,
    summary: Vec<String>,
) -> ReadOnlyRootSummaryV1 {
    ReadOnlyRootSummaryV1 {
        root_keyword: keyword,
        category: "declaration",
        typed_target: name.to_string(),
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: keyword.to_string(),
                kind: "root".to_string(),
                detail: Some("category=declaration".to_string()),
            },
            SymbolItemV1 {
                name: name.to_string(),
                kind: "name".to_string(),
                detail: Some(keyword.to_ascii_lowercase()),
            },
        ],
        summary,
    }
}

fn split_head_v1(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.find(char::is_whitespace) {
        Some(index) => Some((&trimmed[..index], trimmed[index..].trim_start())),
        None => Some((trimmed, "")),
    }
}

fn split_once_case_insensitive_v1<'a>(input: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let haystack_upper = input.to_ascii_uppercase();
    let needle_upper = needle.to_ascii_uppercase();
    let index = haystack_upper.find(&needle_upper)?;
    let before = &input[..index];
    let after = &input[index + needle.len()..];
    Some((before, after))
}

fn split_select_table_and_remainder_v1(input: &str) -> (&str, &str) {
    let where_index = input.to_ascii_uppercase().find(" WHERE ");
    let limit_index = input.to_ascii_uppercase().find(" LIMIT ");
    match (where_index, limit_index) {
        (Some(where_index), Some(limit_index)) => {
            let cut = where_index.min(limit_index);
            (&input[..cut], input[cut..].trim())
        }
        (Some(where_index), None) => (&input[..where_index], input[where_index..].trim()),
        (None, Some(limit_index)) => (&input[..limit_index], input[limit_index..].trim()),
        (None, None) => (input, ""),
    }
}

fn split_select_remainder_v1(
    remainder: &str,
) -> Result<(Option<String>, Option<u64>), ReadOnlyLspErrorV1> {
    let upper = remainder.to_ascii_uppercase();
    let where_index = upper.find("WHERE ");
    let limit_index = upper.find("LIMIT ");

    let where_clause = where_index.map(|index| {
        let start = index + "WHERE ".len();
        let end = limit_index.unwrap_or(remainder.len());
        remainder[start..end].trim().to_string()
    });
    let limit_clause = if let Some(index) = limit_index {
        let raw_limit = remainder[index + "LIMIT ".len()..].trim();
        Some(parse_positive_u64_v1(raw_limit, "limit")?)
    } else {
        None
    };

    if let Some(where_clause) = where_clause.as_ref() {
        validate_select_where_clause_v1(where_clause)?;
    }

    Ok((where_clause, limit_clause))
}

fn validate_select_where_clause_v1(where_clause: &str) -> Result<(), ReadOnlyLspErrorV1> {
    let operators = ["!=", "<=", ">=", "=", "<", ">"];
    let Some(operator) = operators
        .into_iter()
        .find(|operator| where_clause.contains(operator))
    else {
        return Err(syntax_error_v1(
            "invalid_where_clause",
            "SELECT WHERE clause must contain a supported operator",
        ));
    };
    let Some((field, value)) = where_clause.split_once(operator) else {
        return Err(syntax_error_v1(
            "invalid_where_clause",
            "SELECT WHERE clause could not be decoded",
        ));
    };
    if field.trim().is_empty() || value.trim().is_empty() {
        return Err(syntax_error_v1(
            "invalid_where_clause",
            "SELECT WHERE clause must include both field and value",
        ));
    }
    Ok(())
}

fn parse_projection_fields_v1(raw: &str) -> Result<Vec<String>, ReadOnlyLspErrorV1> {
    if raw == "*" {
        return Ok(vec!["*".to_string()]);
    }

    let mut fields = Vec::new();
    for field in raw.split(',') {
        let field = field.trim();
        if field.is_empty() || field.split('.').any(|segment| segment.trim().is_empty()) {
            return Err(syntax_error_v1(
                "invalid_projection_shape",
                format!("invalid SELECT projection field `{field}`"),
            ));
        }
        fields.push(field.to_string());
    }
    if fields.is_empty() {
        return Err(syntax_error_v1(
            "invalid_projection_shape",
            "SELECT projection cannot be empty",
        ));
    }
    Ok(fields)
}

fn parse_named_payload_v1(
    input: &str,
    root_keyword: &'static str,
) -> Result<(String, Vec<PayloadFieldV1>), ReadOnlyLspErrorV1> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(syntax_error_v1(
            "missing_named_target",
            format!("{root_keyword} root requires a target name"),
        ));
    }
    let (subject, fields) = split_subject_and_fields_v1(trimmed);
    let name = subject.trim();
    if name.is_empty() {
        return Err(syntax_error_v1(
            "missing_named_target",
            format!("{root_keyword} root requires a target name"),
        ));
    }
    Ok((name.to_string(), fields))
}

fn split_subject_and_fields_v1(input: &str) -> (&str, Vec<PayloadFieldV1>) {
    let mut parts = input.split(";;");
    let subject = parts.next().unwrap_or_default();
    let fields = parts
        .filter_map(|chunk| parse_payload_field_v1(chunk).ok())
        .collect::<Vec<_>>();
    (subject, fields)
}

fn parse_payload_fields_v1(input: &str) -> Result<Vec<PayloadFieldV1>, ReadOnlyLspErrorV1> {
    let mut fields = Vec::new();
    for chunk in input.split(";;") {
        let field = parse_payload_field_v1(chunk)?;
        fields.push(field);
    }
    Ok(fields)
}

fn parse_payload_field_v1(input: &str) -> Result<PayloadFieldV1, ReadOnlyLspErrorV1> {
    let chunk = input.trim();
    if chunk.is_empty() {
        return Err(syntax_error_v1(
            "empty_payload_clause",
            "payload clause cannot be empty",
        ));
    }
    let Some((key, value)) = chunk.split_once('=') else {
        return Err(syntax_error_v1(
            "invalid_payload_clause",
            format!("payload clause `{chunk}` must be `key=value`"),
        ));
    };
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return Err(syntax_error_v1(
            "invalid_payload_clause",
            format!("payload clause `{chunk}` must include non-empty key and value"),
        ));
    }
    Ok(PayloadFieldV1 {
        key: key.to_string(),
        value: value.to_string(),
    })
}

fn required_field_v1(
    fields: &[PayloadFieldV1],
    key: &str,
    class: DiagnosticClassV1,
) -> Result<String, ReadOnlyLspErrorV1> {
    optional_unique_field_v1(fields, key, class)?.ok_or_else(|| match class {
        DiagnosticClassV1::Syntax => syntax_error_v1(
            "missing_required_clause",
            format!("missing required `{key}` clause"),
        ),
        DiagnosticClassV1::Sema => sema_error_v1(
            "missing_required_clause",
            format!("missing required `{key}` clause"),
        ),
    })
}

fn optional_unique_field_v1(
    fields: &[PayloadFieldV1],
    key: &str,
    class: DiagnosticClassV1,
) -> Result<Option<String>, ReadOnlyLspErrorV1> {
    let mut matches = fields
        .iter()
        .filter(|field| normalize_token_v1(&field.key) == normalize_token_v1(key))
        .map(|field| field.value.clone());

    let first = matches.next();
    if matches.next().is_some() {
        let message = format!("`{key}` clause is duplicated");
        return Err(match class {
            DiagnosticClassV1::Syntax => syntax_error_v1("duplicated_clause", message),
            DiagnosticClassV1::Sema => sema_error_v1("duplicated_clause", message),
        });
    }
    Ok(first)
}

fn parse_positive_u32_v1(raw: &str, field: &str) -> Result<u32, ReadOnlyLspErrorV1> {
    let parsed = raw.parse::<u32>().map_err(|_| {
        syntax_error_v1(
            "invalid_numeric_clause",
            format!("`{field}` must be a positive u32, got `{raw}`"),
        )
    })?;
    if parsed == 0 {
        return Err(sema_error_v1(
            "invalid_numeric_clause",
            format!("`{field}` must be greater than zero"),
        ));
    }
    Ok(parsed)
}

fn parse_positive_u64_v1(raw: &str, field: &str) -> Result<u64, ReadOnlyLspErrorV1> {
    let parsed = raw.parse::<u64>().map_err(|_| {
        syntax_error_v1(
            "invalid_numeric_clause",
            format!("`{field}` must be a positive u64, got `{raw}`"),
        )
    })?;
    if parsed == 0 {
        return Err(sema_error_v1(
            "invalid_numeric_clause",
            format!("`{field}` must be greater than zero"),
        ));
    }
    Ok(parsed)
}

fn syntax_page_to_lsp_error_v1(page: SyntaxDiagnosticPageV1) -> ReadOnlyLspErrorV1 {
    let Some(diagnostic) = page.diagnostics.into_iter().next() else {
        return syntax_error_v1(
            "unknown_syntax_error",
            "syntax rejected input without diagnostics",
        );
    };
    ReadOnlyLspErrorV1 {
        class: diagnostic_class_for_syntax_code_v1(diagnostic.code),
        code: diagnostic.code,
        message: diagnostic.message,
    }
}

fn meta_shape_error_to_lsp_v1(error: MetaCommandShapeErrorV1) -> ReadOnlyLspErrorV1 {
    ReadOnlyLspErrorV1 {
        class: diagnostic_class_for_syntax_code_v1(error.code),
        code: error.code,
        message: error.message,
    }
}

fn diagnostic_class_for_syntax_code_v1(code: &'static str) -> DiagnosticClassV1 {
    match code {
        "use_pack_invalid_activation" => DiagnosticClassV1::Sema,
        _ => DiagnosticClassV1::Syntax,
    }
}

fn pack_activation_mode_summary_v1(mode: PackActivationModeAstV1) -> &'static str {
    match mode {
        PackActivationModeAstV1::Session => "session",
        PackActivationModeAstV1::Workspace => "workspace",
    }
}

fn certified_truth_summary_v1(value: CertifiedTruthRequirementAstV1) -> &'static str {
    match value {
        CertifiedTruthRequirementAstV1::CertifiedOnly => "certified-only",
    }
}

fn certified_admission_summary_v1(value: CertifiedAdmissionPolicyAstV1) -> &'static str {
    match value {
        CertifiedAdmissionPolicyAstV1::FailClosed => "fail-closed",
    }
}

fn normalize_token_v1(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "")
}

fn syntax_error_v1(code: &'static str, message: impl Into<String>) -> ReadOnlyLspErrorV1 {
    ReadOnlyLspErrorV1 {
        class: DiagnosticClassV1::Syntax,
        code,
        message: message.into(),
    }
}

fn sema_error_v1(code: &'static str, message: impl Into<String>) -> ReadOnlyLspErrorV1 {
    ReadOnlyLspErrorV1 {
        class: DiagnosticClassV1::Sema,
        code,
        message: message.into(),
    }
}
use quanta_gqlang_v2_ir::frontend::{
    parse_document_v1, AstRootV1, CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1,
    MetaCommandAstV1, MetaCommandShapeErrorV1, MetaCommandShapeV1, PackActivationModeAstV1,
    SyntaxDiagnosticPageV1,
};
