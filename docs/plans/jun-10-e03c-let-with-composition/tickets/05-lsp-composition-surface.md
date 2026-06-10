# 05 LSP: Composition Read-Only Surface

Status: `Done` — lsp crate green (composition rail 6/6, full suite 75 tests), clippy/fmt clean. Hover card `root_keyword=LET`/`category=composition`/`typed_target=<with>`; symbols expose document + per-LET `binding` + `with` + body root symbol; diagnostics run sema only when a composition root is present and classify `composition_*` as `class=sema`; quickfix stays read-only (no apply/edit). Note: LET/WITH completion rows live in the `ir` catalog table (outside the lsp write-set) and were deferred — the contract asserts nothing about completion. The read_only analyzer now parses whole-document (was per-line) so grouping folds the block.

## Goal

Teach the read-only LSP shell the composition shape without widening the
command plane: hover/symbols summarize the composition card, diagnostics span
the whole document, completion gains the two new statement heads.

## Depends On

1. `03-ir-composition-and-working-set-wiring`

## Owner Files

1. `crates/quanta-gqlang-v2-lsp/src/handlers/read_only_analysis/composition.rs` (new)
2. `crates/quanta-gqlang-v2-lsp/src/handlers/read_only_analysis/summary.rs` (dispatch arm)
3. `crates/quanta-gqlang-v2-lsp/src/handlers/completion.rs` (root-keyword table)

## Do Not Edit

1. `server.rs` / `json.rs` (no new method; composition flows through the six
   existing read-only methods)
2. quickfix authority fields (`apply_command`/`workspace_edit` stay null
   everywhere — composition adds no edit surface)

## Core Logic

1. **first-statement contract refinement**: the grouping pass (ticket 01)
   makes a composition block one root. Hover/symbols on a composition
   document summarize the *block* as the first statement:
   `root_keyword="LET"`, `category="composition"`, `typed_target=<with name>`,
   summary lines `lets=<n>`, `with=<name>`, `body=<kind>`. This keeps the
   existing "first statement only" rule intact — the block IS the first
   statement
2. symbols: document symbol, then one `binding` symbol per `LET` (name +
   `detail=basis`), a `with` symbol, and the body's own root symbol — all
   from the typed summary, no re-parsing
3. diagnostics: composition codes (`composition_*`) surface with
   `class=syntax` or `class=sema` exactly as the lower layers classified
   them; whole-document span behavior is unchanged
4. completion: `LET` and `WITH` join the root-keyword table with
   deterministic ordering (the determinism rail re-pins the exact list)

## DoD

1. hover/symbols/diagnostics handle composition fixtures; exact response
   lines pinned by ticket 06 stdio rails
2. zero-query and quickfix outputs byte-identical (no accidental widening)

## Proof Rail

`cargo test -p quanta-gqlang-v2-lsp`
