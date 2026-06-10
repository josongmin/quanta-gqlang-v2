# 04 CLI: Composition Front-Door Surface

Status: `Done (rail pending lsp)` — cli lib green, clippy/fmt clean; all 6 commands render composition (verified via the binary: `parse.root[0].kind=composition` / `summary=lets=1 with=suspects body=find`, `query.root[0].ir.kind=composition`, doctor `no-result` + `composition-materialization-unavailable`, fmt canonical, S3 `composition_invalid_producer`). S1 aligned to the single-degraded-per-root model (D6): the composition lane is surfaced; the body lane is preserved in the IR, not flattened. The `composition_frontdoor_hellgate.rs` rail is un-ignored but can only RUN once the lsp crate compiles (cli test crate dev-deps lsp) — ticket 05.

## Goal

Render compositions across every CLI command with the existing line-oriented
output contract: parse/sema/query report the composed structure, headless and
doctor stay honestly read-only, fmt is canonical and idempotent.

## Depends On

1. `03-ir-composition-and-working-set-wiring`

## Owner Files

1. `crates/quanta-gqlang-v2-cli/src/commands/query.rs` (summary arms)
2. `crates/quanta-gqlang-v2-cli/src/commands/parse.rs` / `sema.rs` (kind labels)
3. `crates/quanta-gqlang-v2-cli/src/doctor.rs` (classification only if a new
   lane is genuinely needed — expected: none, compositions ride the existing
   three lanes)

## Do Not Edit

1. `main.rs` argv/usage contract (no new subcommand; compositions are inputs,
   not commands)
2. explain command (FIND-via-composition explain is a later initiative)

## Core Logic

Output vocabulary (extends the `key=value` line contract, no new format):

```
parse.root[0].kind=composition
parse.root[0].summary=lets=1 with=suspects body=find
query.root[0].ir.kind=composition
query.root[0].ir.summary=CompositionIrV1 { bindings: ["suspects"], body: find }
headless.degraded[0]=composition-materialization-unavailable
```

1. summary renderers are total matches over the new variants; the composed
   body's kind appears in the summary so a reader sees the pipeline shape
   without a second query
2. doctor: a composition with a sema-invalid producer or undefined alias
   lands in the existing `sema-error` lane (parse faults in `parse-error`);
   a valid composition is `no-result` with the new degraded reason listed —
   no fourth lane
3. inline-argv transport joins args with single spaces, so multi-line
   compositions are `--file`-only by construction; this is documented in the
   usage examples, not special-cased in code (transport parity for
   compositions is therefore file-vs-file canonical-form equality, ticket 06)

## DoD

1. all six commands handle a composition fixture without panic, with exact
   rendered lines pinned in ticket 06's rails
2. all 17 existing example outputs byte-identical (regression)

## Proof Rail

`cargo test -p quanta-gqlang-v2-cli`
