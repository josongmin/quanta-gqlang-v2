# 09 — Conformance, Truth Sync, And Closeout

Status: `PLANNED — ONLY TICKET AUTHORIZED TO CLOSE PACKET`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

F03 final closure, F14, F29, and proof coverage for every other finding.

## Goal

Replace routing-only confidence with semantic conformance, synchronize every
public claim to current behavior, and record an exact-head closure receipt.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | current artifact-hygiene checks, registry inventory concepts, useful family source fixtures | retain only assertions that still target the curated facade |
| `MIGRATE` | three conformance tests and pipeline `audit_18_families.rs` | build one semantic registry with declared/selected/executed equality and payload oracles |
| `DELETE` | routing-only end-to-end labels, stale path/method/example ledgers, unsupported green/LSP claims | delete or mark historical with exact scope |
| `NEW` | semantic family matrix, public API inventory, traceability, immutable-head closeout receipt | these aggregate earlier owner proofs; they do not reimplement semantics |

Before adding a conformance row, point to the owner test it composes. If a new
row merely repeats an owner assertion without crossing a public boundary, keep
it in the owner suite instead. The existing 18-family file is either upgraded
and renamed or retained explicitly as `routing_smoke`; it cannot coexist under
an end-to-end/conformance claim.

## Write Set

1. `crates/quanta-gqlang-v2-conformance/src/lib.rs`
2. the exact conformance tests listed below
3. `/README.md`
4. `/docs/language/README.md`
5. `/docs/language/release_discipline.md`
6. `/docs/language/examples_index.md`
7. `/docs/language/test_hellgate_ledger.md`
8. `/docs/language/ai-profile.md` only if the generated claim inventory selects
   a stale public claim in it
9. `/examples/` only for a source file proven stale by the example matrix;
   otherwise examples are read-only fixtures and only the index changes
10. `/docs/plans/jun-24-dsl-advanced-v3/CURRENT_HEAD_SUPERSESSION.md` — new
11. `/docs/plans/jun-24-dsl-advanced-v3/README.md`,
    `/docs/plans/jun-24-dsl-advanced-v3/SERVING_HANDOFF.md`, and
    `/docs/plans/jun-24-dsl-advanced-v3/tickets/README.md` — top-banner link to
    the supersession only; do not rewrite historical receipts
12. this packet's `README.md`, `TRACEABILITY.md`, `CLOSEOUT_RECEIPT.md`, and
    `TRUTH_CLAIM_INVENTORY.tsv`
13. `/tools/architecture/check_architecture.py` to delete the empty transition
   allowlist and make final mode the default
14. `/.github/workflows/lint.yml` and `/Justfile` to invoke final architecture
   mode
15. other CI/proof files from ticket 01 only when final integration reveals a
    demonstrated defect
16. `/.gitignore` and `/test_repro_cases.txt` — delete tracked repro debris and
    close the hygiene blind spot
17. `crates/quanta-gqlang-v2-pipeline/tests/audit_edge_cases.rs`
18. `crates/quanta-gqlang-v2-cli/tests/examples_release_smoke.rs`
19. `crates/quanta-gqlang-v2-conformance/tests/v3_example_corpus.rs` and its
    owner-local fixture manifest
20. existing integration-test crate headers required to normalize inherited
    reasoned `#![allow]` attributes without changing test behavior
21. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/method_allowlist_doc_gate.rs`
    to move path/protocol truth from the deleted ledger to the final proof-rail
    inventory
22. `crates/quanta-gqlang-v2-frontend-contract/tests/external_consumer/` and the
    facade export inventory introduced by ticket 07
23. `tools/test_frontdoor_parity.py` introduced by Ticket 08 and its `just ci`
    invocation; no conformance→binary Cargo edge is admitted

Ticket 09 must remove the temporary ticket-01 Clippy exceptions for
`clippy::allow_attributes` and `clippy::duplicated_attributes`. Final CI and the
local recipe both run exactly
`cargo clippy --workspace --all-targets -- -D warnings`; no target omission or
lint exception survives closeout.

## Conformance Matrix

For every public family, test:

1. closed-authority positive
2. missing authority
3. unknown reference
4. wrong-kind reference
5. ambiguous reference
6. invalid option combination
7. retained semantic contract fields
8. CLI/service parity
9. deterministic diagnostic code and exact spans
10. reference to the owning resource-boundary receipt or one selected
    cross-front-door projection case; do not repeat every limit for this family
11. one relevant mutation case

The existing 18-family suite is renamed routing smoke unless it is upgraded to
this matrix. Variant/result-kind/effect-only checks are insufficient.

The registry has two disjoint manifests, not one 18-row manifest:
`ROOT_FAMILIES_V1` contains 18 public root families including Composition;
`SUPPLEMENTAL_SURFACE_UNITS_V1` contains standalone Decl and HornRule.
Preserve both sets and assert declared = selected = executed separately for
18 roots and 2 supplemental units. Decl must prove its typed declaration
payload and checked no-plan reason; HornRule must prove its exact
`RuleProgramMemberV2`/document-program outcome (and no copied per-rule plan).
Do not fold the two into the root count or delete their `unit` specs when
moving `self_validation.rs`.

Ticket 08 and owner receipts already prove exact-limit/+1 behavior for every
budget dimension. Ticket 09 records a one-to-one dimension→owner receipt map
and executes only representative cross-boundary projection/parity rows (CLI,
stdio, closed facade), with declared/selected/executed non-vacuity. A
per-family resource pointer is required for traceability; 18 copies of the
same limit test are not.

## Required Cross-Stage Scenarios

1. normal JOIN and explicit CROSS
2. invalid predicate/operator matrix
3. aggregate context and group partition
4. inline VIEW/direct query parity
5. catalog forward/wrong-kind/ambiguous refs
6. REQUIRE CERTIFIED order permutation
7. module/import permutation
8. source self/mutual/negative/aggregate recursion
9. FIND exact witness retention and false-waypoint-bound rejection
10. CHECK/EXPLAIN/SAMPLE/COMPARE/EXPORT option matrices
11. nondeterministic builtin plan effect
12. Unicode/escape/formatter round trips
13. malformed framing recovery
14. every resource-budget dimension accounted for by one owner receipt,
    with representative cross-front-door projection cases

## Documentation Truth

1. Generate or verify method allowlists from code.
2. Generate or verify example inventory bidirectionally.
3. Verify every final proof-rail inventory path exists.
4. Separate historical receipts from current-head receipts.
5. Remove all `closed`, `implemented`, or `green` claims lacking the new exact
   receipt.
6. Describe semantic plans as frontend policy, not physical execution.
7. Describe analysis/open-authority results separately from compilation.

## Step-By-Step Procedure

1. Freeze candidate `HEAD_SHA`, dirty fingerprint, workspace member list, and
   dependency edges before any closeout command.
2. Read receipts for tickets 01-08 and reject any receipt missing RED, mutation,
   or cross-stage proof.
3. Build a machine-readable F01-F37 traceability table with owner commit, test,
   and final status.
4. Rename existing routing-only conformance tests or upgrade them; do not leave
   an end-to-end claim attached to discriminant-only assertions.
5. Add the semantic family matrix and execute every declared row.
6. Add a registry non-vacuity assertion: declared cases equal selected cases
   equal executed cases.
7. Run mutation cases or equivalent controlled reversions for each critical
   owner invariant and record that the intended test fails.
8. Regenerate/validate method allowlist, example inventory, final proof-rail paths,
   and public API inventory from current source.
9. Generate `TRUTH_CLAIM_INVENTORY.tsv` with path, line, exact claim, scope
   (`historical` or `current-head`), supporting receipt, and action. Add one
   Jun-24 supersession document and link it from the three entry banners above;
   do not rewrite historical command receipts as if they never occurred.
10. Prove the transitional dependency count is zero, delete the exception set,
    and change local/CI architecture commands to final mode.
11. Run focused owner suites, then the full final command list on the same
    unchanged SHA.
12. Recheck `git status`, `git diff --check`, SHA, and test inventory after the
    last command.
13. Query remote Actions only if credentials/network are available; otherwise
    record remote verification as unavailable rather than inferred.
14. Change packet status to complete only after every stop condition is false.

## Files To Add Or Move

Upgrade or explicitly reclassify the existing
`current_head_artifact_hygiene.rs`, `registry_non_vacuous.rs`,
`self_validation.rs`, and pipeline `audit_18_families.rs` before creating these
files. One source fixture belongs to one registry row; do not copy the 18-family
examples into a second registry.

1. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` to `crates/quanta-gqlang-v2-conformance/tests/v3_semantic_family_matrix.rs`
2. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-conformance/tests/registry_non_vacuous.rs` to `crates/quanta-gqlang-v2-conformance/tests/v3_registry_non_vacuity_v2.rs`
3. `[ADD]` `crates/quanta-gqlang-v2-conformance/tests/v3_public_api_inventory.rs`
4. `[KEEP+UPDATE]` `crates/quanta-gqlang-v2-conformance/tests/current_head_artifact_hygiene.rs`
5. `[RENAME OR RECLASSIFY]` `crates/quanta-gqlang-v2-pipeline/tests/audit_18_families.rs`; it must not retain a conformance/end-to-end name if routing-only
6. `[ADD]` `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/TRACEABILITY.md`
7. `[ADD]` `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/CLOSEOUT_RECEIPT.md`
8. `[ADD]` `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/TRUTH_CLAIM_INVENTORY.tsv`
9. `[ADD]` `docs/plans/jun-24-dsl-advanced-v3/CURRENT_HEAD_SUPERSESSION.md`
10. `[ADD]` `crates/quanta-gqlang-v2-conformance/tests/v3_example_corpus.rs`
11. `[ADD]` `crates/quanta-gqlang-v2-conformance/tests/fixtures/examples-v2.tsv`
12. `[ADD]` `crates/quanta-gqlang-v2-cli/tests/examples_release_smoke.rs`
13. `[ADD]` `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/TEST_PATH_INVENTORY.tsv`

## Final Test-Suite Disposition

1. Delete `docs/language/test_hellgate_ledger.md`; do not regenerate a second
   proof-path inventory. `TEST_PATH_INVENTORY.tsv` is the sole machine-readable
   authority. Root/language README proof-rail sections link to or are generated
   from it; conformance and hygiene read only the TSV and fail on missing paths.
2. `examples-v2.tsv` declares exactly one row for every `examples/**/*.qglang`
   path with family, authority fixture, expected compiled kind or expected
   diagnostic code, and open-analysis expectation. Tests prove filesystem ↔
   manifest ↔ root README ↔ example index equality, parse every file, check
   format idempotence, execute the declared closed compile/analyze outcome, and
   run the CLI query/fmt smoke. The composition example is included explicitly.
3. Delete tracked `test_repro_cases.txt`, add `/test_*.txt` to `.gitignore`, and
   extend `current_head_artifact_hygiene.rs` to reject root `test_*.txt` files.
4. Reduce `audit_18_families.rs` to an explicitly named routing smoke retaining
   exactly the six mandatory rails: flow, slice, points-to, summary, and both
   CHECK-with-FIND cases. Move f01/f11/f13/f15 variant/value assertions to
   ticket 07 owner tests before deleting them; delete the remaining registry
   restatements.
5. Reduce `audit_edge_cases.rs` to one representative pipeline smoke and
   strengthen it to assert stage, code, and exact span. Surface retains the
   clause-specific message/span matrix.
6. Keep the first two registry non-vacuity tests; delete only the compile-time
   uniqueness restatement and unconsumed keyword table. `SupplementalSurfaceUnitSpecV1.unit`
   must be asserted by the separate two-unit semantic manifest and never
   deleted with its Decl/HornRule source coverage.
7. Repoint the redundant conformance USE PACK row to its stronger pipeline
   owner instead of executing the same source twice. Delete the duplicated
   REQUIRE CERTIFIED rows after the semantic matrix owns result/effect.
8. Run a final naming/comment scan: no stale `lsp`, deleted
   `syntax->sema->ir`, pre-cutover, wrong line-pointer, `_punchlist`, or
   undocumented audit/mutant prefix claim remains in current tests.
9. Record every retained, migrated, deleted, strengthened, and intentionally
   duplicated test function by ticket. Validate aggregate counts in the
   closeout receipt; support modules are not counted as test targets.
10. Generate `TEST_PATH_INVENTORY.tsv` with path, owner ticket, disposition,
    base existence, final existence, and test-target boolean. Its row set must
    equal the unique test/support paths referenced by tickets plus every actual
    added/deleted test path in `git diff --name-status`.

## Traceability Row Schema

```text
finding_id
severity
canonical_owner
repair_commit
red_test
green_test
mutation_test
public_frontdoor_test
status
remaining_boundary
```

Every F01-F37 row is required. `not applicable` must include a reason and the
scope-deletion/documentation change that made it inapplicable.

## Semantic Family Row Schema

```text
family
source
authority_fixture
expected_compiled_kind
expected_semantic_contract
applicability
not_applicable_reason
closed_negative_case
open_diagnostic_case
cli_service_parity
resource_owner_receipt_or_cross_boundary_case
owner_mutation_receipt
```

The registry test fails if any field is absent. The resource field points to
an owner receipt or a selected cross-boundary case; the separate dimension
manifest proves full budget coverage without repeating identical limit cases
for every family. `not_applicable_reason` must be
non-empty exactly when `applicability=not_applicable`; closed and open cases are
separate and are not required to produce the same successful artifact.

## RED

On the audited base:

1. the 18-family suite proves routing labels, not semantic payloads
2. the example index omits a tracked example and does not classify expected
   authority failures
3. the proof ledger contains nonexistent source paths
4. root/language docs claim unsupported LSP behavior
5. Jun-24 status text conflicts with current source and local red gates
6. architecture final mode cannot pass while transitional edges remain

Capture these as inventory/validation failures before editing truth documents.
Do not use a prose-only before/after comparison as the RED receipt.

## Tests

1. semantic family matrix with positive, negative, option, payload, resource,
   and mutation columns
2. declared/selected/executed registry equality
3. public facade inventory against source exports
4. bidirectional example filesystem/index equality
5. final proof-rail inventory path existence and line-anchor validation
6. current command/method allowlist derived from code
7. final-DAG mode with zero transitional edges
8. documentation status scan with explicit historical-scope allowlist

## Final Commands

```bash
git diff --check
cargo fmt --all --check
taplo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace --no-tests pass
cargo test --workspace --doc
python3 tools/architecture/check_architecture.py --require-final
cargo deny check bans licenses sources advisories
typos .
tools/lint/run_semgrep.sh
python3 tools/test_frontdoor_parity.py
just ci
```

Also run the example formatter/idempotence matrix and the semantic family
conformance matrix explicitly.

## Exact Additional Commands

These selectors and file names are pinned. If a prior ticket must rename one,
stop and amend this packet before implementation:

```bash
cargo test -p quanta-gqlang-v2-conformance --test v3_semantic_family_matrix
cargo test -p quanta-gqlang-v2-conformance --test v3_registry_non_vacuity_v2
cargo test -p quanta-gqlang-v2-conformance --test v3_public_api_inventory
python3 tools/test_frontdoor_parity.py
find examples -type f -name '*.qglang' -print | sort
rg -n 'Status:.*(CLOSED|Implemented|green)|repo-local proof is green' README.md docs
git status --short
git rev-parse HEAD
```

The status grep is an audit input, not an automatic failure: every match must be
historical and explicitly scoped, or updated to the fresh receipt.

## Forbidden Shortcuts

1. closing from a focused test subset
2. counting declared-but-unselected cases
3. updating docs before final behavior is frozen
4. using stale remote refs as Actions evidence
5. treating static, mutation, focused, full-workspace, remote CI, and downstream
   activation evidence as interchangeable
6. leaving the worktree dirty without per-path ownership classification

## Closeout Receipt

Record:

1. exact base/head SHA
2. clean/dirty fingerprint before and after
3. workspace member and dependency-edge inventory
4. test selected/executed counts
5. every command exit code
6. Semgrep declared/executed rule counts and target counts
7. example expected/executed counts
8. remote Actions URL/status if independently verified
9. any downstream activation or review gate still open

## Stop Conditions

Do not close when:

1. any old compile seam remains public
2. any compiled success contains unresolved/Unknown values
3. planning still accepts raw HIR
4. a static rule has zero targets
5. conformance checks only routing labels
6. docs claim actual LSP while flat JSON is shipped
7. the worktree contains unrelated or unexplained changes
8. remote state is inferred from cached refs
9. any transitional normal or dev dependency edge remains
10. tickets 01-08 are not contained in one clean committed candidate SHA
11. hosted CI was queried for that SHA and is `failed`

## Definition Of Done

1. every finding F01-F37 is mapped to a landed repair or explicit deleted scope
2. every ticket receipt is present and source-current
3. the final commands are green on one immutable head
4. public docs match that head
5. no runtime/provider/physical-planning ownership leaked into this repo
6. closeout state is `CLOSED_BY_09` only with hosted green, otherwise
   `REPO_LOCAL_CLOSED` with hosted status `unavailable`

## Dependencies

## 2026-09-21 Final Adversarial Audit Corrections

This section is normative and supersedes conflicting earlier wording.

### Truth-source disposition

Delete `docs/language/test_hellgate_ledger.md` only after its live rows are
migrated to the canonical `TEST_PATH_INVENTORY.tsv`. README proof rails link to
or are generated from that inventory. This ticket owns the corresponding rewrite of
`method_allowlist_doc_gate.rs`. The final path-existence check reads the new
authority; it must not reference the deleted ledger.

The semantic matrix has two explicit sections:

1. closed compile: valid, missing, wrong-kind, ambiguous, option, and resource
   cases only where the family owns that dimension;
2. open CLI/service diagnostic parity: unresolved cases and byte/framing
   failures under standard open authority.

Every row carries `applicability: required | not_applicable` and a non-empty
`not_applicable_reason` when skipped. Do not force meaningless variants onto
every family. Owner mutation receipts are referenced once; the final tree runs
representative cross-boundary mutations rather than copied per-family tests.

### Immutable closeout precondition

Packet closure requires a clean committed candidate SHA containing tickets
01-08. The ledger's no-commit policy means the user must authorize that commit;
without authorization ticket 09 stops before closure. If the work remains dirty
or the candidate commit lacks any receipt, a dirty fingerprint is evidence, not
`repair_commit` or closure SHA.

Hosted CI is recorded separately as `green`, `failed`, or `unavailable` after
the immutable local SHA passes. A verified `failed` blocks closure;
`unavailable` permits only `REPO_LOCAL_CLOSED` and remains an explicit external
evidence gap. Local output never fabricates hosted green.

Final facade proof includes the curated `pub use` inventory and the external
fixture that depends only on `frontend-contract`. Confirm zero `_punchlist`
filenames, zero deleted-ledger references, zero old HIR API references, and zero
transition-only dependency exceptions before closeout.

Requires 01-08. Runs serially after all implementation merges.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954`)

This ticket has no implementation or closure receipt. The inherited
`self_validation.rs` drives V1 `compile_program_structural` and checks mostly
family kind/result/effect; it is routing evidence, not the semantic matrix.
Keep one representative routing smoke and migrate existing source fixtures
into the applicability-aware registry. Do not clone all 18 cases into another
test file. A conformance row is executable only if its fixture, authority,
expected semantic payload, owner mutation proof, and selected/executed count
are machine checked; a table in Markdown alone does not count.

Before claiming final status, reconcile every actual test path in the landed
diff against `TEST_PATH_INVENTORY.tsv`, every F01-F37 against `TRACEABILITY.md`,
and every current public method/example/proof claim against its generated
inventory. Then freeze one *clean committed* candidate containing 01-08 and
run the full command list without changing that SHA. The existing dirty
`f9ec954` overlay and prior 822-test base result are not that candidate.
Record hosted CI as independently verified green/failed/unavailable; do not
promote a local pass or a cached remote ref. The no-commit policy leaves this
ticket open until the user authorizes the candidate commit.

## Final Cross-Owner Semantic Fixtures

The 18-root + two-supplemental manifest must include these boundary cases
without cloning the underlying owner suites:

1. Query: same host source self-Join has two distinct scan occurrences;
   Group publishes only ordered keys, aggregate projection can use a sealed
   partition input, ordinary post-group reference cannot; ASC and DESC both
   retain NULLS LAST in the semantic payload and plan/explain output.
2. HornRule supplemental: source-local self/mutual dependency edges reach the
   one document program; builtin and host Predicate calls are typed but do
   not become local SCC edges. Same-spelling source/builtin/host collisions
   reject at the authored call location.
3. Composition: sequential LET outputs are consumed by exact Binder local
   identity. A Sample producer exposes only `path: RepoPath`, PATH or
   REACHABILITY FIND only `function: EntityRef`, and other FIND kinds no
   relation row. Wrong field, wrong kind, and host-fallback shadowing reject;
   `WITH` selects the sealed body output without an independent LET plan.
   The old schema-open empty-projection success must be a closed V2 reject.

Each fixture must assert the exact semantic payload, diagnostic code/span or
typed result, and CLI/stdio parity where applicable. Running a route-only
V1 family label test does not satisfy these rows.
