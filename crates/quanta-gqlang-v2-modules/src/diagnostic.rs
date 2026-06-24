//! Typed module-system diagnostics with stable machine-readable codes.
//!
//! Every failure path in this crate produces a typed [`ModuleDiagnosticV1`]
//! value carrying a stable [`ModuleDiagnosticV1::code`] string and a human
//! `Display` rendering. There is no untyped error path. Cites packet decisions:
//!
//! - `D28`: extension/model pack conflicts and overlapping imports fail closed
//!   with deterministic diagnostics; no last-wins or import-order-sensitive
//!   override. [`ModuleDiagnosticV1::AmbiguousImport`],
//!   [`ModuleDiagnosticV1::OverlappingPackExport`], and
//!   [`ModuleDiagnosticV1::ModelConflict`] are the hard rejects.
//! - `D30`: module/import graph semantics are owned here. The cycle policy is an
//!   EXPLICIT reject ([`ModuleDiagnosticV1::ImportCycle`]) — never a silent
//!   accept — and resolution carries no import-order authority.

use std::fmt;

use crate::version::{VersionRangeV1, VersionV1};

/// A typed module-system diagnostic.
///
/// Each variant is a fail-closed reject the resolver, manifest validator, or
/// extension validator emits. Construction is the only failure channel; callers
/// match on the variant or read [`ModuleDiagnosticV1::code`] for a stable id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleDiagnosticV1 {
    /// An imported short name resolves to more than one distinct fully-qualified
    /// target after shadowing and re-export precedence have been applied. Per
    /// `D28` this is a hard reject with no import-order tiebreak.
    AmbiguousImport {
        /// The ambiguous imported short name.
        name: String,
        /// The distinct fully-qualified targets that collided, in deterministic
        /// sorted order.
        candidates: Vec<String>,
    },
    /// A module/import dependency cycle was detected. Per `D30` cycles reject
    /// explicitly; there is no silent accept and no future cycle semantics are
    /// assumed here.
    ImportCycle {
        /// The module names on the cycle, rotated to start at the lexically
        /// smallest member for determinism, then the entry repeated to close it.
        cycle: Vec<String>,
    },
    /// An import refers to a module that is not present in the graph.
    UnknownModule {
        /// The importing module.
        importer: String,
        /// The missing target module name.
        missing: String,
    },
    /// An import targets a symbol the source module does not export (it is
    /// private or absent). Visibility/export rules reject the import.
    NotExported {
        /// The module that was imported from.
        module: String,
        /// The symbol that is not visible for import.
        symbol: String,
    },
    /// Two packs in the resolved graph export the same public name. Per `D28`
    /// overlapping packs fail closed rather than last-wins.
    OverlappingPackExport {
        /// The exported symbol name that overlaps.
        symbol: String,
        /// The pack identities that both export it, in deterministic order.
        packs: Vec<String>,
    },
    /// Two extension/model packs assert an incompatible fact for the same
    /// modeled target. Per `D28` this composes only by an explicit rule or
    /// rejects; silent last-wins is forbidden.
    ModelConflict {
        /// The modeled target key (e.g. fully-qualified symbol) under conflict.
        target: String,
        /// The attribute whose value disagrees.
        attribute: String,
        /// The conflicting values, in deterministic order.
        values: Vec<String>,
        /// The packs that supplied the conflicting values, aligned with
        /// `values` by index.
        packs: Vec<String>,
    },
    /// A pack's required language/feature compatibility range excludes the host
    /// language version. A version/compat mismatch rejects with this code.
    CompatMismatch {
        /// The pack whose compatibility window was violated.
        pack: String,
        /// The required language compatibility range.
        required: VersionRangeV1,
        /// The actual host language version.
        actual: VersionV1,
    },
    /// A dependency declared by a pack cannot be satisfied by the locked /
    /// available version of that dependency.
    DependencyUnsatisfied {
        /// The depending pack.
        pack: String,
        /// The dependency name.
        dependency: String,
        /// The required version range.
        required: VersionRangeV1,
        /// The version that was offered (locked or available).
        offered: VersionV1,
    },
    /// The lockfile disagrees with the manifest graph: a pack/version pair in
    /// the lock is absent from, or contradicts, the manifest set. Reproducible
    /// offline resolution requires the lock to match exactly.
    LockMismatch {
        /// The pack whose lock entry is stale or contradictory.
        pack: String,
        /// Human description of the mismatch (locked vs manifest).
        detail: String,
    },
    /// A declarative model-pack failed schema validation.
    InvalidModelData {
        /// The pack that contains the invalid model data.
        pack: String,
        /// Human description of the validation failure.
        detail: String,
    },
    /// A parameterized signature was instantiated with an argument count or
    /// labeling that does not match its declared parameters.
    SignatureArityMismatch {
        /// The signature name.
        signature: String,
        /// The number of declared parameters.
        expected: usize,
        /// The number of supplied arguments.
        found: usize,
    },
    /// Two catalog declarations of the same kind register the same name; the
    /// `<kind>::<name>` asset id collides. Per the v2 declaration semantics
    /// duplicate names fail closed rather than last-wins.
    DuplicateCatalogName {
        /// The colliding asset kind tag (`view`/`model`/`pack`/...).
        kind: &'static str,
        /// The duplicated declaration name.
        name: String,
    },
    /// A VIEW's inline SELECT body transitively references itself through other
    /// inline VIEWs. The inline-select reference cycle rejects explicitly.
    ViewInlineCycle {
        /// The VIEW names on the cycle, in DFS order, with the entry repeated to
        /// close the cycle.
        cycle: Vec<String>,
    },
    /// `REQUIRE CERTIFIED` is in force but a PROFILE declares a non-certified
    /// admission policy. Certified gating forces every profile to
    /// `certified-only`; any other admission rejects.
    CertifiedRequiredViolation {
        /// The offending profile name.
        profile: String,
        /// The non-certified admission tag the profile declared.
        admission: &'static str,
    },
    /// Two manifests added to the same [`crate::PackGraphV1`] declare the same
    /// pack name. Per `D28` pack conflicts fail closed with a deterministic
    /// diagnostic — never silent last-wins / overwrite — so the graph rejects
    /// before mutating its state.
    DuplicatePackName {
        /// The colliding pack name.
        name: String,
        /// The version already registered under that name.
        existing: VersionV1,
        /// The version of the rejected duplicate.
        incoming: VersionV1,
    },
    /// `USE PACK <id>` names a pack with no resolvable manifest in the host
    /// pack graph. Pack activation requires a known manifest.
    UnknownPackManifest {
        /// The pack id that could not be resolved to a manifest.
        pack: String,
    },
    /// `USE PACK ... lockfile(<path>)` supplied a lockfile path that is not a
    /// `*.lock` file. Lockfile paths must end in `.lock`.
    InvalidLockfilePath {
        /// The pack whose activation supplied the path.
        pack: String,
        /// The invalid lockfile path as written.
        path: String,
    },
}

impl ModuleDiagnosticV1 {
    /// Stable machine-readable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::AmbiguousImport { .. } => "modules.ambiguous_import",
            Self::ImportCycle { .. } => "modules.import_cycle",
            Self::UnknownModule { .. } => "modules.unknown_module",
            Self::NotExported { .. } => "modules.not_exported",
            Self::OverlappingPackExport { .. } => "modules.overlapping_pack_export",
            Self::ModelConflict { .. } => "modules.model_conflict",
            Self::CompatMismatch { .. } => "modules.compat_mismatch",
            Self::DependencyUnsatisfied { .. } => "modules.dependency_unsatisfied",
            Self::LockMismatch { .. } => "modules.lock_mismatch",
            Self::InvalidModelData { .. } => "modules.invalid_model_data",
            Self::SignatureArityMismatch { .. } => "modules.signature_arity_mismatch",
            Self::DuplicateCatalogName { .. } => "modules.duplicate_catalog_name",
            Self::DuplicatePackName { .. } => "modules.duplicate_pack_name",
            Self::ViewInlineCycle { .. } => "modules.view_inline_cycle",
            Self::CertifiedRequiredViolation { .. } => "modules.certified_required_violation",
            Self::UnknownPackManifest { .. } => "modules.unknown_pack_manifest",
            Self::InvalidLockfilePath { .. } => "modules.invalid_lockfile_path",
        }
    }
}

impl fmt::Display for ModuleDiagnosticV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AmbiguousImport { name, candidates } => write!(
                f,
                "{}: imported symbol `{}` is ambiguous between {}",
                self.code(),
                name,
                candidates.join(", "),
            ),
            Self::ImportCycle { cycle } => write!(
                f,
                "{}: module import cycle {}",
                self.code(),
                cycle.join(" -> "),
            ),
            Self::UnknownModule { importer, missing } => write!(
                f,
                "{}: module `{}` imports unknown module `{}`",
                self.code(),
                importer,
                missing,
            ),
            Self::NotExported { module, symbol } => write!(
                f,
                "{}: module `{}` does not export `{}`",
                self.code(),
                module,
                symbol,
            ),
            Self::OverlappingPackExport { symbol, packs } => write!(
                f,
                "{}: symbol `{}` is exported by overlapping packs {}",
                self.code(),
                symbol,
                packs.join(", "),
            ),
            Self::ModelConflict {
                target,
                attribute,
                values,
                packs,
            } => write!(
                f,
                "{}: model target `{}` attribute `{}` conflicts ({}) from packs {}",
                self.code(),
                target,
                attribute,
                values.join(", "),
                packs.join(", "),
            ),
            Self::CompatMismatch {
                pack,
                required,
                actual,
            } => write!(
                f,
                "{}: pack `{}` requires language {} but host is {}",
                self.code(),
                pack,
                required,
                actual,
            ),
            Self::DependencyUnsatisfied {
                pack,
                dependency,
                required,
                offered,
            } => write!(
                f,
                "{}: pack `{}` requires `{}` {} but {} is offered",
                self.code(),
                pack,
                dependency,
                required,
                offered,
            ),
            Self::LockMismatch { pack, detail } => write!(
                f,
                "{}: lock entry for pack `{}` mismatches manifest: {}",
                self.code(),
                pack,
                detail,
            ),
            Self::InvalidModelData { pack, detail } => write!(
                f,
                "{}: pack `{}` has invalid model data: {}",
                self.code(),
                pack,
                detail,
            ),
            Self::SignatureArityMismatch {
                signature,
                expected,
                found,
            } => write!(
                f,
                "{}: signature `{}` expects {} parameter(s) but got {}",
                self.code(),
                signature,
                expected,
                found,
            ),
            Self::DuplicateCatalogName { kind, name } => write!(
                f,
                "{}: catalog {} `{}` is already declared",
                self.code(),
                kind,
                name,
            ),
            Self::DuplicatePackName {
                name,
                existing,
                incoming,
            } => write!(
                f,
                "{}: pack `{}` is declared twice (existing {} vs incoming {})",
                self.code(),
                name,
                existing,
                incoming,
            ),
            Self::ViewInlineCycle { cycle } => write!(
                f,
                "{}: view inline-select cycle {}",
                self.code(),
                cycle.join(" -> "),
            ),
            Self::CertifiedRequiredViolation { profile, admission } => write!(
                f,
                "{}: REQUIRE CERTIFIED forces certified-only but profile `{}` is {}",
                self.code(),
                profile,
                admission,
            ),
            Self::UnknownPackManifest { pack } => write!(
                f,
                "{}: USE PACK `{}` has no resolvable manifest",
                self.code(),
                pack,
            ),
            Self::InvalidLockfilePath { pack, path } => write!(
                f,
                "{}: USE PACK `{}` lockfile `{}` is not a *.lock file",
                self.code(),
                pack,
                path,
            ),
        }
    }
}

impl std::error::Error for ModuleDiagnosticV1 {}
