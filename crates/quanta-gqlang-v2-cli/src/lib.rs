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

pub mod commands;
pub mod doctor;
pub mod explain_view;
