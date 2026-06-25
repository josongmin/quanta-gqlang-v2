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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::TypeV1;

    /// A fresh env is empty; `lookup` of any name misses.
    #[test]
    fn new_env_is_empty_and_misses_lookups() {
        let env = TypeEnvV1::new();
        assert!(env.is_empty());
        assert_eq!(env.len(), 0);
        assert_eq!(env.lookup("absent"), None);
    }

    /// `insert` binds in place and is observable via `lookup`/`len`.
    #[test]
    fn insert_binds_in_place() {
        let mut env = TypeEnvV1::new();
        env.insert("a", TypeV1::int());
        env.insert("b", TypeV1::string());
        assert!(!env.is_empty());
        assert_eq!(env.len(), 2);
        assert_eq!(env.lookup("a"), Some(&TypeV1::int()));
        assert_eq!(env.lookup("b"), Some(&TypeV1::string()));
        // A still-absent name misses.
        assert_eq!(env.lookup("c"), None);
    }

    /// A later `insert`/`with` for the same name overwrites the binding (last
    /// write wins), and does not grow the map.
    #[test]
    fn rebinding_same_name_overwrites() {
        let mut env = TypeEnvV1::new();
        env.insert("x", TypeV1::int());
        env.insert("x", TypeV1::float());
        assert_eq!(env.len(), 1);
        assert_eq!(env.lookup("x"), Some(&TypeV1::float()));

        let built = TypeEnvV1::new()
            .with("y", TypeV1::int())
            .with("y", TypeV1::bool());
        assert_eq!(built.len(), 1);
        assert_eq!(built.lookup("y"), Some(&TypeV1::bool()));
    }

    /// `iter` yields bindings in deterministic key (BTree) order.
    #[test]
    fn iter_yields_bindings_in_key_order() {
        let env = TypeEnvV1::new()
            .with("zeta", TypeV1::bool())
            .with("alpha", TypeV1::int())
            .with("mu", TypeV1::string());
        let names: Vec<&String> = env.iter().map(|(name, _)| name).collect();
        assert_eq!(names, vec!["alpha", "mu", "zeta"]);
        let alpha_ty = env.iter().find(|(n, _)| n.as_str() == "alpha");
        assert_eq!(alpha_ty.map(|(_, t)| t), Some(&TypeV1::int()));
    }
}
