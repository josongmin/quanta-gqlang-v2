//! Recursive closure, least-fixpoint, and stratification semantics (ticket 05).
//!
//! This module owns the legality of recursion over rule/view composition built
//! on the shared [`quanta_gqlang_v2_hir::RuleHirV1`] carrier. It cites:
//!
//! - `D27`: positive recursion uses EXPLICIT least-fixpoint semantics; recursive
//!   aggregates or unbounded witness/path materialization reject unless a
//!   legality rule explicitly admits them.
//! - `D10`: illegal recursion / negation cycles reject with typed diagnostics,
//!   no silent downgrade.
//!
//! # Why the caller supplies body dependencies
//!
//! Structure-only HIR ([`quanta_gqlang_v2_hir::RuleHirV1`]) cannot by itself say
//! which body symbols are *predicate references* (edges in the dependency
//! graph), which are negated, and which are aggregated — that is binder/typing
//! information (tickets 02/03). So, exactly like ticket 04 takes its aggregate
//! catalog from the caller, a [`RuleDefV1`] pairs a rule's head predicate name
//! with the explicit set of [`BodyLiteralV1`] dependencies the binder resolved.
//! This keeps recursion legality a pure, deterministic graph computation here.
//!
//! # The legality pipeline
//!
//! 1. Build the predicate dependency graph: an edge `p -> q` (with a polarity)
//!    means a rule defining `p` references `q` in its body.
//! 2. Compute strongly-connected components (SCCs) via Tarjan's algorithm; a
//!    predicate is RECURSIVE iff it sits in a non-trivial SCC (or self-loops).
//! 3. A NEGATION CYCLE — a cycle containing at least one negated edge — is
//!    unstratifiable and rejects ([`LogicDiagnosticV1::NegationCycle`]).
//! 4. A recursive predicate whose own recursion goes through negation or
//!    aggregation is NON-MONOTONE and rejects
//!    ([`LogicDiagnosticV1::NonMonotoneRecursion`]); positive recursion ADMITS
//!    under least-fixpoint with a [`StratificationProofV1`].

use std::collections::{BTreeMap, BTreeSet};

use quanta_gqlang_v2_hir::{RuleHirV1, SpanV1};

use crate::diagnostic::{LogicDiagnosticV1, NonMonotoneReasonV1};

/// How a body literal references its predicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyPolarityV1 {
    /// A positive (non-negated, non-aggregated) reference. Monotone.
    Positive,
    /// A negated reference (`not p(...)`). Non-monotone across recursion.
    Negated,
    /// An aggregated reference (`count(p(...))` etc.). Non-monotone across
    /// recursion unless explicitly admitted by the aggregate matrix.
    Aggregated,
}

/// One predicate reference inside a rule body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyLiteralV1 {
    /// The referenced predicate name (the head name of some other / same rule).
    pub predicate: String,
    /// How the reference participates: positive, negated, or aggregated.
    pub polarity: BodyPolarityV1,
}

impl BodyLiteralV1 {
    /// A positive (monotone) body reference.
    #[must_use]
    pub fn positive(predicate: impl Into<String>) -> Self {
        Self {
            predicate: predicate.into(),
            polarity: BodyPolarityV1::Positive,
        }
    }

    /// A negated body reference.
    #[must_use]
    pub fn negated(predicate: impl Into<String>) -> Self {
        Self {
            predicate: predicate.into(),
            polarity: BodyPolarityV1::Negated,
        }
    }

    /// An aggregated body reference.
    #[must_use]
    pub fn aggregated(predicate: impl Into<String>) -> Self {
        Self {
            predicate: predicate.into(),
            polarity: BodyPolarityV1::Aggregated,
        }
    }
}

/// A rule definition: the head predicate name plus its resolved body
/// dependencies, anchored to a shared [`RuleHirV1`] for identity and span.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleDefV1 {
    /// The head predicate this rule (partially) defines.
    pub head: String,
    /// The body predicate references (binder-resolved dependencies).
    pub body: Vec<BodyLiteralV1>,
    /// Span of the rule, for diagnostics (copied from the shared HIR rule).
    pub span: SpanV1,
}

impl RuleDefV1 {
    /// Build a rule definition from a shared HIR rule plus its resolved body
    /// dependencies. The rule's name becomes the head predicate; the span is
    /// copied for diagnostics.
    #[must_use]
    pub fn from_rule(rule: &RuleHirV1, body: Vec<BodyLiteralV1>) -> Self {
        Self {
            head: rule.name.clone(),
            body,
            span: rule.span,
        }
    }

    /// Build a rule definition directly from parts (for callers not holding a
    /// `RuleHirV1`, e.g. synthesized views).
    #[must_use]
    pub fn new(head: impl Into<String>, body: Vec<BodyLiteralV1>, span: SpanV1) -> Self {
        Self {
            head: head.into(),
            body,
            span,
        }
    }
}

/// A least-fixpoint stratification proof for an admitted recursive program.
///
/// This is the EXPLICIT artifact `D27` requires: it records, per predicate, the
/// stratum (evaluation layer) it lands in, and which predicates are recursive.
/// Positive recursion is admitted precisely because such an assignment exists —
/// every negated/aggregated dependency crosses to a strictly lower stratum, and
/// each recursive SCC is evaluated by least-fixpoint within its own stratum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StratificationProofV1 {
    /// Stratum index per predicate. A predicate in stratum `n` may only negate /
    /// aggregate predicates in strata `< n`; positive dependencies may be `<= n`.
    pub strata: BTreeMap<String, usize>,
    /// The predicates that participate in recursion (sit in a non-trivial SCC
    /// or self-loop), in deterministic order.
    pub recursive: BTreeSet<String>,
    /// The number of strata (the highest stratum index + 1, or 0 when empty).
    pub stratum_count: usize,
}

impl StratificationProofV1 {
    /// The stratum a predicate was assigned, if present.
    #[must_use]
    pub fn stratum_of(&self, predicate: &str) -> Option<usize> {
        self.strata.get(predicate).copied()
    }

    /// Whether a predicate is recursive under this program.
    #[must_use]
    pub fn is_recursive(&self, predicate: &str) -> bool {
        self.recursive.contains(predicate)
    }
}

/// A recursive logic program: a set of rule definitions over named predicates.
///
/// Construction is structural only; legality is computed by [`Self::stratify`]
/// which either returns a [`StratificationProofV1`] (admission, with explicit
/// least-fixpoint semantics for any recursive SCC) or a typed
/// [`LogicDiagnosticV1`] (rejection).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecursiveProgramV1 {
    rules: Vec<RuleDefV1>,
}

impl RecursiveProgramV1 {
    /// An empty program.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// A program from rule definitions.
    #[must_use]
    pub fn from_rules(rules: Vec<RuleDefV1>) -> Self {
        Self { rules }
    }

    /// Add a rule definition.
    pub fn add_rule(&mut self, rule: RuleDefV1) {
        self.rules.push(rule);
    }

    /// The rule definitions in this program.
    #[must_use]
    pub fn rules(&self) -> &[RuleDefV1] {
        &self.rules
    }

    /// Compute the least-fixpoint stratification proof, or reject.
    ///
    /// Admits positive recursion with an explicit per-predicate stratum
    /// assignment; rejects negation cycles
    /// ([`LogicDiagnosticV1::NegationCycle`]) and non-monotone recursion
    /// ([`LogicDiagnosticV1::NonMonotoneRecursion`]) with typed diagnostics.
    pub fn stratify(&self) -> Result<StratificationProofV1, LogicDiagnosticV1> {
        let graph = DependencyGraph::build(&self.rules);
        let sccs = graph.strongly_connected_components();

        // A predicate is recursive iff it sits in a non-trivial SCC or has a
        // self-loop.
        let mut recursive: BTreeSet<String> = BTreeSet::new();
        for scc in &sccs {
            if scc.len() > 1 {
                for p in scc {
                    recursive.insert(p.clone());
                }
            } else if let Some(only) = scc.first() {
                if graph.has_self_loop(only) {
                    recursive.insert(only.clone());
                }
            }
        }

        // Within any recursive SCC, a negated or aggregated edge to another
        // member of the SCC is a recursion-through-negation/aggregation, which
        // is both an unstratifiable cycle AND non-monotone. Detect and reject.
        for scc in &sccs {
            let members: BTreeSet<&String> = scc.iter().collect();
            for rule in &self.rules {
                if !members.contains(&rule.head) {
                    continue;
                }
                for lit in &rule.body {
                    if !members.contains(&lit.predicate) {
                        continue;
                    }
                    match lit.polarity {
                        BodyPolarityV1::Positive => {}
                        BodyPolarityV1::Negated => {
                            let cycle = sorted_cycle(scc);
                            return Err(LogicDiagnosticV1::NegationCycle {
                                cycle,
                                span: rule.span,
                            });
                        }
                        BodyPolarityV1::Aggregated => {
                            return Err(LogicDiagnosticV1::NonMonotoneRecursion {
                                predicate: rule.head.clone(),
                                reason: NonMonotoneReasonV1::AggregatedRecursion,
                                span: rule.span,
                            });
                        }
                    }
                }
            }
        }

        // Assign strata. Condense the graph to its SCCs (a DAG) and layer it so
        // that every negated/aggregated edge strictly increases the stratum and
        // every positive edge is non-decreasing.
        let strata = graph.assign_strata(&sccs);
        let stratum_count = strata.values().copied().max().map_or(0, |m| m + 1);

        Ok(StratificationProofV1 {
            strata,
            recursive,
            stratum_count,
        })
    }
}

/// Produce a deterministic, sorted cycle witness for diagnostics.
fn sorted_cycle(scc: &[String]) -> Vec<String> {
    let mut c: Vec<String> = scc.to_vec();
    c.sort();
    c
}

/// The predicate dependency graph with per-edge polarity.
struct DependencyGraph {
    /// All predicate names (heads and referenced), in stable sorted order.
    nodes: Vec<String>,
    /// Adjacency: predicate -> its body references with polarity.
    edges: BTreeMap<String, Vec<(String, BodyPolarityV1)>>,
}

impl DependencyGraph {
    fn build(rules: &[RuleDefV1]) -> Self {
        let mut node_set: BTreeSet<String> = BTreeSet::new();
        let mut edges: BTreeMap<String, Vec<(String, BodyPolarityV1)>> = BTreeMap::new();
        for rule in rules {
            node_set.insert(rule.head.clone());
            let bucket = edges.entry(rule.head.clone()).or_default();
            for lit in &rule.body {
                node_set.insert(lit.predicate.clone());
                bucket.push((lit.predicate.clone(), lit.polarity));
            }
        }
        // Ensure every node has an (possibly empty) adjacency bucket.
        for n in &node_set {
            edges.entry(n.clone()).or_default();
        }
        Self {
            nodes: node_set.into_iter().collect(),
            edges,
        }
    }

    fn successors(&self, node: &str) -> &[(String, BodyPolarityV1)] {
        self.edges.get(node).map_or(&[], Vec::as_slice)
    }

    fn has_self_loop(&self, node: &str) -> bool {
        self.successors(node).iter().any(|(t, _)| t == node)
    }

    /// Tarjan's strongly-connected-components, returning SCCs in reverse
    /// topological order (sinks first). Determinism comes from iterating
    /// `nodes` in sorted order and successors in declared order.
    fn strongly_connected_components(&self) -> Vec<Vec<String>> {
        let mut state = TarjanState::new(self);
        for node in &self.nodes {
            if !state.indices.contains_key(node) {
                state.strong_connect(self, node);
            }
        }
        state.result
    }

    /// Layer predicates into strata. Positive edges keep the source at >= the
    /// target's stratum; negated/aggregated edges force the source strictly
    /// above the target. Members of one SCC share a stratum (cycles are positive
    /// here, since negation cycles already rejected). Computed by relaxation to a
    /// fixpoint over the SCC condensation, which terminates because the
    /// condensation is a DAG.
    fn assign_strata(&self, sccs: &[Vec<String>]) -> BTreeMap<String, usize> {
        // Map each predicate to its SCC representative index.
        let mut scc_of: BTreeMap<String, usize> = BTreeMap::new();
        for (idx, scc) in sccs.iter().enumerate() {
            for p in scc {
                scc_of.insert(p.clone(), idx);
            }
        }
        let mut scc_stratum: Vec<usize> = vec![0; sccs.len()];

        // Relax until stable. Bounded by sccs.len() passes (longest DAG path).
        let mut changed = true;
        let mut passes = 0usize;
        while changed && passes <= sccs.len() {
            changed = false;
            passes = passes.saturating_add(1);
            for node in &self.nodes {
                let Some(&src_scc) = scc_of.get(node) else {
                    continue;
                };
                for (target, polarity) in self.successors(node) {
                    let Some(&dst_scc) = scc_of.get(target) else {
                        continue;
                    };
                    if dst_scc == src_scc {
                        continue; // intra-SCC edge: same stratum
                    }
                    let dst_level = scc_stratum.get(dst_scc).copied().unwrap_or(0);
                    let required = match polarity {
                        BodyPolarityV1::Positive => dst_level,
                        BodyPolarityV1::Negated | BodyPolarityV1::Aggregated => {
                            dst_level.saturating_add(1)
                        }
                    };
                    if let Some(slot) = scc_stratum.get_mut(src_scc) {
                        if *slot < required {
                            *slot = required;
                            changed = true;
                        }
                    }
                }
            }
        }

        let mut strata: BTreeMap<String, usize> = BTreeMap::new();
        for node in &self.nodes {
            let level = scc_of
                .get(node)
                .and_then(|idx| scc_stratum.get(*idx))
                .copied()
                .unwrap_or(0);
            strata.insert(node.clone(), level);
        }
        strata
    }
}

/// Tarjan SCC working state, kept off the recursion to avoid borrow conflicts.
struct TarjanState {
    index: usize,
    indices: BTreeMap<String, usize>,
    lowlink: BTreeMap<String, usize>,
    on_stack: BTreeSet<String>,
    stack: Vec<String>,
    result: Vec<Vec<String>>,
}

impl TarjanState {
    fn new(_graph: &DependencyGraph) -> Self {
        Self {
            index: 0,
            indices: BTreeMap::new(),
            lowlink: BTreeMap::new(),
            on_stack: BTreeSet::new(),
            stack: Vec::new(),
            result: Vec::new(),
        }
    }

    fn strong_connect(&mut self, graph: &DependencyGraph, node: &str) {
        self.indices.insert(node.to_owned(), self.index);
        self.lowlink.insert(node.to_owned(), self.index);
        self.index = self.index.saturating_add(1);
        self.stack.push(node.to_owned());
        self.on_stack.insert(node.to_owned());

        for (target, _) in graph.successors(node) {
            if !self.indices.contains_key(target) {
                self.strong_connect(graph, target);
                let tl = self.lowlink.get(target).copied().unwrap_or(0);
                if let Some(nl) = self.lowlink.get_mut(node) {
                    *nl = (*nl).min(tl);
                }
            } else if self.on_stack.contains(target) {
                let ti = self.indices.get(target).copied().unwrap_or(0);
                if let Some(nl) = self.lowlink.get_mut(node) {
                    *nl = (*nl).min(ti);
                }
            }
        }

        let node_index = self.indices.get(node).copied().unwrap_or(0);
        let node_low = self.lowlink.get(node).copied().unwrap_or(0);
        if node_low == node_index {
            let mut component: Vec<String> = Vec::new();
            while let Some(w) = self.stack.pop() {
                self.on_stack.remove(&w);
                let done = w == node;
                component.push(w);
                if done {
                    break;
                }
            }
            self.result.push(component);
        }
    }
}
