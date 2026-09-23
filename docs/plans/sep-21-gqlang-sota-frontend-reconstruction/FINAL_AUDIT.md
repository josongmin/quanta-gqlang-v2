# Final Current-Head Audit

Status: `HISTORICAL 2026-09-21 FINDING LEDGER; CURRENT ROUTING IN README/TICKET 09`

Current override (2026-09-24): Ticket 09 maps F01–F37 to present owner rails
and intentional scope exclusions; clean source
`08e5ce5ab05e8af141985d3b5f54bbff4a67e97f` passed local exact-SHA
qualification (Receipt 09). The original finding rows below describe the
original audit base, not current open defects. Semantica consumer proof is
separate. The following 2026-09-23 override is historical.

Historical override (2026-09-23): `main@d9d0f28` integrated W1-W6 producer
code, including F04/F26 repairs and the F08 owner oracle. The pre-commit
source/test/tool gate passed 1,168/1,168; clean current-SHA qualification and
Ticket 09 semantic coverage accounting remain open. The F01-F37 rows below
describe failures at their original audit source pin, not automatically open
bugs at the current source. Use Ticket 09's current-source disposition before
authorizing a code edit; another writer's in-flight owner tests are not part
of this baseline.

## 2026-09-21 Current-State Supersession

The current coordinated freeze is
`main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` plus a 178-entry shared dirty
overlay. It is not an immutable candidate. All older line numbers, counts, and
full-workspace receipts in this document describe their named historical
snapshot only.

Resolved or materially advanced since the original audit:

- a document-scoped Sema owner exists and its latest scoped suite is `87/87`;
- Algebra production ingress is now the sealed Sema document/item only;
- direct Algebra dependencies on HIR/Binder and public forgeable Algebra V1
  nodes were removed;
- the architecture audit no longer reports Algebra illegal edges;
- Query EXISTS, nested/multiple correlations, inline/external View, all root
  families, and catalog-session retention have scoped producer coverage.

Current P0 integration blockers:

1. An implicit global aggregate Project does not emit the explicit
   Group/empty-partition contract required by Algebra. Algebra correctly
   rejects rather than infers.
2. The sealed document does not yet carry the downstream budget receipt.
   Algebra currently accepts no canonical budget, recursively lowers deep
   trees, and leaves the original EXISTS-bearing Filter predicate after adding
   Semi/Anti, duplicating that semantic condition.
3. Logic still imports deleted Algebra V1 node/operator types; therefore
   Pipeline library compilation is RED and all downstream workspace claims are
   invalid.
4. Logic still owns raw-HIR/name adapters, one-rule assembly, a duplicate
   EXISTS lowerer, recursive DFS Tarjan without the frontend budget, and graph
   defaults that are not derived from the sealed document.
5. Planning/Pipeline/facade have not completed the semantic-only cut. Recursive
   planner walks, optional-plan invalid states, an undefined authority
   fingerprint, and incomplete facade export enforcement remain.
6. Frontdoors still lack a closed bounded wire-diagnostic model, response and
   diagnostic caps, exact transport/session exit policy, and bounded argv
   handling.
7. No single authoritative semantic manifest, base-to-head inventory,
   external-consumer final gate, exact-SHA hosted receipt, or clean immutable
   candidate exists.

Current P1 producer/authority risks that remain after the latest repairs:

- Sema already compares each EXISTS witness against Binder's exact
  `(reference, owner, ordinal)` before Types input construction; the remaining
  gap is a discriminating mutation suite for dropped/duplicate/wrong-valid-
  owner/wrong-ordinal captures and a reversed-node-ID oracle. Do not duplicate
  Binder authority inside Types;
- the public `ResolvedRelationSourceV2` occurrence wrapper and the
  `ResolvedScanSourceV2` identity alias remain overlapping contracts; converge
  them or explicitly make the occurrence proof document-only;
- add foreign-registry/session and sibling-valid-owner/ordinal transplants that
  discriminate the sealed identities rather than only malformed identities;
- add external compile-fail proof that private Composition identities and
  invalid closed producer/body payloads cannot be minted.

The following earlier findings are now implemented and must not be rebuilt:
private Composition source identity, disjoint host/local graph targets,
generated SAMPLE/FIND origins, checked catalog locations, dependency-first
Inline View topology, nested Query EXISTS registry, and canonical incomplete
analysis diagnostics.

Current proof boundary:

| Scope | Result |
|---|---|
| Types / Modules / Binder owners | `89/89`, `130/130`, `54/54` GREEN; strict Clippy GREEN |
| Sema owner | `87/87` GREEN; strict Clippy GREEN |
| Algebra owner | `4/4` GREEN; strict Clippy GREEN |
| format / diff | `cargo fmt --all --check` and `git diff --check` GREEN |
| architecture | RED with 20 normal transitions, one dev transition, and illegal `sema -> surface` dev edge |
| Pipeline | RED at Logic imports of deleted Algebra V1 API |
| workspace / conformance / frontdoors / hosted CI | unqualified |

No older GREEN row may be used to upgrade one of the unqualified rows above.

Latest shared HEAD observed during execution: `ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
(`main=origin/main`), one mixed external commit after the scoped `f9ec954`
audit. The remaining dirty sema/docs overlay is not covered by that commit.
See `EXECUTION_LEDGER.md`; none of the historical owner receipts is
retroactively upgraded to integrated green.

Base: `main@e896a11024bd5a1e1ef03c094a70032f4e6c765f`

Worktree at audit start: clean.

## Fresh Local Receipt

Executed on `2026-09-21`:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace --no-tests pass` | PASS, 822/822 |
| `cargo test --workspace --doc` | PASS, 0 doctests |
| `taplo fmt --check` | FAIL, 6 manifests |
| `python3 tools/architecture/check_architecture.py` | FAIL, stale deleted-crate topology |
| `cargo deny check bans licenses sources advisories` | FAIL, 6 wildcard path dependencies |
| `typos .` | FAIL, 9 findings |
| `tools/lint/run_semgrep.sh` | exit 0, but 6 of 7 declared rules ran |
| `just ci` | FAIL at Taplo |

This receipt proves that the Rust test suite is broad but not sufficient to
establish semantic or repository closure.

Remote Actions state was not refreshed in this audit. No remote-green claim is
made.

## Current-Source Anchors

These are the canonical producer/consumer seams used by the audit. Line numbers
refer to the frozen base SHA.

1. JOIN is encoded as an outer filter over string-tagged composition:
   [surface/query.rs:167](../../../crates/quanta-gqlang-v2-surface/src/query.rs#L167),
   while normalization expects a different predicate location:
   [algebra/normalize.rs:425](../../../crates/quanta-gqlang-v2-algebra/src/normalize.rs#L425).
2. Predicate checking walks values without enforcing the Bool/operator contract:
   [pipeline/compile.rs:1132](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L1132),
   and whole expressions may be skipped at
   [pipeline/compile.rs:1171](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L1171).
3. Relation HIR uses open string operators:
   [hir/relation.rs:94](../../../crates/quanta-gqlang-v2-hir/src/relation.rs#L94).
4. Document compilation mutates and compiles one item at a time:
   [pipeline/compile.rs:557](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L557).
5. Horn rules bypass Binder and lower independently:
   [pipeline/compile.rs:655](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L655),
   [pipeline/compile.rs:1266](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L1266).
6. Rule call lowering ignores the callee and walks arguments:
   [pipeline/compile.rs:1337](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L1337).
7. Planning hardcodes relational truth classification:
   [pipeline/compile.rs:615](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L615),
   even though `now` is declared nondeterministic:
   [types/builtin.rs:359](../../../crates/quanta-gqlang-v2-types/src/builtin.rs#L359).
8. Raw HIR is the public planning input:
   [planning/item.rs:95](../../../crates/quanta-gqlang-v2-planning/src/item.rs#L95).
9. Every schema name is copied into relation-source authority:
   [pipeline/compile.rs:98](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L98).
10. Type and algebra invariants remain publicly constructible:
    [types/ty.rs:182](../../../crates/quanta-gqlang-v2-types/src/ty.rs#L182),
    [algebra/node.rs:180](../../../crates/quanta-gqlang-v2-algebra/src/node.rs#L180).
11. Catalog resolution is incremental and repeated:
    [modules/catalog.rs:233](../../../crates/quanta-gqlang-v2-modules/src/catalog.rs#L233),
    [pipeline/compile.rs:740](../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs#L740).
12. Waypoint count becomes a path-length bound:
    [logic/graph.rs:253](../../../crates/quanta-gqlang-v2-logic/src/graph.rs#L253).
13. String lexing converts bytes directly and drops unknown escape backslashes:
    [surface/lexer.rs:144](../../../crates/quanta-gqlang-v2-surface/src/lexer.rs#L144).
14. Formatter recursion does not track precedence:
    [surface/format.rs:228](../../../crates/quanta-gqlang-v2-surface/src/format.rs#L228).
15. Transport error recovery clears the entire buffered suffix:
    [diagnostics-stdio/lib.rs:122](../../../crates/quanta-gqlang-v2-diagnostics-stdio/src/lib.rs#L122).
16. The facade glob-exports internal crates:
    [frontend-contract/lib.rs:10](../../../crates/quanta-gqlang-v2-frontend-contract/src/lib.rs#L10).
17. The architecture guard still names deleted workspace members:
    [check_architecture.py:44](../../../tools/architecture/check_architecture.py#L44).

## Deduplicated Finding Ledger

`Owner ticket` points to the first canonical repair. Later tickets may consume
the result but must not reimplement it.

| ID | Sev | Finding | Current evidence | Owner ticket |
|---|---:|---|---|---|
| F01 | P0 | normal `JOIN ... ON` rejects | parser builds outer filter while algebra searches the RHS | 02, 05 |
| F02 | P0 | non-boolean predicates and invalid comparisons compile | pipeline walks values but does not invoke closed bool/operator typing | 04 |
| F03 | P0 | repository CI is locally red | Taplo, architecture, deny, typos fail; Semgrep is vacuous for one rule | 01, 09 |
| F04 | P0 | aggregate placement and nesting are not enforced | raw call-name checks are split between types/algebra | 04, 05 |
| F05 | P1 | binder ignores relation stage order | aliases are pre-collected globally; projection outputs are not frame transitions | 04 |
| F06 | P1 | duplicate projection aliases create order-dependent typing | duplicate record fields plus first-match lookup | 03, 04, 05 |
| F07 | P1 | GROUP partition is lost by aggregate projection | later lowering creates a new empty-key group | 05 |
| F08 | P1 | `REQUIRE CERTIFIED` is document-order dependent | declarations resolve incrementally; META does not revalidate prior assets | 04 |
| F09 | P1 | HIR integrity proof is disconnected from public compile | public item APIs accept arbitrary HIR | 02, 04, 07 |
| F10 | P0 | source rules do not reach real rule-set semantics | every source rule becomes its own one-rule program | 04, 06 |
| F11 | P1 | non-ASCII strings are byte-corrupted | lexer pushes each UTF-8 byte through `char::from` | 02 |
| F12 | P2 | current `lsp` is a custom flat-JSON service | no JSON-RPC, initialize, document lifecycle, UTF-16 positions | 01, 08 |
| F13 | P1 | frontend facade exports internal implementation wholesale | algebra/HIR/planning/types glob re-exports | 07 |
| F14 | P1 | docs and proof ledgers overclaim current behavior | stale LSP methods, examples, path ledger, green/closed claims | 01, 09 |
| F15 | P0 | catalog/graph references are not kind/existence checked | binder defers them but modules resolution is never called by those families | 03, 04 |
| F16 | P0 | inline VIEW bodies bypass relational compilation | catalog registration stores raw query and only runs catalog checks | 04, 05 |
| F17 | P0 | waypoint count is treated as path-length bound | cyclic traversal can loop before/after a waypoint | 06 |
| F18 | P0 | CHECK mode/confirm/emit semantics are discarded | planning returns one label-derived alert plan | 07 |
| F19 | P0 | resolved module graph is not compiler authority | context consumes a separate flat binder graph | 03, 04 |
| F20 | P1 | FIND's validated witness contract is discarded | pipeline validates, then planning reclassifies raw HIR | 06, 07 |
| F21 | P1 | EXPLAIN evidence/truth options are discarded | planning matches `Explain { .. }` and drops fields | 07 |
| F22 | P1 | pack and lock validation can be bypassed | context accepts raw graph; USE PACK checks path suffix/name only | 03 |
| F23 | P1 | algebra/type invariants are debug-only or bypassable | public enum fields and constructors admit invalid states | 03, 05, 07 |
| F24 | P1 | CLI and diagnostics service use different compiler roots | CLI compiles documents; service compiles one root | 04, 08 |
| F25 | P1 | module diagnostics attach whole-item spans | module errors lack primary/secondary/edge locations | 03 |
| F26 | P2 | resource budgets are incomplete | unbounded frame/file widths and recursive graph traversals | 06, 08 |
| F27 | P2 | builders silently overwrite duplicates | imports, declarations, pins, suites, signatures use last-wins paths | 03 |
| F28 | P2 | catalog resolve repeats after each declaration | prevents clean forward refs and adds repeated graph traversal | 04 |
| F29 | P2 | 18-family conformance is routing smoke, not semantic proof | it mostly checks variant/result kind/effect | 01, 09 |
| F30 | P0 | call-shaped rule predicates disappear from dependency graph | rule lowering ignores call callee and visits only arguments | 04, 06 |
| F31 | P0 | builtin purity/determinism/monotonicity/cost are dead metadata | only aggregate identity is consumed; query truth is hardcoded | 04, 07 |
| F32 | P1 | every schema binding becomes relation authority | context does not filter `TypeV1::Relation` | 03 |
| F33 | P1 | parser-valid language and public HIR-valid language differ | parser-only constraints are bypassed by item compile APIs | 02, 04 |
| F34 | P1 | formatter is not precedence-safe for programmatic HIR | binary/bool recursion emits no required parentheses | 02 |
| F35 | P1 | unknown escapes silently change source | `\q` becomes `q` without a diagnostic | 02 |
| F36 | P1 | framing error drops valid buffered suffix requests | transport clears the whole frame buffer | 08 |
| F37 | P2 | numeric range/precision contract is implicit | `i128` and `f64::parse().is_ok()` are accidental language semantics | 03 |

## Reproduced Semantic Failures

The audit reproduced these public behaviors on the frozen base:

```text
from findings f | join repos r on f.repo = r.id
  -> algebra.empty_join_predicate

where 1
where 1 = "x"
where "x" < true
where 1 and 2
where not 1
  -> accepted

rule r : x if r(x)
  -> recursive=false

rule r : x if not r(x)
  -> recursive=false

rule r : x if count(r) > 0
  -> recursive=true and accepted

from users u | project now() as t
  -> plan.truth_lane=deterministic

let x = "\q"
  -> formatter emits let x = "q"
```

## Findings That Are Not Defects By Themselves

1. No physical execution tree: this repo owns frontend semantic planning, not
   runtime physical optimization.
2. No full HM inference: the blocker is bypassing existing closed rules, not the
   absence of general type variables.
3. Programmatic-only algebra operators: acceptable if public docs do not claim
   source reachability and checked constructors guard them.
4. Qualitative semantic cost: acceptable as frontend policy vocabulary; runtime
   cardinality estimates belong downstream.

## Final Audit Conclusion

The current implementation is a real but incompletely integrated frontend. The
first repair must create a single semantic producer and document transaction.
Adding more routing tests or patching consumers independently would preserve the
same false-green architecture.

## Ticket-Plan Audit Delta

A second audit compared the implementation tickets to the frozen source and
current test inventory. It found and corrected four plan-level defects before
implementation:

1. the draft risked copying working owner algorithms into V2 modules even
   though HIR integrity, Binder capture, type registries, module/catalog
   resolution, algebra, Tarjan/stratification, witness contracts, planning
   vocabulary, and transport byte tests already exist;
2. it proposed source `UnionHirV2` without a grammar production and mentioned
   explicit CROSS only downstream; source syntax is now pinned to inner JOIN
   with `on` and explicit `cross join`, with no invented UNION/LIMIT/HAVING
   syntax;
3. ticket 05 attempted to define `AggregateContextV2` while declaring sema
   read-only; ticket 04 now owns that semantic result and ticket 05 consumes it;
4. ticket 08 said CLI passed its budget to compilation even though the pinned
   executable front door is open analysis; both executable paths now use the
   same open document-analysis authority, while closed compilation is
   library-only.
5. ticket 04 attempted to publish `CompiledDocumentV2` before tickets 05
   (algebra), 06 (logic), and 07 (planning) existed. Ticket 04 now stops at the
   atomic `SemanticDocumentV2`; ticket 07 owns complete lower/plan assembly and
   is the first point where final compile/analyze DTOs can exist.
6. tickets 05/06 originally duplicated semantic-owner work: aggregate placement
   was listed under algebra, and rule variable/call resolution under logic
   despite sema being read-only to both. The final-audit corrective sections
   now make ticket 04 the exclusive owner; algebra lowers sealed aggregate
   context and logic consumes resolved rule edges for SCC/stratification.

The 2026-09-21 implementation-aware adversarial pass also found and corrected
the following plan defects: impossible 04-06 pipeline gates, undefined
cross-ticket DTOs and join identity, correlation loss in semi/anti lowering,
no facade-only closed-authority constructor, contradictory analysis error
states, ambiguous authority-byte scope, stale diagnostics paths/protocol truth,
an impossible final parity dependency edge, deleted-ledger references, and the
`_punchlist` filename conflict. The normative corrective sections at the end of
tickets 04-09 override older conflicting prose; implementation receipts must
cite those sections explicitly.

### Implementation-aware correction matrix

| Severity | Defect found during implementation audit | Normative correction |
|---|---|---|
| P0 | 04-06 required pipeline consumers before their migration ticket | 04-06 owner-only focused proof; 07 integration gate |
| P0 | open authority `partial_authority` and infallible standard constructor were undefined | 04 explicit input DTO plus `try_standard_v2` |
| P0 | FIND/CHECK/presentation handoff types had no producer chain | 04 resolved DTOs, 06 graph contract, 07 identity-checked assembly |
| P0 | rule types invented `PredicateIdV2`, vector unary heads, and an unowned Bool type | reuse `ResolvedPredicateIdV2`; Bool proof and unary head fixed |
| P0 | algebra contract invented parallel expression/relation IDs | reuse `SourceQualifiedNodeIdV2` and sema `TypedRelationExprV2` |
| P0 | correlated EXISTS could lose lateral bindings in plain semi/anti nodes | correlation set is mandatory on correlated semi/anti lowering |
| P0 | lowerer outputs had no stable document join key | `SemanticItemIdV2` plus exact-set assembly |
| P0 | facade-only hosts could not construct closed authority | facade-owned checked closed-authority builder |
| P0 | analysis API mixed result errors with item stage states | public `Result<AnalysisDocumentV2, FrontendTransactionErrorV2>`; source-less authority errors are distinct from source-backed document diagnostics |
| P0 | authority profile equality was described as a global byte-total proof | explicit per-admission-session ceiling and maximum-residency semantics |
| P1 | sema and algebra both owned aggregate legality | sema owns; algebra consumes sealed context only |
| P1 | algebra could recompute relation output contracts | exact retained sema contract only |
| P1 | logic budget wording implied runtime traversal/witness emission | static contract/SCC validation charges only |
| P1 | acyclic traversal bypass had no proof producer | all transitive traversal requires explicit bounded depth |
| P1 | diagnostics paths/protocol gate were stale | current paths plus UTF-8 NDJSON truth |
| P1 | CLI/service parity required a forbidden final Cargo edge | dependency-neutral conformance harness |
| P1 | final ledger deletion contradicted its own path gate | migrate proof rows and gate before deletion |
| P1 | `_punchlist`, uniform family matrices, and dirty-SHA closure were contradictory | rename in 06; applicability-aware matrices; clean committed closure precondition |

All rows are plan repairs only until their owning implementation ticket and
receipt are complete. They must not be counted as product closure.

The canonical keep/migrate/delete/build census and existing-test disposition are
in [IMPLEMENTATION_REUSE_AUDIT.md](./IMPLEMENTATION_REUSE_AUDIT.md). Each ticket
now records concrete asset handling, and every implementation receipt must
prove a duplicate-owner/test scan.

## Ticket-Plan Validation Receipt

Validated after the ticket audit on the unchanged product base:

| Check | Result |
|---|---|
| exact product HEAD | `e896a11024bd5a1e1ef03c094a70032f4e6c765f` |
| product-code modifications | none; only this untracked Sep-21 packet exists |
| finding ledger | 37 rows, 37 unique IDs, every F01-F37 named by an owner ticket |
| reuse audit coverage | tickets 01-09 each contain keep/migrate/delete/build disposition |
| test topology | 73 unique paths: 47 in-place, 8 tracked moves, 18 new; one new ratchet is deleted by 08 |
| Markdown relative links | PASS, zero broken |
| code fences/tables | PASS, zero unbalanced fences or inconsistent table shapes |
| unfinished-work marker scan | PASS, zero matches for the packet's forbidden marker set |
| whitespace and spelling | PASS for the packet (`git diff --no-index --check`, `typos`) |
| action commit pins | matched official refs: checkout `v4.4.0`, rust-cache `v2.9.2`, install-action `v2.87.17`, rust-toolchain `stable` snapshot |

No product Cargo suite was rerun for this documentation-only delta. The fresh
822-test/current-gate receipt above remains tied to the same unchanged HEAD; it
is not promoted to repair proof.

## Current Execution Delta — 2026-09-21

The frozen-base findings above remain the causal audit. The live execution
snapshot is now `main@f9ec9543888954106bb7c8bf1dc6500a5e6d34f7` plus the
explicit dirty follow-up recorded by receipts 02 and 03.

| Ticket | Current disposition | Exact boundary |
|---|---|---|
| 01 | `VERIFIED` | proof rail, crate rename, architecture transition gate; no packet closure |
| 02 | `VERIFIED OWNER SCOPE` | direct semantic HIR, UTF-8/format/integrity repairs; stale source module docs corrected |
| 03 | `REPAIR` | earlier 438-test owner receipt is historical; latest modules 126/126 and types 85/85 owner suites plus strict Clippy pass, but sema host/Group ingress and combined final proof are missing |
| 04 | `RED_CAPTURED — IMPLEMENTING` | latest Binder 47/47 plus strict Clippy and sema 52/52 owner-local suites include non-key Group aggregate/collision, exact-tree Rule polarity, Filter/Project/Join/Sort and host Predicate over 8/9 named roots; candidate forgery/transplant, Composition, Inline VIEW, EXISTS, sema strict Clippy (21 lib + 43 lib-test), and all-family transaction remain open |
| 05 | `PLANNED` | algebra library-only check passes, but four owner suites have 74 compiler errors and raw-HIR compatibility remains |
| 06 | `PLANNED` | no current implementation claim |
| 07 | `PLANNED` | 89 planning V1 tests do not prove semantic-only planning; pipeline has 25 compiler errors |
| 08 | `PLANNED` | front-door/NDJSON/legacy deletion not implemented |
| 09 | `PLANNED` | no clean committed candidate, hosted CI, or final closure receipt |

The externally created commits `ceb4142`, `6583091`, and `f9ec954` are
classified as `UNQUALIFIED PARTIAL CONSUMER MIGRATION`. Their presence on
`main`/`origin` does not advance Tickets 04, 05, or 07. Exact blockers and
required regression rails are normative in those ticket files. Packet status
is therefore **active and not complete** despite Tickets 01-03 owner proof.

## Ticket 04 Live Owner/Consumer Adversarial Audit

Frozen comparison: `HEAD=main=origin/main=f9ec9543888954106bb7c8bf1dc6500a5e6d34f7`.
The separate dirty product and packet changes are tracked in
`EXECUTION_LEDGER.md`; this audit does not treat an external mixed commit or an
in-flight file as a verified boundary. The frozen V1 seam inventory is 244
lines, SHA-256
`350ae0442d2f0338eef39a1bb78e9804e19f51bc9bd4a64254d926b6d68f294a`.

| Owner/consumer seam | Live finding | Required proof/owner |
|---|---|---|
| HIR → types/sema | Borrowed checked document could not survive HIR move without a second integrity pass; owned checked carrier is now in dirty HIR | HIR owner: invalid/valid owned proof, one pass, full owner tests and Clippy |
| types → sema | Types previously reminted a document-only source registry; dirty API now accepts exact extended registry. Prior-expression alias dependency exists, but forward declaration is a different legal edge | Types owner: combined identity, declaration-only lazy dependency, no relation forward alias, cycle diagnostics |
| types rule variables → logic | Grammar has no type annotation, and `rule r : x if r(x)` is unanchored; no closed concrete type proof exists | Types owner: bounded singleton constraint solver; sema consumes it once; logic fixtures gain concrete anchor |
| modules catalog → sema | Local IDs minted only at final validation, View-only dependency source, and no Query/Check/Capability/ModelSource/PackSource kind | Modules owner: same-session predeclare/resolve/final validate, full family kind matrix and budget/registry tests |
| graph selector → sema | Parser accepts dotted/slash/`::` paths that modules catalog asset IDs cannot model | Sema authority: separate immutable typed graph-selector catalog, exact role/kind checks, no runtime lookup |
| Binder → sema | Inherited whole-tree prebinding leaked future aliases; dirty owner repair now has sequential frames, default names, exact nested captures and ordinal IDs | Binder owner: 43/43 tests, strict Clippy passed; sema must use facts without re-walking |
| sema → algebra/logic/planning | Draft DTOs risked losing expression shape, graph/check/catalog fields, source identity, and 3 item families; 12-source-stage sketch was unreachable | Sema owner: closed lossless typed tree, six source stages, exact nine-item set, forged-output rejects; Ticket 04 cannot be green from skeleton |
| diagnostics/budget | Source-less authority errors were assigned a fabricated source 0; semantic step schedule unspecified | Distinct authority/document error sum, exact source-backed ordering, documented per-event charges and limit/+1 tests |

Ownership and non-duplication rules are now normative in Tickets 00, 03, 04,
05, 06, 07 and `TARGET_ARCHITECTURE.md`. Architecture checker currently passes
with 14 workspace members, 21 final normal edges present and 22 declared
transitional edges. Pipeline remains a deferred Ticket 07 consumer failure;
no focused owner result upgrades it to a releasable workspace. Ticket 03
remains reopened and Ticket 04 remains implementing until the receipt records
fresh producer tests, exact family coverage, and postimplementation RR.

## Final Ticket-Plan Reconciliation — 2026-09-21 Live Overlay

Frozen reference is `main@f9ec9543888954106bb7c8bf1dc6500a5e6d34f7`
plus the explicitly dirty overlay; earlier `e896a110` and its 822-test run
remain historical base evidence only. The per-ticket live reconciliation
sections are the executable work orders. This audit did not run a final
workspace suite or create a closure receipt.

| Ticket | Already present; do not duplicate | Current gate |
|---|---|---|
| 00-02 | frozen decisions; proof rails; parser/HIR semantic shape and owner receipts | retain historical scope; HIR owned-proof delta needs Ticket 03 fresh proof |
| 03 | one types kernel/registries and one modules validator; latest modules owner-local 126/126 and types 85/85, both strict Clippy | sema host-call and Group aggregate ingress, remaining allocation/contextual negatives, mutation, fresh combined receipt |
| 04 | Binder lexical/capture owner; sema DTO/authority and eight-root source paths; latest 47/47 Binder + 52/52 sema suites cover Group aggregate/collision, Rule polarity, Filter/Project/Join/Sort and host Rule | Group candidate forgery/transplant, Composition, Inline VIEW, EXISTS and other subforms; sema strict Clippy RED and no all-family transaction receipt |
| 05 | one 12-op algebra and existing owner suites | raw-HIR normalizer and 74 test compilation errors; requires sealed six-stage sema input |
| 06 | Tarjan/stratification and graph/witness algorithms | still consumes old HIR/name contracts; source-document reachability and boundedness unproved |
| 07 | planning cost/capability/effect vocabulary and 89 V1 tests | raw-HIR/caller-truth input; exact-key assembly and facade-only external build absent |
| 08 | renamed diagnostics-stdio loop and byte tests | current pipeline edge, old framing/parser, LSP-named parity test, V1 seam inventory active |
| 09 | family registry and routing fixtures | semantic payload matrix, truth inventories, clean candidate, full gates and hosted status absent |

Two additional plan defects were corrected in this pass. First, the Ticket 04
sema draft initially exposed seven relation variants and public constructors
that accepted arbitrary output contracts. The live draft now has six variants
with `JoinModeV2` and crate-scoped stage constructors, but owner-derived
row/provenance/effect equality, item identity, and source acceptance remain
unproved. Second, earlier Ticket 04/05 prose invented `Unnest`, `Intersect`,
and `Except`; the actual `AlgebraOpV1` has exactly 12 variants, of which the
six non-stage variants are Apply, Aggregate, Limit, Union, SemiJoin, and
AntiJoin. Tickets 00/04/05 and the target architecture now pin that exact
vocabulary; no new operator is authorized by the packet.

The last producer/consumer read found identity gaps in the dirty candidates.
The sema graph selector now retains checked host declaration location and full
catalog identity; modules now rejects cross-kind same-spelling collisions and
ties resolved references to its validator-local predeclaration session. Those
producer repairs have focused tests but need final full-suite/Clippy and sema
consumer proof. Tickets 03/04 require a foreign-authority transplant and exact
catalog-reference-set negative before either can return green. The rule
solver now has Record/List constraint branches and an explicit rule-body
relational EXISTS diagnostic; the latter has a semantic prohibition in Ticket
00 while query EXISTS remains a required typed path. The constructed-value
positive now measures 6,884 steps over 74 candidate types after major
charge-before-work repairs. Source-level canonical-budget usage, remaining
fixed/input-map allocation, and host-signature scaling are still unproved.

The second source read also found an invalid-state seam in `sema/src/rule.rs`:
its first draft publicly accepted arbitrary dependency targets, polarities,
and membership with only source-ID validation. The dirty revision narrowed
construction and retained call-node IDs, but still needs exact types-target
comparison and a source rule producer. Ticket 04 owns checked-HIR/types
call-set sealing and negative transplants; Ticket 06 consumes the sealed
result and must not rebuild edges from raw names. A 19-test sema all-target
owner run and 119-test modules run were green on intermediate fingerprints,
then both owners changed source; those numbers are checkpoints, not current
completion receipts.

The current architecture guard and `git diff --check` pass. The guard reports
14 members, 43 normal edges (21 final, 22 declared transitional), and one
transitional dev edge. This is transition-rail evidence, not final-DAG proof.

## Later Producer/Consumer Audit Supersession

The 6,884-step/78-test types and 19-test sema figures above are historical
fingerprints. Types' intermediate shape-charged 8,632-step Record/List fixture
and 79/79 all-target tests plus strict Clippy preceded a later 81/81 types
owner pass with iterative/budgeted rule prepass and lexical-shadow repair.
Sema's earlier 26/26 result preceded a later 36/36 owner-local source run;
strict Clippy remains RED on its last attempted source.
Neither is cross-owner or all-family closure.

The remaining structural gates are explicit in Tickets 03-06: exact modules
catalog asset→typed host predicate identity (no lossy punctuation mapping),
sema's independently checked complete Rule/call set, self-join column origins
keyed by scan occurrence, FIND authored constraints and checked location,
and logic's iterative budgeted Tarjan with a canonical edge/path witness.
Moving EXISTS Semi/Anti lowering to algebra removes logic's sole algebra use;
the corrected final DAG is `logic -> sema + types`. The current checker still
has the former logic→algebra final allowance; Ticket 06 owns its transition,
fixture update, actual dependency removal, and fresh architecture proof.

## Latest Final-Ticket Audit Supersession

The 81/81 types and 122/122 modules counts elsewhere in this historical audit
are superseded by exact-hash owner-local 85/85 and 126/126 results in
`receipts/03.md`. Types/modules now produce distinct host Predicate identity
and a same-session exact catalog proof. Sema has a later host-call source
positive, but no combined Ticket 03 receipt. The sema 36/36 full result is
historical; the latest Binder 47/47 and sema 52/52 all-target owner suites
cover non-key Group aggregate/collision, Rule polarity, Filter/Project/Join/
Sort and host Predicate. Binder strict Clippy passes; sema exits 101 with
21 library and 43 library-test errors.
This is owner-local, not an all-family or pipeline proof.

The final read-only cross-owner audit found Binder GroupBy returning the
entire input frame while algebra emits key-only Group columns. Ticket 04 now
specifies key-only Binder output and a separate aggregate-only partition
authority, with Ticket 05 restricted to consuming the sealed result. Sort's
missing null-placement choice is pinned in Ticket 00 to `NULLS LAST` for both
directions; sema seals it and algebra retains it. Composition's existing
Binder sequential LET/capture behavior is reused, with a sema-local typed
working set rather than host-text substitution. Source/builtin/host Rule
call collisions reject, while host-only success avoids duplicate probe and
resolve charges. Sort, host-call, and non-key Group aggregate source paths now
have owner-local positive/negative proof, but candidate forgery/transplant
rails and complete Group cross-consumer proof remain RED. Composition has no
source path. None is integrated into the public pipeline.
Tickets 04-09 and final-DAG qualification remain open.

One further source inspection found `RuleDependencyCollectorV2` deriving
ancestor aggregate state from *composed* expression effects, which can mark
a source-call sibling under a scalar wrapper as an aggregate dependency.
Tickets 03/04/06 now require the exact resolved builtin overload's intrinsic
effect, preserved call-tree context, and a polarity oracle. The types-owned
exact-ID intrinsic lookup passed 85/85 owner tests; the sema collector code
and a discriminating same-scalar-ancestor source polarity oracle now pass
owner tests. A separate deliberate mutation run is not recorded. Fresh
local `cargo check -p quanta-gqlang-v2-pipeline`
on `ecc38f5`+dirty exited 101 with 25 library errors from retired V1
imports/shape accesses. It remains Ticket 07's atomic consumer migration.

## Final 07-09 Consumer And Front-Door Audit Delta

| Owner | Live evidence and failure mode | Ticket correction |
|---|---|---|
| 07 | `SemanticItemV2` has nine variants; the first V2 planner sketch covered only relation/graph/check/presentation/rule, losing Meta and Composition. Inline VIEW is a Catalog payload with a typed relation but no declaration plan. | Exhaustive plan/nonplan dispatch, nested composition producer/body exact keys, inline VIEW schema lowering without an extra top-level plan. |
| 07 | V1 `SemanticCostV1` has four meaningful success dimensions plus an always-`None` `unsupported_reason_code`; invented CPU/memory/I/O/network/coordination labels have no producer. | Four-field V2 success schema and typed failure-only reason code, with planner and CLI schema tests. |
| 07 | `RelationSourceV2::provenance` is a free string; it cannot prove certified/advisory authority. Existing plan retains render, witness, multiplicity/order, and explain fields. Pipeline still directly imports HIR/Binder/types. | Narrow lane to frontend-checked or exactly modules-validated REQUIRE CERTIFIED; preserve full result/explain carrier; add pipeline manifest/import cut and sema re-export of canonical source ID. |
| 08 | CRLF payload limit requires one-byte CR lookbehind; inline argv and non-regular file reads bypass a file-only precheck. Live public symbols still include `LspServerV1`/`StdioLspErrorV1`. | Precise framing state machine, bounded joined-argv/stream reads, active symbol scan, exact boundary/recovery tests. |
| 08-09 | A conformance Cargo integration test cannot assume dependency binaries have `CARGO_BIN_EXE_*`, and final DAG admits no binary dev edges. | Dependency-neutral `tools/test_frontdoor_parity.py` builds and discovers both bins from Cargo JSON artifacts, then runs black-box parity; final recipe invokes it. |
| 09 | Registry has 18 roots plus separate Decl/HornRule supplemental units; a resource case in every row duplicates owner budget matrices. | Disjoint 18+2 non-vacuity manifests, explicit supplemental payload/plan outcomes, owner-receipt dimension map plus representative cross-front-door limits. |

These are plan corrections, not implementation or verification of Tickets
07-09. The current architecture checker still reports its historical
transition graph; the corrected final target and checker migration are
separate Ticket 06 work.

## Composition And Group Final Adversarial Delta — 2026-09-21

The earlier key-only Group frame and aggregate-positive source tests do not
close the Group contract: the deferred Binder call stack still lacks an
independent checked-HIR ancestry/stage witness and tamper oracle. Ticket 04
now requires exact reference/group/span/call-path comparison against the
authored Project and a mutation matrix before Ticket 05 may lower the sealed
partition. The placement constructor also does not centrally reject
aggregate effects at a Scalar declaration position; this is a static gap
requiring a source-negative before policy repair, not a demonstrated
acceptance claim.

Composition's parser/HIR closed producer and body variants and Binder's
sequential LET frame are reusable. Three missing ingress proofs prevent a
working local table alone from closing it: `FROM` Source currently receives
no Binder source-use resolution; HIR drops the parser's exact source-name
span; and GraphRef resolutions keyed only by enclosing node ID are ambiguous
when several refs share that node. Ticket 02 owns the narrow name-span carrier
delta; Ticket 04 owns exact Binder per-use resolution, local-vs-host sealed
source/graph carriers, nested type-input traversal, and recursive document
assembly checks. The V1 schema-open Composition positive must invert to a
closed V2 negative. Nested Composition is not source-representable in the
current closed grammar, so its capture negative belongs at Binder frame
level unless language scope is explicitly changed.

No ticket state advanced from this read-only audit. The subsequent accidental
test-file truncation during Clippy cleanup was recovered: the 1,047-line
document-transaction test independently matches receipt 04's pre-incident
SHA-256, the kernel test has its full 56 lines, and `git diff --check` passes.
This is recovery only; owner tests/Clippy after subsequent changes are still
required. The pipeline's 25-error compile boundary and final-DAG transition
exceptions remain RED.

Those post-recovery owner commands have since run on the recorded dirty
seven-file fingerprint: sema all-target tests 52/52 and strict all-target
Clippy both exit 0 (`receipts/04.md`). This supersedes the earlier 21+43
Clippy failure, not the missing Composition/EXISTS/Group-tamper/Scalar
placement or full transaction gates. Ticket 04 remains `IMPLEMENTING`.

The next owner delta narrowed two of those gaps. Ticket 02 now retains the
exact parsed `FROM` name span in the existing HIR Source (HIR 72/72, surface
206/206, strict Clippy); no fallback builder synthesizes it. Binder then
records a local Composition LET source-use proof and rejects wrong-kind local
names before host lookup, while exact graph multi-use lookup distinguishes
same-node references and rejects duplicate keys (Binder 52/52 plus strict
Clippy). A demonstrated `sum(1)` Scalar declaration acceptance was repaired
in the central aggregate-context constructor; a nested OrderBy negative also
passes (sema 54/54 plus strict Clippy). Receipts 02/04 contain the exact
dirty fingerprints. Sema still has no Composition source-positive path, and
candidate HIR-ancestry/tamper, Inline VIEW, relational EXISTS, Algebra,
pipeline, and final DAG remain open.

## Final Execution Supersession — `main@ecc38f59` + Dirty Overlay

This section supersedes earlier current-state counts in this audit; historical
fingerprints remain evidence only for the source they name.

| Scope | Result | Proves | Does not prove |
|---|---:|---|---|
| HIR | 72/72 GREEN | exact source-name span carrier/integrity | downstream integration |
| Surface | 206/206 GREEN | parser/formatter byte spans | semantic authority |
| Binder | 52/52 GREEN | exact source/graph uses, scope/capture/Group frames | Sema Composition |
| Types | 85/85 GREEN | closed types, rule solver, host key, intrinsic effects | consumer proof |
| Modules | 126/126 GREEN | catalog/lock/admission and host proof | Sema transplants |
| Sema | 59/59 GREEN on current dirty overlay | direct paths, Group ancestry/tamper, and carrier invariants | Composition, Inline VIEW, query EXISTS, all-family transaction; receipt still names prior 56 fingerprint |
| Logic | 49/49 GREEN | reusable V1 algorithms | sealed semantic input |
| Planning | 89/89 GREEN | reusable V1 vocabulary | V2 assembly/facade |
| Algebra | compile RED | retired raw-HIR contract exposed | Ticket 05 receipt |
| Pipeline | 25 compile errors | exact Ticket 07 cutover boundary | public integration |

The combined HIR/Surface/Binder/Types/Modules/Sema all-target and strict
Clippy commands exit 0. Logic/Planning all-target tests exit 0. Architecture
transition lint exits 0 at 21 final + 22 transitional normal edges and one
transitional dev edge; this is not final mode. `git diff --check` and the
unfinished-marker scan pass. Receipts 05–08 are absent.

The global format gate is independently RED: `cargo fmt --all --check` exits 1
and identifies 23 Binder/HIR/Sema/Surface files. No formatting rewrite was
applied during this audit because the shared tree contains overlapping owner
changes. Owner test/Clippy GREEN therefore remains valid as behavior/lint
evidence but is not a format-complete receipt. Final architecture mode also
exits 1 on the 22 normal plus one dev transition edges.

The Group deferred-candidate gap described above is now owner-closed: Sema
independently walks checked HIR, requires a direct Project-over-Group route,
compares exact reference/span/Group/call-stack identity to Binder's candidate,
and has primitive tamper oracles for reference, span, Group, outer/inner call,
and dropped path. The source negative with intervening Sort passes. This was
not a separate deliberate mutation build and does not close Ticket 05.

The definitive remaining critical path is:

```text
04 Composition + Inline VIEW + query EXISTS + all-family transaction
  -> 05 sealed Sema-to-algebra lowering
  -> 06 sealed rule/graph input and logic-to-algebra edge removal
  -> 07 exact assembly, pipeline cutover, curated facade
  -> 08 bounded NDJSON/CLI front doors
  -> 09 immutable-candidate conformance and truth sync
```

Each ticket now ends with a worker-ready execution block containing exact
reuse boundaries, implementation order, tests, and stop conditions. Those
final blocks are normative when older historical prose reports a different
test count or open item.

One additional invalid-state seam was found after that matrix: the draft Sema
Composition DTO stores each producer and body as arbitrary `SemanticItemV2` and
stores WITH as a string. That cannot represent legal SAMPLE without inventing
a document-root variant, permits illegal producer/body families, and loses the
Binder-owned LET ordinal. Ticket 04 now requires closed producer/body enums,
exact `(source, composition node, ordinal)` identities, a Composition-local
scan-source variant, exact WITH identity/location, and Binder-aware local graph
resolution. Ticket 07 consumes those carriers and may not repair them by name.

## Superseded Multi-Angle RR Checkpoint — 2026-09-21

The closed Composition enums are now present, and two defects found during RR
were repaired in the dirty overlay: nested producer/body IDs are compared to
the expected HIR IDs; Inline VIEW compares a distinct nested Query ID rather
than its CatalogDecl ID; non-row FIND rejects every `Some(output)` shape. Sema
all-target tests pass 59/59 (8+42+1+4+3+1) and strict all-target Clippy passes
for that checkpoint.
This does not advance Ticket 04.

At that checkpoint, the source contract was blocked by the following. The later
`Final Current-Tree Supersession` records which producer carriers now exist:

- Ticket 03: one Types-owned deferred multi-field Query-row dependency is
  required to retain the single whole-document kernel call; the current scalar
  `BindingFromExpression` is insufficient. Relation EXISTS also needs exact
  neutral root/capture identities instead of one source plus an opaque string.
- Ticket 04: local graph targets, canonical generated SAMPLE/FIND origins,
  checked scan-use locations, registry-complete nested carriers, Inline View
  source wiring, and full EXISTS materialization are absent.
- Ticket 07: three recursive algebra walks can overflow; compiled plan
  applicability is optional instead of structural; the public authority hash
  is undefined; facade proof must cover the whole external API.
- Ticket 08: the migration inventory is stale; request validation/diagnostic
  schema/span/exit policy are incomplete; CLI copies unbounded argv; diagnostic
  count and response bytes are uncapped.
- Ticket 09: example and test inventories risk duplicate or empty authorities;
  facade scripts are absent from `just ci`; hygiene is not recursively tracked;
  final evidence needs tested SHA A plus attestation B (or an external
  attestation), not a self-referential one-SHA receipt.

The normative critical path is:

```text
03 deferred Query-row + exact EXISTS capture producer contract
  -> 04 complete semantic source transaction
  -> 05 sealed relational lowering
  -> 06 sealed Rule/Graph logic and final edge
  -> 07 iterative planning + structural facade
  -> 08 bounded CLI/NDJSON
  -> 09 conformance, exact-SHA CI, two-phase attestation
```

No V1 green count, carrier-only unit test, transition architecture pass, or
dirty receipt hash closes a downstream ticket.
## Final Current-Tree Supersession — 2026-09-21

Audit boundary: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
plus the shared dirty overlay. This section supersedes older current-state
counts in this document; older receipt statements remain historical.

| Area | Current observation | Meaning |
|---|---|---|
| HIR | 72/72 GREEN | owner behavior only; name-span test remains untracked |
| Surface | 206/206 GREEN | parser/formatter owner behavior only |
| Types | 86/86 + strict Clippy GREEN | deferred rows and exact EXISTS captures implemented |
| Modules | 126/126 + strict Clippy GREEN | current validator green; View topology phase still absent |
| Binder | 53/53 + strict Clippy GREEN | exact Composition binding/use and no-fallback guards implemented |
| Sema | 59/59 + strict Clippy GREEN | direct families green; Composition/Inline View/Query EXISTS materialization still absent |
| Algebra | RED, 74 test compile errors | retired HIR test contract; Ticket 05 cutover not started |
| Logic | 49/49 GREEN | algorithm owner only; source document adapter absent |
| Planning | 89/89 GREEN | classifier owner only; facade assembler absent |
| Pipeline | RED, 25 lib / 29 lib-test compile errors | structural V1 consumer cutover, not independent fixes |
| Conformance | RED | blocked by Pipeline compilation |
| architecture transition | GREEN | 22 normal + 1 dev exceptions are declared |
| architecture final | RED | the same 22 normal + 1 dev exceptions remain |
| Taplo | GREEN | supersedes the old six-manifest failure |
| Rustfmt | RED, 27 unique files | shared-owner format debt; no bulk format performed |
| diff check | GREEN | no whitespace-error proof only |

### Duplicate implementation audit

The following owner implementations now exist and must be consumed, not
recreated:

- Binder: exact lexical targets, capture edges, relation frames, Composition
  `(owner, ordinal)` proof, self/forward no-host-fallback guard;
- Types: one document kernel, deferred relation rows, exact relation-predicate
  capture inputs, operator/builtin registries and effects;
- Modules: one document catalog validator/session and checked catalog/host-call
  proofs;
- Sema: one document transaction, checked relation outputs and closed item
  vocabulary;
- Logic: Tarjan SCC, stratification, recursive legality and graph contracts;
- Planning: semantic capability/cost-class/result/explain vocabulary.

Compatibility builders, a second Sema type checker, a second catalog resolver,
name-keyed Composition tables, and Pipeline-local semantic compilation are
explicitly prohibited.

### Missing integration found after implementation

1. Sema collects Composition Query rows but rejects the root item during final
   assembly. Producer facts exist without a consumer path.
2. Sema supplies exact Query EXISTS inputs to Types, but its bool materializer
   still rejects the variant and the semantic document has no owned nested
   relation registry.
3. Inline View carrier checks exist, but Sema hard-rejects the source form and
   Modules exposes no pre-finalization topology proof/order phase.
4. Composition identity remains publicly remintable; scan identity and checked
   use location are split; SAMPLE/FIND generated column origins are absent.
5. Graph/CHECK/EXPLAIN local Composition targets are host-only, and catalog
   references discard the validator session proof.
6. Tickets 05-08 remain structural migrations; passing Logic/Planning direct
   suites does not close source or public-facade reachability.

The definitive critical path remains:

```text
03 View-topology phase + carrier mutation receipt
-> 04 Composition + Inline View + Query EXISTS source closure
-> 05 sealed Sema-to-Algebra lowering
-> 06 document-wide Logic adapter
-> 07 semantic plan assembler + explicit facade + Pipeline cutover
-> 08 bounded front-door parity
-> 09 clean-candidate conformance and truth sync
```

## Final RR Supersession After SS-03C / SS-04A — 2026-09-21

Audit boundary remains
`main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` plus a shared dirty
overlay. The previous table is superseded only for Types, Modules and Sema:

| Scope | Current evidence | Correct interpretation |
|---|---:|---|
| Types | 87/87 + strict Clippy GREEN | deferred rows and Types-owned EXISTS effect/order are owner-green; exact Binder owner/ordinal/completeness is still open |
| Modules | 129/129 + strict Clippy GREEN | topology proof/order is implemented; Sema currently submits an empty graph |
| Binder | 53/53 + strict Clippy GREEN | exact relation-source local identity remains reusable |
| Sema | 64/64 + strict Clippy GREEN | Query-only Composition is source-reachable; incomplete analysis is sealed; remaining subfamilies are explicit rejects |

The same-overlay owner runs, package-scoped rustfmt, the later global
`cargo fmt --all --check`, and `git diff --check` all exited 0. This
supersedes the earlier current-state format RED counts; those remain historical
for their named snapshots. Modules order reversal and Composition ordinal-zero mutations
both killed their intended oracles and were reverted before recovery. This is
stronger than static inspection but weaker than an immutable integrated
receipt.

Structural findings that now govern execution:

1. Ticket 03 is not closed by an opaque capture-shaped DTO. Binder owns exact
   lexical captures; Sema must verify complete ordered equality, while Types
   validates HIR membership/source/order and owns base effect policy.
2. Ticket 03's Modules producer is ready. Ticket 04 must replace
   `validate_view_topology(Vec::new())` with exact direct View edges, consume
   dependency-first IDs for inline-query materialization, and finalize the
   same session/proof.
3. Ticket 04's Query LET path is real and must be extended, not duplicated.
   Raw public Composition tuples, host-only graph targets, fabricated
   Sample/FIND expression origins, and dropped registry/session proof remain
   invalid-state seams.
4. Query relational EXISTS still stops between Types and semantic IR. The
   document needs one owned nested-relation registry and exact correlation
   edges; Rule-body relational EXISTS remains a hard reject.
5. Tickets 07-09 retain their final RR corrections: iterative planner walks,
   structural plan applicability, complete facade API allowlist, total bounded
   wire schema and exit policy, one semantic-manifest SSOT, base-to-candidate
   test inventory, recursive hygiene, exact-SHA hosted evidence and two-phase
   attestation.

No downstream ticket status advanced. The critical path is now more precise:

```text
03 capture authority/mutations + frozen receipt
-> 04 private local identity + generated rows + graph capabilities
      + Inline View + Query EXISTS + nine-root source transaction
-> 05 sealed lowering
-> 06 sealed logic adapter
-> 07 iterative planning/facade/pipeline cutover
-> 08 bounded front doors
-> 09 immutable candidate and attestation
```

The downstream RED boundaries were re-executed after this checkpoint: Algebra
all-target test compilation still has 74 retired-HIR errors, Pipeline library
check still has 25 V1/API-cutover errors, and final architecture still rejects
22 normal plus one dev transition edge. No downstream closure is inferred from
the global formatting recovery.

## Final RR Supersession After SS-03E / SS-04E — 2026-09-21

The latest owner checkpoints supersede only the Types/Sema rows below. They
remain dirty-overlay evidence, not an immutable integrated receipt.

| Scope | Current evidence | Correct interpretation |
|---|---:|---|
| Types | 88/88 + strict Clippy GREEN | exact earlier `PriorRelation` row reuse and foreign-target reject are owner-green; reverse-authored Inline View still requires validated pre-evaluation order |
| Sema | 66/66 + strict Clippy GREEN | Query relational EXISTS now seals exact nested relations and Binder correlations; Inline View, non-Query Composition and nine-root transaction remain open |

Query EXISTS is no longer a Types-to-Sema reachability gap. Its semantic root
is backed by a transaction-owned exact registry, and Rule-body EXISTS remains
rejected. The next Ticket 04 P0 is Inline View's Modules-proof-derived Types
pre-evaluation order; no downstream Algebra/Pipeline work is authorized until
that source closure is complete.

## Final RR Supersession After SS-03F / SS-04F — 2026-09-21

The earlier Inline View P0 is no longer an empty-topology/hard-reject path.
Current code has one same-session chain from direct source use through Modules
proof, Types pre-evaluation, asset-keyed materialization, `family=None` edge
retention, and final validation. RR found and repaired the one ordering hole
where an ordinary non-plan consumer could not use an already pre-evaluated View.

Do not mark Ticket 04 complete. Evidence is owner-local/shared-overlay only:
Types 89/89, Modules 130/130, Sema 69/69, strict Clippy and scoped format/diff
GREEN. Missing closure evidence is explicit: diamond/repeated-edge/cache
mutation, Surface-to-HIR route, external typed Views, Composition completion,
EXISTS mutation completeness, Ticket 05 sealed lowering, and immutable
candidate/hosted CI. The global diff check is not green because of an unrelated
bugbash trailing-whitespace line.

## Final RR Supersession After SS-04G — 2026-09-21

The Composition relation-source raw-identity P0 is no longer open. The source
enum now retains a private, Binding-derived carrier and the transaction seals
it after exact Binder target/kind validation. RR confirmed zero residual raw
struct variants, public-field construction rejection through doctest, and
wrong-binding-kind rejection. Owner evidence is Sema 70/70 plus 1/1 doctest,
strict Clippy, package fmt, and scoped diff GREEN on the moving overlay.

Do not infer broader Composition completion: only Query producers and a Query
body are currently materialized. Sample/FIND generated output, graph/CHECK/
EXPLAIN targets, cache/transplant mutations, full front-door integration, and
the immutable candidate remain required Ticket 04 evidence.

## Final RR Supersession After SS-04H — 2026-09-21

Generated Composition row provenance is now structurally honest: a SAMPLE or
row-capable FIND schema cannot be accepted with a fabricated expression key or
another producer's generated ID. RR verified the checks are in the canonical
output/producer constructors, not in a downstream consumer. This removes a
P0 prerequisite only; no non-Query HIR path is enabled yet.

The remaining P0 is explicit: add one Binder-first disjoint Host|Composition
graph-target carrier and reuse it for FIND, SAMPLE scope, CHECK, and EXPLAIN;
then add Types input and transaction materialization in source order. Host
fallback after an exact local Binder target is forbidden.
