# 03 E11C Metric Inventory And External Findings Recovery

Status: `Ready`

Original ticket: [E11C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E11C-metric-inventory-and-external-findings-surface.md)

## Goal

Restore the local `E11C` catalog plane:

1. named metric inventory
2. public external findings surface
3. dedicated `metric_inventory` proof rail

## Depends On

1. `00-packet-scope-and-repo-boundary`

## Unblocks

1. `04-e12c-explain-path-witness-and-why-no-result-ux-recovery`
2. `05-repo-local-serial-integration-and-green-rails`

## Current Base

1. `crates/quanta-gqlang-v2-ir/src/catalog/metric_inventory.rs` is deleted
2. `crates/quanta-gqlang-v2-ir/src/catalog/external_findings.rs` is deleted
3. `crates/quanta-gqlang-v2-ir/tests/metric_inventory.rs` is deleted

## Owner Files

1. `crates/quanta-gqlang-v2-ir/src/catalog/metric_inventory.rs`
2. `crates/quanta-gqlang-v2-ir/src/catalog/external_findings.rs`
3. `crates/quanta-gqlang-v2-ir/tests/metric_inventory.rs`

## Do Not Edit

1. `crates/quanta-gqlang-v2-ir/src/lib.rs`
2. `crates/quanta-gqlang-v2-cli/src/doctor.rs`
3. any LSP files

## Required End State

1. the metric inventory exists as a typed vocabulary in this repo
2. the surface is host-ingested vocabulary only, not analyzer-computed truth
3. external findings nouns exist as public frontend data vocabulary
4. no truth-promotion logic is introduced here
5. the named rail `metric_inventory` exists and passes

## Verification Commands

1. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-ir --test metric_inventory -- --nocapture`

## Not Done If

1. metrics are restored only as ad hoc strings
2. `metric.<name>` is not a typed surface again
3. the implementation introduces backend truth claims
4. the named rail is still missing

