# 00 Packet Scope And Pinned Decisions

Status: `Ready`

Original ticket: [E03C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E03C-let-with-composition.md)

## Goal

Freeze the design decisions every downstream ticket builds on, so no worker
re-litigates grammar shape, gating, or scope mid-implementation.

## Pinned Decisions

### D1 — profile gating is deferred, structurally

RFC-002 gates `LET` by profile (`interactive` allowed, `ci_gate` forbidden).
The sema profile/admissibility subsystem was deleted on this branch
(`dsl-remediation`), so there is no profile carrier to gate against.

Pinned: tickets 01–05 implement composition **without** profile gating. The
typed sema node (`TypedCompositionV1`, ticket 02) carries a
`gating: CompositionGatingV1` field with exactly one variant
(`NotYetGated`) so reintroducing profile admission later is an enum-variant
addition, not a retrofit. No heuristic "default allow" string — the absence
of gating is itself typed.

### D2 — composition is document-structure grouping, not line continuation

The parser's one-statement-per-line, no-continuation invariant
(`crates/quanta-gqlang-v2-syntax/src/parser.rs` `document_statements_v1` doc)
stays untouched. The shape is:

```qglang
LET <name> = <producer-root>          # one line, RHS is a complete root
<consumer-body-root> WITH <binding>   # the body names the working set it consumes
```

`WITH` is the explicit point-of-consumption operator and **trails the body**
(SQL-CTE style: the binding is referenced once, where it is consumed), rather
than standing on its own line. A deterministic document-structure pass groups
statements with zero lookahead heuristics:

1. `LET` lines accumulate bindings in declaration order
2. a body line carrying a trailing ` WITH <binding>` (recognized only when the
   suffix after the last case-insensitive ` WITH ` is a single valid binding
   identifier — so a `WITH` inside a quoted literal is left to the inner root)
   consumes the accumulated bindings and becomes the composed body
3. a `LET` never consumed by such a body fails closed
   (`composition_unconsumed_let`); a standalone `WITH` line fails closed
   (`composition_with_must_trail_body`); any plain root with no trailing `WITH`
   ref and no pending bindings is an ordinary root, exactly as today

This (D-revision, jun-10 critical audit) upgrades the earlier positional
`WITH`-on-its-own-line model to an explicit body→binding data-flow reference,
closing the "body need not reference the binding" hole. Multiline bodies remain
out of scope (single-line roots, per the parser doc).

### D3 — `LET` RHS producer set

Per RFC-002 exploration algorithm step 2, a binding producer must be a
working-set producer. Pinned allow-set, enforced in sema (not the parser, so
the parse error surface stays uniform):

- `SAMPLE` (basis `SearchQuery`)
- `SELECT` (basis `SearchQuery`)
- `FIND` (basis `AiDiscovery`)

Any other root as RHS → `composition_invalid_producer` fail-closed. The
producer→basis mapping is a total match on the typed root kind — no default
arm.

### D4 — binding name rules

- shape: lowercase ASCII identifier `[a-z][a-z0-9_]*`, parser-enforced
  (`composition_invalid_binding_name`)
- duplicate `LET` name in one document → `composition_duplicate_binding`
- `WITH` naming an undefined binding → `composition_undefined_binding`
- forward reference (`WITH x` before `LET x`) → `composition_undefined_binding`
  (resolution is strictly declaration-order)
- an unused `LET` binding is legal (exploration sessions park bindings);
  no warning surface exists in this repo, so nothing is emitted

### D5 — the legacy stub is replaced, not extended

`WithClauseAstV1` / `LetBindingAstV1` in
`crates/quanta-gqlang-v2-syntax/src/interactive/meta.rs` and the
`DocumentAstV1.with_clause: Option<WithClauseAstV1>` field encode the wrong
shape (document-level option, stringly RHS). Ticket 01 deletes them and the
dead formatter branch in the same change that introduces the correct
composition module. No compatibility shim: nothing constructs them today
(verified — zero non-definition references in the workspace).

### D6 — read-only boundary

Composition lowers to typed IR that references `WorkingSetV1` descriptors.
The shell never materializes a set: headless/doctor surface the new degraded
reason `composition-materialization-unavailable`, added to the
`DegradedReasonV1` single-authority enum (it must end in `-unavailable`; the
no-runtime-leakage sweeps enforce this suffix rule). Host-side
materialization belongs to Semantica's exec-contract/host-spi lane — per the
E03C Do-Not-Touch list, those crates are not edited from this repo.

## Repo Boundary

In scope: `crates/quanta-gqlang-v2-{syntax,sema,ir,cli,lsp}` and this repo's
docs/tests. Out of scope: runtime/provider/backed materialization, the
Semantica host repo, legacy `quanta-gqlang-*` crates, `quanta-index` DTOs.

## Unblocks

1. `01-syntax-composition-statements-and-grouping`
2. `06-e2e-red-plan` (red rails encode these decisions verbatim)

## DoD

Every downstream ticket cites the decision it relies on by `D#` rather than
restating or reinterpreting it.
