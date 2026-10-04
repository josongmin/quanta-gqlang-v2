#![expect(
    clippy::panic,
    clippy::expect_used,
    reason = "integration fixtures assert exact admitted and ordinary Binder outcomes"
)]

use quanta_gqlang_v2_binder::{
    AdmittedBinderResolutionV3, BindDiagnosticV1, Binder, BinderDeclarationRequestErrorV3,
    BinderEvidenceReaderV3, BinderKindV1, BindingV1, CaptureV1, ModuleGraphV1, ResolutionV1,
    ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    AliasV1, CheckedHirDocumentOwnedV2, CompositionHirV1, FrontendRequestMaterializationRootV3,
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, HirBuilderV1, HirDocumentV1, HirNodeIdV1, PathSegmentV1,
    ProgramItemHirV1, ProjectionItemV1, SourceIdV2, SpanV1,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const SOURCE_ID: SourceIdV2 = SourceIdV2::from_raw(301);

#[derive(Clone, Copy)]
enum Shape {
    Supported,
    TwoLets,
    WrongWith,
    SelfProducer,
    WrongBody,
    ReboundAlias,
    CapturedProducerAlias,
}

struct Fixture {
    source: String,
    composition: CompositionHirV1,
    checked: CheckedHirDocumentOwnedV2,
    relation_ids: [HirNodeIdV1; 4],
    reference_ids: [HirNodeIdV1; 2],
}

fn fixture(shape: Shape) -> Fixture {
    let b = HirBuilderV1::new();
    let producer_name = if matches!(shape, Shape::SelfProducer) {
        "s"
    } else {
        "records"
    };
    let producer_scan = b.relation_source(
        producer_name,
        SpanV1::new(10, 17),
        Some(AliasV1 {
            name: "p".to_owned(),
            span: SpanV1::new(18, 19),
        }),
        SpanV1::new(10, 19),
    );
    let producer_base = b.symbol("p", SpanV1::new(24, 25));
    let producer_path = b
        .try_path(
            producer_base.clone(),
            vec![PathSegmentV1 {
                name: "id".to_owned(),
                span: SpanV1::new(26, 28),
            }],
            SpanV1::new(24, 28),
        )
        .expect("one producer path");
    let producer_project = b
        .try_project(
            producer_scan.clone(),
            vec![ProjectionItemV1 {
                value: producer_path,
                alias: Some(AliasV1 {
                    name: "id".to_owned(),
                    span: SpanV1::new(32, 34),
                }),
            }],
            SpanV1::new(10, 35),
        )
        .expect("one producer projection");
    let producer_query = b.query(producer_project.clone(), None, SpanV1::new(9, 36));
    let first_binding = b.composition_binding("s", SpanV1::new(4, 5), producer_query.into());

    let body_name = if matches!(shape, Shape::WrongBody) {
        "records"
    } else {
        "s"
    };
    let body_alias = if matches!(shape, Shape::ReboundAlias) {
        "s"
    } else {
        "q"
    };
    let body_scan = b.relation_source(
        body_name,
        SpanV1::new(55, 56),
        Some(AliasV1 {
            name: body_alias.to_owned(),
            span: SpanV1::new(57, 58),
        }),
        SpanV1::new(55, 58),
    );
    let body_base_name = if matches!(shape, Shape::CapturedProducerAlias) {
        "p"
    } else {
        body_alias
    };
    let body_base = b.symbol(body_base_name, SpanV1::new(63, 64));
    let body_path = b
        .try_path(
            body_base.clone(),
            vec![PathSegmentV1 {
                name: "id".to_owned(),
                span: SpanV1::new(65, 67),
            }],
            SpanV1::new(63, 67),
        )
        .expect("one body path");
    let body_project = b
        .try_project(
            body_scan.clone(),
            vec![ProjectionItemV1 {
                value: body_path,
                alias: Some(AliasV1 {
                    name: "out".to_owned(),
                    span: SpanV1::new(71, 74),
                }),
            }],
            SpanV1::new(55, 75),
        )
        .expect("one body projection");
    let body_query = b.query(body_project.clone(), None, SpanV1::new(54, 76));
    let mut bindings = vec![first_binding];
    if matches!(shape, Shape::TwoLets) {
        let second_scan =
            b.relation_source("other", SpanV1::new(40, 45), None, SpanV1::new(40, 45));
        let second_query = b.query(second_scan, None, SpanV1::new(40, 46));
        bindings.push(b.composition_binding("t", SpanV1::new(38, 39), second_query.into()));
    }
    let with_name = if matches!(shape, Shape::WrongWith) {
        "t"
    } else {
        "s"
    };
    let composition = b.composition(
        bindings,
        with_name,
        SpanV1::new(82, 83),
        body_query.into(),
        SpanV1::new(0, 84),
    );
    let source = " ".repeat(84);
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id: SOURCE_ID,
        source_len: source.len(),
        items: vec![ProgramItemHirV1::from(composition.clone())],
    })
    .expect("bounded, unique checked HIR");
    Fixture {
        source,
        composition,
        checked,
        relation_ids: [
            producer_scan.id,
            producer_project.id,
            body_scan.id,
            body_project.id,
        ],
        reference_ids: [producer_base.id, body_base.id],
    }
}

struct Ledger {
    live: AtomicU64,
    peak: AtomicU64,
    limit: AtomicU64,
    calls: AtomicU64,
    last_requested: AtomicU64,
}

struct Sink(Arc<Ledger>);

struct Charge {
    ledger: Arc<Ledger>,
    bytes: u64,
}

impl Drop for Charge {
    fn drop(&mut self) {
        self.ledger.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

impl FrontendRequestStorageSinkV3 for Sink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        self.0
            .last_requested
            .store(logical_bytes, Ordering::Release);
        let bytes = logical_bytes
            .checked_add(u64::try_from(std::mem::size_of::<Charge>()).expect("small charge"))
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.0.live.load(Ordering::Acquire);
        loop {
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.0.limit.load(Ordering::Acquire) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self
                .0
                .live
                .compare_exchange_weak(live, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.0.peak.fetch_max(next, Ordering::AcqRel);
                    self.0.calls.fetch_add(1, Ordering::AcqRel);
                    return Ok(Box::new(Charge {
                        ledger: Arc::clone(&self.0),
                        bytes,
                    }));
                }
                Err(observed) => live = observed,
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

fn sink(limit: u64) -> Sink {
    Sink(Arc::new(Ledger {
        live: AtomicU64::new(0),
        peak: AtomicU64::new(0),
        limit: AtomicU64::new(limit),
        calls: AtomicU64::new(0),
        last_requested: AtomicU64::new(0),
    }))
}

/// This fixture's logical allocation inventory, independently counted from
/// its two one-field Source→Project queries. The private Rust frame has one
/// Vec<Binding> and one Capture tag; this align-up is an ABI layout witness,
/// not a claim about allocator metadata or arbitrary future frame fields.
fn one_let_resolution_bytes() -> u64 {
    let binding = std::mem::size_of::<BindingV1>();
    let index = std::mem::size_of::<(usize, usize, SpanV1)>();
    let frame_payload = std::mem::size_of::<Vec<BindingV1>>() + std::mem::size_of::<CaptureV1>();
    let frame_alignment =
        std::mem::align_of::<Vec<BindingV1>>().max(std::mem::align_of::<CaptureV1>());
    let frame = frame_payload.div_ceil(frame_alignment) * frame_alignment;

    // Each query reserves an alias binding, one input/output binding, and a
    // one-slot duplicate-name index. Its output name can have two live copies.
    let producer_scratch = 3 * binding + index + "p".len() + 2 * "id".len();
    let body_scratch = 3 * binding + index + "q".len() + 2 * "out".len();
    // The LET name has eight concurrent copies at the sealed peak. Other
    // bytes come from two aliases, two output labels, and base-symbol uses.
    let copied_names = 8 * "s".len()
        + "p".len()
        + "q".len()
        + "id".len()
        + "out".len()
        + "p".len()
        + "p".len()
        + "q".len()
        + "q".len();
    let relation_slots = 4 * std::mem::size_of::<(HirNodeIdV1, Vec<BindingV1>)>();
    let retained_bindings = 5 * binding;
    let resolutions = 4 * std::mem::size_of::<ResolutionV1>();
    let diagnostic_slots = 4 * std::mem::size_of::<BindDiagnosticV1>();
    u64::try_from(
        producer_scratch
            + body_scratch
            + copied_names
            + relation_slots
            + retained_bindings
            + resolutions
            + diagnostic_slots
            + 2 * frame,
    )
    .expect("finite fixture inventory fits u64")
}

fn resolve(
    fixture: &Fixture,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedBinderResolutionV3, BinderDeclarationRequestErrorV3> {
    Binder::new(ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(
            &fixture.checked,
            root,
            SOURCE_ID,
            source,
            parent,
        )?
        .resolve_checked_source_bindings_with_request_owner_v3(
            &fixture.checked,
            root,
            SOURCE_ID,
            source,
            parent,
        )
}

#[test]
fn one_let_request_preserves_ordinary_targets_outputs_and_last_owner() {
    let f = fixture(Shape::Supported);
    let mut ordinary = Binder::new(ModuleGraphV1::new());
    ordinary.resolve_composition(&f.composition);
    let ordinary = ordinary.finish();
    assert!(
        ordinary.is_ok(),
        "ordinary diagnostics: {:?}",
        ordinary.diagnostics()
    );

    let sink = sink(u64::MAX);
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, &f.source, &sink);
    let parent = root
        .reserve_v3(SOURCE_ID, &f.source, 0)
        .expect("parent owner");
    let admitted = resolve(&f, &root, &f.source, &parent).expect("supported one-LET Binder");
    assert_eq!(admitted.resolved_reference_count_v3(), 4);
    assert_eq!(admitted.relation_output_count_v3(), 4);
    assert_eq!(
        admitted.resolved_reference_count_v3(),
        ordinary.resolutions().len()
    );
    assert_eq!(
        admitted.relation_output_count_v3(),
        ordinary.relation_outputs().len()
    );
    for relation in f.relation_ids {
        assert_eq!(
            admitted.relation_output(relation),
            ordinary.relation_output(relation)
        );
    }
    for (relation, expected_name) in f.relation_ids.into_iter().zip(["p", "id", "q", "out"]) {
        let [binding] = admitted
            .relation_output(relation)
            .expect("one output binding")
        else {
            panic!("restricted Source or Project has one output");
        };
        assert_eq!(binding.name, expected_name);
    }
    for reference in f.reference_ids {
        assert_eq!(
            admitted.resolution_of(reference),
            ordinary.resolution_of(reference)
        );
    }
    assert_eq!(
        admitted.composition_binding(f.composition.id, 0),
        ordinary.composition_binding(f.composition.id, 0)
    );
    assert!(matches!(
        admitted.composition_binding(f.composition.id, 0),
        Some(binding) if binding.kind == BinderKindV1::CompositionLet
            && binding.identity() == (f.composition.id, 0)
            && binding.name == "s"
    ));
    assert!(matches!(
        admitted.target_of(f.relation_ids[2]),
        Some(ResolvedTargetV1::Local(binding))
            if binding.kind == BinderKindV1::CompositionLet
                && binding.identity() == (f.composition.id, 0)
    ));
    assert_eq!(
        admitted.validate_owner_v3(&root, SOURCE_ID, &f.source),
        Ok(())
    );
    drop(parent);
    assert!(sink.0.live.load(Ordering::Acquire) > 0);
    drop(admitted);
    assert_eq!(sink.0.live.load(Ordering::Acquire), 0);
}

#[test]
fn adjacent_composition_shapes_refuse_before_resolution_reservation() {
    for shape in [
        Shape::TwoLets,
        Shape::WrongWith,
        Shape::SelfProducer,
        Shape::WrongBody,
        Shape::ReboundAlias,
        Shape::CapturedProducerAlias,
    ] {
        let f = fixture(shape);
        let sink = sink(u64::MAX);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, &f.source, &sink);
        let parent = root
            .reserve_v3(SOURCE_ID, &f.source, 0)
            .expect("parent owner");
        let declarations = Binder::new(ModuleGraphV1::new())
            .declare_checked_program_with_request_owner_v3(
                &f.checked, &root, SOURCE_ID, &f.source, &parent,
            )
            .expect("closed declaration phase");
        let calls = sink.0.calls.load(Ordering::Acquire);
        assert!(matches!(
            declarations.resolve_checked_source_bindings_with_request_owner_v3(
                &f.checked, &root, SOURCE_ID, &f.source, &parent,
            ),
            Err(BinderDeclarationRequestErrorV3::UnsupportedShape)
        ));
        assert_eq!(sink.0.calls.load(Ordering::Acquire), calls);
        drop(parent);
        assert_eq!(sink.0.live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn one_let_request_exact_short_and_foreign_owner_boundaries() {
    let f = fixture(Shape::Supported);
    let resolution_bytes = one_let_resolution_bytes();
    let charge_bytes = u64::try_from(std::mem::size_of::<Charge>()).expect("small charge");
    for short in [false, true] {
        let sink = sink(u64::MAX);
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, &f.source, &sink);
        let parent = root
            .reserve_v3(SOURCE_ID, &f.source, 0)
            .expect("parent fits");
        let declarations = Binder::new(ModuleGraphV1::new())
            .declare_checked_program_with_request_owner_v3(
                &f.checked, &root, SOURCE_ID, &f.source, &parent,
            )
            .expect("declaration phase fits before resolution limit");
        let upstream_live = sink.0.live.load(Ordering::Acquire);
        let upstream_calls = sink.0.calls.load(Ordering::Acquire);
        let exact_limit = upstream_live + resolution_bytes + charge_bytes;
        sink.0
            .limit
            .store(exact_limit - u64::from(short), Ordering::Release);
        let result = declarations.resolve_checked_source_bindings_with_request_owner_v3(
            &f.checked, &root, SOURCE_ID, &f.source, &parent,
        );
        assert_eq!(
            sink.0.last_requested.load(Ordering::Acquire),
            resolution_bytes,
            "the production resolution debit must equal fixture inventory"
        );
        if short {
            assert!(matches!(
                result,
                Err(BinderDeclarationRequestErrorV3::Storage(
                    FrontendStorageRefusalV3::BudgetRejected
                ))
            ));
            assert_eq!(sink.0.calls.load(Ordering::Acquire), upstream_calls);
        } else {
            let admitted = result.expect("independently computed exact debit admits");
            assert_eq!(sink.0.calls.load(Ordering::Acquire), upstream_calls + 1);
            assert_eq!(sink.0.peak.load(Ordering::Acquire), exact_limit);
            drop(admitted);
        }
        drop(parent);
        assert_eq!(sink.0.live.load(Ordering::Acquire), 0);
    }

    let foreign = sink(u64::MAX);
    let sink = sink(u64::MAX);
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, &f.source, &sink);
    let foreign_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, &f.source, &foreign);
    let parent = root
        .reserve_v3(SOURCE_ID, &f.source, 0)
        .expect("parent owner");
    let declarations = Binder::new(ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(
            &f.checked, &root, SOURCE_ID, &f.source, &parent,
        )
        .expect("original declarations");
    let calls = sink.0.calls.load(Ordering::Acquire);
    assert!(matches!(
        declarations.resolve_checked_source_bindings_with_request_owner_v3(
            &f.checked,
            &foreign_root,
            SOURCE_ID,
            &f.source,
            &parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    assert_eq!(sink.0.calls.load(Ordering::Acquire), calls);
    assert_eq!(foreign.0.calls.load(Ordering::Acquire), 0);

    let declarations = Binder::new(ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(
            &f.checked, &root, SOURCE_ID, &f.source, &parent,
        )
        .expect("original declarations");
    let calls = sink.0.calls.load(Ordering::Acquire);
    assert!(matches!(
        declarations.resolve_checked_source_bindings_with_request_owner_v3(
            &f.checked,
            &root,
            SOURCE_ID,
            "different source",
            &parent,
        ),
        Err(BinderDeclarationRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignSource
        ))
    ));
    assert_eq!(sink.0.calls.load(Ordering::Acquire), calls);
    drop(parent);
    assert_eq!(sink.0.live.load(Ordering::Acquire), 0);
}
