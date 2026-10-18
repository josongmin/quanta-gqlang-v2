#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "document-assembly rails assert exact sealed payloads with unwrap/expect/indexing/panic; allowed for test targets only"
)]

//! Compiled-document exact-key assembly (ticket 07, lane W4-B).
//!
//! Every rail drives the PUBLIC pipeline entry points over sealed semantic
//! documents (Sema `test_support` fixtures) or over source text plus a real
//! closed authority, then asserts the exact assembled payload:
//!
//! - one plan per applicable authored identity, one checked non-plan reason per
//!   non-executable declaration, one rule-program plan for the exact member set;
//! - missing / extra / duplicate / foreign / wrong-kind membership is rejected
//!   with the closed `pipeline.document.*` code vocabulary;
//! - the traversal budget is the SEALED document receipt, never a caller
//!   argument (exact-limit admits, limit+1 rejects with the algebra code);
//! - group/partition/order identity, CHECK nested-graph identity, rule
//!   membership, and FIND witness retention all change the assembled payload,
//!   so a hardcoded or dropped contract cannot stay green.

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{lower_relation_item, AlgebraFamilyV2};
use quanta_gqlang_v2_logic::RelationShapeV1;
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    VersionV2,
};
use quanta_gqlang_v2_pipeline::{
    compile_resolved_document_v2, compile_source_document_v3, format_source_document_v2,
    AnalysisItemV2, CompiledItemV2, DocumentAssemblyCodeV2, ExecutionAuthorityV3,
    ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionCompareMetricV3,
    ExecutionCompositionBodyV3, ExecutionCompositionProducerV3, ExecutionConfirmModeV3,
    ExecutionDetectionModeV3, ExecutionEmitModeV3, ExecutionEvidenceModeV3,
    ExecutionExplainTargetV3, ExecutionExportFormatV3, ExecutionExpressionShapeV3,
    ExecutionFamilyV3, ExecutionGraphCapabilityV3, ExecutionGraphDedupIdentityV3,
    ExecutionGraphDirectionV3, ExecutionGraphKindV3, ExecutionGraphReferenceKindV3,
    ExecutionGraphReferenceRoleV3, ExecutionGraphRelationShapeV3, ExecutionGraphTargetV3,
    ExecutionGraphWitnessTieBreakV3, ExecutionLiteralV3, ExecutionPresentationKindV3,
    ExecutionRelationAuthorityV3, ExecutionRelationV3, ExecutionRelationalOperatorV3,
    ExecutionSampleModeV3, ExecutionScalarTypeV3, ExecutionSchemaV3, ExecutionShowCategoryV3,
    ExecutionSourceKindV3, ExecutionTruthModeV3, ExecutionValueTypeV3, FrontendAuthorityLocationV2,
    FrontendAuthoritySourceV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    FrontendGraphReferenceInputV2, FrontendLanguageVersionV2, FrontendRelationAuthorityV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendStageV2, FrontendTransactionErrorV2,
    PlannedItemV2,
};
use quanta_gqlang_v2_planning::{
    AuthorityLaneV2, CapabilityV2, CardinalityClassV2, CheckedCompositionPlanInputV2,
    CompositionBodyFamilyV2, CompositionBodyPlanInputV2, CompositionInvalidReasonV2,
    CompositionRelationalBodyPlanInputV2, EffectClassV2, EvaluationDeterminismV2,
    EvidenceRequirementV2, NoPlanReasonV2, OrderingContractV2, PlanPolicyV2, PlanningDiagnosticV1,
    RecursionRiskClassV2, RenderRequirementV2, ResultKindV2, SemanticCostV2,
    WitnessExplosionRiskClassV2, WitnessSelectionPolicyV2,
};
use quanta_gqlang_v2_sema::test_support::{
    deep_relation_chain_document_v2, forged_rule_membership_document_v2,
    forged_small_receipt_bool_tree_document_v2, forged_small_receipt_relation_chain_document_v2,
    reference_graph_document_v2, reference_nested_check_document_v2,
    reference_relation_document_v2, reference_rule_document_v2, ForgedRuleMembershipMutationV2,
    ReferenceBudgetV2, ReferenceGraphDocumentV2, ReferenceGraphReferenceV2,
    ReferenceGraphRequestV2, ReferenceRelationFixtureV2, ReferenceRuleFixtureV2,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{
    CardinalityV1, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2, CompareMetricV1,
    ConfirmModeV1, DetectionModeV1, EmitModeV1, EvidenceModeV1, ExportFormatV1, FrontendBudgetV2,
    GraphDirectionV2, GraphQueryKindV1, GraphReferenceCatalogV2, GraphReferenceInputV2,
    GraphReferenceKindV2, GraphReferenceRoleV2, LocatedSpanV2, ProvenanceNamespaceV2,
    RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2, RelationSourceInputV2,
    ResolvedExplainTargetV2, ResolvedPresentationKindV2, SampleModeV1, SemanticItemIdV2,
    SemanticItemV2, ShowCategoryV1, SourceIdV2, SourceQualifiedNodeIdV2, SourceRegistrationV2,
    SourceRegistryViewV2, SpanV1, TruthModeV1, ValueTypeV1,
};

/// The host registry source identity used by the minimal authority.
const HOST_SOURCE: SourceIdV2 = SourceIdV2::from_raw(1);

/// The document source identity used by the source-text rails.
const DOC_SOURCE: SourceIdV2 = SourceIdV2::from_raw(9);

/// A source document that needs no relation or graph authority: one value
/// declaration, one rule, and one closed-category presentation.
const MINIMAL_SOURCE: &str = "let answer = 1; rule local_rule : x if x = 1; show capabilities;";

/// One sequential composition whose producer is a bounded SAMPLE and whose
/// body consumes the working set, so no host relation authority is needed.
fn composition_source(limit: u32) -> String {
    format!(
        "let sampled = sample query=\"auth\" | mode=lexical | limit={limit} \
         from sampled s | project s.path as path with sampled"
    )
}

/// One typed relation query over the host relation `records`.
const RECORD_SOURCE: &str = "from records r | project r.id as id";
const WORKSPACE_FILES_SOURCE: &str = "from files f | project f.path as path";

/// The stable code of one transaction failure.
fn failure_code(error: &FrontendTransactionErrorV2) -> String {
    match error {
        FrontendTransactionErrorV2::Document(failure) => failure
            .diagnostics()
            .first()
            .expect("a document failure carries at least one diagnostic")
            .code()
            .to_owned(),
        FrontendTransactionErrorV2::Authority(authority) => authority.code().to_owned(),
    }
}

/// Build one closed authority admitting a single host relation `records` with
/// one `id` field, solely from sema/modules re-exports.
fn authority_with_records() -> ClosedFrontendAuthorityV2 {
    authority_with_records_type(ValueTypeV1::int())
}

fn authority_with_records_type(id_type: ValueTypeV1) -> ClosedFrontendAuthorityV2 {
    authority_with_records_type_and_host_id(id_type, "app::records")
}

fn authority_with_records_type_and_host_id(
    id_type: ValueTypeV1,
    host_id: &str,
) -> ClosedFrontendAuthorityV2 {
    authority_with_records_binding_v3(id_type, host_id, None)
}

fn authority_with_records_binding_v3(
    id_type: ValueTypeV1,
    host_id: &str,
    identity: Option<[u8; 32]>,
) -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: HOST_SOURCE,
        source_len: 64,
    }])
    .expect("host registry admits one source");
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: HOST_SOURCE,
            span: SpanV1::new(0, 1),
        })
        .expect("checked host location");
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let profile = budget.authority_profile().expect("budget profile");
    let module = ModuleIdV2::try_new("app", location.clone()).expect("module identity");
    let mut builder =
        ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).expect("module builder");
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .expect("module node admits");
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .expect("module authority seals");
    let relations = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            authority: quanta_gqlang_v2_sema::RelationAuthorityV2::Host,
            provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone())
                .expect("provenance namespace"),
            qualified_name: host_id.to_owned(),
            lookup_name: "records".to_owned(),
            row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                "id",
                id_type,
                location.clone(),
            )
            .expect("id field")])
            .expect("records row"),
            cardinality: CardinalityV1::Bag,
            provenance: "host".to_owned(),
            location: location.clone(),
            host_binding_identity: identity,
        }],
        &host,
        profile.clone(),
    )
    .expect("relation catalog");
    let graph_references = GraphReferenceCatalogV2::try_new(
        vec![
            GraphReferenceInputV2 {
                spelling: "a".to_owned(),
                kind: GraphReferenceKindV2::Entity,
                entity_subtype: None,
                location: location.clone(),
            },
            GraphReferenceInputV2 {
                spelling: "b".to_owned(),
                kind: GraphReferenceKindV2::Entity,
                entity_subtype: None,
                location: location.clone(),
            },
            GraphReferenceInputV2 {
                spelling: "scope".to_owned(),
                kind: GraphReferenceKindV2::Scope,
                entity_subtype: None,
                location: location.clone(),
            },
        ],
        &host,
        profile,
    )
    .expect("graph catalog");
    ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, budget)
        .expect("closed authority seals")
}

fn authority_with_workspace_files(
    authority: FrontendRelationAuthorityV2,
) -> ClosedFrontendAuthorityV2 {
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST_SOURCE,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("workspace authority builder starts");
    let mut relation = FrontendRelationInputV2::canonical_workspace_files(location, location);
    relation.authority = authority;
    builder.add_relation(relation);
    builder.finish().expect("workspace authority seals")
}

/// Build one closed authority with a module graph and no relation/graph
/// authority, solely from sema/modules re-exports.
fn minimal_authority() -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: HOST_SOURCE,
        source_len: 64,
    }])
    .expect("host registry admits one source");
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: HOST_SOURCE,
            span: SpanV1::new(0, 1),
        })
        .expect("checked host location");
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let profile = budget.authority_profile().expect("budget profile");
    let module = ModuleIdV2::try_new("app", location.clone()).expect("module identity");
    let mut builder =
        ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).expect("module builder");
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .expect("module node admits");
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location,
            },
            VersionV2::new(1, 0, 0),
        )
        .expect("module authority seals");
    let relations =
        RelationCatalogV2::try_new(Vec::new(), &host, profile.clone()).expect("relation catalog");
    let graph_references =
        GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile).expect("graph catalog");
    ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, budget)
        .expect("closed authority seals")
}

fn source_policy_authority() -> ClosedFrontendAuthorityV2 {
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST_SOURCE,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("presentation authority builder starts");
    for (spelling, kind) in [
        ("api.entry", GraphReferenceKindV2::Entity),
        ("db.write", GraphReferenceKindV2::Entity),
        ("authz.clean", GraphReferenceKindV2::Sanitizer),
        ("repo", GraphReferenceKindV2::Scope),
        ("mid.one", GraphReferenceKindV2::Entity),
        ("mid.two", GraphReferenceKindV2::Entity),
        ("must.pass", GraphReferenceKindV2::Entity),
        ("recent", GraphReferenceKindV2::Window),
        ("edge", GraphReferenceKindV2::Frontier),
        ("svc.handler", GraphReferenceKindV2::Callable),
        ("pattern.hybrid", GraphReferenceKindV2::Pattern),
        ("src.entry", GraphReferenceKindV2::Entity),
        ("sink.write", GraphReferenceKindV2::Entity),
        ("run.a", GraphReferenceKindV2::Snapshot),
        ("run.b", GraphReferenceKindV2::Snapshot),
        ("result.latest", GraphReferenceKindV2::ExportSource),
    ] {
        builder.add_graph_reference(FrontendGraphReferenceInputV2 {
            spelling: spelling.to_owned(),
            kind,
            entity_subtype: None,
            location,
        });
    }
    builder.add_graph_reference(FrontendGraphReferenceInputV2 {
        spelling: "rule.sqli".to_owned(),
        kind: GraphReferenceKindV2::Entity,
        entity_subtype: Some("rule".to_owned()),
        location,
    });
    builder.add_graph_reference(FrontendGraphReferenceInputV2 {
        spelling: "owasp.sqli".to_owned(),
        kind: GraphReferenceKindV2::Entity,
        entity_subtype: Some("rule".to_owned()),
        location,
    });
    builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
        module_name: "app".to_owned(),
        local_name: "rule.hybrid".to_owned(),
        payload: FrontendCatalogPayloadV2::Rule {
            parameters: Vec::new(),
            result: FrontendScalarTypeV2::Bool,
        },
        provenance: "host".to_owned(),
        location,
    });
    builder.finish().expect("source policy authority seals")
}

/// One bounded FIND request with the exact authored depth.
fn bounded_query(kind: GraphQueryKindV1, depth: u32) -> ReferenceGraphRequestV2 {
    ReferenceGraphRequestV2 {
        kind,
        references: vec![
            ReferenceGraphReferenceV2 {
                role: GraphReferenceRoleV2::Source,
                selector: "user_input",
                span: (5, 15),
            },
            ReferenceGraphReferenceV2 {
                role: GraphReferenceRoleV2::Sink,
                selector: "exec",
                span: (19, 23),
            },
        ],
        depth: Some(depth),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 40),
    }
}

/// Compile one sealed FIND fixture.
fn compile_graph(request: ReferenceGraphRequestV2) -> ReferenceGraphDocumentV2 {
    reference_graph_document_v2(request).expect("bounded FIND fixture resolves")
}

// ---------------------------------------------------------------------------
// Positive assembly
// ---------------------------------------------------------------------------

/// Resolve one source document through the public sema transaction.
fn resolve_minimal_document(
    authority: &ClosedFrontendAuthorityV2,
) -> quanta_gqlang_v2_sema::SemanticDocumentV2 {
    let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
        DOC_SOURCE,
        MINIMAL_SOURCE,
        &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
    )
    .expect("minimal source parses");
    let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("HIR integrity holds");
    quanta_gqlang_v2_sema::resolve_checked_document_v2(checked, authority)
        .expect("semantic transaction succeeds")
}

#[path = "v3_compiled_document_assembly/assembly_tests.rs"]
mod assembly_tests;

#[path = "v3_compiled_document_assembly/request_body_tests.rs"]
mod request_body_tests;

#[path = "v3_compiled_document_assembly/scalar_request_tests.rs"]
mod scalar_request_tests;

#[path = "v3_compiled_document_assembly/execution_contract_tests.rs"]
mod execution_contract_tests;

#[path = "v3_compiled_document_assembly/null_order_v3.rs"]
mod null_order_v3;

#[path = "v3_compiled_document_assembly/host_binding_identity_v3.rs"]
mod host_binding_identity_v3;
