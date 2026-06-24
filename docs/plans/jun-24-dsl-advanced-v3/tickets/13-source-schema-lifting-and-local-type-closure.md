# 13 Source Schema Lifting And Local Type Closure

Status: `Implemented On Current Main`

## Current Main Receipt

As of `2026-06-24`, the local type-closure seam owned by this ticket is live:

1. relation sources now lift source-name and alias row types into the local env
2. filter/project/compose typing runs under merged schema-plus-local authority
3. result-shape typing is checked under the same lifted local authority
4. deterministic env merge is supported by `TypeEnvV1::iter()`
5. proof rails:
   `v3_alias_type_closure_gap.rs`,
   `v3_frontend_context.rs`,
   `audit_edge_cases.rs`,
   `v3_normalize_item_decl_rule.rs`
6. verified with:
   `cargo test -p quanta-gqlang-v2-pipeline`,
   `cargo test --workspace`,
   `cargo clippy --workspace --all-targets`

Narrowing that matters:

1. standalone `let` declaration bodies do not have binder-visible locals to lift
2. the declaration-side failure mode therefore closes as:
   closed body -> full type-check
   free symbol -> fail-closed `bind` reject
3. the live D34 lifting seam is the query / alias / capture / param lane, not a
   hidden declaration-body skip path

## Goal

Make binder-visible locals type-check under a real lifted local environment so
alias-dependent, param-dependent, and capture-dependent expressions can no
longer slip through unchecked.

## Why

The pre-closeout gap was invalid alias-dependent queries compiling because
relation-source schemas were not lifted into the local type environment. The
type stage previously skipped env-open expressions rather than rejecting or
typing them.

That produces false-green behavior on exactly the shapes advanced users rely on:

1. source alias field projection
2. filter predicates over aliased rows
3. correlated/local capture
4. declaration/rule params
5. nested result-shape fields derived from local rows

## Deliverables

1. local typed environment synthesis from binder-visible locals
2. relation-source schema lookup against the explicit frontend context from `12`
3. source alias -> row-type lifting
4. param/local/captured binding -> type lifting
5. path-base typing over lifted row types
6. typed field-projection rejection for missing fields
7. typed operator rejection for alias-dependent bad overloads
8. deterministic policy for genuinely unknown local types:
   reject or explicit limited-authority diagnostic, but no silent skip
9. cross-layer proof rails for query/filter/project/correlated cases

## Pinned Semantics

1. every binder-visible local is either typed or rejected explicitly
2. source aliases denote row values, not opaque strings
3. path segments are field projections over typed row/record values
4. correlated capture keeps lexical ownership from `02` and gains typed closure
   here; no hidden path-scoping magic
5. env-open skip paths are not valid semantic closure for local query
   expressions
6. lifting uses frontend-owned schema/type authority only; it does not reach
   into runtime/provider state

## Non-Goals

1. this ticket does not define the closed type universe:
   ticket `03`
2. this ticket does not redefine binder name resolution:
   ticket `02`
3. this ticket does not normalize algebra:
   ticket `04`
4. this ticket does not define the public semantic-authority bundle:
   ticket `12`

## Scenarios Unlocked

1. `from users u | where u.id + "x" > 1` rejects at `type`
2. `from users u | project u.ghost as id` rejects at `type`
3. correlated inner expressions can type-check against explicit captured row
   types
4. parametric declarations/rules can type-check against their bound parameter
   types instead of acting like open symbols

## Definition Of Done

1. alias/local expressions no longer compile unchecked because the env stayed
   open
2. bad alias-dependent overloads reject with typed diagnostics
3. bad alias-dependent field projections reject with typed diagnostics
4. binder-visible locals have one deterministic typed closure path used by the
   pipeline and tools

## Dependencies

1. requires `00`
2. depends on `02`, `03`, and `12`
3. closes before `04`, `07`, and `08` may claim semantic query closure

## Likely Touch Points

1. `crates/quanta-gqlang-v2-pipeline/src/compile.rs`
2. `crates/quanta-gqlang-v2-types/src/check.rs`
3. `crates/quanta-gqlang-v2-types/src/env.rs`
4. `crates/quanta-gqlang-v2-binder/src/resolve.rs`
5. query success/failure rails in `crates/quanta-gqlang-v2-pipeline/tests/`

## Mandatory Worker Outputs

1. one deterministic local typed-env lifting algorithm
2. no silent env-open skip path for binder-visible locals
3. green lock rails for alias bad-overload and alias bad-projection closure

## Proof Rails

1. keep the alias closure lock rails green in
   `crates/quanta-gqlang-v2-pipeline/tests/v3_alias_type_closure_gap.rs`
2. add one correlated-capture typing rail
3. keep the declaration/body no-skip rail green in
   `crates/quanta-gqlang-v2-pipeline/tests/v3_normalize_item_decl_rule.rs`
4. add one param/local binding typing rail

## Reject Or Escalate

1. silently skipping alias/local expressions at the type stage
2. typing source aliases as ad-hoc strings or untyped opaque handles
3. inventing runtime/provider schema lookups inside the type checker
