//! Source-reachable typed relation stages and retained output contracts.

use crate::{SemanticItemIdV2, TypedBoolExprV2};
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, SortDirectionHirV1};
use quanta_gqlang_v2_modules::CatalogAssetIdV2;
use quanta_gqlang_v2_types::{
    CardinalityV1, ExpressionEffectsV2, RecordTypeV1, RelationSourceIdV2,
    SourceQualifiedNodeIdV2, ValueTypeV1,
};
use std::collections::BTreeSet;

/// Resolved origin of a scan. Catalog Views and host relation sources are
/// distinct authority domains and never share a fabricated identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolvedScanSourceV2 {
    /// Host relation catalog source.
    Relation(RelationSourceIdV2),
    /// Document/module catalog View.
    View(CatalogAssetIdV2),
}

/// Null ordering policy of a checked order key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NullOrderV2 {
    /// Nulls before non-null values.
    First,
    /// Nulls after non-null values.
    Last,
}

/// One checked order key with its source-qualified expression identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderKeyContractV2 {
    expression: SourceQualifiedNodeIdV2,
    direction: SortDirectionHirV1,
    null_order: NullOrderV2,
    value_type: ValueTypeV1,
}

impl OrderKeyContractV2 {
    /// Construct a typed order descriptor.
    #[must_use]
    pub(crate) const fn new(
        expression: SourceQualifiedNodeIdV2,
        direction: SortDirectionHirV1,
        null_order: NullOrderV2,
        value_type: ValueTypeV1,
    ) -> Self {
        Self { expression, direction, null_order, value_type }
    }
    /// Expression identity.
    #[must_use] pub const fn expression(&self) -> SourceQualifiedNodeIdV2 { self.expression }
    /// Direction.
    #[must_use] pub const fn direction(&self) -> SortDirectionHirV1 { self.direction }
    /// Null policy.
    #[must_use] pub const fn null_order(&self) -> NullOrderV2 { self.null_order }
    /// Checked key type.
    #[must_use] pub const fn value_type(&self) -> &ValueTypeV1 { &self.value_type }
}

/// Deterministic set of relation and View origins.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutputProvenanceV2(BTreeSet<ResolvedScanSourceV2>);

impl OutputProvenanceV2 {
    /// Build a unique source set.
    #[must_use]
    pub fn new(sources: impl IntoIterator<Item = ResolvedScanSourceV2>) -> Self {
        Self(sources.into_iter().collect())
    }
    /// Ordered source identities.
    pub fn iter(&self) -> impl Iterator<Item = &ResolvedScanSourceV2> { self.0.iter() }
    /// No source origin.
    #[must_use] pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

/// Exact producer of one output column. Projection expressions are looked up
/// through the sealed semantic document; scan fields retain their catalog ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputColumnOriginV2 {
    /// A named field of one authoritative host relation or catalog View.
    SourceField { source: ResolvedScanSourceV2, scan_stage: SourceQualifiedNodeIdV2, field_name: String },
    /// The typed expression that produced a computed output field.
    Expression(SourceQualifiedNodeIdV2),
}

/// One row-aligned output column and its checked producer location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputColumnContractV2 {
    name: String,
    origin: OutputColumnOriginV2,
    location: CheckedLocatedSpanV2,
}

impl OutputColumnContractV2 {
    pub(crate) fn new(name: String, origin: OutputColumnOriginV2, location: CheckedLocatedSpanV2) -> Self {
        Self { name, origin, location }
    }
    #[must_use] pub fn name(&self) -> &str { &self.name }
    #[must_use] pub const fn origin(&self) -> &OutputColumnOriginV2 { &self.origin }
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.location }
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
        let orders: BTreeSet<_> = ordering.iter().map(OrderKeyContractV2::expression).collect();
        if partitions.len() != partition_keys.len() || orders.len() != ordering.len() {
            return Err("duplicate partition or order key");
        }
        if columns.len() != row.fields().len() || columns.iter().zip(row.fields()).any(|(column, field)|
            column.name != field.name() || match &column.origin {
                OutputColumnOriginV2::SourceField { source, field_name, .. } =>
                    field_name.is_empty() || !provenance.0.contains(source),
                OutputColumnOriginV2::Expression(key) => key.source_id != column.location.location().source_id,
            }
        ) {
            return Err("output columns differ from the typed row or authoritative origins");
        }
        Ok(Self { row, provenance, columns, cardinality, partition_keys, ordering, effects })
    }
    /// Row schema.
    #[must_use] pub const fn row(&self) -> &RecordTypeV1 { &self.row }
    /// Exact source origins.
    #[must_use] pub const fn provenance(&self) -> &OutputProvenanceV2 { &self.provenance }
    /// Ordered per-column producer proofs aligned one-to-one with row fields.
    #[must_use] pub fn columns(&self) -> &[OutputColumnContractV2] { &self.columns }
    /// Cardinality/multiplicity.
    #[must_use] pub const fn cardinality(&self) -> CardinalityV1 { self.cardinality }
    /// Retained grouping keys.
    #[must_use] pub fn partition_keys(&self) -> &[SourceQualifiedNodeIdV2] { &self.partition_keys }
    /// Retained ordering keys.
    #[must_use] pub fn ordering(&self) -> &[OrderKeyContractV2] { &self.ordering }
    /// Composed expression effects.
    #[must_use] pub const fn effects(&self) -> ExpressionEffectsV2 { self.effects }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationStageV2 {
    key: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
    inputs: Vec<Box<TypedRelationExprV2>>,
    expressions: Vec<SourceQualifiedNodeIdV2>,
    predicate: Option<TypedBoolExprV2>,
    source: Option<ResolvedScanSourceV2>,
    join_mode: Option<JoinModeV2>,
    output: RelationOutputContractV2,
}

/// The two source-reachable forms of the single HIR Join stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinModeV2 { Inner, Cross }

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
    pub(crate) output: RelationOutputContractV2,
}

/// Source-reachable stage identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedRelationStageKindV2 { Scan, Filter, Project, Join, Group, Sort }

impl TypedRelationExprV2 {
    /// Check exact stage arity and payload before sealing it. A caller cannot
    /// combine a semantic result from one stage with an unrelated stage kind.
    pub(crate) fn try_from_stage(input: TypedRelationStageInputV2) -> Result<Self, &'static str> {
        let TypedRelationStageInputV2 { key, location, kind, inputs, expressions, predicate, source, join_mode, output } = input;
        let valid = match kind {
            TypedRelationStageKindV2::Scan => inputs.is_empty() && expressions.is_empty() && predicate.is_none() && source.is_some() && join_mode.is_none()
                && output.columns().iter().all(|column| matches!(column.origin(),
                    OutputColumnOriginV2::SourceField { source: origin, scan_stage, .. }
                    if source.as_ref() == Some(origin) && *scan_stage == key)),
            TypedRelationStageKindV2::Filter => inputs.len() == 1 && expressions.is_empty() && predicate.is_some() && source.is_none() && join_mode.is_none(),
            TypedRelationStageKindV2::Project => inputs.len() == 1 && !expressions.is_empty() && predicate.is_none() && source.is_none() && join_mode.is_none(),
            TypedRelationStageKindV2::Join => inputs.len() == 2 && expressions.is_empty() && source.is_none() && matches!((join_mode, predicate.is_some()), (Some(JoinModeV2::Inner), true) | (Some(JoinModeV2::Cross), false)),
            TypedRelationStageKindV2::Group | TypedRelationStageKindV2::Sort => inputs.len() == 1 && !expressions.is_empty() && predicate.is_none() && source.is_none() && join_mode.is_none(),
        };
        if !valid || location.location().source_id != key.source_id
            || inputs.iter().any(|child| child.key().source_id != key.source_id
                || child.location().registry_identity() != location.registry_identity()) {
            return Err("invalid typed relation stage arity or payload");
        }
        let stage = stage(key, location, inputs.into_iter().map(Box::new).collect(), expressions, predicate, source, join_mode, output);
        Ok(match kind {
            TypedRelationStageKindV2::Scan => Self::Scan(stage),
            TypedRelationStageKindV2::Filter => Self::Filter(stage),
            TypedRelationStageKindV2::Project => Self::Project(stage),
            TypedRelationStageKindV2::Join => Self::Join(stage),
            TypedRelationStageKindV2::Group => Self::Group(stage),
            TypedRelationStageKindV2::Sort => Self::Sort(stage),
        })
    }

    /// Construct a scan from an already resolved authority source.
    #[must_use]
    pub(crate) fn scan(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, source: ResolvedScanSourceV2, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Scan, inputs: Vec::new(),
            expressions: Vec::new(), predicate: None, source: Some(source), join_mode: None, output,
        })
    }
    /// Construct a Bool-proven filter.
    #[must_use]
    pub(crate) fn filter(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, input: Self, predicate: TypedBoolExprV2, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Filter, inputs: vec![input],
            expressions: Vec::new(), predicate: Some(predicate), source: None, join_mode: None, output,
        })
    }
    /// Construct a nonempty projection.
    pub(crate) fn try_project(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, input: Self, expressions: Vec<SourceQualifiedNodeIdV2>, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Project, inputs: vec![input],
            expressions, predicate: None, source: None, join_mode: None, output,
        })
    }
    /// Construct an ON-checked inner join.
    #[must_use]
    pub(crate) fn join(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, left: Self, right: Self, predicate: TypedBoolExprV2, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Join, inputs: vec![left, right],
            expressions: Vec::new(), predicate: Some(predicate), source: None,
            join_mode: Some(JoinModeV2::Inner), output,
        })
    }
    /// Construct an explicit cross join.
    #[must_use]
    pub(crate) fn cross(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, left: Self, right: Self, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Join, inputs: vec![left, right],
            expressions: Vec::new(), predicate: None, source: None,
            join_mode: Some(JoinModeV2::Cross), output,
        })
    }
    /// Construct nonempty grouping.
    pub(crate) fn try_group(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, input: Self, keys: Vec<SourceQualifiedNodeIdV2>, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Group, inputs: vec![input],
            expressions: keys, predicate: None, source: None, join_mode: None, output,
        })
    }
    /// Construct nonempty ordering.
    pub(crate) fn try_sort(key: SourceQualifiedNodeIdV2, location: CheckedLocatedSpanV2, input: Self, keys: Vec<SourceQualifiedNodeIdV2>, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        Self::try_from_stage(TypedRelationStageInputV2 {
            key, location, kind: TypedRelationStageKindV2::Sort, inputs: vec![input],
            expressions: keys, predicate: None, source: None, join_mode: None, output,
        })
    }
    /// Retained exact output.
    #[must_use]
    pub const fn output(&self) -> &RelationOutputContractV2 {
        match self {
            Self::Scan(node) | Self::Filter(node) | Self::Project(node) | Self::Join(node)
            | Self::Group(node) | Self::Sort(node) => &node.output,
        }
    }
    /// Exact checked HIR stage identity retained for downstream provenance.
    #[must_use] pub const fn key(&self) -> SourceQualifiedNodeIdV2 { self.stage().key }
    /// Checked source location of the exact HIR relation stage.
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.stage().location }
    /// Typed child stages, in evaluation order.
    #[must_use]
    pub fn inputs(&self) -> impl Iterator<Item = &TypedRelationExprV2> {
        self.stage().inputs.iter().map(Box::as_ref)
    }
    /// Typed expression keys owned by this stage.
    #[must_use] pub fn expressions(&self) -> &[SourceQualifiedNodeIdV2] { &self.stage().expressions }
    /// Checked predicate, when required by this stage.
    #[must_use] pub const fn predicate(&self) -> Option<&TypedBoolExprV2> { self.stage().predicate.as_ref() }
    /// Source identity, present only for a scan.
    #[must_use] pub const fn source(&self) -> Option<&ResolvedScanSourceV2> { self.stage().source.as_ref() }
    /// Checked join subkind; absent on other stages.
    #[must_use] pub const fn join_mode(&self) -> Option<JoinModeV2> { self.stage().join_mode }

    const fn stage(&self) -> &RelationStageV2 {
        match self {
            Self::Scan(node) | Self::Filter(node) | Self::Project(node) | Self::Join(node)
            | Self::Group(node) | Self::Sort(node) => node,
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
    pub(crate) fn try_new(item_id: SemanticItemIdV2, location: CheckedLocatedSpanV2, expected_root: SourceQualifiedNodeIdV2, root: TypedRelationExprV2, output: RelationOutputContractV2) -> Result<Self, &'static str> {
        if root.output() != &output || root.key() != expected_root || expected_root.source_id != item_id.source_id()
            || location.location().source_id != item_id.source_id()
            || root.location().registry_identity() != location.registry_identity() {
            return Err("relation root/output contract mismatch");
        }
        Ok(Self { item_id, location, root, output })
    }
    /// Item ID.
    #[must_use] pub const fn item_id(&self) -> SemanticItemIdV2 { self.item_id }
    /// Checked source location of the authored query item.
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.location }
    /// Typed root.
    #[must_use] pub const fn root(&self) -> &TypedRelationExprV2 { &self.root }
    /// Retained output.
    #[must_use] pub const fn output(&self) -> &RelationOutputContractV2 { &self.output }
}

fn stage(
    key: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
    inputs: Vec<Box<TypedRelationExprV2>>,
    expressions: Vec<SourceQualifiedNodeIdV2>,
    predicate: Option<TypedBoolExprV2>,
    source: Option<ResolvedScanSourceV2>,
    join_mode: Option<JoinModeV2>,
    output: RelationOutputContractV2,
) -> RelationStageV2 {
    RelationStageV2 { key, location, inputs, expressions, predicate, source, join_mode, output }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quanta_gqlang_v2_hir::{HirNodeIdV1, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1};
    use quanta_gqlang_v2_types::RecordFieldTypeV1;

    fn empty_output() -> RelationOutputContractV2 {
        RelationOutputContractV2::try_new(
            RecordTypeV1::try_new(Vec::new()).expect("valid empty row"),
            OutputProvenanceV2::default(), Vec::new(), CardinalityV1::Bag, Vec::new(), Vec::new(),
            ExpressionEffectsV2::pure_scalar(),
        ).expect("valid output")
    }

    #[test]
    fn output_columns_require_exact_row_alignment_and_expression_source() {
        let source_id = SourceIdV2::from_raw(8);
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 { source_id, source_len: 2 }]).expect("valid registry");
        let location = registry.try_check_location(LocatedSpanV2 { source_id, span: SpanV1::new(0, 1) }).expect("valid location");
        let row = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new("value", ValueTypeV1::int(), location.clone()).expect("valid field")]).expect("valid row");
        let key = SourceQualifiedNodeIdV2 { source_id, node_id: HirNodeIdV1::from_raw(9) };
        let column = OutputColumnContractV2::new("value".into(), OutputColumnOriginV2::Expression(key), location.clone());
        let build = |columns| RelationOutputContractV2::try_new(row.clone(), OutputProvenanceV2::default(), columns,
            CardinalityV1::Bag, Vec::new(), Vec::new(), ExpressionEffectsV2::pure_scalar());
        let output = build(vec![column.clone()]).expect("one typed column");
        assert_eq!(output.columns()[0].origin(), &OutputColumnOriginV2::Expression(key));
        assert_eq!(build(Vec::new()).expect_err("missing origin"), "output columns differ from the typed row or authoritative origins");
        let wrong_name = OutputColumnContractV2::new("other".into(), OutputColumnOriginV2::Expression(key), location.clone());
        assert_eq!(build(vec![wrong_name]).expect_err("misaligned row"), "output columns differ from the typed row or authoritative origins");
        let foreign = OutputColumnContractV2::new("value".into(), OutputColumnOriginV2::Expression(SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(10), node_id: key.node_id,
        }), location);
        assert_eq!(build(vec![foreign]).expect_err("foreign expression"), "output columns differ from the typed row or authoritative origins");
    }

    #[test]
    fn join_mode_cannot_disagree_with_predicate_and_six_stage_vocabulary() {
        let source_id = SourceIdV2::from_raw(8);
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 { source_id, source_len: 2 }]).expect("valid registry");
        let location = registry.try_check_location(LocatedSpanV2 { source_id, span: SpanV1::new(0, 1) }).expect("valid location");
        let forged = TypedRelationStageInputV2 {
            key: SourceQualifiedNodeIdV2 { source_id, node_id: HirNodeIdV1::from_raw(3) }, location,
            kind: TypedRelationStageKindV2::Join,
            inputs: Vec::new(), expressions: Vec::new(), predicate: None,
            source: None, join_mode: Some(JoinModeV2::Inner), output: empty_output(),
        };
        assert_eq!(TypedRelationExprV2::try_from_stage(forged).expect_err("inner join needs ON and both inputs"), "invalid typed relation stage arity or payload");
    }

    #[test]
    fn duplicate_output_keys_are_rejected_by_the_only_constructor() {
        let key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(8), node_id: HirNodeIdV1::from_raw(11) };
        assert_eq!(RelationOutputContractV2::try_new(
            RecordTypeV1::try_new(Vec::new()).expect("valid row"),
            OutputProvenanceV2::default(), Vec::new(), CardinalityV1::Bag,
            vec![key, key], Vec::new(), ExpressionEffectsV2::pure_scalar(),
        ).expect_err("duplicate keys"), "duplicate partition or order key");
        let order = OrderKeyContractV2::new(key, SortDirectionHirV1::Desc, NullOrderV2::Last, ValueTypeV1::int());
        assert_eq!(RelationOutputContractV2::try_new(
            RecordTypeV1::try_new(Vec::new()).expect("valid row"),
            OutputProvenanceV2::default(), Vec::new(), CardinalityV1::Bag,
            Vec::new(), vec![order.clone(), order], ExpressionEffectsV2::pure_scalar(),
        ).expect_err("duplicate order key"), "duplicate partition or order key");
    }
}
