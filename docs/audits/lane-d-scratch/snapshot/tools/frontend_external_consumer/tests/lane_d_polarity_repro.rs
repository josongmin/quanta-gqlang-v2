//! Lane D repro: source-predicate call polarity derivation drift between the
//! sema dependency producer and the logic rule-program validator.
//!
//! Shape under test: `s(t(count(u(1))))` in a rule body.
//! - sema visits `t` BEFORE the aggregate call `count` sets its subtree
//!   aggregated flag, so `t` is sealed Positive and `u` Aggregated.
//! - logic re-derives the edges from the sealed expression effects; the `t`
//!   call node's COMPOSED effects include `count`'s Aggregate, so logic marks
//!   every argument of `t` aggregated and derives `t` Aggregated.
//! Divergence direction: logic Aggregated vs sema Positive -> the validator
//! rejects with `logic.rule_program.polarity_flip`, even though sema sealed the
//! document. The test asserts the CORRECT behavior (a compile success) and also
//! reports whether a failure is a LOGIC rejection (sema accepted) or a
//! sema/types rejection (the shape is not legal, which would refute the drift).
//!
//! Scratch copy only: this file lives in the Lane D isolated snapshot, never in
//! the audited working tree.

#![forbid(unsafe_code)]

use quanta_gqlang_v2_frontend_contract::{
    compile_source_document_v3, ClosedFrontendAuthorityBuilderV2, FrontendAuthorityLocationV2,
    FrontendAuthoritySourceV2, FrontendBudgetV2, FrontendLanguageVersionV2,
    FrontendTransactionErrorV2, SourceIdV2,
};

/// The minimal closed authority the external consumer positive rail uses.
fn closed_authority() -> quanta_gqlang_v2_frontend_contract::ClosedFrontendAuthorityV2 {
    ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 64,
        }],
        "app".to_owned(),
        FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 0,
            end: 1,
        },
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("minimal authority starts")
    .finish()
    .expect("minimal authority seals")
}

fn error_codes(error: &FrontendTransactionErrorV2) -> Vec<String> {
    match error {
        FrontendTransactionErrorV2::Authority(authority) => vec![authority.code().to_owned()],
        FrontendTransactionErrorV2::Document(failure) => failure
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                let primary = diagnostic.primary();
                format!(
                    "{} [{}..{}] {}",
                    diagnostic.code(),
                    primary.span.start,
                    primary.span.end,
                    diagnostic.message()
                )
            })
            .collect(),
    }
}

/// `s` takes Bool (bare bool use of the head symbol), `t`/`u` take Int.
const HEAD: &str = "let answer = 1; rule u : x if x = 1; rule t : x if x = 1; \
                    rule s : x if x or false; ";

fn compile(source_id: u32, text: &str, label: &str) -> Result<(), Vec<String>> {
    let authority = closed_authority();
    match compile_source_document_v3(SourceIdV2::from_raw(source_id), text, &authority) {
        Ok(compiled) => {
            println!(
                "{label}: OK items={} :: {text}",
                compiled.execution().items().len()
            );
            Ok(())
        }
        Err(error) => {
            let codes = error_codes(&error);
            println!("{label}: ERR {codes:?}");
            Err(codes)
        }
    }
}

#[test]
fn predicate_call_under_aggregate_keeps_sema_polarity() {
    // Control A: nested predicate call with no aggregate anywhere.
    let a = format!("{HEAD}rule main : y if y = 1 and s(t(1)); show capabilities;");
    // Divergent B: `t`'s argument contains the aggregate call over `u`.
    let b = format!("{HEAD}rule main : y if y = 1 and s(t(count(u(1)))); show capabilities;");

    // Boundary probes: the aggregate under a BUILTIN call (no predicate layer)
    // and a single predicate call with an aggregate argument must both agree.
    let v2 = format!("{HEAD}rule main : y if y = 1 and s(is_null(count(u(1)))); show capabilities;");
    let v4 = format!("{HEAD}rule main : y if y = 1 and t(count(u(1))); show capabilities;");

    // V5: a SCALAR builtin call with a record argument that mixes a predicate
    // call and an aggregate sibling. Sema does not extend the aggregate scope
    // at `is_null` (its intrinsic effect is scalar); logic does (composed).
    let v5 = format!("{HEAD}rule main : y if y = 1 and is_null({{a: t(1), b: count(u(1))}}); show capabilities;");
    let v6 = format!("{HEAD}rule main : y if y = 1 and is_null({{a: t(1), b: 1}}); show capabilities;");

    let v6_result = compile(27, &v6, "V6_record_control");
    let v5_result = compile(26, &v5, "V5_record_under_scalar_builtin");
    assert!(v6_result.is_ok(), "V6 must compile: {v6_result:?}");
    println!("V5 outcome (drift if logic.*): {v5_result:?}");

    let a_result = compile(21, &a, "A_plain");
    let v2_result = compile(24, &v2, "V2_builtin_layer");
    let v4_result = compile(25, &v4, "V4_single_predicate_layer");
    let b_result = compile(22, &b, "B_agg_nested");

    assert!(v2_result.is_ok(), "V2 must compile: {v2_result:?}");
    assert!(v4_result.is_ok(), "V4 must compile: {v4_result:?}");

    assert!(
        a_result.is_ok(),
        "positive neighbor A must compile: {a_result:?}"
    );
    match &b_result {
        Ok(()) => println!("NO_DRIFT: B compiled; the two derivations agree on this shape"),
        Err(codes) => {
            let logic = codes.iter().all(|code| code.starts_with("logic."));
            if logic {
                println!("DRIFT_CONFIRMED: sema sealed the document, logic rejected it: {codes:?}");
            } else {
                println!(
                    "REFUTED: B is not sema-legal (first failure is not a logic rejection): {codes:?}"
                );
            }
            assert!(logic, "B must be either accepted or rejected only by logic");
        }
    }
    assert!(
        b_result.is_ok(),
        "sema-legal B must compile (drift repro): {b_result:?}"
    );
}
