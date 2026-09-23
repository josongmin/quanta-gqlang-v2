# ADR-0002: Test authority and adversarial oracle design

- Status: Accepted
- Date: 2026-09-24
- Scope: frontend owner tests, process gates, and local qualification

## Context

The Sep-22 test-optimization bugbash produced durable implementation and test
design decisions, but its 17 ticket and receipt files retained source-specific
hashes, temporary path leases, intermediate blockers, and obsolete test counts.
Those records were useful for execution custody and are recoverable from Git
through the parent of the deletion commit. They are not current test authority.

## Decision

Delete `docs/bugbash/` and preserve the resulting rules in this ADR and the
executable owners named below. Current coverage mapping remains in the
[test hellgate ledger](../language/test_hellgate_ledger.md); exact committed
source qualification follows the
[local qualification procedure](../../tools/qualification/README.md).

### Grammar authorities are singular and closed

- Multi-character grammar tokens must be consumed atomically. A partial token
  cannot be accepted as a valid neighboring construct.
- Relation-stage keywords and dispatch share one closed table authority in
  `surface/src/query.rs`; tests compare the supported set and exact rejection
  behavior rather than maintaining a second registry.
- Composition producer tests must reach the producer parser itself and retain a
  valid positive neighbor. Testing only an outer composition discriminant is
  insufficient.

### Semantic tests assert payload, location, and branch identity

- Document diagnostics assert exact code, message, and source span where those
  fields are contractual.
- Family and composition tests inspect retained HIR payload and binding/body
  identity, not only `Ok`, an enum discriminant, or a substring.
- Branch tables use inputs that distinguish each failure path. A test fixture
  that can satisfy multiple branches does not prove branch selection.

### Ordering requires an asymmetric oracle

Traversal-order proof must use structurally different children and assert the
exact ordered result. A symmetric self-join fixture remains useful for shape
coverage but cannot prove preorder direction. The current owner fixture is
`ReferenceRelationFixtureV2::AsymmetricJoinPreorder`, consumed by Planning's
provenance tests.

### CLI commands have one registry

`CLI_COMMAND_SPECS_V1` owns shipped command tokens, usage text, and dispatch
identity. Documentation/help gates compare the exact command set, and process
tests invoke every shipped command. Handwritten parallel token lists or
substring membership checks are forbidden.

### CLI ingress tests own both negative and positive boundaries

Inline empty input, zero-byte files, and whitespace-only files must fail with
the expected process status and canonical diagnostics. Positive neighbors must
check actual semantic status, stdout, and stderr. Temporary files are owned by
an RAII helper so failed assertions do not leak fixtures.

### Local test composition has one entry path

The `Justfile` `test` recipe composes Nextest, doctests, and black-box
front-door parity exactly once. Repository-wide `just ci` composes the current
static, test, qualification-custody, architecture, external-consumer, deny, and
Semgrep rails. A new gate must be attached to the owning composition instead of
being duplicated across unrelated recipes.

### Tests must prove reachability and mutation sensitivity

- A selector that executes zero tests is a failed gate.
- A test-only gap requires evidence that the named branch is reached, normally
  through an exact negative and a valid positive neighbor.
- Temporary production mutations may demonstrate sensitivity only in an
  isolated, source-bound workspace and must be reverted before GREEN.
- Historical mutation outcomes are never relabeled as current proof.

### Duplicate tests require semantic proof before deletion

Body hashes, similar names, or overlapping setup do not establish duplication.
Delete a test only when a stronger executable owner asserts the same behavior
and exact failure boundary. Keep distinct payload, ordering, process, and
cross-owner assertions even when they share fixtures.

### Total access and exhaustive contracts are preferred

Production code avoids unchecked indexing when a total slice pattern,
destructuring, `get`, or checked conversion expresses the invariant. Closed
enum tests use exhaustive matching so a new variant breaks the test at compile
time. Shared deterministic formatting helpers, such as lower-hex digest
encoding, replace repeated partial formatting loops.

### Test-support APIs must work in every promised build mode

A test-support fixture exposed to downstream test features must also compile
and execute in its owner unit-test configuration. Feature unification in a
workspace-wide run does not prove a standalone selector. Each advertised
focused command must select and execute at least one intended test.

## Current executable owners

- Surface grammar and exact diagnostics:
  `crates/quanta-gqlang-v2-surface/src/query.rs`,
  `crates/quanta-gqlang-v2-surface/src/family.rs`, and Surface owner tests
- Provenance ordering:
  `crates/quanta-gqlang-v2-sema/src/test_support.rs` and
  `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs`
- CLI registry and process ingress:
  `crates/quanta-gqlang-v2-cli/src/lib.rs`, `src/main.rs`, and CLI process tests
- Local gate composition: `Justfile`
- Repository-wide coverage routing:
  `docs/language/test_hellgate_ledger.md`
- Exact committed-source qualification: `tools/qualification/qualify-local.sh`

## Evidence boundary at consolidation

Clean committed source `0fd549c97e148e1048c76fc0d4c03b029063053b`, tree
`80b791419982cb65c7902a0ac8efe40b7b76dcae`, passed
`just qualify-local` with `just ci`, exit `0`, in 56 seconds. The run selected
and passed 1,176/1,176 Nextest cases, 5/5 doctests, front-door parity 8/8,
qualification custody 18/18, external facade positive 1/1, privacy negatives
15/15, and public API 89/89, plus architecture, both strict Clippy profiles,
deny, and non-vacuous Semgrep.

This proves the repository-local frontend at that exact source. It does not
prove hosted CI, downstream runtime admission, deployment, or activation.

## Consequences

- Test intent is recoverable from executable owners and one current ledger,
  rather than a source-bound bugbash packet.
- Future review findings must name the current owner, reproducer, negative and
  positive oracle, and cheapest trustworthy acceptance command.
- Intermediate lane receipts remain available in Git history but cannot be
  cited as current-source qualification.
- Adding test count alone is not progress; stronger independent behavior
  discrimination is the acceptance criterion.
