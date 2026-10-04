//! Immutable Binder observations with no ownership transfer.

use super::{BindResultV1, BindingV1, HirNodeIdV1, ResolutionV1, ResolvedTargetV1, SpanV1};

/// Borrowed Binder evidence for Type input collection. Implementations expose
/// lookup results only; they do not issue source or request authority. A Type
/// request must consume and retain the concrete admitted owner separately.
pub trait BinderEvidenceReaderV3 {
    fn resolution_of(&self, reference: HirNodeIdV1) -> Option<&ResolutionV1>;
    fn target_of(&self, reference: HirNodeIdV1) -> Option<&ResolvedTargetV1>;
    fn target_of_use(
        &self,
        reference: HirNodeIdV1,
        span: SpanV1,
        name: &str,
    ) -> Option<&ResolvedTargetV1>;
    fn relation_output(&self, relation: HirNodeIdV1) -> Option<&[BindingV1]>;
    fn partition_input(&self, group: HirNodeIdV1) -> Option<&[BindingV1]>;
    fn composition_binding(&self, owner: HirNodeIdV1, ordinal: usize) -> Option<&BindingV1>;
    fn has_capture_set(&self, relation_exists: HirNodeIdV1) -> bool;
    fn capture_edges(&self, relation_exists: HirNodeIdV1) -> &[(HirNodeIdV1, HirNodeIdV1, usize)];
}

impl BinderEvidenceReaderV3 for BindResultV1 {
    fn resolution_of(&self, reference: HirNodeIdV1) -> Option<&ResolutionV1> {
        self.resolutions()
            .iter()
            .find(|entry| entry.reference == reference)
    }
    fn target_of(&self, reference: HirNodeIdV1) -> Option<&ResolvedTargetV1> {
        BindResultV1::target_of(self, reference)
    }

    fn target_of_use(
        &self,
        reference: HirNodeIdV1,
        span: SpanV1,
        name: &str,
    ) -> Option<&ResolvedTargetV1> {
        BindResultV1::target_of_use(self, reference, span, name)
    }

    fn relation_output(&self, relation: HirNodeIdV1) -> Option<&[BindingV1]> {
        BindResultV1::relation_output(self, relation)
    }

    fn partition_input(&self, group: HirNodeIdV1) -> Option<&[BindingV1]> {
        BindResultV1::partition_input(self, group)
    }

    fn composition_binding(&self, owner: HirNodeIdV1, ordinal: usize) -> Option<&BindingV1> {
        BindResultV1::composition_binding(self, owner, ordinal)
    }

    fn has_capture_set(&self, relation_exists: HirNodeIdV1) -> bool {
        BindResultV1::has_capture_set(self, relation_exists)
    }

    fn capture_edges(&self, relation_exists: HirNodeIdV1) -> &[(HirNodeIdV1, HirNodeIdV1, usize)] {
        BindResultV1::capture_edges(self, relation_exists)
    }
}
