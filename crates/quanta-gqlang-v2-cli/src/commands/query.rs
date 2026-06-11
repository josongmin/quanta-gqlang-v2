use quanta_gqlang_v2_ir::frontend::{
    AstRootV1, CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, CheckRootAstV1,
    CheckRootIntentV1, CompareRootAstV1, DescribeRootAstV1, DocumentAstV1, ExplainRootAstV1,
    ExplainShapeAstV1, ExportRootAstV1, FindRootAstV1, GateDeclAstV1, MetaCommandAstV1,
    MetaCommandShapeV1, ModelDeclAstV1, PackActivationModeAstV1, PackDeclAstV1, ProfileDeclAstV1,
    RuleDeclAstV1, SampleRootAstV1, SelectRootAstV1, SemaDiagnosticPageV1, ShowRootAstV1,
    SyntaxDiagnosticPageV1, TypedCertifiedAdmissionPolicyV1, TypedCertifiedTruthRequirementV1,
    TypedDocumentV1, TypedMetaCommandV1, TypedPackActivationModeV1, TypedRootV1, ViewDeclAstV1,
};
use quanta_gqlang_v2_ir::root_ir::{
    CertifiedAdmissionPolicyIrV1, CertifiedTruthRequirementIrV1, CompareMetricIrV1,
    DeclarationIrV1, DegradedReasonV1, ExportKindIrV1, GateBindingIrV1, InteractiveIrV1,
    InteractiveRefIrV1, InteractiveTargetIrV1, MetaIrV1, PackActivationIrV1, ProfileAdmissionIrV1,
    PublicIrRootV1, ViewQueryIrV1,
};
use quanta_gqlang_v2_ir::IrLoweringErrorV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuerySummaryV1 {
    pub root_count: usize,
    pub roots: Vec<QueryRootSummaryV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryRootSummaryV1 {
    pub syntax_kind: &'static str,
    pub syntax_summary: String,
    pub typed_kind: &'static str,
    pub typed_summary: String,
    pub ir_kind: &'static str,
    pub ir_summary: String,
    /// Computed once from the typed IR root; the single source of the read-only
    /// shell degraded lane for both the doctor and headless reports.
    pub degraded_reason: Option<DegradedReasonV1>,
    /// E06C public planning metadata for the root, when the root is on the query
    /// plane (Declaration/Meta carry none). This is the CLI's non-test consumer
    /// of `LoweringReadyIrV1` — it surfaces the planned capability + truth lane.
    pub plan: Option<QueryPlanSummaryV1>,
}

/// The query-plane planning summary a root carries — its required capability set
/// and the truth lane the plan commits to (E06C).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryPlanSummaryV1 {
    pub capability: String,
    pub truth: String,
}

impl QuerySummaryV1 {
    pub fn render_v1(&self) -> String {
        let mut lines = vec![format!("query.root_count={}", self.root_count)];
        for (index, root) in self.roots.iter().enumerate() {
            lines.push(format!(
                "query.root[{index}].syntax.kind={}",
                root.syntax_kind
            ));
            lines.push(format!(
                "query.root[{index}].syntax.summary={}",
                root.syntax_summary
            ));
            lines.push(format!(
                "query.root[{index}].typed.kind={}",
                root.typed_kind
            ));
            lines.push(format!(
                "query.root[{index}].typed.summary={}",
                root.typed_summary
            ));
            lines.push(format!("query.root[{index}].ir.kind={}", root.ir_kind));
            lines.push(format!(
                "query.root[{index}].ir.summary={}",
                root.ir_summary
            ));
            if let Some(plan) = &root.plan {
                lines.push(format!(
                    "query.root[{index}].plan.capability={}",
                    plan.capability
                ));
                lines.push(format!("query.root[{index}].plan.truth={}", plan.truth));
            }
        }
        lines.join("\n")
    }
}

pub fn run(input: &str) -> Result<QuerySummaryV1, String> {
    let doc = parse_document_v1(input).map_err(render_syntax_page_v1)?;
    let typed = analyze_document_v1(&doc).map_err(render_sema_page_v1)?;
    let ir_roots = lower_document_to_ir_v1(&typed).map_err(render_ir_error_v1)?;
    Ok(build_query_summary_v1(&doc, &typed, &ir_roots))
}

pub(crate) fn parse_document_v1(input: &str) -> Result<DocumentAstV1, SyntaxDiagnosticPageV1> {
    quanta_gqlang_v2_ir::frontend::parse_document_v1(input)
}

pub(crate) fn analyze_document_v1(
    doc: &DocumentAstV1,
) -> Result<TypedDocumentV1, SemaDiagnosticPageV1> {
    quanta_gqlang_v2_ir::frontend::analyze_document_v1(doc)
}

pub(crate) fn lower_document_to_ir_v1(
    typed: &TypedDocumentV1,
) -> Result<Vec<PublicIrRootV1>, IrLoweringErrorV1> {
    quanta_gqlang_v2_ir::lower_document_to_ir_v1(typed)
}

pub(crate) fn build_query_summary_v1(
    doc: &DocumentAstV1,
    typed: &TypedDocumentV1,
    ir_roots: &[PublicIrRootV1],
) -> QuerySummaryV1 {
    QuerySummaryV1 {
        root_count: doc.roots.len(),
        roots: doc
            .roots
            .iter()
            .zip(typed.roots.iter())
            .zip(ir_roots.iter())
            .map(|((syntax_root, typed_root), ir_root)| QueryRootSummaryV1 {
                syntax_kind: syntax_root_kind_v1(syntax_root),
                syntax_summary: syntax_root_summary_v1(syntax_root),
                typed_kind: typed_root_kind_v1(typed_root),
                typed_summary: typed_root_summary_v1(typed_root),
                ir_kind: ir_root_kind_v1(ir_root),
                ir_summary: ir_root_summary_v1(ir_root),
                degraded_reason: DegradedReasonV1::for_ir_root_v1(ir_root),
                plan: query_plan_summary_v1(ir_root),
            })
            .collect(),
    }
}

/// Consume the `LoweringReadyIrV1` planning producer for `ir_root` and project
/// its plan into the read-only summary. `None` for query-plane-less roots
/// (Declaration/Meta) and for any plan that fails lowering-ready validation —
/// the summary degrades to no plan line rather than fabricating one.
fn query_plan_summary_v1(ir_root: &PublicIrRootV1) -> Option<QueryPlanSummaryV1> {
    let ready =
        quanta_gqlang_v2_ir::planning_producer::lowering_ready_for_root_v1(ir_root.clone()).ok()?;
    let plan = ready.planning.as_ref()?;
    let capability = plan
        .required_capabilities
        .iter()
        .map(|requirement| format!("{:?}", requirement.capability))
        .collect::<Vec<_>>()
        .join(",");
    Some(QueryPlanSummaryV1 {
        capability,
        truth: format!("{:?}", plan.truth_expectation),
    })
}

pub(crate) fn syntax_root_kind_v1(root: &AstRootV1) -> &'static str {
    match root {
        AstRootV1::Select(_) => "select",
        AstRootV1::Find(_) => "find",
        AstRootV1::Check(_) => "check",
        AstRootV1::Explain(_) => "explain",
        AstRootV1::Show(_) => "show",
        AstRootV1::Describe(_) => "describe",
        AstRootV1::Sample(_) => "sample",
        AstRootV1::Compare(_) => "compare",
        AstRootV1::Export(_) => "export",
        AstRootV1::View(_) => "view",
        AstRootV1::Rule(_) => "rule",
        AstRootV1::Model(_) => "model",
        AstRootV1::Pack(_) => "pack",
        AstRootV1::Profile(_) => "profile",
        AstRootV1::Gate(_) => "gate",
        AstRootV1::Meta(_) => "meta",
        AstRootV1::Composition(_) => "composition",
    }
}

pub(crate) fn typed_root_kind_v1(root: &TypedRootV1) -> &'static str {
    match root {
        TypedRootV1::Select(_) => "select",
        TypedRootV1::Find(_) => "find",
        TypedRootV1::Check(_) => "check",
        TypedRootV1::Explain(_) => "explain",
        TypedRootV1::ShowDescribe(_) => "show_describe",
        TypedRootV1::SampleCompareExport(_) => "sample_compare_export",
        TypedRootV1::Declaration(_) => "declaration",
        TypedRootV1::Meta(_) => "meta",
        TypedRootV1::Composition(_) => "composition",
    }
}

pub(crate) fn ir_root_kind_v1(root: &PublicIrRootV1) -> &'static str {
    match root {
        PublicIrRootV1::Select(_) => "select",
        PublicIrRootV1::Find(_) => "find",
        PublicIrRootV1::Check(_) => "check",
        PublicIrRootV1::Explain(_) => "explain",
        PublicIrRootV1::Retrieval(_) => "retrieval",
        PublicIrRootV1::Interactive(_) => "interactive",
        PublicIrRootV1::Declaration(_) => "declaration",
        PublicIrRootV1::Meta(_) => "meta",
        PublicIrRootV1::Composition(_) => "composition",
    }
}

pub(crate) fn syntax_root_summary_v1(root: &AstRootV1) -> String {
    match root {
        AstRootV1::Select(SelectRootAstV1 { from }) => format!("query={from}"),
        AstRootV1::Find(FindRootAstV1 { subkind, seed }) => {
            format!("subkind={} seed={seed}", subkind.as_str())
        }
        AstRootV1::Check(CheckRootAstV1 { rule_id, scope }) => match root.as_check_intent_v1() {
            Some(intent) => format!(
                "rule_id={rule_id} mode={:?} class={} scope={:?} pattern={:?}",
                intent.execution_mode(),
                intent.class,
                intent.scope,
                intent.pattern
            ),
            None => format!("rule_id={rule_id} scope={scope}"),
        },
        AstRootV1::Explain(ExplainRootAstV1 { target }) => match root.as_explain_shape_v1() {
            Some(shape) => format!(
                "target={} evidence={:?} truth={:?}",
                shape.target, shape.evidence, shape.truth_expectation
            ),
            None => format!("target={}", target.as_deref().unwrap_or("<missing>")),
        },
        AstRootV1::Show(ShowRootAstV1 { category }) => format!("category={category}"),
        AstRootV1::Describe(DescribeRootAstV1 { target }) => describe_summary_v1(target),
        AstRootV1::Sample(SampleRootAstV1 { query_text }) => format!("payload={query_text}"),
        AstRootV1::Compare(CompareRootAstV1 { baseline_ref }) => {
            format!("payload={baseline_ref}")
        }
        AstRootV1::Export(ExportRootAstV1 { source_ref }) => format!("payload={source_ref}"),
        AstRootV1::View(ViewDeclAstV1 { name }) => format!("carrier={name}"),
        AstRootV1::Rule(RuleDeclAstV1 { name }) => format!("carrier={name}"),
        AstRootV1::Model(ModelDeclAstV1 { name }) => format!("carrier={name}"),
        AstRootV1::Pack(PackDeclAstV1 { name }) => format!("carrier={name}"),
        AstRootV1::Profile(ProfileDeclAstV1 { name }) => format!("carrier={name}"),
        AstRootV1::Gate(GateDeclAstV1 { name }) => format!("carrier={name}"),
        AstRootV1::Meta(command) => syntax_meta_summary_v1(command),
        AstRootV1::Composition(spec) => format!(
            "lets={} with={} body={}",
            spec.lets.len(),
            spec.with_name,
            syntax_root_kind_v1(&spec.body)
        ),
    }
}

/// DESCRIBE summary — when `target` names a published entity/relation, render its
/// schema fields from the catalog registry so the headless summary surfaces the
/// real fact surface (E11A: catalog-facing root referencing the schema types),
/// not just the echoed target. An unknown target falls back to `target=<name>`.
fn describe_summary_v1(target: &str) -> String {
    match quanta_gqlang_v2_ir::catalog::fact_catalog::describe_schema_rows_v1(target) {
        Some(rows) => {
            let fields = rows
                .iter()
                .enumerate()
                .map(|(index, (name, type_label))| {
                    format!("describe.schema.field[{index}]={name}:{type_label}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("target={target} {fields}")
        }
        None => format!("target={target}"),
    }
}

pub(crate) fn typed_root_summary_v1(root: &TypedRootV1) -> String {
    match root {
        TypedRootV1::Select(spec) => format!("query={}", spec.from_entity),
        TypedRootV1::Find(spec) => format!("{spec:?}"),
        TypedRootV1::Check(spec) => match spec.intent() {
            Ok(intent) => format!(
                "rule_id={} mode={:?} class={} scope={:?} pattern={:?}",
                intent.rule_id,
                intent.execution_mode(),
                intent.class,
                intent.scope,
                intent.pattern
            ),
            Err(err) => format!("invalid-check: {}", err.message),
        },
        TypedRootV1::Explain(spec) => format!("{spec:?}"),
        TypedRootV1::ShowDescribe(spec) => format!("{spec:?}"),
        TypedRootV1::SampleCompareExport(spec) => format!("{spec:?}"),
        TypedRootV1::Declaration(spec) => format!("{spec:?}"),
        TypedRootV1::Meta(spec) => match spec {
            TypedMetaCommandV1::UsePack {
                pack_id,
                activation_mode,
                lockfile_path,
            } => format!(
                "use-pack id={pack_id} activation={} lockfile={lockfile_path}",
                typed_pack_activation_summary_v1(*activation_mode)
            ),
            TypedMetaCommandV1::RequireCertified {
                truth_requirement,
                admission_policy,
            } => format!(
                "require-certified truth_requirement={} admission_policy={}",
                typed_certified_truth_summary_v1(*truth_requirement),
                typed_certified_admission_summary_v1(*admission_policy)
            ),
        },
        TypedRootV1::Composition(spec) => format!("{spec:?}"),
    }
}

pub(crate) fn ir_root_summary_v1(root: &PublicIrRootV1) -> String {
    match root {
        PublicIrRootV1::Select(spec) => {
            let shape = spec.shape_v1();
            format!(
                "table={} projection={:?} filter={:?} limit={:?}",
                shape.table.dotted_name_v1(),
                shape.projection,
                shape.filter,
                shape.limit
            )
        }
        PublicIrRootV1::Find(spec) => format!("{spec:?}"),
        PublicIrRootV1::Check(spec) => {
            let mode = spec.execution_mode();
            format!("rule_id={} mode={mode:?}", spec.intent().rule_id)
        }
        PublicIrRootV1::Explain(spec) => {
            let shape = spec.shape();
            format!(
                "target={} evidence={:?} truth={:?}",
                shape.target, shape.evidence, shape.truth_expectation
            )
        }
        PublicIrRootV1::Retrieval(spec) => format!(
            "query={} mode={:?} scope={:?} limit={}",
            spec.query, spec.mode, spec.scope, spec.limit
        ),
        PublicIrRootV1::Interactive(spec) => interactive_ir_summary_v1(spec),
        PublicIrRootV1::Declaration(spec) => declaration_ir_summary_v1(spec),
        PublicIrRootV1::Meta(spec) => meta_ir_summary_v1(spec),
        PublicIrRootV1::Composition(spec) => {
            let ids = spec
                .bindings
                .iter()
                .map(|(reference, _)| reference.working_set_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "bindings=[{ids}] narrowed_by={} body={}",
                spec.narrowed_by,
                ir_root_kind_v1(&spec.body)
            )
        }
    }
}

pub(crate) fn render_syntax_page_v1(page: SyntaxDiagnosticPageV1) -> String {
    let mut lines = vec![format!(
        "syntax.diagnostic_count={}",
        page.diagnostics.len()
    )];
    for (index, diagnostic) in page.diagnostics.iter().enumerate() {
        let span = diagnostic.span.as_ref().map_or_else(
            || "none".to_string(),
            |span| format!("{}..{}", span.start, span.end),
        );
        lines.push(format!(
            "syntax.diagnostic[{index}].code={}",
            diagnostic.code
        ));
        lines.push(format!(
            "syntax.diagnostic[{index}].message={}",
            diagnostic.message
        ));
        lines.push(format!("syntax.diagnostic[{index}].span={span}"));
    }
    lines.join("\n")
}

pub(crate) fn render_sema_page_v1(page: SemaDiagnosticPageV1) -> String {
    let mut lines = vec![format!("sema.diagnostic_count={}", page.diagnostics.len())];
    for (index, diagnostic) in page.diagnostics.iter().enumerate() {
        lines.push(format!("sema.diagnostic[{index}]={diagnostic}"));
    }
    lines.join("\n")
}

pub(crate) fn render_ir_error_v1(error: IrLoweringErrorV1) -> String {
    format!("ir.error={}: {}", error.code, error.message)
}

fn syntax_meta_summary_v1(command: &MetaCommandAstV1) -> String {
    match command.shape() {
        Ok(MetaCommandShapeV1::UsePack(spec)) => {
            format!(
                "use-pack id={} activation={} lockfile={}",
                spec.pack_id,
                pack_activation_meta_summary_v1(spec.activation_mode),
                spec.lockfile_path
            )
        }
        Ok(MetaCommandShapeV1::RequireCertified(spec)) => format!(
            "require-certified truth_requirement={} admission_policy={}",
            certified_truth_meta_summary_v1(spec.truth_requirement),
            certified_admission_meta_summary_v1(spec.admission_policy)
        ),
        Err(err) => format!("invalid-meta {}: {}", err.code, err.message),
    }
}

fn interactive_ir_summary_v1(spec: &InteractiveIrV1) -> String {
    match spec {
        InteractiveIrV1::Show { category } => format!("show category={category:?}"),
        InteractiveIrV1::Describe { target } => {
            format!("describe {}", interactive_target_summary_v1(target))
        }
        InteractiveIrV1::Compare {
            baseline,
            candidate,
            metric,
        } => format!(
            "compare baseline={} candidate={} metric={}",
            interactive_ref_summary_v1(baseline),
            interactive_ref_summary_v1(candidate),
            compare_metric_summary_v1(*metric)
        ),
        InteractiveIrV1::Export {
            source,
            kind,
            profile,
        } => format!(
            "export source={} kind={} profile={profile:?}",
            interactive_ref_summary_v1(source),
            export_kind_summary_v1(*kind)
        ),
    }
}

fn declaration_ir_summary_v1(spec: &DeclarationIrV1) -> String {
    match spec {
        DeclarationIrV1::View { name, query } => match query {
            ViewQueryIrV1::QueryRef(query_ref) => format!("view name={name} query-ref={query_ref}"),
            ViewQueryIrV1::InlineSelect(select) => {
                format!("view name={name} inline-select={select}")
            }
        },
        DeclarationIrV1::Rule { name, check_ref } => {
            format!("rule name={name} check-ref={check_ref}")
        }
        DeclarationIrV1::Model {
            name,
            provider,
            source,
        } => format!("model name={name} provider={provider} source={source}"),
        DeclarationIrV1::Pack {
            name,
            source,
            activation,
        } => format!(
            "pack name={name} source={source} activation={}",
            pack_activation_summary_v1(*activation)
        ),
        DeclarationIrV1::Profile {
            name,
            capabilities,
            admission,
        } => format!(
            "profile name={name} capabilities={capabilities:?} admission={}",
            profile_admission_summary_v1(*admission)
        ),
        DeclarationIrV1::Gate {
            name,
            checks,
            binding,
        } => format!(
            "gate name={name} checks={checks:?} binding={}",
            gate_binding_summary_v1(*binding)
        ),
    }
}

fn meta_ir_summary_v1(spec: &MetaIrV1) -> String {
    match spec {
        MetaIrV1::UsePack {
            pack_id,
            activation_mode,
            lockfile_path,
        } => format!(
            "use-pack id={pack_id} activation={} lockfile={lockfile_path}",
            pack_activation_summary_v1(*activation_mode)
        ),
        MetaIrV1::RequireCertified {
            truth_requirement,
            admission_policy,
        } => format!(
            "require-certified truth_requirement={} admission_policy={}",
            certified_truth_ir_summary_v1(*truth_requirement),
            certified_admission_ir_summary_v1(*admission_policy)
        ),
    }
}

fn interactive_target_summary_v1(target: &InteractiveTargetIrV1) -> String {
    format!("{:?}::{}", target.kind, target.path.join("."))
}

fn interactive_ref_summary_v1(reference: &InteractiveRefIrV1) -> String {
    format!("{}::{}", reference.kind, reference.locator)
}

fn compare_metric_summary_v1(metric: CompareMetricIrV1) -> &'static str {
    match metric {
        CompareMetricIrV1::Exact => "exact",
        CompareMetricIrV1::Structural => "structural",
        CompareMetricIrV1::Semantic => "semantic",
    }
}

fn export_kind_summary_v1(kind: ExportKindIrV1) -> &'static str {
    match kind {
        ExportKindIrV1::Json => "json",
        ExportKindIrV1::Csv => "csv",
        ExportKindIrV1::Sarif => "sarif",
        ExportKindIrV1::Markdown => "markdown",
        ExportKindIrV1::GraphMl => "graph-ml",
    }
}

fn pack_activation_summary_v1(mode: PackActivationIrV1) -> &'static str {
    match mode {
        PackActivationIrV1::Manual => "manual",
        PackActivationIrV1::Session => "session",
        PackActivationIrV1::Workspace => "workspace",
    }
}

fn pack_activation_meta_summary_v1(mode: PackActivationModeAstV1) -> &'static str {
    mode.as_str_v1()
}

fn certified_truth_meta_summary_v1(
    truth_requirement: CertifiedTruthRequirementAstV1,
) -> &'static str {
    truth_requirement.as_str_v1()
}

fn certified_admission_meta_summary_v1(
    admission_policy: CertifiedAdmissionPolicyAstV1,
) -> &'static str {
    admission_policy.as_str_v1()
}

fn typed_pack_activation_summary_v1(mode: TypedPackActivationModeV1) -> &'static str {
    mode.as_str_v1()
}

fn typed_certified_truth_summary_v1(
    truth_requirement: TypedCertifiedTruthRequirementV1,
) -> &'static str {
    truth_requirement.as_str_v1()
}

fn typed_certified_admission_summary_v1(
    admission_policy: TypedCertifiedAdmissionPolicyV1,
) -> &'static str {
    admission_policy.as_str_v1()
}

fn certified_truth_ir_summary_v1(truth_requirement: CertifiedTruthRequirementIrV1) -> &'static str {
    truth_requirement.as_str_v1()
}

fn certified_admission_ir_summary_v1(
    admission_policy: CertifiedAdmissionPolicyIrV1,
) -> &'static str {
    admission_policy.as_str_v1()
}

fn profile_admission_summary_v1(policy: ProfileAdmissionIrV1) -> &'static str {
    match policy {
        ProfileAdmissionIrV1::Allowlist => "allowlist",
        ProfileAdmissionIrV1::ReviewRequired => "review-required",
        ProfileAdmissionIrV1::CertifiedOnly => "certified-only",
    }
}

fn gate_binding_summary_v1(binding: GateBindingIrV1) -> &'static str {
    match binding {
        GateBindingIrV1::All => "all",
        GateBindingIrV1::Any => "any",
    }
}

trait AstRootExtV1 {
    fn as_check_intent_v1(&self) -> Option<CheckRootIntentV1>;
    fn as_explain_shape_v1(&self) -> Option<ExplainShapeAstV1>;
}

impl AstRootExtV1 for AstRootV1 {
    fn as_check_intent_v1(&self) -> Option<CheckRootIntentV1> {
        if let AstRootV1::Check(spec) = self {
            spec.intent().ok()
        } else {
            None
        }
    }

    fn as_explain_shape_v1(&self) -> Option<ExplainShapeAstV1> {
        if let AstRootV1::Explain(spec) = self {
            spec.shape().ok()
        } else {
            None
        }
    }
}
