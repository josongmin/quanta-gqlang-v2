//! Recursive-aggregate legality: the explicit admit/reject matrix (ticket 05).
//!
//! Per `D27`, recursive aggregates are ILLEGAL unless their legality and
//! fixpoint behaviour are explicitly specified here. A general recursive
//! aggregate (e.g. `count`/`sum` folding a predicate that recurses through the
//! aggregate) has no monotone least-fixpoint, so it rejects. A FIXED, explicit
//! admit list carries the narrow shapes that DO have a well-defined recursive
//! fixpoint (semiring-style monotone folds over a bounded lattice).
//!
//! The matrix is intentionally CLOSED: an aggregate is admitted only when it is
//! on the admit list AND its recursion is positive (non-negated). Everything
//! else rejects with [`LogicDiagnosticV1::RecursiveAggregate`].

use std::collections::BTreeSet;

use quanta_gqlang_v2_hir::SpanV1;

use crate::diagnostic::LogicDiagnosticV1;

/// The outcome of consulting the recursive-aggregate matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateLegalityV1 {
    /// The aggregate is admitted over recursion (it has a monotone
    /// least-fixpoint over a bounded lattice).
    Admitted,
    /// The aggregate is rejected over recursion.
    Rejected,
}

/// A request to fold an aggregate over a (possibly recursive) predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursiveAggregateV1 {
    /// The aggregate function name as written (e.g. `min`, `max`, `count`).
    pub function: String,
    /// The predicate being folded over.
    pub predicate: String,
    /// Whether that predicate is recursive (sits in a recursive SCC). A
    /// NON-recursive aggregate is always fine and is admitted trivially.
    pub recursive: bool,
    /// Span of the aggregate call.
    pub span: SpanV1,
}

/// The closed admit/reject matrix for recursive aggregates.
///
/// The admit list defaults to the monotone lattice folds `min` and `max`, which
/// have a least-fixpoint over a bounded value domain (each step is monotone and
/// converges). `count` / `sum` / `avg` are NOT admitted by default because they
/// are not monotone-convergent over an unbounded recursion. Callers may extend
/// the admit list (e.g. a host that proves boundedness) but must do so
/// explicitly — there is no implicit admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursiveAggregateMatrixV1 {
    admitted: BTreeSet<String>,
}

impl Default for RecursiveAggregateMatrixV1 {
    fn default() -> Self {
        let mut admitted = BTreeSet::new();
        admitted.insert("min".to_owned());
        admitted.insert("max".to_owned());
        Self { admitted }
    }
}

impl RecursiveAggregateMatrixV1 {
    /// The default matrix: `min` / `max` admitted, everything else rejected.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty matrix that admits NO recursive aggregate (strictest closure).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            admitted: BTreeSet::new(),
        }
    }

    /// A matrix from an explicit admit list of aggregate function names.
    #[must_use]
    pub fn with_admitted(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            admitted: names.into_iter().map(Into::into).collect(),
        }
    }

    /// Whether a function name is on the admit list.
    #[must_use]
    pub fn admits(&self, function: &str) -> bool {
        self.admitted.contains(function)
    }

    /// Classify a recursive-aggregate request against the matrix.
    ///
    /// A non-recursive aggregate is [`AggregateLegalityV1::Admitted`]. A
    /// recursive aggregate is admitted only when its function is on the admit
    /// list, else rejected.
    #[must_use]
    pub fn classify(&self, request: &RecursiveAggregateV1) -> AggregateLegalityV1 {
        if !request.recursive || self.admits(&request.function) {
            AggregateLegalityV1::Admitted
        } else {
            AggregateLegalityV1::Rejected
        }
    }

    /// Check a recursive-aggregate request, returning the typed rejection
    /// diagnostic when the matrix rejects it.
    pub fn check(&self, request: &RecursiveAggregateV1) -> Result<(), LogicDiagnosticV1> {
        match self.classify(request) {
            AggregateLegalityV1::Admitted => Ok(()),
            AggregateLegalityV1::Rejected => Err(LogicDiagnosticV1::RecursiveAggregate {
                function: request.function.clone(),
                predicate: request.predicate.clone(),
                span: request.span,
            }),
        }
    }
}
