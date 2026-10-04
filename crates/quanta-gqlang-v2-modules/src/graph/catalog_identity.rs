//! Catalog identity derivation and source-authored reference inputs.

use super::{excerpt, is_catalog_local_name, ModuleIdV2, NamespaceKindV2};
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::CheckedLocatedSpanV2;

/// Derived identity of one catalog declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatalogAssetIdV2 {
    pub(super) module_id: ModuleIdV2,
    pub(super) namespace_kind: NamespaceKindV2,
    pub(super) local_name: String,
}

impl CatalogAssetIdV2 {
    pub(crate) fn derive(
        module_id: ModuleIdV2,
        namespace_kind: NamespaceKindV2,
        local_name: &str,
        location: &CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !is_catalog_local_name(local_name) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_catalog_name",
                format!("invalid catalog local name `{}`", excerpt(local_name)),
                location,
            ));
        }
        Ok(Self {
            module_id,
            namespace_kind,
            local_name: local_name.to_owned(),
        })
    }

    /// Consume source-authored storage into a local ID instead of cloning the
    /// same name once for the input and once for its sealed identity.
    pub(crate) fn derive_owned(
        module_id: ModuleIdV2,
        namespace_kind: NamespaceKindV2,
        local_name: String,
        location: &CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !is_catalog_local_name(&local_name) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_catalog_name",
                format!("invalid catalog local name `{}`", excerpt(&local_name)),
                location,
            ));
        }
        Ok(Self::from_validated_owned_v3(
            module_id,
            namespace_kind,
            local_name,
        ))
    }

    /// Request predeclaration calls this only after the same Modules owner has
    /// checked the borrowed spelling without constructing a rich diagnostic.
    pub(crate) fn from_validated_owned_v3(
        module_id: ModuleIdV2,
        namespace_kind: NamespaceKindV2,
        local_name: String,
    ) -> Self {
        Self {
            module_id,
            namespace_kind,
            local_name,
        }
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleIdV2 {
        &self.module_id
    }

    #[must_use]
    pub const fn namespace_kind(&self) -> NamespaceKindV2 {
        self.namespace_kind
    }

    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}

/// Declarative asset reference resolved inside an authority transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogAssetRefInputV2 {
    pub module_id: ModuleIdV2,
    pub namespace_kind: NamespaceKindV2,
    pub local_name: String,
    pub location: CheckedLocatedSpanV2,
}

impl CatalogAssetRefInputV2 {
    pub(crate) fn derive(self) -> Result<CatalogAssetIdV2, ModuleDiagnosticV2> {
        CatalogAssetIdV2::derive_owned(
            self.module_id,
            self.namespace_kind,
            self.local_name,
            &self.location,
        )
    }
}
