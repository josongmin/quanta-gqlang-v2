# 02 E06C Public Planning And Lowering-Ready IR Recovery

Status: `Done`

Original ticket: [E06C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E06C-public-planning-and-lowering-ready-ir.md)

## Goal

Restore the local `E06C` planning surface:

1. public planning IR
2. lowering-ready IR metadata preservation
3. explicit capability, truth, and bridge metadata
4. a dedicated `public_planning_ir` proof rail

## Depends On

1. `00-packet-scope-and-repo-boundary`

## Unblocks

1. `05-repo-local-serial-integration-and-green-rails`

## Current Base

1. `crates/quanta-gqlang-v2-ir/src/planning.rs` exists but is narrowed
2. `crates/quanta-gqlang-v2-ir/src/lowering_ready.rs` exists but is narrowed
3. `crates/quanta-gqlang-v2-ir/tests/public_planning_ir.rs` is missing

## Owner Files

1. `crates/quanta-gqlang-v2-ir/src/planning.rs`
2. `crates/quanta-gqlang-v2-ir/src/lowering_ready.rs`
3. `crates/quanta-gqlang-v2-ir/tests/public_planning_ir.rs`

## Do Not Edit

1. `crates/quanta-gqlang-v2-ir/src/lib.rs`
2. `crates/quanta-gqlang-v2-sema/src/lib.rs`
3. any CLI or LSP entrypoint files

## Required End State

1. planning IR remains public-language-facing
2. `required_capabilities`, `truth_expectation`, `bridge_mode`, and export mode
   are explicit
3. roots that require planning metadata fail closed when it is absent
4. lowering-ready normalization preserves planning metadata instead of silently
   dropping it
5. provider IDs, runtime-local bound plans, and execution-lane carriers do not
   leak into this repo's public planning model
6. the named rail `public_planning_ir` exists and passes

## Verification Commands

1. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-ir --test public_planning_ir -- --nocapture`

## Not Done If

1. planning metadata is still optional where the ticket requires it
2. truth/bridge/capability metadata are not all explicit
3. `public_planning_ir` is not restored as a standalone rail
4. the restored IR imports runtime-local or provider-local nouns
