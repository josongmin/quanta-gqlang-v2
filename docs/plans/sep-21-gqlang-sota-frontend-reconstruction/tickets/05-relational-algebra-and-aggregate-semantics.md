# 05 — Relational Algebra And Aggregate Semantics

Status: `PLANNED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

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

## Implementation

1. Change normalizer input from raw Query HIR to `TypedRelationV2`.
2. Make algebra node fields private and constructors fallible where needed.
3. Lower explicit Join HIR directly; ON is never recovered from a Filter.
4. Permit predicate-free joins only for explicit CROSS.
5. Consume semantic output schema, source IDs, and cardinality directly.
6. Preserve GROUP partition keys through aggregate projection.
7. Consume ticket 04's `AggregateContextV2` for projection, predicate, join,
   group key, ordering, HAVING, nested-call, and recursive-rule contexts.
8. Reject nested aggregate calls.
9. Lower aggregate-star explicitly.
10. Lower correlated EXISTS/NOT EXISTS to SemiJoin/AntiJoin using the binder's
    explicit correlation set.
11. Remove debug-only structural invariants.

## Required Input/Output Contract

Algebra receives a sealed relation contract, not HIR plus optional side tables:

```rust
pub struct TypedRelationV2 {
    root: TypedRelationNodeV2,
    output: RelationOutputContractV2,
    span: SpanV1,
}

pub struct RelationOutputContractV2 {
    row: RecordTypeV1,
    cardinality: CardinalityV1,
    relation_sources: BTreeSet<RelationSourceIdV2>,
    provenance: OutputProvenanceV2,
    partition_keys: Vec<TypedExprIdV2>,
    ordering: Vec<OrderKeyContractV2>,
    effects: ExpressionEffectsV2,
}

pub struct CheckedRelationalContractV2 {
    checked_root: CheckedAlgebraNodeV2,
    output: RelationOutputContractV2,
}
```

Ticket 05 is the only producer of `CheckedRelationalContractV2`; ticket 07
consumes this exact type. The normalized root owns the same output contract or a
mechanically derived equivalent. No caller may recompute source identities,
provenance, schema, or effects from projections or raw HIR.

## Step-By-Step Procedure

1. Add private constructors for logical algebra nodes and invariant errors.
2. Add lowering from each `TypedRelationNodeV2` variant one at a time.
3. Lower Scan/Filter/Project before Join/Group/Aggregate/Sort.
4. Lower Join directly from typed left/right/on/kind.
5. Add explicit Cross validation and reject predicate-free non-Cross joins.
6. Verify ticket 04 emitted `AggregateContextV2` on every typed expression and
   require it in aggregate lowering. If absent, reopen 04; this ticket must not
   edit sema.
7. Add recursive aggregate search so nested aggregate calls always reject.
8. Lower aggregate star separately from expression arguments.
   Only `count(*)` is valid in this packet; `sum(*)`, `min(*)`, `max(*)`, mixed
   `count(*, x)`, and a bare `*` all reject in sema before algebra.
9. Preserve existing group partition when applying aggregate projection.
10. Lower EXISTS/NOT EXISTS only from the explicit typed subquery carrier and
    binder-produced correlation columns.
11. Migrate planning consumers to the checked algebra type before deleting raw
    public fields.
12. Remove old raw-HIR normalization entry points and debug-only assertions.

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
from `relation_sources` must fail the checked contract and final compiled DTO
assertion.

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
cargo test -p quanta-gqlang-v2-pipeline --test v3_relational_semantics_v2
cargo clippy -p quanta-gqlang-v2-algebra -p quanta-gqlang-v2-pipeline --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
git diff --check
```

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
7. one pipeline test proving ticket 04 rejects an illegal aggregate before
   algebra; detailed placement/nesting negatives remain in sema owner tests

## Mutation Cases

1. drop Join predicate
2. clear group keys during aggregate lowering
3. treat aggregate star as ordinary symbol
4. bypass duplicate output validation

Each mutation must be killed by an owner or pipeline test.

## Definition Of Done

1. algebra imports semantic contracts, not binder results or raw relation HIR
2. every public algebra value satisfies its invariants in release builds
3. relation output cardinality/partition/schema is never reconstructed later
4. all F01/F04/F06/F07 repros have positive and negative rails

## Dependencies

Requires 04. May run in parallel with 06 after sema contracts are frozen.
