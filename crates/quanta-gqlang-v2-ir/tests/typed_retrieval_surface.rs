#![allow(
    clippy::expect_used,
    reason = "typed retrieval tests assert mode projection + fail-closed flags"
)]
//! F04/E07A hellgate — flat retrieval projects into a typed search-owner spec;
//! an unsupported search-owner flag fails closed.
use quanta_gqlang_v2_ir::retrieval::spec::{RetrievalSpecIrV1, RetrievalSyntaxIrV1};
use quanta_gqlang_v2_ir::retrieval_ir::{RetrievalIrV1, RetrievalModeIrV1};

fn retrieval_v1(query: &str, mode: RetrievalModeIrV1) -> RetrievalIrV1 {
    RetrievalIrV1 {
        query: query.to_string(),
        scope: None,
        mode,
        limit: 20,
    }
}

#[test]
fn each_mode_projects_to_its_typed_surface_v1() {
    assert!(matches!(
        retrieval_v1("auth", RetrievalModeIrV1::Lexical).typed_spec_v1().expect("text"),
        RetrievalSpecIrV1::Text(spec) if spec.syntax == RetrievalSyntaxIrV1::Native
    ));
    assert!(matches!(
        retrieval_v1("auth", RetrievalModeIrV1::Semantic)
            .typed_spec_v1()
            .expect("semantic"),
        RetrievalSpecIrV1::Semantic(_)
    ));
    assert!(matches!(
        retrieval_v1("auth", RetrievalModeIrV1::Hybrid)
            .typed_spec_v1()
            .expect("hybrid"),
        RetrievalSpecIrV1::Hybrid(_)
    ));
}

#[test]
fn unsupported_search_owner_flag_fails_closed_v1() {
    let err = retrieval_v1("auth patterntype:regexp", RetrievalModeIrV1::Lexical)
        .typed_spec_v1()
        .expect_err("unsupported flag must fail closed");
    assert_eq!(err.code, "retrieval_unsupported_flag");
}
