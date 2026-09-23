# Live Goal Backlog — 2026-09-22

Status: `HISTORICAL SNAPSHOT; DO NOT DISPATCH BG-01..BG-13 FROM THIS FILE`

Current override (2026-09-24): qglang frontend source
`b805cef03f96d7f87a7ba8942c499c940e9432a0` passed the frontend local
gate; Ticket 09/Receipt 09 own the evidence and the current non-duplication
closeout decision. This historical backlog is not a dispatch list for
duplicating owner tests or inventories.
Semantica consumer/product execution remains separate. The following
2026-09-23 override is historical.

Historical override (2026-09-23): W1-W6 product code is integrated at
`main@d9d0f28`; this file's `ba42419899cfd8c347ff80627cf1da96bb83768e`
dirty-tree build failures and BG work
order are no longer current. Ticket 09 owns owner-oracle/semantic-family
accounting, repair of a demonstrated gap, clean current-source qualification,
and packet attestation. Semantica consumer execution remains separate. Use
the packet README and Ticket 09 for current routing.

## Scope, snapshot, and proof boundary

This is a current-state backlog, not a new language decision or a completion
receipt. Snapshot: `main@ba42419899cfd8c347ff80627cf1da96bb83768e`,
2026-09-22 16:38 KST, 54 short-status entries in a shared dirty tree. HEAD
advanced from `2bc88e3` while this audit was running. Re-freeze HEAD, status,
source hashes, and active path leases before dispatch. Preserve the staged
external deletion of `test_repro_cases.txt` and all unrelated dirty work.

Goal: a host-agnostic, frontend-only language boundary. Source text must pass
through one checked document transaction, sealed Algebra/Logic contracts,
semantic Planning, one curated facade, bounded CLI/diagnostics-stdio, and
manifest-driven conformance. Physical optimizer rewrites, numerical
cardinality estimates, runtime plans, provider/SDK wiring, and actual JSON-RPC
LSP are **not** closure criteria; Ticket 00 explicitly excludes them.

### Fresh audit evidence on this snapshot

| Gate | Observed result | Claim boundary |
|---|---|---|
| W0-W3 | checkpoint commits and owner receipts exist; W3 Logic normal deps are Sema + Types | historical frozen owner proof, not current whole-workspace proof |
| Planning + Pipeline + facade all-target tests | GREEN: Planning 137, Pipeline 28, facade 0 | scoped package behavior; facade still needs an external consumer oracle |
| Same three crates strict Clippy | GREEN | scoped lint only |
| Facade `cargo check --lib --locked` | GREEN | compilation only |
| CLI + diagnostics-stdio + conformance `cargo check --all-targets --locked` | RED: deleted V1 Pipeline/Planning imports; `conformance` also has a `Copy` derive error | W5/W6 source migration required |
| Final architecture | RED: 18 transitional normal edges, one transitional dev edge, illegal `sema -> surface` dev edge | final DAG not qualified |
| `RUSTDOCFLAGS='-D warnings' cargo doc` for Planning/Pipeline/facade | RED: deleted V1 links in Planning/Pipeline docs and a private-module link | docs are an executable gate gap |
| Semgrep production scan | RED: 157 findings on 99 scanned targets, seven rules; 151 `Result<_, String>`, six silent-error-discard matches | classify production versus test-only before fixing; not equivalent to 157 product defects |
| Rust allow-attribute inventory | 79 `#[allow]`/`#![allow]` matches under `crates/` | current static debt count, not an owner disposition |
| External consumer / frontdoor parity / semantic manifest | scripts and `examples-v2.tsv` absent | no public black-box or corpus closure |

These command results were observed on the live tree, not an immutable
candidate. A later commit or source edit requires a delta audit; do not copy
these counts into a final receipt without rerunning them.

## Dependency order and dispatch rule

```text
W4 contract/authority decisions
  -> W4-G / W4-H (already dispatched; do not duplicate their write leases)
  -> W4 integrator barrier + Receipt 07
  -> W5 CLI || diagnostics-stdio -> serial parity/dependency/doc barrier + Receipt 08
  -> W6 semantic conformance || hygiene/tooling; owner-routed static fixes
  -> W6 final DAG/CI/truth barrier + Receipt 09
  -> W7 immutable candidate A, full local/hosted proof, docs-only attestation B
```

Use `WAVE_EXECUTION_PLAN.md` for path leases and barriers. Each item below has
one acceptance condition; package-green and fixture-minted reachability do
not close a source-reachable contract. Any repair to W1-W3 frozen producer
source reopens its owner receipt and the affected downstream consumer proof.

## P0 — W4 semantic/public boundary (Ticket 07)

### BG-01 — Make the facade a **single dependency** and prove its allowlist

- **Purpose:** the three source-document functions are present and compile,
  but `frontend-contract/Cargo.toml` still has five transitional owner edges
  beyond Pipeline. A curated `lib.rs` alone does not prove external clients
  cannot reach owner internals.
- **Owner/state:** W4 integrator owns Cargo/lock/architecture; W4-H is already
  dispatched for the isolated external-consumer gate. Do not take either lease.
- **Work:** remove facade normal edges to Algebra/HIR/Planning/Surface/Types in
  the same cut that preserves DTO reachability through Pipeline. Build
  `tools/frontend_external_consumer/` with only the facade dependency and
  `tools/test_frontend_external_consumer.py`; require positive compile/run,
  negative compile-fail for each forbidden V1/internal symbol and constructor,
  a full `cargo public-api` allowlist diff, expected compiler-error classes,
  and repository-immutability checks. No new owner-crate dev dependency in the
  external fixture.
- **Accept:** facade normal `cargo tree --depth 1` = Pipeline only; script
  passes all declared positive/negative cases; facade tests and strict Clippy
  pass; no glob/local public-module leakage. Record selected/executed counts
  and source hashes in Receipt 07.

### BG-02 — Settle document-budget ownership before more consumers land

- **Purpose:** `algebra::lower_root` initializes full
  `max_semantic_steps`/`max_expression_nodes` counters per call, Planning's
  `walk` initializes another full semantic-step counter per plan, and Pipeline
  calls those entries per authored item. Exact/+1 within one walk is proven;
  one document's total work is not charged once. This conflicts with
  `TARGET_ARCHITECTURE.md`'s one-owner/non-resettable budget rule.
- **Owner/state:** W4 integrator decision F-07; producer/consumer owners must
  implement the chosen contract together. Do not add an unsealed caller budget
  or another default.
- **Work:** choose and document one of two coherent models: (a) honor the
  current target: Sema charges its semantic-work dimension once, seals the
  admitted document size, and downstream iterative Algebra/Planning walks
  consume that bounded shape without recharging the same dimension; or (b)
  explicitly amend the target to give each downstream walk a separately named
  dimension with one owner-local, non-resettable session across all applicable
  items. Remaining-count state never becomes a public DTO. Reconcile the
  architecture table, Sema receipt, Algebra/Planning signatures, and Pipeline
  invocation count in one amendment. Add a multi-item fixture where each item
  fits individually but the document exceeds the declared total; include a
  nested composition and rule program. Prove exact limit and +1, no reset,
  stable diagnostic code/location, and no double stratification charge.
- **Accept:** one named charge owner per dimension; multi-item exact/+1 tests
  fail if per-call reset is restored; W1-W4 affected owner suites/receipts
  rerun on one fingerprint. This is a contract blocker, not a cosmetic lint.

### BG-03 — Complete W4-G semantic seam and eliminate duplicated decisions

- **Purpose:** Pipeline currently stratifies a rule program and Planning
  stratifies it again; `pipeline::verify_composition` repeats legality already
  owned by Planning. Duplicate validation can diverge and double-charge.
- **Owner/state:** W4-G already dispatched for the one-stratification seam,
  exact assertions, and fabricated-anchor cleanup. W4 integrator owns F-10's
  composition ruling. Do not edit W4-G paths concurrently.
- **Work:** compute and seal the stratification proof once at Logic ingress,
  pass that proof through Planning/Pipeline rather than recomputing; reduce
  Pipeline composition checks to exact document assembly identities only
  (missing/extra/duplicate/wrong-kind/ordinal/body) and leave semantic
  legality to Sema/Planning. Replace the three self-referential assembly
  assertions with exact expected payload fields. Remove synthetic source-0
  locations and the dead source-0 error path where a checked payload location
  exists; typed source-less authority errors remain source-less.
- **Accept:** mutation of the retained proof or composition body identity is
  rejected at the proper owner; source-level rule program is stratified once;
  Pipeline 28+ and Planning 137+ tests, strict Clippy, and exact diagnostic
  code/location regressions pass. Counts may grow; these are floors only.

### BG-04 — Resolve two W4 producer/API gaps without fixture-only claims

- **Purpose:** a real source `USE PACK` still rejects in Sema while Planning's
  `MetaActivation` is exercised only through a crate-local forged document.
  `ResolvedRelationSourceV2` is also a dead public Sema carrier with an
  unchecked constructor. Both weaken the claimed public contract.
- **Owner/state:** W4 integrator F-09 plus a deliberate Sema owner reopen.
  Ticket 00 includes USE PACK in the tracked source family; scoping it out is
  a language-contract change, not a worker workaround.
- **Work:** implement same-session, lockfile-typed USE PACK resolution in the
  Sema transaction using Modules authority; preserve exact pack ID,
  activation mode, source location, and closed/open failure behavior. Then
  prove the actual source path reaches `NoPlan(MetaActivation)` through
  Pipeline/facade. Remove or privatize the dead relation-source constructor
  after checking public API consumers; do not create a second scan identity.
- **Accept:** positive source-text USE PACK + wrong/missing/foreign pack and
  lockfile mutations; no fallback to host/default authority; public API diff
  rejects raw reminting; Sema/Planning/Pipeline receipts refreshed. If the
  product intentionally excludes USE PACK, amend Ticket 00 and the example
  manifest explicitly before calling W4 closed.

### BG-05 — Close W4 proof and documentation, then publish Receipt 07

- **Purpose:** current package tests pass but the facade has zero owner tests,
  no external-consumer script exists, and rustdoc fails on deleted V1 links.
- **Owner/state:** W4 integrator after W4-G/H stop.
- **Work:** rewrite Planning/Pipeline `lib.rs` headers and public diagnostic
  private-module links to the V2 API; add `RUSTDOCFLAGS='-D warnings' cargo doc
  --workspace --no-deps` to the closeout lane. Review unresolved U-01
  catalog-edge total ordering and U-02 same-source-ID/same-length document
  transplant with discriminating fixtures; classify as defect or bounded
  non-goal on evidence, not inference. Re-freeze producer hashes after W4-G/H,
  apply only reserved manifest/API changes, resolve the duplicate contradictory
  W4 rows in `WAVE_EXECUTION_PLAN.md`, and generate Receipt 07.
- **Accept:** Planning/Pipeline/facade all-target tests + doc tests + strict
  Clippy + rustdoc + external-consumer gate; exact owner hashes; architecture
  transition deletion for W4-owned edges. Do not claim whole-workspace GREEN
  while W5/W6 imports are RED.

## P0 — W5 bounded front doors (Ticket 08)

### BG-06 — Rewrite CLI against the facade with bounded ingress

- **Purpose:** `cargo check` currently fails in `v3_query.rs`/`v3_plan.rs` on
  deleted Pipeline/Planning V1 names. `main.rs` still reads files unbounded and
  clones/joins argv before a byte check.
- **Owner/start:** CLI-only W5 lane, after facade DTO/diagnostic-order freeze.
- **Work:** make CLI normal-depend only on `frontend-contract`; query uses
  `analyze_source_document_v2`, fmt uses `format_source_document_v2`, and
  renderers consume the curated V2 DTOs. Stream file/pipe/special-file input
  with a `limit+1` cap; precompute argv UTF-8 bytes plus separators before
  allocation. Never echo rejected source bytes. Exit 0 only for complete
  semantic analysis/valid format; incomplete/error returns 1 with canonical
  ordered diagnostic code/span. Remove obsolete active LSP aliases/tests.
- **Accept:** CLI all-target tests and strict Clippy; exact/+1 file and argv
  cases, malformed UTF-8, errors, multi-item results, stable output/exit.

### BG-07 — Replace stdio framing/JSON with a total bounded NDJSON protocol

- **Purpose:** current service fails compilation on deleted Pipeline V1 API;
  `run_stdio_session_v1` grows an unbounded buffer and brace-count framing
  accepts glued objects. It is not JSON-RPC/LSP.
- **Owner/start:** diagnostics-stdio-only W5 lane, parallel with BG-06 after
  the same facade/budget freeze.
- **Work:** byte-oriented CRLF-aware NDJSON state with max frame/source bytes,
  discard-through-LF recovery, suffix preservation, EOF/pending-CR rules,
  one response per frame, process error latch, and short-write handling.
  Replace `JsonCursorV1`/manual emitters with one maintained parser plus
  duplicate-key visitor and a closed request schema
  `{id,method,source_id,source}`. Wire typed transport byte ranges separately
  from frontend source spans; bound diagnostic count and serialized response
  bytes before growth. Continue after a bad frame; transport errors set final
  process exit 1 while valid semantic incomplete/error remains response data.
- **Accept:** service all-target tests/strict Clippy; split-CRLF at every byte,
  exact/+1 for frame/source/diagnostics/response, duplicate/unknown field,
  integer/string ID range, invalid UTF-8, malformed+valid and oversize+valid
  recovery, one-byte reader, short writer, I/O failure, fresh-process reset.

### BG-08 — One frontdoor parity/inventory integration barrier

- **Purpose:** independent CLI and stdio successes would not prove the same
  semantic status/diagnostic order; old Cargo edges/docs remain misleading.
- **Owner/start:** W5 integrator after BG-06/07 stop.
- **Work:** implement `tools/test_frontdoor_parity.py` and one shared semantic
  corpus against facade, CLI, and stdio; compare exact item/status/diagnostic
  code, span, order, and exit/transport semantics. Remove CLI/stdio direct
  internal edges and the CLI dev edge to stdio; update README, language docs,
  method/command gates, and legacy-name inventory from the actual schema.
- **Accept:** parity script, Ticket 08 test command set, final LSP-name scan
  for active crates, architecture W5-edge deletion, Receipt 08 with one
  fingerprint. Do not advertise LSP/JSON-RPC methods.

## P1 by schedule — W6 conformance and policy (Ticket 09; mandatory for release)

### BG-09 — Replace the hardcoded V1 family table with one semantic manifest

- **Purpose:** `conformance/src/lib.rs` still imports deleted V1 plan enums and
  fails compilation. The 18 tracked examples have no authoritative
  `examples-v2.tsv`; a second Rust table would create dual truth.
- **Owner/start:** W6 semantic lane after facade freeze; full frontdoor rows
  wait for Receipt 08. Integrator first freezes conformance's facade-only
  manifest edge.
- **Work:** add one schema-validated TSV covering exactly the 18 tracked
  `.qglang` examples (authority, applicability, expected semantic payload or
  diagnostic/span, resource/mutation/owner receipt); keep Decl/HornRule in
  the separate two-row supplement. Generate family/exemplar consumers from
  that authority, then delete the V1 table. Add the pinned V2 tests:
  `v3_semantic_family_matrix`, `v3_registry_non_vacuity_v2`, and
  `v3_public_api_inventory`; run sealed facade paths, not internal crates.
- **Accept:** filesystem↔manifest exact equality, declared/selected/executed
  counts, positive payloads and negative exact diagnostics, no empty/ignored
  or label-only test, conformance all-target compile/test GREEN.

### BG-10 — Candidate-scoped recursive hygiene and generated truth

- **Purpose:** current hygiene misses nested artifacts and current-tree diff
  misses landed changes once the candidate is clean.
- **Owner/start:** W6 hygiene/generator lane may implement candidate-independent
  validators now; final outputs are integrator-only after source freeze.
- **Work:** derive paths from `git ls-files` and
  `PACKET_BASE_SHA..candidate` with adds/deletes/both sides of renames; scan
  nested docs/tools/crates; reject backup/repro/`_punchlist`, orphan crates,
  stale anchors, duplicate manifests, and unexplained dirty/untracked files.
  Generate `TRACEABILITY.md`, `TEST_PATH_INVENTORY.tsv`, public API/method
  allowlists, examples index, and closeout template from actual tracked inputs.
  Treat `test_repro_cases.txt`'s staged deletion as externally owned until
  explicitly reconciled; do not silently stage it.
- **Accept:** validators fail on deliberately injected nested/renamed/deleted
  counterexamples, all generated outputs are byte-stable and two-way complete,
  final candidate has an explained clean path inventory.

### BG-11 — Close static-policy debt at the proper owner

- **Purpose:** final Semgrep is RED (157 current findings: 151 string-result,
  six silent-discard), and 79 Rust allow attributes remain. A prior W6-C
  report observed 83 findings on an older tree; that count is stale after W4
  Planning changes. The new `clippy.toml` is untracked and changes global lint
  semantics if adopted.
- **Owner/start:** each source owner fixes its own production finding; static
  policy integrator owns rule scope, harness, Clippy configuration, Justfile,
  and CI. No cross-crate bulk fixer.
- **Work:** classify each Semgrep match by production/test-only and real
  behavior. Preserve typed, code-bearing errors on production paths; scope
  the string-result rule away from test-only helpers through a verified rule
  design, not 151 mechanical signature changes. Rewrite real bare `Err(_)`
  discards to explicit first-error-wins handling. Audit every allow attribute
  for removal or justified `#[expect(..., reason=...)]`; review the untracked
  `clippy.toml` before adopting it. Run Semgrep positive fixtures, clean
  control, and harness-detector tests; remove CI `-A` suppressions only after
  owner fixes and MSRV/current Clippy agree.
- **Accept:** zero blocking production findings under a non-vacuous seven-rule
  scan, all static-policy oracles live in CI, no unexplained allow attributes,
  strict workspace Clippy on both required toolchains. Record dispositions,
  not just a zero count.

### BG-12 — Wire final DAG, CI, docs, and deferred mutation proof

- **Purpose:** final architecture currently has 18 normal transitional edges,
  one transitional dev edge, and illegal `sema -> surface`; CI does not yet
  run all V2 black-box gates. W2/W3 receipts name deferred source-mutation
  inject/revert proof.
- **Owner/start:** W6 integrator after W4/W5 source cuts; owner-local deferred
  mutations run only from an authorized checkpoint in isolated worktrees.
- **Work:** remove each transition with its owning consumer, not by weakening
  the checker; cut illegal test-only Sema→Surface in the owning test seam.
  Wire external-consumer, frontdoor parity, conformance, recursive hygiene,
  Semgrep non-vacuity/harness, rustdoc `-D warnings`, final architecture, MSRV
  and current strict Clippy into `just ci` and hosted CI. Execute deferred
  mutation->RED->revert->GREEN per receipt on isolated worktrees and record
  exact candidate source hashes. Reconcile W4 public docs and the stale
  receipt banners without rewriting historical evidence.
- **Accept:** `check_architecture.py --require-final` GREEN with zero
  transitional/illegal edges; mandatory scripts actually invoked by both
  local recipe and hosted workflow; deferred mutation list dispositioned;
  Receipt 09 candidate-ready, not yet claimed complete.

## P0 release gate — W7 immutable candidate and external proof

### BG-13 — Two-phase closeout without self-invalidating receipts

- **Purpose:** package/owner GREEN on a moving dirty tree does not qualify the
  complete frontend or the Semantica consumer. A receipt written after testing
  changes the SHA it purports to attest.
- **Owner/start:** one integrator, only after BG-01..12 and explicit user
  authorization for commits. No parallel source writer during qualification.
- **Work:** commit candidate A with source/config/generated truth and an
  unclaimed receipt template; run Ticket 09's exact full command order on
  unchanged SHA A (format, Taplo, both strict Clippy toolchains, workspace
  nextest/docs, V2 conformance, black-box scripts, final DAG, deny, typos,
  Semgrep, `just ci`). Record exit, selected/executed counts, fingerprint, and
  any platform/tooling skip. Query hosted required jobs for exactly A and
  record URL, SHA, conclusion, timestamp. Then commit docs/receipt-only
  attestation B, prove `A..B` has no source/config/generated-truth change,
  and rerun claim/diff/format/doc/inventory validators. Semantica lock/QBC is
  separate consumer qualification and must cite its own exact fence.
- **Accept:** `tested_sha=A`, `attestation_sha=B`, no source drift during
  qualification, all required local gates GREEN. Hosted `failed` blocks;
  queued/running stays OPEN; unavailable permits only `REPO_LOCAL_CLOSED`,
  never hosted GREEN. No completion percentage or release claim substitutes
  for these receipts.

## Immediate execution order

1. Do **not** redispatch W4-G/H; collect their owner reports and hashes.
2. Integrator resolves BG-02 and BG-04 before declaring W4 closed; BG-03 and
   BG-01 integrate the already-running lanes.
3. Run BG-05's W4 barrier on a stable source fingerprint. Its output is the
   W5 dispatch fence, not a workspace release.
4. Dispatch BG-06 and BG-07 as disjoint W5 writers; integrate BG-08 serially.
5. Run BG-09 and BG-10 on disjoint paths; route BG-11 findings to owners, then
   integrate BG-12 serially.
6. Only after a clean candidate and approval, execute BG-13.
