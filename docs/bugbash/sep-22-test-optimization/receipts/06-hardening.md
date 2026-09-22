# Receipt 06 — Post-Integration Hardening Round

Base: `16c97e82`. Candidate: base + lane.patch `6f0629cb1fda9803`
(supersedes `704796b6191f2d67`; worktree `/tmp/qgql-integrate`, 12 leased files).
Receipts 00–05 are frozen and unchanged; this file records only follow-up work.

## New mutation kills (were reasoning-only in 01)

- DOC branch-redirect: empty-statement message swapped to the missing-separator
  message → exactly `empty_statement_rejects` + `leading_separator_rejects` FAILED,
  `missing_separator_between_items_rejects` unaffected (branch discrimination proven).
  Reverted via python, `parser.rs` untouched after (`git status` clean).
- VIEW payload-loss probe (temporary test, removed after): body without `where`
  fails the witness (`expected root Filter, got Source(...)`); the old `Inline(_)`
  match would have passed it. Probe deleted; hellgate count unchanged by it.
- PRODUCER admit-explain (`parse_producer` returns Query for `explain`):
  `composition_explain_producer_rejects` FAILED (semantic acceptance, per ticket).
  Reverted; `family.rs` back to REF-01 fix only.

## Structural hardenings

- DOC-02: three separate tests → one table-driven
  `separator_branches_reject_with_exact_oracles` (rows: doubled / leading /
  missing; each row has source, code, message, exact span; trailing `;` stays the
  positive counterexample). Document target 9 → 7 tests, same branch coverage.
- REF-01 positive rail now asserts `rule_ref` span (6..15) in addition to the name.
- CLI authority: new `cli::tests::command_authority_is_single_and_closed` unit test
  (spec↔token derivation, alternation string, alias-kind sharing, uniqueness,
  5 unknown-token rejects).
- Sema inventory rail now also covers `ExistsWithMultiplicativeRight`
  (was listed in the enum but absent from the rail; resolves green).

## Foreign-line incident (lease collision)

Another agent's in-progress types/sema migration inserted one line into the leased
`test_support.rs` in the shared tree:

```rust
authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
```

It matches the shared tree's dirty (old-API) types, but breaks compilation against
HEAD types (field removed) — the candidate worktree's planning build failed with
E0433/E0560 because of it. Fix: removed that single line in the worktree candidate
only; the shared tree keeps it (required by its dirty types). Verified by hunk audit:
shared `test_support.rs` diff = 53 insertions / 0 deletions, of which exactly 1 line
is foreign. Planning 12/12 green after removal.

## Re-verification (worktree candidate, post-hardening)

- surface focused: 7 + 64 + 21 + 6, all ok
- planning `--all-targets`: ok (provenance 12)
- cli `--lib` + doc_gate + open_analysis: 1 + 2 + 5, all ok
- sema `--lib`: 17, ok
- `cargo nextest run --workspace --exclude quanta-gqlang-v2-types`: 966 passed, 0 skipped
  (967 − 2 DOC-merge + 1 CLI-unit = 966, arithmetic closed)
- doctests: 5 passed; front-door parity: 2 passed
- lane rustfmt: clean; lane clippy: clean (workspace clippy still blocked by base
  types breakage + a NEW unleased `types/src/env.rs` breakage from the active migration)

## Still OPEN (not this packet)

1. Base breakages: types lib-test E0271 (op.rs:98), external-consumer allowlist drift,
   now also types lib `env.rs` (active migration, unleased).
2. Candidate uncommitted (no commit authorization).
3. `just test` / `just lint` full runs pending base repair.
