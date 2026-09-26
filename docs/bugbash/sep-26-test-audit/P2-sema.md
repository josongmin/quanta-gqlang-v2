# P2: semantic proof gaps resolved in the working tree

## TA-06: presentation selector use location

Before: the presentation fixture matched most resolved variants with `..` and did not inspect authored reference locations.

Now: `test_support.rs` checks host selector spelling, kind, source ID, and exact use span for DESCRIBE, both COMPARE references, EXPORT, and a newly authored scoped SAMPLE fixture. It also checks the no-scope SAMPLE case. The sema owner unit test passed. A temporary DESCRIBE mutation substituting the whole presentation span made it fail; the mutation was restored.

## TA-07: two-origin provenance

Before: `v3_incremental_relation_frames.rs` supplied the same source twice; a first-input-only set constructor could pass.

Now: that test supplies two distinct identities and a duplicate, then requires exactly two retained identities. The real inner-join fixture also requires both distinct child origins in its output provenance. The integration test and sema owner unit test passed. A temporary first-input-only constructor made the integration test fail; the mutation was restored.

## TA-08: types-kernel invocation claim

Before: `document.rs` incremented a test-only marker immediately before `check_document`; the test counted that marker rather than invocations.

Now: the marker and its exactly-once assertion are removed. The renamed mixed-document test asserts the exact identities and order of five resolved items and passed. Current production source has one `check_document` call in the transaction path, but this test makes no dynamic exactly-once claim.
