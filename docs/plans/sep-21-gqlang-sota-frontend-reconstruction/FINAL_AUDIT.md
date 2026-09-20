# Final Current-Head Audit

Status: `AUDIT COMPLETE — REPAIR ACTIVE; PACKET NOT COMPLETE`

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
| 03 | `REPAIR` | earlier 438-test owner receipt is historical; latest modules 126/126 and types 83/83 owner suites plus strict Clippy pass, but sema host ingress/collision and combined final proof are missing |
| 04 | `RED_CAPTURED — IMPLEMENTING` | Binder 43/43 plus strict Clippy on earlier scope; latest sema 43/43 owner-local suite includes Filter/Project/Join/Sort and host Predicate calls over 8/9 named roots. Group public-frame leak, Composition, Inline VIEW, EXISTS, strict Clippy (45 errors), and all-family transaction remain open |
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
| 03 | one types kernel/registries and one modules validator; latest modules owner-local 126/126 and types 83/83, both strict Clippy | sema host-call ingress/collision/SCC exclusion, remaining allocation/contextual negatives, mutation, fresh combined receipt |
| 04 | Binder lexical/capture owner; sema DTO/authority and eight-root source paths; latest 43/43 suite covers Filter/Project/Join/Sort and host Rule | Group key-only frame plus aggregate-only partition input, Composition, Inline VIEW, EXISTS and other subforms; strict Clippy RED and no all-family transaction receipt |
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
are superseded by exact-hash owner-local 83/83 and 126/126 results in
`receipts/03.md`. Types/modules now produce distinct host Predicate identity
and a same-session exact catalog proof. Sema has a later host-call source
positive, but no combined Ticket 03 receipt. The sema 36/36 full result is
historical; a later 43/43 all-target suite covered Filter/Project/Join/Sort
and host Predicate, while strict Clippy exited 101 with 45 lib-test errors.
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
resolve charges. Sort and host-call source paths now have owner-local proof;
Group has only a later key-only focused 2+2 milestone, with aggregate-only
partition access RED; Composition has no source path. None is integrated
into the public pipeline.
Tickets 04-09 and final-DAG qualification remain open.

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
