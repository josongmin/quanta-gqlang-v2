# 16 Current Main Proof and Artifact Hygiene

## Goal

Freeze the current dirty-tree truth on `main`, classify every open change into a
small number of reviewable buckets, and define the exact proof/artifact hygiene
required before any packet closeout or end-to-end serving claim.

## Current-Head Truth

1. current `main` is dirty as of `2026-06-25`
2. current tracked changes fall into three buckets:
   - language core: `algebra`, `pipeline`, `planning`, `surface`, `types`
   - tooling/proof: `cli`, `lsp`, `conformance`, `frontend-contract`, owner
     rails
   - packet/docs: `docs/language/README.md`, `CUTOVER_INVENTORY.md`,
     `SERVING_HANDOFF.md`, and the `jun-24-dsl-advanced-v3` ticket pack
3. current diff stat snapshot is `52 files changed, 1529 insertions, 568 deletions`
4. the latest `git status --short --branch` snapshot shows tracked modifications
   only; no untracked repro artifacts are present in the current status output

## Deliverables

1. classify every touched path into:
   - `language core`
   - `tooling/proof`
   - `packet/docs`
2. keep repo-local proof claims tied to current tracked deltas, not to older
   green receipts
3. define untracked repro policy:
   - keep only stable, intentionally named regression rails that are wired into
     the proof plan
   - delete scratch repro files, one-off shell scripts, and ad hoc experiments
     before closeout
4. record the exact proof ladder required to move from dirty current-main to a
   credible closeout receipt

## Required Proof Rails

1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets`
3. `cargo test --workspace`
4. targeted rails that must stay green:
   - `crates/quanta-gqlang-v2-pipeline/tests/v3_alias_type_closure_gap.rs`
   - `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
   - `crates/quanta-gqlang-v2-frontend-contract/tests/downstream_contract.rs`
   - CLI/LSP parity rails under `crates/quanta-gqlang-v2-cli/tests` and
     `crates/quanta-gqlang-v2-lsp/tests`

## Definition Of Done

1. every open change is classified under one of the three buckets
2. artifact keep/delete rules are explicit and fail closed
3. repo-local proof commands are pinned and reproducible
4. no one claims end-to-end advanced-serving closure from this ticket alone
