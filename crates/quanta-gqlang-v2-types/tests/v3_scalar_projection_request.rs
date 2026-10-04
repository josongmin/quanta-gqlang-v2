#![expect(clippy::unwrap_used, reason = "independent request owner assertions")]

use quanta_gqlang_v2_hir::{
    AliasV1, CatalogPayloadV1, CheckedHirDocumentOwnedV2, FrontendRequestMaterializationRootV3,
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
    HirBuilderV1, HirDocumentV1, LocatedSpanV2, PathSegmentV1, ProgramItemHirV1, ProjectionItemV1,
    SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1, ViewBodyV1,
};
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
