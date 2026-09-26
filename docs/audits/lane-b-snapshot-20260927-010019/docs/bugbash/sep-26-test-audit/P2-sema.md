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

## Final supplement (P2 round 3)

After TA-06/07/08, a final pass closed four residual oracle gaps.

- **TA-06 extension — EXPLAIN target.** `every_explain_evidence_truth_permutation_resolves`
  now destructures the EXPLAIN `target` instead of `..` and asserts the resolved host
  entity selector's spelling, kind, entity subtype, and exact use span (`8..15`). Before,
  the target was unchecked in all 12 permutations.
- **TA-06 extension — selector entity subtype and SAMPLE metadata.**
  `presentation_payloads_retain_their_exact_facts` now asserts `entity_subtype` for every
  host use (DESCRIBE `Some("entity")`, COMPARE/EXPORT/SCOPE `None`) and pins the SAMPLE
  `query_text` (`"auth"`) and `mode` (`Lexical`) for the unscoped and scoped fixtures.
  Before, the subtype and those metadata fields were discarded by `..`.
- **Absolute semantic-step charge.**
  `implicit_group_semantic_steps_fail_at_the_exact_limit_and_one_over` still discovers the
  boundary, but now pins the discovered charge to a constant
  (`SCALAR_COUNT_STAR_STEPS = 6`). The discovery-only form was self-calibrating: a `+1`
  charge-model drift shifted the discovered limit while the limit/one-over symmetry still
  passed. The `deep_*` bound tests tolerate `+1`/`+2`, so this pin is the rail that catches
  the drift.
- **Over-budget receipt.** `rule_program_receipt_is_exact_at_the_sealed_limit_and_one_over`
  now asserts the one-over document's `max_semantic_steps` as well; previously only the
  exact document's receipt was checked although the test name claims "at the sealed limit
  and one over".

Validation: each new assertion was shown load-bearing with a temporary production mutation
that was then reverted — DESCRIBE use span `17..24 -> 17..25`, EXPLAIN target span
`8..15 -> 8..16`, and a duplicated implicit-Group charge (`exact` `6 -> 7`); each made its
targeted test fail. `cargo test -p quanta-gqlang-v2-sema` passed 147 of 147, the logic
wrapper caller test passed, and `cargo clippy -p quanta-gqlang-v2-sema --tests` was clean.
