//! Deterministic symbol resolution over the shared HIR.
//!
//! The resolver walks a query/decl/rule HIR unit, threads a [`LexicalScopeV1`]
//! with explicit capture edges, builds declaration/import symbol tables, and
//! resolves every bare symbol reference (and every path BASE) to a binder or a
//! typed [`BindDiagnosticV1`]. Cites:
//!
//! - `D3`: resolution runs before any algebra lowering.
//! - `D25`: only lexical, binder-visible scoping; correlated capture is an
//!   explicit scope edge. Path segments are NOT name-resolved here.
//! - `D28`/`D30`: imports resolve against the frozen [`ModuleGraphViewV1`]
//!   only; conflicting imports fail closed with no import-order tiebreak.
//!
//! Resolution is fail-soft in the sense that it collects ALL diagnostics rather
//! than stopping at the first. [`Binder::finish`] applies the stable
//! `(primary span, code, name, rendering)` order, so tree traversal and symbol
//! insertion order cannot select a different first failure. A unit with any
//! diagnostic is rejected (`is_ok()` is false); downstream typing only sees a
//! clean resolution.

use crate::diagnostic::BindDiagnosticV1;
use crate::module_graph::ModuleGraphViewV1;
use crate::scope::{
    BinderKindV1, BindingV1, CaptureV1, LexicalResolutionV1, LexicalScopeV1, ShadowReportKindV1,
};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CatalogDeclHirV1, CatalogPayloadV1,
    CompositionBodyV1, CompositionHirV1, CompositionProducerV1, DeclHirV1, GraphRefV1, HirNodeIdV1,
    JoinHirV1, QueryHirV1, RelationExprHirV1, RelationExprKindV1, RuleHirV1, SpanV1,
    ValueExprHirV1, ValueExprKindV1, ViewBodyV1,
};
use std::collections::{BTreeMap, BTreeSet};

/// Where a resolved value-namespace reference points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedTargetV1 {
    /// Resolved to a local binder (alias/param/let/quantified variable).
    Local(BindingV1),
    /// Resolved to a top-level declaration/rule/view name in this unit.
    Declaration {
        /// The declared name.
        name: String,
        /// The HIR node id of the declaration.
        node: HirNodeIdV1,
        /// Span of the declaration name.
        span: SpanV1,
    },
    /// Resolved to a single unambiguous imported target from the frozen graph.
    Import {
        /// The short imported name as referenced.
        name: String,
        /// The fully-qualified target the frozen graph mapped it to.
        target: String,
    },
}

/// A single resolved reference: the referencing node id and span, plus target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolutionV1 {
    /// HIR node id of the referencing [`ValueExprKindV1::Symbol`] node.
    pub reference: HirNodeIdV1,
    /// Span of the reference.
    pub span: SpanV1,
    /// The name as referenced.
    pub name: String,
    /// What it resolved to.
    pub target: ResolvedTargetV1,
}

/// The result of resolving one HIR unit: every resolved reference plus every
/// diagnostic, with diagnostics in deterministic source order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BindResultV1 {
    resolutions: Vec<ResolutionV1>,
    diagnostics: Vec<BindDiagnosticV1>,
    /// Ordered alias frame produced by every relation node. Consumers use this
    /// side table instead of reconstructing stage visibility from raw HIR.
    relation_outputs: BTreeMap<HirNodeIdV1, Vec<BindingV1>>,
    /// The input frame consumed by grouping. It is not the public group output
    /// and can be admitted later only by an exact aggregate-argument proof.
    partition_inputs: BTreeMap<HirNodeIdV1, Vec<BindingV1>>,
    /// Uncommitted partition-input name candidates. Only an exact later
    /// aggregate-call proof can authorize one; syntactic call position alone
    /// is never a successful semantic resolution.
    partition_candidates: BTreeMap<HirNodeIdV1, PartitionCandidateV1>,
    /// Exact correlation edges keyed by the relational-EXISTS boolean node.
    /// Every visited EXISTS owns an entry, including uncorrelated empty sets.
    capture_edges: BTreeMap<HirNodeIdV1, Vec<(HirNodeIdV1, HirNodeIdV1, usize)>>,
    /// Every Composition LET introduced by the Binder, including unused LETs.
    composition_bindings: BTreeMap<(HirNodeIdV1, usize), BindingV1>,
}

/// Exact source argument reference tentatively linked to a Group input.
/// `call_stack` is outermost-to-innermost; no name or callee spelling grants
/// semantic authority until the types owner resolves each callable overload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionCandidateV1 {
    reference: HirNodeIdV1,
    group: HirNodeIdV1,
    binding: BindingV1,
    call_stack: Vec<HirNodeIdV1>,
    span: SpanV1,
}

impl PartitionCandidateV1 {
    #[must_use]
    pub const fn reference(&self) -> HirNodeIdV1 {
        self.reference
    }
    #[must_use]
    pub const fn group(&self) -> HirNodeIdV1 {
        self.group
    }
    #[must_use]
    pub const fn binding(&self) -> &BindingV1 {
        &self.binding
    }
    #[must_use]
    pub fn call_stack(&self) -> &[HirNodeIdV1] {
        &self.call_stack
    }
    #[must_use]
    pub const fn span(&self) -> SpanV1 {
        self.span
    }
}

impl BindResultV1 {
    /// All successful resolutions, in pre-order.
    #[must_use]
    pub fn resolutions(&self) -> &[ResolutionV1] {
        &self.resolutions
    }

    /// All diagnostics in stable source/code order.
    #[must_use]
    pub fn diagnostics(&self) -> &[BindDiagnosticV1] {
        &self.diagnostics
    }

    /// Whether resolution is clean (no diagnostics). Fail-closed: any
    /// diagnostic makes this false.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Look up the resolved target for a given referencing node id.
    #[must_use]
    pub fn target_of(&self, reference: HirNodeIdV1) -> Option<&ResolvedTargetV1> {
        self.resolutions
            .iter()
            .find(|r| r.reference == reference)
            .map(|r| &r.target)
    }

    /// Resolve one authored use exactly. Graph/CHECK references can share an
    /// enclosing HIR node ID, so callers of those families must not use
    /// [`Self::target_of`]. Duplicate exact keys fail closed.
    #[must_use]
    pub fn target_of_use(
        &self,
        reference: HirNodeIdV1,
        span: SpanV1,
        name: &str,
    ) -> Option<&ResolvedTargetV1> {
        if !self.is_ok() {
            return None;
        }
        let mut matches = self.resolutions.iter().filter(|resolution| {
            resolution.reference == reference && resolution.span == span && resolution.name == name
        });
        let first = matches.next()?;
        matches.next().is_none().then_some(&first.target)
    }

    /// Ordered alias frame produced by one relation node.
    #[must_use]
    pub fn relation_output(&self, relation: HirNodeIdV1) -> Option<&[BindingV1]> {
        self.relation_outputs.get(&relation).map(Vec::as_slice)
    }

    /// Every relation output frame in stable node-id order.
    pub fn relation_outputs(
        &self,
    ) -> impl ExactSizeIterator<Item = (HirNodeIdV1, &[BindingV1])> + '_ {
        self.relation_outputs
            .iter()
            .map(|(node, bindings)| (*node, bindings.as_slice()))
    }

    /// Private-to-the-frontend partition input, distinct from the key-only
    /// public output frame. The semantic owner validates aggregate use.
    #[must_use]
    pub fn partition_input(&self, group: HirNodeIdV1) -> Option<&[BindingV1]> {
        self.partition_inputs.get(&group).map(Vec::as_slice)
    }

    /// Every deferred partition-input use in exact reference-ID order.
    pub fn partition_candidates(&self) -> impl ExactSizeIterator<Item = &PartitionCandidateV1> {
        self.partition_candidates.values()
    }

    /// Whether the relational-EXISTS node was visited and therefore owns an
    /// explicit capture set (which may be empty).
    #[must_use]
    pub fn has_capture_set(&self, relation_exists: HirNodeIdV1) -> bool {
        self.capture_edges.contains_key(&relation_exists)
    }

    /// Exact `(reference_node, binding_owner_node, binding_ordinal)` capture
    /// edges for one relational EXISTS, sorted and deduplicated by stable HIR
    /// identity.
    #[must_use]
    pub fn capture_edges(
        &self,
        relation_exists: HirNodeIdV1,
    ) -> &[(HirNodeIdV1, HirNodeIdV1, usize)] {
        self.capture_edges
            .get(&relation_exists)
            .map_or(&[], Vec::as_slice)
    }

    /// Every explicit relational-EXISTS capture set in node-id order. Empty
    /// sets are retained, so absence cannot be confused with uncorrelatedness.
    pub fn capture_sets(
        &self,
    ) -> impl ExactSizeIterator<Item = (HirNodeIdV1, &[(HirNodeIdV1, HirNodeIdV1, usize)])> + '_
    {
        self.capture_edges
            .iter()
            .map(|(owner, edges)| (*owner, edges.as_slice()))
    }

    /// Exact Binder-owned Composition LET identity. This is the only producer
    /// authority for unused LETs; consumers must not remint owner/ordinal pairs.
    #[must_use]
    pub fn composition_binding(&self, owner: HirNodeIdV1, ordinal: usize) -> Option<&BindingV1> {
        self.composition_bindings.get(&(owner, ordinal))
    }
}

/// Drives resolution over a HIR unit against a frozen module graph.
///
/// One binder is constructed per unit; it borrows the frozen graph and a
/// declaration table built from the surrounding program. Construct via
/// [`Binder::new`], then call one of the `resolve_*` entry points.
pub struct Binder<'g, G: ModuleGraphViewV1> {
    graph: &'g G,
    /// Top-level declaration/rule/view names visible to this unit.
    declarations: BTreeMap<String, DeclSymbolV1>,
    scope: LexicalScopeV1,
    result: BindResultV1,
    /// Active relational EXISTS nodes and their own frame indices. A nested
    /// reference can cross more than one EXISTS boundary; every crossed owner
    /// gets its exact capture edge, not only the innermost node.
    capture_owners: Vec<(HirNodeIdV1, usize)>,
    relation_partition: BTreeMap<HirNodeIdV1, HirNodeIdV1>,
    active_partition: Option<HirNodeIdV1>,
    active_calls: Vec<HirNodeIdV1>,
    /// Current/self and later Composition LET names while resolving one
    /// producer. An unbound relation source matching this set is a forbidden
    /// forward/self reference, never a host fallback candidate.
    forbidden_composition_sources: BTreeMap<String, SpanV1>,
}

/// A top-level declaration symbol-table entry.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeclSymbolV1 {
    node: HirNodeIdV1,
    span: SpanV1,
}

impl<'g, G: ModuleGraphViewV1> Binder<'g, G> {
    /// A binder over `graph` with no top-level declarations yet.
    #[must_use]
    pub fn new(graph: &'g G) -> Self {
        Self {
            graph,
            declarations: BTreeMap::new(),
            scope: LexicalScopeV1::new(),
            result: BindResultV1::default(),
            capture_owners: Vec::new(),
            relation_partition: BTreeMap::new(),
            active_partition: None,
            active_calls: Vec::new(),
            forbidden_composition_sources: BTreeMap::new(),
        }
    }

    /// Register a top-level declaration/rule/view name in the declaration
    /// namespace. A second registration of the same name emits a duplicate
    /// diagnostic and keeps the first binding (fail-closed, order-independent in
    /// effect because both spans are reported).
    pub fn declare(&mut self, name: impl Into<String>, node: HirNodeIdV1, span: SpanV1) {
        let name = name.into();
        if let Some(existing) = self.declarations.get(&name) {
            self.result.diagnostics.push(BindDiagnosticV1::Duplicate {
                name,
                first_span: existing.span,
                second_span: span,
            });
            return;
        }
        let _ = self.declarations.insert(name, DeclSymbolV1 { node, span });
    }

    /// Register every value declaration name before resolving any body, so
    /// forward value references resolve and duplicates are caught up front.
    /// Rule/predicate names remain in their distinct callable namespace and are
    /// intentionally not admitted as bare value symbols here.
    pub fn declare_program(&mut self, decls: &[DeclHirV1], _rules: &[RuleHirV1]) {
        let mut ordered = decls.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|decl| {
            (
                decl.name_span.start,
                decl.name_span.end,
                decl.name.as_str(),
                decl.id,
            )
        });
        for decl in ordered {
            self.declare(decl.name.clone(), decl.id, decl.name_span);
        }
    }

    /// Finish and take the accumulated result.
    #[must_use]
    pub fn finish(mut self) -> BindResultV1 {
        let mut seen_uses = BTreeSet::new();
        for resolution in &self.result.resolutions {
            if !seen_uses.insert((
                resolution.reference,
                resolution.span.start,
                resolution.span.end,
                resolution.name.clone(),
            )) {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::DuplicateReferenceUse {
                        name: resolution.name.clone(),
                        span: resolution.span,
                    });
            }
        }
        self.result.diagnostics.sort_by(|left, right| {
            let left_span = left.primary_span();
            let right_span = right.primary_span();
            (
                left_span.start,
                left_span.end,
                left.code(),
                left.name(),
                left.to_string(),
            )
                .cmp(&(
                    right_span.start,
                    right_span.end,
                    right.code(),
                    right.name(),
                    right.to_string(),
                ))
        });
        for edges in self.result.capture_edges.values_mut() {
            edges.sort_unstable();
            edges.dedup();
        }
        self.result
    }

    // ---- unit entry points -------------------------------------------------

    /// Resolve a query unit. Opens a query-level relation scope; the relation
    /// tree introduces alias bindings, then projection/filter resolve against
    /// them. Quantifiers inside the filter open nested capturing subscopes.
    pub fn resolve_query(&mut self, query: &QueryHirV1) {
        self.scope.push(CaptureV1::Capturing);
        let output = self.resolve_relation(&query.body);
        self.scope.replace_current_bindings(output);
        if let Some(shape) = &query.result_shape {
            self.resolve_shape(shape);
        }
        self.scope.pop();
    }

    /// Resolve a declaration body. `params` are formal parameters introduced
    /// into the body scope (scenario 3: parametric views). The body is the
    /// shared value kernel.
    pub fn resolve_decl(&mut self, decl: &DeclHirV1, params: &[ParamV1]) {
        self.scope.push(CaptureV1::Capturing);
        self.bind_params(params);
        self.resolve_value(&decl.body);
        self.scope.pop();
    }

    /// Resolve binder-owned references within a catalog declaration.
    ///
    /// An inline `VIEW` embeds the canonical [`QueryHirV1`] carrier and must
    /// therefore use the same query resolver as a top-level query.  This keeps
    /// alias frames, capture handling, and symbol diagnostics identical across
    /// the two syntactic placements.  Query references and every non-view
    /// catalog payload remain catalog/modules authority (`D30`), so this method
    /// deliberately does not inspect or resolve their graph references.
    pub fn resolve_catalog_decl(&mut self, decl: &CatalogDeclHirV1) {
        if let CatalogPayloadV1::View(ViewBodyV1::Inline(query)) = &decl.payload {
            self.resolve_query(query);
        }
    }

    /// Resolve a rule unit with formal parameters; head and body both resolve
    /// against the parameter scope.
    pub fn resolve_rule(&mut self, rule: &RuleHirV1, params: &[ParamV1]) {
        self.scope.push(CaptureV1::Capturing);
        self.bind_params(params);
        self.resolve_value(&rule.head);
        self.resolve_bool(&rule.body);
        self.scope.pop();
    }

    /// Resolve a composition unit (`[LET <name> = <producer>]* <body> WITH
    /// <name>`).
    ///
    /// The binder owns the LET...WITH LOCAL scope only (per the task split):
    ///
    /// - Each `LET <name>` introduces a [`BinderKindV1::CompositionLet`] working
    ///   set into one composition frame. A duplicate LET name in that frame is a
    ///   fail-closed [`BindDiagnosticV1::Duplicate`] (`D25`), so binding order is
    ///   never a tiebreak.
    /// - Producers are resolved in order BEFORE their own name is introduced, so
    ///   a producer cannot reference the working set it is defining; later
    ///   producers and the body may reference earlier LET names.
    /// - The body item is resolved against the composition frame: a reference
    ///   that names a LET binding resolves to it.
    /// - `WITH <name>` must resolve to a LET binding in the composition frame; a
    ///   `WITH` naming no LET is a fail-closed [`BindDiagnosticV1::Unresolved`].
    ///
    /// Ref legality of rule/check/source references that are NOT composition-let
    /// names (catalog/module existence) is out of lane (`D30`); this resolver
    /// records a resolution only for references that bind to LOCAL scope and
    /// leaves the rest for the catalog/modules seam.
    pub fn resolve_composition(&mut self, composition: &CompositionHirV1) {
        // A composition's LET working set is frame-local: an inner composition
        // is its OWN scope and must NOT capture an enclosing composition's LET
        // bindings (the same frame-local rule `resolve_with` enforces for the
        // `WITH` consumer). Pushing a NON-capturing edge makes the composition
        // frame a correlation barrier, so a body/producer reference that names
        // only an outer LET is a fail-closed illegal capture (`D25`) instead of
        // a silent scope leak. At the top level there is no enclosing frame, so
        // the barrier is inert.
        self.scope.push(CaptureV1::NonCapturing);
        for (ordinal, binding) in composition.bindings.iter().enumerate() {
            self.forbidden_composition_sources = composition
                .bindings
                .iter()
                .skip(ordinal)
                .map(|candidate| (candidate.name.clone(), candidate.name_span))
                .collect();
            // Resolve the producer in the scope established BEFORE this LET name
            // is visible (sequential LET semantics; no self-reference).
            self.resolve_composition_producer(&binding.producer);
            let introduced = BindingV1::new_indexed(
                binding.name.clone(),
                BinderKindV1::CompositionLet,
                binding.name_span,
                composition.id,
                ordinal,
            );
            let _ = self
                .result
                .composition_bindings
                .insert((composition.id, ordinal), introduced.clone());
            self.introduce(introduced);
        }
        self.forbidden_composition_sources.clear();
        self.resolve_composition_body(&composition.body);
        // WITH must consume a defined LET. Resolve it against the composition
        // frame; an unresolved WITH is fail-closed.
        self.resolve_with(
            &composition.with_name,
            composition.id,
            composition.with_name_span,
        );
        self.scope.pop();
    }

    fn resolve_composition_producer(&mut self, producer: &CompositionProducerV1) {
        match producer {
            CompositionProducerV1::Query(query) => self.resolve_query(query),
            CompositionProducerV1::Sample(sample) => {
                if let Some(scope) = &sample.scope {
                    self.resolve_local_ref(scope, sample.id);
                }
            }
            CompositionProducerV1::GraphQuery(graph) => self.resolve_graph_query(graph),
        }
    }

    fn resolve_composition_body(&mut self, body: &CompositionBodyV1) {
        match body {
            CompositionBodyV1::Query(query) => self.resolve_query(query),
            CompositionBodyV1::GraphQuery(graph) => self.resolve_graph_query(graph),
            CompositionBodyV1::Legality(legality) => self.resolve_legality(legality),
            CompositionBodyV1::Explain(explain) => {
                self.resolve_local_ref(&explain.target_ref, explain.id);
            }
        }
    }

    fn resolve_legality(&mut self, legality: &quanta_gqlang_v2_hir::LegalityHirV1) {
        self.resolve_local_ref(&legality.rule_ref, legality.id);
        for reference in [legality.scope.as_ref(), legality.pattern.as_ref()]
            .into_iter()
            .flatten()
        {
            self.resolve_local_ref(reference, legality.id);
        }
        if let Some(confirm) = &legality.semantic_confirm {
            self.resolve_graph_query(confirm);
        }
    }

    fn resolve_graph_query(&mut self, graph: &quanta_gqlang_v2_hir::GraphQueryHirV1) {
        let endpoints = &graph.endpoints;
        let singular = [
            endpoints.source.as_ref(),
            endpoints.sink.as_ref(),
            endpoints.anchor.as_ref(),
            endpoints.reference.as_ref(),
            endpoints.pointee.as_ref(),
            endpoints.callable.as_ref(),
        ];
        let constraints = &graph.constraints;
        let repeated = constraints
            .via
            .iter()
            .chain(constraints.through.iter())
            .chain(constraints.sanitizers.iter());
        let constrained = [
            constraints.scope.as_ref(),
            constraints.window.as_ref(),
            constraints.frontier.as_ref(),
        ];
        let mut seen = BTreeSet::new();
        for reference in singular
            .into_iter()
            .flatten()
            .chain(repeated)
            .chain(constrained.into_iter().flatten())
        {
            if !seen.insert((
                reference.span.start,
                reference.span.end,
                reference.name.as_str(),
            )) {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::DuplicateReferenceUse {
                        name: reference.name.clone(),
                        span: reference.span,
                    });
                continue;
            }
            self.resolve_local_ref(reference, graph.id);
        }
    }

    /// Resolve a [`GraphRefV1`] against LOCAL lexical scope only. A reference
    /// that binds to a local binder is recorded as a resolution; one that binds
    /// nowhere local is left for the catalog/modules seam (NOT a binder reject).
    /// An attempt to capture a local binding across a non-capturing barrier is
    /// still a fail-closed [`BindDiagnosticV1::IllegalCapture`] (`D25`).
    fn resolve_local_ref(&mut self, reference: &GraphRefV1, node: HirNodeIdV1) {
        match self.scope.resolve(&reference.name) {
            LexicalResolutionV1::Bound(binding) => {
                self.result.resolutions.push(ResolutionV1 {
                    reference: node,
                    span: reference.span,
                    name: reference.name.clone(),
                    target: ResolvedTargetV1::Local(binding),
                });
            }
            LexicalResolutionV1::IllegalCapture(binding) => {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::IllegalCapture {
                        name: reference.name.clone(),
                        span: reference.span,
                        binding_span: binding.span,
                    });
            }
            LexicalResolutionV1::Unbound => {}
        }
    }

    /// Resolve a `WITH <name>` consumer. It MUST bind to a LET in the CURRENT
    /// composition frame only; the working set a `WITH` consumes is frame-local,
    /// so a `WITH` naming an enclosing composition's LET (or no LET at all) is a
    /// fail-closed unresolved reject (`D25`; the binder owns WITH-names-no-LET).
    /// Resolving against only the innermost frame is what stops an inner `WITH`
    /// from illegally capturing an outer LET.
    fn resolve_with(&mut self, name: &str, node: HirNodeIdV1, span: SpanV1) {
        match self.scope.resolve_current_frame(name) {
            LexicalResolutionV1::Bound(binding) => {
                self.result.resolutions.push(ResolutionV1 {
                    reference: node,
                    span,
                    name: name.to_owned(),
                    target: ResolvedTargetV1::Local(binding),
                });
            }
            LexicalResolutionV1::IllegalCapture(binding) => {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::IllegalCapture {
                        name: name.to_owned(),
                        span,
                        binding_span: binding.span,
                    });
            }
            LexicalResolutionV1::Unbound => {
                self.result.diagnostics.push(BindDiagnosticV1::Unresolved {
                    name: name.to_owned(),
                    span,
                });
            }
        }
    }

    // ---- binding introduction ---------------------------------------------

    fn bind_params(&mut self, params: &[ParamV1]) {
        for (ordinal, param) in params.iter().enumerate() {
            self.introduce(BindingV1::new_indexed(
                param.name.clone(),
                BinderKindV1::Param,
                param.span,
                param.node,
                ordinal,
            ));
        }
    }

    /// Introduce a local `let`-style binding into the current scope (scenario:
    /// local let-binding + capture). Public so callers without a `let` HIR node
    /// yet (ticket 06 parser pending) can drive let semantics directly.
    pub fn bind_let(&mut self, name: impl Into<String>, node: HirNodeIdV1, span: SpanV1) {
        self.introduce(BindingV1::new(name, BinderKindV1::Let, span, node));
    }

    /// Open a nested lexical scope with an explicit capture edge to the current
    /// scope. A `Capturing` edge models a correlated subquery that may see
    /// outer aliases; `NonCapturing` models a correlation barrier.
    pub fn open_scope(&mut self, edge: CaptureV1) {
        self.scope.push(edge);
    }

    /// Close the innermost nested scope opened by [`Binder::open_scope`].
    pub fn close_scope(&mut self) {
        self.scope.pop();
    }

    fn introduce(&mut self, binding: BindingV1) {
        let name = binding.name.clone();
        let span = binding.span;
        let report = self.scope.bind(binding);
        if let Some(report) = report {
            match report.kind {
                ShadowReportKindV1::SameFrameDuplicate => {
                    self.result.diagnostics.push(BindDiagnosticV1::Duplicate {
                        name,
                        first_span: report.existing.span,
                        second_span: span,
                    });
                }
                ShadowReportKindV1::OuterShadow => {
                    self.result.diagnostics.push(BindDiagnosticV1::Shadowed {
                        name,
                        outer_span: report.existing.span,
                        inner_span: span,
                    });
                }
            }
        }
    }

    // ---- reference resolution ----------------------------------------------

    fn resolve_relation(&mut self, relation: &RelationExprHirV1) -> Vec<BindingV1> {
        let output = match &relation.kind {
            RelationExprKindV1::Source(source) => {
                // A FROM source is a relation-namespace use. Only a
                // Composition LET may resolve lexically here; genuinely
                // unbound names are left for the host/catalog authority.
                // Record the exact name-token use before its alias enters
                // the current relation frame.
                let (resolution, binding_frame) = self.scope.resolve_with_capture(&source.name);
                match resolution {
                    LexicalResolutionV1::Bound(binding)
                        if binding.kind == BinderKindV1::CompositionLet =>
                    {
                        if let Some(binding_frame) = binding_frame {
                            for &(owner, owner_frame) in &self.capture_owners {
                                if binding_frame < owner_frame {
                                    self.result.capture_edges.entry(owner).or_default().push((
                                        relation.id,
                                        binding.node,
                                        binding.identity().1,
                                    ));
                                }
                            }
                        }
                        self.result.resolutions.push(ResolutionV1 {
                            reference: relation.id,
                            span: source.name_span,
                            name: source.name.clone(),
                            target: ResolvedTargetV1::Local(binding),
                        });
                    }
                    LexicalResolutionV1::Bound(binding) => {
                        self.result
                            .diagnostics
                            .push(BindDiagnosticV1::WrongRelationSourceKind {
                                name: source.name.clone(),
                                span: source.name_span,
                                binding_span: binding.span,
                            });
                    }
                    LexicalResolutionV1::IllegalCapture(binding) => {
                        self.result
                            .diagnostics
                            .push(BindDiagnosticV1::IllegalCapture {
                                name: source.name.clone(),
                                span: source.name_span,
                                binding_span: binding.span,
                            });
                    }
                    LexicalResolutionV1::Unbound => {
                        if self
                            .forbidden_composition_sources
                            .contains_key(&source.name)
                        {
                            self.result.diagnostics.push(BindDiagnosticV1::Unresolved {
                                name: source.name.clone(),
                                span: source.name_span,
                            });
                        }
                    }
                }
                let bindings = source.alias.as_ref().map_or_else(Vec::new, |alias| {
                    vec![BindingV1::new_indexed(
                        alias.name.clone(),
                        BinderKindV1::Alias,
                        alias.span,
                        relation.id,
                        0,
                    )]
                });
                self.checked_new_relation_frame(bindings)
            }
            RelationExprKindV1::Project(project) => {
                let input = self.resolve_relation(&project.input);
                self.scope.replace_current_bindings(input);
                let previous_partition = self.active_partition;
                self.active_partition = self.relation_partition.get(&project.input.id).copied();
                for item in project.items() {
                    self.resolve_value(&item.value);
                }
                self.active_partition = previous_partition;
                if let Some(group) = self.relation_partition.get(&project.input.id).copied() {
                    self.relation_partition.insert(relation.id, group);
                }
                let outputs = project.items().iter().enumerate().map(|(ordinal, item)| {
                    let (name, span) = Self::projection_output_name(item);
                    BindingV1::new_indexed(name, BinderKindV1::Alias, span, relation.id, ordinal)
                });
                self.checked_new_relation_frame(outputs)
            }
            RelationExprKindV1::Filter(filter) => {
                let input = self.resolve_relation(&filter.input);
                self.scope.replace_current_bindings(input.clone());
                let previous_partition = self.active_partition;
                self.active_partition = self.relation_partition.get(&filter.input.id).copied();
                self.resolve_bool(&filter.predicate);
                self.active_partition = previous_partition;
                if let Some(group) = self.relation_partition.get(&filter.input.id).copied() {
                    self.relation_partition.insert(relation.id, group);
                }
                input
            }
            RelationExprKindV1::Join(join) => match join {
                JoinHirV1::Inner { left, right, on } => {
                    let left = self.resolve_relation(left);
                    // Sibling inputs are resolved independently. In particular,
                    // the right alias cannot leak backward into the left input.
                    self.scope.replace_current_bindings(Vec::new());
                    let previous_partition = self.active_partition.take();
                    let right = self.resolve_relation(right);
                    let output = self.checked_relation_frame(left.into_iter().chain(right));
                    self.scope.replace_current_bindings(output.clone());
                    self.resolve_bool(on);
                    self.active_partition = previous_partition;
                    output
                }
                JoinHirV1::Cross { left, right } => {
                    let left = self.resolve_relation(left);
                    self.scope.replace_current_bindings(Vec::new());
                    let previous_partition = self.active_partition.take();
                    let right = self.resolve_relation(right);
                    self.active_partition = previous_partition;
                    self.checked_relation_frame(left.into_iter().chain(right))
                }
            },
            RelationExprKindV1::GroupBy(group) => {
                let input = self.resolve_relation(&group.input);
                self.scope.replace_current_bindings(input.clone());
                let previous_partition = self.active_partition.take();
                for key in group.keys() {
                    self.resolve_value(key);
                }
                self.active_partition = previous_partition;
                self.result.partition_inputs.insert(relation.id, input);
                self.relation_partition.insert(relation.id, relation.id);
                let outputs = group.keys().iter().enumerate().map(|(ordinal, key)| {
                    let (name, span) = Self::value_output_name(key);
                    BindingV1::new_indexed(name, BinderKindV1::Alias, span, relation.id, ordinal)
                });
                self.checked_new_relation_frame(outputs)
            }
            RelationExprKindV1::OrderBy(order) => {
                let input = self.resolve_relation(&order.input);
                self.scope.replace_current_bindings(input.clone());
                let previous_partition = self.active_partition;
                self.active_partition = self.relation_partition.get(&order.input.id).copied();
                for key in order.keys() {
                    self.resolve_value(&key.value);
                }
                self.active_partition = previous_partition;
                if let Some(group) = self.relation_partition.get(&order.input.id).copied() {
                    self.relation_partition.insert(relation.id, group);
                }
                input
            }
        };
        let _ = self
            .result
            .relation_outputs
            .insert(relation.id, output.clone());
        output
    }

    /// Build one deterministic row frame while rejecting duplicate output
    /// names. The first binding remains authoritative so failure paths do not
    /// gain last-wins behavior.
    fn checked_relation_frame(
        &mut self,
        bindings: impl IntoIterator<Item = BindingV1>,
    ) -> Vec<BindingV1> {
        let mut by_name = BTreeMap::<String, SpanV1>::new();
        let mut output = Vec::new();
        for binding in bindings {
            if let Some(existing_span) = by_name.get(&binding.name) {
                self.result.diagnostics.push(BindDiagnosticV1::Duplicate {
                    name: binding.name.clone(),
                    first_span: *existing_span,
                    second_span: binding.span,
                });
            } else {
                let _ = by_name.insert(binding.name.clone(), binding.span);
                output.push(binding);
            }
        }
        output
    }

    /// Validate newly introduced relation bindings against visible lexical
    /// parents exactly once, then validate duplicate names within the frame.
    fn checked_new_relation_frame(
        &mut self,
        bindings: impl IntoIterator<Item = BindingV1>,
    ) -> Vec<BindingV1> {
        let bindings = bindings.into_iter().collect::<Vec<_>>();
        for binding in &bindings {
            if let Some(outer) = self.scope.visible_outer_binding(&binding.name) {
                self.result.diagnostics.push(BindDiagnosticV1::Shadowed {
                    name: binding.name.clone(),
                    outer_span: outer.span,
                    inner_span: binding.span,
                });
            }
        }
        self.checked_relation_frame(bindings)
    }

    /// Canonical projection output naming. Explicit aliases win; otherwise a
    /// symbol keeps its name, a path uses its final segment, and every other
    /// expression uses `col`. Duplicate defaults are rejected by the same row
    /// frame validator as explicit aliases.
    fn projection_output_name(item: &quanta_gqlang_v2_hir::ProjectionItemV1) -> (String, SpanV1) {
        if let Some(alias) = &item.alias {
            return (alias.name.clone(), alias.span);
        }
        Self::value_output_name(&item.value)
    }

    fn value_output_name(value: &ValueExprHirV1) -> (String, SpanV1) {
        match &value.kind {
            ValueExprKindV1::Symbol(symbol) => (symbol.name.clone(), value.span),
            ValueExprKindV1::Path(path) => path.segments.as_slice().last().map_or_else(
                || ("col".to_owned(), value.span),
                |segment| (segment.name.clone(), segment.span),
            ),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Call(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => ("col".to_owned(), value.span),
        }
    }

    fn resolve_bool(&mut self, node: &BoolExprHirV1) {
        match &node.kind {
            BoolExprKindV1::Literal(_) => {}
            BoolExprKindV1::Value(value) => self.resolve_value(value),
            BoolExprKindV1::Compare(compare) => {
                self.resolve_value(&compare.lhs);
                self.resolve_value(&compare.rhs);
            }
            BoolExprKindV1::And(bin) | BoolExprKindV1::Or(bin) => {
                self.resolve_bool(&bin.lhs);
                self.resolve_bool(&bin.rhs);
            }
            BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                self.resolve_bool(inner);
            }
            BoolExprKindV1::LogicalQuantifier(quant) => {
                // A quantifier opens a nested CAPTURING scope: its bound
                // variables are visible to its body, and the body may also see
                // outer bindings (correlated quantification). Capture remains
                // an explicit edge per D25.
                self.scope.push(CaptureV1::Capturing);
                for (ordinal, binder) in quant.binders.as_slice().iter().enumerate() {
                    self.introduce(BindingV1::new_indexed(
                        binder.name.clone(),
                        BinderKindV1::Quantified,
                        binder.span,
                        node.id,
                        ordinal,
                    ));
                }
                self.resolve_bool(&quant.body);
                self.scope.pop();
            }
            BoolExprKindV1::RelationExists(exists) => {
                self.scope.push(CaptureV1::Capturing);
                let _ = self.result.capture_edges.entry(node.id).or_default();
                if let Some(frame_index) = self.scope.current_frame_index() {
                    self.capture_owners.push((node.id, frame_index));
                }
                let output = self.resolve_relation(&exists.relation);
                self.scope.replace_current_bindings(output);
                let _ = self.capture_owners.pop();
                self.scope.pop();
            }
        }
    }

    fn resolve_value(&mut self, node: &ValueExprHirV1) {
        match &node.kind {
            ValueExprKindV1::Literal(_) => {}
            ValueExprKindV1::Symbol(symbol) => {
                self.resolve_name(&symbol.name, node.id, node.span);
            }
            ValueExprKindV1::Path(path) => {
                // D25: resolve only the BASE; path SEGMENTS are field selectors
                // typed later (ticket 03), never name-resolved here.
                self.resolve_value(&path.base);
            }
            ValueExprKindV1::Unary(unary) => self.resolve_value(&unary.operand),
            ValueExprKindV1::Binary(binary) => {
                self.resolve_value(&binary.lhs);
                self.resolve_value(&binary.rhs);
            }
            ValueExprKindV1::Call(call) => {
                self.active_calls.push(node.id);
                for arg in call.args().as_slice() {
                    if let CallArgHirV1::Expr(value) = arg {
                        self.resolve_value(value);
                    }
                }
                self.active_calls.pop();
            }
            ValueExprKindV1::Paren(inner) => self.resolve_value(inner),
            ValueExprKindV1::Record(record) => {
                for field in &record.fields {
                    self.resolve_value(&field.value);
                }
            }
            ValueExprKindV1::List(list) => {
                for element in &list.elements {
                    self.resolve_value(element);
                }
            }
        }
    }

    fn resolve_shape(&mut self, shape: &quanta_gqlang_v2_hir::ResultShapeHirV1) {
        use quanta_gqlang_v2_hir::ResultShapeKindV1;
        match &shape.kind {
            ResultShapeKindV1::Leaf(value) => self.resolve_value(value),
            ResultShapeKindV1::Record(record) => {
                for field in &record.fields {
                    self.resolve_shape(&field.shape);
                }
            }
            ResultShapeKindV1::List(list) => self.resolve_shape(&list.element),
        }
    }

    /// Resolve one bare name. Resolution order is deterministic and explicit
    /// (`D25`): local lexical scope first, then top-level declarations, then the
    /// frozen import graph. Import order is never authority (`D28`/`D30`): an
    /// ambiguous import is a hard reject, not a tiebreak.
    fn resolve_name(&mut self, name: &str, reference: HirNodeIdV1, span: SpanV1) {
        let (resolution, binding_frame) = self.scope.resolve_with_capture(name);
        match resolution {
            LexicalResolutionV1::Bound(binding) => {
                if let Some(binding_frame) = binding_frame {
                    for &(owner, owner_frame) in &self.capture_owners {
                        if binding_frame < owner_frame {
                            self.result.capture_edges.entry(owner).or_default().push((
                                reference,
                                binding.node,
                                binding.identity().1,
                            ));
                        }
                    }
                }
                self.result.resolutions.push(ResolutionV1 {
                    reference,
                    span,
                    name: name.to_owned(),
                    target: ResolvedTargetV1::Local(binding),
                });
                return;
            }
            LexicalResolutionV1::IllegalCapture(binding) => {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::IllegalCapture {
                        name: name.to_owned(),
                        span,
                        binding_span: binding.span,
                    });
                return;
            }
            LexicalResolutionV1::Unbound => {}
        }

        if let Some(decl) = self.declarations.get(name) {
            if let Some(binding) = self
                .active_partition
                .filter(|_| !self.active_calls.is_empty())
                .and_then(|group| self.result.partition_inputs.get(&group))
                .and_then(|frame| frame.iter().find(|binding| binding.name == name))
            {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::PartitionAuthorityCollision {
                        name: name.to_owned(),
                        span,
                        partition_span: binding.span,
                    });
                return;
            }
            self.result.resolutions.push(ResolutionV1 {
                reference,
                span,
                name: name.to_owned(),
                target: ResolvedTargetV1::Declaration {
                    name: name.to_owned(),
                    node: decl.node,
                    span: decl.span,
                },
            });
            return;
        }

        let mut candidates = self.graph.imported_targets(name);
        candidates.sort();
        candidates.dedup();
        if !candidates.is_empty() {
            if let Some(binding) = self
                .active_partition
                .filter(|_| !self.active_calls.is_empty())
                .and_then(|group| self.result.partition_inputs.get(&group))
                .and_then(|frame| frame.iter().find(|binding| binding.name == name))
            {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::PartitionAuthorityCollision {
                        name: name.to_owned(),
                        span,
                        partition_span: binding.span,
                    });
                return;
            }
        }
        match candidates.len() {
            0 => {
                if let (Some(group), false) = (self.active_partition, self.active_calls.is_empty())
                {
                    if let Some(binding) = self
                        .result
                        .partition_inputs
                        .get(&group)
                        .and_then(|frame| frame.iter().find(|binding| binding.name == name))
                        .cloned()
                    {
                        self.result.partition_candidates.insert(
                            reference,
                            PartitionCandidateV1 {
                                reference,
                                group,
                                binding: binding.clone(),
                                call_stack: self.active_calls.clone(),
                                span,
                            },
                        );
                        self.result.resolutions.push(ResolutionV1 {
                            reference,
                            span,
                            name: name.to_owned(),
                            target: ResolvedTargetV1::Local(binding),
                        });
                        return;
                    }
                }
                self.result.diagnostics.push(BindDiagnosticV1::Unresolved {
                    name: name.to_owned(),
                    span,
                });
            }
            1 => {
                let target = candidates.into_iter().next().unwrap_or_default();
                self.result.resolutions.push(ResolutionV1 {
                    reference,
                    span,
                    name: name.to_owned(),
                    target: ResolvedTargetV1::Import {
                        name: name.to_owned(),
                        target,
                    },
                });
            }
            _ => {
                self.result
                    .diagnostics
                    .push(BindDiagnosticV1::AmbiguousImport {
                        name: name.to_owned(),
                        span,
                        candidates,
                    });
            }
        }
    }
}

/// A formal parameter for a parametric declaration/rule/view (scenario 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamV1 {
    /// Parameter name as written.
    pub name: String,
    /// Span of the parameter name.
    pub span: SpanV1,
    /// HIR node id the parameter binds at (the owning decl/rule node).
    pub node: HirNodeIdV1,
}

impl ParamV1 {
    /// Construct a parameter.
    #[must_use]
    pub fn new(name: impl Into<String>, span: SpanV1, node: HirNodeIdV1) -> Self {
        Self {
            name: name.into(),
            span,
            node,
        }
    }
}
