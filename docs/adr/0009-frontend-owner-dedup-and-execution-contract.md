# ADR-0009: Frontend common owners and execution-contract extraction

- Status: Accepted
- Date: 2026-10-10
- Source observation: QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`
- Scope: completed D01–D07 / QFC-01, QFC-03, QFC-06 source work

## Context

The Oct-10 frontend closeout packet mixed completed implementation, successive
focused runs, old qualification failures and one pending downstream pairing gate.
Its four files no longer define an execution queue. Current code/contracts own
behavior; this ADR retains the decisions and their compatibility boundaries.

## Decisions and current owners

| ID | Retained decision and owner | Preserved invariant |
| --- | --- | --- |
| D01 | Modules [catalog vocabulary](../../crates/quanta-gqlang-v2-modules/src/catalog.rs) declares the namespace mapping once; checked/input methods share its exhaustive macro | Keep checked `const` API, all 15 variants and separate payload validation/diagnostic order; no dynamic validator |
| D02 | Modules [lockfile](../../crates/quanta-gqlang-v2-modules/src/lockfile.rs) owns one private canonical writer for `LockfileV2` and `VerifiedLockSetV2` | Sorted pins, exact spaces/version/digest/newline, empty output and existing byte contract |
| D03 | Modules [indexed SCC](../../crates/quanta-gqlang-v2-modules/src/graph/indexed_scc.rs) is the borrowed iterative partition kernel for Modules and Logic | Caller-owned discovery/emission charges, source locations, witness order and typed first refusal; checked scratch accounting. Semantica Core SCC is a separate owner |
| D04 | Modules [indexed DAG walk](../../crates/quanta-gqlang-v2-modules/src/graph/indexed_walk.rs) shares preflight and cursor FIFO; [View topology](../../crates/quanta-gqlang-v2-modules/src/catalog/view_topology.rs) uses a preallocated canonical min-heap | Root multiplicity, dangling-edge charge, overflow/cycle refusal, deterministic minimum-ready order and request scratch bounds; no repeated ready-Vec shifting |
| D05 | Logic [recursion](../../crates/quanta-gqlang-v2-logic/src/recursion.rs) retains canonical dense membership for scan/BFS/strata and borrows two offending minima | One edge-order owner; charge every edge, preserve first tie and negation-before-aggregate, reject malformed internal indices |
| D06 | Lock parsing borrows at most four field slices and checks validated input order after all lines; serialization keeps the D02 writer | No input-width field Vec, order-only name clones or whole-text reconstruction; late invalid field/duplicate precedes ordering errors with exact spans |
| D07 | Internal [execution-contract](../../crates/quanta-gqlang-v2-execution-contract/Cargo.toml) owns DTOs, projection and canonical encoding; Pipeline retains compiler orchestration, error bridges and request permits | Public facade remains the only downstream seam. Private constructors, same item closure/order/bytes/digests, source schema 3 and contract/qualification schema 8 remain intact |

DAG direction is facade → Pipeline + execution-contract; Pipeline →
execution-contract; execution-contract → Algebra/Logic/Planning/Sema. The
internal contract has no normal Pipeline/Surface dependency. Pipeline's
`execution.rs` contains compatibility imports, not a second DTO implementation.
The facade directly reexports contract DTOs. Scalar move-only projection keeps
the existing request-root reservations and drop chain without another conversion
buffer; SHOW and ordinary/lexical compilation use the same projection owner.
Rust definition paths moved, so downstream source closures must be rebuilt.

Record/Union validators retain distinct payload and diagnostic owners. Similar
token bodies alone do not justify an erased validator or another generic engine.
No new external runtime, optimizer, parser or solver was selected.

## Evidence boundary

The retired packet records separate focused results: Modules authority 122,
Logic 84, execution-contract/Pipeline 171 Rust tests + 3 doctests, and external
consumer policy 24. Its graph oracles include 16,384 small DAG cases, actual
production exact/one-short work, 50k depth/width, canonical permutation/tie and
request-storage/refund controls. These are distinct original-source scopes.
AST token census excludes macros and unsampled/small/structurally different
duplicates; it is not a repository-wide zero-duplication proof. Allocator/RSS
and downstream performance were not measured.

Incoming clean source `25fad76cf5b4510244157c2e009b4dc37aa958ff` has a schema-3
local qualification receipt with matching SHA/tree, `dirty=false`, exit 0 and
1,529 executed/passed Nextest cases, zero skips/leaks. Receipt, CI log, raw
inventory/events and source-lock hashes were read back together. This is that
committed frontend's result; subsequent documentation edits require their own
local CI and do not inherit clean-HEAD qualification.

QFC-04/QFC-05 are receipt-derived gates, not tracked completion checkboxes.
Use [qualification](../../tools/qualification/README.md) for the exact current
committed source. No frontend receipt supplies Semantica consumer/product proof.

## Remaining pairing and recovery

QFC-02 remains with the existing Semantica E13 owner: reconcile the old complete
bootstrap revision, current producer source closure (including execution-contract),
subtree/vector policy and actual consumer compile/contract-vector proof together.
Changing expected pins alone does not close that gate. It is independent of
standalone frontend qualification and the Proposed V4 roadmap.

[Remaining work](../work/remaining.md) owns that acceptance. The retired four-file
packet, exact original commands, failure receipts and input hashes are recoverable
at [ADR-0001's Oct-10 baseline](0001-frontend-campaign-consolidation.md#2026-10-10-reconciliation).
Local `.audit/oct10-frontend-closeout/` evidence retains the focused logs; recovery
copies do not issue new execution results.
