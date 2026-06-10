# Jun 10 — E03C `LET` / `WITH` Working-Set Composition

Original ticket: [E03C](../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E03C-let-with-composition.md)
Design authority: RFC-002 §Session Composition (line 624, 742-760), §WorkingSetV1 (line 1302-1317).

## What this initiative ships

`LET` and `WITH` as **working-set composition operators** across the full
frontend pipeline — syntax → sema → normalized IR → CLI → LSP — with the same
fail-closed, read-only, no-runtime-truth discipline as the existing 17 root
families:

```qglang
LET suspects = SAMPLE query=AuthMiddleware ;; scope=repo::auth ;; mode=semantic ;; limit=20
FIND taint api.entry -> db.write WITH suspects
```

- `LET <name> = <producer-root>` materializes as a typed reference to the
  working-set vocabulary (`WorkingSetRefIrV1` over `WorkingSetKindV1` /
  `WorkingSetBasisV1`) — no ad-hoc runtime locals, no hidden session state.
- The body **explicitly names the working set it consumes** via a trailing
  `WITH <binding>` clause (SQL-CTE style: the binding is referenced once, at the
  point of consumption). Undefined aliases fail closed; the body must be a
  working-set consumer root (`FIND` / `SELECT` / `CHECK` / `EXPLAIN`).
- No composition node widens into runtime packet injection: the read-only
  shell lowers compositions to typed IR and reports an honest
  `composition-materialization-unavailable` degraded reason. Materialization
  is host-side (Semantica) work behind the exec-contract seam and is out of
  scope here.

This initiative also **wires the previously production-dead `WorkingSetV1`
subsystem** (`crates/quanta-gqlang-v2-ir/src/working_set.rs`) into the live
lowering pipeline — `LET` is its missing producer.

## Ticket order

| # | Ticket | Lane |
|---|--------|------|
| 00 | packet scope, pinned decisions, repo boundary | serial gate |
| 01 | syntax: composition statements, document grouping, formatter | worker A |
| 02 | sema: typed binding graph, fail-closed resolution | worker A (after 01) |
| 03 | ir: `CompositionIrV1`, `WorkingSetV1` wiring, degraded authority | worker A (after 02) |
| 04 | cli: parse/sema/query/headless/doctor/fmt composition surface | worker B (after 03) |
| 05 | lsp: hover/symbols/diagnostics composition surface | worker C (after 03) |
| 06 | e2e RED plan: scenario matrix written red-first | worker D (red rails may land before 01) |
| 07 | serial integration, docs truth refresh, ledger, `just ci` | serial closeout |

Workers A/B/C own disjoint write-sets; 06's red rails are written first and
flip to green as 01–05 land. 07 is the only ticket allowed to touch shared
docs and the hellgate ledger.

## Non-negotiable rules (carried from repo discipline)

1. no heuristic fallbacks: every accept/reject decision is a deterministic
   grammar or type rule with a stable diagnostic code
2. behavior preservation: all existing 17-root-family outputs stay
   byte-identical (regression rail in ticket 06)
3. fail-closed: every malformed composition surfaces a structured diagnostic,
   never a silent skip or a guessed repair
4. read-only: no materialized counts, no runtime truth, degraded reasons come
   from the single `DegradedReasonV1` authority and end in `-unavailable`
5. no git mutations inside worker lanes; `cargo fmt -p <crate>` only
