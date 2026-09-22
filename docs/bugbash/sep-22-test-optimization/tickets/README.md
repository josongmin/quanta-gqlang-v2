# Sep 22 Test Optimization — Execution Tickets

Status: historical lanes in `receipts/00.md-08-leftover-resolution.md`;
clean current-source review fix and local gate results in
[`receipts/09-review-remediation.md`](../receipts/09-review-remediation.md).

Planning observation: `main@3a9ab271d7706c1c9266551df492f64f4bf68eea`.

This packet closes the eleven findings in `open-items.md` (deleted after W4
verified all eleven against current `main`; see `receipts/07-08` and the W4
close commit). The observation below is a frozen planning snapshot, not an
implementation base: the shared worktree contains unrelated frontend-contract,
pipeline, and sema changes. Ticket 00 must mint a clean current-source base before any RED or
GREEN result is accepted.

The 12-file lane patch and the later 46-path `d353c45a` mixed commit have
different scope. Receipt 09 records that distinction and the later clean
review-fix SHA. The lease table below describes the original lane plan.

## Objective

- Repair the two production defects at their parser owners: `REF-01` and `STAGE-01`.
- Replace weak or unreachable tests with payload-, diagnostic-, and branch-sensitive oracles.
- Make the local test entry point include the shipped front-door parity gate.
- Delete only the two executions proven redundant by stronger exact-message rails.
- Qualify all changes on one immutable source SHA; do not reuse the historical `b6d52b9c...`
  baseline as closure evidence.

## Canonical Order

```text
00 source freeze / admission
  -> { 01 surface || 02 planning || 03 CLI || 04 test authority }
  -> 05 serial integration / qualification
```

Tickets 01–04 may execute in parallel only after Ticket 00 records disjoint path leases. Ticket
05 is serial. A lane that needs a path owned by another lane stops; it does not widen its write
set locally.

## Finding Ownership

| Finding | Ticket | Canonical owner | Closure type |
|---|---|---|---|
| `REF-01` | [01](./01-surface-grammar-and-oracles.md) | Surface reference parser | production repair + exact rails |
| `DOC-01` | [01](./01-surface-grammar-and-oracles.md) | Surface document tests | exact span oracle |
| `DOC-02` | [01](./01-surface-grammar-and-oracles.md) | Surface document tests | branch-specific diagnostic oracle |
| `VIEW-01` | [01](./01-surface-grammar-and-oracles.md) | Surface family tests | retained HIR payload oracle |
| `PRODUCER-01` | [01](./01-surface-grammar-and-oracles.md) | Surface composition tests | reachable producer-branch oracle |
| `STAGE-01` | [01](./01-surface-grammar-and-oracles.md) | Surface relation-stage parser | production authority repair + exact rail |
| `DEDUPE-01` | [01](./01-surface-grammar-and-oracles.md) | Surface family tests | measured deletion after stronger rails |
| `EXPLAIN-01` | [02](./02-planning-provenance-order.md) | Planning provenance traversal | asymmetric semantic oracle |
| `CLI-01` | [03](./03-cli-command-and-input-contract.md) | CLI command registry/dispatcher | single command authority + binary rails |
| `CLI-02` | [03](./03-cli-command-and-input-contract.md) | CLI ingress | inline/file boundary rails |
| `TEST-01` | [04](./04-test-entrypoint-authority.md) | Just test composition | one local composition path |

Each finding appears exactly once. No ticket revives the deleted TSV scanner or treats body-hash
similarity as duplicate proof.

## Path Leases

| Lane | Writable paths | Frozen/read-only dependencies |
|---|---|---|
| 01 Surface | `crates/quanta-gqlang-v2-surface/src/family.rs`, `src/query.rs`, `tests/v3_document_hellgate.rs`, `tests/v3_family_hellgate.rs`, `tests/v3_family_edges.rs`, `tests/v3_surface_query.rs` | HIR/public facade outside Surface |
| 02 Planning | `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs`, `crates/quanta-gqlang-v2-sema/src/test_support.rs` | Sema production, Algebra implementation, and planner traversal source; `planning/src/plan.rs` is mutation-only and must return to its frozen hash |
| 03 CLI | `crates/quanta-gqlang-v2-cli/src/lib.rs`, `src/main.rs`, `tests/command_doc_gate.rs`, `tests/v3_cli_fmt_idempotence.rs`, `tests/open_analysis_contract.rs` | frontend-contract and command implementation crates |
| 04 Test authority | `Justfile` | workflow and test implementation files |
| 05 Integrator | this packet, receipts, and any explicitly approved cross-lane reconciliation | all product paths while qualifying |

`.github/workflows/lint.yml` is read-only for this packet: hosted CI already runs all three test
layers directly. Ticket 04 changes local composition only and verifies CI parity structurally.

## State Model

```text
PLANNED -> ADMITTED -> RED_CAPTURED -> IMPLEMENTED -> FOCUSED_GREEN
        -> INTEGRATED -> LOCAL_QUALIFIED
```

- `FOCUSED_GREEN` belongs to one ticket and one source digest.
- `INTEGRATED` requires all lane commits on one candidate.
- Only Ticket 05 may declare `LOCAL_QUALIFIED`.
- Hosted CI, review, merge, deploy, and activation are separate states and are not claimed here.

## Required Worker Contract

Before editing, record:

```text
ticket=
base_sha=
dirty_fingerprint=
allowed_write_paths=
frozen_input_hashes=
red_command_and_observed_result=
expected_positive_negative_oracles=
mutation_to_kill=
acceptance_commands=
head_sha=
```

For test-only gaps, RED means the deliberate production mutation survives or the named branch is
not reached—not merely that the current test happens to pass. Every mutation is reverted before
GREEN. A selector that runs zero tests is a failed gate.

## Global Stop Conditions

Stop and reopen the ticket when:

1. the frozen base or any leased input changes after RED capture;
2. another active change overlaps an allowed write path;
3. a proposed fix creates a second grammar, command, or test authority;
4. a test can pass by checking only `Ok`, an enum discriminant, a substring, or process exit;
5. runtime/provider/SDK behavior would enter this frontend-only repository;
6. focused commands cannot compile because unrelated shared work is still incomplete;
7. a qualification command selects zero tests or the candidate tree is dirty.

## Receipts

Ticket 05 creates `receipts/00.md` through `receipts/05.md` from lane handoffs. Each receipt must
record base/head SHA, owned paths, pre/post hashes, RED evidence, command/exit/test count, killed
mutation, and remaining risk. Prose such as “tests pass” is insufficient.
