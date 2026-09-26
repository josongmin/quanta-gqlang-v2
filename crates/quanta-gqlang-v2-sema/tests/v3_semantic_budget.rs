#![expect(
    clippy::unwrap_used,
    reason = "owner test assertions inspect exact budget and receipt values"
)]

use quanta_gqlang_v2_hir::{
    AliasV1, CallArgHirV1, CallArgsHirV1, HirBuilderV1, HirDocumentV1, LocatedSpanV2,
    ProgramItemHirV1, ProjectionItemV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2,
    SpanV1,
};
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DownstreamBudgetReceiptV2, FrontendBudgetV2, FrontendStageV2, FrontendTransactionErrorV2,
    GraphReferenceCatalogV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2,
    RelationSourceInputV2, ValueTypeV1,
};

fn budget(
    source: u32,
    frame: u32,
    semantic: u32,
    diagnostics: u32,
    response: u32,
) -> Result<FrontendBudgetV2, quanta_gqlang_v2_sema::FrontendAuthorityErrorV2> {
    budget_with_relation_depth(source, frame, semantic, diagnostics, response, 3)
}

fn budget_with_relation_depth(
    source: u32,
    frame: u32,
    semantic: u32,
    diagnostics: u32,
    response: u32,
    relation_depth: u32,
) -> Result<FrontendBudgetV2, quanta_gqlang_v2_sema::FrontendAuthorityErrorV2> {
    FrontendBudgetV2::try_new(
        source,
        frame,
        3,
        3,
        3,
        relation_depth,
        3,
        3,
        3,
        semantic,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        diagnostics,
        response,
    )
}

/// One tiny-but-real budget whose authority dimensions still admit the
/// minimal closed authority used by the document-level oracles below.
fn document_budget(
    semantic: u32,
    diagnostics: u32,
    response: u32,
) -> Result<FrontendBudgetV2, quanta_gqlang_v2_sema::FrontendAuthorityErrorV2> {
    FrontendBudgetV2::try_new(
        64,
        64,
        64,
        8,
        8,
        8,
        64,
        8,
        8,
        semantic,
        16,
        16,
        64,
        64,
        32,
        16,
        16,
        64,
        16,
        16,
        diagnostics,
        response,
    )
}

#[test]
fn zero_dimensions_and_smaller_frame_reject_without_fabricating_source_span() {
    let error = budget(0, 2, 2, 2, 2).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    let error = budget(3, 2, 2, 2, 2).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    let error = budget(2, 2, 0, 2, 2).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    // The canonical diagnostic-count and response-byte caps reject zero
    // exactly like every other dimension.
    let error = budget(2, 2, 2, 0, 2).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    let error = budget(2, 2, 2, 2, 0).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    let error = budget_with_relation_depth(2, 2, 2, 2, 2, 129).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    assert_eq!(
        budget_with_relation_depth(2, 2, 2, 2, 2, 128)
            .unwrap()
            .max_relation_depth(),
        128
    );
    let error = FrontendBudgetV2::try_new(
        2, 2, 3, 3, 65, 3, 3, 3, 3, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2,
    )
    .unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
}

#[test]
fn exact_positive_limits_and_authority_profile_are_preserved() {
    let budget = budget(2, 2, 1, 4, 5).unwrap();
    assert_eq!(budget.max_source_bytes(), 2);
    assert_eq!(budget.max_frame_bytes(), 2);
    assert_eq!(budget.max_semantic_steps(), 1);
    assert_eq!(budget.max_diagnostics(), 4);
    assert_eq!(budget.max_response_bytes(), 5);
    let profile = budget.authority_profile().unwrap();
    assert_eq!(profile.module_nodes(), 3);
    assert_eq!(profile.module_edges(), 3);
    assert_eq!(profile.module_steps(), 3);
    assert_eq!(profile.authority_bytes(), 3);
    assert_eq!(profile.authority_string_bytes(), 3);
}

#[test]
fn pinned_standard_limits_have_no_zero_or_unbounded_dimensions() {
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    assert_eq!(budget.max_source_bytes(), 1_048_576);
    assert_eq!(budget.max_frame_bytes(), 4_194_304);
    assert_eq!(budget.max_tokens(), 131_072);
    assert_eq!(budget.max_document_items(), 4_096);
    assert_eq!(budget.max_expression_depth(), 64);
    assert_eq!(budget.max_relation_depth(), 128);
    assert_eq!(budget.max_expression_nodes(), 262_144);
    assert_eq!(budget.max_collection_width(), 4_096);
    assert_eq!(budget.max_projection_width(), 4_096);
    assert_eq!(budget.max_semantic_steps(), 1_000_000);
    assert_eq!(budget.max_module_graph_nodes(), 50_000);
    assert_eq!(budget.max_module_graph_edges(), 200_000);
    assert_eq!(budget.max_module_steps(), 1_000_000);
    assert_eq!(budget.max_authority_bytes(), 16_777_216);
    assert_eq!(budget.max_authority_string_bytes(), 65_536);
    assert_eq!(budget.max_rule_graph_nodes(), 50_000);
    assert_eq!(budget.max_rule_graph_edges(), 200_000);
    assert_eq!(budget.max_rule_steps(), 1_000_000);
    assert_eq!(budget.max_witness_depth(), 1_024);
    assert_eq!(budget.max_witnesses(), 10_000);
    assert_eq!(budget.max_diagnostics(), 1_024);
    assert_eq!(budget.max_response_bytes(), 8_388_608);
}

#[test]
fn downstream_receipt_projects_every_dimension_without_extension_or_loss() {
    let budget = budget(2, 4, 7, 6, 9).unwrap();
    let receipt = budget.downstream_receipt();
    // Complete projection: no dimension may be dropped or silently reminted.
    assert_eq!(receipt.max_source_bytes(), budget.max_source_bytes());
    assert_eq!(receipt.max_frame_bytes(), budget.max_frame_bytes());
    assert_eq!(receipt.max_tokens(), budget.max_tokens());
    assert_eq!(receipt.max_document_items(), budget.max_document_items());
    assert_eq!(
        receipt.max_expression_depth(),
        budget.max_expression_depth()
    );
    assert_eq!(receipt.max_relation_depth(), budget.max_relation_depth());
    assert_eq!(
        receipt.max_expression_nodes(),
        budget.max_expression_nodes()
    );
    assert_eq!(
        receipt.max_collection_width(),
        budget.max_collection_width()
    );
    assert_eq!(
        receipt.max_projection_width(),
        budget.max_projection_width()
    );
    assert_eq!(receipt.max_semantic_steps(), budget.max_semantic_steps());
    assert_eq!(
        receipt.max_module_graph_nodes(),
        budget.max_module_graph_nodes()
    );
    assert_eq!(
        receipt.max_module_graph_edges(),
        budget.max_module_graph_edges()
    );
    assert_eq!(receipt.max_module_steps(), budget.max_module_steps());
    assert_eq!(receipt.max_authority_bytes(), budget.max_authority_bytes());
    assert_eq!(
        receipt.max_authority_string_bytes(),
        budget.max_authority_string_bytes()
    );
    assert_eq!(
        receipt.max_rule_graph_nodes(),
        budget.max_rule_graph_nodes()
    );
    assert_eq!(
        receipt.max_rule_graph_edges(),
        budget.max_rule_graph_edges()
    );
    assert_eq!(receipt.max_rule_steps(), budget.max_rule_steps());
    assert_eq!(receipt.max_witness_depth(), budget.max_witness_depth());
    assert_eq!(receipt.max_witnesses(), budget.max_witnesses());
    assert_eq!(receipt.max_diagnostics(), budget.max_diagnostics());
    assert_eq!(receipt.max_response_bytes(), budget.max_response_bytes());
    // Value transplant: a receipt from any other budget profile is unequal,
    // even when only one downstream-relevant dimension differs.
    let standard = FrontendBudgetV2::standard_v2().unwrap();
    assert_ne!(receipt, standard.downstream_receipt());
    assert_ne!(
        receipt.max_response_bytes(),
        standard.downstream_receipt().max_response_bytes()
    );
    assert_eq!(
        budget.downstream_receipt(),
        budget.clone().downstream_receipt()
    );
    // The receipt is an immutable value type: consumers copy it, never extend.
    let copied: DownstreamBudgetReceiptV2 = receipt;
    assert_eq!(copied, budget.downstream_receipt());
}

fn scalar_authority(budget: &FrontendBudgetV2) -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 512,
    }])
    .unwrap();
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .unwrap();
    let module = ModuleIdV2::try_new("app", location.clone()).unwrap();
    let profile = budget.authority_profile().unwrap();
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).unwrap();
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .unwrap();
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let field = RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location.clone()).unwrap();
    let row = RecordTypeV1::try_new(vec![field]).unwrap();
    let relations = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
            provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone()).unwrap(),
            qualified_name: "records".into(),
            lookup_name: "records".into(),
            row,
            cardinality: CardinalityV1::Bag,
            provenance: "host".into(),
            location: location.clone(),
        }],
        &host,
        profile.clone(),
    )
    .unwrap();
    let graphs = GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile).unwrap();
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget.clone()).unwrap()
}

/// `from records as r | project count(*)`: the implicit (scalar) Group path.
fn scalar_count_star_document() -> HirDocumentV1 {
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let count = builder.call(
        "count",
        SpanV1::new(24, 29),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
            span: SpanV1::new(30, 31),
        }])
        .unwrap(),
        SpanV1::new(24, 31),
    );
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: count,
                alias: None,
            }],
            SpanV1::new(16, 31),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 31));
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 64,
        items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }
}

fn resolve_scalar(
    budget: &FrontendBudgetV2,
) -> Result<quanta_gqlang_v2_sema::SemanticDocumentV2, FrontendTransactionErrorV2> {
    resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(scalar_count_star_document()).unwrap(),
        &scalar_authority(budget),
    )
}

#[test]
fn hand_built_hir_depth_rejects_at_the_seal_as_a_resource_error() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let mut relation = builder.relation_source("r", span, None, span);
    for _ in 0..128 {
        relation = builder.relation_filter(relation, builder.bool_literal(true, span), span);
    }
    let document = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
        items: vec![ProgramItemHirV1::from(builder.query(relation, None, span))],
    };
    let error = CheckedSyntaxDocumentV2::try_from_hir(document).unwrap_err();
    assert_eq!(error.diagnostics().len(), 1);
    let diagnostic = error.diagnostics().first().unwrap();
    assert_eq!(diagnostic.stage(), FrontendStageV2::Resource);
    assert_eq!(diagnostic.code(), "hir.resource_limit_exceeded");
    assert_eq!(diagnostic.primary().span, span);
}

#[test]
fn sealed_document_carries_the_exact_immutable_budget_receipt() {
    let owned = document_budget(50, 6, 9).unwrap();
    let document = resolve_scalar(&owned).unwrap();
    // The consumer reads every cap from the document; there is no accepted
    // independent budget argument at any downstream seam.
    assert_eq!(document.budget_receipt(), owned.downstream_receipt());
    assert_eq!(document.budget_receipt().max_semantic_steps(), 50);
    assert_eq!(document.budget_receipt().max_expression_nodes(), 64);
    assert_eq!(document.budget_receipt().max_rule_graph_nodes(), 16);
    assert_eq!(document.budget_receipt().max_rule_graph_edges(), 16);
    assert_eq!(document.budget_receipt().max_rule_steps(), 64);
    assert_eq!(document.budget_receipt().max_witness_depth(), 16);
    assert_eq!(document.budget_receipt().max_witnesses(), 16);
    assert_eq!(document.budget_receipt().max_source_bytes(), 64);
    assert_eq!(document.budget_receipt().max_frame_bytes(), 64);
    assert_eq!(document.budget_receipt().max_diagnostics(), 6);
    assert_eq!(document.budget_receipt().max_response_bytes(), 9);
    // A receipt transplanted from another authority profile can never
    // equal the one sealed into this document.
    let other = document_budget(50, 6, 10).unwrap();
    let other_document = resolve_scalar(&other).unwrap();
    assert_ne!(document.budget_receipt(), other_document.budget_receipt());
    assert_ne!(document.budget_receipt(), other.downstream_receipt());
}

#[test]
#[expect(
    clippy::panic,
    reason = "the exact-limit boundary must be discovered or the oracle fails"
)]
fn implicit_group_semantic_steps_fail_at_the_exact_limit_and_one_over() {
    // Discover the exact charge sum of the documented tiny fixture, then
    // prove the boundary: success at the exact limit, resource failure with
    // the authored span and code one step below.
    // The documented tiny fixture charges exactly six semantic steps. The
    // pinned constant keeps the boundary test from being self-calibrated: a
    // charge-model drift shifts the discovered limit and fails here even
    // though the limit/one-over symmetry alone would still hold.
    const SCALAR_COUNT_STAR_STEPS: u32 = 6;
    let mut exact = None;
    for limit in 1..=64u32 {
        if let Ok(budget) = document_budget(limit, 6, 9) {
            if resolve_scalar(&budget).is_ok() {
                exact = Some(limit);
                break;
            }
        }
    }
    let exact = exact.unwrap_or_else(|| {
        panic!("scalar aggregate fixture must resolve within a bounded step budget");
    });
    assert_eq!(
        exact, SCALAR_COUNT_STAR_STEPS,
        "the discovered charge must match the pinned fixture charge"
    );
    let at_limit = document_budget(exact, 6, 9).unwrap();
    let document = resolve_scalar(&at_limit).unwrap();
    assert_eq!(document.budget_receipt().max_semantic_steps(), exact);
    let under = document_budget(exact - 1, 6, 9).unwrap();
    let FrontendTransactionErrorV2::Document(failure) = resolve_scalar(&under).unwrap_err() else {
        panic!("one semantic step over the limit must be a document failure");
    };
    let diagnostic = failure.diagnostics().first().unwrap();
    assert_eq!(diagnostic.code(), "frontend.resource_limit_exceeded");
    assert_eq!(diagnostic.primary().source_id, SourceIdV2::from_raw(9));
    // The final charge of this fixture is the Sema-owned implicit Group,
    // charged at the authored aggregate projection span.
    assert_eq!(diagnostic.primary().span, SpanV1::new(16, 31));
}
