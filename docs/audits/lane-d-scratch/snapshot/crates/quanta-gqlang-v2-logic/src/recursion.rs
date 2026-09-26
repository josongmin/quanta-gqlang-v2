//! Document-wide rule programs, iterative SCC analysis, and stratification.
//!
//! SS-06B (ticket 06) makes Logic the sole consumer of the sealed semantic
//! document's resolved rule set. This module therefore works on stable
//! [`ResolvedPredicateIdV2`] vertices and [`CheckedLocatedSpanV2`] facts only:
//! there is no raw-HIR ingest, no name-keyed identity, no caller-assembled
//! ingress shortcut, and no spelling-based legality. Exactly one document-wide
//! program is stratified by one algorithm — [`stratify`].
//!
//! # Legality pipeline
//!
//! 1. Project every rule into one canonical dependency graph. A rule head is a
//!    vertex; every resolved body reference is an edge carrying its polarity and
//!    checked location. A body target that is not a member predicate of this
//!    program fails closed with
//!    [`LogicDiagnosticV1::RuleProgramInvalid`]` { reason: NonLocalTarget }` and
//!    never becomes a vertex (`SS-06B` step 4).
//! 2. Compute strongly-connected components with an explicit frame stack, so an
//!    admitted 50k-node chain cannot exhaust the process stack.
//! 3. Select the canonical offending edge for every negated or aggregated
//!    intra-SCC edge, prove an actual return path inside that component, and
//!    reject: a negated recursion is a [`LogicDiagnosticV1::NegationCycle`], an
//!    aggregated recursion is a [`LogicDiagnosticV1::RecursiveAggregate`] under
//!    the fixed reject-all policy. Negation takes precedence over aggregation
//!    (`G12` amendment), so the negated case can never be reported as
//!    non-monotone recursion.
//! 4. Relax the SCC condensation to a stratum fixpoint. Positive edges are
//!    non-decreasing; negated and aggregated edges strictly raise the source
//!    stratum. Positive recursion therefore admits with an explicit
//!    least-fixpoint proof.
//!
//! # Exact budget accounting
//!
//! Raw public programs are admitted against independent rule-count and body-
//! literal limits before any member-map allocation. Resource refusal wins for
//! an over-limit raw program; within both caps, non-local targets still reject
//! before any graph vertex or edge charge.
//!
//! - `max_rule_program_rules`: one preflight admission for the complete raw
//!   rule count, before building the member set. It is independent of the
//!   number of distinct graph vertices.
//! - `max_rule_graph_edges`: one preflight admission for each raw body literal,
//!   before scanning target membership. Over-limit input is refused at the
//!   first literal beyond the cap, even if that literal is non-local.
//!
//! Every graph fact and traversal step is charged BEFORE the work from the
//! document's immutable [`DownstreamBudgetReceiptV2`]. The charge that would
//! exceed a dimension rejects with
//! [`LogicDiagnosticV1::RuleGraphBudgetExceeded`] at the refused step's checked
//! location; the exact sealed limit always succeeds.
//!
//! - `max_rule_graph_nodes`: one charge per distinct member predicate, in
//!   ascending resolved-predicate-identity order.
//! - `max_rule_graph_edges`: one charge per projected call edge, in canonical
//!   `(source, target, polarity, literal location, rule location)` order.
//!   Duplicate facts each charge; no edge is silently dropped.
//! - `max_rule_steps`:
//!   1. one per SCC frame push (exactly one per vertex: roots in the outer
//!      loop, children when first discovered);
//!   2. one per SCC emission (one per component);
//!   3. one per projected call edge examined by the canonical offending-edge
//!      scan (exactly one per edge overall);
//!   4. one per call edge examined by the canonical return-path search;
//!   5. one per call edge examined in every stratum-relaxation pass.
//!
//! # Canonical witnesses
//!
//! Every order that can affect a rendered result is canonical: vertices ascend
//! by resolved predicate identity, adjacency is sorted by
//! `(target, polarity, literal location, rule location)`, components are sorted
//! by member identity, and the offending edge is the minimum candidate under
//! the same key. A rendered [`LogicDiagnosticV1::NegationCycle`] therefore
//! reports the same cycle, offending edge, and rule location under any
//! permutation of rule or body-literal insertion order.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, DownstreamBudgetReceiptV2};
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

use crate::aggregate::RejectAllRecursiveAggregatesV2;
use crate::diagnostic::{
    LogicDiagnosticV1, RuleGraphBudgetDimensionV1, RuleProgramInvalidReasonV1,
};

/// How a body literal references its predicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyPolarityV1 {
    /// A positive (non-negated, non-aggregated) reference. Monotone.
    Positive,
    /// A negated reference (`not p(...)`). Non-monotone across recursion.
    Negated,
    /// An aggregated reference (`count(p(...))` etc.). Non-monotone across
    /// recursion; every call in this packet rejects over a recursive SCC.
    Aggregated,
}

impl BodyPolarityV1 {
    /// Canonical ordering rank. Ordering must not depend on enum declaration
    /// order, so every comparison uses this explicit rank.
    const fn rank(self) -> u8 {
        match self {
            Self::Positive => 0,
            Self::Negated => 1,
            Self::Aggregated => 2,
        }
    }
}

/// One resolved predicate reference inside a rule body.
///
/// The reference is a stable types-owned [`ResolvedPredicateIdV2`] plus a
/// registry-proven [`CheckedLocatedSpanV2`]; Logic never re-resolves a call
/// name, arity, or kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyLiteralV1 {
    predicate: ResolvedPredicateIdV2,
    polarity: BodyPolarityV1,
    aggregate_function: Option<String>,
    location: CheckedLocatedSpanV2,
}

impl BodyLiteralV1 {
    /// Project one resolved body reference.
    ///
    /// An [`BodyPolarityV1::Aggregated`] reference projected here carries no
    /// aggregate-function spelling; the reject-all policy then renders
    /// [`RejectAllRecursiveAggregatesV2::UNSPECIFIED_FUNCTION`]. Use
    /// [`Self::aggregated`] when the spelling as written is known.
    #[must_use]
    pub const fn new(
        predicate: ResolvedPredicateIdV2,
        polarity: BodyPolarityV1,
        location: CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            predicate,
            polarity,
            aggregate_function: None,
            location,
        }
    }

    /// Project one resolved aggregated body reference together with the
    /// aggregate function name as written.
    #[must_use]
    pub fn aggregated(
        predicate: ResolvedPredicateIdV2,
        function: impl Into<String>,
        location: CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            predicate,
            polarity: BodyPolarityV1::Aggregated,
            aggregate_function: Some(function.into()),
            location,
        }
    }

    /// The referenced predicate identity.
    #[must_use]
    pub const fn predicate(&self) -> &ResolvedPredicateIdV2 {
        &self.predicate
    }

    /// How the reference participates: positive, negated, or aggregated.
    #[must_use]
    pub const fn polarity(&self) -> BodyPolarityV1 {
        self.polarity
    }

    /// The aggregate function spelling as written, when this reference was
    /// projected with one.
    #[must_use]
    pub fn aggregate_function(&self) -> Option<&str> {
        self.aggregate_function.as_deref()
    }

    /// The registry-proven checked location of the reference.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// One rule definition: the head predicate plus its resolved body references.
///
/// The projection from the sealed document is owned by the semantic adapter;
/// this carrier is structural only and makes no legality decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDefV1 {
    head: ResolvedPredicateIdV2,
    body: Vec<BodyLiteralV1>,
    location: CheckedLocatedSpanV2,
}

impl RuleDefV1 {
    /// Project one rule from its resolved parts.
    #[must_use]
    pub const fn new(
        head: ResolvedPredicateIdV2,
        body: Vec<BodyLiteralV1>,
        location: CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            head,
            body,
            location,
        }
    }

    /// The predicate this rule (partially) defines.
    #[must_use]
    pub const fn head(&self) -> &ResolvedPredicateIdV2 {
        &self.head
    }

    /// The resolved body references, in authored order.
    #[must_use]
    pub fn body(&self) -> &[BodyLiteralV1] {
        &self.body
    }

    /// The registry-proven checked location of the rule.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// A least-fixpoint stratification proof for an admitted recursive program.
///
/// This is the EXPLICIT artifact `D27` requires: it records, per predicate, the
/// stratum (evaluation layer) it lands in, which predicates are recursive, and
/// the canonical strongly-connected decomposition that was proven. Positive
/// recursion is admitted precisely because such an assignment exists — every
/// negated/aggregated dependency crosses to a strictly lower stratum, and each
/// recursive SCC is evaluated by least-fixpoint within its own stratum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StratificationProofV1 {
    strata: BTreeMap<ResolvedPredicateIdV2, usize>,
    recursive: BTreeSet<ResolvedPredicateIdV2>,
    components: Vec<Vec<ResolvedPredicateIdV2>>,
    stratum_count: usize,
}

impl StratificationProofV1 {
    /// The stratum a predicate was assigned, if it is part of this program.
    #[must_use]
    pub fn stratum_of(&self, predicate: &ResolvedPredicateIdV2) -> Option<usize> {
        self.strata.get(predicate).copied()
    }

    /// Whether a predicate participates in recursion (non-trivial SCC or
    /// self-loop) under this program.
    #[must_use]
    pub fn is_recursive(&self, predicate: &ResolvedPredicateIdV2) -> bool {
        self.recursive.contains(predicate)
    }

    /// The exact stratum assignment, keyed by stable predicate identity.
    #[must_use]
    pub const fn strata(&self) -> &BTreeMap<ResolvedPredicateIdV2, usize> {
        &self.strata
    }

    /// The recursive predicates, in stable predicate-identity order.
    #[must_use]
    pub const fn recursive(&self) -> &BTreeSet<ResolvedPredicateIdV2> {
        &self.recursive
    }

    /// The canonical strongly-connected decomposition: every component lists its
    /// members in ascending predicate identity, and components are ordered by
    /// their member lists. The decomposition is invariant under rule and
    /// body-literal insertion permutation.
    #[must_use]
    pub fn components(&self) -> &[Vec<ResolvedPredicateIdV2>] {
        &self.components
    }

    /// The number of strata (the highest stratum index + 1, or 0 when empty).
    #[must_use]
    pub const fn stratum_count(&self) -> usize {
        self.stratum_count
    }
}

/// One document-wide recursive logic program.
///
/// Construction is structural only; legality is computed by [`stratify`], which
/// either returns a [`StratificationProofV1`] (admission, with explicit
/// least-fixpoint semantics for any recursive SCC) or a typed
/// [`LogicDiagnosticV1`] (rejection). The semantic adapter owns the projection
/// from the sealed document's complete rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursiveProgramV1 {
    rules: Vec<RuleDefV1>,
}

impl RecursiveProgramV1 {
    /// A program from the complete projected rule set.
    #[must_use]
    pub const fn from_rules(rules: Vec<RuleDefV1>) -> Self {
        Self { rules }
    }

    /// The rule definitions in this program, in projected order.
    #[must_use]
    pub fn rules(&self) -> &[RuleDefV1] {
        &self.rules
    }
}

/// Compute the least-fixpoint stratification proof for one document-wide rule
/// program, or reject with a typed diagnostic.
///
/// Admits positive recursion with an explicit per-predicate stratum assignment
/// and canonical SCC decomposition; rejects negation cycles, aggregated
/// recursion, non-local targets, and an exhausted rule-graph budget. Every
/// charge is taken from `budget` before the corresponding work; there is no
/// caller-selected replacement, default, or environment dimension.
pub fn stratify(
    program: &RecursiveProgramV1,
    budget: &DownstreamBudgetReceiptV2,
) -> Result<StratificationProofV1, LogicDiagnosticV1> {
    if let Some(rule) = program
        .rules()
        .get(usize::try_from(budget.max_rule_program_rules()).unwrap_or(usize::MAX))
    {
        return Err(LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension: RuleGraphBudgetDimensionV1::Rules,
            limit: budget.max_rule_program_rules(),
            span: rule.location().clone(),
        });
    }
    let mut body_literals = 0_u32;
    for rule in program.rules() {
        for literal in rule.body() {
            if body_literals >= budget.max_rule_graph_edges() {
                return Err(LogicDiagnosticV1::RuleGraphBudgetExceeded {
                    dimension: RuleGraphBudgetDimensionV1::Edges,
                    limit: budget.max_rule_graph_edges(),
                    span: literal.location().clone(),
                });
            }
            body_literals += 1;
        }
    }
    let mut charge = RuleGraphChargeV1::new(budget);
    let graph = RuleGraphV1::build(program, &mut charge)?;
    let components = TarjanStateV1::run(&graph, &mut charge)?;
    let (negated, aggregated) = scan_offending_edges(&graph, &components, &mut charge)?;

    if let Some(offending) = negated {
        let members = component_members(&components, offending.component);
        let path = canonical_return_path(&graph, &members, &offending, &mut charge)?;
        return Err(LogicDiagnosticV1::NegationCycle {
            cycle: cycle_identities(&graph, &offending, &path)?,
            offending_edge: offending.literal_location,
            span: offending.rule_location,
        });
    }

    if let Some(offending) = aggregated {
        let members = component_members(&components, offending.component);
        // The return path is discharged for aggregate recursion too: the
        // offending edge must provably close a real cycle before the rejection
        // is rendered. The rendered payload carries the aggregate call.
        canonical_return_path(&graph, &members, &offending, &mut charge)?;
        let Some(predicate) = graph.node(offending.target) else {
            return Err(internal_membership_error(&offending.rule_location));
        };
        let rejection = RejectAllRecursiveAggregatesV2::new().check(
            offending
                .aggregate_function
                .as_deref()
                .unwrap_or(RejectAllRecursiveAggregatesV2::UNSPECIFIED_FUNCTION),
            predicate,
            &offending.literal_location,
        );
        return match rejection {
            Err(diagnostic) => Err(diagnostic),
            // The fixed policy has no admitted arm. If that ever changed, this
            // fails closed instead of admitting an unproven recursion.
            Ok(()) => Err(internal_membership_error(&offending.rule_location)),
        };
    }

    let strata = assign_strata(&graph, &components, &mut charge)?;
    let recursive = components
        .iter()
        .filter(|component| is_recursive_component(&graph, component))
        .flatten()
        .filter_map(|member| graph.node(*member).cloned())
        .collect::<BTreeSet<ResolvedPredicateIdV2>>();
    let canonical_components = components
        .iter()
        .map(|component| {
            component
                .iter()
                .filter_map(|member| graph.node(*member).cloned())
                .collect::<Vec<ResolvedPredicateIdV2>>()
        })
        .collect::<Vec<Vec<ResolvedPredicateIdV2>>>();
    let stratum_count = strata
        .values()
        .copied()
        .max()
        .map_or(0, |highest| highest.saturating_add(1));

    Ok(StratificationProofV1 {
        strata,
        recursive,
        components: canonical_components,
        stratum_count,
    })
}

/// Pre-work charging ledger over the immutable rule-graph budget dimensions.
///
/// The charge that would exceed the exact sealed limit rejects before the
/// corresponding allocation or traversal; the exact limit always succeeds.
struct RuleGraphChargeV1 {
    max_nodes: u32,
    max_edges: u32,
    max_steps: u32,
    charged_nodes: u32,
    charged_edges: u32,
    charged_steps: u32,
}

impl RuleGraphChargeV1 {
    const fn new(budget: &DownstreamBudgetReceiptV2) -> Self {
        Self {
            max_nodes: budget.max_rule_graph_nodes(),
            max_edges: budget.max_rule_graph_edges(),
            max_steps: budget.max_rule_steps(),
            charged_nodes: 0,
            charged_edges: 0,
            charged_steps: 0,
        }
    }

    fn charge_node(&mut self, span: &CheckedLocatedSpanV2) -> Result<(), LogicDiagnosticV1> {
        if self.charged_nodes >= self.max_nodes {
            return Err(self.exceeded(RuleGraphBudgetDimensionV1::Nodes, self.max_nodes, span));
        }
        self.charged_nodes = self.charged_nodes.saturating_add(1);
        Ok(())
    }

    fn charge_edge(&mut self, span: &CheckedLocatedSpanV2) -> Result<(), LogicDiagnosticV1> {
        if self.charged_edges >= self.max_edges {
            return Err(self.exceeded(RuleGraphBudgetDimensionV1::Edges, self.max_edges, span));
        }
        self.charged_edges = self.charged_edges.saturating_add(1);
        Ok(())
    }

    fn charge_step(&mut self, span: &CheckedLocatedSpanV2) -> Result<(), LogicDiagnosticV1> {
        if self.charged_steps >= self.max_steps {
            return Err(self.exceeded(RuleGraphBudgetDimensionV1::Steps, self.max_steps, span));
        }
        self.charged_steps = self.charged_steps.saturating_add(1);
        Ok(())
    }

    fn exceeded(
        &self,
        dimension: RuleGraphBudgetDimensionV1,
        limit: u32,
        span: &CheckedLocatedSpanV2,
    ) -> LogicDiagnosticV1 {
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension,
            limit,
            span: span.clone(),
        }
    }
}

/// Canonical ordering of one checked location. The registry proof itself is not
/// orderable, so ordering uses the exact located coordinates; two facts that
/// agree here render identically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LocationOrderV1 {
    source: u32,
    start: usize,
    end: usize,
}

fn location_order(location: &CheckedLocatedSpanV2) -> LocationOrderV1 {
    let located = location.location();
    LocationOrderV1 {
        source: located.source_id.raw(),
        start: located.span.start,
        end: located.span.end,
    }
}

/// Canonical ordering of one projected call edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeOrderV1 {
    source: usize,
    target: usize,
    polarity: u8,
    literal: LocationOrderV1,
    rule: LocationOrderV1,
}

fn edge_order(source: usize, edge: &RuleEdgeV1) -> EdgeOrderV1 {
    EdgeOrderV1 {
        source,
        target: edge.target,
        polarity: edge.polarity.rank(),
        literal: location_order(&edge.literal_location),
        rule: location_order(&edge.rule_location),
    }
}

/// One projected call edge, in canonical form.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RuleEdgeV1 {
    target: usize,
    polarity: BodyPolarityV1,
    aggregate_function: Option<String>,
    literal_location: CheckedLocatedSpanV2,
    rule_location: CheckedLocatedSpanV2,
}

/// The canonical predicate dependency graph.
struct RuleGraphV1 {
    /// Vertices in ascending resolved-predicate-identity order.
    nodes: Vec<ResolvedPredicateIdV2>,
    /// Canonical defining-rule location of each vertex, in vertex order.
    node_locations: Vec<CheckedLocatedSpanV2>,
    /// Canonically ordered adjacency, indexed by vertex index.
    adjacency: Vec<Vec<RuleEdgeV1>>,
}

impl RuleGraphV1 {
    /// Project the complete rule set into one canonical graph.
    ///
    /// Charges every vertex and edge before the corresponding allocation. A
    /// body target outside the member set fails closed with the canonical
    /// (minimum) non-local violation and is never inserted as a vertex.
    fn build(
        program: &RecursiveProgramV1,
        charge: &mut RuleGraphChargeV1,
    ) -> Result<Self, LogicDiagnosticV1> {
        // Pass 1: the exact member (vertex) set plus each member's canonical —
        // that is, minimum — defining-rule location.
        let mut members: BTreeMap<ResolvedPredicateIdV2, CheckedLocatedSpanV2> = BTreeMap::new();
        for rule in program.rules() {
            let location = rule.location();
            members
                .entry(rule.head().clone())
                .and_modify(|current| {
                    if location_order(location) < location_order(current) {
                        *current = location.clone();
                    }
                })
                .or_insert_with(|| location.clone());
        }

        // Pass 2: fail closed on non-local targets before charging any vertex.
        let mut nonlocal: Option<(
            ResolvedPredicateIdV2,
            CheckedLocatedSpanV2,
            CheckedLocatedSpanV2,
        )> = None;
        for rule in program.rules() {
            for literal in rule.body() {
                if members.contains_key(literal.predicate()) {
                    continue;
                }
                let candidate = (
                    literal.predicate().clone(),
                    literal.location().clone(),
                    rule.location().clone(),
                );
                let replace = nonlocal.as_ref().map_or(true, |current| {
                    nonlocal_order(&candidate) < nonlocal_order(current)
                });
                if replace {
                    nonlocal = Some(candidate);
                }
            }
        }
        if let Some((_, literal_location, _)) = nonlocal {
            return Err(LogicDiagnosticV1::RuleProgramInvalid {
                reason: RuleProgramInvalidReasonV1::NonLocalTarget,
                span: literal_location,
            });
        }

        // Vertices: one charge per distinct member, ascending identity.
        let mut nodes: Vec<ResolvedPredicateIdV2> = Vec::new();
        let mut node_locations: Vec<CheckedLocatedSpanV2> = Vec::new();
        let mut index_of: BTreeMap<ResolvedPredicateIdV2, usize> = BTreeMap::new();
        for (predicate, location) in &members {
            charge.charge_node(location)?;
            index_of.insert(predicate.clone(), nodes.len());
            nodes.push(predicate.clone());
            node_locations.push(location.clone());
        }

        // Edges: one charge per projected call edge, before the edge is stored.
        let mut edges: Vec<(usize, RuleEdgeV1)> = Vec::new();
        for rule in program.rules() {
            let Some(&source) = index_of.get(rule.head()) else {
                return Err(internal_membership_error(rule.location()));
            };
            for literal in rule.body() {
                if !members.contains_key(literal.predicate()) {
                    continue;
                }
                charge.charge_edge(literal.location())?;
                let Some(&target) = index_of.get(literal.predicate()) else {
                    return Err(internal_membership_error(literal.location()));
                };
                edges.push((
                    source,
                    RuleEdgeV1 {
                        target,
                        polarity: literal.polarity(),
                        aggregate_function: literal.aggregate_function().map(str::to_owned),
                        literal_location: literal.location().clone(),
                        rule_location: rule.location().clone(),
                    },
                ));
            }
        }

        edges.sort_by_key(|left| edge_order(left.0, &left.1));
        let mut adjacency: Vec<Vec<RuleEdgeV1>> = (0..nodes.len()).map(|_| Vec::new()).collect();
        for (source, edge) in edges {
            if let Some(bucket) = adjacency.get_mut(source) {
                bucket.push(edge);
            }
        }

        Ok(Self {
            nodes,
            node_locations,
            adjacency,
        })
    }

    /// The predicate identity of one vertex index.
    fn node(&self, node: usize) -> Option<&ResolvedPredicateIdV2> {
        self.nodes.get(node)
    }

    /// The canonically ordered outgoing edges of one vertex.
    fn bucket(&self, node: usize) -> &[RuleEdgeV1] {
        self.adjacency.get(node).map_or(&[], Vec::as_slice)
    }

    /// The number of outgoing edges of one vertex.
    fn edge_count(&self, node: usize) -> usize {
        self.bucket(node).len()
    }

    /// One outgoing edge by canonical position.
    fn edge(&self, node: usize, position: usize) -> Option<&RuleEdgeV1> {
        self.bucket(node).get(position)
    }

    /// Whether one vertex has a self-loop.
    fn has_self_loop(&self, node: usize) -> bool {
        self.bucket(node).iter().any(|edge| edge.target == node)
    }
}

/// Canonical ordering of one non-local-target violation.
fn nonlocal_order(
    violation: &(
        ResolvedPredicateIdV2,
        CheckedLocatedSpanV2,
        CheckedLocatedSpanV2,
    ),
) -> (&ResolvedPredicateIdV2, LocationOrderV1, LocationOrderV1) {
    (
        &violation.0,
        location_order(&violation.1),
        location_order(&violation.2),
    )
}

/// Fail-closed diagnostic for an internal membership disagreement.
fn internal_membership_error(span: &CheckedLocatedSpanV2) -> LogicDiagnosticV1 {
    LogicDiagnosticV1::RuleProgramInvalid {
        reason: RuleProgramInvalidReasonV1::MembershipMismatch,
        span: span.clone(),
    }
}

/// One Tarjan frame. The frame owns the checked location that introduced its
/// vertex, so SCC emission charges the exact authored location.
#[derive(Clone, Debug)]
struct TarjanFrameV1 {
    node: usize,
    next_edge: usize,
    location: CheckedLocatedSpanV2,
}

/// Iterative Tarjan working state, kept off the process stack.
struct TarjanStateV1 {
    index: Vec<Option<usize>>,
    lowlink: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    frames: Vec<TarjanFrameV1>,
    components: Vec<Vec<usize>>,
    next_index: usize,
}

impl TarjanStateV1 {
    fn new(vertices: usize) -> Self {
        Self {
            index: (0..vertices).map(|_| None).collect(),
            lowlink: (0..vertices).map(|_| 0).collect(),
            on_stack: (0..vertices).map(|_| false).collect(),
            stack: Vec::new(),
            frames: Vec::new(),
            components: Vec::new(),
            next_index: 0,
        }
    }

    fn index_of(&self, node: usize) -> Option<usize> {
        self.index.get(node).copied().flatten()
    }

    fn lowlink_of(&self, node: usize) -> usize {
        self.lowlink.get(node).copied().unwrap_or(0)
    }

    fn is_on_stack(&self, node: usize) -> bool {
        self.on_stack.get(node).copied().unwrap_or(false)
    }

    fn set_on_stack(&mut self, node: usize, value: bool) {
        if let Some(slot) = self.on_stack.get_mut(node) {
            *slot = value;
        }
    }

    fn set_lowlink_min(&mut self, node: usize, value: usize) {
        if let Some(slot) = self.lowlink.get_mut(node) {
            *slot = (*slot).min(value);
        }
    }

    /// Discover one vertex: assign its index/lowlink, push it on the SCC stack,
    /// and open its frame.
    fn discover(&mut self, node: usize, location: CheckedLocatedSpanV2) {
        let index = self.next_index;
        self.next_index = self.next_index.saturating_add(1);
        if let Some(slot) = self.index.get_mut(node) {
            *slot = Some(index);
        }
        if let Some(slot) = self.lowlink.get_mut(node) {
            *slot = index;
        }
        self.stack.push(node);
        self.set_on_stack(node, true);
        self.frames.push(TarjanFrameV1 {
            node,
            next_edge: 0,
            location,
        });
    }

    /// Tarjan's strongly-connected components over an explicit frame stack.
    ///
    /// Charging: one step per frame push (exactly one per vertex) and one per
    /// SCC emission. Components are returned in canonical form — members
    /// ascending by vertex index (which is ascending predicate identity) and
    /// components ordered by their member lists — so the result is invariant
    /// under rule/edge insertion permutation.
    fn run(
        graph: &RuleGraphV1,
        charge: &mut RuleGraphChargeV1,
    ) -> Result<Vec<Vec<usize>>, LogicDiagnosticV1> {
        let mut state = Self::new(graph.nodes.len());
        // `build` stores exactly one defining-rule location per vertex, so the
        // root frame push always has an exact authored location to charge at.
        for (root, root_location) in graph.node_locations.iter().enumerate() {
            if state.index_of(root).is_some() {
                continue;
            }
            charge.charge_step(root_location)?;
            state.discover(root, root_location.clone());

            while !state.frames.is_empty() {
                let Some(frame) = state.frames.last() else {
                    break;
                };
                let node = frame.node;
                let next_edge = frame.next_edge;
                if next_edge < graph.edge_count(node) {
                    if let Some(top) = state.frames.last_mut() {
                        top.next_edge = top.next_edge.saturating_add(1);
                    }
                    let Some(edge) = graph.edge(node, next_edge) else {
                        continue;
                    };
                    if state.index_of(edge.target).is_none() {
                        charge.charge_step(&edge.literal_location)?;
                        state.discover(edge.target, edge.literal_location.clone());
                    } else if state.is_on_stack(edge.target) {
                        let target_index = state.index_of(edge.target).unwrap_or(0);
                        state.set_lowlink_min(node, target_index);
                    }
                } else {
                    let Some(finished) = state.frames.pop() else {
                        break;
                    };
                    if let Some(parent) = state.frames.last() {
                        let lowlink = state.lowlink_of(finished.node);
                        state.set_lowlink_min(parent.node, lowlink);
                    }
                    if state.lowlink_of(finished.node) == state.index_of(finished.node).unwrap_or(0)
                    {
                        charge.charge_step(&finished.location)?;
                        let mut component: Vec<usize> = Vec::new();
                        while let Some(member) = state.stack.pop() {
                            state.set_on_stack(member, false);
                            component.push(member);
                            if member == finished.node {
                                break;
                            }
                        }
                        state.components.push(component);
                    }
                }
            }
        }

        for component in &mut state.components {
            component.sort_unstable();
        }
        state.components.sort();
        Ok(state.components)
    }
}

/// One candidate offending edge inside a recursive component.
#[derive(Clone, Debug)]
struct OffendingEdgeV1 {
    source: usize,
    target: usize,
    component: usize,
    polarity: BodyPolarityV1,
    aggregate_function: Option<String>,
    literal_location: CheckedLocatedSpanV2,
    rule_location: CheckedLocatedSpanV2,
}

fn offending_order(edge: &OffendingEdgeV1) -> EdgeOrderV1 {
    EdgeOrderV1 {
        source: edge.source,
        target: edge.target,
        polarity: edge.polarity.rank(),
        literal: location_order(&edge.literal_location),
        rule: location_order(&edge.rule_location),
    }
}

/// Scan every projected edge once, charging one step per examined edge, and
/// return the canonical offending edge for negation and for aggregation.
///
/// An edge is offending when it is not positive and both endpoints sit in the
/// same recursive component. The scan cannot depend on component or adjacency
/// insertion order because component members are canonical, adjacency is
/// canonical, and the returned candidates are the minimum under
/// [`EdgeOrderV1`].
fn scan_offending_edges(
    graph: &RuleGraphV1,
    components: &[Vec<usize>],
    charge: &mut RuleGraphChargeV1,
) -> Result<(Option<OffendingEdgeV1>, Option<OffendingEdgeV1>), LogicDiagnosticV1> {
    let mut negated: Vec<OffendingEdgeV1> = Vec::new();
    let mut aggregated: Vec<OffendingEdgeV1> = Vec::new();
    for (component_index, component) in components.iter().enumerate() {
        let recursive = is_recursive_component(graph, component);
        let members: BTreeSet<usize> = if recursive {
            component.iter().copied().collect()
        } else {
            BTreeSet::new()
        };
        for &source in component {
            for edge in graph.bucket(source) {
                charge.charge_step(&edge.literal_location)?;
                if !members.contains(&edge.target) {
                    continue;
                }
                let candidate = OffendingEdgeV1 {
                    source,
                    target: edge.target,
                    component: component_index,
                    polarity: edge.polarity,
                    aggregate_function: edge.aggregate_function.clone(),
                    literal_location: edge.literal_location.clone(),
                    rule_location: edge.rule_location.clone(),
                };
                match edge.polarity {
                    BodyPolarityV1::Positive => {}
                    BodyPolarityV1::Negated => negated.push(candidate),
                    BodyPolarityV1::Aggregated => aggregated.push(candidate),
                }
            }
        }
    }
    Ok((
        negated
            .into_iter()
            .min_by(|left, right| offending_order(left).cmp(&offending_order(right))),
        aggregated
            .into_iter()
            .min_by(|left, right| offending_order(left).cmp(&offending_order(right))),
    ))
}

fn is_recursive_component(graph: &RuleGraphV1, component: &[usize]) -> bool {
    component.len() > 1
        || component
            .first()
            .is_some_and(|only| graph.has_self_loop(*only))
}

fn component_members(components: &[Vec<usize>], component: usize) -> BTreeSet<usize> {
    components
        .get(component)
        .map(|members| members.iter().copied().collect())
        .unwrap_or_default()
}

/// Prove an actual return path inside the offending edge's component.
///
/// Breadth-first search from the offending edge's target back to its source,
/// restricted to component members and traversed in canonical adjacency order.
/// The result is the lexicographically-first shortest return path, so it is
/// invariant under rule/edge insertion permutation. One step is charged per
/// examined edge, before it is examined.
fn canonical_return_path(
    graph: &RuleGraphV1,
    members: &BTreeSet<usize>,
    offending: &OffendingEdgeV1,
    charge: &mut RuleGraphChargeV1,
) -> Result<Vec<usize>, LogicDiagnosticV1> {
    let from = offending.target;
    let to = offending.source;
    let mut predecessor: Vec<Option<usize>> = (0..graph.nodes.len()).map(|_| None).collect();
    let mut seen: Vec<bool> = (0..graph.nodes.len()).map(|_| false).collect();
    let mut queue: VecDeque<usize> = VecDeque::new();
    if let Some(flag) = seen.get_mut(from) {
        *flag = true;
    }
    queue.push_back(from);

    while let Some(node) = queue.pop_front() {
        if node == to {
            break;
        }
        for edge in graph.bucket(node) {
            charge.charge_step(&edge.literal_location)?;
            if !members.contains(&edge.target) {
                continue;
            }
            if seen.get(edge.target).copied().unwrap_or(false) {
                continue;
            }
            if let Some(flag) = seen.get_mut(edge.target) {
                *flag = true;
            }
            if let Some(slot) = predecessor.get_mut(edge.target) {
                *slot = Some(node);
            }
            queue.push_back(edge.target);
        }
    }

    if !seen.get(to).copied().unwrap_or(false) {
        return Err(internal_membership_error(&offending.rule_location));
    }

    let mut path: Vec<usize> = vec![to];
    let mut cursor = to;
    while cursor != from {
        let Some(previous) = predecessor.get(cursor).copied().flatten() else {
            return Err(internal_membership_error(&offending.rule_location));
        };
        path.push(previous);
        cursor = previous;
    }
    path.reverse();
    Ok(path)
}

/// Render the canonical closed cycle for one offending edge.
///
/// The cycle starts at the offending edge's source; consecutive entries are
/// joined by actual projected edges, and the offending edge itself closes the
/// last entry back to the first. A self-loop therefore renders as one identity.
fn cycle_identities(
    graph: &RuleGraphV1,
    offending: &OffendingEdgeV1,
    return_path: &[usize],
) -> Result<Vec<ResolvedPredicateIdV2>, LogicDiagnosticV1> {
    let mut cycle: Vec<ResolvedPredicateIdV2> = Vec::new();
    let Some(source) = graph.node(offending.source) else {
        return Err(internal_membership_error(&offending.rule_location));
    };
    cycle.push(source.clone());
    for member in return_path.iter().take(return_path.len().saturating_sub(1)) {
        let Some(identifier) = graph.node(*member) else {
            return Err(internal_membership_error(&offending.rule_location));
        };
        cycle.push(identifier.clone());
    }
    Ok(cycle)
}

/// Layer predicates into strata by relaxation over the SCC condensation.
///
/// Positive edges keep the source at `>=` the target's stratum; negated and
/// aggregated edges force the source strictly above the target. Members of one
/// SCC share a stratum (a negation or aggregation cycle already rejected).
/// One step is charged per examined edge in every pass, before the pass.
fn assign_strata(
    graph: &RuleGraphV1,
    components: &[Vec<usize>],
    charge: &mut RuleGraphChargeV1,
) -> Result<BTreeMap<ResolvedPredicateIdV2, usize>, LogicDiagnosticV1> {
    let mut component_of: Vec<Option<usize>> = (0..graph.nodes.len()).map(|_| None).collect();
    for (component_index, component) in components.iter().enumerate() {
        for &member in component {
            if let Some(slot) = component_of.get_mut(member) {
                *slot = Some(component_index);
            }
        }
    }

    let mut levels: Vec<usize> = (0..components.len()).map(|_| 0).collect();
    let mut pass = 0usize;
    let mut changed = true;
    while changed && pass <= components.len() {
        changed = false;
        pass = pass.saturating_add(1);
        for source in 0..graph.nodes.len() {
            let Some(source_component) = component_of.get(source).copied().flatten() else {
                continue;
            };
            for edge in graph.bucket(source) {
                charge.charge_step(&edge.literal_location)?;
                let Some(target_component) = component_of.get(edge.target).copied().flatten()
                else {
                    continue;
                };
                if target_component == source_component {
                    continue;
                }
                let target_level = levels.get(target_component).copied().unwrap_or(0);
                let required = match edge.polarity {
                    BodyPolarityV1::Positive => target_level,
                    BodyPolarityV1::Negated | BodyPolarityV1::Aggregated => {
                        target_level.saturating_add(1)
                    }
                };
                if let Some(slot) = levels.get_mut(source_component) {
                    if *slot < required {
                        *slot = required;
                        changed = true;
                    }
                }
            }
        }
    }

    let mut strata: BTreeMap<ResolvedPredicateIdV2, usize> = BTreeMap::new();
    for (index, node) in graph.nodes.iter().enumerate() {
        let level = component_of
            .get(index)
            .copied()
            .flatten()
            .and_then(|component| levels.get(component).copied())
            .unwrap_or(0);
        strata.insert(node.clone(), level);
    }
    Ok(strata)
}
