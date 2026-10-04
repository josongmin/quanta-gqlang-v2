//! Execution projection and seal for the same parser-issued scalar request.

use super::PlannedSourceBodyRequestSourceV3;
use crate::execution::scalar_request::{
    project_scalar_relation_v3, scalar_relation_projection_demand_v3, ScalarProjectionFailureV3,
};
use crate::execution::{ExecutionDocumentV3, ExecutionFamilyV3, ExecutionItemV3};
use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
use quanta_gqlang_v2_planning::AdmittedScalarPlanV3;
use quanta_gqlang_v2_sema::{DownstreamBudgetReceiptV2, SemanticItemIdV2, SourceIdV2};
use quanta_gqlang_v2_surface::{
    FrontendRequestMaterializationRootV3, FrontendRequestOwnedValueV3,
    FrontendRequestStorageSinkV3, FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3,
};

/// Parser-backed, planned scalar relation and its admitted execution DTO.
pub struct ProjectedSourceBodyRequestSourceV3<'authority> {
    owner: FrontendRequestOwnedValueV3<(
        AdmittedScalarPlanV3<'authority>,
        ExecutionItemV3,
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
        use crate::execution::{
            ExecutionExpressionShapeV3, ExecutionRelationAuthorityV3, ExecutionScalarTypeV3,
            ExecutionSourceKindV3, ExecutionValueTypeV3,
        };
        let [item] = self.0.document.owner.value_v3().0.items() else {
            return None;
        };
        let relation = item.relation()?;
        if item.family() != ExecutionFamilyV3::Relation || relation.item_id() != item.item_id() {
            return None;
        }
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
        if source.authority() != ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
            || source.kind() != ExecutionSourceKindV3::Relation
            || source.composition_binding().is_some()
        {
            return None;
        }
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
        Some(AdmittedCanonicalFilesScalarReadV3 {
            item_id: item.item_id(),
            source_namespace: source.namespace()?,
            source_name: source.name(),
            scan_field_name: scan_field.name(),
            output_field_name: output_field.name(),
            scan_cardinality: scan.schema().cardinality(),
            output_cardinality: project.schema().cardinality(),
            binding_source_id: binding.source_id(),
            binding_owner_node: binding.owner_node(),
            binding_ordinal: binding.ordinal(),
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
            let read = plan
                .scalar_semantic_read_v3()
                .ok_or(ScalarExecutionRequestErrorV3::UnsupportedShape)?;
            let bytes = scalar_relation_projection_demand_v3(&read)?;
            let permit = root
                .reserve_v3(read.source_id_v3(), source, bytes)
                .map_err(ScalarExecutionRequestErrorV3::Storage)?;
            let relation = project_scalar_relation_v3(&read)?;
            let item = ExecutionItemV3::new_scalar_relation_v3(read.item_id_v3(), relation, &plan);
            (item, permit)
        };
        Ok::<_, ScalarExecutionRequestErrorV3>((plan, item, permit))
    })?;
    Ok(ProjectedSourceBodyRequestSourceV3 { owner })
}

/// Bind the one admitted execution item and all three canonical digests to
/// the original source. The item Vec and possible boxed-slice replacement
/// enter before their first allocation; digest writers stream into SHA-256.
pub fn seal_scalar_execution_document_with_request_owner_v3(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    projected: ProjectedSourceBodyRequestSourceV3<'_>,
) -> Result<AdmittedScalarExecutionDocumentV3, ScalarExecutionRequestErrorV3> {
    let owner = projected
        .owner
        .try_map_v3(|(plan, item, projection_permit), _parent| {
            plan.validate_owner_v3(root, source)
                .map_err(ScalarExecutionRequestErrorV3::Storage)?;
            let receipt = plan
                .scalar_semantic_read_v3()
                .ok_or(ScalarExecutionRequestErrorV3::UnsupportedShape)?
                .budget_receipt_v3();
            let source_id = item.item_id().source_id();
            let item_id = item.item_id();
            let item_family = item.family();
            let item_slots = std::mem::size_of::<ExecutionItemV3>()
                .checked_mul(2)
                .ok_or(ScalarExecutionRequestErrorV3::DemandOverflow)?;
            let document_permit = root
                .reserve_v3(source_id, source, item_slots)
                .map_err(ScalarExecutionRequestErrorV3::Storage)?;
            let mut items = Vec::new();
            items
                .try_reserve_exact(1)
                .map_err(|_error| ScalarExecutionRequestErrorV3::Allocation)?;
            if items.capacity() > 1 {
                return Err(ScalarExecutionRequestErrorV3::DemandOverflow);
            }
            items.push(item);
            let execution = ExecutionDocumentV3::new(source_id, source.as_bytes(), items);
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
    Ok(AdmittedScalarExecutionDocumentV3 { owner })
}
