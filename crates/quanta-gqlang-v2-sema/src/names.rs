//! Stable document-local identities and resolved reference carriers.

use quanta_gqlang_v2_hir::{HirNodeIdV1, LocatedSpanV2, SourceIdV2};
use quanta_gqlang_v2_modules::{CatalogAssetIdV2, NamespaceKindV2, ResolvedDocumentCatalogReferenceV2};
use quanta_gqlang_v2_types::RelationSourceIdV2;

/// Stable origin class for one resolved value binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolvedBindingKindV2 {
    /// Relation alias or projected output.
    Alias,
    /// Formal parameter.
    Parameter,
    /// Local declaration.
    Local,
    /// Quantifier binder.
    Quantified,
    /// Composition working-set binding.
    Composition,
    /// Top-level document declaration.
    Declaration,
    /// Imported module binding.
    Import,
}

/// Source-qualified semantic binding identity retained by typed expressions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedValueBindingV2 {
    source_id: SourceIdV2,
    target_node: HirNodeIdV1,
    ordinal: usize,
    kind: ResolvedBindingKindV2,
    canonical_name: String,
}

impl ResolvedValueBindingV2 {
    /// Construct a stable binding identity.
    pub fn try_new(
        source_id: SourceIdV2,
        target_node: HirNodeIdV1,
        ordinal: usize,
        kind: ResolvedBindingKindV2,
        canonical_name: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let canonical_name = canonical_name.into();
        if canonical_name.is_empty() {
            return Err("resolved binding name cannot be empty");
        }
        Ok(Self { source_id, target_node, ordinal, kind, canonical_name })
    }
    /// Source identity.
    #[must_use] pub const fn source_id(&self) -> SourceIdV2 { self.source_id }
    /// Binding-site HIR node.
    #[must_use] pub const fn target_node(&self) -> HirNodeIdV1 { self.target_node }
    /// Binder-owned ordinal for siblings sharing the same owner node.
    #[must_use] pub const fn ordinal(&self) -> usize { self.ordinal }
    /// Origin class.
    #[must_use] pub const fn kind(&self) -> ResolvedBindingKindV2 { self.kind }
    /// Canonical resolved name/target.
    #[must_use] pub fn canonical_name(&self) -> &str { &self.canonical_name }
}

/// Sole downstream join key for one authored document item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticItemIdV2 {
    source_id: SourceIdV2,
    hir_item_id: HirNodeIdV1,
}

impl SemanticItemIdV2 {
    /// Derive an item identity from the checked source and HIR item identity.
    #[must_use]
    pub const fn new(source_id: SourceIdV2, hir_item_id: HirNodeIdV1) -> Self {
        Self { source_id, hir_item_id }
    }

    /// Source document identity.
    #[must_use]
    pub const fn source_id(self) -> SourceIdV2 {
        self.source_id
    }

    /// Original HIR item identity.
    #[must_use]
    pub const fn hir_item_id(self) -> HirNodeIdV1 {
        self.hir_item_id
    }
}

/// Resolved relation source including the reference location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRelationSourceV2 {
    id: ResolvedRelationSourceKindV2,
    location: LocatedSpanV2,
}

impl ResolvedRelationSourceV2 {
    /// Construct from the types-owned source identity.
    #[must_use]
    pub const fn new(id: ResolvedRelationSourceKindV2, location: LocatedSpanV2) -> Self {
        Self { id, location }
    }

    /// Types-owned relation source identity.
    #[must_use]
    pub const fn id(&self) -> &ResolvedRelationSourceKindV2 {
        &self.id
    }

    /// Exact reference location.
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location
    }
}

/// Closed identity domains for a resolved source. Local composition identity
/// is binder-owned; catalog IDs remain modules-owned.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolvedRelationSourceKindV2 {
    /// Host relation source.
    Host(RelationSourceIdV2),
    /// Local/imported catalog View.
    View(CatalogAssetIdV2),
    /// Composition local binding identified by owner node and ordinal.
    Composition { source_id: SourceIdV2, owner: HirNodeIdV1, ordinal: usize },
}

/// Kind-checked catalog reference. The modules-owned asset ID remains the
/// identity authority; sema only retains the use-site proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCatalogReferenceV2 {
    id: CatalogAssetIdV2,
    kind: NamespaceKindV2,
    location: LocatedSpanV2,
}

impl ResolvedCatalogReferenceV2 {
    /// Retain the modules-owned exact reference proof without deriving a
    /// second ID or allowing callers to supply an unchecked ID/kind pair.
    #[must_use]
    pub fn from_proof(proof: &ResolvedDocumentCatalogReferenceV2) -> Self {
        Self { id: proof.id().clone(), kind: proof.kind(), location: proof.location().location() }
    }

    /// Modules-owned asset ID.
    #[must_use]
    pub const fn id(&self) -> &CatalogAssetIdV2 {
        &self.id
    }

    /// Checked namespace kind.
    #[must_use]
    pub const fn kind(&self) -> NamespaceKindV2 {
        self.kind
    }

    /// Exact reference location.
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location
    }
}
