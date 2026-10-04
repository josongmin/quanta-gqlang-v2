//! Checked catalog input and Modules proof attachment.

use super::{
    ClosedFrontendAuthorityV2, DeclaredDocumentV2, DocumentCatalogAssetInputV2,
    ExtendedSourceRegistryViewV2, FrontendRequestMaterializationRootV3,
    FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3, HirNodeIdV1, NamespaceKindV2,
    PredeclaredDocumentAssetV2, ProgramItemHirV1,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogAssetInputsRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    SourceMismatch,
    DemandOverflow,
    FallibleAllocation,
    Registry(quanta_gqlang_v2_hir::SourceRegistryDiagnosticV2),
}

/// Modules-issued catalog proofs attached to parser-issued HIR identities.
/// In request execution the enclosing pipeline stage retains the permit for
/// this sorted index and the moved Modules proofs.
pub struct SourceCatalogAssetsV3 {
    by_node: Vec<(HirNodeIdV1, PredeclaredDocumentAssetV2)>,
}

impl SourceCatalogAssetsV3 {
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_node.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_node.is_empty()
    }

    pub(super) fn get(&self, node: &HirNodeIdV1) -> Option<&PredeclaredDocumentAssetV2> {
        self.by_node
            .binary_search_by_key(node, |(candidate, _)| *candidate)
            .ok()
            .and_then(|index| self.by_node.get(index).map(|(_, asset)| asset))
    }

    pub(super) fn try_from_modules_raw(
        declared: &DeclaredDocumentV2,
        assets: Vec<PredeclaredDocumentAssetV2>,
    ) -> Result<Self, SourceCatalogAttachErrorV3> {
        let mut by_node = Vec::new();
        by_node
            .try_reserve_exact(assets.len())
            .map_err(|_error| SourceCatalogAttachErrorV3::FallibleAllocation)?;
        attach_source_catalog_assets_into_v3(declared, assets, by_node)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCatalogAttachErrorV3 {
    Storage(FrontendStorageRefusalV3),
    SourceMismatch,
    DemandOverflow,
    FallibleAllocation,
    CountMismatch,
    InvalidOrdinal,
    DuplicateSourceNode,
    ProofMismatch,
}

/// Attach the validator's exact input ordinals to checked HIR node IDs before
/// building the node index. Names and spans are checked for consistency only;
/// they never select which source node receives a proof.
pub fn attach_source_catalog_assets_with_request_owner_v3(
    declared: &DeclaredDocumentV2,
    assets: Vec<PredeclaredDocumentAssetV2>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    permit: &mut FrontendRetainedStoragePermitV3,
) -> Result<SourceCatalogAssetsV3, SourceCatalogAttachErrorV3> {
    let source_id = declared.checked().source_id();
    permit
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceCatalogAttachErrorV3::Storage)?;
    if declared.checked().source_len() != source.len() {
        return Err(SourceCatalogAttachErrorV3::SourceMismatch);
    }
    let bytes = assets
        .len()
        .checked_mul(std::mem::size_of::<(HirNodeIdV1, PredeclaredDocumentAssetV2)>())
        .ok_or(SourceCatalogAttachErrorV3::DemandOverflow)?;
    if bytes > 0 {
        permit
            .try_extend_v3(root, source_id, source, bytes)
            .map_err(SourceCatalogAttachErrorV3::Storage)?;
    }
    let mut by_node = Vec::new();
    by_node
        .try_reserve_exact(assets.len())
        .map_err(|_error| SourceCatalogAttachErrorV3::FallibleAllocation)?;
    if by_node.capacity() > assets.len() {
        return Err(SourceCatalogAttachErrorV3::FallibleAllocation);
    }
    attach_source_catalog_assets_into_v3(declared, assets, by_node)
}

fn attach_source_catalog_assets_into_v3(
    declared: &DeclaredDocumentV2,
    mut assets: Vec<PredeclaredDocumentAssetV2>,
    mut by_node: Vec<(HirNodeIdV1, PredeclaredDocumentAssetV2)>,
) -> Result<SourceCatalogAssetsV3, SourceCatalogAttachErrorV3> {
    let source_id = declared.checked().source_id();
    let catalog_count = declared
        .checked()
        .checked()
        .document()
        .items
        .iter()
        .filter(|item| matches!(item, ProgramItemHirV1::CatalogDecl(_)))
        .count();
    if catalog_count != assets.len() {
        return Err(SourceCatalogAttachErrorV3::CountMismatch);
    }
    assets.sort_unstable_by_key(PredeclaredDocumentAssetV2::input_ordinal);
    for (ordinal, (decl, asset)) in declared
        .checked()
        .checked()
        .document()
        .items
        .iter()
        .filter_map(|item| match item {
            ProgramItemHirV1::CatalogDecl(decl) => Some(decl),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => None,
        })
        .zip(assets)
        .enumerate()
    {
        if asset.input_ordinal() != ordinal {
            return Err(SourceCatalogAttachErrorV3::InvalidOrdinal);
        }
        if asset.location().location().source_id != source_id
            || asset.location().location().span != decl.name_span
            || asset.id().local_name() != decl.name
            || asset.kind() != catalog_namespace_kind_v3(&decl.payload)
        {
            return Err(SourceCatalogAttachErrorV3::ProofMismatch);
        }
        if by_node.len() >= by_node.capacity() {
            return Err(SourceCatalogAttachErrorV3::FallibleAllocation);
        }
        by_node.push((decl.id, asset));
    }
    by_node.sort_unstable_by_key(|(node, _)| *node);
    if by_node
        .windows(2)
        .any(|pair| matches!(pair, [first, second] if first.0 == second.0))
    {
        return Err(SourceCatalogAttachErrorV3::DuplicateSourceNode);
    }
    Ok(SourceCatalogAssetsV3 { by_node })
}

fn catalog_namespace_kind_v3(payload: &quanta_gqlang_v2_hir::CatalogPayloadV1) -> NamespaceKindV2 {
    match payload {
        quanta_gqlang_v2_hir::CatalogPayloadV1::View(_) => NamespaceKindV2::View,
        quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. } => NamespaceKindV2::Model,
        quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. } => NamespaceKindV2::Pack,
        quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. } => NamespaceKindV2::Profile,
        quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. } => NamespaceKindV2::Gate,
        quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => NamespaceKindV2::Rule,
    }
}

/// Copy exact parser-authored local asset inputs only after their Vec and
/// cloned spellings are reserved on the same request as the checked HIR.
pub fn collect_catalog_asset_inputs_with_request_owner_v3(
    declared: &DeclaredDocumentV2,
    extended: &ExtendedSourceRegistryViewV2,
    authority: &ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    permit: &mut FrontendRetainedStoragePermitV3,
) -> Result<Vec<DocumentCatalogAssetInputV2>, CatalogAssetInputsRequestErrorV3> {
    let source_id = declared.checked().source_id();
    permit
        .validate_owner_v3(root, source_id, source)
        .map_err(CatalogAssetInputsRequestErrorV3::Storage)?;
    if declared.checked().source_len() != source.len()
        || extended.source_len(source_id) != Some(source.len())
    {
        return Err(CatalogAssetInputsRequestErrorV3::SourceMismatch);
    }
    let current_module = authority.modules().resolved_graph().current_module();
    let (slots, bytes) = declared
        .checked()
        .checked()
        .document()
        .items
        .iter()
        .filter_map(|item| match item {
            ProgramItemHirV1::CatalogDecl(decl) => Some(decl),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => None,
        })
        .try_fold((0usize, 0usize), |(slots, bytes), decl| {
            Some((
                slots.checked_add(1)?,
                bytes
                    .checked_add(current_module.as_str().len())?
                    .checked_add(decl.name.len())?,
            ))
        })
        .ok_or(CatalogAssetInputsRequestErrorV3::DemandOverflow)?;
    let bytes = slots
        .checked_mul(std::mem::size_of::<DocumentCatalogAssetInputV2>())
        .and_then(|struct_bytes| struct_bytes.checked_add(bytes))
        .ok_or(CatalogAssetInputsRequestErrorV3::DemandOverflow)?;
    if bytes > 0 {
        permit
            .try_extend_v3(root, source_id, source, bytes)
            .map_err(CatalogAssetInputsRequestErrorV3::Storage)?;
    }
    let mut inputs = Vec::new();
    inputs
        .try_reserve_exact(slots)
        .map_err(|_error| CatalogAssetInputsRequestErrorV3::FallibleAllocation)?;
    if inputs.capacity() > slots {
        return Err(CatalogAssetInputsRequestErrorV3::FallibleAllocation);
    }
    for item in &declared.checked().checked().document().items {
        let ProgramItemHirV1::CatalogDecl(decl) = item else {
            continue;
        };
        if inputs.len() >= slots {
            return Err(CatalogAssetInputsRequestErrorV3::FallibleAllocation);
        }
        let location = quanta_gqlang_v2_hir::LocatedSpanV2 {
            source_id,
            span: decl.name_span,
        };
        let checked_location = extended
            .combined()
            .try_check_location(location)
            .map_err(CatalogAssetInputsRequestErrorV3::Registry)?;
        inputs.push(DocumentCatalogAssetInputV2 {
            owner_module_id: current_module.clone(),
            local_name: decl.name.clone(),
            namespace_kind: catalog_namespace_kind_v3(&decl.payload),
            location: checked_location,
        });
    }
    Ok(inputs)
}
