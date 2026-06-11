//! E06B — reusable product/asset IR.
//!
//! Every declaration lowers to a catalog-addressable product asset with a stable
//! `AssetIdV1` and a typed kind, so the catalog/asset plane can reference
//! declarations by id instead of re-deriving them. Exhaustive over
//! `DeclarationIrV1` (no catch-all) so a new declaration kind fails the build,
//! not silently.

use crate::root_ir::DeclarationIrV1;

/// A stable, catalog-addressable id for a product asset, e.g. `view::recent.errors`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetIdV1(pub String);

impl AssetIdV1 {
    #[must_use]
    pub fn for_declaration_v1(declaration: &DeclarationIrV1) -> Self {
        let (kind, name) = kind_tag_and_name_v1(declaration);
        Self(format!("{kind}::{name}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductAssetKindV1 {
    View,
    Rule,
    Model,
    Pack,
    Profile,
    Gate,
}

/// A declaration lowered into its reusable product-asset form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductAssetIrV1 {
    pub asset_id: AssetIdV1,
    pub kind: ProductAssetKindV1,
    pub declaration: DeclarationIrV1,
}

/// Lower a declaration into a product asset.
#[must_use]
pub fn lower_declaration_to_product_v1(declaration: DeclarationIrV1) -> ProductAssetIrV1 {
    let asset_id = AssetIdV1::for_declaration_v1(&declaration);
    let kind = product_asset_kind_v1(&declaration);
    ProductAssetIrV1 {
        asset_id,
        kind,
        declaration,
    }
}

fn product_asset_kind_v1(declaration: &DeclarationIrV1) -> ProductAssetKindV1 {
    match declaration {
        DeclarationIrV1::View { .. } => ProductAssetKindV1::View,
        DeclarationIrV1::Rule { .. } => ProductAssetKindV1::Rule,
        DeclarationIrV1::Model { .. } => ProductAssetKindV1::Model,
        DeclarationIrV1::Pack { .. } => ProductAssetKindV1::Pack,
        DeclarationIrV1::Profile { .. } => ProductAssetKindV1::Profile,
        DeclarationIrV1::Gate { .. } => ProductAssetKindV1::Gate,
    }
}

fn kind_tag_and_name_v1(declaration: &DeclarationIrV1) -> (&'static str, &str) {
    match declaration {
        DeclarationIrV1::View { name, .. } => ("view", name),
        DeclarationIrV1::Rule { name, .. } => ("rule", name),
        DeclarationIrV1::Model { name, .. } => ("model", name),
        DeclarationIrV1::Pack { name, .. } => ("pack", name),
        DeclarationIrV1::Profile { name, .. } => ("profile", name),
        DeclarationIrV1::Gate { name, .. } => ("gate", name),
    }
}
