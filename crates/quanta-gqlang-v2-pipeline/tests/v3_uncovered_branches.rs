//! Coverage rails for the weakest stage-routing / reject / edge branches of the
//! shared compiler service.
//!
//! Each test drives a previously-uncovered branch and asserts the REAL expected
//! behavior (exact compiled variant, exact stage tag + stable diagnostic code),
//! so an uncovered error / edge path becomes a locked regression guard rather
//! than a silent gap.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rail: assertion panics and exhaustive-by-fallthrough \
              matches over the compiled-outcome enum are intentional"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1};
use quanta_gqlang_v2_pipeline::{
    compile_document_structural, compile_document_with_schema, compile_item_structural,
    compile_item_with_schema, compile_program_structural, compile_program_with_schema,
    CompiledItemV1, NoServeReasonV1, PipelineStageV1,
};
use quanta_gqlang_v2_surface::parse_program;
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

/// Parse one source root into its unified program item.
fn item(src: &str) -> ProgramItemHirV1 {
    let builder = HirBuilderV1::new();
    parse_program(src, &builder)
        .expect("source parses")
        .item
        .to_program_item()
}

/// `users(id: int, age: int)` schema authority used by alias-dependent tests.
fn users_schema() -> TypeEnvV1 {
    TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![
                RecordFieldTyV1::new("id", TypeV1::int()),
                RecordFieldTyV1::new("age", TypeV1::int()),
            ])),
            cardinality: CardinalityV1::Bag,
        },
    )
}

// ---- accessors -------------------------------------------------------------

/// `NoServeReasonV1::keyword` exposes the stable CLI/LSP keyword for each reason.
#[test]
fn no_serve_reason_keywords_are_stable() {
    assert_eq!(
        NoServeReasonV1::ValueDeclaration.keyword(),
        "value_declaration"
    );
    assert_eq!(NoServeReasonV1::RuleArtifact.keyword(), "rule_artifact");
}

/// `CompiledProgramV1::item()` returns the parsed surface carrier verbatim, and
/// for a relational query the `compiled()` outcome is a `RelationalQuery`.
#[test]
fn compiled_program_exposes_surface_item_and_outcome() {
    let compiled =
        compile_program_structural("from users | project 1 as x").expect("query compiles");
    // The item accessor returns the same carrier the parser produced.
    assert_eq!(
        compiled.item(),
        &compile_program_item("from users | project 1 as x")
    );
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::RelationalQuery { .. }
    ));
}

/// Helper: the surface item a freshly-parsed program carries.
fn compile_program_item(src: &str) -> quanta_gqlang_v2_surface::SurfaceItemV1 {
    let builder = HirBuilderV1::new();
    parse_program(src, &builder).expect("source parses").item
}

/// `CompiledItemV1::binding_outputs` is `Some` for a composition and exposes one
/// lifted-output slot per binding, and `None` for any non-composition outcome.
#[test]
fn binding_outputs_is_some_only_for_composition() {
    let schema = users_schema();
    let comp = compile_program_with_schema(
        "let q = from users u | project u.id as id from users x | project 1 as y with q",
        &schema,
    )
    .expect("composition compiles");
    match comp.compiled() {
        CompiledItemV1::Composition { .. } => {
            let outputs = comp
                .compiled()
                .binding_outputs()
                .expect("a composition exposes binding_outputs");
            assert_eq!(outputs.len(), 1, "one binding -> one output slot");
            // The single relational binding projects a named column, so its row
            // schema lifts (Some), carrying the producer's actual cardinality.
            assert!(
                outputs[0].is_some(),
                "named-projection producer lifts a row"
            );
        }
        other => panic!("expected Composition, got {other:?}"),
    }

    // A non-composition outcome carries no binding outputs.
    let q = compile_program_structural("from users | project 1 as x").expect("query compiles");
    assert!(q.compiled().binding_outputs().is_none());
}

// ---- entry-point fan-out (schema / item / document) ------------------------

/// `compile_item_with_schema` threads schema authority into single-item compile,
/// so an alias-dependent projection is type-checked (and an aliased projection
/// onto a bad overload rejects at type, not silently skipped).
#[test]
fn compile_item_with_schema_types_alias_dependent_projection() {
    let schema = users_schema();
    let good = compile_item_with_schema(&item("from users u | project u.id as id"), &schema)
        .expect("alias-typed projection compiles");
    assert!(matches!(good, CompiledItemV1::RelationalQuery { .. }));

    let bad =
        compile_item_with_schema(&item("from users u | project u.id + \"x\" as bad"), &schema)
            .expect_err("an int+string projection over a lifted alias must reject at type");
    assert_eq!(bad.stage(), PipelineStageV1::Type);
    assert_eq!(bad.code(), "types.bad_overload");
}

/// `compile_item_structural` compiles a single already-parsed item with no schema
/// authority; an alias-dependent projection stays env-open (skipped), so it
/// compiles structurally.
#[test]
fn compile_item_structural_compiles_value_decl() {
    let compiled =
        compile_item_structural(&item("let x = 1 + 2")).expect("decl compiles structurally");
    match compiled {
        CompiledItemV1::ValueArtifact {
            value_type,
            no_serve_reason,
            ..
        } => {
            assert_eq!(no_serve_reason, NoServeReasonV1::ValueDeclaration);
            assert_eq!(value_type.map(|t| t.to_string()), Some("int".to_owned()));
        }
        other => panic!("expected ValueArtifact, got {other:?}"),
    }
}

/// `compile_document_structural` compiles every item in order against ONE shared
/// catalog; two distinct catalog declarations both register and yield two
/// `CatalogRegistration` outcomes.
#[test]
fn compile_document_registers_distinct_catalog_decls() {
    let items = [
        item("view a.view | query-ref=base.one"),
        item("view b.view | query-ref=base.two"),
    ];
    let compiled = compile_document_structural(&items).expect("two distinct VIEWs both register");
    assert_eq!(compiled.len(), 2);
    assert!(compiled
        .iter()
        .all(|c| matches!(c, CompiledItemV1::CatalogRegistration { .. })));
}

/// The shared-catalog document path fails closed when a SECOND declaration
/// collides on `<kind>::<name>`: the duplicate rejects tagged `modules`, which a
/// per-item compile (fresh catalog each) could never surface.
#[test]
fn compile_document_rejects_cross_item_duplicate_asset() {
    let items = [
        item("view dup.view | query-ref=base.one"),
        item("view dup.view | query-ref=base.two"),
    ];
    let diag = compile_document_structural(&items)
        .expect_err("a colliding second VIEW declaration must reject at modules");
    assert_eq!(diag.stage(), PipelineStageV1::Modules);
    // Each standalone VIEW compiled on its own succeeds (fresh catalog), proving
    // the collision is genuinely a cross-item / shared-catalog rule.
    assert!(compile_item_structural(&items[0]).is_ok());
    assert!(compile_item_structural(&items[1]).is_ok());
}

/// `compile_document_with_schema` threads schema authority through every item of
/// a shared-catalog document; an alias-typed query compiles, a sibling bad
/// overload short-circuits the whole document at type (D10).
#[test]
fn compile_document_with_schema_short_circuits_on_first_type_error() {
    let schema = users_schema();
    let good = [
        item("from users u | project u.id as id"),
        item("from users u | project u.age as age"),
    ];
    let compiled = compile_document_with_schema(&good, &schema).expect("both queries type");
    assert_eq!(compiled.len(), 2);

    let bad = [
        item("from users u | project u.id as id"),
        item("from users u | project u.id + \"x\" as bad"),
    ];
    let diag = compile_document_with_schema(&bad, &schema)
        .expect_err("the second item's bad overload must short-circuit the document");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
}

// ---- result-shape type stage (type_check_shape) ----------------------------

/// A query carrying a `=> { col: <value> }` record result shape type-checks the
/// shape's leaf values; a well-typed shape compiles.
#[test]
fn record_result_shape_type_checks_and_compiles() {
    let compiled = compile_program_structural("from users | project 1 as a => { col: 1 + 2 }")
        .expect("a well-typed record result shape compiles");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::RelationalQuery { .. }
    ));
}

/// A `=> [ <value> ]` list result shape type-checks its element value.
#[test]
fn list_result_shape_type_checks_and_compiles() {
    let compiled = compile_program_structural("from users | project 1 as a => [ 1 + 2 ]")
        .expect("a well-typed list result shape compiles");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::RelationalQuery { .. }
    ));
}

/// A `=> <value>` leaf result shape with a closed type error rejects at the type
/// stage with the exact overload code — the shape is NOT skipped just because it
/// is post-projection.
#[test]
fn leaf_result_shape_type_error_rejects_at_type() {
    let diag = compile_program_structural("from users | project 1 as a => 1 + \"x\"")
        .expect_err("a closed leaf-shape type error must reject at type");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
}

/// A nested record-shape whose leaf has a closed type error rejects at type,
/// exercising the recursive `type_check_shape` Record arm.
#[test]
fn nested_record_result_shape_type_error_rejects_at_type() {
    let diag = compile_program_structural("from users | project 1 as a => { col: 1 + \"x\" }")
        .expect_err("a closed nested-record shape type error must reject at type");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
}

// ---- filter predicate boolean kernel (type_check_bool) ---------------------

/// An `and` / `or` filter predicate over lifted alias columns type-checks both
/// connective operands; a well-typed compound predicate compiles.
#[test]
fn filter_and_or_predicate_type_checks_and_compiles() {
    let schema = users_schema();
    let compiled = compile_program_with_schema(
        "from users u | where u.age > 18 and u.id < 5 or u.id = 0 | project u.id as a",
        &schema,
    )
    .expect("a well-typed and/or predicate compiles");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::RelationalQuery { .. }
    ));
}

/// A `not` filter predicate exercises the `type_check_bool` Not arm; a
/// closed-but-wrong arithmetic operand inside the negated comparison still
/// rejects at type (an int + string is not a legal overload, unlike `=` which is
/// heterogeneous-legal).
#[test]
fn filter_not_predicate_with_bad_arithmetic_rejects_at_type() {
    let schema = users_schema();
    let diag = compile_program_with_schema(
        "from users u | where not u.id + \"x\" = 1 | project u.id as a",
        &schema,
    )
    .expect_err("an int + string operand under not must reject at type");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
}

/// The companion soundness case: a well-typed `not` predicate compiles, locking
/// the `type_check_bool` Not arm's accept path.
#[test]
fn filter_not_predicate_well_typed_compiles() {
    let schema = users_schema();
    let compiled = compile_program_with_schema(
        "from users u | where not u.id = 1 | project u.id as a",
        &schema,
    )
    .expect("a well-typed negated comparison compiles");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::RelationalQuery { .. }
    ));
}

// ---- value-expression free-symbol walk (collect_free_symbols) --------------

/// A literal-only decl body that nests list / record / unary / paren is
/// env-CLOSED (it references no free symbols), so the free-symbol walk descends
/// through every nested arm and the body is fully type-checked: a bad overload
/// deep inside rejects at type rather than being skipped.
#[test]
fn decl_nested_literal_body_walks_all_value_arms_and_types() {
    let diag = compile_program_structural("let bad = [ { a: -(1) + \"x\" } ]")
        .expect_err("a bad overload nested in list/record/unary/paren must reject at type");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");

    // The same nested shape with a sound body compiles, proving the walk does not
    // over-reject.
    let ok = compile_program_structural("let good = [ { a: -(1) + (2) } ]")
        .expect("a well-typed nested body compiles");
    assert!(matches!(
        ok.compiled(),
        CompiledItemV1::ValueArtifact { .. }
    ));
}

// ---- rule lowering (ensure_supported_rule_head / collect_rule_body) --------

/// A rule whose HEAD is not rooted in a symbol (a bare arithmetic expression)
/// rejects at the logic stage with the unsupported-rule-head code.
#[test]
fn rule_with_non_symbol_head_rejects_at_logic() {
    let diag = compile_program_structural("rule r : 1 + 2 if base")
        .expect_err("an arithmetic rule head has no predicate symbol");
    assert_eq!(diag.stage(), PipelineStageV1::Logic);
    assert_eq!(diag.code(), "logic.unsupported_rule_head");
}

/// A rule body mixing `and` / `or` / `not` / paren / quantifier and nested value
/// forms (unary / binary / call / record / list / path) lowers into the exact
/// deduplicated body literal set with correct polarity.
#[test]
fn rule_body_connectives_and_value_forms_lower_to_dedup_literals() {
    let compiled = compile_program_structural(
        "rule r : head if (alpha) and not beta or f(gamma, delta) and exists z in (alpha or epsilon)",
    )
    .expect("a connective-rich rule body lowers");
    match compiled.compiled() {
        CompiledItemV1::RuleArtifact { program, .. } => {
            let rules = program.rules();
            let body = &rules.first().expect("one rule").body;
            // alpha appears positively twice (paren + quantifier) but dedups to one
            // positive literal; beta is negated; gamma/delta from the call args;
            // epsilon from the quantifier disjunct.
            let positives: Vec<&str> = body
                .iter()
                .filter(|l| matches!(l.polarity, quanta_gqlang_v2_logic::BodyPolarityV1::Positive))
                .map(|l| l.predicate.as_str())
                .collect();
            let negatives: Vec<&str> = body
                .iter()
                .filter(|l| matches!(l.polarity, quanta_gqlang_v2_logic::BodyPolarityV1::Negated))
                .map(|l| l.predicate.as_str())
                .collect();
            // The call arguments contribute predicate symbols; the callee `f` is
            // NOT a predicate symbol (it resolves against the builtin registry,
            // not the rule's body relations), so it is intentionally absent.
            assert!(positives.contains(&"alpha"));
            assert!(positives.contains(&"gamma"));
            assert!(positives.contains(&"delta"));
            assert!(positives.contains(&"epsilon"));
            assert!(
                !positives.contains(&"f"),
                "the callee name is not a body predicate"
            );
            assert_eq!(negatives, vec!["beta"], "only beta is negated");
            // alpha is deduplicated to a single positive literal despite two
            // syntactic occurrences.
            assert_eq!(
                positives.iter().filter(|p| **p == "alpha").count(),
                1,
                "alpha dedups across paren + quantifier occurrences"
            );
        }
        other => panic!("expected RuleArtifact, got {other:?}"),
    }
}

/// A rule body whose literal is a parenthesized symbol still resolves the inner
/// predicate symbol (`root_symbol_name` Paren arm), lowering one positive literal.
#[test]
fn rule_body_paren_symbol_resolves_predicate() {
    let compiled =
        compile_program_structural("rule r : head if (lonely)").expect("paren-symbol body lowers");
    match compiled.compiled() {
        CompiledItemV1::RuleArtifact { program, .. } => {
            let rules = program.rules();
            let body = &rules.first().expect("one rule").body;
            assert_eq!(body.len(), 1);
            assert_eq!(body[0].predicate, "lonely");
        }
        other => panic!("expected RuleArtifact, got {other:?}"),
    }
}
