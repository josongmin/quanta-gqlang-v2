//! v3 lock rail (D34): a composition body must type-check against a local type
//! environment that carries each LET producer's OUTPUT row type, not only the
//! original frontend schema.
//!
//! Regression target `PIPELINE_COMPOSITION_TYPE_ENV_INCOMPLETE`:
//! `compile_composition` compiled each producer and then compiled the body
//! against the unchanged [`FrontendContextV1`] schema, so a body expression that
//! references a producer's aliased output (e.g. `from p x | where x.id ...`) saw
//! `p` as an unknown source. The body's alias `x` stayed env-open and any genuine
//! type error inside it (a bad overload over a producer column, a missing
//! producer column) was silently skipped, violating D34 (binder-visible locals
//! must type-check under a lifted local type environment, capture lane included).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration rail: assertion panics are intentional"
)]

use quanta_gqlang_v2_pipeline::{
    compile_program_structural, compile_program_with_schema, CompiledItemV1,
};
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, ScalarTyV1, TypeEnvV1, TypeV1};

/// `users(id: int, name: string)` as the only frontend schema authority.
fn users_relation_schema() -> TypeEnvV1 {
    TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![
                RecordFieldTyV1::new("id", TypeV1::int()),
                RecordFieldTyV1::new("name", TypeV1::string()),
            ])),
            cardinality: CardinalityV1::Bag,
        },
    )
}

/// A composition whose producer `p` projects an `int` column `id`, and whose body
/// reads `p` as a source and applies a bad overload (`int + string`) to its `id`
/// column, must reject at the type stage once the producer's output row type is
/// lifted into the body's local type environment.
#[test]
fn composition_body_types_against_producer_output_row() {
    let src = "let p = from users u | project u.id as id\n\
               from p x | where x.id + \"q\" > 1 | project x.id as id with p";
    let diag = compile_program_with_schema(src, &users_relation_schema()).expect_err(
        "composition body bad overload over a producer column must reject once the \
         producer output row type is lifted into the body type environment (D34)",
    );
    assert_eq!(
        diag.stage().as_str(),
        "type",
        "the body's bad overload over a producer column is a type-stage reject"
    );
    assert_eq!(diag.code(), "types.bad_overload");
}

/// The same lifting must reject a body that projects a column the producer does
/// NOT output (`x.ghost`), with the typed bad-projection diagnostic.
#[test]
fn composition_body_rejects_missing_producer_column() {
    let src = "let p = from users u | project u.id as id\n\
               from p x | project x.ghost as id with p";
    let diag = compile_program_with_schema(src, &users_relation_schema()).expect_err(
        "selecting a column the producer never projects must reject once the producer \
         output row type is lifted (D34)",
    );
    assert_eq!(diag.stage().as_str(), "type");
    assert_eq!(diag.code(), "types.bad_projection");
}

/// D34 capture-lane discipline: a producer projection that contributes NO named
/// columns (every item is alias-less, e.g. `project u.id, 2`) must NOT bind its
/// name to an empty-record relation. An empty record would silently over-bind the
/// name, making a downstream `from p x | project x.ghost as g` reject as a
/// `types.bad_projection` against a phantom empty schema. Per the documented
/// contract ("the lifted row never over-binds: a downstream reference to such a
/// column simply stays env-open exactly as it would have without lifting"), the
/// name must stay schema-open and the well-formed body must compile cleanly.
///
/// Regression target `PIPELINE_EMPTY_PROJECTION_SILENT_BINDING` (D34).
#[test]
fn composition_empty_projection_does_not_overbind() {
    let src = "let p = from users u | project u.id, 2\n\
               from p x | project x.ghost as g with p";
    let compiled = compile_program_with_schema(src, &users_relation_schema()).expect(
        "a producer that projects no NAMED columns must leave its name schema-open, \
         so a downstream reference stays env-open rather than rejecting against an \
         over-bound empty-record relation (D34)",
    );
    assert!(
        compiled.plan().is_some(),
        "the composition body produces a servable plan"
    );
}

/// Lifting must stay sound: a body that reads a column the producer DOES output,
/// with a well-typed expression, still compiles cleanly.
#[test]
fn composition_body_accepts_valid_producer_column() {
    let src = "let p = from users u | project u.id as id\n\
               from p x | where x.id > 1 | project x.id as id with p";
    let compiled = compile_program_with_schema(src, &users_relation_schema())
        .expect("a well-typed body over a producer column must compile");
    // The outcome is a composition carrier (sanity check on routing).
    assert!(
        compiled.plan().is_some(),
        "the composition body produces a servable plan"
    );
}

fn exact_binding_output_v1(source_v1: &str) -> Option<TypeV1> {
    let compiled_v1 = compile_program_structural(source_v1).expect("composition compiles");
    let CompiledItemV1::Composition {
        binding_outputs, ..
    } = compiled_v1.compiled()
    else {
        panic!("expected compiled composition")
    };
    assert_eq!(binding_outputs.len(), 1);
    binding_outputs.first().cloned().flatten()
}

#[test]
fn sample_producer_issues_exact_path_relation_and_types_body_v1() {
    let source_v1 = "let p = sample query=\"auth\" | mode=lexical | limit=2 \
                     from p x | project x.path as path with p";
    assert_eq!(
        exact_binding_output_v1(source_v1),
        Some(TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "path",
                TypeV1::scalar(ScalarTyV1::RepoPath),
            )])),
            cardinality: CardinalityV1::Set,
        })
    );
}

#[test]
fn sample_producer_rejects_noncontract_body_field_v1() {
    let source_v1 = "let p = sample query=\"auth\" | mode=lexical | limit=2 \
                     from p x | project x.function as function with p";
    let error_v1 = compile_program_structural(source_v1)
        .expect_err("SAMPLE rows expose only the exact `path` field");
    assert_eq!(error_v1.stage().as_str(), "type");
    assert_eq!(error_v1.code(), "types.bad_projection");
}

#[test]
fn graph_find_producers_issue_one_exact_function_relation_v1() {
    for family_v1 in ["path", "reachability"] {
        let source_v1 = format!(
            "let f = find {family_v1} app.start -> app.end | depth=2 \
             from f x | project x.function as function with f"
        );
        assert_eq!(
            exact_binding_output_v1(&source_v1),
            Some(TypeV1::Relation {
                row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                    "function",
                    TypeV1::scalar(ScalarTyV1::EntityRef),
                )])),
                cardinality: CardinalityV1::Set,
            }),
            "{family_v1} must issue the exact function relation"
        );

        let invalid_source_v1 = format!(
            "let f = find {family_v1} app.start -> app.end | depth=2 \
             from f x | project x.path as path with f"
        );
        let error_v1 = compile_program_structural(&invalid_source_v1)
            .expect_err("graph FIND rows expose only the exact `function` field");
        assert_eq!(error_v1.stage().as_str(), "type");
        assert_eq!(error_v1.code(), "types.bad_projection");
    }
}

#[test]
fn unsupported_find_producer_does_not_gain_a_relation_schema_v1() {
    let source_v1 = "let f = find taint app.input -> app.sink | depth=2 \
                     from f x | project x.function as function with f";
    assert_eq!(
        exact_binding_output_v1(source_v1),
        None,
        "TAINT rows have no frontend-issued relational contract"
    );
}

#[test]
fn relational_producer_without_source_schema_does_not_gain_a_guessed_output_v1() {
    let source_v1 = "let p = from files f | project f.path as path \
                     from p x | project x.path as path with p";
    assert_eq!(
        exact_binding_output_v1(source_v1),
        None,
        "schema-open relational producers must not fabricate a field type"
    );
}
