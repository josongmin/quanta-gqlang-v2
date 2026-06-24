use quanta_gqlang_v2_algebra::{normalize_relation, AlgebraOpV1, NormalizeOptionsV1};
use quanta_gqlang_v2_hir::HirBuilderV1;

fn sp(a: usize, b: usize) -> quanta_gqlang_v2_hir::SpanV1 {
    quanta_gqlang_v2_hir::SpanV1::new(a, b)
}

fn main() {
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, sp(0, 5));
    let sorted = b.compose("sort", vec![users], sp(0, 10));
    let limited = b.compose("limit", vec![sorted], sp(0, 15));
    
    let algebra = normalize_relation(&limited, &NormalizeOptionsV1::default())
        .expect("should normalize");

    if let AlgebraOpV1::Limit(limit) = &algebra.op {
        println!("Limit count: {} (u64::MAX = {})", limit.count, u64::MAX);
        println!("Limit offset: {}", limit.offset);
        if limit.count == u64::MAX && limit.offset == 0 {
            println!("CONFIRMED: Limit values are hardcoded to MAX and 0");
        }
    } else {
        panic!("expected Limit");
    }
}
