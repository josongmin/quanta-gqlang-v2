//! Finite, request-owned scalar projection typing without tree-map births.

use super::{
    BTreeMap, CheckedHirDocumentV2, ExpressionEffectsV2, ExpressionTypeKernelV2,
    ExpressionTypeStateV2, ExtendedSourceRegistryViewV2, LocatedSpanV2, RecordTypeV1,
    SourceQualifiedNodeIdV2, TypeDiagnosticV2, TypedExpressionFactV2, TypedExpressionFactsV2,
    ValueExprKindV1, ValueTypeV1,
};
use crate::ty::{RecordFieldTypeV1, RecordFieldV1, ScalarRecordErrorV3};
use quanta_gqlang_v2_hir::{
    BoolExprKindV1, CatalogPayloadV1, FrontendRequestMaterializationRootV3,
    FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3, LiteralV1, ProgramItemHirV1,
    ProjectionV1, QueryHirV1, RelationExprKindV1, RelationFilterV1, RelationSourceV1, SourceIdV2,
    ViewBodyV1,
};
use std::mem::size_of;

#[path = "scalar_request/composition.rs"]
mod composition;
pub use composition::{
    check_one_let_composition_with_request_owner_v3, one_let_composition_type_storage_upper_v3,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ScalarProjectionRequestErrorV3 {
    UnsupportedShape,
    Storage,
    Diagnostic(TypeDiagnosticV2),
}

/// Exact Type facts and both original request debits. Raw request facts cannot
/// be detached through this API.
///
/// ```compile_fail
/// use quanta_gqlang_v2_types::AdmittedScalarProjectionFactsV3;
/// fn detach(facts: &AdmittedScalarProjectionFactsV3) {
///     let _raw = &facts.facts;
/// }
/// ```
///
/// ```compile_fail
/// use quanta_gqlang_v2_types::{AdmittedBuiltinRegistryV3, ExpressionTypeKernelV2};
/// fn bypass(builtins: &AdmittedBuiltinRegistryV3) {
///     let _kernel = ExpressionTypeKernelV2::for_admitted_scalar_projection_v3(builtins);
/// }
/// ```
pub struct AdmittedScalarProjectionFactsV3 {
    facts: TypedExpressionFactsV2,
    builtins: crate::AdmittedBuiltinRegistryV3,
    storage: FrontendRetainedStoragePermitV3,
}

impl AdmittedScalarProjectionFactsV3 {
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.storage.validate_owner_v3(root, source_id, source)?;
        self.builtins.validate_owner_v3(root, source_id, source)
    }
}

pub trait TypeFactsReaderV3 {
    fn get(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExpressionFactV2>;
    fn len(&self) -> usize;
    fn relation_row(&self, root: SourceQualifiedNodeIdV2) -> Option<&RecordTypeV1>;
    fn relation_row_count(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl TypeFactsReaderV3 for TypedExpressionFactsV2 {
    fn get(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExpressionFactV2> {
        self.get(key)
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn relation_row(&self, root: SourceQualifiedNodeIdV2) -> Option<&RecordTypeV1> {
        self.relation_row(root)
    }
    fn relation_row_count(&self) -> usize {
        self.relation_row_count()
    }
}

impl TypeFactsReaderV3 for AdmittedScalarProjectionFactsV3 {
    fn get(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExpressionFactV2> {
        self.facts.get(key)
    }
    fn len(&self) -> usize {
        self.facts.len()
    }
    fn relation_row(&self, root: SourceQualifiedNodeIdV2) -> Option<&RecordTypeV1> {
        self.facts.relation_row(root)
    }
    fn relation_row_count(&self) -> usize {
        self.facts.relation_row_count()
    }
}

pub enum ScalarProjectionCheckRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    UnsupportedShape,
    DemandOverflow,
    FallibleAllocation,
    Diagnostic {
        diagnostics: Vec<TypeDiagnosticV2>,
        _storage: FrontendRetainedStoragePermitV3,
    },
}

impl std::fmt::Debug for ScalarProjectionCheckRequestErrorV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(error) => formatter.debug_tuple("Storage").field(error).finish(),
            Self::UnsupportedShape => formatter.write_str("UnsupportedShape"),
            Self::DemandOverflow => formatter.write_str("DemandOverflow"),
            Self::FallibleAllocation => formatter.write_str("FallibleAllocation"),
            Self::Diagnostic { diagnostics, .. } => formatter
                .debug_tuple("Diagnostic")
                .field(diagnostics)
                .finish(),
        }
    }
}

/// Recompute exact finite Type demand from checked HIR and the borrowed row,
/// validate both debit owners, then run the private Vec checker.
pub fn check_scalar_projection_with_request_owner_v3(
    checked: &CheckedHirDocumentV2<'_>,
    extended: &ExtendedSourceRegistryViewV2,
    source_row: &RecordTypeV1,
    builtins: crate::AdmittedBuiltinRegistryV3,
    storage: FrontendRetainedStoragePermitV3,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source_id: SourceIdV2,
    source: &str,
) -> Result<AdmittedScalarProjectionFactsV3, ScalarProjectionCheckRequestErrorV3> {
    storage
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarProjectionCheckRequestErrorV3::Storage)?;
    builtins
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarProjectionCheckRequestErrorV3::Storage)?;
    let required = scalar_projection_checked_demand_v3(checked, source_row, source_id, source)?;
    storage
        .validate_minimum_reservation_v3(root, source_id, source, required)
        .map_err(ScalarProjectionCheckRequestErrorV3::Storage)?;
    let mut diagnostics = Vec::new();
    diagnostics
        .try_reserve_exact(1)
        .map_err(|_allocation| ScalarProjectionCheckRequestErrorV3::FallibleAllocation)?;
    if diagnostics.capacity() > 1 {
        return Err(ScalarProjectionCheckRequestErrorV3::FallibleAllocation);
    }
    #[cfg(test)]
    crate::ty::request_storage_observer::note_vec(&diagnostics);
    let kernel = ExpressionTypeKernelV2::for_admitted_scalar_projection_v3(&builtins);
    let facts = match kernel.check_admitted_scalar_projection_v3(checked, extended, source_row) {
        Ok(facts) => facts,
        Err(ScalarProjectionRequestErrorV3::UnsupportedShape) => {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        }
        Err(ScalarProjectionRequestErrorV3::Storage) => {
            return Err(ScalarProjectionCheckRequestErrorV3::FallibleAllocation);
        }
        Err(ScalarProjectionRequestErrorV3::Diagnostic(diagnostic)) => {
            diagnostics.push(diagnostic);
            return Err(ScalarProjectionCheckRequestErrorV3::Diagnostic {
                diagnostics,
                _storage: storage,
            });
        }
    };
    Ok(AdmittedScalarProjectionFactsV3 {
        facts,
        builtins,
        storage,
    })
}

fn scalar_projection_checked_demand_v3(
    checked: &CheckedHirDocumentV2<'_>,
    source_row: &RecordTypeV1,
    source_id: SourceIdV2,
    source: &str,
) -> Result<usize, ScalarProjectionCheckRequestErrorV3> {
    let syntax = checked.document();
    if syntax.source_id != source_id || syntax.source_len != source.len() {
        return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
    }
    let query = finite_scalar_query_v3(checked)
        .ok_or(ScalarProjectionCheckRequestErrorV3::UnsupportedShape)?;
    if query.result_shape.is_some() {
        return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
    }
    let (project, scan, filter) = request_project_parts_v3(query)
        .ok_or(ScalarProjectionCheckRequestErrorV3::UnsupportedShape)?;
    let Some(alias) = &scan.alias else {
        return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
    };
    let schema = source_row.fields();
    if schema.is_empty()
        || project.items().is_empty()
        || schema.iter().any(|field| {
            let value = field.value_type();
            value.as_type().as_scalar().is_none()
                && !value
                    .as_optional()
                    .is_some_and(|optional| optional.inner() == &ValueTypeV1::string())
                && !value.as_collection().is_some_and(|collection| {
                    collection.cardinality() == crate::CardinalityV1::Sequence
                        && collection.element() == &ValueTypeV1::string()
                })
        })
    {
        return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
    }
    let schema_names = schema
        .iter()
        .try_fold(0usize, |total, field| total.checked_add(field.name().len()))
        .ok_or(ScalarProjectionCheckRequestErrorV3::DemandOverflow)?;
    let mut projection_names = 0usize;
    for item in project.items() {
        let ValueExprKindV1::Path(path) = &item.value.kind else {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        };
        let [segment] = path.segments.as_slice() else {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        };
        let ValueExprKindV1::Symbol(base) = &path.base.kind else {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        };
        if base.name != alias.name || source_row.field(&segment.name).is_none() {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        }
        let name = item
            .alias
            .as_ref()
            .map_or(segment.name.as_str(), |alias| alias.name.as_str());
        projection_names = projection_names
            .checked_add(name.len())
            .ok_or(ScalarProjectionCheckRequestErrorV3::DemandOverflow)?;
    }
    let base = scalar_projection_type_storage_upper_v3(
        source.len(),
        schema.len(),
        schema_names,
        project.items().len(),
        projection_names,
    )
    .ok_or(ScalarProjectionCheckRequestErrorV3::DemandOverflow)?;
    if let Some(filter) = filter {
        let BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        };
        let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        };
        let [field] = path.segments.as_slice() else {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        };
        let field_type = source_row
            .field(&field.name)
            .ok_or(ScalarProjectionCheckRequestErrorV3::UnsupportedShape)?
            .value_type();
        let (literal_type, literal_bytes) = match &compare.rhs.kind {
            ValueExprKindV1::Literal(LiteralV1::Int(text))
                if !text.is_empty()
                    && text.bytes().all(|byte| byte.is_ascii_digit())
                    && text.parse::<i128>().is_ok() =>
            {
                (ValueTypeV1::int(), 0)
            }
            ValueExprKindV1::Literal(LiteralV1::Str(text)) => (ValueTypeV1::string(), text.len()),
            ValueExprKindV1::Literal(LiteralV1::Bool(_)) => (ValueTypeV1::bool(), 0),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Symbol(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Call(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {
                return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape)
            }
        };
        if crate::op::scalar_comparison_overload_v3(&compare.op.symbol, field_type, &literal_type)
            .is_none()
        {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        }
        scalar_filter_type_storage_upper_v3(base, project.items().len(), literal_bytes)
            .ok_or(ScalarProjectionCheckRequestErrorV3::DemandOverflow)
    } else {
        Ok(base)
    }
}

fn request_project_parts_v3(
    query: &QueryHirV1,
) -> Option<(&ProjectionV1, &RelationSourceV1, Option<&RelationFilterV1>)> {
    let RelationExprKindV1::Project(project) = &query.body.kind else {
        return None;
    };
    match &project.input.kind {
        RelationExprKindV1::Source(scan) => Some((project, scan, None)),
        RelationExprKindV1::Filter(filter) => {
            let RelationExprKindV1::Source(scan) = &filter.input.kind else {
                return None;
            };
            let BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
                return None;
            };
            let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
                return None;
            };
            let [field] = path.segments.as_slice() else {
                return None;
            };
            let ValueExprKindV1::Symbol(base) = &path.base.kind else {
                return None;
            };
            if !matches!(
                &compare.rhs.kind,
                ValueExprKindV1::Literal(
                    LiteralV1::Int(_) | LiteralV1::Str(_) | LiteralV1::Bool(_)
                )
            ) {
                return None;
            }
            if scan
                .alias
                .as_ref()
                .map_or(true, |alias| alias.name != base.name)
                || field.name.is_empty()
            {
                return None;
            }
            Some((project, scan, Some(filter)))
        }
        RelationExprKindV1::Project(_)
        | RelationExprKindV1::Join(_)
        | RelationExprKindV1::GroupBy(_)
        | RelationExprKindV1::OrderBy(_)
        | RelationExprKindV1::Limit(_) => None,
    }
}

/// Extra simultaneous Vec facts/rows for one checked Filter. The existing
/// scalar upper still accounts for the projected fields and source row.
#[must_use]
pub fn scalar_filter_type_storage_upper_v3(
    base: usize,
    projection_fields: usize,
    literal_bytes: usize,
) -> Option<usize> {
    let width = |count: usize, slot: usize| count.checked_next_power_of_two()?.checked_mul(slot);
    let base_facts = width(
        projection_fields.checked_mul(2)?,
        size_of::<(SourceQualifiedNodeIdV2, TypedExpressionFactV2)>(),
    )?;
    let filtered_facts = width(
        projection_fields.checked_mul(2)?.checked_add(4)?,
        size_of::<(SourceQualifiedNodeIdV2, TypedExpressionFactV2)>(),
    )?;
    let row_slot = size_of::<(SourceQualifiedNodeIdV2, RecordTypeV1)>();
    filtered_facts
        .checked_sub(base_facts)?
        .checked_add(width(3, row_slot)?.checked_sub(width(2, row_slot)?)?)
        .and_then(|extra| extra.checked_add(literal_bytes))
        .and_then(|extra| base.checked_add(extra))
}

fn finite_scalar_query_v3<'a>(checked: &CheckedHirDocumentV2<'a>) -> Option<&'a QueryHirV1> {
    match checked.document().items.as_slice() {
        [ProgramItemHirV1::Query(query)] => Some(query),
        [ProgramItemHirV1::CatalogDecl(view)] => {
            let CatalogPayloadV1::View(ViewBodyV1::Inline(query)) = &view.payload else {
                return None;
            };
            Some(query)
        }
        _ => None,
    }
}

/// Upper bound for the finite request path's simultaneous Vec and Arc births.
/// No private standard-library tree layout enters the contract.
#[must_use]
pub fn scalar_projection_type_storage_upper_v3(
    _source_bytes: usize,
    schema_fields: usize,
    _schema_name_bytes: usize,
    projection_fields: usize,
    projection_name_bytes: usize,
) -> Option<usize> {
    let width = |count: usize, slot: usize| count.checked_next_power_of_two()?.checked_mul(slot);
    let facts = width(
        projection_fields.checked_mul(2)?,
        size_of::<(SourceQualifiedNodeIdV2, TypedExpressionFactV2)>(),
    )?;
    let rows = width(2, size_of::<(SourceQualifiedNodeIdV2, RecordTypeV1)>())?;
    let input = width(projection_fields, size_of::<RecordFieldTypeV1>())?;
    let output = width(projection_fields, size_of::<RecordFieldV1>())?;
    let duplicates = width(projection_fields, size_of::<(usize, &RecordFieldTypeV1)>())?;
    let schema_index = width(schema_fields, size_of::<&RecordFieldV1>())?;
    let arc_row = projection_fields
        .checked_mul(size_of::<RecordFieldV1>())?
        .checked_add(size_of::<[usize; 2]>())?
        .checked_add(std::mem::align_of::<RecordFieldV1>())?;
    facts
        .checked_add(rows)?
        .checked_add(input)?
        .checked_add(output)?
        .checked_add(duplicates)?
        .checked_add(schema_index)?
        .checked_add(arc_row)?
        .checked_add(projection_name_bytes.checked_mul(2)?)?
        .checked_add(size_of::<TypeDiagnosticV2>())
}

impl ExpressionTypeKernelV2 {
    /// Type the admitted single inline View Source→Project subset. The caller
    /// retains its request permit through the returned facts and any diagnostic.
    pub(super) fn check_admitted_scalar_projection_v3(
        &self,
        checked: &CheckedHirDocumentV2<'_>,
        extended: &ExtendedSourceRegistryViewV2,
        source_row: &RecordTypeV1,
    ) -> Result<TypedExpressionFactsV2, ScalarProjectionRequestErrorV3> {
        if !matches!(self.builtins, super::KernelBuiltinsV2::Shared(_)) {
            return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
        }
        let syntax = checked.document();
        let query = finite_scalar_query_v3(checked)
            .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
        if query.result_shape.is_some() {
            return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
        }
        let (project, scan, filter) = request_project_parts_v3(query)
            .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
        let Some(alias) = &scan.alias else {
            return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
        };
        let items = project.items();
        if items.is_empty() || source_row.fields().is_empty() {
            return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
        }
        let mut source_index = Vec::new();
        reserve(&mut source_index, source_row.fields().len())?;
        source_index.extend(source_row.fields().iter());
        source_index.sort_unstable_by(|left: &&RecordFieldV1, right| left.name().cmp(right.name()));
        let mut facts = Vec::new();
        reserve(
            &mut facts,
            items
                .len()
                .checked_mul(2)
                .and_then(|count| count.checked_add(if filter.is_some() { 4 } else { 0 }))
                .ok_or(ScalarProjectionRequestErrorV3::Storage)?,
        )?;
        let mut output_fields = Vec::new();
        reserve(&mut output_fields, items.len())?;
        if let Some(filter) = filter {
            let BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            let [segment] = path.segments.as_slice() else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Symbol(base) = &path.base.kind else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            if scan
                .alias
                .as_ref()
                .map_or(true, |alias| alias.name != base.name)
            {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            }
            let field = source_row
                .field(&segment.name)
                .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
            let (literal_type, literal) = match &compare.rhs.kind {
                ValueExprKindV1::Literal(LiteralV1::Int(text)) => {
                    let value = text
                        .parse::<i128>()
                        .map_err(|_parse_error| ScalarProjectionRequestErrorV3::UnsupportedShape)?;
                    (ValueTypeV1::int(), crate::TypedLiteralValueV2::Int(value))
                }
                ValueExprKindV1::Literal(LiteralV1::Str(text)) => {
                    let mut copied = String::new();
                    copied
                        .try_reserve_exact(text.len())
                        .map_err(|_allocation| ScalarProjectionRequestErrorV3::Storage)?;
                    if copied.capacity() > text.len() {
                        return Err(ScalarProjectionRequestErrorV3::Storage);
                    }
                    copied.push_str(text);
                    #[cfg(test)]
                    crate::ty::request_storage_observer::note_string(&copied);
                    (
                        ValueTypeV1::string(),
                        crate::TypedLiteralValueV2::String(copied),
                    )
                }
                ValueExprKindV1::Literal(LiteralV1::Bool(value)) => (
                    ValueTypeV1::bool(),
                    crate::TypedLiteralValueV2::Bool(*value),
                ),
                ValueExprKindV1::Literal(_)
                | ValueExprKindV1::Symbol(_)
                | ValueExprKindV1::Path(_)
                | ValueExprKindV1::Unary(_)
                | ValueExprKindV1::Binary(_)
                | ValueExprKindV1::Call(_)
                | ValueExprKindV1::Paren(_)
                | ValueExprKindV1::Record(_)
                | ValueExprKindV1::List(_) => {
                    return Err(ScalarProjectionRequestErrorV3::UnsupportedShape)
                }
            };
            let overload = crate::op::scalar_comparison_overload_v3(
                &compare.op.symbol,
                field.value_type(),
                &literal_type,
            )
            .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
            let base_type = ValueTypeV1::try_record(source_row.clone())
                .map_err(ScalarProjectionRequestErrorV3::Diagnostic)?;
            let pure = ExpressionEffectsV2::pure_scalar();
            facts.push((key(syntax.source_id, path.base.id), known(base_type, pure)));
            facts.push((
                key(syntax.source_id, compare.lhs.id),
                known(field.value_type().clone(), pure),
            ));
            let mut literal_fact = known(literal_type, pure);
            literal_fact.literal = Some(crate::TypedLiteralFactV2::Value(literal));
            facts.push((key(syntax.source_id, compare.rhs.id), literal_fact));
            let mut predicate_fact = known(ValueTypeV1::bool(), overload.effects);
            predicate_fact.operator_id = Some(overload.id);
            facts.push((key(syntax.source_id, filter.predicate.id), predicate_fact));
        }
        for item in items {
            let ValueExprKindV1::Path(path) = &item.value.kind else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            let [segment] = path.segments.as_slice() else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Symbol(base) = &path.base.kind else {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            };
            if base.name != alias.name {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            }
            let field_index = source_index
                .binary_search_by(|field| field.name().cmp(&segment.name))
                .map_err(|_missing| ScalarProjectionRequestErrorV3::UnsupportedShape)?;
            let field = source_index
                .get(field_index)
                .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
            if field.value_type().as_type().as_scalar().is_none() {
                return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
            }
            let output_name = item
                .alias
                .as_ref()
                .map_or(segment.name.as_str(), |alias| alias.name.as_str());
            let name_span = item
                .alias
                .as_ref()
                .map_or(item.value.span, |alias| alias.span);
            let location = extended
                .combined()
                .try_check_location(LocatedSpanV2 {
                    source_id: syntax.source_id,
                    span: name_span,
                })
                .map_err(|_source_error| ScalarProjectionRequestErrorV3::UnsupportedShape)?;
            output_fields.push(
                RecordFieldTypeV1::try_new(output_name, field.value_type().clone(), location)
                    .map_err(ScalarProjectionRequestErrorV3::Diagnostic)?,
            );
            let base_type = ValueTypeV1::try_record(source_row.clone())
                .map_err(ScalarProjectionRequestErrorV3::Diagnostic)?;
            let pure = ExpressionEffectsV2::pure_scalar();
            facts.push((key(syntax.source_id, path.base.id), known(base_type, pure)));
            facts.push((
                key(syntax.source_id, item.value.id),
                known(field.value_type().clone(), pure),
            ));
        }
        let project_row = RecordTypeV1::try_new_scalar_request_v3(output_fields).map_err(
            |error| match error {
                ScalarRecordErrorV3::Storage => ScalarProjectionRequestErrorV3::Storage,
                ScalarRecordErrorV3::Diagnostic(diagnostic) => {
                    ScalarProjectionRequestErrorV3::Diagnostic(diagnostic)
                }
            },
        )?;
        facts.sort_unstable_by_key(|entry| entry.0);
        let source_key = key(
            syntax.source_id,
            filter.map_or(project.input.id, |filter| filter.input.id),
        );
        let project_key = key(syntax.source_id, query.body.id);
        let mut rows = Vec::new();
        reserve(&mut rows, if filter.is_some() { 3 } else { 2 })?;
        rows.push((source_key, source_row.clone()));
        if filter.is_some() {
            rows.push((key(syntax.source_id, project.input.id), source_row.clone()));
        }
        rows.push((project_key, project_row));
        rows.sort_unstable_by_key(|entry| entry.0);
        Ok(TypedExpressionFactsV2 {
            facts: BTreeMap::new(),
            relation_rows: BTreeMap::new(),
            request_facts: Some(facts),
            request_rows: Some(rows),
        })
    }
}

fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<(), ScalarProjectionRequestErrorV3> {
    values
        .try_reserve_exact(count)
        .map_err(|_allocation| ScalarProjectionRequestErrorV3::Storage)?;
    if values.capacity()
        > count
            .checked_next_power_of_two()
            .ok_or(ScalarProjectionRequestErrorV3::Storage)?
    {
        return Err(ScalarProjectionRequestErrorV3::Storage);
    }
    #[cfg(test)]
    crate::ty::request_storage_observer::note_vec(values);
    Ok(())
}

fn key(
    source_id: quanta_gqlang_v2_hir::SourceIdV2,
    node_id: quanta_gqlang_v2_hir::HirNodeIdV1,
) -> SourceQualifiedNodeIdV2 {
    SourceQualifiedNodeIdV2 { source_id, node_id }
}

fn known(value_type: ValueTypeV1, effects: ExpressionEffectsV2) -> TypedExpressionFactV2 {
    TypedExpressionFactV2 {
        state: ExpressionTypeStateV2::Known(value_type),
        operator_id: None,
        resolved_call: None,
        intrinsic_call_aggregation: None,
        effects,
        literal: None,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "bounded allocation witness fixtures")]
mod storage_tests {
    use super::*;
    use crate::BuiltinRegistryV2;
    use quanta_gqlang_v2_hir::{
        AliasV1, CheckedHirDocumentOwnedV2, FrontendRequestMaterializationRootV3,
        FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
        HirBuilderV1, HirDocumentV1, PathSegmentV1, ProjectionItemV1, SourceIdV2,
        SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
    };
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    struct Ledger(AtomicU64);
    struct CountingSink(Arc<Ledger>);
    struct Charge {
        ledger: Arc<Ledger>,
        bytes: u64,
    }
    impl Drop for Charge {
        fn drop(&mut self) {
            self.ledger.0.fetch_sub(self.bytes, Ordering::SeqCst);
        }
    }
    impl FrontendRequestStorageSinkV3 for CountingSink {
        fn reserve_v3(
            &self,
            logical_bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            self.0 .0.fetch_add(logical_bytes, Ordering::SeqCst);
            Ok(Box::new(Charge {
                ledger: self.0.clone(),
                bytes: logical_bytes,
            }))
        }
        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            permit
                .as_any_v3()
                .downcast_ref::<Charge>()
                .is_some_and(|charge| Arc::ptr_eq(&charge.ledger, &self.0))
        }
    }

    #[test]
    fn builtin_clone_retains_original_debit_after_admitted_registry_drops() {
        let source = "source";
        let source_id = SourceIdV2::from_raw(41);
        let ledger = Arc::new(Ledger(AtomicU64::new(0)));
        let sink = CountingSink(ledger.clone());
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink);
        let parent = root.reserve_v3(source_id, source, 1).unwrap();
        let builtins =
            BuiltinRegistryV2::try_closed_with_request_owner_v3(&root, source_id, source, &parent)
                .unwrap();
        let kernel = ExpressionTypeKernelV2::for_admitted_scalar_projection_v3(&builtins);
        let survivor = kernel.clone();
        drop(builtins);
        drop(parent);
        drop(kernel);
        assert!(ledger.0.load(Ordering::SeqCst) > 0);
        drop(survivor);
        assert_eq!(ledger.0.load(Ordering::SeqCst), 0);
    }

    struct Sink;
    struct Permit;
    impl FrontendRequestStorageSinkV3 for Sink {
        fn reserve_v3(
            &self,
            _logical_bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            Ok(Box::new(Permit))
        }

        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            permit.as_any_v3().is::<Permit>()
        }
    }

    #[test]
    fn actual_scalar_birth_capacities_fit_upper_on_success_and_diagnostic() {
        for (count, duplicate) in [(1, false), (9, false), (4, true)] {
            check_birth_bound(count, duplicate);
        }
    }

    fn check_birth_bound(count: usize, duplicate: bool) {
        let source = "x".repeat(512);
        let source_id = SourceIdV2::from_raw(121);
        let span = SpanV1::new(0, 100);
        let builder = HirBuilderV1::new();
        let scan = builder.relation_source(
            "records",
            SpanV1::new(0, 7),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(8, 9),
            }),
            span,
        );
        let mut projection = Vec::new();
        let mut names = Vec::new();
        for ordinal in 0..count {
            let base = builder.symbol("r", SpanV1::new(10, 11));
            let value = builder
                .try_path(
                    base,
                    vec![PathSegmentV1 {
                        name: format!("f{ordinal}"),
                        span: SpanV1::new(12, 14),
                    }],
                    SpanV1::new(10, 14),
                )
                .unwrap();
            let name = if duplicate && ordinal == 2 {
                "o1".to_owned()
            } else {
                format!("o{ordinal}")
            };
            names.push(name.clone());
            projection.push(ProjectionItemV1 {
                value,
                alias: Some(AliasV1 {
                    name,
                    span: SpanV1::new(15, 17),
                }),
            });
        }
        let project = builder.try_project(scan, projection, span).unwrap();
        let query = builder.query(project, None, span);
        let catalog = builder.catalog_decl(
            "out",
            SpanV1::new(0, 3),
            CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(query))),
            span,
        );
        let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
            source_id,
            source_len: source.len(),
            items: vec![ProgramItemHirV1::CatalogDecl(Box::new(catalog))],
        })
        .unwrap();
        let extended = SourceRegistryViewV2::try_new(Vec::new())
            .unwrap()
            .try_extend_document(SourceRegistrationV2 {
                source_id,
                source_len: source.len(),
            })
            .unwrap();
        let location = extended
            .combined()
            .try_check_location(LocatedSpanV2 {
                source_id,
                span: SpanV1::new(15, 17),
            })
            .unwrap();
        let source_fields: Vec<_> = (0..count)
            .map(|ordinal| {
                RecordFieldTypeV1::try_new(
                    &format!("f{ordinal}"),
                    ValueTypeV1::int(),
                    location.clone(),
                )
                .unwrap()
            })
            .collect();
        let row = RecordTypeV1::try_new(source_fields).unwrap();
        let output_name_bytes = names.iter().map(String::len).sum();
        let schema_name_bytes = row.fields().iter().map(|field| field.name().len()).sum();
        let upper = scalar_projection_type_storage_upper_v3(
            source.len(),
            row.fields().len(),
            schema_name_bytes,
            count,
            output_name_bytes,
        )
        .unwrap();
        let sink = Sink;
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
        let parent = root.reserve_v3(source_id, &source, 1).unwrap();
        let builtins =
            BuiltinRegistryV2::try_closed_with_request_owner_v3(&root, source_id, &source, &parent)
                .unwrap();
        let storage = root.reserve_v3(source_id, &source, upper).unwrap();

        crate::ty::request_storage_observer::begin();
        let result = check_scalar_projection_with_request_owner_v3(
            &checked.as_checked(),
            &extended,
            &row,
            builtins,
            storage,
            &root,
            source_id,
            &source,
        );
        let observed = crate::ty::request_storage_observer::finish();
        if duplicate {
            assert!(matches!(
                result,
                Err(ScalarProjectionCheckRequestErrorV3::Diagnostic { diagnostics, .. })
                    if matches!(diagnostics.as_slice(), [TypeDiagnosticV2::DuplicateRecordField { .. }])
            ));
        } else {
            assert!(result.is_ok());
        }
        assert!(observed > 0);
        assert!(
            observed <= upper,
            "count={count} duplicate={duplicate} observed={observed} upper={upper}"
        );
    }
}
