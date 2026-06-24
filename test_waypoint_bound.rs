use quanta_gqlang_v2_hir::{GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, HirNodeIdV1, SpanV1};
use quanta_gqlang_v2_logic::GraphQuerySemanticsV1;

fn id(n: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(n)
}

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

fn gref(name: &str, a: usize, b: usize) -> GraphEndpointsV1 {
    GraphEndpointsV1 {
        source: Some(quanta_gqlang_v2_hir::EntityRefHirV1::new(
            id(1),
            sp(a, b),
            name.to_owned(),
        )),
        ..GraphEndpointsV1::default()
    }
}

fn main() {
    // Create a reachability query with 2 waypoints (via A, via B)
    let endpoints = GraphEndpointsV1 {
        source: Some(quanta_gqlang_v2_hir::EntityRefHirV1::new(
            id(1),
            sp(0, 3),
            "source".to_owned(),
        )),
        sink: Some(quanta_gqlang_v2_hir::EntityRefHirV1::new(
            id(2),
            sp(5, 9),
            "sink".to_owned(),
        )),
        ..GraphEndpointsV1::default()
    };
    
    let constraints = GraphConstraintsV1 {
        via: vec![
            quanta_gqlang_v2_hir::EntityRefHirV1::new(id(3), sp(12, 13), "A".to_owned()),
            quanta_gqlang_v2_hir::EntityRefHirV1::new(id(4), sp(15, 16), "B".to_owned()),
        ],
        ..GraphConstraintsV1::default()
    };
    
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 20),
        GraphQueryKindV1::Reachability,
        endpoints,
        constraints,
    );
    
    let contract = GraphQuerySemanticsV1::classify(&query).expect("should classify");
    
    println!("Max path length: {:?}", contract.witness.max_path_length);
    println!("With 2 waypoints, the bound is: {}", contract.witness.max_path_length.unwrap_or(0));
    println!("But a minimal path needs at least 3 hops: source -> A -> B -> sink (or other orderings)");
    println!("The bound of {} is LESS than the minimum required hops!", contract.witness.max_path_length.unwrap_or(0));
}
