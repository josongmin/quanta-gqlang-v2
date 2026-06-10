#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "typed-param acceptance asserts validated schema decoding and exercises fail-closed rejects"
)]

//! F10 proof rail (jun-10-gqlnag-v2-followup) — E04C typed parameter schema,
//! SYNTAX half (`--test typed_params`).
//!
//! NOTE TO OTHER AGENTS: this file pins the E04C syntax-side parameter schema.
//! Invariants that must never regress:
//!   1. A `param <name>: <type>` declaration decodes into a typed
//!      [`TypedParamSchemaV1`] carrying NAME, TYPE, CARDINALITY, and
//!      DEFAULTABILITY — the four properties E04C requires. No property is
//!      implicit/runtime-derived.
//!   2. The schema fails CLOSED: an unknown type, an empty enum domain, a
//!      malformed cardinality, or a default whose token is not a member of the
//!      declared enum domain is rejected at decode time with a typed diagnostic.
//!      Coercion never invents a value the surface did not declare.
//!
//! Acceptance AND reject coverage below. Do not weaken into positive-only.
//!
//! DEFERRED (noted in the F10 ticket): the E04C SEMA half
//! (`sema/src/declarations/typed_params.rs`, argument-binding rules, sema reject
//! fixtures) and the IR product lowering are owned by a separate ticket/agent
//! and are intentionally NOT implemented here.

use quanta_gqlang_v2_syntax::declarations::typed_params::{
    ParamCardinalityV1, ParamTypeV1, TypedParamSchemaV1,
};

#[test]
fn enum_param_decodes_type_cardinality_and_defaultability_v1() {
    let schema =
        TypedParamSchemaV1::parse_decl_v1("param severity: enum<low|medium|high> = medium")
            .expect("enum param decodes");

    assert_eq!(schema.name, "severity");
    assert_eq!(schema.cardinality, ParamCardinalityV1::Required);
    assert_eq!(
        schema.param_type,
        ParamTypeV1::Enum(vec![
            "low".to_string(),
            "medium".to_string(),
            "high".to_string(),
        ])
    );
    // Defaultability: an explicit default makes the param defaultable.
    assert_eq!(schema.default.as_deref(), Some("medium"));
    assert!(schema.is_defaultable_v1());
}

#[test]
fn scalar_and_optional_cardinality_decode_v1() {
    let schema = TypedParamSchemaV1::parse_decl_v1("param max_depth: u32?")
        .expect("optional scalar decodes");
    assert_eq!(schema.name, "max_depth");
    assert_eq!(schema.param_type, ParamTypeV1::Scalar("u32".to_string()));
    assert_eq!(schema.cardinality, ParamCardinalityV1::Optional);
    assert!(schema.default.is_none());
    assert!(!schema.is_defaultable_v1());
}

#[test]
fn named_domain_param_decodes_v1() {
    let schema = TypedParamSchemaV1::parse_decl_v1("param scope: source_scope")
        .expect("named-domain param decodes");
    assert_eq!(
        schema.param_type,
        ParamTypeV1::Domain("source_scope".to_string())
    );
    assert_eq!(schema.cardinality, ParamCardinalityV1::Required);
}

#[test]
fn unknown_type_fails_closed_v1() {
    let err = TypedParamSchemaV1::parse_decl_v1("param x: bigint")
        .expect_err("unknown scalar type must fail closed");
    assert_eq!(err.code, "param_unknown_type");
}

#[test]
fn empty_enum_domain_fails_closed_v1() {
    let err = TypedParamSchemaV1::parse_decl_v1("param x: enum<>")
        .expect_err("empty enum domain must fail closed");
    assert_eq!(err.code, "param_empty_enum_domain");
}

#[test]
fn default_outside_enum_domain_fails_closed_v1() {
    // Defaultability must be type-checked: a default that is not a declared enum
    // member is rejected, never silently coerced.
    let err = TypedParamSchemaV1::parse_decl_v1("param severity: enum<low|high> = medium")
        .expect_err("default outside enum domain must fail closed");
    assert_eq!(err.code, "param_default_not_in_domain");
}

#[test]
fn missing_type_annotation_fails_closed_v1() {
    let err = TypedParamSchemaV1::parse_decl_v1("param severity")
        .expect_err("missing `: <type>` must fail closed");
    assert_eq!(err.code, "param_missing_type");
}

#[test]
fn non_param_head_fails_closed_v1() {
    let err = TypedParamSchemaV1::parse_decl_v1("let x: u32")
        .expect_err("non-param declaration must fail closed");
    assert_eq!(err.code, "param_missing_keyword");
}

#[test]
fn duplicate_enum_member_fails_closed_v1() {
    let err = TypedParamSchemaV1::parse_decl_v1("param x: enum<low|low>")
        .expect_err("duplicate enum member must fail closed");
    assert_eq!(err.code, "param_duplicate_enum_member");
}
