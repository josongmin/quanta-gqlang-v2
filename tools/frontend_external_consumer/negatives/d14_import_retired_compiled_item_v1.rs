//! Negative case D14_import_retired_compiled_item_v1 (class D: retired boundary).
//!
//! V1 HIR and compiled-item DTOs are not a compatibility surface. Downstream
//! serving must consume `CompiledDocumentV2::execution()` and its sealed V2
//! execution DTOs instead.
//!
//! Expected: E0432, "no `CompiledItemV1` in the root".
use quanta_gqlang_v2_frontend_contract::CompiledItemV1;

fn main() {
    let _ = std::mem::size_of::<CompiledItemV1>();
}
