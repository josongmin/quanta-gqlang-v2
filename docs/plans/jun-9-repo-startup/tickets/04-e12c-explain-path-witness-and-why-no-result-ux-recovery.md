# 04 E12C Explain, Path Witness, And Why-No-Result UX Recovery

Status: `Done`

Original ticket: [E12C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E12C-explain-and-path-witness-ux.md)

## Goal

Restore the local `E12C` UX surface:

1. CLI explain presentation
2. LSP explain presentation
3. path witness and degraded badge rendering
4. dedicated `explain_path_ux` proof rail

## Depends On

1. `00-packet-scope-and-repo-boundary`
2. `03-e11c-metric-inventory-and-external-findings-recovery`

## Unblocks

1. `05-repo-local-serial-integration-and-green-rails`

## Current Base

1. `crates/quanta-gqlang-v2-cli/src/explain_view.rs` is deleted
2. `crates/quanta-gqlang-v2-lsp/src/handlers/explain_view.rs` is deleted
3. `crates/quanta-gqlang-v2-cli/tests/explain_path_ux.rs` is deleted
4. degraded-reason authority still exists and must stay shared

## Owner Files

1. `crates/quanta-gqlang-v2-cli/src/explain_view.rs`
2. `crates/quanta-gqlang-v2-lsp/src/handlers/explain_view.rs`
3. `crates/quanta-gqlang-v2-cli/tests/explain_path_ux.rs`

## Do Not Edit

1. `crates/quanta-gqlang-v2-cli/src/doctor.rs`
2. `crates/quanta-gqlang-v2-lsp/src/server.rs`
3. `crates/quanta-gqlang-v2-ir/src/catalog/external_findings.rs`
4. `crates/quanta-gqlang-v2-ir/src/root_ir.rs`

## Required End State

1. explain UX stays presentation-only
2. path witness rendering exists again without inventing runtime truth
3. degraded badges use the shared degraded-reason authority instead of duplicating strings
4. why-no-result guidance is explicit again
5. the named rail `explain_path_ux` exists and passes

## Verification Commands

1. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-cli --test explain_path_ux -- --nocapture`

## Not Done If

1. explain UX still only exists as generic doctor/headless text
2. degraded reasons are re-copied into presenter-local switches
3. runtime truth is invented in CLI or LSP messaging
4. the named rail is still missing
