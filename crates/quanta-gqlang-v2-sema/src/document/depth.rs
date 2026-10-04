//! Bounded relation-depth traversal with per-live-stack request storage.

use super::{
    CheckedSyntaxDocumentV2, DeclarationBuildErrorV3, DeclaredRequestErrorV3, FrontendBudgetV2,
    FrontendRequestMaterializationRootV3, HirVisitorV1, SourceIdV2, SpanV1,
};

/// Reject relation trees before recursive Binder and type-owner traversal can
/// exceed the transaction's checked stack-depth budget.
#[derive(Clone, Copy)]
pub(super) struct DepthScratchRequestV3<'a, 'source> {
    pub(super) root: &'a FrontendRequestMaterializationRootV3<'source>,
    pub(super) source_id: SourceIdV2,
    pub(super) source: &'source str,
    pub(super) slots: usize,
}

pub(super) fn check_relation_depth_impl(
    checked: &CheckedSyntaxDocumentV2,
    budget: &FrontendBudgetV2,
    request: Option<DepthScratchRequestV3<'_, '_>>,
) -> Result<(), DeclaredRequestErrorV3> {
    struct Guard<'a, 'source> {
        current_depth: u32,
        limit: u32,
        exceeded: Option<SpanV1>,
        request: Option<DepthScratchRequestV3<'a, 'source>>,
        storage_error: Option<DeclaredRequestErrorV3>,
    }
    impl HirVisitorV1 for Guard<'_, '_> {
        fn visit_relation(&mut self, root: &quanta_gqlang_v2_hir::RelationExprHirV1) {
            use quanta_gqlang_v2_hir::{JoinHirV1, RelationExprKindV1};

            if self.exceeded.is_some() || self.storage_error.is_some() {
                return;
            }
            let previous = self.current_depth;
            let request = self.request;
            let _scratch = if let Some(request) = request {
                let Some(bytes) = request.slots.checked_mul(std::mem::size_of::<(
                    &quanta_gqlang_v2_hir::RelationExprHirV1,
                    u32,
                )>()) else {
                    self.storage_error = Some(DeclaredRequestErrorV3::DemandOverflow);
                    return;
                };
                if request.slots == 0 {
                    self.storage_error = Some(DeclaredRequestErrorV3::FallibleAllocation);
                    return;
                }
                match request
                    .root
                    .reserve_v3(request.source_id, request.source, bytes)
                {
                    Ok(scratch) => Some(scratch),
                    Err(error) => {
                        self.storage_error = Some(DeclaredRequestErrorV3::Storage(error));
                        return;
                    }
                }
            } else {
                None
            };
            let mut stack = Vec::new();
            if let Some(request) = request {
                if stack.try_reserve_exact(request.slots).is_err()
                    || stack.capacity() > request.slots
                {
                    self.storage_error = Some(DeclaredRequestErrorV3::FallibleAllocation);
                    return;
                }
            }
            stack.push((root, previous.saturating_add(1)));
            macro_rules! push_checked {
                ($frame:expr) => {{
                    if request.is_some_and(|request| stack.len() >= request.slots) {
                        self.storage_error = Some(DeclaredRequestErrorV3::FallibleAllocation);
                        break;
                    }
                    stack.push($frame);
                }};
            }
            while let Some((node, depth)) = stack.pop() {
                if depth > self.limit {
                    self.exceeded = Some(node.span);
                    break;
                }
                self.current_depth = depth;
                match &node.kind {
                    RelationExprKindV1::Source(_) => {}
                    RelationExprKindV1::Project(project) => {
                        for item in project.items() {
                            self.visit_value(&item.value);
                        }
                        push_checked!((&project.input, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::Filter(filter) => {
                        self.visit_bool(&filter.predicate);
                        push_checked!((&filter.input, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::Join(join) => {
                        let (left, right) = match join {
                            JoinHirV1::Inner { left, right, on } => {
                                self.visit_bool(on);
                                (left, right)
                            }
                            JoinHirV1::Cross { left, right } => (left, right),
                        };
                        push_checked!((right, depth.saturating_add(1)));
                        push_checked!((left, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::GroupBy(group) => {
                        for key in group.keys() {
                            self.visit_value(key);
                        }
                        push_checked!((&group.input, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::OrderBy(order) => {
                        for key in order.keys() {
                            self.visit_value(&key.value);
                        }
                        push_checked!((&order.input, depth.saturating_add(1)));
                    }
                }
                if self.exceeded.is_some() || self.storage_error.is_some() {
                    break;
                }
            }
            self.current_depth = previous;
        }
    }
    let mut guard = Guard {
        current_depth: 0,
        limit: budget.max_relation_depth(),
        exceeded: None,
        request,
        storage_error: None,
    };
    for item in &checked.checked().document().items {
        guard.visit_item(item);
        if let Some(error) = guard.storage_error.take() {
            return Err(error);
        }
        if let Some(span) = guard.exceeded {
            return Err(DeclaredRequestErrorV3::Validation(
                DeclarationBuildErrorV3::RelationDepthLimit(span),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_relation_depth_impl, DepthScratchRequestV3};
    use crate::document::{CheckedSyntaxDocumentV2, DeclaredRequestErrorV3};
    use crate::FrontendBudgetV2;
    use quanta_gqlang_v2_hir::{
        FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
        FrontendRequestStorageSinkV3, FrontendStorageRefusalV3, HirBuilderV1, HirDocumentV1,
        ProgramItemHirV1, RelationExistsPolarityV1, RelationExprHirV1, SourceIdV2, SpanV1,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Ledger {
        live: u64,
        peak: u64,
        requests: Vec<u64>,
    }

    struct Sink {
        ledger: Arc<Mutex<Ledger>>,
        limit: u64,
    }

    struct Permit {
        ledger: Arc<Mutex<Ledger>>,
        bytes: u64,
    }

    impl Drop for Permit {
        fn drop(&mut self) {
            if let Ok(mut ledger) = self.ledger.lock() {
                ledger.live -= self.bytes;
            }
        }
    }

    impl FrontendRequestStorageSinkV3 for Sink {
        fn reserve_v3(
            &self,
            logical_bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            let permit_bytes = u64::try_from(std::mem::size_of::<Permit>())
                .map_err(|_error| FrontendStorageRefusalV3::DemandOverflow)?;
            let Some(bytes) = logical_bytes.checked_add(permit_bytes) else {
                return Err(FrontendStorageRefusalV3::DemandOverflow);
            };
            let Ok(mut ledger) = self.ledger.lock() else {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            };
            let Some(next) = ledger.live.checked_add(bytes) else {
                return Err(FrontendStorageRefusalV3::DemandOverflow);
            };
            if next > self.limit {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            ledger.live = next;
            ledger.peak = ledger.peak.max(next);
            ledger.requests.push(logical_bytes);
            Ok(Box::new(Permit {
                ledger: Arc::clone(&self.ledger),
                bytes,
            }))
        }

        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            permit
                .as_any_v3()
                .downcast_ref::<Permit>()
                .is_some_and(|permit| Arc::ptr_eq(&permit.ledger, &self.ledger))
        }
    }

    fn nested_exists() -> Result<CheckedSyntaxDocumentV2, String> {
        let builder = HirBuilderV1::new();
        let inner =
            builder.relation_source("records", SpanV1::new(20, 27), None, SpanV1::new(20, 27));
        let exists =
            builder.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(15, 28));
        let outer = builder.relation_source("records", SpanV1::new(1, 8), None, SpanV1::new(1, 8));
        let relation = builder.relation_filter(outer, exists, SpanV1::new(1, 29));
        let query = builder.query(relation, None, SpanV1::new(0, 29));
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(7),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .map_err(|error| format!("{error:?}"))
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "independent scratch ledger assertions"
    )]
    fn nested_exists_reserves_both_live_stacks_and_releases_them() -> Result<(), String> {
        let checked = nested_exists()?;
        let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
        let source = "x".repeat(64);
        let slots = checked.node_count();
        let scratch_bytes = slots
            .checked_mul(std::mem::size_of::<(&RelationExprHirV1, u32)>())
            .ok_or("scratch byte overflow")?;
        let charged_bytes = u64::try_from(scratch_bytes + std::mem::size_of::<Permit>())
            .map_err(|error| error.to_string())?;
        let ledger = Arc::new(Mutex::new(Ledger::default()));
        let sink = Sink {
            ledger: Arc::clone(&ledger),
            limit: charged_bytes * 2,
        };
        let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
            checked.source_id(),
            &source,
            &sink,
        );
        check_relation_depth_impl(
            &checked,
            &budget,
            Some(DepthScratchRequestV3 {
                root: &root,
                source_id: checked.source_id(),
                source: &source,
                slots,
            }),
        )
        .map_err(|error| format!("{error:?}"))?;
        let state = ledger.lock().map_err(|error| error.to_string())?;
        assert_eq!(
            state.requests,
            vec![u64::try_from(scratch_bytes).map_err(|error| error.to_string())?; 2]
        );
        assert_eq!(state.peak, charged_bytes * 2);
        assert_eq!(state.live, 0);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "independent refusal ledger assertions"
    )]
    fn nested_exists_short_or_foreign_root_refuses_and_releases() -> Result<(), String> {
        let checked = nested_exists()?;
        let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
        let source = "x".repeat(64);
        let slots = checked.node_count();
        let scratch_bytes = slots
            .checked_mul(std::mem::size_of::<(&RelationExprHirV1, u32)>())
            .ok_or("scratch byte overflow")?;
        let one_stack = u64::try_from(scratch_bytes + std::mem::size_of::<Permit>())
            .map_err(|error| error.to_string())?;
        let ledger = Arc::new(Mutex::new(Ledger::default()));
        let sink = Sink {
            ledger: Arc::clone(&ledger),
            limit: one_stack,
        };
        let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
            checked.source_id(),
            &source,
            &sink,
        );
        assert!(matches!(
            check_relation_depth_impl(
                &checked,
                &budget,
                Some(DepthScratchRequestV3 {
                    root: &root,
                    source_id: checked.source_id(),
                    source: &source,
                    slots,
                }),
            ),
            Err(DeclaredRequestErrorV3::Storage(
                FrontendStorageRefusalV3::BudgetRejected
            ))
        ));
        assert_eq!(ledger.lock().map_err(|error| error.to_string())?.live, 0);
        let equal_bytes_foreign = "x".repeat(64);
        let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
            checked.source_id(),
            &equal_bytes_foreign,
            &sink,
        );
        assert!(matches!(
            check_relation_depth_impl(
                &checked,
                &budget,
                Some(DepthScratchRequestV3 {
                    root: &foreign_root,
                    source_id: checked.source_id(),
                    source: &source,
                    slots,
                }),
            ),
            Err(DeclaredRequestErrorV3::Storage(
                FrontendStorageRefusalV3::ForeignSource
            ))
        ));
        assert_eq!(ledger.lock().map_err(|error| error.to_string())?.live, 0);
        Ok(())
    }
}
