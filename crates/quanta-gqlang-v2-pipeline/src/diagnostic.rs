//! The unified, stage-tagged, fail-closed pipeline diagnostic (ticket 08, D22).
//!
//! [`PipelineDiagnosticV1`] is the ONE error type the shared compiler service
//! returns. It wraps each stage's already-typed diagnostic by value, so the
//! original stable code and exact span are NEVER lost in a rewrap: a caller can
//! match the stage tag for routing and still recover the inner crate's typed
//! diagnostic verbatim. This is the fail-closed contract behind D22 (CLI / LSP /
//! formatter / tests all consume the same diagnostics) and D10 (no silent
//! downgrade — every stage failure is an explicit typed reject).
//!
//! The pipeline short-circuits on the FIRST failing stage, so exactly one inner
//! diagnostic is carried.

use std::fmt;

use quanta_gqlang_v2_algebra::AlgebraDiagnosticV1;
use quanta_gqlang_v2_binder::BindDiagnosticV1;
use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_logic::LogicDiagnosticV1;
use quanta_gqlang_v2_modules::ModuleDiagnosticV1;
use quanta_gqlang_v2_planning::PlanningDiagnosticV1;
use quanta_gqlang_v2_surface::ParseDiagnosticV1;
use quanta_gqlang_v2_types::TypeErrorV1;

/// The compiler stage a [`PipelineDiagnosticV1`] originates from.
///
/// Ordered earliest-to-latest in pipeline execution order, so a caller can
/// reason about which stage short-circuited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PipelineStageV1 {
    /// Surface parsing (`surface::parse_program`).
    Parse,
    /// Binder / scope resolution.
    Bind,
    /// Type checking.
    Type,
    /// Algebra normalization.
    Algebra,
    /// Logic legality (recursion / witness).
    Logic,
    /// Catalog / module registration (declarations + meta directives).
    Modules,
    /// Planning / truth / evidence / effect.
    Planning,
}

impl PipelineStageV1 {
    /// The stable stage tag string, shared by CLI / LSP / tests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Bind => "bind",
            Self::Type => "type",
            Self::Algebra => "algebra",
            Self::Logic => "logic",
            Self::Modules => "modules",
            Self::Planning => "planning",
        }
    }
}

impl fmt::Display for PipelineStageV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A unified, stage-tagged compiler diagnostic.
///
/// Each variant wraps the owning stage's typed diagnostic verbatim. The inner
/// diagnostic's stable code and span are preserved; this wrapper only adds the
/// stage routing tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineDiagnosticV1 {
    /// A surface parse/lex failure.
    Parse(ParseDiagnosticV1),
    /// A binder / scope resolution failure.
    Bind(BindDiagnosticV1),
    /// A type-checking failure.
    Type(TypeErrorV1),
    /// An algebra normalization failure.
    Algebra(AlgebraDiagnosticV1),
    /// A logic-legality failure (recursion / witness).
    Logic(LogicDiagnosticV1),
    /// A catalog / module registration failure (duplicate asset, VIEW inline
    /// cycle, certified-required violation, unknown pack manifest, invalid
    /// lockfile path). [`ModuleDiagnosticV1`] carries no source span, so the
    /// pipeline pairs it with the span of the declaration / meta directive that
    /// triggered it, preserving the inner stable code verbatim.
    ///
    /// The inner diagnostic is boxed so this large-payload variant does not bloat
    /// the whole enum (and every `Result` that returns it).
    Modules(Box<ModuleDiagnosticV1>, SpanV1),
    /// A planning failure (unsupported construct mix).
    Planning(PlanningDiagnosticV1),
}

impl PipelineDiagnosticV1 {
    /// The stage this diagnostic originates from.
    #[must_use]
    pub const fn stage(&self) -> PipelineStageV1 {
        match self {
            Self::Parse(_) => PipelineStageV1::Parse,
            Self::Bind(_) => PipelineStageV1::Bind,
            Self::Type(_) => PipelineStageV1::Type,
            Self::Algebra(_) => PipelineStageV1::Algebra,
            Self::Logic(_) => PipelineStageV1::Logic,
            Self::Modules(..) => PipelineStageV1::Modules,
            Self::Planning(_) => PipelineStageV1::Planning,
        }
    }

    /// The inner stage diagnostic's stable, machine-readable code.
    ///
    /// This is the SAME code the owning crate emits; the wrapper never rewrites
    /// it. CLI / LSP / tests match on this without re-reading stage internals.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::Parse(inner) => inner.code_str().to_owned(),
            Self::Bind(inner) => inner.code().to_owned(),
            Self::Type(inner) => inner.code().to_owned(),
            Self::Algebra(inner) => inner.code(),
            Self::Logic(inner) => inner.code().to_owned(),
            Self::Modules(inner, _) => inner.code().to_owned(),
            Self::Planning(inner) => inner.code(),
        }
    }

    /// The primary source span the inner diagnostic points at.
    ///
    /// Preserved from the inner typed diagnostic; never synthesized here.
    #[must_use]
    pub fn span(&self) -> SpanV1 {
        match self {
            Self::Parse(inner) => inner.span,
            Self::Bind(inner) => inner.primary_span(),
            Self::Type(inner) => inner.span(),
            Self::Algebra(inner) => inner.span(),
            Self::Logic(inner) => inner.span(),
            Self::Modules(_, span) => *span,
            Self::Planning(inner) => inner.span(),
        }
    }
}

impl fmt::Display for PipelineDiagnosticV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.stage())?;
        match self {
            Self::Parse(inner) => write!(f, "{inner}"),
            Self::Bind(inner) => write!(f, "{inner}"),
            Self::Type(inner) => write!(f, "{inner}"),
            Self::Algebra(inner) => write!(f, "{}: {}", inner.code(), inner.message()),
            Self::Logic(inner) => write!(f, "{}: {}", inner.code(), inner.message()),
            Self::Modules(inner, _) => write!(f, "{inner}"),
            Self::Planning(inner) => write!(f, "{}: {}", inner.code(), inner.message()),
        }
    }
}

impl std::error::Error for PipelineDiagnosticV1 {}
