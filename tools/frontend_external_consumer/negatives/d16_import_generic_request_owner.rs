//! The facade does not export a generic request owner that could retain
//! arbitrary producer values outside the sealed parsed/checked stages.

use quanta_gqlang_v2_frontend_contract::FrontendRequestOwnedValueV3;

fn main() {
    let _ = std::mem::size_of::<FrontendRequestOwnedValueV3<()>>();
}
