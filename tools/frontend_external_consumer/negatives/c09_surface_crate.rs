//! Negative case C09_surface_crate (class C: undeclared owner crate).
//!
//! The fixture's only declared dependency is the facade, so `quanta_gqlang_v2_surface` is not in
//! its extern prelude even where it is a transitive edge of the graph. An owner
//! crate is never a legal import for a sibling repository.
//!
//! Expected: E0432, "use of undeclared crate or module `quanta_gqlang_v2_surface`".
use quanta_gqlang_v2_surface::ParseBudgetV2;

fn main() {}
