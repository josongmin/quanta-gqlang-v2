//! QGLang v3 module system, packs, and extension models (ticket 11).
//!
//! Owns the reuse/extension layer above the closed language core: the
//! module/import graph the binder (ticket 02) consumes, parameterized
//! signatures, pack manifests + lockfiles, visibility/export rules, query
//! metadata, and declarative extension/model packs. Cites packet decisions:
//!
//! - `D19`: reusable module/import/signature boundaries and packable query
//!   libraries are first-class architecture, modeled here as concrete types
//!   rather than tooling sugar.
//! - `D28`: extension/model pack conflicts and overlapping imports fail closed
//!   with deterministic diagnostics; no last-wins. See
//!   [`ModuleDiagnosticV1::AmbiguousImport`],
//!   [`ModuleDiagnosticV1::OverlappingPackExport`], and
//!   [`ModuleDiagnosticV1::ModelConflict`].
//! - `D30`: this crate OWNS module/import graph semantics (shadowing, re-export
//!   precedence, ambiguity, cycle policy) and exposes them as a frozen
//!   [`ResolvedGraphV1`]. There is no import-order authority; the binder only
//!   consumes the frozen result.
//!
//! # Out of lane
//!
//! Binder name resolution itself is ticket 02; overload / closed-type legality
//! is ticket 03; runtime/provider wiring stays downstream (`D7`). This crate
//! produces frozen graphs, validated packs, and typed diagnostics only.
//!
//! # Binder integration seam
//!
//! [`ResolvedGraphV1::binder_view`] projects a frozen module into the flat
//! `short name -> distinct targets` shape the binder's `ModuleGraphViewV1`
//! consumes. Because resolution has already collapsed each name to one target
//! (or rejected on ambiguity), every projected set has at most one element, so
//! the binder never re-derives precedence (`D30`).

mod catalog;
mod diagnostic;
mod extension;
mod graph;
mod lockfile;
mod manifest;
mod metadata;
mod signature;
mod version;
mod visibility;

pub use catalog::{CatalogAssetV1, CatalogKindV1, CatalogV1, ResolvedCatalogV1};
pub use diagnostic::ModuleDiagnosticV1;
pub use extension::{compose_models, ComposedModelV1, ModelFactV1, ModelPackV1, ModelSchemaV1};
pub use graph::{ModuleGraphBuilderV1, ResolvedGraphV1};
pub use lockfile::LockfileV1;
pub use manifest::{DependencyReqV1, PackGraphV1, PackIdV1, PackManifestV1, SuiteV1};
pub use metadata::{HelpDocV1, QueryKindV1, QueryMetadataV1, SeverityClassV1};
pub use signature::{SignatureInstanceV1, SignatureParamV1, SignatureV1};
pub use version::{VersionRangeV1, VersionV1};
pub use visibility::VisibilityV1;
