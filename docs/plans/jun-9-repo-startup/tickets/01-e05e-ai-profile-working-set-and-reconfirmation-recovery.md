# 01 E05E AI Profile, Working Set, And Reconfirmation Recovery

Status: `Done`

Original ticket: [E05E](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E05E-ai-exploration-profile-working-set-and-reconfirmation-semantics.md)

## Goal

Restore the local `E05E` surface inside this repo:

1. AI exploration profile semantics
2. typed working-set carriers
3. explicit reconfirmation rules
4. a dedicated `ai_profile_contract` proof rail

## Depends On

1. `00-packet-scope-and-repo-boundary`

## Unblocks

1. `05-repo-local-serial-integration-and-green-rails`

## Current Base

1. `crates/quanta-gqlang-v2-ir/src/ai_plan.rs` still exists but is narrowed
2. `crates/quanta-gqlang-v2-ir/src/working_set.rs` still exists but is minimal
3. sema-side AI/profile/admissibility files were deleted
4. `crates/quanta-gqlang-v2-ir/tests/ai_profile_contract.rs` is missing

## Owner Files

1. `crates/quanta-gqlang-v2-sema/src/ai/`
2. `crates/quanta-gqlang-v2-sema/src/profile.rs`
3. `crates/quanta-gqlang-v2-sema/src/admissibility.rs`
4. `crates/quanta-gqlang-v2-ir/src/ai_plan.rs`
5. `crates/quanta-gqlang-v2-ir/src/working_set.rs`
6. `crates/quanta-gqlang-v2-ir/tests/ai_profile_contract.rs`

## Do Not Edit

1. `crates/quanta-gqlang-v2-sema/src/lib.rs`
2. `crates/quanta-gqlang-v2-ir/src/lib.rs`
3. `docs/language/ai-profile.md`
4. any CLI or LSP entrypoint files

## Required End State

1. AI remains a profile on the same DSL, not a separate runtime contract
2. `AiExplorationPlanV1`, `AiObjectiveV1`, `ExplorationBudgetV1`, and
   `WorkingSetV1` are explicit again
3. reconfirmation is explicit as same-intent replay through `ci_gate` or `lint`
4. final policy truth cannot be emitted directly from the AI lane
5. provider-local or runtime-local IDs do not leak into the public carriers
6. the named rail `ai_profile_contract` exists and passes

## Verification Commands

1. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-ir --test ai_profile_contract -- --nocapture`
2. `cargo test --manifest-path /Users/songmin/Documents/code-new/quanta-gqlang-v2/Cargo.toml -p quanta-gqlang-v2-sema -- --nocapture`

## Not Done If

1. AI still has only a narrowed placeholder shape
2. reconfirmation is implied but not encoded
3. `ai_profile_contract` is restored only as generic workspace coverage
4. the restored shape invents backend truth or runtime execution behavior
