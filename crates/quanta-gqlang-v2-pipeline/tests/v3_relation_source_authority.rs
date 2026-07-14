#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "owner-local contract rail: assertion panics are intentional"
)]
//! Owner-local relation-source authority contract.
//!
//! This rail proves the exact identity variants issued by schema, builtin, and
//! composition authorities, plus typed rejection for a closed-catalog miss.

use quanta_gqlang_v2_algebra::{
    AlgebraDiagnosticV1, AlgebraNodeV1, AlgebraOpV1, BuiltinRelationV1, RelationSourceIdentityV1,
    ScanV1,
};
use quanta_gqlang_v2_pipeline::{
    compile_program_structural, compile_with_context, CompiledItemV1, FrontendContextV1,
    PipelineDiagnosticV1, PipelineStageV1,
};
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

fn project_scan_v1(node_v1: &AlgebraNodeV1) -> &ScanV1 {
    let AlgebraOpV1::Project(project_v1) = &node_v1.op else {
        panic!("fixture must normalize to project over scan")
    };
    let AlgebraOpV1::Scan(scan_v1) = &project_v1.input.op else {
        panic!("fixture project input must remain a scan")
    };
    scan_v1
}

#[test]
fn schema_context_issues_exact_named_relation_identity_v1() {
    let schema_v1 = TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "id",
                TypeV1::int(),
            )])),
            cardinality: CardinalityV1::Bag,
        },
    );
    let program_v1 = compile_with_context(
        "from users u | project u.id as id",
        &FrontendContextV1::schema_only(schema_v1),
    )
    .expect("schema authority must issue the exact named relation identity");
    let CompiledItemV1::RelationalQuery { algebra, .. } = program_v1.compiled() else {
        panic!("fixture must compile as relational query")
    };
    assert_eq!(
        project_scan_v1(algebra).source_identity_v1,
        RelationSourceIdentityV1::Named("users".to_string())
    );
}

#[test]
fn schema_context_rejects_relation_absent_from_explicit_authority_v1() {
    let schema_v1 = TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "id",
                TypeV1::int(),
            )])),
            cardinality: CardinalityV1::Bag,
        },
    );
    let error_v1 = compile_with_context(
        "from orders o | project o.id as id",
        &FrontendContextV1::schema_only(schema_v1),
    )
    .expect_err("explicit schema authority must reject an absent relation source");

    assert_eq!(error_v1.stage(), PipelineStageV1::Algebra);
    assert_eq!(error_v1.code(), "algebra.unknown_relation_source");
    assert!(matches!(
        error_v1,
        PipelineDiagnosticV1::Algebra(AlgebraDiagnosticV1::UnknownRelationSource {
            relation,
            ..
        }) if relation == "orders"
    ));
}

#[test]
fn composition_binding_identity_is_independent_of_optional_row_schema_v1() {
    let schema_v1 = TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "id",
                TypeV1::int(),
            )])),
            cardinality: CardinalityV1::Bag,
        },
    );
    let program_v1 = compile_with_context(
        "let p = from users u | project u.id, 2 \
         from p x | project x.ghost as g with p",
        &FrontendContextV1::schema_only(schema_v1),
    )
    .expect("composition identity remains exact when producer row schema is unavailable");
    let CompiledItemV1::Composition {
        binding_outputs,
        body,
        ..
    } = program_v1.compiled()
    else {
        panic!("fixture must compile as composition")
    };
    assert_eq!(binding_outputs, &[None]);
    let CompiledItemV1::RelationalQuery { algebra, .. } = body.as_ref() else {
        panic!("fixture body must remain relational")
    };
    assert_eq!(
        project_scan_v1(algebra).source_identity_v1,
        RelationSourceIdentityV1::CompositionBinding("p".to_string())
    );
}

#[test]
fn canonical_query_context_resolves_builtin_and_composition_sources_once_v1() {
    let program_v1 = compile_with_context(
        "let p = from files f | project f.path as path \
         from p x | project x.path as path with p",
        &FrontendContextV1::canonical_query_serving_v1(),
    )
    .expect("closed canonical relation authority must compile exact composition");
    let CompiledItemV1::Composition { bindings, body, .. } = program_v1.compiled() else {
        panic!("fixture must compile as composition")
    };
    let Some(CompiledItemV1::RelationalQuery {
        algebra: producer_v1,
        ..
    }) = bindings.first()
    else {
        panic!("fixture must retain relational producer")
    };
    assert_eq!(
        project_scan_v1(producer_v1).source_identity_v1,
        RelationSourceIdentityV1::Builtin(BuiltinRelationV1::WorkspaceFiles)
    );
    let CompiledItemV1::RelationalQuery {
        algebra: body_v1, ..
    } = body.as_ref()
    else {
        panic!("fixture must retain relational body")
    };
    assert_eq!(
        project_scan_v1(body_v1).source_identity_v1,
        RelationSourceIdentityV1::CompositionBinding("p".to_string())
    );
}

#[test]
fn canonical_query_context_rejects_unknown_relation_while_structural_stays_open_v1() {
    let source_v1 = "from unknown_relation u | project u.path as path";
    let error_v1 =
        compile_with_context(source_v1, &FrontendContextV1::canonical_query_serving_v1())
            .expect_err("closed canonical authority must reject an absent relation source");
    assert_eq!(error_v1.stage(), PipelineStageV1::Algebra);
    assert_eq!(error_v1.code(), "algebra.unknown_relation_source");
    assert_eq!(
        error_v1.to_string(),
        "[algebra] algebra.unknown_relation_source: relation source `unknown_relation` is absent \
         from the closed compile authority"
    );
    assert!(matches!(
        error_v1,
        PipelineDiagnosticV1::Algebra(AlgebraDiagnosticV1::UnknownRelationSource {
            relation,
            ..
        }) if relation == "unknown_relation"
    ));

    let structural_v1 = compile_program_structural(source_v1)
        .expect("explicit structural compilation must remain schema-open");
    let CompiledItemV1::RelationalQuery { algebra, .. } = structural_v1.compiled() else {
        panic!("structural fixture must remain relational")
    };
    assert_eq!(
        project_scan_v1(algebra).source_identity_v1,
        RelationSourceIdentityV1::SchemaOpen
    );
}
