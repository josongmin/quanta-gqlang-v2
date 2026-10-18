//! Source-reachable typed relation stages and retained output contracts.

use crate::{
    ResolvedRelationSourceKindV2, ResolvedValueBindingV2, SemanticItemIdV2, TypedBoolExprV2,
};
use quanta_gqlang_v2_hir::CheckedLocatedSpanV2;

mod order_hir_v3;
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, ExpressionEffectsV2, RecordTypeV1, SourceQualifiedNodeIdV2,
    ValueTypeV1,
};
use std::collections::BTreeSet;

/// Canonical resolved origin of a scan. This alias deliberately reuses the
/// names owner's disjoint host/View/Composition identity instead of maintaining
/// a second relation-source enum in the relation IR.
pub type ResolvedScanSourceV2 = ResolvedRelationSourceKindV2;

/// Null ordering policy of a checked order key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NullOrderV2 {
    /// Nulls before non-null values.
    First,
    /// Nulls after non-null values.
    Last,
}

/// Direction of an order key after semantic sealing.
///
/// This is deliberately a semantic carrier rather than a re-exported HIR tag:
/// algebra consumes a checked order contract and must not depend on HIR.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SortDirectionV2 {
    /// Ascending order, including an omitted source direction.
    Asc,
    /// Descending order.
    Desc,
}

/// One checked order key with its source-qualified expression identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderKeyContractV2 {
    expression: SourceQualifiedNodeIdV2,
    direction: SortDirectionV2,
    null_order: NullOrderV2,
    value_type: ValueTypeV1,
}

impl OrderKeyContractV2 {
    /// Construct a typed order descriptor.
    #[must_use]
    pub(crate) const fn new(
        expression: SourceQualifiedNodeIdV2,
        direction: SortDirectionV2,
        null_order: NullOrderV2,
        value_type: ValueTypeV1,
    ) -> Self {
        Self {
            expression,
            direction,
            null_order,
            value_type,
        }
    }
    /// Expression identity.
    #[must_use]
    pub const fn expression(&self) -> SourceQualifiedNodeIdV2 {
        self.expression
    }
    /// Direction.
    #[must_use]
    pub const fn direction(&self) -> SortDirectionV2 {
        self.direction
    }
    /// Null policy.
    #[must_use]
    pub const fn null_order(&self) -> NullOrderV2 {
        self.null_order
    }
    /// Checked key type.
    #[must_use]
    pub const fn value_type(&self) -> &ValueTypeV1 {
        &self.value_type
    }
}

/// Deterministic set of relation and View origins.
#[derive(Clone, Debug)]
enum ProvenanceStorageV2 {
    Set(BTreeSet<ResolvedScanSourceV2>),
    Single(ResolvedScanSourceV2),
}

#[derive(Clone, Debug)]
pub struct OutputProvenanceV2(ProvenanceStorageV2);

impl Default for OutputProvenanceV2 {
    fn default() -> Self {
        Self(ProvenanceStorageV2::Set(BTreeSet::new()))
    }
}

impl PartialEq for OutputProvenanceV2 {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for OutputProvenanceV2 {}

impl OutputProvenanceV2 {
    /// Build a unique source set.
    #[must_use]
    pub fn new(sources: impl IntoIterator<Item = ResolvedScanSourceV2>) -> Self {
        Self(ProvenanceStorageV2::Set(sources.into_iter().collect()))
    }

    pub(crate) fn from_single_source_v3(source: ResolvedScanSourceV2) -> Self {
        Self(ProvenanceStorageV2::Single(source))
    }
    /// Ordered source identities.
    pub fn iter(&self) -> impl Iterator<Item = &ResolvedScanSourceV2> {
        let single = match &self.0 {
            ProvenanceStorageV2::Single(source) => Some(source),
            ProvenanceStorageV2::Set(_) => None,
        };
        let set = match &self.0 {
            ProvenanceStorageV2::Set(set) => Some(set),
            ProvenanceStorageV2::Single(_) => None,
        };
        single
            .into_iter()
            .chain(set.into_iter().flat_map(BTreeSet::iter))
    }

    fn union_iter<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a ResolvedScanSourceV2> {
        let mut left = self.iter().peekable();
        let mut right = other.iter().peekable();
        std::iter::from_fn(move || match (left.peek(), right.peek()) {
            (Some(first), Some(second)) => match first.cmp(second) {
                std::cmp::Ordering::Less => left.next(),
                std::cmp::Ordering::Equal => {
                    let _ = right.next();
                    left.next()
                }
                std::cmp::Ordering::Greater => right.next(),
            },
            (Some(_), None) => left.next(),
            (None, Some(_)) => right.next(),
            (None, None) => None,
        })
    }

    fn contains(&self, source: &ResolvedScanSourceV2) -> bool {
        match &self.0 {
            ProvenanceStorageV2::Set(set) => set.contains(source),
            ProvenanceStorageV2::Single(value) => value == source,
        }
    }
    /// No source origin.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
}

/// Exact producer of one output column. Projection expressions are looked up
/// through the sealed semantic document; scan fields retain their catalog ID;
/// closed frontend-generated rows retain the exact nested producer item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputColumnOriginV2 {
    /// A named field of one authoritative host relation or catalog View.
    SourceField {
        source: ResolvedScanSourceV2,
        scan_stage: SourceQualifiedNodeIdV2,
        field_name: String,
    },
    /// The typed expression that produced a computed output field.
    Expression(SourceQualifiedNodeIdV2),
    /// A closed frontend row generated by one nested Composition producer.
    ///
    /// This is deliberately not represented as an expression key: SAMPLE and
    /// row-capable FIND do not own a typed value expression that can be used as
    /// an origin. The item identity prevents a schema-shaped foreign row from
    /// being relabeled as this producer's result.
    Generated(SemanticItemIdV2),
}

/// One row-aligned output column and its checked producer location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputColumnContractV2 {
    name: String,
    origin: OutputColumnOriginV2,
    location: CheckedLocatedSpanV2,
}

impl OutputColumnContractV2 {
    pub(crate) fn new(
        name: String,
        origin: OutputColumnOriginV2,
        location: CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            name,
            origin,
            location,
        }
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn origin(&self) -> &OutputColumnOriginV2 {
        &self.origin
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// Complete relation output proof retained by algebra rather than reconstructed
/// from syntax. The row schema is checked by the types owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationOutputContractV2 {
    row: RecordTypeV1,
    provenance: OutputProvenanceV2,
    columns: Vec<OutputColumnContractV2>,
    cardinality: CardinalityV1,
    partition_keys: Vec<SourceQualifiedNodeIdV2>,
    ordering: Vec<OrderKeyContractV2>,
    effects: ExpressionEffectsV2,
}

impl RelationOutputContractV2 {
    /// Construct an output proof; repeated partition/order keys are rejected.
    pub(crate) fn try_new(
        row: RecordTypeV1,
        provenance: OutputProvenanceV2,
        columns: Vec<OutputColumnContractV2>,
        cardinality: CardinalityV1,
        partition_keys: Vec<SourceQualifiedNodeIdV2>,
        ordering: Vec<OrderKeyContractV2>,
        effects: ExpressionEffectsV2,
    ) -> Result<Self, &'static str> {
        let partitions: BTreeSet<_> = partition_keys.iter().copied().collect();
        let orders: BTreeSet<_> = ordering
            .iter()
            .map(OrderKeyContractV2::expression)
            .collect();
        if partitions.len() != partition_keys.len() || orders.len() != ordering.len() {
            return Err("duplicate partition or order key");
        }
        if !columns_aligned_with_row(&row, &columns) {
            return Err("output columns differ from the typed row or authoritative origins");
        }
        if columns.iter().any(|column| {
            matches!(&column.origin,
                OutputColumnOriginV2::SourceField { source, field_name, .. }
                    if field_name.is_empty() || !provenance.contains(source))
        }) {
            return Err("output columns differ from the typed row or authoritative origins");
        }
        Ok(Self {
            row,
            provenance,
            columns,
            cardinality,
            partition_keys,
            ordering,
            effects,
        })
    }

    /// The one aggregate-output sealing constructor. An aggregate-ready
    /// projection output is always a set, retains exactly its partition
    /// authority's key vector (empty for a producer-sealed scalar partition),
    /// keeps one row-aligned expression origin per output column, carries no
    /// ordering, and its composed effects must retain the aggregate class.
    /// Cardinality is pinned here; a caller cannot downgrade or choose it.
    pub(crate) fn try_aggregate(
        row: RecordTypeV1,
        provenance: OutputProvenanceV2,
        columns: Vec<OutputColumnContractV2>,
        partition_keys: Vec<SourceQualifiedNodeIdV2>,
        effects: ExpressionEffectsV2,
        partition_authority_keys: &[SourceQualifiedNodeIdV2],
    ) -> Result<Self, &'static str> {
        if partition_keys != partition_authority_keys {
            return Err("aggregate output partition identity differs from its Group authority");
        }
        if !columns_aligned_with_row(&row, &columns) {
            return Err("aggregate output columns differ from the typed row");
        }
        if columns
            .iter()
            .any(|column| !matches!(column.origin(), OutputColumnOriginV2::Expression(_)))
        {
            return Err("aggregate output column lacks a typed expression origin");
        }
        if effects.aggregation != AggregationClassV1::Aggregate {
            return Err("aggregate output effects must retain the aggregate class");
        }
        Ok(Self {
            row,
            provenance,
            columns,
            cardinality: CardinalityV1::Set,
            partition_keys,
            ordering: Vec::new(),
            effects,
        })
    }
    /// Row schema.
    #[must_use]
    pub const fn row(&self) -> &RecordTypeV1 {
        &self.row
    }
    /// Exact source origins.
    #[must_use]
    pub const fn provenance(&self) -> &OutputProvenanceV2 {
        &self.provenance
    }
    /// Ordered per-column producer proofs aligned one-to-one with row fields.
    #[must_use]
    pub fn columns(&self) -> &[OutputColumnContractV2] {
        &self.columns
    }
    /// Whether every output column has exactly the corresponding expression
    /// origin, including source identity, order and total width. This compares
    /// sealed identities only; it does not infer projections from names.
    pub fn has_exact_projection_origins(
        &self,
        expressions: impl IntoIterator<Item = SourceQualifiedNodeIdV2>,
    ) -> bool {
        self.columns
            .iter()
            .map(|column| match column.origin() {
                OutputColumnOriginV2::Expression(key) => Some(*key),
                OutputColumnOriginV2::SourceField { .. } | OutputColumnOriginV2::Generated(_) => {
                    None
                }
            })
            .eq(expressions.into_iter().map(Some))
    }
    /// Exact inner/cross-join output frame. Types and ordered column origins
    /// concatenate the operands; authored joins produce an unordered bag and
    /// consume any operand partition/order metadata.
    #[must_use]
    pub fn has_exact_join_frame(&self, left: &Self, right: &Self) -> bool {
        self.cardinality == CardinalityV1::Bag
            && self.partition_keys.is_empty()
            && self.ordering.is_empty()
            && self
                .row
                .fields()
                .iter()
                .eq(left.row.fields().iter().chain(right.row.fields()))
            && self
                .columns
                .iter()
                .eq(left.columns.iter().chain(&right.columns))
            && self
                .provenance
                .iter()
                .eq(left.provenance.union_iter(&right.provenance))
    }
    /// Cardinality/multiplicity.
    #[must_use]
    pub const fn cardinality(&self) -> CardinalityV1 {
        self.cardinality
    }
    /// Retained grouping keys.
    #[must_use]
    pub fn partition_keys(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.partition_keys
    }
    /// Retained ordering keys.
    #[must_use]
    pub fn ordering(&self) -> &[OrderKeyContractV2] {
        &self.ordering
    }
    /// Composed expression effects.
    #[must_use]
    pub const fn effects(&self) -> ExpressionEffectsV2 {
        self.effects
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationStageV2 {
    key: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
    inputs: Vec<TypedRelationExprV2>,
    expressions: Vec<SourceQualifiedNodeIdV2>,
    predicate: Option<TypedBoolExprV2>,
    source: Option<ResolvedScanSourceV2>,
    join_mode: Option<JoinModeV2>,
    implicit_partition: bool,
    partition_input: Vec<PartitionInputBindingV2>,
    output: RelationOutputContractV2,
}

/// Row alignment shared by every output constructor: one column per typed row
/// field, matching names, with expression/generated origins anchored to the
/// column's own checked source location.
fn columns_aligned_with_row(row: &RecordTypeV1, columns: &[OutputColumnContractV2]) -> bool {
    columns.len() == row.fields().len()
        && columns.iter().zip(row.fields()).all(|(column, field)| {
            column.name == field.name()
                && match &column.origin {
                    OutputColumnOriginV2::SourceField { .. } => true,
                    OutputColumnOriginV2::Expression(key) => {
                        key.source_id == column.location.location().source_id
                    }
                    OutputColumnOriginV2::Generated(producer) => {
                        producer.source_id() == column.location.location().source_id
                    }
                }
        })
}

/// Pre-group Binder identity retained separately from the key-only row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionInputBindingV2 {
    binding: ResolvedValueBindingV2,
    location: CheckedLocatedSpanV2,
}

impl PartitionInputBindingV2 {
    pub(crate) fn try_new(
        binding: ResolvedValueBindingV2,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, &'static str> {
        if binding.source_id() != location.location().source_id {
            return Err("partition binding and source differ");
        }
        Ok(Self { binding, location })
    }
    #[must_use]
    pub const fn binding(&self) -> &ResolvedValueBindingV2 {
        &self.binding
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// The two source-reachable forms of the single HIR Join stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinModeV2 {
    Inner,
    Cross,
}

/// Source-reachable typed relation vocabulary. Programmatic-only algebra
/// operators belong to ticket 05 and are not duplicated in sema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedRelationExprV2 {
    /// Source or View scan.
    Scan(RelationStageV2),
    /// Predicate filter.
    Filter(RelationStageV2),
    /// Projection and row-frame replacement.
    Project(RelationStageV2),
    /// Inner/cross join, checked by its retained subkind.
    Join(RelationStageV2),
    /// Group-by stage.
    Group(RelationStageV2),
    /// Ordered stage.
    Sort(RelationStageV2),
    /// Relational truncation retaining its exact source count.
    Limit(RelationStageV2, u64),
}

/// Producer-only input to the checked stage constructor. The stage checks its
/// structural contract once and retains the exact output proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedRelationStageInputV2 {
    pub(crate) key: SourceQualifiedNodeIdV2,
    pub(crate) location: CheckedLocatedSpanV2,
    pub(crate) kind: TypedRelationStageKindV2,
    pub(crate) inputs: Vec<TypedRelationExprV2>,
    pub(crate) expressions: Vec<SourceQualifiedNodeIdV2>,
    pub(crate) predicate: Option<TypedBoolExprV2>,
    pub(crate) source: Option<ResolvedScanSourceV2>,
    pub(crate) join_mode: Option<JoinModeV2>,
    pub(crate) implicit_partition: bool,
    pub(crate) partition_input: Vec<PartitionInputBindingV2>,
    pub(crate) output: RelationOutputContractV2,
}

/// Source-reachable stage identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedRelationStageKindV2 {
    Scan,
    Filter,
    Project,
    Join,
    Group,
    Sort,
    Limit(u64),
}

impl TypedRelationExprV2 {
    /// Check exact stage arity and payload before sealing it. A caller cannot
    /// combine a semantic result from one stage with an unrelated stage kind.
    pub(crate) fn try_from_stage(input: TypedRelationStageInputV2) -> Result<Self, &'static str> {
        let TypedRelationStageInputV2 {
            key,
            location,
            kind,
            inputs,
            expressions,
            predicate,
            source,
            join_mode,
            implicit_partition,
            partition_input,
            output,
        } = input;
        // An implicit (empty-partition) Group is minted only by the Sema
        // producer for a legal scalar aggregate; every authored stage and the
        // crate-local grouping constructor carry authored nonempty keys.
        let group_shape_ok = match kind {
            TypedRelationStageKindV2::Group => {
                if implicit_partition {
                    expressions.is_empty() && partition_input.is_empty()
                } else {
                    !expressions.is_empty()
                }
            }
            TypedRelationStageKindV2::Scan
            | TypedRelationStageKindV2::Filter
            | TypedRelationStageKindV2::Project
            | TypedRelationStageKindV2::Join
            | TypedRelationStageKindV2::Sort
            | TypedRelationStageKindV2::Limit(_) => !implicit_partition,
        };
        let valid = match kind {
            TypedRelationStageKindV2::Scan => {
                inputs.is_empty()
                    && expressions.is_empty()
                    && predicate.is_none()
                    && source.is_some()
                    && join_mode.is_none()
                    && output.columns().iter().all(|column| {
                        matches!(column.origin(),
                    OutputColumnOriginV2::SourceField { source: origin, scan_stage, .. }
                    if source.as_ref() == Some(origin) && *scan_stage == key)
                    })
            }
            TypedRelationStageKindV2::Filter => {
                inputs.len() == 1
                    && expressions.is_empty()
                    && predicate.is_some()
                    && source.is_none()
                    && join_mode.is_none()
            }
            TypedRelationStageKindV2::Project => {
                inputs.len() == 1
                    && !expressions.is_empty()
                    && output.has_exact_projection_origins(expressions.iter().copied())
                    && predicate.is_none()
                    && source.is_none()
                    && join_mode.is_none()
            }
            TypedRelationStageKindV2::Join => {
                matches!(inputs.as_slice(), [left, right] if output.has_exact_join_frame(left.output(), right.output()))
                    && expressions.is_empty()
                    && source.is_none()
                    && matches!(
                        (join_mode, predicate.is_some()),
                        (Some(JoinModeV2::Inner), true) | (Some(JoinModeV2::Cross), false)
                    )
            }
            TypedRelationStageKindV2::Group if implicit_partition => {
                inputs.len() == 1
                    && expressions.is_empty()
                    && predicate.is_none()
                    && source.is_none()
                    && join_mode.is_none()
                    && output.partition_keys().is_empty()
                    && output.columns().is_empty()
                    && output.row().fields().is_empty()
                    && output.cardinality() == CardinalityV1::Set
                    && output.ordering().is_empty()
            }
            TypedRelationStageKindV2::Limit(_) => {
                inputs.len() == 1
                    && expressions.is_empty()
                    && predicate.is_none()
                    && source.is_none()
                    && join_mode.is_none()
                    && inputs
                        .first()
                        .is_some_and(|child| child.output() == &output)
            }
            TypedRelationStageKindV2::Group | TypedRelationStageKindV2::Sort => {
                inputs.len() == 1
                    && !expressions.is_empty()
                    && predicate.is_none()
                    && source.is_none()
                    && join_mode.is_none()
            }
        };
        if !valid
            || !group_shape_ok
            || (kind != TypedRelationStageKindV2::Group && !partition_input.is_empty())
            || (implicit_partition && !partition_input.is_empty())
            || partition_input.iter().any(|binding| {
                binding.binding.source_id() != key.source_id
                    || binding.location.registry_identity() != location.registry_identity()
            })
            || location.location().source_id != key.source_id
            || inputs.iter().any(|child| {
                child.key().source_id != key.source_id
                    || child.location().registry_identity() != location.registry_identity()
            })
        {
            return Err("invalid typed relation stage arity or payload");
        }
        let stage = stage(
            key,
            location,
            inputs,
            expressions,
            predicate,
            source,
            join_mode,
            implicit_partition,
            partition_input,
            output,
        );
        Ok(match kind {
            TypedRelationStageKindV2::Scan => Self::Scan(stage),
            TypedRelationStageKindV2::Filter => Self::Filter(stage),
            TypedRelationStageKindV2::Project => Self::Project(stage),
            TypedRelationStageKindV2::Join => Self::Join(stage),
            TypedRelationStageKindV2::Group => Self::Group(stage),
            TypedRelationStageKindV2::Sort => Self::Sort(stage),
            TypedRelationStageKindV2::Limit(count) => Self::Limit(stage, count),
        })
    }

    /// Construct a scan from an already resolved authority source.
    pub(crate) fn scan(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        source: ResolvedScanSourceV2,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Scan,
            inputs: Vec::new(),
            expressions: Vec::new(),
            predicate: None,
            source: Some(source),
            join_mode: None,
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Construct a Bool-proven filter.
    pub(crate) fn filter(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
        predicate: TypedBoolExprV2,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Filter,
            inputs: vec![input],
            expressions: Vec::new(),
            predicate: Some(predicate),
            source: None,
            join_mode: None,
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Construct a nonempty ordinary projection.
    pub(crate) fn try_project(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
        expressions: Vec<SourceQualifiedNodeIdV2>,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Project,
            inputs: vec![input],
            expressions,
            predicate: None,
            source: None,
            join_mode: None,
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// The one aggregate-projection constructor. The input must be exactly one
    /// Group stage (authored nonempty keys, or the producer-sealed implicit
    /// empty partition of a scalar aggregate); the sealed output must retain
    /// that Group's partition identity, Set cardinality, and one row-aligned
    /// typed-expression origin per output column. No public constructor can
    /// produce any of these shapes.
    pub(crate) fn try_aggregate_project(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
        expressions: Vec<SourceQualifiedNodeIdV2>,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        let TypedRelationExprV2::Group(_) = &input else {
            return Err("aggregate projection must consume exactly one Group stage");
        };
        if output.cardinality() != CardinalityV1::Set
            || output.partition_keys() != input.output().partition_keys()
        {
            return Err("aggregate projection output loses its Group partition identity");
        }
        if !output.has_exact_projection_origins(expressions.iter().copied()) {
            return Err("aggregate projection origins are not row-aligned with its expressions");
        }
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Project,
            inputs: vec![input],
            expressions,
            predicate: None,
            source: None,
            join_mode: None,
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Construct an ON-checked inner join.
    pub(crate) fn join(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        left: Self,
        right: Self,
        predicate: TypedBoolExprV2,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Join,
            inputs: vec![left, right],
            expressions: Vec::new(),
            predicate: Some(predicate),
            source: None,
            join_mode: Some(JoinModeV2::Inner),
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Construct an explicit cross join.
    pub(crate) fn cross(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        left: Self,
        right: Self,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Join,
            inputs: vec![left, right],
            expressions: Vec::new(),
            predicate: None,
            source: None,
            join_mode: Some(JoinModeV2::Cross),
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Construct nonempty authored grouping.
    pub(crate) fn try_group(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
        keys: Vec<SourceQualifiedNodeIdV2>,
        partition_input: Vec<PartitionInputBindingV2>,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Group,
            inputs: vec![input],
            expressions: keys,
            predicate: None,
            source: None,
            join_mode: None,
            implicit_partition: false,
            partition_input,
            output,
        })
    }
    /// The producer-sealed scalar aggregate partition: one private Group with
    /// no public keys whose exact pre-fold input contract is the retained
    /// input stage output. The output cannot be supplied by a caller: the
    /// empty key-only row, Set cardinality, inherited provenance, and pre-fold
    /// effects are sealed here from the input stage itself.
    pub(crate) fn implicit_group(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
    ) -> Result<Self, &'static str> {
        let previous = input.output();
        let row = RecordTypeV1::try_new(Vec::new()).map_err(|_error| {
            "implicit scalar aggregate partition requires an empty key-only row"
        })?;
        let output = RelationOutputContractV2::try_new(
            row,
            previous.provenance().clone(),
            Vec::new(),
            CardinalityV1::Set,
            Vec::new(),
            Vec::new(),
            previous.effects(),
        )
        .map_err(|_error| "implicit scalar aggregate partition output is not sealable")?;
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Group,
            inputs: vec![input],
            expressions: Vec::new(),
            predicate: None,
            source: None,
            join_mode: None,
            implicit_partition: true,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Construct nonempty ordering.
    pub(crate) fn try_sort(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
        keys: Vec<SourceQualifiedNodeIdV2>,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Sort,
            inputs: vec![input],
            expressions: keys,
            predicate: None,
            source: None,
            join_mode: None,
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }
    /// Seal truncation without changing row, provenance, ordering or effects.
    pub(crate) fn try_limit(
        key: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
        input: Self,
        count: u64,
    ) -> Result<Self, &'static str> {
        let output = input.output().clone();
        Self::try_from_stage(TypedRelationStageInputV2 {
            key,
            location,
            kind: TypedRelationStageKindV2::Limit(count),
            inputs: vec![input],
            expressions: Vec::new(),
            predicate: None,
            source: None,
            join_mode: None,
            implicit_partition: false,
            partition_input: Vec::new(),
            output,
        })
    }

    #[must_use]
    pub const fn limit_count(&self) -> Option<u64> {
        match self {
            Self::Limit(_, count) => Some(*count),
            _ => None,
        }
    }

    /// Retained exact output.
    #[must_use]
    pub const fn output(&self) -> &RelationOutputContractV2 {
        match self {
            Self::Scan(node)
            | Self::Filter(node)
            | Self::Project(node)
            | Self::Join(node)
            | Self::Group(node)
            | Self::Sort(node)
            | Self::Limit(node, _) => &node.output,
        }
    }
    /// Exact checked HIR stage identity retained for downstream provenance.
    #[must_use]
    pub const fn key(&self) -> SourceQualifiedNodeIdV2 {
        self.stage().key
    }
    /// Checked source location of the exact HIR relation stage.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.stage().location
    }
    /// Typed child stages, in evaluation order.
    pub fn inputs(&self) -> impl DoubleEndedIterator<Item = &TypedRelationExprV2> {
        self.stage().inputs.iter()
    }
    /// Typed expression keys owned by this stage.
    #[must_use]
    pub fn expressions(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.stage().expressions
    }
    /// Checked predicate, when required by this stage.
    #[must_use]
    pub const fn predicate(&self) -> Option<&TypedBoolExprV2> {
        self.stage().predicate.as_ref()
    }
    /// Source identity, present only for a scan.
    #[must_use]
    pub const fn source(&self) -> Option<&ResolvedScanSourceV2> {
        self.stage().source.as_ref()
    }
    /// Checked join subkind; absent on other stages.
    #[must_use]
    pub const fn join_mode(&self) -> Option<JoinModeV2> {
        self.stage().join_mode
    }
    /// Exact pre-group Binder names; never part of the public key-only row.
    #[must_use]
    pub fn partition_input(&self) -> &[PartitionInputBindingV2] {
        &self.stage().partition_input
    }

    const fn stage(&self) -> &RelationStageV2 {
        match self {
            Self::Scan(node)
            | Self::Filter(node)
            | Self::Project(node)
            | Self::Join(node)
            | Self::Group(node)
            | Self::Sort(node)
            | Self::Limit(node, _) => node,
        }
    }
}

/// Typed relation associated with exactly one document item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedRelationV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    root: TypedRelationExprV2,
    output: RelationOutputContractV2,
}

impl TypedRelationV2 {
    /// Seal a typed root with its exact retained output.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        location: CheckedLocatedSpanV2,
        expected_root: SourceQualifiedNodeIdV2,
        root: TypedRelationExprV2,
        output: RelationOutputContractV2,
    ) -> Result<Self, &'static str> {
        if root.output() != &output
            || root.key() != expected_root
            || expected_root.source_id != item_id.source_id()
            || location.location().source_id != item_id.source_id()
            || root.location().registry_identity() != location.registry_identity()
        {
            return Err("relation root/output contract mismatch");
        }
        Ok(Self {
            item_id,
            location,
            root,
            output,
        })
    }
    /// Item ID.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Checked source location of the authored query item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Typed root.
    #[must_use]
    pub const fn root(&self) -> &TypedRelationExprV2 {
        &self.root
    }
    /// Retained output.
    #[must_use]
    pub const fn output(&self) -> &RelationOutputContractV2 {
        &self.output
    }
}

fn stage(
    key: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
    inputs: Vec<TypedRelationExprV2>,
    expressions: Vec<SourceQualifiedNodeIdV2>,
    predicate: Option<TypedBoolExprV2>,
    source: Option<ResolvedScanSourceV2>,
    join_mode: Option<JoinModeV2>,
    implicit_partition: bool,
    partition_input: Vec<PartitionInputBindingV2>,
    output: RelationOutputContractV2,
) -> RelationStageV2 {
    RelationStageV2 {
        key,
        location,
        inputs,
        expressions,
        predicate,
        source,
        join_mode,
        implicit_partition,
        partition_input,
        output,
    }
}

#[cfg(test)]
#[path = "relation/tests.rs"]
mod tests;
