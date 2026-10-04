//! Opaque algebra nodes backed by sealed semantic contracts.

use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, CompositionInstanceV2, JoinModeV2, OrderKeyContractV2,
    OutputColumnOriginV2, PartitionInputBindingV2, RelationExistsPolarityV2,
    RelationOutputContractV2, ResolvedScanSourceV2, SemanticItemIdV2, TypedBoolExprV2,
    TypedCorrelationEdgeV2, TypedExprV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, ExpressionTypeStateV2, SourceQualifiedNodeIdV2,
};

/// Closed relational operator vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlgebraFamilyV2 {
    /// Authoritative source or View scan.
    Scan,
    /// Predicate restriction.
    Filter,
    /// Row-frame projection.
    Project,
    /// Inner or explicit cross join.
    Join,
    /// Source grouping stage.
    Group,
    /// Aggregate folds over a Group.
    Aggregate,
    /// Source ordering stage.
    Sort,
    /// Positive relational existence.
    SemiJoin,
    /// Negative relational existence.
    AntiJoin,
    /// Several relational existence conjuncts over one input frame.
    ExistsFilter,
}

impl AlgebraFamilyV2 {
    /// Stable public spelling.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Filter => "filter",
            Self::Project => "project",
            Self::Join => "join",
            Self::Group => "group",
            Self::Aggregate => "aggregate",
            Self::Sort => "sort",
            Self::SemiJoin => "semi_join",
            Self::AntiJoin => "anti_join",
            Self::ExistsFilter => "exists_filter",
        }
    }
}

/// Exact, duplicate-free correlations retained by a Semi/Anti join.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationSetV2(Vec<TypedCorrelationEdgeV2>);

impl CorrelationSetV2 {
    pub(crate) fn try_new(edges: Vec<TypedCorrelationEdgeV2>) -> Result<Self, ()> {
        let mut references = std::collections::BTreeSet::new();
        if edges
            .iter()
            .any(|edge| !references.insert(edge.reference()))
        {
            return Err(());
        }
        Ok(Self(edges))
    }

    /// Correlation edges in the semantic document's stable order.
    pub fn iter(&self) -> impl Iterator<Item = &TypedCorrelationEdgeV2> {
        self.0.iter()
    }

    /// Whether the nested relation is uncorrelated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One authored existential condition and its checked nested relation.
/// Keeping the pair together prevents condition/right drift before projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistentialTermV2 {
    expression: SourceQualifiedNodeIdV2,
    polarity: RelationExistsPolarityV2,
    correlations: CorrelationSetV2,
    right: Box<CheckedAlgebraNodeV2>,
}

impl ExistentialTermV2 {
    pub(crate) fn new(
        expression: SourceQualifiedNodeIdV2,
        polarity: RelationExistsPolarityV2,
        correlations: CorrelationSetV2,
        right: CheckedAlgebraNodeV2,
    ) -> Self {
        Self {
            expression,
            polarity,
            correlations,
            right: Box::new(right),
        }
    }

    #[must_use]
    pub const fn expression(&self) -> SourceQualifiedNodeIdV2 {
        self.expression
    }
    #[must_use]
    pub const fn polarity(&self) -> RelationExistsPolarityV2 {
        self.polarity
    }
    #[must_use]
    pub const fn correlations(&self) -> &CorrelationSetV2 {
        &self.correlations
    }
    #[must_use]
    pub fn right(&self) -> &CheckedAlgebraNodeV2 {
        self.right.as_ref()
    }
}

/// One fully lowered relation item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweredRelationItemV2 {
    item_id: SemanticItemIdV2,
    root: CheckedAlgebraNodeV2,
    output: RelationOutputContractV2,
}

impl LoweredRelationItemV2 {
    pub(crate) fn new(item_id: SemanticItemIdV2, root: CheckedAlgebraNodeV2) -> Self {
        let output = root.output().clone();
        Self {
            item_id,
            root,
            output,
        }
    }

    /// Exact source item selected for lowering.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// Checked algebra root.
    #[must_use]
    pub const fn root(&self) -> &CheckedAlgebraNodeV2 {
        &self.root
    }

    /// Exact root output contract, retained as an item-level convenience.
    #[must_use]
    pub const fn output(&self) -> &RelationOutputContractV2 {
        &self.output
    }
}

/// Algebra node with private construction and immutable semantic contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedAlgebraNodeV2 {
    item_id: SemanticItemIdV2,
    stage_key: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
    output: RelationOutputContractV2,
    op: AlgebraOpV2,
    composition_origin: Option<(CompositionInstanceV2, SourceQualifiedNodeIdV2)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AlgebraOpV2 {
    Scan {
        source: ResolvedScanSourceV2,
    },
    Filter {
        input: Box<CheckedAlgebraNodeV2>,
        conjuncts: Vec<TypedBoolExprV2>,
    },
    Project {
        input: Box<CheckedAlgebraNodeV2>,
        expressions: Vec<TypedExprV2>,
    },
    Join {
        left: Box<CheckedAlgebraNodeV2>,
        right: Box<CheckedAlgebraNodeV2>,
        mode: JoinModeV2,
        predicate: Option<TypedBoolExprV2>,
    },
    Group {
        input: Box<CheckedAlgebraNodeV2>,
        keys: Vec<TypedExprV2>,
        partition_input: Vec<PartitionInputBindingV2>,
    },
    Aggregate {
        input: Box<CheckedAlgebraNodeV2>,
        expressions: Vec<TypedExprV2>,
    },
    Sort {
        input: Box<CheckedAlgebraNodeV2>,
        keys: Vec<OrderKeyContractV2>,
    },
    SemiJoin {
        left: Box<CheckedAlgebraNodeV2>,
        right: Box<CheckedAlgebraNodeV2>,
        correlations: CorrelationSetV2,
    },
    AntiJoin {
        left: Box<CheckedAlgebraNodeV2>,
        right: Box<CheckedAlgebraNodeV2>,
        correlations: CorrelationSetV2,
    },
    ExistsFilter {
        input: Box<CheckedAlgebraNodeV2>,
        terms: Vec<ExistentialTermV2>,
    },
}

impl CheckedAlgebraNodeV2 {
    /// Assemble one node from a sealed stage contract and a lowered operator.
    ///
    /// Construction checks source identity and family-specific output
    /// relationships: retained frames, exact join operands, and computed
    /// field identities/types. Composed expression effects remain the sealed
    /// semantic fact, with aggregate class checked for aggregate stages.
    /// Lexical binding and complete EXISTS capture sets are Binder/Sema
    /// authority; normalization validates and carries those facts with their
    /// registered nested roots before calling this constructor. This is not
    /// an independent reconstruction of lexical capture completeness.
    pub(crate) fn new(
        item_id: SemanticItemIdV2,
        stage_key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        output: RelationOutputContractV2,
        op: AlgebraOpV2,
    ) -> Result<Self, ()> {
        if item_id.source_id() != stage_key.source_id
            || location.location().source_id != stage_key.source_id
            || !op_agrees_with_output(&op, &output, stage_key)
        {
            return Err(());
        }
        Ok(Self {
            item_id,
            stage_key,
            location,
            output,
            op,
            composition_origin: None,
        })
    }

    /// Bind this lowered nested root to its owner and original semantic root.
    /// Canonical rewrites may change `stage_key`, so it cannot serve as this proof.
    pub(crate) fn with_composition_origin(
        mut self,
        instance: CompositionInstanceV2,
        root_key: SourceQualifiedNodeIdV2,
    ) -> Self {
        self.composition_origin = Some((instance, root_key));
        self
    }

    /// Whether this algebra root came from this exact composition and body root.
    #[must_use]
    pub fn matches_composition_origin(
        &self,
        instance: &CompositionInstanceV2,
        root_key: SourceQualifiedNodeIdV2,
    ) -> bool {
        self.composition_origin
            .as_ref()
            .is_some_and(|(minted, key)| minted.same_instance(instance) && *key == root_key)
    }

    /// Closed operator family.
    #[must_use]
    pub const fn family(&self) -> AlgebraFamilyV2 {
        match self.op {
            AlgebraOpV2::Scan { .. } => AlgebraFamilyV2::Scan,
            AlgebraOpV2::Filter { .. } => AlgebraFamilyV2::Filter,
            AlgebraOpV2::Project { .. } => AlgebraFamilyV2::Project,
            AlgebraOpV2::Join { .. } => AlgebraFamilyV2::Join,
            AlgebraOpV2::Group { .. } => AlgebraFamilyV2::Group,
            AlgebraOpV2::Aggregate { .. } => AlgebraFamilyV2::Aggregate,
            AlgebraOpV2::Sort { .. } => AlgebraFamilyV2::Sort,
            AlgebraOpV2::SemiJoin { .. } => AlgebraFamilyV2::SemiJoin,
            AlgebraOpV2::AntiJoin { .. } => AlgebraFamilyV2::AntiJoin,
            AlgebraOpV2::ExistsFilter { .. } => AlgebraFamilyV2::ExistsFilter,
        }
    }

    /// Owning semantic item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// Source-qualified semantic stage identity.
    #[must_use]
    pub const fn stage_key(&self) -> SourceQualifiedNodeIdV2 {
        self.stage_key
    }

    /// Exact checked semantic source location.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }

    /// Exact output contract supplied by semantic analysis.
    #[must_use]
    pub const fn output(&self) -> &RelationOutputContractV2 {
        &self.output
    }

    /// Child nodes in evaluation order.
    #[must_use]
    pub fn children(&self) -> Vec<&CheckedAlgebraNodeV2> {
        match &self.op {
            AlgebraOpV2::Scan { .. } => Vec::new(),
            AlgebraOpV2::Filter { input, .. }
            | AlgebraOpV2::Project { input, .. }
            | AlgebraOpV2::Group { input, .. }
            | AlgebraOpV2::Aggregate { input, .. }
            | AlgebraOpV2::Sort { input, .. } => vec![input],
            AlgebraOpV2::Join { left, right, .. }
            | AlgebraOpV2::SemiJoin { left, right, .. }
            | AlgebraOpV2::AntiJoin { left, right, .. } => vec![left, right],
            AlgebraOpV2::ExistsFilter { input, terms } => {
                let mut children = Vec::with_capacity(1 + terms.len());
                children.push(input.as_ref());
                children.extend(terms.iter().map(ExistentialTermV2::right));
                children
            }
        }
    }

    /// Resolved scan source, when this node is a scan.
    #[must_use]
    pub const fn scan_source(&self) -> Option<&ResolvedScanSourceV2> {
        match &self.op {
            AlgebraOpV2::Scan { source } => Some(source),
            AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => None,
        }
    }

    /// Typed ON predicate retained by an Inner Join. A Cross Join carries
    /// none, and a Filter exposes its ordinary conjunction through
    /// [`Self::filter_conjuncts`] instead.
    #[must_use]
    pub const fn predicate(&self) -> Option<&TypedBoolExprV2> {
        match &self.op {
            AlgebraOpV2::Join { predicate, .. } => predicate.as_ref(),
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => None,
        }
    }

    /// The full ordinary conjunction retained by a Filter, in stable authored
    /// left-to-right order. Relational EXISTS conjuncts never appear here:
    /// they lower to the Semi/Anti joins below this node.
    pub fn filter_conjuncts(&self) -> &[TypedBoolExprV2] {
        match &self.op {
            AlgebraOpV2::Filter { conjuncts, .. } => conjuncts,
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => &[],
        }
    }

    /// Source-checked join subkind.
    #[must_use]
    pub const fn join_mode(&self) -> Option<JoinModeV2> {
        match &self.op {
            AlgebraOpV2::Join { mode, .. } => Some(*mode),
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => None,
        }
    }

    /// Typed expression facts owned by Project, Group, or Aggregate.
    pub fn expressions(&self) -> &[TypedExprV2] {
        match &self.op {
            AlgebraOpV2::Project { expressions, .. }
            | AlgebraOpV2::Aggregate { expressions, .. } => expressions,
            AlgebraOpV2::Group { keys, .. } => keys,
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => &[],
        }
    }

    /// Ordering proofs owned by Sort.
    pub fn order_keys(&self) -> &[OrderKeyContractV2] {
        match &self.op {
            AlgebraOpV2::Sort { keys, .. } => keys,
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => &[],
        }
    }

    /// Binder-owned pre-group bindings retained by Group.
    pub fn partition_input(&self) -> &[PartitionInputBindingV2] {
        match &self.op {
            AlgebraOpV2::Group {
                partition_input, ..
            } => partition_input,
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::ExistsFilter { .. } => &[],
        }
    }

    /// Correlation proof owned by SemiJoin or AntiJoin.
    #[must_use]
    pub const fn correlations(&self) -> Option<&CorrelationSetV2> {
        match &self.op {
            AlgebraOpV2::SemiJoin { correlations, .. }
            | AlgebraOpV2::AntiJoin { correlations, .. } => Some(correlations),
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::ExistsFilter { .. } => None,
        }
    }

    /// Authored conditions of a flat existential filter. The first child is
    /// the input, and each following child matches one condition in order.
    pub fn existential_terms(&self) -> &[ExistentialTermV2] {
        match &self.op {
            AlgebraOpV2::ExistsFilter { terms, .. } => terms,
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. } => &[],
        }
    }

    pub(crate) const fn scan(source: ResolvedScanSourceV2) -> AlgebraOpV2 {
        AlgebraOpV2::Scan { source }
    }

    pub(crate) fn filter(
        input: CheckedAlgebraNodeV2,
        conjuncts: Vec<TypedBoolExprV2>,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::Filter {
            input: Box::new(input),
            conjuncts,
        }
    }

    pub(crate) fn project(
        input: CheckedAlgebraNodeV2,
        expressions: Vec<TypedExprV2>,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::Project {
            input: Box::new(input),
            expressions,
        }
    }

    pub(crate) fn join(
        left: CheckedAlgebraNodeV2,
        right: CheckedAlgebraNodeV2,
        mode: JoinModeV2,
        predicate: Option<TypedBoolExprV2>,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::Join {
            left: Box::new(left),
            right: Box::new(right),
            mode,
            predicate,
        }
    }

    pub(crate) fn group(
        input: CheckedAlgebraNodeV2,
        keys: Vec<TypedExprV2>,
        partition_input: Vec<PartitionInputBindingV2>,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::Group {
            input: Box::new(input),
            keys,
            partition_input,
        }
    }

    pub(crate) fn aggregate(
        input: CheckedAlgebraNodeV2,
        expressions: Vec<TypedExprV2>,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::Aggregate {
            input: Box::new(input),
            expressions,
        }
    }

    pub(crate) fn sort(input: CheckedAlgebraNodeV2, keys: Vec<OrderKeyContractV2>) -> AlgebraOpV2 {
        AlgebraOpV2::Sort {
            input: Box::new(input),
            keys,
        }
    }

    pub(crate) fn semi_join(
        left: CheckedAlgebraNodeV2,
        right: CheckedAlgebraNodeV2,
        correlations: CorrelationSetV2,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::SemiJoin {
            left: Box::new(left),
            right: Box::new(right),
            correlations,
        }
    }

    pub(crate) fn anti_join(
        left: CheckedAlgebraNodeV2,
        right: CheckedAlgebraNodeV2,
        correlations: CorrelationSetV2,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::AntiJoin {
            left: Box::new(left),
            right: Box::new(right),
            correlations,
        }
    }

    pub(crate) fn exists_filter(
        input: CheckedAlgebraNodeV2,
        terms: Vec<ExistentialTermV2>,
    ) -> AlgebraOpV2 {
        AlgebraOpV2::ExistsFilter {
            input: Box::new(input),
            terms,
        }
    }
}

/// Family-specific agreement between a lowered operator and the exact sealed
/// output it must retain. Every check consumes Sema-owned shapes only; no
/// schema, provenance, or legality fact is recomputed from syntax.
fn op_agrees_with_output(
    op: &AlgebraOpV2,
    output: &RelationOutputContractV2,
    stage_key: SourceQualifiedNodeIdV2,
) -> bool {
    match op {
        AlgebraOpV2::Scan { .. } => scan_agrees_with_output(stage_key, output),
        AlgebraOpV2::Filter { input, conjuncts } => {
            !conjuncts.is_empty() && preserves_input_frame(output, input.output())
        }
        AlgebraOpV2::Project { input, expressions } => {
            !expressions.is_empty()
                && output.provenance() == input.output().provenance()
                && projection_agrees_with_output(expressions.iter().map(Some), output)
        }
        AlgebraOpV2::Join {
            left,
            right,
            mode,
            predicate,
        } => {
            matches!(
                (*mode, predicate.is_some()),
                (JoinModeV2::Inner, true) | (JoinModeV2::Cross, false)
            ) && output.has_exact_join_frame(left.output(), right.output())
        }
        AlgebraOpV2::Group {
            input,
            keys,
            partition_input,
        } => {
            // Authored grouping: key-only row with one expression-origin
            // column per key, exact partition-key identity, and a duplicate
            // free Binder partition proof. The partition input is the sealed
            // pre-group Binder frame (for an aliased scan, one binding per
            // alias), so its width is not re-derived here.
            output.provenance() == input.output().provenance()
                && if !keys.is_empty() {
                    let mut identities = std::collections::BTreeSet::new();
                    projection_agrees_with_output(keys.iter().map(Some), output)
                        && output
                            .partition_keys()
                            .iter()
                            .copied()
                            .eq(keys.iter().map(TypedExprV2::key))
                        && output.cardinality() == CardinalityV1::Set
                        && output.ordering().is_empty()
                        && partition_input.iter().all(|binding| {
                            identities.insert((
                                binding.binding().target_node(),
                                binding.binding().ordinal(),
                            ))
                        })
                } else {
                    // Producer-sealed implicit empty partition for a scalar
                    // aggregate: no keys, no row, no columns, no ordering.
                    partition_input.is_empty()
                        && output.row().fields().is_empty()
                        && output.columns().is_empty()
                        && output.partition_keys().is_empty()
                        && output.ordering().is_empty()
                        && output.cardinality() == CardinalityV1::Set
                }
        }
        AlgebraOpV2::Aggregate { input, expressions } => {
            input.family() == AlgebraFamilyV2::Group
                && !expressions.is_empty()
                // A sealed aggregate projection is uniform: Sema classifies
                // every fold expression as an aggregate before the stage is
                // sealed, so a scalar/aggregate mix is an impossible forged
                // contract and must fail at this constructor boundary.
                && expressions
                    .iter()
                    .all(|expression| {
                        expression.aggregate_context().aggregation()
                            == AggregationClassV1::Aggregate
                    })
                && output.partition_keys() == input.output().partition_keys()
                && output.cardinality() == CardinalityV1::Set
                && output.ordering().is_empty()
                && output.effects().aggregation == AggregationClassV1::Aggregate
                && projection_agrees_with_output(expressions.iter().map(Some), output)
        }
        AlgebraOpV2::Sort { input, keys } => {
            output.ordering() == keys.as_slice()
                && output.cardinality() == CardinalityV1::Sequence
                && output.row() == input.output().row()
                && output.columns() == input.output().columns()
                && output.partition_keys() == input.output().partition_keys()
        }
        AlgebraOpV2::SemiJoin { left, .. } | AlgebraOpV2::AntiJoin { left, .. } => {
            preserves_input_frame(output, left.output())
        }
        AlgebraOpV2::ExistsFilter { input, terms } => {
            terms.len() >= 2
                && terms.iter().all(|term| {
                    term.expression.source_id == stage_key.source_id
                        && term.right.stage_key().source_id == stage_key.source_id
                })
                && preserves_input_frame(output, input.output())
        }
    }
}

/// The same checked Scan origin rule serves the V2 node constructor and the
/// request-owned borrowed scalar view.
pub(crate) fn scan_agrees_with_output(
    stage_key: SourceQualifiedNodeIdV2,
    output: &RelationOutputContractV2,
) -> bool {
    output.columns().iter().all(|column| {
        matches!(
            column.origin(),
            OutputColumnOriginV2::SourceField { scan_stage, .. } if *scan_stage == stage_key
        )
    })
}

/// Compare already checked types, ordered expression IDs and output origins.
/// The request path supplies fallible borrowed lookups; None must reject.
pub(crate) fn projection_agrees_with_output<'a>(
    expressions: impl IntoIterator<Item = Option<&'a TypedExprV2>>,
    output: &RelationOutputContractV2,
) -> bool {
    if output.columns().len() != output.row().fields().len() {
        return false;
    }
    let mut expressions = expressions.into_iter();
    for (column, field) in output.columns().iter().zip(output.row().fields()) {
        let Some(Some(expression)) = expressions.next() else {
            return false;
        };
        if !matches!(column.origin(), OutputColumnOriginV2::Expression(key)
            if *key == expression.key())
            || !matches!(expression.type_state(), ExpressionTypeStateV2::Known(value)
                if value == field.value_type())
        {
            return false;
        }
    }
    expressions.next().is_none()
}

/// Exact frame preservation (row, provenance, columns, cardinality,
/// partition, ordering) required of predicate-restricting stages. Composed
/// expression effects are deliberately excluded: Sema extends a Filter
/// stage's effects with its predicate's effects.
fn preserves_input_frame(
    output: &RelationOutputContractV2,
    input: &RelationOutputContractV2,
) -> bool {
    output.row() == input.row()
        && output.provenance() == input.provenance()
        && output.columns() == input.columns()
        && output.cardinality() == input.cardinality()
        && output.partition_keys() == input.partition_keys()
        && output.ordering() == input.ordering()
}

#[cfg(test)]
mod projection_contract_tests {
    use super::*;
    use quanta_gqlang_v2_sema::test_support::{
        reference_relation_document_v2, ReferenceRelationFixtureV2,
    };

    #[test]
    #[expect(
        clippy::expect_used,
        reason = "exact lowered projection constructor controls"
    )]
    fn checked_project_rejects_changed_expression_count_and_order() {
        let fixture =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)
                .expect("real projection fixture");
        let lowered = crate::lower_relation_item(fixture.document(), fixture.item_id())
            .expect("canonical projection lowers");
        let root = lowered.root();
        let AlgebraOpV2::Project { input, expressions } = &root.op else {
            unreachable!("fixture lowers to Project");
        };
        let build = |expressions| {
            CheckedAlgebraNodeV2::new(
                root.item_id,
                root.stage_key,
                root.location.clone(),
                root.output.clone(),
                CheckedAlgebraNodeV2::project((**input).clone(), expressions),
            )
        };
        assert_eq!(expressions.len(), 2);
        assert!(build(expressions.clone()).is_ok());
        assert!(!projection_agrees_with_output(
            expressions
                .iter()
                .enumerate()
                .map(|(index, expression)| (index != 1).then_some(expression)),
            root.output(),
        ));
        let mut short = expressions.clone();
        short.pop();
        let mut long = expressions.clone();
        long.push(expressions.first().expect("first expression").clone());
        let mut reversed = expressions.clone();
        reversed.reverse();
        for changed in [Vec::new(), short, long, reversed] {
            assert_eq!(build(changed), Err(()));
        }
    }
    #[test]
    #[expect(clippy::expect_used, reason = "checked join constructor controls")]
    fn checked_join_rejects_changed_frame_facts() {
        use quanta_gqlang_v2_sema::test_support::{
            forged_output_contract_v2, OutputContractMutationV2 as Mutation,
            ReferenceRelationFixtureV2 as Fixture,
        };
        for fixture in [
            Fixture::CrossJoin,
            Fixture::InnerJoinOn,
            Fixture::SelfJoinInner,
        ] {
            let document = reference_relation_document_v2(fixture).expect("real join");
            let lowered = crate::lower_relation_item(document.document(), document.item_id())
                .expect("real join lowers");
            let root = lowered.root();
            assert_eq!(root.family(), AlgebraFamilyV2::Join);
            let build = |output| {
                CheckedAlgebraNodeV2::new(
                    root.item_id,
                    root.stage_key,
                    root.location.clone(),
                    output,
                    root.op.clone(),
                )
            };
            assert!(build(root.output.clone()).is_ok());
            for mutation in [
                Mutation::SetCardinality,
                Mutation::SequenceCardinality,
                Mutation::FirstFieldType,
                Mutation::FirstFieldOptionality,
                Mutation::Partition,
                Mutation::Ordering,
            ] {
                let output = forged_output_contract_v2(root.output(), root.stage_key(), mutation)
                    .expect("generic output is row-aligned");
                assert_ne!(&output, root.output(), "{fixture:?} {mutation:?}");
                assert_eq!(build(output), Err(()), "{fixture:?} {mutation:?}");
            }
        }
    }

    #[test]
    #[expect(clippy::expect_used, reason = "checked projection type controls")]
    fn checked_projections_reject_types_differing_from_expression_facts() {
        use quanta_gqlang_v2_sema::test_support::{
            forged_output_contract_v2, OutputContractMutationV2 as Mutation,
            ReferenceRelationFixtureV2 as Fixture,
        };
        for (fixture, family) in [
            (
                Fixture::ProjectMultipleExpressions,
                AlgebraFamilyV2::Project,
            ),
            (Fixture::GroupedAggregateSum, AlgebraFamilyV2::Aggregate),
            (Fixture::ScalarCountStar, AlgebraFamilyV2::Aggregate),
        ] {
            let document = reference_relation_document_v2(fixture).expect("real projection");
            let lowered = crate::lower_relation_item(document.document(), document.item_id())
                .expect("real projection lowers");
            let root = lowered.root();
            assert_eq!(root.family(), family);
            let mut checked_nodes = vec![root];
            if let AlgebraOpV2::Aggregate { input, .. } = &root.op {
                if !input.expressions().is_empty() {
                    assert_eq!(input.family(), AlgebraFamilyV2::Group);
                    checked_nodes.push(input);
                }
            }
            for root in checked_nodes {
                let build = |output| {
                    CheckedAlgebraNodeV2::new(
                        root.item_id,
                        root.stage_key,
                        root.location.clone(),
                        output,
                        root.op.clone(),
                    )
                };
                assert!(build(root.output.clone()).is_ok());
                for mutation in [Mutation::FirstFieldType, Mutation::FirstFieldOptionality] {
                    let output =
                        forged_output_contract_v2(root.output(), root.stage_key(), mutation)
                            .expect("generic output is row-aligned");
                    assert_ne!(output.row(), root.output().row());
                    assert_eq!(output.columns(), root.output().columns());
                    assert_eq!(build(output), Err(()), "{fixture:?} {mutation:?}");
                }
            }
        }
    }
}
