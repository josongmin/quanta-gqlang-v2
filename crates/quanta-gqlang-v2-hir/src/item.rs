//! The unified top-level program-item model for the v3 language core.
//!
//! [`ProgramItemHirV1`] is the canonical "one parsed item" type: it unifies the
//! original Query/Decl/Rule carriers with the families the v3 frontend must
//! reach FRONTEND-CONTRACT parity on (the 18 v2 root families). Everything
//! downstream (surface, binder, logic, planning, modules) consumes this one
//! shape, so it is frozen here as a coherent API.
//!
//! This crate only fixes STRUCTURE plus identity (every node carries a unique
//! [`HirNodeIdV1`] and a [`SpanV1`]). It deliberately does NOT decide:
//!
//! - result-kind / effect / capability classification (logic + planning),
//! - bounded-witness legality of FIND (logic),
//! - catalog registration, duplicate-name rejection, admission gating (modules),
//! - binding / scope resolution of LET...WITH (binder).
//!
//! Endpoints, refs, categories, modes, formats, etc. are OPEN tags here:
//! their legality is a typed-later attachment point. The enums that ARE closed
//! (FIND subkind, confirm/emit/mode, activation, admission, binding, evidence,
//! truth, metric, export format) mirror the v2 vocabulary so the later seams
//! validate against a stable, exhaustive set rather than free strings.

use crate::id::HirNodeIdV1;
use crate::span::SpanV1;
use crate::unit::{DeclHirV1, QueryHirV1, RuleHirV1};

/// The canonical top-level item: every parsed root in the v3 language lowers to
/// exactly one of these variants.
///
/// The original three carriers ([`QueryHirV1`], [`DeclHirV1`], [`RuleHirV1`])
/// remain first-class so existing trees embed unchanged; the new variants cover
/// the remaining families.
///
/// Variants box their payloads so the unified item stays one pointer wide
/// regardless of which family it carries; this keeps the enum cheap to move and
/// match across the many downstream seams that consume it.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgramItemHirV1 {
    /// A relational query (SELECT / `from | where | project` pipe).
    Query(Box<QueryHirV1>),
    /// A value declaration carrier (legacy let/const-style binding).
    Decl(Box<DeclHirV1>),
    /// A rule carrier (head implied by a boolean body).
    Rule(Box<RuleHirV1>),
    /// A graph query (FIND): reachability / flow / slice / taint / path /
    /// points-to / summary.
    GraphQuery(Box<GraphQueryHirV1>),
    /// A legality check (CHECK) against a rule reference.
    Legality(Box<LegalityHirV1>),
    /// A presentation verb (EXPLAIN / SHOW / DESCRIBE / SAMPLE / COMPARE /
    /// EXPORT).
    Presentation(Box<PresentationHirV1>),
    /// A catalog declaration (VIEW / MODEL / PACK / PROFILE / GATE / RULE
    /// binding): registers in the catalog, produces no plan.
    CatalogDecl(Box<CatalogDeclHirV1>),
    /// A composition (`LET <name> = <producer>]* <body> WITH <name>`).
    Composition(Box<CompositionHirV1>),
    /// A meta directive (USE PACK / REQUIRE CERTIFIED).
    Meta(Box<MetaHirV1>),
}

impl ProgramItemHirV1 {
    /// The node id of whichever variant this item carries.
    #[must_use]
    pub fn id(&self) -> HirNodeIdV1 {
        match self {
            Self::Query(node) => node.id,
            Self::Decl(node) => node.id,
            Self::Rule(node) => node.id,
            Self::GraphQuery(node) => node.id,
            Self::Legality(node) => node.id,
            Self::Presentation(node) => node.id,
            Self::CatalogDecl(node) => node.id,
            Self::Composition(node) => node.id,
            Self::Meta(node) => node.id,
        }
    }

    /// The source span of whichever variant this item carries.
    #[must_use]
    pub fn span(&self) -> SpanV1 {
        match self {
            Self::Query(node) => node.span,
            Self::Decl(node) => node.span,
            Self::Rule(node) => node.span,
            Self::GraphQuery(node) => node.span,
            Self::Legality(node) => node.span,
            Self::Presentation(node) => node.span,
            Self::CatalogDecl(node) => node.span,
            Self::Composition(node) => node.span,
            Self::Meta(node) => node.span,
        }
    }
}

impl From<QueryHirV1> for ProgramItemHirV1 {
    fn from(node: QueryHirV1) -> Self {
        Self::Query(Box::new(node))
    }
}

impl From<DeclHirV1> for ProgramItemHirV1 {
    fn from(node: DeclHirV1) -> Self {
        Self::Decl(Box::new(node))
    }
}

impl From<RuleHirV1> for ProgramItemHirV1 {
    fn from(node: RuleHirV1) -> Self {
        Self::Rule(Box::new(node))
    }
}

impl From<GraphQueryHirV1> for ProgramItemHirV1 {
    fn from(node: GraphQueryHirV1) -> Self {
        Self::GraphQuery(Box::new(node))
    }
}

impl From<LegalityHirV1> for ProgramItemHirV1 {
    fn from(node: LegalityHirV1) -> Self {
        Self::Legality(Box::new(node))
    }
}

impl From<PresentationHirV1> for ProgramItemHirV1 {
    fn from(node: PresentationHirV1) -> Self {
        Self::Presentation(Box::new(node))
    }
}

impl From<CatalogDeclHirV1> for ProgramItemHirV1 {
    fn from(node: CatalogDeclHirV1) -> Self {
        Self::CatalogDecl(Box::new(node))
    }
}

impl From<CompositionHirV1> for ProgramItemHirV1 {
    fn from(node: CompositionHirV1) -> Self {
        Self::Composition(Box::new(node))
    }
}

impl From<MetaHirV1> for ProgramItemHirV1 {
    fn from(node: MetaHirV1) -> Self {
        Self::Meta(Box::new(node))
    }
}

// ---- FIND ------------------------------------------------------------------

/// A graph query (FIND). Structure only; bounded-witness legality is logic's.
///
/// Target classification (decided later, by logic + planning):
/// `result_kind = path_witness`, `effect = evidence_only`, with
/// witness / recursive-closure capability.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQueryHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The FIND subkind.
    pub kind: GraphQueryKindV1,
    /// Endpoints (source / sink / anchor / reference / pointee / callable).
    /// Each is optional per subkind; presence legality is logic's.
    pub endpoints: GraphEndpointsV1,
    /// Traversal constraints (via / through / sanitizers / scope / window /
    /// direction / frontier / depth / summary facets).
    pub constraints: GraphConstraintsV1,
}

impl GraphQueryHirV1 {
    /// Construct a graph query carrier.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        kind: GraphQueryKindV1,
        endpoints: GraphEndpointsV1,
        constraints: GraphConstraintsV1,
    ) -> Self {
        Self {
            id,
            span,
            kind,
            endpoints,
            constraints,
        }
    }
}

/// The closed set of FIND subkinds (mirrors v2 vocabulary).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraphQueryKindV1 {
    /// Reachability between source and sink.
    Reachability,
    /// Data/control flow.
    Flow,
    /// Program slice.
    Slice,
    /// Taint propagation.
    Taint,
    /// A path witness.
    Path,
    /// Points-to / alias.
    PointsTo,
    /// A summary over a callable/region.
    Summary,
}

/// An open symbol-or-path endpoint reference.
///
/// Resolution to a concrete entity is the binder's job; here it is the name as
/// written plus its span. A dotted path (`a.b.c`) is kept verbatim.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphRefV1 {
    /// The reference as written (symbol or dotted path).
    pub name: String,
    /// Span of the reference.
    pub span: SpanV1,
}

/// FIND endpoints. Each is optional; per-subkind presence legality is logic's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphEndpointsV1 {
    /// The source endpoint.
    pub source: Option<GraphRefV1>,
    /// The sink endpoint.
    pub sink: Option<GraphRefV1>,
    /// The anchor endpoint (slice/summary).
    pub anchor: Option<GraphRefV1>,
    /// The reference endpoint (points-to).
    pub reference: Option<GraphRefV1>,
    /// The pointee endpoint (points-to).
    pub pointee: Option<GraphRefV1>,
    /// The callable endpoint (summary).
    pub callable: Option<GraphRefV1>,
}

/// Open traversal direction tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDirectionV1 {
    /// The direction keyword as written (e.g. `"forward"`, `"backward"`).
    pub keyword: String,
}

/// FIND traversal constraints (structure only).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphConstraintsV1 {
    /// `via` waypoint references, in written order.
    pub via: Vec<GraphRefV1>,
    /// `through` intermediate references, in written order.
    pub through: Vec<GraphRefV1>,
    /// Sanitizer references, in written order.
    pub sanitizers: Vec<GraphRefV1>,
    /// Optional scope reference.
    pub scope: Option<GraphRefV1>,
    /// Optional window reference.
    pub window: Option<GraphRefV1>,
    /// Optional traversal direction (open tag).
    pub direction: Option<GraphDirectionV1>,
    /// Optional frontier reference.
    pub frontier: Option<GraphRefV1>,
    /// Optional traversal depth bound. Bounded-witness legality is logic's.
    pub depth: Option<u32>,
    /// Summary facet names, in written order.
    pub summary_facets: Vec<String>,
}

// ---- CHECK -----------------------------------------------------------------

/// A legality check (CHECK). Target classification: `result_kind = alert`,
/// `effect = read_only`.
#[derive(Clone, Debug, PartialEq)]
pub struct LegalityHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The rule reference this check evaluates.
    pub rule_ref: GraphRefV1,
    /// Optional scope reference.
    pub scope: Option<GraphRefV1>,
    /// Optional pattern reference.
    pub pattern: Option<GraphRefV1>,
    /// Explicit graph query used to semantically confirm pattern candidates.
    ///
    /// This is owned by the public CHECK syntax. Hosts lower this query through
    /// the same canonical FIND lowering seam as a top-level graph query; rule
    /// names are never interpreted as hidden query-registry keys.
    pub semantic_confirm: Option<Box<GraphQueryHirV1>>,
    /// Confirmation policy.
    pub confirm: ConfirmModeV1,
    /// Emission target.
    pub emit: EmitModeV1,
    /// Detection mode.
    pub mode: DetectionModeV1,
}

impl LegalityHirV1 {
    /// Construct a legality-check carrier.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        rule_ref: GraphRefV1,
        scope: Option<GraphRefV1>,
        pattern: Option<GraphRefV1>,
        confirm: ConfirmModeV1,
        emit: EmitModeV1,
        mode: DetectionModeV1,
    ) -> Self {
        Self {
            id,
            span,
            rule_ref,
            scope,
            pattern,
            semantic_confirm: None,
            confirm,
            emit,
            mode,
        }
    }

    /// Attach the explicit semantic-confirm query carried by CHECK syntax.
    #[must_use]
    pub fn with_semantic_confirm(mut self, query: GraphQueryHirV1) -> Self {
        self.semantic_confirm = Some(Box::new(query));
        self
    }
}

/// CHECK confirmation policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConfirmModeV1 {
    /// Skip confirmation.
    Skip,
    /// Prompt for confirmation.
    Prompt,
    /// Require confirmation.
    Require,
}

/// CHECK emission target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmitModeV1 {
    /// Emit findings.
    Findings,
    /// Emit SARIF.
    Sarif,
}

/// CHECK detection mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DetectionModeV1 {
    /// Pattern matching only.
    PatternOnly,
    /// Semantic analysis only.
    SemanticOnly,
    /// Hybrid pattern + semantic.
    Hybrid,
}

// ---- presentation verbs ----------------------------------------------------

/// A presentation verb (EXPLAIN / SHOW / DESCRIBE / SAMPLE / COMPARE / EXPORT).
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The verb-specific payload.
    pub kind: PresentationKindV1,
}

/// Variant-specific SAMPLE node used where only SAMPLE is legal.
#[derive(Clone, Debug, PartialEq)]
pub struct SampleHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// Retrieval query text.
    pub query_text: String,
    /// Retrieval mode.
    pub mode: SampleModeV1,
    /// Positive result limit.
    pub limit: u32,
    /// Optional scope.
    pub scope: Option<GraphRefV1>,
}

/// Variant-specific EXPLAIN node used where only EXPLAIN is legal.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplainHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// Explained target.
    pub target_ref: GraphRefV1,
    /// Evidence mode.
    pub evidence: EvidenceModeV1,
    /// Truth mode.
    pub truth: TruthModeV1,
}

impl TryFrom<PresentationHirV1> for SampleHirV1 {
    type Error = PresentationHirV1;

    fn try_from(node: PresentationHirV1) -> Result<Self, Self::Error> {
        match node {
            PresentationHirV1 {
                id,
                span,
                kind:
                    PresentationKindV1::Sample {
                        query_text,
                        mode,
                        limit,
                        scope,
                    },
            } => Ok(Self {
                id,
                span,
                query_text,
                mode,
                limit,
                scope,
            }),
            other => Err(other),
        }
    }
}

impl TryFrom<PresentationHirV1> for ExplainHirV1 {
    type Error = PresentationHirV1;

    fn try_from(node: PresentationHirV1) -> Result<Self, Self::Error> {
        match node {
            PresentationHirV1 {
                id,
                span,
                kind:
                    PresentationKindV1::Explain {
                        target_ref,
                        evidence,
                        truth,
                    },
            } => Ok(Self {
                id,
                span,
                target_ref,
                evidence,
                truth,
            }),
            other => Err(other),
        }
    }
}

impl PresentationHirV1 {
    /// Construct a presentation carrier.
    #[must_use]
    pub fn new(id: HirNodeIdV1, span: SpanV1, kind: PresentationKindV1) -> Self {
        Self { id, span, kind }
    }
}

/// Verb-specific payload of a [`PresentationHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub enum PresentationKindV1 {
    /// EXPLAIN a rule/graph reference. `result_kind = proof_explain`.
    Explain {
        /// The reference being explained.
        target_ref: GraphRefV1,
        /// Evidence mode.
        evidence: EvidenceModeV1,
        /// Truth mode.
        truth: TruthModeV1,
    },
    /// SHOW a catalog category. `result_kind = table`.
    Show {
        /// The introspection category.
        category: ShowCategoryV1,
    },
    /// DESCRIBE an entity by `kind::path`. `result_kind = alert`.
    Describe {
        /// The entity kind tag (open).
        kind: String,
        /// The entity path reference.
        path: GraphRefV1,
    },
    /// SAMPLE a retrieval query. `result_kind = table`.
    Sample {
        /// The query text.
        query_text: String,
        /// Retrieval mode.
        mode: SampleModeV1,
        /// Result limit (must be `> 0`; legality is logic's).
        limit: u32,
        /// Optional scope reference.
        scope: Option<GraphRefV1>,
    },
    /// COMPARE a baseline against a candidate. `result_kind = path_witness`.
    Compare {
        /// The baseline reference.
        baseline_ref: GraphRefV1,
        /// The candidate reference (explicit candidate required).
        candidate_ref: GraphRefV1,
        /// Comparison metric.
        metric: CompareMetricV1,
    },
    /// EXPORT a source under a format. `effect = export_only`.
    Export {
        /// The source reference.
        source_ref: GraphRefV1,
        /// Export format.
        format: ExportFormatV1,
        /// Optional profile reference.
        profile: Option<GraphRefV1>,
    },
}

/// EXPLAIN evidence mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EvidenceModeV1 {
    /// Automatic evidence selection.
    Auto,
    /// Trace evidence.
    Trace,
    /// Citation evidence.
    Citation,
    /// Counterexample evidence.
    Counterexample,
}

/// EXPLAIN truth mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TruthModeV1 {
    /// Inspect the truth.
    Inspect,
    /// Confirm the truth.
    Confirm,
    /// Refute the truth.
    Refute,
}

/// SHOW catalog introspection category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShowCategoryV1 {
    /// Capabilities catalog.
    Capabilities,
    /// Models catalog.
    Models,
    /// Views catalog.
    Views,
    /// Rules catalog.
    Rules,
    /// Packs catalog.
    Packs,
    /// Profiles catalog.
    Profiles,
    /// Gates catalog.
    Gates,
}

/// SAMPLE retrieval mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SampleModeV1 {
    /// Lexical retrieval.
    Lexical,
    /// Semantic retrieval.
    Semantic,
    /// Hybrid retrieval.
    Hybrid,
}

/// COMPARE metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompareMetricV1 {
    /// Exact comparison.
    Exact,
    /// Structural comparison.
    Structural,
    /// Semantic comparison.
    Semantic,
}

/// EXPORT format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportFormatV1 {
    /// JSON export.
    Json,
    /// CSV export.
    Csv,
    /// SARIF export.
    Sarif,
    /// Markdown export.
    Markdown,
    /// GraphML export.
    GraphMl,
}

// ---- catalog declarations --------------------------------------------------

/// A catalog declaration (VIEW / MODEL / PACK / PROFILE / GATE / RULE binding).
///
/// Declarations register in the catalog and produce NO plan. Duplicate-name
/// rejection, VIEW inline-select cycle rejection, and REQUIRE CERTIFIED
/// admission forcing are MODULES' concerns; here it is structure only.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogDeclHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// Declared name.
    pub name: String,
    /// Span of the declared name.
    pub name_span: SpanV1,
    /// Kind-tagged payload.
    pub payload: CatalogPayloadV1,
}

impl CatalogDeclHirV1 {
    /// Construct a catalog declaration carrier.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        name: impl Into<String>,
        name_span: SpanV1,
        payload: CatalogPayloadV1,
    ) -> Self {
        Self {
            id,
            span,
            name: name.into(),
            name_span,
            payload,
        }
    }
}

/// Kind-tagged payload of a [`CatalogDeclHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub enum CatalogPayloadV1 {
    /// VIEW: a query reference XOR an inline SELECT. The XOR legality and
    /// inline-select cycle rejection are modules'.
    View(ViewBodyV1),
    /// MODEL: a provider plus a source reference.
    Model {
        /// The provider tag (open).
        provider: String,
        /// The source reference.
        source: GraphRefV1,
    },
    /// PACK: a source reference plus an activation policy.
    Pack {
        /// The source reference.
        source: GraphRefV1,
        /// The activation policy.
        activation: ActivationV1,
    },
    /// PROFILE: capabilities plus an admission policy.
    Profile {
        /// Capability references, in written order.
        capabilities: Vec<GraphRefV1>,
        /// The admission policy.
        admission: AdmissionV1,
    },
    /// GATE: checks plus a binding policy.
    Gate {
        /// Check references, in written order.
        checks: Vec<GraphRefV1>,
        /// The binding policy.
        binding: GateBindingV1,
    },
    /// RULE binding: a check reference.
    RuleBinding {
        /// The check reference this rule binds.
        check_ref: GraphRefV1,
    },
}

/// VIEW body: a query reference XOR an inline SELECT query.
///
/// The XOR is represented as two variants so it is impossible to hold both; the
/// inline form embeds a [`QueryHirV1`] so the SELECT pipe is the shared query
/// carrier rather than a second relational tree.
#[derive(Clone, Debug, PartialEq)]
pub enum ViewBodyV1 {
    /// A reference to an existing named query.
    QueryRef(GraphRefV1),
    /// An inline SELECT query body.
    Inline(Box<QueryHirV1>),
}

/// PACK activation policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivationV1 {
    /// Manual activation.
    Manual,
    /// Session-scoped activation.
    Session,
    /// Workspace-scoped activation.
    Workspace,
}

/// PROFILE admission policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AdmissionV1 {
    /// Allowlist admission.
    Allowlist,
    /// Review-required admission.
    ReviewRequired,
    /// Certified-only admission. REQUIRE CERTIFIED forces all profiles here.
    CertifiedOnly,
}

/// GATE binding policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GateBindingV1 {
    /// All checks must pass.
    All,
    /// Any check passing suffices.
    Any,
}

// ---- composition -----------------------------------------------------------

/// A composition (`[LET <name> = <producer>]* <body> WITH <name>`).
///
/// Producer and body carriers are closed enums, so an invalid family cannot be
/// represented. Name resolution and duplicate-name checks remain parser/binder
/// responsibilities.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The ordered LET bindings.
    pub bindings: Vec<CompositionBindingV1>,
    /// The name consumed by `WITH`.
    pub with_name: String,
    /// Span of the `WITH` name.
    pub with_name_span: SpanV1,
    /// The composition body (FIND / SELECT / CHECK / EXPLAIN by legality).
    pub body: CompositionBodyV1,
}

impl CompositionHirV1 {
    /// Construct a composition carrier.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        bindings: Vec<CompositionBindingV1>,
        with_name: impl Into<String>,
        with_name_span: SpanV1,
        body: CompositionBodyV1,
    ) -> Self {
        Self {
            id,
            span,
            bindings,
            with_name: with_name.into(),
            with_name_span,
            body,
        }
    }
}

/// A single `LET <name> = <producer>` binding.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionBindingV1 {
    /// The bound name.
    pub name: String,
    /// Span of the bound name.
    pub name_span: SpanV1,
    /// The producer (SAMPLE / SELECT / FIND by legality).
    pub producer: CompositionProducerV1,
}

impl CompositionBindingV1 {
    /// Construct a composition binding.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        name_span: SpanV1,
        producer: CompositionProducerV1,
    ) -> Self {
        Self {
            name: name.into(),
            name_span,
            producer,
        }
    }
}

/// Closed set of composition producer families.
///
/// Invalid producer families do not type-check:
/// ```compile_fail
/// use quanta_gqlang_v2_hir::{CompositionBindingV1, MetaKindV1, MetaHirV1, HirNodeIdV1, ProgramItemHirV1, SpanV1};
/// let meta = MetaHirV1::new(HirNodeIdV1::from_raw(1), SpanV1::new(0, 1), MetaKindV1::RequireCertified);
/// let _ = CompositionBindingV1::new("x", SpanV1::new(0, 1), ProgramItemHirV1::from(meta));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum CompositionProducerV1 {
    /// Relational SELECT/query producer.
    Query(Box<QueryHirV1>),
    /// SAMPLE producer.
    Sample(Box<SampleHirV1>),
    /// FIND producer.
    GraphQuery(Box<GraphQueryHirV1>),
}

/// Closed set of composition body families.
#[derive(Clone, Debug, PartialEq)]
pub enum CompositionBodyV1 {
    /// Relational SELECT/query body.
    Query(Box<QueryHirV1>),
    /// FIND body.
    GraphQuery(Box<GraphQueryHirV1>),
    /// CHECK body.
    Legality(Box<LegalityHirV1>),
    /// EXPLAIN body.
    Explain(Box<ExplainHirV1>),
}

impl From<QueryHirV1> for CompositionProducerV1 {
    fn from(value: QueryHirV1) -> Self {
        Self::Query(Box::new(value))
    }
}

impl From<GraphQueryHirV1> for CompositionProducerV1 {
    fn from(value: GraphQueryHirV1) -> Self {
        Self::GraphQuery(Box::new(value))
    }
}

impl From<QueryHirV1> for CompositionBodyV1 {
    fn from(value: QueryHirV1) -> Self {
        Self::Query(Box::new(value))
    }
}

impl From<GraphQueryHirV1> for CompositionBodyV1 {
    fn from(value: GraphQueryHirV1) -> Self {
        Self::GraphQuery(Box::new(value))
    }
}

impl From<LegalityHirV1> for CompositionBodyV1 {
    fn from(value: LegalityHirV1) -> Self {
        Self::Legality(Box::new(value))
    }
}

// ---- meta ------------------------------------------------------------------

/// A meta directive (USE PACK / REQUIRE CERTIFIED).
///
/// Effect-style, no relational plan: pack activation plus certified gating.
#[derive(Clone, Debug, PartialEq)]
pub struct MetaHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The directive payload.
    pub kind: MetaKindV1,
}

impl MetaHirV1 {
    /// Construct a meta directive carrier.
    #[must_use]
    pub fn new(id: HirNodeIdV1, span: SpanV1, kind: MetaKindV1) -> Self {
        Self { id, span, kind }
    }
}

/// Payload of a [`MetaHirV1`] directive.
#[derive(Clone, Debug, PartialEq)]
pub enum MetaKindV1 {
    /// `USE PACK <id> activation(session|workspace) lockfile(*.lock)`.
    UsePack(UsePackHirV1),
    /// `REQUIRE CERTIFIED` (no args): forces all profiles to certified-only.
    RequireCertified,
}

/// USE PACK activation excludes the catalog PACK-only manual policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UsePackActivationV1 {
    /// Session-scoped activation.
    Session,
    /// Workspace-scoped activation.
    Workspace,
}

/// Valid-by-construction USE PACK payload.
///
/// A lockfile is required by the type:
/// ```compile_fail
/// use quanta_gqlang_v2_hir::{UsePackActivationV1, UsePackHirV1};
/// let _ = UsePackHirV1 {
///     pack_id: "p".into(),
///     activation: UsePackActivationV1::Session,
///     lockfile: None,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsePackHirV1 {
    /// Pack id as written.
    pub pack_id: String,
    /// Restricted activation policy.
    pub activation: UsePackActivationV1,
    /// Required lockfile path.
    pub lockfile: String,
}
