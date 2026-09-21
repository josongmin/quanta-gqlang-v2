# Logic V1 Migration Inventory — Lane W2-C Report (integrator-sealed 2026-09-21)

Produced by read-only lane W2-C against the W1-era working tree (base `e9dd3e4`
+ overlay). Zero product writes; this file is the integrator's verbatim
preservation of the lane's message-only deliverable. Authority: Ticket 06
SS-06B supersession section. This inventory is the W3 lane-split planning
input; it does not modify Ticket 06 obligations.

## Baseline facts (static)

- Compile-RED confirmed without building: `logic/src/source.rs:14` imports
  `AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, SemiJoinV1`;
  `tests/v3_logic_source.rs:19` imports `normalize_relation, AlgebraOpV1,
  NormalizeOptionsV1`. `rg` over `algebra/src` returns ZERO of these symbols
  (`lib.rs` now exports only `AlgebraDiagnosticV2/AlgebraFailureKindV2/
  AlgebraFamilyV2/CheckedAlgebraNodeV2/CorrelationSetV2/LoweredRelationItemV2/
  lower_relation_item`).
- All SS-06B migration targets exist in Sema: `ResolvedRuleV2` (rule.rs:69),
  `ResolvedRuleCallV2` (rule.rs:22, `polarity()` :57), `RuleCallPolarityV2`
  (rule.rs:11), `ResolvedPredicateIdV2`, `ResolvedGraphQueryV2` (item.rs:198)
  + `GraphWitnessBoundsV2` (item.rs:212, `witness_bounds()` :352),
  `TypedBoolExprV2` (expr.rs:429), `SemanticDocumentV2::rules()`
  (document.rs:849), `GraphQueryKindV1` re-exported via sema `lib.rs:24`.
  No producer gap for W3.
- Static test census: 44 `#[test]` across 8 files + 1 doctest
  (source.rs:46). Historical "48 tests"/"49/49" no longer match the tree;
  use 44+1 as the migration base.

## 1. Production inventory (src) — KEEP / MIGRATE / DELETE

### lib.rs (56 lines) — integrator-reserved (W3)
- :40-45 mod decls — KEEP (module set unchanged; source.rs repurposed as
  adapter home).
- :47-55 pub use re-exports incl. `ExistenceCheckV1/ExistencePolarityV1` —
  MIGRATE: drop source.rs exports; add `RuleProgramContractV2`/
  `GraphQueryContractV2`/reject-all policy.
- :1-38 crate docs — MIGRATE: rewrite for Sema-input logic layer.

### source.rs (145 lines) — W3-C's file; ALL current contents DELETE
- :14-15 algebra V1 + hir imports — DELETE (compile-RED root; never restore).
- :21-28 `ExistencePolarityV1` — DELETE (sema RelationExists owns polarity).
- :33-38 `keyword()`, :46-59 doctest, :61-67 `from_keyword()` — DELETE
  (surface/sema owns keyword parsing; spelling pins die).
- :79-115 `ExistenceCheckV1` + checked ctor — DELETE (second EXISTS producer;
  Ticket 05 sole owner).
- :124-144 `lower()` Semi/Anti — DELETE (semantics transfer to Ticket 05
  owner tests); file becomes `RuleProgramContractV2::from_document` adapter.

### recursion.rs (448 lines) — W3-A
- :43-51 `BodyPolarityV1` — KEEP (mapped from `RuleCallPolarityV2`).
- :55-88 `BodyLiteralV1` + polarity ctors — MIGRATE to stable IDs.
- :94-101 `RuleDefV1` — MIGRATE (head -> stable predicate ID +
  `SemanticItemIdV2` + checked location).
- :108-114 `from_rule(&RuleHirV1)` — DELETE (named verbatim in SS-06B).
- :119-125 `new(head,body,span)` — DELETE (projection from sealed
  `ResolvedRuleV2` replaces).
- :136-158 `StratificationProofV1` + queries — MIGRATE (ID-keyed; becomes
  `proof` field of `RuleProgramContractV2`).
- :168-194 `RecursiveProgramV1{new,from_rules,add_rule,rules}` — DELETE
  (caller-selected Vec forbidden; `from_document` sole ingress).
- :202-265 `stratify()` — KEEP ALGORITHM / MIGRATE INPUTS; add canonical
  offending-edge + actual-return-path witness, permutation-stable.
- :269-273 `sorted_cycle()` — DELETE (sorted names are not a witness).
- :276-303 `DependencyGraph::build` — MIGRATE; FAIL CLOSED on nonlocal/
  undeclared target (current code inserts every callee incl. undeclared —
  flagged defect); builtin/host never vertices.
- :305-324 successors/has_self_loop/SCC driver — KEEP.
- :332-386 `assign_strata` — KEEP (already iterative); add per-relaxation
  charges.
- :390-447 `TarjanState` + recursive `strong_connect` — MIGRATE to explicit
  iterative frames + charges; 50k chain without process stack.

### aggregate.rs (121 lines) — W3-A; entire file DELETE-replace
- `AggregateLegalityV1`, `RecursiveAggregateV1`, `RecursiveAggregateMatrixV1`
  (+Default admitting min/max), `new/empty/with_admitted`, `admits/classify`,
  `check()` — ALL DELETE; replaced by fixed reject-all policy carrier
  (`RejectAllRecursiveAggregatesV2`); nonrecursive legality stays Sema's.

### graph.rs (294 lines) — W3-B
- :42-52 `graph_query_keyword`, :57-95 endpoint roles + keyword, :135-150
  required-endpoints table, :180-190 `dedup_identity` — KEEP (inputs change).
- :99-108 `is_present` — MIGRATE to `ResolvedGraphQueryV2` selectors.
- :117-129 `GraphQueryContractV1`/`GraphQuerySemanticsV1` — MIGRATE to V2:
  private fields, owner-only fallible ctor, exact bounds accessors,
  `SemanticItemIdV2` + checked location.
- :160-170 `relation_shape` — MIGRATE; PointsTo Acyclic exemption DELETE.
- :195 `DEFAULT_MAX_WITNESSES=64` — DELETE (bounds only from
  `GraphWitnessBoundsV2`/budget).
- :203-251 `classify(&GraphQueryHirV1)` — MIGRATE to `ResolvedGraphQueryV2`;
  ordered via/through retained.
- :269-293 `path_length_bound` — DELETE POLICY (waypoint-count-as-depth and
  depth+waypoint conflict both die; depth mandatory+positive; depth+waypoints
  VALID).

### witness.rs (178 lines) — W3-B
- :43-53 `DedupIdentityV1` — KEEP; :58-64 `keyword` — DELETE (unconsumed).
- :70-86 `WitnessTieBreakV1` — KEEP enum / DELETE keyword accessor.
- :90-106 `WitnessRequestV1` — MIGRATE (dies as caller request; sealed bounds
  + budget validation: depth <= max_witness_depth, cap <= max_witnesses).
- :114-126 `WitnessContractV1` — MIGRATE (private fields + owner ctor).
- :135-177 `bind()` — KEEP VALIDATION CORE; add budget exact/+1.

### diagnostic.rs (271 lines) — integrator-reserved (W3)
- DELETE variants: `UnknownQuantifier` (:24-29), `UnsupportedRuleHead`
  (:34-37, dead), `GraphConflictingPathBound` (:104-109).
- KEEP variants: `NegationCycle` (payload -> canonical edge + actual return
  path + checked location), `RecursiveAggregate`, `UnboundedWitness` (+4
  reasons), `GraphEndpointMissing/Unexpected`, `NonMonotoneRecursion`.
- :162-270 code()/span()/message() — MIGRATE: `SpanV1` ->
  `CheckedLocatedSpanV2` everywhere; drop deleted variants' messages.

### Cargo.toml — integrator-only
- :17 algebra dep DELETE; :18 hir dep DELETE; :19 types dep KEEP (final =
  Sema + Types only).

## 2. Test inventory (44 + 1 doctest)

- `v3_logic_recursion.rs` (9): KEEP mutual-recursion/self-loop/deep-chain
  positives + layered strata; DELETE from_rule/matrix-admit/with_admitted/
  add_rule rows; MIGRATE closure + strata rows to IDs.
- `v3_logic_source.rs` (5 + doctest): ALL current cases DELETE — EXISTS
  semantics transfer to Ticket 05; keyword/quantifier rows move to surface/
  sema; file becomes W3-C adapter suite.
- `v3_logic_hellgate.rs` (12): rows 1,2,3,11,12 -> W3-A (canonical-edge
  upgrades); rows 4-10 -> W3-B (witness owner) with A consuming codes.
- `v3_graph_query_hellgate.rs` (6): 2 KEEP, 2 MIGRATE (exact retained
  depth/cap), 2 FLIP->DELETE (PointsTo-without-depth now REJECTS;
  via-without-depth now REJECTS).
- `v3_graph_query_depth_waypoint_conflict.rs` (2): both FLIP->DELETE
  (depth+waypoint becomes VALID; `GraphConflictingPathBound` dies).
- `v3_logic_witness.rs` (7): 4 MIGRATE (budget exact/+1, retention), 2 DELETE
  (spelling pins), 1 DELETE (acyclic-without-bound positive).
- `test_endpoint_conflict.rs` (1): KEEP/MIGRATE (span -> checked location).
- `v3_logic_punchlist.rs` (2): MOVE+REWRITE to `v3_logic_mutation_rails.rs`;
  sweep supplies explicit depth/cap per subkind (PointsTo needs depth).
- `tests/support/mod.rs`: fixture builders MIGRATE
  (sp -> CheckedLocatedSpanV2-style, gref -> resolved selectors).

TALLY: production ~17 KEEP / 22 MIGRATE / 21 DELETE (source.rs + aggregate.rs
nearly wholesale). Tests: 14 KEEP / 15 MIGRATE / 15 DELETE-or-FLIP (+1 doctest
DELETE).

## 3. W3 lane split (adopted by integrator)

- W3-A: `recursion.rs` + `aggregate.rs` + rule/SCC tests (6 surviving
  recursion rows + hellgate rows 1,2,3,11,12) + NEW: acyclic diamond,
  canonical-edge/permutation witnesses, every-spelling aggregate rejects,
  budget exact/+1 + 50k chain.
- W3-B: `graph.rs` + `witness.rs` + graph/witness tests (hellgate rows 4-10,
  graph_query suites, witness suite, endpoint conflict, punchlist->rails) +
  via/through order retention + nested CHECK/FIND.
- W3-C: `source.rs` (delete contents) -> `RuleProgramContractV2::from_document`
  + membership/edge/target/polarity/location validation; rewrite
  `v3_logic_source.rs` as adapter suite; largest NEW-test block = membership
  forgery matrix.
- Integrator-reserved at W3: `lib.rs`, `diagnostic.rs`, `Cargo.toml`
  (+lockfile), architecture fixtures, Receipt 06.

## 4. Coverage gaps routed (NEW work, no V1 coverage)

G1 acyclic diamond (W3-A); G2 canonical offending edge + return path +
permutation-invariance, mandatory after sorted_cycle deletion (W3-A); G3
entire membership/edge-forgery class (W3-C); G4 builtin/host same-spelling
exclusion + collision (W3-A/C); G5 budget exact/+1 + 50k chain (W3-A); G6
per-subkind depth/cap exact/+1 (W3-B); G7 ordered via/through + depth+
waypoint positive (W3-B); G8 nested CHECK/FIND (W3-B); G9 every recursive
aggregate spelling rejects (W3-A); G10 architecture no-HIR/Algebra assertion
(integrator, W3 barrier); G11 source-level fixtures through a sema document
(W3-C + Ticket 07); G12 ticket inconsistency — see below.

## 5. Algorithm reuse boundary (read-only judgment)

REUSE: Tarjan SCC core (recursion.rs:390-447, convert to iterative frames +
charges), assign_strata relaxation (:332-386, add charges), DependencyGraph
mechanics + SCC driver (:305-324), graph endpoint legality table + ALL-scan
+ dedup_identity, witness bind() degeneracy/order checks.
MUST-DIE: source.rs lower() (duplicate producer), path_length_bound
waypoint-inference + PointsTo exemption + DEFAULT_MAX_WITNESSES=64, the
entire aggregate name-keyed matrix API, sorted_cycle name witness, from_rule
raw-HIR ingest, caller-assembled RecursiveProgramV1 ingress.

## Integrator decisions recorded from this report

1. **W3 sequencing (adopted)**: the integrator pre-cuts `diagnostic.rs`
   (DELETE variants + SpanV1 -> CheckedLocatedSpanV2 sweep) at W3 barrier
   start, BEFORE W3-A/B test freezes, to avoid double-churn on diagnostic
   payloads.
2. **G12 ticket amendment (adopted in principle)**: Ticket 06's "add an exact
   `logic.non_monotone_recursion.negated_recursion` fixture" wording
   contradicts the deliberately-unreachable negated arm (diagnostic.rs:110-119
   dead-variant note; hellgate :307 is the precedence rail). The integrator
   will amend the ticket wording at W3 dispatch instead of adding a
   contradictory fixture.
3. Historical 48/49 counts are superseded by the static 44+1 census; no prior
   number may be carried into Receipt 06 without fresh execution (already an
   SS-06B rule).
