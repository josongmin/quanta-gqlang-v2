//! Canonical v3 surface formatter: the exact inverse of the grammar.
//!
//! This crate owns the v3 surface grammar, so it owns the canonical renderer.
//! [`format_item`] re-renders EVERY top-level family
//! ([`ProgramItemHirV1`]) back to v3 surface text, and [`format_document`]
//! renders an ordered list of items joined by the document `;` separator.
//!
//! ## Guarantees (driven by the formatter hellgate rails)
//!
//! - **Round-trip-stable**: `parse(format(parse(src)))` is structurally equal to
//!   `parse(src)` (ids/spans aside) for every family.
//! - **Idempotent**: `format(format(x)) == format(x)` — the canonical text is a
//!   fixed point, because re-parsing canonical text yields the same structure
//!   which renders to the same text.
//!
//! The renderer is STRUCTURE-faithful: it emits exactly the clauses/forms the
//! parser accepts, never a lossy `#[unformatted]` marker, and never invents
//! clauses absent from the HIR (so defaulted-but-unwritten modes are omitted only
//! when the parser also treats them as defaults — see the per-family notes).
//!
//! Join, group, and order stages are rendered from their dedicated closed HIR
//! variants; there is no open operator tag or synthetic relation operand.

use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CatalogPayloadV1,
    CompareMetricV1, CompositionBodyV1, CompositionHirV1, CompositionProducerV1, ConfirmModeV1,
    DetectionModeV1, EmitModeV1, EvidenceModeV1, ExportFormatV1, GateBindingV1, GraphConstraintsV1,
    GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1, JoinHirV1, LegalityHirV1,
    LiteralV1, MetaKindV1, PresentationHirV1, PresentationKindV1, ProgramItemHirV1, QueryHirV1,
    RelationExprHirV1, RelationExprKindV1, ResultShapeHirV1, ResultShapeKindV1, SampleModeV1,
    ShowCategoryV1, SortDirectionHirV1, TruthModeV1, UsePackActivationV1, ValueExprHirV1,
    ValueExprKindV1, ViewBodyV1,
};

/// Render an ordered list of top-level items to a canonical v3 document.
///
/// Items are joined by the document separator (`;`) on their own lines; the
/// final item carries no trailing separator. The result re-parses through
/// [`crate::parse_document`] to the same ordered structure.
#[must_use]
pub fn format_document(items: &[ProgramItemHirV1]) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(";\n");
        }
        out.push_str(&format_item(item));
    }
    out
}

/// Render a single top-level item to canonical v3 surface text.
#[must_use]
pub fn format_item(item: &ProgramItemHirV1) -> String {
    match item {
        ProgramItemHirV1::Query(q) => format_query(q),
        ProgramItemHirV1::Decl(d) => format!("let {} = {}", d.name, format_value(&d.body)),
        ProgramItemHirV1::Rule(r) => format!(
            "rule {} : {} if {}",
            r.name,
            format_value(&r.head),
            format_bool(&r.body)
        ),
        ProgramItemHirV1::GraphQuery(g) => format_find(g),
        ProgramItemHirV1::Legality(c) => format_check(c),
        ProgramItemHirV1::Presentation(p) => format_presentation(p),
        ProgramItemHirV1::CatalogDecl(d) => format_catalog_decl(d),
        ProgramItemHirV1::Composition(c) => format_composition(c),
        ProgramItemHirV1::Meta(m) => format_meta(m),
    }
}

// ---- query -----------------------------------------------------------------

fn format_query(q: &QueryHirV1) -> String {
    let mut out = String::from("from ");
    out.push_str(&format_relation(&q.body));
    if let Some(shape) = &q.result_shape {
        out.push_str(" => ");
        out.push_str(&format_shape(shape));
    }
    out
}

/// Render a relation as `<source> [| stage]*` WITHOUT the leading `from `.
fn format_relation(rel: &RelationExprHirV1) -> String {
    match &rel.kind {
        RelationExprKindV1::Source(src) => match &src.alias {
            Some(a) => format!("{} {}", src.name, a.name),
            None => src.name.clone(),
        },
        RelationExprKindV1::Project(proj) => {
            let items = proj
                .items()
                .iter()
                .map(format_projection_item)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} | project {items}", format_relation(&proj.input))
        }
        RelationExprKindV1::Filter(filter) => {
            format!(
                "{} | where {}",
                format_relation(&filter.input),
                format_bool(&filter.predicate)
            )
        }
        RelationExprKindV1::Join(JoinHirV1::Inner { left, right, on }) => format!(
            "{} | join {} on {}",
            format_relation(left),
            format_source_only(right),
            format_bool(on)
        ),
        RelationExprKindV1::Join(JoinHirV1::Cross { left, right }) => format!(
            "{} | cross join {}",
            format_relation(left),
            format_source_only(right)
        ),
        RelationExprKindV1::GroupBy(group) => format!(
            "{} | group by {}",
            format_relation(&group.input),
            group
                .keys()
                .iter()
                .map(format_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        RelationExprKindV1::OrderBy(order) => format!(
            "{} | order by {}",
            format_relation(&order.input),
            order
                .keys()
                .iter()
                .map(|key| format!(
                    "{} {}",
                    format_value(&key.value),
                    match key.direction {
                        SortDirectionHirV1::Asc => "asc",
                        SortDirectionHirV1::Desc => "desc",
                    }
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Render a join right-hand source: `<name> [<alias>]`. The right operand of a
/// join compose is always a relation source by the grammar.
fn format_source_only(rel: &RelationExprHirV1) -> String {
    match &rel.kind {
        RelationExprKindV1::Source(src) => match &src.alias {
            Some(a) => format!("{} {}", src.name, a.name),
            None => src.name.clone(),
        },
        RelationExprKindV1::Project(_)
        | RelationExprKindV1::Filter(_)
        | RelationExprKindV1::Join(_)
        | RelationExprKindV1::GroupBy(_)
        | RelationExprKindV1::OrderBy(_) => format_relation(rel),
    }
}

fn format_projection_item(item: &quanta_gqlang_v2_hir::ProjectionItemV1) -> String {
    match &item.alias {
        Some(a) => format!("{} as {}", format_value(&item.value), a.name),
        None => format_value(&item.value),
    }
}

// ---- result shape ----------------------------------------------------------

fn format_shape(shape: &ResultShapeHirV1) -> String {
    match &shape.kind {
        ResultShapeKindV1::Leaf(value) => format_value(value),
        ResultShapeKindV1::Record(rec) => {
            let fields = rec
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name, format_shape(&f.shape)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {fields} }}")
        }
        ResultShapeKindV1::List(list) => format!("[ {} ]", format_shape(&list.element)),
    }
}

// ---- value / bool expressions ----------------------------------------------

fn format_value(value: &ValueExprHirV1) -> String {
    format_value_prec(value, 0, false)
}

fn value_precedence(value: &ValueExprHirV1) -> u8 {
    match &value.kind {
        ValueExprKindV1::Binary(binary) => match binary.op.symbol.as_str() {
            "+" | "-" => 10,
            "*" | "/" | "%" => 20,
            _ => 5,
        },
        ValueExprKindV1::Unary(_) => 30,
        ValueExprKindV1::Literal(_)
        | ValueExprKindV1::Symbol(_)
        | ValueExprKindV1::Path(_)
        | ValueExprKindV1::Call(_)
        | ValueExprKindV1::Paren(_)
        | ValueExprKindV1::Record(_)
        | ValueExprKindV1::List(_) => 40,
    }
}

fn format_value_prec(value: &ValueExprHirV1, parent: u8, right_child: bool) -> String {
    let precedence = value_precedence(value);
    match &value.kind {
        ValueExprKindV1::Literal(lit) => format_literal(lit),
        ValueExprKindV1::Symbol(sym) => sym.name.clone(),
        ValueExprKindV1::Path(path) => {
            let mut out = format_value_prec(&path.base, 40, false);
            for seg in path.segments.as_slice() {
                out.push('.');
                out.push_str(&seg.name);
            }
            out
        }
        ValueExprKindV1::Unary(unary) => {
            format!(
                "{}{}",
                unary.op.symbol,
                format_value_prec(&unary.operand, precedence, false)
            )
        }
        ValueExprKindV1::Binary(bin) => {
            let rendered = format!(
                "{} {} {}",
                format_value_prec(&bin.lhs, precedence, false),
                bin.op.symbol,
                format_value_prec(&bin.rhs, precedence, true)
            );
            if precedence < parent || (right_child && precedence == parent) {
                format!("({rendered})")
            } else {
                rendered
            }
        }
        ValueExprKindV1::Call(call) => {
            let args = call
                .args()
                .as_slice()
                .iter()
                .map(|arg| match arg {
                    CallArgHirV1::Expr(value) => format_value(value),
                    CallArgHirV1::Star { .. } => "*".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({args})", call.callee.name)
        }
        ValueExprKindV1::Paren(inner) => format!("({})", format_value(inner)),
        ValueExprKindV1::Record(rec) => {
            let fields = rec
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name, format_value(&f.value)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {fields} }}")
        }
        ValueExprKindV1::List(list) => {
            let elems = list
                .elements
                .iter()
                .map(format_value)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{elems}]")
        }
    }
}

fn format_literal(lit: &LiteralV1) -> String {
    match lit {
        LiteralV1::Int(raw) | LiteralV1::Float(raw) => raw.clone(),
        LiteralV1::Str(content) => format!("\"{}\"", escape_str(content)),
        LiteralV1::Bool(true) => "true".to_string(),
        LiteralV1::Bool(false) => "false".to_string(),
        LiteralV1::Null => "null".to_string(),
    }
}

fn format_bool(pred: &BoolExprHirV1) -> String {
    format_bool_prec(pred, 0, false)
}

fn bool_precedence(pred: &BoolExprHirV1) -> u8 {
    match pred.kind {
        BoolExprKindV1::Or(_) => 10,
        BoolExprKindV1::And(_) => 20,
        BoolExprKindV1::Not(_) => 30,
        BoolExprKindV1::Literal(_)
        | BoolExprKindV1::Value(_)
        | BoolExprKindV1::Compare(_)
        | BoolExprKindV1::Paren(_)
        | BoolExprKindV1::LogicalQuantifier(_)
        | BoolExprKindV1::RelationExists(_) => 40,
    }
}

fn format_bool_prec(pred: &BoolExprHirV1, parent: u8, right_child: bool) -> String {
    let precedence = bool_precedence(pred);
    let rendered = match &pred.kind {
        BoolExprKindV1::Literal(true) => "true".to_string(),
        BoolExprKindV1::Literal(false) => "false".to_string(),
        BoolExprKindV1::Value(value) => format_value(value),
        BoolExprKindV1::Compare(cmp) => format!(
            "{} {} {}",
            format_value(&cmp.lhs),
            cmp.op.symbol,
            format_value(&cmp.rhs)
        ),
        BoolExprKindV1::And(b) => {
            format!(
                "{} and {}",
                format_bool_prec(&b.lhs, precedence, false),
                format_bool_prec(&b.rhs, precedence, true)
            )
        }
        BoolExprKindV1::Or(b) => {
            format!(
                "{} or {}",
                format_bool_prec(&b.lhs, precedence, false),
                format_bool_prec(&b.rhs, precedence, true)
            )
        }
        BoolExprKindV1::Not(inner) => format!("not {}", format_bool_prec(inner, precedence, false)),
        BoolExprKindV1::Paren(inner) => format!("({})", format_bool(inner)),
        BoolExprKindV1::LogicalQuantifier(q) => format!(
            "{} {} in ({})",
            match q.kind {
                quanta_gqlang_v2_hir::LogicalQuantifierKindV1::Exists => "exists",
                quanta_gqlang_v2_hir::LogicalQuantifierKindV1::Forall => "forall",
            },
            q.binders
                .as_slice()
                .iter()
                .map(|binder| binder.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            format_bool(&q.body)
        ),
        BoolExprKindV1::RelationExists(exists) => format!(
            "{}(from {})",
            match exists.polarity {
                quanta_gqlang_v2_hir::RelationExistsPolarityV1::Exists => "exists ",
                quanta_gqlang_v2_hir::RelationExistsPolarityV1::NotExists => "not exists ",
            },
            format_relation(&exists.relation)
        ),
    };
    if precedence < parent || (right_child && precedence == parent) {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn escape_str(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for ch in content.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            control if control.is_control() => {
                out.push_str(&format!("\\u{{{:x}}}", u32::from(control)))
            }
            other => out.push(other),
        }
    }
    out
}

// ---- FIND ------------------------------------------------------------------

fn format_find(g: &GraphQueryHirV1) -> String {
    let mut out = format!("find {}", find_subkind(g.kind));
    let GraphEndpointsV1 {
        source,
        sink,
        anchor,
        reference,
        pointee,
        callable,
    } = &g.endpoints;
    // The `<source> -> <sink>` seed renders inline when both are present.
    if let (Some(src), Some(snk)) = (source, sink) {
        out.push_str(&format!(" {} -> {}", src.name, snk.name));
    }

    let GraphConstraintsV1 {
        via,
        through,
        sanitizers,
        scope,
        window,
        direction,
        frontier,
        depth,
        summary_facets,
    } = &g.constraints;

    push_ref_list(&mut out, "via", via);
    push_ref_list(&mut out, "through", through);
    push_ref_list(&mut out, "sanitizers", sanitizers);
    push_ref_opt(&mut out, "scope", scope);
    push_ref_opt(&mut out, "window", window);
    push_ref_opt(&mut out, "frontier", frontier);
    if let Some(dir) = direction {
        out.push_str(&format!(" | direction={}", dir.keyword));
    }
    if let Some(d) = depth {
        out.push_str(&format!(" | depth={d}"));
    }
    push_ref_opt(&mut out, "anchor", anchor);
    push_ref_opt(&mut out, "reference", reference);
    push_ref_opt(&mut out, "pointee", pointee);
    push_ref_opt(&mut out, "callable", callable);
    if !summary_facets.is_empty() {
        out.push_str(&format!(" | facets={}", summary_facets.join(", ")));
    }
    out
}

fn find_subkind(kind: GraphQueryKindV1) -> &'static str {
    match kind {
        GraphQueryKindV1::Reachability => "reachability",
        GraphQueryKindV1::Flow => "flow",
        GraphQueryKindV1::Slice => "slice",
        GraphQueryKindV1::Taint => "taint",
        GraphQueryKindV1::Path => "path",
        GraphQueryKindV1::PointsTo => "points_to",
        GraphQueryKindV1::Summary => "summary",
    }
}

fn push_ref_list(out: &mut String, key: &str, refs: &[GraphRefV1]) {
    if refs.is_empty() {
        return;
    }
    let names = refs
        .iter()
        .map(|r| r.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(" | {key}={names}"));
}

fn push_ref_opt(out: &mut String, key: &str, r: &Option<GraphRefV1>) {
    if let Some(r) = r {
        out.push_str(&format!(" | {key}={}", r.name));
    }
}

// ---- CHECK -----------------------------------------------------------------

fn format_check(c: &LegalityHirV1) -> String {
    let mut out = format!("check {}", c.rule_ref.name);
    push_ref_opt(&mut out, "scope", &c.scope);
    push_ref_opt(&mut out, "pattern", &c.pattern);
    // confirm is required by the grammar, so it is always rendered.
    out.push_str(&format!(" | confirm={}", confirm_mode(c.confirm)));
    out.push_str(&format!(" | emit={}", emit_mode(c.emit)));
    out.push_str(&format!(" | mode={}", detection_mode(c.mode)));
    if let Some(confirm) = &c.semantic_confirm {
        out.push_str(" using ");
        out.push_str(&format_find(confirm));
    }
    out
}

fn confirm_mode(m: ConfirmModeV1) -> &'static str {
    match m {
        ConfirmModeV1::Skip => "skip",
        ConfirmModeV1::Prompt => "prompt",
        ConfirmModeV1::Require => "require",
    }
}

fn emit_mode(m: EmitModeV1) -> &'static str {
    match m {
        EmitModeV1::Findings => "findings",
        EmitModeV1::Sarif => "sarif",
    }
}

fn detection_mode(m: DetectionModeV1) -> &'static str {
    match m {
        DetectionModeV1::PatternOnly => "pattern",
        DetectionModeV1::SemanticOnly => "semantic",
        DetectionModeV1::Hybrid => "hybrid",
    }
}

// ---- presentation verbs ----------------------------------------------------

fn format_presentation(p: &PresentationHirV1) -> String {
    match &p.kind {
        PresentationKindV1::Explain {
            target_ref,
            evidence,
            truth,
        } => format!(
            "explain {} | evidence={} | truth={}",
            target_ref.name,
            evidence_mode(*evidence),
            truth_mode(*truth)
        ),
        PresentationKindV1::Show { category } => {
            format!("show {}", show_category(*category))
        }
        PresentationKindV1::Describe { kind, path } => {
            format!("describe {kind}::{}", path.name)
        }
        PresentationKindV1::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            let mut out = format!(
                "sample query=\"{}\" | mode={} | limit={limit}",
                escape_str(query_text),
                sample_mode(*mode)
            );
            push_ref_opt(&mut out, "scope", scope);
            out
        }
        PresentationKindV1::Compare {
            baseline_ref,
            candidate_ref,
            metric,
        } => format!(
            "compare baseline={} | candidate={} | metric={}",
            baseline_ref.name,
            candidate_ref.name,
            compare_metric(*metric)
        ),
        PresentationKindV1::Export {
            source_ref,
            format,
            profile,
        } => {
            let mut out = format!(
                "export source={} | kind={}",
                source_ref.name,
                export_format(*format)
            );
            push_ref_opt(&mut out, "profile", profile);
            out
        }
    }
}

fn evidence_mode(m: EvidenceModeV1) -> &'static str {
    match m {
        EvidenceModeV1::Auto => "auto",
        EvidenceModeV1::Trace => "trace",
        EvidenceModeV1::Citation => "citation",
        EvidenceModeV1::Counterexample => "counterexample",
    }
}

fn truth_mode(m: TruthModeV1) -> &'static str {
    match m {
        TruthModeV1::Inspect => "inspect",
        TruthModeV1::Confirm => "confirm",
        TruthModeV1::Refute => "refute",
    }
}

fn show_category(c: ShowCategoryV1) -> &'static str {
    match c {
        ShowCategoryV1::Capabilities => "capabilities",
        ShowCategoryV1::Models => "models",
        ShowCategoryV1::Views => "views",
        ShowCategoryV1::Rules => "rules",
        ShowCategoryV1::Packs => "packs",
        ShowCategoryV1::Profiles => "profiles",
        ShowCategoryV1::Gates => "gates",
    }
}

fn sample_mode(m: SampleModeV1) -> &'static str {
    match m {
        SampleModeV1::Lexical => "lexical",
        SampleModeV1::Semantic => "semantic",
        SampleModeV1::Hybrid => "hybrid",
    }
}

fn compare_metric(m: CompareMetricV1) -> &'static str {
    match m {
        CompareMetricV1::Exact => "exact",
        CompareMetricV1::Structural => "structural",
        CompareMetricV1::Semantic => "semantic",
    }
}

fn export_format(f: ExportFormatV1) -> &'static str {
    match f {
        ExportFormatV1::Json => "json",
        ExportFormatV1::Csv => "csv",
        ExportFormatV1::Sarif => "sarif",
        ExportFormatV1::Markdown => "markdown",
        ExportFormatV1::GraphMl => "graph-ml",
    }
}

// ---- catalog declarations --------------------------------------------------

fn format_catalog_decl(d: &quanta_gqlang_v2_hir::CatalogDeclHirV1) -> String {
    match &d.payload {
        CatalogPayloadV1::View(body) => match body {
            ViewBodyV1::QueryRef(r) => {
                format!("view {} | query-ref={}", d.name, r.name)
            }
            ViewBodyV1::Inline(query) => {
                format!("view {} | select={}", d.name, format_query(query))
            }
        },
        CatalogPayloadV1::Model { provider, source } => {
            format!(
                "model {} | provider={provider} | source={}",
                d.name, source.name
            )
        }
        CatalogPayloadV1::Pack { source, activation } => {
            format!(
                "pack {} | source={} | activation={}",
                d.name,
                source.name,
                activation_policy(*activation)
            )
        }
        CatalogPayloadV1::Profile {
            capabilities,
            admission,
        } => {
            let mut out = format!("profile {}", d.name);
            push_ref_list(&mut out, "capabilities", capabilities);
            out.push_str(&format!(" | admission={}", admission_policy(*admission)));
            out
        }
        CatalogPayloadV1::Gate { checks, binding } => {
            let mut out = format!("gate {}", d.name);
            push_ref_list(&mut out, "checks", checks);
            out.push_str(&format!(" | binding={}", gate_binding(*binding)));
            out
        }
        CatalogPayloadV1::RuleBinding { check_ref } => {
            format!("ruledef {} | check-ref={}", d.name, check_ref.name)
        }
    }
}

fn activation_policy(a: ActivationV1) -> &'static str {
    match a {
        ActivationV1::Manual => "manual",
        ActivationV1::Session => "session",
        ActivationV1::Workspace => "workspace",
    }
}

fn admission_policy(a: AdmissionV1) -> &'static str {
    match a {
        AdmissionV1::Allowlist => "allowlist",
        AdmissionV1::ReviewRequired => "review-required",
        AdmissionV1::CertifiedOnly => "certified-only",
    }
}

fn gate_binding(b: GateBindingV1) -> &'static str {
    match b {
        GateBindingV1::All => "all",
        GateBindingV1::Any => "any",
    }
}

// ---- composition -----------------------------------------------------------

fn format_composition(c: &CompositionHirV1) -> String {
    let mut out = String::new();
    for binding in &c.bindings {
        out.push_str(&format!(
            "let {} = {} ",
            binding.name,
            format_composition_producer(&binding.producer)
        ));
    }
    out.push_str(&format_composition_body(&c.body));
    out.push_str(&format!(" with {}", c.with_name));
    out
}

fn format_composition_producer(producer: &CompositionProducerV1) -> String {
    match producer {
        CompositionProducerV1::Query(query) => format_query(query),
        CompositionProducerV1::GraphQuery(graph) => format_find(graph),
        CompositionProducerV1::Sample(sample) => {
            let mut out = format!(
                "sample query=\"{}\" | mode={} | limit={}",
                escape_str(&sample.query_text),
                sample_mode(sample.mode),
                sample.limit
            );
            push_ref_opt(&mut out, "scope", &sample.scope);
            out
        }
    }
}

fn format_composition_body(body: &CompositionBodyV1) -> String {
    match body {
        CompositionBodyV1::Query(query) => format_query(query),
        CompositionBodyV1::GraphQuery(graph) => format_find(graph),
        CompositionBodyV1::Legality(legality) => format_check(legality),
        CompositionBodyV1::Explain(explain) => format!(
            "explain {} | evidence={} | truth={}",
            explain.target_ref.name,
            evidence_mode(explain.evidence),
            truth_mode(explain.truth)
        ),
    }
}

// ---- meta ------------------------------------------------------------------

fn format_meta(m: &quanta_gqlang_v2_hir::MetaHirV1) -> String {
    match &m.kind {
        MetaKindV1::UsePack(use_pack) => format!(
            "use pack {} | activation={} | lockfile={}",
            use_pack.pack_id,
            match use_pack.activation {
                UsePackActivationV1::Session => "session",
                UsePackActivationV1::Workspace => "workspace",
            },
            use_pack.lockfile
        ),
        MetaKindV1::RequireCertified => "require certified".to_string(),
    }
}
