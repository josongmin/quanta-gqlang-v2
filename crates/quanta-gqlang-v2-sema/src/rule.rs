//! Resolved unary rule contracts consumed by the logic owner.

use crate::{SemanticItemIdV2, TypedBoolExprV2, TypedExprV2};
use quanta_gqlang_v2_hir::CheckedLocatedSpanV2;
use quanta_gqlang_v2_types::{
    ResolvedCallTargetV2, ResolvedPredicateIdV2, SourceQualifiedNodeIdV2,
};

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
        expected_calls: &[(
            SourceQualifiedNodeIdV2,
            ResolvedCallTargetV2,
            RuleCallPolarityV2,
            CheckedLocatedSpanV2,
        )],
        predicate_id: ResolvedPredicateIdV2,
        head_term: TypedExprV2,
        body: TypedBoolExprV2,
        calls: Vec<ResolvedRuleCallV2>,
    ) -> Result<Self, &'static str> {
        if head_term.key() != expected_head
            || body.expression().key() != expected_body
            || expected_head.source_id != item_id.source_id()
            || expected_body.source_id != item_id.source_id()
            || calls
                .iter()
                .map(|call| {
                    (
                        call.call,
                        call.target.clone(),
                        call.polarity,
                        call.location.clone(),
                    )
                })
                .collect::<Vec<_>>()
                != expected_calls
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
    /// validator. Compiled only under the `test` feature; never part of the
    /// public contract and never reachable from a normal build.
    #[cfg(feature = "test")]
    #[doc(hidden)]
    #[must_use]
    pub(crate) fn forged_with_calls(mut self, calls: Vec<ResolvedRuleCallV2>) -> Self {
        self.calls = calls;
        self
    }
}
