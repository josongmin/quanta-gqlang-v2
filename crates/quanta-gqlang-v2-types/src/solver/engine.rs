//! Candidate construction and finite-domain propagation.

use super::{
    charge_prepass_steps, type_shape_cost, BTreeMap, BTreeSet, BoolExprHirV1, BoolExprKindV1,
    BuiltinRegistryV2, CallArgHirV1, CallableSignatureV2, Constraint, ExtendedSourceRegistryViewV2,
    HirNodeIdV1, HirVisitorV1, LiteralV1, LocatedSpanV2, OperatorRegistryV2, RecordFieldTypeV1,
    RecordTypeV1, ResolvedPredicateIdV2, RuleBindingIdV2, RuleCallableTargetV2, RuleTypeBudgetV2,
    RuleTypeUsageV2, SourceIdV2, SourceQualifiedNodeIdV2, SpanV1, TypeDiagnosticV2, ValueExprHirV1,
    ValueExprKindV1, ValueTypeV1, Variable,
};

/// Candidate constructed types are derived once, before worklist allocation.
/// This is finite-domain generation from checked HIR, not overload selection.
pub(super) struct ConstructedCandidateTypes<'a> {
    source_id: SourceIdV2,
    registry: &'a ExtendedSourceRegistryViewV2,
    budget: RuleTypeBudgetV2,
    pub(super) steps: u32,
    base: BTreeSet<ValueTypeV1>,
    possible: BTreeMap<HirNodeIdV1, BTreeSet<ValueTypeV1>>,
    pub(super) candidates: BTreeSet<ValueTypeV1>,
    pub(super) error: Option<TypeDiagnosticV2>,
}

impl<'a> ConstructedCandidateTypes<'a> {
    pub(super) fn new(
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
    pub(super) fn charge_many(&mut self, span: SpanV1, amount: usize) {
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
                        Err(TypeDiagnosticV2::TypeShapeLimit { .. }) => {}
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

pub(super) struct Engine<'a> {
    pub(super) source_id: SourceIdV2,
    pub(super) budget: RuleTypeBudgetV2,
    pub(super) usage: RuleTypeUsageV2,
    pub(super) universe: BTreeSet<ValueTypeV1>,
    pub(super) domains: BTreeMap<Variable, BTreeSet<ValueTypeV1>>,
    pub(super) constraints: Vec<Constraint>,
    pub(super) symbol_map: BTreeMap<HirNodeIdV1, RuleBindingIdV2>,
    pub(super) call_map: BTreeMap<HirNodeIdV1, RuleCallableTargetV2>,
    pub(super) predicate_bindings: BTreeMap<ResolvedPredicateIdV2, RuleBindingIdV2>,
    pub(super) operators: &'a OperatorRegistryV2,
    pub(super) builtins: &'a BuiltinRegistryV2,
}

impl Engine<'_> {
    pub(super) fn is_recursive(
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
    pub(super) fn charge(
        &mut self,
        dimension: &'static str,
        span: SpanV1,
    ) -> Result<(), TypeDiagnosticV2> {
        self.charge_many(dimension, 1, span)
    }
    pub(super) fn reserve_materialization(
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
    pub(super) fn charge_many(
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
    pub(super) fn intern(
        &mut self,
        variable: Variable,
        span: SpanV1,
    ) -> Result<Variable, TypeDiagnosticV2> {
        if !self.domains.contains_key(&variable) {
            self.charge("nodes", span)?;
            self.charge_many("steps", type_shape_cost(&self.universe), span)?;
            let _ = self.domains.insert(variable.clone(), self.universe.clone());
        }
        Ok(variable)
    }
    pub(super) fn add(
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
    #[expect(clippy::too_many_lines, reason = "legacy iterative rule value walk")]
    pub(super) fn value(&mut self, node: &ValueExprHirV1) -> Result<Variable, TypeDiagnosticV2> {
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
    pub(super) fn boolean(&mut self, node: &BoolExprHirV1) -> Result<Variable, TypeDiagnosticV2> {
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
    pub(super) fn propagate(&mut self) -> Result<(), TypeDiagnosticV2> {
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
