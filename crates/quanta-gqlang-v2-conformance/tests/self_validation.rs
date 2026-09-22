#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "conformance tests fail with family-specific evidence"
)]

use std::collections::BTreeSet;

use quanta_gqlang_v2_conformance::{
    root_family_specs_v2, ExpectedDispositionV2, RootFamilySpecV2, ROOT_FAMILIES_V2,
    SUPPLEMENTAL_SOURCES_V2, UNSUPPORTED_SOURCE_V2,
};
use quanta_gqlang_v2_frontend_contract::{
    analyze_source_document_v2, compile_source_document_v3, AnalysisItemV2, AnalysisStatusV2,
    ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2, CompiledItemV2,
    FrontendAuthorityErrorV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    FrontendGraphReferenceInputV2, FrontendLanguageVersionV2, FrontendLockfileInputV2,
    FrontendPackInputV2, FrontendRelationCardinalityV2, FrontendRelationFieldInputV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, GraphReferenceKindV2, OpenAnalysisAuthorityV2,
    PlannedItemV2, SourceIdV2,
};

const HOST: SourceIdV2 = SourceIdV2::from_raw(1);
const LOCK: SourceIdV2 = SourceIdV2::from_raw(2);
const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn location(start: usize, end: usize) -> FrontendAuthorityLocationV2 {
    FrontendAuthorityLocationV2 {
        source_id: HOST,
        start,
        end,
    }
}

fn closed_authority() -> ClosedFrontendAuthorityV2 {
    closed_authority_with_lock_digest(DIGEST).expect("closed conformance authority")
}

fn closed_authority_with_lock_digest(
    lock_digest: &str,
) -> Result<ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2> {
    let lock_text = format!("secure-pack 1.0.0 {lock_digest}\n");
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![
            FrontendAuthoritySourceV2 {
                source_id: HOST,
                source_len: 4_096,
            },
            FrontendAuthoritySourceV2 {
                source_id: LOCK,
                source_len: lock_text.len(),
            },
        ],
        "app".to_owned(),
        location(0, 1),
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("authority builder");

    for (index, name) in ["users", "findings", "repos", "person"]
        .into_iter()
        .enumerate()
    {
        let base = 10 + index * 16;
        builder.add_relation(FrontendRelationInputV2 {
            authority: quanta_gqlang_v2_frontend_contract::FrontendRelationAuthorityV2::Host,
            provenance_namespace: "host".to_owned(),
            qualified_name: format!("app::{name}"),
            lookup_name: name.to_owned(),
            fields: [
                ("id", FrontendScalarTypeV2::Int),
                ("name", FrontendScalarTypeV2::String),
                ("active", FrontendScalarTypeV2::Bool),
                ("repo", FrontendScalarTypeV2::Int),
                ("score", FrontendScalarTypeV2::Int),
                ("age", FrontendScalarTypeV2::Int),
                ("t1", FrontendScalarTypeV2::String),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(field_index, (field, scalar_type))| FrontendRelationFieldInputV2 {
                    name: field.to_owned(),
                    scalar_type,
                    optional: false,
                    location: location(base + field_index, base + field_index + 1),
                },
            )
            .collect(),
            cardinality: FrontendRelationCardinalityV2::Bag,
            provenance: "host".to_owned(),
            location: location(base, base + 8),
        });
    }

    for (index, (spelling, kind, subtype)) in [
        ("api.entry", GraphReferenceKindV2::Entity, None),
        ("db.write", GraphReferenceKindV2::Entity, None),
        ("authz.clean", GraphReferenceKindV2::Sanitizer, None),
        ("a.b", GraphReferenceKindV2::Entity, None),
        ("c.d", GraphReferenceKindV2::Entity, None),
        ("rule.hybrid", GraphReferenceKindV2::Pattern, None),
        ("repo", GraphReferenceKindV2::Scope, None),
        ("rule.sqli", GraphReferenceKindV2::Entity, Some("rule")),
        ("owasp.sqli", GraphReferenceKindV2::Entity, Some("rule")),
        ("run.a", GraphReferenceKindV2::Snapshot, None),
        ("run.b", GraphReferenceKindV2::Snapshot, None),
        ("result.latest", GraphReferenceKindV2::ExportSource, None),
        ("pattern.hybrid", GraphReferenceKindV2::Pattern, None),
        ("src.entry", GraphReferenceKindV2::Entity, None),
        ("sink.write", GraphReferenceKindV2::Entity, None),
    ]
    .into_iter()
    .enumerate()
    {
        builder.add_graph_reference(FrontendGraphReferenceInputV2 {
            spelling: spelling.to_owned(),
            kind,
            entity_subtype: subtype.map(str::to_owned),
            location: location(200 + index, 201 + index),
        });
    }

    for (index, (local_name, payload)) in [
        (
            "rule.hybrid",
            FrontendCatalogPayloadV2::Rule {
                parameters: Vec::new(),
                result: FrontendScalarTypeV2::Bool,
            },
        ),
        ("queries.catalog.users", FrontendCatalogPayloadV2::Query),
        ("checks.shell-exec", FrontendCatalogPayloadV2::Check),
        ("models.code", FrontendCatalogPayloadV2::ModelSource),
        ("registry.security", FrontendCatalogPayloadV2::PackSource),
        ("graph.read", FrontendCatalogPayloadV2::Capability),
        ("graph.trace", FrontendCatalogPayloadV2::Capability),
        ("owasp.sql", FrontendCatalogPayloadV2::Check),
        ("authz.path", FrontendCatalogPayloadV2::Check),
    ]
    .into_iter()
    .enumerate()
    {
        builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
            module_name: "app".to_owned(),
            local_name: local_name.to_owned(),
            payload,
            provenance: "host".to_owned(),
            location: location(240 + index, 241 + index),
        });
    }

    builder.add_pack(FrontendPackInputV2 {
        pack_id: "secure-pack".to_owned(),
        version: FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
        content_digest: DIGEST.to_owned(),
        language_lower: FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
        language_upper: FrontendLanguageVersionV2 {
            major: 2,
            minor: 0,
            patch: 0,
        },
        location: location(300, 311),
    });
    builder.add_lockfile(FrontendLockfileInputV2 {
        path: "packs.secure-pack.lock".to_owned(),
        source_id: LOCK,
        text: lock_text,
        location: location(312, 334),
    });
    builder.finish()
}

fn source_limited_budget(max_source_bytes: u32) -> FrontendBudgetV2 {
    let standard = FrontendBudgetV2::standard_v2().expect("standard budget");
    FrontendBudgetV2::try_new(
        max_source_bytes,
        standard.max_frame_bytes(),
        standard.max_tokens(),
        standard.max_document_items(),
        standard.max_expression_depth(),
        standard.max_expression_nodes(),
        standard.max_collection_width(),
        standard.max_projection_width(),
        standard.max_semantic_steps(),
        standard.max_module_graph_nodes(),
        standard.max_module_graph_edges(),
        standard.max_module_steps(),
        standard.max_authority_bytes(),
        standard.max_authority_string_bytes(),
        standard.max_rule_graph_nodes(),
        standard.max_rule_graph_edges(),
        standard.max_rule_steps(),
        standard.max_witness_depth(),
        standard.max_witnesses(),
        standard.max_diagnostics(),
        standard.max_response_bytes(),
    )
    .expect("source-limited budget")
}

#[test]
fn registry_has_exactly_eighteen_unique_families() {
    assert_eq!(ROOT_FAMILIES_V2.len(), 18);
    assert_eq!(
        ROOT_FAMILIES_V2
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        ROOT_FAMILIES_V2.len()
    );
    assert_eq!(root_family_specs_v2().len(), ROOT_FAMILIES_V2.len());
}

#[test]
fn every_family_compiles_through_the_facade_with_declared_disposition() {
    let authority = closed_authority();
    for (index, spec) in root_family_specs_v2().enumerate() {
        assert!(spec.source.trim_start().starts_with(spec.keyword));
        let document = compile_source_document_v3(
            SourceIdV2::from_raw(100 + u32::try_from(index).expect("small registry")),
            spec.source,
            &authority,
        )
        .unwrap_or_else(|error| panic!("{:?} failed: {error:?}", spec.family));
        assert_eq!(document.items().len(), 1, "{:?}", spec.family);
        assert_disposition(
            spec,
            document.items().first().expect("exactly one compiled item"),
        );
    }
}

fn assert_disposition(spec: RootFamilySpecV2, item: &CompiledItemV2) {
    match (spec.expected, item) {
        (
            ExpectedDispositionV2::Planned {
                kind,
                result_kind,
                effect,
            },
            CompiledItemV2::Planned(planned),
        ) => {
            assert_eq!(planned_kind(planned), kind, "{:?}", spec.family);
            let plan = planned
                .plan()
                .unwrap_or_else(|| panic!("{:?} expected a per-item plan", spec.family));
            assert_eq!(
                plan.result().result_kind().keyword(),
                result_kind,
                "{:?}",
                spec.family
            );
            assert_eq!(plan.effect().keyword(), effect, "{:?}", spec.family);
        }
        (
            ExpectedDispositionV2::NoPlan { reason },
            CompiledItemV2::NoPlan { reason: actual, .. },
        ) => {
            assert_eq!(actual.code(), reason, "{:?}", spec.family);
        }
        (expected, actual) => panic!("{:?}: expected {expected:?}, got {actual:?}", spec.family),
    }
}

fn planned_kind(item: &PlannedItemV2) -> &'static str {
    match item {
        PlannedItemV2::Relation { .. } => "relation",
        PlannedItemV2::Graph { .. } => "graph",
        PlannedItemV2::Check { .. } => "check",
        PlannedItemV2::Presentation { .. } => "presentation",
        PlannedItemV2::Composition { .. } => "composition",
        PlannedItemV2::MetaRequireCertified { .. } => "meta_require_certified",
        PlannedItemV2::RuleMember(_) => "rule_member",
    }
}

#[test]
fn open_authority_reports_every_valid_family_as_incomplete_not_compiled() {
    let authority = OpenAnalysisAuthorityV2::try_standard_v2().expect("open authority");
    for (index, spec) in root_family_specs_v2().enumerate() {
        let source_id = SourceIdV2::from_raw(200 + u32::try_from(index).expect("small registry"));
        let first = analyze_source_document_v2(source_id, spec.source, &authority)
            .unwrap_or_else(|error| panic!("{:?} open analysis failed: {error:?}", spec.family));
        let second = analyze_source_document_v2(source_id, spec.source, &authority)
            .expect("repeat analysis");
        assert_eq!(first, second, "{:?} must be deterministic", spec.family);
        assert_eq!(
            first.status(),
            AnalysisStatusV2::Incomplete,
            "{:?}",
            spec.family
        );
        assert_eq!(first.status().keyword(), "incomplete");
        assert!(!first.unresolved().is_empty(), "{:?}", spec.family);
        let diagnostics = first.diagnostics();
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code())
                .collect::<Vec<_>>(),
            [
                "pipeline.analysis.missing_graph_authority",
                "pipeline.analysis.missing_module_authority",
                "pipeline.analysis.missing_relation_authority",
            ],
            "{:?}",
            spec.family
        );
        for diagnostic in diagnostics {
            assert_eq!(diagnostic.stage().keyword(), "declaration");
            assert_eq!(diagnostic.severity().keyword(), "warning");
            assert_eq!(diagnostic.primary().source_id, source_id);
            assert_eq!(diagnostic.primary().span.start, 0);
            assert_eq!(diagnostic.primary().span.end, spec.source.len());
        }
        assert!(first
            .items()
            .iter()
            .all(|item| matches!(item, AnalysisItemV2::BlockedByDiagnostics(_))));
    }
}

#[test]
fn facade_budget_and_pack_lock_failures_are_explicit() {
    let authority = OpenAnalysisAuthorityV2::try_empty_v2(source_limited_budget(4))
        .expect("limited open authority");
    let failure =
        analyze_source_document_v2(SourceIdV2::from_raw(400), "show capabilities", &authority)
            .expect_err("source past max_source_bytes must reject");
    match failure {
        quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) => {
            assert_eq!(failure.diagnostics().len(), 1);
            let diagnostic = failure
                .diagnostics()
                .first()
                .expect("one budget diagnostic");
            assert_eq!(diagnostic.code(), "frontend.resource_limit_exceeded");
            assert_eq!(diagnostic.primary().span.start, 0);
            assert_eq!(diagnostic.primary().span.end, "show capabilities".len());
        }
        quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Authority(error) => {
            panic!("unexpected authority failure: {error:?}")
        }
    }

    let error = closed_authority_with_lock_digest(
        "1111111111111111111111111111111111111111111111111111111111111111",
    )
    .expect_err("pack/lock digest mismatch must reject");
    assert_eq!(error.code(), "modules.lock_mismatch");
}

#[test]
fn supplemental_units_are_explicit_and_unsupported_source_fails_closed() {
    let authority = closed_authority();
    let value = compile_source_document_v3(
        SourceIdV2::from_raw(300),
        SUPPLEMENTAL_SOURCES_V2[0],
        &authority,
    )
    .expect("value declaration compiles");
    assert!(matches!(
        value.items().first().expect("one value declaration"),
        CompiledItemV2::NoPlan { .. }
    ));

    let rule = compile_source_document_v3(
        SourceIdV2::from_raw(301),
        SUPPLEMENTAL_SOURCES_V2[1],
        &authority,
    )
    .expect("rule compiles");
    assert!(matches!(
        rule.items().first().expect("one rule declaration"),
        CompiledItemV2::Planned(planned)
            if matches!(planned.as_ref(), PlannedItemV2::RuleMember(_))
    ));
    assert!(rule.rule_program().is_some());

    assert!(compile_source_document_v3(
        SourceIdV2::from_raw(302),
        UNSUPPORTED_SOURCE_V2,
        &authority
    )
    .is_err());
}
