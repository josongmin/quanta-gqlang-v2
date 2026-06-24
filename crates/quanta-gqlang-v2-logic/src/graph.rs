//! FIND graph-query SOURCE semantics (ticket 05).
//!
//! This module owns the SOURCE-LEVEL MEANING of the FIND family carried by
//! [`quanta_gqlang_v2_hir::GraphQueryHirV1`]: each subkind
//! (reachability / flow / slice / taint / path / points_to / summary) is a logic
//! graph query whose FRONTEND CONTRACT this layer classifies. We do NOT execute
//! traversal here — the FIND family's runtime (real taint propagation, concrete
//! path enumeration) is downstream (semantica / D7 / D14). What this layer owns
//! and proves, fail-closed:
//!
//! 1. ENDPOINT-PRESENCE LEGALITY — each subkind REQUIRES a specific set of
//!    endpoints (e.g. `taint` needs a `source` and a `sink`; `points_to` needs a
//!    `reference` and a `pointee`). A missing required endpoint rejects with
//!    [`LogicDiagnosticV1::GraphEndpointMissing`] (`D10`: no silent downgrade).
//! 2. BOUNDED-WITNESS LEGALITY — a FIND materializes path witnesses
//!    (`result_kind = path_witness`), so the bounded-witness contract this crate
//!    already owns ([`crate::WitnessContractV1`]) applies. Subkinds whose
//!    underlying relation can CYCLE (call/flow graphs) require a path-length
//!    bound; without an explicit `depth` and without bounding waypoints
//!    (`via` / `through`) the shape is UNBOUNDED and rejects with
//!    [`LogicDiagnosticV1::UnboundedWitness`] (`D31`). A `points_to` query
//!    resolves aliases over an acyclic treatment and needs no path bound.
//!
//! The result is a [`GraphQueryContractV1`]: the validated, bounded contract
//! whose mere existence proves the FIND shape is admissible at the frontend.

use quanta_gqlang_v2_hir::{
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, SpanV1,
};

use crate::diagnostic::LogicDiagnosticV1;
use crate::witness::{
    DedupIdentityV1, RelationShapeV1, WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};

/// Stable keyword for a FIND subkind, used in diagnostics and explain output.
///
/// Crate-internal: rendered into [`crate::LogicDiagnosticV1`] messages. Not part
/// of the public surface (no downstream crate consumes it), so it stays
/// `pub(crate)` to satisfy `unreachable_pub`.
#[must_use]
pub(crate) const fn graph_query_keyword(kind: GraphQueryKindV1) -> &'static str {
    match kind {
        GraphQueryKindV1::Reachability => "reachability",
        GraphQueryKindV1::Flow => "flow",
        GraphQueryKindV1::Slice => "slice",
        GraphQueryKindV1::Taint => "taint",
        GraphQueryKindV1::Path => "path",
        GraphQueryKindV1::PointsTo => "points_to",
        GraphQueryKindV1::Summary => "summary",
    }
}

/// The endpoint roles a FIND subkind can require. Mirrors the open endpoint
/// slots on [`GraphEndpointsV1`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphEndpointRoleV1 {
    /// The flow / reachability source.
    Source,
    /// The flow / reachability sink.
    Sink,
    /// The slice / summary anchor.
    Anchor,
    /// The points-to reference.
    Reference,
    /// The points-to pointee.
    Pointee,
    /// The summary callable.
    Callable,
}

impl GraphEndpointRoleV1 {
    /// Every endpoint role, in deterministic diagnostic order. Used to detect
    /// endpoints a subkind does not accept.
    const ALL: [Self; 6] = [
        Self::Source,
        Self::Sink,
        Self::Anchor,
        Self::Reference,
        Self::Pointee,
        Self::Callable,
    ];

    /// Stable keyword for explain / diagnostics.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Sink => "sink",
            Self::Anchor => "anchor",
            Self::Reference => "reference",
            Self::Pointee => "pointee",
            Self::Callable => "callable",
        }
    }

    /// Whether the endpoint of this role is present in `endpoints`.
    #[must_use]
    fn is_present(self, endpoints: &GraphEndpointsV1) -> bool {
        match self {
            Self::Source => endpoints.source.is_some(),
            Self::Sink => endpoints.sink.is_some(),
            Self::Anchor => endpoints.anchor.is_some(),
            Self::Reference => endpoints.reference.is_some(),
            Self::Pointee => endpoints.pointee.is_some(),
            Self::Callable => endpoints.callable.is_some(),
        }
    }
}

/// The validated, BOUNDED contract for a classified FIND graph query.
///
/// Constructable only through [`GraphQuerySemanticsV1::classify`], which enforces
/// endpoint-presence legality and binds the bounded-witness contract; its
/// presence is the proof that the FIND shape is admissible at the frontend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphQueryContractV1 {
    /// The FIND subkind this contract classifies.
    pub kind: GraphQueryKindV1,
    /// The bounded witness/path materialization contract.
    pub witness: WitnessContractV1,
    /// Span of the originating graph query.
    pub span: SpanV1,
}

/// FIND graph-query source semantics: per-subkind endpoint legality + bounded
/// witness classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphQuerySemanticsV1;

impl GraphQuerySemanticsV1 {
    /// The endpoints a subkind REQUIRES, in deterministic check order. The first
    /// missing one rejects, so the order is part of the diagnostic contract.
    #[must_use]
    fn required_endpoints(kind: GraphQueryKindV1) -> &'static [GraphEndpointRoleV1] {
        use GraphEndpointRoleV1::{Anchor, Callable, Pointee, Reference, Sink, Source};
        match kind {
            // Reachability / flow / taint / path are source -> sink traversals.
            GraphQueryKindV1::Reachability
            | GraphQueryKindV1::Flow
            | GraphQueryKindV1::Taint
            | GraphQueryKindV1::Path => &[Source, Sink],
            // A slice is anchored at a program point.
            GraphQueryKindV1::Slice => &[Anchor],
            // Points-to relates a reference to its pointee.
            GraphQueryKindV1::PointsTo => &[Reference, Pointee],
            // A summary is over a callable.
            GraphQueryKindV1::Summary => &[Callable],
        }
    }

    /// Whether a subkind traverses a relation that can CYCLE (so witness
    /// enumeration needs a path-length bound) or an acyclic treatment.
    ///
    /// Reachability / flow / slice / taint / path / summary all traverse the
    /// program's call/flow graph, which can cycle (recursion, loops). Points-to
    /// is alias resolution over an acyclic-by-treatment relation, so it needs no
    /// path bound.
    #[must_use]
    const fn relation_shape(kind: GraphQueryKindV1) -> RelationShapeV1 {
        match kind {
            GraphQueryKindV1::PointsTo => RelationShapeV1::Acyclic,
            GraphQueryKindV1::Reachability
            | GraphQueryKindV1::Flow
            | GraphQueryKindV1::Slice
            | GraphQueryKindV1::Taint
            | GraphQueryKindV1::Path
            | GraphQueryKindV1::Summary => RelationShapeV1::Cyclic,
        }
    }

    /// The dedup identity for a subkind's witnesses.
    ///
    /// `path` / `flow` / `taint` distinguish distinct ROUTES between the same
    /// endpoints (each concrete path is its own evidence), so they dedup by the
    /// full node sequence. Reachability collapses to its endpoint pair (the
    /// truth is "is there a path", one witness per pair). Slice / summary /
    /// points_to dedup by the node SET they touch.
    #[must_use]
    const fn dedup_identity(kind: GraphQueryKindV1) -> DedupIdentityV1 {
        match kind {
            GraphQueryKindV1::Path | GraphQueryKindV1::Flow | GraphQueryKindV1::Taint => {
                DedupIdentityV1::NodeSequence
            }
            GraphQueryKindV1::Reachability => DedupIdentityV1::Endpoints,
            GraphQueryKindV1::Slice | GraphQueryKindV1::Summary | GraphQueryKindV1::PointsTo => {
                DedupIdentityV1::NodeSet
            }
        }
    }

    /// Default max-witness cap when the query supplies no explicit one. A finite,
    /// non-zero default keeps the frontend contract bounded; downstream may
    /// tighten it.
    const DEFAULT_MAX_WITNESSES: u64 = 64;

    /// Classify a FIND graph query into a bounded [`GraphQueryContractV1`],
    /// rejecting illegal endpoint shapes and unbounded witness families with a
    /// typed [`LogicDiagnosticV1`].
    ///
    /// This is a FRONTEND classification: it proves legality + boundedness and
    /// does NOT execute traversal.
    pub fn classify(query: &GraphQueryHirV1) -> Result<GraphQueryContractV1, LogicDiagnosticV1> {
        // 1. Endpoint-presence legality (deterministic order).
        let allowed = Self::required_endpoints(query.kind);
        for &role in allowed {
            if !role.is_present(&query.endpoints) {
                return Err(LogicDiagnosticV1::GraphEndpointMissing {
                    kind: query.kind,
                    role,
                    span: query.span,
                });
            }
        }

        // 1b. Endpoint-conflict legality: an endpoint the subkind does not
        //     accept (e.g. a `slice` carrying a `source`/`sink`) fails closed so
        //     the seed cannot encode two traversals at once (`D10`).
        for &role in &GraphEndpointRoleV1::ALL {
            if role.is_present(&query.endpoints) && !allowed.contains(&role) {
                return Err(LogicDiagnosticV1::GraphEndpointUnexpected {
                    kind: query.kind,
                    role,
                    span: query.span,
                });
            }
        }

        // 2. Bounded-witness legality. Derive the witness request from the
        //    subkind's relation shape and the query's structural bounds, then
        //    delegate to the witness contract this crate already owns so the
        //    reject codes are shared.
        let shape = Self::relation_shape(query.kind);
        let max_path_length =
            Self::path_length_bound(query.kind, query.span, shape, &query.constraints)?;
        let request = WitnessRequestV1 {
            shape,
            dedup: Self::dedup_identity(query.kind),
            max_witnesses: Some(Self::DEFAULT_MAX_WITNESSES),
            max_path_length,
            tie_break: WitnessTieBreakV1::ShortestThenLexical,
            span: query.span,
        };
        let witness = WitnessContractV1::bind(&request)?;

        Ok(GraphQueryContractV1 {
            kind: query.kind,
            witness,
            span: query.span,
        })
    }

    /// The path-length bound a query supplies, if any.
    ///
    /// An explicit `depth` is the direct bound. Failing that, bounding waypoints
    /// (`via` / `through`) constrain enumeration to pass through fixed points, so
    /// the count of waypoint hops serves as a structural bound. An acyclic
    /// relation needs no bound regardless. The returned `None` over a cyclic
    /// relation is exactly what [`WitnessContractV1::bind`] rejects.
    ///
    /// `depth` and waypoints are TWO COMPETING binding strategies. Supplying both
    /// over a cyclic relation is rejected with
    /// [`LogicDiagnosticV1::GraphConflictingPathBound`] rather than silently
    /// letting `depth` win and dropping the waypoint constraint: a silent
    /// downgrade would violate `D10`, and accepting a seed that means two
    /// different things at once would violate `D26`. (An ACYCLIC relation derives
    /// no bound from waypoints, so there is no competing strategy to conflict
    /// with and `depth` simply applies.)
    fn path_length_bound(
        kind: GraphQueryKindV1,
        span: SpanV1,
        shape: RelationShapeV1,
        constraints: &GraphConstraintsV1,
    ) -> Result<Option<u64>, LogicDiagnosticV1> {
        let waypoints = constraints.via.len() + constraints.through.len();
        if let Some(depth) = constraints.depth {
            if shape == RelationShapeV1::Cyclic && waypoints > 0 {
                return Err(LogicDiagnosticV1::GraphConflictingPathBound { kind, span });
            }
            return Ok(Some(u64::from(depth)));
        }
        if shape == RelationShapeV1::Acyclic {
            // No structural bound needed; acyclic enumeration is finite.
            return Ok(None);
        }
        if waypoints > 0 {
            // Each waypoint is a mandatory hop; the witness must pass through all
            // of them, so the waypoint count bounds enumeration length. (Saturate
            // the rare overflow into the cap rather than wrapping.)
            return Ok(Some(u64::try_from(waypoints).unwrap_or(u64::MAX)));
        }
        Ok(None)
    }
}
