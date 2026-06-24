# 03 Type Universe And Operator Function Registry

## Goal

Complete the closed type system and operator/function registry for `v3`.

## Why

Expression power without a typed operator system degenerates into ad-hoc
runtime guessing. This repo must own type truth at the frontend boundary.

Current head already ships `quanta-gqlang-v2-types` as the owner lane. This
ticket extends and hardens that lane; it does not authorize a second type
checker or operator registry.

## Deliverables

1. scalar type universe
2. structural value types:
   - tuples
   - records
   - optionals
   - lists/sets
3. algebraic datatypes and type unions
4. semantic reference/domain types
5. nullability / optionality rules
6. relation/set/cardinality typing primitives
7. location/path/witness/proof value families
8. closed operator registry
9. closed builtin function registry
10. overload resolution rules
11. function metadata:
    - purity
    - determinism
    - monotonicity
    - aggregation class
    - cost class
12. typed diagnostics for mismatches

## Initial Type Families

1. `bool`
2. `int`
3. `float`
4. `string`
5. `duration`
6. `location`
7. `path`
8. `entity_ref`
9. `symbol_ref`
10. `rule_ref`
11. `source_scope`
12. `proof_tree`

## Structural Type Families

1. tuple and record literals/results
2. optional and union-typed values
3. collection/container values
4. witness/path/proof payload values
5. generic relation row shapes
6. nested result shape values with stable field typing

## Initial Builtin Families

1. arithmetic
2. comparison
3. boolean
4. aggregate
5. string predicates
6. null/default helpers
7. collection predicates
8. tuple/record projection helpers
9. path/witness helpers
10. proof/explain helpers

## Non-Goals

1. this ticket does not define HIR node taxonomy
2. this ticket does not own scope/binding rules
3. this ticket consumes resolved HIR/binder outputs rather than duplicating them
4. this ticket does not lift relation-source schemas into alias/local envs; the
   binder->types closure seam is ticket `13`

## Scenarios Unlocked

1. aggregate expressions
   - `count(*)`, `sum(metric.test_failures)`
2. typed string filters
   - `starts_with(path, "src/")`
3. typed conditional expressions
   - `case when severity = "critical" then 100 else 10 end`
4. typed path/explanation payloads
   - witness rows carry locations and path segments without stringly blobs
5. purity-aware builtin admission
   - non-deterministic or non-monotone functions are visible to later planning
     and recursion checks
6. nested result typing
   - projected child records and explain trees keep exact field shapes

## Definition Of Done

1. typed HIR nodes resolve to closed frontend-owned types
2. operator/function lookup does not depend on backend/provider state
3. aggregate/function/type mismatch rails are green
4. structural/container/path result values are first-class, not encoded as
   ad-hoc strings
5. builtin metadata is rich enough for recursion legality, optimization, and
   cost diagnostics downstream
6. proof/explain payloads and nested result structures have closed typed shapes

## Dependencies

1. requires `00`
2. depends on `01`
3. consumes binder outputs from `02`
4. provides type carriers consumed by `12` and `13`

## Likely Touch Points

1. `crates/quanta-gqlang-v2-types/src/ty.rs`
2. `crates/quanta-gqlang-v2-types/src/op.rs`
3. `crates/quanta-gqlang-v2-types/src/builtin.rs`
4. `crates/quanta-gqlang-v2-types/src/check.rs`
5. `crates/quanta-gqlang-v2-types/src/env.rs`
6. `crates/quanta-gqlang-v2-hir/src/shape.rs` only if shared typed shape
   carriage changes

## Mandatory Worker Outputs

1. one closed type universe with no backend-dependent lookup
2. one closed builtin/operator registry with metadata fields used later by `05`
   and `07`
3. typed reject codes for mismatch, bad overload, and illegal aggregate usage

## Proof Rails

1. extend `crates/quanta-gqlang-v2-types/tests/v3_types_universe.rs`
2. extend `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs`
3. add at least one fixture for nested result typing and one for path/proof
   payload typing

## Reject Or Escalate

1. scope or import resolution logic:
   belongs to `02`
2. recursion legality policy:
   belongs to `05`
3. planner cost classification or result rendering:
   belongs to `07`/`08`
4. local alias/source schema lifting and env-closure policy:
   belongs to `13`
