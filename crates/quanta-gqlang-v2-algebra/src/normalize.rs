//! The sole semantic-document to algebra lowering cut.

use crate::diagnostic::{AlgebraDiagnosticV2, AlgebraFailureKindV2};
use crate::node::{
    AlgebraFamilyV2, CheckedAlgebraNodeV2, CorrelationSetV2, ExistentialTermV2,
    LoweredRelationItemV2,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, JoinModeV2, RelationExistsPolarityV2,
    RelationOutputContractV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    SemanticDocumentV2, SemanticItemIdV2, SemanticItemV2, TypedBoolExprV2, TypedExprShapeV2,
    TypedExprV2, TypedRelationExprV2,
};
use quanta_gqlang_v2_types::{AggregationClassV1, SourceQualifiedNodeIdV2};
use std::collections::{BTreeMap, BTreeSet};

/// Lower exactly one sealed relation item from a semantic document.
///
/// This is the only public source ingress. It neither accepts HIR nor options
/// nor a budget: source names, call identities, output contracts, aggregate
/// classification, correlations, and the traversal caps are all sealed by
/// Sema, and both lowering walks are charged from the document's immutable
/// downstream budget receipt before every visit.
///
/// # Errors
///
/// Returns a typed diagnostic when the requested item is absent/non-relational,
/// or when a sealed document is internally inconsistent for algebra lowering.
pub fn lower_relation_item(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
) -> Result<LoweredRelationItemV2, AlgebraDiagnosticV2> {
    AlgebraSessionV2::new(document).lower_relation_item(item_id)
}

/// One document-scoped Algebra lowering transaction.
///
/// Top-level relations, inline Views, and composition relations consume the
/// same semantic-step and expression-node ledgers. The borrowed document makes
/// the remaining allowance non-transferable to another semantic transaction.
/// Nested-root ownership is indexed once per owner and charges those same
/// ledgers. Repeated lookups reuse that index but still compare the exact tree.
pub struct AlgebraSessionV2<'a> {
    document: &'a SemanticDocumentV2,
    budget: LoweringBudgetV2,
    owned_relations: BTreeMap<SemanticItemIdV2, OwnedRelationsV2<'a>>,
}

type OwnedRelationsV2<'a> = BTreeMap<SourceQualifiedNodeIdV2, &'a TypedRelationExprV2>;

impl<'a> AlgebraSessionV2<'a> {
    /// Start one lowering transaction from the document's immutable receipt.
    #[must_use]
    pub fn new(document: &'a SemanticDocumentV2) -> Self {
        Self {
            document,
            budget: LoweringBudgetV2::new(document.budget_receipt()),
            owned_relations: BTreeMap::new(),
        }
    }

    /// Lower one top-level relation and charge the shared document ledgers.
    pub fn lower_relation_item(
        &mut self,
        item_id: SemanticItemIdV2,
    ) -> Result<LoweredRelationItemV2, AlgebraDiagnosticV2> {
        let document = self.document;
        let Some(item) = document.item(item_id) else {
            return Err(AlgebraDiagnosticV2::new(
                AlgebraFailureKindV2::NonRelationItem,
                None,
                "requested semantic item is absent from the document",
            ));
        };
        let SemanticItemV2::Relation(relation) = item else {
            return Err(AlgebraDiagnosticV2::new(
                AlgebraFailureKindV2::NonRelationItem,
                None,
                "requested semantic item is not a relation",
            ));
        };
        if item_id.source_id() != document.source_id() {
            return Err(AlgebraDiagnosticV2::new(
                AlgebraFailureKindV2::ForeignSemanticFact,
                Some(relation.location().clone()),
                "requested item belongs to another semantic document",
            ));
        }
        let root = lower_root_with_budget(document, item_id, relation.root(), &mut self.budget)?;
        if root.output() != relation.output() {
            return Err(failure(
                AlgebraFailureKindV2::InvalidStageContract,
                relation.location(),
                "lowered root output differs from the sealed relation output",
            ));
        }
        Ok(LoweredRelationItemV2::new(item_id, root))
    }

    /// Lower one nested relation and charge the shared document ledgers.
    pub fn lower_nested_relation(
        &mut self,
        owner_item_id: SemanticItemIdV2,
        root: &TypedRelationExprV2,
    ) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
        let document = self.document;
        self.validate_nested_owner(owner_item_id, root)?;
        let lowered = lower_root_with_budget(document, owner_item_id, root, &mut self.budget)?;
        Ok(match document.item(owner_item_id) {
            Some(SemanticItemV2::Composition(composition)) => {
                lowered.with_composition_origin(composition.instance().clone(), root.key())
            }
            _ => lowered,
        })
    }
}

/// Lower one NESTED relation (a composition body or producer relation that is
/// not itself a top-level document item) through the exact same canonical
/// lowering path as [`lower_relation_item`].
///
/// The owning item identity is supplied by the caller (the composition item
/// that owns the nested root) so every diagnostic, scan identity, and budget
/// charge stays anchored on a real document item.
///
/// # Errors
///
/// Returns a typed diagnostic when the root does not belong to this document,
/// is not one of the document's sealed nested relations, or when the sealed
/// relation is internally inconsistent for algebra lowering.
pub fn lower_nested_relation_v2(
    document: &SemanticDocumentV2,
    owner_item_id: SemanticItemIdV2,
    root: &TypedRelationExprV2,
) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
    AlgebraSessionV2::new(document).lower_nested_relation(owner_item_id, root)
}

impl<'a> AlgebraSessionV2<'a> {
    fn validate_nested_owner(
        &mut self,
        owner: SemanticItemIdV2,
        root: &TypedRelationExprV2,
    ) -> Result<(), AlgebraDiagnosticV2> {
        if owner.source_id() != self.document.source_id() {
            return Err(failure(
                AlgebraFailureKindV2::ForeignSemanticFact,
                root.location(),
                "owning item belongs to another semantic document",
            ));
        }
        if root.key().source_id != self.document.source_id() {
            return Err(failure(
                AlgebraFailureKindV2::ForeignSemanticFact,
                root.location(),
                "nested relation root belongs to another semantic document",
            ));
        }
        // Invalid caller-supplied identities must not grow the session cache.
        if self.document.item(owner).is_none() {
            return Err(failure(
                AlgebraFailureKindV2::ForeignSemanticFact,
                root.location(),
                "nested relation root is not owned by the requested semantic item",
            ));
        }
        if let std::collections::btree_map::Entry::Vacant(entry) = self.owned_relations.entry(owner)
        {
            // Publish only a complete index. A failed build consumes its work
            // allowance but never installs partial ownership authority.
            let index = discover_owned_relations(self.document, owner, &mut self.budget)?;
            entry.insert(index);
        }
        if !self
            .owned_relations
            .get(&owner)
            .and_then(|index| index.get(&root.key()))
            .is_some_and(|sealed| *sealed == root)
        {
            return Err(failure(
                AlgebraFailureKindV2::ForeignSemanticFact,
                root.location(),
                "nested relation root is not owned by the requested semantic item",
            ));
        }
        Ok(())
    }
}

/// Index only the exact roots the owner can lower separately: direct inline /
/// composition roots and transitively registered EXISTS roots, never arbitrary
/// subtrees. Every distinct discovery visit charges the same transaction
/// ledgers as lowering. References borrow the immutable session document.
fn discover_owned_relations<'a>(
    document: &'a SemanticDocumentV2,
    owner: SemanticItemIdV2,
    budget: &mut LoweringBudgetV2,
) -> Result<OwnedRelationsV2<'a>, AlgebraDiagnosticV2> {
    let mut owned = BTreeMap::new();
    let mut relations = Vec::new();
    let Some(item) = document.item(owner) else {
        return Ok(owned);
    };
    match item {
        SemanticItemV2::Relation(relation) => relations.push(relation.root()),
        SemanticItemV2::Catalog(catalog) => {
            if let Some(view) = catalog.inline_view() {
                insert_owned_root(&mut owned, view.root())?;
                relations.push(view.root());
            }
        }
        SemanticItemV2::Composition(composition) => {
            for binding in composition.bindings() {
                if let ResolvedCompositionProducerV2::Query(query) = binding.producer() {
                    insert_owned_root(&mut owned, query.root())?;
                    relations.push(query.root());
                }
            }
            if let ResolvedCompositionBodyV2::Query(query) = composition.body() {
                insert_owned_root(&mut owned, query.root())?;
                relations.push(query.root());
            }
        }
        SemanticItemV2::Graph(_)
        | SemanticItemV2::Command(_)
        | SemanticItemV2::Check(_)
        | SemanticItemV2::Presentation(_)
        | SemanticItemV2::Declaration(_)
        | SemanticItemV2::Meta(_)
        | SemanticItemV2::Rule(_) => {}
    }
    // Visit direct roots in authored order. Stack traversal within each root
    // is deterministic; it does not depend on map insertion order.
    relations.reverse();
    let mut expressions = Vec::new();
    let mut visited_relations = BTreeSet::new();
    let mut visited_expressions = BTreeSet::new();
    while !relations.is_empty() || !expressions.is_empty() {
        while let Some(relation) = relations.pop() {
            if !visited_relations.insert(relation.key()) {
                continue;
            }
            if budget.semantic_steps == 0 {
                return Err(failure(
                    AlgebraFailureKindV2::TraversalBudgetExceeded,
                    relation.location(),
                    "ownership discovery exceeded the document's max_semantic_steps budget",
                ));
            }
            budget.semantic_steps -= 1;
            relations.extend(relation.inputs());
            expressions.extend(relation.expressions().iter().copied());
            if let Some(predicate) = relation.predicate() {
                expressions.push(predicate.expression().key());
            }
        }
        while let Some(key) = expressions.pop() {
            if !visited_expressions.insert(key) {
                continue;
            }
            let expression = required_expression_by_key(document, key, None)?;
            if budget.expression_nodes == 0 {
                return Err(failure(
                    AlgebraFailureKindV2::TraversalBudgetExceeded,
                    expression.location(),
                    "ownership discovery exceeded the document's max_expression_nodes budget",
                ));
            }
            budget.expression_nodes -= 1;
            if let TypedExprShapeV2::RelationExists { relation_root, .. } = expression.shape() {
                let sealed = document.nested_relation(*relation_root).ok_or_else(|| {
                    failure(
                        AlgebraFailureKindV2::MissingNestedRelation,
                        expression.location(),
                        "relational EXISTS root is absent from the semantic document",
                    )
                })?;
                insert_owned_root(&mut owned, sealed)?;
                relations.push(sealed);
            } else {
                expressions.extend(expression_children(expression.shape()));
            }
        }
    }
    Ok(owned)
}

fn insert_owned_root<'a>(
    owned: &mut OwnedRelationsV2<'a>,
    root: &'a TypedRelationExprV2,
) -> Result<(), AlgebraDiagnosticV2> {
    if owned
        .insert(root.key(), root)
        .is_some_and(|previous| previous != root)
    {
        return Err(failure(
            AlgebraFailureKindV2::InvalidStageContract,
            root.location(),
            "owned nested roots disagree under the same identity",
        ));
    }
    Ok(())
}

struct LoweringBudgetV2 {
    semantic_steps: u32,
    expression_nodes: u32,
}

impl LoweringBudgetV2 {
    const fn new(receipt: DownstreamBudgetReceiptV2) -> Self {
        Self {
            semantic_steps: receipt.max_semantic_steps(),
            expression_nodes: receipt.max_expression_nodes(),
        }
    }
}

/// One relation stage awaiting its lowerable children.
struct Visit<'a>(&'a TypedRelationExprV2);

/// One relation stage whose children are lowered and whose node must be
/// assembled from the sealed stage contract plus the visit-time plan.
struct Finish<'a> {
    stage: &'a TypedRelationExprV2,
    plan: StagePlan<'a>,
}

/// Visit-time lowering plan. Everything except the Filter decomposition is
/// re-read from the sealed stage at assembly time.
enum StagePlan<'a> {
    Scan,
    Filter {
        /// Relational EXISTS conjunct leaves in stable authored order, with
        /// their checked nested roots and correlation proofs.
        exists: Vec<ExistsLeaf<'a>>,
        /// The full ordinary conjunction in stable authored order.
        ordinary: Vec<TypedBoolExprV2>,
    },
    Project,
    Join,
    Group,
    Sort,
}

/// One validated relational EXISTS conjunct awaiting its Semi/Anti assembly.
struct ExistsLeaf<'a> {
    expression: &'a TypedExprV2,
    polarity: RelationExistsPolarityV2,
    right: &'a TypedRelationExprV2,
    correlations: CorrelationSetV2,
}

/// Lower a relation root with iterative frames charged from the document's
/// budget receipt: every relation stage visit charges one
/// `max_semantic_steps` step before it is visited, and every expression node
/// visited by the predicate walks charges one `max_expression_nodes` node
/// before it is visited. Exact budgets succeed; one visit beyond the cap
/// rejects at the checked location of the stage or expression being visited.
#[cfg(test)]
fn lower_root(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    root: &TypedRelationExprV2,
    receipt: DownstreamBudgetReceiptV2,
) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
    let mut budget = LoweringBudgetV2::new(receipt);
    lower_root_with_budget(document, item_id, root, &mut budget)
}

fn lower_root_with_budget(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    root: &TypedRelationExprV2,
    budget: &mut LoweringBudgetV2,
) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
    let mut stack: Vec<Frame<'_>> = vec![Frame::Visit(Visit(root))];
    let mut results: Vec<CheckedAlgebraNodeV2> = Vec::new();
    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Visit(Visit(stage)) => {
                if budget.semantic_steps == 0 {
                    return Err(failure(
                        AlgebraFailureKindV2::TraversalBudgetExceeded,
                        stage.location(),
                        "relation lowering exceeded the document's max_semantic_steps budget",
                    ));
                }
                budget.semantic_steps -= 1;
                let location = stage.location();
                if stage.key().source_id != document.source_id()
                    || location.location().source_id != document.source_id()
                {
                    return Err(failure(
                        AlgebraFailureKindV2::ForeignSemanticFact,
                        location,
                        "relation stage belongs to another semantic document",
                    ));
                }
                match stage {
                    TypedRelationExprV2::Scan(_) => {
                        stack.push(Frame::Finish(Finish {
                            stage,
                            plan: StagePlan::Scan,
                        }));
                    }
                    TypedRelationExprV2::Filter(_) => {
                        let input = one_input(stage)?;
                        let Some(predicate) = stage.predicate().cloned() else {
                            return Err(failure(
                                AlgebraFailureKindV2::InvalidStageContract,
                                location,
                                "filter stage has no typed predicate",
                            ));
                        };
                        let (exists, ordinary) =
                            decompose_predicate(document, predicate, &mut budget.expression_nodes)?;
                        let mut nested: Vec<&TypedRelationExprV2> =
                            exists.iter().map(|leaf| leaf.right).collect();
                        stack.push(Frame::Finish(Finish {
                            stage,
                            plan: StagePlan::Filter { exists, ordinary },
                        }));
                        for right in nested.drain(..).rev() {
                            stack.push(Frame::Visit(Visit(right)));
                        }
                        stack.push(Frame::Visit(Visit(input)));
                    }
                    TypedRelationExprV2::Project(_) => {
                        let input = one_input(stage)?;
                        stack.push(Frame::Finish(Finish {
                            stage,
                            plan: StagePlan::Project,
                        }));
                        stack.push(Frame::Visit(Visit(input)));
                    }
                    TypedRelationExprV2::Join(_) => {
                        let (left, right) = two_inputs(stage)?;
                        stack.push(Frame::Finish(Finish {
                            stage,
                            plan: StagePlan::Join,
                        }));
                        stack.push(Frame::Visit(Visit(right)));
                        stack.push(Frame::Visit(Visit(left)));
                    }
                    TypedRelationExprV2::Group(_) => {
                        let input = one_input(stage)?;
                        stack.push(Frame::Finish(Finish {
                            stage,
                            plan: StagePlan::Group,
                        }));
                        stack.push(Frame::Visit(Visit(input)));
                    }
                    TypedRelationExprV2::Sort(_) => {
                        let input = one_input(stage)?;
                        stack.push(Frame::Finish(Finish {
                            stage,
                            plan: StagePlan::Sort,
                        }));
                        stack.push(Frame::Visit(Visit(input)));
                    }
                }
            }
            Frame::Finish(Finish { stage, plan }) => {
                let node = assemble(document, item_id, stage, plan, &mut results)?;
                results.push(node);
            }
        }
    }
    results.pop().ok_or_else(|| {
        AlgebraDiagnosticV2::new(
            AlgebraFailureKindV2::InvalidStageContract,
            None,
            "relation lowering did not produce exactly one root",
        )
    })
}

enum Frame<'a> {
    Visit(Visit<'a>),
    Finish(Finish<'a>),
}

/// Assemble one node from a finished stage, consuming exactly the child
/// results this stage's plan lowered.
fn assemble(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    stage: &TypedRelationExprV2,
    plan: StagePlan<'_>,
    results: &mut Vec<CheckedAlgebraNodeV2>,
) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
    let location = stage.location();
    let output = stage.output().clone();
    match plan {
        StagePlan::Scan => {
            let Some(source) = stage.source().cloned() else {
                return Err(failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "scan stage has no resolved source",
                ));
            };
            CheckedAlgebraNodeV2::new(
                item_id,
                stage.key(),
                location.clone(),
                output,
                CheckedAlgebraNodeV2::scan(source),
            )
            .map_err(|()| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "scan stage cannot form a checked algebra node",
                )
            })
        }
        StagePlan::Filter { exists, ordinary } => {
            assemble_filter(item_id, stage, location, output, exists, ordinary, results)
        }
        StagePlan::Project => {
            let expressions = checked_expressions(document, stage.expressions(), location)?;
            let input = take_one(results, location)?;
            let aggregate = expressions.iter().any(|expression| {
                expression.aggregate_context().aggregation() == AggregationClassV1::Aggregate
            });
            if aggregate {
                if input.family() != AlgebraFamilyV2::Group {
                    return Err(failure(
                        AlgebraFailureKindV2::AggregateWithoutGroup,
                        location,
                        "aggregate projection must directly follow a checked Group stage",
                    ));
                }
                CheckedAlgebraNodeV2::new(
                    item_id,
                    stage.key(),
                    location.clone(),
                    output,
                    CheckedAlgebraNodeV2::aggregate(input, expressions),
                )
            } else {
                CheckedAlgebraNodeV2::new(
                    item_id,
                    stage.key(),
                    location.clone(),
                    output,
                    CheckedAlgebraNodeV2::project(input, expressions),
                )
            }
            .map_err(|()| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "projection stage cannot form a checked algebra node",
                )
            })
        }
        StagePlan::Join => {
            if results.len() < 2 {
                return Err(invalid_arity(location));
            }
            let mut drained: Vec<CheckedAlgebraNodeV2> =
                results.drain(results.len() - 2..).collect();
            let Some(right) = drained.pop() else {
                return Err(invalid_arity(location));
            };
            let Some(left) = drained.pop() else {
                return Err(invalid_arity(location));
            };
            let Some(mode) = stage.join_mode() else {
                return Err(failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "join stage has no checked join mode",
                ));
            };
            let predicate = stage.predicate().cloned();
            match (mode, predicate.is_some()) {
                (JoinModeV2::Inner, true) | (JoinModeV2::Cross, false) => {}
                _ => {
                    return Err(failure(
                        AlgebraFailureKindV2::InvalidStageContract,
                        location,
                        "join mode and predicate disagree",
                    ));
                }
            }
            CheckedAlgebraNodeV2::new(
                item_id,
                stage.key(),
                location.clone(),
                output,
                CheckedAlgebraNodeV2::join(left, right, mode, predicate),
            )
            .map_err(|()| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "join stage cannot form a checked algebra node",
                )
            })
        }
        StagePlan::Group => {
            let keys = checked_expressions(document, stage.expressions(), location)?;
            let input = take_one(results, location)?;
            CheckedAlgebraNodeV2::new(
                item_id,
                stage.key(),
                location.clone(),
                output,
                CheckedAlgebraNodeV2::group(input, keys, stage.partition_input().to_vec()),
            )
            .map_err(|()| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "group stage cannot form a checked algebra node",
                )
            })
        }
        StagePlan::Sort => {
            let expressions = checked_expressions(document, stage.expressions(), location)?;
            let input = take_one(results, location)?;
            let keys = output.ordering().to_vec();
            if expressions.len() != keys.len()
                || expressions
                    .iter()
                    .zip(&keys)
                    .any(|(expression, key)| expression.key() != key.expression())
            {
                return Err(failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "sort expressions differ from the sealed order contract",
                ));
            }
            CheckedAlgebraNodeV2::new(
                item_id,
                stage.key(),
                location.clone(),
                output,
                CheckedAlgebraNodeV2::sort(input, keys),
            )
            .map_err(|()| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "sort stage cannot form a checked algebra node",
                )
            })
        }
    }
}

fn assemble_filter(
    item_id: SemanticItemIdV2,
    stage: &TypedRelationExprV2,
    location: &CheckedLocatedSpanV2,
    output: RelationOutputContractV2,
    exists: Vec<ExistsLeaf<'_>>,
    ordinary: Vec<TypedBoolExprV2>,
    results: &mut Vec<CheckedAlgebraNodeV2>,
) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
    let take = 1 + exists.len();
    if results.len() < take {
        return Err(invalid_arity(location));
    }
    let mut drained: Vec<CheckedAlgebraNodeV2> = results.drain(results.len() - take..).collect();
    // The lowered children arrive in stable authored order: the
    // filtered input first, then each EXISTS nested root.
    let mut chain = drained.remove(0);
    if exists.len() > 1 {
        let terms = exists
            .into_iter()
            .zip(drained)
            .map(|(leaf, right)| {
                ExistentialTermV2::new(
                    leaf.expression.key(),
                    leaf.polarity,
                    leaf.correlations,
                    right,
                )
            })
            .collect();
        chain = CheckedAlgebraNodeV2::new(
            item_id,
            stage.key(),
            location.clone(),
            output.clone(),
            CheckedAlgebraNodeV2::exists_filter(chain, terms),
        )
        .map_err(|()| {
            failure(
                AlgebraFailureKindV2::InvalidStageContract,
                location,
                "relational EXISTS conjuncts cannot form a checked algebra filter",
            )
        })?;
    } else if let Some((leaf, right)) = exists.into_iter().zip(drained).next() {
        let op = match leaf.polarity {
            RelationExistsPolarityV2::Exists => {
                CheckedAlgebraNodeV2::semi_join(chain, right, leaf.correlations)
            }
            RelationExistsPolarityV2::NotExists => {
                CheckedAlgebraNodeV2::anti_join(chain, right, leaf.correlations)
            }
        };
        chain = CheckedAlgebraNodeV2::new(
            item_id,
            leaf.expression.key(),
            leaf.expression.location().clone(),
            output.clone(),
            op,
        )
        .map_err(|()| {
            failure(
                AlgebraFailureKindV2::InvalidStageContract,
                leaf.expression.location(),
                "relational EXISTS cannot form a checked algebra join",
            )
        })?;
    }
    if !ordinary.is_empty() {
        let op = CheckedAlgebraNodeV2::filter(chain, ordinary);
        return CheckedAlgebraNodeV2::new(item_id, stage.key(), location.clone(), output, op)
            .map_err(|()| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    location,
                    "filter stage cannot form a checked algebra node",
                )
            });
    }
    // A pure relational EXISTS predicate: the deepest Semi/Anti join
    // is the stage root and no residual Filter node exists.
    Ok(chain)
}

/// Decompose one checked Bool predicate into its top-level conjunction, in
/// stable authored left-to-right order.
///
/// A direct relational EXISTS leaf (optionally parenthesized or directly
/// negated) becomes a validated Semi/Anti candidate; an ordinary leaf must be
/// free of nested EXISTS below OR/scalar/call/record/list/quantifier shapes
/// and is retained as the residual Filter's ordinary
/// conjunction. Every visited expression node charges the shared
/// `max_expression_nodes` ledger before it is visited, and every EXISTS-side
/// fact (nested root, duplicate-free capture set, same-document references)
/// is validated before any node is constructed.
fn decompose_predicate<'a>(
    document: &'a SemanticDocumentV2,
    predicate: TypedBoolExprV2,
    expression_nodes: &mut u32,
) -> Result<(Vec<ExistsLeaf<'a>>, Vec<TypedBoolExprV2>), AlgebraDiagnosticV2> {
    let root_key = predicate.expression().key();
    let root =
        required_expression_by_key(document, root_key, Some(predicate.expression().location()))?;
    let mut work: Vec<&'a TypedExprV2> = vec![root];
    let mut exists = Vec::new();
    let mut ordinary = Vec::new();
    while let Some(expression) = work.pop() {
        if *expression_nodes == 0 {
            return Err(failure(
                AlgebraFailureKindV2::TraversalBudgetExceeded,
                expression.location(),
                "predicate traversal exceeded the document's max_expression_nodes budget",
            ));
        }
        *expression_nodes -= 1;
        match expression.shape() {
            TypedExprShapeV2::Paren { inner } => {
                work.push(required_expression(
                    document,
                    *inner,
                    expression.location(),
                )?);
            }
            TypedExprShapeV2::And { lhs, rhs, .. } => {
                let left = required_expression(document, *lhs, expression.location())?;
                let right = required_expression(document, *rhs, expression.location())?;
                work.push(right);
                work.push(left);
            }
            TypedExprShapeV2::RelationExists { .. } => {
                exists.push(exists_leaf(document, expression, false)?);
            }
            TypedExprShapeV2::Not { operand, .. } => {
                let inner = required_expression(document, *operand, expression.location())?;
                if let Some(leaf) = direct_exists_under_parens(document, inner, expression_nodes)? {
                    exists.push(exists_leaf(document, leaf, true)?);
                } else {
                    reject_nested_exists(document, expression, expression_nodes)?;
                    ordinary.push(bool_carrier(root_key, predicate.clone(), expression)?);
                }
            }
            // Every other shape is an ordinary conjunct leaf; it may keep the
            // residual Filter only when no relational EXISTS hides below it.
            TypedExprShapeV2::Literal(_)
            | TypedExprShapeV2::Binding(_)
            | TypedExprShapeV2::Path { .. }
            | TypedExprShapeV2::Unary { .. }
            | TypedExprShapeV2::Binary { .. }
            | TypedExprShapeV2::Call { .. }
            | TypedExprShapeV2::Record(_)
            | TypedExprShapeV2::List(_)
            | TypedExprShapeV2::BoolValue { .. }
            | TypedExprShapeV2::Compare { .. }
            | TypedExprShapeV2::Or { .. }
            | TypedExprShapeV2::LogicalQuantifier { .. } => {
                reject_nested_exists(document, expression, expression_nodes)?;
                ordinary.push(bool_carrier(root_key, predicate.clone(), expression)?);
            }
        }
    }
    if exists.is_empty() && ordinary.is_empty() {
        return Err(failure(
            AlgebraFailureKindV2::InvalidStageContract,
            root.location(),
            "filter predicate decomposed into no conjuncts",
        ));
    }
    Ok((exists, ordinary))
}

fn bool_carrier(
    root_key: SourceQualifiedNodeIdV2,
    root: TypedBoolExprV2,
    expression: &TypedExprV2,
) -> Result<TypedBoolExprV2, AlgebraDiagnosticV2> {
    if expression.key() == root_key {
        Ok(root)
    } else {
        TypedBoolExprV2::try_new(expression.clone(), expression.location().location()).map_err(
            |_type_error| {
                failure(
                    AlgebraFailureKindV2::InvalidStageContract,
                    expression.location(),
                    "filter conjunct is not a checked Bool fact",
                )
            },
        )
    }
}

/// A NOT only changes an EXISTS literal when every wrapper is transparent.
fn direct_exists_under_parens<'a>(
    document: &'a SemanticDocumentV2,
    mut expression: &'a TypedExprV2,
    expression_nodes: &mut u32,
) -> Result<Option<&'a TypedExprV2>, AlgebraDiagnosticV2> {
    let remaining_before_probe = *expression_nodes;
    loop {
        if *expression_nodes == 0 {
            return Err(failure(
                AlgebraFailureKindV2::TraversalBudgetExceeded,
                expression.location(),
                "predicate traversal exceeded the document's max_expression_nodes budget",
            ));
        }
        *expression_nodes -= 1;
        match expression.shape() {
            TypedExprShapeV2::Paren { inner } => {
                expression = required_expression(document, *inner, expression.location())?;
            }
            TypedExprShapeV2::RelationExists { .. } => return Ok(Some(expression)),
            TypedExprShapeV2::Literal(_)
            | TypedExprShapeV2::Binding(_)
            | TypedExprShapeV2::Path { .. }
            | TypedExprShapeV2::Unary { .. }
            | TypedExprShapeV2::Binary { .. }
            | TypedExprShapeV2::Call { .. }
            | TypedExprShapeV2::Record(_)
            | TypedExprShapeV2::List(_)
            | TypedExprShapeV2::BoolValue { .. }
            | TypedExprShapeV2::Compare { .. }
            | TypedExprShapeV2::And { .. }
            | TypedExprShapeV2::Or { .. }
            | TypedExprShapeV2::Not { .. }
            | TypedExprShapeV2::LogicalQuantifier { .. } => {
                // The ordinary-leaf rejection walk will charge this path
                // again, so the failed direct-literal probe consumes no budget.
                *expression_nodes = remaining_before_probe;
                return Ok(None);
            }
        }
    }
}

/// Validate one relational EXISTS leaf against the document before any node
/// construction: the exact nested root must be present, the capture set must
/// be duplicate-free, and every correlation reference and binding must
/// belong to this document.
fn exists_leaf<'a>(
    document: &'a SemanticDocumentV2,
    expression: &'a TypedExprV2,
    negated: bool,
) -> Result<ExistsLeaf<'a>, AlgebraDiagnosticV2> {
    let TypedExprShapeV2::RelationExists {
        polarity,
        relation_root,
        correlations,
    } = expression.shape()
    else {
        return Err(failure(
            AlgebraFailureKindV2::InvalidStageContract,
            expression.location(),
            "relation-exists collector returned a non-EXISTS leaf",
        ));
    };
    let Some(right) = document.nested_relation(*relation_root) else {
        return Err(failure(
            AlgebraFailureKindV2::MissingNestedRelation,
            expression.location(),
            "relational EXISTS root is absent from the semantic document",
        ));
    };
    let correlations = CorrelationSetV2::try_new(correlations.clone()).map_err(|()| {
        failure(
            AlgebraFailureKindV2::DuplicateCorrelationReference,
            expression.location(),
            "relational EXISTS has a duplicate capture reference",
        )
    })?;
    if correlations.iter().any(|edge| {
        edge.reference().source_id != document.source_id()
            || document.expression(edge.reference()).is_none()
            || edge.binding().source_id() != document.source_id()
    }) {
        return Err(failure(
            AlgebraFailureKindV2::ForeignSemanticFact,
            expression.location(),
            "relational EXISTS correlation is absent or foreign",
        ));
    }
    Ok(ExistsLeaf {
        expression,
        polarity: if negated {
            match polarity {
                RelationExistsPolarityV2::Exists => RelationExistsPolarityV2::NotExists,
                RelationExistsPolarityV2::NotExists => RelationExistsPolarityV2::Exists,
            }
        } else {
            *polarity
        },
        right,
        correlations,
    })
}

/// Reject a relational EXISTS anywhere below one ordinary conjunct leaf.
/// The descendant search is iterative and charges the shared
/// `max_expression_nodes` ledger before every visited node.
fn reject_nested_exists(
    document: &SemanticDocumentV2,
    leaf: &TypedExprV2,
    expression_nodes: &mut u32,
) -> Result<(), AlgebraDiagnosticV2> {
    let mut stack = vec![leaf.key()];
    let mut visited: BTreeSet<SourceQualifiedNodeIdV2> = BTreeSet::new();
    while let Some(key) = stack.pop() {
        if !visited.insert(key) {
            continue;
        }
        let expression = required_expression_by_key(document, key, Some(leaf.location()))?;
        if *expression_nodes == 0 {
            return Err(failure(
                AlgebraFailureKindV2::TraversalBudgetExceeded,
                expression.location(),
                "predicate traversal exceeded the document's max_expression_nodes budget",
            ));
        }
        *expression_nodes -= 1;
        if matches!(expression.shape(), TypedExprShapeV2::RelationExists { .. }) {
            return Err(failure(
                AlgebraFailureKindV2::InvalidStageContract,
                expression.location(),
                "relational EXISTS must appear as a predicate conjunct",
            ));
        }
        stack.extend(expression_children(expression.shape()));
    }
    Ok(())
}

fn expression_children(shape: &TypedExprShapeV2) -> Vec<SourceQualifiedNodeIdV2> {
    match shape {
        TypedExprShapeV2::Literal(_)
        | TypedExprShapeV2::Binding(_)
        | TypedExprShapeV2::RelationExists { .. } => Vec::new(),
        TypedExprShapeV2::Path { base, .. }
        | TypedExprShapeV2::Unary { operand: base, .. }
        | TypedExprShapeV2::Paren { inner: base }
        | TypedExprShapeV2::BoolValue { value: base }
        | TypedExprShapeV2::Not { operand: base, .. }
        | TypedExprShapeV2::LogicalQuantifier { body: base, .. } => vec![*base],
        TypedExprShapeV2::Binary { lhs, rhs, .. }
        | TypedExprShapeV2::Compare { lhs, rhs, .. }
        | TypedExprShapeV2::And { lhs, rhs, .. }
        | TypedExprShapeV2::Or { lhs, rhs, .. } => vec![*lhs, *rhs],
        TypedExprShapeV2::Call { arguments, .. } => arguments
            .iter()
            .filter_map(|argument| match argument {
                quanta_gqlang_v2_sema::TypedCallArgumentV2::Expression(key) => Some(*key),
                quanta_gqlang_v2_sema::TypedCallArgumentV2::Star => None,
            })
            .collect(),
        TypedExprShapeV2::Record(fields) => fields.iter().map(|field| field.value()).collect(),
        TypedExprShapeV2::List(items) => items.clone(),
    }
}

fn take_one(
    results: &mut Vec<CheckedAlgebraNodeV2>,
    location: &CheckedLocatedSpanV2,
) -> Result<CheckedAlgebraNodeV2, AlgebraDiagnosticV2> {
    results.pop().ok_or_else(|| invalid_arity(location))
}

fn one_input(stage: &TypedRelationExprV2) -> Result<&TypedRelationExprV2, AlgebraDiagnosticV2> {
    let mut inputs = stage.inputs();
    let Some(input) = inputs.next() else {
        return Err(invalid_arity(stage.location()));
    };
    if inputs.next().is_some() {
        return Err(invalid_arity(stage.location()));
    }
    Ok(input)
}

fn two_inputs(
    stage: &TypedRelationExprV2,
) -> Result<(&TypedRelationExprV2, &TypedRelationExprV2), AlgebraDiagnosticV2> {
    let mut inputs = stage.inputs();
    let Some(left) = inputs.next() else {
        return Err(invalid_arity(stage.location()));
    };
    let Some(right) = inputs.next() else {
        return Err(invalid_arity(stage.location()));
    };
    if inputs.next().is_some() {
        return Err(invalid_arity(stage.location()));
    }
    Ok((left, right))
}

fn checked_expressions(
    document: &SemanticDocumentV2,
    keys: &[SourceQualifiedNodeIdV2],
    location: &CheckedLocatedSpanV2,
) -> Result<Vec<TypedExprV2>, AlgebraDiagnosticV2> {
    keys.iter()
        .map(|key| {
            if key.source_id != document.source_id() {
                return Err(failure(
                    AlgebraFailureKindV2::ForeignSemanticFact,
                    location,
                    "stage expression belongs to another semantic document",
                ));
            }
            document.expression(*key).cloned().ok_or_else(|| {
                failure(
                    AlgebraFailureKindV2::MissingExpression,
                    location,
                    "stage expression is absent from the semantic document",
                )
            })
        })
        .collect()
}

fn required_expression<'a>(
    document: &'a SemanticDocumentV2,
    key: SourceQualifiedNodeIdV2,
    location: &CheckedLocatedSpanV2,
) -> Result<&'a TypedExprV2, AlgebraDiagnosticV2> {
    required_expression_by_key(document, key, Some(location))
}

fn required_expression_by_key<'a>(
    document: &'a SemanticDocumentV2,
    key: SourceQualifiedNodeIdV2,
    location: Option<&CheckedLocatedSpanV2>,
) -> Result<&'a TypedExprV2, AlgebraDiagnosticV2> {
    if key.source_id != document.source_id() {
        return Err(AlgebraDiagnosticV2::new(
            AlgebraFailureKindV2::ForeignSemanticFact,
            location.cloned(),
            "expression belongs to another semantic document",
        ));
    }
    document.expression(key).ok_or_else(|| {
        AlgebraDiagnosticV2::new(
            AlgebraFailureKindV2::MissingExpression,
            location.cloned(),
            "expression is absent from the semantic document",
        )
    })
}

fn invalid_arity(location: &CheckedLocatedSpanV2) -> AlgebraDiagnosticV2 {
    failure(
        AlgebraFailureKindV2::InvalidStageContract,
        location,
        "relation stage has an invalid number of inputs",
    )
}

fn failure(
    kind: AlgebraFailureKindV2,
    location: &CheckedLocatedSpanV2,
    message: &'static str,
) -> AlgebraDiagnosticV2 {
    AlgebraDiagnosticV2::new(kind, Some(location.clone()), message)
}

#[cfg(test)]
#[path = "normalize/tests.rs"]
mod tests;
