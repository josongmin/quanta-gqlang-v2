//! Finite Source and optional Filter body demand before materialization.

use super::{SourceBodyMaterializationRequestErrorV3, ValueExprKindV1};

pub(super) fn scalar_source_body_demand_v3(
    project: &quanta_gqlang_v2_hir::ProjectionV1,
    filter: Option<&quanta_gqlang_v2_hir::RelationFilterV1>,
    host: &quanta_gqlang_v2_types::RelationSourceV2,
    alias_len: usize,
    authority: &super::super::ClosedFrontendAuthorityV2,
    source_len: usize,
) -> Result<usize, SourceBodyMaterializationRequestErrorV3> {
    let steps = project
        .items()
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(if filter.is_some() { 8 } else { 3 }))
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let max_steps = usize::try_from(authority.budget().max_semantic_steps())
        .map_err(|_error| SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    if steps > max_steps {
        return Err(SourceBodyMaterializationRequestErrorV3::SemanticStepLimit);
    }
    let schema = host.relation_type().row().fields();
    let schema_names = schema
        .iter()
        .try_fold(0_usize, |n, field| n.checked_add(field.name().len()))
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let projection_names = project
        .items()
        .iter()
        .try_fold(0_usize, |n, item| {
            let name = item.alias.as_ref().map_or_else(
                || match &item.value.kind {
                    ValueExprKindV1::Path(path) => path
                        .segments
                        .as_slice()
                        .first()
                        .map_or("", |field| field.name.as_str()),
                    ValueExprKindV1::Literal(_)
                    | ValueExprKindV1::Symbol(_)
                    | ValueExprKindV1::Unary(_)
                    | ValueExprKindV1::Binary(_)
                    | ValueExprKindV1::Call(_)
                    | ValueExprKindV1::Paren(_)
                    | ValueExprKindV1::Record(_)
                    | ValueExprKindV1::List(_) => "",
                },
                |alias| alias.name.as_str(),
            );
            n.checked_add(name.len())
        })
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let path_names = project
        .items()
        .iter()
        .try_fold(0_usize, |n, item| {
            let ValueExprKindV1::Path(path) = &item.value.kind else {
                return None;
            };
            let [field] = path.segments.as_slice() else {
                return None;
            };
            n.checked_add(field.name.len())
        })
        .ok_or(SourceBodyMaterializationRequestErrorV3::UnsupportedShape)?;
    let host_id = host
        .id()
        .namespace()
        .as_str()
        .len()
        .checked_add(host.id().qualified_name().len())
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let filter_names = if let Some(filter) = filter {
        let quanta_gqlang_v2_hir::BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
            return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
        };
        let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
            return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
        };
        let [field] = path.segments.as_slice() else {
            return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
        };
        let literal_bytes = match &compare.rhs.kind {
            ValueExprKindV1::Literal(quanta_gqlang_v2_hir::LiteralV1::Str(value)) => value.len(),
            ValueExprKindV1::Literal(
                quanta_gqlang_v2_hir::LiteralV1::Int(_) | quanta_gqlang_v2_hir::LiteralV1::Bool(_),
            ) => 0,
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Symbol(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Call(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {
                return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape)
            }
        };
        Some((field.name.len(), literal_bytes))
    } else {
        None
    };
    let bytes = super::scalar_projection_body_storage_upper_v3(
        schema.len(),
        schema_names,
        project.items().len(),
        projection_names,
        path_names,
        alias_len,
        host_id,
        source_len,
        filter_names,
    )
    .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    Ok(bytes)
}
