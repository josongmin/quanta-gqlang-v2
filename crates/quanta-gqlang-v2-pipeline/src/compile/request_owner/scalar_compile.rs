//! One finite bare Query request from parser storage through execution seal.

use super::{
    begin_bind_source_document_with_request_owner_v3,
    begin_prepare_source_body_with_request_owner_v3,
    begin_resolve_source_document_with_request_owner_v3,
    check_source_document_with_request_owner_v3, declare_source_document_with_request_owner_v3,
    lower_source_body_with_request_owner_v3, materialize_source_body_with_request_owner_v3,
    parse_source_document_with_request_owner_v3, plan_source_body_with_request_owner_v3,
    predeclare_source_catalog_with_request_owner_v3, project_source_body_with_request_owner_v3,
    resolve_source_bindings_with_request_owner_v3, seal_source_catalog_with_request_owner_v3,
    seal_source_execution_document_with_request_owner_v3,
    topologize_source_body_with_request_owner_v3, type_source_body_with_request_owner_v3,
    AdmittedScalarExecutionDocumentV3, AdmittedSourceExecutionDocumentV3,
    FrontendRequestCheckErrorV3, FrontendRequestPredeclareErrorV3, ParsedRequestSourceV3,
    ScalarExecutionRequestErrorV3, SourceBodyTopologyRequestErrorV3, SourceCatalogSealErrorV3,
};
use quanta_gqlang_v2_algebra::ScalarAlgebraRequestErrorV3;
use quanta_gqlang_v2_planning::ScalarPlanningRequestErrorV3;
use quanta_gqlang_v2_sema::{
    BinderDeclarationRequestErrorV3, BuiltinRegistryRequestErrorV3, ClosedFrontendAuthorityV2,
    DeclaredRequestErrorV3, DocumentValidatorRequestErrorV3, FrontendTransactionErrorV2,
    SourceBodyMaterializationRequestErrorV3, SourceBodyTypesRequestErrorV3, SourceIdV2,
};
use quanta_gqlang_v2_surface::{FrontendRequestMaterializationRootV3, SourceRootFamilyV3};

/// The first failing owner on the bounded scalar Query route.
#[derive(Debug)]
pub enum ScalarSourceCompileRequestErrorV3 {
    Parse(FrontendTransactionErrorV2),
    ParsedBudgetMismatch,
    UnsupportedRoot,
    Check(FrontendRequestCheckErrorV3),
    Declare(DeclaredRequestErrorV3),
    ModulesBegin(DocumentValidatorRequestErrorV3),
    Predeclare(FrontendRequestPredeclareErrorV3),
    Builtin(BuiltinRegistryRequestErrorV3),
    Binder(BinderDeclarationRequestErrorV3),
    Type(SourceBodyTypesRequestErrorV3),
    Topology(SourceBodyTopologyRequestErrorV3),
    Body(SourceBodyMaterializationRequestErrorV3),
    Catalog(SourceCatalogSealErrorV3),
    Algebra(ScalarAlgebraRequestErrorV3),
    Planning(ScalarPlanningRequestErrorV3),
    Execution(ScalarExecutionRequestErrorV3),
}

/// Compose the already bounded source owners for exactly one bare Query.
/// Other families fail after parser admission and before semantic work.
pub fn compile_scalar_source_with_request_owner_v3(
    source_id: SourceIdV2,
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> Result<AdmittedScalarExecutionDocumentV3, ScalarSourceCompileRequestErrorV3> {
    let parsed =
        parse_source_document_with_request_owner_v3(source_id, source, authority.budget(), root)
            .map_err(ScalarSourceCompileRequestErrorV3::Parse)?;
    compile_scalar_parsed_source_with_request_owner_v3(source_id, source, authority, root, parsed)
}

/// Consume the same parser-issued source that was inspected for route selection.
pub fn compile_scalar_parsed_source_with_request_owner_v3(
    source_id: SourceIdV2,
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    parsed: ParsedRequestSourceV3,
) -> Result<AdmittedScalarExecutionDocumentV3, ScalarSourceCompileRequestErrorV3> {
    if parsed.budget != *authority.budget() {
        return Err(ScalarSourceCompileRequestErrorV3::ParsedBudgetMismatch);
    }
    if !matches!(parsed.root_family_v3(), Ok(SourceRootFamilyV3::Relation)) {
        return Err(ScalarSourceCompileRequestErrorV3::UnsupportedRoot);
    }
    match compile_source_parsed_with_request_owner_v3(source_id, source, authority, root, parsed)? {
        AdmittedSourceExecutionDocumentV3::Scalar(value) => Ok(value),
        AdmittedSourceExecutionDocumentV3::OneLet(_) => {
            Err(ScalarSourceCompileRequestErrorV3::UnsupportedRoot)
        }
    }
}

/// One parser-issued source, one request root and one stage chain for bare
/// scalar Query or Binder-authenticated one-LET Composition.
pub fn compile_source_parsed_with_request_owner_v3(
    source_id: SourceIdV2,
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    parsed: ParsedRequestSourceV3,
) -> Result<AdmittedSourceExecutionDocumentV3, ScalarSourceCompileRequestErrorV3> {
    if parsed.budget != *authority.budget() {
        return Err(ScalarSourceCompileRequestErrorV3::ParsedBudgetMismatch);
    }
    if !matches!(
        parsed.root_family_v3(),
        Ok(SourceRootFamilyV3::Relation | SourceRootFamilyV3::Composition)
    ) {
        return Err(ScalarSourceCompileRequestErrorV3::UnsupportedRoot);
    }
    let checked = check_source_document_with_request_owner_v3(source_id, source, root, parsed)
        .map_err(ScalarSourceCompileRequestErrorV3::Check)?;
    let declared = declare_source_document_with_request_owner_v3(source, authority, root, checked)
        .map_err(ScalarSourceCompileRequestErrorV3::Declare)?;
    let resolving = begin_resolve_source_document_with_request_owner_v3(source, root, declared)
        .map_err(ScalarSourceCompileRequestErrorV3::ModulesBegin)?;
    let predeclared = predeclare_source_catalog_with_request_owner_v3(source, root, resolving)
        .map_err(ScalarSourceCompileRequestErrorV3::Predeclare)?;
    let preparing = begin_prepare_source_body_with_request_owner_v3(source, root, predeclared)
        .map_err(ScalarSourceCompileRequestErrorV3::Builtin)?;
    let declaring = begin_bind_source_document_with_request_owner_v3(source, root, preparing)
        .map_err(ScalarSourceCompileRequestErrorV3::Binder)?;
    let resolved = resolve_source_bindings_with_request_owner_v3(source, root, declaring)
        .map_err(ScalarSourceCompileRequestErrorV3::Binder)?;
    let typed = type_source_body_with_request_owner_v3(source, root, resolved)
        .map_err(ScalarSourceCompileRequestErrorV3::Type)?;
    let topology = topologize_source_body_with_request_owner_v3(source, root, typed)
        .map_err(ScalarSourceCompileRequestErrorV3::Topology)?;
    let body = materialize_source_body_with_request_owner_v3(source, root, topology)
        .map_err(ScalarSourceCompileRequestErrorV3::Body)?;
    let catalog = seal_source_catalog_with_request_owner_v3(source, root, body)
        .map_err(ScalarSourceCompileRequestErrorV3::Catalog)?;
    let algebra = lower_source_body_with_request_owner_v3(source, root, catalog)
        .map_err(ScalarSourceCompileRequestErrorV3::Algebra)?;
    let planned = plan_source_body_with_request_owner_v3(source, root, algebra)
        .map_err(ScalarSourceCompileRequestErrorV3::Planning)?;
    let projected = project_source_body_with_request_owner_v3(source, root, planned)
        .map_err(ScalarSourceCompileRequestErrorV3::Execution)?;
    seal_source_execution_document_with_request_owner_v3(source, root, projected)
        .map_err(ScalarSourceCompileRequestErrorV3::Execution)
}
