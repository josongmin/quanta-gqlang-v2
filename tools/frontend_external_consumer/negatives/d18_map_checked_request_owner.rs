//! A sibling crate cannot replace checked HIR through a producer mapping hook.

use quanta_gqlang_v2_frontend_contract::CheckedRequestSourceV3;

fn probe(checked: CheckedRequestSourceV3) {
    let _ = checked.try_map_v3(|_| ());
}

fn main() {
    let _ = probe as fn(CheckedRequestSourceV3);
}
