# P2: current sema proof gaps

These are untested semantic invariants in the dirty working tree, not observed production failures. The two mechanical fail-open fixes in `src/test_support.rs` are present; they are not open findings.

## TA-06: presentation selector use location is not pinned

- `crates/quanta-gqlang-v2-sema/src/test_support.rs:4932-4965` names an exact-payload test, but DESCRIBE matches `Describe { .. }` and SAMPLE/COMPARE/EXPORT ignore their reference payloads. The authored DESCRIBE selector is `input_a` at `17..24` (`:3970,4077-4079`).
- `src/document.rs:5220-5238` currently resolves the DESCRIBE use at `path.span`. A change to the containing presentation span would preserve the variant and target identity, leaving this test green. The pipeline presentation rail checks selector spellings but not these use locations.
- Repair: assert the resolved target and exact checked use location for DESCRIBE, then cover the authored reference locations of COMPARE/EXPORT and any optional SAMPLE scope. A selector-location mutation must fail the owner test.

## TA-07: provenance set is only tested with one distinct source

- `tests/v3_incremental_relation_frames.rs:48-52` constructs `OutputProvenanceV2::new([relation.clone(), relation.clone()])` and expects one value. It proves deduplication, but a constructor that keeps only the first input also passes.
- `src/document.rs:4140-4145` unions left and right provenance for a join. No sema assertion found in this audit requires two distinct origins in that output.
- Repair: construct two distinct source identities plus a duplicate, assert the exact two-element set, and check a join's output provenance. A first-element-only mutation must fail.

## TA-08: the exactly-once kernel test counts a nearby marker

- `src/document.rs:6105-6107` calls `record_types_kernel_document_call()` immediately before `kernel.check_document(&type_inputs)`. The test at `:7255-7279` asserts the marker count is one.
- A second `check_document` call without that marker would leave the counter at one. Current source shows one call; no duplicate execution was observed.
- Repair: count actual kernel invocations at the call boundary or use a single-use session wrapper. If exact invocation count is not a maintained contract, narrow the test name and ledger claim instead. A second invocation must be detected before retaining the exactly-once claim.
