//! Checked module, catalog, pack, lock, signature, and model authority.
//!
//! Public ingress is declarative and source-checked. Completed authority and
//! derived IDs are opaque; one session-owning builder validates and seals them.

mod authority;
mod budget;
mod catalog;
mod diagnostic;
mod extension;
mod graph;
mod lockfile;
mod manifest;
mod signature;
mod version;
mod visibility;

pub use authority::{
    DocumentCatalogValidatorV2, ModuleAuthorityBuilderV2, ValidatedModuleAuthorityPartsV2,
};
pub use budget::ModuleBudgetV2;
pub use catalog::{
    DocumentCatalogAssetInputV2, DocumentCatalogAuthorityV2, DocumentCatalogDependencyInputV2,
    DocumentCatalogInputV2, ExternalCatalogAssetInputV2, ExternalCatalogAssetV2,
    ExternalCatalogDependencyInputV2, ExternalCatalogPayloadInputV2, ExternalCatalogPayloadV2,
    ExternalCatalogV2, ExternalPackContractV2, GateBindingV2, GateContractV2,
    ProfileAdmissionInputV2, ProfileAdmissionV2, RequireCertifiedInputV2,
};
pub use diagnostic::{ModuleDiagnosticV2, ResourceExhaustionV2};
pub use extension::{
    ComposedModelFactV2, ComposedModelV2, LocatedModelValueInputV2, ModelAttributeInputV2,
    ModelFactInputV2, ModelPackInputV2, ModelSchemaInputV2, ModelSchemaV2,
};
pub use graph::{
    AmbiguousCandidateV2, CatalogAssetIdV2, CatalogAssetRefInputV2, CurrentModuleSelectionInputV2,
    LocatedEdgeV2, ModuleDeclarationInputV2, ModuleGraphBuilderV2, ModuleIdV2, ModuleImportInputV2,
    ModuleNodeInputV2, ModuleReexportInputV2, NamespaceKindV2, ResolvedBindingV2, ResolvedGraphV2,
    ResolvedModuleBindingsV2,
};
pub use lockfile::{
    CatalogPackActivationV2, LockPinV2, LockReferenceInputV2, LockReferenceV2, LockfileV2,
    PackActivationInputV2, UsePackActivationV2, ValidatedPackActivationV2,
    ValidatedPackActivationsV2, ValidatedPackAuthorityV2, VerifiedLockSetV2,
};
pub use manifest::{
    ContentDigestV2, LocatedDependencyInputV2, LocatedExportInputV2, LocatedSuiteInputV2,
    LocatedSuiteMemberInputV2, PackCoordinateV2, PackGraphV2, PackIdV2, PackManifestInputV2,
    PackManifestV2, ValidatedPackGraphV2, ValidatedPackManifestV2,
};
pub use signature::{
    CallableKindV2, LocatedParameterInputV2, SignatureInputV2, SignatureParameterV2, SignatureV2,
};
pub use version::{VersionRangeV2, VersionV2};
pub use visibility::VisibilityV2;
