//! FIND graph-query contract over the sealed resolved FIND input (ticket 06,
//! lane W3-B).
//!
//! This module owns the SOURCE-LEVEL MEANING of the FIND family carried by
//! [`quanta_gqlang_v2_sema::ResolvedGraphQueryV2`]: each subkind (reachability /
//! flow / slice / taint / path / points_to / summary) is a graph query whose
//! FRONTEND CONTRACT this layer seals. Traversal is NOT executed here — the FIND
//! family's runtime (concrete taint propagation, path enumeration) is
//! downstream. What this layer owns and proves, fail-closed:
//!
//! 1. ENDPOINT-ROLE LEGALITY — each subkind REQUIRES a specific set of resolved
//!    endpoint roles (e.g. `taint` needs a `source` and a `sink`; `points_to`
//!    needs a `reference` and a `pointee`). A missing required endpoint rejects
//!    with [`LogicDiagnosticV1::GraphEndpointMissing`], and an endpoint the
//!    subkind does not accept (e.g. a `slice` carrying a `source`/`sink`)
//!    rejects with [`LogicDiagnosticV1::GraphEndpointUnexpected`], so one seed
//!    cannot mean two traversals at once (`D10`: no silent downgrade).
//! 2. EXPLICIT BOUNDED DEPTH — every FIND family traverses a relation that can
//!    revisit nodes, so an explicit positive `depth` is mandatory. The retired
//!    `points_to` acyclic exemption is gone and no other subkind inherits it:
//!    this packet admits no unowned acyclic-proof shortcut. `via`/`through`
//!    waypoints are ordered PATH PREDICATES; they are retained, `depth` plus
//!    waypoints is VALID, and waypoints never substitute for `depth`.
//! 3. SEALED WITNESS BOUNDS — the effective witness cap and depth come from the
//!    document's immutable [`quanta_gqlang_v2_sema::GraphWitnessBoundsV2`]; this
//!    layer has no independent default cap and validates the resolved bounds
//!    against the sealed budget.
//!
//! The result is a [`GraphQueryContractV2`]: the validated, bounded contract
//! whose mere existence proves the FIND shape is admissible at the frontend. It
//! is produced ONLY from the exact resolved query; there is no raw-HIR or
//! caller-built ingress.

use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, GraphDirectionV2, GraphQueryKindV1, GraphReferenceRoleV2,
    ResolvedGraphQueryV2, ResolvedGraphReferenceV2, SemanticItemIdV2,
};

use crate::diagnostic::{LogicDiagnosticV1, UnboundedWitnessReasonV1};
use crate::witness::{
    DedupIdentityV1, RelationShapeV1, WitnessBindingV2, WitnessContractV2, WitnessTieBreakV1,
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

/// The endpoint roles a FIND subkind can require. Mirrors the singular resolved
/// FIND reference roles; `via`/`through`/`sanitizer`/`scope`/`window`/`frontier`
/// are path predicates or metadata, never endpoints.
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

    /// The endpoint role a resolved FIND reference fills, if it is an endpoint
    /// role at all.
    const fn from_role(role: GraphReferenceRoleV2) -> Option<Self> {
        match role {
            GraphReferenceRoleV2::Source => Some(Self::Source),
            GraphReferenceRoleV2::Sink => Some(Self::Sink),
            GraphReferenceRoleV2::Anchor => Some(Self::Anchor),
            GraphReferenceRoleV2::Reference => Some(Self::Reference),
            GraphReferenceRoleV2::Pointee => Some(Self::Pointee),
            GraphReferenceRoleV2::Callable => Some(Self::Callable),
            GraphReferenceRoleV2::Via
            | GraphReferenceRoleV2::Through
            | GraphReferenceRoleV2::Sanitizer
            | GraphReferenceRoleV2::Scope
            | GraphReferenceRoleV2::Window
            | GraphReferenceRoleV2::Frontier => None,
        }
    }
}

/// A role-legality violation before it is anchored at a checked location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EndpointViolationV2 {
    /// A required endpoint role was absent.
    Missing(GraphEndpointRoleV1),
    /// An endpoint role the subkind does not accept was present.
    Unexpected(GraphEndpointRoleV1),
}

impl EndpointViolationV2 {
    /// Anchor the violation at the offending FIND item's checked location.
    fn into_diagnostic(
        self,
        kind: GraphQueryKindV1,
        span: CheckedLocatedSpanV2,
    ) -> LogicDiagnosticV1 {
        match self {
            Self::Missing(role) => LogicDiagnosticV1::GraphEndpointMissing { kind, role, span },
            Self::Unexpected(role) => {
                LogicDiagnosticV1::GraphEndpointUnexpected { kind, role, span }
            }
        }
    }
}

/// The endpoints a subkind REQUIRES, in deterministic check order. The first
/// missing one rejects, so this order is part of the diagnostic contract.
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

/// Endpoint-presence and endpoint-conflict legality over the resolved singular
/// endpoint roles.
///
/// Missing roles are checked first (in [`required_endpoints`] order), then
/// unaccepted roles in [`GraphEndpointRoleV1::ALL`] order, so the reported role
/// is deterministic when several endpoints are wrong at once.
pub(crate) fn endpoint_legality(
    kind: GraphQueryKindV1,
    present: &[GraphEndpointRoleV1],
) -> Result<(), EndpointViolationV2> {
    let required = required_endpoints(kind);
    for &role in required {
        if !present.contains(&role) {
            return Err(EndpointViolationV2::Missing(role));
        }
    }
    for &role in &GraphEndpointRoleV1::ALL {
        if present.contains(&role) && !required.contains(&role) {
            return Err(EndpointViolationV2::Unexpected(role));
        }
    }
    Ok(())
}

/// The dedup identity for a subkind's witnesses.
///
/// `path` / `flow` / `taint` distinguish distinct ROUTES between the same
/// endpoints (each concrete path is its own evidence), so they dedup by the
/// full node sequence. Reachability collapses to its endpoint pair (the truth is
/// "is there a path", one witness per pair). Slice / summary / points_to dedup by
/// the node SET they touch.
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

/// Build the boundedness binding for a resolved FIND family.
///
/// Every family binds as [`RelationShapeV1::Cyclic`]: a FIND enumerates paths
/// over a call/flow/alias relation that can revisit nodes, waypoints are path
/// predicates rather than termination bounds, and this packet admits no
/// acyclic-proof shortcut (the retired `points_to` exemption is not inherited by
/// any subkind).
pub(crate) fn witness_binding(
    kind: GraphQueryKindV1,
    depth: u32,
    sealed_max_depth: u32,
    sealed_max_witnesses: u32,
) -> WitnessBindingV2 {
    WitnessBindingV2 {
        shape: RelationShapeV1::Cyclic,
        dedup: dedup_identity(kind),
        max_witnesses: Some(sealed_max_witnesses),
        max_path_length: Some(depth),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        sealed_max_witnesses,
        sealed_max_depth,
    }
}

/// The validated, BOUNDED contract for one resolved FIND graph query.
///
/// Constructable only through [`GraphQueryContractV2::from_resolved`], which
/// enforces endpoint-role legality, the explicit positive depth, and the sealed
/// witness bounds; its presence is the proof that the FIND shape is admissible
/// at the frontend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphQueryContractV2 {
    item_id: SemanticItemIdV2,
    kind: GraphQueryKindV1,
    location: CheckedLocatedSpanV2,
    depth: u32,
    direction: Option<GraphDirectionV2>,
    summary_facets: Vec<String>,
    references: Vec<ResolvedGraphReferenceV2>,
    witness: WitnessContractV2,
}

impl GraphQueryContractV2 {
    /// Seal the exact resolved FIND query into a bounded contract, rejecting
    /// illegal endpoint roles, a missing depth, and any bound that is degenerate
    /// or exceeds the document's sealed witness budget with a typed
    /// [`LogicDiagnosticV1`].
    ///
    /// This is a FRONTEND contract: it proves legality + boundedness and does
    /// NOT execute traversal.
    pub fn from_resolved(query: &ResolvedGraphQueryV2) -> Result<Self, LogicDiagnosticV1> {
        let location = query.location().clone();

        // 1. Endpoint-role legality over the resolved selectors (deterministic
        //    order; missing before unexpected).
        let present = query
            .references()
            .iter()
            .filter_map(|reference| GraphEndpointRoleV1::from_role(reference.role()))
            .collect::<Vec<_>>();
        if let Err(violation) = endpoint_legality(query.kind(), &present) {
            return Err(violation.into_diagnostic(query.kind(), location));
        }

        // 2. An explicit POSITIVE depth for every transitive family. Waypoints
        //    are path predicates, so they can never stand in for this bound.
        let Some(depth) = query.depth() else {
            return Err(LogicDiagnosticV1::UnboundedWitness {
                reason: UnboundedWitnessReasonV1::MissingPathLengthBound,
                span: location,
            });
        };

        // 3. Exact sealed bounds from the document; no independent default.
        let bounds = query.witness_bounds();
        let witness = WitnessContractV2::bind(
            &witness_binding(
                query.kind(),
                depth,
                bounds.max_depth(),
                bounds.max_witnesses(),
            ),
            query.location(),
        )?;

        Ok(Self {
            item_id: query.item_id(),
            kind: query.kind(),
            location,
            depth,
            direction: query.direction(),
            summary_facets: query.summary_facets().to_vec(),
            references: query.references().to_vec(),
            witness,
        })
    }

    /// Semantic item identity of the FIND item this contract seals.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The FIND subkind this contract classifies.
    #[must_use]
    pub const fn kind(&self) -> GraphQueryKindV1 {
        self.kind
    }

    /// Checked location of the originating FIND item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }

    /// The explicit positive traversal depth, exactly as resolved.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    /// The authored traversal direction, when the source supplied one.
    #[must_use]
    pub const fn direction(&self) -> Option<GraphDirectionV2> {
        self.direction
    }

    /// Summary facets retained from the resolved query.
    #[must_use]
    pub fn summary_facets(&self) -> &[String] {
        &self.summary_facets
    }

    /// Every resolved reference in the Sema-owned deterministic role order,
    /// including sanitizer/scope/window/frontier facts that are not endpoints.
    #[must_use]
    pub fn references(&self) -> &[ResolvedGraphReferenceV2] {
        &self.references
    }

    /// The ordered singular endpoint references
    /// (source/sink/anchor/reference/pointee/callable) the resolved query
    /// supplied.
    pub fn endpoints(&self) -> impl Iterator<Item = &ResolvedGraphReferenceV2> + '_ {
        self.references
            .iter()
            .filter(|reference| GraphEndpointRoleV1::from_role(reference.role()).is_some())
    }

    /// The ordered `via`/`through` path constraints. They restrict WHICH paths
    /// are witnesses; they never bound path length.
    pub fn waypoints(&self) -> impl Iterator<Item = &ResolvedGraphReferenceV2> + '_ {
        self.references.iter().filter(|reference| {
            matches!(
                reference.role(),
                GraphReferenceRoleV2::Via | GraphReferenceRoleV2::Through
            )
        })
    }

    /// The bounded witness contract this FIND was sealed with.
    #[must_use]
    pub const fn witness(&self) -> &WitnessContractV2 {
        &self.witness
    }

    /// The relation shape the witness contract was bound against. Every FIND
    /// family in this packet binds as [`RelationShapeV1::Cyclic`].
    #[must_use]
    pub const fn shape(&self) -> RelationShapeV1 {
        self.witness.shape()
    }

    /// The dedup identity for this subkind's witnesses.
    #[must_use]
    pub const fn dedup(&self) -> DedupIdentityV1 {
        self.witness.dedup()
    }

    /// The canonical witness tie-break.
    #[must_use]
    pub const fn tie_break(&self) -> WitnessTieBreakV1 {
        self.witness.tie_break()
    }

    /// The effective max-witness cap: the document's sealed `max_witnesses`.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.witness.max_witnesses()
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "owner contract rails assert exact endpoint/witness rejects on fixed inputs"
    )]

    use super::*;
    use crate::diagnostic::UnboundedWitnessReasonV1;

    const SEALED_MAX_WITNESSES: u32 = 32;
    const SEALED_MAX_DEPTH: u32 = 16;

    const ALL_KINDS: [GraphQueryKindV1; 7] = [
        GraphQueryKindV1::Reachability,
        GraphQueryKindV1::Flow,
        GraphQueryKindV1::Slice,
        GraphQueryKindV1::Taint,
        GraphQueryKindV1::Path,
        GraphQueryKindV1::PointsTo,
        GraphQueryKindV1::Summary,
    ];

    fn required(kind: GraphQueryKindV1) -> Vec<GraphEndpointRoleV1> {
        required_endpoints(kind).to_vec()
    }

    /// The INDEPENDENT required-role expectation: the authored contract for each
    /// subkind, written out here rather than read back from production.
    ///
    /// Without this table the negative rails below are self-referential: they
    /// build `present` from `required_endpoints(kind)` itself, so a mutant that
    /// narrows a subkind's required set is never contradicted by the very loop
    /// that claims every required role is enforced.
    fn required_expectations() -> Vec<(GraphQueryKindV1, &'static [GraphEndpointRoleV1])> {
        use GraphEndpointRoleV1::{Anchor, Callable, Pointee, Reference, Sink, Source};
        vec![
            (GraphQueryKindV1::Reachability, &[Source, Sink]),
            (GraphQueryKindV1::Flow, &[Source, Sink]),
            (GraphQueryKindV1::Slice, &[Anchor]),
            (GraphQueryKindV1::Taint, &[Source, Sink]),
            (GraphQueryKindV1::Path, &[Source, Sink]),
            (GraphQueryKindV1::PointsTo, &[Reference, Pointee]),
            (GraphQueryKindV1::Summary, &[Callable]),
        ]
    }

    // The production required-role table equals the authored expectation for
    // every subkind, and that expectation covers every subkind exactly once.
    #[test]
    fn required_endpoint_table_matches_the_authored_expectation() {
        let expectations = required_expectations();
        assert_eq!(
            expectations.len(),
            ALL_KINDS.len(),
            "the expectation table covers every subkind exactly once"
        );
        for kind in ALL_KINDS {
            let expected: Vec<_> = expectations
                .iter()
                .filter(|(expected_kind, _)| *expected_kind == kind)
                .map(|(_, roles)| *roles)
                .collect();
            assert_eq!(expected.len(), 1, "{kind:?} appears exactly once");
            assert_eq!(
                required(kind),
                expected[0],
                "{kind:?} must require exactly the authored role set"
            );
        }
    }

    // Every subkind admits exactly its required endpoint roles.
    #[test]
    fn every_subkind_admits_its_required_endpoints() {
        for (kind, _) in required_expectations() {
            let present = required(kind);
            assert_eq!(
                endpoint_legality(kind, &present),
                Ok(()),
                "{kind:?} must admit {present:?}"
            );
        }
    }

    // Every required role is genuinely required: dropping it reports exactly
    // that role as missing. The expected role set comes from the authored
    // table, so a narrowed production set cannot make this loop vacuous.
    #[test]
    fn every_required_endpoint_is_enforced_by_exact_role() {
        for (kind, expected_roles) in required_expectations() {
            assert!(
                !expected_roles.is_empty(),
                "{kind:?} must declare at least one required endpoint role"
            );
            for (index, role) in expected_roles.iter().enumerate() {
                let mut present = expected_roles.to_vec();
                present.remove(index);
                assert_eq!(
                    endpoint_legality(kind, &present),
                    Err(EndpointViolationV2::Missing(*role)),
                    "{kind:?} must require {role:?}"
                );
            }
        }
    }

    // Every endpoint role outside a subkind's exact required set is rejected.
    // This closes the full 7-kind x 6-endpoint negative matrix rather than
    // relying on representative points-to/slice examples. The accepted set is
    // the AUTHORED expectation, not production's own output.
    #[test]
    fn every_subkind_rejects_every_unexpected_endpoint() {
        for (kind, expected_roles) in required_expectations() {
            for role in GraphEndpointRoleV1::ALL {
                if expected_roles.contains(&role) {
                    continue;
                }
                let mut present = expected_roles.to_vec();
                present.push(role);
                assert_eq!(
                    endpoint_legality(kind, &present),
                    Err(EndpointViolationV2::Unexpected(role)),
                    "{kind:?} must reject unexpected {role:?}"
                );
            }
        }
    }

    // The points_to acyclic exemption is dead policy: points_to without a path
    // bound now rejects, and an endpoint its subkind does not accept rejects.
    #[test]
    fn points_to_rejects_a_callable_endpoint() {
        assert_eq!(
            endpoint_legality(
                GraphQueryKindV1::PointsTo,
                &[
                    GraphEndpointRoleV1::Reference,
                    GraphEndpointRoleV1::Pointee,
                    GraphEndpointRoleV1::Callable,
                ],
            ),
            Err(EndpointViolationV2::Unexpected(
                GraphEndpointRoleV1::Callable
            ))
        );
    }

    // A slice anchors only; a source/sink is an unexpected endpoint. The scan
    // order reports `source` before `sink`.
    #[test]
    fn slice_rejects_extraneous_endpoints_in_deterministic_order() {
        assert_eq!(
            endpoint_legality(
                GraphQueryKindV1::Slice,
                &[
                    GraphEndpointRoleV1::Anchor,
                    GraphEndpointRoleV1::Source,
                    GraphEndpointRoleV1::Sink,
                ],
            ),
            Err(EndpointViolationV2::Unexpected(GraphEndpointRoleV1::Source))
        );
    }

    // A missing required role wins over an extraneous one, so the first defect
    // in (missing, unexpected) order is stable.
    #[test]
    fn missing_required_endpoint_wins_over_extraneous_endpoint() {
        assert_eq!(
            endpoint_legality(GraphQueryKindV1::Slice, &[GraphEndpointRoleV1::Source],),
            Err(EndpointViolationV2::Missing(GraphEndpointRoleV1::Anchor))
        );
    }

    // Taint without a sink reports the exact missing role.
    #[test]
    fn taint_requires_a_sink() {
        assert_eq!(
            endpoint_legality(GraphQueryKindV1::Taint, &[GraphEndpointRoleV1::Source]),
            Err(EndpointViolationV2::Missing(GraphEndpointRoleV1::Sink))
        );
    }

    // Dedup identity is fixed per subkind.
    #[test]
    fn dedup_identity_is_fixed_per_subkind() {
        assert_eq!(
            dedup_identity(GraphQueryKindV1::Path),
            DedupIdentityV1::NodeSequence
        );
        assert_eq!(
            dedup_identity(GraphQueryKindV1::Flow),
            DedupIdentityV1::NodeSequence
        );
        assert_eq!(
            dedup_identity(GraphQueryKindV1::Taint),
            DedupIdentityV1::NodeSequence
        );
        assert_eq!(
            dedup_identity(GraphQueryKindV1::Reachability),
            DedupIdentityV1::Endpoints
        );
        assert_eq!(
            dedup_identity(GraphQueryKindV1::Slice),
            DedupIdentityV1::NodeSet
        );
        assert_eq!(
            dedup_identity(GraphQueryKindV1::Summary),
            DedupIdentityV1::NodeSet
        );
        assert_eq!(
            dedup_identity(GraphQueryKindV1::PointsTo),
            DedupIdentityV1::NodeSet
        );
    }

    // No subkind keeps an acyclic bypass: every family binds as Cyclic with the
    // sealed cap and the exact resolved depth.
    #[test]
    fn every_subkind_binds_cyclic_with_sealed_cap_and_exact_depth() {
        for kind in ALL_KINDS {
            let binding = witness_binding(kind, 5, SEALED_MAX_DEPTH, SEALED_MAX_WITNESSES);
            assert_eq!(binding.shape, RelationShapeV1::Cyclic, "{kind:?}");
            assert_eq!(
                binding.max_witnesses,
                Some(SEALED_MAX_WITNESSES),
                "{kind:?}"
            );
            assert_eq!(binding.max_path_length, Some(5), "{kind:?}");
            assert_eq!(binding.tie_break, WitnessTieBreakV1::ShortestThenLexical);
            let contract = WitnessContractV2::validate(&binding).expect("sealed binding admits");
            assert_eq!(contract.max_witnesses(), SEALED_MAX_WITNESSES);
            assert_eq!(contract.max_path_length(), Some(5));
        }
    }

    // The binding carries the sealed budget through, so the binder can reject a
    // bound that exceeds the document receipt (exact limit succeeds, +1 fails).
    #[test]
    fn binding_enforces_the_sealed_budget_at_exact_limit_and_one_over() {
        let exact = witness_binding(
            GraphQueryKindV1::Reachability,
            SEALED_MAX_DEPTH,
            SEALED_MAX_DEPTH,
            SEALED_MAX_WITNESSES,
        );
        assert!(WitnessContractV2::validate(&exact).is_ok());

        let mut over_depth = witness_binding(
            GraphQueryKindV1::Reachability,
            1,
            SEALED_MAX_DEPTH,
            SEALED_MAX_WITNESSES,
        );
        over_depth.max_path_length = Some(SEALED_MAX_DEPTH + 1);
        assert_eq!(
            WitnessContractV2::validate(&over_depth).expect_err("depth +1 rejects"),
            UnboundedWitnessReasonV1::PathLengthBoundOverBudget
        );

        let mut over_cap = witness_binding(
            GraphQueryKindV1::Reachability,
            1,
            SEALED_MAX_DEPTH,
            SEALED_MAX_WITNESSES,
        );
        over_cap.max_witnesses = Some(SEALED_MAX_WITNESSES + 1);
        assert_eq!(
            WitnessContractV2::validate(&over_cap).expect_err("cap +1 rejects"),
            UnboundedWitnessReasonV1::MaxWitnessesOverBudget
        );
    }

    // A zero depth is a degenerate bound, and a missing depth over a transitive
    // family is unbounded; both reject through the witness owner.
    #[test]
    fn zero_and_missing_depth_reject() {
        let zero = witness_binding(
            GraphQueryKindV1::Reachability,
            0,
            SEALED_MAX_DEPTH,
            SEALED_MAX_WITNESSES,
        );
        assert_eq!(
            WitnessContractV2::validate(&zero).expect_err("zero depth rejects"),
            UnboundedWitnessReasonV1::ZeroPathLengthBound
        );

        let mut missing = witness_binding(
            GraphQueryKindV1::Reachability,
            1,
            SEALED_MAX_DEPTH,
            SEALED_MAX_WITNESSES,
        );
        missing.max_path_length = None;
        assert_eq!(
            WitnessContractV2::validate(&missing).expect_err("missing depth rejects"),
            UnboundedWitnessReasonV1::MissingPathLengthBound
        );
    }

    // Subkind keywords are stable (they render into diagnostics). The table
    // covers EVERY subkind, not a representative sample: a keyword mutant in an
    // unlisted arm would otherwise keep this rail green.
    #[test]
    fn subkind_keywords_are_stable() {
        let expected = [
            (GraphQueryKindV1::Reachability, "reachability"),
            (GraphQueryKindV1::Flow, "flow"),
            (GraphQueryKindV1::Slice, "slice"),
            (GraphQueryKindV1::Taint, "taint"),
            (GraphQueryKindV1::Path, "path"),
            (GraphQueryKindV1::PointsTo, "points_to"),
            (GraphQueryKindV1::Summary, "summary"),
        ];
        assert_eq!(
            expected.len(),
            ALL_KINDS.len(),
            "the keyword table covers every subkind in ALL_KINDS"
        );
        for (kind, keyword) in expected {
            assert_eq!(graph_query_keyword(kind), keyword, "{kind:?}");
        }
    }

    // Endpoint-role keywords are stable (they render into diagnostics). The
    // table covers EVERY role.
    #[test]
    fn endpoint_role_keywords_are_stable() {
        let expected = [
            (GraphEndpointRoleV1::Source, "source"),
            (GraphEndpointRoleV1::Sink, "sink"),
            (GraphEndpointRoleV1::Anchor, "anchor"),
            (GraphEndpointRoleV1::Reference, "reference"),
            (GraphEndpointRoleV1::Pointee, "pointee"),
            (GraphEndpointRoleV1::Callable, "callable"),
        ];
        assert_eq!(
            expected.len(),
            GraphEndpointRoleV1::ALL.len(),
            "the keyword table covers every endpoint role"
        );
        for (role, keyword) in expected {
            assert_eq!(role.keyword(), keyword, "{role:?}");
        }
    }
}
