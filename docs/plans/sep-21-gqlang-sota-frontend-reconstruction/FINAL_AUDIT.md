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
    [lsp/lib.rs:118](../../../crates/quanta-gqlang-v2-lsp/src/lib.rs#L118).
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
| P0 | analysis API mixed result errors with item stage states | public `Result<AnalysisDocumentV2, FrontendFailureV2>` |
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
