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
//! Resolution is fail-soft in the sense that it collects ALL diagnostics in
//! deterministic pre-order rather than stopping at the first, so a single pass
//! reports every reject. A unit with any diagnostic is rejected (`is_ok()` is
//! false); downstream typing (ticket 03) only ever sees a clean resolution.

use crate::diagnostic::BindDiagnosticV1;
use crate::module_graph::ModuleGraphViewV1;
use crate::scope::{
    BinderKindV1, BindingV1, CaptureV1, LexicalResolutionV1, LexicalScopeV1, ShadowReportKindV1,
};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CompositionHirV1, DeclHirV1, GraphRefV1, HirNodeIdV1,
    ProgramItemHirV1, QueryHirV1, RelationExprHirV1, RelationExprKindV1, RuleHirV1, SpanV1,
    ValueExprHirV1, ValueExprKindV1,
};
use std::collections::BTreeMap;

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
/// diagnostic, both in deterministic pre-order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BindResultV1 {
    resolutions: Vec<ResolutionV1>,
    diagnostics: Vec<BindDiagnosticV1>,
}

impl BindResultV1 {
    /// All successful resolutions, in pre-order.
    #[must_use]
    pub fn resolutions(&self) -> &[ResolutionV1] {
        &self.resolutions
    }

    /// All diagnostics, in pre-order.
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

    /// Register every declaration/rule name in a program slice before resolving
    /// any bodies, so forward references across declarations resolve and
    /// duplicates are caught up front.
    pub fn declare_program(&mut self, decls: &[DeclHirV1], rules: &[RuleHirV1]) {
        for decl in decls {
            self.declare(decl.name.clone(), decl.id, decl.name_span);
        }
        for rule in rules {
            self.declare(rule.name.clone(), rule.id, rule.name_span);
        }
    }

    /// Finish and take the accumulated result.
    #[must_use]
    pub fn finish(self) -> BindResultV1 {
        self.result
    }

    // ---- unit entry points -------------------------------------------------

    /// Resolve a query unit. Opens a query-level relation scope; the relation
    /// tree introduces alias bindings, then projection/filter resolve against
    /// them. Quantifiers inside the filter open nested capturing subscopes.
    pub fn resolve_query(&mut self, query: &QueryHirV1) {
        self.scope.push(CaptureV1::Capturing);
        self.bind_relation_sources(&query.body);
        self.resolve_relation(&query.body);
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
        for binding in &composition.bindings {
            // Resolve the producer in the scope established BEFORE this LET name
            // is visible (sequential LET semantics; no self-reference).
            self.resolve_item(&binding.producer);
            self.introduce(BindingV1::new(
                binding.name.clone(),
                BinderKindV1::CompositionLet,
                binding.name_span,
                composition.id,
            ));
        }
        self.resolve_item(&composition.body);
        // WITH must consume a defined LET. Resolve it against the composition
        // frame; an unresolved WITH is fail-closed.
        self.resolve_with(
            &composition.with_name,
            composition.id,
            composition.with_name_span,
        );
        self.scope.pop();
    }

    /// Resolve a program item's LOCAL binder references. Dispatches by family;
    /// for the new (graph/legality/presentation/meta/nested-composition) items
    /// only references that bind to LOCAL scope are recorded, leaving catalog /
    /// module ref legality to its owning seam (`D30`).
    fn resolve_item(&mut self, item: &ProgramItemHirV1) {
        match item {
            ProgramItemHirV1::Query(query) => self.resolve_query(query),
            ProgramItemHirV1::Decl(decl) => self.resolve_decl(decl, &[]),
            ProgramItemHirV1::Rule(rule) => self.resolve_rule(rule, &[]),
            ProgramItemHirV1::GraphQuery(graph) => self.resolve_graph_query(graph),
            ProgramItemHirV1::Legality(legality) => {
                self.resolve_local_ref(&legality.rule_ref, legality.id);
                for opt in [legality.scope.as_ref(), legality.pattern.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    self.resolve_local_ref(opt, legality.id);
                }
                if let Some(confirm) = &legality.semantic_confirm {
                    self.resolve_graph_query(confirm);
                }
            }
            ProgramItemHirV1::Presentation(presentation) => {
                self.resolve_presentation(presentation);
            }
            ProgramItemHirV1::Composition(inner) => self.resolve_composition(inner),
            // Catalog declarations register names (modules' concern) and meta
            // directives carry no local binder references.
            ProgramItemHirV1::CatalogDecl(_) | ProgramItemHirV1::Meta(_) => {}
        }
    }

    fn resolve_graph_query(&mut self, graph: &quanta_gqlang_v2_hir::GraphQueryHirV1) {
        let endpoints = &graph.endpoints;
        for endpoint in [
            endpoints.source.as_ref(),
            endpoints.sink.as_ref(),
            endpoints.anchor.as_ref(),
            endpoints.reference.as_ref(),
            endpoints.pointee.as_ref(),
            endpoints.callable.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            self.resolve_local_ref(endpoint, graph.id);
        }
        let constraints = &graph.constraints;
        for waypoint in constraints
            .via
            .iter()
            .chain(constraints.through.iter())
            .chain(constraints.sanitizers.iter())
        {
            self.resolve_local_ref(waypoint, graph.id);
        }
        for waypoint in [
            constraints.scope.as_ref(),
            constraints.window.as_ref(),
            constraints.frontier.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            self.resolve_local_ref(waypoint, graph.id);
        }
    }

    fn resolve_presentation(&mut self, presentation: &quanta_gqlang_v2_hir::PresentationHirV1) {
        use quanta_gqlang_v2_hir::PresentationKindV1;
        let node = presentation.id;
        match &presentation.kind {
            PresentationKindV1::Explain { target_ref, .. } => {
                self.resolve_local_ref(target_ref, node);
            }
            PresentationKindV1::Describe { path, .. } => self.resolve_local_ref(path, node),
            PresentationKindV1::Sample { scope, .. } => {
                if let Some(scope) = scope {
                    self.resolve_local_ref(scope, node);
                }
            }
            PresentationKindV1::Compare {
                baseline_ref,
                candidate_ref,
                ..
            } => {
                self.resolve_local_ref(baseline_ref, node);
                self.resolve_local_ref(candidate_ref, node);
            }
            PresentationKindV1::Export {
                source_ref,
                profile,
                ..
            } => {
                self.resolve_local_ref(source_ref, node);
                if let Some(profile) = profile {
                    self.resolve_local_ref(profile, node);
                }
            }
            PresentationKindV1::Show { .. } => {}
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
        for param in params {
            self.introduce(BindingV1::new(
                param.name.clone(),
                BinderKindV1::Param,
                param.span,
                param.node,
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

    fn bind_relation_sources(&mut self, relation: &RelationExprHirV1) {
        match &relation.kind {
            RelationExprKindV1::Source(source) => {
                if let Some(alias) = &source.alias {
                    self.introduce(BindingV1::new(
                        alias.name.clone(),
                        BinderKindV1::Alias,
                        alias.span,
                        relation.id,
                    ));
                }
            }
            RelationExprKindV1::Project(project) => {
                self.bind_relation_sources(&project.input);
            }
            RelationExprKindV1::Filter(filter) => {
                self.bind_relation_sources(&filter.input);
            }
            RelationExprKindV1::Compose(compose) => {
                for operand in &compose.operands {
                    self.bind_relation_sources(operand);
                }
            }
        }
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

    fn resolve_relation(&mut self, relation: &RelationExprHirV1) {
        match &relation.kind {
            RelationExprKindV1::Source(_) => {}
            RelationExprKindV1::Project(project) => {
                self.resolve_relation(&project.input);
                for item in &project.items {
                    self.resolve_value(&item.value);
                }
            }
            RelationExprKindV1::Filter(filter) => {
                self.resolve_relation(&filter.input);
                self.resolve_bool(&filter.predicate);
            }
            RelationExprKindV1::Compose(compose) => {
                for operand in &compose.operands {
                    self.resolve_relation(operand);
                }
            }
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
            BoolExprKindV1::Quantifier(quant) => {
                // A quantifier opens a nested CAPTURING scope: its bound
                // variables are visible to its body, and the body may also see
                // outer bindings (correlated quantification). Capture remains
                // an explicit edge per D25.
                self.scope.push(CaptureV1::Capturing);
                for binder in &quant.binders {
                    self.introduce(BindingV1::new(
                        binder.clone(),
                        BinderKindV1::Quantified,
                        node.span,
                        node.id,
                    ));
                }
                self.resolve_bool(&quant.body);
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
                for arg in &call.args {
                    self.resolve_value(arg);
                }
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
        match self.scope.resolve(name) {
            LexicalResolutionV1::Bound(binding) => {
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
        match candidates.len() {
            0 => {
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
