//! Stable document-local identities and resolved reference carriers.

use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, HirNodeIdV1, LocatedSpanV2, SourceIdV2};
use quanta_gqlang_v2_modules::{
    CatalogAssetIdV2, NamespaceKindV2, ResolvedDocumentCatalogReferenceV2,
};
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
        Ok(Self {
            source_id,
            target_node,
            ordinal,
            kind,
            canonical_name,
        })
    }
    /// Source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }
    /// Binding-site HIR node.
    #[must_use]
    pub const fn target_node(&self) -> HirNodeIdV1 {
        self.target_node
    }
    /// Binder-owned ordinal for siblings sharing the same owner node.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    /// Origin class.
    #[must_use]
    pub const fn kind(&self) -> ResolvedBindingKindV2 {
        self.kind
    }
    /// Canonical resolved name/target.
    #[must_use]
    pub fn canonical_name(&self) -> &str {
        &self.canonical_name
    }
}

/// Binder-derived identity of one Composition working-set binding.
///
/// The fields deliberately remain private: a relation scan can retain this
/// identity, but no public Sema API can remint a Composition source from a
/// raw `(source, owner, ordinal)` tuple. The document transaction creates it
/// only after preserving the Binder's exact resolved binding.
///
/// ```compile_fail
/// let _ = quanta_gqlang_v2_sema::ResolvedCompositionSourceIdV2 {
///     binding: unreachable!(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedCompositionSourceIdV2 {
    binding: ResolvedValueBindingV2,
}

impl ResolvedCompositionSourceIdV2 {
    /// Seal one exact Binder-owned Composition binding for a relation source.
    pub(crate) fn try_from_binding(binding: ResolvedValueBindingV2) -> Result<Self, &'static str> {
        if binding.kind() != ResolvedBindingKindV2::Composition {
            return Err("composition source requires a Binder Composition binding");
        }
        Ok(Self { binding })
    }

    /// Exact retained Binder binding; callers may inspect it but cannot
    /// construct another composition source identity from its coordinates.
    #[must_use]
    pub const fn binding(&self) -> &ResolvedValueBindingV2 {
        &self.binding
    }
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
        Self {
            source_id,
            hir_item_id,
        }
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
    Relation(RelationSourceIdV2),
    /// Local/imported catalog View.
    View(CatalogAssetIdV2),
    /// Composition local binding with its sealed Binder identity.
    Composition(ResolvedCompositionSourceIdV2),
}

/// Kind-checked catalog reference. The modules-owned asset ID remains the
/// identity authority; sema only retains the use-site proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCatalogReferenceV2 {
    id: CatalogAssetIdV2,
    kind: NamespaceKindV2,
    location: CheckedLocatedSpanV2,
}

impl ResolvedCatalogReferenceV2 {
    /// Retain the modules-owned exact reference proof without deriving a
    /// second ID or allowing callers to supply an unchecked ID/kind pair.
    #[must_use]
    pub fn from_proof(proof: &ResolvedDocumentCatalogReferenceV2) -> Self {
        Self {
            id: proof.id().clone(),
            kind: proof.kind(),
            location: proof.location().clone(),
        }
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

    /// Exact reference location and the registry session that checked it.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unwrap_used,
        reason = "identity owner tests construct minimal Binder-carried inputs"
    )]

    use super::*;

    #[test]
    fn composition_source_identity_accepts_only_a_composition_binding() {
        let source_id = SourceIdV2::from_raw(9);
        let owner = HirNodeIdV1::from_raw(41);
        let composition = ResolvedValueBindingV2::try_new(
            source_id,
            owner,
            3,
            ResolvedBindingKindV2::Composition,
            "rows",
        )
        .unwrap();
        let sealed = ResolvedCompositionSourceIdV2::try_from_binding(composition.clone()).unwrap();

        assert_eq!(sealed.binding(), &composition);
        assert!(ResolvedCompositionSourceIdV2::try_from_binding(
            ResolvedValueBindingV2::try_new(
                source_id,
                owner,
                3,
                ResolvedBindingKindV2::Alias,
                "rows",
            )
            .unwrap(),
        )
        .is_err());
    }
}
