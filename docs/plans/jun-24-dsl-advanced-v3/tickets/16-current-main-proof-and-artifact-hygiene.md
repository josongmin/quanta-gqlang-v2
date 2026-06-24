# 16 Current Main Proof and Artifact Hygiene

Status: materially closed on current head; proof refreshed and worktree clean

## Goal

Close the remaining repo-local current-main delta in `quanta-gqlang-v2` and
refresh the proof receipt so the qglang side is ready to block only on
semantica D5, not on stale local hygiene claims.

## Current-Head Truth

1. the working tree is clean, but the COMMITTED tree had carried four repo-root
   artifacts that an empty `git status` masks (they were tracked, not untracked):
   `libtest_soundness_issue.rlib`, `test_limit_claim.rs`,
   `test_soundness_issue.rs`, `test_waypoint_bound.rs`. All four were removed with
   `git rm`; a Rust workspace keeps sources under `crates/<crate>` and compiled
   objects under `target/`, never at the repo root.
2. the artifact boundary is now ENFORCED, not asserted by hand: the hellgate rail
   `crates/quanta-gqlang-v2-conformance/tests/current_head_artifact_hygiene.rs`
   fails closed if any loose `.rs` / `.rlib` / `.rmeta` artifact reappears at the
   root (RED before removal, listing the four offenders; GREEN after). `.gitignore`
   additionally ignores `*.rlib`, `*.rmeta`, and `/test_*.rs`.
3. `git diff --check` is clean
4. current proof refresh on `2026-06-25`:
   - `cargo fmt --all -- --check` passed
   - `cargo clippy --workspace --all-targets` passed (0 warnings/errors)
   - `cargo test --workspace` passed: 418 ok / 0 failed
5. the older broad dirty-tree snapshot and the later one-file narrowed snapshot
   are both stale and must not be reused as current-head truth

## Deliverables

1. keep repo-local proof claims tied to the refreshed `2026-06-25` receipt, not
   any older dirty-tree snapshot
2. preserve the current clean artifact boundary:
   no ad hoc repro files, no scratch prompt files outside the packet, and no
   stale dirty-tree claims in packet docs
3. record the exact proof ladder that must stay green when current head moves

## Required Proof Rails

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets`
3. `cargo test --workspace`
4. artifact-hygiene guard (this ticket's hellgate):
   `crates/quanta-gqlang-v2-conformance/tests/current_head_artifact_hygiene.rs`
5. targeted rails that must stay green:
   - `crates/quanta-gqlang-v2-pipeline/tests/v3_alias_type_closure_gap.rs`
   - `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
   - `crates/quanta-gqlang-v2-frontend-contract/tests/downstream_contract.rs`
   - CLI/LSP parity rails under `crates/quanta-gqlang-v2-cli/tests` and
     `crates/quanta-gqlang-v2-lsp/tests`
5. `git diff --check`

## Definition Of Done

1. the `2026-06-25` repo-local proof receipt is explicit and reproducible
2. packet docs no longer describe a stale dirty-tree state
3. no one claims end-to-end advanced-serving closure from this ticket alone
