#![forbid(unsafe_code)]
//! Dev-only v3 PARITY GUARD — proves the v3 frontend (`quanta-gqlang-v2-pipeline`
//! over `quanta-gqlang-v2-surface`) covers ALL 18 DSL root families end to end,
//! and separately pins the current explicit artifact/no-plan behavior of
//! standalone `let` / horn-`rule` units through the public pipeline seam.
//!
//! This crate replaces the former v2-IR-based root-family registry. It ships no
//! production behavior (`publish = false`); it exists so the single v3 compiler
//! is *proven* to classify every front-door family — the 17 root families plus
//! the clause-level `LET`/`WITH` composition family (a composition of roots, not
//! itself one of the 17 — see `docs/language/README.md`).
//!
//! Each family carries ONE representative v3 surface example string and its
//! expected classification: the compiled-item kind ([`FamilyKindV1`]) and, for
//! plan-producing families, the plan's `result_kind` and `effect`. The
//! `tests/self_validation.rs` hellgate drives every example through
//! [`quanta_gqlang_v2_pipeline::compile_program_structural`] and asserts the outcome, so this
//! crate now PROVES v3 covers all 18 families (a fail-closed test pins that an
//! unsupported construct still rejects).
//!
//! Construction is a `match` over [`RootFamilyV1`] in [`spec_for_v1`], so adding
//! a family variant is a compile error until every field is supplied.

pub mod leakage;

use quanta_gqlang_v2_planning::{EffectClassV1, ResultKindV1};

/// Declare the closed `RootFamilyV1` enum AND the canonical `ROOT_FAMILIES_V1`
/// list from ONE source list, so the enum and the iteration list can never
/// drift: adding a family is a single edit to the invocation below, and
/// `spec_for_v1`'s exhaustive `match` then refuses to compile until its spec is
/// supplied. (Rust has no built-in variant iteration; this macro is the
/// no-dependency way to keep the single enumeration honest.)
macro_rules! declare_root_families_v1 {
    ($($variant:ident),+ $(,)?) => {
        /// The closed DSL front-door families (the 17 root families plus the
        /// clause-level LET/WITH composition family). Closed-world by design:
        /// extension is a new variant plus the compiler enumerating every site
        /// that must answer.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum RootFamilyV1 {
            $($variant),+
        }

        /// Every family, in canonical order — generated from the same list as the
        /// enum, so it always covers exactly the variant set.
        pub const ROOT_FAMILIES_V1: &[RootFamilyV1] = &[$(RootFamilyV1::$variant),+];
    };
}

declare_root_families_v1!(
    Select,
    Find,
    Check,
    Explain,
    Show,
    Describe,
    Sample,
    Compare,
    Export,
    View,
    Rule,
    Model,
    Pack,
    Profile,
    Gate,
    UsePack,
    RequireCertified,
    Composition,
);

/// The v3 compiled-item kind a family lands on — the stable "item.kind" each
/// family is proven to classify into. Mirrors the
/// [`quanta_gqlang_v2_pipeline::CompiledItemV1`] variants without re-exporting the
/// pipeline's payload-carrying enum, so the registry asserts a discriminant the
/// pipeline owns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyKindV1 {
    /// A relational query (the `SELECT`/`from..project` pipe).
    RelationalQuery,
    /// A standalone `let` value artifact with no serving plan.
    ValueArtifact,
    /// A graph query (`FIND`).
    GraphQuery,
    /// A legality check (`CHECK`).
    Legality,
    /// A presentation verb (`EXPLAIN` / `SHOW` / `DESCRIBE` / `SAMPLE` /
    /// `COMPARE` / `EXPORT`).
    Presentation,
    /// A standalone horn-rule logic artifact with no serving plan.
    RuleArtifact,
    /// A catalog declaration (`VIEW` / `MODEL` / `PACK` / `PROFILE` / `GATE` /
    /// `RULE`), registered with no plan.
    CatalogRegistration,
    /// A composition (`LET..WITH`).
    Composition,
    /// A meta directive (`USE PACK` / `REQUIRE CERTIFIED`).
    Meta,
}

impl FamilyKindV1 {
    /// Stable keyword for diagnostics and drift messages.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::RelationalQuery => "relational_query",
            Self::ValueArtifact => "value_artifact",
            Self::GraphQuery => "graph_query",
            Self::Legality => "legality",
            Self::Presentation => "presentation",
            Self::RuleArtifact => "rule_artifact",
            Self::CatalogRegistration => "catalog_registration",
            Self::Composition => "composition",
            Self::Meta => "meta",
        }
    }
}

/// The plan classification a plan-producing family is proven to land on: the
/// `result_kind` and `effect` the v3 pipeline's public plan must carry. Families
/// that produce NO plan (catalog declarations, `USE PACK`) carry `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanOutcomeV1 {
    /// The plan's result kind.
    pub result_kind: ResultKindV1,
    /// The plan's effect class.
    pub effect: EffectClassV1,
}

const fn plan_outcome_v1(
    result_kind: ResultKindV1,
    effect: EffectClassV1,
) -> Option<PlanOutcomeV1> {
    Some(PlanOutcomeV1 {
        result_kind,
        effect,
    })
}

/// What the v3 pipeline is proven to do with a family's representative example.
///
/// Most families compile to a [`FamilyKindV1`]. `USE PACK` is the one family
/// whose validity is inherently SCHEMA-bound: its pack manifest lives in a graph
/// the public [`quanta_gqlang_v2_pipeline::compile_program_structural`] entry point does not
/// (and must not) carry, so the honest provable outcome through the public
/// entry point is a fail-closed `modules` reject. Recognizing-and-routing the
/// family to its modules stage IS coverage; this variant pins the stable reject
/// code so the family stays exercised end to end.
#[derive(Clone, Copy, Debug)]
pub enum FamilyOutcomeV1 {
    /// The example compiles to this compiled-item kind, with the given plan
    /// outcome (for plan-producing families) or `None` (no-plan families).
    Compiles {
        /// The compiled-item kind.
        kind: FamilyKindV1,
        /// The plan outcome, or `None` for catalog registrations.
        plan: Option<PlanOutcomeV1>,
    },
    /// The example is recognized and routed to its stage, then fails closed with
    /// this stable diagnostic code (the schema-bound `USE PACK` family).
    FailsClosed {
        /// The stable diagnostic code the reject carries.
        code: &'static str,
    },
}

/// The full v3 parity spec for one root family: the canonical surface example,
/// the leading keyword, and the outcome the v3 pipeline is proven to produce.
#[derive(Clone, Copy, Debug)]
pub struct RootFamilySpecV1 {
    /// The family this spec describes.
    pub family: RootFamilyV1,
    /// The leading v3 surface keyword (the first token of the example).
    pub keyword: &'static str,
    /// A representative v3 surface example string.
    pub example: &'static str,
    /// The outcome the v3 pipeline is proven to produce for the example.
    pub outcome: FamilyOutcomeV1,
}

/// Standalone surface items that the parser accepts but the root-family registry
/// deliberately does not count as one of the 18 servable/root families.
///
/// Today this is exactly the value-only `let` declaration unit and the horn
/// `rule` unit. They are parsed into shared HIR, routed into the pipeline, and
/// now compile into explicit non-servable artifacts rather than failing closed
/// or pretending to be relational plans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupplementalSurfaceUnitV1 {
    /// A standalone `let <name> = <value>` declaration unit.
    Decl,
    /// A standalone horn-rule `rule <name> : <head> if <body>` unit.
    HornRule,
}

/// The canonical set of supplemental units pinned separately from the 18 root
/// families.
pub const SUPPLEMENTAL_SURFACE_UNITS_V1: &[SupplementalSurfaceUnitV1] = &[
    SupplementalSurfaceUnitV1::Decl,
    SupplementalSurfaceUnitV1::HornRule,
];

/// The current public-pipeline outcome for one supplemental unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupplementalOutcomeV1 {
    /// The compiled-item discriminant the unit lands on.
    pub kind: FamilyKindV1,
    /// Whether the compiled unit carries a serving plan.
    pub has_plan: bool,
}

/// One supplemental-unit spec: keyword, representative example, and the exact
/// current public-pipeline outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupplementalSurfaceUnitSpecV1 {
    /// The standalone parsed unit.
    pub unit: SupplementalSurfaceUnitV1,
    /// The leading keyword of the example.
    pub keyword: &'static str,
    /// A representative surface example accepted by the parser.
    pub example: &'static str,
    /// The current public-pipeline outcome this unit produces.
    pub outcome: SupplementalOutcomeV1,
}

const fn compiles_v1(kind: FamilyKindV1, plan: Option<PlanOutcomeV1>) -> FamilyOutcomeV1 {
    FamilyOutcomeV1::Compiles { kind, plan }
}

const fn supplemental_compiles_v1(kind: FamilyKindV1, has_plan: bool) -> SupplementalOutcomeV1 {
    SupplementalOutcomeV1 { kind, has_plan }
}

/// The single per-family authority. Exhaustive `match`: a new [`RootFamilyV1`]
/// variant cannot compile until its spec is supplied here.
#[must_use]
pub const fn spec_for_v1(family: RootFamilyV1) -> RootFamilySpecV1 {
    use EffectClassV1 as Effect;
    use FamilyKindV1 as Kind;
    use ResultKindV1 as Result;

    match family {
        RootFamilyV1::Select => RootFamilySpecV1 {
            family,
            keyword: "from",
            example: "from users u | where u.id > 1 | project u.id as id, u.name as name",
            outcome: compiles_v1(
                Kind::RelationalQuery,
                plan_outcome_v1(Result::Table, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Find => RootFamilySpecV1 {
            family,
            keyword: "find",
            example: "find taint api.entry -> db.write | sanitizers=authz.clean | depth=3",
            outcome: compiles_v1(
                Kind::GraphQuery,
                plan_outcome_v1(Result::PathWitness, Effect::ProofEvidenceOnly),
            ),
        },
        RootFamilyV1::Check => RootFamilySpecV1 {
            family,
            keyword: "check",
            example: "check rule.hybrid | scope=repo | confirm=require | emit=sarif | mode=hybrid",
            outcome: compiles_v1(
                Kind::Legality,
                plan_outcome_v1(Result::Alert, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Explain => RootFamilySpecV1 {
            family,
            keyword: "explain",
            example: "explain rule.sqli | evidence=trace | truth=confirm",
            outcome: compiles_v1(
                Kind::Presentation,
                plan_outcome_v1(Result::ProofExplain, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Show => RootFamilySpecV1 {
            family,
            keyword: "show",
            example: "show capabilities",
            outcome: compiles_v1(
                Kind::Presentation,
                plan_outcome_v1(Result::Table, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Describe => RootFamilySpecV1 {
            family,
            keyword: "describe",
            example: "describe rule::owasp.sqli",
            outcome: compiles_v1(
                Kind::Presentation,
                plan_outcome_v1(Result::Alert, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Sample => RootFamilySpecV1 {
            family,
            keyword: "sample",
            example: "sample query=\"callgraph\" | mode=semantic | limit=5 | scope=repo",
            outcome: compiles_v1(
                Kind::Presentation,
                plan_outcome_v1(Result::Table, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Compare => RootFamilySpecV1 {
            family,
            keyword: "compare",
            example: "compare baseline=run.a | candidate=run.b | metric=semantic",
            outcome: compiles_v1(
                Kind::Presentation,
                plan_outcome_v1(Result::PathWitness, Effect::ReadOnly),
            ),
        },
        RootFamilyV1::Export => RootFamilySpecV1 {
            family,
            keyword: "export",
            example: "export source=result.latest | kind=sarif | profile=strict",
            outcome: compiles_v1(
                Kind::Presentation,
                plan_outcome_v1(Result::Table, Effect::ExportOnly),
            ),
        },
        RootFamilyV1::View => RootFamilySpecV1 {
            family,
            keyword: "view",
            example: "view catalog.users | query-ref=queries.catalog.users",
            outcome: compiles_v1(Kind::CatalogRegistration, None),
        },
        // The catalog RULE binding family uses the `ruledef` keyword; bare `rule`
        // keeps its horn-rule meaning in the surface grammar.
        RootFamilyV1::Rule => RootFamilySpecV1 {
            family,
            keyword: "ruledef",
            example: "ruledef shell.exec | check-ref=checks.shell-exec",
            outcome: compiles_v1(Kind::CatalogRegistration, None),
        },
        RootFamilyV1::Model => RootFamilySpecV1 {
            family,
            keyword: "model",
            example: "model embeddings.code | provider=openai | source=models.code",
            outcome: compiles_v1(Kind::CatalogRegistration, None),
        },
        RootFamilyV1::Pack => RootFamilySpecV1 {
            family,
            keyword: "pack",
            example: "pack secure-pack | source=registry.security | activation=workspace",
            outcome: compiles_v1(Kind::CatalogRegistration, None),
        },
        RootFamilyV1::Profile => RootFamilySpecV1 {
            family,
            keyword: "profile",
            example:
                "profile strict | capabilities=graph.read,graph.trace | admission=certified-only",
            outcome: compiles_v1(Kind::CatalogRegistration, None),
        },
        RootFamilyV1::Gate => RootFamilySpecV1 {
            family,
            keyword: "gate",
            example: "gate release | checks=owasp.sql,authz.path | binding=all",
            outcome: compiles_v1(Kind::CatalogRegistration, None),
        },
        // USE PACK is schema-bound: its pack manifest lives in a graph the public
        // entry point does not carry, so the provable v3 outcome through
        // `compile_program_structural` is a fail-closed `modules` reject. The family is still
        // recognized and routed to its modules stage — that is the coverage.
        RootFamilyV1::UsePack => RootFamilySpecV1 {
            family,
            keyword: "use",
            example: "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock",
            outcome: FamilyOutcomeV1::FailsClosed {
                code: "modules.unknown_pack_manifest",
            },
        },
        RootFamilyV1::RequireCertified => RootFamilySpecV1 {
            family,
            keyword: "require",
            example: "require certified",
            outcome: compiles_v1(
                Kind::Meta,
                plan_outcome_v1(Result::ProofExplain, Effect::ProofEvidenceOnly),
            ),
        },
        // `LET <name> = <producer> <body> WITH <binding>` (the clause-level
        // composition family). Its plan is the BODY's plan.
        RootFamilyV1::Composition => RootFamilySpecV1 {
            family,
            keyword: "let",
            example: "let suspects = find taint a.b -> c.d | depth=2 \
                      find taint api.entry -> db.write | depth=3 with suspects",
            // The composition body here is a bounded FIND -> path_witness.
            outcome: compiles_v1(
                Kind::Composition,
                plan_outcome_v1(Result::PathWitness, Effect::ProofEvidenceOnly),
            ),
        },
    }
}

/// The full v3 parity registry, one spec per family in [`ROOT_FAMILIES_V1`] order.
#[must_use]
pub fn root_family_specs_v1() -> Vec<RootFamilySpecV1> {
    ROOT_FAMILIES_V1.iter().copied().map(spec_for_v1).collect()
}

/// The current standalone parsed units that are OUTSIDE the 18-family servable
/// registry, in canonical order.
#[must_use]
pub fn supplemental_surface_unit_specs_v1() -> Vec<SupplementalSurfaceUnitSpecV1> {
    SUPPLEMENTAL_SURFACE_UNITS_V1
        .iter()
        .copied()
        .map(supplemental_surface_unit_spec_v1)
        .collect()
}

/// The single authority for standalone parsed units that are not yet part of
/// the 18-family servable registry.
#[must_use]
pub const fn supplemental_surface_unit_spec_v1(
    unit: SupplementalSurfaceUnitV1,
) -> SupplementalSurfaceUnitSpecV1 {
    match unit {
        SupplementalSurfaceUnitV1::Decl => SupplementalSurfaceUnitSpecV1 {
            unit,
            keyword: "let",
            example: "let x = 1 + 2",
            outcome: supplemental_compiles_v1(FamilyKindV1::ValueArtifact, false),
        },
        SupplementalSurfaceUnitV1::HornRule => SupplementalSurfaceUnitSpecV1 {
            unit,
            keyword: "rule",
            example: "rule adult : person.id if person.age >= 18",
            outcome: supplemental_compiles_v1(FamilyKindV1::RuleArtifact, false),
        },
    }
}

/// A canonical construct the v3 frontend MUST reject fail-closed: a malformed
/// surface source the pipeline cannot classify into any family. Used by the
/// fail-closed parity test so coverage of the 18 families is paired with a proof
/// that an unsupported construct does not silently slip through.
pub const UNSUPPORTED_CONSTRUCT_V1: &str = "totally not a qglang root";
