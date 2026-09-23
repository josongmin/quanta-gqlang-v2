//! The sole semantic-document to algebra lowering cut.

use crate::diagnostic::{AlgebraDiagnosticV2, AlgebraFailureKindV2};
use crate::node::{AlgebraFamilyV2, CheckedAlgebraNodeV2, CorrelationSetV2, LoweredRelationItemV2};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, JoinModeV2, RelationExistsPolarityV2,
    ResolvedCompositionBodyV2, ResolvedCompositionProducerV2, SemanticDocumentV2, SemanticItemIdV2,
    SemanticItemV2, TypedBoolExprV2, TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2,
    TypedRelationExprV2,
};
use quanta_gqlang_v2_types::{AggregationClassV1, SourceQualifiedNodeIdV2};
use std::collections::BTreeSet;

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
pub struct AlgebraSessionV2<'a> {
    document: &'a SemanticDocumentV2,
    budget: LoweringBudgetV2,
}

impl<'a> AlgebraSessionV2<'a> {
    /// Start one lowering transaction from the document's immutable receipt.
    #[must_use]
    pub fn new(document: &'a SemanticDocumentV2) -> Self {
        Self {
            document,
            budget: LoweringBudgetV2::new(document.budget_receipt()),
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
        validate_nested_owner(document, owner_item_id, root)?;
        lower_root_with_budget(document, owner_item_id, root, &mut self.budget)
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

fn validate_nested_owner(
    document: &SemanticDocumentV2,
    owner_item_id: SemanticItemIdV2,
    root: &TypedRelationExprV2,
) -> Result<(), AlgebraDiagnosticV2> {
    if owner_item_id.source_id() != document.source_id() {
        return Err(AlgebraDiagnosticV2::new(
            AlgebraFailureKindV2::ForeignSemanticFact,
            Some(root.location().clone()),
            "owning item belongs to another semantic document",
        ));
    }
    if root.key().source_id != document.source_id() {
        return Err(AlgebraDiagnosticV2::new(
            AlgebraFailureKindV2::ForeignSemanticFact,
            Some(root.location().clone()),
            "nested relation root belongs to another semantic document",
        ));
    }
    if !item_owns_nested_relation(document, owner_item_id, root) {
        return Err(failure(
            AlgebraFailureKindV2::ForeignSemanticFact,
            root.location(),
            "nested relation root is not owned by the requested semantic item",
        ));
    }
    Ok(())
}

/// Whether `root` is one of the exact relation trees sealed under `owner`.
///
/// Top-level relations own only their relational-EXISTS trees; Catalog Views
/// and Compositions additionally own their direct inline/producer/body roots.
/// Descendant discovery follows the document's sealed expression identities,
/// so another same-source item's structurally compatible relation cannot be
/// relabeled with this owner's identity.
fn item_owns_nested_relation(
    document: &SemanticDocumentV2,
    owner: SemanticItemIdV2,
    root: &TypedRelationExprV2,
) -> bool {
    let Some(item) = document.item(owner) else {
        return false;
    };
    match item {
        SemanticItemV2::Relation(relation) => {
            relation_references_nested(document, relation.root(), root)
        }
        SemanticItemV2::Catalog(catalog) => catalog.inline_view().is_some_and(|view| {
            view.root() == root || relation_references_nested(document, view.root(), root)
        }),
        SemanticItemV2::Composition(composition) => {
            composition.bindings().iter().any(|binding| {
                matches!(binding.producer(), ResolvedCompositionProducerV2::Query(query)
                    if query.root() == root
                        || relation_references_nested(document, query.root(), root))
            }) || matches!(composition.body(), ResolvedCompositionBodyV2::Query(query)
                if query.root() == root
                    || relation_references_nested(document, query.root(), root))
        }
        SemanticItemV2::Graph(_)
        | SemanticItemV2::Check(_)
        | SemanticItemV2::Presentation(_)
        | SemanticItemV2::Declaration(_)
        | SemanticItemV2::Meta(_)
        | SemanticItemV2::Rule(_) => false,
    }
}

/// Whether `candidate` contains a relational-EXISTS expression whose sealed
/// registry root is exactly `nested`.
fn relation_references_nested(
    document: &SemanticDocumentV2,
    candidate: &TypedRelationExprV2,
    nested: &TypedRelationExprV2,
) -> bool {
    let mut relations = vec![candidate];
    let mut expressions = Vec::new();
    let mut visited_relations = BTreeSet::new();
    let mut visited_expressions = BTreeSet::new();
    while !relations.is_empty() || !expressions.is_empty() {
        while let Some(relation) = relations.pop() {
            if !visited_relations.insert(relation.key()) {
                continue;
            }
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
            let Some(expression) = document.expression(key) else {
                continue;
            };
            match expression.shape() {
                TypedExprShapeV2::RelationExists { relation_root, .. } => {
                    let Some(sealed) = document.nested_relation(*relation_root) else {
                        continue;
                    };
                    if *relation_root == nested.key() && sealed == nested {
                        return true;
                    }
                    // Nested EXISTS trees remain owned transitively by the
                    // same top-level item. Continue through the sealed root so
                    // a second-level relation can neither be rejected as
                    // foreign nor relabeled under a sibling item.
                    relations.push(sealed);
                }
                TypedExprShapeV2::Path { base, .. }
                | TypedExprShapeV2::Unary { operand: base, .. }
                | TypedExprShapeV2::Paren { inner: base }
                | TypedExprShapeV2::BoolValue { value: base }
                | TypedExprShapeV2::Not { operand: base, .. } => expressions.push(*base),
                TypedExprShapeV2::Binary { lhs, rhs, .. }
                | TypedExprShapeV2::Compare { lhs, rhs, .. }
                | TypedExprShapeV2::And { lhs, rhs, .. }
                | TypedExprShapeV2::Or { lhs, rhs, .. } => {
                    expressions.push(*rhs);
                    expressions.push(*lhs);
                }
                TypedExprShapeV2::Call { arguments, .. } => {
                    expressions.extend(arguments.iter().filter_map(|argument| match argument {
                        TypedCallArgumentV2::Expression(key) => Some(*key),
                        TypedCallArgumentV2::Star => None,
                    }))
                }
                TypedExprShapeV2::Record(fields) => {
                    expressions.extend(fields.iter().map(|field| field.value()));
                }
                TypedExprShapeV2::List(items) => expressions.extend(items.iter().copied()),
                TypedExprShapeV2::LogicalQuantifier { body, .. } => expressions.push(*body),
                TypedExprShapeV2::Literal(_) | TypedExprShapeV2::Binding(_) => {}
            }
        }
    }
    false
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
            let take = 1 + exists.len();
            if results.len() < take {
                return Err(invalid_arity(location));
            }
            let mut drained: Vec<CheckedAlgebraNodeV2> =
                results.drain(results.len() - take..).collect();
            // The lowered children arrive in stable authored order: the
            // filtered input first, then each EXISTS nested root.
            let mut chain = drained.remove(0);
            for (leaf, right) in exists.into_iter().zip(drained) {
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
                return CheckedAlgebraNodeV2::new(
                    item_id,
                    stage.key(),
                    location.clone(),
                    output,
                    op,
                )
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

/// Decompose one checked Bool predicate into its top-level conjunction, in
/// stable authored left-to-right order.
///
/// A relational EXISTS leaf becomes a validated Semi/Anti candidate; an
/// ordinary leaf must be free of nested EXISTS below OR/NOT/call/record/
/// list/quantifier shapes and is retained as the residual Filter's ordinary
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
            TypedExprShapeV2::And { lhs, rhs, .. } => {
                let left = required_expression(document, *lhs, expression.location())?;
                let right = required_expression(document, *rhs, expression.location())?;
                work.push(right);
                work.push(left);
            }
            TypedExprShapeV2::RelationExists { .. } => {
                exists.push(exists_leaf(document, expression)?);
            }
            // Every other shape is an ordinary conjunct leaf; it may keep the
            // residual Filter only when no relational EXISTS hides below it.
            TypedExprShapeV2::Literal(_)
            | TypedExprShapeV2::Binding(_)
            | TypedExprShapeV2::Path { .. }
            | TypedExprShapeV2::Unary { .. }
            | TypedExprShapeV2::Binary { .. }
            | TypedExprShapeV2::Call { .. }
            | TypedExprShapeV2::Paren { .. }
            | TypedExprShapeV2::Record(_)
            | TypedExprShapeV2::List(_)
            | TypedExprShapeV2::BoolValue { .. }
            | TypedExprShapeV2::Compare { .. }
            | TypedExprShapeV2::Or { .. }
            | TypedExprShapeV2::Not { .. }
            | TypedExprShapeV2::LogicalQuantifier { .. } => {
                reject_nested_exists(document, expression, expression_nodes)?;
                let carrier = if expression.key() == root_key {
                    predicate.clone()
                } else {
                    TypedBoolExprV2::try_new(expression.clone(), expression.location().location())
                        .map_err(|_failure| {
                        failure(
                            AlgebraFailureKindV2::InvalidStageContract,
                            expression.location(),
                            "filter conjunct is not a checked Bool fact",
                        )
                    })?
                };
                ordinary.push(carrier);
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

/// Validate one relational EXISTS leaf against the document before any node
/// construction: the exact nested root must be present, the capture set must
/// be duplicate-free, and every correlation reference and binding must
/// belong to this document.
fn exists_leaf<'a>(
    document: &'a SemanticDocumentV2,
    expression: &'a TypedExprV2,
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
        polarity: *polarity,
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
mod tests {
    #![expect(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "owner tests build sealed semantic documents and assert exact trees at fixed positions"
    )]

    use super::*;
    use crate::node::AlgebraFamilyV2;
    use quanta_gqlang_v2_sema::test_support::{
        forged_mixed_scalar_aggregate_projection_document_v2,
        forged_small_receipt_bool_tree_document_v2,
        forged_small_receipt_relation_chain_document_v2, forged_wrong_nested_root_document_v2,
        reference_relation_document_v2, ReferenceRelationFixtureV2,
        SMALL_BUDGET_MAX_EXPRESSION_NODES, SMALL_BUDGET_MAX_SEMANTIC_STEPS,
    };
    use quanta_gqlang_v2_sema::{
        DownstreamBudgetReceiptV2, FrontendBudgetV2, NullOrderV2, OutputColumnOriginV2,
        SortDirectionV2, TypedCallArgumentV2,
    };

    /// Smallest legal budget: every dimension 1 except the two under test,
    /// with frame bytes covering source bytes. The public entry always reads
    /// the document receipt; this helper reaches the private frame machinery
    /// only to prove exact-limit and one-over behavior.
    fn tiny_receipt(
        max_semantic_steps: u32,
        max_expression_nodes: u32,
    ) -> DownstreamBudgetReceiptV2 {
        FrontendBudgetV2::try_new(
            1,
            2,
            1,
            1,
            1,
            1,
            max_expression_nodes,
            1,
            1,
            max_semantic_steps,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            2,
        )
        .unwrap()
        .downstream_receipt()
    }

    fn families_in_authoring_order(root: &CheckedAlgebraNodeV2) -> Vec<AlgebraFamilyV2> {
        let mut stack = vec![root];
        let mut families = Vec::new();
        while let Some(node) = stack.pop() {
            families.push(node.family());
            let mut children = node.children();
            stack.extend(children.drain(..).rev());
        }
        families
    }

    #[test]
    fn pure_exists_predicate_leaves_no_residual_filter_node() {
        let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Exists).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::SemiJoin);
        assert_eq!(
            families_in_authoring_order(root),
            vec![
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Scan
            ],
            "a pure relational EXISTS predicate must not retain a Filter node"
        );
        assert!(root.filter_conjuncts().is_empty());
        assert!(root.correlations().is_some_and(|set| set.is_empty()));
        assert_eq!(root.output(), lowered.output());
        assert_eq!(root.children()[1].family(), AlgebraFamilyV2::Scan);
    }

    #[test]
    fn pure_not_exists_predicate_leaves_no_residual_filter_node() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::NotExists).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::AntiJoin);
        assert_eq!(
            families_in_authoring_order(root),
            vec![
                AlgebraFamilyV2::AntiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Scan
            ],
            "a pure relational NOT EXISTS predicate must not retain a Filter node"
        );
        assert!(root.filter_conjuncts().is_empty());
        assert!(root.correlations().is_some_and(|set| set.is_empty()));
    }

    #[test]
    fn ordinary_filter_keeps_one_filter_with_the_sealed_bool_fact() {
        let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Filter);
        let conjuncts = root.filter_conjuncts();
        assert_eq!(
            conjuncts.len(),
            1,
            "an ordinary predicate is retained as one full conjunction"
        );
        // The fixture authors `bool_value(bool_lit(true))`: the retained
        // conjunct is the BoolValue fact, and its forwarded child is the
        // document-owned Bool literal.
        let TypedExprShapeV2::BoolValue { value } = conjuncts[0].expression().shape() else {
            panic!("the retained conjunct must be the sealed typed Bool fact");
        };
        let child = reference
            .document()
            .expression(*value)
            .expect("the BoolValue child is a document expression");
        assert!(matches!(child.shape(), TypedExprShapeV2::Literal(_)));
        let child = &root.children()[0];
        assert_eq!(child.family(), AlgebraFamilyV2::Scan);
        assert_eq!(root.output().row(), child.output().row());
        assert_eq!(root.output().columns(), child.output().columns());
        assert_eq!(root.output().cardinality(), child.output().cardinality());
        assert_eq!(
            root.output().partition_keys(),
            child.output().partition_keys()
        );
        assert_eq!(root.output().ordering(), child.output().ordering());
    }

    #[test]
    fn relation_walk_exact_budget_succeeds_and_one_over_rejects() {
        let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();
        let document = reference.document();
        let relation = match document.item(reference.item_id()) {
            Some(quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation)) => relation,
            other => panic!("filter fixture must hold a relation item: {other:?}"),
        };
        // The fixture lowers exactly two relation stages: Filter and Scan.
        let exact = lower_root(
            document,
            reference.item_id(),
            relation.root(),
            tiny_receipt(2, 8),
        );
        assert!(exact.is_ok(), "exact max_semantic_steps must succeed");

        let over = lower_root(
            document,
            reference.item_id(),
            relation.root(),
            tiny_receipt(1, 8),
        )
        .unwrap_err();
        assert_eq!(over.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
        assert_eq!(over.code(), "algebra.traversal_budget_exceeded");
        assert!(
            over.location().is_some(),
            "the over-budget rejection names a checked stage location"
        );
    }

    #[test]
    fn document_lowering_session_does_not_reset_between_items() {
        let reference =
            forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1)
                .expect("exact small-receipt chain");
        let mut session = AlgebraSessionV2::new(reference.document());
        session
            .lower_relation_item(reference.item_id())
            .expect("first traversal consumes the exact document allowance");
        let error = session
            .lower_relation_item(reference.item_id())
            .expect_err("a second traversal cannot reuse the consumed allowance");
        assert_eq!(error.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
    }

    #[test]
    fn expression_walk_exact_budget_succeeds_and_one_over_rejects() {
        let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();
        let document = reference.document();
        let relation = match document.item(reference.item_id()) {
            Some(quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation)) => relation,
            other => panic!("filter fixture must hold a relation item: {other:?}"),
        };
        // The predicate is one BoolValue leaf forwarding one Bool literal:
        // the decomposition walk visits the leaf, then the descendant search
        // visits the leaf and its literal child — three charged nodes.
        let exact = lower_root(
            document,
            reference.item_id(),
            relation.root(),
            tiny_receipt(4, 3),
        );
        assert!(exact.is_ok(), "exact max_expression_nodes must succeed");

        let over = lower_root(
            document,
            reference.item_id(),
            relation.root(),
            tiny_receipt(4, 2),
        )
        .unwrap_err();
        assert_eq!(over.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
        assert_eq!(over.code(), "algebra.traversal_budget_exceeded");
        assert!(over.location().is_some());
    }

    #[test]
    fn public_entry_reads_only_the_document_budget_receipt() {
        let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Exists).unwrap();
        let document = reference.document();
        // Three relation stages (filter, input scan, nested scan) and one
        // EXISTS leaf sit far below the standard receipt's caps; the public
        // entry accepts no budget argument of its own.
        assert!(document.budget_receipt().max_semantic_steps() >= 3);
        let lowered = lower_relation_item(document, reference.item_id()).unwrap();
        assert_eq!(lowered.root().family(), AlgebraFamilyV2::SemiJoin);
    }

    fn scan_qualified_name(node: &CheckedAlgebraNodeV2) -> &str {
        match node.scan_source().expect("node must be a scan") {
            quanta_gqlang_v2_sema::ResolvedScanSourceV2::Relation(id) => id.qualified_name(),
            quanta_gqlang_v2_sema::ResolvedScanSourceV2::View(_)
            | quanta_gqlang_v2_sema::ResolvedScanSourceV2::Composition(_) => {
                panic!("reference fixtures scan host relations only")
            }
        }
    }

    #[test]
    fn grouped_aggregate_preserves_the_sealed_partition_identity() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)
                .unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Aggregate);
        let group = &root.children()[0];
        assert_eq!(group.family(), AlgebraFamilyV2::Group);
        assert_eq!(group.children()[0].family(), AlgebraFamilyV2::Scan);
        // Group: exactly one authored key, key-only row, partition proof
        // aligned with the pre-group input width, Set output.
        assert_eq!(group.expressions().len(), 1);
        assert_eq!(group.output().row().fields().len(), 1);
        assert_eq!(group.output().partition_keys().len(), 1);
        assert_eq!(
            group.partition_input().len(),
            group.children()[0].output().row().fields().len()
        );
        assert_eq!(
            group.output().cardinality(),
            quanta_gqlang_v2_types::CardinalityV1::Set
        );
        // Aggregate: consumes exactly the one Group, retains its partition
        // identity verbatim, and keeps the fold expression row-aligned with
        // the output column origins (the ordering property that generalizes
        // to multiple aggregates).
        assert_eq!(
            root.output().partition_keys(),
            group.output().partition_keys(),
            "aggregate output must retain its Group partition identity"
        );
        assert_eq!(root.expressions().len(), 1);
        assert_eq!(
            root.expressions()[0].aggregate_context().aggregation(),
            AggregationClassV1::Aggregate
        );
        assert_eq!(root.output().columns().len(), 1);
        assert!(matches!(
            root.output().columns()[0].origin(),
            OutputColumnOriginV2::Expression(origin)
                if *origin == root.expressions()[0].key()
        ));
    }

    #[test]
    fn scalar_count_star_consumes_the_producer_sealed_empty_partition() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Aggregate);
        let group = &root.children()[0];
        assert_eq!(group.family(), AlgebraFamilyV2::Group);
        // The implicit Group(empty) is key-only: no keys, no row, no columns,
        // no partition, Set cardinality.
        assert!(group.expressions().is_empty());
        assert!(group.partition_input().is_empty());
        assert!(group.output().row().fields().is_empty());
        assert!(group.output().columns().is_empty());
        assert!(group.output().partition_keys().is_empty());
        assert_eq!(
            group.output().cardinality(),
            quanta_gqlang_v2_types::CardinalityV1::Set
        );
        assert_eq!(group.children()[0].family(), AlgebraFamilyV2::Scan);
        // count(*): the retained fold uses the authored star, never a
        // zero-argument call spelling.
        assert_eq!(root.expressions().len(), 1);
        let fold = &root.expressions()[0];
        assert!(fold.aggregate_context().uses_star());
        assert_eq!(
            fold.aggregate_context().aggregation(),
            AggregationClassV1::Aggregate
        );
        assert!(matches!(
            fold.shape(),
            TypedExprShapeV2::Call { arguments, .. }
                if matches!(arguments.as_slice(), [TypedCallArgumentV2::Star])
        ));
        assert!(root.output().partition_keys().is_empty());
    }

    #[test]
    fn inner_join_retains_the_typed_on_and_ordered_child_schema() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::InnerJoinOn).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Join);
        assert_eq!(
            root.join_mode(),
            Some(quanta_gqlang_v2_sema::JoinModeV2::Inner)
        );
        assert!(root.predicate().is_some());
        let children = root.children();
        assert_eq!(children.len(), 2);
        // Ordered concatenation of the child columns and the union of the
        // child provenance are checked at construction.
        assert_eq!(
            root.output().columns(),
            children[0]
                .output()
                .columns()
                .iter()
                .chain(children[1].output().columns().iter())
                .cloned()
                .collect::<Vec<_>>()
        );
        assert_eq!(root.output().provenance().iter().count(), 2);
    }

    #[test]
    fn project_retains_exact_expressions_and_row_aligned_origins() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)
                .unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Project);
        assert_eq!(root.expressions().len(), 2);
        assert_eq!(root.output().columns().len(), 2);
        for (expression, column) in root.expressions().iter().zip(root.output().columns()) {
            assert!(matches!(
                column.origin(),
                OutputColumnOriginV2::Expression(origin) if *origin == expression.key()
            ));
        }
        assert_eq!(root.children()[0].family(), AlgebraFamilyV2::Scan);
    }

    #[test]
    fn sort_retains_direction_null_order_and_sealed_key_identity() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::SortDesc).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Sort);
        let keys = root.order_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].direction(), SortDirectionV2::Desc);
        assert_eq!(keys[0].null_order(), NullOrderV2::Last);
        // The key's value type and identity stay sealed: the ordering equals
        // the output ordering contract and the input frame is preserved.
        let input = &root.children()[0];
        assert_eq!(root.output().ordering(), keys);
        assert_eq!(
            keys[0].value_type(),
            input.output().row().fields()[0].value_type()
        );
        assert_eq!(root.output().row(), input.output().row());
    }

    #[test]
    fn correlated_exists_retains_the_authored_capture_edge_set() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists).unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        // The outer filter stage vanishes (pure EXISTS); the NESTED relation's
        // own ordinary predicate keeps exactly its own Filter stage.
        assert_eq!(
            families_in_authoring_order(root),
            vec![
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Filter,
                AlgebraFamilyV2::Scan,
            ]
        );
        assert_eq!(root.children()[0].family(), AlgebraFamilyV2::Scan);
        assert!(root.children()[1].filter_conjuncts().len() == 1);
        let correlations = root.correlations().expect("semi join retains correlations");
        assert_eq!(correlations.iter().count(), 1);
        let edge = &correlations.iter().collect::<Vec<_>>()[0];
        // The correlation is a resolved outer binding identity plus the inner
        // reference location, in authored stable order.
        assert_eq!(edge.binding().canonical_name(), "r");
        assert!(reference.document().expression(edge.reference()).is_some());
    }

    #[test]
    fn correlated_not_exists_retains_the_authored_capture_edge_set() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedNotExists)
                .unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::AntiJoin);
        assert!(root
            .correlations()
            .is_some_and(|set| set.iter().count() == 1));
    }

    #[test]
    fn conjunctive_exists_pair_lowers_each_leaf_exactly_once_in_stable_order() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ConjunctiveExistsPair)
                .unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        let families = families_in_authoring_order(root);
        assert_eq!(
            families,
            vec![
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Scan,
            ],
            "two conjunctive EXISTS lower to exactly two SemiJoins over the input scan"
        );
        // Stable authored order: the FIRST conjunct (`other`) attaches
        // innermost; the SECOND (`records2`) is the outer right input.
        let outer = root;
        let inner = &outer.children()[0];
        assert_eq!(scan_qualified_name(outer.children()[1]), "app::records2");
        assert_eq!(scan_qualified_name(inner.children()[1]), "app::other");
        assert_eq!(scan_qualified_name(inner.children()[0]), "app::records");
        assert!(outer.correlations().is_some_and(|set| set.is_empty()));
    }

    #[test]
    fn mixed_conjunction_keeps_exactly_one_residual_filter() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ExistsWithBoolConjunct)
                .unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        // The residual Filter is the stage root and holds the FULL ordinary
        // conjunction; the EXISTS lowered exactly once below it. The ordinary
        // conjunct is the authored BoolValue fact wrapping its Bool literal.
        assert_eq!(root.family(), AlgebraFamilyV2::Filter);
        let conjuncts = root.filter_conjuncts();
        assert_eq!(conjuncts.len(), 1);
        assert!(matches!(
            conjuncts[0].expression().shape(),
            TypedExprShapeV2::BoolValue { .. }
        ));
        assert_eq!(
            families_in_authoring_order(root),
            vec![
                AlgebraFamilyV2::Filter,
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Scan,
            ]
        );
        let semi = &root.children()[0];
        assert_eq!(scan_qualified_name(semi.children()[1]), "app::other");
        assert_eq!(scan_qualified_name(semi.children()[0]), "app::records");
        // Filter and SemiJoin both retain the filter stage's exact output.
        assert_eq!(root.output(), semi.output());
        assert_eq!(root.output(), lowered.output());
    }

    #[test]
    fn nested_exists_lowers_each_level_once_with_separate_capture_sets() {
        let reference =
            reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels)
                .unwrap();
        let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
        let root = lowered.root();
        let families = families_in_authoring_order(root);
        assert_eq!(
            families,
            vec![
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Filter,
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Filter,
                AlgebraFamilyV2::Scan,
            ],
            "each EXISTS level lowers to exactly one SemiJoin; ordinary conjuncts stay in exactly one residual Filter per stage"
        );
        // The outer EXISTS captures the outer binding at both reference
        // sites; the nested EXISTS carries its own single-edge capture set.
        assert!(root
            .correlations()
            .is_some_and(|set| set.iter().count() == 2));
        let middle_filter = &root.children()[1];
        assert_eq!(middle_filter.family(), AlgebraFamilyV2::Filter);
        assert_eq!(middle_filter.filter_conjuncts().len(), 1);
        let nested_semi = &middle_filter.children()[0];
        assert!(nested_semi
            .correlations()
            .is_some_and(|set| set.iter().count() == 1));
    }

    #[test]
    fn mixed_scalar_aggregate_projection_is_an_impossible_contract() {
        // Forged at the document boundary: one scalar group key and one
        // aggregate fold in the same projection over a real Group. Sema
        // seals aggregate projections uniformly, so this shape can only be
        // minted through the test seam and must die at the checked
        // Aggregate constructor, not lower as a mixed Aggregate node.
        let forged = forged_mixed_scalar_aggregate_projection_document_v2().unwrap();
        let diagnostic = lower_relation_item(forged.document(), forged.item_id())
            .expect_err("a mixed scalar/aggregate projection must not lower");
        assert_eq!(
            diagnostic.kind(),
            AlgebraFailureKindV2::InvalidStageContract
        );
        assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
        assert!(
            diagnostic.location().is_some(),
            "the rejection names the forged projection stage"
        );
    }

    #[test]
    fn nested_registry_tree_with_a_swapped_inner_scan_rejects() {
        // Forged nested registry entry: the EXISTS root key still resolves,
        // but the registered tree's inner scan carries a minted stage
        // identity while the nested Filter keeps its sealed output columns.
        // The row-aligned frame check at the nested Filter constructor must
        // reject the disagreement; the right tree is consumed exactly by
        // registry key and never re-derived from the outer predicate.
        let forged = forged_wrong_nested_root_document_v2().unwrap();
        let diagnostic = lower_relation_item(forged.document(), forged.item_id())
            .expect_err("a nested tree whose frame disagrees with its stages must not lower");
        assert_eq!(
            diagnostic.kind(),
            AlgebraFailureKindV2::InvalidStageContract
        );
        assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
        assert!(
            diagnostic.location().is_some(),
            "the rejection names the inconsistent nested stage"
        );
    }

    #[test]
    fn public_relation_walk_small_receipt_exact_and_one_over() {
        // A forged small-receipt chain: sema resolves it under the standard
        // budget and the document carries the small receipt, so the relation
        // walk's own charging is the only limit. A chain of depth d lowers
        // d + 1 relation stages.
        let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
        let exact = forged_small_receipt_relation_chain_document_v2(exact_depth).unwrap();
        let lowered =
            lower_relation_item(exact.document(), exact.item_id()).expect("exact budget lowers");
        assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);

        let over = forged_small_receipt_relation_chain_document_v2(exact_depth + 1).unwrap();
        let diagnostic = lower_relation_item(over.document(), over.item_id())
            .expect_err("one stage over the receipt must reject");
        assert_eq!(
            diagnostic.kind(),
            AlgebraFailureKindV2::TraversalBudgetExceeded
        );
        assert_eq!(diagnostic.code(), "algebra.traversal_budget_exceeded");
        assert!(diagnostic.location().is_some());
    }

    #[test]
    fn public_expression_walk_small_receipt_exact_and_one_over() {
        // A left-leaning conjunction of depth d has d And nodes and d + 1
        // BoolValue leaves, each forwarding one Bool literal: the
        // decomposition walk charges 2 * d + 1 nodes and the per-leaf
        // descendant search charges two more per leaf, 4 * d + 3 charged
        // nodes in total. Depth 1 charges seven of the eight small-receipt
        // nodes; depth 2 charges eleven and rejects one over.
        let exact = forged_small_receipt_bool_tree_document_v2(1).unwrap();
        let lowered =
            lower_relation_item(exact.document(), exact.item_id()).expect("exact budget lowers");
        assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);
        assert_eq!(lowered.root().filter_conjuncts().len(), 2);

        let over = forged_small_receipt_bool_tree_document_v2(2).unwrap();
        assert_eq!(
            over.document().budget_receipt().max_expression_nodes(),
            SMALL_BUDGET_MAX_EXPRESSION_NODES
        );
        let diagnostic = lower_relation_item(over.document(), over.item_id())
            .expect_err("one node over the receipt must reject");
        assert_eq!(
            diagnostic.kind(),
            AlgebraFailureKindV2::TraversalBudgetExceeded
        );
        assert_eq!(diagnostic.code(), "algebra.traversal_budget_exceeded");
        assert!(diagnostic.location().is_some());
    }
}
