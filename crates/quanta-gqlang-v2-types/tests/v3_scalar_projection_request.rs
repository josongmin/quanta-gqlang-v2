#![expect(clippy::unwrap_used, reason = "independent request owner assertions")]

use quanta_gqlang_v2_hir::{
    AliasV1, CatalogPayloadV1, CheckedHirDocumentOwnedV2, FrontendRequestMaterializationRootV3,
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
    HirBuilderV1, HirDocumentV1, LocatedSpanV2, PathSegmentV1, ProgramItemHirV1, ProjectionItemV1,
    SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1, ViewBodyV1,
};
use quanta_gqlang_v2_types::CardinalityV1;
use quanta_gqlang_v2_types::{
    check_scalar_projection_with_request_owner_v3, scalar_projection_type_storage_upper_v3,
    AdmittedBuiltinRegistryV3, BuiltinRegistryV2, DeferredRelationRowFieldInputV2,
    DeferredRelationRowInputV2, ExpressionTypeKernelV2, RecordFieldTypeV1, RecordTypeV1,
    RelationRowFieldSourceV2, ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2,
    ResolvedTypeInputsV2, SourceQualifiedNodeIdV2, TypeFactsReaderV3, ValueTypeV1,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Ledger {
    live: AtomicU64,
}
struct Sink(Arc<Ledger>);
struct Charge {
    ledger: Arc<Ledger>,
    bytes: u64,
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.ledger.live.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}
impl FrontendRequestStorageSinkV3 for Sink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        self.0.live.fetch_add(logical_bytes, Ordering::SeqCst);
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
fn finite_request_scalar_projection_matches_legacy_owner_for_one_and_nine_fields() {
    for count in [1_usize, 9] {
        compare_projection(count, false);
    }
    compare_projection(1, true);
}

const HOST_FIELDS: [(&str, bool, bool); 9] = [
    ("row_kind", false, false),
    ("scope_id", false, false),
    ("summary_symbol_id", true, false),
    ("summary_source_scope", true, false),
    ("summary_effect_axes", false, true),
    ("edge_caller_symbol", true, false),
    ("edge_callee_symbol", true, false),
    ("edge_effect_axis", true, false),
    ("degraded_scope", true, false),
];

#[test]
fn nine_field_host_scan_preserves_optional_sequence_but_only_selects_scalar() {
    for (selected, accepted) in [
        ("row_kind", true),
        ("summary_effect_axes", false),
        ("summary_symbol_id", false),
    ] {
        let source = "x".repeat(512);
        let source_id = SourceIdV2::from_raw(if accepted { 201 } else { 202 });
        let b = HirBuilderV1::new();
        let span = SpanV1::new(0, 100);
        let scan = b.relation_source(
            "effect",
            SpanV1::new(0, 6),
            Some(AliasV1 {
                name: "e".into(),
                span: SpanV1::new(7, 8),
            }),
            span,
        );
        let scan_id = scan.id;
        let base = b.symbol("e", SpanV1::new(10, 11));
        let path = b
            .try_path(
                base.clone(),
                vec![PathSegmentV1 {
                    name: selected.into(),
                    span: SpanV1::new(12, 31),
                }],
                SpanV1::new(10, 31),
            )
            .unwrap();
        let path_id = path.id;
        let project = b
            .try_project(
                scan,
                vec![ProjectionItemV1 {
                    value: path,
                    alias: Some(AliasV1 {
                        name: "kind".into(),
                        span: SpanV1::new(32, 36),
                    }),
                }],
                span,
            )
            .unwrap();
        let project_id = project.id;
        let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
            source_id,
            source_len: source.len(),
            items: vec![ProgramItemHirV1::Query(Box::new(
                b.query(project, None, span),
            ))],
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
                span: SpanV1::new(10, 11),
            })
            .unwrap();
        let fields = HOST_FIELDS
            .into_iter()
            .map(|(name, optional, sequence)| {
                let value = if optional {
                    ValueTypeV1::try_optional(ValueTypeV1::string()).unwrap()
                } else if sequence {
                    ValueTypeV1::try_collection(ValueTypeV1::string(), CardinalityV1::Sequence)
                        .unwrap()
                } else {
                    ValueTypeV1::string()
                };
                RecordFieldTypeV1::try_new(name, value, location.clone()).unwrap()
            })
            .collect();
        let row = RecordTypeV1::try_new(fields).unwrap();
        assert_eq!(row.fields().len(), 9);
        assert_eq!(row.fields().get(4).unwrap().name(), "summary_effect_axes");
        let ledger = Arc::new(Ledger {
            live: AtomicU64::new(0),
        });
        let sink = Sink(ledger.clone());
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
        let parent = root.reserve_v3(source_id, &source, 1).unwrap();
        let builtins =
            BuiltinRegistryV2::try_closed_with_request_owner_v3(&root, source_id, &source, &parent)
                .unwrap();
        let schema_names = row.fields().iter().map(|field| field.name().len()).sum();
        // The final parameter is the *output name byte count*, not the two
        // HIR references. `kind` requires four retained bytes.
        let demand = scalar_projection_type_storage_upper_v3(
            source.len(),
            row.fields().len(),
            schema_names,
            1,
            "kind".len(),
        )
        .unwrap();
        let storage = root.reserve_v3(source_id, &source, demand).unwrap();
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
        if accepted {
            let request = result.unwrap();
            let key = |node_id| SourceQualifiedNodeIdV2 { source_id, node_id };
            assert_eq!(request.relation_row(key(scan_id)), Some(&row));
            assert_eq!(
                request
                    .relation_row(key(project_id))
                    .unwrap()
                    .fields()
                    .first()
                    .unwrap()
                    .name(),
                "kind"
            );
            assert!(request.get(key(path_id)).is_some());
            assert!(request.get(key(base.id)).is_some());
            let source_fields = row
                .fields()
                .iter()
                .enumerate()
                .map(|(ordinal, field)| {
                    DeferredRelationRowFieldInputV2::new(
                        ordinal,
                        field.name().to_owned(),
                        location.clone(),
                        RelationRowFieldSourceV2::Known(field.value_type().clone()),
                    )
                })
                .collect();
            let project_fields = vec![DeferredRelationRowFieldInputV2::new(
                0,
                "kind".to_owned(),
                location.clone(),
                RelationRowFieldSourceV2::PriorExpression(key(path_id)),
            )];
            let inputs = ResolvedTypeInputsV2::try_new(
                checked.as_checked(),
                &extended,
                vec![ResolvedTypeInputEntryV2 {
                    key: key(base.id),
                    value: ResolvedTypeInputValueV2::Binding(
                        quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(
                            ValueTypeV1::try_record(row.clone()).unwrap(),
                        ),
                    ),
                }],
                Vec::new(),
                vec![
                    DeferredRelationRowInputV2::known_source(key(scan_id), 9, source_fields),
                    DeferredRelationRowInputV2::expression_fields(
                        key(project_id),
                        1,
                        project_fields,
                    ),
                ],
            )
            .unwrap();
            let ordinary = ExpressionTypeKernelV2::try_closed()
                .unwrap()
                .check_document(&inputs)
                .unwrap();
            assert_eq!(
                request.relation_row(key(scan_id)),
                ordinary.relation_row(key(scan_id))
            );
            assert_eq!(
                request.relation_row(key(project_id)),
                ordinary.relation_row(key(project_id))
            );
            assert_eq!(request.get(key(path_id)), ordinary.get(key(path_id)));
            drop(parent);
            assert!(ledger.live.load(Ordering::SeqCst) > 0);
            drop(request);
        } else {
            assert!(
                result.is_err(),
                "selected Optional/Sequence must remain non-scalar"
            );
            drop(parent);
        }
        assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
    }
}

fn compare_projection(count: usize, bare_query: bool) {
    let source = "x".repeat(512);
    let source_id = SourceIdV2::from_raw(u32::try_from(count).unwrap());
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 100);
    let alias = AliasV1 {
        name: "r".into(),
        span: SpanV1::new(8, 9),
    };
    let scan = builder.relation_source("records", SpanV1::new(0, 7), Some(alias), span);
    let source_id_node = scan.id;
    let mut items = Vec::new();
    let mut base_ids = Vec::new();
    let mut path_ids = Vec::new();
    for index in (0..count).rev() {
        let base = builder.symbol("r", SpanV1::new(10, 11));
        base_ids.push(base.id);
        let path = builder
            .try_path(
                base,
                vec![PathSegmentV1 {
                    name: format!("f{index}"),
                    span: SpanV1::new(12, 14),
                }],
                SpanV1::new(10, 14),
            )
            .unwrap();
        path_ids.push(path.id);
        items.push(ProjectionItemV1 {
            value: path,
            alias: Some(AliasV1 {
                name: format!("o{index}"),
                span: SpanV1::new(15, 17),
            }),
        });
    }
    let projected = builder.try_project(scan, items, span).unwrap();
    let project_id = projected.id;
    let query = builder.query(projected, None, span);
    let item = if bare_query {
        ProgramItemHirV1::Query(Box::new(query))
    } else {
        ProgramItemHirV1::CatalogDecl(Box::new(builder.catalog_decl(
            "out",
            SpanV1::new(0, 3),
            CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(query))),
            span,
        )))
    };
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![item],
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
    let fields: Vec<_> = (0..count)
        .map(|index| {
            RecordFieldTypeV1::try_new(&format!("f{index}"), ValueTypeV1::int(), location.clone())
                .unwrap()
        })
        .collect();
    let row = RecordTypeV1::try_new(fields).unwrap();
    let ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
    });
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    let parent = root.reserve_v3(source_id, &source, 1).unwrap();
    let builtins: AdmittedBuiltinRegistryV3 =
        BuiltinRegistryV2::try_closed_with_request_owner_v3(&root, source_id, &source, &parent)
            .unwrap();
    let schema_names: usize = row.fields().iter().map(|field| field.name().len()).sum();
    let demand = scalar_projection_type_storage_upper_v3(
        source.len(),
        count,
        schema_names,
        count,
        count * 2,
    )
    .unwrap();
    let type_storage = root.reserve_v3(source_id, &source, demand).unwrap();
    let request = check_scalar_projection_with_request_owner_v3(
        &checked.as_checked(),
        &extended,
        &row,
        builtins,
        type_storage,
        &root,
        source_id,
        &source,
    )
    .unwrap();
    let key = |node_id| SourceQualifiedNodeIdV2 { source_id, node_id };
    let source_fields = (0..count)
        .map(|index| {
            DeferredRelationRowFieldInputV2::new(
                index,
                format!("f{index}"),
                location.clone(),
                RelationRowFieldSourceV2::Known(ValueTypeV1::int()),
            )
        })
        .collect();
    let project_fields = (0..count)
        .map(|ordinal| {
            let index = count - 1 - ordinal;
            DeferredRelationRowFieldInputV2::new(
                ordinal,
                format!("o{index}"),
                location.clone(),
                RelationRowFieldSourceV2::PriorExpression(key(*path_ids.get(ordinal).unwrap())),
            )
        })
        .collect();
    let entries = base_ids
        .iter()
        .map(|node_id| ResolvedTypeInputEntryV2 {
            key: key(*node_id),
            value: ResolvedTypeInputValueV2::Binding(
                quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(
                    ValueTypeV1::try_record(row.clone()).unwrap(),
                ),
            ),
        })
        .collect();
    let rows = vec![
        DeferredRelationRowInputV2::known_source(key(source_id_node), count, source_fields),
        DeferredRelationRowInputV2::expression_fields(key(project_id), count, project_fields),
    ];
    let inputs =
        ResolvedTypeInputsV2::try_new(checked.as_checked(), &extended, entries, Vec::new(), rows)
            .unwrap();
    let legacy = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(request.relation_row_count(), legacy.relation_row_count());
    assert_eq!(
        request.relation_row(key(source_id_node)),
        legacy.relation_row(key(source_id_node))
    );
    assert_eq!(
        request.relation_row(key(project_id)),
        legacy.relation_row(key(project_id))
    );
    assert_eq!(request.len(), legacy.len());
    for node_id in base_ids.into_iter().chain(path_ids) {
        assert_eq!(request.get(key(node_id)), legacy.get(key(node_id)));
    }
    if count == 1 && !bare_query {
        check_owner_refusals(
            &checked, &extended, &row, &source, source_id, &root, &parent, demand,
        );
    }
    drop(inputs);
    drop(legacy);
    drop(parent);
    assert!(ledger.live.load(Ordering::SeqCst) > 0);
    drop(request);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}

fn check_owner_refusals(
    checked: &CheckedHirDocumentOwnedV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    row: &RecordTypeV1,
    source: &str,
    source_id: SourceIdV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    parent: &quanta_gqlang_v2_hir::FrontendRetainedStoragePermitV3,
    demand: usize,
) {
    let builtins =
        BuiltinRegistryV2::try_closed_with_request_owner_v3(root, source_id, source, parent)
            .unwrap();
    let short = root.reserve_v3(source_id, source, demand - 1).unwrap();
    assert!(matches!(
        check_scalar_projection_with_request_owner_v3(
            &checked.as_checked(),
            extended,
            row,
            builtins,
            short,
            root,
            source_id,
            source
        ),
        Err(
            quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::Storage(
                FrontendStorageRefusalV3::BudgetRejected
            )
        )
    ));

    let foreign_ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
    });
    let foreign_sink = Sink(foreign_ledger.clone());
    let foreign_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &foreign_sink);
    let foreign_parent = foreign_root.reserve_v3(source_id, source, 1).unwrap();
    let foreign_builtins = BuiltinRegistryV2::try_closed_with_request_owner_v3(
        &foreign_root,
        source_id,
        source,
        &foreign_parent,
    )
    .unwrap();
    let own_storage = root.reserve_v3(source_id, source, demand).unwrap();
    assert!(matches!(
        check_scalar_projection_with_request_owner_v3(
            &checked.as_checked(),
            extended,
            row,
            foreign_builtins,
            own_storage,
            root,
            source_id,
            source
        ),
        Err(
            quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            )
        )
    ));

    let own_builtins =
        BuiltinRegistryV2::try_closed_with_request_owner_v3(root, source_id, source, parent)
            .unwrap();
    let foreign_storage = foreign_root.reserve_v3(source_id, source, demand).unwrap();
    assert!(matches!(
        check_scalar_projection_with_request_owner_v3(
            &checked.as_checked(),
            extended,
            row,
            own_builtins,
            foreign_storage,
            root,
            source_id,
            source
        ),
        Err(
            quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            )
        )
    ));
    drop(foreign_parent);
    assert_eq!(foreign_ledger.live.load(Ordering::SeqCst), 0);

    let builder = HirBuilderV1::new();
    let bare = builder.query(
        builder.relation_source("records", SpanV1::new(0, 7), None, SpanV1::new(0, 100)),
        None,
        SpanV1::new(0, 100),
    );
    let unsupported = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![ProgramItemHirV1::Query(Box::new(bare))],
    })
    .unwrap();
    let builtins =
        BuiltinRegistryV2::try_closed_with_request_owner_v3(root, source_id, source, parent)
            .unwrap();
    let storage = root.reserve_v3(source_id, source, demand).unwrap();
    assert!(matches!(
        check_scalar_projection_with_request_owner_v3(
            &unsupported.as_checked(),
            extended,
            row,
            builtins,
            storage,
            root,
            source_id,
            source
        ),
        Err(quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::UnsupportedShape)
    ));
}
