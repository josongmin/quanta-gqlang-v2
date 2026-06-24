# 01 HIR SSOT And Generic Expression Kernel

## Goal

Stabilize and close the typed-HIR-facing semantic layer shared across query,
declaration, rule, and composition surfaces.

## Why

Current architecture has no generic expression authority. Root-local carriers
encode semantics per root and prevent reusable binding, typing, and
normalization.

Current head already ships `quanta-gqlang-v2-hir` plus `surface`->HIR lowering.
This ticket extends that owner lane to closure; it does not authorize a second
HIR stack.

## Deliverables

1. one singular HIR SSOT remains authoritative
2. `ValueExprHirV1`
3. `BoolExprHirV1`
4. `RelationExprHirV1`
5. `QueryHirV1`
6. `DeclHirV1`
7. `RuleHirV1`
8. `ResultShapeHirV1`
9. typed node ids / spans / source mapping hooks

## Minimum Expression Surface

1. literals
2. symbol references
3. field/path access
4. unary operators
5. binary operators
6. function calls
7. parenthesized expressions
8. projection items with aliases
9. nested record/list shape construction through shared HIR nodes

## Non-Goals

1. join semantics
2. recursion semantics
3. planner integration
4. closed type universe or builtin/operator legality

Typed HIR here means shared node shapes plus typed attachment points. The
concrete type lattice, builtin registry, and overload rules remain owned by
`03`.

Those belong to later tickets. This ticket only establishes the shared HIR
shapes.

`ResultShapeHirV1` is the only frontend-owned structural authority for nested
result shaping. Later tickets may type, normalize, or classify that shape, but
may not replace it with a second ad-hoc tree.

## Scenarios Unlocked

1. computed projection
   - `SELECT metric.test_failures + metric.test_errors AS risk`
2. reusable boolean fragments
   - `active = true AND severity = "high"`
3. declaration-local typed expressions
   - view/rule surfaces no longer need root-specific scalar carriers
4. nested result shaping
   - projected child records/explain trees share one HIR structure across CLI,
     LSP, and planning

## Definition Of Done

1. no new advanced surface is added directly to root-local `v2` typed structs
2. shared HIR nodes exist and are test-covered
3. surface nodes target HIR construction without bypassing into ad-hoc
   algebra/planning side channels
4. nested result shaping has one shared HIR representation rather than
   per-surface custom structs

## Dependencies

1. requires `00`
2. should land before `02` through `08`

## Likely Touch Points

1. `crates/quanta-gqlang-v2-hir/src/item.rs`
2. `crates/quanta-gqlang-v2-hir/src/expr.rs`
3. `crates/quanta-gqlang-v2-hir/src/relation.rs`
4. `crates/quanta-gqlang-v2-hir/src/shape.rs`
5. `crates/quanta-gqlang-v2-hir/src/unit.rs`
6. `crates/quanta-gqlang-v2-surface/src/parser.rs`

## Mandatory Worker Outputs

1. one shared HIR authority for value/bool/relation/query/decl/rule/result-shape
   nodes
2. migration inventory of any remaining root-local typed carriers that still
   bypass HIR
3. at least one nested result-shape fixture proving `ResultShapeHirV1` is real
   and shared

## Proof Rails

1. crate-local HIR construction tests with span/node-id coverage
2. extend `crates/quanta-gqlang-v2-hir/tests/v3_hir_expr_kernel.rs`
3. extend `crates/quanta-gqlang-v2-hir/tests/v3_item_hellgate.rs`
4. extend `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_kernel.rs`

## Reject Or Escalate

1. planner truth/effect logic work:
   belongs to `07`
2. closed type or overload legality work:
   belongs to `03`
3. any shortcut that reintroduces root-local typed structs for new surfaces
