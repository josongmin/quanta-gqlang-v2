# Sep-21 Frontend Reconstruction Execution Ledger

Frozen on `2026-09-21` before product edits.

## Source Freeze

- Base SHA: `e896a11024bd5a1e1ef03c094a70032f4e6c765f`
- Branch: `main`
- Upstream: `origin/main`
- Product-tree state at freeze: no tracked modifications
- Packet-owned untracked tree:
  `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/`
- External concurrent untracked tree, read-only for this packet:
  `docs/bugbash/sep-21-test-improvement/`
- External concurrent untracked audit, read-only for this packet:
  `docs/test-suite-audit.md`
- Integrator: `/root`
- Execution protocol: `ss`; checkpoint review protocol: `rr`
- Commit policy: implementation remains uncommitted unless the user explicitly
  authorizes commits. Receipts therefore identify the frozen base plus the
  scoped working-tree fingerprint until a commit SHA exists.

### Integration history after the original freeze

- Original audit and implementation base:
  `e896a11024bd5a1e1ef03c094a70032f4e6c765f`.
- First externally created integration commit:
  `96eab779bfce0a5178dbbd48e86bd457158db228`, subject `done`.
- `96eab779` contains the verified Ticket 01/02 stack and plan/bugbash material,
  but it is not a clean Ticket 01/02 boundary: it also contains Ticket 03-owned
  destructive type/module edits. Those edits were not covered by receipts
  01/02 and must be audited by receipt 03 against the original `e896a110...`
  source, not treated as trusted baseline behavior.
- Second externally created integration commit:
  `fcf106b56e794eb38f74c52b3bd2c1cf34f77fdc`, subject `done`.
- `fcf106b` is an atomic-red partial Ticket 03 snapshot: HIR was green, types did
  not compile, modules had no Ticket 03 source delta, formatting failed, and no
  receipt 03 existed. It is an integration state only, not a verified ticket
  boundary or releasable candidate.
- Preservation: no reset, amend, or history rewrite was performed.
- Ticket 03 work continued from `fcf106b` under `/root` for HIR/types and
  `/root/ticket03_modules_impl` for modules; later external commits absorbed
  that work before its final owner receipt was written.
- During final RR, three external commits advanced and pushed `main`/`origin`
  without a commit action by this execution owner:
  `ceb41425c8dc5caa3f524e2b86c7e0f21416e672` (`checkpoint frontend HIR and
  binder authority reconstruction`),
  `6583091cc5483f7e193c2113858dad8916585303` (`align algebra with closed
  relational HIR`), and
  `f9ec9543888954106bb7c8bf1dc6500a5e6d34f7` (`align planning with closed
  composition HIR`). No reset or history rewrite was performed.
- `ceb4142` absorbed the active Ticket 01-03/docs work together with binder
  consumer changes; `6583091` added algebra consumer changes; `f9ec954` added
  planning changes and more Ticket 03 tests. These commits are integration
  snapshots, not automatic verification of tickets 03-07. Their mixed scope is
  audited by receipt 03 plus separate consumer-delta review.
- After `f9ec954`, two module-only lint repairs remain dirty. The execution
  owner did not create or amend these external commits.
- Final Ticket 03 RR added two missing semantic rails (`sum(*)` rejection and
  absent-not-exported), expanded the fingerprint field matrix, and retained one
  earlier module lint repair. Its three-file product dirty fingerprint and all
  proof are recorded in `receipts/03.md`; Ticket 03 is owner-verified without a
  clean commit boundary.
- Independent audit classifies `ceb4142`, `6583091`, and `f9ec954` as
  `UNQUALIFIED PARTIAL CONSUMER MIGRATION`. Binder has 29 focused V1 tests
  green; algebra tests are red with 74 compiler errors across four suites;
  planning has 89 focused V1 tests green; pipeline check is red with 25
  compiler errors; strict binder/planning Clippy is red on 6 denied
  allow-attribute errors in algebra. Tickets 04, 05, and 07 contain the exact
  structural blockers and remain `PLANNED`.
- Evidence rule: receipts 01/02 retain their original frozen-source evidence;
  their schema backfills below do not extend that evidence. Receipt 03 must
  separately enumerate inherited contamination in `e896a110..96eab779` and
  active repair in `96eab779..final`, while comparing preserved algorithms and
  tests to `e896a110`.

The external bugbash tree and root test-suite audit may inform verification, but
no packet worker may edit, move, delete, stage, or claim ownership of them.

## Accepted Decisions

All thirty decisions (the original twenty plus owner/consumer audit addenda
21–30) in
[00 — Packet Scope And Pinned Decisions](tickets/00-packet-scope-and-pinned-decisions.md)
are accepted without replacement. In particular:

1. `quanta-gqlang-v2-sema` is the only new semantic owner.
2. The V2 cutover is breaking and atomic; no compatibility facade is allowed.
3. Closed compilation and open analysis remain distinct result types.
4. Ticket 01 mechanically renames the current private flat-JSON transport to
   `quanta-gqlang-v2-diagnostics-stdio`; ticket 08 rewrites framing to bounded
   UTF-8 NDJSON. This packet does not call it LSP or add JSON-RPC behavior.
5. Rule arity remains unary.
6. Numeric semantics remain `i128` plus finite `f64`.
7. Tickets 01-09 may not widen language, runtime, provider, SDK, optimizer, or
   physical-planning scope independently.

## Ownership And Dependency State

`write set` points to the authoritative enumerated section in each ticket; the
short scope is an additional collision check, not a substitute for that list.

| Ticket | Owner | State | Dependency state | Short write set |
|---|---|---|---|---|
| 00 | `/root` | `ACCEPTED` | base frozen | packet decisions and this ledger only |
| 01 | `/root/ticket01_impl` | `VERIFIED` on receipt fingerprint; post-receipt doc-gate delta open | 00 satisfied; execute later doc-gate change on integrated source before current-tree closure | workspace manifests/lock, empty sema crate, architecture/Semgrep/CI gates, mechanical `lsp` to `diagnostics-stdio` move, receipt 01 |
| 02 | `/root/ticket02_prep` | `VERIFIED` | RR re-review clean; owner-local proof green | surface and HIR owners/tests named in ticket 02, receipt 02 |
| 03 | `/root` + producer owners | `REPAIR` | prior owner proof preserved; Ticket 04 exposed new HIR/types/modules producer seams not covered by receipt 03 | HIR owned proof, types registry/declaration/rule solver, modules predeclaration/family refs, receipt 03 delta |
| 04 | `/root` + scoped owners | `RED_CAPTURED — IMPLEMENTING` | 02 verified; 03 reopened producer seams; Binder owner proof is focused only | sema, Binder, integrator-owned pipeline seam, legacy API inventory, receipt 04 |
| 05 | `/root/algebra_impl` | `QUEUED` | waits for 04 | algebra owner/tests plus integrator-owned pipeline seam, receipt 05 |
| 06 | `/root/logic_impl` | `QUEUED` | waits for 04 | logic owner/tests plus integrator-owned pipeline seam, receipt 06 |
| 07 | `/root` | `QUEUED` | waits for 05 and 06 | planning, final document assembly, curated facade, integrator seams, receipt 07 |
| 08 | `/root/frontdoors_impl` | `QUEUED` | waits for 07 | CLI, diagnostics stdio, legacy seam deletion, public docs, integrator manifests, receipt 08 |
| 09 | `/root` | `QUEUED` | waits for 08 | conformance, truth sync, transition deletion, final gates and closeout receipt |

## Integration Rules

1. `/root` is the only writer for root `Cargo.toml`, `Cargo.lock`, shared
   pipeline/facade manifests, and any file marked `integrator only` after
   ticket 01.
2. A worker owns only its ticket's enumerated write set. Cross-owner defects
   reopen the owning ticket; consumers do not patch around them.
3. Ticket 01 is serialized. Ticket 03 types RED preparation may overlap ticket
   02, but modules implementation follows ticket 02's shared HIR/location cut.
   Tickets 05 and 06 may implement in parallel only after ticket 04 passes its
   checkpoint. Tickets 02-08 are one unreleasable atomic cutover stack.
4. Broad Cargo verification is serialized by `/root`. Workers run only focused
   owner-local checks that do not compete for the shared target directory.
5. Every checkpoint records the diff scope, owner/consumer trace, duplicate
   owner scan, focused tests, failures, repairs, and residual risk in that
   ticket's receipt.
6. The repository does not provide the Semantica-specific persona/session
   scripts referenced by the generic `rr` workflow. Reviews therefore apply
   `rr`'s diff-first, live-source, sibling-universe, adversarial, and evidence
   rules directly and record the missing scripts as repository-inapplicable;
   no substitute authority file is invented.

## State Transitions

Allowed transitions are:

```text
QUEUED -> READ_ONLY_PREP -> PREP_REVIEWED -> ACTIVE -> REVIEW -> REPAIR -> VERIFIED
                                                      REVIEW ------------> VERIFIED
```

`VERIFIED` requires the ticket definition of done and a completed receipt.
Only ticket 09 may mark the packet `REPO_LOCAL_CLOSED` (hosted CI unavailable)
or `CLOSED_BY_09` (hosted CI green). A verified hosted failure blocks both.

## 2026-09-21 Implementation-Aware Plan Re-Audit

The final multi-angle audit found that the original 04-09 prose was not
executable against the active 02/03 cutover. The ticket-local corrective
sections are normative. Concrete repairs:

1. tickets 04-06 now stop at owner-only `FOCUSED_GREEN`; every deferred
   consumer compiler failure is recorded and ticket 07 is the first pipeline
   integration gate;
2. `SemanticItemIdV2` is the sole document item join key and all lowerer output
   sets are checked for exact equality before assembly;
3. types reuse `SourceQualifiedNodeIdV2`, `ResolvedCallTargetV2`, and
   `ResolvedPredicateIdV2`; the plan no longer invents parallel IDs;
4. open authority input/absence rules and fallible standard construction are
   explicit; ticket 07 provides a facade-only closed-authority builder;
5. correlated semi/anti lowering retains an explicit correlation set;
6. authority byte limits are documented honestly as per sealed admission
   session, not a global total implied by profile equality;
7. diagnostics stdio is pinned to UTF-8 NDJSON, the stale LSP paths are removed,
   and CLI/service parity moves to a dependency-neutral conformance harness;
8. ticket 09 migrates the proof ledger before deletion, separates closed/open
   matrices, and refuses closure without a clean committed candidate SHA.

No ticket 04 implementation starts until ticket 03's final RR and receipt are
complete. These plan corrections do not advance any implementation state.

## 2026-09-21 Ticket 03 Verification And Consumer Delta Audit

- Ticket 03 advanced `ACTIVE -> REVIEW -> REPAIR -> VERIFIED` on
  `HEAD=f9ec9543888954106bb7c8bf1dc6500a5e6d34f7` plus the exact three-file
  product follow-up fingerprint recorded in `receipts/03.md`.
- Final RR repaired the API sketch, authority-byte ownership wording,
  unsupported aggregate-Star and absent-export negatives, and field-by-field
  fingerprint mutation matrix. Owner proof is 69 HIR + 204 surface + 56 types
  + 109 modules tests, strict four-crate Clippy, architecture checker, and seven
  architecture unit tests.
- The deliberate aggregate-Star mutation failed its exact owner rail, was
  reverted, and the full owner proof returned green.
- Ticket 04 is now dependency-ready but remains `PLANNED`; no implementation
  was started in this audit turn.
- The inherited external consumer range `fcf106b..f9ec954` was audited
  separately. Binder/planning focused green does not close their V2 contracts;
  algebra and pipeline remain compiler-red as recorded above and in Tickets
  04/05/07.

## 2026-09-21 Ticket 04 Start

- Frozen start: `HEAD=main=origin/main=f9ec9543888954106bb7c8bf1dc6500a5e6d34f7`.
  Ticket 02/03 dirty owner follow-ups and packet docs are preserved and do not
  overlap Ticket 04 product owners.
- State: `PLANNED -> READY -> RED_CAPTURED`.
- RED evidence: sema selected zero tests; binder's 29 V1-focused tests passed
  but do not exercise incremental Join/projection frames or explicit EXISTS
  capture sets; `cargo check -p quanta-gqlang-v2-pipeline --all-targets` failed
  with 25 library errors plus 4 lib-test errors on retired module/type APIs,
  private HIR fields, removed Compose/Quantifier, and composition DTO mismatch.
- Integrator added only the four final-DAG path dependencies to the reserved
  sema manifest. Sema implementation, binder migration, and adversarial review
  proceed on disjoint write sets; pipeline remains deferred until Ticket 07.
- Sema implementation exposed a missing upstream carrier: later-stage aliases
  could not depend on an earlier expression fact without sema synthesizing a
  type or invoking the types kernel more than once. Ticket 03 temporarily
  reopens `VERIFIED -> REPAIR`; types owns the new
  `BindingFromExpression(SourceQualifiedNodeIdV2)` edge and fail-closed
  foreign/missing/self/forward/cycle validation. Ticket 04 consumes it and does
  not infer the type itself.

## 2026-09-21 Ticket 04 First Adversarial Checkpoint

- `HEAD=main=origin/main=f9ec9543888954106bb7c8bf1dc6500a5e6d34f7` throughout this
  checkpoint. Shared dirty product edits were preserved; no commit, reset, or
  cherry-pick was performed by this owner.
- Binder owner implemented sequential Join/projection frame visibility,
  default output names and duplicate rejection, explicit nested/empty EXISTS
  capture sets, exact `(owner HIR node, ordinal)` binding identity, and stable
  diagnostic order. Its focused owner proof is 43/43 tests plus strict Clippy.
  This does not prove sema or pipeline integration.
- HIR now has an owned checked-document proof (one document-wide integrity
  traversal, borrowed `as_checked` view). Types node-ID inputs now preserve
  the caller's extended host+document registry and a prior-expression binding
  dependency. HIR/types focused tests passed 70/59, respectively, before the
  additional declaration/rule producer work below. These counts are a
  checkpoint, not final Ticket 03 verification.
- Read-only RR found further P0 ownership gaps: forward `let` references need
  declaration-only lazy type dependencies while later relation aliases remain
  forbidden; source rules need a types-owned bounded singleton type proof;
  modules must predeclare IDs and validate every catalog family through one
  session; generic graph selectors need host-agnostic typed authority; the
  sema item/expr/relation DTOs must be lossless and source-reachable only.
  Source-less authority errors cannot be fabricated as document byte zero.
- Ticket 00/03/04/05/06 and the architecture owner map were amended with exact
  producer/consumer decisions, kind matrix, tests, and completion gates.
  `/root/ticket04_types_solver` owns the types producer repair;
  `/root/ticket04_rr_audit` was reassigned the modules producer repair;
  `/root/ticket04_sema_impl` owns the sema consumer. Their disjoint source
  write sets are active. Ticket 04 remains `RED_CAPTURED — IMPLEMENTING` and
  Ticket 03 remains `REPAIR` until fresh owner receipts exist.

## 2026-09-21 All-Ticket Final Reconciliation Checkpoint

- Exact reference is still `main@f9ec9543888954106bb7c8bf1dc6500a5e6d34f7`
  plus shared dirty edits. No implementation commit or history rewrite was
  made by the integrator. Per-ticket current-code/duplicate-owner/remaining
  work sections were added to Tickets 03-09; Tickets 00/04/05 and target
  architecture were corrected to the actual six source stages and 12 existing
  algebra operators. The invented `Unnest`/`Intersect`/`Except` work was
  explicitly removed.
- Live sema RR found a seven-stage relation draft and public arbitrary-output
  constructors; the owner narrowed the dirty draft to six stages with a
  checked Join subkind and crate-scoped stage constructors. It still rejects
  every nonempty document in `resolve_document_bodies`, so no Ticket 04 green
  is recorded. Exact catalog-ref/proof and item-ID ties remain open.
- Types' initial rule solver subset is 5/5 green and declaration-dependency
  subset 3/3 green; full solver matrix and final full owner receipt remain
  open. Modules' new predeclaration/family suite subset is 6/6 green; its
  full owner suite/strict Clippy and final receipt remain open. Subset outcomes
  are not promoted to Ticket 03 `VERIFIED`.
- `python3 tools/architecture/check_architecture.py` passed at this checkpoint:
  14 members, 43 normal edges (21 final, 22 declared transitional), one
  declared transitional dev edge. `git diff --check` passed. Neither is
  final-DAG, workspace, remote CI, or packet closure evidence.

## 2026-09-21 Second All-Ticket Live Audit

- HEAD remains `f9ec9543888954106bb7c8bf1dc6500a5e6d34f7`; the shared
  dirty overlay is moving as the disjoint HIR/types/modules/sema owners work.
  Tickets 01/02 retain their frozen owner receipts, 03 is `REPAIR`, 04 is
  `RED_CAPTURED — IMPLEMENTING`, and 05-09 are not started. No owner-local
  checkpoint is promoted to workspace integration.
- Ticket 03 types now has a declaration-only lazy dependency and finite-domain
  rule solver using the existing registries. Latest all-targets check passed;
  the earlier 67-test full suite predates new tests. The post-repair types
  suite passed 78/78 and strict Clippy passed on that checkpoint.
  Constructed Record/List now measures 6,884 steps over 74 candidate types
  after major charge-before-work repairs; canonical source budget usage,
  remaining fixed/input-map allocation, contextual negatives, and
  sema-owned host-signature
  sealing remain gates. Modules now has one-session
  predeclaration/resolution/probe, local/host/visible/external provenance
  collision rejects, and 122/122 owner-local tests plus strict Clippy on
  verified file hashes in `receipts/03.md`. This does not prove sema consumes
  all family references or make Ticket 03 verified.
- Ticket 04 sema has 8/8 focused `v3_document_transaction` tests, including
  forward value declarations and source-rule relational EXISTS rejection.
  A separate earlier 19-test sema all-target run passed, followed by further
  GraphQuery/Presentation source edits not covered by that result. Sema strict
  Clippy and full owner/family matrix have not passed. Binder's 43/43 and
  strict Clippy are separate owner proof, not document transaction proof.
  Later isolated GraphQuery, SHOW, Catalog/RuleBinding, CHECK, and Query/Source
  tests brought source-positive root reachability to 7/9; Rule and
  Composition plus non-Scan relation stages remain RED. The latest sema
  `cargo check --tests` passed, but no post-delta full suite/Clippy receipt
  exists.
- The shared Boolean parser permits relational EXISTS in a rule body. Ticket
  00 now pins an explicit semantic prohibition there, with exact-span source
  and direct-solver negatives; query EXISTS remains supported and must be
  typed/capturing. Tickets 03/04 contain the work. No grammar split or silent
  unsupported behavior is inferred.
- `python3 tools/architecture/check_architecture.py` passed again on this
  overlay: 14 members, 43 normal edges (21 final, 22 transitional), one
  transitional dev edge. This is transition-only evidence. The 05-09
  dependency and final-DAG gates in their tickets remain unchanged.
- Ticket 01 post-receipt doc-gate recheck attempted on the live tree and
  exited 101 before selecting the test: its pipeline dependency still has
  25 V1-consumer compiler errors. The new assertion remains unexecuted;
  Ticket 07's pipeline integration must precede the exact rerun. Final
  architecture mode independently exits 1 on the 22 normal and one dev
  declared transitional edges; transition-mode green is not final-DAG proof.

## Later Ticket 03-06 Contract Audit Delta

- Types' subsequent owner-local frozen source passed 79/79 all-target tests
  and strict Clippy. Shape-aware Record/List usage is 15 nodes, 20 edges,
  8,632 steps over 74 candidates; the earlier 78/78 and 6,884-step entries
  are historical. Receipt 03 records hashes. Ticket 03 remains `REPAIR`:
  source budget projection, catalog predicate identity, remaining allocation
  audit, and combined owner/consumer proof are open.
- Sema later passed 26/26 all-target owner tests before an additional FIND
  depth-negative edit. Its strict Clippy remains RED and Rule/Composition
  source producers are absent; no Ticket 04 `FOCUSED_GREEN` is recorded.
  Row-aligned output columns now carry an exact scan-stage origin and FIND
  carries checked item location plus budget caps. These producer shapes still
  require wrong-scan/foreign-location mutation proof.
- The read-only Ticket 05/06 handoff found that a bare `Vec<ResolvedRuleV2>`
  cannot prove a complete source program, and `ResolvedRuleV2::try_new`
  currently compares caller-assembled expected/actual call lists. Ticket 04
  must seal complete Rule membership and independent checked-HIR/types call
  equality. Ticket 06 must use iterative budgeted Tarjan, reject undeclared
  targets, and produce a canonical real edge/path cycle witness. Its current
  logic→algebra edge exists only for EXISTS Semi/Anti, now Ticket 05-owned.
  The target DAG was corrected to `logic -> sema + types`; the checker and
  actual Cargo edge remain unchanged until Ticket 06's recorded migration.
- A later focused sema document suite selected/executed 16/16 after a simple
  unary Rule source path and FIND depth-over-budget negative. Named root
  reachability is now 8/9; call-bearing Rule still rejects and Composition
  remains absent. The result is not a post-edit all-target or strict Clippy
  receipt, and Ticket 06 SCC source reachability is still RED.

## Later Ticket 07-09 Adversarial Audit Delta

- Ticket 07's initial planner sketch omitted Meta and Composition, lacked a
  concrete result/explain carrier, and left pipeline/Cargo.toml outside its
  edge-cut write set. Its cost labels also conflicted with live V1 fields.
  The ticket now pins exhaustive plan/nonplan dispatch, exact nested/inline
  identities, retained result/explain facts, and a four-field V2 success
  cost schema; the dead V1 unsupported field moves to typed diagnostics.
  No Ticket 07 code was executed or called green.
- Ticket 08's NDJSON CRLF limit needed one pending-CR state; inline argv and
  non-regular files needed preallocation byte caps. Live `Lsp` public symbols
  escaped the package-name-only scan. The ticket now names exact machine
  states and boundary cases. Ticket 09's CLI/service parity runs via a
  dependency-neutral script that builds/discovers both binaries, not a
  conformance Cargo test with unavailable dependency `CARGO_BIN_EXE_*`.
- Ticket 09 now accounts separately for 18 root families and Decl/HornRule
  supplemental units. Its resource map references owner receipts per
  dimension and runs representative cross-boundary cases instead of copying
  every limit case into every family row. These are queued acceptance
  requirements, not a completed conformance receipt.

## Latest Ticket 03/04 Owner Checkpoints

- Types superseded the earlier 79/79 checkpoint with 81/81 all-target tests
  and strict Clippy after iterative budgeted rule prepass, nearest lexical
  shadow verification, and types-owned unary source-callable projection.
  Hashes are in `receipts/03.md`. Host catalog predicate identity remains a
  separate types/modules producer change, not covered by this result.
- Sema superseded the earlier 30/30 and 34/34 checkpoints with 36/36
  all-target owner tests. Source-local self/mutual Rule calls now produce
  exact typed dependency edges; builtin calls stay outside SCC. Host calls,
  Query non-Scan stages, Inline VIEW, Composition, and strict Clippy remain
  open. Neither owner checkpoint is a Ticket 03/04 closeout or Ticket 06
  source SCC legality proof.

## Final Ticket Audit Delta — owner-local evidence and new semantic seams

- The preceding 81/81 types and 122/122 modules lines are historical. Latest
  owner-local source/test fingerprints in `receipts/03.md` passed types 85/85
  and modules 126/126 all-target tests plus strict Clippy. Distinct host key
  and same-session catalog proof are produced; sema host-call ingress has a
  source positive, but a combined Ticket 03 receipt is still open.
- Ticket 04's 36/36 full sema result predates later source work. A subsequent
  43/43 all-target run covered Filter/Project/inner Join/Sort and a
  modules-proven host Predicate call; strict Clippy exited 101 with 45
  lib-test errors. Binder GroupBy still leaks the pre-group frame;
  Composition local working-set authority is pinned but not implemented.
- Ticket 04 owns Binder key-only Group output and aggregate-only partition
  authority; Ticket 05 consumes that sealed contract. Ticket 00 pins both
  sort directions to NULLS LAST. Types owns host key, modules owns host
  provenance proof, sema owns call collision and SCC-edge selection. These
  splits prevent algebra, logic, or planning from adding parallel resolvers.

The 43/43 sema run's three-file hash subset and strict-Clippy RED are in
`receipts/04.md`. Group, Composition, Inline VIEW, relational EXISTS/graph
subqueries, and complete all-family transaction still block Ticket 04
focused green. The receipt is not a whole dirty-tree fingerprint.

After that 43/43 run, Binder and sema each passed two focused Group tests on
the key-only public frame and Set row, retaining the pre-group frame as a
separate partition input. Aggregate-only candidate authorization is still
RED; this focused 2+2 does not supersede all-target/Clippy evidence.

## External Shared-Main Advance During Final Audit

The working branch and remote-tracking `main` advanced externally from
`f9ec9543888954106bb7c8bf1dc6500a5e6d34f7` to
`ecc38f59e9fdcd62b5e030445188ffa8b01e100a` (`done`). Read-only
`git diff-tree` shows one mixed 64-path commit spanning Binder, HIR,
modules, sema, surface, types, packet docs, and receipts. It incorporated
many previously dirty owner and plan files. No root-agent commit, reset,
rebase, or history rewrite caused this advance. The remaining dirty overlay
at observation included sema `document.rs`, `lib.rs`, `relation.rs` and
packet status/receipt files. This commit is **not** a clean Ticket 03/04
qualification boundary: owner-local tests were run on specific earlier file
hashes, Ticket 04 strict Clippy was RED, and Tickets 05-09 remain open.
Re-freeze `ecc38f5` plus the current dirty fingerprint before any next
owner/integration receipt; do not retroactively label the mixed commit GREEN.

Fresh local `cargo check -p quanta-gqlang-v2-pipeline` on the
`ecc38f5`+dirty overlay exited 101 with 25 library compiler errors.
`compile.rs` and `lib.rs` still import removed V1 modules/types symbols,
call generic item compilation with closed Composition producer/body enums,
and read retired/private HIR variants/fields. This is the same structural
Ticket 07 consumer migration, not 25 independent fixes. Two external
registered-QBC reports separately observed 0 executed artifacts at the
pipeline compile boundary (one reported 30 all-target errors); those
external counts are not substituted for this local 25-error check.

Fresh architecture rerun on `ecc38f5`+dirty: seven checker unit tests passed;
`python3 tools/architecture/check_architecture.py --require-final` exited 1
with 22 transitional normal edges and one transitional dev edge remaining
(14 members, 43 normal edges). `git diff --check` exited 0. Transition lint
is not final-DAG proof and the mixed commit did not remove the exceptions.

Latest Ticket 04 owner-local supersession on `ecc38f5`+dirty: Binder 46/46
and sema 51/51 all-target tests passed; combined strict Clippy exited 101
with 21 sema lib and 43 sema lib-test errors. Exact six-file hash subset
is in `receipts/04.md`. Non-key Group aggregate source positive and
placement negatives are now present. Candidate forgery/outer-call
transplant, declaration/import collision, targeted Rule polarity oracle,
Composition, Inline VIEW, EXISTS and full family coverage remain open;
the owner result is not pipeline/logic/planning integration proof.

The next bounded owner snapshot superseded 46/46 + 51/51: Binder 47/47 and
strict Clippy pass; sema 52/52 passes, strict Clippy remains RED (21 library,
43 library-test errors). The exact same-scalar-call Rule polarity fixture
discriminates the old composed-effect defect, and declaration/import
partition-alias collision rejects at its authored span. `receipts/04.md`
records checked hashes. Candidate forgery/outer-call transplant and the
remaining Ticket 04 families/gates are still open.

## Ticket 02/04 Exact-Use Audit And Sema Lint Checkpoint

On `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` plus the shared
dirty overlay, the final read-only Composition/Group pass found a missing
parsed `FROM` name-span HIR carrier, absent Binder source-use resolution,
ambiguous node-only lookup for multi-use GraphRef, unproved independent HIR
ancestry for deferred Group candidates, and an unguarded Scalar aggregate
placement path. Tickets 02/04, the ticket index, reuse audit, final audit,
and receipt 04 now specify exact owner changes and mutation/source oracles.
No source-family status advanced from the document audit.

A faulty bulk test rewrite during sema Clippy cleanup was stopped and
recovered from exact HEAD plus chronological owner diffs. The restored
document-transaction test matched its pre-incident SHA-256
`4a6ea0b5f10d0c8702ff2f924d8ac848471a5cbc7c26ed50d90962d8574f3a51`;
the kernel test regained all 56 lines. Post-recovery sema all-target tests
passed 52/52 and strict all-target Clippy passed; root independently reran
both commands and `git diff --check`. Exact current seven-file SHA-256 values
and remaining narrowly scoped lint expectations are in `receipts/04.md`.
This closes the earlier sema lint failure only, not Ticket 04's semantic or
pipeline integration gates.

The integrator immediately reran `cargo check -p
quanta-gqlang-v2-pipeline --message-format short` on the post-lint dirty
overlay: exit 101, 25 library compile errors. The failures remain retired
V1 modules/types imports and closed HIR carrier mismatches in pipeline
`compile.rs`/`diagnostic.rs`/`lib.rs`; this is Ticket 07 consumer cutover,
not an owner-local sema regression or a successful integration check.

## Exact `FROM` Producer/Consumer Delta And Scalar Placement RED→GREEN

On the unchanged `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
with a moving shared dirty overlay, Ticket 02 added the parsed name-token
span to the one HIR Source carrier and its builder/integrity traversal.
The owner proof passed HIR 72/72 and surface 206/206 all-target tests plus
strict Clippy. Binder/Types, Sema, Algebra, Logic, and Planning test callers
were migrated to the breaking explicit-span builder; no compatibility shim
was added. Binder 52/52, Types 85/85, Sema 54/54, Planning 89/89, and Logic
49/49 owner suites passed on their respective dirty snapshots. Algebra's
four integration suites remain compiler-RED on retired HIR APIs; the
pipeline's V1 import/shape cutover is not qualified. The synthetic Binder
`catalog.*` fixtures' compressed spans are not a parser byte-span oracle.

Ticket 04 used the new span for exact Binder `FROM` local LET resolution,
wrong-kind local reject without host fallback, and graph multi-use lookup
keyed by node+span+spelling with duplicate-key rejection. Five new Binder
source-use tests passed. Separately, a new sema negative first demonstrated
that a standalone `sum(1)` declaration was accepted as a Scalar aggregate;
the central aggregate-context constructor now rejects it and nested OrderBy
aggregation at the inner call span. Sema's 54/54 and strict Clippy passed.
These are owner-local deltas only; Sema Composition still rejects, Group
candidate HIR-ancestry/tamper proof remains, and no Ticket 07 pipeline
qualification exists. Exact file hashes and command scopes are in receipts
02/04; `git diff --check` passed.

## Final Ticket Audit Checkpoint — `main@ecc38f59` + Dirty Overlay

No ticket state was advanced by documentation alone.

| Ticket | State after audit | Latest observed evidence | Next state-changing output |
|---|---|---|---|
| 00 | ACCEPTED | marker scan 0; diff check GREEN | decision amendment only if scope changes |
| 01 | historical VERIFIED; delta open | transition lint GREEN at 21/22/1 | post-receipt doc gate and later final-edge cleanup |
| 02 | behavior/lint checkpoint; format RED | HIR 72, Surface 206, strict Clippy GREEN; untracked name-span test omitted from receipt | inventory/hash/format repair, then frozen delta receipt |
| 03 | REPAIR | Types 86, Modules 126, strict Clippy GREEN on observed overlay; API inventory/test hashes stale | View-topology proof, owner mutation matrix, combined receipt |
| 04 | IMPLEMENTING | Binder 53; current Sema 59 and strict all-target Clippy GREEN; receipt 04 remains at the prior 56 fingerprint | Composition, Inline View, query EXISTS, nine-family transaction and refreshed receipt |
| 05 | PLANNED | algebra suites compile RED with 74 retired-HIR errors | sealed Sema-input lowering and receipt 05 |
| 06 | PLANNED | Logic 49 V1 tests GREEN | sealed Rule/Graph adapter, mutations, final edge change |
| 07 | PLANNED | Planning 89 V1 tests GREEN; Pipeline 25-error RED | exact assembler, pipeline/facade cutover, receipt 07 |
| 08 | PLANNED | V1 transport seams active | bounded NDJSON and facade-only executables, receipt 08 |
| 09 | PLANNED | receipts 05–08 absent; dirty tree | authorized clean candidate and complete final recipe |

The commands were run against the same HEAD but a shared dirty overlay. They
are local evidence, not an immutable integrated receipt. Every ticket now has
an exact final write sequence and stop condition; workers must use that final
section instead of inferring status from an older intermediate count.

Final gate addendum: `cargo fmt --all --check` exited 1 and listed 27 files
across Binder, HIR tests, Sema, and Surface tests. The audit did not mutate the
shared tree with a workspace-wide formatter. `--require-final` architecture
mode exited 1 on 22 transitional normal edges plus one transitional dev edge.
These are separate blockers from the passing owner tests/Clippy and the passing
transition-mode architecture lint.
## Final Implementation-Overlap Audit Checkpoint — 2026-09-21

Boundary: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` plus a moving,
shared dirty overlay. No commit, reset, rebase, staging change, or workspace-wide
format was performed by this audit.

| Ticket | State | Latest local evidence | Exact next state-changing output |
|---|---|---|---|
| 00 | ACCEPTED | frontend-only/owner boundaries unchanged | amendment only for an explicit scope decision |
| 01 | receipt-scoped VERIFIED; delta open | arch unit 7/7, transition GREEN, final RED 22+1, Taplo GREEN, Rustfmt RED 27 files | owner formatting plus eventual zero-exception final DAG |
| 02 | receipt-scoped VERIFIED; delta open | HIR 72, Surface 206; new name-span test untracked | track/format/inventory/hash delta receipt |
| 03 | REPAIR | Types 86, Modules 126, both strict Clippy GREEN | View topology phase, mutation matrix, combined frozen receipt |
| 04 | IMPLEMENTING | Binder 53, Sema 59, both strict Clippy GREEN | source-reachable Composition/Inline View/Query EXISTS and nine-root receipt |
| 05 | PLANNED | Algebra RED with 74 test compile errors | sealed Sema-input cutover, no HIR compatibility shim |
| 06 | PLANNED | Logic 49 direct tests GREEN | source document Rule/Graph adapter and receipt |
| 07 | PLANNED | Planning 89 direct tests GREEN; Pipeline RED 25/29 | exhaustive assembler, explicit facade, Pipeline cutover |
| 08 | PLANNED | blocked by Pipeline/facade | bounded NDJSON and three-front-door parity receipt |
| 09 | PLANNED | Conformance compile RED; receipts 05-08 absent | authorized clean candidate, manifest-driven full recipe |

New partial work in Tickets 03/04 must not be mistaken for closure. Types now
accepts deferred prior-relation rows and exact EXISTS captures, and Sema feeds
both in one kernel call. Sema still rejects the corresponding final semantic
forms. Ticket 04 therefore owns the consumer/materialization work; Ticket 03
owns only the remaining producer/topology proof and owner mutations.

Each ticket's final current-tree audit section is normative for the next worker.
Older counts and hashes remain evidence for their stated snapshots only.

## SS-03C / SS-04A Integrated Checkpoint — 2026-09-21

Boundary: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` plus the shared
dirty overlay. No commit, reset, rebase, staging change, or workspace-wide
format was performed. The staged deletion of root `test_repro_cases.txt`
remains externally owned and untouched.

| Owner | Result | Newly proved | Explicitly not proved |
|---|---:|---|---|
| Types | 87/87 + strict Clippy GREEN | specialized deferred rows plus Types-owned EXISTS base effects/order oracle | exact Binder capture-vector owner/ordinal/completeness; immutable receipt |
| Modules | 129/129 + strict Clippy GREEN | same-session topology phase/proof, dependency-first order, terminal phase misuse | Sema direct-edge production/materialization; diamond tie-break mutation |
| Binder | 53/53 + strict Clippy GREEN | exact Composition uses and no host fallback for relational source | graph/Sample local target capability closure |
| Sema | 64/64 + strict Clippy GREEN | Query LET chain, prior-row source, Query body, exact WITH, canonical incomplete analysis | non-Query Composition, Inline View, Query EXISTS, nine-root document |

Package-scoped rustfmt, the later global `cargo fmt --all --check`, and
`git diff --check` exited 0. Two deliberate
mutations were killed and reverted before recovery: reversing Modules'
dependency-first result, and forcing every Composition-local scan ordinal to
zero. These results are dirty-overlay checkpoints recorded in receipts 03/04;
they do not advance either ticket to complete.

RR corrected two overclaims in the earlier ledger. Types' EXISTS carrier is
not yet an independent proof of exact target ordinal or capture completeness,
and Sema's empty topology proof is not Inline View integration. The current
critical path is therefore:

```text
03 exact capture contract + topology permutation/public-API receipt
-> 04 private Composition identity + non-Query producers/bodies
      + Inline View topology consumption + Query EXISTS semantic registry
-> 05 -> 06 -> 07 -> 08 -> 09
```

Latest integration RED recheck on the same overlay: Algebra all-target
test compilation exits 101 with 74 retired-HIR errors; Pipeline library check
exits 101 with 25 V1/API-cutover errors; final architecture exits 1 with 22
normal plus one dev transition edge. Transition architecture remains GREEN.

## SS-04C Binder Inline View Dispatch — 2026-09-21

Binder now resolves only inline View queries through `resolve_catalog_decl`;
Sema and Pipeline dispatch catalog declarations to that API. Binder evidence is
54/54 all-target plus strict Clippy GREEN, focused inline-view 3/3 GREEN,
global rustfmt GREEN, and diff check GREEN. This closes the prior silent Binder
skip, but it is not a semantic Inline View completion: the Modules topology
proof and dependency-first Sema materialization cache remain P0 work under
Ticket 04.

## SS-03E / SS-04D Deferred-Row and Inline-Order Checkpoint — 2026-09-21

Types now has an 88/88 all-target, strict-Clippy-green owner checkpoint with a
positive exact-earlier `PriorRelation` row oracle and foreign-target mutation.
Global rustfmt and diff check are green. RR found that this does not make
Inline View ready: Modules topology may be reverse-authored, whereas Types
currently consumes deferred rows in HIR order. Ticket 04 must add one
Types-validated topology-derived pre-evaluation order before any Inline View
semantic materialization; relaxing the existing order guard is invalid.

## SS-04E Query EXISTS Registry Checkpoint — 2026-09-21

Sema 66/66 all-target and strict Clippy are GREEN after Query relational EXISTS
became source-reachable semantic IR. The sealed document now contains an exact
nested-relation registry and exact Binder correlation edges; correlated EXISTS,
uncorrelated NOT EXISTS, and Rule-body reject have direct source tests. Inline
View's topology-derived Types order remains the next P0; no downstream Algebra
or Pipeline cutover is authorized by this checkpoint.

## SS-03F / SS-04F Inline View Dependency-First Checkpoint — 2026-09-21

Types validates and prewalks a source/root/unique/dependency-ordered relation
plan before authored traversal; the legacy authored-order constructor remains
unchanged. RR repaired the normal-consumer case: an unplanned relation can use
an already prewalked plan target, but cannot use a non-plan forward target.

Sema now consumes the existing Modules topology proof rather than creating a
second graph. It probes catalog Views without terminalizing absent host-relation
names, sends direct View edges once, projects `dependency_first_order()` into
Types, materializes an asset-keyed cache in that order, stores direct catalog
uses as `family=None`, and finalizes the same validator/proof. Inline host,
reverse-authored local chain, direct-edge retention, cycle, and optional-probe
tests pass. Types 89/89, Modules 130/130, and Sema 69/69 all-target suites plus
strict Clippy are GREEN on the shared overlay.

This is not Ticket 03/04 closure. Required before downstream authorization:
topology diamond/repeated-use and cache-transplant mutations, Surface-to-HIR
integration, external typed View output authority, non-Query Composition,
remaining EXISTS captures/registry mutations, and a frozen receipt. Global
diff check is RED only for an externally owned trailing whitespace line in the
bugbash document; scoped changed-path diff check is GREEN.

## SS-04G Composition Identity Checkpoint — 2026-09-21

Sema now seals a Composition relation source in a private Binder-derived
carrier instead of exposing a remintable `(source_id, owner, ordinal)` variant.
The only transaction caller preserves the Binder-resolved local binding and
the carrier has no public constructor. Sema 70/70 all-target, 1/1 doctest,
strict Clippy, package fmt check, scoped diff check, and a zero-result raw
variant scan are GREEN. This does not close non-Query Composition, generated
rows, graph capability authority, or Ticket 04.

## SS-04H Generated-Row Contract Checkpoint — 2026-09-21

SAMPLE/PATH-row DTO construction now requires a producer-owned generated
column origin, pure effects, empty provenance, and exact nested item identity.
This removes the prior arbitrary-expression-key seam before the transaction
can enable a non-Query source path. Sema 70/70 plus 1/1 doctest, strict
Clippy, fmt, and scoped diff are GREEN. Graph local-target authority and
source reachability remain RED by design.

## SS-04L Composition Producer Source-Reachability Checkpoint — 2026-09-21

Sema now materializes `SAMPLE` as the canonical generated
`Set<{path: RepoPath}>` relation and PATH/REACHABILITY FIND as canonical
generated `Set<{function: EntityRef}>`; Flow and other non-row FIND forms
remain explicitly output-less. The one source suite proves body type input,
exact generated origin, noncontract SAMPLE-field rejection, PATH body use,
and local PATH-as-Entity graph use with no host selector fallback. Sema
all-target is 75/75 with strict Clippy, fmt, and scoped diff GREEN.

Remaining Ticket 04 P0: local SAMPLE Scope source-positive/mismatch rails,
Composition CHECK and EXPLAIN bodies routed through the shared Binder-first
resolver, same-name host-collision mutation, and fresh full transaction/e2e
evidence. No ticket status advances from this owner-local checkpoint.

## SS-04M Scope / EXPLAIN and CHECK Resolver Checkpoint — 2026-09-21

Sema source-reaches a prior SAMPLE as local Scope and a prior PATH LET as local
EXPLAIN Entity, with empty host graph authority in both oracles. Composition
CHECK's optional graph references now use the same Binder-first resolver, but
its Rule stays Modules-owned and has not yet received a Composition-specific
source-positive/mutation oracle. The Composition materializer preserves
source-less authority failures by returning the transaction error directly.
Sema all-target is 77/77; strict Clippy, fmt, and scoped diff are green.

Remaining Ticket 04 P0 narrows to CHECK Rule/local-capability mutation proof,
same-name host-collision rails, Surface-to-HIR-to-Sema composition e2e, and
the full transaction/frozen-candidate gates.

## SS-04N CHECK Rule/Scope/Nested-FIND Source Checkpoint — 2026-09-21

The Composition source suite now proves the cross-owner CHECK path as one
transaction: Modules RuleBinding identity, prior SAMPLE Scope, and prior PATH
Entity in nested semantic FIND. The adjacent PATH-as-Scope mutation rejects at
the exact authored span. Sema all-target is 80/80, with strict Clippy, fmt,
and scoped diff green. No synthetic Pattern producer was introduced.

Remaining Ticket 04 P0: same-name host-collision rails, Surface-to-HIR-to-Sema
Composition e2e, transaction-wide mutations, and frozen-candidate/whole-
workspace qualification.

## SS-04O Same-Name Host Collision Checkpoint — 2026-09-21

The CHECK mismatch fixture now admits host graph `scope` with the valid Scope
kind while a prior Composition PATH LET uses that same spelling. Binder-first
resolution sees the local target first, rejects its Entity-vs-Scope mismatch at
the authored use, and never falls through to host. Focused source suite 13/13
and strict focused Clippy are green. Ticket 04 still requires the Surface/HIR
front-door Composition e2e and whole-transaction qualification.

## SS-04P Composition Front-Door Checkpoint — 2026-09-21

Real public source now runs Surface parse -> checked HIR document -> Sema for
every permitted Composition body family, including a same-document `ruledef`
for CHECK. The new dependency is test-only; no runtime or provider substrate
enters Sema. The dedicated SAMPLE path proves exact producer/body row equality.
Sema all-target is 82/82 with strict Clippy, fmt, and scoped diff green.

Remaining Ticket 04 work: mixed nine-root transaction, remaining cross-session
and cache/capture mutation matrix, Inline View external-payload/diamond gates,
then frozen-candidate and whole-workspace qualification.

## SS-04Q Mixed All-Variant Transaction Checkpoint — 2026-09-21

The mixed-root source oracle is now green through real Surface -> HIR -> Sema
entry. It has ten source items but all nine semantic variants: the two Catalog
items are a RuleBinding required by CHECK and a certified Profile exercised by
Meta. The Rule has a genuine inference anchor (`x = 1`); this was established
by RED first, since literal heads reject structurally and unanchored symbolic
heads reject as ambiguous.

The final semantic-document assertion requires exactly one each of
Declaration, Rule, Relation, Graph, Check, Presentation, Composition, and
Meta, plus exactly two Catalog items. Sema all-target is 83/83, strict Clippy,
fmt, and scoped diff are green. Remaining Ticket 04 gates are now the
cross-session/cache/capture mutation matrix, Inline View external-payload and
diamond behavior, frozen candidate, and whole-workspace qualification.

## SS-04R External Typed View And Diamond Checkpoint — 2026-09-21

Sema now consumes Modules-owned external `View(RelationTypeV1)` payloads as
closed relation sources. The exact catalog asset, row and cardinality are
retained; a same-spelling host relation is neither consulted nor required.
The source is rejected only when it has neither a local inline body nor an
external typed payload.

The View topology oracle now includes a reverse-authored diamond. Sema
materializes it through the existing opaque Modules dependency-first proof and
asserts exact target assets plus four direct no-family edges. Sema all-target
is 85/85; strict Clippy, fmt, and scoped diff are green. Ticket 04 still needs
cross-session/cache/capture mutations, a frozen candidate, and
whole-workspace qualification.

## SS-04S Multi-Reference EXISTS Capture Checkpoint — 2026-09-21

The EXISTS registry now has a source oracle where one outer alias is captured
twice from a nested predicate. Both Binder-derived edges retain their own
reference node/span while sharing the exact outer owner/ordinal; a
binding-level summary cannot pass. The nested relation root and inner scan ID
are asserted too. Sema all-target is 86/86 with strict Clippy, fmt, and scoped
diff green. Nested multi-EXISTS and private cache/session mutation coverage
remain Ticket 04 work.

## SS-04T Nested EXISTS Boundary Checkpoint — 2026-09-21

A nested source EXISTS now verifies boundary-specific capture propagation and
two distinct sealed nested relation roots. The outer shape retains direct plus
transitive outer references; the inner shape retains only the transitive one.
Sema all-target is 87/87 with strict Clippy, fmt, and scoped diff green.
Private cache/session and forged-input mutation rails remain before frozen and
whole-workspace qualification.

## SS-04U Sealed Downstream Tags Checkpoint — 2026-09-21

Ticket 05 audit exposed three published HIR tag leaks from the Sema contract.
Sema now owns `SortDirectionV2`, `LogicalQuantifierKindV2`, and
`RelationExistsPolarityV2`, converting at materialization. This is the required
producer repair for an Algebra-to-Sema-only dependency edge. Sema all-target is
87/87 with strict Clippy, fmt, and scoped diff green. It is not Ticket 04 or
Ticket 05 closure.

## SS-05A / SS-07A Current-Tree Baseline — 2026-09-21

Algebra's 13 legacy integration tests are green but remain raw-HIR coverage;
they do not qualify Ticket 05. Workspace qualification is currently blocked
at Pipeline compilation: 25 errors from direct imports of deleted V1
Modules/Types APIs and retired HIR shapes. The only authorized repair is the
Ticket 07 single-Sema-document pipeline cut, after Ticket 05 exposes sealed
algebra lowering; import aliases are explicitly rejected.

## SS-08A / SS-09A Current-Tree Audit — 2026-09-21

Ticket 08 remains pre-implementation: direct V1 Pipeline consumers, unbounded
brace-framed flat JSON, and missing diagnostic/response caps prevent frontdoor
qualification. Ticket 09 remains pre-closeout: V1 conformance cannot compile
through Pipeline and must not create a second manifest. Semgrep non-vacuity is
implemented and retained; all packet closure evidence remains blocked behind
the sealed 05–08 path and an immutable candidate.

## SS-06A Current-Tree Audit — 2026-09-21

Logic has reusable algorithms but no Sema document adapter; raw HIR/name logic,
one-rule Pipeline assembly, independent graph/witness defaults, and a second
EXISTS-to-algebra producer remain. A Logic test/list attempt timed out before
enumeration, so historical `49/49` is not current qualification evidence.

## SS-04V Catalog Reference Session Retention — 2026-09-21

Sema now retains Modules' checked catalog-reference location instead of a raw
span. Sema all-target remains 87/87 with strict Clippy, fmt, and scoped diff
green. This is a narrow producer repair; no downstream closure is claimed.

## SS-05B / RR-05B Semantic-Document Algebra Cut — 2026-09-21

Algebra now has one production ingress, `lower_relation_item`, from a sealed
Sema document/item. Raw HIR/Binder inputs, caller source/aggregate options,
public forgeable V1 nodes, and their direct dependencies are deleted. The four
Algebra owner tests now consume feature-gated Sema reference transactions, so
the architecture audit no longer reports Algebra→HIR or Algebra→Modules dev
edges. Package all-target tests, strict Clippy, Sema all-feature lib Clippy,
format, and scoped diff are green.

RR found a real producer blocker: aggregate Projects still do not carry an
explicit/empty Group output contract. The lowerer rejects this shape rather
than guessing semantics. Ticket 04 reopens for that producer repair; Ticket 05
remains in progress. Global architecture stays RED only on the pre-existing
Sema→Surface dev edge.

## Final Parallel Ticket Audit And Current Proof Boundary — 2026-09-21

This entry supersedes older current-state counts while preserving their frozen
historical receipts. Final audit observation:

```text
HEAD: ecc38f59e9fdcd62b5e030445188ffa8b01e100a
branch: main
worktree: shared dirty overlay, 178 status entries
immutable candidate: none
```

The audit split Tickets 00–03, 04–06, and 07–09 into disjoint documentation
write sets, then reconciled their findings against current source. No product
source was edited by the final audit workers.

Current executed evidence:

| Gate | Result |
|---|---|
| HIR all-target | 72/72 GREEN |
| Surface all-target | 212/212 GREEN |
| Binder all-target | 54/54 GREEN |
| Types all-target | 89/89 GREEN |
| Modules all-target | 130/130 GREEN |
| Sema all-target | 87/87 GREEN |
| Algebra all-target | 4/4 GREEN |
| strict owner Clippy | HIR/Surface/Binder/Types/Modules/Sema/Algebra GREEN on their recorded current runs |
| `cargo fmt --all --check` | GREEN on current dirty bytes |
| `git diff --check` | GREEN |
| Pipeline library check | RED: Logic imports four deleted Algebra V1 symbols at `logic/src/source.rs:14` |
| architecture | RED: 20 transitional normal edges, one transitional dev edge, and illegal `sema -> surface` dev edge |
| Semgrep | non-vacuous but RED: 7 rules, 95 targets, 13 findings |

Ticket disposition after the audit:

1. 00 is accepted decision authority.
2. 01 is historical-receipt proof but current-tree architecture RED.
3. 02 is owner-green on the dirty overlay; receipt refresh/immutable proof is
   open.
4. 03 owners are green; exact EXISTS capture ordinal/completeness/order
   mutations, public-boundary inventory, and frozen combined receipt remain.
5. 04 has the complete source family/View/Composition/EXISTS transaction
   surface at the scoped owner level. It still must emit an explicit empty
   Group contract for implicit global aggregate Project, seal downstream budget
   receipt, and close the remaining transplant/public-boundary mutations.
6. 05 has one sealed Sema ingress, but aggregate semantics, EXISTS predicate
   removal after Semi/Anti lowering, traversal budget, and the operator/mutation
   matrix remain.
7. 06 is compile RED and has no current V2 Logic test receipt.
8. 07 is blocked on 06; the Planning/Pipeline/facade semantic-only cut is not
   implemented.
9. 08 is audit-only; bounded NDJSON, wire diagnostics, CLI bounds, parity, and
   facade-only dependencies are not implemented.
10. 09 is audit-only; manifest/conformance/inventory/final-DAG/exact-SHA hosted
    and two-phase attestation evidence are absent.

Receipts 05–09 now exist so every ticket has a place to record evidence.
Receipts 06–09 and the blocker portion of 07 are explicitly audit-only; file
existence is not implementation or qualification.

Mandatory next sequence:

```text
SS-03H cross-owner proof + SS-04W aggregate/budget producer repair
  -> SS-05C algebra semantic and budget matrix
  -> SS-06B sealed document Logic cut
  -> SS-07B Planning/Pipeline/facade cut from RR-07B
  -> SS-08B bounded frontdoors from RR-08B
  -> SS-09B immutable candidate and two-phase closeout from RR-09B
```

No ticket may add a V1 alias or local semantic fallback to bypass this order.

## Final Worker-Readiness Audit — 2026-09-21

The ticket packet was re-audited specifically for delegation to agents. Each
active ticket now exposes, in its final section or the dispatch cards:

1. exact current state and non-claims;
2. start gate and upstream dependency;
3. canonical owner and primary writable paths;
4. observable RED or missing proof;
5. ordered implementation obligations;
6. positive, negative, and deliberate-mutation oracles;
7. exact acceptance commands;
8. receipt path and downstream handoff;
9. duplicate-work and stop conditions;
10. allowed parallel work and integrator-reserved paths.

Corrections made by this audit:

- replaced stale current routing from SS-03F with next checkpoint SS-03H;
- added RR-02F so the owner-green HIR/Surface delta is not silently skipped;
- prohibited unrestricted parallel writes between 03H and 04W because both
  require the Sema transaction seam;
- reserved `sema/tests/v3_document_transaction.rs`, Receipts 03/04, packet
  indexes, and this ledger to the integrator during that parallel phase;
- added state banners to Receipts 05–09 so audit-only files cannot be mistaken
  for implementation receipts;
- distinguished audit-start dirty count 174 from final doc-packet count 178
  without rewriting source-bound hashes.

No product source was changed in this worker-readiness pass. A delegated agent
must copy the assignment manifest in `tickets/WORKER_START_HERE.md`, fill every
field, and re-freeze because the shared worktree may advance after this audit.

## Canonical Wave Scheduler Audit — 2026-09-21

`WAVE_EXECUTION_PLAN.md` now supersedes the coarse parallel-routing statements
above. This is a documentation/scheduling change only; no product source or
qualification state changed.

Corrections:

- Ticket 02 routes to its ticket-authorized `SS-02G`, not the stale `RR-02F`
  label. The required future `receipts/02-public-api-inventory.md` is an
  explicit new artifact and Receipt 02 remains integrator-owned.
- Ticket 03 and Ticket 04 do not receive independent Sema writers. One combined
  `SS-03H-S + SS-04W` lane owns `sema/src/document.rs`, the transaction test,
  capture/invocation proof, aggregate producer, and full budget authority.
- Types/Modules mutations and Surface/HIR inventory remain disjoint parallel
  lanes. Receipts, manifests, packet indexes, and this ledger are serial
  integration paths.
- Later safe splits are path-leased inside ordered consumer waves: Algebra
  source/tests; Logic rule/graph/adapter; Planning/Pipeline; CLI/stdio; and
  conformance/hygiene. Each wave ends in one integrator barrier.
- Lane-local evidence freezes declared input closures. Global wave/workspace/
  candidate proof still requires all source writers to stop on one unchanged
  fingerprint.
- Temporary source mutation/revert is prohibited in the shared dirty overlay;
  it requires an authorized checkpoint plus isolated worktrees. Adversarial
  data fixtures remain normal owner tests.

The current runnable entry is W0 followed by W1. W2-W7 remain blocked by their
recorded entrance gates. The current Logic compile, architecture, Semgrep, and
candidate states remain RED/open exactly as recorded in the final audit.

## W0 Freeze And Lease Issuance — 2026-09-21 (SS protocol execution start)

User authorization recorded in-session: run the full SS protocol W0-W7 in
parallel (max 3 workers + 1 integrator), checkpoint-commit semantically
meaningful changes to `main` as they qualify, and keep this ledger, receipts,
and ticket docs current at every transition.

### Freeze record

- `base_sha`: `ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
- `overlay_status_hash` (`git status --short | shasum -a 256`):
  `999a0db0875be0ea e01fd4151896248a7b5179eef6fc45eb465b874a156c620e`
  (179 entries; audit-count 178 overlay + 1 staged external deletion)
- Pre-existing dirty ownership stands as audited. The staged external deletion
  `test_repro_cases.txt` remains outside this packet: never restored, committed,
  or absorbed. All integrator commits use explicit pathspecs so it stays
  unstaged.

### Baseline RED and static state (evidence only, no repair in W0)

- First RED reproduced:
  `CARGO_TARGET_DIR=/tmp/qg-w0-red cargo check -p quanta-gqlang-v2-logic --all-targets`
  fails with `error[E0432]` at `crates/quanta-gqlang-v2-logic/src/source.rs:14`
  on unresolved `AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, SemiJoinV1`
  (lib + lib-test targets). Matches the recorded packet RED.
- Architecture `--require-final`: RED. 19 transitional normal edges (CLI x5,
  conformance x4, diagnostics-stdio x1, frontend-contract x5, logic x1,
  pipeline x3, planning x1), 1 transitional dev edge (`cli ->
  diagnostics-stdio`), 1 illegal dev internal edge (`sema -> surface`).
- Semgrep `--config .semgrep/qglang-frontend.yml` (v1.157.0): 13 findings,
  0 errors.
- Allow-attribute scan: `rg -o 'allow\(' crates | wc -l` = 98.

### Active lease table (W1)

| Lane | Work package | Allowed write paths | Pre-hash (sha256/16) |
|---|---|---|---|
| W1-A | SS-02G | new `receipts/02-public-api-inventory.md` only; product source read-only | (new file; HIR `lib.rs` a75f4a45d9771afd, Surface `lib.rs` 570d6db19b3541dd frozen as read-only inputs) |
| W1-B | SS-03H Types/Modules | `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs`, `crates/quanta-gqlang-v2-modules/tests/v3_document_predeclaration.rs` | `1eea493d2b633a74`, `d68a0f115973ae36` |
| W1-C | SS-03H-S + SS-04W (sole Sema writer) | `crates/quanta-gqlang-v2-sema/src/{authority,document,relation,diagnostic,lib}.rs`, `crates/quanta-gqlang-v2-sema/tests/{v3_document_transaction,v3_semantic_budget}.rs` | authority `77a96717473cbc8e`, document `bf58c71e2bed9d01`, relation `4d25d0bb53543ef1`, diagnostic `4c06e8425f9c3537`, lib `eb7265530b48ec09`, v3_document_transaction `b56cc69daf1aa312`, v3_semantic_budget `f71878fb7abb2978` |

Reserved to the integrator for all of W1: Receipts 02/03/04 final writes,
`sema/Cargo.toml`, any `crates/*/src/lib.rs` export change beyond a leased
file, root manifests/lockfile, lint/Justfile, architecture fixtures, packet
index docs, and this ledger. Workers may not run git state-mutating commands;
the integrator performs all checkpoint commits with explicit pathspecs.

### W0 barrier verdict

`GREEN`. Every W1 lane has exact allowed paths, reserved paths, frozen producer
inputs (HIR/Surface), and a named RED (W1-C inherits the recorded Sema
owner-oracle REDs from Tickets 03/04; W1-A is inventory-only with
`BLOCKED_TOOLING` fallback; W1-B's RED is the missing adversarial fixtures
themselves). W1 dispatch follows.

### W1 lease amendment 1 — 2026-09-21 (integrator-approved)

W1-C reported that SS-04W's mandatory `max_diagnostics` +
`max_response_bytes` on `FrontendBudgetV2::try_new` (19 -> 21 positional
args) breaks compilation at exactly one unleased call site:
`crates/quanta-gqlang-v2-sema/tests/v3_reference_resolution.rs`
(`profile_and_registry_mismatch_reject_before_document_work`). All other
callers use `standard_v2()`/`authority_profile()` and are source-compatible.

Amendment: W1-C's write lease is extended to exactly that single mechanical
arity update in that one test function — no other edit to that file. Any
further unleased compile break stops the lane for a new amendment request
instead of widening this one. The edit is mechanical fallout of the leased
budget-authority change, not new semantic surface; it will be verified at the
W1 barrier (unchanged-fingerprint scoped diff review).

### W1 lane completions — running log

- **Lane W2-C dispatched early (2026-09-21, integrator decision).** After W1-A
  completed, the wave's third worker slot was free. The optional W2-C Logic
  migration inventory (read-only, report-only, zero product writes, no build
  commands) was dispatched onto the idle W1-A agent rather than left queued
  behind the W1 barrier. Rationale: W2-C has no dependency on W1 output (it
  maps existing V1 Logic cases to KEEP/MIGRATE/DELETE against Ticket 06's
  already-frozen SS-06B sequence), cannot violate any lease, and its report
  is a required input for the W3 lane split. Worker concurrency stayed within
  the 3-worker cap (B, C, W2-C). Heavy builds were prohibited on this lane
  while W1-B/C qualification builds run.

- **W1-A / SS-02G COMPLETE** (first lane down). Artifact
  `receipts/02-public-api-inventory.md` sha256-16 `bd2be881b0ff0c13`.
  Manifest complete; live re-freeze recorded (HEAD `e9dd3e4`, status-list
  sha256 `2e0bf1fc13f3c285...`, 177 entries). Key evidence: sole 4-arg
  `relation_source` at `hir/src/build.rs:300-315`, sole production caller
  `surface/src/query.rs:79`, one `cargo public-api` item (HIR 4621 / Surface
  545 items listed), zero overloads/legacy constructors. Untracked name-span
  test `hir/tests/v3_relation_source_name_span.rs` is already
  Cargo-auto-enrolled and green (2/2) — integrator performs the `git add` at
  the barrier checkpoint; no manifest change required. Deferred:
  `cargo semver-checks` (rustc 1.81 < 1.93), two-sided public-api diff vs
  `base_sha` (needs a baseline worktree, permitted only after a checkpoint
  commit exists). Owner-call flag recorded for Ticket 02: pub fields on
  `RelationSourceV1` permit test struct-literal construction (not an
  overload); field-narrowing remains a Ticket 02 owner decision.

- **W1-B / SS-03H Types/Modules COMPLETE** (second lane down; zero production
  defects found). Post-hashes: `types/tests/v3_types_check_edges.rs`
  `50e1049b058f9d8f...`, `modules/tests/v3_document_predeclaration.rs`
  `fc3120659f2ed11d...` (full sha256 in lane report). RED proven pre-edit
  (targeted filters select 0; baselines types 89/89, modules 130/130).
  Acceptance on one fingerprint (isolated `/tmp/qg-w1-laneB`; frozen inputs
  types/src `406ad1f32d9efd80`, modules/src `a1b8f817fec118ef`, hir/src
  `31a4efee67045ae0` verified unchanged across every command): types 91/91,
  modules 132/132, strict Clippy exit 0, scoped fmt exit 0, `git diff --check`
  0. Four semantic-payload fixtures: three-producer pre-evaluation
  target-following (plan/authored/dependency orders all distinct, swapped
  targets assert new exact rows, plan transplant rejects with exact
  root/target/span), chain-transplant fail-closed pair, permutation-stable
  topology diamond (16 permutations, identical canonical order, duplicate-edge
  idempotence with min-location retention), and two-cycle located-witness
  canonicalization (exact primary + witness spans under all permutations).
  Oracle non-vacuity via four test-local mutations killed then restored
  byte-exact; deliberate-source-mutation proof remains deferred to a
  checkpoint+worktree lane per wave plan. Public-API inventory draft verdict:
  no public constructor in either crate can mint a topology proof, checked
  authority, typed fact, or pre-evaluation result. SS-03G remaining items 2/3
  now covered by owner tests; 1/4 sit with the Sema lane, 5/6 with the
  integrator. Flagged discrepancies (live HEAD `e9dd3e4`, overlay hash
  `648a985a91e00d20` vs dispatch values) are concurrent-lane drift; leased
  inputs never changed.

- **Lane W2-C COMPLETE** (read-only Logic migration inventory; zero writes,
  message-only deliverable preserved verbatim by the integrator as
  `tickets/06-migration-inventory-w2c.md`). Key results: SS-06B RED verified
  statically; all Sema migration targets present (no W3 producer gap);
  static census 44+1 supersedes historical 48/49; production dispositions
  17 KEEP / 22 MIGRATE / 21 DELETE (source.rs + aggregate.rs wholesale);
  test dispositions 14/15/15 (+4 policy flips: PointsTo-without-depth and
  via-without-depth become rejects; both depth+waypoint conflict rails become
  positives); W3 lane split adopted; 12 coverage gaps routed (largest: G3
  membership-forgery class with zero V1 coverage, G11 source-level document
  fixtures, G9 every-spelling aggregate rejection). Two integrator decisions
  adopted and recorded in the sealed file: (1) pre-cut `diagnostic.rs` at W3
  barrier start before A/B test freezes; (2) amend Ticket 06's
  `.negated_recursion` fixture wording (G12) instead of resurrecting a dead
  diagnostic arm.

## W1 Barrier Record — GREEN — 2026-09-21

```text
wave=W1
entry_sha=29d65356b37b6e16c65fe2d8dd9603279cfe78a3 (docs-only commits above product base ecc38f59)
entry_status_hash=2e0bf1fc13f3c285e93e77eb235ac7e3dcc2d94fe714216982969d24a4175747 (177 entries)
lane_ids=W1-A/SS-02G, W1-B/SS-03H, W1-C/SS-03H-S+SS-04W (+ early W2-C inventory on the recycled W1-A slot)
leased_paths_and_pre_hashes=as issued at W0; all post-hashes verified by integrator against the live tree (10/10 match)
producer_contract_hashes=HIR/Surface frozen per W1-A artifact (23-file manifest); Types/Modules/Binder frozen per W1-B/C manifests; unchanged across every lane command
lane_red_evidence=W1-A: read-only (no RED). W1-B: fixture-absence filters select 0 (28/20 baselines). W1-C: baseline 87/87 + zero-match grep for budget_receipt/max_diagnostics/max_response_bytes/sealed aggregate constructors
lane_acceptance_commands_and_counts=q02 HIR+Surface all targets ok / Clippy 0; q03 binder 48 + types 91 + modules 132 all ok / Clippy 0; q04 sema 102/102 / Clippy 0; scoped fmt 0; git diff --check 0; entry==exit fingerprint
mutation_ids_killed_and_reverted=W1-B: 4 test-local mutations (row-target, FIFO order, wrong cycle witness, wrong transplant span) killed, restored byte-exact. W1-C: 8-mutation capture forgery matrix, forged aggregate matrix, receipt identity/value mutations as cfg(test) oracles; deliberate source-mutation builds deferred (shared-overlay rule) with kill map recorded in Receipt 04
reserved_path_changes_applied_by_integrator=amendment 1 (v3_reference_resolution.rs single arity edit) executed by the lane under grant; verified via report-quoted before/after + post-hash match; no other reserved path changed
exit_sha=see W1 checkpoint commit (this record precedes it)
exit_status_hash=same as entry at qualification time
receipt_paths=receipts/02.md (SS-02G refresh section), receipts/03.md (W1 closure), receipts/04.md (SS-04W owner receipt) + receipts/02-public-api-inventory.md artifact
deferred_failures=semver-checks BLOCKED_TOOLING (rustc 1.81); accumulation-side diagnostic/response charging exact/+1 owned by Tickets 07/08; source-mutation kill builds need authorized worktree (now possible: checkpoint commit exists)
next_wave_gate=W2 OPEN: aggregate-ready contract + 21-dimension budget receipt frozen (see Receipt 04 Downstream freeze); W2-A/B leases issued next; algebra-crate overlay freeze via /tmp/qg-freeze-crates.sh before dispatch
```

W1 verdict: `GREEN`. All three lanes and the serial barrier passed on one
unchanged fingerprint. Wave status table updated; W2 dispatch follows.

### W2 dispatch — 2026-09-21

Entrance gate met (W1 GREEN, receipts closed, checkpoint `1a9ccad`). Algebra
crate fully frozen per the W2+ process fix (`/tmp/qg-freeze-w2.txt`: Cargo.toml
`808a3d12`, diagnostic `00f65f93`, lib `933d1f0f`, node `ed84e886`, normalize
`50af5871`, four integration tests `3d423bd9`/`016e036a`/`1ab2bdd1`/`40833b9a`,
RETIRED_LAWS `e2c7e6d9`). Frozen Sema producer hashes published to both lanes
(authority/document/relation/lib as in the W1 barrier record). Lanes: W2-A
(edc7dbfd, lowering implementation, lease src/{normalize,node,diagnostic}.rs)
and W2-B (239cfd20, adversarial matrix, lease tests/*.rs) with the hard start
gate: W2-B designs/inventories only until W2-A announces constructor/
test-support freeze. W2-C inventory was completed early and sealed
(`tickets/06-migration-inventory-w2c.md`), so the third slot stays free for
integrator use or W2 escalation. Cargo.lock remains staged-but-uncommitted by
design; it settles at the wave whose manifest edges finalize it.

### W2 lane D dispatch + gate coordination — 2026-09-21

W2-B's pre-gate report proved the SS-05C owner-test matrix is inexpressible
with the current 3-variant sema test-support fixture set, and that forged
mutation rails are mintable only from inside the sema crate
(`SemanticDocumentV2::try_assemble` is `pub(crate)`; that unforgeability is a
W1 freeze and stays). Integrator decisions:

- **Lane W2-D dispatched** (third slot, previously freed by the early W2-C
  completion): exclusive lease `sema/src/test_support.rs`, additive-only,
  everything behind the existing `test` feature + `#[doc(hidden)]`. Adds the
  positive fixture variants W2-B designed (Project multi-expr, InnerJoin+ON
  incl. self-join, Group->Aggregate grouped sum, scalar count(*), Sort family
  incl. NULLS LAST on a new nullable field, EXISTS family x6, deep relation/
  Bool chains, second-source-id), a small-budget authority (tiny
  max_semantic_steps / max_expression_nodes for exact/+1 rails), and
  `forged_*` DTO builders for the algebra mutation rails. Constraint: no
  weakening of W1 seals — forged shapes exist only under the test feature;
  any pub(crate) seam gap inside the frozen producer files comes back as an
  amendment request, not an edit.
- **EXISTS residual contract pinned by integrator** (per RR-05C): single
  residual Filter holding the full ordinary conjunction when ordinary
  conjuncts exist; no Filter for pure EXISTS; EXISTS leaves lower exactly
  once in stable left-to-right order.
- W2-B gate stays CLOSED with the 41-fixture design accepted; it opens when
  W2-A's freeze announcement (now required to include charged budget
  dimensions + over-budget diagnostic, Apply/Limit/Union disposition, and
  the private constructor surface) is relayed together with W2-D's landed
  builder surface.

### W2 incident and restart record — 2026-09-21

- Lanes W2-A and W2-D failed mid-flight before any announcement. Tree audit at
  restart: W2-A left zero writes (all entry-freeze hashes intact). W2-D left a
  partial additive edit to its leased `sema/src/test_support.rs`
  (Exists/NotExists fixture variants, verified well-formed) — kept as the
  restart baseline.
- UNEXPLAINED MUTATIONS found and handled: (1) worktree deletion of
  `logic/tests/v3_logic_source.rs` (outside every active lease; no lane
  reported it) — RESTORED from index (`git checkout --`, sha256-16
  `d7bf2b602168b5d4`); (2) `Cargo.lock` gained unstaged worktree changes on
  top of the intentionally staged W1-era copy (some W2-era cargo run touched
  it) — left as-is, integrator-reserved, settles at the wave that finalizes
  manifest edges. If either recurs, the owning lane's qualification is
  invalid per the Failure And Reopen rules.
- Both lanes re-dispatched fresh (W2-A bb90b3d2 with the full original
  mission + freeze-announcement duty; W2-D 6b7dcecc instructed to verify and
  continue from the partial edit). W2-B remains correctly gated.

### W2 self-audit incident 2 — overlay file loss — 2026-09-21

The new barrier self-audit (`/tmp/qg-selfaudit.sh`: frozen-input hashes,
unauthorized-mutation scan, staging check, protected-file check) caught an
out-of-lease rewrite of `algebra/tests/v3_algebra_normalization.rs`
(freeze `1ab2bdd19d6a840e` -> a 58-line V2 smoke test; author under
investigation — failed-lane leftover or an active lane's boundary error;
both W2-A and W2-B were asked to claim or deny).

Integrator error during restoration: `git checkout -- <path>` restores the
INDEX (= HEAD) version, not the overlay worktree state, because overlay
files were never staged. The file's overlay version (the W2 freeze content)
is therefore LOST on disk. Impact is bounded: per lane W2-B's verified
inventory the file held exactly one test (Filter positive: predicate + child
Scan + output preservation) and its three sibling files are intact at their
freeze hashes (`3d423bd9`/`016e036a`/`40833b9a`), so the pattern is fully
recoverable. Recovery: lane W2-B reconstructs the original as its first
post-gate write (its design already keeps the four originals), and the
integrator verifies the reconstruction against the inventory description.

Process fix (effective now): wave freezes copy frozen files into a content
vault (`/tmp/qg-freeze-vault/<tag>/`) in addition to hashing, so any future
restoration has a true source. The current W2 vault is created from the
still-intact freeze set plus the HEAD fallback for the lost file.

Cargo.lock staging was cleaned (`git restore --staged`); its worktree
changes remain integrator-owned and unsettle until a manifest-finalizing
wave.

### W2 freeze announcement sealed + gate opened (phased) — 2026-09-21

W2-A delivered the freeze announcement (binding for W2-B): budget charging =
iterative Visit/Finish relation walk, one max_semantic_steps charge per stage
before visit; two iterative expression walks (And decomposition + descendant
search) sharing one max_expression_nodes ledger charged before every node;
source is SemanticDocumentV2::budget_receipt() only; over-budget diagnostic =
algebra.traversal_budget_exceeded naming the dimension at the refused stage's
checked location. Apply/Limit/Union OMITTED from the completion claim (no
landed Ticket 07 consumer); variants stay unconstructed. Private surface:
Filter payload = full ordinary conjunction (Vec<TypedBoolExprV2>, one Filter
max, zero for pure EXISTS, Semi/Anti carry exact outer output, residual
Filter above the Semi/Anti chain); predicate() is Join-ON-only with
filter_conjuncts() on Filter; CheckedAlgebraNodeV2::new fail-closes on
op<->output disagreement per operator family.

Mystery edit resolved: the running W2-A denies writing under tests/ (writes
verified within src lease); the 58-line test version predates the restart —
it was the FAILED original W2-A's unreported leftover, and it was green at
restart entry. The integrator's checkout then replaced it with the
non-compiling pre-SS-05B V1 suite (3320cb3c = HEAD). Recovery authorized:
lane W2-B rewrites v3_algebra_normalization.rs as its first phase-1 act
(original Filter positive per its inventory + the two EXISTS-shape tests
migrated to the pinned contract). W2-B gate OPENED PHASED: phase 1 =
reconstruction + rails not needing W2-D; phase 2 = forged/new-fixture rails
after W2-D lands. W2-D is mid-flight on test_support (file at d9c93d12,
temporarily non-compiling — W2-A's full acceptance rerun waits for it; W2-A
may scope interim evidence to --lib + deferred entry, barrier runs
all-targets on the final tree).
