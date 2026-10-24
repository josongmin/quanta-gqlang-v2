//! Source-bounded projection of the admitted scalar Project(Scan) family.

use super::{
    project_scalar_plan, ExecutionCompositionBindingV3, ExecutionExpressionV3, ExecutionFamilyV3,
    ExecutionFieldV3, ExecutionItemV3, ExecutionRecordFieldV3, ExecutionRelationV3,
    ExecutionRelationalOperatorV3,
};
use quanta_gqlang_v2_sema::{
    ExpressionTypeStateV2, ResolvedScanSourceV2, ScalarSourceProjectReadV3, ScalarTyV1,
    SourceQualifiedNodeIdV2, TypedExprShapeV2, TypedLiteralFactV2, TypedLiteralValueV2,
    ValueTypeV1,
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

fn field_type_bytes(value: &ValueTypeV1, scan: bool) -> Result<usize, ScalarProjectionFailureV3> {
    if value.as_type().as_scalar().is_some() || value.is_null() {
        return Ok(0);
    }
    if scan
        && (value
            .as_optional()
            .is_some_and(|inner| inner.inner().as_type().as_scalar() == Some(&ScalarTyV1::String))
            || value.as_collection().is_some_and(|collection| {
                collection.cardinality() == quanta_gqlang_v2_sema::CardinalityV1::Sequence
                    && collection.element().as_type().as_scalar() == Some(&ScalarTyV1::String)
            }))
    {
        return boxed_slots_peak::<super::ExecutionValueTypeV3>(1)
            .ok_or(ScalarProjectionFailureV3::DemandOverflow);
    }
    Err(ScalarProjectionFailureV3::UnsupportedShape)
}

fn field_bytes(
    output: &quanta_gqlang_v2_sema::RelationOutputContractV2,
    scan: bool,
) -> Result<usize, ScalarProjectionFailureV3> {
    let fields = output.row().fields();
    let slots = boxed_slots_peak::<ExecutionFieldV3>(fields.len())
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    fields.iter().try_fold(slots, |total, field| {
        let nested = field_type_bytes(field.value_type(), scan)?;
        total
            .checked_add(field.name().len())
            .and_then(|sum| sum.checked_add(nested))
            .ok_or(ScalarProjectionFailureV3::DemandOverflow)
    })
}

/// The admitted Filter subset has one checked scalar path, a same-typed
/// scalar literal, and their Bool comparison. Return the closed identities in
/// source-independent order; the final DTO sorts them with Project identities.
fn filter_expression_ids(
    read: &ScalarSourceProjectReadV3<'_, '_>,
) -> Result<Option<[SourceQualifiedNodeIdV2; 4]>, ScalarProjectionFailureV3> {
    let Some(predicate_key) = read.filter_predicate_key_v3() else {
        return if read.filter_key_v3().is_none() && read.filter_output_v3().is_none() {
            Ok(None)
        } else {
            Err(ScalarProjectionFailureV3::UnsupportedShape)
        };
    };
    if read.filter_key_v3().is_none() || read.filter_output_v3().is_none() {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let predicate = read
        .expression_v3(predicate_key)
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let TypedExprShapeV2::Compare { operator, lhs, rhs } = predicate.shape() else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let operator_symbol = operator
        .symbol()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    if predicate.key() != predicate_key
        || predicate_key.source_id != read.source_id_v3()
        || predicate.type_state() != &ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let path = read
        .expression_v3(*lhs)
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let TypedExprShapeV2::Path { base, segments } = path.shape() else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let [segment] = segments.as_slice() else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let binding = read
        .expression_v3(*base)
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let literal = read
        .expression_v3(*rhs)
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let (ExpressionTypeStateV2::Known(path_type), ExpressionTypeStateV2::Known(literal_type)) =
        (path.type_state(), literal.type_state())
    else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let TypedExprShapeV2::Literal(TypedLiteralFactV2::Value(literal_value)) = literal.shape()
    else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let literal_supported = match literal_value {
        TypedLiteralValueV2::Int(_) => {
            path_type == &ValueTypeV1::int()
                && matches!(operator_symbol, "=" | "!=" | "<" | "<=" | ">" | ">=")
        }
        TypedLiteralValueV2::String(_) => {
            path_type == &ValueTypeV1::string()
                && matches!(operator_symbol, "=" | "!=" | "<" | "<=" | ">" | ">=")
        }
        TypedLiteralValueV2::Bool(_) => {
            path_type == &ValueTypeV1::bool() && matches!(operator_symbol, "=" | "!=")
        }
        TypedLiteralValueV2::Float(_) | TypedLiteralValueV2::Null => false,
    };
    if path.key() != *lhs
        || binding.key() != *base
        || literal.key() != *rhs
        || [predicate_key, *lhs, *base, *rhs]
            .iter()
            .any(|key| key.source_id != read.source_id_v3())
        || !literal_supported
        || literal_type != path_type
        || !matches!(binding.shape(), TypedExprShapeV2::Binding(_))
        || !matches!(binding.type_state(), ExpressionTypeStateV2::Known(value)
            if value.as_record().is_some_and(|row| row.field(segment.name()).is_some_and(|field|
                field.value_type() == path_type)))
    {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    Ok(Some([predicate_key, *lhs, *base, *rhs]))
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
    let filter = read.filter_output_v3();
    let predicate_ids = filter_expression_ids(read)?;
    let expression_count = projection_count
        .checked_mul(2)
        .and_then(|count| count.checked_add(if predicate_ids.is_some() { 4 } else { 0 }))
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    if read.expression_keys_v3().len() != expression_count {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let filter_bytes = match filter {
        Some(row) => field_bytes(row, true)?,
        None => 0,
    };
    let mut bytes = field_bytes(scan, true)?
        .checked_add(field_bytes(project, false)?)
        .and_then(|value| value.checked_add(filter_bytes))
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    let slots = [
        boxed_slots_peak::<SourceQualifiedNodeIdV2>(projection_count), // Project operator
        boxed_slots_peak::<SourceQualifiedNodeIdV2>(usize::from(predicate_ids.is_some())),
        expression_count.checked_mul(std::mem::size_of::<SourceQualifiedNodeIdV2>()),
        boxed_slots_peak::<ExecutionExpressionV3>(expression_count),
        boxed_slots_peak::<ExecutionRelationalOperatorV3>(1), // Scan child
        boxed_slots_peak::<ExecutionRelationalOperatorV3>(usize::from(predicate_ids.is_some())),
    ]
    .into_iter()
    .try_fold(0_usize, |sum, part| sum.checked_add(part?))
    .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    bytes = bytes
        .checked_add(slots)
        .and_then(|value| value.checked_add(source_bytes))
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    bytes = bytes
        .checked_add(project_expression_storage_bytes(read)?)
        .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
    if let Some(ids) = predicate_ids {
        for key in ids {
            bytes = bytes
                .checked_add(checked_expression_bytes(read, key)?)
                .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
        }
    }
    Ok(bytes)
}

/// Checked HIR owns unique syntax identities and the semantic seal equates
/// its complete expression census with that HIR. Each projected Path therefore
/// owns a distinct Binding; a Filter owns four more distinct facts. Visit those
/// exact references once rather than rebuilding membership by searching every
/// Project root for every retained expression. The caller checks the closed
/// census length before this walk. No scratch allocation or second index arises.
fn project_expression_storage_bytes(
    read: &ScalarSourceProjectReadV3<'_, '_>,
) -> Result<usize, ScalarProjectionFailureV3> {
    let mut bytes = 0_usize;
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
        if base_value.as_record().is_none() {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        }
        if segment.name().is_empty() || binding.canonical_name().is_empty() {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        }
        for expression in [expr, base_expr] {
            if expression.key().source_id != read.source_id_v3() {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            }
            bytes = bytes
                .checked_add(expression_shape_bytes(expression, false)?)
                .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
        }
    }
    Ok(bytes)
}

fn checked_expression_bytes(
    read: &ScalarSourceProjectReadV3<'_, '_>,
    key: SourceQualifiedNodeIdV2,
) -> Result<usize, ScalarProjectionFailureV3> {
    let expression = read
        .expression_v3(key)
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    if key.source_id != read.source_id_v3() || expression.key() != key {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    expression_shape_bytes(expression, true)
}

fn expression_shape_bytes(
    expr: &quanta_gqlang_v2_sema::TypedExprV2,
    has_filter: bool,
) -> Result<usize, ScalarProjectionFailureV3> {
    match expr.shape() {
        TypedExprShapeV2::Binding(binding) => {
            let ExpressionTypeStateV2::Known(value) = expr.type_state() else {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            };
            let record = value
                .as_record()
                .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
            let slots = boxed_slots_peak::<ExecutionRecordFieldV3>(record.fields().len())
                .ok_or(ScalarProjectionFailureV3::DemandOverflow)?;
            record
                .fields()
                .iter()
                .try_fold(slots, |sum, field| {
                    let nested = field_type_bytes(field.value_type(), true)?;
                    sum.checked_add(field.name().len())
                        .and_then(|value| value.checked_add(nested))
                        .ok_or(ScalarProjectionFailureV3::DemandOverflow)
                })?
                .checked_add(binding.canonical_name().len())
                .ok_or(ScalarProjectionFailureV3::DemandOverflow)
        }
        TypedExprShapeV2::Path { segments, .. } => {
            let [segment] = segments.as_slice() else {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            };
            let ExpressionTypeStateV2::Known(value) = expr.type_state() else {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            };
            if value.as_type().as_scalar().is_none() {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            }
            boxed_slots_peak::<String>(1)
                .and_then(|value| value.checked_add(segment.name().len()))
                .ok_or(ScalarProjectionFailureV3::DemandOverflow)
        }
        TypedExprShapeV2::Literal(fact) => {
            if !has_filter {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            }
            match fact {
                TypedLiteralFactV2::Value(
                    TypedLiteralValueV2::Int(_) | TypedLiteralValueV2::Bool(_),
                ) => Ok(0),
                TypedLiteralFactV2::Value(TypedLiteralValueV2::String(value)) => Ok(value.len()),
                TypedLiteralFactV2::Value(
                    TypedLiteralValueV2::Float(_) | TypedLiteralValueV2::Null,
                )
                | TypedLiteralFactV2::DeferredI128MinMagnitude => {
                    Err(ScalarProjectionFailureV3::UnsupportedShape)
                }
            }
        }
        TypedExprShapeV2::Compare { operator, .. } => {
            if !has_filter {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            }
            operator
                .symbol()
                .map(str::len)
                .ok_or(ScalarProjectionFailureV3::UnsupportedShape)
        }
        TypedExprShapeV2::Unary { .. }
        | TypedExprShapeV2::Binary { .. }
        | TypedExprShapeV2::Call { .. }
        | TypedExprShapeV2::Paren { .. }
        | TypedExprShapeV2::Record(_)
        | TypedExprShapeV2::List(_)
        | TypedExprShapeV2::BoolValue { .. }
        | TypedExprShapeV2::And { .. }
        | TypedExprShapeV2::Or { .. }
        | TypedExprShapeV2::Not { .. }
        | TypedExprShapeV2::LogicalQuantifier { .. }
        | TypedExprShapeV2::RelationExists { .. } => {
            Err(ScalarProjectionFailureV3::UnsupportedShape)
        }
    }
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
    let schema = field_bytes(reads[0].project_output_v3(), false)?;
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
    let predicate_ids = filter_expression_ids(read)?;
    let expression_count = read.expression_keys_v3().len();
    let mut project_keys = bounded::slots::<SourceQualifiedNodeIdV2>(count)?;
    let mut closure = bounded::slots::<SourceQualifiedNodeIdV2>(expression_count)?;
    for key in read.project_expression_keys_v3() {
        let expr = read
            .expression_v3(key)
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        let TypedExprShapeV2::Path { base, .. } = expr.shape() else {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        };
        project_keys.push(key);
        if read.expression_v3(*base).is_none() {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        }
    }
    closure.extend(read.expression_keys_v3());
    closure.sort_unstable();
    closure.dedup();
    if closure.len() != expression_count {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
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
        schema: bounded::scan_schema(read.scan_output_v3())?,
        predicates: Box::new([]),
        expressions: Box::new([]),
        correlations: Box::new([]),
        existential_terms: Box::new([]),
        partition_input: Box::new([]),
        children: Box::new([]),
    };
    let child = if let Some(predicate) = predicate_ids {
        let filter_key = read
            .filter_key_v3()
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        let filter_output = read
            .filter_output_v3()
            .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
        let mut predicates = bounded::slots::<SourceQualifiedNodeIdV2>(1)?;
        predicates.push(predicate[0]);
        let mut filter_children = bounded::slots::<ExecutionRelationalOperatorV3>(1)?;
        filter_children.push(scan);
        ExecutionRelationalOperatorV3 {
            family: quanta_gqlang_v2_algebra::AlgebraFamilyV2::Filter,
            join_mode: None,
            limit: None,
            stage: filter_key,
            source: None,
            schema: bounded::scan_schema(filter_output)?,
            predicates: predicates.into_boxed_slice(),
            expressions: Box::new([]),
            correlations: Box::new([]),
            existential_terms: Box::new([]),
            partition_input: Box::new([]),
            children: filter_children.into_boxed_slice(),
        }
    } else {
        scan
    };
    let mut children = bounded::slots::<ExecutionRelationalOperatorV3>(1)?;
    children.push(child);
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
