//! The ONE iterative, deterministic, budgeted semantic traversal (ticket 07,
//! RR-07B item 3).
//!
//! The V1 recursive walks (`capability::collect`, `explain::collect_families`,
//! `plan::classify_node`) are deleted. Every derived fact a plan needs —
//! capabilities, the ordered algebra family list, the coarse cardinality class,
//! and the folded expression-effect summary — now comes from this single
//! traversal over the sealed inputs:
//!
//! - it is ITERATIVE: an explicit frame stack, so a 50k-node chain cannot
//!   exhaust the process stack;
//! - it is DETERMINISTIC: frames are pushed in reverse so nodes are visited
//!   root-first and left-before-right, and the composition root precedes its
//!   ordered producers, which precede its body;
//! - it is BUDGETED: every visited node charges exactly one
//!   `max_semantic_steps` step BEFORE it is visited, against the document's
//!   immutable receipt (there is no planner-local budget). The exact sealed limit
//!   succeeds and limit+1 rejects at the refused node's checked location with
//!   [`PlanningDiagnosticV1::SemanticStepBudgetExceeded`].

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{AlgebraFamilyV2, CheckedAlgebraNodeV2};
use quanta_gqlang_v2_logic::{GraphQueryContractV2, RelationShapeV1};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, LocatedSpanV2, ResolvedRuleV2, SemanticItemV2,
    SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::ExpressionEffectsV2;

use crate::capability::{capability_for_family, CapabilityV2};
use crate::composition::CompositionBodyPlanInputV2;
use crate::cost::CardinalityClassV2;
use crate::diagnostic::PlanningDiagnosticV1;
use crate::item::SemanticPlanInputV2;
use crate::policy::EffectSummaryV2;

/// Everything one budgeted traversal derives from a sealed input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TraversalV2 {
    /// Capability requirements implied by the visited sealed constructs.
    pub(crate) capabilities: BTreeSet<CapabilityV2>,
    /// Checked algebra families, root-first in stable pre-order. Empty for the
    /// non-relational arms, which have no algebra tree.
    pub(crate) families: Vec<AlgebraFamilyV2>,
    /// The coarse cardinality class of the planned result.
    pub(crate) cardinality_class: CardinalityClassV2,
    /// The folded expression-effect projection of every retained effect visited.
    pub(crate) effects: EffectSummaryV2,
    /// How many nodes were charged and visited.
    pub(crate) nodes: u32,
    /// Whether a visited SEALED GRAPH is over a cyclic relation (the rule-program
    /// arm contributes recursion separately, from its stratification proof).
    pub(crate) graph_recursive: bool,
    /// Whether any visited algebra node is an `Aggregate` carrying an
    /// aggregate-placed expression effect.
    pub(crate) aggregation: bool,
}

/// One pending traversal unit. `Enter`/`Exit` pairs implement the post-order
/// cardinality fold without recursion.
enum FrameV2<'a> {
    /// One sealed input arm.
    Input(&'a SemanticPlanInputV2),
    /// One sealed graph contract (a top-level FIND or a CHECK's nested confirm).
    Graph(&'a GraphQueryContractV2),
    /// One document rule member.
    RuleMember(&'a ResolvedRuleV2),
    /// One composition body input.
    Body(&'a CompositionBodyPlanInputV2),
    /// A checked algebra node about to be visited.
    AlgebraEnter(&'a CheckedAlgebraNodeV2),
    /// A checked algebra node whose children are all visited.
    AlgebraExit(&'a CheckedAlgebraNodeV2),
}

/// Charge-before-visit semantic-step accounting against the document receipt.
struct StepChargeV2 {
    limit: u32,
    used: u32,
}

impl StepChargeV2 {
    const fn new(limit: u32) -> Self {
        Self { limit, used: 0 }
    }

    /// Charge one step for a node, or refuse at the exact sealed limit.
    fn charge(&mut self, span: &CheckedLocatedSpanV2) -> Result<(), PlanningDiagnosticV1> {
        if self.used == self.limit {
            return Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded {
                limit: self.limit,
                span: span.clone(),
            });
        }
        self.used = self.used.saturating_add(1);
        Ok(())
    }
}

/// Walk one sealed planning input, charging the document's `max_semantic_steps`
/// once per visited node.
///
/// # Errors
///
/// Returns [`PlanningDiagnosticV1::SemanticStepBudgetExceeded`] at the first
/// node whose charge would exceed the sealed limit.
pub(crate) fn traverse(
    input: &SemanticPlanInputV2,
    budget: DownstreamBudgetReceiptV2,
) -> Result<TraversalV2, PlanningDiagnosticV1> {
    let mut charge = StepChargeV2::new(budget.max_semantic_steps());
    let mut capabilities = BTreeSet::new();
    let mut families = Vec::new();
    let mut effects = ExpressionEffectsV2::pure_scalar();
    let mut nodes = 0_u32;
    let mut graph_recursive = false;
    let mut aggregation = false;
    let mut plan_class = None;
    let mut algebra_depth = 0_u32;
    let mut values: Vec<CardinalityClassV2> = Vec::new();
    let mut stack: Vec<FrameV2<'_>> = vec![FrameV2::Input(input)];

    while let Some(frame) = stack.pop() {
        match frame {
            FrameV2::Input(arm) => match arm {
                SemanticPlanInputV2::Relational(item) => {
                    push_algebra(&mut stack, item.root());
                }
                SemanticPlanInputV2::Graph(contract) => {
                    stack.push(FrameV2::Graph(contract));
                }
                SemanticPlanInputV2::Check(contract) => {
                    charge.charge(contract.location())?;
                    nodes = nodes.saturating_add(1);
                    if let Some(nested) = contract.nested_graph() {
                        stack.push(FrameV2::Graph(nested));
                    }
                    // A CHECK alerts over the rule it evaluates: its row count is
                    // bounded by the nested confirm's closure only when that
                    // closure recurses.
                    plan_class = Some(match contract.nested_graph() {
                        Some(nested) if nested.shape() == RelationShapeV1::Cyclic => {
                            CardinalityClassV2::Closure
                        }
                        Some(_) | None => CardinalityClassV2::LinearInInput,
                    });
                }
                SemanticPlanInputV2::Presentation(_contract) => {
                    // The presentation item itself is the only visited node; its
                    // sealed payload is read by the plan projection.
                    let location = presentation_anchor(arm);
                    charge.charge(location)?;
                    nodes = nodes.saturating_add(1);
                    plan_class = Some(presentation_cardinality(arm));
                }
                SemanticPlanInputV2::Rule(contract) => {
                    if let Some(first) = contract.rules().first() {
                        charge.charge(rule_location(first))?;
                        nodes = nodes.saturating_add(1);
                    }
                    for rule in contract.rules().iter().rev() {
                        stack.push(FrameV2::RuleMember(rule));
                    }
                }
                SemanticPlanInputV2::MetaRequireCertified(meta) => {
                    charge.charge(meta.location())?;
                    nodes = nodes.saturating_add(1);
                    plan_class = Some(CardinalityClassV2::LinearInInput);
                }
                SemanticPlanInputV2::Composition(composition) => {
                    charge.charge(composition.location())?;
                    nodes = nodes.saturating_add(1);
                    // Ordered producers are visited root-first, in authored LET
                    // order, BEFORE the body; none of them is a plan of its own.
                    for producer in composition.producers() {
                        charge.charge(producer.name_location())?;
                        nodes = nodes.saturating_add(1);
                    }
                    stack.push(FrameV2::Body(composition.body()));
                }
            },
            FrameV2::Graph(contract) => {
                charge.charge(contract.location())?;
                nodes = nodes.saturating_add(1);
                capabilities.insert(CapabilityV2::WitnessMaterialization);
                if contract.shape() == RelationShapeV1::Cyclic {
                    capabilities.insert(CapabilityV2::RecursiveClosure);
                    graph_recursive = true;
                }
                if !stack_has_algebra(&stack) {
                    plan_class = Some(if contract.shape() == RelationShapeV1::Cyclic {
                        CardinalityClassV2::Closure
                    } else {
                        CardinalityClassV2::LinearInInput
                    });
                }
            }
            FrameV2::RuleMember(rule) => {
                charge.charge(rule_location(rule))?;
                nodes = nodes.saturating_add(1);
                effects = effects.join(rule.head_term().effects());
                effects = effects.join(rule.body().expression().effects());
            }
            FrameV2::Body(body) => match body {
                CompositionBodyPlanInputV2::Relational { root, .. } => {
                    push_algebra(&mut stack, root);
                }
                CompositionBodyPlanInputV2::Graph(contract) => {
                    stack.push(FrameV2::Graph(contract));
                }
                CompositionBodyPlanInputV2::Check(contract) => {
                    charge.charge(contract.location())?;
                    nodes = nodes.saturating_add(1);
                    if let Some(nested) = contract.nested_graph() {
                        stack.push(FrameV2::Graph(nested));
                    }
                    plan_class = Some(match contract.nested_graph() {
                        Some(nested) if nested.shape() == RelationShapeV1::Cyclic => {
                            CardinalityClassV2::Closure
                        }
                        Some(_) | None => CardinalityClassV2::LinearInInput,
                    });
                }
                CompositionBodyPlanInputV2::Explain(contract) => {
                    charge.charge(contract.location())?;
                    nodes = nodes.saturating_add(1);
                    plan_class = Some(CardinalityClassV2::LinearInInput);
                }
            },
            FrameV2::AlgebraEnter(node) => {
                charge.charge(node.location())?;
                nodes = nodes.saturating_add(1);
                algebra_depth = algebra_depth.saturating_add(1);
                let family = node.family();
                families.push(family);
                if let Some(capability) = capability_for_family(family) {
                    capabilities.insert(capability);
                }
                if family == AlgebraFamilyV2::Aggregate {
                    aggregation = true;
                }
                effects = effects.join(node.output().effects());
                stack.push(FrameV2::AlgebraExit(node));
                let children = node.children();
                for child in children.iter().rev() {
                    stack.push(FrameV2::AlgebraEnter(child));
                }
            }
            FrameV2::AlgebraExit(node) => {
                let class = exit_class(node, &mut values);
                algebra_depth = algebra_depth.saturating_sub(1);
                if algebra_depth == 0 {
                    plan_class = Some(class);
                }
                values.push(class);
            }
        }
    }

    Ok(TraversalV2 {
        capabilities,
        families,
        cardinality_class: plan_class.unwrap_or(CardinalityClassV2::LinearInInput),
        effects: EffectSummaryV2::new(
            effects.purity,
            effects.determinism,
            effects.monotonicity,
            effects.aggregation,
        ),
        nodes,
        graph_recursive,
        aggregation,
    })
}

/// Push an algebra walk's enter frame (children are discovered on entry).
fn push_algebra<'a>(stack: &mut Vec<FrameV2<'a>>, root: &'a CheckedAlgebraNodeV2) {
    stack.push(FrameV2::AlgebraEnter(root));
}

/// Whether the pending stack still holds an algebra frame, which means a nested
/// graph is being visited inside a relational plan rather than as the plan root.
fn stack_has_algebra(stack: &[FrameV2<'_>]) -> bool {
    stack
        .iter()
        .any(|frame| matches!(frame, FrameV2::AlgebraEnter(_) | FrameV2::AlgebraExit(_)))
}

/// The coarse cardinality class of one algebra node, folded from its children's
/// already-computed classes.
fn exit_class(
    node: &CheckedAlgebraNodeV2,
    values: &mut Vec<CardinalityClassV2>,
) -> CardinalityClassV2 {
    let family = node.family();
    match family {
        AlgebraFamilyV2::Scan => CardinalityClassV2::LinearInInput,
        AlgebraFamilyV2::Limit => CardinalityClassV2::Bounded,
        AlgebraFamilyV2::Join | AlgebraFamilyV2::Apply => CardinalityClassV2::Multiplicative,
        AlgebraFamilyV2::Aggregate => {
            // An aggregate's output cardinality is governed by its GROUPING
            // partition, not by the size of the input it folds: a GROUPED
            // aggregate emits one row per distinct key, while a SCALAR aggregate
            // has no partition keys and folds the whole input into one row. The
            // fold reads the ABSENCE of a non-empty partition from the sealed
            // operator rather than asserting the empty-keyed `Group` structure.
            let grouped = matches!(
                node.children().first(),
                Some(input)
                    if matches!(
                        input.family(),
                        AlgebraFamilyV2::Group
                    ) && !input.expressions().is_empty()
            );
            let child = values.pop().unwrap_or(CardinalityClassV2::LinearInInput);
            if grouped {
                return child;
            }
            CardinalityClassV2::ScalarOrSingleton
        }
        AlgebraFamilyV2::Group
        | AlgebraFamilyV2::Project
        | AlgebraFamilyV2::Filter
        | AlgebraFamilyV2::Sort
        | AlgebraFamilyV2::SemiJoin
        | AlgebraFamilyV2::AntiJoin => values.pop().unwrap_or(CardinalityClassV2::LinearInInput),
        AlgebraFamilyV2::Union => {
            let right = values.pop().unwrap_or(CardinalityClassV2::LinearInInput);
            let left = values.pop().unwrap_or(CardinalityClassV2::LinearInInput);
            if left.rank() >= right.rank() {
                left
            } else {
                right
            }
        }
    }
}

/// The checked location a presentation diagnostic or plan anchors on.
fn presentation_anchor(arm: &SemanticPlanInputV2) -> &CheckedLocatedSpanV2 {
    match arm {
        SemanticPlanInputV2::Presentation(contract) => contract.location(),
        SemanticPlanInputV2::Relational(item) => item.root().location(),
        SemanticPlanInputV2::Graph(contract) => contract.location(),
        SemanticPlanInputV2::Check(contract) => contract.location(),
        SemanticPlanInputV2::Rule(program) => program
            .rules()
            .first()
            .map_or_else(|| program_budget_anchor(program), rule_location),
        SemanticPlanInputV2::MetaRequireCertified(meta) => meta.location(),
        SemanticPlanInputV2::Composition(composition) => composition.location(),
    }
}

/// A rule program with no member has no rule location; Sema's document facts are
/// still checked, so planning rejects that input before it reaches a projection.
fn program_budget_anchor(
    program: &quanta_gqlang_v2_logic::RuleProgramContractV2,
) -> &'static CheckedLocatedSpanV2 {
    let _ = program;
    unreachable_program_anchor()
}

/// The rule's own checked body location.
fn rule_location(rule: &ResolvedRuleV2) -> &CheckedLocatedSpanV2 {
    rule.body().expression().location()
}

/// No reference implementation can build a located span from nothing, so an
/// empty rule program is rejected by the plan entry before any projection runs.
fn unreachable_program_anchor() -> &'static CheckedLocatedSpanV2 {
    static ANCHOR: std::sync::OnceLock<CheckedLocatedSpanV2> = std::sync::OnceLock::new();
    ANCHOR.get_or_init(|| {
        // A registry-proven point span from a private synthetic registry: no
        // test-only fixture and no hir edge participate in production code.
        let source_id = SourceIdV2::from_raw(u32::MAX - 1);
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id,
            source_len: 1,
        }])
        .unwrap_or_else(|_error| unreachable!("point registry is well formed"));
        registry
            .try_check_location(LocatedSpanV2 {
                source_id,
                span: SpanV1::point(0),
            })
            .unwrap_or_else(|_error| unreachable!("point span is inside its registry"))
    })
}

/// The cardinality class a presentation's result carries.
fn presentation_cardinality(arm: &SemanticPlanInputV2) -> CardinalityClassV2 {
    let SemanticPlanInputV2::Presentation(contract) = arm else {
        return CardinalityClassV2::LinearInInput;
    };
    match contract.kind() {
        quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Sample { .. } => {
            CardinalityClassV2::Bounded
        }
        quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Explain { .. }
        | quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Show { .. }
        | quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Describe { .. }
        | quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Compare { .. }
        | quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Export { .. } => {
            CardinalityClassV2::LinearInInput
        }
    }
}
