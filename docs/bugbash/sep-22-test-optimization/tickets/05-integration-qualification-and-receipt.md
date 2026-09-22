# 05 — Integration Qualification And Receipt

Status at the original integration: `INTEGRATED` with `LOCAL_QUALIFIED` blocked
by base breakage (see receipts/05.md). Current review remediation and its
clean-source gate results are in receipts/09-review-remediation.md.

Findings owned: none. This ticket is the only local closeout authority for Tickets 01–04.

## Start Gate

- Ticket 00 base/digest manifest exists.
- Tickets 01–04 each provide a focused-green receipt and one commit or clean patch.
- All deliberate mutations were reverted.
- No leased product path has uncommitted or unowned changes.

## Integration Procedure

1. Recreate an isolated integration worktree at Ticket 00's exact `base_sha`.
2. Apply lane commits in numeric order: Surface, Planning, CLI, Test authority.
3. Before each apply, verify touched paths are a subset of that lane's lease.
4. Reject a lane if it changes manifests, workflow files, pipeline, Sema production,
   frontend-contract, or packet status without an approved ticket amendment. Ticket 02's exact
   `sema/src/test_support.rs` lease is the only Sema exception.
5. Resolve no semantic conflicts locally. Return conflicts to the owning ticket and regenerate
   its receipt against the new base.
6. Run focused commands from every ticket, then the full gates below on the same tree.
7. Commit the candidate, require a clean worktree, and rerun source-bound commands on that exact
   candidate SHA.

## Finding Accounting

Before qualification, mechanically confirm the set is exactly:

```text
CLI-01 CLI-02 DEDUPE-01 DOC-01 DOC-02 EXPLAIN-01
PRODUCER-01 REF-01 STAGE-01 TEST-01 VIEW-01
```

Every ID must have one owner receipt and one final result. No `OPEN` item may disappear through a
rename or “covered elsewhere” statement.

## Full Local Gates

```bash
git diff --check
cargo fmt --all --check
taplo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest list --workspace
cargo nextest run --workspace --no-tests pass
RUSTDOCFLAGS="-D warnings" cargo test --workspace --doc
python3 tools/test_frontdoor_parity.py
python3 tools/architecture/tests/test_architecture.py
python3 tools/architecture/check_architecture.py
python3 tools/test_frontend_external_consumer.py
typos docs/bugbash/sep-22-test-optimization
```

Run `just test` and `just static` as composition proofs after their individual commands are green.
Use `just --dry-run ci` to prove that parity appears once through the current gate graph;
qualify `just policy` and `just ci` separately from the Rust test packet.
Record selected/executed counts and compare them with Ticket 00. Explain all deltas: new oracle
rows, the two deliberate deletions, or unrelated base movement are not interchangeable.

## Adversarial Review

Inspect the integrated diff for:

- optional single-colon acceptance;
- duplicated relation-stage vocabularies;
- `contains`/substring, discriminant-only, success-only, or broad span assertions;
- producer tests that fail before `parse_producer`;
- symmetric provenance fixtures claimed as order proof;
- duplicate command-token authorities;
- parity invoked zero or twice through `just ci`;
- reintroduced scanner/TSV inventories;
- runtime/provider/SDK leakage.

## Receipt Set

Create `docs/bugbash/sep-22-test-optimization/receipts/00.md` through `05.md` with:

```text
ticket=
base_sha=
head_sha=
dirty_before=
dirty_after=
owned_files_and_pre_post_hashes=
red_commands_and_results=
green_commands_exit_codes_counts=
killed_mutations=
finding_results=
test_count_delta=
remaining_risks=
hosted_ci_status=
```

The final receipt binds every result to the committed candidate SHA and includes
`git status --porcelain=v1` showing empty output.

## Definition Of Done

- All eleven findings are closed once, with owner evidence.
- Focused and workspace gates are green on one immutable clean candidate.
- Test-count changes are explained from `cargo nextest list`, not the retired TSV.
- Local qualification is reported separately from hosted CI, review, merge, deploy, and activation.
- Any unavailable hosted evidence remains explicitly `OPEN`; it does not block an accurate local
  qualification claim.
