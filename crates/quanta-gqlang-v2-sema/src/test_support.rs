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

/// Forged document for the wrong-scan-occurrence rail, minted from the real
/// self-join fixture: the left projection's output column origin is rebuilt as
/// a `SourceField` whose `scan_stage` is the right scan occurrence of the same
/// host relation (a same-source, wrong-occurrence transplant in a row-aligned
/// column origin).
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_wrong_scan_occurrence_document_v2() -> Result<ReferenceRelationDocumentV2, String> {
    let (checked, real, item_id) = self_join_parts()?;
    let relation = relation_item(&real, item_id)?;
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
    let forged_left = TypedRelationExprV2::try_project(
        left_project.key(),
        left_project.location().clone(),
        left_scan,
        left_project.expressions().to_vec(),
        transplanted_output,
    )
    .map_err(str::to_owned)?;
    let forged_root = TypedRelationExprV2::join(
        relation.root().key(),
        relation.root().location().clone(),
        forged_left,
        right_project,
        relation
            .root()
            .predicate()
            .ok_or("join has no predicate")?
            .clone(),
        relation.output().clone(),
    )
    .map_err(str::to_owned)?;
    reassemble_with_forged_relation(&checked, &real, item_id, forged_root)
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
                        .map(|(value, direction)| OrderKeyHirV1 { value, direction })
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
                    vec![OrderKeyHirV1 {
                        value: key,
                        direction: SortDirectionHirV1::Desc,
                    }],
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
            vec![OrderKeyHirV1 {
                value: right_key,
                direction: SortDirectionHirV1::Asc,
            }],
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

fn self_join_parts() -> Result<
    (
        CheckedSyntaxDocumentV2,
        SemanticDocumentV2,
        SemanticItemIdV2,
    ),
    String,
> {
    let builder = HirBuilderV1::new();
    let (relation, source_len) = self_join_hir(&builder)?;
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
    })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::indexing_slicing,
        clippy::panic_in_result_fn,
        reason = "owner assertions pin exact fixture shapes and index pinned capture vectors"
    )]

    use super::*;
    use crate::{NullOrderV2, RelationExistsPolarityV2, SortDirectionV2};

    #[test]
    fn every_positive_fixture_variant_resolves() -> Result<(), String> {
        let variants = [
            ReferenceRelationFixtureV2::Scan,
            ReferenceRelationFixtureV2::Filter,
            ReferenceRelationFixtureV2::CrossJoin,
            ReferenceRelationFixtureV2::Exists,
            ReferenceRelationFixtureV2::NotExists,
            ReferenceRelationFixtureV2::ProjectMultipleExpressions,
            ReferenceRelationFixtureV2::InnerJoinOn,
            ReferenceRelationFixtureV2::SelfJoinInner,
            ReferenceRelationFixtureV2::GroupedAggregateSum,
            ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
            ReferenceRelationFixtureV2::ScalarCountStar,
            ReferenceRelationFixtureV2::SortAsc,
            ReferenceRelationFixtureV2::SortDesc,
            ReferenceRelationFixtureV2::SortNullsLastNullableField,
            ReferenceRelationFixtureV2::SortMultiKeyMixedDirections,
            ReferenceRelationFixtureV2::SortAliasKeyed,
            ReferenceRelationFixtureV2::SortByProjectedExpression,
            ReferenceRelationFixtureV2::CorrelatedExists,
            ReferenceRelationFixtureV2::CorrelatedNotExists,
            ReferenceRelationFixtureV2::ConjunctiveExistsPair,
            ReferenceRelationFixtureV2::DisjunctiveExistsPair,
            ReferenceRelationFixtureV2::NegatedExists,
            ReferenceRelationFixtureV2::ExistsWithBoolConjunct,
            ReferenceRelationFixtureV2::NestedExistsTwoLevels,
            ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight,
            ReferenceRelationFixtureV2::AsymmetricJoinPreorder,
        ];
        for fixture in variants {
            let reference = reference_relation_document_v2(fixture)?;
            assert!(relation_item(reference.document(), reference.item_id()).is_ok());
        }
        Ok(())
    }

    #[test]
    fn projection_fixture_retains_expression_origins_and_types() -> Result<(), String> {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)?;
        let relation = relation_item(reference.document(), reference.item_id())?;
        let columns = relation.output().columns();
        assert_eq!(columns.len(), 2);
        let first = columns.first().ok_or("first column expected")?;
        let second = columns.get(1).ok_or("second column expected")?;
        assert_eq!(first.name(), "num");
        assert_eq!(second.name(), "label");
        assert!(matches!(
            first.origin(),
            OutputColumnOriginV2::Expression(_)
        ));
        assert!(matches!(
            second.origin(),
            OutputColumnOriginV2::Expression(_)
        ));
        assert_eq!(
            relation
                .output()
                .row()
                .fields()
                .get(1)
                .ok_or("second field expected")?
                .value_type(),
            &ValueTypeV1::string(),
        );
        Ok(())
    }

    #[test]
    fn join_fixtures_retain_mode_and_distinct_scan_occurrences() -> Result<(), String> {
        let join = reference_relation_document_v2(ReferenceRelationFixtureV2::InnerJoinOn)?;
        let relation = relation_item(join.document(), join.item_id())?;
        assert_eq!(relation.root().join_mode(), Some(JoinModeV2::Inner));
        assert!(relation.root().predicate().is_some());
        assert_eq!(relation.root().inputs().count(), 2);

        let self_join = reference_relation_document_v2(ReferenceRelationFixtureV2::SelfJoinInner)?;
        let relation = relation_item(self_join.document(), self_join.item_id())?;
        let mut inputs = relation.root().inputs();
        let left = inputs.next().ok_or("left input expected")?;
        let right = inputs.next().ok_or("right input expected")?;
        let left_scan = left.inputs().next().ok_or("left scan expected")?;
        let right_scan = right.inputs().next().ok_or("right scan expected")?;
        assert_eq!(left_scan.source(), right_scan.source());
        assert_ne!(left_scan.key(), right_scan.key());
        Ok(())
    }

    #[test]
    fn aggregate_fixtures_retain_group_shapes() -> Result<(), String> {
        let grouped =
            reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)?;
        let relation = relation_item(grouped.document(), grouped.item_id())?;
        assert!(matches!(relation.root(), TypedRelationExprV2::Project(_)));
        let group = relation
            .root()
            .inputs()
            .next()
            .ok_or("projection input expected")?;
        assert!(matches!(group, TypedRelationExprV2::Group(_)));
        assert_eq!(group.expressions().len(), 1);
        assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
        assert_eq!(
            relation.output().partition_keys(),
            group.output().partition_keys()
        );

        let scalar = reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar)?;
        let relation = relation_item(scalar.document(), scalar.item_id())?;
        let group = relation
            .root()
            .inputs()
            .next()
            .ok_or("implicit group expected")?;
        assert!(matches!(group, TypedRelationExprV2::Group(_)));
        assert!(group.expressions().is_empty());
        assert!(group.partition_input().is_empty());
        assert!(relation.output().partition_keys().is_empty());
        Ok(())
    }

    #[test]
    fn sort_fixtures_retain_directions_and_null_policy() -> Result<(), String> {
        let cases = [
            (
                ReferenceRelationFixtureV2::SortAsc,
                vec![(SortDirectionV2::Asc, NullOrderV2::Last)],
            ),
            (
                ReferenceRelationFixtureV2::SortDesc,
                vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
            ),
            (
                ReferenceRelationFixtureV2::SortNullsLastNullableField,
                vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
            ),
            (
                ReferenceRelationFixtureV2::SortMultiKeyMixedDirections,
                vec![
                    (SortDirectionV2::Asc, NullOrderV2::Last),
                    (SortDirectionV2::Desc, NullOrderV2::Last),
                ],
            ),
            (
                ReferenceRelationFixtureV2::SortAliasKeyed,
                vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
            ),
            (
                ReferenceRelationFixtureV2::SortByProjectedExpression,
                vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
            ),
        ];
        for (fixture, expected) in cases {
            let reference = reference_relation_document_v2(fixture)?;
            let relation = relation_item(reference.document(), reference.item_id())?;
            assert!(matches!(relation.root(), TypedRelationExprV2::Sort(_)));
            let actual: Vec<_> = relation
                .output()
                .ordering()
                .iter()
                .map(|key| (key.direction(), key.null_order()))
                .collect();
            assert_eq!(actual, expected);
        }
        Ok(())
    }

    #[test]
    fn exists_fixtures_retain_correlations_and_conjuncts() -> Result<(), String> {
        let correlated =
            reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists)?;
        let mut correlated_shapes =
            correlated
                .document()
                .expressions()
                .filter_map(|(_key, expression)| {
                    if let TypedExprShapeV2::RelationExists {
                        polarity,
                        correlations,
                        ..
                    } = expression.shape()
                    {
                        Some((*polarity, correlations.len()))
                    } else {
                        None
                    }
                });
        assert_eq!(
            correlated_shapes.next(),
            Some((RelationExistsPolarityV2::Exists, 1))
        );

        let not_exists =
            reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedNotExists)?;
        assert!(not_exists.document().expressions().any(
            |(_key, expression)| matches!(expression.shape(),
                TypedExprShapeV2::RelationExists {
                    polarity: RelationExistsPolarityV2::NotExists,
                    correlations,
                    ..
                } if !correlations.is_empty())
        ));

        let nested =
            reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels)?;
        assert_eq!(nested.document().nested_relations().count(), 2);
        let pair =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ConjunctiveExistsPair)?;
        assert_eq!(pair.document().nested_relations().count(), 2);
        let mixed =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ExistsWithBoolConjunct)?;
        assert_eq!(mixed.document().nested_relations().count(), 1);

        let disjunctive =
            reference_relation_document_v2(ReferenceRelationFixtureV2::DisjunctiveExistsPair)?;
        assert_eq!(disjunctive.document().nested_relations().count(), 2);
        let negated = reference_relation_document_v2(ReferenceRelationFixtureV2::NegatedExists)?;
        assert_eq!(negated.document().nested_relations().count(), 1);
        assert!(negated
            .document()
            .expressions()
            .any(|(_key, expression)| matches!(expression.shape(), TypedExprShapeV2::Not { .. })));
        assert!(disjunctive
            .document()
            .expressions()
            .any(|(_key, expression)| matches!(expression.shape(), TypedExprShapeV2::Or { .. })));
        Ok(())
    }

    #[test]
    fn second_source_document_is_foreign_to_the_primary_source() -> Result<(), String> {
        let secondary = second_source_relation_document_v2()?;
        assert_eq!(
            secondary.document().source_id(),
            SECONDARY_REFERENCE_SOURCE_V2
        );
        assert!(relation_item(secondary.document(), secondary.item_id()).is_ok());
        let primary = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)?;
        assert_ne!(
            primary.document().source_id(),
            secondary.document().source_id()
        );
        assert!(primary.document().item(secondary.item_id()).is_none());
        Ok(())
    }

    #[test]
    fn deep_chain_bounds_are_exact_at_the_small_budget() -> Result<(), String> {
        let exact = deep_relation_chain_document_v2(
            SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH,
            ReferenceBudgetV2::Small,
        )?;
        assert_eq!(
            exact.document().budget_receipt().max_semantic_steps(),
            SMALL_BUDGET_MAX_SEMANTIC_STEPS
        );
        let over = deep_relation_chain_document_v2(
            SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH,
            ReferenceBudgetV2::Small,
        );
        assert!(over
            .err()
            .is_some_and(|error| error.contains("semantic_steps exceeds frontend limit")));
        let deep = deep_relation_chain_document_v2(24, ReferenceBudgetV2::Standard)?;
        assert!(relation_item(deep.document(), deep.item_id()).is_ok());
        Ok(())
    }

    #[test]
    fn deep_bool_tree_bounds_follow_the_documented_formulas() -> Result<(), String> {
        let exact = deep_bool_tree_document_v2(
            SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH,
            ReferenceBudgetV2::Small,
        )?;
        assert_eq!(
            exact.document().budget_receipt().max_expression_nodes(),
            SMALL_BUDGET_MAX_EXPRESSION_NODES
        );
        assert_eq!(
            u32::try_from(exact.document().expressions().count())
                .map_err(|_error| "expression fact count exceeds u32".to_owned())?,
            3 * SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH + 2
        );
        let over =
            deep_bool_tree_document_v2(SMALL_BUDGET_BOOL_TREE_OVER_DEPTH, ReferenceBudgetV2::Small);
        assert!(over
            .err()
            .is_some_and(|error| error.contains("semantic_steps exceeds frontend limit")));
        Ok(())
    }

    #[test]
    fn two_key_group_fixture_retains_both_partition_keys_in_authored_order() -> Result<(), String> {
        let reference = reference_relation_document_v2(
            ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
        )?;
        let relation = relation_item(reference.document(), reference.item_id())?;
        assert!(matches!(relation.root(), TypedRelationExprV2::Project(_)));
        let group = relation
            .root()
            .inputs()
            .next()
            .ok_or("projection input expected")?;
        assert!(matches!(group, TypedRelationExprV2::Group(_)));
        assert_eq!(group.expressions().len(), 2);
        assert_eq!(group.output().partition_keys().len(), 2);
        assert_eq!(relation.output().partition_keys().len(), 2);
        assert_eq!(
            relation.output().partition_keys(),
            group.output().partition_keys()
        );
        let first_key = group
            .expressions()
            .first()
            .copied()
            .ok_or("first key expected")?;
        let second_key = group
            .expressions()
            .get(1)
            .copied()
            .ok_or("second key expected")?;
        assert_ne!(first_key, second_key);
        let first_name = group
            .output()
            .columns()
            .first()
            .ok_or("first key column expected")?
            .name()
            .to_owned();
        let second_name = group
            .output()
            .columns()
            .get(1)
            .ok_or("second key column expected")?
            .name()
            .to_owned();
        assert_eq!(first_name, "id");
        assert_eq!(second_name, "value");
        Ok(())
    }

    #[test]
    fn forged_aggregate_without_group_removes_the_implicit_group() -> Result<(), String> {
        let forged = forged_aggregate_without_group_document_v2()?;
        let relation = relation_item(forged.document(), forged.item_id())?;
        assert!(matches!(relation.root(), TypedRelationExprV2::Project(_)));
        let input = relation
            .root()
            .inputs()
            .next()
            .ok_or("projection input expected")?;
        assert!(matches!(input, TypedRelationExprV2::Scan(_)));

        let real = reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar)?;
        let real_relation = relation_item(real.document(), real.item_id())?;
        let real_group = real_relation
            .root()
            .inputs()
            .next()
            .ok_or("real group expected")?;
        assert!(matches!(real_group, TypedRelationExprV2::Group(_)));
        Ok(())
    }

    #[test]
    fn forged_mixed_projection_carries_scalar_and_aggregate_columns() -> Result<(), String> {
        let forged = forged_mixed_scalar_aggregate_projection_document_v2()?;
        let relation = relation_item(forged.document(), forged.item_id())?;
        assert_eq!(relation.root().expressions().len(), 2);
        assert_eq!(relation.output().columns().len(), 2);
        let aggregate_classes: Vec<_> = relation
            .root()
            .expressions()
            .iter()
            .filter_map(|key| {
                forged
                    .document()
                    .expression(*key)
                    .map(|expression| expression.aggregate_context().aggregation())
            })
            .collect();
        assert_eq!(aggregate_classes.len(), 2);
        assert!(aggregate_classes.contains(&AggregationClassV1::Aggregate));
        assert!(aggregate_classes.contains(&AggregationClassV1::Scalar));
        Ok(())
    }

    #[test]
    fn forged_cardinality_downgrade_pins_bag_on_the_aggregate_stage() -> Result<(), String> {
        let forged = forged_cardinality_downgrade_document_v2()?;
        let relation = relation_item(forged.document(), forged.item_id())?;
        assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
        let real = reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)?;
        let real_relation = relation_item(real.document(), real.item_id())?;
        assert_eq!(real_relation.output().cardinality(), CardinalityV1::Set);
        Ok(())
    }

    #[test]
    fn forged_wrong_scan_occurrence_transplants_the_right_stage_key() -> Result<(), String> {
        let forged = forged_wrong_scan_occurrence_document_v2()?;
        let relation = relation_item(forged.document(), forged.item_id())?;
        assert!(matches!(relation.root(), TypedRelationExprV2::Join(_)));
        let mut inputs = relation.root().inputs();
        let left = inputs.next().ok_or("left input expected")?;
        let right = inputs.next().ok_or("right input expected")?;
        let right_scan = right.inputs().next().ok_or("right scan expected")?;
        let OutputColumnOriginV2::SourceField { scan_stage, .. } = left
            .output()
            .columns()
            .first()
            .ok_or("left column expected")?
            .origin()
        else {
            return Err("forged left column must carry a SourceField origin".to_owned());
        };
        assert_eq!(*scan_stage, right_scan.key());
        assert_ne!(*scan_stage, left.key());
        Ok(())
    }

    #[test]
    fn forged_wrong_nested_root_swaps_the_nested_tree() -> Result<(), String> {
        let forged = forged_wrong_nested_root_document_v2()?;
        let real = reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists)?;
        let forged_key = exists_root(forged.document())?;
        let real_key = exists_root(real.document())?;
        assert_eq!(forged_key, real_key);
        let forged_nested = forged
            .document()
            .nested_relation(forged_key)
            .ok_or("forged nested relation expected")?;
        let real_nested = real
            .document()
            .nested_relation(real_key)
            .ok_or("real nested relation expected")?;
        assert_eq!(forged_nested.key(), real_nested.key());
        assert_ne!(
            forged_nested.inputs().next().map(TypedRelationExprV2::key),
            real_nested.inputs().next().map(TypedRelationExprV2::key)
        );
        Ok(())
    }

    #[test]
    fn forged_capture_mutations_assemble_or_reject_exactly() -> Result<(), String> {
        let capture_vector = |document: &ReferenceRelationDocumentV2| -> Result<Vec<SourceQualifiedNodeIdV2>, String> {
            let relation = relation_item(document.document(), document.item_id())?;
            if let TypedExprShapeV2::RelationExists { correlations, .. } = relation
                .root()
                .predicate()
                .ok_or("outer predicate expected")?
                .expression()
                .shape()
            {
                Ok(correlations.iter().map(|edge| edge.reference()).collect())
            } else {
                Err("EXISTS predicate expected".to_owned())
            }
        };
        let (_checked, real, item_id) = two_capture_correlated_exists_parts()?;
        assert!(relation_item(&real, item_id).is_ok());
        let pristine_document = ReferenceRelationDocumentV2 {
            document: real.clone(),
            item_id,
        };
        let pristine = capture_vector(&pristine_document)?;
        assert_eq!(pristine.len(), 2);
        assert_ne!(pristine[0], pristine[1]);

        let dropped = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Dropped)?;
        assert_eq!(capture_vector(&dropped)?, vec![pristine[0]]);
        let swapped = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Swapped)?;
        assert_eq!(capture_vector(&swapped)?, vec![pristine[1], pristine[0]]);

        let duplicate = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Duplicate);
        assert!(duplicate.err().is_some_and(
            |error| error.contains("duplicate correlation reference in one nested relation")
        ));
        let foreign = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Foreign);
        assert!(foreign.err().is_some_and(|error| error
            .contains("correlation reference is foreign to this document's expressions")));

        // The map fact and the Filter predicate carry the same mutated vector.
        let swapped_fact = swapped
            .document()
            .expressions()
            .find_map(|(_key, expression)| {
                if let TypedExprShapeV2::RelationExists { correlations, .. } = expression.shape() {
                    Some(
                        correlations
                            .iter()
                            .map(|edge| edge.reference())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .ok_or("swapped document must retain its EXISTS fact")?;
        assert_eq!(swapped_fact, vec![pristine[1], pristine[0]]);
        assert!(relation_item(swapped.document(), swapped.item_id()).is_ok());
        assert_eq!(swapped.document().nested_relations().count(), 1);
        Ok(())
    }

    #[test]
    fn forged_join_seal_rejections_are_exact() -> Result<(), String> {
        assert_eq!(
            forged_inner_join_without_on_seal_v2().err(),
            Some("invalid typed relation stage arity or payload")
        );
        assert_eq!(
            forged_cross_join_with_on_seal_v2().err(),
            Some("invalid typed relation stage arity or payload")
        );
        Ok(())
    }

    #[test]
    fn capture_edge_mutations_match_their_exact_shapes() -> Result<(), String> {
        let pristine = pristine_capture_edge_set_v2()?;
        assert_eq!(pristine.len(), 2);
        assert_ne!(
            pristine.first().map(|edge| edge.reference().node_id),
            pristine.get(1).map(|edge| edge.reference().node_id)
        );
        let dropped = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Dropped)?;
        assert_eq!(dropped.len(), 1);
        let duplicate = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Duplicate)?;
        assert_eq!(duplicate.len(), 2);
        assert_eq!(
            duplicate.first().map(|edge| edge.reference().node_id),
            duplicate.get(1).map(|edge| edge.reference().node_id)
        );
        let swapped = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Swapped)?;
        assert_eq!(
            swapped.first().map(|edge| edge.reference().node_id),
            pristine.get(1).map(|edge| edge.reference().node_id)
        );
        let foreign = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Foreign)?;
        assert_eq!(
            foreign.get(1).map(|edge| edge.reference().node_id),
            Some(HirNodeIdV1::from_raw(9_999))
        );
        Ok(())
    }

    #[test]
    fn forged_small_receipt_documents_carry_oversized_trees() -> Result<(), String> {
        let chain = forged_small_receipt_relation_chain_document_v2(
            SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH,
        )?;
        assert_eq!(
            chain.document().budget_receipt().max_semantic_steps(),
            SMALL_BUDGET_MAX_SEMANTIC_STEPS
        );
        assert!(relation_item(chain.document(), chain.item_id()).is_ok());

        let tree = forged_small_receipt_bool_tree_document_v2(4)?;
        assert_eq!(
            tree.document().budget_receipt().max_expression_nodes(),
            SMALL_BUDGET_MAX_EXPRESSION_NODES
        );
        assert_eq!(tree.document().expressions().count(), 14);
        Ok(())
    }
}

// ---- graph fixtures (W3-B verification scaffold) ---------------------------

/// Fixed source length of every reference graph fixture document.
pub const REFERENCE_GRAPH_SOURCE_LEN_V2: usize = 4_096;

/// One requested resolved FIND reference.
#[derive(Clone, Debug)]
pub struct ReferenceGraphReferenceV2 {
    /// Semantic FIND role.
    pub role: GraphReferenceRoleV2,
    /// Host selector spelling registered for the role's required kind.
    pub selector: &'static str,
    /// Authored byte span of the reference.
    pub span: (usize, usize),
}

/// One requested resolved FIND document.
#[derive(Clone, Debug)]
pub struct ReferenceGraphRequestV2 {
    /// FIND subkind.
    pub kind: GraphQueryKindV1,
    /// Ordered references.
    pub references: Vec<ReferenceGraphReferenceV2>,
    /// Optional authored depth.
    pub depth: Option<u32>,
    /// Optional direction keyword.
    pub direction: Option<&'static str>,
    /// Summary facets.
    pub summary_facets: Vec<String>,
    /// Authored span of the FIND item.
    pub item_span: (usize, usize),
}

/// A sealed graph fixture document plus its FIND item identity.
#[derive(Clone, Debug)]
pub struct ReferenceGraphDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferenceGraphDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }
    /// Identity of the FIND item.
    #[must_use]
    pub const fn graph_item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// The resolved FIND item.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Graph item is unreachable"
    )]
    pub fn graph(&self) -> &ResolvedGraphQueryV2 {
        match self.document.item(self.item_id) {
            Some(SemanticItemV2::Graph(graph)) => graph,
            _ => panic!("graph fixture item is a resolved FIND"),
        }
    }
}

/// A sealed nested CHECK/FIND fixture document plus both item identities.
#[derive(Clone, Debug)]
pub struct ReferenceNestedCheckDocumentV2 {
    document: SemanticDocumentV2,
    check_item_id: SemanticItemIdV2,
    nested_graph_item_id: SemanticItemIdV2,
}

/// Caller-selected CHECK shape resolved through the real semantic transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceCheckFixtureV2 {
    /// Authored detection mode.
    pub mode: DetectionModeV1,
    /// Whether the CHECK carries a resolved pattern selector.
    pub with_pattern: bool,
    /// Whether the CHECK carries an explicit nested semantic-confirm FIND.
    pub with_nested_graph: bool,
}

/// A configurable sealed CHECK fixture and its optional nested FIND identity.
#[derive(Clone, Debug)]
pub struct ReferenceCheckDocumentV2 {
    document: SemanticDocumentV2,
    check_item_id: SemanticItemIdV2,
    nested_graph_item_id: Option<SemanticItemIdV2>,
}

impl ReferenceCheckDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }
    /// Identity of the CHECK item.
    #[must_use]
    pub const fn check_item_id(&self) -> SemanticItemIdV2 {
        self.check_item_id
    }
    /// Identity of the nested FIND when the fixture requested one.
    #[must_use]
    pub const fn nested_graph_item_id(&self) -> Option<SemanticItemIdV2> {
        self.nested_graph_item_id
    }
    /// The resolved CHECK item.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Check item is unreachable"
    )]
    pub fn check(&self) -> &crate::ResolvedCheckV2 {
        match self.document.item(self.check_item_id) {
            Some(SemanticItemV2::Check(check)) => check,
            _ => panic!("check fixture item is a resolved CHECK"),
        }
    }
}

impl ReferenceNestedCheckDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }
    /// Identity of the CHECK item.
    #[must_use]
    pub const fn check_item_id(&self) -> SemanticItemIdV2 {
        self.check_item_id
    }
    /// Identity of the nested FIND item.
    #[must_use]
    pub const fn nested_graph_item_id(&self) -> SemanticItemIdV2 {
        self.nested_graph_item_id
    }
}

/// Mint one checked location against the reference graph fixture registry.
///
/// # Errors
///
/// Returns a fixture error when the byte range is out of bounds.
pub fn reference_checked_span_v2(start: usize, end: usize) -> Result<CheckedLocatedSpanV2, String> {
    let registry = SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 256,
        },
        SourceRegistrationV2 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        },
    ])
    .map_err(|error| error.to_string())?;
    registry
        .try_check_location(LocatedSpanV2 {
            source_id: REFERENCE_SOURCE_V2,
            span: SpanV1::new(start, end),
        })
        .map_err(|error| format!("{error:?}"))
}

fn graph_reference_kind(role: GraphReferenceRoleV2) -> GraphReferenceKindV2 {
    match role {
        GraphReferenceRoleV2::Source
        | GraphReferenceRoleV2::Sink
        | GraphReferenceRoleV2::Anchor
        | GraphReferenceRoleV2::Reference
        | GraphReferenceRoleV2::Pointee
        | GraphReferenceRoleV2::Via
        | GraphReferenceRoleV2::Through => GraphReferenceKindV2::Entity,
        GraphReferenceRoleV2::Callable => GraphReferenceKindV2::Callable,
        GraphReferenceRoleV2::Sanitizer => GraphReferenceKindV2::Sanitizer,
        GraphReferenceRoleV2::Scope => GraphReferenceKindV2::Scope,
        GraphReferenceRoleV2::Window => GraphReferenceKindV2::Window,
        GraphReferenceRoleV2::Frontier => GraphReferenceKindV2::Frontier,
    }
}

fn graph_hir_query(request: &ReferenceGraphRequestV2) -> Result<GraphQueryHirV1, String> {
    graph_hir_query_with_node_v2(request, HirNodeIdV1::from_raw(2))
}

/// Author one FIND query under a caller-chosen node identity. The reference
/// fixtures use node 2; [`reference_alternate_graph_document_v2`] uses
/// [`ALTERNATE_GRAPH_NODE_ID_V2`] so a nested-graph identity mismatch is
/// reachable end to end.
fn graph_hir_query_with_node_v2(
    request: &ReferenceGraphRequestV2,
    node: HirNodeIdV1,
) -> Result<GraphQueryHirV1, String> {
    let mut endpoints = GraphEndpointsV1::default();
    let mut constraints = GraphConstraintsV1 {
        depth: request.depth,
        direction: request.direction.map(|keyword| GraphDirectionV1 {
            keyword: keyword.to_owned(),
        }),
        summary_facets: request.summary_facets.clone(),
        ..GraphConstraintsV1::default()
    };
    for reference in &request.references {
        let graph_ref = GraphRefV1 {
            name: reference.selector.to_owned(),
            span: SpanV1::new(reference.span.0, reference.span.1),
        };
        match reference.role {
            GraphReferenceRoleV2::Source => endpoints.source = Some(graph_ref),
            GraphReferenceRoleV2::Sink => endpoints.sink = Some(graph_ref),
            GraphReferenceRoleV2::Anchor => endpoints.anchor = Some(graph_ref),
            GraphReferenceRoleV2::Reference => endpoints.reference = Some(graph_ref),
            GraphReferenceRoleV2::Pointee => endpoints.pointee = Some(graph_ref),
            GraphReferenceRoleV2::Callable => endpoints.callable = Some(graph_ref),
            GraphReferenceRoleV2::Via => constraints.via.push(graph_ref),
            GraphReferenceRoleV2::Through => constraints.through.push(graph_ref),
            GraphReferenceRoleV2::Sanitizer => constraints.sanitizers.push(graph_ref),
            GraphReferenceRoleV2::Scope => constraints.scope = Some(graph_ref),
            GraphReferenceRoleV2::Window => constraints.window = Some(graph_ref),
            GraphReferenceRoleV2::Frontier => constraints.frontier = Some(graph_ref),
        }
    }
    Ok(GraphQueryHirV1::new(
        node,
        SpanV1::new(request.item_span.0, request.item_span.1),
        request.kind,
        endpoints,
        constraints,
    ))
}

fn graph_authority_v2(
    references: &[ReferenceGraphReferenceV2],
    with_rule: bool,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    graph_authority_with_budget_v2(references, with_rule, budget)
}

fn graph_authority_with_budget_v2(
    references: &[ReferenceGraphReferenceV2],
    with_rule: bool,
    budget: FrontendBudgetV2,
) -> Result<ClosedFrontendAuthorityV2, String> {
    graph_authority_with_graph_inputs_v2(with_rule, budget, |host, _location| {
        let mut inputs = Vec::<GraphReferenceInputV2>::new();
        for (index, reference) in references.iter().enumerate() {
            let span = SpanV1::new(1 + index, 2 + index);
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: reference.selector.to_owned(),
                kind: graph_reference_kind(reference.role),
                entity_subtype: None,
                location: declaration,
            });
        }
        Ok(inputs)
    })
}

/// Seal one closed authority whose graph-selector inputs are derived from the
/// already-checked host registry.
fn graph_authority_with_graph_inputs_v2(
    with_rule: bool,
    budget: FrontendBudgetV2,
    make_inputs: impl FnOnce(
        &SourceRegistryViewV2,
        &CheckedLocatedSpanV2,
    ) -> Result<Vec<GraphReferenceInputV2>, String>,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 256,
    }])
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
    if with_rule {
        modules
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".to_owned(),
                local_name: "rule1".to_owned(),
                kind_token: "rule".to_owned(),
                typed_payload: ExternalCatalogPayloadInputV2::Rule(SignatureInputV2 {
                    parameters: Vec::<LocatedParameterInputV2>::new(),
                    result: ValueTypeV1::bool(),
                    callable_kind: CallableKindV2::Rule,
                    effects: ExpressionEffectsV2::pure_scalar(),
                    location: location.clone(),
                }),
                provenance: "host".to_owned(),
                location: location.clone(),
                dependencies: Vec::new(),
            })
            .map_err(|error| format!("{error:?}"))?;
    }
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .map_err(|error| error.to_string())?;
    let relations = RelationCatalogV2::try_new(Vec::new(), &host, profile.clone())
        .map_err(|error| format!("{error:?}"))?;
    let inputs = make_inputs(&host, &location)?;
    let graphs = GraphReferenceCatalogV2::try_new(inputs, &host, profile)
        .map_err(|error| format!("{error:?}"))?;
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget)
        .map_err(|error| format!("{error:?}"))
}

/// Resolve one reference FIND fixture through the real semantic transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_graph_document_v2(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceGraphDocumentV2, String> {
    let query = graph_hir_query(&request)?;
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let authority = graph_authority_v2(&request.references, false)?;
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
            items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
        })
        .map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

/// Resolve one reference nested CHECK/FIND fixture through the real semantic
/// transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_nested_check_document_v2(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceNestedCheckDocumentV2, String> {
    let fixture = reference_check_document_v2(
        request,
        ReferenceCheckFixtureV2 {
            mode: DetectionModeV1::SemanticOnly,
            with_pattern: false,
            with_nested_graph: true,
        },
    )?;
    let nested_graph_item_id = fixture
        .nested_graph_item_id
        .ok_or("nested CHECK fixture must carry a nested graph")?;
    Ok(ReferenceNestedCheckDocumentV2 {
        document: fixture.document,
        check_item_id: fixture.check_item_id,
        nested_graph_item_id,
    })
}

/// Resolve a caller-selected CHECK mode and pattern/nested-FIND combination.
///
/// The options are intentionally not normalized: tests can author both the
/// coherent combinations and forged policy combinations while every retained
/// identity/reference still comes from the real semantic transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the selected shape cannot seal.
pub fn reference_check_document_v2(
    request: ReferenceGraphRequestV2,
    fixture: ReferenceCheckFixtureV2,
) -> Result<ReferenceCheckDocumentV2, String> {
    const PATTERN_NAME: &str = "pattern1";
    const PATTERN_SPAN: SpanV1 = SpanV1::new(106, 114);

    let query = fixture
        .with_nested_graph
        .then(|| graph_hir_query(&request))
        .transpose()?;
    let nested_graph_item_id = query
        .as_ref()
        .map(|query| SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id));
    let pattern = fixture.with_pattern.then(|| GraphRefV1 {
        name: PATTERN_NAME.to_owned(),
        span: PATTERN_SPAN,
    });
    let mut check = LegalityHirV1::new(
        HirNodeIdV1::from_raw(3),
        SpanV1::new(request.item_span.0, request.item_span.1),
        GraphRefV1 {
            name: "rule1".to_owned(),
            span: SpanV1::new(100, 105),
        },
        None,
        pattern,
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        fixture.mode,
    );
    if let Some(query) = query {
        check = check.with_semantic_confirm(query);
    }
    let check_item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, check.id);
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let authority = graph_authority_with_graph_inputs_v2(true, budget, |host, _location| {
        let mut inputs = Vec::new();
        for (index, reference) in request.references.iter().enumerate() {
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(1 + index, 2 + index),
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: reference.selector.to_owned(),
                kind: graph_reference_kind(reference.role),
                entity_subtype: None,
                location: declaration,
            });
        }
        if fixture.with_pattern {
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(200, 201),
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: PATTERN_NAME.to_owned(),
                kind: GraphReferenceKindV2::Pattern,
                entity_subtype: None,
                location: declaration,
            });
        }
        Ok(inputs)
    })?;
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
            items: vec![ProgramItemHirV1::Legality(Box::new(check))],
        })
        .map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceCheckDocumentV2 {
        document,
        check_item_id,
        nested_graph_item_id,
    })
}

// ---- rule documents --------------------------------------------------------

/// Authored body shape of one fixture rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuleBodyShapeV2 {
    /// Only the anchor comparison; the body proves no call at all.
    AnchorOnly,
    /// `r(x)` — one positive source-predicate dependency.
    Positive(&'static str),
    /// `not r(x)` — one negated source-predicate dependency.
    Negated(&'static str),
    /// `count(r(x)) > 0` — a source-predicate call under an aggregate call.
    Aggregated(&'static str),
    /// `count(x) > 0` — a builtin call with no source-predicate target.
    BuiltinOnly,
}

/// Rule-document fixtures consumed by the Logic owner rails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceRuleFixtureV2 {
    /// One rule whose body calls nothing.
    SingleRule,
    /// One rule that positively depends on itself.
    PositiveSelfRecursion,
    /// One rule that negated-depends on itself.
    NegatedSelfRecursion,
    /// One rule that folds itself through an aggregate call.
    AggregatedSelfRecursion,
    /// Two rules with a mutual (r -> s, s -> r) dependency.
    MutualRecursionPair,
    /// Two rules with a single nonrecursive edge (r -> s).
    NonRecursiveDependency,
    /// Two rules with a single negated nonrecursive edge (r -> not s).
    NegatedNonRecursiveDependency,
    /// One rule whose only call is a builtin, never a graph vertex.
    BuiltinCallOnly,
}

/// Expression wrappers forged around one otherwise real nonrecursive rule
/// call. These fixtures exercise downstream closed-tree walkers over shapes
/// that cannot all be authored around a Bool-returning rule call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedRuleCallWrapperV2 {
    /// Unary expression wrapper.
    Unary,
    /// Binary expression wrapper, with the same call on both edges.
    Binary,
    /// Comparison expression wrapper.
    Compare,
    /// Logical disjunction wrapper.
    Or,
    /// Parenthesized expression wrapper.
    Paren,
    /// Path-base expression wrapper.
    Path,
    /// Record-field expression wrapper.
    Record,
    /// List-item expression wrapper.
    List,
    /// Logical-quantifier body wrapper.
    LogicalQuantifier,
}

/// A sealed rule document plus its Rule item identities in document order.
#[derive(Clone, Debug)]
pub struct ReferenceRuleDocumentV2 {
    document: SemanticDocumentV2,
    item_ids: Vec<SemanticItemIdV2>,
}

impl ReferenceRuleDocumentV2 {
    /// The sealed semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// The Rule item identities in document order.
    #[must_use]
    pub fn item_ids(&self) -> &[SemanticItemIdV2] {
        &self.item_ids
    }

    /// The document's own immutable downstream budget receipt.
    #[must_use]
    pub const fn budget(&self) -> DownstreamBudgetReceiptV2 {
        self.document.budget_receipt()
    }
}

fn wrap_rule_document_v2(document: SemanticDocumentV2) -> ReferenceRuleDocumentV2 {
    let item_ids = document
        .order()
        .iter()
        .copied()
        .filter(|id| matches!(document.item(*id), Some(SemanticItemV2::Rule(_))))
        .collect();
    ReferenceRuleDocumentV2 { document, item_ids }
}

fn rule_item_v2(
    builder: &HirBuilderV1,
    name: &str,
    shape: RuleBodyShapeV2,
    start: usize,
) -> Result<ProgramItemHirV1, String> {
    let head = builder.symbol("x", SpanV1::new(start + 9, start + 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(start + 14, start + 15)),
        builder.int("1", SpanV1::new(start + 18, start + 19)),
        SpanV1::new(start + 14, start + 19),
    );
    let source_call = |callee: &str, at: usize| -> Result<_, String> {
        Ok(builder.call(
            callee,
            SpanV1::new(at, at + 1),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                builder.symbol("x", SpanV1::new(at + 2, at + 3)),
            )])
            .map_err(|error| format!("{error:?}"))?,
            SpanV1::new(at, at + 4),
        ))
    };
    let body = match shape {
        RuleBodyShapeV2::AnchorOnly => anchor,
        RuleBodyShapeV2::Positive(callee) => {
            let call = source_call(callee, start + 24)?;
            builder.and(
                anchor,
                builder.bool_value(call, SpanV1::new(start + 24, start + 28)),
                SpanV1::new(start + 14, start + 28),
            )
        }
        RuleBodyShapeV2::Negated(callee) => {
            let call = source_call(callee, start + 24)?;
            builder.and(
                anchor,
                builder.not(
                    builder.bool_value(call, SpanV1::new(start + 24, start + 28)),
                    SpanV1::new(start + 24, start + 28),
                ),
                SpanV1::new(start + 14, start + 28),
            )
        }
        RuleBodyShapeV2::Aggregated(callee) => {
            let inner = source_call(callee, start + 40)?;
            let count = builder.call(
                "count",
                SpanV1::new(start + 34, start + 39),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)])
                    .map_err(|error| format!("{error:?}"))?,
                SpanV1::new(start + 34, start + 45),
            );
            let predicate = builder.compare(
                ">",
                count,
                builder.int("0", SpanV1::new(start + 48, start + 49)),
                SpanV1::new(start + 34, start + 49),
            );
            builder.and(anchor, predicate, SpanV1::new(start + 14, start + 49))
        }
        RuleBodyShapeV2::BuiltinOnly => {
            let builtin = builder.call(
                "count",
                SpanV1::new(start + 24, start + 29),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                    builder.int("1", SpanV1::new(start + 30, start + 31)),
                )])
                .map_err(|error| format!("{error:?}"))?,
                SpanV1::new(start + 24, start + 32),
            );
            let predicate = builder.compare(
                ">",
                builtin,
                builder.int("0", SpanV1::new(start + 34, start + 35)),
                SpanV1::new(start + 24, start + 35),
            );
            builder.and(anchor, predicate, SpanV1::new(start + 14, start + 35))
        }
    };
    Ok(ProgramItemHirV1::Rule(Box::new(builder.rule(
        name,
        SpanV1::new(start + 5, start + 6),
        head,
        body,
        SpanV1::new(start, start + 50),
    ))))
}

/// Seal one rule-document fixture through the real transaction.
pub fn reference_rule_document_v2(
    fixture: ReferenceRuleFixtureV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let checked = rule_checked_document_v2(fixture)?;
    let document = resolve_checked_document_v2(checked, &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok(wrap_rule_document_v2(document))
}

/// Forge one wrapper around the real `r -> s` call while preserving the real
/// source-qualified call identity, target, polarity, and checked location.
pub fn forged_rule_call_wrapper_document_v2(
    wrapper: ForgedRuleCallWrapperV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(ReferenceRuleFixtureV2::NonRecursiveDependency)?;
    let first = *base
        .item_ids
        .first()
        .ok_or("the nonrecursive rule fixture names its caller")?;
    let (root, call) = match base.document.item(first) {
        Some(SemanticItemV2::Rule(rule)) => (
            rule.body().expression().key(),
            rule.calls()
                .first()
                .ok_or("the nonrecursive rule fixture retains its call")?
                .call(),
        ),
        _ => return Err("the nonrecursive fixture caller is a rule".to_owned()),
    };
    let document = base.document.forged_expressions(move |expressions| {
        let Some(expression) = expressions.remove(&root) else {
            return;
        };
        let location = expression.location().clone();
        let TypedExprShapeV2::And { operator, .. } = expression.shape() else {
            expressions.insert(root, expression);
            return;
        };
        let operator = *operator;
        let shape = match wrapper {
            ForgedRuleCallWrapperV2::Unary => TypedExprShapeV2::Unary {
                operator,
                operand: call,
            },
            ForgedRuleCallWrapperV2::Binary => TypedExprShapeV2::Binary {
                operator,
                lhs: call,
                rhs: call,
            },
            ForgedRuleCallWrapperV2::Compare => TypedExprShapeV2::Compare {
                operator,
                lhs: call,
                rhs: call,
            },
            ForgedRuleCallWrapperV2::Or => TypedExprShapeV2::Or {
                operator,
                lhs: call,
                rhs: call,
            },
            ForgedRuleCallWrapperV2::Paren => TypedExprShapeV2::Paren { inner: call },
            ForgedRuleCallWrapperV2::Path => TypedExprShapeV2::Path {
                base: call,
                segments: vec![crate::TypedPathSegmentV2::new(
                    "field".to_owned(),
                    location.clone(),
                )],
            },
            ForgedRuleCallWrapperV2::Record => {
                let Ok(field) = crate::TypedRecordFieldV2::try_new("field", location, call) else {
                    expressions.insert(root, expression);
                    return;
                };
                TypedExprShapeV2::Record(vec![field])
            }
            ForgedRuleCallWrapperV2::List => TypedExprShapeV2::List(vec![call]),
            ForgedRuleCallWrapperV2::LogicalQuantifier => TypedExprShapeV2::LogicalQuantifier {
                kind: crate::LogicalQuantifierKindV2::Exists,
                binders: Vec::new(),
                body: call,
            },
        };
        expressions.insert(root, expression.forged_with_shape(shape));
    });
    Ok(wrap_rule_document_v2(document))
}

/// Author the checked HIR of one rule-document fixture. Both the resolving path
/// ([`reference_rule_document_v2`]) and the small-receipt re-sealing path
/// ([`forged_small_receipt_rule_document_v2`]) must author the identical node
/// sequence, so this is the one authoring site for the fixture family.
fn rule_checked_document_v2(
    fixture: ReferenceRuleFixtureV2,
) -> Result<CheckedSyntaxDocumentV2, String> {
    use ReferenceRuleFixtureV2 as Fixture;
    let specs: &[(&str, RuleBodyShapeV2, usize)] = match fixture {
        Fixture::SingleRule => &[("r", RuleBodyShapeV2::AnchorOnly, 0)],
        Fixture::PositiveSelfRecursion => &[("r", RuleBodyShapeV2::Positive("r"), 0)],
        Fixture::NegatedSelfRecursion => &[("r", RuleBodyShapeV2::Negated("r"), 0)],
        Fixture::AggregatedSelfRecursion => &[("r", RuleBodyShapeV2::Aggregated("r"), 0)],
        Fixture::MutualRecursionPair => &[
            ("r", RuleBodyShapeV2::Positive("s"), 0),
            ("s", RuleBodyShapeV2::Positive("r"), 60),
        ],
        Fixture::NonRecursiveDependency => &[
            ("r", RuleBodyShapeV2::Positive("s"), 0),
            ("s", RuleBodyShapeV2::AnchorOnly, 60),
        ],
        Fixture::NegatedNonRecursiveDependency => &[
            ("r", RuleBodyShapeV2::Negated("s"), 0),
            ("s", RuleBodyShapeV2::AnchorOnly, 60),
        ],
        Fixture::BuiltinCallOnly => &[("r", RuleBodyShapeV2::BuiltinOnly, 0)],
    };
    let owned = specs
        .iter()
        .map(|(name, shape, start)| ((*name).to_owned(), *shape, *start))
        .collect::<Vec<_>>();
    rule_checked_document_from_specs_v2(&owned)
}

/// Author a checked rule document from ordered `(name, body shape, start)`
/// specifications.
fn rule_checked_document_from_specs_v2(
    specs: &[(String, RuleBodyShapeV2, usize)],
) -> Result<CheckedSyntaxDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let mut items = Vec::new();
    for (name, shape, start) in specs {
        items.push(rule_item_v2(&builder, name, *shape, *start)?);
    }
    let source_len = specs
        .iter()
        .map(|(_, _, start)| start + 50)
        .max()
        .unwrap_or(50);
    CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items,
    })
    .map_err(|error| format!("{error:?}"))
}

/// Membership mutations minted from the mutual-recursion pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedRuleMembershipMutationV2 {
    /// The second Rule item is removed while its identity stays in `order`.
    DroppedItem,
    /// The second Rule item is kept while its identity leaves `order`.
    OrphanItem,
    /// The second Rule identity is authored twice in `order`.
    DuplicateOrderId,
}

/// Seal one membership-forged rule document.
pub fn forged_rule_membership_document_v2(
    mutation: ForgedRuleMembershipMutationV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)?;
    let second = *base
        .item_ids
        .get(1)
        .ok_or("the mutual-recursion fixture names two rules")?;
    let document = base
        .document
        .forged_membership(move |order, items| match mutation {
            ForgedRuleMembershipMutationV2::DroppedItem => {
                items.remove(&second);
            }
            ForgedRuleMembershipMutationV2::OrphanItem => {
                order.retain(|id| *id != second);
            }
            ForgedRuleMembershipMutationV2::DuplicateOrderId => {
                order.push(second);
            }
        });
    Ok(wrap_rule_document_v2(document))
}

/// Resolved-call mutations forged through the in-crate seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedRuleFactMutationV2 {
    /// The first call's target is a predicate no document member defines.
    NonLocalTarget,
    /// The first call's target disagrees with its own expression fact.
    TargetTransplanted,
    /// The first call's polarity disagrees with the proven body polarity.
    PolarityFlipped,
    /// The first call's node identity is absent from the expression facts.
    UnknownCallId,
    /// The first call's location comes from another registry identity.
    ForeignCallLocation,
    /// The rule loses the call edge its body still proves.
    DroppedCall,
    /// The rule duplicates the call edge it already carries.
    ExtraCall,
}

/// Seal one rule-fact-forged document.
pub fn forged_rule_fact_document_v2(
    fixture: ReferenceRuleFixtureV2,
    mutation: ForgedRuleFactMutationV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(fixture)?;
    let target_item = *base
        .item_ids
        .first()
        .ok_or("a rule-fact fixture names at least one rule")?;
    let foreign_location = foreign_call_location_v2()?;
    let document = base.document.forged_membership(move |_order, items| {
        let Some(SemanticItemV2::Rule(rule)) = items.remove(&target_item) else {
            return;
        };
        let original = rule.calls().to_vec();
        let Some(first) = original.first() else {
            items.insert(target_item, SemanticItemV2::Rule(rule));
            return;
        };
        let rewritten = match mutation {
            ForgedRuleFactMutationV2::NonLocalTarget => {
                let ghost = quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("ghost");
                match ghost {
                    Ok(ghost) => original
                        .iter()
                        .map(|call| {
                            replace_call_v2(
                                call,
                                quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(
                                    ghost.clone(),
                                ),
                                call.polarity(),
                                call.location().clone(),
                            )
                        })
                        .collect(),
                    Err(_mutation_error) => original.clone(),
                }
            }
            ForgedRuleFactMutationV2::TargetTransplanted => {
                let own = crate::ResolvedRuleV2::predicate_id(&rule).clone();
                vec![replace_call_v2(
                    first,
                    quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(own),
                    first.polarity(),
                    first.location().clone(),
                )]
            }
            ForgedRuleFactMutationV2::PolarityFlipped => vec![replace_call_v2(
                first,
                first.target().clone(),
                crate::RuleCallPolarityV2::Negated,
                first.location().clone(),
            )],
            ForgedRuleFactMutationV2::UnknownCallId => vec![replace_call_v2(
                first,
                first.target().clone(),
                first.polarity(),
                first.location().clone(),
            )
            .with_call_node_v2(SourceQualifiedNodeIdV2 {
                source_id: REFERENCE_SOURCE_V2,
                node_id: HirNodeIdV1::from_raw(u32::MAX),
            })],
            ForgedRuleFactMutationV2::ForeignCallLocation => vec![replace_call_v2(
                first,
                first.target().clone(),
                first.polarity(),
                foreign_location.clone(),
            )],
            ForgedRuleFactMutationV2::DroppedCall => Vec::new(),
            ForgedRuleFactMutationV2::ExtraCall => vec![first.clone(), first.clone()],
        };
        items.insert(
            target_item,
            SemanticItemV2::Rule(rule.forged_with_calls(rewritten)),
        );
    });
    Ok(wrap_rule_document_v2(document))
}

fn replace_call_v2(
    call: &crate::ResolvedRuleCallV2,
    target: quanta_gqlang_v2_types::ResolvedCallTargetV2,
    polarity: crate::RuleCallPolarityV2,
    location: CheckedLocatedSpanV2,
) -> crate::ResolvedRuleCallV2 {
    crate::ResolvedRuleCallV2::new(call.call(), target, polarity, location)
}

trait WithCallNodeV2 {
    fn with_call_node_v2(self, node: SourceQualifiedNodeIdV2) -> Self;
}

impl WithCallNodeV2 for crate::ResolvedRuleCallV2 {
    fn with_call_node_v2(self, node: SourceQualifiedNodeIdV2) -> Self {
        let target = self.target().clone();
        let polarity = self.polarity();
        let location = self.location().clone();
        crate::ResolvedRuleCallV2::new(node, target, polarity, location)
    }
}

fn foreign_call_location_v2() -> Result<CheckedLocatedSpanV2, String> {
    capture_edge_registry()?
        .try_check_location(LocatedSpanV2 {
            source_id: REFERENCE_SOURCE_V2,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))
}

// ---- meta fixtures ---------------------------------------------------------

/// Canonical source text of [`ReferenceMetaFixtureV2::UsePack`].
pub const REFERENCE_USE_PACK_SOURCE_V2: &str =
    "use pack acme.pack | activation=session | lockfile=acme.lock;";

/// Pack identity authored by [`ReferenceMetaFixtureV2::UsePack`].
pub const REFERENCE_USE_PACK_ID_V2: &str = "acme.pack";

/// Lockfile authored by [`ReferenceMetaFixtureV2::UsePack`].
pub const REFERENCE_USE_PACK_LOCKFILE_V2: &str = "acme.lock";

/// Canonical source text of [`ReferenceMetaFixtureV2::RequireCertified`].
pub const REFERENCE_REQUIRE_CERTIFIED_SOURCE_V2: &str = "require certified;";

/// Payload family of one reference meta-directive document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceMetaFixtureV2 {
    /// `USE PACK acme.pack | activation=session | lockfile=acme.lock`.
    UsePack,
    /// `REQUIRE CERTIFIED`.
    RequireCertified,
}

/// A sealed meta-directive document plus its one Meta item identity.
#[derive(Clone, Debug)]
pub struct ReferenceMetaDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferenceMetaDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the sole Meta item.
    #[must_use]
    pub const fn meta_item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The resolved directive payload of that item.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Meta item is unreachable"
    )]
    pub fn kind(&self) -> &ResolvedMetaKindV2 {
        match self.document.item(self.item_id) {
            Some(SemanticItemV2::Meta(meta)) => meta.kind(),
            _ => panic!("meta fixture item is a resolved meta directive"),
        }
    }
}

fn meta_hir_v2(builder: &HirBuilderV1, fixture: ReferenceMetaFixtureV2) -> (MetaHirV1, usize) {
    match fixture {
        ReferenceMetaFixtureV2::RequireCertified => (
            builder.meta(MetaKindV1::RequireCertified, SpanV1::new(0, 17)),
            18,
        ),
        ReferenceMetaFixtureV2::UsePack => {
            let source_len = REFERENCE_USE_PACK_SOURCE_V2.len();
            let meta = builder.meta(
                MetaKindV1::UsePack(UsePackHirV1 {
                    pack_id: REFERENCE_USE_PACK_ID_V2.to_owned(),
                    activation: UsePackActivationV1::Session,
                    lockfile: REFERENCE_USE_PACK_LOCKFILE_V2.to_owned(),
                }),
                SpanV1::new(0, source_len - 1),
            );
            (meta, source_len)
        }
    }
}

fn meta_checked_document_v2(
    fixture: ReferenceMetaFixtureV2,
) -> Result<(CheckedSyntaxDocumentV2, SemanticItemIdV2), String> {
    let builder = HirBuilderV1::new();
    let (meta, source_len) = meta_hir_v2(&builder, fixture);
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, meta.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Meta(Box::new(meta))],
    })
    .map_err(|error| format!("{error:?}"))?;
    Ok((checked, item_id))
}

/// Resolve one reference meta-directive fixture through the real semantic
/// transaction.
///
/// `RequireCertified` seals a document holding exactly one
/// [`ResolvedMetaKindV2::RequireCertified`] item, whose directive is also the
/// document catalog's validated certification requirement.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_meta_document_v2(
    kind: ReferenceMetaFixtureV2,
) -> Result<ReferenceMetaDocumentV2, String> {
    let (checked, item_id) = meta_checked_document_v2(kind)?;
    let document = resolve_checked_document_v2(checked, &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceMetaDocumentV2 { document, item_id })
}

/// Compatibility test helper for the former forged USE PACK fixture.
///
/// The producer path is now implemented, so this helper deliberately delegates
/// to the real transaction instead of bypassing Modules validation.
///
/// # Errors
///
/// Returns a fixture-construction error if the reassembled document no longer
/// satisfies the assembly contract.
pub fn forged_use_pack_meta_document_v2() -> Result<ReferenceMetaDocumentV2, String> {
    reference_meta_document_v2(ReferenceMetaFixtureV2::UsePack)
}

// ---- presentation fixtures -------------------------------------------------

/// Entity selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2: &str = "input_a";

/// DESCRIBE subtype admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2: &str = "entity";

/// COMPARE baseline selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2: &str = "baseline_a";

/// COMPARE candidate selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2: &str = "candidate_b";

/// EXPORT source selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2: &str = "export_source";

/// Source length of every reference presentation fixture document.
pub const REFERENCE_PRESENTATION_SOURCE_LEN_V2: usize = 64;

/// Positive SAMPLE limit authored by the reference SAMPLE presentation.
pub const REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2: u32 = 5;

/// One reference presentation verb.
///
/// `Explain` carries the exact evidence/truth pair, so all twelve permutations
/// are reachable, including the two incoherent pairs planning refuses
/// (`Counterexample` + `Confirm`, `Citation` + `Refute`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferencePresentationFixtureV2 {
    /// EXPLAIN one admitted entity selector.
    Explain {
        /// Authored evidence mode.
        evidence: EvidenceModeV1,
        /// Authored truth mode.
        truth: TruthModeV1,
    },
    /// SHOW capabilities.
    Show,
    /// DESCRIBE the admitted `entity` subtype of the entity selector.
    Describe,
    /// SAMPLE with a positive limit and no scope.
    Sample,
    /// COMPARE two admitted snapshot selectors.
    Compare,
    /// EXPORT the admitted export source as SARIF.
    Export,
}

/// A sealed presentation document plus its one Presentation item identity.
#[derive(Clone, Debug)]
pub struct ReferencePresentationDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferencePresentationDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the sole Presentation item.
    #[must_use]
    pub const fn presentation_item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The resolved presentation contract.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Presentation item is unreachable"
    )]
    pub fn presentation(&self) -> &ResolvedPresentationV2 {
        match self.document.item(self.item_id) {
            Some(SemanticItemV2::Presentation(presentation)) => presentation,
            _ => panic!("presentation fixture item is a resolved presentation"),
        }
    }
}

fn reference_graph_selector_v2(name: &str, start: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.to_owned(),
        span: SpanV1::new(start, start + name.len()),
    }
}

fn reference_presentation_hir(
    builder: &HirBuilderV1,
    fixture: ReferencePresentationFixtureV2,
) -> (PresentationHirV1, usize) {
    let source_len = REFERENCE_PRESENTATION_SOURCE_LEN_V2;
    let item_span = |end: usize| SpanV1::new(0, end);
    let kind = match fixture {
        ReferencePresentationFixtureV2::Explain { evidence, truth } => {
            PresentationKindV1::Explain {
                target_ref: reference_graph_selector_v2(
                    REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2,
                    8,
                ),
                evidence,
                truth,
            }
        }
        ReferencePresentationFixtureV2::Show => PresentationKindV1::Show {
            category: ShowCategoryV1::Capabilities,
        },
        ReferencePresentationFixtureV2::Describe => PresentationKindV1::Describe {
            kind: REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2.to_owned(),
            path: reference_graph_selector_v2(REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2, 17),
        },
        ReferencePresentationFixtureV2::Sample => PresentationKindV1::Sample {
            query_text: "auth".to_owned(),
            mode: SampleModeV1::Lexical,
            limit: REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2,
            scope: None,
        },
        ReferencePresentationFixtureV2::Compare => PresentationKindV1::Compare {
            baseline_ref: reference_graph_selector_v2(
                REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2,
                17,
            ),
            candidate_ref: reference_graph_selector_v2(
                REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2,
                30,
            ),
            metric: CompareMetricV1::Structural,
        },
        ReferencePresentationFixtureV2::Export => PresentationKindV1::Export {
            source_ref: reference_graph_selector_v2(REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2, 14),
            format: ExportFormatV1::Sarif,
            profile: None,
        },
    };
    let end = match fixture {
        ReferencePresentationFixtureV2::Explain { .. } => 24,
        ReferencePresentationFixtureV2::Show => 16,
        ReferencePresentationFixtureV2::Describe => 25,
        ReferencePresentationFixtureV2::Sample => 40,
        ReferencePresentationFixtureV2::Compare => 42,
        ReferencePresentationFixtureV2::Export => 26,
    };
    (builder.presentation(kind, item_span(end)), source_len)
}

fn presentation_authority_v2() -> Result<ClosedFrontendAuthorityV2, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    graph_authority_with_graph_inputs_v2(false, budget, |host, _location| {
        let mut inputs = Vec::<GraphReferenceInputV2>::new();
        for (index, (spelling, kind, subtype)) in [
            (
                REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2,
                GraphReferenceKindV2::Entity,
                Some(REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2),
            ),
            (
                REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2,
                GraphReferenceKindV2::Snapshot,
                None,
            ),
            (
                REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2,
                GraphReferenceKindV2::Snapshot,
                None,
            ),
            (
                REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2,
                GraphReferenceKindV2::ExportSource,
                None,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let span = SpanV1::new(32 + index, 33 + index);
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: spelling.to_owned(),
                kind,
                entity_subtype: subtype.map(str::to_owned),
                location: declaration,
            });
        }
        Ok(inputs)
    })
}

/// Resolve one reference presentation fixture through the real semantic
/// transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_presentation_document_v2(
    fixture: ReferencePresentationFixtureV2,
) -> Result<ReferencePresentationDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let (presentation, source_len) = reference_presentation_hir(&builder, fixture);
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, presentation.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Presentation(Box::new(presentation))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document = resolve_checked_document_v2(checked, &presentation_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ReferencePresentationDocumentV2 { document, item_id })
}

// ---- composition fixtures --------------------------------------------------

/// Source length of every reference composition fixture document.
pub const REFERENCE_COMPOSITION_SOURCE_LEN_V2: usize = 256;

/// Entity selector admitted for the reference composition FIND producers.
pub const REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2: &str = "input_a";

/// Sink selector admitted for the reference composition FIND producers.
pub const REFERENCE_COMPOSITION_SINK_SELECTOR_V2: &str = "exec_b";

/// Pattern selector admitted for the reference composition CHECK body.
pub const REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2: &str = "pattern_a";

/// Rule identity admitted for the reference composition CHECK body.
pub const REFERENCE_COMPOSITION_RULE_V2: &str = "rule1";

/// Closed body family of one reference composition document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceCompositionBodyV2 {
    /// A relational working-set body over the ordered LET producers.
    Relational,
    /// A FIND body.
    Graph,
    /// A CHECK body.
    Check,
    /// An EXPLAIN body over a prior FIND producer.
    Explain,
}

/// A sealed composition document plus the facts the planner re-checks.
#[derive(Clone, Debug)]
pub struct ReferenceCompositionDocumentV2 {
    document: SemanticDocumentV2,
    composition_item_id: SemanticItemIdV2,
    body_item_id: SemanticItemIdV2,
    producer_item_ids: Vec<SemanticItemIdV2>,
    with_binding: ResolvedValueBindingV2,
}

impl ReferenceCompositionDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the composition root item.
    #[must_use]
    pub const fn composition_item_id(&self) -> SemanticItemIdV2 {
        self.composition_item_id
    }

    /// Identity of the sealed body item.
    #[must_use]
    pub const fn body_item_id(&self) -> SemanticItemIdV2 {
        self.body_item_id
    }

    /// Ordered producer identities, in authored LET order.
    #[must_use]
    pub fn producer_item_ids(&self) -> &[SemanticItemIdV2] {
        &self.producer_item_ids
    }

    /// The exact `WITH` binding identity, which equals exactly one producer's.
    #[must_use]
    pub const fn with_binding(&self) -> &ResolvedValueBindingV2 {
        &self.with_binding
    }

    /// The resolved composition contract itself.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Composition item is unreachable"
    )]
    pub fn composition(&self) -> &ResolvedCompositionV2 {
        match self.document.item(self.composition_item_id) {
            Some(SemanticItemV2::Composition(composition)) => composition,
            _ => panic!("composition fixture item is a resolved composition"),
        }
    }
}

fn composition_sample_producer(
    builder: &HirBuilderV1,
    name: &str,
    name_start: usize,
    producer_start: usize,
    query_text: &str,
    limit: u32,
) -> CompositionBindingV1 {
    let sample = SampleHirV1 {
        id: builder.next_id(),
        span: SpanV1::new(producer_start, producer_start + 36),
        query_text: query_text.to_owned(),
        mode: SampleModeV1::Lexical,
        limit,
        scope: None,
    };
    builder.composition_binding(
        name,
        SpanV1::new(name_start, name_start + name.len()),
        CompositionProducerV1::Sample(Box::new(sample)),
    )
}

fn composition_find_query(
    builder: &HirBuilderV1,
    item_span: SpanV1,
    source_span: SpanV1,
    sink_span: SpanV1,
) -> GraphQueryHirV1 {
    let endpoints = GraphEndpointsV1 {
        source: Some(GraphRefV1 {
            name: REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2.to_owned(),
            span: source_span,
        }),
        sink: Some(GraphRefV1 {
            name: REFERENCE_COMPOSITION_SINK_SELECTOR_V2.to_owned(),
            span: sink_span,
        }),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(1),
        ..GraphConstraintsV1::default()
    };
    builder.graph_query(GraphQueryKindV1::Path, endpoints, constraints, item_span)
}

fn composition_find_producer(
    builder: &HirBuilderV1,
    name: &str,
    name_start: usize,
    item_span: SpanV1,
    source_span: SpanV1,
    sink_span: SpanV1,
) -> CompositionBindingV1 {
    let query = composition_find_query(builder, item_span, source_span, sink_span);
    builder.composition_binding(
        name,
        SpanV1::new(name_start, name_start + name.len()),
        CompositionProducerV1::GraphQuery(Box::new(query)),
    )
}

fn reference_composition_hir(
    builder: &HirBuilderV1,
    body: ReferenceCompositionBodyV2,
) -> Result<CompositionHirV1, String> {
    let composition = match body {
        ReferenceCompositionBodyV2::Relational => {
            let sampled = composition_sample_producer(builder, "sampled", 4, 14, "auth", 2);
            let sampled_again = composition_sample_producer(builder, "sampled2", 61, 71, "docs", 3);
            let scan = builder.relation_source(
                "sampled",
                SpanV1::new(125, 132),
                Some(alias("s", 133, 134)),
                SpanV1::new(125, 134),
            );
            let value = field_path(builder, "s", 152, "path", 154)?;
            let project = builder
                .try_project(
                    scan,
                    vec![ProjectionItemV1 {
                        value,
                        alias: Some(alias("path", 168, 172)),
                    }],
                    SpanV1::new(125, 172),
                )
                .map_err(|error| error.to_owned())?;
            let query = builder.query(project, None, SpanV1::new(120, 172));
            builder.composition(
                vec![sampled, sampled_again],
                "sampled",
                SpanV1::new(240, 247),
                CompositionBodyV1::Query(Box::new(query)),
                SpanV1::new(0, 247),
            )
        }
        ReferenceCompositionBodyV2::Graph => {
            let paths = composition_find_producer(
                builder,
                "paths",
                4,
                SpanV1::new(12, 70),
                SpanV1::new(24, 31),
                SpanV1::new(36, 42),
            );
            let body = composition_find_query(
                builder,
                SpanV1::new(80, 140),
                SpanV1::new(92, 99),
                SpanV1::new(104, 110),
            );
            builder.composition(
                vec![paths],
                "paths",
                SpanV1::new(240, 245),
                CompositionBodyV1::GraphQuery(Box::new(body)),
                SpanV1::new(0, 245),
            )
        }
        ReferenceCompositionBodyV2::Check => {
            let sampled = composition_sample_producer(builder, "sampled", 4, 14, "auth", 2);
            let check = builder.legality(
                GraphRefV1 {
                    name: REFERENCE_COMPOSITION_RULE_V2.to_owned(),
                    span: SpanV1::new(86, 91),
                },
                None,
                Some(GraphRefV1 {
                    name: REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2.to_owned(),
                    span: SpanV1::new(110, 119),
                }),
                ConfirmModeV1::Skip,
                EmitModeV1::Findings,
                DetectionModeV1::PatternOnly,
                SpanV1::new(80, 200),
            );
            builder.composition(
                vec![sampled],
                "sampled",
                SpanV1::new(240, 247),
                CompositionBodyV1::Legality(Box::new(check)),
                SpanV1::new(0, 247),
            )
        }
        ReferenceCompositionBodyV2::Explain => {
            let paths = composition_find_producer(
                builder,
                "paths",
                4,
                SpanV1::new(12, 70),
                SpanV1::new(24, 31),
                SpanV1::new(36, 42),
            );
            let explain = ExplainHirV1 {
                id: builder.next_id(),
                span: SpanV1::new(80, 140),
                target_ref: GraphRefV1 {
                    name: "paths".to_owned(),
                    span: SpanV1::new(88, 93),
                },
                evidence: EvidenceModeV1::Auto,
                truth: TruthModeV1::Inspect,
            };
            builder.composition(
                vec![paths],
                "paths",
                SpanV1::new(240, 245),
                CompositionBodyV1::Explain(Box::new(explain)),
                SpanV1::new(0, 245),
            )
        }
    };
    Ok(composition)
}

fn composition_authority_v2(
    body: ReferenceCompositionBodyV2,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let with_rule = body == ReferenceCompositionBodyV2::Check;
    graph_authority_with_graph_inputs_v2(with_rule, budget, |host, _location| {
        let mut admissions: Vec<(&str, GraphReferenceKindV2, Option<&str>)> = Vec::new();
        if body != ReferenceCompositionBodyV2::Relational {
            admissions.push((
                REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2,
                GraphReferenceKindV2::Entity,
                None,
            ));
            admissions.push((
                REFERENCE_COMPOSITION_SINK_SELECTOR_V2,
                GraphReferenceKindV2::Entity,
                None,
            ));
        }
        if body == ReferenceCompositionBodyV2::Check {
            admissions.push((
                REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2,
                GraphReferenceKindV2::Pattern,
                None,
            ));
        }
        let mut inputs = Vec::<GraphReferenceInputV2>::new();
        for (index, (spelling, kind, subtype)) in admissions.into_iter().enumerate() {
            let span = SpanV1::new(64 + index, 65 + index);
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: spelling.to_owned(),
                kind,
                entity_subtype: subtype.map(str::to_owned),
                location: declaration,
            });
        }
        Ok(inputs)
    })
}

/// Resolve one reference composition fixture through the real semantic
/// transaction.
///
/// The `Relational` body is the pipeline-proven working-set shape: an ordered
/// SAMPLE producer named by `WITH`, consumed by a body query over that working
/// set. Every body family also carries at least one ordered producer whose
/// sealed relation output the planner retains.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_composition_document_v2(
    body: ReferenceCompositionBodyV2,
) -> Result<ReferenceCompositionDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let composition = reference_composition_hir(&builder, body)?;
    let composition_item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, composition.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len: REFERENCE_COMPOSITION_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::Composition(Box::new(composition))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document = resolve_checked_document_v2(checked, &composition_authority_v2(body)?)
        .map_err(|error| format!("{error:?}"))?;
    let (body_item_id, producer_item_ids, with_binding) = match document.item(composition_item_id) {
        Some(SemanticItemV2::Composition(composition)) => (
            composition.body().item_id(),
            composition
                .bindings()
                .iter()
                .map(|binding| binding.producer().item_id())
                .collect::<Vec<_>>(),
            composition.with_target().clone(),
        ),
        _ => return Err("composition fixture did not seal its composition item".to_owned()),
    };
    Ok(ReferenceCompositionDocumentV2 {
        document,
        composition_item_id,
        body_item_id,
        producer_item_ids,
        with_binding,
    })
}

// ---- small-receipt rule documents ------------------------------------------

/// Exact `max_semantic_steps` charge of a document rule program with
/// `member_count` members: one program charge plus one charge per member, so
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] admits exactly seven members and refuses
/// the eighth.
#[must_use]
pub const fn reference_rule_program_charge_v2(member_count: u32) -> u32 {
    member_count.saturating_add(1)
}

/// Member count whose program charge exactly equals
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`].
pub const SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS: u32 = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;

/// Member count whose program charge is exactly one step over
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`].
pub const SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS: u32 = SMALL_BUDGET_MAX_SEMANTIC_STEPS;

/// Re-seal an already-resolved rule document under the small step receipt.
fn reseal_rule_document_with_receipt_v2(
    checked: &CheckedSyntaxDocumentV2,
    reference: &SemanticDocumentV2,
    receipt: DownstreamBudgetReceiptV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let (items, expressions, nested) = document_parts(reference);
    let document = reassemble_document(
        checked,
        &reference_authority_v2()?,
        items,
        expressions,
        nested,
        receipt,
    )?;
    Ok(wrap_rule_document_v2(document))
}

/// Seal one rule-document fixture under a receipt whose `max_semantic_steps` is
/// exactly [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] while every other dimension stays
/// standard.
///
/// The members, identities, and expression facts are the already-resolved
/// fixture's, so the rule-program walk charges exactly
/// [`reference_rule_program_charge_v2`] steps for the fixture's member count.
/// The fixed fixtures carry at most two members (three steps), so use
/// [`forged_small_receipt_rule_program_document_v2`] for the exact/over pair at
/// the sealed limit.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_small_receipt_rule_document_v2(
    fixture: ReferenceRuleFixtureV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let checked = rule_checked_document_v2(fixture)?;
    let reference = reference_rule_document_v2(fixture)?;
    reseal_rule_document_with_receipt_v2(&checked, reference.document(), small_steps_receipt()?)
}

/// Seal an `member_count`-member anchor-only rule program under the same small
/// step receipt.
///
/// [`SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS`] members charge exactly
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] steps and plan;
/// [`SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS`] members charge one step more and
/// are refused at the first uncharged member.
///
/// # Errors
///
/// Returns a fixture-construction error when `member_count` is zero or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_small_receipt_rule_program_document_v2(
    member_count: u32,
) -> Result<ReferenceRuleDocumentV2, String> {
    if member_count == 0 {
        return Err("a rule program fixture names at least one rule".to_owned());
    }
    let specs = (0..member_count)
        .map(|index| {
            let ordinal = usize::try_from(index).map_err(|_error| "member overflow".to_owned())?;
            Ok((
                format!("r{index}"),
                RuleBodyShapeV2::AnchorOnly,
                ordinal * 60,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let checked = rule_checked_document_from_specs_v2(&specs)?;
    let document = resolve_checked_document_v2(checked.clone(), &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    reseal_rule_document_with_receipt_v2(&checked, &document, small_steps_receipt()?)
}

// ---- narrow-witness and alternate-identity graph documents -----------------

/// `max_witnesses` of every narrow-witness graph fixture.
pub const NARROW_WITNESS_MAX_WITNESSES_V2: u32 = 1;

/// Node identity used by [`reference_alternate_graph_document_v2`]; distinct
/// from the [`HirNodeIdV1::from_raw`]`(2)` every reference FIND fixture seals.
pub const ALTERNATE_GRAPH_NODE_ID_V2: u32 = 5_001;

/// The ordered reference set one FIND subkind legally requires, mirroring the
/// consumer-side request builders so a narrow-witness document is a drop-in
/// replacement for a standard-budget one.
#[must_use]
pub fn reference_witness_graph_references_v2(
    kind: GraphQueryKindV1,
) -> Vec<ReferenceGraphReferenceV2> {
    use GraphReferenceRoleV2 as Role;
    let reference =
        |role: Role, selector: &'static str, span: (usize, usize)| ReferenceGraphReferenceV2 {
            role,
            selector,
            span,
        };
    match kind {
        GraphQueryKindV1::Reachability
        | GraphQueryKindV1::Flow
        | GraphQueryKindV1::Taint
        | GraphQueryKindV1::Path => vec![
            reference(Role::Source, "input_a", (20, 27)),
            reference(Role::Sink, "exec_b", (30, 36)),
        ],
        GraphQueryKindV1::Slice => vec![reference(Role::Anchor, "input_a", (20, 27))],
        GraphQueryKindV1::PointsTo => vec![
            reference(Role::Reference, "input_a", (20, 27)),
            reference(Role::Pointee, "exec_b", (30, 36)),
        ],
        GraphQueryKindV1::Summary => vec![reference(Role::Callable, "input_a", (20, 27))],
    }
}

fn narrow_witness_budget_v2() -> Result<FrontendBudgetV2, String> {
    let standard = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    FrontendBudgetV2::try_new(
        standard.max_source_bytes(),
        standard.max_frame_bytes(),
        standard.max_tokens(),
        standard.max_document_items(),
        standard.max_expression_depth(),
        standard.max_relation_depth(),
        standard.max_expression_nodes(),
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
        NARROW_WITNESS_MAX_WITNESSES_V2,
        standard.max_diagnostics(),
        standard.max_response_bytes(),
    )
    .map_err(|error| format!("{error:?}"))
}

/// Resolve one reference FIND fixture under a budget whose `max_witnesses` is
/// exactly [`NARROW_WITNESS_MAX_WITNESSES_V2`]; every other dimension is the
/// standard budget's. The sealed FIND retains those exact witness bounds, so a
/// consumer derives the canonical-one witness selection from the document
/// itself rather than choosing it.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixture no longer satisfies the
/// semantic contract.
pub fn reference_narrow_witness_graph_document_v2(
    kind: GraphQueryKindV1,
    depth: u32,
) -> Result<ReferenceGraphDocumentV2, String> {
    let request = ReferenceGraphRequestV2 {
        kind,
        references: reference_witness_graph_references_v2(kind),
        depth: Some(depth),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    };
    let query = graph_hir_query(&request)?;
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let authority =
        graph_authority_with_budget_v2(&request.references, false, narrow_witness_budget_v2()?)?;
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document =
        resolve_checked_document_v2(checked, &authority).map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

/// Resolve one reference FIND fixture whose graph item identity is
/// [`ALTERNATE_GRAPH_NODE_ID_V2`] instead of the reference node `2`.
///
/// A consumer pairs this document's sealed graph contract with another
/// document's resolved CHECK to reach
/// `planning.invalid_check.nested_graph_identity_mismatch` end to end.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixture no longer satisfies the
/// semantic contract.
pub fn reference_alternate_graph_document_v2(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceGraphDocumentV2, String> {
    let query =
        graph_hir_query_with_node_v2(&request, HirNodeIdV1::from_raw(ALTERNATE_GRAPH_NODE_ID_V2))?;
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let authority = graph_authority_v2(&request.references, false)?;
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document =
        resolve_checked_document_v2(checked, &authority).map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

/// Resolve the standard Taint FIND under
/// [`SECONDARY_REFERENCE_SOURCE_V2`]. Its graph shape matches the primary
/// reference fixture while every semantic identity is source-qualified by the
/// second document.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed FIND no longer seals.
pub fn second_source_graph_document_v2() -> Result<ReferenceGraphDocumentV2, String> {
    let kind = GraphQueryKindV1::Taint;
    let request = ReferenceGraphRequestV2 {
        kind,
        references: reference_witness_graph_references_v2(kind),
        depth: Some(4),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    };
    let query = graph_hir_query(&request)?;
    let item_id = SemanticItemIdV2::new(SECONDARY_REFERENCE_SOURCE_V2, query.id);
    let authority = graph_authority_v2(&request.references, false)?;
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SECONDARY_REFERENCE_SOURCE_V2,
        source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document =
        resolve_checked_document_v2(checked, &authority).map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

// ---- W4-D fixture units ----------------------------------------------------

#[cfg(test)]
mod w4d_tests {
    #![expect(
        clippy::indexing_slicing,
        clippy::panic_in_result_fn,
        clippy::wildcard_enum_match_arm,
        reason = "owner assertions pin exact fixture shapes, index pinned producer vectors, and \
                  name the payload vocabulary they reject"
    )]

    use super::*;

    // ---- meta ----------------------------------------------------------

    #[test]
    fn require_certified_meta_document_seals_exactly_one_directive() -> Result<(), String> {
        let document = reference_meta_document_v2(ReferenceMetaFixtureV2::RequireCertified)?;
        assert_eq!(document.document().items().count(), 1);
        assert!(matches!(
            document.kind(),
            ResolvedMetaKindV2::RequireCertified
        ));
        assert!(matches!(
            document.document().item(document.meta_item_id()),
            Some(SemanticItemV2::Meta(_))
        ));
        assert_eq!(document.document().order(), [document.meta_item_id()]);
        Ok(())
    }

    #[test]
    fn use_pack_meta_document_seals_through_the_real_transaction() -> Result<(), String> {
        let document = reference_meta_document_v2(ReferenceMetaFixtureV2::UsePack)?;
        match document.kind() {
            ResolvedMetaKindV2::UsePack { pack, activation } => {
                assert_eq!(pack.as_str(), REFERENCE_USE_PACK_ID_V2);
                assert_eq!(*activation, UsePackActivationV1::Session);
            }
            ResolvedMetaKindV2::RequireCertified => {
                return Err("the fixture must retain USE PACK".to_owned())
            }
        }
        Ok(())
    }

    #[test]
    fn forged_use_pack_meta_document_seals_the_resolved_use_pack_kind() -> Result<(), String> {
        let document = forged_use_pack_meta_document_v2()?;
        match document.kind() {
            ResolvedMetaKindV2::UsePack { pack, activation } => {
                assert_eq!(pack.as_str(), REFERENCE_USE_PACK_ID_V2);
                assert_eq!(*activation, UsePackActivationV1::Session);
            }
            ResolvedMetaKindV2::RequireCertified => {
                return Err("the forged fixture seals a USE PACK payload".to_owned())
            }
        }
        assert_eq!(document.document().items().count(), 1);
        assert_eq!(document.document().order(), [document.meta_item_id()]);
        Ok(())
    }

    // ---- presentation --------------------------------------------------

    #[test]
    fn every_presentation_fixture_resolves() -> Result<(), String> {
        for fixture in [
            ReferencePresentationFixtureV2::Show,
            ReferencePresentationFixtureV2::Describe,
            ReferencePresentationFixtureV2::Sample,
            ReferencePresentationFixtureV2::Compare,
            ReferencePresentationFixtureV2::Export,
        ] {
            let document = reference_presentation_document_v2(fixture)?;
            assert_eq!(document.document().items().count(), 1);
            assert_eq!(
                document.presentation().item_id(),
                document.presentation_item_id()
            );
        }
        Ok(())
    }

    #[test]
    fn every_explain_evidence_truth_permutation_resolves() -> Result<(), String> {
        const EVIDENCE: [EvidenceModeV1; 4] = [
            EvidenceModeV1::Auto,
            EvidenceModeV1::Trace,
            EvidenceModeV1::Citation,
            EvidenceModeV1::Counterexample,
        ];
        const TRUTH: [TruthModeV1; 3] = [
            TruthModeV1::Inspect,
            TruthModeV1::Confirm,
            TruthModeV1::Refute,
        ];
        let mut seen = 0_u8;
        for evidence in EVIDENCE {
            for truth in TRUTH {
                let document =
                    reference_presentation_document_v2(ReferencePresentationFixtureV2::Explain {
                        evidence,
                        truth,
                    })?;
                match document.presentation().kind() {
                    crate::ResolvedPresentationKindV2::Explain {
                        evidence: sealed_evidence,
                        truth: sealed_truth,
                        ..
                    } => {
                        assert_eq!(*sealed_evidence, evidence);
                        assert_eq!(*sealed_truth, truth);
                    }
                    other => return Err(format!("EXPLAIN payload expected, got {other:?}")),
                }
                seen += 1;
            }
        }
        assert_eq!(seen, 12, "every evidence/truth permutation is reachable");
        Ok(())
    }

    #[test]
    fn presentation_payloads_retain_their_exact_facts() -> Result<(), String> {
        let show = reference_presentation_document_v2(ReferencePresentationFixtureV2::Show)?;
        assert!(matches!(
            show.presentation().kind(),
            crate::ResolvedPresentationKindV2::Show {
                category: ShowCategoryV1::Capabilities
            }
        ));

        let describe =
            reference_presentation_document_v2(ReferencePresentationFixtureV2::Describe)?;
        assert!(matches!(
            describe.presentation().kind(),
            crate::ResolvedPresentationKindV2::Describe { .. }
        ));

        let sample = reference_presentation_document_v2(ReferencePresentationFixtureV2::Sample)?;
        assert!(matches!(
            sample.presentation().kind(),
            crate::ResolvedPresentationKindV2::Sample { limit, .. }
                if *limit == REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2
        ));

        let compare = reference_presentation_document_v2(ReferencePresentationFixtureV2::Compare)?;
        assert!(matches!(
            compare.presentation().kind(),
            crate::ResolvedPresentationKindV2::Compare {
                metric: CompareMetricV1::Structural,
                ..
            }
        ));

        let export = reference_presentation_document_v2(ReferencePresentationFixtureV2::Export)?;
        assert!(matches!(
            export.presentation().kind(),
            crate::ResolvedPresentationKindV2::Export {
                format: ExportFormatV1::Sarif,
                ..
            }
        ));
        Ok(())
    }

    // ---- composition ---------------------------------------------------

    #[test]
    fn every_composition_body_fixture_resolves_with_ordered_producers() -> Result<(), String> {
        for body in [
            ReferenceCompositionBodyV2::Relational,
            ReferenceCompositionBodyV2::Graph,
            ReferenceCompositionBodyV2::Check,
            ReferenceCompositionBodyV2::Explain,
        ] {
            let document = reference_composition_document_v2(body)?;
            assert_eq!(
                document.document().order(),
                [document.composition_item_id()]
            );
            assert!(!document.producer_item_ids().is_empty());
            assert_ne!(document.body_item_id(), document.composition_item_id());
            let composition = document.composition();
            assert_eq!(composition.item_id(), document.composition_item_id());
            assert_eq!(composition.body().item_id(), document.body_item_id());
            assert_eq!(composition.with_target(), document.with_binding());
            let selected = composition
                .bindings()
                .iter()
                .filter(|binding| binding.binding() == document.with_binding())
                .count();
            assert_eq!(selected, 1, "WITH must select exactly one ordered producer");
            for (ordinal, binding) in composition.bindings().iter().enumerate() {
                assert_eq!(binding.binding().ordinal(), ordinal);
                assert_eq!(
                    binding.producer().item_id(),
                    document.producer_item_ids()[ordinal]
                );
                assert!(binding.relation_output().is_some());
            }
        }
        Ok(())
    }

    #[test]
    fn relational_composition_body_consumes_the_working_set() -> Result<(), String> {
        let document = reference_composition_document_v2(ReferenceCompositionBodyV2::Relational)?;
        assert_eq!(document.producer_item_ids().len(), 2);
        let composition = document.composition();
        let body = match composition.body() {
            crate::ResolvedCompositionBodyV2::Query(query) => query,
            other => {
                return Err(format!(
                    "relational composition body expected, got {other:?}"
                ))
            }
        };
        let scan = body
            .root()
            .inputs()
            .next()
            .ok_or("the body projection must consume a working set")?;
        let source = scan
            .source()
            .ok_or("the working-set scan must retain its resolved source")?;
        assert!(matches!(source, ResolvedScanSourceV2::Composition(_)));
        assert_eq!(
            composition.bindings()[0].name(),
            composition.with_target().canonical_name()
        );
        Ok(())
    }

    #[test]
    fn composition_bodies_keep_their_closed_family() -> Result<(), String> {
        let graph = reference_composition_document_v2(ReferenceCompositionBodyV2::Graph)?;
        assert!(matches!(
            graph.composition().body(),
            crate::ResolvedCompositionBodyV2::Graph(_)
        ));
        let check = reference_composition_document_v2(ReferenceCompositionBodyV2::Check)?;
        assert!(matches!(
            check.composition().body(),
            crate::ResolvedCompositionBodyV2::Check(_)
        ));
        let explain = reference_composition_document_v2(ReferenceCompositionBodyV2::Explain)?;
        match explain.composition().body() {
            crate::ResolvedCompositionBodyV2::Explain(presentation) => {
                assert!(matches!(
                    presentation.kind(),
                    crate::ResolvedPresentationKindV2::Explain { .. }
                ));
            }
            other => return Err(format!("EXPLAIN composition body expected, got {other:?}")),
        }
        Ok(())
    }

    // ---- relation shapes -----------------------------------------------

    #[test]
    fn multiplicative_right_semi_input_fixture_resolves() -> Result<(), String> {
        let reference = reference_relation_document_v2(
            ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight,
        )?;
        let root = exists_root(reference.document())?;
        let nested = reference
            .document()
            .nested_relation(root)
            .ok_or("the EXISTS fixture must register its nested relation")?;
        assert!(matches!(nested, TypedRelationExprV2::Join(_)));
        assert_eq!(nested.join_mode(), Some(JoinModeV2::Cross));
        assert_eq!(nested.inputs().count(), 2);
        Ok(())
    }

    // ---- small-receipt rule documents ----------------------------------

    #[test]
    fn forged_small_receipt_rule_document_carries_the_small_budget() -> Result<(), String> {
        let document =
            forged_small_receipt_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)?;
        assert_eq!(
            document.budget().max_semantic_steps(),
            SMALL_BUDGET_MAX_SEMANTIC_STEPS
        );
        assert_eq!(document.item_ids().len(), 2);
        assert_eq!(
            reference_rule_program_charge_v2(2),
            3,
            "two members charge one program plus two member steps"
        );
        Ok(())
    }

    #[test]
    fn rule_program_receipt_is_exact_at_the_sealed_limit_and_one_over() -> Result<(), String> {
        let exact =
            forged_small_receipt_rule_program_document_v2(SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS)?;
        assert_eq!(
            exact.budget().max_semantic_steps(),
            SMALL_BUDGET_MAX_SEMANTIC_STEPS
        );
        assert_eq!(
            u32::try_from(exact.item_ids().len())
                .map_err(|_error| "member count exceeds u32".to_owned())?,
            SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS
        );
        assert_eq!(
            reference_rule_program_charge_v2(SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS),
            SMALL_BUDGET_MAX_SEMANTIC_STEPS
        );

        let over =
            forged_small_receipt_rule_program_document_v2(SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS)?;
        assert_eq!(
            reference_rule_program_charge_v2(SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS),
            SMALL_BUDGET_MAX_SEMANTIC_STEPS + 1
        );
        assert_eq!(
            u32::try_from(over.item_ids().len())
                .map_err(|_error| "member count exceeds u32".to_owned())?,
            SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS
        );
        assert!(forged_small_receipt_rule_program_document_v2(0).is_err());
        Ok(())
    }

    // ---- narrow witness and alternate graph identity -------------------

    #[test]
    fn narrow_witness_document_seals_a_one_witness_cap() -> Result<(), String> {
        let document = reference_narrow_witness_graph_document_v2(GraphQueryKindV1::Path, 4)?;
        assert_eq!(
            document.graph().witness_bounds().max_witnesses(),
            NARROW_WITNESS_MAX_WITNESSES_V2
        );
        assert_eq!(document.graph().kind(), GraphQueryKindV1::Path);
        assert_eq!(document.graph().depth(), Some(4));
        Ok(())
    }

    #[test]
    fn alternate_graph_document_seals_a_distinct_node_identity() -> Result<(), String> {
        let request = |kind| ReferenceGraphRequestV2 {
            kind,
            references: reference_witness_graph_references_v2(kind),
            depth: Some(4),
            direction: None,
            summary_facets: Vec::new(),
            item_span: (0, 8),
        };
        let primary = reference_graph_document_v2(request(GraphQueryKindV1::Taint))?;
        let alternate = reference_alternate_graph_document_v2(request(GraphQueryKindV1::Taint))?;
        assert_ne!(alternate.graph_item_id(), primary.graph_item_id());
        assert_eq!(
            alternate.graph_item_id().hir_item_id(),
            HirNodeIdV1::from_raw(ALTERNATE_GRAPH_NODE_ID_V2)
        );
        assert_eq!(
            primary.graph_item_id().hir_item_id(),
            HirNodeIdV1::from_raw(2)
        );
        Ok(())
    }

    #[test]
    fn second_source_graph_document_has_a_foreign_source_qualified_identity() -> Result<(), String>
    {
        let primary = reference_graph_document_v2(ReferenceGraphRequestV2 {
            kind: GraphQueryKindV1::Taint,
            references: reference_witness_graph_references_v2(GraphQueryKindV1::Taint),
            depth: Some(4),
            direction: None,
            summary_facets: Vec::new(),
            item_span: (0, 8),
        })?;
        let secondary = second_source_graph_document_v2()?;
        assert_eq!(
            secondary.document().source_id(),
            SECONDARY_REFERENCE_SOURCE_V2
        );
        assert_eq!(
            secondary.graph_item_id().source_id(),
            SECONDARY_REFERENCE_SOURCE_V2
        );
        assert_ne!(secondary.graph_item_id(), primary.graph_item_id());
        assert_eq!(
            secondary.graph_item_id().hir_item_id(),
            primary.graph_item_id().hir_item_id()
        );
        assert_eq!(secondary.graph().kind(), primary.graph().kind());
        assert_eq!(secondary.graph().depth(), primary.graph().depth());
        Ok(())
    }

    #[test]
    fn configurable_check_fixture_seals_the_full_mode_option_matrix() -> Result<(), String> {
        for mode in [
            DetectionModeV1::PatternOnly,
            DetectionModeV1::SemanticOnly,
            DetectionModeV1::Hybrid,
        ] {
            for with_pattern in [false, true] {
                for with_nested_graph in [false, true] {
                    let request = ReferenceGraphRequestV2 {
                        kind: GraphQueryKindV1::Taint,
                        references: reference_witness_graph_references_v2(GraphQueryKindV1::Taint),
                        depth: Some(4),
                        direction: None,
                        summary_facets: Vec::new(),
                        item_span: (0, 80),
                    };
                    let fixture = reference_check_document_v2(
                        request,
                        ReferenceCheckFixtureV2 {
                            mode,
                            with_pattern,
                            with_nested_graph,
                        },
                    )?;
                    let check = fixture.check();
                    assert_eq!(check.mode(), mode);
                    assert_eq!(check.pattern().is_some(), with_pattern);
                    assert_eq!(check.nested_graph().is_some(), with_nested_graph);
                    assert_eq!(
                        check
                            .nested_graph()
                            .map(crate::ResolvedGraphQueryV2::item_id),
                        fixture.nested_graph_item_id()
                    );
                    assert_eq!(check.item_id(), fixture.check_item_id());
                }
            }
        }
        Ok(())
    }
}
