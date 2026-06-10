# 03 IR: `CompositionIrV1` And `WorkingSetV1` Wiring

Status: `Done` — IR crate green (4 composition tests + full suite), clippy/fmt clean. `WorkingSetV1` now has its first live producer; `WorkingSetRefIrV1` has no items field (no runtime-packet widening by construction); `DegradedReasonV1::CompositionMaterializationUnavailable` added + round-trips; body degraded lane preserved. Note: `composition_ir` placed flat (matches IR's flat module layout) rather than under `interactive/`; consumed via full `ir::composition_ir::` path (frontend re-exports syntax/sema-owned types only).

## Goal

Lower compositions to normalized public IR and, in doing so, give the
production-dead `WorkingSetV1` subsystem its first live producer (`LET`).
Extend the degraded-reason single authority with the composition variant.

## Depends On

1. `02-sema-typed-binding-graph`

## Unblocks

1. `04-cli-composition-surface`
2. `05-lsp-composition-surface`

## Current Base

1. `crates/quanta-gqlang-v2-ir/src/working_set.rs` exists, validated, and is
   referenced only by `tests/ai_profile_contract.rs` (production-dead — this
   ticket wires it)
2. `DegradedReasonV1` in `root_ir.rs` is the single degraded authority; every
   code ends `-unavailable` (enforced by the no-runtime-leakage sweeps)

## Owner Files

1. `crates/quanta-gqlang-v2-ir/src/interactive/composition_ir.rs` (new)
2. `crates/quanta-gqlang-v2-ir/src/root_ir.rs` (enum variants only)
3. `crates/quanta-gqlang-v2-ir/src/lib.rs` (lowering arm + module decl)
4. `crates/quanta-gqlang-v2-ir/src/frontend.rs` (re-export the new public types)
5. `crates/quanta-gqlang-v2-ir/tests/composition_ir.rs` (new)

## Do Not Edit

1. `working_set.rs` type shapes (consume as-is; it already validates)
2. existing lowering arms
3. explain presentation, planning, facts modules

## Core Logic

```rust
pub struct WorkingSetRefIrV1 {
    pub working_set_id: String,          // the binding name — stable public id
    pub kind: WorkingSetKindV1,          // CandidateSet (total: all D3 producers)
    pub basis: WorkingSetBasisV1,        // from the typed basis, total map
}

pub struct CompositionIrV1 {
    pub bindings: Vec<(WorkingSetRefIrV1, Box<PublicIrRootV1>)>, // ref + lowered producer
    pub narrowed_by: usize,              // index into bindings
    pub body: Box<PublicIrRootV1>,
}
```

1. `PublicIrRootV1` gains `Composition(CompositionIrV1)`
2. `DegradedReasonV1` gains `CompositionMaterializationUnavailable` with code
   `composition-materialization-unavailable`; `for_ir_root_v1` maps the new
   variant to it (total match — the compiler enforces the new arm)
3. each binding's producer and the body lower through the existing per-root
   lowering (reuse, never duplicate); their own degraded reasons are
   preserved on the composed node, not swallowed
4. the `WorkingSetRefIrV1` is a *descriptor*, not a materialized set: it has
   no items field. The full `WorkingSetV1` (with items) remains the host-side
   materialization shape; the ref carries exactly the identity + basis a host
   needs to materialize later. This keeps DoD line "no composition node
   widens into direct runtime packet injection" true by construction.

## DoD

1. `tests/composition_ir.rs` proves: lowering happy path, degraded reason
   code + `-unavailable` suffix, producer/body degraded preservation, and
   `WorkingSetRefIrV1` field totality per D3 basis mapping
2. existing IR rails (`public_planning_ir`, `degraded_reason`, j01/j02) green
   unchanged

## Proof Rail

`cargo test -p quanta-gqlang-v2-ir`
