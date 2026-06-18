#![allow(
    clippy::expect_used,
    reason = "fragment policy tests assert typed dialect admission"
)]
//! F04/E07B hellgate — retrieval fragment dialect restriction.
use quanta_gqlang_v2_sema::fragment_policy::FragmentPolicyV1;

#[test]
fn sourcegraph_dialect_admits_v1() {
    FragmentPolicyV1::admit_dialect_v1("sourcegraph").expect("sourcegraph admits");
    assert!(FragmentPolicyV1::allows_v1("Sourcegraph"));
}

#[test]
fn unsupported_dialect_fails_closed_v1() {
    let err = FragmentPolicyV1::admit_dialect_v1("regex").expect_err("regex must fail closed");
    assert_eq!(err.code, "fragment_dialect_rejected");
}

#[test]
fn missing_dialect_fails_closed_v1() {
    let err =
        FragmentPolicyV1::admit_dialect_v1("  ").expect_err("missing dialect must fail closed");
    assert_eq!(err.code, "fragment_dialect_missing");
}
