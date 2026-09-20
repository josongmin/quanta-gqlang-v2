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

All twenty decisions in
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
  owner-local source/test fingerprints in `receipts/03.md` passed types 83/83
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
