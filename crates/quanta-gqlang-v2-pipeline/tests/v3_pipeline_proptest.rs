//! v3 structural property rails for the ONE shared frontend compiler service
//! (ticket 08, D22).
//!
//! These are PROPERTY tests, not examples. They assert the strongest structural
//! guarantees over the full parse -> bind -> type -> algebra/logic -> plan stack
//! reached through the public string-entry API:
//!
//! 1. TOTALITY / panic-freedom: [`compile_program_structural`] and
//!    [`compile_program_with_schema`] (empty schema) must NEVER panic on an
//!    ARBITRARY `String`. Every input yields `Ok(CompiledProgramV1)` or a
//!    stage-tagged [`PipelineDiagnosticV1`] — never an unwind. The ~45
//!    `expect`/indexing sites in the stack must hold against adversarial input.
//! 2. DETERMINISM: compiling the same input twice yields an identical
//!    `CompiledProgramV1` / an identical diagnostic stage+code+span (no
//!    HashMap-ordering or other nondeterministic leakage).
//!
//! A property failure here is a REAL structural bug (a panicking index/expect or
//! a nondeterministic path), to be fixed by fail-closing the offending site, not
//! by weakening the property.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration test rail: assertion panics are intentional"
)]

use std::panic::{catch_unwind, AssertUnwindSafe};

use proptest::prelude::*;
use quanta_gqlang_v2_pipeline::{
    compile_program_structural, compile_program_with_schema, CompiledProgramV1,
    PipelineDiagnosticV1,
};
use quanta_gqlang_v2_types::TypeEnvV1;

/// Bounded config so the rail stays fast while still exercising many inputs.
fn cfg() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        ..ProptestConfig::default()
    }
}

/// A compact, deterministic identity of a compile outcome, used to compare two
/// runs of the SAME input. On `Ok` we lean on `CompiledProgramV1: PartialEq`; on
/// `Err` we pin the stage tag, stable code, and exact span (the contract the
/// pipeline diagnostic promises). Comparing the whole `Result` directly also
/// works (both arms are `PartialEq`); the tagged form gives a sharper failure
/// message if a determinism break is ever found.
fn outcome_identity(
    r: &Result<CompiledProgramV1, PipelineDiagnosticV1>,
) -> Result<CompiledProgramV1, (&'static str, String, String)> {
    match r {
        Ok(p) => Ok(p.clone()),
        Err(d) => Err((d.stage().as_str(), d.code(), format!("{:?}", d.span()))),
    }
}

/// Catch any unwind from a closure, mapping the panic payload to a readable
/// string so a totality failure reports the offending message.
fn caught<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|p| {
        p.downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_owned())
    })
}

// ---------------------------------------------------------------------------
// Generators: arbitrary strings spanning the adversarial space the prompt asks
// for — ascii, unicode, control chars, huge/empty, deeply nested brackets, and
// structured-ish QGLang-shaped fragments.
// ---------------------------------------------------------------------------

/// Wholly arbitrary `String`: any unicode (includes control chars, NUL, BOM,
/// surrogpate-free scalar values), length 0..=256.
fn arb_any_string() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..=256).prop_map(|cs| cs.into_iter().collect())
}

/// Arbitrary ASCII (the lexer's hot path), including control bytes and
/// punctuation that the tokenizer must classify or reject.
fn arb_ascii_string() -> impl Strategy<Value = String> {
    proptest::collection::vec(0u8..=127, 0..=256)
        .prop_map(|bs| bs.into_iter().map(char::from).collect())
}

/// Deeply / unbalanced nested brackets and parens — the classic stack-exhaustion
/// and "index past end" trap for hand-written recursive descent.
fn arb_brackets() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just('('),
            Just(')'),
            Just('['),
            Just(']'),
            Just('{'),
            Just('}'),
            Just(' '),
            Just('|'),
            Just(','),
        ],
        0..=512,
    )
    .prop_map(|cs| cs.into_iter().collect())
}

/// Structured-ish QGLang fragments: real keywords/operators glued together in
/// arbitrary, often-illegal order. This drives inputs DEEP into bind/type/algebra
/// where the indexing/expect sites live, not just the lexer.
fn arb_structured() -> impl Strategy<Value = String> {
    let token = prop_oneof![
        Just("from"),
        Just("where"),
        Just("project"),
        Just("group"),
        Just("by"),
        Just("order"),
        Just("as"),
        Just("rule"),
        Just("check"),
        Just("find"),
        Just("forall"),
        Just("exists"),
        Just("view"),
        Just("define"),
        Just("u"),
        Just("u.id"),
        Just("u.name"),
        Just("users"),
        Just("findings"),
        Just("1"),
        Just("-1"),
        Just("0"),
        Just(">"),
        Just("<"),
        Just("="),
        Just("|"),
        Just(","),
        Just("("),
        Just(")"),
        Just("\""),
        Just("::"),
        Just("\n"),
        Just(" "),
    ];
    proptest::collection::vec(token, 0..=64).prop_map(|ts| ts.join(" "))
}

/// One combined strategy weaving every shape together.
fn arb_source() -> impl Strategy<Value = String> {
    prop_oneof![
        4 => arb_structured(),
        2 => arb_brackets(),
        2 => arb_ascii_string(),
        2 => arb_any_string(),
        // A few fixed adversarial constants always in the mix.
        1 => prop_oneof![
            Just(String::new()),
            Just(" ".repeat(4096)),
            Just("(".repeat(4096)),
            Just("from ".repeat(2048)),
            Just("\u{feff}\u{0}\u{7f}from x".to_owned()),
            Just("from \u{1f600} | where \u{0} > 1".to_owned()),
        ],
    ]
}

// ---------------------------------------------------------------------------
// Rails
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(cfg())]

    /// TOTALITY: structural compile of any arbitrary string never unwinds.
    #[test]
    fn totality_structural_never_panics(src in arb_source()) {
        let res = caught(|| compile_program_structural(&src));
        prop_assert!(
            res.is_ok(),
            "compile_program_structural panicked on {:?}: {}",
            src,
            res.err().unwrap_or_default()
        );
    }

    /// TOTALITY: schema-authority compile (empty schema) of any arbitrary string
    /// never unwinds. Exercises the schema/type lift paths in addition to parse.
    #[test]
    fn totality_with_empty_schema_never_panics(src in arb_source()) {
        let res = caught(|| {
            let schema = TypeEnvV1::new();
            compile_program_with_schema(&src, &schema)
        });
        prop_assert!(
            res.is_ok(),
            "compile_program_with_schema panicked on {:?}: {}",
            src,
            res.err().unwrap_or_default()
        );
    }

    /// DETERMINISM: structural compile of the same input twice is identical
    /// (Ok value equal, or Err stage+code+span equal).
    #[test]
    fn determinism_structural(src in arb_source()) {
        let a = compile_program_structural(&src);
        let b = compile_program_structural(&src);
        prop_assert_eq!(
            outcome_identity(&a),
            outcome_identity(&b),
            "nondeterministic structural compile for {:?}",
            src
        );
        // The whole Result must also be bit-for-bit equal (catches Ok payload
        // drift the identity projection might miss).
        prop_assert!(a == b, "Result inequality for {:?}", src);
    }

    /// DETERMINISM: schema-authority compile of the same input twice is identical.
    #[test]
    fn determinism_with_empty_schema(src in arb_source()) {
        let schema = TypeEnvV1::new();
        let a = compile_program_with_schema(&src, &schema);
        let b = compile_program_with_schema(&src, &schema);
        prop_assert_eq!(
            outcome_identity(&a),
            outcome_identity(&b),
            "nondeterministic schema compile for {:?}",
            src
        );
        prop_assert!(a == b, "Result inequality (schema) for {:?}", src);
    }
}
