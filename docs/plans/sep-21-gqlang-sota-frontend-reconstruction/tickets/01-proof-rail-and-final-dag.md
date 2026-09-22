# 01 — Proof Rail And Final DAG

Status: `CURRENT TREE RED — HISTORICAL RECEIPT ONLY`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

F03, F12's transport-identity portion, and the proof-gate portion of F14/F29.

## Goal

Make repository gates describe and inspect the architecture that will actually
exist for the rest of the packet.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | current Cargo workspace, CI job structure, `Justfile` command surface, transport implementation/tests | change only proven-bad configuration and names; the transport move must be byte-behavior preserving |
| `MIGRATE` | `check_architecture.py`, Semgrep runner/rules, old `lsp` crate | replace stale inventories with the pinned DAG, add non-vacuity, and tracked-move the crate/tests |
| `DELETE` | deleted-crate topology, stale Semgrep include, floating action tags, wildcard path versions, old LSP identity | remove in the same commit that installs the exact replacement |
| `NEW` | empty sema crate, architecture fixtures, per-rule Semgrep fixtures, non-vacuity tests | no semantic implementation is permitted here |

Do not create a second architecture checker or a copied diagnostics crate. The
production checker and moved transport remain the single implementations.

## Write Set

1. `/Cargo.toml`
2. `/Cargo.lock`
3. `/crates/quanta-gqlang-v2-sema/Cargo.toml`
4. `/crates/quanta-gqlang-v2-sema/src/lib.rs`
5. `/tools/architecture/check_architecture.py`
6. `/.semgrep/qglang-frontend.yml`
7. `/tools/lint/run_semgrep.sh`
8. `/.github/workflows/lint.yml`
9. `/deny.toml`
10. `/.taplo.toml`
11. `/.typos.toml`
12. `/rust-toolchain.toml`
13. `/Justfile`
14. `/crates/quanta-gqlang-v2-algebra/Cargo.toml`
15. `/crates/quanta-gqlang-v2-cli/Cargo.toml`
16. `/crates/quanta-gqlang-v2-conformance/Cargo.toml`
17. `/crates/quanta-gqlang-v2-frontend-contract/Cargo.toml`
18. `/crates/quanta-gqlang-v2-logic/Cargo.toml`
19. `/crates/quanta-gqlang-v2-pipeline/Cargo.toml`
20. `/crates/quanta-gqlang-v2-planning/Cargo.toml`
21. existing `/crates/quanta-gqlang-v2-lsp/` during atomic rename
22. new `/crates/quanta-gqlang-v2-diagnostics-stdio/`
23. `/README.md` and `/docs/language/README.md` for transport-name truth
24. `/docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/01.md`
25. typo-only edits in
    `crates/quanta-gqlang-v2-algebra/src/node.rs`,
    `crates/quanta-gqlang-v2-modules/src/lockfile.rs`,
    `crates/quanta-gqlang-v2-modules/tests/v3_modules_version_lock_edges.rs`,
    `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`, and
    `crates/quanta-gqlang-v2-surface/tests/v3_mutation_rails.rs`
26. `crates/quanta-gqlang-v2-conformance/src/lib.rs` and
    `crates/quanta-gqlang-v2-conformance/src/leakage.rs` — delete the orphaned
    public denylist after its complete token set moves to the Semgrep SSOT
27. `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs` and the tracked-moved
    `diagnostics-stdio/tests/method_allowlist_doc_gate.rs`
28. `crates/quanta-gqlang-v2-cli/tests/v3_cli_lsp_parity.rs` for the required
    import-only transport identity rename; no test behavior changes are allowed
29. `/crates/quanta-gqlang-v2-surface/Cargo.toml` — exact MSRV-compatible
    `proptest` development dependency pin shared with pipeline
30. `/crates/quanta-gqlang-v2-cli/src/main.rs` — behavior-preserving
    `ExitCode` return required by the restored all-target Clippy rail

## Implementation

1. Add an intentionally minimal sema crate with no copied legacy behavior or
   fake semantic API.
2. Mechanically rename the flat-JSON crate/binary to
   `quanta-gqlang-v2-diagnostics-stdio` without changing request behavior.
3. Encode the final dependency DAG plus an exact, shrinking transitional-edge
   ratchet from the audited base.
4. Inspect all workspace members and normal/dev dependency edges.
5. Keep explicit dev-edge exceptions small and documented.
6. Reject runtime, SDK, index, provider, and legacy gqlang dependencies.
7. Split gate ownership once: the Python architecture checker owns workspace
   membership, normal/dev dependency edges, and forbidden backend/legacy
   dependencies from Cargo metadata; Semgrep owns Rust source imports/tokens,
   host coupling, hub bypass, silent error discard, and typed-error policy.
   Delete the overlapping Cargo-dependency Semgrep rules and remove source-text
   regex scanning plus the dead `leakage.rs` exemption from the Python checker.
8. Replace stale Semgrep crate paths with current and target owners. Retain or
   create exactly these seven source-policy rule IDs: runtime/SDK/index import,
   legacy gqlang import, host-coupling token, frontend-hub bypass,
   runtime-truth token, silent `Err(_)` discard, and core `Result<_, String>`.
9. Add rule non-vacuity: every configured rule must run or the gate fails.
10. Fix six manifest format failures and six deny wildcard dependencies.
11. Fix current typo findings without broad unrelated rewrites.
12. Pin toolchain/MSRV and installed CI tool versions.
13. Move all runtime-truth tokens from the unconsumed conformance `leakage.rs`
    into the Semgrep runtime-truth rule, delete the dead module/export, and
    remove every exclusion for that file. Semgrep is the only token-policy
    owner; do not leave two denylist tables or claim an output sweep exists.

## Step-By-Step Procedure

1. Add the new sema crate with only `#![forbid(unsafe_code)]` and crate docs.
   Do not add fake modules, stub APIs, or code copied from deleted crates.
2. Add the crate to `[workspace].members` and `[workspace.dependencies]` with the
   same exact version policy as the other workspace crates.
3. Move `crates/quanta-gqlang-v2-lsp/` to
   `crates/quanta-gqlang-v2-diagnostics-stdio/`; change package, lib, bin, Rust
   import, test, workspace, lockfile, README, and language-doc names in the same
   commit. Remove the workspace `lsp` keyword. Preserve behavior for ticket 08
   to harden.
4. Rewrite `EXPECTED_WORKSPACE_MEMBERS` to this final 14-crate inventory:
   `algebra`, `binder`, `cli`, `conformance`, `diagnostics-stdio`,
   `frontend-contract`, `hir`, `logic`, `modules`, `pipeline`, `planning`,
   `sema`, `surface`, and `types`. The old `lsp` name must not remain.
5. Rewrite `ALLOWED_INTERNAL_DEPENDENCIES` to the final DAG in
   `../TARGET_ARCHITECTURE.md`; include normal and dev edges separately. Add the
   exact migration exceptions below as `TRANSITIONAL_NORMAL_EDGES` and
   `TRANSITIONAL_DEV_EDGES`. Default mode reports their present count;
   `--require-final` rejects any present transitional edge.
6. Add architecture test fixtures under
   `tools/architecture/tests/fixtures/{valid,reverse-edge,runtime-edge}` and a
   test runner that invokes the same production checker.
7. Apply the gate-ownership split above. The architecture checker must not scan
   Rust source, and Semgrep must not restate dependency rules already enforced
   from Cargo metadata. Move the live hub-bypass/runtime-truth policies into
   Semgrep before deleting their Python copies so no enforcement gap exists.
8. Change the typed-error rule's deleted `syntax/sema/ir` paths to the exact
   current core owners. Do not broaden it to `crates/**/*.rs`.
9. Extend `run_semgrep.sh` to compare configured rule IDs with executed rule IDs
   and fail on any missing rule or zero total targets.
10. Add one tracked positive fixture per retained/new Semgrep rule. Mark fixtures as test data
   so the production scan does not treat the intentional violation as a real
   finding.
11. Run `taplo fmt` only on the six failing manifests, inspect the diff, then run
   `taplo fmt --check` across the workspace.
12. Add explicit workspace versions to the six frontend-contract path
   dependencies. Do not relax cargo-deny's wildcard policy.
13. Correct the nine current typo findings or add narrowly justified identifiers
   to the typo configuration. Do not add broad token-fragment or parser-error
   allowlists.
14. Pin Rust/MSRV and tool action versions in CI, then make local Just recipes and
    Actions invoke equivalent flags.
    Both must retain `--workspace --all-targets`; until ticket 09 normalizes the
    inherited reasoned integration-test attributes, the command may exempt only
    `clippy::allow_attributes` and `clippy::duplicated_attributes`. No target
    class may be omitted. Because Rust 1.81 does not expose
    `clippy::unused_result_ok`, the pinned 1.95 lane and matching local recipe
    explicitly add `-D clippy::unused_result_ok`; deleting the policy is not an
    allowed MSRV workaround.
15. During the mechanical transport move, correct the stale test-support comment
    that names deleted `stdio_read_only.rs`; this is identity cleanup, not a
    behavior change.
16. Rewrite the moved method documentation gate to inspect both root README and
    language README and pin the one actual `v3_diagnostics` method plus the
    explicit non-LSP identity. The root README must not continue advertising
    six deleted methods.

## Files To Add

1. `crates/quanta-gqlang-v2-sema/Cargo.toml`
2. `crates/quanta-gqlang-v2-sema/src/lib.rs`
3. `tools/architecture/tests/test_architecture.py`
4. `tools/architecture/tests/fixtures/valid/`
5. `tools/architecture/tests/fixtures/reverse-edge/`
6. `tools/architecture/tests/fixtures/runtime-edge/`
7. `tools/lint/tests/fixtures/semgrep/` with one positive fixture per rule
8. `tools/lint/tests/test_semgrep_non_vacuity.py`
9. `crates/quanta-gqlang-v2-diagnostics-stdio/` created by tracked move from the
   old crate; no duplicate old directory remains
10. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/01.md`

## Required Proof Fixtures

Architecture checker accepts `--root <fixture-workspace>` so tests execute the
production checker against isolated metadata:

| Fixture | Exact contents | Expected result |
|---|---|---|
| `valid` | 14 named empty crates and exactly the target normal edges | exit 0, 14 members, nonzero edges |
| `transitional-valid` | final edges plus one declared transitional edge | default exit 0 and transition count 1; `--require-final` exit 1 |
| `reverse-edge` | `sema -> planning` added to `valid` | exit 1 naming that edge |
| `runtime-edge` | `cli -> quanta-runtime` path dependency added | exit 1 naming forbidden dependency |

Create one `.rs` fixture named after each of the seven source-policy Semgrep
rule IDs under `tools/lint/tests/fixtures/semgrep/`. The non-vacuity test runs every
rule twice: its positive fixture must report that exact ID, while a clean
control must report none. It also removes one configured include path in a temp
copy and proves the runner rejects the unexecuted rule.

The runtime-truth rule has a table-driven non-vacuity subtest for all 27 tokens
migrated from `leakage.rs`, including `rows=`/`verdict=` prefix shapes, plus
near-miss clean identifiers such as `query_result_count`, `no_result_reason`,
and `table_rows=5`. One representative positive fixture is not sufficient for
this alternation-heavy rule.

## Pinned CI Toolchain

Use these exact versions for this packet:

| Tool/action | Pin | Verified upstream ref on audit date |
|---|---|---|
| Rust MSRV and repository default | `1.81.0` | repository `rust-version` |
| Rust current compatibility lane | `1.95.0` | toolchain release |
| Taplo CLI | `0.10.0` | crate release |
| cargo-nextest | `0.9.104` | crate release |
| cargo-deny | `0.19.7` | crate release |
| typos-cli | `1.46.3` | crate release |
| Semgrep | `1.157.0` | PyPI release |
| `actions/checkout` | `11d5960a326750d5838078e36cf38b85af677262` | `v4.4.0` |
| `dtolnay/rust-toolchain` | `6bed0761d98439e5a578e2877258200ad565ba87` | `stable` branch snapshot |
| `Swatinem/rust-cache` | `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` | dereferenced `v2.9.2` |
| `taiki-e/install-action` | `94c31af3204a9f15ab40b35ad084410b905bbc73` | `v2.87.17` |

The action commits were resolved from their official remotes on `2026-09-21`.
Workflow comments record the human-readable major tag next to each immutable
SHA. `rust-toolchain.toml` pins `1.81.0`; CI runs the full required suite on MSRV
and at least `cargo check --workspace` plus tests on `1.95.0`. Every installer
receives the exact CLI version above. Semgrep uses
`python3 -m pip install semgrep==1.157.0`.

## Transitional Edge Ratchet

These names are evaluated after the mechanical diagnostics-stdio rename. No
other internal edge is temporarily allowed.

```text
normal algebra -> hir
normal algebra -> binder
normal cli -> hir
normal cli -> pipeline
normal cli -> planning
normal cli -> surface
normal cli -> types
normal conformance -> hir
normal conformance -> pipeline
normal conformance -> planning
normal conformance -> surface
normal frontend-contract -> algebra
normal frontend-contract -> hir
normal frontend-contract -> planning
normal frontend-contract -> surface
normal frontend-contract -> types
normal logic -> hir
normal diagnostics-stdio -> pipeline
normal pipeline -> hir
normal pipeline -> binder
normal pipeline -> types
normal planning -> hir
dev    cli -> diagnostics-stdio
```

The checker prints `transitional_present`, `transitional_declared`, and the
sorted present edges. The declared set is fixed in ticket 01; later work removes
actual edges but never adds exceptions. Ticket 09 deletes the now-empty
transitional set and changes CI to invoke `--require-final`.

## Exact Verification Order

```bash
python3 tools/architecture/tests/test_architecture.py
python3 tools/architecture/check_architecture.py
tools/lint/run_semgrep.sh
taplo fmt --check
cargo deny check bans licenses sources advisories
typos .
cargo metadata --format-version 1 --no-deps
cargo test -p quanta-gqlang-v2-diagnostics-stdio
cargo check --workspace
rg -n 'uses: [^ ]+@(v[0-9]+|stable|main|master)$' .github/workflows
git diff --check
```

Capture rule configured/executed counts and architecture package/edge counts in
the ticket receipt. The action-ref scan must return no matches. An exit code
without proof counts is insufficient.

## Forbidden Shortcuts

1. deleting failing gates
2. adding broad allowlists for current failures
3. keeping both old and new expected crate inventories
4. counting an unexecuted Semgrep rule as success
5. introducing sema behavior before tickets 02-04 define it
6. copying the old transport instead of moving it
7. changing framing or request semantics during the mechanical rename
8. allowing a wildcard transition such as `cli -> *`
9. making default architecture mode ignore or hide present transitional edges
10. enforcing the same dependency or source policy independently in both
    architecture Python and Semgrep
11. excluding dead `leakage.rs` from the runtime-truth rule or retaining its
    unconsumed public token table beside Semgrep

## RED

Fresh base behavior:

1. `just ci` stops at Taplo.
2. architecture expects deleted syntax/sema/ir crates.
3. cargo-deny rejects six frontend-contract dependencies.
4. typos reports nine findings.
5. Semgrep declares seven rules but runs six.
6. a crate named LSP implements a private flat-JSON protocol.

## Tests

1. architecture fixture with one illegal reverse dependency
2. architecture fixture with one forbidden runtime dependency
3. Semgrep fixture per rule
4. manifest inventory drift test
5. a test that deleting or misspelling a Semgrep include path fails
6. all moved diagnostics-stdio tests pass unchanged after import/binary rename
7. current test files are moved, not copied; `find crates -path '*lsp*'` and an
   old-package scan prove that no duplicate transport suite remains
8. CLI command and diagnostics method allowlists assert exact equality and
   non-emptiness so an empty constant cannot make the doc gate pass vacuously

## Definition Of Done

```bash
cargo fmt --all --check
taplo fmt --check
python3 tools/architecture/check_architecture.py
cargo deny check bans licenses sources advisories
typos .
tools/lint/run_semgrep.sh
```

All pass and each reports nonzero inspected targets. Request/response behavior
remains unchanged; only the transport package/binary identity and the new empty
sema crate differ. The receipt records exactly 22 present transitional normal
edges and one present transitional dev edge after the rename.

## Dependencies

Requires ticket 00. Blocks every other implementation ticket.

## Current-Head Documentation-Truth Addendum — 2026-09-21

The final ticket audit found that `docs/language/test_hellgate_ledger.md`, which
the root README advertised as the complete mapping, still named the retired v2
syntax/sema/IR and LSP rails. All 50 unique referenced Rust test paths were
absent. The active README surfaces were correct, so this was stale proof
documentation rather than a missing transport migration.

Repair:

1. Replaced the ledger with the current 14-crate frontend authority map and 77
   existing Rust rail paths.
2. Corrected the stale modules subsection in `docs/language/README.md` to the
   actual `v3_modules_v2_authority.rs` owner rail and listed all three current
   conformance rails.
3. Extended `method_allowlist_doc_gate.rs` so every backticked Rust path in the
   ledger must exist and the deleted LSP package path cannot reappear.

A direct repository path scan reports `ledger_paths=77 missing=0` and the stale
package/test-name scan is empty. The expanded Rust doc-gate cannot be executed
at this intermediate checkpoint because Cargo first compiles the deliberately
unmigrated binder consumer of ticket 02's breaking HIR. Its execution remains a
ticket-04+/09 integration obligation; this addendum does not overwrite the
historical green result recorded in receipt 01.

The post-receipt `method_allowlist_doc_gate.rs` source change is therefore not
covered by receipt 01. Do not call the current-tree Ticket 01 gate green until
the exact new test executes on the integrated stack. Preserve the earlier
fingerprint's verified evidence separately.

Recheck on `main@f9ec954` plus the active dirty overlay:
`cargo test -p quanta-gqlang-v2-diagnostics-stdio --test
method_allowlist_doc_gate` exited 101 before selecting that test because the
unchanged pipeline V1 consumer failed compilation with 25 errors (retired
modules/types imports, private HIR fields, removed Compose/Quantifier, and
obsolete type construction). This is an integration dependency failure, not
a passed or failed doc-gate assertion. Re-execute the exact command after
Ticket 07 migrates the pipeline; do not bypass the compiler with a text-only
path scan and declare the Rust gate green.

## 2026-09-21 Live-Tree Duplicate/Scope Check

The crate/binary is already `quanta-gqlang-v2-diagnostics-stdio`; Ticket 08
changes its protocol in place and must not create another transport crate.
The current architecture checker passes with 14 members, 43 normal edges
(21 final, 22 declared transitional), and one transitional dev edge. This
reconfirms the transition ratchet only; Ticket 09, not Ticket 01, removes the
exception set and proves final mode. No new Ticket 01 implementation work was
identified by this pass. Its receipt remains tied to its recorded fingerprint.

## Post-Receipt DAG Correction Assigned To Ticket 06

The original final allowlist included `logic -> algebra` solely for the
inherited EXISTS→Semi/Anti assembly in `logic/src/source.rs`. Ticket 05 now
owns that relational lowering and Ticket 06 owns Rule/Graph contracts, so the
final target is `logic -> sema + types`. This is a planned change to the
architecture checker, not evidence that its current allowlist has changed.
Ticket 06's integrator write set must migrate the old normal edge to an exact
transitional edge while it is present, remove the actual Cargo edge, then
remove that transition entry. Update the valid/transition fixtures and rerun
the architecture unit suite plus both checker modes. The 21-final-edge
Ticket 01 receipt is historical; only a post-change receipt can establish the
revised 20-edge final allowlist.

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: historical owner receipt retained; current tree is
`POST-RECEIPT DELTA UNQUALIFIED`. Do not rebuild the diagnostics crate rename,
Semgrep runner, architecture checker, or transition ratchet.

Current observed architecture proof:

```text
workspace_members=14
normal_edges=43 final_present=21 transitional_present=22
dev_edges=1 final_dev_present=0 transitional_dev_present=1
```

The checker exits 0 because every live non-final edge is ratcheted. This is not
final-DAG proof. Exact remaining work:

1. Ticket 06 removes `logic -> algebra` and its HIR-facing logic adapters, then
   changes the final normal allowlist from 21 to 20 and updates positive,
   transition, and invalid architecture fixtures in the same patch.
2. Ticket 07 removes pipeline/planning raw-HIR and direct owner edges.
3. Ticket 08 removes executable direct internal edges, including the CLI dev
   edge to diagnostics-stdio.
4. Ticket 09 requires `transitional_present=0` and
   `transitional_dev_present=0`; deleting an exception before its Cargo edge or
   leaving an unused exception both fail.
5. Re-run `method_allowlist_doc_gate` only after Ticket 07 restores pipeline
   compilation. Its post-receipt source change remains unqualified until then.

Stop condition: do not edit Ticket 01 production assets merely to make a later
consumer compile. Later tickets own their actual edges and consumers.

Current global formatting gate: `cargo fmt --all --check` exits 1 and reports
23 Binder/HIR/Sema/Surface source or test files. The integrator must format each
owned ticket write set after reconciling shared changes, then rerun tests and
Clippy because formatting changes the receipt fingerprint. Do not run a blind
workspace rewrite over a moving shared tree or treat historical Ticket 01
format evidence as current-tree proof.

### Current format supersession

The later integrated `cargo fmt --all --check` exits 0. This supersedes the
earlier 23/27-file format RED observation, but does not change this ticket's
architecture verdict: transition mode is GREEN and `--require-final` remains
RED on 22 normal plus one dev transition edge.

## Final Independent RR — No Duplicate Work

Classification remains unchanged: transition architecture is currently green
with 21 final normal edges, 22 transitional normal edges, and one transitional
dev edge; `--require-final` is red on those 23 exceptions. Do not add another
architecture checker or transport. Ticket 06 removes Logic's obsolete edge,
Tickets 07–08 remove their consumer edges, and Ticket 09 alone flips CI to
final mode and refreshes the post-receipt doc-gate proof.
## 2026-09-21 Final Current-Tree Audit

Status remains receipt-scoped. On `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
plus the current dirty overlay:

- architecture checker unit tests: 7/7 GREEN;
- transition-mode architecture lint: GREEN at 14 members, 43 normal edges,
  22 declared transitional normal edges, and one transitional dev edge;
- `--require-final`: RED on those same 22 normal plus one dev edge;
- `taplo fmt --check`: GREEN;
- `git diff --check`: GREEN;
- `cargo fmt --all --check`: RED on 27 unique files.

Do not reopen the architecture algorithm or add a second dependency checker.
The remaining work is mechanical consumer cutover: Tickets 05-08 remove the
declared transitional edges, then Ticket 09 runs final mode. File-scoped
formatting must be done by each active owner; a workspace-wide formatter over
the shared dirty tree is prohibited.

Receipt 01 is historical proof for its frozen fingerprint. It must not be
rewritten to claim current-tree final DAG or format closure.

## SS-01F Current Architecture Re-Audit — 2026-09-21

Audit snapshot: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`,
174 short-status entries. This supersedes the older 43-edge/22-transition
current-tree counts above; those paragraphs remain historical observations.

### Current execution evidence

Executed during this audit:

```text
python3 tools/architecture/tests/test_architecture.py
  7/7 GREEN

python3 tools/architecture/check_architecture.py
  RED
  workspace_members=14
  normal_edges=42 final_present=22 transitional_present=20 transitional_declared=22
  dev_edges=3 final_dev_present=1 transitional_dev_present=1 transitional_dev_declared=1
  illegal dev edge: sema -> surface

python3 tools/architecture/check_architecture.py --require-final
  RED on the same illegal dev edge plus 20 normal and one dev transition

cargo fmt --all --check
  GREEN
```

The two declared Algebra transitions are absent because Algebra production now
depends on Sema and Types. Default mode is genuinely RED, not transition-mode
GREEN: `crates/quanta-gqlang-v2-sema/Cargo.toml` has a test-support
`quanta-gqlang-v2-surface` dev edge that is neither final nor transitional.

Historical receipt 01 remains evidence for its recorded fingerprint only. No
Semgrep, deny, typos, workspace compile, hosted CI, or clean-candidate run was
performed by this audit.

### Exact remaining work

1. Ticket 04/05 test-fixture ownership removes `sema -> surface` without
   weakening the graph. Move source-to-Sema fixture composition to a lawful
   consumer/test-support owner, or feed Sema a checked HIR document. Do not add
   the reverse edge to an allowlist merely to make default mode pass.
2. Ticket 06 removes `logic -> hir`; if Logic no longer consumes Algebra after
   its document adapter lands, update the final edge set and all fixtures in the
   same cut rather than preserving a fictional edge.
3. Ticket 07 migrates Pipeline, Planning, Frontend Contract, and Conformance;
   remove each corresponding transitional Cargo edge with its consumer.
4. Ticket 08 migrates CLI and diagnostics-stdio frontdoors and removes the CLI
   test-only diagnostics edge.
5. Ticket 09 deletes all unused transition declarations, requires both checker
   modes to report zero transition/illegal edges, then runs Semgrep non-vacuity,
   Taplo, deny, typos, workspace checks/tests/Clippy, and exact-SHA hosted CI.

### Owner boundary

`tools/architecture/check_architecture.py` remains the sole Cargo-edge oracle;
`tools/lint/run_semgrep.sh` remains the source-policy oracle. Consumers remove
their actual dependencies. Ticket 01 must not gain consumer adapters, a second
checker, an exception for `sema -> surface`, or false final edges.

### Next SS / RR checklist

`SS-01G` freezes the post-consumer Cargo manifests and records all three edge
kinds plus exact checker output. RR requires:

- the seven architecture fixtures still pass;
- default and `--require-final` both exit 0;
- every declared final edge is justified by a live consumer;
- transition sets are empty, not merely absent from the current graph;
- the illegal Sema dev edge is removed at its fixture owner;
- current Semgrep configured/executed counts and non-vacuity are captured;
- results are bound to a clean candidate SHA rather than this dirty overlay.
