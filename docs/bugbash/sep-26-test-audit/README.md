# Sep 26 test audit: resolution record

Current-source follow-up: [Sep 26 re-audit and implemented improvements](reaudit-2026-09-26.md).
It records RA-01~05, the optional test/tooling improvements, and 12 compiling
mutations rejected by 13 targeted test runs; its gate evidence is separate from
the historical integration result below.

Implementation base: committed HEAD `27386d1776710022372380a0bd30bde2efdb740c`; local integration ran on `2d5647b3d48a3d4a6f76cc4cae6f06d2a0a7120c` plus this patch. The audit found test-oracle and proof-claim gaps; it did not establish a production behavior failure. The seven retained gaps are addressed. Clean-HEAD qualification is a separate result.

| ID | Resolution | Detail |
|---|---|---|
| TA-01 | Shipped diagnostics binary test pins one complete malformed-source diagnostic. | [P8](P8-front-doors.md#ta-01-error-diagnostic-identity) |
| TA-05 | Shipped CLI test pins exact canonical output for a messy declaration. | [P8](P8-front-doors.md#ta-05-canonical-cli-output) |
| TA-06 | Sema fixture checks selector identity and exact use location, including scoped SAMPLE. | [P2](P2-sema.md#ta-06-presentation-selector-use-location) |
| TA-07 | Constructor and real join tests require two distinct provenance origins while deduplicating repeats. | [P2](P2-sema.md#ta-07-two-origin-provenance) |
| TA-08 | Removed the unbound counter and exactly-once test claim; the mixed-document test now pins five item identities and their order. | [P2](P2-sema.md#ta-08-types-kernel-invocation-claim) |
| TA-09 | HIR visitor tests trace auxiliary spans across the relevant families and reject one out-of-bounds reference. | [P7](P7-hir-binder.md#ta-09-auxiliary-span-visitor-coverage) |
| TA-10 | Binder test distinguishes immediate noncapturing and capturing child scopes. | [P7](P7-hir-binder.md#ta-10-immediate-noncapturing-barrier) |

Focused positive tests passed: diagnostics binary 1, CLI binary 1, binder 1, sema library 54, sema provenance integration 1, and HIR traversal 14. Six temporary behavior mutations (TA-01, TA-05, TA-06, TA-07, TA-09, TA-10) each made its targeted test fail and were restored. The first TA-05 mutation only caused a compile error and was discarded; the reported result uses a compiling validated-identity mutation. TA-08 resolves a false proof claim rather than adding a call-count test.

TA-02 was rejected after comparing combined and individual malformed requests. TA-03 and TA-04 were repaired by the P9 owner in the base commit; they are not open findings.

Dirty-tree `just ci` passed all groups on source snapshot `98ebe50dcfd62363df1da137b0cc8031d988b5a2a0de3d2ffaa9f97386bf9fdc` (nextest 1263/1263, doctest 5/5, front-door parity 10/10, plus policy and external-consumer gates). The first run failed Clippy on a remaining assertion in the renamed TA-08 test; it was changed to a `Result` error before the successful rerun. This local CI result is not a clean committed-HEAD qualification receipt.
