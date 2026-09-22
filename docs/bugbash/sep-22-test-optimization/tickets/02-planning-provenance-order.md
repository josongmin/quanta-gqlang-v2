# 02 — Planning Provenance Order Oracle

Status: `FOCUSED_GREEN (see receipts/02.md)`

Finding owned: `EXPLAIN-01`.

## Goal

Prove the public explain provenance contract is root-left-right preorder. Existing join fixtures
have identical left/right family shapes, so swapping child traversal produces the same vector.

## Write Set

- `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs`
- `crates/quanta-gqlang-v2-sema/src/test_support.rs`

Production traversal in `planning/src/plan.rs` is frozen except for the isolated deliberate
mutation below. It must have the same hash before and after mutation and must not enter the lane
commit. If the unmutated traversal is wrong, reopen this ticket with that production path; do not
hide it by changing expected order.

## Required Fixture

Add one explicit `ReferenceRelationFixtureV2` member in the existing sealed Sema test-support
matrix, named for asymmetric join preorder, whose children are structurally distinguishable:

- left: `Project -> Filter -> Scan` or another existing three-family chain;
- right: `Sort -> Scan` or a different two-family chain;
- root: `Join`.

Construct it through the same checked semantic document path as the other reference relation
fixtures, then feed it through `support::relation_input`, `lower_relation_item`, and the public
`plan` entry point. Add it to the fixture owner’s inventory/exhaustiveness rail. Do not forge a
planner result, expose a caller-built Algebra node, or hand-write the expected provenance list as
the actual result.

## Oracle

Assert both:

1. exact node count;
2. exact ordered family vector:
   `Join`, every left-subtree family in preorder, then every right-subtree family in preorder.

Also assert the result came from the normal planner/explain entry point used by the neighboring
tests. An internal helper called directly is not sufficient.

## RED And Mutation Proof

1. On the frozen base, show the current symmetric self-join test remains green when child traversal
   in the test model is conceptually swapped; record why it is non-discriminating.
2. Add the asymmetric fixture and oracle.
3. Deliberately swap left/right recursive calls in the local traversal source, run only the new
   test, and record its exact failure vector.
4. Revert the mutation and verify the source hash returns to the intended implementation.

Never leave the mutation in a shared worktree.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-planning --test v3_explain_provenance
cargo test -p quanta-gqlang-v2-planning --all-targets
cargo clippy -p quanta-gqlang-v2-planning --all-targets -- -D warnings
cargo test -p quanta-gqlang-v2-sema --lib test_support
cargo clippy -p quanta-gqlang-v2-sema --all-targets -- -D warnings
```

The `test_support` module is compiled for Sema unit tests and remains available
to downstream fixtures through the `test` feature. Use an exact selector for
`test_support::tests::every_positive_fixture_variant_resolves` when checking
the fixture inventory alone; a zero-test result is a failed gate.

## Definition Of Done

- One asymmetric reachable fixture distinguishes left from right.
- Exact family order and node count fail under child-order reversal.
- Existing symmetric tests remain as coverage for their separate shapes, not as order proof.
- The fixture remains under the existing `test_support` authority; no new production or
  runtime-facing contract is introduced.
