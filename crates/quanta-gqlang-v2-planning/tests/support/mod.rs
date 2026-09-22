#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]
#![allow(
    dead_code,
    reason = "shared test helpers are used by a subset of the integration-test crates that include this module"
)]

//! Shared V2 planning fixture builders.
//!
//! The V2 planner has NO caller-assembled carrier: every arm is a sealed
//! upstream contract (ticket 05's lowering product for `Relational`, ticket 06's
//! logic contracts for `Graph`/`Rule`, this crate's validated wrappers for
//! `Check`/`Presentation`/`Composition`, Sema's resolved directive for the meta
//! arm). The only document ingress available to these tests is therefore Sema's
//! `test_support` fixture matrix, plus this crate's own validated
//! `CheckContractV2::from_resolved`.
//!
//! Deliberately NOT hoisted: the V1 `id`/`span`/`col`/`lit`/`pred`/`node`/`scan`
//! algebra-node builders. The V2 entry points never accept a caller-built
//! algebra node, so those helpers have no remaining call site; the raw-HIR
//! `quanta-gqlang-v2-hir` crate is not a dependency of this crate either.

use quanta_gqlang_v2_algebra::lower_relation_item;
use quanta_gqlang_v2_logic::{GraphQueryContractV2, RuleProgramContractV2};
use quanta_gqlang_v2_planning::{
    plan, CheckContractV2, CheckPolicyV2, EffectSummaryV2, GraphWitnessRetentionV2, PlanPolicyV2,
    RuleProgramPolicyV2, SemanticPlanInputV2, SemanticPlanV2,
};
use quanta_gqlang_v2_sema::test_support::{
    reference_graph_document_v2, reference_nested_check_document_v2,
    reference_relation_document_v2, reference_rule_document_v2, ReferenceGraphDocumentV2,
    ReferenceGraphReferenceV2, ReferenceGraphRequestV2, ReferenceNestedCheckDocumentV2,
    ReferenceRelationDocumentV2, ReferenceRelationFixtureV2, ReferenceRuleDocumentV2,
    ReferenceRuleFixtureV2,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, GraphQueryKindV1, GraphReferenceRoleV2, SemanticDocumentV2,
    SemanticItemIdV2, SemanticItemV2,
};

/// The explicit positive depth every shared FIND/CHECK fixture seals.
pub(crate) const FIXTURE_GRAPH_DEPTH_V2: u32 = 4;

/// The authored FIND item span of every shared graph fixture.
pub(crate) const FIXTURE_GRAPH_ITEM_SPAN_V2: (usize, usize) = (0, 8);

/// One sealed relation fixture together with the planning carrier lowered from
/// it by ticket 05's owner entry point.
pub(crate) fn relation_input(
    fixture: ReferenceRelationFixtureV2,
) -> Result<(ReferenceRelationDocumentV2, SemanticPlanInputV2), String> {
    let document = reference_relation_document_v2(fixture)?;
    let input = lower_relation_item(document.document(), document.item_id())
        .map(SemanticPlanInputV2::Relational)
        .map_err(|error| format!("{error:?}"))?;
    Ok((document, input))
}

/// One sealed relation fixture and its assembled plan.
pub(crate) fn planned_relation(
    fixture: ReferenceRelationFixtureV2,
) -> Result<(ReferenceRelationDocumentV2, SemanticPlanV2), String> {
    let (document, input) = relation_input(fixture)?;
    let planned = plan(&input, document.document()).map_err(|error| format!("{error:?}"))?;
    Ok((document, planned))
}

/// The ordered reference set one FIND subkind legally requires.
pub(crate) fn graph_references(kind: GraphQueryKindV1) -> Vec<ReferenceGraphReferenceV2> {
    use GraphReferenceRoleV2 as Role;
    let reference =
        |role: Role, selector: &'static str, span: (usize, usize)| ReferenceGraphReferenceV2 {
            role,
            selector,
            span,
        };
    match kind {
        GraphQueryKindV1::Reachability
        | GraphQueryKindV1::Flow
        | GraphQueryKindV1::Taint
        | GraphQueryKindV1::Path => vec![
            reference(Role::Source, "input_a", (20, 27)),
            reference(Role::Sink, "exec_b", (30, 36)),
        ],
        GraphQueryKindV1::Slice => vec![reference(Role::Anchor, "input_a", (20, 27))],
        GraphQueryKindV1::PointsTo => vec![
            reference(Role::Reference, "input_a", (20, 27)),
            reference(Role::Pointee, "exec_b", (30, 36)),
        ],
        GraphQueryKindV1::Summary => vec![reference(Role::Callable, "input_a", (20, 27))],
    }
}

/// One sealed FIND fixture document of the requested subkind and depth.
pub(crate) fn graph_document(
    kind: GraphQueryKindV1,
    depth: u32,
) -> Result<ReferenceGraphDocumentV2, String> {
    reference_graph_document_v2(ReferenceGraphRequestV2 {
        kind,
        references: graph_references(kind),
        depth: Some(depth),
        direction: None,
        summary_facets: Vec::new(),
        item_span: FIXTURE_GRAPH_ITEM_SPAN_V2,
    })
}

/// One sealed FIND fixture document plus the graph carrier logic seals from it.
pub(crate) fn graph_input(
    kind: GraphQueryKindV1,
    depth: u32,
) -> Result<(ReferenceGraphDocumentV2, SemanticPlanInputV2), String> {
    let document = graph_document(kind, depth)?;
    let contract = graph_contract(&document)?;
    Ok((document, SemanticPlanInputV2::Graph(contract)))
}

/// The bounded graph contract logic seals from a sealed FIND fixture item.
pub(crate) fn graph_contract(
    document: &ReferenceGraphDocumentV2,
) -> Result<GraphQueryContractV2, String> {
    GraphQueryContractV2::from_resolved(document.graph()).map_err(|error| format!("{error:?}"))
}

/// The one shared nested CHECK/FIND fixture document (a `taint` semantic
/// confirm input, `SemanticOnly` detection, `Skip` confirmation, `Findings`
/// emission).
pub(crate) fn check_document() -> Result<ReferenceNestedCheckDocumentV2, String> {
    reference_nested_check_document_v2(ReferenceGraphRequestV2 {
        kind: GraphQueryKindV1::Taint,
        references: graph_references(GraphQueryKindV1::Taint),
        depth: Some(FIXTURE_GRAPH_DEPTH_V2),
        direction: None,
        summary_facets: Vec::new(),
        item_span: FIXTURE_GRAPH_ITEM_SPAN_V2,
    })
}

/// The resolved CHECK item of the shared nested CHECK/FIND fixture.
pub(crate) fn resolved_check(
    document: &ReferenceNestedCheckDocumentV2,
) -> Result<&quanta_gqlang_v2_sema::ResolvedCheckV2, String> {
    match document.document().item(document.check_item_id()) {
        Some(SemanticItemV2::Check(check)) => Ok(check),
        _ => Err("the nested-check fixture item is a resolved CHECK".to_owned()),
    }
}

/// The validated CHECK contract of the shared nested CHECK/FIND fixture.
pub(crate) fn check_contract(
    document: &ReferenceNestedCheckDocumentV2,
) -> Result<CheckContractV2, String> {
    let check = resolved_check(document)?;
    let nested = check
        .nested_graph()
        .map(GraphQueryContractV2::from_resolved)
        .transpose()
        .map_err(|error| format!("{error:?}"))?;
    CheckContractV2::from_resolved(check, nested).map_err(|error| format!("{error:?}"))
}

/// The shared nested CHECK/FIND fixture document plus its checked carrier.
pub(crate) fn check_input() -> Result<(ReferenceNestedCheckDocumentV2, SemanticPlanInputV2), String>
{
    let document = check_document()?;
    let contract = check_contract(&document)?;
    Ok((document, SemanticPlanInputV2::Check(contract)))
}

/// One sealed rule fixture document plus the document-wide rule program contract.
pub(crate) fn rule_input(
    fixture: ReferenceRuleFixtureV2,
) -> Result<(ReferenceRuleDocumentV2, SemanticPlanInputV2), String> {
    let document = reference_rule_document_v2(fixture)?;
    let contract = RuleProgramContractV2::from_document(document.document())
        .map_err(|error| format!("{error:?}"))?;
    Ok((document, SemanticPlanInputV2::Rule(contract)))
}

/// The checked location a sealed item is anchored on, mirroring the anchor the
/// planner attributes an item-level outcome to.
pub(crate) fn item_location(item: &SemanticItemV2) -> Option<CheckedLocatedSpanV2> {
    match item {
        SemanticItemV2::Relation(relation) => Some(relation.location().clone()),
        SemanticItemV2::Rule(rule) => Some(rule.body().expression().location().clone()),
        SemanticItemV2::Graph(graph) => Some(graph.location().clone()),
        SemanticItemV2::Check(check) => Some(check.location().clone()),
        SemanticItemV2::Presentation(presentation) => Some(presentation.location().clone()),
        SemanticItemV2::Catalog(catalog) => {
            catalog.inline_view().map(|view| view.location().clone())
        }
        SemanticItemV2::Declaration(declaration) => Some(declaration.body().location().clone()),
        SemanticItemV2::Composition(composition) => Some(composition.location().clone()),
        SemanticItemV2::Meta(meta) => Some(meta.location().clone()),
    }
}

/// The checked location of one document item.
pub(crate) fn document_item_location(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
) -> Option<CheckedLocatedSpanV2> {
    document.item(id).and_then(item_location)
}

/// The visited checked algebra node count and folded effect summary of a plan
/// that must carry the relational policy.
pub(crate) fn relational_policy(planned: &SemanticPlanV2) -> Option<(u32, EffectSummaryV2)> {
    match planned.policy() {
        PlanPolicyV2::Relational { nodes, effects } => Some((*nodes, *effects)),
        PlanPolicyV2::Graph(_)
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::Rule(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_) => None,
    }
}

/// The retained graph witness provenance of a plan that must carry the graph
/// policy.
pub(crate) fn graph_policy(planned: &SemanticPlanV2) -> Option<&GraphWitnessRetentionV2> {
    match planned.policy() {
        PlanPolicyV2::Graph(retention) => Some(retention),
        PlanPolicyV2::Relational { .. }
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::Rule(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_) => None,
    }
}

/// The retained CHECK policy of a plan that must carry the CHECK policy.
pub(crate) fn check_policy(planned: &SemanticPlanV2) -> Option<&CheckPolicyV2> {
    match planned.policy() {
        PlanPolicyV2::Check(policy) => Some(policy),
        PlanPolicyV2::Relational { .. }
        | PlanPolicyV2::Graph(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::Rule(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_) => None,
    }
}

/// The retained rule-program summary of a plan that must carry the rule policy.
pub(crate) fn rule_policy(planned: &SemanticPlanV2) -> Option<&RuleProgramPolicyV2> {
    match planned.policy() {
        PlanPolicyV2::Rule(policy) => Some(policy),
        PlanPolicyV2::Relational { .. }
        | PlanPolicyV2::Graph(_)
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_) => None,
    }
}

/// The deepest single-child node of a checked algebra tree: the node a
/// charge-before-visit traversal refuses last for a linear relation chain.
pub(crate) fn deepest_node(
    root: &quanta_gqlang_v2_algebra::CheckedAlgebraNodeV2,
) -> &quanta_gqlang_v2_algebra::CheckedAlgebraNodeV2 {
    let mut node = root;
    loop {
        let children = node.children();
        match children.first() {
            Some(child) if children.len() == 1 => node = child,
            Some(_) | None => return node,
        }
    }
}
