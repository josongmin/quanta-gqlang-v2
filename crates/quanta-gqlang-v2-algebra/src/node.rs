//! Opaque algebra nodes backed by sealed semantic contracts.

use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, JoinModeV2, OrderKeyContractV2, OutputColumnOriginV2, OutputProvenanceV2,
    PartitionInputBindingV2, RelationOutputContractV2, ResolvedScanSourceV2, SemanticItemIdV2,
    TypedBoolExprV2, TypedCorrelationEdgeV2, TypedExprV2,
};
use quanta_gqlang_v2_types::{AggregationClassV1, CardinalityV1, SourceQualifiedNodeIdV2};

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
    /// Programmatic correlated/lateral operator.
    Apply,
    /// Source grouping stage.
    Group,
    /// Aggregate folds over a Group.
    Aggregate,
    /// Source ordering stage.
    Sort,
    /// Programmatic ordered window.
    Limit,
    /// Programmatic compatible-arm union.
    Union,
    /// Positive relational existence.
    SemiJoin,
    /// Negative relational existence.
    AntiJoin,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    dead_code,
    reason = "Apply, Limit, and Union are intentionally programmatic-only; their sealed constructors land with their owning planner contracts"
)]
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
    Apply {
        left: Box<CheckedAlgebraNodeV2>,
        right: Box<CheckedAlgebraNodeV2>,
        correlations: CorrelationSetV2,
    },
    Limit {
        input: Box<CheckedAlgebraNodeV2>,
        count: u64,
        offset: u64,
    },
    Union {
        left: Box<CheckedAlgebraNodeV2>,
        right: Box<CheckedAlgebraNodeV2>,
    },
}

impl CheckedAlgebraNodeV2 {
    /// Assemble one node from a sealed stage contract and a lowered operator.
    ///
    /// Construction is fail-closed: the node identity must be coherent with
    /// its checked location, and the operator must agree with the exact
    /// sealed output (schema, provenance, cardinality, partition, ordering,
    /// and family-specific identity such as join mode, group keys, or
    /// aggregate partition retention). Composed expression effects are the
    /// one field a predicate-carrying stage may extend over its input.
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
        })
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
            AlgebraOpV2::Apply { .. } => AlgebraFamilyV2::Apply,
            AlgebraOpV2::Limit { .. } => AlgebraFamilyV2::Limit,
            AlgebraOpV2::Union { .. } => AlgebraFamilyV2::Union,
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
            | AlgebraOpV2::Sort { input, .. }
            | AlgebraOpV2::Limit { input, .. } => vec![input],
            AlgebraOpV2::Join { left, right, .. }
            | AlgebraOpV2::SemiJoin { left, right, .. }
            | AlgebraOpV2::AntiJoin { left, right, .. }
            | AlgebraOpV2::Apply { left, right, .. }
            | AlgebraOpV2::Union { left, right } => vec![left, right],
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => None,
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => None,
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => &[],
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => None,
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => &[],
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => &[],
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
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => &[],
        }
    }

    /// Correlation proof owned by Apply, SemiJoin, or AntiJoin.
    #[must_use]
    pub const fn correlations(&self) -> Option<&CorrelationSetV2> {
        match &self.op {
            AlgebraOpV2::SemiJoin { correlations, .. }
            | AlgebraOpV2::AntiJoin { correlations, .. }
            | AlgebraOpV2::Apply { correlations, .. } => Some(correlations),
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::Limit { .. }
            | AlgebraOpV2::Union { .. } => None,
        }
    }

    /// Programmatic Limit window, when applicable.
    #[must_use]
    pub const fn limit_window(&self) -> Option<(u64, u64)> {
        match &self.op {
            AlgebraOpV2::Limit { count, offset, .. } => Some((*count, *offset)),
            AlgebraOpV2::Scan { .. }
            | AlgebraOpV2::Filter { .. }
            | AlgebraOpV2::Project { .. }
            | AlgebraOpV2::Join { .. }
            | AlgebraOpV2::Group { .. }
            | AlgebraOpV2::Aggregate { .. }
            | AlgebraOpV2::Sort { .. }
            | AlgebraOpV2::SemiJoin { .. }
            | AlgebraOpV2::AntiJoin { .. }
            | AlgebraOpV2::Apply { .. }
            | AlgebraOpV2::Union { .. } => None,
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
        AlgebraOpV2::Scan { .. } => output.columns().iter().all(|column| {
            matches!(
                column.origin(),
                OutputColumnOriginV2::SourceField { scan_stage, .. } if *scan_stage == stage_key
            )
        }),
        AlgebraOpV2::Filter { input, conjuncts } => {
            !conjuncts.is_empty() && preserves_input_frame(output, input.output())
        }
        AlgebraOpV2::Project { input, .. } => output.provenance() == input.output().provenance(),
        AlgebraOpV2::Join {
            left,
            right,
            mode,
            predicate,
        } => {
            matches!(
                (*mode, predicate.is_some()),
                (JoinModeV2::Inner, true) | (JoinModeV2::Cross, false)
            ) && output.columns().iter().eq(left
                .output()
                .columns()
                .iter()
                .chain(right.output().columns().iter()))
                && *output.provenance()
                    == OutputProvenanceV2::new(
                        left.output()
                            .provenance()
                            .iter()
                            .chain(right.output().provenance().iter())
                            .cloned(),
                    )
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
                    output.columns().len() == keys.len()
                        && output.columns().iter().zip(keys).all(|(column, key)| {
                            *column.origin() == OutputColumnOriginV2::Expression(key.key())
                        })
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
                && output
                    .columns()
                    .iter()
                    .zip(expressions)
                    .all(|(column, expression)| {
                        *column.origin() == OutputColumnOriginV2::Expression(expression.key())
                    })
                && output.columns().len() == expressions.len()
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
        AlgebraOpV2::Apply { left, .. } => preserves_input_frame(output, left.output()),
        AlgebraOpV2::Limit { input, .. } => preserves_input_frame(output, input.output()),
        AlgebraOpV2::Union { left, right } => {
            preserves_input_frame(output, left.output())
                && preserves_input_frame(output, right.output())
        }
    }
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
