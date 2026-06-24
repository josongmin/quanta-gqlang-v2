# 04 Relational Algebra And Normalization

## Goal

Extend current query lowering to full normalized relational algebra.

## Why

Current IR is root-carrier oriented. It cannot represent join/subquery/grouping
as first-class algebra and therefore cannot scale to advanced query semantics.

Current head already ships `quanta-gqlang-v2-algebra` plus pipeline lowering
into normalized algebra for part of the surface. This ticket extends that owner
lane to advanced closure; it does not authorize a second normalization stack.

## Deliverables

1. algebra nodes:
   - `Scan`
   - `Project`
   - `Filter`
   - `Join`
   - `Apply`
   - `Group`
   - `Aggregate`
   - `Sort`
   - `Limit`
   - `Union`
   - `SemiJoin`
   - `AntiJoin`
2. normalization passes from HIR query shapes to algebra
3. canonicalization rules
4. set/bag/cardinality semantics for algebra operators
5. stable diagnostics for unsupported algebra states

## Required Semantics

1. explicit column provenance
2. alias-aware field resolution
3. grouping keys vs aggregate expressions separation
4. sort/limit semantics independent of rendering
5. correlated/lateral subquery normalization without root-local side channels
6. duplicate-elimination behavior explicit per operator family

## Non-Goals

1. `exists` / `not exists` source semantics do not live here; this ticket owns
   the algebra target shape they lower into.
2. recursion legality and stratification do not live here.
3. planning capability/truth/effect derivation does not live here.

## Scenarios Unlocked

1. multi-table joins
2. grouped risk summaries
3. top-k ranking queries
4. reusable subquery fragments expressed as algebra
5. correlated subqueries and nested aggregate lookups without syntax-local hacks
6. duplicate-sensitive aggregates with explicit multiplicity semantics

## Definition Of Done

1. `SELECT` no longer lowers directly to a narrow table/filter/limit IR
2. join/group/order/having/subquery shapes can be represented without ad-hoc
   side channels
3. normalization rails cover provenance, grouping, and alias edge cases
4. correlated/lateral forms lower into explicit algebra nodes rather than
   planner-only heuristics
5. set/bag semantics are explicit enough that duplicate-sensitive behavior is
   predictable in tests and planning

## Dependencies

1. requires `00`
2. depends on `01`, `02`, and `03`
3. must not pull recursion legality or planner truth rules from `05`/`07`

## Likely Touch Points

1. `crates/quanta-gqlang-v2-algebra/src/node.rs`
2. `crates/quanta-gqlang-v2-algebra/src/normalize.rs`
3. `crates/quanta-gqlang-v2-algebra/src/diagnostic.rs`
4. `crates/quanta-gqlang-v2-pipeline/src/compile.rs`

## Mandatory Worker Outputs

1. explicit algebra node family with correlated/lateral representation
2. documented set/bag behavior per operator family
3. provenance-preserving normalization path from shared HIR into algebra

## Proof Rails

1. extend `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_normalization.rs`
2. extend `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_hellgate.rs`
3. extend `crates/quanta-gqlang-v2-pipeline/tests/v3_surface_to_algebra_group_by.rs`
4. extend `crates/quanta-gqlang-v2-pipeline/tests/v3_surface_to_algebra_order_by.rs`
5. add at least one correlated subquery rail and one duplicate-sensitive
   aggregate rail

## Reject Or Escalate

1. source-level `exists` or recursion legality policy:
   belongs to `05`
2. planner capability/truth/effect derivation:
   belongs to `07`
3. syntax-only sugar changes that skip normalized algebra ownership
