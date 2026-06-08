# 06 LSP Read-Only Interpreter Extraction

Status: `Ready`

## Goal

Extract the shared read-only interpretation logic out of `hover.rs` so hover,
diagnostics, symbols, and quickfix stop depending on one oversized owner file.

## Depends On

1. `05-repo-local-serial-integration-and-green-rails`

## Current Base

1. `crates/quanta-gqlang-v2-lsp/src/handlers/hover.rs` is the shared interpreter
2. `diagnostics.rs`, `symbols.rs`, and `quickfix.rs` all depend on that owner
3. this is a repo-local cleanup ticket, not an RFC ticket restore

## Owner Files

1. `crates/quanta-gqlang-v2-lsp/src/handlers/hover.rs`
2. `crates/quanta-gqlang-v2-lsp/src/handlers/diagnostics.rs`
3. `crates/quanta-gqlang-v2-lsp/src/handlers/symbols.rs`
4. `crates/quanta-gqlang-v2-lsp/src/handlers/quickfix.rs`
5. any new read-only analysis helper module under `crates/quanta-gqlang-v2-lsp/src/handlers/`

## Required End State

1. one shared read-only analysis owner exists outside `hover.rs`
2. hover, diagnostics, symbols, and quickfix become thin adapters
3. no new mutating method semantics are introduced
4. behavior stays fail-closed

## Verification Commands

1. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-lsp --test read_only_lsp -- --nocapture`
2. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-lsp --test autocomplete_quickfix -- --nocapture`
3. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml --workspace -- --nocapture`

## Not Done If

1. the shared interpreter still lives only in `hover.rs`
2. a handler regains mutation or backend truth behavior
3. cleanup changes the visible read-only shell contract

