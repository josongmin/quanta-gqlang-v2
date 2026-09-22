# Receipt 08 — Leftover Resolution Round (`0684706` → `fd3c2ed`)

Follow-up to receipts 00–07. Between rounds the owner landed `68d3e9a`,
`09319d3` (W1-C gate-graph cleanup), `987cbce`, `e87c044`, and `fd3c2ed`
(which swept this round's own pipeline/types/clippy fixes mid-session with the
message "chore: total-access clippy cleanups in execution zone").

## Fixes applied by this round

- `types/src/env.rs` (2 lints): `fields()[0]` ×2 → single-element slice pattern
  in the `matches!` guard. No behavior change; total by construction.
- `sema/tests/v3_authority_builder.rs` (1 lint): `fields[0]` mutation →
  `first_mut().expect(...)` with fixture-shape message.
- `pipeline/src/execution.rs` (17 lints, all in committed code):
  - hex `format!`-in-`collect` → shared `hex_lower()` helper using
    `char::from_digit` (also replaced the `write!+expect` in `to_hex`, same helper);
  - 2 × `as u64` → `u64::try_from(...).unwrap_or(u64::MAX)` (+ `saturating_mul`
    for the bit length);
  - sha256 reworked index-free: `zip(chunks_exact)`, `.get()/get_mut()`,
    array destructuring, `chunks_mut` output. Published empty-vector test green.
  - No `allow` attributes added anywhere (repo has zero `allow(indexing)` precedent
    in production code; `.get()` is the established convention).
- `pipeline/tests/v3_compiled_document_assembly.rs` (1 lint): `_ => {}` →
  exhaustive 9-variant list with future-variant-breaks-build comment.
- `pipeline/src/compile.rs` (1.95-only lint): `ItemRunV2::Planned` payload →
  boxed `PlannedRunV2`; 7 use sites updated (one pass-through site now moves the
  box instead of destructure-reconstruct, saving an alloc).
- `cli/tests/command_doc_gate.rs` (3 lints, own file): added `expect_used` to the
  test-target lint header per repo test convention. Still uncommitted (1 line).
- `.typos.toml`: bugbash receipts exclusion + taplo-canonical formatting (carried
  over from receipt 07; still uncommitted).

## Verification at `fd3c2ed` + 2 uncommitted files

- `just static` (fmt, taplo, clippy-msrv, clippy-current, typos): exit 0
- `just test` (nextest, doctest, parity): exit 0
- `cargo nextest run --workspace`: 1064 passed, 0 skipped
- doctests 5, parity 2, arch tests OK, external-consumer PASS (0 violations)
- `cargo deny check bans licenses sources`: ok. `advisories`: environment-blocked
  (read-only advisory-DB lock path in sandbox; needs network CI).
- semgrep: environment-blocked (binary crashes on any invocation: OCaml CA-store
  error; non-vacuity test fails only because the binary cannot start, not on rule
  content; telemetry-off retry same).
- `just boundary` no longer exists (W1-C renamed it to `policy`); one run failed
  with INFRASTRUCTURE verdict because the owner committed mid-run (concurrent
  writer), not on contract content. Re-ran components directly: all pass.
- TEST-01 re-verified under the new graph: `test: nextest doctest
  frontdoor-parity`, `ci` expansion contains the parity script exactly once.
- All 8 oracle anchors from tickets 01–03 confirmed present in their original
  files; focused targets green (surface 7/64/6, provenance 13 — W1-B added one).

## Remaining

1. Commit `.typos.toml` + `command_doc_gate.rs` + `receipts/07.md` + this file
   (no commit authorization; both code files are REQUIRED for green).
2. `deny advisories` + semgrep need hosted CI (sandbox-blocked here).
3. `just policy` full run pending (1) and CI layers.
