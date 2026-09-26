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
        let source_predicates = predicate_map.values().cloned().collect::<BTreeSet<_>>();
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

        let mut prepass_steps = expected.steps;
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
        let mut engine = Engine {
            source_id,
            budget,
            usage: RuleTypeUsageV2 {
                steps: constructed.steps,
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

/// Candidate constructed types are derived once, before worklist allocation.
/// This is finite-domain generation from checked HIR, not overload selection.
struct ConstructedCandidateTypes<'a> {
    source_id: SourceIdV2,
    registry: &'a ExtendedSourceRegistryViewV2,
    budget: RuleTypeBudgetV2,
    steps: u32,
    base: BTreeSet<ValueTypeV1>,
    possible: BTreeMap<HirNodeIdV1, BTreeSet<ValueTypeV1>>,
    candidates: BTreeSet<ValueTypeV1>,
    error: Option<TypeDiagnosticV2>,
}

impl<'a> ConstructedCandidateTypes<'a> {
    fn new(
        source_id: SourceIdV2,
        registry: &'a ExtendedSourceRegistryViewV2,
        budget: RuleTypeBudgetV2,
        steps: u32,
        base: BTreeSet<ValueTypeV1>,
    ) -> Self {
        Self {
            source_id,
            registry,
            budget,
            steps,
            base,
            possible: BTreeMap::new(),
            candidates: BTreeSet::new(),
            error: None,
        }
    }
    fn charge(&mut self, span: SpanV1) {
        self.charge_many(span, 1);
    }
    fn charge_many(&mut self, span: SpanV1, amount: usize) {
        if self.error.is_none() {
            if let Err(error) =
                charge_prepass_steps(&mut self.steps, self.budget, self.source_id, span, amount)
            {
                self.error = Some(error);
            }
        }
    }
    fn clone_candidates(&mut self, id: HirNodeIdV1, span: SpanV1) -> BTreeSet<ValueTypeV1> {
        let amount = type_shape_cost(self.possible.get(&id).unwrap_or(&self.base));
        self.charge_many(span, amount);
        if self.error.is_some() {
            return BTreeSet::new();
        }
        self.possible.get(&id).unwrap_or(&self.base).clone()
    }
}

impl HirVisitorV1 for ConstructedCandidateTypes<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        if self.error.is_some() {
            return;
        }
        self.charge(node.span);
        if self.error.is_some() {
            return;
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
        if self.error.is_some() {
            return;
        }
        let possible = match &node.kind {
            ValueExprKindV1::Literal(literal) => Some(BTreeSet::from([match literal {
                LiteralV1::Int(_) => ValueTypeV1::int(),
                LiteralV1::Float(_) => ValueTypeV1::float(),
                LiteralV1::Str(_) => ValueTypeV1::string(),
                LiteralV1::Bool(_) => ValueTypeV1::bool(),
                LiteralV1::Null => ValueTypeV1::null(),
            }])),
            ValueExprKindV1::Paren(inner) => self
                .possible
                .contains_key(&inner.id)
                .then(|| self.clone_candidates(inner.id, node.span)),
            ValueExprKindV1::List(list) => {
                let mut common = if let Some(first) = list.elements.first() {
                    self.clone_candidates(first.id, node.span)
                } else {
                    self.charge_many(node.span, type_shape_cost(&self.base));
                    if self.error.is_some() {
                        return;
                    }
                    self.base.clone()
                };
                if self.error.is_some() {
                    return;
                }
                for element in list.elements.iter().skip(1) {
                    self.charge_many(node.span, type_shape_cost(&common));
                    if self.error.is_some() {
                        return;
                    }
                    let options = self.possible.get(&element.id).unwrap_or(&self.base);
                    common.retain(|ty| options.contains(ty));
                }
                let mut lists = BTreeSet::new();
                for ty in common {
                    self.charge_many(node.span, ty.node_count());
                    if self.error.is_some() {
                        return;
                    }
                    if let Ok(list_type) = ValueTypeV1::try_list(ty) {
                        let _ = lists.insert(list_type);
                    }
                }
                self.charge_many(node.span, type_shape_cost(&lists));
                if self.error.is_some() {
                    return;
                }
                self.candidates.extend(lists.iter().cloned());
                Some(lists)
            }
            ValueExprKindV1::Record(record) => {
                let mut rows = vec![Vec::new()];
                let mut names = Vec::new();
                for field in &record.fields {
                    let options = self.clone_candidates(field.value.id, field.name_span);
                    if self.error.is_some() {
                        return;
                    }
                    let location = match self.registry.try_check_location(LocatedSpanV2 {
                        source_id: self.source_id,
                        span: field.name_span,
                    }) {
                        Ok(location) => location,
                        Err(_source_error) => {
                            self.error = Some(TypeDiagnosticV2::SourceRegistryMismatch {
                                location: LocatedSpanV2 {
                                    source_id: self.source_id,
                                    span: field.name_span,
                                },
                            });
                            return;
                        }
                    };
                    names.push((field.name.as_str(), location));
                    let mut next = Vec::new();
                    for row in rows {
                        for ty in &options {
                            self.charge_many(
                                field.name_span,
                                type_shape_cost(&row)
                                    .saturating_add(ty.node_count())
                                    .saturating_add(1),
                            );
                            if self.error.is_some() {
                                return;
                            }
                            let mut extended = row.clone();
                            extended.push(ty.clone());
                            next.push(extended);
                        }
                    }
                    rows = next;
                }
                let mut records = BTreeSet::new();
                for row in rows {
                    self.charge_many(node.span, type_shape_cost(&row).saturating_add(1));
                    if self.error.is_some() {
                        return;
                    }
                    let fields = names
                        .iter()
                        .zip(row)
                        .map(|((name, location), ty)| {
                            RecordFieldTypeV1::try_new(name, ty, location.clone())
                        })
                        .collect::<Result<Vec<_>, _>>();
                    let ty = fields
                        .and_then(RecordTypeV1::try_new)
                        .and_then(ValueTypeV1::try_record);
                    match ty {
                        Ok(ty) => {
                            let _ = records.insert(ty);
                        }
                        Err(TypeDiagnosticV2::TypeShapeLimit) => {}
                        Err(error) => {
                            self.error = Some(error);
                            return;
                        }
                    }
                }
                self.charge_many(node.span, type_shape_cost(&records));
                if self.error.is_some() {
                    return;
                }
                self.candidates.extend(records.iter().cloned());
                Some(records)
            }
            ValueExprKindV1::Symbol(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Call(_) => None,
        };
        if let Some(possible) = possible {
            let _ = self.possible.insert(node.id, possible);
        }
    }
}

struct Engine<'a> {
    source_id: SourceIdV2,
    budget: RuleTypeBudgetV2,
    usage: RuleTypeUsageV2,
    universe: BTreeSet<ValueTypeV1>,
    domains: BTreeMap<Variable, BTreeSet<ValueTypeV1>>,
    constraints: Vec<Constraint>,
    symbol_map: BTreeMap<HirNodeIdV1, RuleBindingIdV2>,
    call_map: BTreeMap<HirNodeIdV1, RuleCallableTargetV2>,
    predicate_bindings: BTreeMap<ResolvedPredicateIdV2, RuleBindingIdV2>,
    operators: &'a OperatorRegistryV2,
    builtins: &'a BuiltinRegistryV2,
}

impl Engine<'_> {
    fn is_recursive(
        &mut self,
        predicate: &ResolvedPredicateIdV2,
        graph: &BTreeMap<ResolvedPredicateIdV2, BTreeSet<ResolvedPredicateIdV2>>,
        span: SpanV1,
    ) -> Result<bool, TypeDiagnosticV2> {
        self.charge_many("steps", graph.get(predicate).map_or(0, BTreeSet::len), span)?;
        let mut pending = graph.get(predicate).cloned().unwrap_or_default();
        let mut seen = BTreeSet::new();
        while let Some(next) = pending.pop_first() {
            self.charge("steps", span)?;
            if &next == predicate {
                return Ok(true);
            }
            if seen.insert(next.clone()) {
                if let Some(edges) = graph.get(&next) {
                    self.charge_many("steps", edges.len(), span)?;
                    pending.extend(edges.iter().cloned());
                }
            }
        }
        Ok(false)
    }
    fn location(&self, span: SpanV1) -> LocatedSpanV2 {
        LocatedSpanV2 {
            source_id: self.source_id,
            span,
        }
    }
    fn charge(&mut self, dimension: &'static str, span: SpanV1) -> Result<(), TypeDiagnosticV2> {
        self.charge_many(dimension, 1, span)
    }
    fn reserve_materialization(
        &mut self,
        rows: usize,
        width: usize,
        span: SpanV1,
    ) -> Result<(), TypeDiagnosticV2> {
        if self.usage.edges >= self.budget.edges.get() {
            return Err(TypeDiagnosticV2::RuleTypeResourceLimit {
                dimension: "edges",
                location: self.location(span),
            });
        }
        let cells = rows
            .checked_mul(width)
            .ok_or(TypeDiagnosticV2::RuleTypeResourceLimit {
                dimension: "steps",
                location: self.location(span),
            })?;
        self.charge_many("steps", cells, span)
    }
    fn charge_many(
        &mut self,
        dimension: &'static str,
        amount: usize,
        span: SpanV1,
    ) -> Result<(), TypeDiagnosticV2> {
        let (count, limit) = match dimension {
            "nodes" => (&mut self.usage.nodes, self.budget.nodes.get()),
            "edges" => (&mut self.usage.edges, self.budget.edges.get()),
            _ => (&mut self.usage.steps, self.budget.steps.get()),
        };
        let amount = u32::try_from(amount).map_err(|_conversion_error| {
            TypeDiagnosticV2::RuleTypeResourceLimit {
                dimension,
                location: LocatedSpanV2 {
                    source_id: self.source_id,
                    span,
                },
            }
        })?;
        *count = count
            .checked_add(amount)
            .ok_or(TypeDiagnosticV2::RuleTypeResourceLimit {
                dimension,
                location: LocatedSpanV2 {
                    source_id: self.source_id,
                    span,
                },
            })?;
        if *count > limit {
            return Err(TypeDiagnosticV2::RuleTypeResourceLimit {
                dimension,
                location: LocatedSpanV2 {
                    source_id: self.source_id,
                    span,
                },
            });
        }
        Ok(())
    }
    fn intern(&mut self, variable: Variable, span: SpanV1) -> Result<Variable, TypeDiagnosticV2> {
        if !self.domains.contains_key(&variable) {
            self.charge("nodes", span)?;
            self.charge_many("steps", type_shape_cost(&self.universe), span)?;
            let _ = self.domains.insert(variable.clone(), self.universe.clone());
        }
        Ok(variable)
    }
    fn add(
        &mut self,
        variables: Vec<Variable>,
        allowed: Vec<Vec<ValueTypeV1>>,
        span: SpanV1,
    ) -> Result<(), TypeDiagnosticV2> {
        if allowed.iter().any(|tuple| tuple.len() != variables.len()) {
            return Err(TypeDiagnosticV2::InvalidRuleInput {
                location: self.location(span),
            });
        }
        self.charge("edges", span)?;
        self.charge_many("steps", allowed.len(), span)?;
        if allowed.is_empty() {
            return Err(TypeDiagnosticV2::ContradictoryRuleType {
                location: self.location(span),
            });
        }
        self.constraints.push(Constraint {
            variables,
            allowed,
            location: self.location(span),
        });
        Ok(())
    }
    fn fixed(
        &mut self,
        variable: Variable,
        ty: ValueTypeV1,
        span: SpanV1,
    ) -> Result<(), TypeDiagnosticV2> {
        self.add(vec![variable], vec![vec![ty]], span)
    }
    fn equal(
        &mut self,
        left: Variable,
        right: Variable,
        span: SpanV1,
    ) -> Result<(), TypeDiagnosticV2> {
        self.reserve_materialization(self.universe.len(), 2, span)?;
        let tuples = self
            .universe
            .iter()
            .cloned()
            .map(|ty| vec![ty.clone(), ty])
            .collect();
        self.add(vec![left, right], tuples, span)
    }
    fn value(&mut self, node: &ValueExprHirV1) -> Result<Variable, TypeDiagnosticV2> {
        let var = self.intern(
            Variable::Value(SourceQualifiedNodeIdV2 {
                source_id: self.source_id,
                node_id: node.id,
            }),
            node.span,
        )?;
        match &node.kind {
            ValueExprKindV1::Literal(lit) => {
                let ty = match lit {
                    LiteralV1::Int(_) => ValueTypeV1::int(),
                    LiteralV1::Float(_) => ValueTypeV1::float(),
                    LiteralV1::Str(_) => ValueTypeV1::string(),
                    LiteralV1::Bool(_) => ValueTypeV1::bool(),
                    LiteralV1::Null => ValueTypeV1::null(),
                };
                self.fixed(var.clone(), ty, node.span)?;
            }
            ValueExprKindV1::Symbol(_) => {
                let binding = self.symbol_map.get(&node.id).copied().ok_or(
                    TypeDiagnosticV2::InvalidRuleInput {
                        location: self.location(node.span),
                    },
                )?;
                self.equal(var.clone(), Variable::Binding(binding), node.span)?;
            }
            ValueExprKindV1::Paren(inner) => {
                let child = self.value(inner)?;
                self.equal(var.clone(), child, node.span)?;
            }
            ValueExprKindV1::Unary(unary) => {
                let operand = self.value(&unary.operand)?;
                self.reserve_materialization(
                    self.operators.unary_overloads(&unary.op.symbol).len(),
                    2,
                    node.span,
                )?;
                let tuples = self
                    .operators
                    .unary_overloads(&unary.op.symbol)
                    .iter()
                    .map(|op| vec![op.operand.clone(), op.result.clone()])
                    .collect();
                self.add(vec![operand, var.clone()], tuples, node.span)?;
            }
            ValueExprKindV1::Binary(binary) => {
                let lhs = self.value(&binary.lhs)?;
                let rhs = self.value(&binary.rhs)?;
                self.reserve_materialization(
                    self.operators.binary_overloads(&binary.op.symbol).len(),
                    3,
                    node.span,
                )?;
                let tuples = self
                    .operators
                    .binary_overloads(&binary.op.symbol)
                    .iter()
                    .map(|op| vec![op.lhs.clone(), op.rhs.clone(), op.result.clone()])
                    .collect();
                self.add(vec![lhs, rhs, var.clone()], tuples, node.span)?;
            }
            ValueExprKindV1::Call(call) => {
                let target_cost = match self.call_map.get(&node.id) {
                    Some(RuleCallableTargetV2::HostPredicate(signature)) => type_shape_cost(
                        signature
                            .params()
                            .iter()
                            .chain(std::iter::once(signature.result())),
                    ),
                    Some(RuleCallableTargetV2::Builtin(_))
                    | Some(RuleCallableTargetV2::SourcePredicate(_))
                    | Some(RuleCallableTargetV2::Unknown) => 1,
                    None => {
                        return Err(TypeDiagnosticV2::InvalidRuleInput {
                            location: self.location(node.span),
                        })
                    }
                };
                self.charge_many("steps", target_cost, node.span)?;
                let target = self.call_map.get(&node.id).cloned().ok_or(
                    TypeDiagnosticV2::InvalidRuleInput {
                        location: self.location(node.span),
                    },
                )?;
                let mut args = Vec::new();
                let mut star = false;
                for arg in call.args().as_slice() {
                    match arg {
                        CallArgHirV1::Expr(expr) => args.push(self.value(expr)?),
                        CallArgHirV1::Star { .. } => star = true,
                    }
                }
                let mut tuples = Vec::new();
                let mut unresolved = false;
                match target {
                    RuleCallableTargetV2::Builtin(family) => {
                        for entry in self.builtins.lookup_family(family) {
                            match &entry.signature {
                                CallableSignatureV2::Fixed { params, result }
                                    if !star && params.len() == args.len() =>
                                {
                                    self.reserve_materialization(
                                        1,
                                        params.len().saturating_add(1),
                                        node.span,
                                    )?;
                                    let mut row = params.clone();
                                    row.push(result.clone());
                                    tuples.push(row);
                                }
                                CallableSignatureV2::AnySingle { result }
                                    if !star && args.len() == 1 =>
                                {
                                    self.reserve_materialization(
                                        self.universe.len(),
                                        2,
                                        node.span,
                                    )?;
                                    for ty in &self.universe {
                                        tuples.push(vec![ty.clone(), result.clone()]);
                                    }
                                }
                                CallableSignatureV2::AggregateStar { result }
                                    if star && args.is_empty() =>
                                {
                                    self.reserve_materialization(1, 1, node.span)?;
                                    tuples.push(vec![result.clone()])
                                }
                                CallableSignatureV2::Fixed { .. }
                                | CallableSignatureV2::AnySingle { .. }
                                | CallableSignatureV2::AggregateStar { .. } => {}
                            }
                        }
                    }
                    RuleCallableTargetV2::HostPredicate(signature) => {
                        if !star && signature.params().len() == args.len() {
                            self.reserve_materialization(
                                1,
                                signature.params().len().saturating_add(1),
                                node.span,
                            )?;
                            let mut row = signature.params().to_vec();
                            row.push(signature.result().clone());
                            tuples.push(row);
                        }
                    }
                    RuleCallableTargetV2::SourcePredicate(id) => {
                        if !star && args.len() == 1 {
                            let binding = self.predicate_bindings.get(&id).copied().ok_or(
                                TypeDiagnosticV2::UnknownRuleCallable {
                                    location: self.location(node.span),
                                },
                            )?;
                            self.reserve_materialization(self.universe.len(), 3, node.span)?;
                            for ty in &self.universe {
                                tuples.push(vec![ty.clone(), ty.clone(), ValueTypeV1::bool()]);
                            }
                            args.push(Variable::Binding(binding));
                        }
                    }
                    RuleCallableTargetV2::Unknown => {
                        unresolved = true;
                    }
                }
                if !unresolved {
                    args.push(var.clone());
                    self.add(args, tuples, node.span)?;
                }
            }
            ValueExprKindV1::Path(path) => {
                let base = self.value(&path.base)?;
                self.reserve_materialization(
                    self.universe.len(),
                    path.segments.as_slice().len().saturating_add(2),
                    node.span,
                )?;
                let tuples = self
                    .universe
                    .iter()
                    .filter_map(|ty| {
                        let mut current = ty.clone();
                        for segment in path.segments.as_slice() {
                            let record = current.as_record()?;
                            current = record.field(&segment.name)?.value_type().clone();
                        }
                        Some(vec![ty.clone(), current])
                    })
                    .collect();
                self.add(vec![base, var.clone()], tuples, node.span)?;
            }
            ValueExprKindV1::Record(record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|field| self.value(&field.value))
                    .collect::<Result<Vec<_>, _>>()?;
                self.reserve_materialization(
                    self.universe.len(),
                    record.fields.len().saturating_add(1),
                    node.span,
                )?;
                let tuples = self
                    .universe
                    .iter()
                    .filter_map(|ty| {
                        let schema = ty.as_record()?;
                        if schema.fields().len() != record.fields.len() {
                            return None;
                        }
                        let mut row = Vec::with_capacity(record.fields.len() + 1);
                        for field in &record.fields {
                            row.push(schema.field(&field.name)?.value_type().clone());
                        }
                        row.push(ty.clone());
                        Some(row)
                    })
                    .collect();
                let mut variables = fields;
                variables.push(var.clone());
                self.add(variables, tuples, node.span)?;
            }
            ValueExprKindV1::List(list) => {
                let elements = list
                    .elements
                    .iter()
                    .map(|element| self.value(element))
                    .collect::<Result<Vec<_>, _>>()?;
                self.reserve_materialization(
                    self.universe.len(),
                    list.elements.len().saturating_add(1),
                    node.span,
                )?;
                let tuples = self
                    .universe
                    .iter()
                    .filter_map(|ty| {
                        let collection = ty.as_collection()?;
                        if collection.cardinality() != crate::CardinalityV1::Sequence {
                            return None;
                        }
                        let mut row = vec![collection.element().clone(); list.elements.len()];
                        row.push(ty.clone());
                        Some(row)
                    })
                    .collect();
                let mut variables = elements;
                variables.push(var.clone());
                self.add(variables, tuples, node.span)?;
            }
        }
        Ok(var)
    }
    fn boolean(&mut self, node: &BoolExprHirV1) -> Result<Variable, TypeDiagnosticV2> {
        let var = self.intern(
            Variable::Bool(SourceQualifiedNodeIdV2 {
                source_id: self.source_id,
                node_id: node.id,
            }),
            node.span,
        )?;
        self.fixed(var.clone(), ValueTypeV1::bool(), node.span)?;
        match &node.kind {
            BoolExprKindV1::Literal(_) => {}
            BoolExprKindV1::Value(value) => {
                let child = self.value(value)?;
                self.equal(child, var.clone(), node.span)?;
            }
            BoolExprKindV1::Compare(compare) => {
                let lhs = self.value(&compare.lhs)?;
                let rhs = self.value(&compare.rhs)?;
                self.reserve_materialization(
                    self.operators.binary_overloads(&compare.op.symbol).len(),
                    3,
                    node.span,
                )?;
                let tuples = self
                    .operators
                    .binary_overloads(&compare.op.symbol)
                    .iter()
                    .filter(|op| op.result == ValueTypeV1::bool())
                    .map(|op| vec![op.lhs.clone(), op.rhs.clone(), op.result.clone()])
                    .collect();
                self.add(vec![lhs, rhs, var.clone()], tuples, node.span)?;
            }
            BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
                let lhs = self.boolean(&binary.lhs)?;
                let rhs = self.boolean(&binary.rhs)?;
                self.add(
                    vec![lhs, rhs, var.clone()],
                    vec![vec![ValueTypeV1::bool(); 3]],
                    node.span,
                )?;
            }
            BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                let child = self.boolean(inner)?;
                self.equal(child, var.clone(), node.span)?;
            }
            BoolExprKindV1::LogicalQuantifier(quant) => {
                let child = self.boolean(&quant.body)?;
                self.equal(child, var.clone(), node.span)?;
            }
            BoolExprKindV1::RelationExists(_) => {
                return Err(TypeDiagnosticV2::RelationExistsInRule {
                    location: self.location(node.span),
                })
            }
        }
        Ok(var)
    }
    fn propagate(&mut self) -> Result<(), TypeDiagnosticV2> {
        let mut pending = (0..self.constraints.len()).collect::<BTreeSet<_>>();
        while let Some(index) = pending.pop_first() {
            let location = self
                .constraints
                .get(index)
                .ok_or(TypeDiagnosticV2::InvalidRuleInput {
                    location: self.location(SpanV1::point(0)),
                })?
                .location;
            self.charge("steps", location.span)?;
            let constraint = self
                .constraints
                .get(index)
                .ok_or(TypeDiagnosticV2::InvalidRuleInput { location })?;
            let scan_cells = constraint
                .allowed
                .len()
                .checked_mul(constraint.variables.len().saturating_add(1))
                .ok_or(TypeDiagnosticV2::RuleTypeResourceLimit {
                    dimension: "steps",
                    location,
                })?;
            self.charge_many("steps", scan_cells, location.span)?;
            let constraint = self
                .constraints
                .get(index)
                .ok_or(TypeDiagnosticV2::InvalidRuleInput { location })?;
            let viable = constraint
                .allowed
                .iter()
                .filter(|tuple| {
                    tuple.iter().zip(&constraint.variables).all(|(ty, var)| {
                        self.domains
                            .get(var)
                            .is_some_and(|domain| domain.contains(ty))
                    })
                })
                .collect::<Vec<_>>();
            if viable.is_empty() {
                return Err(TypeDiagnosticV2::ContradictoryRuleType { location });
            }
            let mut changed = BTreeSet::new();
            for (position, variable) in constraint.variables.iter().enumerate() {
                let supported = viable
                    .iter()
                    .filter_map(|tuple| tuple.get(position).cloned())
                    .collect::<BTreeSet<_>>();
                let domain = self
                    .domains
                    .get_mut(variable)
                    .ok_or(TypeDiagnosticV2::InvalidRuleInput { location })?;
                let old_len = domain.len();
                domain.retain(|ty| supported.contains(ty));
                if domain.len() != old_len {
                    let _ = changed.insert(variable.clone());
                }
            }
            if !changed.is_empty() {
                for (other_index, other) in self.constraints.iter().enumerate() {
                    if other_index != index
                        && other.variables.iter().any(|var| changed.contains(var))
                    {
                        let _ = pending.insert(other_index);
                    }
                }
            }
        }
        Ok(())
    }
}
