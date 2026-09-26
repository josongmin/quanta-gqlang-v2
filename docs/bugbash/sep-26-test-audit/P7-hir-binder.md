# P7: current HIR and binder proof gaps

No production failure was observed. The round-2 fixes to depth-child sentinels, HIR error payloads, and binder diagnostics are present in the working tree.

## TA-09: auxiliary spans are not traced across the visitor families

- `crates/quanta-gqlang-v2-hir/src/visit.rs` forwards auxiliary spans from relation aliases, projection aliases, result-shape fields, presentation selectors, graph references, and composition binding names. `src/integrity.rs:658` consumes this visitor hook to validate spans.
- `tests/v3_visit_traversal.rs:43-113` compares an exact span trace for six catalog payloads. The other listed families lack a corresponding exact auxiliary-span trace; count-only node walks do not detect a removed span callback.
- Repair: add small per-family `AuxiliarySpanTrace` fixtures with exact authored order and an invalid-span rejection for at least one previously uncovered branch. Removing a required callback must fail.

## TA-10: an immediate noncapturing shadow barrier lacks a control

- `crates/quanta-gqlang-v2-binder/src/scope.rs:198-201` returns `None` when the current frame's parent edge is `NonCapturing`. The existing `v3_binder_mutants.rs:216-242` barrier test places the noncapturing edge on a middle frame; it cannot detect deletion of the early top-frame guard.
- Repair: put `x` in an outer frame, push a directly `NonCapturing` child, bind another `x`, and require no outer-shadow report. Pair it with a `Capturing` child that must report `OuterShadow`. Removing the early guard must fail.

The old claim that there is no positive shadow or open-scope control anywhere in the crate is not retained: `v3_binder_hellgate.rs:93-116` checks outer shadow through `Binder`, and `v3_binder_scope.rs:321-344` checks an open-scope local resolution.
