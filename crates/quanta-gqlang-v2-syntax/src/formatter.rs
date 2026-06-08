use crate::ast::{AstRootV1, DocumentAstV1};
use crate::declarations::pack::{
    GateCheckBindingModeV1, GateDeclShapeAstV1, PackActivationModeV1, PackDeclShapeAstV1,
    ProfileAdmissionPolicyV1, ProfileDeclShapeAstV1,
};
use crate::declarations::view::{
    ModelDeclShapeAstV1, RuleDeclShapeAstV1, ViewDeclShapeAstV1, ViewQueryShapeAstV1,
};
use crate::interactive::meta::{
    CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, LetBindingAstV1,
    MetaCommandAstV1, MetaCommandShapeV1, WithClauseAstV1,
};
use crate::interactive::sample::{
    CompareIntentAstV1, CompareMetricV1, ExportIntentAstV1, ExportKindV1, InteractiveRefAstV1,
    SampleRetrievalIntentAstV1, SampleRetrievalModeV1,
};
use crate::interactive::show::{DescribeTargetAstV1, DescribeTargetKindAstV1};
use crate::roots::check::CheckRootIntentV1;
use crate::roots::explain::{ExplainEvidenceModeV1, ExplainTruthExpectationV1};
use crate::roots::find::{FindSubkindV1, ParsedFindRootSeedV1};
use crate::roots::select::{
    SelectFilterAstV1, SelectFilterOpV1, SelectLiteralAstV1, SelectProjectionAstV1,
    SelectQueryShapeV1,
};

pub fn format_document_v1(doc: &DocumentAstV1) -> String {
    let mut lines = Vec::new();
    if let Some(with_clause) = &doc.with_clause {
        lines.push(format_with_clause_v1(with_clause));
    }
    for root in &doc.roots {
        lines.push(format_root_v1(root));
    }
    lines.join("\n")
}

fn format_with_clause_v1(with_clause: &WithClauseAstV1) -> String {
    if with_clause.bindings.is_empty() {
        return "WITH".to_string();
    }
    let bindings = with_clause
        .bindings
        .iter()
        .map(format_let_binding_v1)
        .collect::<Vec<_>>()
        .join(", ");
    format!("WITH {bindings}")
}

fn format_let_binding_v1(binding: &LetBindingAstV1) -> String {
    format!("LET {} = {}", binding.name.trim(), binding.value.trim())
}

fn format_root_v1(root: &AstRootV1) -> String {
    match root {
        AstRootV1::Select(spec) => spec
            .shape_v1()
            .map(format_select_shape_v1)
            .unwrap_or_else(|_| format!("SELECT {}", spec.raw_query_v1().trim())),
        AstRootV1::Find(spec) => spec
            .parse_seed()
            .map(|seed| format_find_root_v1(spec.subkind, &seed))
            .unwrap_or_else(|_| format!("FIND {} {}", spec.subkind.as_str(), spec.seed.trim())),
        AstRootV1::Check(spec) => spec
            .intent()
            .map(format_check_intent_v1)
            .unwrap_or_else(|_| format!("CHECK {} {}", spec.rule_id.trim(), spec.scope.trim())),
        AstRootV1::Explain(spec) => spec
            .shape()
            .map(format_explain_shape_v1)
            .unwrap_or_else(|_| format!("EXPLAIN {}", spec.target.as_deref().unwrap_or("").trim())),
        AstRootV1::Show(spec) => spec
            .category_spec()
            .map(|category| format!("SHOW {}", category.canonical_name_v1()))
            .unwrap_or_else(|_| format!("SHOW {}", spec.category.trim())),
        AstRootV1::Describe(spec) => spec
            .target_spec()
            .map(|target| format!("DESCRIBE {}", format_describe_target_v1(&target)))
            .unwrap_or_else(|_| format!("DESCRIBE {}", spec.target.trim())),
        AstRootV1::Sample(spec) => spec
            .intent()
            .map(|intent| format!("SAMPLE {}", format_sample_intent_v1(&intent)))
            .unwrap_or_else(|_| format!("SAMPLE {}", spec.query_text.trim())),
        AstRootV1::Compare(spec) => spec
            .intent()
            .map(|intent| format!("COMPARE {}", format_compare_intent_v1(&intent)))
            .unwrap_or_else(|_| format!("COMPARE {}", spec.baseline_ref.trim())),
        AstRootV1::Export(spec) => spec
            .intent()
            .map(|intent| format!("EXPORT {}", format_export_intent_v1(&intent)))
            .unwrap_or_else(|_| format!("EXPORT {}", spec.source_ref.trim())),
        AstRootV1::View(spec) => spec
            .shape_v1()
            .map(|shape| format!("VIEW {}", format_view_decl_v1(&shape)))
            .unwrap_or_else(|_| format!("VIEW {}", spec.name.trim())),
        AstRootV1::Rule(spec) => spec
            .shape_v1()
            .map(|shape| format!("RULE {}", format_rule_decl_v1(&shape)))
            .unwrap_or_else(|_| format!("RULE {}", spec.name.trim())),
        AstRootV1::Model(spec) => spec
            .shape_v1()
            .map(|shape| format!("MODEL {}", format_model_decl_v1(&shape)))
            .unwrap_or_else(|_| format!("MODEL {}", spec.name.trim())),
        AstRootV1::Pack(spec) => spec
            .shape()
            .map(|shape| format!("PACK {}", format_pack_decl_v1(&shape)))
            .unwrap_or_else(|_| format!("PACK {}", spec.name.trim())),
        AstRootV1::Profile(spec) => spec
            .shape()
            .map(|shape| format!("PROFILE {}", format_profile_decl_v1(&shape)))
            .unwrap_or_else(|_| format!("PROFILE {}", spec.name.trim())),
        AstRootV1::Gate(spec) => spec
            .shape()
            .map(|shape| format!("GATE {}", format_gate_decl_v1(&shape)))
            .unwrap_or_else(|_| format!("GATE {}", spec.name.trim())),
        AstRootV1::Meta(command) => format_meta_command_v1(command),
    }
}

fn format_select_shape_v1(shape: SelectQueryShapeV1) -> String {
    let projection = match shape.projection {
        SelectProjectionAstV1::Wildcard => "*".to_string(),
        SelectProjectionAstV1::Fields(fields) => fields
            .into_iter()
            .map(|field| field.dotted_name_v1())
            .collect::<Vec<_>>()
            .join(", "),
    };
    let mut line = format!("SELECT {projection} FROM {}", shape.table.dotted_name_v1());
    if let Some(filter) = shape.filter {
        line.push_str(" WHERE ");
        line.push_str(&format_select_filter_v1(&filter));
    }
    if let Some(limit) = shape.limit {
        line.push_str(" LIMIT ");
        line.push_str(&limit.to_string());
    }
    line
}

fn format_select_filter_v1(filter: &SelectFilterAstV1) -> String {
    format!(
        "{} {} {}",
        filter.field.dotted_name_v1(),
        format_select_filter_op_v1(&filter.op),
        format_select_literal_v1(&filter.value)
    )
}

fn format_select_filter_op_v1(op: &SelectFilterOpV1) -> &'static str {
    match op {
        SelectFilterOpV1::Eq => "=",
        SelectFilterOpV1::NotEq => "!=",
        SelectFilterOpV1::Lt => "<",
        SelectFilterOpV1::Lte => "<=",
        SelectFilterOpV1::Gt => ">",
        SelectFilterOpV1::Gte => ">=",
    }
}

fn format_select_literal_v1(value: &SelectLiteralAstV1) -> String {
    match value {
        SelectLiteralAstV1::String(inner) => format!("'{}'", inner.replace('\'', "''")),
        SelectLiteralAstV1::Integer(inner) => inner.to_string(),
        SelectLiteralAstV1::Boolean(inner) => inner.to_string(),
    }
}

fn format_find_root_v1(subkind: FindSubkindV1, seed: &ParsedFindRootSeedV1) -> String {
    format!("FIND {} {}", subkind.as_str(), format_find_seed_v1(seed))
}

fn format_find_seed_v1(seed: &ParsedFindRootSeedV1) -> String {
    match seed {
        ParsedFindRootSeedV1::Reachability(spec) => format_binary_seed_v1(
            &spec.source,
            "->",
            &spec.sink,
            &[
                directive_csv_v1("via", &spec.via),
                directive_opt_v1("depth", spec.depth.as_deref()),
            ],
        ),
        ParsedFindRootSeedV1::Flow(spec) => format_binary_seed_v1(
            &spec.source,
            "->",
            &spec.sink,
            &[
                directive_csv_v1("through", &spec.through),
                directive_opt_v1("window", spec.window.as_deref().map(format_flow_window_v1)),
            ],
        ),
        ParsedFindRootSeedV1::Slice(spec) => format_unary_seed_v1(
            &spec.anchor,
            &[
                directive_opt_v1(
                    "direction",
                    spec.direction.as_deref().map(format_slice_direction_v1),
                ),
                directive_opt_v1("frontier", spec.frontier.as_deref()),
            ],
        ),
        ParsedFindRootSeedV1::Taint(spec) => format_binary_seed_v1(
            &spec.source,
            "->",
            &spec.sink,
            &[
                directive_csv_v1("sanitizers", &spec.sanitizers),
                directive_opt_v1("depth", spec.depth.as_deref()),
            ],
        ),
        ParsedFindRootSeedV1::Path(spec) => format_binary_seed_v1(
            &spec.source,
            "->",
            &spec.sink,
            &[
                directive_csv_v1("through", &spec.through),
                directive_opt_v1("depth", spec.depth.as_deref()),
            ],
        ),
        ParsedFindRootSeedV1::PointsTo(spec) => format_binary_seed_v1(
            &spec.reference,
            "=>",
            &spec.pointee,
            &[directive_opt_v1("scope", spec.scope.as_deref())],
        ),
        ParsedFindRootSeedV1::Summary(spec) => format_unary_seed_v1(
            &spec.callable,
            &[
                directive_csv_v1("include", &spec.include),
                directive_opt_v1("depth", spec.depth.as_deref()),
            ],
        ),
    }
}

fn format_binary_seed_v1(
    left: &str,
    operator: &str,
    right: &str,
    directives: &[Option<String>],
) -> String {
    let mut seed = format!("{left} {operator} {right}");
    for directive in directives.iter().flatten() {
        seed.push_str(" | ");
        seed.push_str(directive);
    }
    seed
}

fn format_unary_seed_v1(anchor: &str, directives: &[Option<String>]) -> String {
    let mut seed = anchor.to_string();
    for directive in directives.iter().flatten() {
        seed.push_str(" | ");
        seed.push_str(directive);
    }
    seed
}

fn directive_opt_v1(key: &str, value: Option<&str>) -> Option<String> {
    value.map(|value| format!("{key}={value}"))
}

fn directive_csv_v1(key: &str, values: &[String]) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(format!("{key}={}", values.join(",")))
    }
}

fn format_flow_window_v1(value: &str) -> &str {
    value
}

fn format_slice_direction_v1(value: &str) -> &str {
    value
}

fn format_check_intent_v1(intent: CheckRootIntentV1) -> String {
    format!("CHECK {} {}", intent.rule_id, intent.encode_scope_payload())
}

fn format_explain_shape_v1(shape: crate::roots::explain::ExplainShapeAstV1) -> String {
    format!(
        "EXPLAIN {} ;; evidence={} ;; truth={}",
        shape.target,
        format_explain_evidence_v1(shape.evidence),
        format_explain_truth_v1(shape.truth_expectation)
    )
}

fn format_explain_evidence_v1(value: ExplainEvidenceModeV1) -> &'static str {
    match value {
        ExplainEvidenceModeV1::Auto => "auto",
        ExplainEvidenceModeV1::Trace => "trace",
        ExplainEvidenceModeV1::Citation => "citation",
        ExplainEvidenceModeV1::Counterexample => "counterexample",
    }
}

fn format_explain_truth_v1(value: ExplainTruthExpectationV1) -> &'static str {
    match value {
        ExplainTruthExpectationV1::Inspect => "inspect",
        ExplainTruthExpectationV1::Confirm => "confirm",
        ExplainTruthExpectationV1::Refute => "refute",
    }
}

fn format_describe_target_v1(target: &DescribeTargetAstV1) -> String {
    format!(
        "{}::{}",
        format_describe_target_kind_v1(target.kind),
        target.path.join(".")
    )
}

fn format_describe_target_kind_v1(kind: DescribeTargetKindAstV1) -> &'static str {
    match kind {
        DescribeTargetKindAstV1::Capability => "capability",
        DescribeTargetKindAstV1::Model => "model",
        DescribeTargetKindAstV1::View => "view",
        DescribeTargetKindAstV1::Rule => "rule",
        DescribeTargetKindAstV1::Pack => "pack",
        DescribeTargetKindAstV1::Profile => "profile",
        DescribeTargetKindAstV1::Gate => "gate",
    }
}

fn format_sample_intent_v1(intent: &SampleRetrievalIntentAstV1) -> String {
    let mut clauses = vec![
        format!("query={}", intent.query),
        format!("mode={}", format_sample_mode_v1(intent.mode)),
        format!("limit={}", intent.limit),
    ];
    if let Some(scope) = &intent.scope {
        clauses.push(format!("scope={}", format_interactive_ref_v1(scope)));
    }
    clauses.join(" ;; ")
}

fn format_sample_mode_v1(mode: SampleRetrievalModeV1) -> &'static str {
    match mode {
        SampleRetrievalModeV1::Lexical => "lexical",
        SampleRetrievalModeV1::Semantic => "semantic",
        SampleRetrievalModeV1::Hybrid => "hybrid",
    }
}

fn format_compare_intent_v1(intent: &CompareIntentAstV1) -> String {
    [
        format!("baseline={}", format_interactive_ref_v1(&intent.baseline)),
        format!("candidate={}", format_interactive_ref_v1(&intent.candidate)),
        format!("metric={}", format_compare_metric_v1(intent.metric)),
    ]
    .join(" ;; ")
}

fn format_compare_metric_v1(metric: CompareMetricV1) -> &'static str {
    match metric {
        CompareMetricV1::Exact => "exact",
        CompareMetricV1::Structural => "structural",
        CompareMetricV1::Semantic => "semantic",
    }
}

fn format_export_intent_v1(intent: &ExportIntentAstV1) -> String {
    let mut clauses = vec![
        format!("source={}", format_interactive_ref_v1(&intent.source)),
        format!("kind={}", format_export_kind_v1(intent.kind)),
    ];
    if let Some(profile) = &intent.profile {
        clauses.push(format!("profile={profile}"));
    }
    clauses.join(" ;; ")
}

fn format_export_kind_v1(kind: ExportKindV1) -> &'static str {
    match kind {
        ExportKindV1::Json => "json",
        ExportKindV1::Csv => "csv",
        ExportKindV1::Sarif => "sarif",
        ExportKindV1::Markdown => "markdown",
        ExportKindV1::GraphMl => "graph-ml",
    }
}

fn format_interactive_ref_v1(reference: &InteractiveRefAstV1) -> String {
    format!("{}::{}", reference.kind, reference.locator)
}

fn format_view_decl_v1(shape: &ViewDeclShapeAstV1) -> String {
    match &shape.query {
        ViewQueryShapeAstV1::QueryRef(query_ref) => {
            format!("{} ;; query-ref={query_ref}", shape.name)
        }
        ViewQueryShapeAstV1::InlineSelect(select) => {
            format!("{} ;; body={}", shape.name, select.raw_query_v1())
        }
    }
}

fn format_rule_decl_v1(shape: &RuleDeclShapeAstV1) -> String {
    format!("{} ;; check-ref={}", shape.name, shape.linkage.check_ref)
}

fn format_model_decl_v1(shape: &ModelDeclShapeAstV1) -> String {
    format!(
        "{} ;; provider={} ;; source={}",
        shape.name, shape.binding.provider, shape.binding.source
    )
}

fn format_pack_decl_v1(shape: &PackDeclShapeAstV1) -> String {
    format!(
        "{} ;; source={} ;; activation={}",
        shape.name,
        shape.source,
        format_pack_activation_v1(shape.activation)
    )
}

fn format_pack_activation_v1(value: PackActivationModeV1) -> &'static str {
    match value {
        PackActivationModeV1::Manual => "manual",
        PackActivationModeV1::Session => "session",
        PackActivationModeV1::Workspace => "workspace",
    }
}

fn format_profile_decl_v1(shape: &ProfileDeclShapeAstV1) -> String {
    format!(
        "{} ;; capabilities={} ;; admission={}",
        shape.name,
        shape.capabilities.join(","),
        format_profile_admission_v1(shape.admission)
    )
}

fn format_profile_admission_v1(value: ProfileAdmissionPolicyV1) -> &'static str {
    match value {
        ProfileAdmissionPolicyV1::Allowlist => "allowlist",
        ProfileAdmissionPolicyV1::ReviewRequired => "review-required",
        ProfileAdmissionPolicyV1::CertifiedOnly => "certified-only",
    }
}

fn format_gate_decl_v1(shape: &GateDeclShapeAstV1) -> String {
    format!(
        "{} ;; checks={} ;; binding={}",
        shape.name,
        shape.checks.join(","),
        format_gate_binding_v1(shape.binding)
    )
}

fn format_gate_binding_v1(value: GateCheckBindingModeV1) -> &'static str {
    match value {
        GateCheckBindingModeV1::All => "all",
        GateCheckBindingModeV1::Any => "any",
    }
}

fn format_meta_command_v1(command: &MetaCommandAstV1) -> String {
    match command.shape() {
        Ok(MetaCommandShapeV1::UsePack(spec)) => {
            let mut line = format!("USE PACK {}", spec.pack_id);
            line.push_str(" ;; activation=");
            line.push_str(format_pack_activation_meta_v1(spec.activation_mode));
            line.push_str(" ;; lockfile=");
            line.push_str(spec.lockfile_path);
            line
        }
        Ok(MetaCommandShapeV1::RequireCertified(spec)) => {
            let _ = (
                format_certified_truth_v1(spec.truth_requirement),
                format_certified_admission_v1(spec.admission_policy),
            );
            "REQUIRE CERTIFIED".to_string()
        }
        Err(_) => match command {
            MetaCommandAstV1::UsePack {
                pack_id,
                activation_mode,
                lockfile_path,
            } => {
                let mut line = format!("USE PACK {}", pack_id.trim());
                if let Some(mode) = activation_mode
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    line.push_str(" ;; activation=");
                    line.push_str(mode);
                }
                if let Some(lockfile) = lockfile_path
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    line.push_str(" ;; lockfile=");
                    line.push_str(lockfile);
                }
                line
            }
            MetaCommandAstV1::RequireCertified => "REQUIRE CERTIFIED".to_string(),
        },
    }
}

fn format_pack_activation_meta_v1(
    value: crate::interactive::meta::PackActivationModeAstV1,
) -> &'static str {
    match value {
        crate::interactive::meta::PackActivationModeAstV1::Session => "session",
        crate::interactive::meta::PackActivationModeAstV1::Workspace => "workspace",
    }
}

fn format_certified_truth_v1(value: CertifiedTruthRequirementAstV1) -> &'static str {
    match value {
        CertifiedTruthRequirementAstV1::CertifiedOnly => "certified-only",
    }
}

fn format_certified_admission_v1(value: CertifiedAdmissionPolicyAstV1) -> &'static str {
    match value {
        CertifiedAdmissionPolicyAstV1::FailClosed => "fail-closed",
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "formatter tests assert parse success for validated round-trip fixtures"
    )]

    use super::format_document_v1;
    use crate::parse_document_v1;

    fn assert_idempotent_v1(query: &str) {
        let doc = parse_document_v1(query).expect("parse");
        let formatted = format_document_v1(&doc);
        let reparsed = parse_document_v1(&formatted).expect("reparse");
        let reformatted = format_document_v1(&reparsed);
        assert_eq!(formatted, reformatted);
    }

    #[test]
    fn formats_core_and_interactive_roots_idempotently() {
        for query in [
            "SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 4",
            "FIND taint api.entry -> db.write | sanitizers=authz.clean | depth=3",
            "CHECK rule.hybrid check:v1|class=security|scope=repo%3A%2F%2Fruntime|pattern=call%20%3D%20%22std%3A%3Aprocess%3A%3ACommand%22|confirm=require",
            "EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm",
            "SHOW capabilities",
            "DESCRIBE rule::owasp.sql-injection",
            "SAMPLE query=callgraph entrypoints ;; scope=repo::frontend ;; mode=semantic ;; limit=5",
            "COMPARE baseline=run::baseline-1 ;; candidate=run::candidate-2 ;; metric=semantic",
            "EXPORT source=result::latest ;; kind=sarif ;; profile=strict",
        ] {
            assert_idempotent_v1(query);
        }
    }

    #[test]
    fn formats_declarations_and_meta_roots_idempotently() {
        for query in [
            "VIEW catalog.users ;; query-ref=queries/catalog/users",
            "RULE owasp.sql ;; check-ref=checks/owasp/sql",
            "MODEL github.repo ;; provider=github ;; source=repositories/default",
            "PACK secure-pack ;; source=registry::security ;; activation=session",
            "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
            "GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=any",
            "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
            "REQUIRE CERTIFIED",
        ] {
            assert_idempotent_v1(query);
        }
    }
}
