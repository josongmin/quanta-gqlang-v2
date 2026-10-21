//! Execution projection and seal for the same parser-issued scalar request.

use super::PlannedSourceBodyRequestSourceV3;
use crate::execution::{ExecutionDocumentV3, ExecutionFamilyV3, ExecutionItemV3};
use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
use quanta_gqlang_v2_execution_contract::producer::{
    onelet_composition_projection_demand_v3, project_onelet_item_v3, project_scalar_item_v3,
    scalar_relation_projection_demand_v3, seal_source_item_v3, ProjectedSourceItemV3,
    ScalarProjectionFailureV3,
};
use quanta_gqlang_v2_planning::AdmittedSourcePlanV3;
use quanta_gqlang_v2_sema::{DownstreamBudgetReceiptV2, SemanticItemIdV2, SourceIdV2};
use quanta_gqlang_v2_surface::{
    FrontendRequestMaterializationRootV3, FrontendRequestOwnedValueV3,
    FrontendRequestStorageSinkV3, FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3,
};

/// Parser-backed, planned scalar relation and its admitted execution DTO.
pub struct ProjectedSourceBodyRequestSourceV3<'authority> {
    owner: FrontendRequestOwnedValueV3<(
        AdmittedSourcePlanV3<'authority>,
        ProjectedSourceItemV3,
        FrontendRetainedStoragePermitV3,
    )>,
}

/// One source-bound execution document retaining the parser/stage permit chain
/// and current execution buffers. Consumed semantic and plan backing may be
/// released at seal; its raw cloneable V3 DTO remains private.
pub struct AdmittedScalarExecutionDocumentV3 {
    owner: FrontendRequestOwnedValueV3<(
        ExecutionDocumentV3,
        FrontendRetainedStoragePermitV3,
        FrontendRetainedStoragePermitV3,
        DownstreamBudgetReceiptV2,
        SemanticItemIdV2,
        ExecutionFamilyV3,
    )>,
}

pub struct AdmittedOneLetExecutionDocumentV3(AdmittedScalarExecutionDocumentV3);

/// Sharing retains the exact request-owned document and its Arc permit. The
/// wrapper keeps a Composition distinct from the scalar runtime capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedAdmittedOneLetExecutionDocumentV3(SharedAdmittedScalarExecutionDocumentV3);

/// Borrowed Composition facts. The cloneable execution DTO stays in its
/// request-owned allocation; this reader exposes no nested DTO reference.
#[derive(Clone, Copy)]
pub struct AdmittedOneLetCompositionReadV3<'a> {
    composition: &'a crate::execution::ExecutionCompositionV3,
}

impl std::fmt::Debug for AdmittedOneLetCompositionReadV3<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedOneLetCompositionReadV3")
            .field("binding_count", &self.composition.bindings().len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for AdmittedOneLetCompositionReadV3<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.composition, other.composition)
    }
}

impl Eq for AdmittedOneLetCompositionReadV3<'_> {}

#[derive(Clone, Copy)]
pub struct AdmittedOneLetBindingReadV3<'a> {
    binding: &'a crate::execution::ExecutionCompositionBindingV3,
    with_binding: &'a crate::execution::ExecutionBindingV3,
}

#[derive(Clone, Copy)]
pub struct AdmittedBindingIdentityReadV3<'a>(&'a crate::execution::ExecutionBindingV3);

impl<'a> AdmittedBindingIdentityReadV3<'a> {
    #[must_use]
    pub fn source_id_v3(self) -> SourceIdV2 {
        self.0.source_id()
    }
    #[must_use]
    pub fn owner_node_v3(self) -> u32 {
        self.0.owner_node()
    }
    #[must_use]
    pub fn ordinal_v3(self) -> usize {
        self.0.ordinal()
    }
    #[must_use]
    pub fn name_v3(self) -> &'a str {
        self.0.name()
    }
    #[must_use]
    pub fn matches_v3(self, other: Self) -> bool {
        self.0 == other.0
    }
}

/// One already-sealed, one-field RepoPath Project(Scan) relation, expressed
/// only as borrowed scalars. Its source and binding are still compared by the
/// runtime's canonical policy before an executable plan is issued.
#[derive(Clone, Copy)]
pub struct AdmittedProjectScanReadV3<'a> {
    item_id: SemanticItemIdV2,
    source_authority: crate::execution::ExecutionRelationAuthorityV3,
    source_kind: crate::execution::ExecutionSourceKindV3,
    source_namespace: Option<&'a str>,
    source_name: &'a str,
    source_binding: Option<AdmittedBindingIdentityReadV3<'a>>,
    scan_field_name: &'a str,
    output_field_name: &'a str,
    scan_cardinality: crate::execution::ExecutionCardinalityV3,
    output_cardinality: crate::execution::ExecutionCardinalityV3,
    scan_binding: AdmittedBindingIdentityReadV3<'a>,
}

impl<'a> AdmittedProjectScanReadV3<'a> {
    #[must_use]
    pub fn item_id_v3(self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub fn source_authority_v3(self) -> crate::execution::ExecutionRelationAuthorityV3 {
        self.source_authority
    }
    #[must_use]
    pub fn source_kind_v3(self) -> crate::execution::ExecutionSourceKindV3 {
        self.source_kind
    }
    #[must_use]
    pub fn source_namespace_v3(self) -> Option<&'a str> {
        self.source_namespace
    }
    #[must_use]
    pub fn source_name_v3(self) -> &'a str {
        self.source_name
    }
    #[must_use]
    pub fn source_binding_v3(self) -> Option<AdmittedBindingIdentityReadV3<'a>> {
        self.source_binding
    }
    #[must_use]
    pub fn scan_field_name_v3(self) -> &'a str {
        self.scan_field_name
    }
    #[must_use]
    pub fn output_field_name_v3(self) -> &'a str {
        self.output_field_name
    }
    #[must_use]
    pub fn scan_cardinality_v3(self) -> crate::execution::ExecutionCardinalityV3 {
        self.scan_cardinality
    }
    #[must_use]
    pub fn output_cardinality_v3(self) -> crate::execution::ExecutionCardinalityV3 {
        self.output_cardinality
    }
    #[must_use]
    pub fn scan_binding_v3(self) -> AdmittedBindingIdentityReadV3<'a> {
        self.scan_binding
    }
}

impl<'a> AdmittedOneLetCompositionReadV3<'a> {
    /// Bounded work for the two fixed-shape relation inspections below. Only
    /// lengths and first-child/first-field metadata are read in this preflight;
    /// no relation or expression payload is copied.
    #[must_use]
    pub fn project_scan_inspection_work_upper_v3(self) -> Option<u64> {
        let mut bytes = self.composition.with_binding().name().len();
        if let Some(binding) = self.composition.bindings().first() {
            bytes = bytes.checked_add(binding.binding().name().len())?;
            if let Some(schema) = binding.schema() {
                bytes = bytes.checked_add(
                    schema
                        .fields()
                        .first()
                        .map_or(0, |field| field.name().len()),
                )?;
            }
            if let Some(relation) = binding.relation() {
                bytes = bytes.checked_add(project_scan_inspection_bytes_v3(relation)?)?;
            }
        }
        if let Some(relation) = self.composition.body_relation() {
            bytes = bytes.checked_add(project_scan_inspection_bytes_v3(relation)?)?;
        }
        u64::try_from(bytes).ok()?.checked_mul(8)?.checked_add(128)
    }

    #[must_use]
    pub fn binding_count_v3(self) -> usize {
        self.composition.bindings().len()
    }

    #[must_use]
    pub fn binding_at_v3(self, index: usize) -> Option<AdmittedOneLetBindingReadV3<'a>> {
        self.composition
            .bindings()
            .get(index)
            .map(|binding| AdmittedOneLetBindingReadV3 {
                binding,
                with_binding: self.composition.with_binding(),
            })
    }

    #[must_use]
    pub fn with_binding_v3(self) -> AdmittedBindingIdentityReadV3<'a> {
        AdmittedBindingIdentityReadV3(self.composition.with_binding())
    }

    #[must_use]
    pub fn body_item_id_v3(self) -> SemanticItemIdV2 {
        self.composition.body_item_id()
    }

    #[must_use]
    pub fn body_family_v3(self) -> ExecutionFamilyV3 {
        self.composition.body_family()
    }

    #[must_use]
    pub fn body_is_relation_v3(self) -> bool {
        matches!(
            self.composition.body(),
            crate::execution::ExecutionCompositionBodyV3::Relation
        )
    }

    #[must_use]
    pub fn body_relation_present_v3(self) -> bool {
        self.composition.body_relation().is_some()
    }

    #[must_use]
    pub fn body_project_scan_v3(self) -> Option<AdmittedProjectScanReadV3<'a>> {
        project_scan_read_v3(self.composition.body_relation()?)
    }
}

fn project_scan_inspection_bytes_v3(
    relation: &crate::execution::ExecutionRelationV3,
) -> Option<usize> {
    let root = relation.root();
    let mut bytes = root
        .schema()
        .fields()
        .first()
        .map_or(0, |field| field.name().len());
    if let Some(scan) = root.children().first() {
        bytes = bytes.checked_add(
            scan.schema()
                .fields()
                .first()
                .map_or(0, |field| field.name().len()),
        )?;
        if let Some(source) = scan.source() {
            bytes = bytes.checked_add(source.name().len())?;
            bytes = bytes.checked_add(source.namespace().map_or(0, str::len))?;
            bytes = bytes.checked_add(
                source
                    .composition_binding()
                    .map_or(0, |binding| binding.name().len()),
            )?;
        }
    }
    for expression in relation.expressions().iter().take(2) {
        match expression.shape() {
            crate::execution::ExecutionExpressionShapeV3::Binding(binding) => {
                bytes = bytes.checked_add(binding.name().len())?;
            }
            crate::execution::ExecutionExpressionShapeV3::Path { segments, .. } => {
                bytes = bytes.checked_add(segments.first().map_or(0, String::len))?;
            }
            crate::execution::ExecutionExpressionShapeV3::Literal(_)
            | crate::execution::ExecutionExpressionShapeV3::Unary { .. }
            | crate::execution::ExecutionExpressionShapeV3::Binary { .. }
            | crate::execution::ExecutionExpressionShapeV3::Call { .. }
            | crate::execution::ExecutionExpressionShapeV3::Paren(_)
            | crate::execution::ExecutionExpressionShapeV3::Record(_)
            | crate::execution::ExecutionExpressionShapeV3::List(_)
            | crate::execution::ExecutionExpressionShapeV3::BoolValue(_)
            | crate::execution::ExecutionExpressionShapeV3::Compare { .. }
            | crate::execution::ExecutionExpressionShapeV3::And { .. }
            | crate::execution::ExecutionExpressionShapeV3::Or { .. }
            | crate::execution::ExecutionExpressionShapeV3::Not { .. }
            | crate::execution::ExecutionExpressionShapeV3::LogicalQuantifier { .. }
            | crate::execution::ExecutionExpressionShapeV3::RelationExists { .. } => {}
        }
    }
    Some(bytes)
}

impl<'a> AdmittedOneLetBindingReadV3<'a> {
    #[must_use]
    pub fn name_v3(self) -> &'a str {
        self.binding.binding().name()
    }

    #[must_use]
    pub fn with_name_v3(self) -> &'a str {
        self.with_binding.name()
    }

    #[must_use]
    pub fn matches_with_binding_v3(self) -> bool {
        self.binding.binding() == self.with_binding
    }

    #[must_use]
    pub fn identity_v3(self) -> AdmittedBindingIdentityReadV3<'a> {
        AdmittedBindingIdentityReadV3(self.binding.binding())
    }

    #[must_use]
    pub fn producer_item_id_v3(self) -> SemanticItemIdV2 {
        self.binding.producer_item_id()
    }

    #[must_use]
    pub fn producer_family_v3(self) -> ExecutionFamilyV3 {
        self.binding.producer_family()
    }

    #[must_use]
    pub fn producer_is_relation_v3(self) -> bool {
        matches!(
            self.binding.producer(),
            crate::execution::ExecutionCompositionProducerV3::Relation
        )
    }

    #[must_use]
    pub fn producer_relation_present_v3(self) -> bool {
        self.binding.relation().is_some()
    }

    #[must_use]
    pub fn producer_schema_present_v3(self) -> bool {
        self.binding.schema().is_some()
    }

    #[must_use]
    pub fn producer_project_scan_v3(self) -> Option<AdmittedProjectScanReadV3<'a>> {
        project_scan_read_v3(self.binding.relation()?)
    }

    #[must_use]
    pub fn producer_schema_matches_project_scan_v3(
        self,
        read: AdmittedProjectScanReadV3<'_>,
    ) -> bool {
        let Some(schema) = self.binding.schema() else {
            return false;
        };
        let [field] = schema.fields() else {
            return false;
        };
        field.name() == read.output_field_name_v3()
            && field.value_type()
                == &crate::execution::ExecutionValueTypeV3::Scalar(
                    crate::execution::ExecutionScalarTypeV3::RepoPath,
                )
            && schema.cardinality() == read.output_cardinality_v3()
            && schema.partition_keys().is_empty()
            && schema.order().is_empty()
    }

    #[must_use]
    pub fn sample_v3(self) -> Option<(crate::execution::ExecutionSampleModeV3, u32, bool)> {
        match self.binding.producer() {
            crate::execution::ExecutionCompositionProducerV3::Sample {
                mode, limit, scope, ..
            } => Some((*mode, *limit, scope.is_some())),
            crate::execution::ExecutionCompositionProducerV3::Relation
            | crate::execution::ExecutionCompositionProducerV3::Graph { .. } => None,
        }
    }
}

impl AdmittedOneLetExecutionDocumentV3 {
    pub fn into_shared_with_request_owner_v3(
        self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<SharedAdmittedOneLetExecutionDocumentV3, ScalarExecutionRequestErrorV3> {
        self.0
            .into_shared_with_request_owner_v3(root, source)
            .map(SharedAdmittedOneLetExecutionDocumentV3)
    }
}

impl SharedAdmittedOneLetExecutionDocumentV3 {
    pub fn shares_payload_v3(&self, other: &Self) -> bool {
        self.0.shares_payload_v3(&other.0)
    }
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self.0.source_id_v3()
    }
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.0.item_id_v3()
    }
    pub fn qualification_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.0.qualification_digest_v3()
    }
    pub fn source_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.0.source_digest_v3()
    }
    pub fn contract_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.0.contract_digest_v3()
    }
    pub fn item_family_v3(&self) -> ExecutionFamilyV3 {
        self.0.item_family_v3()
    }
    pub fn matches_source_bytes_v3(&self, source: &[u8]) -> bool {
        self.0.matches_source_bytes_v3(source)
    }
    fn composition_payload_v3(&self) -> Option<&crate::execution::ExecutionCompositionV3> {
        let [item] = self.0 .0.document.owner.value_v3().0.items() else {
            return None;
        };
        if item.item_id() != self.item_id_v3() || item.family() != ExecutionFamilyV3::Composition {
            return None;
        }
        item.composition()
    }
    #[must_use]
    pub fn composition_read_v3(&self) -> Option<AdmittedOneLetCompositionReadV3<'_>> {
        self.composition_payload_v3()
            .map(|composition| AdmittedOneLetCompositionReadV3 { composition })
    }
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.0.validate_owner_v3(root, source)
    }
    pub fn validate_issued_sink_v3(
        &self,
        sink: &dyn FrontendRequestStorageSinkV3,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.0.validate_issued_sink_v3(sink)
    }
}

pub enum AdmittedSourceExecutionDocumentV3 {
    Scalar(AdmittedScalarExecutionDocumentV3),
    OneLet(AdmittedOneLetExecutionDocumentV3),
}

struct SharedScalarExecutionDocumentPayloadV3 {
    document: AdmittedScalarExecutionDocumentV3,
    _arc_permit: FrontendRetainedStoragePermitV3,
}

/// Clone shares the immutable payload and all request permits. No DTO deep
/// clone or second issuer is involved.
#[derive(Clone)]
pub struct SharedAdmittedScalarExecutionDocumentV3(
    std::sync::Arc<SharedScalarExecutionDocumentPayloadV3>,
);

/// Borrowed, source-derived facts for the one canonical files Scan→Project
/// runtime family. The DTO and its cloneable nested values remain private.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmittedCanonicalFilesScalarReadV3<'a> {
    item_id: SemanticItemIdV2,
    source_namespace: &'a str,
    source_name: &'a str,
    scan_field_name: &'a str,
    output_field_name: &'a str,
    scan_cardinality: crate::execution::ExecutionCardinalityV3,
    output_cardinality: crate::execution::ExecutionCardinalityV3,
    binding_source_id: SourceIdV2,
    binding_owner_node: u32,
    binding_ordinal: usize,
}

impl<'a> AdmittedCanonicalFilesScalarReadV3<'a> {
    #[must_use]
    pub const fn item_id(self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn source_namespace(self) -> &'a str {
        self.source_namespace
    }
    #[must_use]
    pub const fn source_name(self) -> &'a str {
        self.source_name
    }
    #[must_use]
    pub const fn scan_field_name(self) -> &'a str {
        self.scan_field_name
    }
    #[must_use]
    pub const fn output_field_name(self) -> &'a str {
        self.output_field_name
    }
    #[must_use]
    pub const fn scan_cardinality(self) -> crate::execution::ExecutionCardinalityV3 {
        self.scan_cardinality
    }
    #[must_use]
    pub const fn output_cardinality(self) -> crate::execution::ExecutionCardinalityV3 {
        self.output_cardinality
    }
    #[must_use]
    pub const fn binding_source_id(self) -> SourceIdV2 {
        self.binding_source_id
    }
    #[must_use]
    pub const fn binding_owner_node(self) -> u32 {
        self.binding_owner_node
    }
    #[must_use]
    pub const fn binding_ordinal(self) -> usize {
        self.binding_ordinal
    }
}

/// One canonical Project(Scan) shape reader shared by scalar and one-LET
/// owners. It does not issue source authority or expose its cloneable DTO.
fn project_scan_read_v3(
    relation: &crate::execution::ExecutionRelationV3,
) -> Option<AdmittedProjectScanReadV3<'_>> {
    use crate::execution::{
        ExecutionExpressionShapeV3, ExecutionScalarTypeV3, ExecutionValueTypeV3,
    };
    let project = relation.root();
    let [scan] = project.children() else {
        return None;
    };
    if project.family() != AlgebraFamilyV2::Project
        || scan.family() != AlgebraFamilyV2::Scan
        || project.source().is_some()
        || !scan.children().is_empty()
        || scan.join_mode().is_some()
        || project.join_mode().is_some()
        || !project.predicates().is_empty()
        || !scan.predicates().is_empty()
        || !scan.expressions().is_empty()
        || !project.correlations().is_empty()
        || !scan.correlations().is_empty()
        || !project.existential_terms().is_empty()
        || !scan.existential_terms().is_empty()
        || !project.partition_input().is_empty()
        || !scan.partition_input().is_empty()
    {
        return None;
    }
    let source = scan.source()?;
    let [scan_field] = scan.schema().fields() else {
        return None;
    };
    let [output_field] = project.schema().fields() else {
        return None;
    };
    let expected = ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::RepoPath);
    if scan_field.value_type() != &expected
        || output_field.value_type() != &expected
        || !scan.schema().partition_keys().is_empty()
        || !scan.schema().order().is_empty()
        || !project.schema().partition_keys().is_empty()
        || !project.schema().order().is_empty()
    {
        return None;
    }
    let [path_id] = project.expressions() else {
        return None;
    };
    let [first, second] = relation.expressions() else {
        return None;
    };
    let path = if first.id() == *path_id {
        first
    } else if second.id() == *path_id {
        second
    } else {
        return None;
    };
    let ExecutionExpressionShapeV3::Path { base, segments } = path.shape() else {
        return None;
    };
    let [segment] = segments.as_ref() else {
        return None;
    };
    let binding_expression = if first.id() == *base {
        first
    } else if second.id() == *base {
        second
    } else {
        return None;
    };
    let ExecutionExpressionShapeV3::Binding(binding) = binding_expression.shape() else {
        return None;
    };
    let ExecutionValueTypeV3::Record(fields) = binding_expression.value_type() else {
        return None;
    };
    let [field] = fields.as_ref() else {
        return None;
    };
    if path.value_type() != &expected
        || field.name() != segment
        || field.value_type() != &expected
        || segment != scan_field.name()
        || binding.source_id() != scan.stage().source_id
        || binding.owner_node() != scan.stage().node_id.raw()
        || first.id() == second.id()
        || *path_id == *base
    {
        return None;
    }
    Some(AdmittedProjectScanReadV3 {
        item_id: relation.item_id(),
        source_authority: source.authority(),
        source_kind: source.kind(),
        source_namespace: source.namespace(),
        source_name: source.name(),
        source_binding: source
            .composition_binding()
            .map(AdmittedBindingIdentityReadV3),
        scan_field_name: scan_field.name(),
        output_field_name: output_field.name(),
        scan_cardinality: scan.schema().cardinality(),
        output_cardinality: project.schema().cardinality(),
        scan_binding: AdmittedBindingIdentityReadV3(binding),
    })
}

impl PartialEq for SharedAdmittedScalarExecutionDocumentV3 {
    fn eq(&self, other: &Self) -> bool {
        self.shares_payload_v3(other)
    }
}

impl Eq for SharedAdmittedScalarExecutionDocumentV3 {}

impl std::fmt::Debug for SharedAdmittedScalarExecutionDocumentV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SharedAdmittedScalarExecutionDocumentV3")
            .field("source_id", &self.source_id_v3())
            .field("item_id", &self.item_id_v3())
            .field("qualification_digest", &self.qualification_digest_v3())
            .finish_non_exhaustive()
    }
}

impl SharedAdmittedScalarExecutionDocumentV3 {
    #[must_use]
    pub fn shares_payload_v3(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
    #[must_use]
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self.0.document.source_id_v3()
    }
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.0.document.item_id_v3()
    }
    #[must_use]
    pub fn qualification_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.0.document.qualification_digest_v3()
    }
    #[must_use]
    pub fn source_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.0.document.source_digest_v3()
    }
    #[must_use]
    pub fn contract_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.0.document.contract_digest_v3()
    }
    #[must_use]
    pub fn item_family_v3(&self) -> crate::execution::ExecutionFamilyV3 {
        self.0.document.item_family_v3()
    }
    #[must_use]
    pub fn matches_source_bytes_v3(&self, source: &[u8]) -> bool {
        self.0.document.matches_source_bytes_v3(source)
    }
    /// Inspect the sealed DTO without exposing its cloneable relation tree.
    /// A source name is compared to issued runtime capability after the
    /// authority enum and exact Scan→Project shape have been verified here.
    #[must_use]
    pub fn canonical_files_scalar_read_v3(&self) -> Option<AdmittedCanonicalFilesScalarReadV3<'_>> {
        use crate::execution::{ExecutionRelationAuthorityV3, ExecutionSourceKindV3};
        let [item] = self.0.document.owner.value_v3().0.items() else {
            return None;
        };
        let relation = item.relation()?;
        if item.family() != ExecutionFamilyV3::Relation || relation.item_id() != item.item_id() {
            return None;
        }
        let read = project_scan_read_v3(relation)?;
        if read.source_authority != ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
            || read.source_kind != ExecutionSourceKindV3::Relation
            || read.source_binding.is_some()
        {
            return None;
        }
        Some(AdmittedCanonicalFilesScalarReadV3 {
            item_id: item.item_id(),
            source_namespace: read.source_namespace?,
            source_name: read.source_name,
            scan_field_name: read.scan_field_name,
            output_field_name: read.output_field_name,
            scan_cardinality: read.scan_cardinality,
            output_cardinality: read.output_cardinality,
            binding_source_id: read.scan_binding.source_id_v3(),
            binding_owner_node: read.scan_binding.owner_node_v3(),
            binding_ordinal: read.scan_binding.ordinal_v3(),
        })
    }
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.0.document.validate_owner_v3(root, source)?;
        self.0
            ._arc_permit
            .validate_owner_v3(root, self.source_id_v3(), source)
    }

    pub fn validate_issued_sink_v3(
        &self,
        sink: &dyn FrontendRequestStorageSinkV3,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.0.document.validate_issued_sink_v3(sink)?;
        self.0._arc_permit.validate_issued_sink_v3(sink)
    }
}

impl SharedAdmittedScalarExecutionDocumentV3 {
    /// Borrow the already-sealed Project(Scan) facts without exposing or
    /// cloning the execution DTO. Runtime still checks the issued authority.
    #[must_use]
    pub fn project_scan_scalar_read_v3(&self) -> Option<AdmittedProjectScanReadV3<'_>> {
        let [item] = self.0.document.owner.value_v3().0.items() else {
            return None;
        };
        let relation = item.relation()?;
        if item.family() != ExecutionFamilyV3::Relation || relation.item_id() != item.item_id() {
            return None;
        }
        project_scan_read_v3(relation)
    }

    /// Borrow the complete relation without exposing any cloneable DTO backing.
    /// The read and every nested observation remain tied to this shared owner.
    pub fn relation_read_v3(
        &self,
    ) -> Option<quanta_gqlang_v2_execution_contract::ExecutionRelationReadV3<'_>> {
        let [item] = self.0.document.owner.value_v3().0.items() else {
            return None;
        };
        let relation = item.relation()?;
        if item.family() != ExecutionFamilyV3::Relation || relation.item_id() != item.item_id() {
            return None;
        }
        Some(relation.read_v3())
    }
}

impl AdmittedScalarExecutionDocumentV3 {
    /// Charge the owned payload and a fixed shared-control allowance before
    /// the first Arc allocation. This ledger is a logical storage quota; it
    /// does not claim exact allocator metadata or a private std Arc layout.
    pub fn into_shared_with_request_owner_v3(
        self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<SharedAdmittedScalarExecutionDocumentV3, ScalarExecutionRequestErrorV3> {
        self.validate_owner_v3(root, source)
            .map_err(ScalarExecutionRequestErrorV3::Storage)?;
        const SHARED_CONTROL_ALLOWANCE_V3: usize = 64;
        let bytes = std::mem::size_of::<SharedScalarExecutionDocumentPayloadV3>()
            .checked_add(SHARED_CONTROL_ALLOWANCE_V3)
            .ok_or(ScalarExecutionRequestErrorV3::DemandOverflow)?;
        let permit = root
            .reserve_v3(self.source_id_v3(), source, bytes)
            .map_err(ScalarExecutionRequestErrorV3::Storage)?;
        Ok(SharedAdmittedScalarExecutionDocumentV3(
            std::sync::Arc::new(SharedScalarExecutionDocumentPayloadV3 {
                document: self,
                _arc_permit: permit,
            }),
        ))
    }

    #[must_use]
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self.owner.value_v3().0.source_id()
    }

    #[must_use]
    pub fn qualification_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.owner.value_v3().0.qualification_digest()
    }

    #[must_use]
    pub fn source_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.owner.value_v3().0.source_digest()
    }

    #[must_use]
    pub fn contract_digest_v3(&self) -> crate::execution::ExecutionDigestV3 {
        self.owner.value_v3().0.contract_digest()
    }

    #[must_use]
    pub fn item_family_v3(&self) -> crate::execution::ExecutionFamilyV3 {
        self.owner.value_v3().5
    }

    #[must_use]
    pub fn matches_source_bytes_v3(&self, source: &[u8]) -> bool {
        self.owner.value_v3().0.matches_source_bytes_v3(source)
    }

    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.owner.value_v3().4
    }

    #[must_use]
    pub fn expression_count_v3(&self) -> usize {
        self.owner
            .value_v3()
            .0
            .items()
            .first()
            .and_then(ExecutionItemV3::relation)
            .map_or(0, |relation| relation.expressions().len())
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        let source_id = self.source_id_v3();
        self.owner.validate_owner_v3(root, source_id, source)?;
        self.owner
            .value_v3()
            .1
            .validate_owner_v3(root, source_id, source)?;
        self.owner
            .value_v3()
            .2
            .validate_owner_v3(root, source_id, source)
    }

    pub fn validate_issued_sink_v3(
        &self,
        sink: &dyn FrontendRequestStorageSinkV3,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.owner.validate_issued_sink_v3(sink)?;
        self.owner.value_v3().1.validate_issued_sink_v3(sink)?;
        self.owner.value_v3().2.validate_issued_sink_v3(sink)
    }
}

impl ProjectedSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.owner.value_v3().1.item_id()
    }

    #[must_use]
    pub fn expression_count_v3(&self) -> usize {
        self.owner
            .value_v3()
            .1
            .relation()
            .map_or(0, |relation| relation.expressions().len())
    }
}

#[derive(Debug)]
pub enum ScalarExecutionRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    UnsupportedShape,
    DemandOverflow,
    Allocation,
    Closure,
}

impl From<ScalarProjectionFailureV3> for ScalarExecutionRequestErrorV3 {
    fn from(error: ScalarProjectionFailureV3) -> Self {
        match error {
            ScalarProjectionFailureV3::UnsupportedShape => Self::UnsupportedShape,
            ScalarProjectionFailureV3::DemandOverflow => Self::DemandOverflow,
            ScalarProjectionFailureV3::Allocation => Self::Allocation,
        }
    }
}

/// Project the planned scalar relation only after all execution DTO allocations
/// have entered the same original request root. The parsed carrier, plan and
/// projection permit remain one move-only owner.
pub fn project_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    planned: PlannedSourceBodyRequestSourceV3<'authority>,
) -> Result<ProjectedSourceBodyRequestSourceV3<'authority>, ScalarExecutionRequestErrorV3> {
    let owner = planned.owner.try_map_v3(|plan, _parent| {
        plan.validate_owner_v3(root, source)
            .map_err(ScalarExecutionRequestErrorV3::Storage)?;
        let (item, permit) = {
            let bytes = match &plan {
                AdmittedSourcePlanV3::Scalar(scalar) => {
                    let read = scalar
                        .scalar_semantic_read_v3()
                        .ok_or(ScalarExecutionRequestErrorV3::UnsupportedShape)?;
                    scalar_relation_projection_demand_v3(&read)?
                }
                AdmittedSourcePlanV3::OneLet(onelet) => {
                    onelet_composition_projection_demand_v3(onelet)?
                }
            };
            let permit = root
                .reserve_v3(plan.item_id_v3().source_id(), source, bytes)
                .map_err(ScalarExecutionRequestErrorV3::Storage)?;
            let item = match &plan {
                AdmittedSourcePlanV3::Scalar(scalar) => {
                    let read = scalar
                        .scalar_semantic_read_v3()
                        .ok_or(ScalarExecutionRequestErrorV3::UnsupportedShape)?;
                    ProjectedSourceItemV3::Scalar(project_scalar_item_v3(&read, scalar)?)
                }
                AdmittedSourcePlanV3::OneLet(_) => {
                    ProjectedSourceItemV3::OneLet(project_onelet_item_v3(&plan)?)
                }
            };
            (item, permit)
        };
        Ok::<_, ScalarExecutionRequestErrorV3>((plan, item, permit))
    })?;
    Ok(ProjectedSourceBodyRequestSourceV3 { owner })
}

/// Bind the one admitted execution item and all three canonical digests to
/// the original source. The item Vec and possible boxed-slice replacement
/// enter before their first allocation; digest writers stream into SHA-256.
pub fn seal_source_execution_document_with_request_owner_v3(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    projected: ProjectedSourceBodyRequestSourceV3<'_>,
) -> Result<AdmittedSourceExecutionDocumentV3, ScalarExecutionRequestErrorV3> {
    let is_onelet = matches!(
        &projected.owner.value_v3().0,
        AdmittedSourcePlanV3::OneLet(_)
    );
    let owner = projected
        .owner
        .try_map_v3(|(plan, item, projection_permit), _parent| {
            plan.validate_owner_v3(root, source)
                .map_err(ScalarExecutionRequestErrorV3::Storage)?;
            let receipt = match &plan {
                AdmittedSourcePlanV3::Scalar(scalar) => scalar
                    .scalar_semantic_read_v3()
                    .ok_or(ScalarExecutionRequestErrorV3::UnsupportedShape)?
                    .budget_receipt_v3(),
                AdmittedSourcePlanV3::OneLet(onelet) => onelet
                    .algebra_v3()
                    .source_project_reads_v3()
                    .ok_or(ScalarExecutionRequestErrorV3::UnsupportedShape)?[1]
                    .budget_receipt_v3(),
            };
            let item_id = item.item_id();
            let item_family = item.family();
            let item_slots = std::mem::size_of::<ExecutionItemV3>()
                .checked_mul(2)
                .ok_or(ScalarExecutionRequestErrorV3::DemandOverflow)?;
            let document_permit = root
                .reserve_v3(item_id.source_id(), source, item_slots)
                .map_err(ScalarExecutionRequestErrorV3::Storage)?;
            let execution = seal_source_item_v3(item, source.as_bytes())?;
            drop(plan);
            Ok::<_, ScalarExecutionRequestErrorV3>((
                execution,
                projection_permit,
                document_permit,
                receipt,
                item_id,
                item_family,
            ))
        })?;
    let document = AdmittedScalarExecutionDocumentV3 { owner };
    Ok(if is_onelet {
        AdmittedSourceExecutionDocumentV3::OneLet(AdmittedOneLetExecutionDocumentV3(document))
    } else {
        AdmittedSourceExecutionDocumentV3::Scalar(document)
    })
}

/// Preserve the scalar ingress contract while the shared source issuer also
/// accepts the checked one-LET Composition family.
pub fn seal_scalar_execution_document_with_request_owner_v3(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    projected: ProjectedSourceBodyRequestSourceV3<'_>,
) -> Result<AdmittedScalarExecutionDocumentV3, ScalarExecutionRequestErrorV3> {
    match seal_source_execution_document_with_request_owner_v3(source, root, projected)? {
        AdmittedSourceExecutionDocumentV3::Scalar(value) => Ok(value),
        AdmittedSourceExecutionDocumentV3::OneLet(_) => {
            Err(ScalarExecutionRequestErrorV3::UnsupportedShape)
        }
    }
}
