# Receipt 07 — Post-Commit Audit (`d353c45a`)

Owner committed all lane work + packet as `d353c45a` ("done"). This receipt records
fresh verification against the committed tree (no worktree needed) and the one
supplement this audit required.

## Finding re-verification (all green on HEAD)

- Ticket commands re-run: surface hellgate 64 + edges 21, document 7, query 6,
  surface `--all-targets` 17/17 ok; planning provenance 12; cli doc_gate 2,
  fmt_idempotence 2, open_analysis 5, cli `--lib` 1.
- Content: REF-01 3 rails; STAGE table (3 refs); DOC table test; VIEW witness
  (`Inline(query)` + `segments[0]`); PRODUCER rail + neighbor; DEDUPE fns absent
  (only removal comments remain); EXPLAIN fixture + inventory; CLI spec table
  (lib 18 refs, main 4 refs) + dispatch test; CLI-02 RAII + 8 refs; Justfile
  composition (nextest/doc-test/parity).
- Mutation-only paths (`plan.rs`, `parser.rs`) clean; no probe/mutation residue
  (`tmp_`, `clausX` in code, `fmtx` absent; `clausX` survives only inside
  receipts text, which is now typos-excluded by data class).

## Gate battery on `d353c45a`

- `cargo fmt --all --check`: PASS
- `taplo fmt --check`: PASS
- `typos .`: PASS (after the supplement below; was exit 2 with 3 hits, all in
  receipts 00/01: hash fragments `ede`, `ba`, mutation token `claus`)
- `cargo check --workspace --all-targets`: PASS (types E0271 fixed by owner's commit)
- `cargo nextest run --workspace`: 1062 passed, 0 skipped (no exclusions)
- doctests: 5 passed; front-door parity: 2 passed
- arch tests: OK; `check_architecture.py`: exit 0
- `test_frontend_external_consumer.py`: verdict=PASS, 0 violations
- `just test`: exit 0
- `cargo clippy --workspace --all-targets -- -D warnings`: FAIL — 2 lints, both
  `indexing may panic` at unleased `types/src/env.rs:312-313` (owner's active
  migration). Zero lane lints. `just lint` therefore stops at its `clippy` step.

## Supplement applied by this audit (uncommitted, for owner review)

- `.typos.toml`: added `docs/bugbash/**/receipts/**` to `extend-exclude` (same
  data class as the already-excluded `docs/plans/**/receipts/**`: receipts embed
  SHA prefixes/mutation markers; ticket prose stays checked), formatted to
  taplo-canonical form (4-space indent, sorted). Without it `just lint` fails at
  `typos`; with only the exclusion but single-line form it fails at `taplo`.

## Remaining (outside this packet)

1. `types/src/env.rs` clippy indexing lints (owner's migration file, unleased).
2. `deny`/`semgrep` CI layers not run locally (network/hosted).
3. Candidate commit for the `.typos.toml` supplement (no commit authorization).
4. `tools/frontend_external_consumer` WIP (d13/d14 negatives) still dirty — consumer
   gate itself passes.
