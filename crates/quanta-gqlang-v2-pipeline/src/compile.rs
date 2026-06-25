//! The single end-to-end compile entry point (ticket 08, D22).
//!
//! [`compile_program_structural`] runs the FULL v3 frontend stack against a source string
//! and routes EVERY parsed root family through the one shared compiler. There is
//! no second parser / binder / type / algebra / planner anywhere downstream
//! (D22). Stages run in order and SHORT-CIRCUIT on the first failure, returning a
//! stage-tagged [`PipelineDiagnosticV1`] that preserves the inner crate's typed
//! code and span (D10 fail-closed).
//!
//! ## Per-family routing
//!
//! `surface::parse_program` lowers a root to one of the [`SurfaceItemV1`]
//! carriers; the unified [`ProgramItemHirV1`] payload (FIND / CHECK /
//! presentation / catalog / composition / meta) joins the original
//! Query/Decl/Rule carriers. Each family runs the right stages and lands in one
//! [`CompiledItemV1`] variant:
//!
//! - relational `Query` (and a `SELECT`-pipe inside `Program`): surface -> bind
//!   -> type -> algebra -> plan, the existing path, kept byte-for-byte.
//! - value `Decl`: surface -> bind -> type -> explicit non-servable value
//!   artifact.
//! - `FIND`: surface -> bind -> the logic graph-query contract -> planning
//!   classify. An unbounded / illegal FIND rejects tagged `logic`.
//! - `CHECK` / presentation verbs: surface -> bind (local refs only) -> planning
//!   classify.
//! - horn `Rule`: surface -> type -> source-rule lowering -> recursive-program
//!   artifact with explicit stratification proof.
//! - catalog declarations: surface -> modules catalog register; produces NO plan
//!   (a duplicate / cycle / certified violation rejects tagged `modules`).
//! - composition (`LET..WITH`): binder scope first (undefined / duplicate
//!   binding reject), then each producer + the body compile recursively.
//! - meta (`USE PACK` / `REQUIRE CERTIFIED`): modules validates the directive,
//!   then planning classifies its effect.
//!
//! ## Type-environment seam (schema-aware, no schema authority here)
//!
//! The semantic surface takes an explicit [`FrontendContextV1`] carrying a
//! frozen module graph plus schema/type authority. The type stage still only
//! checks expressions CLOSED under the available authority, but relation-source
//! schemas now lift into local alias/source row bindings, so alias-dependent
//! query expressions are typed rather than silently skipped when schema
//! authority is supplied.

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{normalize_query, AlgebraNodeV1, NormalizeOptionsV1};
use quanta_gqlang_v2_binder::{Binder, ModuleGraphV1};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CatalogDeclHirV1, CompositionHirV1, DeclHirV1, GraphQueryHirV1,
    HirBuilderV1, LegalityHirV1, MetaHirV1, MetaKindV1, PresentationHirV1, ProgramItemHirV1,
    QueryHirV1, RelationExprHirV1, RelationExprKindV1, ResultShapeHirV1, ResultShapeKindV1,
    RuleHirV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_logic::{
    BodyLiteralV1, GraphQuerySemanticsV1, LogicDiagnosticV1, RecursiveProgramV1, RuleDefV1,
    StratificationProofV1,
};
use quanta_gqlang_v2_modules::{CatalogV1, PackGraphV1};
use quanta_gqlang_v2_planning::{
    classify_item, produce_plan, EffectClassV1, ItemClassificationV1, PlanRequestV1, PublicPlanV1,
    RecursionShapeV1, RequestedResultKindV1, TruthLaneV1,
};
use quanta_gqlang_v2_surface::{parse_program, SurfaceItemV1};
use quanta_gqlang_v2_types::{BuiltinRegistryV1, CardinalityV1, TypeCheckerV1, TypeEnvV1, TypeV1};

use crate::diagnostic::PipelineDiagnosticV1;

/// Explicit frontend semantic authority bundle.
///
/// Semantic compile now consumes a frozen module graph plus a source-schema/type
/// environment through one inspectable value rather than inventing empty
/// defaults internally. Structural and schema-only helpers remain available,
/// but they are named explicitly and are not semantic proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendContextV1 {
    module_graph: ModuleGraphV1,
    schema: TypeEnvV1,
}

impl FrontendContextV1 {
    /// Build a context from explicit semantic authority.
    #[must_use]
    pub fn new(module_graph: ModuleGraphV1, schema: TypeEnvV1) -> Self {
        Self {
            module_graph,
            schema,
        }
    }

    /// Structural-only default context: empty module graph and empty schema.
    #[must_use]
    pub fn structural_only() -> Self {
        Self::new(ModuleGraphV1::new(), TypeEnvV1::new())
    }

    /// Explicit partial-authority constructor for callers that only have schema
    /// authority.
    #[must_use]
    pub fn schema_only(schema: TypeEnvV1) -> Self {
        Self::new(ModuleGraphV1::new(), schema)
    }

    /// Frozen module/import graph authority.
    #[must_use]
    pub fn module_graph(&self) -> &ModuleGraphV1 {
        &self.module_graph
    }

    /// Frozen source-schema/type authority.
    #[must_use]
    pub fn schema(&self) -> &TypeEnvV1 {
        &self.schema
    }
}

/// Why a compiled frontend artifact is not servable at this seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoServeReasonV1 {
    /// A value declaration compiled as a frontend-owned value artifact.
    ValueDeclaration,
    /// A horn rule compiled as a frontend-owned logic artifact.
    RuleArtifact,
}

impl NoServeReasonV1 {
    /// Stable keyword for CLI/LSP/tests.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::ValueDeclaration => "value_declaration",
            Self::RuleArtifact => "rule_artifact",
        }
    }
}

/// The per-family compiled outcome.
///
/// Each variant captures exactly the stages that family runs, so no outcome is
/// flattened into a relational-only shape. The shared HIR is retained in every
/// variant so callers recover the parsed tree without re-parsing.
#[derive(Clone, Debug, PartialEq)]
pub enum CompiledItemV1 {
    /// A relational query (SELECT pipe): the full surface -> bind -> type ->
    /// algebra -> plan path. `hir` is the relational query carrier.
    RelationalQuery {
        /// The query HIR.
        hir: Box<QueryHirV1>,
        /// The normalized algebra root the plan keys off.
        algebra: Box<AlgebraNodeV1>,
        /// The host-agnostic public plan.
        plan: Box<PublicPlanV1>,
    },
    /// A non-relational value declaration compiled into a host-agnostic
    /// frontend artifact with no serving plan.
    ValueArtifact {
        /// The declaration HIR.
        hir: Box<DeclHirV1>,
        /// The inferred value type when the body closed under the supplied
        /// authority; `None` when the body stayed authority-open.
        value_type: Option<TypeV1>,
        /// Explicit non-serve classification.
        no_serve_reason: NoServeReasonV1,
    },
    /// A graph query (FIND): bind -> logic contract -> planning classify. The
    /// plan's `result_kind` is `path_witness`.
    GraphQuery {
        /// The FIND HIR.
        hir: Box<GraphQueryHirV1>,
        /// The classified path-witness plan.
        plan: Box<PublicPlanV1>,
    },
    /// A horn rule compiled into a frontend-owned recursive-program artifact
    /// with an explicit least-fixpoint/stratification proof and no serving plan.
    RuleArtifact {
        /// The rule HIR.
        hir: Box<RuleHirV1>,
        /// The lowered recursive program rooted at this rule.
        program: Box<RecursiveProgramV1>,
        /// The explicit recursion admission artifact.
        proof: Box<StratificationProofV1>,
        /// Explicit non-serve classification.
        no_serve_reason: NoServeReasonV1,
    },
    /// A legality check (CHECK): bind -> planning classify. The plan's
    /// `result_kind` is `alert`.
    Legality {
        /// The CHECK HIR.
        hir: Box<LegalityHirV1>,
        /// The classified alert plan.
        plan: Box<PublicPlanV1>,
    },
    /// A presentation verb (EXPLAIN / SHOW / DESCRIBE / SAMPLE / COMPARE /
    /// EXPORT): bind -> planning classify.
    Presentation {
        /// The presentation HIR.
        hir: Box<PresentationHirV1>,
        /// The classified plan (result-kind / effect vary by verb).
        plan: Box<PublicPlanV1>,
    },
    /// A catalog declaration (VIEW / MODEL / PACK / PROFILE / GATE / RULE):
    /// registered in the modules catalog, producing NO plan.
    CatalogRegistration {
        /// The declaration HIR.
        hir: Box<CatalogDeclHirV1>,
        /// The minted stable `<kind>::<name>` asset id.
        asset_id: String,
    },
    /// A composition (`LET..WITH`): each producer and the body compile
    /// recursively after binder scope resolution.
    Composition {
        /// The composition HIR.
        hir: Box<CompositionHirV1>,
        /// Each binding's compiled producer, in written order.
        bindings: Vec<CompiledItemV1>,
        /// The OUTPUT relation type each binding lifts into the local type
        /// environment for later producers and the body (D34 capture lane),
        /// positionally aligned with `bindings`. `None` when the producer
        /// denotes no row schema (non-relational, or a projection that names no
        /// columns), so the binding name stays schema-open. When present, the
        /// relation's cardinality is the producer's ACTUAL compiled-algebra
        /// cardinality (D26), not a hardcoded default.
        binding_outputs: Vec<Option<TypeV1>>,
        /// The compiled body.
        body: Box<CompiledItemV1>,
    },
    /// A meta directive (USE PACK / REQUIRE CERTIFIED): validated in modules,
    /// then classified for its effect. `USE PACK` produces no plan
    /// (`plan = None`); `REQUIRE CERTIFIED` is a proof-confirmed gate.
    Meta {
        /// The meta HIR.
        hir: Box<MetaHirV1>,
        /// The classified gate plan, when the directive carries one.
        plan: Option<Box<PublicPlanV1>>,
    },
}

/// The fully compiled program: the parsed [`SurfaceItemV1`] plus the per-family
/// [`CompiledItemV1`] outcome.
///
/// Deterministic by construction: every contained type derives `PartialEq` and
/// every stage is order-deterministic, so compiling the same source twice yields
/// an equal value (see the determinism rail).
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledProgramV1 {
    item: SurfaceItemV1,
    compiled: CompiledItemV1,
}

impl CompiledProgramV1 {
    /// The shared HIR surface item as the parser produced it.
    #[must_use]
    pub fn item(&self) -> &SurfaceItemV1 {
        &self.item
    }

    /// The per-family compiled outcome.
    #[must_use]
    pub fn compiled(&self) -> &CompiledItemV1 {
        &self.compiled
    }

    /// The query HIR, if this program is a relational query unit.
    #[must_use]
    pub fn query(&self) -> Option<&QueryHirV1> {
        match &self.compiled {
            CompiledItemV1::RelationalQuery { hir, .. } => Some(hir),
            CompiledItemV1::ValueArtifact { .. }
            | CompiledItemV1::GraphQuery { .. }
            | CompiledItemV1::Legality { .. }
            | CompiledItemV1::Presentation { .. }
            | CompiledItemV1::RuleArtifact { .. }
            | CompiledItemV1::CatalogRegistration { .. }
            | CompiledItemV1::Composition { .. }
            | CompiledItemV1::Meta { .. } => None,
        }
    }

    /// The normalized algebra root, if this program compiled a relational query.
    /// Only the relational path produces an algebra root; other families key
    /// their plan off the logic/item classification, not a relational tree.
    #[must_use]
    pub fn algebra(&self) -> Option<&AlgebraNodeV1> {
        match &self.compiled {
            CompiledItemV1::RelationalQuery { algebra, .. } => Some(algebra),
            CompiledItemV1::ValueArtifact { .. }
            | CompiledItemV1::GraphQuery { .. }
            | CompiledItemV1::Legality { .. }
            | CompiledItemV1::Presentation { .. }
            | CompiledItemV1::RuleArtifact { .. }
            | CompiledItemV1::CatalogRegistration { .. }
            | CompiledItemV1::Composition { .. }
            | CompiledItemV1::Meta { .. } => None,
        }
    }

    /// The host-agnostic public plan, when this program produces one. Catalog
    /// declarations and `USE PACK` register/activate but produce no plan.
    #[must_use]
    pub fn plan(&self) -> Option<&PublicPlanV1> {
        self.compiled.plan()
    }
}

impl CompiledItemV1 {
    /// The host-agnostic public plan this outcome carries, if any.
    #[must_use]
    pub fn plan(&self) -> Option<&PublicPlanV1> {
        match self {
            Self::RelationalQuery { plan, .. }
            | Self::GraphQuery { plan, .. }
            | Self::Legality { plan, .. }
            | Self::Presentation { plan, .. } => Some(plan),
            Self::Meta { plan, .. } => plan.as_deref(),
            Self::Composition { body, .. } => body.plan(),
            Self::ValueArtifact { .. }
            | Self::RuleArtifact { .. }
            | Self::CatalogRegistration { .. } => None,
        }
    }

    /// The lifted OUTPUT relation types this outcome's composition bindings
    /// expose to later producers and the body, positionally aligned with the
    /// bindings; `None` for non-composition outcomes. Each present relation
    /// carries the producer's actual compiled-algebra cardinality (D26).
    #[must_use]
    pub fn binding_outputs(&self) -> Option<&[Option<TypeV1>]> {
        match self {
            Self::Composition {
                binding_outputs, ..
            } => Some(binding_outputs),
            Self::RelationalQuery { .. }
            | Self::GraphQuery { .. }
            | Self::Legality { .. }
            | Self::Presentation { .. }
            | Self::Meta { .. }
            | Self::ValueArtifact { .. }
            | Self::RuleArtifact { .. }
            | Self::CatalogRegistration { .. } => None,
        }
    }
}

/// Compile a source string through the full v3 frontend stack with explicit
/// structural-only authority.
///
/// # Errors
///
/// Returns a stage-tagged [`PipelineDiagnosticV1`] on the first failing stage,
/// preserving the inner stage diagnostic's stable code and span.
pub fn compile_program_structural(source: &str) -> Result<CompiledProgramV1, PipelineDiagnosticV1> {
    compile_with_context(source, &FrontendContextV1::structural_only())
}

/// Compile a source string through the full stack against a caller-supplied
/// schema [`TypeEnvV1`] only.
///
/// # Errors
///
/// Returns a stage-tagged [`PipelineDiagnosticV1`] on the first failing stage.
pub fn compile_program_with_schema(
    source: &str,
    schema: &TypeEnvV1,
) -> Result<CompiledProgramV1, PipelineDiagnosticV1> {
    compile_with_context(source, &FrontendContextV1::schema_only(schema.clone()))
}

/// Compile a source string through the full stack against explicit semantic
/// authority.
///
/// # Errors
///
/// Returns a stage-tagged [`PipelineDiagnosticV1`] on the first failing stage.
pub fn compile_with_context(
    source: &str,
    context: &FrontendContextV1,
) -> Result<CompiledProgramV1, PipelineDiagnosticV1> {
    // ---- stage 1: parse ----------------------------------------------------
    let builder = HirBuilderV1::new();
    let parsed = parse_program(source, &builder).map_err(PipelineDiagnosticV1::Parse)?;
    let item = parsed.item;

    // The unified item view used for binding, logic, and classification. The
    // original carriers map onto the same `ProgramItemHirV1` the new families
    // already use, so one routine handles every family.
    let program = item.to_program_item();

    let compiled = compile_item_with_context(&program, context)?;
    Ok(CompiledProgramV1 { item, compiled })
}

/// Compile one already-parsed program item through the stages its family runs.
///
/// This is the recursive core every entry point funnels through: top-level
/// items, composition producers, and composition bodies all compile here, so the
/// per-family stage routing is defined in exactly one place.
///
/// # Errors
///
/// Returns a stage-tagged [`PipelineDiagnosticV1`] on the first failing stage.
pub fn compile_item_structural(
    item: &ProgramItemHirV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    compile_item_with_context(item, &FrontendContextV1::structural_only())
}

/// Compile one already-parsed program item through the stages its family runs
/// against a caller-supplied schema [`TypeEnvV1`] only.
///
/// # Errors
///
/// Returns a stage-tagged [`PipelineDiagnosticV1`] on the first failing stage.
pub fn compile_item_with_schema(
    item: &ProgramItemHirV1,
    schema: &TypeEnvV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    compile_item_with_context(item, &FrontendContextV1::schema_only(schema.clone()))
}

/// Compile one already-parsed program item through explicit semantic authority.
///
/// # Errors
///
/// Returns a stage-tagged [`PipelineDiagnosticV1`] on the first failing stage.
pub fn compile_item_with_context(
    item: &ProgramItemHirV1,
    context: &FrontendContextV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    // A stand-alone item registers into its own throwaway catalog; cross-item
    // duplicate / cycle / certified rejection needs a SHARED catalog, which is
    // what the document entry points thread.
    let mut catalog = CatalogV1::new();
    compile_item_in(item, context, &mut catalog)
}

/// Compile a document of program items that share ONE catalog, so cross-item
/// catalog rules fire: a second declaration colliding on `<kind>::<name>`, a
/// VIEW inline-select cycle across declarations, or a REQUIRE CERTIFIED that a
/// later PROFILE violates rejects tagged `modules`. Each item compiles in order
/// and the first failing stage short-circuits the whole document (D10).
///
/// `parse_program` lowers a single root, so callers build the item list (one
/// per parsed source) and feed it here; this is the multi-item path the
/// single-root parser cannot express on its own.
///
/// # Errors
///
/// Returns the first failing item's stage-tagged [`PipelineDiagnosticV1`].
pub fn compile_document_structural(
    items: &[ProgramItemHirV1],
) -> Result<Vec<CompiledItemV1>, PipelineDiagnosticV1> {
    compile_document_with_context(items, &FrontendContextV1::structural_only())
}

/// Compile a document of program items that share ONE catalog against a
/// caller-supplied schema [`TypeEnvV1`] only.
///
/// # Errors
///
/// Returns the first failing item's stage-tagged [`PipelineDiagnosticV1`].
pub fn compile_document_with_schema(
    items: &[ProgramItemHirV1],
    schema: &TypeEnvV1,
) -> Result<Vec<CompiledItemV1>, PipelineDiagnosticV1> {
    compile_document_with_context(items, &FrontendContextV1::schema_only(schema.clone()))
}

/// Compile a document of items through explicit semantic authority.
///
/// # Errors
///
/// Returns the first failing item's stage-tagged [`PipelineDiagnosticV1`].
pub fn compile_document_with_context(
    items: &[ProgramItemHirV1],
    context: &FrontendContextV1,
) -> Result<Vec<CompiledItemV1>, PipelineDiagnosticV1> {
    let mut catalog = CatalogV1::new();
    let mut compiled = Vec::with_capacity(items.len());
    for item in items {
        compiled.push(compile_item_in(item, context, &mut catalog)?);
    }
    Ok(compiled)
}

/// The per-family routing core, threaded with the catalog the registration /
/// meta stages register into.
fn compile_item_in(
    item: &ProgramItemHirV1,
    context: &FrontendContextV1,
    catalog: &mut CatalogV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    match item {
        ProgramItemHirV1::Query(query) => compile_relational(query, context),
        ProgramItemHirV1::Decl(decl) => compile_decl(decl, context),
        ProgramItemHirV1::Rule(rule) => compile_rule(rule, context),
        ProgramItemHirV1::GraphQuery(find) => compile_find(context, item, find),
        ProgramItemHirV1::Legality(check) => compile_legality(context, item, check),
        ProgramItemHirV1::Presentation(pres) => compile_presentation(context, item, pres),
        ProgramItemHirV1::CatalogDecl(decl) => compile_catalog_decl(decl, catalog),
        ProgramItemHirV1::Composition(comp) => compile_composition(comp, context, catalog),
        ProgramItemHirV1::Meta(meta) => compile_meta(meta, catalog),
    }
}

// ---- relational path (unchanged stages) ------------------------------------

/// The existing relational path: bind -> type -> algebra -> plan.
fn compile_relational(
    query: &QueryHirV1,
    context: &FrontendContextV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    // ---- bind --------------------------------------------------------------
    let mut binder = Binder::new(context.module_graph());
    binder.resolve_query(query);
    finish_bind(binder)?;

    // ---- type --------------------------------------------------------------
    let checker = TypeCheckerV1::new();
    let local_env = type_check_relation(&checker, &query.body, context.schema())?;
    if let Some(shape) = &query.result_shape {
        type_check_shape(&checker, shape, &merged_env(context.schema(), &local_env))?;
    }

    // ---- algebra -----------------------------------------------------------
    let options = aggregate_options(checker.builtins());
    let algebra = normalize_query(query, &options).map_err(PipelineDiagnosticV1::Algebra)?;

    // ---- logic (relational queries carry no source-level recursion here) ----
    let recursion = RecursionShapeV1::None;

    // ---- planning ----------------------------------------------------------
    let request = PlanRequestV1 {
        root: algebra.clone(),
        recursion,
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let plan = produce_plan(&request).map_err(PipelineDiagnosticV1::Planning)?;

    Ok(CompiledItemV1::RelationalQuery {
        hir: Box::new(query.clone()),
        algebra: Box::new(algebra),
        plan: Box::new(plan),
    })
}

/// A `let` declaration: bind + type-check, then produce an explicit
/// non-servable value artifact.
fn compile_decl(
    decl: &DeclHirV1,
    context: &FrontendContextV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    let mut binder = Binder::new(context.module_graph());
    binder.resolve_decl(decl, &[]);
    finish_bind(binder)?;

    let checker = TypeCheckerV1::new();
    let value_type = infer_value_type(&checker, &decl.body, context.schema())?;

    Ok(CompiledItemV1::ValueArtifact {
        hir: Box::new(decl.clone()),
        value_type,
        no_serve_reason: NoServeReasonV1::ValueDeclaration,
    })
}

/// A horn `rule`: type-check what is closed under current authority, then lower
/// into a recursive-program artifact with explicit stratification proof.
fn compile_rule(
    rule: &RuleHirV1,
    context: &FrontendContextV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    let checker = TypeCheckerV1::new();
    type_check_value(&checker, &rule.head, context.schema())?;
    type_check_bool(&checker, &rule.body, context.schema())?;

    let program = lower_source_rule(rule)?;
    let proof = program.stratify().map_err(PipelineDiagnosticV1::Logic)?;

    Ok(CompiledItemV1::RuleArtifact {
        hir: Box::new(rule.clone()),
        program: Box::new(program),
        proof: Box::new(proof),
        no_serve_reason: NoServeReasonV1::RuleArtifact,
    })
}

// ---- FIND ------------------------------------------------------------------

/// FIND: bind (local endpoint refs) -> the logic graph-query contract -> the
/// planning classification. An illegal endpoint shape or an unbounded witness
/// family rejects tagged `logic`; the inner logic code/span is preserved.
fn compile_find(
    context: &FrontendContextV1,
    item: &ProgramItemHirV1,
    find: &GraphQueryHirV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    bind_item(item, context.module_graph())?;

    // Logic stage: the graph-query contract owns endpoint-presence + bounded
    // witness legality. Run it explicitly so an unbounded FIND is tagged `logic`
    // (rather than relayed through planning).
    GraphQuerySemanticsV1::classify(find).map_err(PipelineDiagnosticV1::Logic)?;

    let plan = classify_to_plan(item)?;
    Ok(CompiledItemV1::GraphQuery {
        hir: Box::new(find.clone()),
        plan: Box::new(plan),
    })
}

// ---- CHECK -----------------------------------------------------------------

/// CHECK: bind (local refs) -> planning classify -> `alert` plan.
fn compile_legality(
    context: &FrontendContextV1,
    item: &ProgramItemHirV1,
    check: &LegalityHirV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    bind_item(item, context.module_graph())?;
    let plan = classify_to_plan(item)?;
    Ok(CompiledItemV1::Legality {
        hir: Box::new(check.clone()),
        plan: Box::new(plan),
    })
}

// ---- presentation ----------------------------------------------------------

/// Presentation verbs: bind (local refs) -> planning classify. The result-kind /
/// effect vary per verb (EXPLAIN -> proof_explain, SHOW/SAMPLE -> table,
/// DESCRIBE -> alert, COMPARE -> path_witness, EXPORT -> export-only).
fn compile_presentation(
    context: &FrontendContextV1,
    item: &ProgramItemHirV1,
    pres: &PresentationHirV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    bind_item(item, context.module_graph())?;
    let plan = classify_to_plan(item)?;
    Ok(CompiledItemV1::Presentation {
        hir: Box::new(pres.clone()),
        plan: Box::new(plan),
    })
}

// ---- catalog declarations --------------------------------------------------

/// A catalog declaration: register it in a fresh modules [`CatalogV1`]. A
/// duplicate `<kind>::<name>` (or a VIEW inline cycle / certified violation on
/// resolve) rejects tagged `modules`, pairing the inner code with the decl span.
fn compile_catalog_decl(
    decl: &CatalogDeclHirV1,
    catalog: &mut CatalogV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    catalog
        .register(decl.clone())
        .map_err(|module| PipelineDiagnosticV1::Modules(Box::new(module), decl.span))?;
    // Resolving the (possibly multi-decl) catalog enforces VIEW inline-select
    // cycles and REQUIRE CERTIFIED admission forcing.
    catalog
        .resolve()
        .map_err(|module| PipelineDiagnosticV1::Modules(Box::new(module), decl.span))?;

    // The stable asset id is `<kind>::<name>`, matching the catalog's minting.
    let kind = quanta_gqlang_v2_modules::CatalogKindV1::of_payload(&decl.payload);
    let asset_id = format!("{}::{}", kind.tag(), decl.name);
    Ok(CompiledItemV1::CatalogRegistration {
        hir: Box::new(decl.clone()),
        asset_id,
    })
}

// ---- composition -----------------------------------------------------------

/// A composition: the binder owns LET..WITH local scope (undefined / duplicate
/// binding, illegal capture reject tagged `bind`); then each producer and the
/// body compile recursively through [`compile_item_in`], threading the document
/// catalog so cross-item catalog rules (a producer and the body declaring the
/// same `<kind>::<name>` asset) fire across the composition's nested items.
fn compile_composition(
    comp: &CompositionHirV1,
    context: &FrontendContextV1,
    catalog: &mut CatalogV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    // ---- bind: LET..WITH scope ---------------------------------------------
    let mut binder = Binder::new(context.module_graph());
    binder.resolve_composition(comp);
    finish_bind(binder)?;

    // ---- compile each producer + the body recursively ----------------------
    // Producers and the body share the document catalog scope (D10): a duplicate
    // catalog declaration across nested items rejects tagged `modules`, just as
    // it does at the document top level.
    //
    // D34 (capture lane): each `LET <name> = <producer>` binding is binder-visible
    // to later producers and the body. Its OUTPUT row type must therefore lift
    // into the local type environment so a body/later-producer expression over a
    // producer column (`from <name> x | where x.col ...`) is type-checked, not
    // silently skipped because the source name stayed schema-open. Accumulate each
    // producer's output relation type into a working schema and compile subsequent
    // producers and the body against that augmented context.
    let mut local_context = context.clone();
    let mut bindings = Vec::with_capacity(comp.bindings.len());
    let mut binding_outputs = Vec::with_capacity(comp.bindings.len());
    for binding in &comp.bindings {
        let compiled = compile_item_in(&binding.producer, &local_context, catalog)?;
        // D26: lift the producer's ACTUAL normalized output cardinality, not a
        // hardcoded Bag. A relational producer carries its algebra root, whose
        // `cardinality` is the explicit multiplicity / duplicate-elimination /
        // order semantics the plan keys off. Threading it into the lifted schema
        // relation keeps that semantics visible to downstream producers and the
        // body, instead of silently downgrading a Set/Sequence producer to Bag.
        let producer_cardinality = match &compiled {
            CompiledItemV1::RelationalQuery { algebra, .. } => algebra.cardinality,
            CompiledItemV1::ValueArtifact { .. }
            | CompiledItemV1::GraphQuery { .. }
            | CompiledItemV1::RuleArtifact { .. }
            | CompiledItemV1::Legality { .. }
            | CompiledItemV1::Presentation { .. }
            | CompiledItemV1::CatalogRegistration { .. }
            | CompiledItemV1::Composition { .. }
            | CompiledItemV1::Meta { .. } => CardinalityV1::Bag,
        };
        let relation =
            producer_output_relation(&binding.producer, &local_context, producer_cardinality);
        if let Some(relation) = &relation {
            let mut schema = local_context.schema().clone();
            schema.insert(binding.name.clone(), relation.clone());
            local_context = FrontendContextV1::new(local_context.module_graph().clone(), schema);
        }
        binding_outputs.push(relation);
        bindings.push(compiled);
    }
    let body = compile_item_in(&comp.body, &local_context, catalog)?;

    Ok(CompiledItemV1::Composition {
        hir: Box::new(comp.clone()),
        bindings,
        binding_outputs,
        body: Box::new(body),
    })
}

// ---- meta ------------------------------------------------------------------

/// A meta directive: modules validates it (`USE PACK` lockfile / manifest;
/// `REQUIRE CERTIFIED` flips the catalog flag), then planning classifies its
/// effect. `USE PACK` produces no plan; `REQUIRE CERTIFIED` is a gate plan.
fn compile_meta(
    meta: &MetaHirV1,
    catalog: &mut CatalogV1,
) -> Result<CompiledItemV1, PipelineDiagnosticV1> {
    // Modules stage: validate the directive against an empty pack graph. A
    // REQUIRE CERTIFIED needs no manifest; a USE PACK with an unknown pack id (or
    // a bad lockfile path) rejects tagged `modules`.
    let manifests = PackGraphV1::new();
    catalog
        .apply_meta(meta, &manifests)
        .map_err(|module| PipelineDiagnosticV1::Modules(Box::new(module), meta.span))?;

    // Planning stage: classify the directive's effect.
    let plan = match meta.kind {
        // USE PACK is effect-style activation with no plan.
        MetaKindV1::UsePack { .. } => None,
        // REQUIRE CERTIFIED is a proof-confirmed truth gate.
        MetaKindV1::RequireCertified => {
            let item = ProgramItemHirV1::from(meta.clone());
            Some(Box::new(classify_to_plan(&item)?))
        }
    };

    Ok(CompiledItemV1::Meta {
        hir: Box::new(meta.clone()),
        plan,
    })
}

// ---- composition producer output schema (D34 capture lane) -----------------

/// The output relation type a composition producer exposes to later producers
/// and the body, when one is derivable under the supplied authority.
///
/// Only a relational-query producer denotes a row-shaped working set whose
/// columns a downstream `from <name> x | ... x.col ...` expression references.
/// Its output row is the record of its TOP-LEVEL projection aliases, each typed
/// under the same lifted source-local environment the relational path already
/// uses (`type_check_relation`). A projection item whose type stays
/// authority-open contributes no field rather than a guessed one, so the lifted
/// row never over-binds: a downstream reference to such a column simply stays
/// env-open exactly as it would have without lifting.
///
/// Non-relational producers (FIND / CHECK / presentation / value decl / rule /
/// catalog / meta / nested composition) carry no row schema to lift here, so
/// `None` is returned and the binding name stays schema-open (no silent
/// over-binding, D34).
///
/// The lifted relation's `cardinality` is the producer's ACTUAL normalized
/// output cardinality (D26): the caller threads the producer algebra root's
/// `cardinality` so a Set/Sequence producer is not silently downgraded to Bag
/// when its schema is lifted into the local type environment.
fn producer_output_relation(
    producer: &ProgramItemHirV1,
    context: &FrontendContextV1,
    cardinality: CardinalityV1,
) -> Option<TypeV1> {
    let ProgramItemHirV1::Query(query) = producer else {
        return None;
    };
    let checker = TypeCheckerV1::new();
    let output_env = type_check_relation(&checker, &query.body, context.schema()).ok()?;
    let local_env = merged_env(context.schema(), &output_env);
    let fields = projection_row_fields(&checker, &query.body, &local_env)?;
    // D26: carry the producer's ACTUAL normalized output cardinality so the lifted
    // schema relation preserves multiplicity / duplicate-elimination / order
    // semantics rather than silently defaulting every producer to Bag.
    Some(TypeV1::Relation {
        row: Box::new(TypeV1::record(fields)),
        cardinality,
    })
}

/// The top-level projection's aliased output columns, typed under `env`. Returns
/// `None` when the relation does not terminate in a projection (no nameable
/// output columns to lift). A projection item without an alias, or one whose
/// value type stays authority-open, contributes no field.
fn projection_row_fields(
    checker: &TypeCheckerV1,
    relation: &RelationExprHirV1,
    env: &TypeEnvV1,
) -> Option<Vec<quanta_gqlang_v2_types::RecordFieldTyV1>> {
    let RelationExprKindV1::Project(project) = &relation.kind else {
        return None;
    };
    let mut fields = Vec::new();
    for item in &project.items {
        if let Some(alias) = &item.alias {
            if let Ok(Some(ty)) = infer_value_type(checker, &item.value, env) {
                fields.push(quanta_gqlang_v2_types::RecordFieldTyV1::new(
                    alias.name.clone(),
                    ty,
                ));
            }
        }
    }
    // A projection that contributes NO named columns (every item is alias-less, or
    // every aliased item stays authority-open) denotes no nameable output row. Lift
    // `None` rather than `Some(vec![])` so the binding is not created against a
    // phantom empty-record relation: the name stays schema-open and a downstream
    // reference stays env-open exactly as it would have without lifting (D34, the
    // lifted row never over-binds).
    if fields.is_empty() {
        return None;
    }
    Some(fields)
}

// ---- shared stage helpers --------------------------------------------------

/// Bind one program item against the default (empty) frozen module graph,
/// surfacing the first diagnostic in deterministic pre-order, fail-closed.
fn bind_item(item: &ProgramItemHirV1, graph: &ModuleGraphV1) -> Result<(), PipelineDiagnosticV1> {
    let mut binder = Binder::new(graph);
    match item {
        ProgramItemHirV1::Query(query) => binder.resolve_query(query),
        ProgramItemHirV1::Decl(decl) => binder.resolve_decl(decl, &[]),
        ProgramItemHirV1::Rule(rule) => binder.resolve_rule(rule, &[]),
        ProgramItemHirV1::Composition(comp) => binder.resolve_composition(comp),
        // FIND / CHECK / presentation / catalog / meta resolve their LOCAL refs
        // through the composition resolver's per-family arm; a stand-alone item
        // has no enclosing LET scope, so only illegal-capture / unresolved local
        // bindings can reject here (catalog refs stay modules' concern, D30).
        other @ (ProgramItemHirV1::GraphQuery(_)
        | ProgramItemHirV1::Legality(_)
        | ProgramItemHirV1::Presentation(_)
        | ProgramItemHirV1::CatalogDecl(_)
        | ProgramItemHirV1::Meta(_)) => bind_standalone(&mut binder, other),
    }
    finish_bind(binder)
}

/// Resolve a stand-alone (non-composition) item's local references by wrapping it
/// in a trivial composition frame the binder already knows how to walk. The frame
/// introduces no bindings, so only genuine local-scope rejects surface.
fn bind_standalone(binder: &mut Binder<'_, ModuleGraphV1>, item: &ProgramItemHirV1) {
    binder.open_scope(quanta_gqlang_v2_binder::CaptureV1::Capturing);
    resolve_local(binder, item);
    binder.close_scope();
}

/// Dispatch a single item's local-ref resolution. The binder exposes
/// `resolve_query` / `resolve_decl` / `resolve_rule` directly; the new families
/// only carry local refs that resolve to enclosing LET bindings, which a
/// stand-alone item never has, so resolving them is a no-op that still asserts
/// the family is handled (no silent skip).
fn resolve_local(binder: &mut Binder<'_, ModuleGraphV1>, item: &ProgramItemHirV1) {
    match item {
        ProgramItemHirV1::Query(query) => binder.resolve_query(query),
        ProgramItemHirV1::Decl(decl) => binder.resolve_decl(decl, &[]),
        ProgramItemHirV1::Rule(rule) => binder.resolve_rule(rule, &[]),
        ProgramItemHirV1::Composition(comp) => binder.resolve_composition(comp),
        // FIND / CHECK / presentation / catalog / meta carry only refs whose
        // legality is the catalog/modules seam's (D30); with no local scope to
        // bind into there is nothing for the binder to resolve or reject.
        ProgramItemHirV1::GraphQuery(_)
        | ProgramItemHirV1::Legality(_)
        | ProgramItemHirV1::Presentation(_)
        | ProgramItemHirV1::CatalogDecl(_)
        | ProgramItemHirV1::Meta(_) => {}
    }
}

/// Drain a binder's accumulated diagnostics, surfacing the first deterministic
/// reject as a `bind`-stage failure (D10 fail-closed).
fn finish_bind(binder: Binder<'_, ModuleGraphV1>) -> Result<(), PipelineDiagnosticV1> {
    let result = binder.finish();
    if let Some(first) = result.diagnostics().first() {
        return Err(PipelineDiagnosticV1::Bind(first.clone()));
    }
    Ok(())
}

/// Run planning's per-item classification, mapping a NO-PLAN marker or a reject
/// into a `planning`-stage failure. Only the families that DO produce a plan
/// (FIND / CHECK / presentation / REQUIRE CERTIFIED) call this.
fn classify_to_plan(item: &ProgramItemHirV1) -> Result<PublicPlanV1, PipelineDiagnosticV1> {
    match classify_item(item) {
        ItemClassificationV1::Plan(plan) => Ok(plan),
        ItemClassificationV1::Reject(planning) => Err(PipelineDiagnosticV1::Planning(planning)),
        // A family routed here is expected to plan; a NO-PLAN marker means the
        // routing is wrong. Surface it as the planning reject for the relational
        // deferral so there is no silent success.
        ItemClassificationV1::NoPlan(_) => Err(PipelineDiagnosticV1::Algebra(
            unsupported_non_query(item.span()),
        )),
    }
}

/// Derive the aggregate-callee catalog from the closed builtin registry, so the
/// normalizer does not re-implement aggregate identity (D12 / ticket 04 seam).
fn aggregate_options(builtins: &BuiltinRegistryV1) -> NormalizeOptionsV1 {
    let aggregates: BTreeSet<String> = builtins
        .entries()
        .iter()
        .filter(|entry| entry.meta.is_aggregate())
        .map(|entry| entry.name.clone())
        .collect();
    NormalizeOptionsV1::with_aggregates(aggregates)
}

/// A typed algebra diagnostic for a non-relational unit reaching the relational
/// plan path. Uses the existing `UnsupportedComposition` reject so no new
/// diagnostic vocabulary is invented out of lane.
fn unsupported_non_query(
    span: quanta_gqlang_v2_hir::SpanV1,
) -> quanta_gqlang_v2_algebra::AlgebraDiagnosticV1 {
    quanta_gqlang_v2_algebra::AlgebraDiagnosticV1::UnsupportedComposition {
        symbol: "<non-query-unit>".to_owned(),
        span,
    }
}

// ---- type stage helpers ----------------------------------------------------

fn type_check_relation(
    checker: &TypeCheckerV1,
    relation: &RelationExprHirV1,
    schema: &TypeEnvV1,
) -> Result<TypeEnvV1, PipelineDiagnosticV1> {
    match &relation.kind {
        RelationExprKindV1::Source(source) => Ok(lift_source_locals(source, schema)),
        RelationExprKindV1::Project(project) => {
            let input_env = type_check_relation(checker, &project.input, schema)?;
            let local_env = merged_env(schema, &input_env);
            let mut output_env = input_env.clone();
            for item in &project.items {
                let ty = infer_value_type(checker, &item.value, &local_env)?;
                if let (Some(alias), Some(ty)) = (&item.alias, ty) {
                    output_env.insert(alias.name.clone(), ty);
                }
            }
            Ok(output_env)
        }
        RelationExprKindV1::Filter(filter) => {
            let input_env = type_check_relation(checker, &filter.input, schema)?;
            let local_env = merged_env(schema, &input_env);
            type_check_bool(checker, &filter.predicate, &local_env)?;
            Ok(input_env)
        }
        RelationExprKindV1::Compose(compose) => {
            let mut merged = TypeEnvV1::new();
            for operand in &compose.operands {
                let operand_env = type_check_relation(checker, operand, schema)?;
                for (name, ty) in operand_env.iter() {
                    merged.insert(name.clone(), ty.clone());
                }
            }
            Ok(merged)
        }
    }
}

fn type_check_shape(
    checker: &TypeCheckerV1,
    shape: &ResultShapeHirV1,
    env: &TypeEnvV1,
) -> Result<(), PipelineDiagnosticV1> {
    match &shape.kind {
        ResultShapeKindV1::Leaf(value) => type_check_value(checker, value, env),
        ResultShapeKindV1::Record(record) => {
            for field in &record.fields {
                type_check_shape(checker, &field.shape, env)?;
            }
            Ok(())
        }
        ResultShapeKindV1::List(list) => type_check_shape(checker, &list.element, env),
    }
}

fn type_check_bool(
    checker: &TypeCheckerV1,
    node: &BoolExprHirV1,
    env: &TypeEnvV1,
) -> Result<(), PipelineDiagnosticV1> {
    match &node.kind {
        BoolExprKindV1::Literal(_) => Ok(()),
        BoolExprKindV1::Value(value) => type_check_value(checker, value, env),
        BoolExprKindV1::Compare(compare) => {
            type_check_value(checker, &compare.lhs, env)?;
            type_check_value(checker, &compare.rhs, env)
        }
        BoolExprKindV1::And(bin) | BoolExprKindV1::Or(bin) => {
            type_check_bool(checker, &bin.lhs, env)?;
            type_check_bool(checker, &bin.rhs, env)
        }
        BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
            type_check_bool(checker, inner, env)
        }
        BoolExprKindV1::Quantifier(quant) => {
            // The quantifier binders are in scope of the body. Without binding
            // them, every body reference to a binder is a free, env-open symbol,
            // so `type_check_value` would skip the whole body and a genuine type
            // error inside it (e.g. `t + "string"`) would pass silently. Extend a
            // child env binding each binder to the `null` placeholder so that a
            // body closed under the binders gets type-checked. Schema-dependent
            // binder element types remain out of this packet's lane; `null` is a
            // sound placeholder because no operator/builtin accepts it except
            // assignment into an optional, so it neither over-accepts arithmetic
            // nor invents a spurious concrete type.
            let mut body_env = env.clone();
            for binder in &quant.binders {
                body_env.insert(binder.clone(), TypeV1::Null);
            }
            type_check_bool(checker, &quant.body, &body_env)
        }
    }
}

/// Type-check a single value expression IF it is closed under `env` (every free
/// symbol is bound). Schema-dependent expressions are skipped, not rejected.
fn type_check_value(
    checker: &TypeCheckerV1,
    value: &ValueExprHirV1,
    env: &TypeEnvV1,
) -> Result<(), PipelineDiagnosticV1> {
    infer_value_type(checker, value, env).map(|_| ())
}

fn infer_value_type(
    checker: &TypeCheckerV1,
    value: &ValueExprHirV1,
    env: &TypeEnvV1,
) -> Result<Option<TypeV1>, PipelineDiagnosticV1> {
    if env_closed(value, env) {
        checker
            .type_value(value, env)
            .map(Some)
            .map_err(PipelineDiagnosticV1::Type)
    } else {
        Ok(None)
    }
}

/// Whether every free symbol referenced by `value` is bound in `env`.
fn env_closed(value: &ValueExprHirV1, env: &TypeEnvV1) -> bool {
    let mut free = BTreeSet::new();
    collect_free_symbols(value, &mut free);
    free.iter().all(|name| env.lookup(name).is_some())
}

/// Collect the free symbol names a value expression references (path bases and
/// bare symbols). Callee names are NOT symbols (they resolve against the builtin
/// registry, not the env).
fn collect_free_symbols(value: &ValueExprHirV1, acc: &mut BTreeSet<String>) {
    match &value.kind {
        ValueExprKindV1::Literal(_) => {}
        ValueExprKindV1::Symbol(symbol) => {
            let _ = acc.insert(symbol.name.clone());
        }
        ValueExprKindV1::Path(path) => collect_free_symbols(&path.base, acc),
        ValueExprKindV1::Unary(unary) => collect_free_symbols(&unary.operand, acc),
        ValueExprKindV1::Binary(binary) => {
            collect_free_symbols(&binary.lhs, acc);
            collect_free_symbols(&binary.rhs, acc);
        }
        ValueExprKindV1::Call(call) => {
            for arg in &call.args {
                collect_free_symbols(arg, acc);
            }
        }
        ValueExprKindV1::Paren(inner) => collect_free_symbols(inner, acc),
        ValueExprKindV1::Record(record) => {
            for field in &record.fields {
                collect_free_symbols(&field.value, acc);
            }
        }
        ValueExprKindV1::List(list) => {
            for element in &list.elements {
                collect_free_symbols(element, acc);
            }
        }
    }
}

fn merged_env(base: &TypeEnvV1, locals: &TypeEnvV1) -> TypeEnvV1 {
    let mut merged = base.clone();
    for (name, ty) in locals.iter() {
        merged.insert(name.clone(), ty.clone());
    }
    merged
}

fn lift_source_locals(
    source: &quanta_gqlang_v2_hir::RelationSourceV1,
    schema: &TypeEnvV1,
) -> TypeEnvV1 {
    let mut locals = TypeEnvV1::new();
    if let Some(row) = schema_row_type(schema, &source.name) {
        if let Some(alias) = &source.alias {
            locals.insert(alias.name.clone(), row.clone());
        }
        locals.insert(source.name.clone(), row);
    }
    locals
}

fn schema_row_type(schema: &TypeEnvV1, name: &str) -> Option<TypeV1> {
    match schema.lookup(name) {
        Some(TypeV1::Relation { row, .. }) => Some((**row).clone()),
        _ => None,
    }
}

fn lower_source_rule(rule: &RuleHirV1) -> Result<RecursiveProgramV1, PipelineDiagnosticV1> {
    ensure_supported_rule_head(&rule.head, rule.span).map_err(PipelineDiagnosticV1::Logic)?;
    let mut seen = BTreeSet::new();
    let mut body = Vec::new();
    collect_rule_body_literals(&rule.body, false, &mut seen, &mut body);
    Ok(RecursiveProgramV1::from_rules(vec![RuleDefV1::from_rule(
        rule, body,
    )]))
}

fn ensure_supported_rule_head(
    head: &ValueExprHirV1,
    span: quanta_gqlang_v2_hir::SpanV1,
) -> Result<(), LogicDiagnosticV1> {
    if root_symbol_name(head).is_some() {
        Ok(())
    } else {
        Err(LogicDiagnosticV1::UnsupportedRuleHead { span })
    }
}

fn collect_rule_body_literals(
    node: &BoolExprHirV1,
    negated: bool,
    seen: &mut BTreeSet<(String, bool)>,
    out: &mut Vec<BodyLiteralV1>,
) {
    match &node.kind {
        BoolExprKindV1::Literal(_) => {}
        BoolExprKindV1::Value(value) => collect_rule_value_literal(value, negated, seen, out),
        BoolExprKindV1::Compare(compare) => {
            collect_rule_value_literal(&compare.lhs, negated, seen, out);
            collect_rule_value_literal(&compare.rhs, negated, seen, out);
        }
        BoolExprKindV1::And(bin) | BoolExprKindV1::Or(bin) => {
            collect_rule_body_literals(&bin.lhs, negated, seen, out);
            collect_rule_body_literals(&bin.rhs, negated, seen, out);
        }
        BoolExprKindV1::Not(inner) => collect_rule_body_literals(inner, !negated, seen, out),
        BoolExprKindV1::Paren(inner) => collect_rule_body_literals(inner, negated, seen, out),
        BoolExprKindV1::Quantifier(quant) => {
            collect_rule_body_literals(&quant.body, negated, seen, out);
        }
    }
}

fn collect_rule_value_literal(
    value: &ValueExprHirV1,
    negated: bool,
    seen: &mut BTreeSet<(String, bool)>,
    out: &mut Vec<BodyLiteralV1>,
) {
    if let Some(name) = root_symbol_name(value) {
        let key = (name.clone(), negated);
        if seen.insert(key) {
            out.push(if negated {
                BodyLiteralV1::negated(name)
            } else {
                BodyLiteralV1::positive(name)
            });
        }
    }
    match &value.kind {
        ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) | ValueExprKindV1::Path(_) => {}
        ValueExprKindV1::Unary(unary) => {
            collect_rule_value_literal(&unary.operand, negated, seen, out)
        }
        ValueExprKindV1::Binary(binary) => {
            collect_rule_value_literal(&binary.lhs, negated, seen, out);
            collect_rule_value_literal(&binary.rhs, negated, seen, out);
        }
        ValueExprKindV1::Call(call) => {
            for arg in &call.args {
                collect_rule_value_literal(arg, negated, seen, out);
            }
        }
        ValueExprKindV1::Paren(inner) => collect_rule_value_literal(inner, negated, seen, out),
        ValueExprKindV1::Record(record) => {
            for field in &record.fields {
                collect_rule_value_literal(&field.value, negated, seen, out);
            }
        }
        ValueExprKindV1::List(list) => {
            for element in &list.elements {
                collect_rule_value_literal(element, negated, seen, out);
            }
        }
    }
}

fn root_symbol_name(value: &ValueExprHirV1) -> Option<String> {
    match &value.kind {
        ValueExprKindV1::Symbol(symbol) => Some(symbol.name.clone()),
        ValueExprKindV1::Path(path) => root_symbol_name(&path.base),
        ValueExprKindV1::Paren(inner) => root_symbol_name(inner),
        ValueExprKindV1::Literal(_)
        | ValueExprKindV1::Unary(_)
        | ValueExprKindV1::Binary(_)
        | ValueExprKindV1::Call(_)
        | ValueExprKindV1::Record(_)
        | ValueExprKindV1::List(_) => None,
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "unit rail: assertion panics are intentional"
)]
mod tests {
    use super::{
        bind_item, bind_standalone, producer_output_relation, resolve_local, Binder,
        FrontendContextV1, ModuleGraphV1,
    };
    use quanta_gqlang_v2_hir::HirBuilderV1;
    use quanta_gqlang_v2_surface::parse_program;
    use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

    /// `users(id: int)` as the only frontend schema authority.
    fn users_schema() -> TypeEnvV1 {
        TypeEnvV1::new().with(
            "users",
            TypeV1::Relation {
                row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                    "id",
                    TypeV1::int(),
                )])),
                cardinality: CardinalityV1::Bag,
            },
        )
    }

    /// Parse `src` and return its single program item.
    fn item(src: &str) -> quanta_gqlang_v2_hir::ProgramItemHirV1 {
        let builder = HirBuilderV1::new();
        parse_program(src, &builder)
            .expect("producer parses")
            .item
            .to_program_item()
    }

    /// The lifted cardinality this helper reads off the relation it returns.
    fn lifted_cardinality(relation: &TypeV1) -> CardinalityV1 {
        let TypeV1::Relation { cardinality, .. } = relation else {
            panic!("producer output must lift a Relation, got {relation:?}")
        };
        *cardinality
    }

    /// D26 red rail: the lifted producer relation must carry the producer's ACTUAL
    /// normalized output cardinality, not a hardcoded `Bag`. Before the fix
    /// `producer_output_relation` always stamped `CardinalityV1::Bag` regardless of
    /// the producer's real multiplicity, silently downgrading a `Set` (or
    /// `Sequence`) producer and breaking the D26 guarantee that multiplicity /
    /// duplicate-elimination / output-order semantics are explicit and testable.
    #[test]
    fn lifted_producer_relation_preserves_set_cardinality() {
        let producer = item("from users u | project u.id as id");
        let context = FrontendContextV1::schema_only(users_schema());
        let relation = producer_output_relation(&producer, &context, CardinalityV1::Set)
            .expect("a relational producer with a named projection lifts a relation");
        assert_eq!(
            lifted_cardinality(&relation),
            CardinalityV1::Set,
            "a Set-cardinality producer must lift a Set relation, not a downgraded Bag (D26)"
        );
    }

    /// The companion order-preserving case: a `Sequence` producer must lift a
    /// `Sequence` relation, never a `Bag`.
    #[test]
    fn lifted_producer_relation_preserves_sequence_cardinality() {
        let producer = item("from users u | project u.id as id");
        let context = FrontendContextV1::schema_only(users_schema());
        let relation = producer_output_relation(&producer, &context, CardinalityV1::Sequence)
            .expect("a relational producer with a named projection lifts a relation");
        assert_eq!(
            lifted_cardinality(&relation),
            CardinalityV1::Sequence,
            "a Sequence-cardinality producer must lift a Sequence relation (D26 order semantics)"
        );
    }

    /// Soundness: a genuine `Bag` producer still lifts a `Bag` relation, so the fix
    /// threads the real cardinality rather than inverting the default.
    #[test]
    fn lifted_producer_relation_preserves_bag_cardinality() {
        let producer = item("from users u | project u.id as id");
        let context = FrontendContextV1::schema_only(users_schema());
        let relation = producer_output_relation(&producer, &context, CardinalityV1::Bag)
            .expect("a relational producer with a named projection lifts a relation");
        assert_eq!(lifted_cardinality(&relation), CardinalityV1::Bag);
    }

    /// `bind_item` is fail-closed: a query item carrying an unresolved free symbol
    /// must surface a `bind`-stage reject, NOT silently succeed. Pins that the
    /// helper actually runs the binder and propagates its first diagnostic; a body
    /// replaced with `Ok(())` would swallow the reject.
    #[test]
    fn bind_item_propagates_unresolved_symbol_reject() {
        let query = item("from users u | where unknownvar > 1 | project u.id as id");
        let graph = ModuleGraphV1::new();
        let err = bind_item(&query, &graph)
            .expect_err("an unresolved free symbol in a query body must reject at bind");
        assert_eq!(err.code(), "binder.unresolved_symbol");
    }

    /// `bind_item` on a clean, fully-resolved query item returns `Ok(())`; pairs
    /// with the reject case so the helper is pinned to BOTH outcomes (a body that
    /// always errored would fail here).
    #[test]
    fn bind_item_accepts_a_resolved_query() {
        let query = item("from users u | project u.id as id");
        let graph = ModuleGraphV1::new();
        assert!(
            bind_item(&query, &graph).is_ok(),
            "a fully-resolved query binds cleanly"
        );
    }

    /// `resolve_local` dispatches a query item into the binder and records its
    /// unresolved-symbol diagnostic. A body replaced with `()` would resolve
    /// nothing, leaving the binder diagnostic-free.
    #[test]
    fn resolve_local_records_query_binder_diagnostic() {
        let query = item("from users u | where unknownvar > 1 | project u.id as id");
        let graph = ModuleGraphV1::new();
        let mut binder = Binder::new(&graph);
        resolve_local(&mut binder, &query);
        let result = binder.finish();
        assert!(
            result
                .diagnostics()
                .iter()
                .any(|d| d.code() == "binder.unresolved_symbol"),
            "resolve_local must run the binder, surfacing the unresolved-symbol reject"
        );
    }

    /// `bind_standalone` wraps `resolve_local` in a balanced scope frame and must
    /// still surface the wrapped item's binder diagnostics. A body replaced with
    /// `()` would record nothing.
    #[test]
    fn bind_standalone_surfaces_wrapped_binder_diagnostic() {
        let query = item("from users u | where unknownvar > 1 | project u.id as id");
        let graph = ModuleGraphV1::new();
        let mut binder = Binder::new(&graph);
        bind_standalone(&mut binder, &query);
        let result = binder.finish();
        assert!(
            result
                .diagnostics()
                .iter()
                .any(|d| d.code() == "binder.unresolved_symbol"),
            "bind_standalone must resolve the wrapped item, surfacing its reject"
        );
    }
}
