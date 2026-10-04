#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "request owner assertions"
)]

use super::*;
use crate::resolve::{CaptureV1, FrontendStorageRefusalV3, ResolvedTargetV1};
use quanta_gqlang_v2_hir::{
    AliasV1, FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, HirBuilderV1,
    HirDocumentV1, ProjectionItemV1,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Ledger {
    live: AtomicU64,
    limit: AtomicU64,
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
        let bytes = logical_bytes
            .checked_add(u64::try_from(std::mem::size_of::<Charge>()).unwrap())
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        loop {
            let live = self.0.live.load(Ordering::SeqCst);
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.0.limit.load(Ordering::SeqCst) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            if self
                .0
                .live
                .compare_exchange(live, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(Box::new(Charge {
                    ledger: self.0.clone(),
                    bytes,
                }));
            }
        }
    }
    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<Charge>()
            .is_some_and(|charge| Arc::ptr_eq(&charge.ledger, &self.0))
    }
}

#[test]
fn checked_source_declaration_admits_before_birth_and_releases_last_owner() {
    let source = "let answer = 1; let next = answer; from records r";
    let source_id = SourceIdV2::from_raw(81);
    let builder = HirBuilderV1::new();
    let literal = builder.int("1", SpanV1::new(13, 14));
    let declaration = builder.decl("answer", SpanV1::new(4, 10), literal, SpanV1::new(0, 14));
    let reference = builder.symbol("answer", SpanV1::new(27, 33));
    let next = builder.decl("next", SpanV1::new(20, 24), reference, SpanV1::new(16, 33));
    let relation = builder.relation_source(
        "records",
        SpanV1::new(40, 47),
        Some(quanta_gqlang_v2_hir::AliasV1 {
            name: "r".to_owned(),
            span: SpanV1::new(48, 49),
        }),
        SpanV1::new(35, 49),
    );
    let query = builder.query(relation, None, SpanV1::new(35, 49));
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![
            ProgramItemHirV1::Decl(Box::new(declaration)),
            ProgramItemHirV1::Decl(Box::new(next)),
            ProgramItemHirV1::Query(Box::new(query)),
        ],
    })
    .expect("checked source");
    let ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink);
    let parent = root.reserve_v3(source_id, source, 1).expect("source owner");
    let binder = Binder::new(crate::ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(&checked, &root, source_id, source, &parent)
        .expect("admitted declaration");
    assert_eq!(binder.declared_name_count_v3(), 2);
    let bound = binder
        .resolve_checked_source_bindings_with_request_owner_v3(
            &checked, &root, source_id, source, &parent,
        )
        .expect("admitted value Binder storage");
    assert_eq!(bound.resolved_reference_count_v3(), 1);
    assert_eq!(bound.relation_output_count_v3(), 1);
    assert_eq!(bound.validate_owner_v3(&root, source_id, source), Ok(()));
    let foreign_ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let foreign_sink = Sink(foreign_ledger.clone());
    let foreign_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &foreign_sink);
    assert_eq!(
        bound.validate_owner_v3(&foreign_root, source_id, source),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    assert_eq!(foreign_ledger.live.load(Ordering::SeqCst), 0);
    assert_eq!(
        bound
            .bound
            .relation_outputs()
            .next()
            .unwrap()
            .1
            .first()
            .unwrap()
            .name,
        "r"
    );
    assert!(matches!(
        &bound.bound.resolutions().first().unwrap().target,
        ResolvedTargetV1::Declaration { .. }
    ));
    drop(parent);
    assert!(ledger.live.load(Ordering::SeqCst) > 0);
    drop(bound);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);

    // The internal once-only guard remains exercised without exposing a
    // public path that could separate the result from either permit.
    let repeat_parent = root
        .reserve_v3(source_id, source, 1)
        .expect("repeat parent");
    let mut repeat = Binder::new(crate::ModuleGraphV1::new());
    let repeat_table = repeat
        .declare_checked_program_inner_v3(&checked, &root, source_id, source, &repeat_parent)
        .expect("repeat declaration");
    repeat
        .resolve_checked_source_bindings_inner_v3(
            &checked,
            &root,
            source_id,
            source,
            &repeat_parent,
        )
        .expect("first resolution");
    assert!(matches!(
        repeat.resolve_checked_source_bindings_inner_v3(
            &checked,
            &root,
            source_id,
            source,
            &repeat_parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Phase)
    ));
    drop(repeat);
    drop(repeat_table);
    drop(repeat_parent);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);

    checked_source_rejection_cases(&checked, &root, &ledger, source_id, source);
}

fn checked_source_rejection_cases(
    checked: &CheckedHirDocumentOwnedV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    ledger: &Arc<Ledger>,
    source_id: SourceIdV2,
    source: &str,
) {
    let short_ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let short_sink = Sink(short_ledger.clone());
    let short_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &short_sink);
    let short_parent = short_root
        .reserve_v3(source_id, source, 1)
        .expect("short parent");
    let scratch_birth = 4 * std::mem::size_of::<&DeclHirV1>() + std::mem::size_of::<Charge>();
    short_ledger.limit.store(
        short_ledger.live.load(Ordering::SeqCst) + u64::try_from(scratch_birth).unwrap() - 1,
        Ordering::SeqCst,
    );
    let short_binder = Binder::new(crate::ModuleGraphV1::new());
    assert!(matches!(
        short_binder.declare_checked_program_with_request_owner_v3(
            checked,
            &short_root,
            source_id,
            source,
            &short_parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    drop(short_parent);
    assert_eq!(short_ledger.live.load(Ordering::SeqCst), 0);

    let frame_ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let frame_sink = Sink(frame_ledger.clone());
    let frame_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &frame_sink);
    let frame_parent = frame_root
        .reserve_v3(source_id, source, 1)
        .expect("frame parent");
    let table = Binder::new(crate::ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(
            checked,
            &frame_root,
            source_id,
            source,
            &frame_parent,
        )
        .expect("frame case declaration");
    let frame_birth = LexicalScopeV1::one_frame_storage_bytes_v3()
        + std::mem::size_of::<ResolutionV1>()
        + std::mem::size_of::<(HirNodeIdV1, Vec<BindingV1>)>()
        + 2 * "answer".len()
        + 2 * (std::mem::size_of::<BindingV1>() + "r".len())
        + std::mem::size_of::<Charge>();
    frame_ledger.limit.store(
        frame_ledger.live.load(Ordering::SeqCst) + u64::try_from(frame_birth).unwrap() - 1,
        Ordering::SeqCst,
    );
    assert!(matches!(
        table.resolve_checked_source_bindings_with_request_owner_v3(
            checked,
            &frame_root,
            source_id,
            source,
            &frame_parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    drop(frame_parent);
    assert_eq!(frame_ledger.live.load(Ordering::SeqCst), 0);

    let foreign_ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let foreign_sink = Sink(foreign_ledger.clone());
    let foreign_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &foreign_sink);
    let own_parent = root.reserve_v3(source_id, source, 1).expect("own parent");
    let foreign_binder = Binder::new(crate::ModuleGraphV1::new());
    assert!(matches!(
        foreign_binder.declare_checked_program_with_request_owner_v3(
            checked,
            &foreign_root,
            source_id,
            source,
            &own_parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    let own_table = Binder::new(crate::ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(
            checked,
            root,
            source_id,
            source,
            &own_parent,
        )
        .expect("owner declaration");
    assert!(matches!(
        own_table.resolve_checked_source_bindings_with_request_owner_v3(
            checked,
            &foreign_root,
            source_id,
            source,
            &own_parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    let mut raw_binder = Binder::new(crate::ModuleGraphV1::new());
    raw_binder.open_scope(CaptureV1::Capturing);
    raw_binder.close_scope();
    let raw_table = raw_binder
        .declare_checked_program_with_request_owner_v3(
            checked,
            root,
            source_id,
            source,
            &own_parent,
        )
        .expect("declaration table does not certify prior raw scope storage");
    assert!(matches!(
        raw_table.resolve_checked_source_bindings_with_request_owner_v3(
            checked,
            root,
            source_id,
            source,
            &own_parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Phase)
    ));
    drop(own_parent);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
    assert_eq!(foreign_ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn request_declaration_name_index_keeps_first_source_span() {
    let source = "let z = 1; let a = 2; let z = 3; let z = 4";
    let source_id = SourceIdV2::from_raw(82);
    let builder = HirBuilderV1::new();
    let first = builder.decl(
        "z",
        SpanV1::new(4, 5),
        builder.int("1", SpanV1::new(8, 9)),
        SpanV1::new(0, 9),
    );
    let middle = builder.decl(
        "a",
        SpanV1::new(15, 16),
        builder.int("2", SpanV1::new(19, 20)),
        SpanV1::new(11, 20),
    );
    let repeated = builder.decl(
        "z",
        SpanV1::new(26, 27),
        builder.int("3", SpanV1::new(30, 31)),
        SpanV1::new(22, 31),
    );
    let later = builder.decl(
        "z",
        SpanV1::new(37, 38),
        builder.int("4", SpanV1::new(41, 42)),
        SpanV1::new(33, 42),
    );
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![
            later.clone().into(),
            repeated.clone().into(),
            middle.clone().into(),
            first.clone().into(),
        ],
    })
    .expect("checked duplicate source");
    let ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink);
    let parent = root.reserve_v3(source_id, source, 1).expect("parent");
    let declared = Binder::new(crate::ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(&checked, &root, source_id, source, &parent)
        .expect("source-indexed declaration");
    assert_eq!(declared.declared_name_count_v3(), 2);
    assert!(matches!(
        declared.binder.result.diagnostics.as_slice(),
        [BindDiagnosticV1::Duplicate { name: first_name, first_span: first, second_span: second },
         BindDiagnosticV1::Duplicate { name: second_name, first_span: next_first, second_span: next_second }]
            if first_name == "z" && second_name == "z"
            && *first == SpanV1::new(4, 5) && *next_first == SpanV1::new(4, 5)
            && *second == SpanV1::new(26, 27)
            && *next_second == SpanV1::new(37, 38)
    ));
    let mut legacy = Binder::new(crate::ModuleGraphV1::new());
    legacy.declare_program(&[later, repeated, middle, first], &[]);
    assert_eq!(
        declared.binder.result.diagnostics,
        legacy.finish().diagnostics()
    );
    drop((declared, parent));
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn declaration_carrier_rejects_paired_foreign_root_even_when_empty() {
    for with_declaration in [false, true] {
        let source = if with_declaration { "let a = 1" } else { "" };
        let source_id = SourceIdV2::from_raw(83);
        let builder = HirBuilderV1::new();
        let items = if with_declaration {
            let decl = builder.decl(
                "a",
                SpanV1::new(4, 5),
                builder.int("1", SpanV1::new(8, 9)),
                SpanV1::new(0, 9),
            );
            vec![ProgramItemHirV1::Decl(Box::new(decl))]
        } else {
            Vec::new()
        };
        let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
            source_id,
            source_len: source.len(),
            items,
        })
        .expect("checked source");
        let ledger_a = Arc::new(Ledger {
            live: AtomicU64::new(0),
            limit: AtomicU64::new(u64::MAX),
        });
        let ledger_b = Arc::new(Ledger {
            live: AtomicU64::new(0),
            limit: AtomicU64::new(u64::MAX),
        });
        let sink_a = Sink(ledger_a.clone());
        let sink_b = Sink(ledger_b.clone());
        let root_a =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink_a);
        let root_b =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink_b);
        let parent_a = root_a.reserve_v3(source_id, source, 1).expect("parent A");
        let parent_b = root_b.reserve_v3(source_id, source, 1).expect("parent B");
        let declarations = Binder::new(crate::ModuleGraphV1::new())
            .declare_checked_program_with_request_owner_v3(
                &checked, &root_a, source_id, source, &parent_a,
            )
            .expect("declaration A");
        assert!(matches!(
            declarations.resolve_checked_source_bindings_with_request_owner_v3(
                &checked, &root_b, source_id, source, &parent_b,
            ),
            Err(BinderDeclarationRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignRequestRoot
            ))
        ));
        drop((parent_a, parent_b));
        assert_eq!(ledger_a.live.load(Ordering::SeqCst), 0);
        assert_eq!(ledger_b.live.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn request_relation_frames_keep_legacy_node_id_order() {
    let source = " ".repeat(60);
    let source_id = SourceIdV2::from_raw(84);
    let builder = HirBuilderV1::new();
    let mut queries = Vec::new();
    for (ordinal, name) in ["a", "b", "c"].into_iter().enumerate() {
        let start = ordinal * 10;
        let relation = builder.relation_source(
            "records",
            SpanV1::new(start, start + 7),
            Some(AliasV1 {
                name: name.to_owned(),
                span: SpanV1::new(start + 8, start + 9),
            }),
            SpanV1::new(start, start + 9),
        );
        queries.push(builder.query(relation, None, SpanV1::new(start, start + 9)));
    }
    let [first, second, third] = queries.try_into().expect("three queries");
    let legacy_queries = [third.clone(), first.clone(), second.clone()];
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![third.into(), first.into(), second.into()],
    })
    .expect("checked source");
    let ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    let parent = root.reserve_v3(source_id, &source, 1).expect("parent");
    let bound = Binder::new(crate::ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(&checked, &root, source_id, &source, &parent)
        .expect("declaration carrier")
        .resolve_checked_source_bindings_with_request_owner_v3(
            &checked, &root, source_id, &source, &parent,
        )
        .expect("request frames");
    let mut legacy = Binder::new(crate::ModuleGraphV1::new());
    for query in &legacy_queries {
        legacy.resolve_query(query);
    }
    let conventional = legacy.finish();
    assert_eq!(
        bound.bound.relation_outputs().collect::<Vec<_>>(),
        conventional.relation_outputs().collect::<Vec<_>>()
    );
    drop((bound, parent));
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn request_project_duplicate_diagnostic_keeps_first_source_binding() {
    let source = " ".repeat(64);
    let source_id = SourceIdV2::from_raw(85);
    let builder = HirBuilderV1::new();
    let relation = builder.relation_source(
        "records",
        SpanV1::new(0, 7),
        Some(AliasV1 {
            name: "r".to_owned(),
            span: SpanV1::new(8, 9),
        }),
        SpanV1::new(0, 9),
    );
    let projection = builder
        .try_project(
            relation,
            vec![
                ProjectionItemV1 {
                    value: builder.symbol("r", SpanV1::new(15, 16)),
                    alias: Some(AliasV1 {
                        name: "same".to_owned(),
                        span: SpanV1::new(20, 24),
                    }),
                },
                ProjectionItemV1 {
                    value: builder.symbol("r", SpanV1::new(30, 31)),
                    alias: Some(AliasV1 {
                        name: "same".to_owned(),
                        span: SpanV1::new(40, 44),
                    }),
                },
            ],
            SpanV1::new(0, 44),
        )
        .expect("project");
    let query = builder.query(projection, None, SpanV1::new(0, 44));
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id,
        source_len: source.len(),
        items: vec![query.clone().into()],
    })
    .expect("checked project");
    let mut legacy = Binder::new(crate::ModuleGraphV1::new());
    legacy.resolve_query(&query);
    let expected = legacy.finish();
    assert!(matches!(
        expected.diagnostics(),
        [BindDiagnosticV1::Duplicate { first_span, second_span, .. }]
            if *first_span == SpanV1::new(20, 24)
                && *second_span == SpanV1::new(40, 44)
    ));
    let ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    let parent = root.reserve_v3(source_id, &source, 1).expect("parent");
    let mut binder = Binder::new(crate::ModuleGraphV1::new());
    let declaration_storage = binder
        .declare_checked_program_inner_v3(&checked, &root, source_id, &source, &parent)
        .expect("empty declaration");
    assert!(matches!(
        binder.resolve_checked_source_bindings_inner_v3(
            &checked, &root, source_id, &source, &parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Phase)
    ));
    assert_eq!(binder.result.diagnostics, expected.diagnostics());
    drop((binder, declaration_storage, parent));
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}
