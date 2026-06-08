use super::{ReadOnlyRootSummaryV1, SymbolItemV1};
use quanta_gqlang_v2_ir::frontend::{
    AstRootV1, CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, CompareRootAstV1,
    DescribeRootAstV1, ExportRootAstV1, FindRootAstV1, GateDeclAstV1, MetaCommandAstV1,
    MetaCommandShapeV1, ModelDeclAstV1, PackActivationModeAstV1, PackDeclAstV1, ProfileDeclAstV1,
    RuleDeclAstV1, SampleRootAstV1, SelectFieldRefAstV1, SelectProjectionAstV1, SelectRootAstV1,
    ShowRootAstV1, ViewDeclAstV1,
};

/// Build the read-only presentation from a canonically-validated typed root. This
/// is pure formatting — every accept/reject decision was already made by the
/// parser — so it can never diverge from the grammar.
pub(super) fn summary_from_root_v1(root: &AstRootV1) -> ReadOnlyRootSummaryV1 {
    match root {
        AstRootV1::Select(spec) => select_summary_v1(spec),
        AstRootV1::Find(FindRootAstV1 { subkind, seed }) => root_summary_v1(
            "FIND",
            "query",
            format!("find::{}", subkind.as_str()),
            vec![format!("seed={}", seed.trim())],
        ),
        AstRootV1::Check(spec) => {
            let summary = match spec.intent() {
                Ok(intent) => vec![
                    format!("class={}", intent.class),
                    format!("scope={:?}", intent.scope),
                    format!("pattern={:?}", intent.pattern),
                    format!("confirm={:?}", intent.confirm),
                ],
                Err(_) => vec![format!("scope={}", spec.scope.trim())],
            };
            root_summary_v1("CHECK", "query", spec.rule_id.clone(), summary)
        }
        AstRootV1::Explain(spec) => {
            let (typed_target, summary) = match spec.shape() {
                Ok(shape) => (
                    shape.target.clone(),
                    vec![
                        format!("evidence={:?}", shape.evidence),
                        format!("truth={:?}", shape.truth_expectation),
                    ],
                ),
                Err(_) => (spec.target.clone().unwrap_or_default(), Vec::new()),
            };
            root_summary_v1("EXPLAIN", "query", typed_target, summary)
        }
        AstRootV1::Show(ShowRootAstV1 { category }) => root_summary_v1(
            "SHOW",
            "interactive",
            category.clone(),
            vec![format!("catalog_category={category}")],
        ),
        AstRootV1::Describe(DescribeRootAstV1 { target }) => root_summary_v1(
            "DESCRIBE",
            "interactive",
            target.clone(),
            vec![format!("target={target}")],
        ),
        AstRootV1::Sample(SampleRootAstV1 { query_text }) => root_summary_v1(
            "SAMPLE",
            "interactive",
            query_text.clone(),
            vec![format!("payload={query_text}")],
        ),
        AstRootV1::Compare(CompareRootAstV1 { baseline_ref }) => root_summary_v1(
            "COMPARE",
            "interactive",
            baseline_ref.clone(),
            vec![format!("payload={baseline_ref}")],
        ),
        AstRootV1::Export(ExportRootAstV1 { source_ref }) => root_summary_v1(
            "EXPORT",
            "interactive",
            source_ref.clone(),
            vec![format!("payload={source_ref}")],
        ),
        AstRootV1::View(ViewDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("VIEW", name)
        }
        AstRootV1::Rule(RuleDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("RULE", name)
        }
        AstRootV1::Model(ModelDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("MODEL", name)
        }
        AstRootV1::Pack(PackDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("PACK", name)
        }
        AstRootV1::Profile(ProfileDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("PROFILE", name)
        }
        AstRootV1::Gate(GateDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("GATE", name)
        }
        AstRootV1::Meta(command) => meta_summary_v1(command),
    }
}

fn select_summary_v1(spec: &SelectRootAstV1) -> ReadOnlyRootSummaryV1 {
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

fn meta_summary_v1(command: &MetaCommandAstV1) -> ReadOnlyRootSummaryV1 {
    // The command already parsed, so `shape()` succeeds; fall back defensively.
    match command.shape() {
        Ok(MetaCommandShapeV1::UsePack(spec)) => {
            let activation = pack_activation_mode_summary_v1(spec.activation_mode);
            ReadOnlyRootSummaryV1 {
                root_keyword: "USE PACK",
                category: "meta",
                typed_target: spec.pack_id.to_string(),
                summary: vec![
                    format!("activation={activation}"),
                    format!("lockfile={}", spec.lockfile_path),
                ],
                symbols: vec![
                    document_symbol_v1(),
                    root_symbol_v1("USE PACK", "meta"),
                    SymbolItemV1 {
                        name: spec.pack_id.to_string(),
                        kind: "pack".to_string(),
                        detail: Some(format!(
                            "activation={activation} lockfile={}",
                            spec.lockfile_path
                        )),
                    },
                ],
            }
        }
        Ok(MetaCommandShapeV1::RequireCertified(spec)) => {
            let truth = certified_truth_summary_v1(spec.truth_requirement);
            let admission = certified_admission_summary_v1(spec.admission_policy);
            ReadOnlyRootSummaryV1 {
                root_keyword: "REQUIRE CERTIFIED",
                category: "meta",
                typed_target: "certified-only".to_string(),
                summary: vec![format!("truth={truth}"), format!("admission={admission}")],
                symbols: vec![
                    document_symbol_v1(),
                    root_symbol_v1("REQUIRE CERTIFIED", "meta"),
                ],
            }
        }
        Err(_) => root_summary_v1("USE PACK", "meta", String::new(), Vec::new()),
    }
}

/// A generic single-target root card (document symbol + root symbol + target).
fn root_summary_v1(
    root_keyword: &'static str,
    category: &'static str,
    typed_target: String,
    summary: Vec<String>,
) -> ReadOnlyRootSummaryV1 {
    let symbols = vec![
        document_symbol_v1(),
        root_symbol_v1(root_keyword, category),
        SymbolItemV1 {
            name: typed_target.clone(),
            kind: "target".to_string(),
            detail: Some(format!("{}.target", category)),
        },
    ];
    ReadOnlyRootSummaryV1 {
        root_keyword,
        category,
        typed_target,
        summary,
        symbols,
    }
}

fn declaration_summary_from_carrier_v1(
    keyword: &'static str,
    carrier: &str,
) -> ReadOnlyRootSummaryV1 {
    // The carrier already parsed, so its clause structure is valid; split it only to
    // present the declared name and clauses (display formatting, not validation).
    let name = carrier.split(";;").next().unwrap_or(carrier).trim();
    let clauses = carrier
        .split(";;")
        .skip(1)
        .map(|clause| clause.trim().to_string())
        .filter(|clause| !clause.is_empty())
        .collect();
    declaration_summary_v1(keyword, name, clauses)
}

fn document_symbol_v1() -> SymbolItemV1 {
    SymbolItemV1 {
        name: "document".to_string(),
        kind: "document".to_string(),
        detail: Some("roots=1".to_string()),
    }
}

fn root_symbol_v1(keyword: &str, category: &str) -> SymbolItemV1 {
    SymbolItemV1 {
        name: keyword.to_string(),
        kind: "root".to_string(),
        detail: Some(format!("category={category}")),
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
