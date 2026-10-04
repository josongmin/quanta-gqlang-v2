//! QGLang v3 binder / scope / symbol resolution (ticket 02).
//!
//! Owns the deterministic lexical + relation scope model, declaration/import
//! symbol tables, alias/param/let binding, and typed resolution diagnostics over
//! the shared HIR from ticket 01. Cites packet decisions:
//!
//! - `D3`: binding/scope resolution runs before algebra lowering. This crate is
//!   the binder seam between HIR construction and typing (ticket 03).
//! - `D25`: name resolution is lexical and binder-visible only. Capture across a
//!   scope boundary is an EXPLICIT edge ([`CaptureV1`]); there is no hidden
//!   path-scoping. Path access resolves only its base symbol.
//! - `D28`: conflicting imports fail closed deterministically — an ambiguous
//!   import is a hard reject ([`BindDiagnosticV1::AmbiguousImport`]) with no
//!   last-wins / import-order tiebreak.
//! - `D30`: module/import graph semantics (re-export precedence, cycles, import
//!   order) are owned by ticket 11. This crate only CONSUMES the frozen graph
//!   through [`ModuleGraphViewV1`]; it invents no import authority.
//!
//! # Out of lane
//!
//! Overload / type legality is ticket 03; planner concerns are ticket 07;
//! the concrete module-graph builder is ticket 11. This crate consumes those
//! boundaries through [`ModuleGraphViewV1`] and produces a [`BindResultV1`] of
//! resolved references plus fail-closed diagnostics.

mod diagnostic;
mod module_graph;
mod resolve;
mod scope;

pub use diagnostic::BindDiagnosticV1;
pub use module_graph::{ModuleGraphV1, ModuleGraphViewV1};
pub use resolve::{
    AdmittedBinderDeclarationsV3, AdmittedBinderResolutionV3, BindResultV1, Binder,
    BinderDeclarationRequestErrorV3, BinderEvidenceReaderV3, ParamV1, ResolutionV1,
    ResolvedTargetV1,
};
pub use scope::{
    BinderKindV1, BindingV1, CaptureV1, LexicalResolutionV1, LexicalScopeV1, ShadowReportKindV1,
    ShadowReportV1,
};
