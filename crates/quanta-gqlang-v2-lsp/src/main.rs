#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic,
        reason = "unit-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
    )
)]

fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(quanta_gqlang_v2_lsp::run_stdio_front_door_v1())
}
