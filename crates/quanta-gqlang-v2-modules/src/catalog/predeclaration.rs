//! Sealed local catalog IDs and locations for one document validator session.

use std::sync::Arc;

use super::{ensure_registry, DocumentCatalogAssetInputV2, PredeclaredDocumentAssetV2};
use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{CatalogAssetIdV2, NamespaceKindV2};
use crate::session_identity::{DocumentSessionIdentityV2, PredeclaredRequestCustodyV3};
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, LocatedSpanV2, SourceRegistryViewV2};

#[derive(Clone, Debug)]
pub(crate) struct PredeclaredDocumentAssetsV2 {
    pub(super) assets: Vec<(
        CatalogAssetIdV2,
        NamespaceKindV2,
        CheckedLocatedSpanV2,
        usize,
    )>,
    session_key: Arc<DocumentSessionIdentityV2>,
    pub(super) _request_custody: Option<Arc<PredeclaredRequestCustodyV3>>,
}

/// Request-local predeclaration refusal with no diagnostic payload allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentPredeclareRequestErrorV3 {
    Storage(quanta_gqlang_v2_hir::FrontendStorageRefusalV3),
    Phase,
    DemandOverflow,
    FallibleAllocation,
    RegistryMismatch(LocatedSpanV2),
    InvalidName(LocatedSpanV2),
    Duplicate(LocatedSpanV2, LocatedSpanV2),
    WrongModule(LocatedSpanV2),
    HostConflict(LocatedSpanV2, LocatedSpanV2),
    HostRegistryMismatch,
    Resource {
        dimension: &'static str,
        limit: usize,
        observed: usize,
    },
}

impl PredeclaredDocumentAssetsV2 {
    pub(crate) fn try_new(
        inputs: Vec<DocumentCatalogAssetInputV2>,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: Arc<DocumentSessionIdentityV2>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        Self::try_new_with_storage(inputs, source_registry, session, session_key, Vec::new())
    }

    pub(crate) fn try_new_with_storage(
        inputs: Vec<DocumentCatalogAssetInputV2>,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: Arc<DocumentSessionIdentityV2>,
        mut assets: Vec<(
            CatalogAssetIdV2,
            NamespaceKindV2,
            CheckedLocatedSpanV2,
            usize,
        )>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        for (input_ordinal, asset) in inputs.into_iter().enumerate() {
            session.charge_node()?;
            session.charge_string(asset.owner_module_id.as_str())?;
            session.charge_string(&asset.local_name)?;
            ensure_registry(&asset.location, source_registry)?;
            let id = CatalogAssetIdV2::derive_owned(
                asset.owner_module_id,
                asset.namespace_kind,
                asset.local_name,
                &asset.location,
            )?;
            match assets.binary_search_by(|(candidate, _, _, _)| candidate.cmp(&id)) {
                Ok(index) => {
                    let Some((_, _, original, _)) = assets.get(index) else {
                        return Err(ModuleDiagnosticV2::without_location(
                            "modules.internal_predeclared_index_mismatch",
                            "predeclared asset index is outside the sealed asset set",
                        ));
                    };
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.duplicate_document_catalog_asset",
                        "duplicate document catalog asset",
                        &asset.location,
                        original,
                    ));
                }
                Err(index) => assets.insert(
                    index,
                    (id, asset.namespace_kind, asset.location, input_ordinal),
                ),
            }
        }
        Ok(Self {
            assets,
            session_key,
            _request_custody: None,
        })
    }

    pub(crate) fn try_new_request_v3(
        inputs: Vec<DocumentCatalogAssetInputV2>,
        source_registry: &SourceRegistryViewV2,
        session_key: Arc<DocumentSessionIdentityV2>,
        _request_custody: Arc<PredeclaredRequestCustodyV3>,
        mut assets: Vec<(
            CatalogAssetIdV2,
            NamespaceKindV2,
            CheckedLocatedSpanV2,
            usize,
        )>,
    ) -> Result<Self, DocumentPredeclareRequestErrorV3> {
        let identity = source_registry.identity();
        let mut first_invalid = None;
        for (input_ordinal, asset) in inputs.into_iter().enumerate() {
            if asset.location.registry_identity() != &identity {
                if first_invalid.is_none() {
                    first_invalid = Some((
                        input_ordinal,
                        DocumentPredeclareRequestErrorV3::RegistryMismatch(
                            asset.location.location(),
                        ),
                    ));
                }
                continue;
            }
            if !crate::graph::is_catalog_local_name(&asset.local_name) {
                if first_invalid.is_none() {
                    first_invalid = Some((
                        input_ordinal,
                        DocumentPredeclareRequestErrorV3::InvalidName(asset.location.location()),
                    ));
                }
                continue;
            }
            let id = CatalogAssetIdV2::from_validated_owned_v3(
                asset.owner_module_id,
                asset.namespace_kind,
                asset.local_name,
            );
            if assets.len() >= assets.capacity() {
                return Err(DocumentPredeclareRequestErrorV3::FallibleAllocation);
            }
            assets.push((id, asset.namespace_kind, asset.location, input_ordinal));
        }
        // The request path owns exact source-count capacity, so append first
        // and sort once. The witness chooses the earliest authored duplicate
        // even though IDs sort independently of source order.
        assets.sort_unstable_by(|left, right| left.0.cmp(&right.0).then(left.3.cmp(&right.3)));
        let duplicate = assets
            .windows(2)
            .filter_map(|pair| match pair {
                [first, second] if first.0 == second.0 => {
                    Some((second.3, second.2.location(), first.2.location()))
                }
                _ => None,
            })
            .min_by_key(|(ordinal, _, _)| *ordinal);
        match (first_invalid, duplicate) {
            (
                Some((invalid_ordinal, invalid)),
                Some((duplicate_ordinal, _duplicate, _original)),
            ) if invalid_ordinal < duplicate_ordinal => return Err(invalid),
            (_, Some((_, duplicate, original))) => {
                return Err(DocumentPredeclareRequestErrorV3::Duplicate(
                    duplicate, original,
                ));
            }
            (Some((_, invalid)), None) => return Err(invalid),
            (None, None) => {}
        }
        Ok(Self {
            assets,
            session_key,
            _request_custody: Some(_request_custody),
        })
    }
    pub(crate) fn get(&self, id: &CatalogAssetIdV2) -> Option<NamespaceKindV2> {
        self.assets
            .binary_search_by(|(candidate, _, _, _)| candidate.cmp(id))
            .ok()
            .and_then(|index| self.assets.get(index))
            .map(|(_, kind, _, _)| *kind)
    }
    pub(crate) fn location(&self, id: &CatalogAssetIdV2) -> Option<&CheckedLocatedSpanV2> {
        self.assets
            .binary_search_by(|(candidate, _, _, _)| candidate.cmp(id))
            .ok()
            .and_then(|index| self.assets.get(index))
            .map(|(_, _, location, _)| location)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = PredeclaredDocumentAssetV2> + '_ {
        self.assets.iter().map(
            |(id, kind, location, input_ordinal)| PredeclaredDocumentAssetV2 {
                id: id.clone(),
                kind: *kind,
                location: location.clone(),
                session_key: self.session_key.clone(),
                input_ordinal: *input_ordinal,
                _request_custody: self._request_custody.clone(),
            },
        )
    }
    pub(crate) fn iter_borrowed(
        &self,
    ) -> impl Iterator<Item = (&CatalogAssetIdV2, NamespaceKindV2, &CheckedLocatedSpanV2)> + '_
    {
        self.assets
            .iter()
            .map(|(id, kind, location, _)| (id, *kind, location))
    }
    pub(crate) fn belongs_to(&self, key: &Arc<DocumentSessionIdentityV2>) -> bool {
        Arc::ptr_eq(&self.session_key, key)
    }
}
