# 05 Repo-Local Serial Integration And Green Rails

Status: `Done`

## Goal

Serially integrate `01` through `04`, restore the repo-local glue, and close the
repo-local proof story with `just ci`.

## Depends On

1. `01-e05e-ai-profile-working-set-and-reconfirmation-recovery`
2. `02-e06c-public-planning-and-lowering-ready-ir-recovery`
3. `03-e11c-metric-inventory-and-external-findings-recovery`
4. `04-e12c-explain-path-witness-and-why-no-result-ux-recovery`

## Unblocks

1. `06-lsp-read-only-interpreter-extraction`
2. `07-ir-facade-and-lint-test-root-cleanup`

## Current Base

1. shared integration files are heavily modified and must stay single-writer
2. local docs still describe the narrowed read-only shell story
3. `Cargo.toml` currently fails `taplo fmt --check`

## Owner Files

1. `crates/quanta-gqlang-v2-sema/src/lib.rs`
2. `crates/quanta-gqlang-v2-ir/src/lib.rs`
3. `crates/quanta-gqlang-v2-cli/src/doctor.rs`
4. `crates/quanta-gqlang-v2-cli/src/main.rs`
5. `crates/quanta-gqlang-v2-lsp/src/server.rs`
6. `crates/quanta-gqlang-v2-lsp/src/main.rs`
7. `crates/quanta-gqlang-v2-lsp/src/handlers/mod.rs`
8. `Cargo.toml`
9. `docs/language/README.md`
10. `docs/language/release_discipline.md`
11. `docs/language/ai-profile.md`

## Required End State

1. shared module wiring for `01` through `04` is restored
2. local docs describe the restored surfaces without changing the repo boundary:
   - frontend-only
   - read-only shell
   - no runtime truth claims
3. `Cargo.toml` is formatted and accepted by `taplo`
4. the repo-local proof story is:
   - named rails restored
   - `cargo test --workspace` green
   - `just ci` green

## Verification Commands

1. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml --workspace -- --nocapture`
2. `just ci`

## Not Done If

1. any of the named rails are still absent
2. docs still describe the narrower base as if the restored local ticket scope does not exist
3. `taplo`, `clippy`, `nextest`, doc tests, or semgrep are still red
4. the repo-local closeout claim depends on any cross-repo gate
