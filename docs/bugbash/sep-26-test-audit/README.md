# Sep 26 test audit: final triage and repair order

Audit basis: committed HEAD `0492d3ea22ccee5f1024e04b7638adfcab85d2ec` plus the uncommitted source/test edits observed on 2026-09-26. The findings below are test or gate-contract gaps; no production behavior failure was established. Focused checks and source inspection are not `just ci` or clean-HEAD `just qualify-local` receipts.

| ID | Current finding | Evidence | Repair |
|---|---|---|---|
| TA-01 | An error response with the wrong nonempty diagnostic list still passes its binary test. | [P8](P8-front-doors.md#ta-01-error-diagnostic-identity-is-unchecked) | Assert diagnostic count, code, source ID, and span for the existing malformed-source fixture. |
| TA-05 | The shipped CLI formatter can return a noncanonical fixed point without failing its messy-input tests. | [P8](P8-front-doors.md#ta-05-the-cli-formatter-has-no-noncanonical-output-oracle) | Pin one exact canonical binary output from a messy input. |
| TA-06 | Sema's presentation fixture does not verify authored selector use locations. | [P2](P2-sema.md#ta-06-presentation-selector-use-location-is-not-pinned) | Assert resolved selectors and exact use spans. |
| TA-07 | Sema's provenance test cannot detect loss of a second distinct source. | [P2](P2-sema.md#ta-07-provenance-set-is-only-tested-with-one-distinct-source) | Test deduplication and two-source retention, including a join. |
| TA-08 | The exactly-once types-kernel test counts a marker adjacent to the call, not the invocation. | [P2](P2-sema.md#ta-08-the-exactly-once-kernel-test-counts-a-nearby-marker) | Instrument the actual call boundary or narrow the claim. |
| TA-09 | HIR visitor auxiliary spans outside the catalog matrix lack exact trace coverage. | [P7](P7-hir-binder.md#ta-09-auxiliary-spans-are-not-traced-across-the-visitor-families) | Add per-family span traces and an invalid-span control. |
| TA-10 | Binder's immediate noncapturing shadow barrier is untested. | [P7](P7-hir-binder.md#ta-10-an-immediate-noncapturing-shadow-barrier-lacks-a-control) | Pair immediate noncapturing and capturing scope controls. |

## Repair order and acceptance

1. **Front doors (TA-01, TA-05):** assert one complete diagnostic from the error fixture and one exact canonical CLI result for a messy source. Check that a wrong-but-nonempty diagnostic and a validated identity formatter fail their respective tests.
2. **Sema (TA-06, TA-07):** pin authored presentation selector locations and a two-distinct-source provenance result. Exercise a wrong selector span and a first-source-only provenance mutation against the focused rails.
3. **HIR and binder (TA-09, TA-10):** add exact auxiliary-span traces for the untested visitor families and an immediate `NonCapturing` shadow case with a `Capturing` control. Remove a representative span callback and the early barrier guard in isolated mutation checks; each must fail.
4. **Types-kernel proof (TA-08):** count actual `check_document` invocations at the boundary or drop the exactly-once claim. Keep the claim only if a second unmarked call is detected by the rail.
5. Integrate the owned edits on a stable tree, run the affected owner tests, then `just ci` once. Claim a clean committed-HEAD receipt only after committing the intended tree and running `just qualify-local` from that clean state.

## Rejected candidate

The prior TA-02 claim about a combined `source:[]` and `extra` request was false. The current diagnostics binary rejects the combined request for the wrong `source` type before reading `extra`; the same type error occurs without `extra`, while `extra` alone produces an unknown-field error. No test change is required for that candidate.

TA-03 and TA-04 were repaired in the concurrent P9 owner edits: the child-PID probe now requires an independently absent PID for a nonzero `ps` result, and the two-build rebuild smoke again uses a non-discovered `smoke_` method. The owner reported focused checks; current-source inspection confirms both edits. They are not open items or clean-HEAD qualification claims.

Historic per-owner passes are not current integration proof. The deep reports were screened for the source-backed gaps above; their other weak-oracle candidates were not promoted to required fixes without an independent contract or uncovered route.
