#![expect(
    clippy::expect_used,
    reason = "owner regression asserts exact stage outcomes"
)]

use super::*;
use crate::compile::{
    begin_bind_source_document_with_request_owner_v3,
    begin_prepare_source_body_with_request_owner_v3,
    begin_resolve_source_document_with_request_owner_v3,
    check_source_document_with_request_owner_v3, declare_source_document_with_request_owner_v3,
    parse_source_document_with_request_owner_v3, predeclare_source_catalog_with_request_owner_v3,
    resolve_source_bindings_with_request_owner_v3, seal_source_catalog_with_request_owner_v3,
};
use quanta_gqlang_v2_sema::{
    ClosedFrontendAuthorityBuilderV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendLanguageVersionV2, FrontendRelationInputV2,
    ScalarSemanticAssemblyErrorV3, SourceIdV2,
};
use quanta_gqlang_v2_surface::{
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const DOCUMENT_SOURCE: SourceIdV2 = SourceIdV2::from_raw(9);
const HOST_SOURCE: SourceIdV2 = SourceIdV2::from_raw(2);
const SOURCE: &str = "from files f | project f.path as path";

struct Sink {
    live: Arc<AtomicU64>,
    peak: AtomicU64,
    limit: AtomicU64,
    reserve_calls: AtomicU64,
    issuer: Arc<()>,
}

struct Permit {
    live: Arc<AtomicU64>,
    bytes: u64,
    issuer: Arc<()>,
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

impl FrontendRequestStorageSinkV3 for Sink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        self.reserve_calls.fetch_add(1, Ordering::AcqRel);
        let bytes = logical_bytes
            .checked_add(
                u64::try_from(std::mem::size_of::<Permit>())
                    .map_err(|_error| FrontendStorageRefusalV3::DemandOverflow)?,
            )
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.live.load(Ordering::Acquire);
        loop {
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.limit.load(Ordering::Acquire) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self
                .live
                .compare_exchange_weak(live, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.peak.fetch_max(next, Ordering::AcqRel);
                    break;
                }
                Err(observed) => live = observed,
            }
        }
        Ok(Box::new(Permit {
            live: Arc::clone(&self.live),
            bytes,
            issuer: Arc::clone(&self.issuer),
        }))
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<Permit>()
            .is_some_and(|value| Arc::ptr_eq(&value.issuer, &self.issuer))
    }
}

fn authority() -> ClosedFrontendAuthorityV2 {
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("budget"),
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
    .expect("authority builder");
    builder.add_relation(FrontendRelationInputV2::canonical_workspace_files(
        location, location,
    ));
    builder.finish().expect("closed authority")
}

fn materialized<'a>(
    source: &str,
    authority: &'a ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> MaterializedSourceBodyRequestSourceV3<'a> {
    let parsed = parse_source_document_with_request_owner_v3(
        DOCUMENT_SOURCE,
        source,
        authority.budget(),
        root,
    )
    .expect("parser owner");
    let checked =
        check_source_document_with_request_owner_v3(DOCUMENT_SOURCE, source, root, parsed)
            .expect("checked owner");
    let declared = declare_source_document_with_request_owner_v3(source, authority, root, checked)
        .expect("declared owner");
    let resolving = begin_resolve_source_document_with_request_owner_v3(source, root, declared)
        .expect("session owner");
    let predeclared = predeclare_source_catalog_with_request_owner_v3(source, root, resolving)
        .expect("catalog owner");
    let preparing = begin_prepare_source_body_with_request_owner_v3(source, root, predeclared)
        .expect("builtin owner");
    let declaring = begin_bind_source_document_with_request_owner_v3(source, root, preparing)
        .expect("Binder declaration owner");
    let resolved = resolve_source_bindings_with_request_owner_v3(source, root, declaring)
        .expect("Binder resolution owner");
    let typed =
        type_source_body_with_request_owner_v3(source, root, resolved).expect("Types owner");
    let topology =
        topologize_source_body_with_request_owner_v3(source, root, typed).expect("topology owner");
    materialize_source_body_with_request_owner_v3(source, root, topology).expect("body owner")
}

#[test]
fn same_root_cross_session_catalog_seal_refuses_before_reservation() {
    for no_spare in [false, true] {
        let source = SOURCE;
        let live = Arc::new(AtomicU64::new(0));
        let sink = Sink {
            live: Arc::clone(&live),
            peak: AtomicU64::new(0),
            limit: AtomicU64::new(u64::MAX),
            reserve_calls: AtomicU64::new(0),
            issuer: Arc::new(()),
        };
        let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
            DOCUMENT_SOURCE,
            source,
            &sink,
        );
        let authority = authority();
        for _ in 0..2 {
            let same = materialized(source, &authority, &root);
            let sealed = seal_source_catalog_with_request_owner_v3(source, &root, same)
                .expect("same session seals");
            drop(sealed);
            assert_eq!(live.load(Ordering::Acquire), 0);
        }
        let a = materialized(source, &authority, &root);
        let b = materialized(source, &authority, &root);
        assert_eq!(a.owner.value_v3().4.source_id_v3(), DOCUMENT_SOURCE);
        assert_eq!(b.owner.value_v3().4.source_id_v3(), DOCUMENT_SOURCE);
        let live_before = live.load(Ordering::Acquire);
        let peak_before = sink.peak.load(Ordering::Acquire);
        let calls_before = sink.reserve_calls.load(Ordering::Acquire);
        if no_spare {
            sink.limit.store(live_before, Ordering::Release);
        }
        let result = a.owner.try_map_v3(
            |(_a_extended, _a_session, _a_assets, _a_topology, a_body), _a_parent| {
                let a_body = match a_body {
                    quanta_gqlang_v2_sema::AdmittedSourceBodyV3::Scalar(body) => Some(body),
                    quanta_gqlang_v2_sema::AdmittedSourceBodyV3::OneLet(_) => None,
                }
                .expect("scalar fixture body");
                b.owner.try_map_v3(
                    |(_b_extended, b_session, _b_assets, b_topology, _b_body), b_parent| {
                        assert_eq!(
                            b_parent.validate_owner_v3(&root, DOCUMENT_SOURCE, source),
                            Ok(())
                        );
                        quanta_gqlang_v2_sema::seal_source_scalar_catalog_with_request_owner_v3(
                            b_session, b_topology, a_body, &root, source, b_parent,
                        )
                    },
                )
            },
        );
        let error = result.err().expect("mixed session refuses");
        assert!(
            matches!(
                error,
                SourceCatalogSealErrorV3::Semantic(
                    ScalarSemanticAssemblyErrorV3::ForeignCatalogSession
                )
            ),
            "{error:?}"
        );
        assert_eq!(sink.reserve_calls.load(Ordering::Acquire), calls_before);
        assert_eq!(sink.peak.load(Ordering::Acquire), peak_before);
        assert!(live.load(Ordering::Acquire) <= live_before);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}
