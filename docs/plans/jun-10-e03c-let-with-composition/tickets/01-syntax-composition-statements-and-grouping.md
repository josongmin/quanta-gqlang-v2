# 01 Syntax: Composition Statements And Document Grouping

Status: `Done` — syntax crate green (14 composition tests + 47 existing), clippy/fmt clean. Self-audit fixed a weak producer-error assertion and corrected an incorrectly specified "forward reference" red-rail case (it is `composition_with_missing_body`, not `composition_undefined_binding` — a true forward reference is grammatically unreachable).

## Goal

Own the `LET` / `WITH` grammar end to end in the syntax crate: statement
parsing, deterministic document grouping (D2), binding-name validation (D4),
canonical formatting, and the removal of the wrong-shaped legacy stub (D5).

## Depends On

1. `00-packet-scope-and-pinned-decisions`

## Unblocks

1. `02-sema-typed-binding-graph`

## Current Base

1. `parse_root_v1` dispatches 17 root heads; `LET`/`WITH` fall through to
   `unsupported_root`
2. `document_statements_v1` is line-oriented with no continuation (kept as-is)
3. dead stub: `WithClauseAstV1`/`LetBindingAstV1` in `interactive/meta.rs`,
   `DocumentAstV1.with_clause` hardcoded `None` in `parser.rs:35`, unreachable
   `format_with_clause_v1` in `formatter.rs`

## Owner Files

1. `crates/quanta-gqlang-v2-syntax/src/interactive/composition.rs` (new)
2. `crates/quanta-gqlang-v2-syntax/src/parser.rs`
3. `crates/quanta-gqlang-v2-syntax/src/ast.rs`
4. `crates/quanta-gqlang-v2-syntax/src/formatter.rs`
5. `crates/quanta-gqlang-v2-syntax/src/interactive/meta.rs` (stub removal only)
6. `crates/quanta-gqlang-v2-syntax/src/lib.rs` (module decl only)
7. `crates/quanta-gqlang-v2-syntax/tests/let_with_composition.rs` (new — the
   E03C-named proof rail)

## Do Not Edit

1. any sema/ir/cli/lsp crate
2. `document_statements_v1` semantics (one statement per line stays)
3. any existing root family parser

## Core Logic

New AST (in `interactive/composition.rs`):

```rust
pub struct LetBindingAstV1 {            // replaces the meta.rs stub
    pub name: String,                   // D4-validated identifier
    pub producer: Box<AstRootV1>,       // complete parsed root, never a string
}

pub struct CompositionRootAstV1 {
    pub lets: Vec<LetBindingAstV1>,     // declaration order
    pub with_name: String,              // the narrowing alias
    pub body: Box<AstRootV1>,           // the statement after WITH
}
```

`DocumentAstV1` drops `with_clause`; `AstRootV1` gains a
`Composition(CompositionRootAstV1)` variant.

Parsing (all fail-closed, stable codes):

1. `LET` head → split on the first `=`; missing `=` →
   `composition_let_missing_equals`; empty/invalid name →
   `composition_invalid_binding_name`; RHS parsed via `parse_root_v1`
   (errors propagate with their own codes); RHS that is itself `LET`/`WITH`
   → `composition_nested_binding`
2. `WITH` head → exactly one identifier tail; anything else →
   `composition_invalid_with_target`
3. document grouping pass (after per-line parse, before returning
   `DocumentAstV1`): fold the statement list per D2. `WITH` not followed by
   a plain root → `composition_with_missing_body`. Trailing `LET`s with no
   `WITH` in the document are legal standalone bindings only when a `WITH`
   follows later; a document whose final grouped node would be bindings with
   no body → `composition_unconsumed_let` — there is no silent drop
4. duplicate binding name → `composition_duplicate_binding` (parser-level:
   purely lexical, needs no types)

Formatter: canonical form is exactly `LET {name} = {format_root(producer)}`
per binding line, `WITH {name}`, then the formatted body line. Round-trip
(`fmt(parse(x)) == fmt(fmt(parse(x)))`) must hold; the existing fmt
idempotence hellgate picks composition up via ticket 06 fixtures.

Stub removal (same change): delete `WithClauseAstV1`/old `LetBindingAstV1`
from `meta.rs`, the `with_clause` field, `parser.rs:35`, and
`format_with_clause_v1` + its call site.

## DoD

1. `tests/let_with_composition.rs` covers: happy 1-let, multi-let, every
   diagnostic code above (table-driven, exact code + message-fragment
   asserts), formatter canonical round-trip, and a no-composition document
   parsing byte-identically to before (regression guard)
2. all existing syntax tests green unchanged
3. no `unwrap`/`panic` outside `#[cfg(test)]`; total matches, no default arms
   on new enums

## Proof Rail

`cargo test -p quanta-gqlang-v2-syntax --test let_with_composition -- --nocapture`
plus the full crate suite.
