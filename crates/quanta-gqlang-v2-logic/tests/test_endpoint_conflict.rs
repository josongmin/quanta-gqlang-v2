#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test harness uses panics"
)]
//! Test to verify that FIND endpoint validation rejects extra endpoints on subkinds
//! that don't require them (e.g., slice with source/sink).

use quanta_gqlang_v2_hir::{
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1,
    HirNodeIdV1, SpanV1,
};
use quanta_gqlang_v2_logic::GraphQuerySemanticsV1;

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

fn id(n: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(n)
}

fn gref(name: &str, a: usize, b: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.to_owned(),
        span: sp(a, b),
    }
}

#[test]
fn slice_with_extra_endpoints_should_reject() {
    // slice requires ONLY anchor, but we supply source and sink too
    let endpoints = GraphEndpointsV1 {
        anchor: Some(gref("p.foo", 20, 25)),
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 40),
        GraphQueryKindV1::Slice,
        endpoints,
        GraphConstraintsV1 {
            depth: Some(5),
            ..GraphConstraintsV1::default()
        },
    );

    // Should reject because slice doesn't allow source/sink
    let result = GraphQuerySemanticsV1::classify(&query);
    match result {
        Ok(_) => panic!("FAIL: slice with source/sink should be rejected but was accepted"),
        Err(e) => {
            println!("PASS: slice with extra endpoints rejected: {}", e.message());
        }
    }
}
