# 06 E2E RED Plan — Scenario Matrix, Written Red-First

Status: `Done` — all e2e rails written red-first and now GREEN: `composition_frontdoor_hellgate.rs` (9, CLI) + `stdio_composition_contract.rs` (6, LSP), zero remaining `#[ignore]`. S1-S8 covered (S2 forward-ref corrected to `composition_with_missing_body`; S6 added to `no_runtime_leakage_cli_sweep.rs`). S1's "both degraded reasons" was aligned to the single-degraded-per-root model (D6): composition lane surfaced; body lane preserved in IR (proven by the IR rail).

## Goal

Encode every E03C scenario as a failing binary front-door test BEFORE the
implementation lands, so 01–05 are "make the red rail green" with no room for
spec drift. Rails follow the repo's hellgate idiom: spawned binaries, exact
exit codes, byte-pinned stdout/stderr where deterministic, structured
diagnostic codes everywhere else.

## Owner Files (all new)

1. `crates/quanta-gqlang-v2-cli/tests/composition_frontdoor_hellgate.rs`
2. `crates/quanta-gqlang-v2-lsp/tests/stdio_composition_contract.rs`
3. composition fixture: `examples/composition/let_with_find.qglang` (ships
   only when 07 promotes it into the documented corpus)

Red-rail mechanics: until 01–05 land, each test is `#[ignore = "red: E03C
pending"]` with the assertion bodies already exact. 07 removes every
`#[ignore]` as the flip-to-green gate — a leftover `ignore` fails the
closeout grep.

## Scenario Matrix

### S1 — happy pipeline (the E03C proof fixture)

```qglang
LET suspects = SAMPLE query=AuthMiddleware ;; scope=repo::auth ;; mode=semantic ;; limit=20
WITH suspects
FIND taint api.entry -> db.write
```

| rail | pinned behavior |
|------|-----------------|
| cli parse | exit 0, `parse.root_count=1`, `parse.root[0].kind=composition`, summary carries `lets=1 with=suspects body=find` |
| cli sema | exit 0, `sema.root[0].kind=composition`, resolved binding visible |
| cli query | exit 0, `query.root[0].ir.kind=composition`, working-set ref id `suspects` in summary |
| cli headless | exit 0, `headless.mode=read-only`, `composition-materialization-unavailable` AND the FIND producer's `runtime-host-routing-unavailable` both listed (degraded preservation) |
| cli doctor | exit 0, `doctor.status=no-result`, same degraded pair |
| cli fmt | byte-canonical output; `fmt(fmt(x)) == fmt(x)` (extends the existing idempotence rail) |
| lsp hover | exact card line: `root_keyword=LET`, `category=composition`, `typed_target=suspects` |
| lsp symbols | document → binding(`suspects`) → with → body root symbols, exact line |
| lsp diagnostics | `result:[]` exactly (clean doc) |

### S2 — undefined alias fails closed

`WITH ghost` + body, no `LET ghost`. cli doctor → exit 1,
`doctor.status=sema-error`, `composition_undefined_binding` in the pinned
diagnostic line; lsp diagnostics → one item, `class=sema`, exact code.
Variant: forward reference (`WITH x` line before `LET x` line) — same code
(D4 declaration-order rule proven at the front door).

### S3 — invalid producer fails closed

`LET x = SHOW models` (SHOW ∉ D3 allow-set). doctor sema-error lane, code
`composition_invalid_producer`. Variant: `LET x = USE PACK ...` (meta root)
— same code, proving the allow-set is positive, not a denylist.

### S4 — structural parse faults (each its own exact code, parse-error lane)

| input | code |
|-------|------|
| `LET x SAMPLE ...` (no `=`) | `composition_let_missing_equals` |
| `LET 9bad = SELECT ...` | `composition_invalid_binding_name` |
| `LET x = LET y = ...` | `composition_nested_binding` |
| `WITH` (no name) / `WITH a b` | `composition_invalid_with_target` |
| `WITH x` as final line | `composition_with_missing_body` |
| `LET x = SELECT ...` alone, no WITH ever | `composition_unconsumed_let` |
| duplicate `LET x =` twice | `composition_duplicate_binding` |

### S5 — producer's own errors pass through unmasked

`LET x = FIND path a -> b | depth=0` + `WITH x` + body → sema-error lane
carrying the FIND `` `depth` must be greater than zero `` diagnostic, NOT a
composition code. Composition must never swallow or rewrap inner root
diagnostics.

### S6 — no-runtime-leakage extension

The composition fixture joins the existing sweep corpora
(`no_runtime_leakage_cli_sweep.rs`, `no_runtime_leakage_lsp_sweep.rs` —
ticket 07 adds it to their input lists): all denylist tokens absent, every
degraded reason ends `-unavailable`, no materialized item counts anywhere in
any command/method output (a `WorkingSetRefIrV1` has no items by
construction; this rail proves the rendering doesn't invent any).

### S7 — backward compatibility (the regression hellgate)

All 17 shipped examples produce byte-identical output across
parse/sema/query/fmt/doctor/headless versus pre-E03C. Mechanism: the
existing `examples_binary_frontdoor_matrix.rs` and sweep rails already pin
this — S7's job is the explicit closeout assertion that they still pass
untouched. Additionally: a document mixing a plain root line ABOVE a
composition block (`SHOW models\nLET x = ...\nWITH x\nFIND ...`) parses as
two roots with the plain root first, proving grouping never reaches
backward.

### S8 — transport and framing edges

1. compositions are `--file`-only (inline argv collapses newlines): inline
   attempt yields the body-less/ungrouped parse faults of S4, never a
   half-composed root — pinned exactly
2. lsp stdio: composition document via `{"method":"diagnostics", ...}` with
   `\n` JSON escapes — clean result `[]`; first-statement contract: hover on
   `composition-doc\nSHOW views` summarizes the composition block, never the
   trailing root, and never leaks `views`
3. empty/whitespace lines INSIDE a block (`LET ...\n\nWITH ...`) — pinned
   per D2: blank lines are filtered before grouping, so the block survives;
   the rail freezes that choice

## DoD

1. every scenario above exists as a runnable test with exact assertions
   before 01 merges (red), and every `#[ignore]` is gone after 05 (green)
2. zero new loose `contains`-only assertions where output is deterministic —
   exact-line `assert_eq!` per the hardened hellgate idiom
3. S7 byte-identity holds at closeout

## Proof Rail

```
cargo test -p quanta-gqlang-v2-cli --test composition_frontdoor_hellgate -- --include-ignored
cargo test -p quanta-gqlang-v2-lsp --test stdio_composition_contract -- --include-ignored
just ci
```
