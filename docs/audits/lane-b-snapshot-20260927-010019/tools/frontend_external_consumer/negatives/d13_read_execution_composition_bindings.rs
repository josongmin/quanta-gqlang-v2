//! Negative case D13_read_execution_composition_bindings (class D: V2 handoff privacy).
//!
//! Binding order, producer/body IDs, and the WITH target are emitted together
//! by the frontend. A downstream adapter may read them only through
//! `bindings()` and `with_binding()`; it cannot replace or reorder the backing
//! array.
//!
//! Expected: E0616, "field `bindings` of struct `ExecutionCompositionV3` is private".
use quanta_gqlang_v2_frontend_contract::ExecutionCompositionV3;

fn probe(composition: &ExecutionCompositionV3) {
    let _ = &composition.bindings;
}

fn main() {
    let _ = probe as fn(&ExecutionCompositionV3);
}
