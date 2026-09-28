//! Resolved unary rule contracts consumed by the logic owner.

use crate::{
    SemanticItemIdV2, TypedBoolExprV2, TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2,
    TypedRelationExprV2,
};
use quanta_gqlang_v2_hir::CheckedLocatedSpanV2;
use quanta_gqlang_v2_types::{
    AggregationClassV1, ResolvedCallTargetV2, ResolvedPredicateIdV2, SourceQualifiedNodeIdV2,
};
use std::collections::BTreeSet;

/// A rule-body shape that cannot prove its source predicate calls are literals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleBodyClassificationErrorV2 {
    location: Option<CheckedLocatedSpanV2>,
    reason: &'static str,
}

impl RuleBodyClassificationErrorV2 {
    #[must_use]
    pub const fn location(&self) -> Option<&CheckedLocatedSpanV2> {
        self.location.as_ref()
    }

    #[must_use]
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RuleBodyPositionV2 {
    Truth,
    NegatedTruth,
    Aggregated,
    OpaqueScalar,
}

impl RuleBodyPositionV2 {
    fn negated(self) -> Self {
        match self {
            Self::Truth => Self::NegatedTruth,
            Self::NegatedTruth => Self::Truth,
            Self::Aggregated => Self::Aggregated,
            Self::OpaqueScalar => Self::OpaqueScalar,
        }
    }
}

#[derive(Clone, Copy)]
enum RuleBodyNodeV2 {
    Expression(SourceQualifiedNodeIdV2, RuleBodyPositionV2, bool),
    Relation(SourceQualifiedNodeIdV2),
}

/// Classify source calls from the sealed typed shape. A source call is an SCC
/// edge only when it follows a BoolValue through transparent parentheses in the
/// truth spine. Every other reachable source call is an error, including
/// aggregate arguments and scalar comparisons. Both the document producer and
/// Logic use this walk.
pub(crate) fn classify_rule_body_calls<'a>(
    root: SourceQualifiedNodeIdV2,
    expression: impl Fn(SourceQualifiedNodeIdV2) -> Option<&'a TypedExprV2>,
    relation: impl Fn(SourceQualifiedNodeIdV2) -> Option<&'a TypedRelationExprV2>,
) -> Result<Vec<ResolvedRuleCallV2>, RuleBodyClassificationErrorV2> {
    let mut calls = Vec::new();
    let mut seen = BTreeSet::new();
    let mut seen_relations = BTreeSet::new();
    let mut stack = vec![RuleBodyNodeV2::Expression(
        root,
        RuleBodyPositionV2::Truth,
        false,
    )];
    while let Some(node) = stack.pop() {
        match node {
            RuleBodyNodeV2::Expression(key, position, direct_literal) => {
                if !seen.insert((key, position, direct_literal)) {
                    continue;
                }
                let Some(expr) = expression(key) else {
                    return Err(RuleBodyClassificationErrorV2 {
                        location: None,
                        reason: "rule body typed expression is missing",
                    });
                };
                let child = |key, position| RuleBodyNodeV2::Expression(key, position, false);
                match expr.shape() {
                    TypedExprShapeV2::BoolValue { value } => {
                        let direct = !direct_literal
                            && matches!(
                                position,
                                RuleBodyPositionV2::Truth | RuleBodyPositionV2::NegatedTruth
                            );
                        stack.push(RuleBodyNodeV2::Expression(
                            *value,
                            if direct {
                                position
                            } else {
                                RuleBodyPositionV2::OpaqueScalar
                            },
                            direct,
                        ));
                    }
                    TypedExprShapeV2::Not { operand, .. } => {
                        stack.push(child(*operand, position.negated()));
                    }
                    TypedExprShapeV2::Paren { inner } => {
                        stack.push(RuleBodyNodeV2::Expression(*inner, position, direct_literal));
                    }
                    TypedExprShapeV2::And { lhs, rhs, .. }
                    | TypedExprShapeV2::Or { lhs, rhs, .. } => {
                        stack.push(child(*rhs, position));
                        stack.push(child(*lhs, position));
                    }
                    TypedExprShapeV2::Call { target, arguments } => {
                        if matches!(target, ResolvedCallTargetV2::Predicate(_)) {
                            if !direct_literal
                                || !matches!(
                                    position,
                                    RuleBodyPositionV2::Truth | RuleBodyPositionV2::NegatedTruth
                                )
                            {
                                return Err(RuleBodyClassificationErrorV2 {
                                    location: Some(expr.location().clone()),
                                    reason:
                                        "source predicate call is not a direct rule truth literal",
                                });
                            }
                            calls.push(ResolvedRuleCallV2::new(
                                key,
                                target.clone(),
                                if position == RuleBodyPositionV2::NegatedTruth {
                                    RuleCallPolarityV2::Negated
                                } else {
                                    RuleCallPolarityV2::Positive
                                },
                                expr.location().clone(),
                            ));
                        }
                        let aggregate = expr.intrinsic_call_aggregation().ok_or_else(|| {
                            RuleBodyClassificationErrorV2 {
                                location: Some(expr.location().clone()),
                                reason: "rule call lacks typed intrinsic aggregation",
                            }
                        })? == AggregationClassV1::Aggregate;
                        for argument in arguments.iter().rev() {
                            if let TypedCallArgumentV2::Expression(key) = argument {
                                stack.push(child(
                                    *key,
                                    if aggregate || position == RuleBodyPositionV2::Aggregated {
                                        RuleBodyPositionV2::Aggregated
                                    } else {
                                        RuleBodyPositionV2::OpaqueScalar
                                    },
                                ));
                            }
                        }
                    }
                    TypedExprShapeV2::Compare { lhs, rhs, .. }
                    | TypedExprShapeV2::Binary { lhs, rhs, .. } => {
                        stack.push(child(*rhs, RuleBodyPositionV2::OpaqueScalar));
                        stack.push(child(*lhs, RuleBodyPositionV2::OpaqueScalar));
                    }
                    TypedExprShapeV2::Unary { operand, .. } => {
                        stack.push(child(*operand, RuleBodyPositionV2::OpaqueScalar));
                    }
                    TypedExprShapeV2::Path { base, .. } => {
                        stack.push(child(*base, RuleBodyPositionV2::OpaqueScalar));
                    }
                    TypedExprShapeV2::Record(fields) => {
                        for field in fields.iter().rev() {
                            stack.push(child(field.value(), RuleBodyPositionV2::OpaqueScalar));
                        }
                    }
                    TypedExprShapeV2::List(items) => {
                        for item in items.iter().rev() {
                            stack.push(child(*item, RuleBodyPositionV2::OpaqueScalar));
                        }
                    }
                    TypedExprShapeV2::LogicalQuantifier { body, .. } => {
                        stack.push(child(*body, RuleBodyPositionV2::OpaqueScalar));
                    }
                    TypedExprShapeV2::RelationExists { relation_root, .. } => {
                        stack.push(RuleBodyNodeV2::Relation(*relation_root));
                    }
                    TypedExprShapeV2::Literal(_) | TypedExprShapeV2::Binding(_) => {}
                }
            }
            RuleBodyNodeV2::Relation(key) => {
                if !seen_relations.insert(key) {
                    continue;
                }
                let Some(relation) = relation(key) else {
                    return Err(RuleBodyClassificationErrorV2 {
                        location: None,
                        reason: "rule body typed nested relation is missing",
                    });
                };
                for input in relation.inputs().rev() {
                    stack.push(RuleBodyNodeV2::Relation(input.key()));
                }
                for key in relation.expressions().iter().rev() {
                    stack.push(RuleBodyNodeV2::Expression(
                        *key,
                        RuleBodyPositionV2::OpaqueScalar,
                        false,
                    ));
                }
                if let Some(predicate) = relation.predicate() {
                    stack.push(RuleBodyNodeV2::Expression(
                        predicate.expression().key(),
                        RuleBodyPositionV2::OpaqueScalar,
                        false,
                    ));
                }
            }
        }
    }
    Ok(calls)
}

/// Semantic polarity of one resolved rule-body call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleCallPolarityV2 {
    /// Positive dependency.
    Positive,
    /// Dependency below logical negation.
    Negated,
    /// Dependency below aggregate evaluation.
    Aggregated,
}

/// One exact resolved call edge retained for SCC/stratification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRuleCallV2 {
    call: SourceQualifiedNodeIdV2,
    target: ResolvedCallTargetV2,
    polarity: RuleCallPolarityV2,
    location: CheckedLocatedSpanV2,
}

impl ResolvedRuleCallV2 {
    /// Construct a resolved call edge.
    #[must_use]
    pub(crate) fn new(
        call: SourceQualifiedNodeIdV2,
        target: ResolvedCallTargetV2,
        polarity: RuleCallPolarityV2,
        location: CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            call,
            target,
            polarity,
            location,
        }
    }
    /// Exact checked HIR call-node identity.
    #[must_use]
    pub const fn call(&self) -> SourceQualifiedNodeIdV2 {
        self.call
    }
    /// Resolved types-owned call target.
    #[must_use]
    pub const fn target(&self) -> &ResolvedCallTargetV2 {
        &self.target
    }
    /// Dependency polarity.
    #[must_use]
    pub const fn polarity(&self) -> RuleCallPolarityV2 {
        self.polarity
    }
    /// Exact authored call location.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// One fully bound and typed unary source rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRuleV2 {
    item_id: SemanticItemIdV2,
    predicate_id: ResolvedPredicateIdV2,
    head_term: TypedExprV2,
    body: TypedBoolExprV2,
    calls: Vec<ResolvedRuleCallV2>,
}

impl ResolvedRuleV2 {
    /// Seal one rule. Call order is retained as authored; logic derives graph
    /// order independently from resolved IDs.
    pub(crate) fn try_new(
        item_id: SemanticItemIdV2,
        expected_head: SourceQualifiedNodeIdV2,
        expected_body: SourceQualifiedNodeIdV2,
        predicate_id: ResolvedPredicateIdV2,
        head_term: TypedExprV2,
        body: TypedBoolExprV2,
        calls: Vec<ResolvedRuleCallV2>,
    ) -> Result<Self, &'static str> {
        if head_term.key() != expected_head
            || body.expression().key() != expected_body
            || expected_head.source_id != item_id.source_id()
            || expected_body.source_id != item_id.source_id()
            || calls.iter().any(|call| {
                call.call.source_id != item_id.source_id()
                    || call.location.location().source_id != item_id.source_id()
                    || !matches!(call.target, ResolvedCallTargetV2::Predicate(_))
            })
        {
            return Err("rule semantic facts differ from the checked rule body");
        }
        Ok(Self {
            item_id,
            predicate_id,
            head_term,
            body,
            calls,
        })
    }
    /// Item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// Predeclared predicate identity.
    #[must_use]
    pub const fn predicate_id(&self) -> &ResolvedPredicateIdV2 {
        &self.predicate_id
    }
    /// Unary head term.
    #[must_use]
    pub const fn head_term(&self) -> &TypedExprV2 {
        &self.head_term
    }
    /// Bool-proven body.
    #[must_use]
    pub const fn body(&self) -> &TypedBoolExprV2 {
        &self.body
    }
    /// Ordered resolved calls.
    #[must_use]
    pub fn calls(&self) -> &[ResolvedRuleCallV2] {
        &self.calls
    }

    /// Test-only minting seam: replace the resolved call vector WITHOUT
    /// revalidation so adversarial-document rails can forge rule facts
    /// (nonlocal/transplanted target, polarity flip, unknown call id,
    /// foreign location, dropped/extra call) against the real Logic
    /// validator. Compiled for unit tests or under the `test` feature; never
    /// part of a normal build.
    #[cfg(any(test, feature = "test"))]
    #[doc(hidden)]
    #[must_use]
    pub fn forged_with_calls(mut self, calls: Vec<ResolvedRuleCallV2>) -> Self {
        self.calls = calls;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        reference_rule_document_v2, ReferenceRuleFixtureV2, SECONDARY_REFERENCE_SOURCE_V2,
    };

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "owner test propagates fixture errors and asserts sealed source ownership"
    )]
    fn rule_seal_rejects_foreign_call_source() -> Result<(), String> {
        let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::PositiveSelfRecursion)?;
        let rule = fixture
            .document()
            .rules()
            .next()
            .ok_or("fixture must retain one source rule")?;
        let seal = |calls: Vec<_>| {
            ResolvedRuleV2::try_new(
                rule.item_id(),
                rule.head_term().key(),
                rule.body().expression().key(),
                rule.predicate_id().clone(),
                rule.head_term().clone(),
                rule.body().clone(),
                calls,
            )
        };
        assert_eq!(seal(rule.calls().to_vec()), Ok(rule.clone()));
        let mut foreign_calls = rule.calls().to_vec();
        let [foreign_call] = foreign_calls.as_mut_slice() else {
            return Err("fixture must retain exactly one source call".into());
        };
        foreign_call.call.source_id = SECONDARY_REFERENCE_SOURCE_V2;
        assert_eq!(
            seal(foreign_calls),
            Err("rule semantic facts differ from the checked rule body"),
        );
        Ok(())
    }
}
