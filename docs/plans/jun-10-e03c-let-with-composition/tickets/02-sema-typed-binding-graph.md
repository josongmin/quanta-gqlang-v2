# 02 Sema: Typed Binding Graph And Fail-Closed Resolution

Status: `Done` — sema crate green (5 composition tests + full suite), clippy/fmt clean. Alias resolves to index; D3 allow-set is a total match; producer errors pass through unmasked; `CompositionGatingV1::NotYetGated` carries D1.

## Goal

Type the composition node: resolve every `WITH` alias to a declared binding
by index, enforce the D3 producer allow-set, and emit the typed node the IR
lowers from — all total, no default arms, no string-keyed lookups surviving
into the typed result.

## Depends On

1. `01-syntax-composition-statements-and-grouping`

## Unblocks

1. `03-ir-composition-and-working-set-wiring`

## Owner Files

1. `crates/quanta-gqlang-v2-sema/src/interactive/composition.rs` (new)
2. `crates/quanta-gqlang-v2-sema/src/lib.rs` (dispatch arm + module decl)
3. `crates/quanta-gqlang-v2-sema/src/lib_tests.rs` (wiring tests)

## Do Not Edit

1. existing root analyzers
2. syntax crate (its contract is frozen by 01)

## Core Logic

```rust
pub enum CompositionGatingV1 { NotYetGated }          // D1 carrier

pub struct TypedLetBindingV1 {
    pub name: String,
    pub producer: Box<TypedRootV1>,                   // analyzed, not re-parsed
    pub basis: TypedWorkingSetBasisV1,                // total map per D3
}

pub struct TypedCompositionV1 {
    pub bindings: Vec<TypedLetBindingV1>,
    pub with_index: usize,                            // resolved index, not a name
    pub body: Box<TypedRootV1>,
    pub gating: CompositionGatingV1,
}
```

Rules (each a stable sema diagnostic, fail-closed):

1. producer kind ∉ {Sample, Select, Find} → `composition_invalid_producer`
   (total match on the typed root enum — adding a root family later forces a
   compile error here, not a silent acceptance)
2. `WITH` name not among binding names (declaration-order scan, D4) →
   `composition_undefined_binding`; resolution result is the index, so the
   IR layer never re-resolves strings
3. producer and body are analyzed by the existing per-root analyzers —
   composition adds structure, it never re-implements root semantics
4. body that is itself a composition is unrepresentable by construction
   (syntax already rejects nesting) — assert via the type, not a runtime check

## DoD

1. `lib_tests.rs` proves: happy resolution (index correctness), every
   diagnostic code, and that a producer's own sema errors (e.g. FIND
   `depth=0`) surface through the composition unchanged
2. sema conformance suite green unchanged

## Proof Rail

`cargo test -p quanta-gqlang-v2-sema`
