//! The existing sealed source identity, including its issued Host instance.

use super::{ExecutionBindingV3, ExecutionRelationAuthorityV3, ExecutionSourceKindV3};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSourceV3 {
    pub(crate) authority: ExecutionRelationAuthorityV3,
    pub(crate) kind: ExecutionSourceKindV3,
    pub(crate) namespace: Option<String>,
    pub(crate) name: String,
    pub(crate) composition_binding: Option<ExecutionBindingV3>,
    pub(crate) host_binding_identity: Option<[u8; 32]>,
}
impl ExecutionSourceV3 {
    /// Exact Host instance chosen by the existing issued catalog.
    #[must_use]
    pub const fn host_binding_identity(&self) -> Option<[u8; 32]> {
        self.host_binding_identity
    }
    #[must_use]
    pub const fn authority(&self) -> ExecutionRelationAuthorityV3 {
        self.authority
    }
    #[must_use]
    pub const fn kind(&self) -> ExecutionSourceKindV3 {
        self.kind
    }
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Sealed Binder slot for a Composition source. Its name is also encoded in
    /// the scan and its exact slot is encoded by the enclosing composition.
    #[must_use]
    pub const fn composition_binding(&self) -> Option<&ExecutionBindingV3> {
        self.composition_binding.as_ref()
    }
}
