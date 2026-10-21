//! Source-bounded projection of the admitted scalar Project(Scan) family.

use super::{
    project_scalar_plan, ExecutionCompositionBindingV3, ExecutionExpressionV3, ExecutionFamilyV3,
    ExecutionFieldV3, ExecutionItemV3, ExecutionRecordFieldV3, ExecutionRelationV3,
    ExecutionRelationalOperatorV3,
};
use quanta_gqlang_v2_sema::{
    ExpressionTypeStateV2, ResolvedScanSourceV2, ScalarSourceProjectReadV3,
    SourceQualifiedNodeIdV2, TypedExprShapeV2,
};

pub(crate) mod bounded;

impl ExecutionItemV3 {
    pub(crate) fn new_scalar_relation_v3(
        item_id: quanta_gqlang_v2_sema::SemanticItemIdV2,
        relation: ExecutionRelationV3,
        plan: &quanta_gqlang_v2_planning::AdmittedScalarPlanV3<'_>,
    ) -> Self {
        Self {
            item_id,
            family: ExecutionFamilyV3::Relation,
            relation: Some(relation),
            graph: None,
            check: None,
            presentation: None,
            composition: None,
            command: None,
            plan: project_scalar_plan(plan),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarProjectionFailureV3 {
    UnsupportedShape,
    DemandOverflow,
    Allocation,
}

/// A possible Vec-to-Box replacement keeps the old and new slot buffers live
/// together. Count that peak from the exact source inventory, even on pinned
/// allocators that can reuse the Vec buffer in place.
fn boxed_slots_peak<T>(count: usize) -> Option<usize> {
    count.checked_mul(std::mem::size_of::<T>())?.checked_mul(2)
}

fn scalar_field_bytes(output: &quanta_gqlang_v2_sema::RelationOutputContractV2) -> Option<usize> {
    let fields = output.row().fields();
    let slots = boxed_slots_peak::<ExecutionFieldV3>(fields.len())?;
    fields.iter().try_fold(slots, |total, field| {
        let value = field.value_type();
        if value.as_type().as_scalar().is_none() && !value.is_null() {
            return None;
        }
        total.checked_add(field.name().len())
    })
}

/// Validate every supported expression and count its first projection birth.
/// This read walk allocates nothing and uses only the retained semantic map.
pub fn scalar_relation_projection_demand_v3(
    read: &ScalarSourceProjectReadV3<'_, '_>,
) -> Result<usize, ScalarProjectionFailureV3> {
    let source = read
        .scan_source_v3()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let source_bytes = match source {
        ResolvedScanSourceV2::Relation(id) => id
            .namespace()
            .as_str()
            .len()
            .checked_add(id.qualified_name().len())
            .ok_or(ScalarProjectionFailureV3::DemandOverflow)?,
        ResolvedScanSourceV2::Composition(id) => id
            .binding()
            .canonical_name()
            .len()
            .checked_mul(2)
            .ok_or(ScalarProjectionFailureV3::DemandOverflow)?,
        ResolvedScanSourceV2::View(_) => return Err(ScalarProjectionFailureV3::UnsupportedShape),
    };
    let scan = read.scan_output_v3();
    let project = read.project_output_v3();
    if !scan.partition_keys().is_empty()
        || !scan.ordering().is_empty()
        || !project.partition_keys().is_empty()
        || !project.ordering().is_empty()
    {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let projection_count = read.project_expression_keys_v3().len();
    if projection_count == 0 || project.row().fields().len() != projection_count {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let mut bytes = scalar_field_bytes(scan)
        .and_then(|scan_bytes| scalar_field_bytes(project)?.checked_add(scan_bytes))
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let id_count = projection_count
        .checked_mul(2)
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    let slots = [
        boxed_slots_peak::<SourceQualifiedNodeIdV2>(projection_count), // Project operator
        id_count.checked_mul(std::mem::size_of::<SourceQualifiedNodeIdV2>()), // sorted scratch
        boxed_slots_peak::<ExecutionExpressionV3>(id_count),           // typed expression table
        boxed_slots_peak::<ExecutionRelationalOperatorV3>(1),          // Scan child
    ]
    .into_iter()
    .try_fold(0_usize, |sum, part| sum.checked_add(part?))
    .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    bytes = bytes
        .checked_add(slots)
        .and_then(|value| value.checked_add(source_bytes))
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    for key in read.project_expression_keys_v3() {
        let expr = read
            .expression_v3(key)
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        if expr.key() != key
            || !matches!(
                expr.type_state(),
                ExpressionTypeStateV2::Known(value)
                    if value.as_type().as_scalar().is_some()
            )
        {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        }
        let TypedExprShapeV2::Path { base, segments } = expr.shape() else {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        };
        let [segment] = segments.as_slice() else {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        };
        let base_expr = read
            .expression_v3(*base)
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        let TypedExprShapeV2::Binding(binding) = base_expr.shape() else {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        };
        if base_expr.key() != *base {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        }
        let ExpressionTypeStateV2::Known(base_value) = base_expr.type_state() else {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        };
        let base_record = base_value
            .as_record()
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        let base_record_bytes = base_record
            .fields()
            .iter()
            .try_fold(
                boxed_slots_peak::<ExecutionRecordFieldV3>(base_record.fields().len())
                    .ok_or(ScalarProjectionFailureV3::DemandOverflow)?,
                |sum, field| {
                    let value = field.value_type();
                    if value.as_type().as_scalar().is_none() && !value.is_null() {
                        return None;
                    }
                    sum.checked_add(field.name().len())
                },
            )
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        bytes = bytes
            .checked_add(base_record_bytes)
            .and_then(|value| value.checked_add(binding.canonical_name().len()))
            .and_then(|value| value.checked_add(boxed_slots_peak::<String>(1)?))
            .and_then(|value| value.checked_add(segment.name().len()))
            .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    }
    Ok(bytes)
}

/// Reserve both Query projections and the one Composition envelope before
/// either execution relation is born. The local-source String and binding copy
/// are counted by the body's scalar demand.
pub fn onelet_composition_projection_demand_v3(
    plan: &quanta_gqlang_v2_planning::AdmittedOneLetPlanV3<'_>,
) -> Result<usize, ScalarProjectionFailureV3> {
    let reads = plan
        .algebra_v3()
        .source_project_reads_v3()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    if !matches!(
        reads[0].scan_source_v3(),
        Some(ResolvedScanSourceV2::Relation(_))
    ) || !matches!(
        reads[1].scan_source_v3(),
        Some(ResolvedScanSourceV2::Composition(_))
    ) || reads[0].project_output_v3().row() != reads[1].scan_output_v3().row()
    {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let binding = plan
        .algebra_v3()
        .semantic_v3()
        .with_binding_v3()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let producer = scalar_relation_projection_demand_v3(&reads[0])?;
    let body = scalar_relation_projection_demand_v3(&reads[1])?;
    let schema = scalar_field_bytes(reads[0].project_output_v3())
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let bindings = boxed_slots_peak::<ExecutionCompositionBindingV3>(1)
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    producer
        .checked_add(body)
        .and_then(|value| value.checked_add(schema))
        .and_then(|value| value.checked_add(bindings))
        .and_then(|value| value.checked_add(binding.canonical_name().len().checked_mul(2)?))
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)
}

/// Called only after the same request root has reserved the demand above.
pub(crate) fn project_scalar_relation_v3(
    read: &ScalarSourceProjectReadV3<'_, '_>,
) -> Result<ExecutionRelationV3, ScalarProjectionFailureV3> {
    let source = read
        .scan_source_v3()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let count = read.project_expression_keys_v3().len();
    let mut project_keys = bounded::slots::<SourceQualifiedNodeIdV2>(count)?;
    let mut closure = bounded::slots::<SourceQualifiedNodeIdV2>(
        count
            .checked_mul(2)
            .ok_or(ScalarProjectionFailureV3::DemandOverflow)?,
    )?;
    for key in read.project_expression_keys_v3() {
        let expr = read
            .expression_v3(key)
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        let TypedExprShapeV2::Path { base, .. } = expr.shape() else {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        };
        project_keys.push(key);
        closure.push(key);
        closure.push(*base);
    }
    closure.sort_unstable();
    closure.dedup();
    let mut expressions = bounded::slots::<ExecutionExpressionV3>(closure.len())?;
    for key in closure {
        let expr = read
            .expression_v3(key)
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        expressions.push(bounded::expression(expr)?);
    }
    let scan = ExecutionRelationalOperatorV3 {
        family: quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
        join_mode: None,
        limit: None,
        stage: read.scan_key_v3(),
        source: Some(bounded::source(source)?),
        schema: bounded::schema(read.scan_output_v3())?,
        predicates: Box::new([]),
        expressions: Box::new([]),
        correlations: Box::new([]),
        existential_terms: Box::new([]),
        partition_input: Box::new([]),
        children: Box::new([]),
    };
    let mut children = bounded::slots::<ExecutionRelationalOperatorV3>(1)?;
    children.push(scan);
    let project = ExecutionRelationalOperatorV3 {
        family: quanta_gqlang_v2_algebra::AlgebraFamilyV2::Project,
        join_mode: None,
        limit: None,
        stage: read.project_key_v3(),
        source: None,
        schema: bounded::schema(read.project_output_v3())?,
        predicates: Box::new([]),
        expressions: project_keys.into_boxed_slice(),
        correlations: Box::new([]),
        existential_terms: Box::new([]),
        partition_input: Box::new([]),
        children: children.into_boxed_slice(),
    };
    Ok(ExecutionRelationV3 {
        item_id: read.item_id_v3(),
        root: project,
        expressions: expressions.into_boxed_slice(),
    })
}
