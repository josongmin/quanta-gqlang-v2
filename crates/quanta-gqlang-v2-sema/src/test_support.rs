//! Feature-gated reference transactions for downstream owner tests.
//!
//! This module is not a frontend API. It gives dependent crate tests a real
//! sealed document without making them depend directly on HIR, Modules, or the
//! authority builders. The positive fixtures cover only stable, legal relation
//! shapes.
//!
//! The `forged_*` builders additionally mint deliberately mutated semantic
//! carriers through `pub(crate)` seams that only exist inside this crate. They
//! exist exclusively for the ticket-05 algebra mutation rails: every
//! downstream mutation test needs a shape no public constructor can produce.
//! Nothing in this module is compiled without the `test` feature, and the
//! module is `#[doc(hidden)]`; no fixture can leak into a non-test build.
//!
//! Beyond the relation/graph/nested-CHECK/rule families, the document families
//! downstream planning and pipeline rails need are:
//!
//! - meta directives ([`reference_meta_document_v2`]): `REQUIRE CERTIFIED` and
//!   `USE PACK` both mint through the real document transaction;
//! - presentations ([`reference_presentation_document_v2`]): all six verbs,
//!   with every EXPLAIN evidence/truth permutation including the two planning
//!   refuses;
//! - compositions ([`reference_composition_document_v2`]): all four closed body
//!   families over ordered LET producers;
//! - small-receipt rule programs
//!   ([`forged_small_receipt_rule_document_v2`] and
//!   [`forged_small_receipt_rule_program_document_v2`]);
//! - narrow-witness and alternate-identity FIND documents
//!   ([`reference_narrow_witness_graph_document_v2`],
//!   [`reference_alternate_graph_document_v2`]).
//!
//! Fixtures exist only for authorable source shapes. Retired algebra variants
//! are not represented by synthetic semantic carriers.

use crate::{
    resolve_checked_document_v2, CheckedLocatedSpanV2, CheckedSyntaxDocumentV2,
    ClosedFrontendAuthorityV2, DeclaredDocumentV2, DownstreamBudgetReceiptV2, FrontendBudgetV2,
    GraphReferenceCatalogV2, GraphReferenceInputV2, GraphReferenceKindV2, GraphReferenceRoleV2,
    JoinModeV2, OutputColumnContractV2, OutputColumnOriginV2, OutputProvenanceV2,
    RelationOutputContractV2, ResolvedBindingKindV2, ResolvedCompositionSourceIdV2,
    ResolvedCompositionV2, ResolvedGraphQueryV2, ResolvedMetaKindV2, ResolvedPresentationV2,
    ResolvedScanSourceV2, ResolvedValueBindingV2, SemanticDocumentV2, SemanticItemIdV2,
    SemanticItemV2, TypedBoolExprV2, TypedCorrelationEdgeV2, TypedExprShapeV2, TypedExprV2,
    TypedRelationExprV2, TypedRelationStageInputV2, TypedRelationStageKindV2, TypedRelationV2,
};
use quanta_gqlang_v2_hir::{
    AliasV1, CallArgHirV1, CallArgsHirV1, CompareMetricV1, CompositionBindingV1, CompositionBodyV1,
    CompositionHirV1, CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExplainHirV1, ExportFormatV1, GraphConstraintsV1, GraphDirectionV1,
    GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1, HirBuilderV1, HirDocumentV1,
    HirNodeIdV1, LegalityHirV1, LocatedSpanV2, MetaHirV1, MetaKindV1, OrderKeyHirV1, PathSegmentV1,
    PresentationHirV1, PresentationKindV1, ProgramItemHirV1, ProjectionItemV1, QueryHirV1,
    RelationExistsPolarityV1, RelationExprHirV1, SampleHirV1, SampleModeV1, ShowCategoryV1,
    SortDirectionHirV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
    TruthModeV1, UsePackActivationV1, UsePackHirV1, ValueExprHirV1,
};
use quanta_gqlang_v2_modules::{
    CallableKindV2, ContentDigestV2, CurrentModuleSelectionInputV2, DocumentCatalogFinalInputV2,
    DocumentCatalogValidatorV2, ExternalCatalogAssetInputV2, ExternalCatalogPayloadInputV2,
    LocatedParameterInputV2, LockReferenceInputV2, ModuleAuthorityBuilderV2, ModuleIdV2,
    ModuleNodeInputV2, PackIdV2, PackManifestInputV2, SignatureInputV2, VersionRangeV2, VersionV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, ExpressionEffectsV2, ProvenanceNamespaceV2,
    RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2, RelationSourceInputV2,
    SourceQualifiedNodeIdV2, ValueTypeV1,
};
use std::collections::BTreeMap;

/// Test-only SHOW HIR storage demand; downstream budget tests need no HIR edge.
#[must_use]
pub fn show_hir_storage_bytes_v2(item_count: usize) -> usize {
    std::mem::size_of::<Vec<ProgramItemHirV1>>()
        + item_count
            * (std::mem::size_of::<ProgramItemHirV1>() + std::mem::size_of::<PresentationHirV1>())
}

/// Independent output mutations for checked-stage constructor controls.
#[derive(Clone, Copy, Debug)]
pub enum OutputContractMutationV2 {
    SetCardinality,
    SequenceCardinality,
    FirstFieldType,
    FirstFieldOptionality,
    Partition,
    Ordering,
}

/// Rebuild a row-aligned output while changing one stage-specific fact.
/// The generic output constructor must still accept it; the stage constructor
/// is responsible for rejecting its disagreement with the actual operands.
pub fn forged_output_contract_v2(
    original: &RelationOutputContractV2,
    stage: SourceQualifiedNodeIdV2,
    mutation: OutputContractMutationV2,
) -> Result<RelationOutputContractV2, String> {
    let mut row = original.row().clone();
    let mut cardinality = original.cardinality();
    let mut partition = original.partition_keys().to_vec();
    let mut ordering = original.ordering().to_vec();
    match mutation {
        OutputContractMutationV2::SetCardinality => cardinality = CardinalityV1::Set,
        OutputContractMutationV2::SequenceCardinality => cardinality = CardinalityV1::Sequence,
        OutputContractMutationV2::FirstFieldType
        | OutputContractMutationV2::FirstFieldOptionality => {
            if row.fields().is_empty() {
                return Err("row mutation requires a field".to_owned());
            }
            let fields = row
                .fields()
                .iter()
                .zip(original.columns())
                .enumerate()
                .map(|(i, (field, column))| {
                    let value_type = if i != 0 {
                        field.value_type().clone()
                    } else if matches!(mutation, OutputContractMutationV2::FirstFieldOptionality) {
                        ValueTypeV1::try_optional(field.value_type().clone())
                            .map_err(|error| format!("{error:?}"))?
                    } else if field.value_type() == &ValueTypeV1::string() {
                        ValueTypeV1::int()
                    } else {
                        ValueTypeV1::string()
                    };
                    RecordFieldTypeV1::try_new(field.name(), value_type, column.location().clone())
                        .map_err(|error| format!("{error:?}"))
                })
                .collect::<Result<Vec<_>, String>>()?;
            row = RecordTypeV1::try_new(fields).map_err(|error| format!("{error:?}"))?;
        }
        OutputContractMutationV2::Partition => partition = vec![stage],
        OutputContractMutationV2::Ordering => {
            ordering = vec![crate::OrderKeyContractV2::new(
                stage,
                crate::SortDirectionV2::Asc,
                crate::NullOrderV2::Last,
                ValueTypeV1::int(),
            )]
        }
    }
    RelationOutputContractV2::try_new(
        row,
        original.provenance().clone(),
        original.columns().to_vec(),
        cardinality,
        partition,
        ordering,
        original.effects(),
    )
    .map_err(str::to_owned)
}

/// Fixed source identity used by reference relation fixtures.
pub const REFERENCE_SOURCE_V2: SourceIdV2 = SourceIdV2::from_raw(9);

/// Second fixed source identity, resolved against a different host registry
/// identity (registration length 512 instead of 256), for cross-document
/// foreign-fact rails.
pub const SECONDARY_REFERENCE_SOURCE_V2: SourceIdV2 = SourceIdV2::from_raw(10);

/// `max_semantic_steps` of [`ReferenceBudgetV2::Small`].
///
/// A reference filter chain of depth `d` charges `2 * d + 3` semantic steps
/// (one item charge, `d + 1` relation stages, `d` Bool literals, and one fixed
/// document charge), so depth
/// [`SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH`] resolves inside this limit and
/// one more filter exceeds it.
pub const SMALL_BUDGET_MAX_SEMANTIC_STEPS: u32 = 8;

/// `max_expression_nodes` of [`ReferenceBudgetV2::Small`].
///
/// Sema resolution does not charge expression nodes today; downstream
/// expression walks read this dimension from the document's immutable budget
/// receipt. The forged small-receipt builders pair it with oversized trees for
/// the exact/limit+1 rails.
pub const SMALL_BUDGET_MAX_EXPRESSION_NODES: u32 = 8;

/// Deepest reference filter chain that resolves under
/// [`ReferenceBudgetV2::Small`] (charges exactly
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] steps).
pub const SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH: u32 = 2;

/// First reference filter chain depth rejected by [`ReferenceBudgetV2::Small`]
/// (charges 10 steps).
pub const SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH: u32 = 3;

/// Deepest reference Bool tree that resolves under [`ReferenceBudgetV2::Small`]
/// (6 semantic steps, 2 expression facts; every leaf is a `BoolValue` wrapping
/// one Bool literal, so a left-leaning chain of depth `d` carries `3 * d + 2`
/// expression facts and costs `3 * d + 6` semantic steps).
pub const SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH: u32 = 0;

/// First reference Bool tree depth rejected by [`ReferenceBudgetV2::Small`] (9
/// semantic steps).
pub const SMALL_BUDGET_BOOL_TREE_OVER_DEPTH: u32 = 1;

/// Largest chain depth the reference deep builders author.
pub const MAX_REFERENCE_DEPTH_V2: u32 = 512;

/// An item identity guaranteed to be absent from every reference fixture.
#[must_use]
pub fn reference_absent_item_id_v2() -> SemanticItemIdV2 {
    SemanticItemIdV2::new(REFERENCE_SOURCE_V2, HirNodeIdV1::from_raw(u32::MAX))
}

/// Budget authority used by a parameterized reference fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceBudgetV2 {
    /// `FrontendBudgetV2::standard_v2()`; deep shapes of any reference depth
    /// resolve.
    Standard,
    /// Every standard dimension except `max_semantic_steps =
    /// `[`SMALL_BUDGET_MAX_SEMANTIC_STEPS`]` and `max_expression_nodes =
    /// `[`SMALL_BUDGET_MAX_EXPRESSION_NODES`].
    Small,
}

/// Legal source relation shapes available to downstream tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceRelationFixtureV2 {
    /// One authoritative Scan.
    Scan,
    /// Scan followed by a Bool-proven Filter.
    Filter,
    /// Two scans combined by an explicit predicate-free Cross join.
    CrossJoin,
    /// A relational EXISTS predicate over an uncorrelated nested scan.
    Exists,
    /// A relational NOT EXISTS predicate over an uncorrelated nested scan.
    NotExists,
    /// Projection of one Int path expression and one String literal
    /// expression, each with an exact `Expression` output origin.
    ProjectMultipleExpressions,
    /// Inner join with an authored ON comparison over two distinct host scans
    /// (`records r`, `other o`).
    InnerJoinOn,
    /// Self inner join over two distinct scan occurrences of one host
    /// relation, each side projected first, joined on the projected aliases.
    SelfJoinInner,
    /// Authored `group by r.id` (exactly one Group stage) followed by an
    /// aggregate projection `total: sum(r.id)` over the one-field `records`
    /// relation.
    GroupedAggregateSum,
    /// Authored `group by r.id, r.value` (exactly one Group stage carrying
    /// two partition keys in authored order) followed by an aggregate
    /// projection `total: sum(r.id)`.
    GroupedAggregateTwoPartitionKeys,
    /// Scalar aggregate `total: count(*)` through the producer-sealed implicit
    /// empty-partition Group.
    ScalarCountStar,
    /// Single-key ascending sort over a host Int field.
    SortAsc,
    /// Single-key descending sort over a host Int field.
    SortDesc,
    /// Descending sort with NULLS LAST over the nullable `label` field of the
    /// reference-wide relation (an `Option<String>` column).
    SortNullsLastNullableField,
    /// Two-key sort with mixed directions over two host fields.
    SortMultiKeyMixedDirections,
    /// Sort keyed by a projected output alias (a Binder-owned projection
    /// binding, not a host field).
    SortAliasKeyed,
    /// Sort keyed by a parenthesized expression over a projected output alias.
    SortByProjectedExpression,
    /// Correlated relational EXISTS: the nested scan captures the outer alias
    /// `r` at one inner reference, sealed as one `TypedCorrelationEdgeV2`.
    CorrelatedExists,
    /// Correlated relational NOT EXISTS with the same single capture edge.
    CorrelatedNotExists,
    /// Two uncorrelated relational EXISTS predicates in one conjunctive Filter
    /// predicate.
    ConjunctiveExistsPair,
    /// Two uncorrelated relational EXISTS predicates in one disjunctive Filter
    /// predicate (EXISTS under Or).
    DisjunctiveExistsPair,
    /// One relational EXISTS predicate under logical Not in a Filter
    /// predicate (EXISTS under Not, distinct from the relational NOT EXISTS
    /// polarity of [`ReferenceRelationFixtureV2::NotExists`]).
    NegatedExists,
    /// One ordinary Bool conjunct and one relational EXISTS conjunct in the
    /// same Filter predicate.
    ExistsWithBoolConjunct,
    /// Relational EXISTS nested two levels deep, retaining separate outer and
    /// inner capture sets.
    NestedExistsTwoLevels,
    /// A relational EXISTS whose RIGHT semi input is MULTIPLICATIVE: the nested
    /// arm cross-joins two host scans. The semi/anti classification must still
    /// be its left arm's class, because the right arm only decides existence.
    ExistsWithMultiplicativeRight,
    /// Cross join with structurally DISTINGUISHABLE arms for preorder proof:
    /// left is `Project -> Filter -> Scan`, right is `Sort -> Scan`
    /// (EXPLAIN-01). Symmetric join fixtures cannot discriminate a
    /// left/right traversal swap; this one can.
    AsymmetricJoinPreorder,
}

/// A sealed reference transaction plus its one relation item identity.
#[derive(Clone, Debug)]
pub struct ReferenceRelationDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferenceRelationDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the sole relation item.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
}

/// Resolve one reference relation fixture through the real semantic transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_relation_document_v2(
    fixture: ReferenceRelationFixtureV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let (relation, source_len) = reference_relation_hir(fixture, &builder)?;
    resolve_reference(
        &builder,
        relation,
        source_len,
        REFERENCE_SOURCE_V2,
        reference_authority_v2,
    )
}

/// Resolve the reference Scan fixture under [`SECONDARY_REFERENCE_SOURCE_V2`]
/// against a second host registry identity. Every stage, expression, and
/// location of this document is foreign to the primary reference documents and
/// vice versa.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn second_source_relation_document_v2() -> Result<ReferenceRelationDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    resolve_reference(
        &builder,
        scan,
        12,
        SECONDARY_REFERENCE_SOURCE_V2,
        secondary_authority_v2,
    )
}

/// Author and resolve a chain of `depth` Bool-literal Filters over one scan.
///
/// The chain charges `2 * depth + 3` semantic steps: under
/// [`ReferenceBudgetV2::Small`] depth
/// [`SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH`] resolves inside the step limit
/// and [`SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH`] rejects with
/// `frontend.resource_limit_exceeded`.
///
/// # Errors
///
/// Returns a fixture-construction error when `depth` exceeds
/// [`MAX_REFERENCE_DEPTH_V2`] or the semantic transaction rejects the chain.
pub fn deep_relation_chain_document_v2(
    depth: u32,
    budget: ReferenceBudgetV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    check_depth(depth)?;
    let builder = HirBuilderV1::new();
    let mut relation =
        builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    for index in 0..usize::try_from(depth).map_err(|_error| "depth overflow".to_owned())? {
        let literal_start = 40 + 4 * index;
        let literal = builder.bool_literal(true, SpanV1::new(literal_start, literal_start + 4));
        relation = builder.relation_filter(relation, literal, SpanV1::new(5, literal_start + 4));
    }
    let source_len =
        48 + 4 * usize::try_from(depth).map_err(|_error| "depth overflow".to_owned())?;
    resolve_reference_with_budget(&builder, relation, source_len, budget)
}

/// Author and resolve one scan filtered by a left-leaning conjunction whose
/// leaves are `BoolValue`-wrapped Bool literals (`3 * depth + 2` expression
/// facts, `3 * depth + 6` semantic steps).
///
/// # Errors
///
/// Returns a fixture-construction error when `depth` exceeds
/// [`MAX_REFERENCE_DEPTH_V2`] or the semantic transaction rejects the tree.
pub fn deep_bool_tree_document_v2(
    depth: u32,
    budget: ReferenceBudgetV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    check_depth(depth)?;
    let builder = HirBuilderV1::new();
    let (filter, source_len) = deep_bool_tree_parts(&builder, depth)?;
    resolve_reference_with_budget(&builder, filter, source_len, budget)
}

/// Author the shared deep Bool-tree fixture relation: one scan filtered by a
/// left-leaning conjunction of `depth + 1` `BoolValue`-wrapped Bool literals
/// (`depth` And nodes, `depth + 1` BoolValue nodes, and `depth + 1` literal
/// children, so `3 * depth + 2` expression facts). Both the resolving path and
/// the checked-document path must author the identical node sequence: the
/// forged small-receipt rail re-seals facts minted by one under the declared
/// identity of the other.
fn deep_bool_tree_parts(
    builder: &HirBuilderV1,
    depth: u32,
) -> Result<(RelationExprHirV1, usize), String> {
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let mut tree = bool_value_leaf(builder, 20);
    for index in 0..usize::try_from(depth).map_err(|_error| "depth overflow".to_owned())? {
        let leaf_start = 30 + 4 * index;
        let leaf = bool_value_leaf(builder, leaf_start);
        tree = builder.and(tree, leaf, SpanV1::new(20, leaf_start + 4));
    }
    let end = 30 + 4 * usize::try_from(depth).map_err(|_error| "depth overflow".to_owned())?;
    let filter = builder.relation_filter(scan, tree, SpanV1::new(5, end));
    Ok((filter, end + 8))
}

/// Exact mutation applied by [`forged_capture_edge_set_v2`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedCaptureEdgeMutationV2 {
    /// Only the first edge remains; the second capture is dropped.
    Dropped,
    /// The first edge appears twice; the duplicate reference must reject.
    Duplicate,
    /// Both edges retained in reversed stable order.
    Swapped,
    /// The second edge references a node absent from every reference document
    /// expression set (a foreign capture reference).
    Foreign,
}

/// The pristine two-edge capture vector the mutations start from: one outer
/// alias binding captured at two distinct inner reference nodes.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing registry or binding
/// identity can no longer be minted.
pub fn pristine_capture_edge_set_v2() -> Result<Vec<TypedCorrelationEdgeV2>, String> {
    mint_capture_edges(None)
}

/// A mutated two-edge correlation capture vector for the Semi/Anti join
/// correlation rails. The vector cannot be assembled outside the sema crate:
/// [`TypedCorrelationEdgeV2`] has a `pub(crate)` constructor, and no sealed
/// document can carry the duplicate or foreign variants (the document
/// finalizer rejects both).
///
/// # Errors
///
/// Returns a fixture-construction error if the backing registry or binding
/// identity can no longer be minted.
pub fn forged_capture_edge_set_v2(
    mutation: ForgedCaptureEdgeMutationV2,
) -> Result<Vec<TypedCorrelationEdgeV2>, String> {
    mint_capture_edges(Some(mutation))
}

/// Attempt to seal a Join stage input carrying `JoinModeV2::Inner` with no ON
/// predicate. The producer-only input DTO is assembled through its `pub(crate)`
/// fields; the expected result is the rejection
/// `Err("invalid typed relation stage arity or payload")`, proving an
/// Inner-without-ON relation input can never become a sealed stage.
///
/// # Errors
///
/// The rejection itself: `Err` with the exact constructor diagnostic.
pub fn forged_inner_join_without_on_seal_v2() -> Result<TypedRelationExprV2, &'static str> {
    let (left, right, location, key, output) = forged_join_seal_parts()?;
    let input = TypedRelationStageInputV2 {
        key,
        location,
        kind: TypedRelationStageKindV2::Join,
        inputs: vec![left, right],
        expressions: Vec::new(),
        predicate: None,
        source: None,
        join_mode: Some(JoinModeV2::Inner),
        implicit_partition: false,
        partition_input: Vec::new(),
        output,
    };
    TypedRelationExprV2::try_from_stage(input)
}

/// Attempt to seal a Join stage input carrying `JoinModeV2::Cross` with an ON
/// predicate. The expected result is the rejection
/// `Err("invalid typed relation stage arity or payload")`, proving a
/// Cross-with-ON relation input can never become a sealed stage.
///
/// # Errors
///
/// The rejection itself: `Err` with the exact constructor diagnostic.
pub fn forged_cross_join_with_on_seal_v2() -> Result<TypedRelationExprV2, &'static str> {
    let (left, right, location, key, output) = forged_join_seal_parts()?;
    let predicate = reference_bool_predicate()?;
    let input = TypedRelationStageInputV2 {
        key,
        location,
        kind: TypedRelationStageKindV2::Join,
        inputs: vec![left, right],
        expressions: Vec::new(),
        predicate: Some(predicate),
        source: None,
        join_mode: Some(JoinModeV2::Cross),
        implicit_partition: false,
        partition_input: Vec::new(),
        output,
    };
    TypedRelationExprV2::try_from_stage(input)
}

/// Forged document for the `algebra.aggregate_without_group` rail, minted from
/// the real scalar `count(*)` fixture: the producer-sealed implicit
/// empty-partition Group is removed, so the aggregate projection sits directly
/// over the Scan while every expression fact and the aggregate output contract
/// stay untouched.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_aggregate_without_group_document_v2() -> Result<ReferenceRelationDocumentV2, String> {
    let (checked, real, item_id) = scalar_aggregate_parts()?;
    let relation = relation_item(&real, item_id)?;
    let group = relation
        .root()
        .inputs()
        .next()
        .ok_or("aggregate projection has no group input")?;
    let scan = group
        .inputs()
        .next()
        .ok_or("implicit group has no scan input")?
        .clone();
    let forged_root = TypedRelationExprV2::try_project(
        relation.root().key(),
        relation.root().location().clone(),
        scan,
        relation.root().expressions().to_vec(),
        relation.output().clone(),
    )
    .map_err(str::to_owned)?;
    reassemble_with_forged_relation(&checked, &real, item_id, forged_root)
}

/// Forged document for the mixed scalar/aggregate projection rail, minted from
/// the real grouped-sum fixture: the aggregate projection additionally carries
/// the plain scalar grouping-path expression as a second output column, a
/// shape the sealed aggregate constructors reject but a structurally valid
/// ordinary projection can carry.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_mixed_scalar_aggregate_projection_document_v2(
) -> Result<ReferenceRelationDocumentV2, String> {
    let (checked, real, item_id) = grouped_aggregate_parts()?;
    let relation = relation_item(&real, item_id)?;
    let group = relation
        .root()
        .inputs()
        .next()
        .ok_or("aggregate projection has no group input")?
        .clone();
    let key_expression = group
        .expressions()
        .first()
        .copied()
        .ok_or("authored group has no key expression")?;
    let sum_expression = relation
        .root()
        .expressions()
        .first()
        .copied()
        .ok_or("aggregate projection has no expression")?;
    let key_column = group
        .output()
        .columns()
        .first()
        .ok_or("group output has no key column")?;
    let total_column = relation
        .output()
        .columns()
        .first()
        .ok_or("aggregate output has no column")?;
    let mut row_fields = Vec::new();
    for (field, column) in group
        .output()
        .row()
        .fields()
        .iter()
        .chain(relation.output().row().fields().iter())
        .zip(std::iter::once(key_column).chain(std::iter::once(total_column)))
    {
        row_fields.push(
            RecordFieldTypeV1::try_new(
                field.name(),
                field.value_type().clone(),
                column.location().clone(),
            )
            .map_err(|error| format!("{error:?}"))?,
        );
    }
    let row = RecordTypeV1::try_new(row_fields).map_err(|error| format!("{error:?}"))?;
    let mixed_columns = vec![
        OutputColumnContractV2::new(
            key_column.name().to_owned(),
            OutputColumnOriginV2::Expression(key_expression),
            key_column.location().clone(),
        ),
        OutputColumnContractV2::new(
            total_column.name().to_owned(),
            OutputColumnOriginV2::Expression(sum_expression),
            total_column.location().clone(),
        ),
    ];
    let mixed_output = RelationOutputContractV2::try_new(
        row,
        group.output().provenance().clone(),
        mixed_columns,
        CardinalityV1::Set,
        group.output().partition_keys().to_vec(),
        Vec::new(),
        ExpressionEffectsV2 {
            aggregation: AggregationClassV1::Aggregate,
            ..relation.output().effects()
        },
    )
    .map_err(str::to_owned)?;
    let forged_root = TypedRelationExprV2::try_project(
        relation.root().key(),
        relation.root().location().clone(),
        group,
        vec![key_expression, sum_expression],
        mixed_output,
    )
    .map_err(str::to_owned)?;
    reassemble_with_forged_relation(&checked, &real, item_id, forged_root)
}

/// Forged document for the cardinality-downgrade rail, minted from the real
/// grouped-sum fixture: the aggregate projection output contract is rebuilt
/// through the ordinary output constructor with `CardinalityV1::Bag`, a
/// caller-chosen cardinality on the Set-locked aggregate stage.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_cardinality_downgrade_document_v2() -> Result<ReferenceRelationDocumentV2, String> {
    let (checked, real, item_id) = grouped_aggregate_parts()?;
    let relation = relation_item(&real, item_id)?;
    let group = relation
        .root()
        .inputs()
        .next()
        .ok_or("aggregate projection has no group input")?
        .clone();
    let downgraded = RelationOutputContractV2::try_new(
        relation.output().row().clone(),
        relation.output().provenance().clone(),
        relation
            .output()
            .columns()
            .iter()
            .map(|column| {
                OutputColumnContractV2::new(
                    column.name().to_owned(),
                    column.origin().clone(),
                    column.location().clone(),
                )
            })
            .collect(),
        CardinalityV1::Bag,
        group.output().partition_keys().to_vec(),
        Vec::new(),
        relation.output().effects(),
    )
    .map_err(str::to_owned)?;
    let forged_root = TypedRelationExprV2::try_project(
        relation.root().key(),
        relation.root().location().clone(),
        group,
        relation.root().expressions().to_vec(),
        downgraded,
    )
    .map_err(str::to_owned)?;
    reassemble_with_forged_relation(&checked, &real, item_id, forged_root)
}

/// Attempt a same-source, wrong-occurrence transplant from a real self-join.
/// Project sealing must reject the SourceField origin before assembly.
#[cfg(test)]
fn try_wrong_scan_occurrence_projection_v2() -> Result<(), String> {
    let real = reference_relation_document_v2(ReferenceRelationFixtureV2::SelfJoinInner)?;
    let relation = relation_item(real.document(), real.item_id())?;
    let mut join_inputs = relation.root().inputs();
    let left_project = join_inputs.next().ok_or("join has no left input")?.clone();
    let right_project = join_inputs.next().ok_or("join has no right input")?.clone();
    if join_inputs.next().is_some() {
        return Err("join has more than two inputs".to_owned());
    }
    let left_scan = left_project
        .inputs()
        .next()
        .ok_or("left projection has no scan")?
        .clone();
    let right_scan = right_project
        .inputs()
        .next()
        .ok_or("right projection has no scan")?
        .clone();
    if left_scan.key() == right_scan.key() {
        return Err("self-join requires distinct scan occurrences".to_owned());
    }
    let scan_source = left_scan
        .source()
        .cloned()
        .ok_or("left scan has no resolved source")?;
    let left_column = left_project
        .output()
        .columns()
        .first()
        .ok_or("left projection has no column")?;
    let transplanted_column = OutputColumnContractV2::new(
        left_column.name().to_owned(),
        OutputColumnOriginV2::SourceField {
            source: scan_source,
            scan_stage: right_scan.key(),
            field_name: "id".to_owned(),
        },
        left_column.location().clone(),
    );
    let transplanted_output = RelationOutputContractV2::try_new(
        left_project.output().row().clone(),
        left_project.output().provenance().clone(),
        vec![transplanted_column],
        left_project.output().cardinality(),
        left_project.output().partition_keys().to_vec(),
        Vec::new(),
        left_project.output().effects(),
    )
    .map_err(str::to_owned)?;
    TypedRelationExprV2::try_project(
        left_project.key(),
        left_project.location().clone(),
        left_scan,
        left_project.expressions().to_vec(),
        transplanted_output,
    )
    .map(|_| ())
    .map_err(str::to_owned)
}

/// Forged document for the wrong-nested-root rail, minted from the real
/// correlated EXISTS fixture: the nested relation registered under the EXISTS
/// root is replaced by a structurally valid but different tree (a fresh scan
/// occurrence of the same host relation under a fresh node id at the same
/// checked location), so a consumer that trusts the registry key alone lowers
/// the wrong EXISTS right input.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_wrong_nested_root_document_v2() -> Result<ReferenceRelationDocumentV2, String> {
    let (checked, real, item_id) = correlated_exists_parts()?;
    let exists_key = exists_root(&real)?;
    let real_nested = real
        .nested_relation(exists_key)
        .ok_or("correlated fixture has no nested relation")?
        .clone();
    if !matches!(real_nested, TypedRelationExprV2::Filter(_)) {
        return Err("nested relation is not a filter".to_owned());
    }
    let real_scan = real_nested
        .inputs()
        .next()
        .ok_or("nested filter has no scan")?
        .clone();
    let scan_source = real_scan.source().cloned().ok_or("scan source expected")?;
    let fresh_scan_key = SourceQualifiedNodeIdV2 {
        source_id: REFERENCE_SOURCE_V2,
        node_id: HirNodeIdV1::from_raw(9_001),
    };
    let fresh_scan_output = RelationOutputContractV2::try_new(
        real_scan.output().row().clone(),
        real_scan.output().provenance().clone(),
        real_scan
            .output()
            .columns()
            .iter()
            .map(|column| {
                OutputColumnContractV2::new(
                    column.name().to_owned(),
                    OutputColumnOriginV2::SourceField {
                        source: scan_source.clone(),
                        scan_stage: fresh_scan_key,
                        field_name: column.name().to_owned(),
                    },
                    column.location().clone(),
                )
            })
            .collect(),
        real_scan.output().cardinality(),
        Vec::new(),
        Vec::new(),
        real_scan.output().effects(),
    )
    .map_err(str::to_owned)?;
    let fresh_scan = TypedRelationExprV2::scan(
        fresh_scan_key,
        real_scan.location().clone(),
        scan_source,
        fresh_scan_output,
    )
    .map_err(str::to_owned)?;
    let wrong_tree = TypedRelationExprV2::filter(
        real_nested.key(),
        real_nested.location().clone(),
        fresh_scan,
        real_nested
            .predicate()
            .ok_or("nested filter has no predicate")?
            .clone(),
        real_nested.output().clone(),
    )
    .map_err(str::to_owned)?;
    let (mut items, expressions, mut nested) = document_parts(&real);
    let relation = relation_item(&real, item_id)?.clone();
    items.retain(|item| !matches!(item, SemanticItemV2::Relation(r) if r.item_id() == item_id));
    items.push(SemanticItemV2::Relation(relation));
    nested.insert(exists_key, wrong_tree);
    let document = reassemble_document(
        &checked,
        &reference_authority_v2()?,
        items,
        expressions,
        nested,
        real.budget_receipt(),
    )?;
    Ok(ReferenceRelationDocumentV2 { document, item_id })
}

/// Forged document for the correlation capture-mutation rails, minted from a
/// real two-capture correlated EXISTS fixture (`f.id = r.id AND f.value =
/// r.value` over `records2`): the sealed two-edge capture vector of the
/// RelationExists fact is replaced through the crate's test-only reshape seam
/// in both carriers (the expression fact and the outer Filter predicate), so
/// every other identity, fact, nested relation, and the receipt stay sealed.
///
/// `Dropped` and `Swapped` assemble: the document finalizer retains them
/// (it checks reference membership, uniqueness, and registry identity, not
/// the authored capture count or order). `Duplicate` and `Foreign` cannot
/// become documents: assembly rejects them through the real finalizer, and
/// the `Err` text contains exactly `duplicate correlation reference in one
/// nested relation` respectively `correlation reference is foreign to this
/// document's expressions`.
///
/// # Errors
///
/// Returns the finalizer rejection itself for `Duplicate` and `Foreign`;
/// otherwise a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_capture_mutation_document_v2(
    mutation: ForgedCaptureEdgeMutationV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    let (checked, real, item_id) = two_capture_correlated_exists_parts()?;
    let exists_entry = real
        .expressions()
        .find(|(_key, expression)| {
            matches!(
                expression.shape(),
                TypedExprShapeV2::RelationExists { correlations, .. } if correlations.len() == 2
            )
        })
        .ok_or_else(|| "two-capture fixture must carry a two-edge EXISTS fact".to_owned())?;
    let (exists_key, exists_expression) = (*exists_entry.0, exists_entry.1);
    let (first, second) =
        if let TypedExprShapeV2::RelationExists { correlations, .. } = exists_expression.shape() {
            (
                correlations
                    .first()
                    .cloned()
                    .ok_or("first capture edge expected")?,
                correlations
                    .get(1)
                    .cloned()
                    .ok_or("second capture edge expected")?,
            )
        } else {
            return Err("two-edge RelationExists fact expected".to_owned());
        };
    let mutated = match mutation {
        ForgedCaptureEdgeMutationV2::Dropped => vec![first],
        ForgedCaptureEdgeMutationV2::Duplicate => vec![first.clone(), first],
        ForgedCaptureEdgeMutationV2::Swapped => vec![second, first],
        ForgedCaptureEdgeMutationV2::Foreign => vec![
            first,
            TypedCorrelationEdgeV2::new(
                second.binding().clone(),
                SourceQualifiedNodeIdV2 {
                    source_id: REFERENCE_SOURCE_V2,
                    node_id: HirNodeIdV1::from_raw(u32::MAX),
                },
                second.location().clone(),
            ),
        ],
    };
    let rebuilt = exists_expression
        .clone()
        .with_relation_exists_correlations(mutated)
        .map_err(|error| format!("{error:?}"))?;
    let relation = relation_item(&real, item_id)?;
    let root = relation.root();
    let input = root
        .inputs()
        .next()
        .ok_or("outer filter has no scan input")?
        .clone();
    let predicate = TypedBoolExprV2::try_new(rebuilt.clone(), rebuilt.location().location())
        .map_err(|error| format!("{error:?}"))?;
    let forged_root = TypedRelationExprV2::filter(
        root.key(),
        root.location().clone(),
        input,
        predicate,
        root.output().clone(),
    )
    .map_err(|error| error.to_string())?;
    let (mut items, mut expressions, nested) = document_parts(&real);
    expressions.insert(exists_key, rebuilt);
    let forged = TypedRelationV2::try_new(
        item_id,
        relation.location().clone(),
        forged_root.key(),
        forged_root.clone(),
        forged_root.output().clone(),
    )
    .map_err(str::to_owned)?;
    items.retain(|item| !matches!(item, SemanticItemV2::Relation(r) if r.item_id() == item_id));
    items.push(SemanticItemV2::Relation(forged));
    let document = reassemble_document(
        &checked,
        &reference_authority_v2()?,
        items,
        expressions,
        nested,
        real.budget_receipt(),
    )?;
    Ok(ReferenceRelationDocumentV2 { document, item_id })
}

/// Forged document for the downstream-budget rail: a relation chain of the
/// requested `depth` (resolved under the standard budget) is reassembled with
/// a receipt whose `max_semantic_steps` is downgraded to
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] while every other dimension stays
/// standard, so the relation walk's own step charging is the only limit. A
/// chain of depth `d` lowers `d + 1` relation stages: depth
/// `SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1` is the exact fit and one more filter
/// is over.
///
/// # Errors
///
/// Returns a fixture-construction error when `depth` exceeds
/// [`MAX_REFERENCE_DEPTH_V2`] or the reassembled document no longer satisfies
/// the assembly contract.
pub fn forged_small_receipt_relation_chain_document_v2(
    depth: u32,
) -> Result<ReferenceRelationDocumentV2, String> {
    check_depth(depth)?;
    let reference = deep_relation_chain_document_v2(depth, ReferenceBudgetV2::Standard)?;
    let checked = deep_chain_checked_document_v2(depth)?;
    reassemble_with_receipt(&checked, &reference, small_steps_receipt()?)
}

/// Forged document for the expression-walk budget rail: a Bool tree of the
/// requested `depth` (resolved under the standard budget) is reassembled with
/// a receipt whose `max_expression_nodes` is downgraded to
/// [`SMALL_BUDGET_MAX_EXPRESSION_NODES`] while every other dimension stays
/// standard. Each leaf is a BoolValue wrapping one Bool literal; a
/// decomposition walk plus descendant search charges `4 * depth + 3` nodes, so
/// depth 1 charges exactly seven of the eight and depth 2 charges eleven.
///
/// # Errors
///
/// Returns a fixture-construction error when `depth` exceeds
/// [`MAX_REFERENCE_DEPTH_V2`] or the reassembled document no longer satisfies
/// the assembly contract.
pub fn forged_small_receipt_bool_tree_document_v2(
    depth: u32,
) -> Result<ReferenceRelationDocumentV2, String> {
    check_depth(depth)?;
    let reference = deep_bool_tree_document_v2(depth, ReferenceBudgetV2::Standard)?;
    let checked = deep_bool_tree_checked_document_v2(depth)?;
    reassemble_with_receipt(&checked, &reference, small_nodes_receipt()?)
}

// ---- fixture HIR authoring -------------------------------------------------

#[expect(
    clippy::too_many_lines,
    reason = "fixture authoring variants await decomposition"
)]
fn reference_relation_hir(
    fixture: ReferenceRelationFixtureV2,
    builder: &HirBuilderV1,
) -> Result<(RelationExprHirV1, usize), String> {
    match fixture {
        ReferenceRelationFixtureV2::Scan => Ok((
            builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12)),
            12,
        )),
        ReferenceRelationFixtureV2::Filter => Ok((
            builder.relation_filter(
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12)),
                builder.bool_value(
                    builder.bool_lit(true, SpanV1::new(20, 24)),
                    SpanV1::new(19, 24),
                ),
                SpanV1::new(5, 24),
            ),
            28,
        )),
        ReferenceRelationFixtureV2::CrossJoin => {
            let left =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let right =
                builder.relation_source("other", SpanV1::new(19, 24), None, SpanV1::new(19, 24));
            Ok((builder.cross_join(left, right, SpanV1::new(5, 24)), 24))
        }
        ReferenceRelationFixtureV2::Exists | ReferenceRelationFixtureV2::NotExists => {
            let outer =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let inner =
                builder.relation_source("other", SpanV1::new(25, 30), None, SpanV1::new(25, 30));
            let polarity = if fixture == ReferenceRelationFixtureV2::Exists {
                RelationExistsPolarityV1::Exists
            } else {
                RelationExistsPolarityV1::NotExists
            };
            let predicate = builder.relation_exists(polarity, inner, SpanV1::new(18, 31));
            Ok((
                builder.relation_filter(outer, predicate, SpanV1::new(5, 31)),
                32,
            ))
        }
        ReferenceRelationFixtureV2::ProjectMultipleExpressions => {
            let scan = aliased_scan(builder, "records2", "r", 5, 13, 14);
            let id_path = field_path(builder, "r", 20, "id", 22)?;
            let label = builder.string("fixed", SpanV1::new(30, 36));
            let project = builder
                .try_project(
                    scan,
                    vec![
                        ProjectionItemV1 {
                            value: id_path,
                            alias: Some(alias("num", 40, 43)),
                        },
                        ProjectionItemV1 {
                            value: label,
                            alias: Some(alias("label", 46, 51)),
                        },
                    ],
                    SpanV1::new(5, 51),
                )
                .map_err(|error| error.to_owned())?;
            Ok((project, 56))
        }
        ReferenceRelationFixtureV2::InnerJoinOn => {
            let left = aliased_scan(builder, "records", "r", 5, 12, 13);
            let right = aliased_scan(builder, "other", "o", 20, 25, 26);
            let left_key = field_path(builder, "r", 31, "id", 33)?;
            let right_key = field_path(builder, "o", 38, "other_id", 40)?;
            let on = builder.compare("=", left_key, right_key, SpanV1::new(31, 48));
            Ok((builder.inner_join(left, right, on, SpanV1::new(5, 48)), 52))
        }
        ReferenceRelationFixtureV2::SelfJoinInner => {
            let (join, len) = self_join_hir(builder)?;
            Ok((join, len))
        }
        ReferenceRelationFixtureV2::AsymmetricJoinPreorder => {
            let (join, len) = asymmetric_join_hir(builder)?;
            Ok((join, len))
        }
        ReferenceRelationFixtureV2::GroupedAggregateSum => {
            let (project, len) = grouped_aggregate_hir(builder)?;
            Ok((project, len))
        }
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys => {
            let (project, len) = grouped_two_key_aggregate_hir(builder)?;
            Ok((project, len))
        }
        ReferenceRelationFixtureV2::ScalarCountStar => {
            let (project, len) = scalar_aggregate_hir(builder)?;
            Ok((project, len))
        }
        ReferenceRelationFixtureV2::SortAsc
        | ReferenceRelationFixtureV2::SortDesc
        | ReferenceRelationFixtureV2::SortNullsLastNullableField
        | ReferenceRelationFixtureV2::SortMultiKeyMixedDirections => {
            let scan = aliased_scan(builder, "records3", "r", 5, 13, 14);
            let (keys, end) = if fixture == ReferenceRelationFixtureV2::SortAsc {
                (
                    vec![(
                        field_path(builder, "r", 20, "id", 22)?,
                        SortDirectionHirV1::Asc,
                    )],
                    24,
                )
            } else if fixture == ReferenceRelationFixtureV2::SortDesc {
                (
                    vec![(
                        field_path(builder, "r", 20, "id", 22)?,
                        SortDirectionHirV1::Desc,
                    )],
                    24,
                )
            } else if fixture == ReferenceRelationFixtureV2::SortNullsLastNullableField {
                (
                    vec![(
                        field_path(builder, "r", 20, "label", 22)?,
                        SortDirectionHirV1::Desc,
                    )],
                    27,
                )
            } else if fixture == ReferenceRelationFixtureV2::SortMultiKeyMixedDirections {
                (
                    vec![
                        (
                            field_path(builder, "r", 20, "value", 22)?,
                            SortDirectionHirV1::Asc,
                        ),
                        (
                            field_path(builder, "r", 30, "id", 32)?,
                            SortDirectionHirV1::Desc,
                        ),
                    ],
                    34,
                )
            } else {
                return Err("unreachable sort family arm".to_owned());
            };
            let order = builder
                .order_by(
                    scan,
                    keys.into_iter()
                        .map(|(value, direction)| OrderKeyHirV1::nulls_last(value, direction))
                        .collect(),
                    SpanV1::new(5, end),
                )
                .map_err(|error| error.to_owned())?;
            Ok((order, end + 4))
        }
        ReferenceRelationFixtureV2::SortAliasKeyed
        | ReferenceRelationFixtureV2::SortByProjectedExpression => {
            let scan = aliased_scan(builder, "records", "r", 5, 12, 13);
            let id_path = field_path(builder, "r", 20, "id", 22)?;
            let project = builder
                .try_project(
                    scan,
                    vec![ProjectionItemV1 {
                        value: id_path,
                        alias: Some(alias("score", 28, 33)),
                    }],
                    SpanV1::new(5, 33),
                )
                .map_err(|error| error.to_owned())?;
            let key = if fixture == ReferenceRelationFixtureV2::SortAliasKeyed {
                builder.symbol("score", SpanV1::new(40, 45))
            } else {
                builder.paren_value(
                    builder.symbol("score", SpanV1::new(41, 46)),
                    SpanV1::new(40, 47),
                )
            };
            let order = builder
                .order_by(
                    project,
                    vec![OrderKeyHirV1::nulls_last(key, SortDirectionHirV1::Desc)],
                    SpanV1::new(5, 47),
                )
                .map_err(|error| error.to_owned())?;
            Ok((order, 52))
        }
        ReferenceRelationFixtureV2::CorrelatedExists
        | ReferenceRelationFixtureV2::CorrelatedNotExists => {
            let (filter, len) = correlated_exists_hir(builder, fixture)?;
            Ok((filter, len))
        }
        ReferenceRelationFixtureV2::ConjunctiveExistsPair => {
            let outer =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let first_inner =
                builder.relation_source("other", SpanV1::new(23, 28), None, SpanV1::new(23, 28));
            let first = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                first_inner,
                SpanV1::new(16, 29),
            );
            let second_inner =
                builder.relation_source("records2", SpanV1::new(44, 52), None, SpanV1::new(44, 52));
            let second = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                second_inner,
                SpanV1::new(36, 52),
            );
            let both = builder.and(first, second, SpanV1::new(16, 52));
            Ok((builder.relation_filter(outer, both, SpanV1::new(5, 52)), 56))
        }
        ReferenceRelationFixtureV2::DisjunctiveExistsPair => {
            let outer =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let first_inner =
                builder.relation_source("other", SpanV1::new(23, 28), None, SpanV1::new(23, 28));
            let first = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                first_inner,
                SpanV1::new(16, 29),
            );
            let second_inner =
                builder.relation_source("records2", SpanV1::new(44, 52), None, SpanV1::new(44, 52));
            let second = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                second_inner,
                SpanV1::new(36, 52),
            );
            let either = builder.or(first, second, SpanV1::new(16, 52));
            Ok((
                builder.relation_filter(outer, either, SpanV1::new(5, 52)),
                56,
            ))
        }
        ReferenceRelationFixtureV2::NegatedExists => {
            let outer =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let inner =
                builder.relation_source("other", SpanV1::new(25, 30), None, SpanV1::new(25, 30));
            let exists = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                inner,
                SpanV1::new(18, 31),
            );
            let negated = builder.not(exists, SpanV1::new(17, 32));
            Ok((
                builder.relation_filter(outer, negated, SpanV1::new(5, 32)),
                36,
            ))
        }
        ReferenceRelationFixtureV2::ExistsWithBoolConjunct => {
            let outer =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let ordinary = builder.bool_value(
                builder.bool_lit(true, SpanV1::new(17, 21)),
                SpanV1::new(16, 21),
            );
            let inner =
                builder.relation_source("other", SpanV1::new(31, 36), None, SpanV1::new(31, 36));
            let exists = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                inner,
                SpanV1::new(24, 37),
            );
            let both = builder.and(ordinary, exists, SpanV1::new(16, 37));
            Ok((builder.relation_filter(outer, both, SpanV1::new(5, 37)), 40))
        }
        ReferenceRelationFixtureV2::NestedExistsTwoLevels => {
            let (filter, len) = nested_exists_hir(builder)?;
            Ok((filter, len))
        }
        ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight => {
            let outer =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let left_inner =
                builder.relation_source("other", SpanV1::new(23, 28), None, SpanV1::new(23, 28));
            let right_inner =
                builder.relation_source("records2", SpanV1::new(33, 41), None, SpanV1::new(33, 41));
            let nested = builder.cross_join(left_inner, right_inner, SpanV1::new(23, 41));
            let exists = builder.relation_exists(
                RelationExistsPolarityV1::Exists,
                nested,
                SpanV1::new(16, 42),
            );
            Ok((
                builder.relation_filter(outer, exists, SpanV1::new(5, 42)),
                48,
            ))
        }
    }
}

fn alias(name: &str, start: usize, end: usize) -> AliasV1 {
    AliasV1 {
        name: name.to_owned(),
        span: SpanV1::new(start, end),
    }
}

fn aliased_scan(
    builder: &HirBuilderV1,
    name: &str,
    alias_name: &str,
    name_start: usize,
    name_end: usize,
    alias_start: usize,
) -> RelationExprHirV1 {
    builder.relation_source(
        name,
        SpanV1::new(name_start, name_end),
        Some(alias(alias_name, alias_start, alias_start + 1)),
        SpanV1::new(name_start, alias_start + 1),
    )
}

fn field_path(
    builder: &HirBuilderV1,
    base: &str,
    base_start: usize,
    field: &str,
    field_start: usize,
) -> Result<ValueExprHirV1, String> {
    let field_end = field_start + field.len();
    builder
        .try_path(
            builder.symbol(base, SpanV1::new(base_start, base_start + 1)),
            vec![PathSegmentV1 {
                name: field.to_owned(),
                span: SpanV1::new(field_start, field_end),
            }],
            SpanV1::new(base_start, field_end),
        )
        .map_err(|error| error.to_owned())
}

fn bool_value_leaf(builder: &HirBuilderV1, start: usize) -> quanta_gqlang_v2_hir::BoolExprHirV1 {
    builder.bool_value(
        builder.bool_lit(true, SpanV1::new(start + 1, start + 5)),
        SpanV1::new(start, start + 5),
    )
}

fn aggregate_call(
    builder: &HirBuilderV1,
    callee: &str,
    callee_start: usize,
    callee_end: usize,
    argument: ValueExprHirV1,
    call_end: usize,
) -> Result<ValueExprHirV1, String> {
    let args = CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)])
        .map_err(|error| error.to_owned())?;
    Ok(builder.call(
        callee,
        SpanV1::new(callee_start, callee_end),
        args,
        SpanV1::new(callee_start, call_end),
    ))
}

fn self_join_hir(builder: &HirBuilderV1) -> Result<(RelationExprHirV1, usize), String> {
    let left_scan = aliased_scan(builder, "records", "l", 5, 12, 13);
    let left_value = field_path(builder, "l", 20, "id", 22)?;
    let left = builder
        .try_project(
            left_scan,
            vec![ProjectionItemV1 {
                value: left_value,
                alias: Some(alias("left_id", 28, 35)),
            }],
            SpanV1::new(5, 35),
        )
        .map_err(|error| error.to_owned())?;
    let right_scan = aliased_scan(builder, "records", "r", 40, 47, 48);
    let right_value = field_path(builder, "r", 55, "id", 57)?;
    let right = builder
        .try_project(
            right_scan,
            vec![ProjectionItemV1 {
                value: right_value,
                alias: Some(alias("right_id", 63, 71)),
            }],
            SpanV1::new(40, 71),
        )
        .map_err(|error| error.to_owned())?;
    let on = builder.compare(
        "=",
        builder.symbol("left_id", SpanV1::new(75, 82)),
        builder.symbol("right_id", SpanV1::new(85, 93)),
        SpanV1::new(75, 93),
    );
    Ok((builder.inner_join(left, right, on, SpanV1::new(5, 93)), 96))
}

/// Cross join whose arms are structurally distinguishable (EXPLAIN-01):
/// left `Project -> Filter -> Scan` over `records`/`l`, right `Sort -> Scan`
/// over `records`/`s`. Preorder families are
/// `Join, Project, Filter, Scan, Sort, Scan`; swapping child traversal yields
/// `Join, Sort, Scan, Project, Filter, Scan`.
fn asymmetric_join_hir(builder: &HirBuilderV1) -> Result<(RelationExprHirV1, usize), String> {
    let left_scan = aliased_scan(builder, "records", "l", 5, 12, 13);
    let left_filtered = builder.relation_filter(
        left_scan,
        builder.bool_value(
            builder.bool_lit(true, SpanV1::new(20, 24)),
            SpanV1::new(19, 24),
        ),
        SpanV1::new(5, 24),
    );
    let left_value = field_path(builder, "l", 30, "id", 32)?;
    let left = builder
        .try_project(
            left_filtered,
            vec![ProjectionItemV1 {
                value: left_value,
                alias: Some(alias("left_id", 36, 43)),
            }],
            SpanV1::new(5, 43),
        )
        .map_err(|error| error.to_owned())?;
    let right_scan = aliased_scan(builder, "records", "s", 50, 57, 58);
    let right_key = field_path(builder, "s", 65, "id", 67)?;
    let right = builder
        .order_by(
            right_scan,
            vec![OrderKeyHirV1::nulls_last(
                right_key,
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(50, 72),
        )
        .map_err(|error| error.to_owned())?;
    Ok((builder.cross_join(left, right, SpanV1::new(5, 72)), 76))
}

fn grouped_aggregate_hir(builder: &HirBuilderV1) -> Result<(RelationExprHirV1, usize), String> {
    let scan = aliased_scan(builder, "records", "r", 5, 12, 13);
    let group_key = field_path(builder, "r", 22, "id", 24)?;
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .map_err(|error| error.to_owned())?;
    let argument = field_path(builder, "r", 42, "id", 44)?;
    let sum = aggregate_call(builder, "sum", 38, 41, argument, 50)?;
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(alias("total", 55, 60)),
            }],
            SpanV1::new(5, 60),
        )
        .map_err(|error| error.to_owned())?;
    Ok((project, 64))
}

fn grouped_two_key_aggregate_hir(
    builder: &HirBuilderV1,
) -> Result<(RelationExprHirV1, usize), String> {
    let scan = aliased_scan(builder, "records3", "r", 5, 13, 14);
    let first_key = field_path(builder, "r", 22, "id", 24)?;
    let second_key = field_path(builder, "r", 26, "value", 28)?;
    let group = builder
        .group_by(scan, vec![first_key, second_key], SpanV1::new(5, 34))
        .map_err(|error| error.to_owned())?;
    let argument = field_path(builder, "r", 46, "id", 48)?;
    let sum = aggregate_call(builder, "sum", 42, 45, argument, 52)?;
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(alias("total", 57, 62)),
            }],
            SpanV1::new(5, 62),
        )
        .map_err(|error| error.to_owned())?;
    Ok((project, 66))
}

fn scalar_aggregate_hir(builder: &HirBuilderV1) -> Result<(RelationExprHirV1, usize), String> {
    let scan = aliased_scan(builder, "records", "r", 5, 12, 13);
    let star = CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
        span: SpanV1::new(30, 31),
    }])
    .map_err(|error| error.to_owned())?;
    let count = builder.call("count", SpanV1::new(24, 29), star, SpanV1::new(24, 31));
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: count,
                alias: Some(alias("total", 36, 41)),
            }],
            SpanV1::new(16, 41),
        )
        .map_err(|error| error.to_owned())?;
    Ok((project, 64))
}

fn correlated_exists_hir(
    builder: &HirBuilderV1,
    fixture: ReferenceRelationFixtureV2,
) -> Result<(RelationExprHirV1, usize), String> {
    let outer = aliased_scan(builder, "records2", "r", 5, 13, 14);
    let inner = aliased_scan(builder, "records2", "f", 25, 33, 34);
    let inner_key = field_path(builder, "f", 38, "id", 40)?;
    let captured = field_path(builder, "r", 45, "id", 47)?;
    let compare = builder.compare("=", inner_key, captured, SpanV1::new(38, 49));
    let inner_filter = builder.relation_filter(inner, compare, SpanV1::new(25, 49));
    let polarity = if fixture == ReferenceRelationFixtureV2::CorrelatedExists {
        RelationExistsPolarityV1::Exists
    } else {
        RelationExistsPolarityV1::NotExists
    };
    let exists = builder.relation_exists(polarity, inner_filter, SpanV1::new(20, 50));
    Ok((
        builder.relation_filter(outer, exists, SpanV1::new(5, 50)),
        64,
    ))
}

fn two_capture_correlated_exists_hir(
    builder: &HirBuilderV1,
) -> Result<(RelationExprHirV1, usize), String> {
    let outer = aliased_scan(builder, "records2", "r", 5, 13, 14);
    let inner = aliased_scan(builder, "records2", "f", 25, 33, 34);
    let id_compare = builder.compare(
        "=",
        field_path(builder, "f", 38, "id", 40)?,
        field_path(builder, "r", 45, "id", 47)?,
        SpanV1::new(38, 49),
    );
    let value_compare = builder.compare(
        "=",
        field_path(builder, "f", 54, "value", 56)?,
        field_path(builder, "r", 58, "value", 60)?,
        SpanV1::new(54, 65),
    );
    let predicate = builder.and(id_compare, value_compare, SpanV1::new(38, 65));
    let inner_filter = builder.relation_filter(inner, predicate, SpanV1::new(25, 65));
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 66),
    );
    Ok((
        builder.relation_filter(outer, exists, SpanV1::new(5, 66)),
        80,
    ))
}

fn nested_exists_hir(builder: &HirBuilderV1) -> Result<(RelationExprHirV1, usize), String> {
    let outer = aliased_scan(builder, "records2", "r", 5, 13, 14);
    let middle = aliased_scan(builder, "records2", "f", 25, 33, 34);
    let deepest = aliased_scan(builder, "records2", "g", 65, 73, 74);
    let direct = builder.compare(
        "=",
        field_path(builder, "f", 38, "id", 40)?,
        field_path(builder, "r", 45, "id", 47)?,
        SpanV1::new(38, 49),
    );
    let deep = builder.compare(
        "=",
        field_path(builder, "g", 78, "value", 80)?,
        field_path(builder, "r", 84, "value", 86)?,
        SpanV1::new(78, 91),
    );
    let deepest_filter = builder.relation_filter(deepest, deep, SpanV1::new(65, 91));
    let nested_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        deepest_filter,
        SpanV1::new(60, 92),
    );
    let middle_filter = builder.relation_filter(
        middle,
        builder.and(direct, nested_exists, SpanV1::new(38, 92)),
        SpanV1::new(25, 92),
    );
    let outer_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        middle_filter,
        SpanV1::new(20, 93),
    );
    Ok((
        builder.relation_filter(outer, outer_exists, SpanV1::new(5, 93)),
        104,
    ))
}

// ---- forged-document plumbing ----------------------------------------------

fn scalar_aggregate_parts() -> Result<
    (
        CheckedSyntaxDocumentV2,
        SemanticDocumentV2,
        SemanticItemIdV2,
    ),
    String,
> {
    let builder = HirBuilderV1::new();
    let (relation, source_len) = scalar_aggregate_hir(&builder)?;
    resolve_fixture_parts(&builder, relation, source_len)
}

fn grouped_aggregate_parts() -> Result<
    (
        CheckedSyntaxDocumentV2,
        SemanticDocumentV2,
        SemanticItemIdV2,
    ),
    String,
> {
    let builder = HirBuilderV1::new();
    let (relation, source_len) = grouped_aggregate_hir(&builder)?;
    resolve_fixture_parts(&builder, relation, source_len)
}

fn correlated_exists_parts() -> Result<
    (
        CheckedSyntaxDocumentV2,
        SemanticDocumentV2,
        SemanticItemIdV2,
    ),
    String,
> {
    let builder = HirBuilderV1::new();
    let (relation, source_len) =
        correlated_exists_hir(&builder, ReferenceRelationFixtureV2::CorrelatedExists)?;
    resolve_fixture_parts(&builder, relation, source_len)
}

fn two_capture_correlated_exists_parts() -> Result<
    (
        CheckedSyntaxDocumentV2,
        SemanticDocumentV2,
        SemanticItemIdV2,
    ),
    String,
> {
    let builder = HirBuilderV1::new();
    let (relation, source_len) = two_capture_correlated_exists_hir(&builder)?;
    resolve_fixture_parts(&builder, relation, source_len)
}

fn resolve_fixture_parts(
    builder: &HirBuilderV1,
    relation: RelationExprHirV1,
    source_len: usize,
) -> Result<
    (
        CheckedSyntaxDocumentV2,
        SemanticDocumentV2,
        SemanticItemIdV2,
    ),
    String,
> {
    let query = builder.query(relation, None, SpanV1::new(0, source_len - 1));
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Query(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document = resolve_checked_document_v2(checked.clone(), &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok((checked, document, item_id))
}

fn exists_root(document: &SemanticDocumentV2) -> Result<SourceQualifiedNodeIdV2, String> {
    document
        .expressions()
        .find_map(|(_key, expression)| {
            if let TypedExprShapeV2::RelationExists { relation_root, .. } = expression.shape() {
                Some(*relation_root)
            } else {
                None
            }
        })
        .ok_or_else(|| "relational EXISTS root expected".to_owned())
}

fn relation_item(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
) -> Result<&TypedRelationV2, String> {
    match document.item(item_id) {
        Some(SemanticItemV2::Relation(relation)) => Ok(relation),
        _ => Err("reference document lacks its relation item".to_owned()),
    }
}

fn document_parts(
    document: &SemanticDocumentV2,
) -> (
    Vec<SemanticItemV2>,
    BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
) {
    (
        document.items().map(|(_id, item)| item.clone()).collect(),
        document
            .expressions()
            .map(|(key, value)| (*key, value.clone()))
            .collect(),
        document
            .nested_relations()
            .map(|(key, value)| (*key, value.clone()))
            .collect(),
    )
}

fn reassemble_document(
    checked: &CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    items: Vec<SemanticItemV2>,
    expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    receipt: DownstreamBudgetReceiptV2,
) -> Result<SemanticDocumentV2, String> {
    let extended = authority
        .try_extend_document(checked.source_id(), checked.source_len())
        .map_err(|error| error.to_string())?;
    let declared = DeclaredDocumentV2::try_new(checked.clone(), authority.budget(), &extended)
        .map_err(|error| format!("{error:?}"))?;
    let type_fact_count = expressions.len();
    let mut validator = DocumentCatalogValidatorV2::try_new(extended, authority.modules())
        .map_err(|error| error.to_string())?;
    validator
        .predeclare_assets(Vec::new())
        .map_err(|error| error.to_string())?;
    let topology = validator
        .validate_view_topology(Vec::new())
        .map_err(|error| error.to_string())?;
    let catalog = validator
        .validate_predeclared(
            topology,
            DocumentCatalogFinalInputV2 {
                profile_admissions: Vec::new(),
                activations: Vec::new(),
                require_certified_directives: Vec::new(),
                family_references: Vec::new(),
            },
        )
        .map_err(|error| error.to_string())?;
    SemanticDocumentV2::try_assemble(
        &declared,
        items,
        expressions,
        nested_relations,
        type_fact_count,
        catalog,
        authority.graph_references(),
        receipt,
    )
    .map_err(|error| format!("{error:?}"))
}

/// Re-seal one real fixture document with a forged relation root, keeping
/// every expression fact, nested relation, item identity, and the original
/// receipt.
fn reassemble_with_forged_relation(
    checked: &CheckedSyntaxDocumentV2,
    real: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    forged_root: TypedRelationExprV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    let real_relation = relation_item(real, item_id)?;
    let forged = TypedRelationV2::try_new(
        item_id,
        real_relation.location().clone(),
        forged_root.key(),
        forged_root.clone(),
        forged_root.output().clone(),
    )
    .map_err(str::to_owned)?;
    let (mut items, expressions, nested) = document_parts(real);
    items.retain(|item| !matches!(item, SemanticItemV2::Relation(r) if r.item_id() == item_id));
    items.push(SemanticItemV2::Relation(forged));
    let document = reassemble_document(
        checked,
        &reference_authority_v2()?,
        items,
        expressions,
        nested,
        real.budget_receipt(),
    )?;
    Ok(ReferenceRelationDocumentV2 { document, item_id })
}

fn reassemble_with_receipt(
    checked: &CheckedSyntaxDocumentV2,
    reference: &ReferenceRelationDocumentV2,
    receipt: DownstreamBudgetReceiptV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    let (items, expressions, nested) = document_parts(reference.document());
    let document = reassemble_document(
        checked,
        &reference_authority_v2()?,
        items,
        expressions,
        nested,
        receipt,
    )?;
    Ok(ReferenceRelationDocumentV2 {
        document,
        item_id: reference.item_id(),
    })
}

fn small_steps_receipt() -> Result<DownstreamBudgetReceiptV2, String> {
    let standard = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let small = FrontendBudgetV2::try_new(
        standard.max_source_bytes(),
        standard.max_frame_bytes(),
        standard.max_tokens(),
        standard.max_document_items(),
        standard.max_expression_depth(),
        standard.max_relation_depth(),
        standard.max_expression_nodes(),
        standard.max_collection_width(),
        standard.max_projection_width(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS,
        standard.max_module_graph_nodes(),
        standard.max_module_graph_edges(),
        standard.max_module_steps(),
        standard.max_authority_bytes(),
        standard.max_authority_string_bytes(),
        standard.max_rule_graph_nodes(),
        standard.max_rule_graph_edges(),
        standard.max_rule_steps(),
        standard.max_witness_depth(),
        standard.max_witnesses(),
        standard.max_diagnostics(),
        standard.max_response_bytes(),
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(small.downstream_receipt())
}

fn small_nodes_receipt() -> Result<DownstreamBudgetReceiptV2, String> {
    let standard = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let small = FrontendBudgetV2::try_new(
        standard.max_source_bytes(),
        standard.max_frame_bytes(),
        standard.max_tokens(),
        standard.max_document_items(),
        standard.max_expression_depth(),
        standard.max_relation_depth(),
        SMALL_BUDGET_MAX_EXPRESSION_NODES,
        standard.max_collection_width(),
        standard.max_projection_width(),
        standard.max_semantic_steps(),
        standard.max_module_graph_nodes(),
        standard.max_module_graph_edges(),
        standard.max_module_steps(),
        standard.max_authority_bytes(),
        standard.max_authority_string_bytes(),
        standard.max_rule_graph_nodes(),
        standard.max_rule_graph_edges(),
        standard.max_rule_steps(),
        standard.max_witness_depth(),
        standard.max_witnesses(),
        standard.max_diagnostics(),
        standard.max_response_bytes(),
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(small.downstream_receipt())
}

fn deep_chain_checked_document_v2(depth: u32) -> Result<CheckedSyntaxDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let mut relation =
        builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    for index in 0..usize::try_from(depth).map_err(|_error| "depth overflow".to_owned())? {
        let literal_start = 40 + 4 * index;
        let literal = builder.bool_literal(true, SpanV1::new(literal_start, literal_start + 4));
        relation = builder.relation_filter(relation, literal, SpanV1::new(5, literal_start + 4));
    }
    let source_len =
        48 + 4 * usize::try_from(depth).map_err(|_error| "depth overflow".to_owned())?;
    checked_query_document(
        builder.query(relation, None, SpanV1::new(0, source_len - 1)),
        source_len,
    )
}

fn deep_bool_tree_checked_document_v2(depth: u32) -> Result<CheckedSyntaxDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let (filter, source_len) = deep_bool_tree_parts(&builder, depth)?;
    checked_query_document(
        builder.query(filter, None, SpanV1::new(0, source_len - 1)),
        source_len,
    )
}

fn checked_query_document(
    query: QueryHirV1,
    source_len: usize,
) -> Result<CheckedSyntaxDocumentV2, String> {
    CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Query(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))
}

fn mint_capture_edges(
    mutation: Option<ForgedCaptureEdgeMutationV2>,
) -> Result<Vec<TypedCorrelationEdgeV2>, String> {
    let host = capture_edge_registry()?;
    let first_location = host
        .try_check_location(LocatedSpanV2 {
            source_id: REFERENCE_SOURCE_V2,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| error.to_string())?;
    let second_location = host
        .try_check_location(LocatedSpanV2 {
            source_id: REFERENCE_SOURCE_V2,
            span: SpanV1::new(2, 3),
        })
        .map_err(|error| error.to_string())?;
    let binding = ResolvedValueBindingV2::try_new(
        REFERENCE_SOURCE_V2,
        HirNodeIdV1::from_raw(7),
        0,
        ResolvedBindingKindV2::Alias,
        "r",
    )
    .map_err(str::to_owned)?;
    let first = TypedCorrelationEdgeV2::new(
        binding.clone(),
        SourceQualifiedNodeIdV2 {
            source_id: REFERENCE_SOURCE_V2,
            node_id: HirNodeIdV1::from_raw(40),
        },
        first_location,
    );
    let second_reference = match mutation {
        Some(ForgedCaptureEdgeMutationV2::Foreign) => HirNodeIdV1::from_raw(9_999),
        _ => HirNodeIdV1::from_raw(50),
    };
    let second = TypedCorrelationEdgeV2::new(
        binding,
        SourceQualifiedNodeIdV2 {
            source_id: REFERENCE_SOURCE_V2,
            node_id: second_reference,
        },
        second_location,
    );
    Ok(match mutation {
        None | Some(ForgedCaptureEdgeMutationV2::Foreign) => vec![first, second],
        Some(ForgedCaptureEdgeMutationV2::Dropped) => vec![first],
        Some(ForgedCaptureEdgeMutationV2::Duplicate) => vec![first.clone(), first],
        Some(ForgedCaptureEdgeMutationV2::Swapped) => vec![second, first],
    })
}

fn capture_edge_registry() -> Result<SourceRegistryViewV2, String> {
    SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 256,
        },
        SourceRegistrationV2 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: 64,
        },
    ])
    .map_err(|error| error.to_string())
}

fn forged_join_seal_parts() -> Result<
    (
        TypedRelationExprV2,
        TypedRelationExprV2,
        CheckedLocatedSpanV2,
        SourceQualifiedNodeIdV2,
        RelationOutputContractV2,
    ),
    &'static str,
> {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(8),
        source_len: 64,
    }])
    .map_err(|_error| "forged seal registry must be constructible")?;
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(8),
            span: SpanV1::new(0, 1),
        })
        .map_err(|_error| "forged seal location must be checkable")?;
    let key = |node: u32| SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(8),
        node_id: HirNodeIdV1::from_raw(node),
    };
    let binding = ResolvedValueBindingV2::try_new(
        SourceIdV2::from_raw(8),
        HirNodeIdV1::from_raw(2),
        0,
        ResolvedBindingKindV2::Composition,
        "q",
    )
    .map_err(|_error| "composition binding must be constructible")?;
    let scan_source = ResolvedScanSourceV2::Composition(
        ResolvedCompositionSourceIdV2::try_from_binding(binding)
            .map_err(|_error| "composition source must be sealable")?,
    );
    let output = RelationOutputContractV2::try_new(
        RecordTypeV1::try_new(Vec::new()).map_err(|_error| "empty row must be constructible")?,
        OutputProvenanceV2::new([scan_source.clone()]),
        Vec::new(),
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(|_error| "scan output must be sealable")?;
    let scan = TypedRelationExprV2::scan(key(3), location.clone(), scan_source, output)
        .map_err(|_error| "forged seal scan must be sealable")?;
    Ok((scan.clone(), scan, location, key(4), empty_bag_output()?))
}

fn empty_bag_output() -> Result<RelationOutputContractV2, &'static str> {
    RelationOutputContractV2::try_new(
        RecordTypeV1::try_new(Vec::new()).map_err(|_error| "empty row must be constructible")?,
        OutputProvenanceV2::default(),
        Vec::new(),
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
}

fn reference_bool_predicate() -> Result<TypedBoolExprV2, &'static str> {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter)
        .map_err(|_error| "filter fixture must resolve")?;
    let relation = relation_item(reference.document(), reference.item_id())
        .map_err(|_error| "filter fixture must carry its relation item")?;
    relation
        .root()
        .predicate()
        .cloned()
        .ok_or("filter fixture has no predicate")
}

fn check_depth(depth: u32) -> Result<(), String> {
    if depth > MAX_REFERENCE_DEPTH_V2 {
        return Err(format!(
            "reference depth {depth} exceeds the supported maximum {MAX_REFERENCE_DEPTH_V2}"
        ));
    }
    Ok(())
}

fn resolve_reference(
    builder: &HirBuilderV1,
    relation: RelationExprHirV1,
    source_len: usize,
    source_id: SourceIdV2,
    authority: fn() -> Result<ClosedFrontendAuthorityV2, String>,
) -> Result<ReferenceRelationDocumentV2, String> {
    let authority = authority()?;
    let query = builder.query(relation, None, SpanV1::new(0, source_len - 1));
    let item_id = SemanticItemIdV2::new(source_id, query.id);
    let document = resolve_query_document(source_id, source_len, query, &authority)?;
    Ok(ReferenceRelationDocumentV2 { document, item_id })
}

fn resolve_reference_with_budget(
    builder: &HirBuilderV1,
    relation: RelationExprHirV1,
    source_len: usize,
    budget: ReferenceBudgetV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    let authority = reference_authority_with_budget_v2(budget)?;
    let query = builder.query(relation, None, SpanV1::new(0, source_len - 1));
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let document = resolve_query_document(REFERENCE_SOURCE_V2, source_len, query, &authority)?;
    Ok(ReferenceRelationDocumentV2 { document, item_id })
}

fn resolve_query_document(
    source_id: SourceIdV2,
    source_len: usize,
    query: QueryHirV1,
    authority: &ClosedFrontendAuthorityV2,
) -> Result<SemanticDocumentV2, String> {
    resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .map_err(|error| format!("{error:?}"))?,
        authority,
    )
    .map_err(|error| format!("{error:?}"))
}

// ---- authorities -----------------------------------------------------------

fn reference_authority_v2() -> Result<ClosedFrontendAuthorityV2, String> {
    authority_with_registry_len(
        256,
        FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?,
    )
}

fn secondary_authority_v2() -> Result<ClosedFrontendAuthorityV2, String> {
    authority_with_registry_len(
        512,
        FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?,
    )
}

fn reference_authority_with_budget_v2(
    budget: ReferenceBudgetV2,
) -> Result<ClosedFrontendAuthorityV2, String> {
    match budget {
        ReferenceBudgetV2::Standard => reference_authority_v2(),
        ReferenceBudgetV2::Small => {
            let budget = FrontendBudgetV2::try_new(
                1_048_576,
                4_194_304,
                131_072,
                4_096,
                64,
                128,
                SMALL_BUDGET_MAX_EXPRESSION_NODES,
                4_096,
                4_096,
                SMALL_BUDGET_MAX_SEMANTIC_STEPS,
                50_000,
                200_000,
                1_000_000,
                16_777_216,
                65_536,
                50_000,
                200_000,
                1_000_000,
                1_024,
                10_000,
                1_024,
                8_388_608,
            )
            .map_err(|error| format!("{error:?}"))?;
            authority_with_registry_len(256, budget)
        }
    }
}

fn authority_with_registry_len(
    host_source_len: usize,
    budget: FrontendBudgetV2,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let digest_text = "0".repeat(64);
    let lock_text = format!("{REFERENCE_USE_PACK_ID_V2} 1.0.0 {digest_text}\n");
    let host = SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: host_source_len,
        },
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(2),
            source_len: lock_text.len(),
        },
    ])
    .map_err(|error| error.to_string())?;
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let module =
        ModuleIdV2::try_new("app", location.clone()).map_err(|error| format!("{error:?}"))?;
    let profile = budget
        .authority_profile()
        .map_err(|error| format!("{error:?}"))?;
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone())
        .map_err(|error| format!("{error:?}"))?;
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .map_err(|error| format!("{error:?}"))?;
    let pack = PackIdV2::try_new(REFERENCE_USE_PACK_ID_V2, location.clone())
        .map_err(|error| format!("{error:?}"))?;
    modules
        .try_add_manifest(PackManifestInputV2 {
            pack_id: pack,
            version: VersionV2::new(1, 0, 0),
            content_digest: ContentDigestV2::try_parse(&digest_text, location.clone())
                .map_err(|error| format!("{error:?}"))?,
            language_range: VersionRangeV2::try_new(
                VersionV2::new(1, 0, 0),
                VersionV2::new(2, 0, 0),
                location.clone(),
            )
            .map_err(|error| format!("{error:?}"))?,
            dependencies: Vec::new(),
            exports: Vec::new(),
            suites: Vec::new(),
            location: location.clone(),
        })
        .map_err(|error| format!("{error:?}"))?;
    modules
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: REFERENCE_USE_PACK_LOCKFILE_V2.to_owned(),
                location: location.clone(),
            },
            SourceIdV2::from_raw(2),
            &lock_text,
        )
        .map_err(|error| format!("{error:?}"))?;
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .map_err(|error| error.to_string())?;
    let relation_inputs = [
        host_relation("records", &[("id", ValueTypeV1::int())], &location)?,
        host_relation("other", &[("other_id", ValueTypeV1::int())], &location)?,
        host_relation(
            "records2",
            &[("id", ValueTypeV1::int()), ("value", ValueTypeV1::int())],
            &location,
        )?,
        host_relation(
            "records3",
            &[
                ("id", ValueTypeV1::int()),
                ("value", ValueTypeV1::int()),
                (
                    "label",
                    ValueTypeV1::try_optional(ValueTypeV1::string())
                        .map_err(|error| format!("{error:?}"))?,
                ),
            ],
            &location,
        )?,
    ];
    let relations = RelationCatalogV2::try_new(relation_inputs.to_vec(), &host, profile.clone())
        .map_err(|error| format!("{error:?}"))?;
    let graphs = GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile)
        .map_err(|error| format!("{error:?}"))?;
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget)
        .map_err(|error| format!("{error:?}"))
}

fn host_relation(
    lookup_name: &str,
    fields: &[(&str, ValueTypeV1)],
    location: &CheckedLocatedSpanV2,
) -> Result<RelationSourceInputV2, String> {
    let provenance_namespace = ProvenanceNamespaceV2::try_new("host", location.clone())
        .map_err(|error| format!("{error:?}"))?;
    let row = RecordTypeV1::try_new(
        fields
            .iter()
            .map(|(name, value_type)| {
                RecordFieldTypeV1::try_new(name, value_type.clone(), location.clone())
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("{error:?}"))?,
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(RelationSourceInputV2 {
        authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
        provenance_namespace,
        qualified_name: format!("app::{lookup_name}"),
        lookup_name: lookup_name.to_owned(),
        row,
        cardinality: CardinalityV1::Bag,
        provenance: "host".to_owned(),
        location: location.clone(),
        host_binding_identity: None,
    })
}

#[cfg(test)]
#[path = "test_support/tests.rs"]
mod tests;

mod families;

pub use families::{
    forged_non_source_rule_call_document_v2, forged_rule_call_shape_target_document_v2,
    forged_rule_call_wrapper_document_v2, forged_rule_fact_document_v2,
    forged_rule_membership_document_v2, forged_small_receipt_rule_document_v2,
    forged_small_receipt_rule_program_document_v2, forged_use_pack_meta_document_v2,
    reference_alternate_graph_document_v2, reference_check_document_v2, reference_checked_span_v2,
    reference_composition_document_v2, reference_graph_document_v2, reference_meta_document_v2,
    reference_narrow_witness_graph_document_v2, reference_nested_check_document_v2,
    reference_presentation_document_v2, reference_query_producer_composition_v2,
    reference_rule_document_v2, reference_rule_program_charge_v2,
    reference_witness_graph_references_v2, second_source_graph_document_v2,
    ForgedRuleCallWrapperV2, ForgedRuleFactMutationV2, ForgedRuleMembershipMutationV2,
    ReferenceCheckDocumentV2, ReferenceCheckFixtureV2, ReferenceCompositionBodyV2,
    ReferenceCompositionDocumentV2, ReferenceGraphDocumentV2, ReferenceGraphReferenceV2,
    ReferenceGraphRequestV2, ReferenceMetaDocumentV2, ReferenceMetaFixtureV2,
    ReferenceNestedCheckDocumentV2, ReferencePresentationDocumentV2,
    ReferencePresentationFixtureV2, ReferenceRuleDocumentV2, ReferenceRuleFixtureV2,
    ALTERNATE_GRAPH_NODE_ID_V2, NARROW_WITNESS_MAX_WITNESSES_V2,
    REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2, REFERENCE_COMPOSITION_RULE_V2,
    REFERENCE_COMPOSITION_SINK_SELECTOR_V2, REFERENCE_COMPOSITION_SOURCE_LEN_V2,
    REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2, REFERENCE_GRAPH_SOURCE_LEN_V2,
    REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2, REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2,
    REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2, REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2,
    REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2, REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2,
    REFERENCE_PRESENTATION_SCOPE_SELECTOR_V2, REFERENCE_PRESENTATION_SOURCE_LEN_V2,
    REFERENCE_REQUIRE_CERTIFIED_SOURCE_V2, REFERENCE_USE_PACK_ID_V2,
    REFERENCE_USE_PACK_LOCKFILE_V2, REFERENCE_USE_PACK_SOURCE_V2,
    SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS, SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS,
};

// ---- W4-D fixture units ----------------------------------------------------

#[cfg(test)]
#[path = "test_support/w4d_tests.rs"]
mod w4d_tests;
