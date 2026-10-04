//! Sealed HIR document integrity and source identity.

use super::{
    document_failure, location, CheckedHirDocumentOwnedV2, CheckedSyntaxDocumentV2,
    FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2, HirDocumentV1,
    SourceIdV2, SpanV1,
};

impl CheckedSyntaxDocumentV2 {
    /// Concrete scratch demand owned by the syntax checker, before allocation.
    #[must_use]
    pub fn request_integrity_scratch_bytes_v3(issued_node_slots: usize) -> Option<usize> {
        quanta_gqlang_v2_hir::request_integrity_scratch_bytes_v3(issued_node_slots)
    }

    /// Consume raw HIR and invoke the sole HIR document integrity checker.
    pub fn try_from_hir(document: HirDocumentV1) -> Result<Self, FrontendFailureV2> {
        let source_id = document.source_id;
        let checked = CheckedHirDocumentOwnedV2::try_new(document)
            .map_err(|error| integrity_failure(source_id, error))?;
        Ok(Self { checked })
    }

    /// Consume source-parser HIR using the caller's preadmitted indexed-ID
    /// and iterative depth scratch. The caller retains its scratch permit
    /// until this method returns.
    pub fn try_from_hir_with_source_slots_v3(
        document: HirDocumentV1,
        issued_node_slots: usize,
    ) -> Result<Self, quanta_gqlang_v2_hir::HirRequestIntegrityErrorV3> {
        let checked =
            CheckedHirDocumentOwnedV2::try_new_with_source_slots_v3(document, issued_node_slots)?;
        Ok(Self { checked })
    }

    /// Consume SHOW HIR through the zero-heap monotonic identity proof.
    pub fn try_from_show_hir_monotonic_v2(
        document: HirDocumentV1,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = document.source_id;
        let checked =
            CheckedHirDocumentOwnedV2::try_new_show_monotonic_v2(document).map_err(|error| {
                match error {
                quanta_gqlang_v2_hir::ShowIntegrityErrorV2::Integrity(error) => {
                    integrity_failure(source_id, error)
                }
                quanta_gqlang_v2_hir::ShowIntegrityErrorV2::IncompleteShowDocument
                | quanta_gqlang_v2_hir::ShowIntegrityErrorV2::NonMonotonicIdentity => {
                    document_failure(
                        "frontend.show_integrity_incomplete",
                        source_id,
                        SpanV1::point(0),
                        "SHOW HIR does not satisfy the parser-issued monotonic identity contract",
                    )
                }
            }
            })?;
        Ok(Self { checked })
    }

    /// Checked source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.checked.document().source_id
    }
    /// Sole source-length authority.
    #[must_use]
    pub const fn source_len(&self) -> usize {
        self.checked.document().source_len
    }
    /// Document-wide count of checked HIR nodes.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.checked.node_count()
    }
    /// Borrow the integrity-proven document for owner kernels.
    #[must_use]
    pub const fn checked(&self) -> &CheckedHirDocumentOwnedV2 {
        &self.checked
    }
}

fn integrity_failure(
    source_id: SourceIdV2,
    error: quanta_gqlang_v2_hir::HirIntegrityErrorV1,
) -> FrontendFailureV2 {
    let (span, secondary) = match &error {
        quanta_gqlang_v2_hir::HirIntegrityErrorV1::DuplicateNodeId {
            first_span,
            second_span,
            ..
        } => (*second_span, vec![location(source_id, *first_span)]),
        quanta_gqlang_v2_hir::HirIntegrityErrorV1::InvertedSpan { span, .. }
        | quanta_gqlang_v2_hir::HirIntegrityErrorV1::SpanOutOfBounds { span, .. }
        | quanta_gqlang_v2_hir::HirIntegrityErrorV1::InvertedAuxiliarySpan { span }
        | quanta_gqlang_v2_hir::HirIntegrityErrorV1::AuxiliarySpanOutOfBounds { span, .. }
        | quanta_gqlang_v2_hir::HirIntegrityErrorV1::ExcessiveDepth { span, .. } => {
            (*span, Vec::new())
        }
    };
    FrontendFailureV2::one(FrontendDiagnosticV2::new(
        if matches!(
            error,
            quanta_gqlang_v2_hir::HirIntegrityErrorV1::ExcessiveDepth { .. }
        ) {
            FrontendStageV2::Resource
        } else {
            FrontendStageV2::Integrity
        },
        error.code(),
        FrontendSeverityV2::Error,
        location(source_id, span),
        secondary,
        error.to_string(),
    ))
}
