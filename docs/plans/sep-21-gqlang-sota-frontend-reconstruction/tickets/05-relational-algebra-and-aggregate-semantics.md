# 05 — Relational Algebra And Aggregate Semantics

Status: `PLANNED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

F01, F04, F06's algebra half, F07, F16's lowering half, and F23's algebra
invariants.

## Goal

Lower only typed relational contracts into valid-by-construction logical
algebra, preserving stage output semantics exactly.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | all 12 `AlgebraOpV1` families, provenance/source identity, cardinality semantics, aggregate-star carrier, valid normalization helpers | preserve one logical algebra and migrate its existing operator/provenance tests |
| `MIGRATE` | `AlgebraNodeV1` constructors and `normalize_query` | make construction checked and change input from raw HIR to sealed typed relations |
| `DELETE` | raw-HIR normalizer entry, whole-tree alias precollection, string composition dispatch, `_keys` decoding, debug-only invariant enforcement | delete when typed lowering covers the source-reachable stages |
| `NEW` | typed lowering adapter, explicit join/correlation lowering, release-mode constructor errors | do not create a second algebra tree or duplicate normalizer |

The four existing algebra suites and the pipeline group/order tests are
migrated in place. Their operator, provenance, and cardinality assertions remain
useful; raw-HIR, `_keys`, and debug-assert expectations are replaced, not copied
into parallel V2 tests.

## Write Set

1. `crates/quanta-gqlang-v2-algebra/src/node.rs`
2. `crates/quanta-gqlang-v2-algebra/src/normalize.rs`
3. `crates/quanta-gqlang-v2-algebra/src/diagnostic.rs`
4. `crates/quanta-gqlang-v2-algebra/src/lib.rs`
5. `crates/quanta-gqlang-v2-algebra/Cargo.toml` — integrator only; remove
   direct HIR/Binder edges and add sema
6. `/Cargo.lock` — integrator only if the edge cut changes it
7. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` — integrator only
8. the exact algebra and pipeline relational tests listed below
9. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/05.md`
10. `crates/quanta-gqlang-v2-algebra/tests/support/mod.rs` only when repeated
    helpers from touched algebra tests are actually consolidated

`crates/quanta-gqlang-v2-sema/src/relation.rs` is a read-only input owned and
frozen by ticket 04. If the required contract is absent, stop and amend 04; do
not patch sema from this ticket.

The final DAG permits algebra → sema/types but not algebra → HIR. Current
`algebra/src/node.rs` and `diagnostic.rs` name HIR `SpanV1`/`HirNodeIdV1`, so
removing the Cargo edge without replacing their carriers is not executable.
Ticket 04 must expose checked stage/item locations; Ticket 05 migrates public
node/diagnostic fields to those sealed semantic locations or source-qualified
IDs. Do not keep a hidden direct HIR import or turn exact spans into byte-zero
errors.
Its row-aligned SourceField origin must distinguish two scan occurrences of
the same host/View asset (e.g. a self-join with different aliases); the asset
ID alone does not preserve the existing algebra's column provenance.

## Implementation

1. Change normalizer input from raw Query HIR to `TypedRelationV2`.
2. Make algebra node fields private and constructors fallible where needed.
3. Lower explicit Join HIR directly; ON is never recovered from a Filter.
4. Permit predicate-free joins only for explicit CROSS.
5. Consume semantic output schema, source IDs, and cardinality directly.
6. Preserve GROUP partition keys through aggregate projection.
7. Consume ticket 04's `AggregateContextV2` for projection, predicate, join,
   group key, ordering, HAVING, nested-call, and recursive-rule contexts.
8. Trust the sealed aggregate legality proof; reject only an impossible forged
   contract at the private constructor boundary.
9. Lower aggregate-star explicitly.
10. Lower correlated EXISTS/NOT EXISTS to correlation-carrying SemiJoin/
    AntiJoin; uncorrelated forms use plain SemiJoin/AntiJoin.
11. Remove debug-only structural invariants.

## Required Input/Output Contract

Algebra receives a sealed relation contract, not HIR plus optional side tables:

```rust
pub struct TypedRelationV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    root: TypedRelationExprV2,
    output: RelationOutputContractV2,
}

pub struct RelationOutputContractV2 {
    row: RecordTypeV1,
    cardinality: CardinalityV1,
    provenance: OutputProvenanceV2,
    columns: Vec<OutputColumnContractV2>, // exact row-order alignment
    partition_keys: Vec<SourceQualifiedNodeIdV2>,
    ordering: Vec<OrderKeyContractV2>,
    effects: ExpressionEffectsV2,
}

pub struct LoweredRelationItemV2 {
    item_id: SemanticItemIdV2,
    checked_root: CheckedAlgebraNodeV2,
    output: RelationOutputContractV2,
}
```

Ticket 05 is the only producer of `LoweredRelationItemV2`; ticket 07
consumes this exact type. The lowering entry accepts the sealed
`SemanticDocumentV2` plus an item ID, not an arbitrary expression or subquery
side table: it obtains `TypedRelationV2`, every referenced `TypedExprV2`, and
nested typed EXISTS relations by exact checked ID from that one document.
The normalized root retains the exact input output contract. No caller may
recompute source identities, per-column provenance, schema, or effects from
projections or raw HIR.

`OutputColumnContractV2` retains name, checked producer location, and an
`OutputColumnOriginV2` of either `SourceField { source, scan_stage,
field_name }` or `Expression(typed_expression_id)`. For a self-join, `source`
alone is ambiguous: algebra must preserve the exact `scan_stage` occurrence.
The sema constructor must prove that the scan stage exists under this item,
that its sealed source and field match, and that each expression origin is a
typed member of the same document. A same-source, wrong-scan transplant must
reject before lowering; algebra cannot reconstruct this proof from aliases.

## Step-By-Step Procedure

1. Add private constructors for logical algebra nodes and invariant errors.
2. Add lowering from each sema-owned `TypedRelationExprV2` variant one at a time.
3. Lower Scan/Filter/Project before Join/Group/Aggregate/Sort.
4. Lower Join directly from typed left/right/on/kind.
5. Add explicit Cross validation and reject predicate-free non-Cross joins.
6. Verify ticket 04 emitted `AggregateContextV2` on every typed expression and
   require it in aggregate lowering. If absent, reopen 04; this ticket must not
   edit sema.
7. Consume `AggregateContextV2`; do not add an algebra-side recursive aggregate
   search or a second legality decision.
8. Lower aggregate star separately from expression arguments.
   Only `count(*)` is valid in this packet; `sum(*)`, `min(*)`, `max(*)`, mixed
   `count(*, x)`, and a bare `*` all reject in sema before algebra.
9. Preserve existing group partition when applying aggregate projection.
10. Lower EXISTS/NOT EXISTS only from the explicit typed subquery carrier and
    binder-produced correlation set, retaining that set on correlated nodes.
11. Remove old raw-HIR normalization entry points and debug-only assertions.

## Exact Aggregate Matrix

| Context | Aggregate allowed |
|---|---|
| projection after grouping | yes |
| scalar projection without explicit group | yes, implicit empty partition |
| WHERE | no |
| JOIN ON | no |
| GROUP key | no |
| aggregate argument containing aggregate | no |
| ORDER BY | group key or named aggregate output only after grouping; pre-group source fields reject |
| HAVING | reserved semantic context only; current grammar has no HAVING production |
| recursive rule cycle | owned by ticket 06; every aggregate rejects in a recursive SCC |

Do not add HAVING syntax in this packet. `AggregateContextV2::Having` exists for
the semantic boundary but has no source producer and must be covered only by an
owner unit test. Recursive aggregate calls always reject in this packet; ticket
06 does not admit an exception.

## Test Files

1. `[UPDATE]` `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_normalization.rs` — typed Join/Group/Aggregate/Sort lowering
2. `[UPDATE]` `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_hellgate.rs` — checked constructors and illegal join/output shapes
3. `[UPDATE]` `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_mutants.rs` — predicate/partition/star mutation kills
4. `[UPDATE]` `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_probe.rs` — cardinality, partition, and EXISTS trees
5. `[ADD]` `crates/quanta-gqlang-v2-algebra/tests/v3_typed_lowering.rs` — sealed-input boundary only
6. `[UPDATE]` `crates/quanta-gqlang-v2-pipeline/tests/v3_surface_to_algebra_group_by.rs` and `crates/quanta-gqlang-v2-pipeline/tests/v3_surface_to_algebra_order_by.rs` — delete `_keys` expectations
7. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_relational_semantics_v2.rs` — public semantic-to-algebra cross-stage rail

Add a source-identity-drop mutation case: deleting any scan/correlation source
from the sealed `OutputProvenanceV2` or swapping any row-aligned
`OutputColumnOriginV2` must fail the checked contract and final compiled DTO
assertion.

Move the existing `logic/tests/v3_logic_source.rs` EXISTS→Semi/Anti behavior
into algebra's typed-lowering owner tests when cutting logic's algebra edge in
Ticket 06. Preserve both positive and negative polarity, and add a correlated
capture identity assertion; do not leave a duplicate EXISTS lowering path in
logic. Ticket 06 may then reuse the old file name only for distinct resolved
Rule/Graph adapter tests, not the moved algebra behavior.

Add an exact negative fixture for
`algebra.ill_formed_aggregate.empty_aggregate_projection`. Delete
`sort_direction_keywords_are_exact_stable_strings` together with the
unconsumed `SortDirectionV1::keyword` accessor. Retain the other five algebra
mutant tests: they cover distinct normalization branches. Normalize lint header
text only in touched files after semantic migration.

Do not repeat the existing 12-operator/provenance/cardinality coverage in the
two new boundary files.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-algebra
cargo clippy -p quanta-gqlang-v2-algebra --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
git diff --check
```

`v3_relational_semantics_v2` and existing group/order pipeline tests are
deferred to ticket 07 and enumerated in receipt 05.

## Forbidden Shortcuts

1. accepting both `Filter(Compose(join))` and the new Join carrier
2. inspecting callee strings to determine aggregate identity
3. creating an empty-key Group when a partition already exists
4. exposing public node fields plus a separate `validate()` convention
5. retyping projection expressions during composition or planning

## RED

Capture these against the ticket-05 start commit, after ticket 04 is green:

1. no algebra entry point accepts `TypedRelationV2`
2. `normalize_query` still accepts raw `QueryHirV1`
3. group partition becomes an empty-key scalar group
4. public algebra fields/debug-only checks admit duplicate or malformed output
5. typed relational EXISTS/NOT EXISTS has no SemiJoin/AntiJoin consumer

The original JOIN/aggregate/star failures are base evidence owned by tickets
02/04. Do not reintroduce or duplicate their semantic checks merely to create a
ticket-05 RED.

## Tests

1. JOIN positive, missing-ON negative, explicit-CROSS positive
2. exact logical tree and predicate-origin assertions
3. valid aggregate and aggregate-star lowering from ticket 04's sealed context
4. group-by-with-hidden-key cardinality/partition test
5. correlated and uncorrelated EXISTS/NOT EXISTS trees
6. constructor invalid-state tests
7. one ticket-07 pipeline test proving ticket 04 rejects an illegal aggregate
   before algebra; ticket 05 records it as deferred and keeps detailed
   placement/nesting negatives in sema owner tests

## Mutation Cases

1. drop Join predicate
2. clear group keys during aggregate lowering
3. treat aggregate star as ordinary symbol
4. bypass duplicate output validation

Each mutation must be killed by an algebra owner test; ticket 07 separately
proves the integrated pipeline path.

## Definition Of Done

1. algebra imports semantic contracts, not binder results or raw relation HIR
2. every public algebra value satisfies its invariants in release builds
3. relation output cardinality/partition/schema is never reconstructed later
4. all F01/F04/F06/F07 repros have positive and negative rails

## Dependencies

## 2026-09-21 Final Adversarial Audit Corrections

This section is normative and supersedes conflicting earlier wording.

### Owner boundary and focused proof

- Ticket 05 may reach only `FOCUSED_GREEN`; its owner proof compiles/tests
  `algebra` only. Pipeline relational tests are deferred to ticket 07 and the
  exact deferred failures belong in receipt 05.
- Algebra consumes sema-owned `TypedExprV2`, `AggregateContextV2`,
  `OrderKeyContractV2`, and the exact retained `RelationOutputContractV2`.
  It must not reconstruct schema, provenance, effects, cardinality, aggregate
  placement, nesting, star legality, or call resolution.
- Ticket 04 must first add checked per-column origin aligned to each row
  field and retain nested typed relation roots for EXISTS/NOT EXISTS. The
  current sema source-set `OutputProvenanceV2` and `relation_root` ID alone
  cannot supply `algebra::ColumnProvenanceV1` or a Semi/Anti right input.
  Reopen the sema producer rather than manufacturing columns or reading HIR
  in the algebra lowerer. Ticket 05 tests missing/extra/wrong-key expression,
  swapped column origin, missing nested subrelation, and altered capture edge.
- Delete the earlier instruction to update planning in step 11. Planning is a
  ticket 07 consumer and is read-only here.

### Exact checked algebra contract

`CheckedAlgebraNodeV2` is algebra-owned, has private fields, retains one
`SemanticItemIdV2`, and is constructible only by the lowering transaction.
There is no `TypedRelationNodeV2`, no `TypedExprIdV2`, and no second call-target
type. Per-item output is:

```text
LoweredRelationItemV2 {
  item_id: SemanticItemIdV2,
  root: CheckedAlgebraNodeV2,
  output: RelationOutputContractV2,
}
```

The lowering transaction returns an exact-key map and rejects duplicate output
keys before insertion. Ticket 07 compares the produced key set with the
applicable semantic-item key set; order-based zip is forbidden.

### Correlation-preserving EXISTS

Uncorrelated EXISTS/NOT EXISTS may lower to plain SemiJoin/AntiJoin. Correlated
forms must retain a non-empty checked `CorrelationSetV2` on the semi/anti node
(or an equivalent `ApplyKindV2::{Semi, Anti}` node); a plain node without that
carrier is invalid. Each correlation entry is a resolved outer binding identity
plus the inner reference location. Empty, duplicate, foreign-scope, or dropped
correlation sets reject.

Required exact-tree oracles are separate for uncorrelated EXISTS, correlated
EXISTS, uncorrelated NOT EXISTS, and correlated NOT EXISTS. Add a mutation rail
that deletes one correlation entry and proves the contract validator fails.

### Consumer migration inventory

Migrate `algebra/src/normalize.rs` from private HIR fields and removed variants.
Do not patch around them with public compatibility fields. Record zero matches
for `.items`, `.segments`, `.args`, `RelationExprKindV1::Compose`, and
`BoolExprKindV1::Quantifier` outside their owners. The group/order pipeline
tests are migrated and executed only during ticket 07 integration.

### Inherited external delta audit (`fcf106b..f9ec954`)

Commit `6583091` is an **UNQUALIFIED PARTIAL CONSUMER MIGRATION** and is a
delete/rewrite input, not Ticket 05 progress. `cargo check -p
quanta-gqlang-v2-algebra --lib` passes, but all four algebra test crates fail to
compile with 74 errors. The source still exposes raw-HIR `normalize_query` and
`normalize_relation`, globally precollects aliases with replacement ignored,
identifies aggregates by callee strings, and recomputes schema/cardinality/
provenance. A private `CompositionOpCompatV1`/`CompositionV1` plus dead
string-dispatch normalizer duplicates the deleted language shape behind bare
`#[allow(dead_code)]`; delete it rather than adding allow reasons.

The cutover must additionally prove:

1. public algebra accepts only `TypedRelationV2`; no raw HIR plus boolean or
   caller assertion may stand in for binder/sema proof;
2. aggregate Star remains an explicit carrier and never collapses to a
   zero-argument call;
3. `group by ... | project ... aggregate` retains the sealed partition, and
   `project ... aggregate | group by ...` follows Ticket 04's pinned stage
   contract without producing a second accidental Group;
4. The six source-reachable sema relation stages lower through one exhaustive
   enum match. Algebra then derives and validates its full 12-operator family
   programmatically; it must not pretend all 12 have source HIR variants.
   Join provenance accepts the sema sum identity over host
   `RelationSourceIdV2` and modules `CatalogAssetIdV2` (including local/import
   View) without lossy string conversion;
5. the direct HIR dependency, compat carrier, dead dispatcher, alias
   precollection, and string aggregate search have zero definitions;
6. owner strict Clippy is green with no bare allow attributes.

Current evidence is deliberately split: algebra library check green; owner
tests red (74 compiler errors across 4 suites); downstream pipeline red (25
compiler errors); strict planning/binder Clippy invocation red because the
algebra compat block emits 6 denied allow-attribute errors. Receipt 05 must
record all four outcomes and may not promote the library-only check.

Requires 04. May run in parallel with 06 after sema contracts are frozen.

## 2026-09-21 Ticket 04 Producer/Consumer RR Delta

- Freeze a `TypedRelationV2` with exactly six source HIR stage variants before
  implementing this consumer. A 12-variant sema enum is a wrong producer, not
  a reason for algebra to support unreachable syntax. This ticket owns
  the six existing non-stage algebra operators Apply, Aggregate, Limit,
  Union, SemiJoin, and AntiJoin and their checked constructors;
  source-reachability claims are
  limited to grammar-backed stages and EXISTS lowering.
- The sema output proof must include exact row schema, source-sum provenance,
  cardinality, partition, ordering, effects, aggregate placement, and explicit
  correlation. Algebra may reject a semantically unsupported combination but
  may not recompute or overwrite these fields to make it pass.
- Add distinct stage-order tests for `group by ... | project ... aggregate`,
  `project ... aggregate | group by ...`, and `order by` after projection.
  Pin whether a projection preserves or invalidates partition keys by exact
  projected expression identity, not by name; a dropped key cannot survive.
  Assert no accidental second Group node and no unqualified aggregate after
  a stage that invalidates its grouping proof.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954`)

No Ticket 05 implementation is accepted yet. The inherited algebra library
check succeeds, but the four existing algebra test targets fail compilation
with 74 errors. `normalize_query(&QueryHirV1, ...)` and
`normalize_relation` remain raw-HIR entry points. Preserve the one 12-op
algebra/normalizer; delete its dead composition compatibility representation
and migrate the input, rather than adding a second V2 tree.

Implementation handoff is blocked until Ticket 04 freezes a non-forgeable
six-stage `TypedRelationV2` and exact output/provenance/correlation proof. The
owner task is then one exhaustive lowering match plus checked constructors for
programmatic-only operators. Record zero raw-HIR relation dependency and zero
`CompositionOpCompatV1`/string aggregate dispatch; run all four migrated owner
suites, the new sealed-input boundary suite, strict Clippy, and mutation rails.
The pipeline group/order/EXISTS tests remain deferred to Ticket 07, not
silently counted in Ticket 05's focused result.

## 2026-09-21 Exact Algebra Handoff Audit

Read-only source audit of the inherited algebra identified one existing
12-operator owner (`algebra/src/node.rs:177-332`) and one raw-HIR normalizer
(`algebra/src/normalize.rs:40-133`). Preserve the operators, not the unsafe
input. The exhaustive source mapping is Scan→Scan, Filter→Filter,
Project→Project or typed Aggregate over retained/implicit Group,
Join→Inner/Cross Join, GroupBy→Group, OrderBy→Sort. Aggregate, SemiJoin, and
AntiJoin are derived by typed source semantics; Apply, Limit, and Union remain
checked programmatic-only operators. No seventh source stage is authorized.

Delete/migrate in one cut: `NormalizeOptionsV1` and `normalize_query`/
`normalize_relation` raw-HIR entry; global `collect_aliases` and string source
authority (`normalize.rs:30-205`); dead `CompositionOpCompatV1`/
`CompositionV1` and string dispatch (`:143-155,451-972`); `_keys`,
Filter-derived ON, string aggregate classifier/fold (`:973-1065`), and raw
string provenance/correlation walkers (`:1066-1314`). `node.rs:177-225`
public fields/debug-only checks become checked private construction. Replace
`node.rs:30-110` string source authority and `:355-548` raw HIR expression/
predicate fields only after sema provides exact typed facts. Avoid a second
algebra tree or compatibility mode.

Existing owner-test disposition is by behavior, not deletion count:

- `v3_algebra_normalization.rs`: migrate the eight source cases to sealed
  Scan/Project/Join/Group/Sort and checked programmatic Apply/Limit paths.
- `v3_algebra_hellgate.rs`: migrate source/string authority cases to sema
  identity; delete only unsupported composition/arity tests made impossible
  by closed stage enums; retain Apply/Limit/Union/Join/Cross constructor
  negatives. Nested aggregate legality moves to sema; algebra keeps a forged
  contract guard.
- `v3_algebra_mutants.rs`: retain the 12-family vocabulary test and migrate
  projection/ON/correlation/DESC mutations; delete the string sort-keyword
  stability test only after typed direction coverage replaces it.
- `v3_algebra_probe.rs`: retain scalar Group+Aggregate, Union positional
  provenance, and distinct Apply origin cases on checked inputs.

The test migration includes `v3_typed_lowering.rs` for missing/extra/wrong
semantic expression IDs, swapped column origin, missing nested EXISTS right
tree, correlation deletion, partition drop, and aggregate Star. Pipeline
`v3_surface_to_algebra_group_by.rs`, `...order_by.rs`, and the omitted
`v3_relation_source_authority.rs` are explicit Ticket 07 consumer migration;
their V1 field accesses are not Ticket 05 focused-green evidence. Making
algebra node fields private also breaks direct logic/planning consumers;
Ticket 06/07 must migrate those callers in the same atomic stack before
workspace compilation is expected.

## Final Producer/Consumer Contract Delta — Group And Sort

The live Binder GroupBy frame currently leaks the entire pre-group row, while
the algebra Group output is key-only. Ticket 04 owns the Binder correction
and separate aggregate-only partition-input authority. Do not compensate in
this lowerer by copying the input columns or by guessing `sum` from its text.

The lowerer shall accept one sealed key-only Group output with ordered
`(group stage ID, key ordinal)` identities, exact typed key-expression IDs,
partition-input proof, Set cardinality, and aligned `OutputColumnContractV2`
origins. It rejects a missing/extra/swapped key, an input-row column leaking
into public Group output, and an aggregate whose argument provenance is not
authorized by that partition proof. For Group→aggregate Project, preserve the
original partition keys and emit exactly one Group plus one typed Aggregate;
for scalar aggregate Project, use the separately sealed implicit empty
partition. No second Group is synthesized from a callee spelling.

Ticket 00 pins both ascending and descending source OrderBy to `NULLS LAST`.
Sema owns `OrderKeyContractV2::{expression,direction,null_order,value_type}`;
algebra preserves those four checked fields and creates a Sort over the
already-sealed input output schema. If the existing `SortKeyV1` cannot retain
null placement, extend its canonical node once; do not drop the policy or add
an algebra-only default. Exact-tree tests cover ASC/DESC on nullable keys,
multi-key order, order after Project alias, and failure when a dropped
pre-Project/Group field is referenced.
