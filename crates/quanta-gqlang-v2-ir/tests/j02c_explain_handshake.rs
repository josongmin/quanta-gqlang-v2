#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::explain_ir::{ExplainEvidenceModeIrV1, ExplainTruthExpectationIrV1};
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;
use quanta_gqlang_v2_ir::{lower_document_to_ir_v1, lower_root_to_ir_v1};
use quanta_gqlang_v2_sema::analyze_document_v1;
use quanta_gqlang_v2_sema::TypedRootV1;
use quanta_gqlang_v2_syntax::parse_document_v1;

fn lower_root_v1(input: &str) -> PublicIrRootV1 {
    lower_document_to_ir_v1(
        &analyze_document_v1(
            &parse_document_v1(input)
                .unwrap_or_else(|page| panic!("expected parse success for `{input}`: {page:?}")),
        )
        .unwrap_or_else(|page| panic!("expected sema success for `{input}`: {page:?}")),
    )
    .unwrap_or_else(|err| panic!("expected IR lowering success for `{input}`: {err:?}"))
    .into_iter()
    .next()
    .expect("one root")
}

#[test]
fn explain_handshake_preserves_truth_and_evidence_shape() {
    let root =
        lower_root_v1("EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm");
    let PublicIrRootV1::Explain(explain) = root else {
        panic!("expected EXPLAIN IR root");
    };

    let shape = explain.shape();
    assert_eq!(shape.target, "rule::owasp.sql-injection");
    assert_eq!(shape.evidence, ExplainEvidenceModeIrV1::Trace);
    assert_eq!(
        shape.truth_expectation,
        ExplainTruthExpectationIrV1::Confirm
    );
}

#[test]
fn explain_handshake_rejects_duplicated_truth_clause_fail_closed() {
    let lowered = lower_root_to_ir_v1(&TypedRootV1::Explain(
        quanta_gqlang_v2_sema::roots::explain::TypedExplainRootV1 {
            target: Some("result::latest ;; truth=inspect ;; truth=confirm".to_string()),
        },
    ));

    assert_eq!(
        lowered,
        Err(quanta_gqlang_v2_ir::IrLoweringErrorV1 {
            code: "invalid_explain_target",
            message: "EXPLAIN truth expectation clause is duplicated".to_string(),
        }),
    );
}
