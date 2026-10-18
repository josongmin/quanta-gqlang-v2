#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "conformance tests fail with family-specific evidence"
)]

use std::collections::BTreeSet;

use quanta_gqlang_v2_conformance::{
    root_family_specs_v2, ExpectedDispositionV2, RootFamilySpecV2, RootFamilyV2, ROOT_FAMILIES_V2,
    SUPPLEMENTAL_SOURCES_V2, UNSUPPORTED_SOURCE_V2,
};
use quanta_gqlang_v2_frontend_contract::{
    analyze_source_document_v2, classify_single_source_root_v3, compile_source_document_v3,
    AlgebraFamilyV2, AnalysisItemV2, AnalysisStatusV2, ClosedFrontendAuthorityBuilderV2,
    ClosedFrontendAuthorityV2, CompiledItemV2, ExecutionCallTargetV3, ExecutionCommandPhaseV3,
    ExecutionEffectV3, ExecutionExpressionShapeV3, ExecutionFamilyV3, ExecutionJoinModeV3,
    FrontendAuthorityErrorV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    FrontendGraphReferenceInputV2, FrontendLanguageVersionV2, FrontendLockfileInputV2,
    FrontendPackInputV2, FrontendRelationCardinalityV2, FrontendRelationFieldInputV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendSourceRootFamilyV3,
    FrontendSymbolTargetInputV3, GraphReferenceKindV2, OpenAnalysisAuthorityV2, PlannedItemV2,
    SemanticItemIdV2, SourceIdV2,
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
                    sequence: false,
                    optional: false,
                    location: location(base + field_index, base + field_index + 1),
                },
            )
            .collect(),
            cardinality: FrontendRelationCardinalityV2::Bag,
            provenance: "host".to_owned(),
            location: location(base, base + 8),
            host_binding_identity: None,
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
        ("p", GraphReferenceKindV2::Pattern, None),
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

    builder.add_symbol_target(FrontendSymbolTargetInputV3 {
        selector: "symbol.one".to_owned(),
        repo_path: "src/a.py".to_owned(),
        symbol_id: "def:17".to_owned(),
        start_byte: 17,
        end_byte: 20,
        location: location(340, 341),
    });

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
        standard.max_relation_depth(),
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
fn registry_has_exactly_nineteen_unique_families() {
    assert_eq!(ROOT_FAMILIES_V2.len(), 19);
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

fn root_keyword(root: FrontendSourceRootFamilyV3) -> &'static str {
    match root {
        FrontendSourceRootFamilyV3::Relation => "relation",
        FrontendSourceRootFamilyV3::Declaration => "declaration",
        FrontendSourceRootFamilyV3::Rule => "rule",
        FrontendSourceRootFamilyV3::Graph => "graph",
        FrontendSourceRootFamilyV3::Check => "check",
        FrontendSourceRootFamilyV3::Presentation => "presentation",
        FrontendSourceRootFamilyV3::Catalog => "catalog",
        FrontendSourceRootFamilyV3::Composition => "composition",
        FrontendSourceRootFamilyV3::Meta => "meta",
        FrontendSourceRootFamilyV3::Command => "command",
    }
}

#[test]
fn registered_and_supplemental_sources_cover_closed_root_classification() {
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let mut observed = BTreeSet::new();
    for spec in root_family_specs_v2() {
        let expected = match spec.family {
            RootFamilyV2::Select => FrontendSourceRootFamilyV3::Relation,
            RootFamilyV2::Find => FrontendSourceRootFamilyV3::Graph,
            RootFamilyV2::Check => FrontendSourceRootFamilyV3::Check,
            RootFamilyV2::Explain
            | RootFamilyV2::Show
            | RootFamilyV2::Describe
            | RootFamilyV2::Sample
            | RootFamilyV2::Compare
            | RootFamilyV2::Export => FrontendSourceRootFamilyV3::Presentation,
            RootFamilyV2::View
            | RootFamilyV2::RuleBinding
            | RootFamilyV2::Model
            | RootFamilyV2::Pack
            | RootFamilyV2::Profile
            | RootFamilyV2::Gate => FrontendSourceRootFamilyV3::Catalog,
            RootFamilyV2::UsePack | RootFamilyV2::RequireCertified => {
                FrontendSourceRootFamilyV3::Meta
            }
            RootFamilyV2::Composition => FrontendSourceRootFamilyV3::Composition,
            RootFamilyV2::Rename => FrontendSourceRootFamilyV3::Command,
        };
        let actual = classify_single_source_root_v3(SourceIdV2::from_raw(90), spec.source, &budget)
            .expect("registered source classifies without host authority");
        assert_eq!(actual, expected, "{:?}", spec.family);
        observed.insert(root_keyword(actual));
    }
    for (source, expected) in SUPPLEMENTAL_SOURCES_V2.into_iter().zip([
        FrontendSourceRootFamilyV3::Declaration,
        FrontendSourceRootFamilyV3::Rule,
    ]) {
        let actual = classify_single_source_root_v3(SourceIdV2::from_raw(90), source, &budget)
            .expect("supplemental source classifies without host authority");
        assert_eq!(actual, expected, "{source}");
        observed.insert(root_keyword(actual));
    }
    assert_eq!(
        observed,
        BTreeSet::from([
            "relation",
            "declaration",
            "rule",
            "graph",
            "check",
            "presentation",
            "catalog",
            "composition",
            "meta",
            "command",
        ]),
        "every closed classifier variant needs a source witness",
    );
}

#[test]
fn rename_preview_and_apply_preserve_exact_intent_through_facade() {
    let authority = closed_authority();
    for (source, phase, effect, token) in [
        (
            "rename symbol.one to \"new_name\" | phase=preview | snapshot=\"snap-7\"",
            ExecutionCommandPhaseV3::Preview,
            ExecutionEffectV3::ReadOnly,
            None,
        ),
        (
            "rename symbol.one to \"new_name\" | phase=apply | snapshot=\"snap-7\" | preview_token=\"issued-1\"",
            ExecutionCommandPhaseV3::Apply,
            ExecutionEffectV3::MutationRequiresGrant,
            Some("issued-1"),
        ),
    ] {
        let document = compile_source_document_v3(SourceIdV2::from_raw(91), source, &authority)
            .expect("closed rename intent compiles through the facade");
        let [item] = document.execution().items() else {
            panic!("one exact command item");
        };
        assert_eq!(item.family(), ExecutionFamilyV3::Command);
        assert_eq!(item.plan().effect(), effect);
        let command = item.command().expect("complete rename payload");
        assert_eq!(command.phase(), phase);
        assert_eq!(command.target().selector(), "symbol.one");
        assert_eq!(command.target().repo_path(), "src/a.py");
        assert_eq!(command.target().symbol_id(), "def:17");
        assert_eq!(command.target().start_byte(), 17);
        assert_eq!(command.target().end_byte(), 20);
        assert_eq!(command.new_name(), "new_name");
        assert_eq!(command.expected_snapshot(), "snap-7");
        assert_eq!(command.preview_token(), token);
    }
}

#[test]
fn authored_result_shapes_reject_closed_compile_without_omission() {
    let authority = closed_authority();
    let source_id = SourceIdV2::from_raw(92);
    let plain = "from users u | project u.id as id";
    let control = compile_source_document_v3(source_id, plain, &authority)
        .expect("the same query without a result shape compiles");
    assert_eq!(control.execution().items().len(), 1);

    for source in [
        "from users u | project u.id as id => 1",
        "from users u | project u.id as id => { id: 1 }",
        "from users u | project u.id as id => { row: { id: 1, tags: [ \"tag\" ] } }",
        "let shaped = from users u | project u.id as id => { id: 1 } from shaped s with shaped",
    ] {
        let error = compile_source_document_v3(source_id, source, &authority)
            .expect_err("an unsupported authored shape cannot issue an execution document");
        let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) =
            error
        else {
            panic!("{source}: expected a source-anchored rejection");
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("{source}: one explicit result-shape diagnostic");
        };
        assert_eq!(
            diagnostic.code(),
            "sema.document_validation_failed",
            "{source}"
        );
        assert_eq!(diagnostic.stage().keyword(), "validation", "{source}");
        assert_eq!(diagnostic.severity().keyword(), "error", "{source}");
        assert_eq!(
            diagnostic.message(),
            "authored query result shapes are not supported by closed compilation",
            "{source}"
        );
        assert_eq!(diagnostic.primary().source_id, source_id);
        let start = if source.starts_with("let ") { 13 } else { 0 };
        let end = if source.starts_with("let ") {
            59
        } else {
            source.len()
        };
        assert_eq!(diagnostic.primary().span.start, start, "{source}");
        assert_eq!(diagnostic.primary().span.end, end, "{source}");
    }
}

#[test]
fn every_family_compiles_through_the_facade_with_declared_disposition() {
    let authority = closed_authority();
    let specs: Vec<RootFamilySpecV2> = root_family_specs_v2().collect();
    assert_eq!(
        specs.len(),
        ROOT_FAMILIES_V2.len(),
        "root family registry must not be empty or drift"
    );
    assert_eq!(
        specs.iter().map(|spec| spec.family).collect::<Vec<_>>(),
        ROOT_FAMILIES_V2.to_vec(),
        "registry specs must map one-to-one onto ROOT_FAMILIES_V2 in order"
    );
    for (index, spec) in specs.into_iter().enumerate() {
        assert_eq!(
            spec.source.split_whitespace().next(),
            Some(spec.keyword),
            "{:?}: source must open with its declared keyword",
            spec.family
        );
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
        match (
            spec.expected,
            document.items().first().expect("one compiled item"),
        ) {
            (ExpectedDispositionV2::NoPlan { .. }, CompiledItemV2::NoPlan { .. }) => {
                assert!(document.execution().items().is_empty(), "{:?}", spec.family);
            }
            (ExpectedDispositionV2::Planned { kind, .. }, CompiledItemV2::Planned(planned)) => {
                let [execution_item] = document.execution().items() else {
                    panic!("{:?}: one V3 plan-required item", spec.family);
                };
                let expected_family = match kind {
                    "relation" => ExecutionFamilyV3::Relation,
                    "graph" => ExecutionFamilyV3::Graph,
                    "check" => ExecutionFamilyV3::Check,
                    "presentation" => ExecutionFamilyV3::Presentation,
                    "composition" => ExecutionFamilyV3::Composition,
                    "meta_require_certified" => ExecutionFamilyV3::MetaRequireCertified,
                    "command" => ExecutionFamilyV3::Command,
                    other => panic!("unmapped V3 family: {other}"),
                };
                assert_eq!(
                    execution_item.item_id(),
                    planned.item_id(),
                    "{:?}",
                    spec.family
                );
                assert_eq!(
                    execution_item.family(),
                    expected_family,
                    "{:?}",
                    spec.family
                );
                assert_eq!(
                    execution_item.relation().is_some(),
                    expected_family == ExecutionFamilyV3::Relation,
                    "{:?}",
                    spec.family,
                );
                assert_eq!(
                    execution_item.composition().is_some(),
                    expected_family == ExecutionFamilyV3::Composition,
                    "{:?}",
                    spec.family,
                );
                assert_eq!(
                    execution_item.command().is_some(),
                    expected_family == ExecutionFamilyV3::Command,
                    "{:?}",
                    spec.family,
                );
            }
            _ => panic!(
                "{:?}: V2 disposition mismatch before V3 mapping",
                spec.family
            ),
        }
    }
}

/// A selected acceptance subset reads the actual shipped files. The separate
/// family registry checks every family, while these cases detect drift between
/// published source bytes and closed-authority frontend behavior.
#[test]
fn selected_shipped_examples_compile_with_pinned_frontend_families() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("conformance crate is under crates/");
    let authority = closed_authority();
    let examples = [
        (
            "examples/select/basic.qglang",
            "relation",
            ExecutionFamilyV3::Relation,
        ),
        (
            "examples/check/hybrid_rule.qglang",
            "check",
            ExecutionFamilyV3::Check,
        ),
        (
            "examples/find/taint.qglang",
            "graph",
            ExecutionFamilyV3::Graph,
        ),
        (
            "examples/explain/why_no_result.qglang",
            "presentation",
            ExecutionFamilyV3::Presentation,
        ),
        (
            "examples/show/capabilities.qglang",
            "presentation",
            ExecutionFamilyV3::Presentation,
        ),
        (
            "examples/describe/rule_detail.qglang",
            "presentation",
            ExecutionFamilyV3::Presentation,
        ),
        (
            "examples/sample/retrieval.qglang",
            "presentation",
            ExecutionFamilyV3::Presentation,
        ),
        (
            "examples/compare/semantic_delta.qglang",
            "presentation",
            ExecutionFamilyV3::Presentation,
        ),
        (
            "examples/composition/let_with_find.qglang",
            "composition",
            ExecutionFamilyV3::Composition,
        ),
        (
            "examples/require_certified/strict_only.qglang",
            "meta_require_certified",
            ExecutionFamilyV3::MetaRequireCertified,
        ),
    ];
    assert_eq!(
        examples.len(),
        10,
        "the selected shipped acceptance corpus stays populated"
    );
    for (index, (path, expected_kind, expected_family)) in examples.into_iter().enumerate() {
        let source = std::fs::read_to_string(repo.join(path))
            .unwrap_or_else(|error| panic!("{path}: read shipped source: {error}"));
        let document = compile_source_document_v3(
            SourceIdV2::from_raw(500 + u32::try_from(index).expect("small corpus")),
            &source,
            &authority,
        )
        .unwrap_or_else(|error| panic!("{path}: closed frontend rejected: {error:?}"));
        assert_eq!(document.items().len(), 1, "{path}: one authored item");
        let Some(CompiledItemV2::Planned(planned)) = document.items().first() else {
            panic!("{path}: expected one planned item");
        };
        assert_eq!(planned_kind(planned), expected_kind, "{path}: planned kind");
        let [execution_item] = document.execution().items() else {
            panic!("{path}: expected one execution producer item");
        };
        assert_eq!(
            execution_item.family(),
            expected_family,
            "{path}: V3 family"
        );
    }
}

#[test]
fn shipped_catalog_declarations_retain_typed_nonexecution() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("conformance crate is under crates/");
    let authority = closed_authority();
    let examples = [
        ("examples/view/catalog_users.qglang", "catalog_declaration"),
        ("examples/rule/shell_exec.qglang", "catalog_declaration"),
        (
            "examples/model/embeddings_code.qglang",
            "catalog_declaration",
        ),
        (
            "examples/pack/security_default.qglang",
            "catalog_declaration",
        ),
        (
            "examples/profile/strict_review.qglang",
            "catalog_declaration",
        ),
        ("examples/gate/release_gate.qglang", "catalog_declaration"),
        ("examples/use_pack/secure_pack.qglang", "meta_activation"),
    ];
    for (index, (path, expected_reason)) in examples.into_iter().enumerate() {
        let source = std::fs::read_to_string(repo.join(path))
            .unwrap_or_else(|error| panic!("{path}: read shipped source: {error}"));
        let document = compile_source_document_v3(
            SourceIdV2::from_raw(600 + u32::try_from(index).expect("small corpus")),
            &source,
            &authority,
        )
        .unwrap_or_else(|error| panic!("{path}: closed frontend rejected: {error:?}"));
        assert_eq!(document.items().len(), 1, "{path}: one authored item");
        let Some(CompiledItemV2::NoPlan { reason, .. }) = document.items().first() else {
            panic!("{path}: expected typed catalog non-plan");
        };
        assert_eq!(reason.code(), expected_reason, "{path}");
        assert!(document.execution().items().is_empty(), "{path}");
    }
}

#[test]
fn shipped_export_without_authored_profile_rejects_closed_compilation() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("conformance crate is under crates/");
    let source = std::fs::read_to_string(repo.join("examples/export/sarif.qglang"))
        .expect("shipped export source");
    let error = compile_source_document_v3(SourceIdV2::from_raw(700), &source, &closed_authority())
        .expect_err("the shipped export names a profile absent from this document");
    let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) = error
    else {
        panic!("export must reject at the authored profile reference");
    };
    let [diagnostic] = failure.diagnostics() else {
        panic!("export must have one primary profile diagnostic");
    };
    assert_eq!(diagnostic.code(), "modules.unknown_catalog_reference");
    assert_eq!(diagnostic.primary().source_id, SourceIdV2::from_raw(700));
    assert_eq!(diagnostic.primary().span.start, 51);
    assert_eq!(diagnostic.primary().span.end, 57);
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
        PlannedItemV2::Command { .. } => "command",
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
    let specs: Vec<RootFamilySpecV2> = root_family_specs_v2().collect();
    assert_eq!(
        specs.len(),
        ROOT_FAMILIES_V2.len(),
        "root family registry must not be empty or drift"
    );
    for (index, spec) in specs.into_iter().enumerate() {
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
        let expected_codes = if spec.family == RootFamilyV2::Rename {
            vec![
                "pipeline.analysis.missing_graph_authority",
                "pipeline.analysis.missing_module_authority",
                "pipeline.analysis.missing_relation_authority",
                "pipeline.analysis.missing_symbol_target_authority",
            ]
        } else {
            vec![
                "pipeline.analysis.missing_graph_authority",
                "pipeline.analysis.missing_module_authority",
                "pipeline.analysis.missing_relation_authority",
            ]
        };
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code())
                .collect::<Vec<_>>(),
            expected_codes,
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
        assert!(
            !first.items().is_empty(),
            "{:?}: open analysis must retain the authored items as blocked",
            spec.family
        );
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
fn closed_source_predicates_reject_non_boolean_and_invalid_comparison() {
    let authority = closed_authority();
    for source in [
        "from users u | where u.active",
        "from users u | where u.age > 1",
    ] {
        let document = compile_source_document_v3(SourceIdV2::from_raw(401), source, &authority)
            .expect("boolean predicate must compile");
        let [CompiledItemV2::Planned(planned)] = document.items() else {
            panic!("{source}: expected exactly one planned relation");
        };
        assert!(matches!(planned.as_ref(), PlannedItemV2::Relation { .. }));
        assert_eq!(document.execution().items().len(), 1);
    }

    for (source, span) in [
        ("from users u | where 1", (21, 22)),
        ("from users u | where u.age", (21, 26)),
        ("from users u | where u.name > 1", (21, 31)),
    ] {
        let error = compile_source_document_v3(SourceIdV2::from_raw(401), source, &authority)
            .expect_err("invalid predicate must not produce an execution document");
        let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) =
            error
        else {
            panic!("invalid predicate must fail in the document transaction");
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("{source}: expected one typed diagnostic");
        };
        assert_eq!(diagnostic.code(), "types.bad_overload", "{source}");
        assert_eq!(diagnostic.primary().source_id, SourceIdV2::from_raw(401));
        assert_eq!(
            (
                diagnostic.primary().span.start,
                diagnostic.primary().span.end
            ),
            span,
            "{source}"
        );
    }
}

#[test]
fn authored_join_on_and_cross_join_retain_distinct_v3_semantics() {
    let authority = closed_authority();
    for (source, mode, filtered_left, predicate_count) in [
        (
            "from findings f | project f.repo as finding_repo | join repos r on finding_repo = r.id",
            ExecutionJoinModeV3::Inner,
            false,
            1,
        ),
        (
            "from findings f | where f.score > 1 | project f.repo as finding_repo | join repos r on finding_repo = r.id",
            ExecutionJoinModeV3::Inner,
            true,
            1,
        ),
        (
            "from findings f | project f.repo as finding_repo | cross join repos r",
            ExecutionJoinModeV3::Cross,
            false,
            0,
        ),
    ] {
        let document = compile_source_document_v3(SourceIdV2::from_raw(402), source, &authority)
            .expect("authored join must compile through the facade");
        let [item] = document.execution().items() else {
            panic!("{source}: expected one executable item");
        };
        let relation = item.relation().expect("joined relation");
        let root = relation.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Join, "{source}");
        assert_eq!(root.join_mode(), Some(mode), "{source}");
        assert_eq!(root.predicates().len(), predicate_count, "{source}");
        let [left, right] = root.children() else {
            panic!("{source}: join must retain exactly two inputs");
        };
        assert_eq!(left.family(), AlgebraFamilyV2::Project, "{source}");
        assert_eq!(right.family(), AlgebraFamilyV2::Scan, "{source}");
        assert_ne!(left.stage(), right.stage(), "{source}");
        assert_eq!(right.source().expect("right host source").name(), "app::repos");
        let [upstream] = left.children() else {
            panic!("{source}: projection must retain one input");
        };
        assert_eq!(
            upstream.family(),
            if filtered_left {
                AlgebraFamilyV2::Filter
            } else {
                AlgebraFamilyV2::Scan
            },
            "{source}"
        );
        assert_eq!(upstream.predicates().len(), usize::from(filtered_left));
        let scan = if filtered_left {
            let [scan] = upstream.children() else {
                panic!("{source}: filter must retain one scan");
            };
            scan
        } else {
            upstream
        };
        assert_eq!(scan.source().expect("left host source").name(), "app::findings");
        if mode == ExecutionJoinModeV3::Inner {
            let [predicate] = root.predicates() else {
                panic!("{source}: inner join must retain one ON predicate");
            };
            assert!(relation.expressions().iter().any(|expression| {
                expression.id() == *predicate
                    && matches!(
                        expression.shape(),
                        ExecutionExpressionShapeV3::Compare { .. }
                    )
            }));
        } else {
            assert!(root.predicates().is_empty(), "CROSS cannot gain ON");
        }
    }
}

#[test]
fn authored_aggregates_retain_partition_truth_and_reject_illegal_contexts() {
    let authority = closed_authority();

    for (source, expected_partition_keys) in [
        (
            "from findings f | group by f.repo | project sum(f.score) as total",
            1,
        ),
        ("from findings f | project sum(f.score) as total", 0),
    ] {
        let document = compile_source_document_v3(SourceIdV2::from_raw(403), source, &authority)
            .expect("legal aggregate must compile through the facade");
        let [item] = document.execution().items() else {
            panic!("{source}: expected one executable item");
        };
        let relation = item.relation().expect("aggregate relation");
        let aggregate = relation.root();
        assert_eq!(aggregate.family(), AlgebraFamilyV2::Aggregate, "{source}");
        let [group] = aggregate.children() else {
            panic!("{source}: aggregate must consume exactly one group");
        };
        assert_eq!(group.family(), AlgebraFamilyV2::Group, "{source}");
        assert_eq!(
            group.schema().partition_keys().len(),
            expected_partition_keys
        );
        assert_eq!(
            aggregate.schema().partition_keys(),
            group.schema().partition_keys(),
            "{source}: aggregate must preserve group partition identity"
        );
        let [aggregate_id] = aggregate.expressions() else {
            panic!("{source}: expected exactly one aggregate expression");
        };
        let expression = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == *aggregate_id)
            .expect("aggregate expression must exist in the execution table");
        let ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Builtin { name },
            arguments,
        } = expression.shape()
        else {
            panic!("{source}: authored aggregate must retain a builtin call");
        };
        assert_eq!(name, "sum", "{source}");
        let [Some(argument_id)] = arguments.as_ref() else {
            panic!("{source}: sum must retain exactly one expression argument");
        };
        let argument = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == *argument_id)
            .expect("sum argument must exist in the execution table");
        let ExecutionExpressionShapeV3::Path { base, segments } = argument.shape() else {
            panic!("{source}: sum must retain the authored f.score path");
        };
        assert_eq!(segments.as_ref(), ["score"], "{source}");
        let base = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == *base)
            .expect("field-path base must exist in the execution table");
        let ExecutionExpressionShapeV3::Binding(binding) = base.shape() else {
            panic!("{source}: field-path base must retain its binding");
        };
        assert_eq!(binding.name(), "f", "{source}");
        let [field] = aggregate.schema().fields() else {
            panic!("{source}: aggregate must expose exactly one field");
        };
        assert_eq!(field.name(), "total", "{source}");
    }

    for (source, expected_span) in [
        ("from findings f | where sum(f.score) > 1", (24, 36)),
        (
            "from findings f | project sum(sum(f.score)) as total",
            (30, 42),
        ),
        (
            "from findings f | project count(is_null(sum(f.score))) as total",
            (40, 52),
        ),
    ] {
        let error = compile_source_document_v3(SourceIdV2::from_raw(403), source, &authority)
            .expect_err("illegal aggregate context must fail before execution projection");
        let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) =
            error
        else {
            panic!("{source}: expected document transaction failure");
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("{source}: expected one exact semantic diagnostic");
        };
        assert_eq!(
            diagnostic.code(),
            "sema.illegal_aggregate_context",
            "{source}"
        );
        assert_eq!(diagnostic.primary().source_id, SourceIdV2::from_raw(403));
        assert_eq!(
            (
                diagnostic.primary().span.start,
                diagnostic.primary().span.end
            ),
            expected_span,
            "{source}"
        );
    }
}

#[test]
fn two_authored_mutual_rules_reach_one_checked_program() {
    let authority = closed_authority();
    let source = "rule r : x if x = 1 and s(x); rule s : x if x = 1 and r(x)";
    let document = compile_source_document_v3(SourceIdV2::from_raw(406), source, &authority)
        .expect("mutual source rules must form one checked program");
    assert_eq!(document.items().len(), 2);
    let program = document
        .rule_program()
        .expect("one document-wide rule program");
    assert_eq!(program.members().len(), 2);
    assert_eq!(program.stratification().recursive().len(), 2);
    assert_ne!(
        program
            .members()
            .first()
            .expect("first member")
            .member_item_id(),
        program
            .members()
            .get(1)
            .expect("second member")
            .member_item_id()
    );
    for (item, member) in document.items().iter().zip(program.members()) {
        let CompiledItemV2::Planned(planned) = item else {
            panic!("each authored rule must retain a checked member reference");
        };
        let PlannedItemV2::RuleMember(reference) = planned.as_ref() else {
            panic!("each authored rule must be a rule-program member");
        };
        assert_eq!(*reference, *member);
        assert_eq!(reference.program_id(), program.program_id());
    }
    assert_eq!(document.execution().items().len(), 0);
}

#[test]
fn source_rule_calls_require_direct_truth_literals_before_plan_assembly() {
    let authority = closed_authority();
    for body in ["p(x)", "(p(x))", "((p(x)))", "(p(x) or false)"] {
        let source = format!("rule p : x if x = 1 and {body}");
        let compiled = compile_source_document_v3(SourceIdV2::from_raw(406), &source, &authority)
            .unwrap_or_else(|error| panic!("{body}: direct recursion must compile: {error:?}"));
        let program = compiled.rule_program().expect("one checked rule program");
        assert_eq!(program.members().len(), 1, "{body}");
        assert_eq!(program.stratification().recursive().len(), 1, "{body}");
    }

    for body in [
        "p(x) = false",
        "(p(x) = true)",
        "(p(x)) = true",
        "is_null({a: p(x)})",
        "is_null({a: (p(x))})",
        "p(count(p(x)))",
    ] {
        let source = format!("rule p : x if x = 1 and {body}");
        let error = compile_source_document_v3(SourceIdV2::from_raw(406), &source, &authority)
            .expect_err("scalar-wrapped source call cannot produce a compiled plan");
        let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) =
            error
        else {
            panic!("{body}: expected a source-anchored document failure");
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("{body}: expected one typed failure");
        };
        assert_eq!(diagnostic.code(), "sema.invalid_item_shape", "{body}");
    }

    for body in ["not p(x)", "not (p(x))", "not ((p(x)))"] {
        let source = format!("rule p : x if x = 1 and {body}");
        let error = compile_source_document_v3(SourceIdV2::from_raw(406), &source, &authority)
            .expect_err("negative recursive SCC cannot produce a compiled plan");
        let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) =
            error
        else {
            panic!("{body}: expected a source-anchored document failure");
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("{body}: expected one logic failure");
        };
        assert_eq!(diagnostic.code(), "logic.negation_cycle", "{body}");
    }
}

#[test]
fn both_not_equal_surface_spellings_resolve_to_the_same_closed_operator() {
    let authority = closed_authority();
    for spelling in ["<>", "!="] {
        let source = format!("from findings f | where f.score {spelling} 1");
        let document = compile_source_document_v3(SourceIdV2::from_raw(407), &source, &authority)
            .unwrap_or_else(|error| {
                panic!("{spelling}: surface not-equal must compile: {error:?}")
            });
        let [item] = document.execution().items() else {
            panic!("{spelling}: expected one executable relation");
        };
        let relation = item.relation().expect("filtered relation");
        let comparisons = relation
            .expressions()
            .iter()
            .filter_map(|expression| {
                if let ExecutionExpressionShapeV3::Compare { operator, .. } = expression.shape() {
                    Some(operator)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let [operator] = comparisons.as_slice() else {
            panic!("{spelling}: expected one not-equal comparison");
        };
        assert_eq!(operator.symbol(), "!=", "{spelling}");
        assert_eq!(operator.overload_id(), 321, "{spelling}");
    }
}

#[test]
fn parenthesized_i128_minimum_compiles_but_positive_magnitude_rejects() {
    let authority = closed_authority();
    let magnitude = "170141183460469231731687303715884105728";
    for expression in [
        format!("-{magnitude}"),
        format!("-({magnitude})"),
        format!("-(({magnitude}))"),
    ] {
        let source = format!("let minimum = {expression}");
        let compiled = compile_source_document_v3(SourceIdV2::from_raw(408), &source, &authority)
            .unwrap_or_else(|error| panic!("{expression}: {error:?}"));
        assert_eq!(compiled.items().len(), 1, "{expression}");
    }
    for expression in [magnitude.to_owned(), format!("+({magnitude})")] {
        let source = format!("let overflow = {expression}");
        let error = compile_source_document_v3(SourceIdV2::from_raw(408), &source, &authority)
            .expect_err("positive magnitude exceeds i128::MAX");
        let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) =
            error
        else {
            panic!("{expression}: expected document failure");
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("{expression}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.code(),
            "types.integer_out_of_range",
            "{expression}"
        );
    }
}

#[test]
fn repeated_catalog_reference_uses_modules_canonical_dependency() {
    let authority = closed_authority();
    for source in [
        "profile strict-review | capabilities=graph.read | admission=certified-only",
        "profile strict-review | capabilities=graph.read,graph.read | admission=certified-only",
        "profile strict-review | capabilities=graph.read,graph.trace,graph.read | admission=certified-only",
        "profile strict-review | capabilities=graph.trace,graph.read,graph.trace | admission=certified-only",
        "gate release-gate | checks=owasp.sql,owasp.sql | binding=all",
        "gate release-gate | checks=authz.path,owasp.sql,authz.path | binding=all",
    ] {
        let document = compile_source_document_v3(SourceIdV2::from_raw(409), source, &authority)
            .unwrap_or_else(|error| {
                panic!("modules-admitted catalog reference must remain admitted: {error:?}")
            });
        assert_eq!(document.items().len(), 1, "{source}");
        assert_eq!(document.execution().items().len(), 0, "{source}");
    }
    let source = "profile strict-review | capabilities=graph.read,absent.capability,graph.read | admission=certified-only";
    let error = compile_source_document_v3(SourceIdV2::from_raw(409), source, &authority)
        .expect_err("deduplicating a known edge must not admit an unknown target");
    let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) = error
    else {
        panic!("expected document failure");
    };
    let [diagnostic] = failure.diagnostics() else {
        panic!("expected one unresolved target diagnostic");
    };
    assert_eq!(diagnostic.code(), "modules.unknown_catalog_reference");
}

#[test]
fn supplemental_units_are_explicit_and_unsupported_source_fails_closed() {
    let authority = closed_authority();
    assert_eq!(SUPPLEMENTAL_SOURCES_V2.len(), 2);
    let value = compile_source_document_v3(
        SourceIdV2::from_raw(300),
        SUPPLEMENTAL_SOURCES_V2[0],
        &authority,
    )
    .expect("value declaration compiles");
    let [CompiledItemV2::NoPlan {
        item_id: value_id,
        reason: value_reason,
    }] = value.items()
    else {
        panic!("value declaration must have exactly one checked no-plan item");
    };
    assert_eq!(value_reason.code(), "value_declaration");
    assert_eq!(value.item(*value_id), value.items().first());
    let pair = compile_source_document_v3(
        SourceIdV2::from_raw(303),
        "let first = 1; let second = 2",
        &authority,
    )
    .expect("two value declarations compile");
    let [first, second] = pair.items() else {
        panic!("expected two source-ordered compiled items");
    };
    assert_ne!(first.item_id(), second.item_id());
    assert_eq!(pair.item(first.item_id()), Some(first));
    assert_eq!(pair.item(second.item_id()), Some(second));
    let absent = SemanticItemIdV2::new(SourceIdV2::from_raw(304), first.item_id().hir_item_id());
    assert!(
        pair.item(absent).is_none(),
        "foreign compiled item must miss"
    );
    assert!(value.rule_program().is_none());
    assert!(value.execution().items().is_empty());

    let rule = compile_source_document_v3(
        SourceIdV2::from_raw(301),
        SUPPLEMENTAL_SOURCES_V2[1],
        &authority,
    )
    .expect("rule compiles");
    let [CompiledItemV2::Planned(planned)] = rule.items() else {
        panic!("rule declaration must have exactly one checked member item");
    };
    let PlannedItemV2::RuleMember(member) = planned.as_ref() else {
        panic!("rule declaration must retain its program member reference");
    };
    let program = rule.rule_program().expect("one document-wide rule program");
    assert_eq!(program.members(), [*member]);
    assert_eq!(member.member_item_id(), planned.item_id());
    assert_eq!(member.program_id(), program.program_id());
    assert!(rule.execution().items().is_empty());

    let error =
        compile_source_document_v3(SourceIdV2::from_raw(302), UNSUPPORTED_SOURCE_V2, &authority)
            .expect_err("unsupported root must fail before a compiled document exists");
    let quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure) = error
    else {
        panic!("unsupported source must fail in the document transaction");
    };
    let [diagnostic] = failure.diagnostics() else {
        panic!("unsupported source must emit exactly one diagnostic");
    };
    assert_eq!(diagnostic.code(), "surface.parse.unexpected_token");
    assert_eq!(diagnostic.primary().source_id, SourceIdV2::from_raw(302));
    assert_eq!(diagnostic.primary().span.start, 0);
    assert_eq!(diagnostic.primary().span.end, 7);
}
