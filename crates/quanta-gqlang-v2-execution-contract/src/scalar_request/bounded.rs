//! Fallible, capacity-checked births for the finite scalar request projection.

use super::ScalarProjectionFailureV3;
use crate::{
    project_cardinality, project_value_type, ExecutionBindingV3, ExecutionExpressionShapeV3,
    ExecutionExpressionV3, ExecutionFieldV3, ExecutionRecordFieldV3, ExecutionRelationAuthorityV3,
    ExecutionSchemaV3, ExecutionSourceKindV3, ExecutionSourceV3, ExecutionValueTypeV3,
    RelationAuthorityV2,
};
use quanta_gqlang_v2_sema::{
    ExpressionTypeStateV2, RelationOutputContractV2, ResolvedRelationSourceKindV2,
    ResolvedValueBindingV2, TypedExprShapeV2, TypedExprV2, ValueTypeV1,
};

/// The demand charges `count` slots. Reject an allocator's larger capacity
/// before writing any element into an uncharged slot.
pub(crate) fn slots<T>(count: usize) -> Result<Vec<T>, ScalarProjectionFailureV3> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_error| ScalarProjectionFailureV3::Allocation)?;
    if std::mem::size_of::<T>() != 0 && values.capacity() > count {
        return Err(ScalarProjectionFailureV3::DemandOverflow);
    }
    Ok(values)
}

fn string(value: &str) -> Result<String, ScalarProjectionFailureV3> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_error| ScalarProjectionFailureV3::Allocation)?;
    if copied.capacity() > value.len() {
        return Err(ScalarProjectionFailureV3::DemandOverflow);
    }
    copied.push_str(value);
    Ok(copied)
}

fn scalar_value(value: &ValueTypeV1) -> Result<ExecutionValueTypeV3, ScalarProjectionFailureV3> {
    if value.as_type().as_scalar().is_none() && !value.is_null() {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    // The generic projector has no nested allocation for these two variants.
    Ok(project_value_type(value))
}

fn record_value(value: &ValueTypeV1) -> Result<ExecutionValueTypeV3, ScalarProjectionFailureV3> {
    let record = value
        .as_record()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let mut fields = slots::<ExecutionRecordFieldV3>(record.fields().len())?;
    for field in record.fields() {
        fields.push(ExecutionRecordFieldV3 {
            name: string(field.name())?,
            value_type: scalar_value(field.value_type())?,
        });
    }
    Ok(ExecutionValueTypeV3::Record(fields.into_boxed_slice()))
}

fn binding(
    value: &ResolvedValueBindingV2,
) -> Result<ExecutionBindingV3, ScalarProjectionFailureV3> {
    Ok(ExecutionBindingV3 {
        source_id: value.source_id(),
        owner_node: value.target_node().raw(),
        ordinal: value.ordinal(),
        name: string(value.canonical_name())?,
    })
}

pub(super) fn expression(
    expr: &TypedExprV2,
) -> Result<ExecutionExpressionV3, ScalarProjectionFailureV3> {
    let ExpressionTypeStateV2::Known(value) = expr.type_state() else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let (value_type, shape) = match expr.shape() {
        TypedExprShapeV2::Binding(value_binding) => (
            record_value(value)?,
            ExecutionExpressionShapeV3::Binding(binding(value_binding)?),
        ),
        TypedExprShapeV2::Path { base, segments } => {
            let [segment] = segments.as_slice() else {
                return Err(ScalarProjectionFailureV3::UnsupportedShape);
            };
            let mut names = slots::<String>(1)?;
            names.push(string(segment.name())?);
            (
                scalar_value(value)?,
                ExecutionExpressionShapeV3::Path {
                    base: *base,
                    segments: names.into_boxed_slice(),
                },
            )
        }
        TypedExprShapeV2::Literal(_)
        | TypedExprShapeV2::Unary { .. }
        | TypedExprShapeV2::Binary { .. }
        | TypedExprShapeV2::Call { .. }
        | TypedExprShapeV2::Paren { .. }
        | TypedExprShapeV2::Record(_)
        | TypedExprShapeV2::List(_)
        | TypedExprShapeV2::BoolValue { .. }
        | TypedExprShapeV2::Compare { .. }
        | TypedExprShapeV2::And { .. }
        | TypedExprShapeV2::Or { .. }
        | TypedExprShapeV2::Not { .. }
        | TypedExprShapeV2::LogicalQuantifier { .. }
        | TypedExprShapeV2::RelationExists { .. } => {
            return Err(ScalarProjectionFailureV3::UnsupportedShape);
        }
    };
    Ok(ExecutionExpressionV3 {
        id: expr.key(),
        value_type,
        shape,
    })
}

pub(super) fn schema(
    output: &RelationOutputContractV2,
) -> Result<ExecutionSchemaV3, ScalarProjectionFailureV3> {
    if !output.partition_keys().is_empty() || !output.ordering().is_empty() {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let mut fields = slots::<ExecutionFieldV3>(output.row().fields().len())?;
    for field in output.row().fields() {
        fields.push(ExecutionFieldV3 {
            name: string(field.name())?,
            value_type: scalar_value(field.value_type())?,
        });
    }
    Ok(ExecutionSchemaV3 {
        fields: fields.into_boxed_slice(),
        cardinality: project_cardinality(output.cardinality()),
        partition_keys: Box::new([]),
        order: Box::new([]),
    })
}

pub(super) fn source(
    source: &ResolvedRelationSourceKindV2,
) -> Result<ExecutionSourceV3, ScalarProjectionFailureV3> {
    let ResolvedRelationSourceKindV2::Relation(id) = source else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    Ok(ExecutionSourceV3 {
        authority: match id.authority() {
            RelationAuthorityV2::Host => ExecutionRelationAuthorityV3::Host,
            RelationAuthorityV2::CanonicalWorkspaceFiles => {
                ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
            }
        },
        kind: ExecutionSourceKindV3::Relation,
        namespace: Some(string(id.namespace().as_str())?),
        name: string(id.qualified_name())?,
        host_binding_identity: id.host_binding_identity(),
        composition_binding: None,
    })
}
