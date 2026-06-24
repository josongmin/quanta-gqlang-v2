//! Source-level `exists` / `not exists` semantics (ticket 05).
//!
//! This module owns the SOURCE-LEVEL MEANING of existential and anti-existential
//! checks; the normalized algebra TARGET forms (`SemiJoin` / `AntiJoin`) are
//! owned by ticket 04 (`D12`). A source `exists` over a relation correlated to an
//! outer row means "keep the outer row when the inner relation is non-empty",
//! which is exactly a [`quanta_gqlang_v2_algebra::AlgebraOpV1::SemiJoin`]; a
//! `not exists` is the complementary [`quanta_gqlang_v2_algebra::AlgebraOpV1::AntiJoin`].
//!
//! Only the source vocabulary `exists` / `not exists` is recognized; any other
//! quantifier keyword rejects with [`LogicDiagnosticV1::UnknownQuantifier`] so
//! there is no silent downgrade (`D10`).

use quanta_gqlang_v2_algebra::{AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, SemiJoinV1};
use quanta_gqlang_v2_hir::{BoolExprHirV1, SpanV1};

use crate::diagnostic::LogicDiagnosticV1;

/// The source-level polarity of an existence check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExistencePolarityV1 {
    /// `exists`: keep the outer row when the inner relation is non-empty.
    /// Lowers to a semijoin.
    Exists,
    /// `not exists`: keep the outer row when the inner relation is empty.
    /// Lowers to an antijoin.
    NotExists,
}

impl ExistencePolarityV1 {
    /// Stable keyword for explain/diagnostics.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Exists => "exists",
            Self::NotExists => "not exists",
        }
    }

    /// Parse a source quantifier keyword into a polarity.
    ///
    /// `"exists"` -> [`ExistencePolarityV1::Exists`]; `"not exists"` or
    /// `"not_exists"` -> [`ExistencePolarityV1::NotExists`]. Any other keyword is
    /// unknown and rejects.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "exists" => Some(Self::Exists),
            "not exists" | "not_exists" => Some(Self::NotExists),
            _ => None,
        }
    }
}

/// A source-level existence check over a relation, correlated to an outer row by
/// a predicate.
///
/// `outer` is the relation whose rows are kept-or-dropped (the left side of the
/// resulting semi/anti join); `inner` is the existence-tested relation (the right
/// side, which contributes no columns). `predicate` is the correlation condition
/// between them; `None` is an uncorrelated existence check (empty/non-empty of
/// the whole inner relation).
#[derive(Clone, Debug, PartialEq)]
pub struct ExistenceCheckV1 {
    /// `exists` vs `not exists`.
    pub polarity: ExistencePolarityV1,
    /// The outer relation whose rows are filtered by the check.
    pub outer: AlgebraNodeV1,
    /// The existence-tested inner relation.
    pub inner: AlgebraNodeV1,
    /// The correlation predicate, if any.
    pub predicate: Option<BoolExprHirV1>,
    /// Span of the source quantifier construct.
    pub span: SpanV1,
}

impl ExistenceCheckV1 {
    /// Build an existence check from a source quantifier keyword, rejecting an
    /// unknown keyword with a typed diagnostic.
    pub fn from_keyword(
        keyword: &str,
        outer: AlgebraNodeV1,
        inner: AlgebraNodeV1,
        predicate: Option<BoolExprHirV1>,
        span: SpanV1,
    ) -> Result<Self, LogicDiagnosticV1> {
        let polarity = ExistencePolarityV1::from_keyword(keyword).ok_or_else(|| {
            LogicDiagnosticV1::UnknownQuantifier {
                keyword: keyword.to_owned(),
                span,
            }
        })?;
        Ok(Self {
            polarity,
            outer,
            inner,
            predicate,
            span,
        })
    }

    /// Lower this source-level check into its normalized algebra target.
    ///
    /// `exists` -> [`AlgebraOpV1::SemiJoin`]; `not exists` -> [`AlgebraOpV1::AntiJoin`].
    /// The resulting node's output columns and cardinality are exactly the outer
    /// input's (the inner relation contributes no columns and no multiplicity),
    /// matching ticket 04's semi/anti-join contract.
    #[must_use]
    pub fn lower(self) -> AlgebraNodeV1 {
        let columns = self.outer.columns.clone();
        let cardinality = self.outer.cardinality;
        let origin = self.outer.origin;
        let left = Box::new(self.outer);
        let right = Box::new(self.inner);
        let predicate = self.predicate.map(Box::new);
        let op = match self.polarity {
            ExistencePolarityV1::Exists => AlgebraOpV1::SemiJoin(SemiJoinV1 {
                left,
                right,
                predicate,
            }),
            ExistencePolarityV1::NotExists => AlgebraOpV1::AntiJoin(AntiJoinV1 {
                left,
                right,
                predicate,
            }),
        };
        AlgebraNodeV1::new(origin, self.span, columns, cardinality, op)
    }
}
