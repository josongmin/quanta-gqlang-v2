//! Finite-domain rule-variable inference over checked HIR and exact Binder IDs.
//!
//! This is the sole rule-variable constraint engine. It consumes the same
//! closed operator/builtin tables as `ExpressionTypeKernelV2`; sema supplies
//! identities, not candidate types or guessed defaults.

use crate::builtin::{
    BuiltinFamilyIdV2, BuiltinRegistryV2, CallableSignatureV2, ExpressionEffectsV2,
};
use crate::check::{
    ExpressionTypeStateV2, ResolvedHostPredicateCallableV2, ResolvedPredicateCallableV2,
    ResolvedPredicateIdV2, SourceQualifiedNodeIdV2, UnknownTypeCauseV2,
};
use crate::op::OperatorRegistryV2;
use crate::{RecordFieldTypeV1, RecordTypeV1, TypeDiagnosticV2, ValueTypeV1};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CheckedHirDocumentV2,
    ExtendedSourceRegistryViewV2, HirNodeIdV1, HirVisitorV1, LiteralV1, LocatedSpanV2,
    LogicalBinderV1, ProgramItemHirV1, SourceIdV2, SpanV1, ValueExprHirV1, ValueExprKindV1,
};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

mod engine;
use engine::{ConstructedCandidateTypes, Engine};

type ValidatedCallsV2 = (
    BTreeMap<HirNodeIdV1, RuleCallableTargetV2>,
    Vec<(SpanV1, TypeDiagnosticV2)>,
);

/// Exact Binder binding identity, including the ordinal for quantifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleBindingIdV2 {
    pub source_id: SourceIdV2,
    pub owner_node: HirNodeIdV1,
    pub ordinal: usize,
}

impl RuleBindingIdV2 {
    #[must_use]
    pub const fn new(source_id: SourceIdV2, owner_node: HirNodeIdV1, ordinal: usize) -> Self {
        Self {
            source_id,
            owner_node,
            ordinal,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulePredicateInputV2 {
    pub rule_node: SourceQualifiedNodeIdV2,
    pub predicate_id: ResolvedPredicateIdV2,
    pub head_binding: RuleBindingIdV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSymbolInputV2 {
    pub reference: SourceQualifiedNodeIdV2,
    pub binding: RuleBindingIdV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleCallableTargetV2 {
    Builtin(BuiltinFamilyIdV2),
    HostPredicate(ResolvedHostPredicateCallableV2),
    SourcePredicate(ResolvedPredicateIdV2),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleCallInputV2 {
    pub call: SourceQualifiedNodeIdV2,
    pub target: RuleCallableTargetV2,
}

/// Immutable positive work budget. Sema projects frontend limits into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleTypeBudgetV2 {
    nodes: NonZeroU32,
    edges: NonZeroU32,
    steps: NonZeroU32,
}

impl RuleTypeBudgetV2 {
    pub fn try_new(nodes: u32, edges: u32, steps: u32) -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            nodes: NonZeroU32::new(nodes).ok_or(TypeDiagnosticV2::InvalidRuleTypeBudget)?,
            edges: NonZeroU32::new(edges).ok_or(TypeDiagnosticV2::InvalidRuleTypeBudget)?,
            steps: NonZeroU32::new(steps).ok_or(TypeDiagnosticV2::InvalidRuleTypeBudget)?,
        })
    }

    #[must_use]
    pub const fn limits(self) -> (u32, u32, u32) {
        (self.nodes.get(), self.edges.get(), self.steps.get())
    }
}

fn charge_prepass_steps(
    steps: &mut u32,
    budget: RuleTypeBudgetV2,
    source_id: SourceIdV2,
    span: SpanV1,
    amount: usize,
) -> Result<(), TypeDiagnosticV2> {
    let amount = u32::try_from(amount).map_err(|_conversion_error| {
        TypeDiagnosticV2::RuleTypeResourceLimit {
            dimension: "steps",
            location: LocatedSpanV2 { source_id, span },
        }
    })?;
    let next = steps
        .checked_add(amount)
        .ok_or(TypeDiagnosticV2::RuleTypeResourceLimit {
            dimension: "steps",
            location: LocatedSpanV2 { source_id, span },
        })?;
    if next > budget.steps.get() {
        return Err(TypeDiagnosticV2::RuleTypeResourceLimit {
            dimension: "steps",
            location: LocatedSpanV2 { source_id, span },
        });
    }
    *steps = next;
    Ok(())
}

fn type_shape_cost<'a>(types: impl IntoIterator<Item = &'a ValueTypeV1>) -> usize {
    types
        .into_iter()
        .fold(0usize, |cost, ty| cost.saturating_add(ty.node_count()))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuleTypeUsageV2 {
    pub nodes: u32,
    pub edges: u32,
    pub steps: u32,
}

#[derive(Clone, Debug)]
pub struct RuleTypeProofV2 {
    bindings: BTreeMap<RuleBindingIdV2, ValueTypeV1>,
    predicates: BTreeMap<ResolvedPredicateIdV2, ValueTypeV1>,
    usage: RuleTypeUsageV2,
}

/// Open-analysis facts are never a closed type proof. Non-unique variables are
/// represented explicitly as Unknown, with ordered diagnostics.
#[derive(Clone, Debug)]
pub struct RuleTypeAnalysisV2 {
    bindings: BTreeMap<RuleBindingIdV2, ExpressionTypeStateV2>,
    predicates: BTreeMap<ResolvedPredicateIdV2, ExpressionTypeStateV2>,
    diagnostics: Vec<TypeDiagnosticV2>,
    usage: RuleTypeUsageV2,
}

impl RuleTypeAnalysisV2 {
    #[must_use]
    pub fn binding_state(&self, id: RuleBindingIdV2) -> Option<&ExpressionTypeStateV2> {
        self.bindings.get(&id)
    }
    #[must_use]
    pub fn predicate_state(&self, id: &ResolvedPredicateIdV2) -> Option<&ExpressionTypeStateV2> {
        self.predicates.get(id)
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[TypeDiagnosticV2] {
        &self.diagnostics
    }
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.diagnostics.is_empty()
    }
    #[must_use]
    pub const fn usage(&self) -> RuleTypeUsageV2 {
        self.usage
    }
}

impl RuleTypeProofV2 {
    #[must_use]
    pub fn binding_type(&self, id: RuleBindingIdV2) -> Option<&ValueTypeV1> {
        self.bindings.get(&id)
    }
    #[must_use]
    pub fn predicate_type(&self, id: &ResolvedPredicateIdV2) -> Option<&ValueTypeV1> {
        self.predicates.get(id)
    }
    /// A source rule is a unary Bool predicate. Effects remain conservative:
    /// type inference does not prove purity or monotonicity of the rule body.
    pub fn try_source_predicate_callable(
        &self,
        id: &ResolvedPredicateIdV2,
    ) -> Result<Option<ResolvedPredicateCallableV2>, TypeDiagnosticV2> {
        self.predicates
            .get(id)
            .map(|head_type| {
                ResolvedPredicateCallableV2::try_new(
                    id.clone(),
                    vec![head_type.clone()],
                    ValueTypeV1::bool(),
                    ExpressionEffectsV2::conservative_source_predicate(),
                )
            })
            .transpose()
    }
    #[must_use]
    pub const fn usage(&self) -> RuleTypeUsageV2 {
        self.usage
    }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = (RuleBindingIdV2, &ValueTypeV1)> {
        self.bindings.iter().map(|(id, ty)| (*id, ty))
    }
    pub fn predicates(
        &self,
    ) -> impl ExactSizeIterator<Item = (&ResolvedPredicateIdV2, &ValueTypeV1)> {
        self.predicates.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Variable {
    Binding(RuleBindingIdV2),
    Value(SourceQualifiedNodeIdV2),
    Bool(SourceQualifiedNodeIdV2),
}

#[derive(Clone, Debug)]
struct Constraint {
    variables: Vec<Variable>,
    allowed: Vec<Vec<ValueTypeV1>>,
    location: LocatedSpanV2,
}

const MAX_RULE_TYPE_TRAVERSAL_DEPTH: usize = 64;

struct Expected {
    rules: BTreeMap<HirNodeIdV1, (HirNodeIdV1, SpanV1)>,
    rule_names: BTreeMap<HirNodeIdV1, String>,
    bindings: BTreeMap<(HirNodeIdV1, usize), (SpanV1, String)>,
    symbols: BTreeMap<HirNodeIdV1, (SpanV1, String)>,
    nearest_bindings: BTreeMap<HirNodeIdV1, (HirNodeIdV1, usize)>,
    calls: BTreeMap<HirNodeIdV1, (SpanV1, usize, bool)>,
    call_names: BTreeMap<HirNodeIdV1, String>,
    call_owners: BTreeMap<HirNodeIdV1, HirNodeIdV1>,
    source_id: SourceIdV2,
    budget: RuleTypeBudgetV2,
    steps: u32,
}

enum LexicalScope<'a> {
    Rule(HirNodeIdV1, &'a str),
    Quantifier(HirNodeIdV1, &'a [LogicalBinderV1]),
}

impl LexicalScope<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Rule(_, _) => 1,
            Self::Quantifier(_, binders) => binders.len(),
        }
    }

    fn find(&self, name: &str) -> Option<(HirNodeIdV1, usize)> {
        match self {
            Self::Rule(owner, bound) if *bound == name => Some((*owner, 0)),
            Self::Quantifier(owner, binders) => binders
                .iter()
                .enumerate()
                .find(|(_, binder)| binder.name == name)
                .map(|(ordinal, _)| (*owner, ordinal)),
            Self::Rule(_, _) => None,
        }
    }
}

enum ExpectedFrame<'a> {
    Bool(&'a BoolExprHirV1, usize),
    Value(&'a ValueExprHirV1, usize),
    PopScope,
}

impl Expected {
    fn new(source_id: SourceIdV2, budget: RuleTypeBudgetV2) -> Self {
        Self {
            rules: BTreeMap::new(),
            rule_names: BTreeMap::new(),
            bindings: BTreeMap::new(),
            symbols: BTreeMap::new(),
            nearest_bindings: BTreeMap::new(),
            calls: BTreeMap::new(),
            call_names: BTreeMap::new(),
            call_owners: BTreeMap::new(),
            source_id,
            budget,
            steps: 0,
        }
    }

    fn charge(&mut self, span: SpanV1, amount: usize) -> Result<(), TypeDiagnosticV2> {
        charge_prepass_steps(&mut self.steps, self.budget, self.source_id, span, amount)
    }

    fn depth(&self, span: SpanV1, depth: usize) -> Result<(), TypeDiagnosticV2> {
        if depth > MAX_RULE_TYPE_TRAVERSAL_DEPTH {
            return Err(TypeDiagnosticV2::RuleTypeResourceLimit {
                dimension: "depth",
                location: LocatedSpanV2 {
                    source_id: self.source_id,
                    span,
                },
            });
        }
        Ok(())
    }

    fn visit_rule_iterative(
        &mut self,
        rule: &quanta_gqlang_v2_hir::RuleHirV1,
    ) -> Result<(), TypeDiagnosticV2> {
        let ValueExprKindV1::Symbol(head) = &rule.head.kind else {
            return Err(TypeDiagnosticV2::InvalidRuleHead {
                node_id: rule.head.id,
                location: LocatedSpanV2 {
                    source_id: self.source_id,
                    span: rule.head.span,
                },
            });
        };
        self.charge(
            rule.span,
            rule.name
                .len()
                .saturating_add(head.name.len())
                .saturating_add(2),
        )?;
        let _ = self.rules.insert(rule.id, (rule.head.id, rule.span));
        let _ = self.rule_names.insert(rule.id, rule.name.clone());
        let _ = self
            .bindings
            .insert((rule.id, 0), (rule.head.span, head.name.clone()));
        let mut scopes = vec![LexicalScope::Rule(rule.id, &head.name)];
        let mut pending = vec![
            ExpectedFrame::Bool(&rule.body, 1),
            ExpectedFrame::Value(&rule.head, 1),
        ];
        while let Some(frame) = pending.pop() {
            match frame {
                ExpectedFrame::PopScope => {
                    let _ = scopes.pop();
                }
                ExpectedFrame::Bool(node, depth) => {
                    self.depth(node.span, depth)?;
                    self.charge(node.span, 1)?;
                    match &node.kind {
                        BoolExprKindV1::Literal(_) => {}
                        BoolExprKindV1::Value(value) => {
                            self.charge(node.span, 1)?;
                            pending.push(ExpectedFrame::Value(value, depth + 1));
                        }
                        BoolExprKindV1::Compare(compare) => {
                            self.charge(node.span, 2)?;
                            pending.push(ExpectedFrame::Value(&compare.rhs, depth + 1));
                            pending.push(ExpectedFrame::Value(&compare.lhs, depth + 1));
                        }
                        BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
                            self.charge(node.span, 2)?;
                            pending.push(ExpectedFrame::Bool(&binary.rhs, depth + 1));
                            pending.push(ExpectedFrame::Bool(&binary.lhs, depth + 1));
                        }
                        BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                            self.charge(node.span, 1)?;
                            pending.push(ExpectedFrame::Bool(inner, depth + 1));
                        }
                        BoolExprKindV1::LogicalQuantifier(quant) => {
                            let binders = quant.binders.as_slice();
                            self.charge(node.span, binders.len().saturating_add(2))?;
                            let mut names = BTreeSet::new();
                            for (ordinal, binder) in binders.iter().enumerate() {
                                self.charge(binder.span, binder.name.len().saturating_add(1))?;
                                if !names.insert(binder.name.as_str()) {
                                    return Err(TypeDiagnosticV2::InvalidRuleInput {
                                        location: LocatedSpanV2 {
                                            source_id: self.source_id,
                                            span: binder.span,
                                        },
                                    });
                                }
                                let _ = self
                                    .bindings
                                    .insert((node.id, ordinal), (binder.span, binder.name.clone()));
                            }
                            scopes.push(LexicalScope::Quantifier(node.id, binders));
                            pending.push(ExpectedFrame::PopScope);
                            pending.push(ExpectedFrame::Bool(&quant.body, depth + 1));
                        }
                        BoolExprKindV1::RelationExists(_) => {
                            return Err(TypeDiagnosticV2::RelationExistsInRule {
                                location: LocatedSpanV2 {
                                    source_id: self.source_id,
                                    span: node.span,
                                },
                            });
                        }
                    }
                }
                ExpectedFrame::Value(node, depth) => {
                    self.depth(node.span, depth)?;
                    self.charge(node.span, 1)?;
                    match &node.kind {
                        ValueExprKindV1::Literal(_) => {}
                        ValueExprKindV1::Symbol(symbol) => {
                            let scan = scopes
                                .iter()
                                .fold(0usize, |count, scope| count.saturating_add(scope.len()));
                            self.charge(node.span, symbol.name.len().saturating_add(scan))?;
                            let nearest = scopes
                                .iter()
                                .rev()
                                .find_map(|scope| scope.find(&symbol.name));
                            let _ = self
                                .symbols
                                .insert(node.id, (node.span, symbol.name.clone()));
                            if let Some(binding) = nearest {
                                let _ = self.nearest_bindings.insert(node.id, binding);
                            }
                        }
                        ValueExprKindV1::Call(call) => {
                            self.charge(
                                node.span,
                                call.callee.name.len().saturating_add(call.args().len()),
                            )?;
                            let _ = self.call_names.insert(node.id, call.callee.name.clone());
                            let _ = self.calls.insert(
                                node.id,
                                (
                                    node.span,
                                    call.args().len(),
                                    call.args()
                                        .as_slice()
                                        .iter()
                                        .any(|arg| matches!(arg, CallArgHirV1::Star { .. })),
                                ),
                            );
                            let _ = self.call_owners.insert(node.id, rule.id);
                            for arg in call.args().as_slice().iter().rev() {
                                if let CallArgHirV1::Expr(value) = arg {
                                    pending.push(ExpectedFrame::Value(value, depth + 1));
                                }
                            }
                        }
                        ValueExprKindV1::Path(path) => {
                            self.charge(node.span, 1)?;
                            pending.push(ExpectedFrame::Value(&path.base, depth + 1));
                        }
                        ValueExprKindV1::Unary(unary) => {
                            self.charge(node.span, 1)?;
                            pending.push(ExpectedFrame::Value(&unary.operand, depth + 1));
                        }
                        ValueExprKindV1::Binary(binary) => {
                            self.charge(node.span, 2)?;
                            pending.push(ExpectedFrame::Value(&binary.rhs, depth + 1));
                            pending.push(ExpectedFrame::Value(&binary.lhs, depth + 1));
                        }
                        ValueExprKindV1::Paren(inner) => {
                            self.charge(node.span, 1)?;
                            pending.push(ExpectedFrame::Value(inner, depth + 1));
                        }
                        ValueExprKindV1::Record(record) => {
                            self.charge(node.span, record.fields.len())?;
                            for field in record.fields.iter().rev() {
                                pending.push(ExpectedFrame::Value(&field.value, depth + 1));
                            }
                        }
                        ValueExprKindV1::List(list) => {
                            self.charge(node.span, list.elements.len())?;
                            for element in list.elements.iter().rev() {
                                pending.push(ExpectedFrame::Value(element, depth + 1));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct RuleTypeSolverV2 {
    operators: OperatorRegistryV2,
    builtins: BuiltinRegistryV2,
}

impl RuleTypeSolverV2 {
    pub fn try_new(
        operators: OperatorRegistryV2,
        builtins: BuiltinRegistryV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            operators,
            builtins,
        })
    }
    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        Self::try_new(
            OperatorRegistryV2::try_closed()?,
            BuiltinRegistryV2::try_closed()?,
        )
    }

    pub fn solve(
        &self,
        checked: CheckedHirDocumentV2<'_>,
        registry: &ExtendedSourceRegistryViewV2,
        predicates: Vec<RulePredicateInputV2>,
        symbols: Vec<RuleSymbolInputV2>,
        calls: Vec<RuleCallInputV2>,
        budget: RuleTypeBudgetV2,
    ) -> Result<RuleTypeProofV2, TypeDiagnosticV2> {
        let analysis = self.analyze_open(checked, registry, predicates, symbols, calls, budget)?;
        if let Some(diagnostic) = analysis.diagnostics.first() {
            return Err(diagnostic.clone());
        }
        let mut bindings = BTreeMap::new();
        for (id, state) in analysis.bindings {
            match state {
                ExpressionTypeStateV2::Known(ty) => {
                    let _ = bindings.insert(id, ty);
                }
                ExpressionTypeStateV2::Unknown(_) => {
                    return Err(TypeDiagnosticV2::InvalidRuleInput {
                        location: LocatedSpanV2 {
                            source_id: id.source_id,
                            span: SpanV1::point(0),
                        },
                    })
                }
            }
        }
        let mut predicates = BTreeMap::new();
        for (id, state) in analysis.predicates {
            match state {
                ExpressionTypeStateV2::Known(ty) => {
                    let _ = predicates.insert(id, ty);
                }
                ExpressionTypeStateV2::Unknown(_) => {
                    return Err(TypeDiagnosticV2::InvalidRuleInput {
                        location: LocatedSpanV2 {
                            source_id: checked.document().source_id,
                            span: SpanV1::point(0),
                        },
                    })
                }
            }
        }
        Ok(RuleTypeProofV2 {
            bindings,
            predicates,
            usage: analysis.usage,
        })
    }

    pub fn analyze_open(
        &self,
        checked: CheckedHirDocumentV2<'_>,
        registry: &ExtendedSourceRegistryViewV2,
        predicates: Vec<RulePredicateInputV2>,
        symbols: Vec<RuleSymbolInputV2>,
        calls: Vec<RuleCallInputV2>,
        budget: RuleTypeBudgetV2,
    ) -> Result<RuleTypeAnalysisV2, TypeDiagnosticV2> {
        let source_id = checked.document().source_id;
        if registry.source_len(source_id) != Some(checked.document().source_len) {
            return Err(TypeDiagnosticV2::SourceRegistryMismatch {
                location: LocatedSpanV2 {
                    source_id,
                    span: SpanV1::new(0, checked.document().source_len),
                },
            });
        }
        let mut expected = Expected::new(source_id, budget);
        for item in &checked.document().items {
            if let ProgramItemHirV1::Rule(rule) = item {
                expected.visit_rule_iterative(rule)?;
            }
        }
        let predicate_map = Self::validate_predicates(&expected, source_id, predicates)?;
        let source_predicates = predicate_map.values().cloned().collect::<BTreeSet<_>>();
        let symbol_map = Self::validate_symbols(&expected, source_id, symbols)?;
        let (call_map, unresolved_calls) =
            self.validate_calls(&expected, source_id, &source_predicates, calls)?;
        let (universe, prepass_steps) = self.candidate_universe(
            checked,
            registry,
            &call_map,
            source_id,
            budget,
            expected.steps,
        )?;
        let mut engine = Engine {
            source_id,
            budget,
            usage: RuleTypeUsageV2 {
                steps: prepass_steps,
                ..RuleTypeUsageV2::default()
            },
            universe,
            domains: BTreeMap::new(),
            constraints: Vec::new(),
            symbol_map,
            call_map,
            predicate_bindings: predicate_map
                .iter()
                .map(|(rule, id)| (id.clone(), RuleBindingIdV2::new(source_id, *rule, 0)))
                .collect(),
            operators: &self.operators,
            builtins: &self.builtins,
        };
        for ((node, ordinal), (span, _)) in &expected.bindings {
            let _ = engine.intern(
                Variable::Binding(RuleBindingIdV2::new(source_id, *node, *ordinal)),
                *span,
            )?;
        }
        for item in &checked.document().items {
            if let ProgramItemHirV1::Rule(rule) = item {
                let head = engine.value(&rule.head)?;
                let _ = engine.boolean(&rule.body)?;
                engine.reserve_materialization(engine.universe.len(), 2, rule.head.span)?;
                engine.add(
                    vec![
                        head,
                        Variable::Binding(RuleBindingIdV2::new(source_id, rule.id, 0)),
                    ],
                    engine
                        .universe
                        .iter()
                        .cloned()
                        .map(|ty| vec![ty.clone(), ty])
                        .collect(),
                    rule.head.span,
                )?;
            }
        }
        engine.propagate()?;
        Self::finish_analysis(
            engine,
            &expected,
            &predicate_map,
            unresolved_calls,
            source_id,
        )
    }
    fn validate_predicates(
        expected: &Expected,
        source_id: SourceIdV2,
        predicates: Vec<RulePredicateInputV2>,
    ) -> Result<BTreeMap<HirNodeIdV1, ResolvedPredicateIdV2>, TypeDiagnosticV2> {
        let mut predicate_map = BTreeMap::new();
        let mut ids = BTreeSet::new();
        for input in predicates {
            if input.rule_node.source_id != source_id || input.head_binding.source_id != source_id {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: SpanV1::point(0),
                    },
                });
            }
            let Some((head_id, span)) = expected.rules.get(&input.rule_node.node_id).copied()
            else {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: SpanV1::point(0),
                    },
                });
            };
            if input.head_binding != RuleBindingIdV2::new(source_id, input.rule_node.node_id, 0)
                || !expected
                    .rule_names
                    .get(&input.rule_node.node_id)
                    .is_some_and(|name| name == input.predicate_id.as_str())
                || !expected.symbols.contains_key(&head_id)
                || !ids.insert(input.predicate_id.clone())
                || predicate_map
                    .insert(input.rule_node.node_id, input.predicate_id)
                    .is_some()
            {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 { source_id, span },
                });
            }
        }
        if predicate_map.len() != expected.rules.len() {
            return Err(TypeDiagnosticV2::InvalidRuleInput {
                location: LocatedSpanV2 {
                    source_id,
                    span: SpanV1::point(0),
                },
            });
        }
        Ok(predicate_map)
    }

    fn validate_symbols(
        expected: &Expected,
        source_id: SourceIdV2,
        symbols: Vec<RuleSymbolInputV2>,
    ) -> Result<BTreeMap<HirNodeIdV1, RuleBindingIdV2>, TypeDiagnosticV2> {
        let mut symbol_map = BTreeMap::new();
        for input in symbols {
            if input.reference.source_id != source_id || input.binding.source_id != source_id {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: SpanV1::point(0),
                    },
                });
            }
            let Some((span, symbol_name)) = expected.symbols.get(&input.reference.node_id) else {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: SpanV1::point(0),
                    },
                });
            };
            if !expected
                .bindings
                .get(&(input.binding.owner_node, input.binding.ordinal))
                .is_some_and(|(_, name)| name == symbol_name)
                || expected.nearest_bindings.get(&input.reference.node_id)
                    != Some(&(input.binding.owner_node, input.binding.ordinal))
                || symbol_map
                    .insert(input.reference.node_id, input.binding)
                    .is_some()
            {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: *span,
                    },
                });
            }
        }
        if symbol_map.len() != expected.symbols.len()
            || expected
                .symbols
                .keys()
                .any(|key| !symbol_map.contains_key(key))
        {
            return Err(TypeDiagnosticV2::InvalidRuleInput {
                location: LocatedSpanV2 {
                    source_id,
                    span: SpanV1::point(0),
                },
            });
        }
        for (rule_id, (head_id, span)) in &expected.rules {
            if symbol_map.get(head_id) != Some(&RuleBindingIdV2::new(source_id, *rule_id, 0)) {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: *span,
                    },
                });
            }
        }
        Ok(symbol_map)
    }

    fn validate_calls(
        &self,
        expected: &Expected,
        source_id: SourceIdV2,
        source_predicates: &BTreeSet<ResolvedPredicateIdV2>,
        calls: Vec<RuleCallInputV2>,
    ) -> Result<ValidatedCallsV2, TypeDiagnosticV2> {
        let mut call_map = BTreeMap::new();
        let mut unresolved_calls = Vec::new();
        for input in calls {
            if input.call.source_id != source_id {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: SpanV1::point(0),
                    },
                });
            }
            let Some((span, arity, star)) = expected.calls.get(&input.call.node_id).copied() else {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: SpanV1::point(0),
                    },
                });
            };
            let callee_name = expected.call_names.get(&input.call.node_id).ok_or(
                TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 { source_id, span },
                },
            )?;
            match &input.target {
                RuleCallableTargetV2::Builtin(family)
                    if !self
                        .builtins
                        .lookup_family(*family)
                        .any(|entry| entry.name == *callee_name) =>
                {
                    return Err(TypeDiagnosticV2::UnknownRuleCallable {
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
                RuleCallableTargetV2::SourcePredicate(id)
                    if !source_predicates.contains(id) || id.as_str() != callee_name =>
                {
                    return Err(TypeDiagnosticV2::UnknownRuleCallable {
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
                RuleCallableTargetV2::SourcePredicate(_) if arity != 1 || star => {
                    return Err(TypeDiagnosticV2::RulePredicateArity {
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
                RuleCallableTargetV2::HostPredicate(signature)
                    if signature.params().len() != arity || star =>
                {
                    return Err(TypeDiagnosticV2::RulePredicateArity {
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
                RuleCallableTargetV2::Unknown => {
                    unresolved_calls.push((
                        span,
                        TypeDiagnosticV2::UnknownRuleCallable {
                            location: LocatedSpanV2 { source_id, span },
                        },
                    ));
                }
                RuleCallableTargetV2::Builtin(_)
                | RuleCallableTargetV2::HostPredicate(_)
                | RuleCallableTargetV2::SourcePredicate(_) => {}
            }
            if call_map.insert(input.call.node_id, input.target).is_some() {
                return Err(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 { source_id, span },
                });
            }
        }
        if call_map.len() != expected.calls.len()
            || expected.calls.keys().any(|key| !call_map.contains_key(key))
        {
            return Err(TypeDiagnosticV2::InvalidRuleInput {
                location: LocatedSpanV2 {
                    source_id,
                    span: SpanV1::point(0),
                },
            });
        }

        Ok((call_map, unresolved_calls))
    }

    fn candidate_universe(
        &self,
        checked: CheckedHirDocumentV2<'_>,
        registry: &ExtendedSourceRegistryViewV2,
        call_map: &BTreeMap<HirNodeIdV1, RuleCallableTargetV2>,
        source_id: SourceIdV2,
        budget: RuleTypeBudgetV2,
        initial_steps: u32,
    ) -> Result<(BTreeSet<ValueTypeV1>, u32), TypeDiagnosticV2> {
        let mut prepass_steps = initial_steps;
        charge_prepass_steps(&mut prepass_steps, budget, source_id, SpanV1::point(0), 5)?;
        let mut universe = BTreeSet::from([
            ValueTypeV1::bool(),
            ValueTypeV1::int(),
            ValueTypeV1::float(),
            ValueTypeV1::string(),
            ValueTypeV1::null(),
        ]);
        for overload in self.operators.all_unary_overloads() {
            charge_prepass_steps(
                &mut prepass_steps,
                budget,
                source_id,
                SpanV1::point(0),
                type_shape_cost([&overload.operand, &overload.result]),
            )?;
            universe.extend([overload.operand.clone(), overload.result.clone()]);
        }
        for overload in self.operators.all_binary_overloads() {
            charge_prepass_steps(
                &mut prepass_steps,
                budget,
                source_id,
                SpanV1::point(0),
                type_shape_cost([&overload.lhs, &overload.rhs, &overload.result]),
            )?;
            universe.extend([
                overload.lhs.clone(),
                overload.rhs.clone(),
                overload.result.clone(),
            ]);
        }
        for entry in self.builtins.all_entries() {
            match &entry.signature {
                CallableSignatureV2::Fixed { params, result } => {
                    charge_prepass_steps(
                        &mut prepass_steps,
                        budget,
                        source_id,
                        SpanV1::point(0),
                        type_shape_cost(params.iter().chain(std::iter::once(result))),
                    )?;
                    universe.extend(params.iter().cloned());
                    let _ = universe.insert(result.clone());
                }
                CallableSignatureV2::AnySingle { result }
                | CallableSignatureV2::AggregateStar { result } => {
                    charge_prepass_steps(
                        &mut prepass_steps,
                        budget,
                        source_id,
                        SpanV1::point(0),
                        result.node_count(),
                    )?;
                    let _ = universe.insert(result.clone());
                }
            }
        }
        for target in call_map.values() {
            if let RuleCallableTargetV2::HostPredicate(signature) = target {
                charge_prepass_steps(
                    &mut prepass_steps,
                    budget,
                    source_id,
                    SpanV1::point(0),
                    type_shape_cost(
                        signature
                            .params()
                            .iter()
                            .chain(std::iter::once(signature.result())),
                    ),
                )?;
                universe.extend(signature.params().iter().cloned());
                let _ = universe.insert(signature.result().clone());
            }
        }
        charge_prepass_steps(
            &mut prepass_steps,
            budget,
            source_id,
            SpanV1::point(0),
            type_shape_cost(&universe),
        )?;
        let base_types = universe.iter().cloned().collect::<Vec<_>>();
        for ty in base_types {
            charge_prepass_steps(
                &mut prepass_steps,
                budget,
                source_id,
                SpanV1::point(0),
                ty.node_count(),
            )?;
            if let Ok(list) = ValueTypeV1::try_list(ty) {
                let _ = universe.insert(list);
            }
        }
        charge_prepass_steps(
            &mut prepass_steps,
            budget,
            source_id,
            SpanV1::point(0),
            type_shape_cost(&universe),
        )?;
        let mut constructed = ConstructedCandidateTypes::new(
            source_id,
            registry,
            budget,
            prepass_steps,
            universe.clone(),
        );
        for item in &checked.document().items {
            if let ProgramItemHirV1::Rule(rule) = item {
                constructed.visit_rule(rule);
            }
        }
        if let Some(error) = constructed.error.take() {
            return Err(error);
        }
        constructed.charge_many(SpanV1::point(0), type_shape_cost(&constructed.candidates));
        if let Some(error) = constructed.error.take() {
            return Err(error);
        }
        universe.extend(constructed.candidates);
        Ok((universe, constructed.steps))
    }

    fn finish_analysis(
        mut engine: Engine<'_>,
        expected: &Expected,
        predicate_map: &BTreeMap<HirNodeIdV1, ResolvedPredicateIdV2>,
        unresolved_calls: Vec<(SpanV1, TypeDiagnosticV2)>,
        source_id: SourceIdV2,
    ) -> Result<RuleTypeAnalysisV2, TypeDiagnosticV2> {
        let mut source_call_graph: BTreeMap<
            ResolvedPredicateIdV2,
            BTreeSet<ResolvedPredicateIdV2>,
        > = BTreeMap::new();
        for (call_node, owner_rule) in &expected.call_owners {
            engine.charge("steps", SpanV1::point(0))?;
            if let (Some(owner), Some(RuleCallableTargetV2::SourcePredicate(target))) = (
                predicate_map.get(owner_rule),
                engine.call_map.get(call_node).cloned(),
            ) {
                let _ = source_call_graph
                    .entry(owner.clone())
                    .or_default()
                    .insert(target);
            }
        }
        let mut bindings = BTreeMap::new();
        let mut diagnostics = unresolved_calls;
        for ((node, ordinal), (span, _)) in &expected.bindings {
            let id = RuleBindingIdV2::new(source_id, *node, *ordinal);
            let domain_len = engine
                .domains
                .get(&Variable::Binding(id))
                .ok_or(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: *span,
                    },
                })?
                .len();
            engine.charge_many("steps", domain_len, *span)?;
            let domain = engine
                .domains
                .get(&Variable::Binding(id))
                .ok_or(TypeDiagnosticV2::InvalidRuleInput {
                    location: LocatedSpanV2 {
                        source_id,
                        span: *span,
                    },
                })?
                .clone();
            if domain.len() != 1 {
                if *ordinal == 0 {
                    if let Some(predicate) = predicate_map.get(node) {
                        if engine.is_recursive(predicate, &source_call_graph, *span)? {
                            diagnostics.push((
                                *span,
                                TypeDiagnosticV2::UnanchoredRecursiveRule {
                                    location: LocatedSpanV2 {
                                        source_id,
                                        span: *span,
                                    },
                                },
                            ));
                        } else {
                            diagnostics.push((
                                *span,
                                TypeDiagnosticV2::AmbiguousRuleType {
                                    location: LocatedSpanV2 {
                                        source_id,
                                        span: *span,
                                    },
                                    candidates: domain.len(),
                                },
                            ));
                        }
                    } else {
                        diagnostics.push((
                            *span,
                            TypeDiagnosticV2::AmbiguousRuleType {
                                location: LocatedSpanV2 {
                                    source_id,
                                    span: *span,
                                },
                                candidates: domain.len(),
                            },
                        ));
                    }
                } else {
                    diagnostics.push((
                        *span,
                        TypeDiagnosticV2::AmbiguousRuleType {
                            location: LocatedSpanV2 {
                                source_id,
                                span: *span,
                            },
                            candidates: domain.len(),
                        },
                    ));
                }
                let _ = bindings.insert(
                    id,
                    ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnanchoredRuleVariable),
                );
            } else if let Some(ty) = domain.iter().next() {
                let _ = bindings.insert(id, ExpressionTypeStateV2::Known(ty.clone()));
            }
        }
        let predicates = engine
            .predicate_bindings
            .iter()
            .filter_map(|(id, binding)| bindings.get(binding).map(|ty| (id.clone(), ty.clone())))
            .collect();
        // An unresolved callable is the missing authority that may have anchored
        // otherwise ambiguous domains. Report that primary cause first.
        diagnostics.sort_by_key(|(span, diagnostic)| {
            (
                !matches!(diagnostic, TypeDiagnosticV2::UnknownRuleCallable { .. }),
                span.start,
                span.end,
                diagnostic.code(),
            )
        });
        let diagnostics = diagnostics
            .into_iter()
            .map(|(_, diagnostic)| diagnostic)
            .collect();
        Ok(RuleTypeAnalysisV2 {
            bindings,
            predicates,
            diagnostics,
            usage: engine.usage,
        })
    }
}
