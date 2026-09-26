//! Surface diagnostics for the shared stack-safe HIR depth admission.

use crate::budget::ParseBudgetDimensionV2;
use crate::diag::ParseDiagnosticV1;
use crate::parser::SurfaceItemV1;
use quanta_gqlang_v2_hir::{
    check_depths_v2, BoolExprHirV1, HirDepthDiagnosticV2, HirDepthDimensionV2, HirRootRefV1,
    ValueExprHirV1, MAX_SAFE_RELATION_DEPTH_V2,
};

fn surface_diagnostic(error: HirDepthDiagnosticV2) -> ParseDiagnosticV1 {
    let dimension = match error.dimension {
        HirDepthDimensionV2::Expression | HirDepthDimensionV2::Shape => {
            ParseBudgetDimensionV2::ExpressionDepth
        }
        HirDepthDimensionV2::Relation => ParseBudgetDimensionV2::RelationDepth,
    };
    ParseDiagnosticV1::resource_exhausted(dimension, error.limit, error.observed, error.span)
}

pub(crate) fn check_item(
    item: &SurfaceItemV1,
    expression_limit: usize,
    relation_limit: usize,
) -> Result<(), ParseDiagnosticV1> {
    let root = match item {
        SurfaceItemV1::Query(query) => HirRootRefV1::Query(query),
        SurfaceItemV1::Decl(decl) => HirRootRefV1::Decl(decl),
        SurfaceItemV1::Rule(rule) => HirRootRefV1::Rule(rule),
        SurfaceItemV1::Program(program) => HirRootRefV1::Item(program),
    };
    check_depths_v2(root, expression_limit, relation_limit, expression_limit)
        .map_err(surface_diagnostic)
}

pub(crate) fn check_value(value: &ValueExprHirV1, limit: usize) -> Result<(), ParseDiagnosticV1> {
    check_depths_v2(
        HirRootRefV1::Value(value),
        limit,
        MAX_SAFE_RELATION_DEPTH_V2,
        limit,
    )
    .map_err(surface_diagnostic)
}

pub(crate) fn check_bool(value: &BoolExprHirV1, limit: usize) -> Result<(), ParseDiagnosticV1> {
    check_depths_v2(
        HirRootRefV1::Bool(value),
        limit,
        MAX_SAFE_RELATION_DEPTH_V2,
        limit,
    )
    .map_err(surface_diagnostic)
}
