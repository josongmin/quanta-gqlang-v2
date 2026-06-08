# 07 IR Facade And Lint/Test Root Cleanup

Status: `Done`

## Goal

Reduce the umbrella IR facade and shrink the broad test/lint exceptions that
were added at the crate roots.

## Depends On

1. `05-repo-local-serial-integration-and-green-rails`

## Closure Snapshot

1. `crates/quanta-gqlang-v2-ir/src/frontend.rs` is reduced to a narrow boundary shim
2. crate-root test-time allow blocks are removed
3. syntax, sema, and IR test-only lint exceptions are localized to dedicated test modules
4. `cargo clippy --workspace --tests -- -D warnings` is green

## Owner Files

1. `crates/quanta-gqlang-v2-ir/src/frontend.rs`
2. `crates/quanta-gqlang-v2-ir/src/lib.rs`
3. `crates/quanta-gqlang-v2-sema/src/lib.rs`
4. `crates/quanta-gqlang-v2-lsp/src/lib.rs`
5. `crates/quanta-gqlang-v2-cli/src/lib.rs`
6. `crates/quanta-gqlang-v2-syntax/src/lib.rs`
7. `Cargo.toml`
8. any new test-support modules created to move root-local test helpers out

## Required End State

1. the IR facade is smaller or eliminated
2. crate boundaries are visible again through direct or narrower exports
3. workspace-wide `allow_attributes` is tightened back to the intended policy
4. crate-root test helper bloat is moved into dedicated test-support modules or
   integration-test helpers
5. the repo still closes on `just ci`

## Verification Commands

1. `just ci`

## Not Done If

1. `frontend.rs` still hides most syntax/sema/IR ownership behind a broad facade
2. the crate-root allow blocks remain the practical long-term policy
3. cleanup weakens the repo's fail-closed lint posture
