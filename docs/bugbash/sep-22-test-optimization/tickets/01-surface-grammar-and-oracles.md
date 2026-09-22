# 01 — Surface Grammar And Oracle Closure

Status: `FOCUSED_GREEN (see receipts/01.md)`

Findings owned: `REF-01`, `DOC-01`, `DOC-02`, `VIEW-01`, `PRODUCER-01`, `STAGE-01`,
`DEDUPE-01`.

## Goal

Make the public language boundary deterministic at the actual parser owners, then harden tests so
token loss, wrong spans, branch bypass, sibling-stage drift, and redundant executions are visible.
This is one writer lane because the findings share Surface sources and test files.

## Write Set

- `crates/quanta-gqlang-v2-surface/src/family.rs`
- `crates/quanta-gqlang-v2-surface/src/query.rs`
- `crates/quanta-gqlang-v2-surface/tests/v3_document_hellgate.rs`
- `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
- `crates/quanta-gqlang-v2-surface/tests/v3_family_edges.rs` only if an existing stronger rail
  needs exactness preserved; no new duplicate test
- `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs`

Do not edit HIR types, formatter ownership, sema, pipeline, or public facade.

## Production Repairs

### `REF-01` — atomic `::` separator

Owner: `family.rs::parse_ref`.

Current cause: after consuming one `Colon`, the parser conditionally consumes the second. The
documented `::` separator is therefore implemented as `:` plus an optional `:`.

Required implementation:

1. Treat the separator as exactly two consecutive colon tokens.
2. If the second colon is absent, return `UnexpectedToken` at the token immediately following the
   first colon, with the stable expectation `a second ':' in '::'`.
3. After `::`, require an identifier segment with the existing
   `a path segment after '::'` expectation.
4. Keep `.`, `-`, and `/` reference forms unchanged. Do not accept whitespace-normalized or
   recovered single-colon syntax.

Required rails:

- positive: `check repo::auth | confirm=skip` retains the exact ref `repo::auth`;
- negative: `check repo:auth | confirm=skip` rejects at the second-colon expectation;
- negative: `check repo:: | confirm=skip` rejects at the missing segment;
- mutation: restore optional-second-colon behavior; the single-colon rail must fail.

### `STAGE-01` — one closed relation-stage vocabulary

Owner: `query.rs::parse_stage`.

Current cause: accepted keywords are encoded in dispatch branches while the expected-token message
is a second handwritten list that omits `cross`.

Required implementation:

1. Introduce one private closed stage descriptor/table for `where`, `project`, `join`, `cross`,
   `group`, and `order`.
2. Use that authority both to recognize a stage and to construct the complete expected-token
   diagnostic. Dispatch still calls the existing stage parsers; do not add a generic AST or a
   second parser.
3. Preserve accepted syntax and HIR output exactly.
4. Exact-test every retired stage against the complete six-token message.

Mutation: remove `cross` from the diagnostic authority while leaving parsing enabled; the exact
message rail must fail.

## Oracle Repairs

### `DOC-01` / `DOC-02`

Use source-derived byte offsets (`src.find(...)` plus token length), never copied magic numbers.

- `malformed_middle_item_rejects_with_exact_span`: assert `UnexpectedToken`, exact message, and
  the exact `db` span in `find taint api.entry db.write`.
- Table-drive doubled separator, leading separator, and missing separator cases. Each row specifies
  complete source, exact code, exact message, and exact primary span.
- Keep trailing separator as the positive counterexample.
- Mutation: redirect one row to the other diagnostic branch or offset its span by one byte; the
  table must kill both changes.

Do not snapshot debug output or use `contains`/inequalities for these contracts.

### `VIEW-01`

Destructure `ViewBodyV1::Inline(relation)` and assert an independent HIR witness proving the full
`from users u | where u.active` body survived: root filter, scan/source identity, alias, and the
`u.active` predicate path. Prefer the existing Surface test support projection if it exposes these
fields; otherwise inspect the HIR variants directly in this test. Do not format the parsed HIR and
reparse it as the sole oracle.

Mutation: drop the `where` stage or replace `active` with a different member; the test must fail.

### `PRODUCER-01`

The existing source never reaches `parse_producer`; replacing only its assertions is insufficient.

1. Use a valid first binding so composition recognition succeeds.
2. Put invalid `explain` in a subsequent producer position, for example a second
   `let bad = explain ...` before the admitted body/`with` boundary.
3. Prove reachability with exact `UnexpectedToken`, exact
   `a composition producer (from / sample / find)` message, and the `explain` span.
4. Retain a positive neighbor with a valid second `from`, `sample`, or `find` producer.

Mutation: admit `explain` in `parse_producer`; this rail must fail for semantic acceptance, not at
an earlier lookahead.

### `DEDUPE-01`

Only after `v3_family_edges.rs` exact-message rails are green, delete:

- `profile_missing_capabilities_rejects`;
- `gate_missing_checks_rejects`.

Before deletion, temporarily break each owner diagnostic and show the stronger edge test fails.
After deletion, prove no test with the same input and weaker oracle remains. Do not delete positive
Profile/Gate payload tests.

## Execution Order

1. Capture all RED/mutation evidence.
2. Repair `parse_ref` and stage vocabulary authority.
3. Add document, VIEW, and producer oracles.
4. Run all Surface targets.
5. Delete the two redundant tests.
6. Re-run the same commands and strict Clippy.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate --test v3_family_edges
cargo test -p quanta-gqlang-v2-surface --test v3_document_hellgate
cargo test -p quanta-gqlang-v2-surface --test v3_surface_query
cargo test -p quanta-gqlang-v2-surface --all-targets
cargo clippy -p quanta-gqlang-v2-surface --all-targets -- -D warnings
```

Record selected/executed counts. The final Surface test count must decrease by exactly two from the
post-addition count due to `DEDUPE-01`; all newly added table rows remain counted/visible.

## Definition Of Done

- Single colon is rejected and double colon is retained exactly.
- Accepted stage vocabulary and diagnostic vocabulary share one owner.
- All named diagnostics assert exact code/message/span.
- Inline VIEW proves the retained semantic payload.
- The producer test demonstrably reaches the producer parser.
- Exactly two weaker executions are removed after mutation evidence proves coverage continuity.

