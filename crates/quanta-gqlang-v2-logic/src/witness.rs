//! Bounded witness / path materialization contract (ticket 05).
//!
//! Cites `D31`: recursive witness/path materialization requires explicit
//! boundedness, cycle-dedup identity, and tie-break rules; otherwise the shape
//! REJECTS. Reachability TRUTH (a boolean "is there a path") and witness
//! MATERIALIZATION (enumerate one-or-more concrete typed paths) are related but
//! DISTINCT semantics (ticket pinned-semantics 5): a truth query needs no bound,
//! but materializing path witnesses over a recursive relation does.
//!
//! A well-formed [`WitnessContractV1`] carries:
//!
//! 1. a DEDUP IDENTITY ([`DedupIdentityV1`]) — what makes two enumerated
//!    witnesses "the same" so cycles do not re-emit equivalents forever;
//! 2. a finite MAX WITNESS COUNT — the hard cap on emitted witnesses;
//! 3. an optional PATH-LENGTH BOUND — mandatory when the source relation can
//!    cycle, so enumeration cannot revisit nodes without end;
//! 4. a canonical TIE-BREAK ([`WitnessTieBreakV1`]) — deterministic ordering for
//!    equivalent-cost witnesses so the kept set is reproducible.
//!
//! Any missing-or-degenerate bound rejects with
//! [`LogicDiagnosticV1::UnboundedWitness`].

use quanta_gqlang_v2_hir::SpanV1;

use crate::diagnostic::{LogicDiagnosticV1, UnboundedWitnessReasonV1};

/// Whether reachability over the underlying relation is bounded by structure or
/// can cycle (so witness enumeration could revisit nodes forever).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationShapeV1 {
    /// The relation is acyclic (a DAG): every path is finite by structure, so a
    /// path-length bound is OPTIONAL.
    Acyclic,
    /// The relation can cycle: a path-length bound is MANDATORY to keep witness
    /// enumeration finite.
    Cyclic,
}

/// The identity under which two enumerated witnesses are considered equal for
/// cycle-dedup. Materialization keeps at most one witness per dedup-identity
/// class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedupIdentityV1 {
    /// Two witnesses are the same iff their endpoint pair (source, sink) is the
    /// same. Collapses all paths between a pair to one witness.
    Endpoints,
    /// Two witnesses are the same iff their full ordered node sequence is the
    /// same. Distinct routes between the same endpoints are distinct witnesses.
    NodeSequence,
    /// Two witnesses are the same iff their unordered node set is the same.
    /// Distinguishes which nodes were visited but not their order.
    NodeSet,
}

impl DedupIdentityV1 {
    /// Stable keyword for explain/diagnostics.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Endpoints => "endpoints",
            Self::NodeSequence => "node_sequence",
            Self::NodeSet => "node_set",
        }
    }
}

/// The canonical tie-break for equivalent witnesses, so the kept set under a cap
/// is deterministic and reproducible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessTieBreakV1 {
    /// Prefer the shortest path; break further ties by lexical node order.
    ShortestThenLexical,
    /// Prefer the lexically-smallest node sequence outright.
    Lexical,
}

impl WitnessTieBreakV1 {
    /// Stable keyword for explain/diagnostics.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::ShortestThenLexical => "shortest_then_lexical",
            Self::Lexical => "lexical",
        }
    }
}

/// A request to materialize path witnesses over a (recursive) relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessRequestV1 {
    /// Whether the underlying relation can cycle.
    pub shape: RelationShapeV1,
    /// Dedup identity for equivalent witnesses.
    pub dedup: DedupIdentityV1,
    /// Maximum number of witnesses to materialize. `None` is UNBOUNDED and
    /// rejects; `Some(0)` is degenerate and rejects.
    pub max_witnesses: Option<u64>,
    /// Optional path-length bound (max edges in an enumerated path). Mandatory
    /// when `shape` is [`RelationShapeV1::Cyclic`]. `Some(0)` is degenerate
    /// (zero edges traverse nothing) and rejects.
    pub max_path_length: Option<u64>,
    /// Canonical tie-break for equivalent witnesses.
    pub tie_break: WitnessTieBreakV1,
    /// Span of the witness/path request.
    pub span: SpanV1,
}

/// A validated, BOUNDED witness contract.
///
/// Only constructable through [`WitnessContractV1::bind`], which enforces the
/// boundedness rules; the fields are therefore guaranteed finite. The presence
/// of this value is itself the proof that the witness shape is admissible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessContractV1 {
    /// The relation shape this contract was bound against.
    pub shape: RelationShapeV1,
    /// Dedup identity.
    pub dedup: DedupIdentityV1,
    /// The finite, non-zero max-witness cap.
    pub max_witnesses: u64,
    /// The path-length bound (always present for cyclic relations; may be
    /// present for acyclic ones too).
    pub max_path_length: Option<u64>,
    /// Canonical tie-break.
    pub tie_break: WitnessTieBreakV1,
}

impl WitnessContractV1 {
    /// Bind a request into a bounded contract, rejecting unbounded / degenerate
    /// shapes with a typed [`LogicDiagnosticV1::UnboundedWitness`].
    ///
    /// Rejection order is deterministic: a missing cap, then a zero cap, then a
    /// missing path-length bound for a cyclic relation, then a degenerate
    /// (zero) path-length bound.
    pub fn bind(request: &WitnessRequestV1) -> Result<Self, LogicDiagnosticV1> {
        let max_witnesses = match request.max_witnesses {
            None => {
                return Err(LogicDiagnosticV1::UnboundedWitness {
                    reason: UnboundedWitnessReasonV1::MissingMaxWitnesses,
                    span: request.span,
                });
            }
            Some(0) => {
                return Err(LogicDiagnosticV1::UnboundedWitness {
                    reason: UnboundedWitnessReasonV1::ZeroMaxWitnesses,
                    span: request.span,
                });
            }
            Some(cap) => cap,
        };

        if request.shape == RelationShapeV1::Cyclic && request.max_path_length.is_none() {
            return Err(LogicDiagnosticV1::UnboundedWitness {
                reason: UnboundedWitnessReasonV1::MissingPathLengthBound,
                span: request.span,
            });
        }

        // A supplied path-length bound of zero is degenerate: zero edges can
        // never connect two distinct endpoints, so it is an ill-formed bound
        // (parallel to a zero max-witness cap). Reject regardless of shape so a
        // `depth = 0` cannot silently admit a non-traversing contract.
        if request.max_path_length == Some(0) {
            return Err(LogicDiagnosticV1::UnboundedWitness {
                reason: UnboundedWitnessReasonV1::ZeroPathLengthBound,
                span: request.span,
            });
        }

        Ok(Self {
            shape: request.shape,
            dedup: request.dedup,
            max_witnesses,
            max_path_length: request.max_path_length,
            tie_break: request.tie_break,
        })
    }
}
