# A2-3 / C-09 and owner-boundary disposition

Base: `a706c8cb3a8775f7c0e7b5ddca3563dbd0562789`.

## Acceptance and implementation

A2-3 named manual cross-owner budget projection as an extension risk. No current
wrong cap was reproduced. The required protection is an explicit mapping for
every parse-owned dimension, exact preservation of distinct caps, owner validation
on zero/overdepth, and a build obligation when the parse vocabulary grows.

Surface now exposes `ParseBudgetV2::try_from_dimensions`, applying the existing
constructor's validation. Pipeline matches the closed Surface dimension enum
exhaustively to Sema getters. Source, transport and non-parse caps remain with
their existing owners. This uses the existing closed dimension authority instead
of inventing another list of names or a heuristic to infer parser relevance.

C-09's `ResolvedDocumentV2` was not a resolution proof: the sole constructor
wrapped DeclaredDocumentV2 before body resolution ran. There were no workspace
consumers beyond Sema's own transaction. Removed that redundant public owner
wrapper/re-export and pass DeclaredDocumentV2 directly. Body resolution, catalog
validation and final semantic assembly remain in their original order. The
curated frontend API and successful source behavior are unchanged; direct users
of the unsupported Sema-owner wrapper must use the actual transaction instead.

## Proof

- Surface's regression verifies all seven callback dimensions exactly once,
  distinct literal cap preservation, all seven zero refusals, expression depth
  65 refusal and relation depth 129 refusal.
- Pipeline's regression gives the frontend constructor distinct values and
  compares against the independent literal parser profile `(31,37,41,11,13,43,47)`.
- Seven compiling mutants replace each frontend getter with another dimension's
  getter. All fail the projection equality assertion. Restored projection passes.
- A temporary `FutureDimension` enum variant plus its wire spelling makes
  `cargo check -p quanta-gqlang-v2-pipeline --lib` fail with E0004 at the Pipeline
  match. This is an extension compile obligation, not a runtime mutant kill.
- The existing Sema semantic-kernel regression passes after wrapper removal.
  Full clean-HEAD qualification is recorded separately; focused checks alone
  are not repository closure.

Raw mutation/probe logs are in the owner worktree's `target/parse-budget-audit/`.
All fault edits are restored before integration.

## Other current-source classifications

See `docs/contracts/frontend-owner-boundaries.md` for the checked-input chain.

| ID | Disposition and evidence boundary |
| --- | --- |
| C-04 | Raw Binder methods require checked bounded HIR; the production transaction satisfies this in DeclaredDocumentV2 admission before resolve_document_bodies. Clarified the precondition, without claiming raw Binder is independently stack-safe. |
| C-05 | Production uses ModuleImportAdapterV2 over Modules' sealed graph. The inert snapshot cannot mint that proof. Removed stale text claiming the Modules owner did not exist. |
| C-08 | Current authority_profile copies five corresponding caps; downstream_receipt explicitly initializes every receipt field. Current values are coherent; future semantic ownership changes require explicit stage updates. |
| C-09 incomplete carrier | Existing try_new rejects absent unresolved facts, empty codes and foreign source IDs; no conversion to closed semantics. This carrier is not a fabricated successful plan. |
| C-17 usize fallback | An unrepresentable u32 cap saturating to usize::MAX can only reduce the cap. It is exact on supported 32/64-bit hosts; classifying it as fail-open was incorrect. This says nothing about other provenance-identity claims. |

This document resolves the listed owner-boundary questions, not all historical
C-lane suggestions. In particular, it does not claim downstream overflow proof,
performance benchmarks, arbitrary raw-HIR admission safety, or a completed global
A-to-E disposition merely from green tests.
