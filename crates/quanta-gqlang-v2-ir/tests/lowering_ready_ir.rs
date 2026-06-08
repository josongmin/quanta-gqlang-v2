#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::find_ir::FindIrV1;
use quanta_gqlang_v2_ir::lowering_ready::LoweringReadyIrV1;
use quanta_gqlang_v2_ir::planning::{
    BridgeModeV1, CapabilityNameV1, CapabilityReasonV1, CapabilityRequirementV1, ExportModeV1,
    PublicPlanRootKindV1, PublicPlanV1, TruthExpectationV1,
};
use quanta_gqlang_v2_ir::root_ir::{CatalogCategoryIrV1, InteractiveIrV1, PublicIrRootV1};

fn capability_v1(
    capability: CapabilityNameV1,
    reason: CapabilityReasonV1,
) -> CapabilityRequirementV1 {
    CapabilityRequirementV1 { capability, reason }
}

#[test]
fn roots_requiring_planning_metadata_fail_closed_v1() {
    let find = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Find(FindIrV1 {
            subkind: "path".to_string(),
            seed: "entry -> sink".to_string(),
        }),
        None,
    )
    .expect_err("FIND must require planning metadata");
    assert_eq!(find.code, "missing_planning_metadata");

    let interactive = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Interactive(InteractiveIrV1::Show {
            category: CatalogCategoryIrV1::Capabilities,
        }),
        None,
    )
    .expect_err("interactive root must require planning metadata");
    assert_eq!(interactive.code, "missing_planning_metadata");
}

#[test]
fn planning_root_mismatch_and_missing_capabilities_fail_closed_v1() {
    let mismatch_plan = PublicPlanV1 {
        root: PublicPlanRootKindV1::Interactive,
        required_capabilities: vec![capability_v1(
            CapabilityNameV1::CatalogIntrospection,
            CapabilityReasonV1::CatalogRead,
        )],
        truth_expectation: TruthExpectationV1::Observational,
        bridge_mode: BridgeModeV1::DirectRead,
        export_mode: ExportModeV1::None,
        working_set_kind: None,
        ai_plan: None,
    };
    let mismatch = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Find(FindIrV1 {
            subkind: "taint".to_string(),
            seed: "api.entry -> db.write".to_string(),
        }),
        Some(mismatch_plan),
    )
    .expect_err("root mismatch must fail closed");
    assert_eq!(mismatch.code, "planning_root_mismatch");

    let missing_caps_plan = PublicPlanV1 {
        root: PublicPlanRootKindV1::Interactive,
        required_capabilities: Vec::new(),
        truth_expectation: TruthExpectationV1::Observational,
        bridge_mode: BridgeModeV1::DirectRead,
        export_mode: ExportModeV1::None,
        working_set_kind: None,
        ai_plan: None,
    };
    let missing_caps = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Interactive(InteractiveIrV1::Show {
            category: CatalogCategoryIrV1::Capabilities,
        }),
        Some(missing_caps_plan),
    )
    .expect_err("planning without capabilities must fail closed");
    assert_eq!(missing_caps.code, "planning_missing_capabilities");
}

#[test]
fn normalized_lowering_ready_ir_preserves_public_shape_v1() {
    let plan = PublicPlanV1 {
        root: PublicPlanRootKindV1::Interactive,
        required_capabilities: vec![capability_v1(
            CapabilityNameV1::CatalogIntrospection,
            CapabilityReasonV1::CatalogRead,
        )],
        truth_expectation: TruthExpectationV1::Observational,
        bridge_mode: BridgeModeV1::DirectRead,
        export_mode: ExportModeV1::None,
        working_set_kind: None,
        ai_plan: None,
    };
    let ready = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Interactive(InteractiveIrV1::Show {
            category: CatalogCategoryIrV1::Capabilities,
        }),
        Some(plan.clone()),
    )
    .expect("valid lowering-ready IR");

    let normalized = ready.normalized_v1().expect("normalized lowering-ready IR");
    assert_eq!(normalized.planning, Some(plan));
    assert_eq!(normalized.root_kind_v1(), PublicPlanRootKindV1::Interactive);
}
