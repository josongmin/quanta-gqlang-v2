//! Allocation-free checked Algebra view for one admitted Source -> Project.

use crate::node::{projection_agrees_with_output, scan_agrees_with_output};
use crate::{AlgebraFailureKindV2, AlgebraFamilyV2};
use quanta_gqlang_v2_sema::{
    AdmittedOneLetSemanticDocumentV3, AdmittedScalarSemanticDocumentV3,
    AdmittedSourceSemanticDocumentV3, CheckedLocatedSpanV2, DownstreamBudgetReceiptV2,
    FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3, OutputColumnOriginV2,
    RelationOutputContractV2, ResolvedScanSourceV2, ScalarSourceProjectReadV3, SemanticItemIdV2,
    TypedExprShapeV2, TypedExprV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, ExpressionEffectsV2, ExpressionTypeStateV2,
};

/// The semantic producer and its request permits remain attached. The finite
/// algebra family borrows the two sealed output proofs instead of cloning their
/// BTree provenance or issuing a detached V2 lowering product.
pub struct AdmittedScalarAlgebraV3<'authority> {
    semantic: AdmittedScalarSemanticDocumentV3<'authority>,
}

/// The two authenticated Query roots remain with the Composition semantic owner.
pub struct AdmittedOneLetAlgebraV3<'authority> {
    semantic: AdmittedOneLetSemanticDocumentV3<'authority>,
}

pub enum AdmittedSourceAlgebraV3<'authority> {
    Scalar(AdmittedScalarAlgebraV3<'authority>),
    OneLet(AdmittedOneLetAlgebraV3<'authority>),
}

impl AdmittedSourceAlgebraV3<'_> {
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        match self {
            Self::Scalar(value) => value.item_id_v3(),
            Self::OneLet(value) => value.semantic.authored_item_id_v3(),
        }
    }

    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        match self {
            Self::Scalar(value) => value.output_field_count_v3(),
            Self::OneLet(value) => value.semantic.output_field_count_v3(),
        }
    }

    #[must_use]
    pub const fn root_family_v3(&self) -> AlgebraFamilyV2 {
        AlgebraFamilyV2::Project
    }
}

impl AdmittedOneLetAlgebraV3<'_> {
    #[must_use]
    pub fn source_project_reads_v3(&self) -> Option<[ScalarSourceProjectReadV3<'_, '_>; 2]> {
        self.semantic.source_project_reads_v3()
    }

    #[must_use]
    pub fn semantic_v3(&self) -> &AdmittedOneLetSemanticDocumentV3<'_> {
        &self.semantic
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.semantic.validate_owner_v3(root, source)
    }
}

/// Immutable checked scalar Algebra observations for Planning.
pub struct ScalarAlgebraReadV3<'read, 'authority> {
    view: ScalarSourceProjectReadV3<'read, 'authority>,
}

impl ScalarAlgebraReadV3<'_, '_> {
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.view.item_id_v3()
    }

    #[must_use]
    pub fn budget_receipt_v3(&self) -> DownstreamBudgetReceiptV2 {
        self.view.budget_receipt_v3()
    }

    #[must_use]
    pub fn project_location_v3(&self) -> &CheckedLocatedSpanV2 {
        self.view.project_location_v3()
    }

    #[must_use]
    pub fn authored_item_location_v3(&self) -> &CheckedLocatedSpanV2 {
        self.view.authored_item_location_v3()
    }

    #[must_use]
    pub fn scan_location_v3(&self) -> &CheckedLocatedSpanV2 {
        self.view.scan_location_v3()
    }

    #[must_use]
    pub fn project_cardinality_v3(&self) -> CardinalityV1 {
        self.view.project_output_v3().cardinality()
    }

    #[must_use]
    pub fn project_effects_v3(&self) -> ExpressionEffectsV2 {
        self.view.project_output_v3().effects()
    }

    #[must_use]
    pub fn scan_effects_v3(&self) -> ExpressionEffectsV2 {
        self.view.scan_output_v3().effects()
    }

    /// Each key was checked at admission. The Option remains explicit so a
    /// later owner change cannot turn a missing fact into a silent omission.
    pub fn project_expression_effects_v3(
        &self,
    ) -> impl ExactSizeIterator<Item = Option<ExpressionEffectsV2>> + '_ {
        self.view
            .project_expression_keys_v3()
            .map(|key| self.view.expression_v3(key).map(TypedExprV2::effects))
    }
}

impl AdmittedScalarAlgebraV3<'_> {
    /// Borrow the original checked Sema scalar facts with their owner alive.
    #[must_use]
    pub fn scalar_semantic_read_v3(&self) -> Option<ScalarSourceProjectReadV3<'_, '_>> {
        self.semantic.scalar_source_project_read_v3()
    }

    /// Only the checked Project(Scan) shape receives a normalized view.
    #[must_use]
    pub fn scalar_read_v3(&self) -> Option<ScalarAlgebraReadV3<'_, '_>> {
        self.scalar_semantic_read_v3()
            .map(|view| ScalarAlgebraReadV3 { view })
    }

    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.semantic.authored_item_id_v3()
    }

    #[must_use]
    pub const fn root_family_v3(&self) -> AlgebraFamilyV2 {
        AlgebraFamilyV2::Project
    }

    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        self.semantic.output_field_count_v3()
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.semantic.validate_owner_v3(root, source)
    }
}

#[derive(Debug)]
pub enum ScalarAlgebraRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    UnsupportedShape,
    SemanticStepLimit,
    Algebra(AlgebraFailureKindV2),
}

fn finite_output(
    output: &RelationOutputContractV2,
    source: &ResolvedScanSourceV2,
    scan: bool,
) -> bool {
    if !output.partition_keys().is_empty() || !output.ordering().is_empty() {
        return false;
    }
    let mut provenance = output.provenance().iter();
    if provenance.next() != Some(source) || provenance.next().is_some() {
        return false;
    }
    output.columns().iter().all(|column| match column.origin() {
        OutputColumnOriginV2::SourceField {
            source: field,
            field_name,
            ..
        } => scan && field == source && !field_name.is_empty(),
        OutputColumnOriginV2::Expression(_) => !scan,
        OutputColumnOriginV2::Generated(_) => false,
    })
}

fn checked_scalar_view(
    view: &ScalarSourceProjectReadV3<'_, '_>,
    composition_source: bool,
) -> Result<(), ScalarAlgebraRequestErrorV3> {
    let source = view
        .scan_source_v3()
        .ok_or(ScalarAlgebraRequestErrorV3::UnsupportedShape)?;
    if !(if composition_source {
        matches!(source, ResolvedScanSourceV2::Composition(_))
    } else {
        matches!(source, ResolvedScanSourceV2::Relation(_))
    }) || !finite_output(view.scan_output_v3(), source, true)
        || !finite_output(view.project_output_v3(), source, false)
    {
        return Err(ScalarAlgebraRequestErrorV3::UnsupportedShape);
    }
    let source_id = view.source_id_v3();
    if view.item_id_v3().source_id() != source_id
        || view.scan_key_v3().source_id != source_id
        || view.project_key_v3().source_id != source_id
        || view.scan_location_v3().location().source_id != source_id
        || view.project_location_v3().location().source_id != source_id
        || !scan_agrees_with_output(view.scan_key_v3(), view.scan_output_v3())
        || view.scan_output_v3().provenance() != view.project_output_v3().provenance()
    {
        return Err(ScalarAlgebraRequestErrorV3::Algebra(
            AlgebraFailureKindV2::InvalidStageContract,
        ));
    }
    let output = view.project_output_v3();
    let keys = view.project_expression_keys_v3();
    if keys.len() == 0 || keys.len() != output.columns().len() {
        return Err(ScalarAlgebraRequestErrorV3::UnsupportedShape);
    }
    if !projection_agrees_with_output(
        view.project_expression_keys_v3().map(|key| {
            view.expression_v3(key)
                .filter(|expression| expression.key() == key)
        }),
        output,
    ) {
        return Err(ScalarAlgebraRequestErrorV3::Algebra(
            AlgebraFailureKindV2::InvalidStageContract,
        ));
    }
    for key in view.project_expression_keys_v3() {
        let expression = view
            .expression_v3(key)
            .ok_or(ScalarAlgebraRequestErrorV3::UnsupportedShape)?;
        if !matches!(expression.type_state(), ExpressionTypeStateV2::Known(value)
                if value.as_type().as_scalar().is_some())
            || expression.aggregate_context().aggregation() != AggregationClassV1::Scalar
            || expression.resolved_call().is_some()
            || !matches!(expression.shape(), TypedExprShapeV2::Path { segments, .. }
                if segments.len() == 1)
        {
            return Err(ScalarAlgebraRequestErrorV3::Algebra(
                AlgebraFailureKindV2::InvalidStageContract,
            ));
        }
    }
    Ok(())
}

/// Consume only the concrete semantic owner. This scalar normalization checks
/// the same Source/Project identities and output agreement as ordinary Algebra
/// without cloning the Sema-owned output contracts.
pub fn lower_scalar_source_project_with_request_owner_v3<'authority>(
    semantic: AdmittedScalarSemanticDocumentV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
) -> Result<AdmittedScalarAlgebraV3<'authority>, ScalarAlgebraRequestErrorV3> {
    semantic
        .validate_owner_v3(root, source)
        .map_err(ScalarAlgebraRequestErrorV3::Storage)?;
    let view = semantic
        .scalar_source_project_read_v3()
        .ok_or(ScalarAlgebraRequestErrorV3::UnsupportedShape)?;
    if view.budget_receipt_v3().max_semantic_steps() < 2 {
        return Err(ScalarAlgebraRequestErrorV3::SemanticStepLimit);
    }
    checked_scalar_view(&view, false)?;
    Ok(AdmittedScalarAlgebraV3 { semantic })
}

/// Normalize both already sealed Query roots using the same scalar shape
/// checker. Their exact producer-to-local-source row join was checked by Sema.
pub fn lower_source_semantic_with_request_owner_v3<'authority>(
    semantic: AdmittedSourceSemanticDocumentV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
) -> Result<AdmittedSourceAlgebraV3<'authority>, ScalarAlgebraRequestErrorV3> {
    match semantic {
        AdmittedSourceSemanticDocumentV3::Scalar(value) => {
            lower_scalar_source_project_with_request_owner_v3(value, root, source)
                .map(AdmittedSourceAlgebraV3::Scalar)
        }
        AdmittedSourceSemanticDocumentV3::OneLet(value) => {
            value
                .validate_owner_v3(root, source)
                .map_err(ScalarAlgebraRequestErrorV3::Storage)?;
            let reads = value
                .source_project_reads_v3()
                .ok_or(ScalarAlgebraRequestErrorV3::UnsupportedShape)?;
            if reads[0].budget_receipt_v3().max_semantic_steps() < 6 {
                return Err(ScalarAlgebraRequestErrorV3::SemanticStepLimit);
            }
            checked_scalar_view(&reads[0], false)?;
            checked_scalar_view(&reads[1], true)?;
            if reads[0].project_output_v3().row() != reads[1].scan_output_v3().row() {
                return Err(ScalarAlgebraRequestErrorV3::Algebra(
                    AlgebraFailureKindV2::InvalidStageContract,
                ));
            }
            Ok(AdmittedSourceAlgebraV3::OneLet(AdmittedOneLetAlgebraV3 {
                semantic: value,
            }))
        }
    }
}
