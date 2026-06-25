//! Deterministic v3 plan / cost / explain rendering (ticket 08, D32 / D22).
//!
//! Renders the host-agnostic [`quanta_gqlang_v2_planning::PublicPlanV1`] from the
//! shared pipeline ([`quanta_gqlang_v2_pipeline::compile_program_structural`], D22) into a
//! STABLE `key=value` line report. The renderer consumes the fixed semantic-cost
//! schema from ticket 07 unchanged (D32): the five fields
//! `estimated_cardinality_class`, `recursion_risk_class`,
//! `witness_explosion_risk_class`, `ordering_contract`, and
//! `unsupported_reason_code` are surfaced verbatim with their stable keywords and
//! never re-derived here.
//!
//! Every collection is rendered in a stable order:
//!
//! - capabilities come from a `BTreeSet` and render in `Ord` order;
//! - explain families render root-first in the planner's pre-order;
//! - the cost fields render in the fixed `SemanticCostV1::FIELD_NAMES` order.
//!
//! This is the ONE renderer CLI / LSP / tests reuse for the plan summary so the
//! semantic payload cannot drift across surfaces.

use quanta_gqlang_v2_pipeline::CompiledItemV1;
use quanta_gqlang_v2_planning::{PublicPlanV1, SemanticCostV1, WitnessSelectionPolicyV1};

/// Render the whole per-family compiled outcome into a stable, line-oriented
/// report — the ONE renderer CLI / LSP / tests reuse so the surfaced payload
/// cannot drift across surfaces or item kinds.
///
/// Every variant leads with a stable `item.kind=<tag>` line so the consumer can
/// tell the families apart without re-deriving the kind, then renders exactly the
/// payload that family carries:
///
/// - a plan-bearing item (relational / FIND / CHECK / presentation) renders its
///   [`render_plan_summary_v1`] (result_kind + effect + capabilities + the fixed
///   5-field cost schema);
/// - a catalog registration renders its minted asset id (`registered
///   <kind>::<name>`), which carries no plan (D);
/// - a composition renders its body plan plus each bound producer's kind, in
///   written order, then recurses into the body;
/// - a meta directive renders its effect (`use pack` activation vs
///   `require certified` gate); `USE PACK` carries no plan, `REQUIRE CERTIFIED`
///   appends its gate plan.
///
/// The semantic payload itself is never re-derived here (D22): every keyword and
/// cost field is surfaced verbatim from the compiled outcome.
#[must_use]
pub fn render_compiled_item_v1(compiled: &CompiledItemV1) -> String {
    match compiled {
        CompiledItemV1::RelationalQuery { plan, .. } => {
            with_kind_v1("relational_query", render_plan_summary_v1(plan))
        }
        CompiledItemV1::ValueArtifact {
            value_type,
            no_serve_reason,
            ..
        } => with_kind_v1(
            "value_artifact",
            format!(
                "artifact.serveability=no_serve\nartifact.no_serve_reason={}\nartifact.value_type={}",
                no_serve_reason.keyword(),
                value_type
                    .as_ref()
                    .map_or_else(|| "unknown".to_owned(), ToString::to_string)
            ),
        ),
        CompiledItemV1::GraphQuery { plan, .. } => {
            with_kind_v1("graph_query", render_plan_summary_v1(plan))
        }
        CompiledItemV1::Legality { plan, .. } => {
            with_kind_v1("legality", render_plan_summary_v1(plan))
        }
        CompiledItemV1::Presentation { plan, .. } => {
            with_kind_v1("presentation", render_plan_summary_v1(plan))
        }
        CompiledItemV1::RuleArtifact {
            proof,
            no_serve_reason,
            ..
        } => with_kind_v1(
            "rule_artifact",
            format!(
                "artifact.serveability=no_serve\nartifact.no_serve_reason={}\nartifact.recursive={}\nartifact.strata={}",
                no_serve_reason.keyword(),
                !proof.recursive.is_empty(),
                proof.stratum_count
            ),
        ),
        CompiledItemV1::CatalogRegistration { asset_id, .. } => {
            with_kind_v1("catalog_registration", format!("registered {asset_id}"))
        }
        CompiledItemV1::Composition { bindings, body, .. } => render_composition_v1(bindings, body),
        CompiledItemV1::Meta { hir, plan } => render_meta_v1(hir, plan.as_deref()),
    }
}

/// Prefix a rendered body with its stable `item.kind=<tag>` line.
fn with_kind_v1(kind: &str, body: String) -> String {
    format!("item.kind={kind}\n{body}")
}

/// Render a composition: each bound producer's kind in written order, then the
/// body's full compiled rendering (which carries the body plan). The body is the
/// composition's consumed result, so its plan is surfaced verbatim through the
/// shared per-item renderer rather than re-derived here.
fn render_composition_v1(bindings: &[CompiledItemV1], body: &CompiledItemV1) -> String {
    let mut lines = vec![
        "item.kind=composition".to_owned(),
        format!("composition.binding_count={}", bindings.len()),
    ];
    for (index, binding) in bindings.iter().enumerate() {
        lines.push(format!(
            "composition.binding[{index}].kind={}",
            item_kind_tag_v1(binding)
        ));
    }
    // The body carries the consumed plan; render it through the SAME per-item
    // renderer (indent-free, line-oriented) so the body's payload matches a
    // stand-alone item of the same kind byte-for-byte.
    lines.push("composition.body:".to_owned());
    lines.push(render_compiled_item_v1(body));
    lines.join("\n")
}

/// Render a meta directive's effect. `USE PACK` is effect-style activation with no
/// plan; `REQUIRE CERTIFIED` is a proof-confirmed gate whose plan is appended.
fn render_meta_v1(hir: &quanta_gqlang_v2_hir::MetaHirV1, plan: Option<&PublicPlanV1>) -> String {
    use quanta_gqlang_v2_hir::MetaKindV1;
    let mut lines = vec!["item.kind=meta".to_owned()];
    match &hir.kind {
        MetaKindV1::UsePack { pack_id, .. } => {
            lines.push("meta.effect=use pack".to_owned());
            lines.push(format!("meta.pack_id={pack_id}"));
        }
        MetaKindV1::RequireCertified => {
            lines.push("meta.effect=require certified gate".to_owned());
        }
    }
    // A gate-bearing meta (REQUIRE CERTIFIED) appends its plan; USE PACK has none.
    match plan {
        Some(plan) => lines.push(render_plan_summary_v1(plan)),
        None => lines.push("plan.none=true".to_owned()),
    }
    lines.join("\n")
}

/// The stable kind tag for a compiled item, used when listing a composition's
/// bound producers without rendering their full payload.
fn item_kind_tag_v1(compiled: &CompiledItemV1) -> &'static str {
    match compiled {
        CompiledItemV1::RelationalQuery { .. } => "relational_query",
        CompiledItemV1::ValueArtifact { .. } => "value_artifact",
        CompiledItemV1::GraphQuery { .. } => "graph_query",
        CompiledItemV1::Legality { .. } => "legality",
        CompiledItemV1::Presentation { .. } => "presentation",
        CompiledItemV1::RuleArtifact { .. } => "rule_artifact",
        CompiledItemV1::CatalogRegistration { .. } => "catalog_registration",
        CompiledItemV1::Composition { .. } => "composition",
        CompiledItemV1::Meta { .. } => "meta",
    }
}

/// Render a compiled plan into a stable, line-oriented summary including the
/// fixed semantic-cost schema.
#[must_use]
pub fn render_plan_summary_v1(plan: &PublicPlanV1) -> String {
    let mut lines = Vec::new();

    // ---- plan summary header (stable, derived from the plan only) ----------
    lines.push(format!("plan.result_kind={}", plan.result_kind.keyword()));
    lines.push(format!("plan.truth_lane={}", plan.truth_lane.keyword()));
    lines.push(format!("plan.evidence={}", plan.evidence.keyword()));
    lines.push(format!("plan.effect={}", plan.effect.keyword()));
    lines.push(format!(
        "plan.render_requirement={}",
        plan.render_requirement.keyword()
    ));

    // ---- capabilities (BTreeSet -> Ord order, deterministic) ---------------
    let capabilities = plan
        .capabilities
        .iter()
        .map(|cap| cap.keyword())
        .collect::<Vec<_>>()
        .join(",");
    lines.push(format!("plan.capabilities={capabilities}"));

    // ---- multiplicity / order contract (D26: unordered stays unordered) ----
    lines.push(format!(
        "plan.multiplicity.cardinality={}",
        plan.multiplicity_order.cardinality.keyword()
    ));
    lines.push(format!(
        "plan.multiplicity.eliminates_duplicates={}",
        plan.multiplicity_order.eliminates_duplicates()
    ));
    lines.push(format!(
        "plan.multiplicity.is_ordered={}",
        plan.multiplicity_order.is_ordered()
    ));

    // ---- witness selection policy, when materialized -----------------------
    lines.push(format!(
        "plan.witness_selection={}",
        render_witness_selection_v1(plan.witness_selection)
    ));

    // ---- explain note (logical provenance, root-first stable order) --------
    lines.push(format!("plan.explain.recursive={}", plan.explain.recursive));
    lines.push(format!(
        "plan.explain.root_span={}..{}",
        plan.explain.root_span.start, plan.explain.root_span.end
    ));
    let families = plan
        .explain
        .families
        .iter()
        .map(|family| family.keyword())
        .collect::<Vec<_>>()
        .join(",");
    lines.push(format!("plan.explain.families={families}"));

    // ---- fixed semantic-cost schema (D32, verbatim field names) ------------
    lines.extend(render_cost_lines_v1(&plan.cost));

    lines.join("\n")
}

/// Render exactly the five fixed semantic-cost fields, in
/// [`SemanticCostV1::FIELD_NAMES`] order, each as `cost.<field>=<keyword>`. The
/// `unsupported_reason_code` is `none` when servable so the field is always
/// present and the schema never drifts (D32).
#[must_use]
pub fn render_cost_lines_v1(cost: &SemanticCostV1) -> Vec<String> {
    vec![
        format!(
            "cost.estimated_cardinality_class={}",
            cost.estimated_cardinality_class.keyword()
        ),
        format!(
            "cost.recursion_risk_class={}",
            cost.recursion_risk_class.keyword()
        ),
        format!(
            "cost.witness_explosion_risk_class={}",
            cost.witness_explosion_risk_class.keyword()
        ),
        format!(
            "cost.ordering_contract={}",
            cost.ordering_contract.keyword()
        ),
        format!(
            "cost.unsupported_reason_code={}",
            cost.unsupported_reason_code.as_deref().unwrap_or("none")
        ),
    ]
}

/// Render the witness-selection policy deterministically. `TopK` exposes its cap
/// `k` so the rendered set bound is visible without breaking the closed keyword
/// set; an absent policy renders `none`.
fn render_witness_selection_v1(selection: Option<WitnessSelectionPolicyV1>) -> String {
    match selection {
        None => "none".to_owned(),
        Some(WitnessSelectionPolicyV1::TopK { k }) => format!("top_k(k={k})"),
        Some(other) => other.keyword().to_owned(),
    }
}

#[cfg(test)]
mod render_witness_selection_tests_v1 {
    use super::render_witness_selection_v1;
    use quanta_gqlang_v2_planning::WitnessSelectionPolicyV1;

    /// An absent policy renders exactly `none`. Pins the `None` arm against the
    /// whole-body stubs (`String::new()` / `"xyzzy".into()`).
    #[test]
    fn absent_policy_renders_none_v1() {
        assert_eq!(render_witness_selection_v1(None), "none");
    }

    /// `TopK` exposes its finite cap `k` in the rendered set bound. Pins the
    /// `top_k(k={k})` format string (and the whole-body stubs): the cap is
    /// surfaced verbatim, not the closed keyword `top_k`.
    #[test]
    fn top_k_renders_cap_inline_v1() {
        assert_eq!(
            render_witness_selection_v1(Some(WitnessSelectionPolicyV1::TopK { k: 7 })),
            "top_k(k=7)"
        );
    }

    /// The non-`TopK` policies render their closed keyword verbatim. Pins the
    /// `other.keyword()` arm against the whole-body stubs.
    #[test]
    fn keyword_policies_render_their_keyword_v1() {
        assert_eq!(
            render_witness_selection_v1(Some(WitnessSelectionPolicyV1::CanonicalOne)),
            "canonical_one"
        );
        assert_eq!(
            render_witness_selection_v1(Some(WitnessSelectionPolicyV1::AllWitness)),
            "all_witness"
        );
    }
}
