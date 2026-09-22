# 09 — Conformance, Truth Sync, And Closeout

Status: `PLANNED — CLOSEOUT BLOCKED; ONLY TICKET AUTHORIZED TO CLOSE PACKET`

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
3. `[IMPLEMENTED IN DIRTY OVERLAY — UNQUALIFIED]` tracked
   `test_repro_cases.txt` is deleted, `/test_*.txt` is ignored, and root
   hygiene coverage exists. Do not repeat these edits. Qualify and extend the
   implementation with the recursive tracked-path work in the final RR section.
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
    added/deleted/renamed test path in
    `git diff --name-status --find-renames "$PACKET_BASE_SHA".."$HEAD_SHA"`.
    Plain `git diff` is forbidden on the clean candidate because it is empty.

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
python3 tools/test_frontend_external_consumer.py
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
python3 tools/test_frontend_external_consumer.py
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
10. tickets 01-08 receipts plus Ticket 09's pre-attestation implementation are
    not contained in one clean committed candidate SHA
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
01-08 plus Ticket 09's conformance/CI implementation and unclaimed receipt
template. The ledger's no-commit policy means the user must authorize that
commit;
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

This ticket has no closure receipt. The dirty overlay already contains the
root repro-file deletion/ignore/hygiene edits and Composition README/example
index entries; classify them as `IMPLEMENTED IN DIRTY OVERLAY — UNQUALIFIED`
and do not schedule duplicate edits. The inherited
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
inventory. Then freeze one *clean committed* candidate containing Tickets
01-08 plus Ticket 09's pre-attestation implementation, and run the full command
list without changing that SHA. The existing dirty
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

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: `PLANNED — NO CLOSEOUT RECEIPT`. Receipts 01–04 exist; 05–08 are
absent. The tree is dirty, Algebra and Pipeline are red, transition edges remain,
and hosted CI is unverified. Ticket 09 must not paper over any of those facts.

Exact closeout procedure:

1. Require completed receipts 01–08 and verify every receipt fingerprint
   against the candidate content. Historical owner-green hashes remain valid
   only for the exact files they name.
2. Build the semantic manifest as 18 root-family rows plus separate Decl and
   HornRule supplemental rows. Each row has fixture, applicability, authority,
   expected semantic payload or exact diagnostic/span, owner receipt, and
   selected/executed count. Do not clone owner test matrices.
3. Generate/reconcile `TEST_PATH_INVENTORY.tsv`, `TRACEABILITY.md`, README
   method/examples claims, and F01–F37 from the landed tree. Migrate the useful
   ledger rows before deleting the old ledger; zero dangling path references.
4. Run representative cross-owner fixtures for Group/Sort/self-Join,
   Rule/builtin/host separation, Composition working-set rows, Inline VIEW,
   query EXISTS, closed-authority failure, and front-door parity.
5. Require final architecture mode with zero transition edges/exceptions, zero
   active V1 seam definitions/usages, zero `_punchlist` artifacts, and no
   runtime/provider/SDK dependency or public noun.
6. Freeze a clean committed candidate SHA only after user authorization. Run
   the complete final command list without changing that SHA; any source change
   invalidates the run and requires a new candidate.
7. Record hosted CI independently as green, failed, or unavailable. Failed
   blocks closure; unavailable permits only `REPO_LOCAL_CLOSED` with the gap
   stated. Local tests never imply hosted green.

Current hard blockers are therefore 05 contract migration, 06 sealed logic
input/final edge, 07 pipeline/facade cutover, 08 bounded front doors, missing
05–08 receipts, and the immutable candidate. Only this ticket may advance the
packet to closeout.

The latest global format check also exits 1 on 23 files. Ticket 09 requires a
zero-diff `cargo fmt --all --check` on the immutable candidate; passing Clippy
does not substitute for it. The formatting repair belongs to the ticket owners
whose files are listed by the check and must precede candidate freeze.

## Final Independent RR — Closeout Authority Corrections

1. **One semantic manifest SSOT.** Expand `examples-v2.tsv` (or its final
   replacement) with all 18 root-family semantic matrix fields and load it from
   conformance. Keep a separate two-row Decl/HornRule supplemental manifest.
   Generate README/example-index views and delete the parallel hardcoded Rust
   example/outcome table. Enforce
   `filesystem == root manifest == selected == executed`.
2. **Recursive tracked hygiene.** Replace one-directory `src`/`tests` walking
   with recursive tracked-path authority, preferably a `git ls-files`-derived
   fixture with explicit historical allowlists. Reject root repro patterns,
   backup suffixes, and `_punchlist` anywhere under source, tests, docs, or
   tools. Add nested mutations and remove stale “Ticket 16” labels.
3. **Immutable test inventory.** Pin `PACKET_BASE_SHA` before candidate freeze
   and derive landed paths from `PACKET_BASE_SHA..HEAD_SHA`, including rename
   source/destination, additions, deletions, support modules, and test-target
   classification. Preserve the separate pre-commit dirty ownership snapshot.
4. **Facade/frontdoor CI.** Add both external-consumer and frontdoor-parity
   scripts to named local and hosted CI recipes. Record selected/executed counts
   and exit codes; green Rust tests do not replace either script.
5. **Allow-attribute migration.** After Tickets 02–08 delete old seams,
   regenerate the complete `#[allow]`/`#![allow]` path inventory. Assign
   semantic cases to their owner; convert justified survivors to
   `#[expect(..., reason=...)]` and delete obsolete ones. Final proof requires
   zero `rg '#!?\[allow\(' crates --glob '*.rs'` plus MSRV and current strict
   Clippy. The current audit found 119 occurrences. Removing a CI `-A` flag
   alone is not a migration plan.
6. **Exact-SHA hosted evidence.** Record remote URL and `gh auth status`; query
   required workflow jobs by candidate SHA and store URLs, conclusions, and
   timestamps. Auth/network/no-pushed-SHA failures are `unavailable` only with
   the attempted command/error. Queued or in-progress remains `OPEN`. Push/PR
   is separate user authorization.

### Two-phase immutable evidence

A commit cannot contain evidence produced after testing itself. Default
closeout is:

1. User-authorized candidate commit A contains Tickets 01–09 implementation,
   generated inventories, and an unclaimed receipt template. Run the complete
   command set on SHA A.
2. Create attestation commit B containing only keyed results/receipts/docs.
   Verify `A..B` is receipt/docs-only and rerun diff, format, doc, inventory,
   and claim validators on B. Record `tested_sha=A` and
   `attestation_sha=B`.

Alternatively keep the attestation external and state that A does not contain
its post-run receipt. Do not claim one SHA both contains and predates its
post-run evidence. The candidate includes Ticket 09 conformance/CI
implementation, not only Tickets 01–08.
## 2026-09-21 Final Current-Tree Audit

Status remains `PLANNED — ONLY TICKET AUTHORIZED TO CLOSE PACKET`. Current
closeout is impossible:

- receipts 05-08 are absent;
- Algebra is test-compile RED with 74 errors;
- Pipeline is compile RED with 25 library/29 library-test errors;
- Conformance fails at Pipeline compilation;
- final architecture is RED on 22 normal plus one dev transition;
- workspace format is RED on 27 unique files;
- the worktree is dirty and includes an externally owned staged deletion of
  `test_repro_cases.txt`.

Closeout must use one authorized clean candidate and two source identities:
the exact base commit and the exact candidate tree/commit. Required order:

1. Validate receipts 00-08 against their frozen hashes and selected/executed
   counts; stale dirty-overlay checkpoints are not receipts.
2. Load one authoritative 18-root semantic manifest plus the two supplemental
   Decl/Rule units. Generate docs/index/test expectations from it; do not keep
   independent hardcoded lists.
3. Run owner suites, facade/front-door parity, deliberate mutations, public API
   inventory, recursive artifact hygiene, `just ci`, final architecture,
   formatting, and clean-tree checks on the same candidate.
4. Compare base-to-candidate tracked and untracked inventories, including
   deleted/renamed files. Fail on orphaned crates, stale paths, duplicated
   manifests, compatibility shims, and missing receipts.
5. Update truth documents only from observed results. Distinguish historical
   receipt proof from current candidate proof and never infer remote CI or
   deployment/activation.

Only after every command is green may this ticket change the packet status to
closed.

### Latest closeout-gate recheck

`cargo fmt --all --check` now exits 0 and supersedes the earlier 27-file RED
observation. Closeout remains impossible: Algebra still exits 101 with 74
test-compilation errors, Pipeline library check still exits 101 with 25
errors, final architecture still has 22 normal plus one dev transition edge,
receipts 05-08 are absent, and the tree is not an authorized immutable
candidate. Formatting recovery must not be promoted to packet closure.

## SS-09A Current-Tree Conformance/Closeout Audit — 2026-09-21

Status remains `PLANNED — ONLY TICKET AUTHORIZED TO CLOSE PACKET`. Correct two
stale observations before using this ticket as authority: Receipt 05 now
exists, and Algebra's current 13 integration tests compile and pass. They are
still raw-HIR legacy coverage, not Ticket 05 semantic evidence. Pipeline
remains the actual workspace blocker (`cargo check -p quanta-gqlang-v2-pipeline
--lib`: 25 compile errors), so conformance cannot run.

Current conformance is a V1 routing guard: it directly depends on HIR,
Pipeline, Planning, and Surface and checks only family/plan summaries. It has
no sealed authority fixture, retained semantic payload matrix, mutation
receipt, selected=executed accounting, or CLI/stdio parity. Replace it only
after Tickets 05–08 expose their sealed public path; do not build a parallel
manifest/test registry while V1 remains active.

Already useful work to preserve: Semgrep is no longer the original zero-match
corruption. Its runner creates a positive fixture for each of seven configured
rules and enforces configured=executed. Surface example/doc rails are also
useful inputs, but must be migrated into one conformance-owned manifest rather
than copied. Current hygiene is too shallow for candidate closure.

Final closeout remains ordered behind 05–08: one authoritative semantic
manifest plus two supplemental Decl/Rule rows; migrated conformance and
frontdoor/facade parity; candidate-scoped base..candidate inventory; final CI
and architecture edge removal; then the two-commit tested-SHA/attestation-SHA
procedure already specified above. The dirty overlay and staged external
`test_repro_cases.txt` deletion cannot be treated as a candidate.

## RR-09B Authoritative Current-State Reconciliation — 2026-09-21

This is the current closeout authority for `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
plus the 178-entry dirty overlay. It supersedes earlier claims that Algebra had
13 legacy tests/74 compile errors, that 22 normal transition edges remain, that
119 allow attributes remain, or that Receipts 05-07 are absent.

### Current gate ledger

| Gate | Current observed evidence | State |
|---|---|---|
| Receipts | 01-07 exist; 07 is blocker-only; 08/09 were absent before this audit and the newly created 08/09 files are audit-only | OPEN |
| Algebra | all-target owner suite: 4/4 integration tests pass | owner slice GREEN, packet not closed |
| Logic/consumer compile | Logic fails on four deleted Algebra V1 symbols; Planning and Pipeline checks exit 101 at that dependency | RED |
| Format | `cargo fmt --all --check`: exit 0 | GREEN for current dirty bytes only |
| Architecture | 42 normal edges = 22 final + 20 transitional; three dev edges = one final + one transitional + illegal `sema -> surface`; final mode exits 1 | RED |
| Semgrep | seven rules execute over 95 targets; 13 blocking findings, all in current Sema paths | non-vacuous but RED |
| Allow attributes | current Rust scan finds 95 `#[allow]`/`#![allow]` occurrences | migration OPEN |
| Conformance | hardcoded V1 `root_family_specs_v1`; route/kind/result/effect assertions only | insufficient |
| Examples | 18 tracked `.qglang` files; no authoritative `examples-v2.tsv` exists | registry OPEN |
| Required V2 tests | `v3_semantic_family_matrix`, `v3_registry_non_vacuity_v2`, and `v3_public_api_inventory` do not exist | OPEN |
| External scripts | `tools/test_frontend_external_consumer.py` and `tools/test_frontdoor_parity.py` do not exist | OPEN |
| CI recipes | neither script is in `just ci` or `.github/workflows/lint.yml`; architecture CI does not request final mode | OPEN |
| Hygiene | test scans only root files and one non-recursive `src/tests` level; staged external deletion `test_repro_cases.txt` remains | RED for candidate closure |
| Hosted CI | no exact-candidate SHA, URL, job conclusion, or authenticated query | UNVERIFIED |

The Semgrep infrastructure is not the original zero-match corruption: its
configured/executed rule count is meaningful. Its current result is still RED,
not “fixed,” because it reports four silent error discards and nine string-typed
error results in Sema. Ticket owners must disposition those exact findings;
Ticket 09 only verifies zero current findings.

### Exact remaining work

1. **Finish Tickets 05-08 first.** Require final owner receipts for aggregate
   Algebra, sealed document Logic, Planning/Pipeline/facade, and bounded
   frontdoors. Audit-only receipts do not satisfy this precondition.
2. **Create one semantic manifest authority.** Add
   `conformance/tests/fixtures/examples-v2.tsv` with exactly the 18 filesystem
   examples and all required authority/applicability/expected-payload/
   diagnostic/resource/mutation/owner-receipt fields. Keep Decl and HornRule as
   the separate two-row supplemental manifest. Delete the hardcoded Rust table
   only after generated consumers pass equality.
3. **Implement V2 conformance owners.** Add the three pinned test targets and
   make them execute the sealed facade path. Record declared, selected, and
   executed counts; assert exact semantic payload or diagnostic/span, not only
   family labels.
4. **Generate truth inventories.** Create `TRACEABILITY.md`,
   `TEST_PATH_INVENTORY.tsv`, final public API inventory, examples index, method
   allowlist, and closeout template from current tracked inputs. Validate both
   directions and every referenced path/anchor. Migrate live ledger rows before
   deleting the legacy ledger.
5. **Make hygiene recursive and candidate-scoped.** Derive tracked paths from
   `git ls-files` and `PACKET_BASE_SHA..candidate`, including adds, deletes, and
   both sides of renames. Reject nested backup/repro/`_punchlist` artifacts,
   orphaned crates, stale paths, duplicate manifests, and unexplained dirty or
   untracked files.
6. **Close static-policy debt.** Resolve the 13 observed Semgrep findings,
   migrate all 95 allow attributes to owner-scoped code or justified
   `#[expect(..., reason=...)]`, remove CI `-A` suppressions, and require both
   MSRV and current strict Clippy. Re-run counts; do not pin today's numbers as
   the desired result.
7. **Wire missing integration gates.** Implement the facade consumer and
   frontdoor parity scripts in Tickets 07/08. Add both to `just ci` and hosted
   CI. Change the closure architecture job to `--require-final` only when all
   transition edges and exceptions are actually removed.
8. **Freeze evidence in two phases.** After user authorization, commit candidate
   A containing implementation, generated inventories, and an unclaimed receipt
   template. Run all commands without changing A. Then create docs/receipt-only
   attestation B, prove `A..B` contains no source/config change, and rerun the
   claim/diff/format/doc/inventory validators. Record `tested_sha=A` and
   `attestation_sha=B`.
9. **Query hosted evidence independently.** Record remote URL, auth result,
   candidate SHA, required job URLs/conclusions, and timestamp. `failed` blocks;
   `unavailable` permits only `REPO_LOCAL_CLOSED`; queued/running remains OPEN.

### Final proof order

The following order is mandatory because a source change invalidates all later
evidence:

```bash
git status --short
git rev-parse HEAD
git diff --check
cargo fmt --all --check
taplo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo +1.95.0 clippy --workspace --all-targets -- -D clippy::unused_result_ok -D warnings
cargo nextest run --workspace --no-tests pass
cargo test --workspace --doc
cargo test -p quanta-gqlang-v2-conformance --test v3_semantic_family_matrix
cargo test -p quanta-gqlang-v2-conformance --test v3_registry_non_vacuity_v2
cargo test -p quanta-gqlang-v2-conformance --test v3_public_api_inventory
python3 tools/test_frontend_external_consumer.py
python3 tools/test_frontdoor_parity.py
python3 tools/architecture/check_architecture.py --require-final
cargo deny check bans licenses sources advisories
typos .
tools/lint/run_semgrep.sh
just ci
git status --short
git rev-parse HEAD
```

Every command records exit code, selected/executed count, and the unchanged
tested SHA. A focused owner pass, current dirty-tree format pass, non-vacuous but
failing Semgrep run, or transitional architecture pass cannot substitute for
this sequence.

### Parallelization constraint

Before 07/08 close, Ticket 09 may only implement/read-review manifest schemas,
generators, and candidate-independent validators. After the public facade is
frozen, semantic manifest/conformance work and recursive hygiene/inventory work
may proceed in parallel on disjoint files. CI recipe integration, truth-doc
generation, candidate freeze, full commands, hosted query, and attestation are
strictly serial. No parallel writer may modify candidate A after qualification
starts.
