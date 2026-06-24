// Comprehensive audit of all 18 families
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration test audit: panic on failure is intentional"
)]

use quanta_gqlang_v2_pipeline::{compile_program_structural, CompiledItemV1};

// Family 1: SELECT (Relational Query) - no schema so only literal projections
#[test]
fn f01_relational_query_select() {
    let src = "from users | project 1 as x, 2 as y";
    let compiled = compile_program_structural(src).expect("SELECT must compile");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::RelationalQuery { .. }),
        "SELECT should produce RelationalQuery"
    );
    assert!(compiled.query().is_some(), "SELECT carries QueryHirV1");
    assert!(compiled.algebra().is_some(), "SELECT carries AlgebraNodeV1");
    assert!(compiled.plan().is_some(), "SELECT carries PublicPlanV1");
}

// Family 2-8: FIND subkinds (Graph Queries)
#[test]
fn f02_find_reachability() {
    let src = "find reachability src -> dst | depth=5";
    let compiled = compile_program_structural(src).expect("FIND reachability must compile");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::GraphQuery { .. }),
        "FIND should produce GraphQuery"
    );
    assert!(compiled.plan().is_some(), "FIND carries path_witness plan");
}

#[test]
fn f03_find_flow() {
    let src = "find flow api.entry -> db.write | depth=3";
    let compiled = compile_program_structural(src).expect("FIND flow must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::GraphQuery { .. }
    ));
}

// FIND slice requires anchor endpoint
#[test]
fn f04_find_slice() {
    let src = "find slice | anchor=var | depth=2";
    let compiled = compile_program_structural(src).expect("FIND slice must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::GraphQuery { .. }
    ));
}

#[test]
fn f05_find_taint() {
    let src = "find taint a.b -> c.d | depth=2";
    let compiled = compile_program_structural(src).expect("FIND taint must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::GraphQuery { .. }
    ));
}

#[test]
fn f06_find_path() {
    let src = "find path src -> dst | depth=1";
    let compiled = compile_program_structural(src).expect("FIND path must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::GraphQuery { .. }
    ));
}

// FIND points_to requires reference endpoint
#[test]
fn f07_find_points_to() {
    let src = "find points_to | reference=ptr | pointee=obj | depth=2";
    let compiled = compile_program_structural(src).expect("FIND points_to must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::GraphQuery { .. }
    ));
}

// FIND summary requires depth or facets with bounding constraint
#[test]
fn f08_find_summary() {
    let src = "find summary | callable=func | facets=input,output | depth=1";
    let compiled = compile_program_structural(src).expect("FIND summary must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::GraphQuery { .. }
    ));
}

// Family 9: CHECK (Legality)
#[test]
fn f09_check_legality() {
    let src = "check rule.sqli | scope=app | confirm=require | emit=findings | mode=hybrid";
    let compiled = compile_program_structural(src).expect("CHECK must compile");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::Legality { .. }),
        "CHECK should produce Legality"
    );
    assert!(compiled.plan().is_some(), "CHECK carries alert plan");
}

// Families 10-15: Presentation verbs
#[test]
fn f10_explain_presentation() {
    let src = "explain rule.auth | evidence=trace | truth=confirm";
    let compiled = compile_program_structural(src).expect("EXPLAIN must compile");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::Presentation { .. }),
        "EXPLAIN should produce Presentation"
    );
    assert!(compiled.plan().is_some(), "EXPLAIN carries plan");
}

#[test]
fn f11_show_presentation() {
    let src = "show models";
    let compiled = compile_program_structural(src).expect("SHOW must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::Presentation { .. }
    ));
}

#[test]
fn f12_describe_presentation() {
    let src = "describe rule::sqli";
    let compiled = compile_program_structural(src).expect("DESCRIBE must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::Presentation { .. }
    ));
}

#[test]
fn f13_sample_presentation() {
    let src = "sample query=\"select * from users\" | mode=lexical | limit=10";
    let compiled = compile_program_structural(src).expect("SAMPLE must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::Presentation { .. }
    ));
}

#[test]
fn f14_compare_presentation() {
    let src = "compare baseline=v1 | candidate=v2 | metric=semantic";
    let compiled = compile_program_structural(src).expect("COMPARE must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::Presentation { .. }
    ));
}

#[test]
fn f15_export_presentation() {
    let src = "export source=query | kind=json";
    let compiled = compile_program_structural(src).expect("EXPORT must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::Presentation { .. }
    ));
}

// Families 16-21: Catalog declarations
#[test]
fn f16_view_catalog_decl() {
    let src = "view my.view | query-ref=base.query";
    let compiled = compile_program_structural(src).expect("VIEW must compile");
    assert!(
        matches!(
            compiled.compiled(),
            CompiledItemV1::CatalogRegistration { .. }
        ),
        "VIEW should produce CatalogRegistration"
    );
    assert!(compiled.plan().is_none(), "VIEW produces no plan");
}

#[test]
fn f17_model_catalog_decl() {
    let src = "model my.model | provider=openai | source=models.gpt4";
    let compiled = compile_program_structural(src).expect("MODEL must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::CatalogRegistration { .. }
    ));
}

#[test]
fn f18_pack_catalog_decl() {
    let src = "pack my.pack | source=packs.v1 | activation=session";
    let compiled = compile_program_structural(src).expect("PACK must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::CatalogRegistration { .. }
    ));
}

#[test]
fn f19_profile_catalog_decl() {
    let src = "profile my.profile | capabilities=cap1,cap2 | admission=allowlist";
    let compiled = compile_program_structural(src).expect("PROFILE must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::CatalogRegistration { .. }
    ));
}

#[test]
fn f20_gate_catalog_decl() {
    let src = "gate my.gate | checks=check1,check2 | binding=all";
    let compiled = compile_program_structural(src).expect("GATE must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::CatalogRegistration { .. }
    ));
}

#[test]
fn f21_rule_binding_catalog_decl() {
    let src = "ruledef my.rule | check-ref=rules.check";
    let compiled = compile_program_structural(src).expect("RULEDEF must compile");
    assert!(matches!(
        compiled.compiled(),
        CompiledItemV1::CatalogRegistration { .. }
    ));
}

// Families 22-23: Meta directives
// NOTE: USE PACK requires a valid manifest; with empty PackGraphV1, unknown pack id fails.
// This is deferred (known capability).
#[test]
fn f22_use_pack_meta_reject() {
    let src = "use pack my.pack | activation=session | lockfile=my.lock";
    let result = compile_program_structural(src);
    // Expected to fail with modules diagnostic (unknown pack)
    assert!(result.is_err(), "USE PACK with unknown pack should fail");
    let err = result.unwrap_err();
    assert_eq!(err.stage().as_str(), "modules");
}

#[test]
fn f23_require_certified_meta() {
    let src = "require certified";
    let compiled = compile_program_structural(src).expect("REQUIRE CERTIFIED must compile");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::Meta { .. }),
        "REQUIRE CERTIFIED should produce Meta"
    );
    assert!(
        compiled.plan().is_some(),
        "REQUIRE CERTIFIED carries gate plan"
    );
}

// Family 24: Composition
#[test]
fn f24_composition_let_with() {
    let src = "let q = find taint a -> b | depth=2 find taint x -> y | depth=1 with q";
    let compiled = compile_program_structural(src).expect("COMPOSITION must compile");
    assert!(
        matches!(compiled.compiled(), CompiledItemV1::Composition { .. }),
        "COMPOSITION should produce Composition"
    );
    assert!(compiled.plan().is_some(), "COMPOSITION's body carries plan");
}
