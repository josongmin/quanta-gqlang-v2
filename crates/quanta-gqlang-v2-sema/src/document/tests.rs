#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner assertions inspect exact transaction shapes and fail on any mismatch"
)]

use super::*;

#[test]
fn partition_candidate_shape_rejects_every_transplanted_identity_dimension() {
    let reference = HirNodeIdV1::from_raw(11);
    let group = HirNodeIdV1::from_raw(7);
    let outer = HirNodeIdV1::from_raw(13);
    let inner = HirNodeIdV1::from_raw(17);
    let span = SpanV1::new(40, 41);
    let witnesses = BTreeMap::from([(
        reference,
        PartitionCandidateShapeV2 {
            span,
            direct_group: group,
            call_arguments: vec![outer, inner],
        },
    )]);
    assert!(
        check_partition_candidate_shape(&witnesses, reference, span, group, &[outer, inner],)
            .is_ok()
    );
    for (candidate_reference, candidate_span, candidate_group, candidate_calls) in [
        (HirNodeIdV1::from_raw(12), span, group, vec![outer, inner]),
        (reference, SpanV1::new(41, 42), group, vec![outer, inner]),
        (
            reference,
            span,
            HirNodeIdV1::from_raw(8),
            vec![outer, inner],
        ),
        (
            reference,
            span,
            group,
            vec![HirNodeIdV1::from_raw(14), inner],
        ),
        (reference, span, group, vec![outer]),
        (
            reference,
            span,
            group,
            vec![outer, HirNodeIdV1::from_raw(18)],
        ),
    ] {
        assert!(check_partition_candidate_shape(
            &witnesses,
            candidate_reference,
            candidate_span,
            candidate_group,
            &candidate_calls,
        )
        .is_err());
    }
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "owner test assertions must fail on a budget mismatch"
)]
fn semantic_steps_fail_at_exact_bound_with_source_location() -> Result<(), String> {
    let budget = FrontendBudgetV2::try_new(
        10, 10, 10, 10, 10, 10, 10, 10, 10, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
    )
    .map_err(|error| format!("{error:?}"))?;
    let mut steps = SemanticStepCounterV2::new(&budget);
    let source_id = SourceIdV2::from_raw(7);
    let span = SpanV1::new(1, 2);
    assert!(steps.charge(source_id, span).is_ok());
    assert!(steps.charge(source_id, span).is_ok());
    let error = steps
        .charge(source_id, span)
        .err()
        .ok_or("third semantic node must exceed two-step bound")?;
    let diagnostic = error
        .diagnostics()
        .first()
        .ok_or("resource failure must have one diagnostic")?;
    assert_eq!(diagnostic.code(), "frontend.resource_limit_exceeded");
    assert_eq!(diagnostic.primary(), location(source_id, span));
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "owner test assertions must reject forged incomplete states"
)]
fn incomplete_analysis_canonicalizes_valid_input_and_rejects_invalid_states() -> Result<(), String>
{
    let source = SourceIdV2::from_raw(9);
    let earlier = UnresolvedSemanticFactV2::new("sema.a", location(source, SpanV1::new(1, 2)));
    let later = UnresolvedSemanticFactV2::new("sema.b", location(source, SpanV1::new(3, 4)));
    let warning = FrontendDiagnosticV2::new(
        FrontendStageV2::Resolution,
        "sema.warning",
        FrontendSeverityV2::Warning,
        location(source, SpanV1::new(3, 4)),
        Vec::new(),
        "warning",
    );
    let note = FrontendDiagnosticV2::new(
        FrontendStageV2::Resolution,
        "sema.note",
        FrontendSeverityV2::Note,
        location(source, SpanV1::new(1, 2)),
        Vec::new(),
        "note",
    );
    let canonical = IncompleteSemanticDocumentV2::try_new(
        source,
        vec![later.clone(), earlier.clone()],
        vec![warning.clone(), note.clone()],
    )
    .map_err(str::to_owned)?;
    assert_eq!(canonical.unresolved(), &[earlier.clone(), later]);
    assert_eq!(canonical.diagnostics(), &[note, warning.clone()]);
    assert!(IncompleteSemanticDocumentV2::try_new(
        source,
        vec![earlier.clone(), earlier],
        Vec::new(),
    )
    .is_err());
    assert!(IncompleteSemanticDocumentV2::try_new(
        source,
        vec![UnresolvedSemanticFactV2::new(
            "sema.foreign",
            location(SourceIdV2::from_raw(10), SpanV1::new(1, 2)),
        )],
        Vec::new(),
    )
    .is_err());
    assert!(IncompleteSemanticDocumentV2::try_new(
        source,
        vec![UnresolvedSemanticFactV2::new(
            "sema.live",
            location(source, SpanV1::new(1, 2))
        )],
        vec![warning.clone(), warning],
    )
    .is_err());
    assert!(IncompleteSemanticDocumentV2::try_new(
        source,
        vec![UnresolvedSemanticFactV2::new(
            "sema.live",
            location(source, SpanV1::new(1, 2))
        )],
        vec![FrontendDiagnosticV2::new(
            FrontendStageV2::Validation,
            "sema.error",
            FrontendSeverityV2::Error,
            location(source, SpanV1::new(1, 2)),
            Vec::new(),
            "error",
        )],
    )
    .is_err());
    Ok(())
}

// ---- SS-03H-S + SS-04W owner oracles ---------------------------------

use quanta_gqlang_v2_hir::{
    AliasV1, CallArgHirV1, CallArgsHirV1, CatalogDeclHirV1, CatalogPayloadV1, CompositionBindingV1,
    CompositionBodyV1, CompositionHirV1, CompositionProducerV1, DeclHirV1, GraphRefV1,
    HirBuilderV1, LocatedSpanV2, PathSegmentV1, ProjectionItemV1, RelationExistsPolarityV1,
    RenamePhaseV3, SourceRegistrationV2, SourceRegistryViewV2, ViewBodyV1,
};
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    VersionV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2,
    RelationSourceInputV2,
};

struct EmptyModuleGraphV1;
impl ModuleGraphViewV1 for EmptyModuleGraphV1 {
    fn imported_targets(&self, _name: &str) -> Vec<String> {
        Vec::new()
    }
}

fn owner_authority(budget: &FrontendBudgetV2) -> Result<ClosedFrontendAuthorityV2, String> {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 512,
    }])
    .map_err(|error| format!("{error:?}"))?;
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let module = ModuleIdV2::try_new("app", location.clone()).map_err(|e| format!("{e:?}"))?;
    let profile = budget
        .authority_profile()
        .map_err(|error| format!("{error:?}"))?;
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone())
        .map_err(|error| format!("{error:?}"))?;
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .map_err(|error| format!("{error:?}"))?;
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .map_err(|error| error.to_string())?;
    let field = RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location.clone())
        .map_err(|error| format!("{error:?}"))?;
    let row = RecordTypeV1::try_new(vec![field]).map_err(|error| format!("{error:?}"))?;
    let relations = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
            provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone())
                .map_err(|error| format!("{error:?}"))?,
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
    .map_err(|error| format!("{error:?}"))?;
    let graphs = GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile)
        .map_err(|error| format!("{error:?}"))?;
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget.clone())
        .map_err(|error| format!("{error:?}"))
}

#[test]
fn show_source_declaration_keeps_exact_host_identity_and_rejects_adjacent_forms() {
    use quanta_gqlang_v2_hir::{PresentationHirV1, PresentationKindV1, ShowCategoryV1};

    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let authority = owner_authority(&budget).unwrap();
    let source_id = SourceIdV2::from_raw(2);
    let show = |id| {
        ProgramItemHirV1::from(PresentationHirV1::new(
            HirNodeIdV1::from_raw(id),
            SpanV1::new(0, 11),
            PresentationKindV1::Show {
                category: ShowCategoryV1::Models,
            },
        ))
    };
    let syntax = || HirDocumentV1 {
        source_id,
        source_len: 11,
        items: vec![show(1), show(2)],
    };
    let checked = CheckedSyntaxDocumentV2::try_from_show_hir_monotonic_v2(syntax()).unwrap();
    let declared = DeclaredShowSourceV2::try_new(checked, &authority).unwrap();
    assert_eq!(declared.source_id(), source_id);
    assert_eq!(declared.item_count(), 2);
    assert_eq!(declared.source_len(), Some(11));
    assert_eq!(
        declared.source_anchor().location(),
        location(source_id, SpanV1::point(0))
    );
    assert_ne!(
        declared.source_anchor().registry_identity(),
        &authority.modules().source_registry().identity()
    );

    let one_item_budget = FrontendBudgetV2::try_new(
        1024, 1024, 1024, 1, 8, 8, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024,
        1024, 1024, 8, 1024, 1024, 1024,
    )
    .unwrap();
    let one_item_authority = owner_authority(&one_item_budget).unwrap();
    let checked = CheckedSyntaxDocumentV2::try_from_show_hir_monotonic_v2(syntax()).unwrap();
    assert_eq!(
        DeclaredShowSourceV2::try_new(checked, &one_item_authority)
            .unwrap_err()
            .diagnostics()[0]
            .code(),
        "frontend.resource_limit_exceeded"
    );

    let adjacent = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id,
        source_len: 11,
        items: vec![show(2), show(1)],
    })
    .unwrap();
    assert_eq!(
        DeclaredShowSourceV2::try_new(adjacent, &authority)
            .unwrap_err()
            .diagnostics()[0]
            .code(),
        "sema.show_source_incomplete"
    );
    let collision = CheckedSyntaxDocumentV2::try_from_show_hir_monotonic_v2(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 11,
        items: vec![show(1)],
    })
    .unwrap();
    assert_eq!(
        DeclaredShowSourceV2::try_new(collision, &authority)
            .unwrap_err()
            .diagnostics()[0]
            .code(),
        "hir.source_registry.duplicate_source_id"
    );
}

fn rename_authority() -> Result<ClosedFrontendAuthorityV2, String> {
    let host_source = SourceIdV2::from_raw(1);
    let host_location = crate::FrontendAuthorityLocationV2 {
        source_id: host_source,
        start: 0,
        end: 1,
    };
    let mut builder = crate::ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?,
        vec![crate::FrontendAuthoritySourceV2 {
            source_id: host_source,
            source_len: 64,
        }],
        "app".to_owned(),
        host_location,
        crate::FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .map_err(|error| format!("{error:?}"))?;
    builder.add_symbol_target(crate::FrontendSymbolTargetInputV3 {
        selector: "symbol.one".to_owned(),
        repo_path: "src/a.py".to_owned(),
        symbol_id: "def:17".to_owned(),
        start_byte: 17,
        end_byte: 20,
        location: host_location,
    });
    builder.finish().map_err(|error| format!("{error:?}"))
}

fn raw_rename_document(phase: RenamePhaseV3, preview_token: Option<String>) -> HirDocumentV1 {
    let builder = HirBuilderV1::new();
    let command = builder.rename_command(
        GraphRefV1 {
            name: "symbol.one".to_owned(),
            span: SpanV1::new(7, 17),
        },
        "renamed".to_owned(),
        phase,
        "snap-7".to_owned(),
        preview_token,
        SpanV1::new(0, 32),
    );
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 64,
        items: vec![ProgramItemHirV1::from(command)],
    }
}

#[test]
fn raw_hir_rename_enforces_exact_phase_token_combinations() -> Result<(), String> {
    let authority = rename_authority()?;
    for (phase, token, expected_code) in [
        (RenamePhaseV3::Preview, None, None),
        (RenamePhaseV3::Apply, Some("issued".to_owned()), None),
        (
            RenamePhaseV3::Preview,
            Some(String::new()),
            Some("sema.invalid_rename_command"),
        ),
        (
            RenamePhaseV3::Preview,
            Some("issued".to_owned()),
            Some("sema.invalid_rename_command"),
        ),
        (
            RenamePhaseV3::Apply,
            None,
            Some("sema.invalid_rename_command"),
        ),
        (
            RenamePhaseV3::Apply,
            Some(String::new()),
            Some("sema.invalid_rename_command"),
        ),
    ] {
        let checked = CheckedSyntaxDocumentV2::try_from_hir(raw_rename_document(phase, token))
            .map_err(|error| format!("{error:?}"))?;
        let result = resolve_checked_document_v2(checked, &authority);
        match expected_code {
            None => {
                result.map_err(|error| format!("valid {phase:?} intent rejected: {error:?}"))?;
            }
            Some(expected) => match result {
                Err(FrontendTransactionErrorV2::Document(failure))
                    if failure
                        .diagnostics()
                        .iter()
                        .any(|diagnostic| diagnostic.code() == expected) => {}
                other => {
                    return Err(format!(
                        "invalid {phase:?} token state must fail with {expected}, got {other:?}"
                    ));
                }
            },
        }
    }
    Ok(())
}

/// One `from records as r | project count(*)` document: the legal scalar
/// aggregate shape, including the star call node identity.
fn scalar_count_star_document() -> (HirDocumentV1, HirNodeIdV1, HirNodeIdV1) {
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
    let scan_id = scan.id;
    let count = builder.call(
        "count",
        SpanV1::new(24, 29),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
            span: SpanV1::new(30, 31),
        }])
        .unwrap(),
        SpanV1::new(24, 31),
    );
    let count_id = count.id;
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: count,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(36, 41),
                }),
            }],
            SpanV1::new(16, 41),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 41));
    (
        HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        },
        scan_id,
        count_id,
    )
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "owner assertions inspect the sealed scalar aggregate shape"
)]
fn scalar_count_star_seals_one_private_empty_partition_group() -> Result<(), String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let authority = owner_authority(&budget)?;
    let (document, scan_id, count_id) = scalar_count_star_document();
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(document).map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    let item = document
        .items()
        .next()
        .map(|(_, item)| item)
        .unwrap_or_else(|| {
            panic!("one relation item must be sealed");
        });
    let SemanticItemV2::Relation(relation) = item else {
        panic!("scalar aggregate must retain its typed relation item");
    };
    // Exact Group(empty) -> Project(aggregate) shape; no caller option.
    let TypedRelationExprV2::Project(_) = relation.root() else {
        panic!("aggregate projection root expected");
    };
    let group = relation.root().inputs().next().unwrap();
    let TypedRelationExprV2::Group(_) = group else {
        panic!("aggregate projection must consume one Group");
    };
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert!(group.output().partition_keys().is_empty());
    assert!(group.output().columns().is_empty());
    assert!(group.output().row().fields().is_empty());
    // Pre-fold input contract: the exact pre-fold scan stage with its row.
    let scan = group.inputs().next().unwrap();
    let TypedRelationExprV2::Scan(_) = scan else {
        panic!("implicit group must fold the exact prior scan");
    };
    assert_eq!(scan.key().node_id, scan_id);
    assert_eq!(scan.output().row().fields().len(), 1);
    // Aggregate output contract.
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
    assert!(relation.output().partition_keys().is_empty());
    assert!(relation.output().ordering().is_empty());
    assert_eq!(relation.output().columns().len(), 1);
    // Star retained and the resolved aggregate call stays a types fact.
    let count_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: count_id,
    };
    let count = document.expression(count_key).unwrap();
    let TypedExprShapeV2::Call { arguments, .. } = count.shape() else {
        panic!("count(*) must retain its call shape");
    };
    assert_eq!(arguments, &[TypedCallArgumentV2::Star]);
    assert_eq!(
        count.aggregate_context().aggregation(),
        AggregationClassV1::Aggregate
    );
    assert!(count.aggregate_context().uses_star());
    assert_eq!(
        relation.root().expressions().first().map(|key| key.node_id),
        Some(count_id)
    );
    Ok(())
}

#[test]
fn bare_star_outside_an_aggregate_call_is_rejected_by_the_position_contract() {
    let raw = location(SourceIdV2::from_raw(9), SpanV1::new(3, 4));
    // A bare star never carries aggregate effects, so the central
    // aggregate-position contract rejects it before any relation stage.
    let error = AggregateContextV2::try_new(
        AggregatePositionV2::Projection,
        AggregationClassV1::Scalar,
        true,
        raw,
    )
    .unwrap_err();
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_star"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(3, 4)
    );
}

#[test]
fn mixed_document_transaction_resolves_all_items() -> Result<(), String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let authority = owner_authority(&budget)?;
    let builder = HirBuilderV1::new();
    // Declaration.
    let decl = DeclHirV1::new(
        HirNodeIdV1::from_raw(900),
        SpanV1::new(0, 16),
        "total".into(),
        SpanV1::new(4, 9),
        builder.int("42", SpanV1::new(12, 14)),
    );
    // Plain query with a projection.
    let plain_scan = builder.relation_source(
        "records",
        SpanV1::new(24, 31),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(32, 33),
        }),
        SpanV1::new(24, 33),
    );
    let plain_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(44, 46),
            }],
            SpanV1::new(42, 46),
        )
        .unwrap();
    let plain_project = builder
        .try_project(
            plain_scan,
            vec![ProjectionItemV1 {
                value: plain_key,
                alias: None,
            }],
            SpanV1::new(36, 46),
        )
        .unwrap();
    let plain_query = builder.query(plain_project, None, SpanV1::new(20, 46));
    // Correlated relational EXISTS query.
    let outer_scan = builder.relation_source(
        "records",
        SpanV1::new(56, 63),
        Some(AliasV1 {
            name: "o".into(),
            span: SpanV1::new(64, 65),
        }),
        SpanV1::new(56, 65),
    );
    let inner_scan = builder.relation_source(
        "records",
        SpanV1::new(76, 83),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(84, 85),
        }),
        SpanV1::new(76, 85),
    );
    let inner_path = builder
        .try_path(
            builder.symbol("f", SpanV1::new(90, 91)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(92, 94),
            }],
            SpanV1::new(90, 94),
        )
        .unwrap();
    let outer_path = builder
        .try_path(
            builder.symbol("o", SpanV1::new(97, 98)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(99, 101),
            }],
            SpanV1::new(97, 101),
        )
        .unwrap();
    let predicate = builder.compare("=", inner_path, outer_path, SpanV1::new(90, 101));
    let inner_filter = builder.relation_filter(inner_scan, predicate, SpanV1::new(76, 104));
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(68, 105),
    );
    let outer_filter = builder.relation_filter(outer_scan, exists, SpanV1::new(56, 106));
    let exists_query = builder.query(outer_filter, None, SpanV1::new(50, 106));
    // Inline VIEW (a candidate for a second kernel pass).
    let inline = builder.query(
        builder.relation_source(
            "records",
            SpanV1::new(124, 131),
            None,
            SpanV1::new(124, 131),
        ),
        None,
        SpanV1::new(117, 131),
    );
    let view = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(200),
        SpanV1::new(110, 131),
        "v",
        SpanV1::new(115, 116),
        CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(inline))),
    );
    // Composition with one Query LET and a Query body.
    let let_scan = builder.relation_source(
        "records",
        SpanV1::new(150, 157),
        None,
        SpanV1::new(150, 157),
    );
    let let_query = builder.query(let_scan, None, SpanV1::new(144, 157));
    let body_scan =
        builder.relation_source("q", SpanV1::new(170, 171), None, SpanV1::new(170, 171));
    let body_query = builder.query(body_scan, None, SpanV1::new(164, 171));
    let composition = CompositionHirV1::new(
        HirNodeIdV1::from_raw(300),
        SpanV1::new(136, 171),
        vec![CompositionBindingV1::new(
            "q",
            SpanV1::new(140, 141),
            CompositionProducerV1::Query(Box::new(let_query)),
        )],
        "q",
        SpanV1::new(166, 167),
        CompositionBodyV1::Query(Box::new(body_query)),
    );
    let source_id = SourceIdV2::from_raw(9);
    let expected_order = [
        SemanticItemIdV2::new(source_id, decl.id),
        SemanticItemIdV2::new(source_id, plain_query.id),
        SemanticItemIdV2::new(source_id, exists_query.id),
        SemanticItemIdV2::new(source_id, view.id),
        SemanticItemIdV2::new(source_id, composition.id),
    ];
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 512,
            items: vec![
                ProgramItemHirV1::Decl(Box::new(decl)),
                ProgramItemHirV1::Query(Box::new(plain_query)),
                ProgramItemHirV1::Query(Box::new(exists_query)),
                ProgramItemHirV1::CatalogDecl(Box::new(view)),
                ProgramItemHirV1::Composition(Box::new(composition)),
            ],
        })
        .map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    if document.order() != expected_order {
        return Err(format!(
            "mixed document item order differs: expected {expected_order:?}, got {:?}",
            document.order()
        ));
    }
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "forged witnesses must fail the cross-owner equality validator"
)]
fn capture_vector_forgery_matrix_rejects_every_mutated_witness() -> Result<(), String> {
    let builder = HirBuilderV1::new();
    let outer_scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let inner_scan = builder.relation_source(
        "records",
        SpanV1::new(25, 32),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(33, 34),
        }),
        SpanV1::new(25, 34),
    );
    let inner_path = builder
        .try_path(
            builder.symbol("f", SpanV1::new(38, 39)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(40, 42),
            }],
            SpanV1::new(38, 42),
        )
        .unwrap();
    let captured = builder
        .try_path(
            builder.symbol("r", SpanV1::new(45, 46)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(47, 49),
            }],
            SpanV1::new(45, 49),
        )
        .unwrap();
    let predicate = builder.compare("=", inner_path, captured, SpanV1::new(38, 49));
    let inner_filter = builder.relation_filter(inner_scan, predicate, SpanV1::new(25, 52));
    let inner_filter_id = inner_filter.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 53),
    );
    let exists_id = exists.id;
    let outer_filter = builder.relation_filter(outer_scan.clone(), exists, SpanV1::new(5, 54));
    let query = builder.query(outer_filter, None, SpanV1::new(0, 54));
    let mut binder = Binder::new(&EmptyModuleGraphV1);
    binder.declare_program(&[], &[]);
    binder.resolve_query(&query);
    let bound = binder.finish();
    assert!(bound.is_ok(), "{:?}", bound.diagnostics());
    let witness = capture_vector_witness(&bound, exists_id, inner_filter_id);
    assert_eq!(witness.edges.len(), 1, "one exact capture edge expected");
    // Positive: the untampered Binder witness validates.
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &witness, &bound)
            .map_err(str::to_owned)?,
        witness.edges
    );
    // Dropped edge.
    let mut dropped = witness.clone();
    dropped.edges.pop();
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &dropped, &bound).err(),
        Some("capture vector dropped or added an edge")
    );
    // Duplicate reference.
    let mut duplicate = witness.clone();
    duplicate.edges.push(duplicate.edges[0]);
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &duplicate, &bound).err(),
        Some("duplicate capture reference in witness")
    );
    // Reversed two-capture order: a second edge on the same binding.
    let second = builder
        .try_path(
            builder.symbol("r", SpanV1::new(57, 58)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(59, 61),
            }],
            SpanV1::new(57, 61),
        )
        .unwrap();
    let lhs = builder
        .try_path(
            builder.symbol("g", SpanV1::new(64, 65)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(66, 68),
            }],
            SpanV1::new(64, 68),
        )
        .unwrap();
    // A second, distinct reference to the same outer alias: the capture
    // vector must retain two ordered edges, one per authored reference.
    let second_again = builder
        .try_path(
            builder.symbol("r", SpanV1::new(91, 92)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(93, 95),
            }],
            SpanV1::new(91, 95),
        )
        .unwrap();
    let lhs_again = builder
        .try_path(
            builder.symbol("g", SpanV1::new(96, 97)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(98, 100),
            }],
            SpanV1::new(96, 100),
        )
        .unwrap();
    let second_predicate = builder.and(
        builder.compare("=", lhs, second, SpanV1::new(64, 68)),
        builder.compare("=", lhs_again, second_again, SpanV1::new(91, 100)),
        SpanV1::new(64, 100),
    );
    let second_inner = builder.relation_filter(
        builder.relation_source(
            "records",
            SpanV1::new(70, 77),
            Some(AliasV1 {
                name: "g".into(),
                span: SpanV1::new(78, 79),
            }),
            SpanV1::new(70, 79),
        ),
        second_predicate,
        SpanV1::new(70, 82),
    );
    let second_inner_id = second_inner.id;
    let second_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        second_inner,
        SpanV1::new(84, 85),
    );
    let second_exists_id = second_exists.id;
    let second_outer =
        builder.relation_filter(outer_scan.clone(), second_exists, SpanV1::new(5, 90));
    let second_query = builder.query(second_outer, None, SpanV1::new(0, 90));
    let mut second_binder = Binder::new(&EmptyModuleGraphV1);
    second_binder.declare_program(&[], &[]);
    second_binder.resolve_query(&second_query);
    let second_bound = second_binder.finish();
    assert!(second_bound.is_ok(), "{:?}", second_bound.diagnostics());
    let two = capture_vector_witness(&second_bound, second_exists_id, second_inner_id);
    assert_eq!(two.edges.len(), 2, "two exact capture edges expected");
    let mut reversed = two.clone();
    reversed.edges.reverse();
    assert_eq!(
        validate_capture_vector(second_exists_id, second_inner_id, &reversed, &second_bound).err(),
        Some("capture edge reference differs from Binder proof")
    );
    assert_eq!(
        validate_capture_vector(second_exists_id, second_inner_id, &two, &second_bound)
            .map_err(str::to_owned)?,
        two.edges
    );
    // Wrong owner.
    let mut wrong_owner = witness.clone();
    wrong_owner.edges[0].owner = HirNodeIdV1::from_raw(999);
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &wrong_owner, &bound).err(),
        Some("capture edge owner differs from Binder proof")
    );
    // Wrong ordinal.
    let mut wrong_ordinal = witness.clone();
    wrong_ordinal.edges[0].ordinal += 1;
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &wrong_ordinal, &bound).err(),
        Some("capture edge ordinal differs from Binder proof")
    );
    // Foreign reference node.
    let mut foreign = witness.clone();
    foreign.edges[0].reference = HirNodeIdV1::from_raw(4242);
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &foreign, &bound).err(),
        Some("capture edge reference differs from Binder proof")
    );
    // Wrong nested relation root.
    let mut wrong_root = witness.clone();
    wrong_root.relation_root = HirNodeIdV1::from_raw(777);
    assert_eq!(
        validate_capture_vector(exists_id, inner_filter_id, &wrong_root, &bound).err(),
        Some("capture witness nested relation root differs from checked HIR")
    );
    // Empty-set omission: an EXISTS node Binder never visited.
    assert_eq!(
        validate_capture_vector(
            HirNodeIdV1::from_raw(888),
            inner_filter_id,
            &witness,
            &bound
        )
        .err(),
        Some("Binder omitted the explicit relational EXISTS capture set")
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "finalizer mutations must fail before document assembly"
)]
fn nested_relation_finalizer_rejects_dropped_extra_and_forged_correlations() -> Result<(), String> {
    // Real transaction output: the finalizer consumes the sealed maps.
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let authority = owner_authority(&budget)?;
    let builder = HirBuilderV1::new();
    let outer_scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let inner_scan = builder.relation_source(
        "records",
        SpanV1::new(25, 32),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(33, 34),
        }),
        SpanV1::new(25, 34),
    );
    let inner_path = builder
        .try_path(
            builder.symbol("f", SpanV1::new(38, 39)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(40, 42),
            }],
            SpanV1::new(38, 42),
        )
        .unwrap();
    let captured = builder
        .try_path(
            builder.symbol("r", SpanV1::new(45, 46)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(47, 49),
            }],
            SpanV1::new(45, 49),
        )
        .unwrap();
    let predicate = builder.compare("=", inner_path, captured, SpanV1::new(38, 49));
    let inner_filter = builder.relation_filter(inner_scan, predicate, SpanV1::new(25, 52));
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 53),
    );
    let outer_filter = builder.relation_filter(outer_scan, exists, SpanV1::new(5, 54));
    let query = builder.query(outer_filter, None, SpanV1::new(0, 54));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 96,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    let source = document.source_id();
    let mut expressions = BTreeMap::new();
    for (key, value) in document.expressions() {
        expressions.insert(*key, value.clone());
    }
    let nested = document
        .nested_relations()
        .map(|(key, value)| (*key, value.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(nested.len(), 1, "one nested relation expected");
    let identity = document
        .nested_relations()
        .next()
        .unwrap()
        .1
        .location()
        .registry_identity()
        .clone();
    // Positive: the sealed transaction finalizes.
    check_nested_relation_finalizer(source, &identity, &expressions, &nested)
        .map_err(str::to_owned)?;
    // Dropped nested root.
    let root = *nested.keys().next().unwrap();
    let mut dropped = nested.clone();
    dropped.remove(&root);
    assert_eq!(
        check_nested_relation_finalizer(source, &identity, &expressions, &dropped).err(),
        Some("nested relation registry differs from relational EXISTS expression facts")
    );
    // Extra nested root: a registry-consistent scan under an absent root.
    let extra_binding = ResolvedValueBindingV2::try_new(
        source,
        HirNodeIdV1::from_raw(700),
        0,
        ResolvedBindingKindV2::Composition,
        "q".to_owned(),
    )
    .map_err(|error| error.to_string())?;
    let extra_source = ResolvedScanSourceV2::Composition(
        ResolvedCompositionSourceIdV2::try_from_binding(extra_binding)
            .map_err(|error| error.to_string())?,
    );
    let extra_output = RelationOutputContractV2::try_new(
        RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
        OutputProvenanceV2::new([extra_source.clone()]),
        Vec::new(),
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(str::to_owned)?;
    let extra_scan = TypedRelationExprV2::scan(
        quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id: source,
            node_id: HirNodeIdV1::from_raw(701),
        },
        document
            .nested_relations()
            .next()
            .unwrap()
            .1
            .location()
            .clone(),
        extra_source,
        extra_output,
    )
    .map_err(str::to_owned)?;
    let mut extra = nested.clone();
    extra.insert(
        quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id: source,
            node_id: HirNodeIdV1::from_raw(701),
        },
        extra_scan,
    );
    assert_eq!(
        check_nested_relation_finalizer(source, &identity, &expressions, &extra).err(),
        Some("nested relation registry differs from relational EXISTS expression facts")
    );
    // Dropped correlation-reference expression.
    let correlation_reference = expressions
        .values()
        .find_map(|expression| {
            let TypedExprShapeV2::RelationExists { correlations, .. } = expression.shape() else {
                return None;
            };
            correlations.first().map(|edge| edge.reference())
        })
        .unwrap();
    let mut no_reference = expressions.clone();
    no_reference.remove(&correlation_reference);
    assert_eq!(
        check_nested_relation_finalizer(source, &identity, &no_reference, &nested).err(),
        Some("correlation reference is foreign to this document's expressions")
    );
    // Forged correlation witnesses against the real expression keys.
    let keys = expressions.keys().copied().collect::<BTreeSet<_>>();
    let real = CorrelationWitnessV2 {
        reference: correlation_reference,
        binding_source: source,
        registry: identity.clone(),
    };
    check_correlation_witnesses(source, &identity, &keys, std::slice::from_ref(&real))
        .map_err(str::to_owned)?;
    let foreign = CorrelationWitnessV2 {
        reference: quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: HirNodeIdV1::from_raw(4242),
        },
        binding_source: source,
        registry: identity.clone(),
    };
    assert_eq!(
        check_correlation_witnesses(source, &identity, &keys, &[foreign]).err(),
        Some("correlation reference is foreign to this document's expressions")
    );
    assert_eq!(
        check_correlation_witnesses(source, &identity, &keys, &[real.clone(), real.clone()]).err(),
        Some("duplicate correlation reference in one nested relation")
    );
    // Cross-registry checked location: a second host registry admitting
    // the same document source yields a different registry identity.
    let other_host = SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 512,
        },
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(2),
            source_len: 8,
        },
    ])
    .map_err(|error| format!("{error:?}"))?;
    let cross = CorrelationWitnessV2 {
        reference: correlation_reference,
        binding_source: source,
        registry: other_host.identity().clone(),
    };
    assert_eq!(
        check_correlation_witnesses(source, &identity, &keys, &[cross]).err(),
        Some("correlation reference location belongs to another source registry")
    );
    Ok(())
}
