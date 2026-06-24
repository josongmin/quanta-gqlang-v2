# 16 Current Main Proof and Artifact Hygiene

## Goal

Close the remaining repo-local current-main delta in `quanta-gqlang-v2` and
refresh the proof receipt so the qglang side is ready to block only on
semantica D5, not on stale local hygiene claims.

## Current-Head Truth

1. current `main` is still dirty as of `2026-06-25`, but the open delta is now
   narrowed to one tracked file only:
   `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
2. the remaining diff in that file is formatting/assert-layout churn, not a new
   crate-family or ownership lane
3. no untracked repro artifacts are present in the latest
   `git status --short --branch` output
4. current proof refresh on `2026-06-25`:
   - `git diff --check` passed
   - `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate --quiet` passed
   - `cargo test --workspace --quiet` passed
   - `cargo clippy --workspace --all-targets --quiet` passed

## Deliverables

1. decide whether the last tracked diff in `v3_family_hellgate.rs` is:
   - intentional readability/formatting worth landing, or
   - pure churn worth reverting
2. keep repo-local proof claims tied to the refreshed `2026-06-25` receipt, not
   the older bulk-dirty snapshot
3. preserve the current clean artifact boundary:
   no ad hoc repro files, no scratch prompt files outside the packet, and no
   stale dirty-tree claims in packet docs
4. record the exact proof ladder that must stay green when the last diff is
   resolved

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
5. `git diff --check`

## Definition Of Done

1. the last tracked diff is either landed intentionally or removed
2. the `2026-06-25` repo-local proof receipt is explicit and reproducible
3. packet docs no longer describe a stale large dirty-tree state
4. no one claims end-to-end advanced-serving closure from this ticket alone
