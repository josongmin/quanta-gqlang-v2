//! The caller-provided type environment seam (ticket 03).
//!
//! To stay parallel-safe with the binder crate (ticket 02), this crate does NOT
//! resolve scopes or imports. Instead the caller supplies a symbol -> type map
//! that the checker consults for bare [`quanta_gqlang_v2_hir::SymbolRefV1`]
//! references. Ticket 02 will produce such a map from resolved bindings; until
//! then any caller (tests included) can build one directly.

use crate::ty::TypeV1;
use std::collections::BTreeMap;

/// A symbol -> type environment supplied by the caller.
///
/// Backed by a `BTreeMap` for deterministic iteration. The checker only *reads*
/// it; it never mutates or invents bindings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeEnvV1 {
    bindings: BTreeMap<String, TypeV1>,
}

impl TypeEnvV1 {
    /// An empty environment.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }

    /// Bind `name` to `ty`, returning the updated environment (builder style).
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, ty: TypeV1) -> Self {
        let _ = self.bindings.insert(name.into(), ty);
        self
    }

    /// Bind `name` to `ty` in place.
    pub fn insert(&mut self, name: impl Into<String>, ty: TypeV1) {
        let _ = self.bindings.insert(name.into(), ty);
    }

    /// Resolve a symbol to its type, if bound.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&TypeV1> {
        self.bindings.get(name)
    }

    /// Iterate over all bindings in deterministic key order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &TypeV1)> {
        self.bindings.iter()
    }

    /// Number of bindings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether the environment is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}
