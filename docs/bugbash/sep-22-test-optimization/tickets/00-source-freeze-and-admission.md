# 00 — Source Freeze And Admission

Status: `ADMITTED (base 16c97e82; two base breakages recorded, see receipts/00.md)`

Findings owned: none. This ticket controls whether findings may be worked or qualified.

## Why This Gate Exists

The audit evidence was produced at `b6d52b9c49208cadb177e246fc72faf79f4e20cc`; the planning
observation is `3a9ab271d7706c1c9266551df492f64f4bf68eea`; the shared worktree also
contains unrelated frontend-contract, pipeline, and sema edits. A passing result on any one of
those states does not qualify a different state.

## Admission Procedure

1. Wait until the intended base commit exists and the target-owned paths in README's lease table
   have no uncommitted edits.
2. Create isolated worktrees from the exact admitted commit. Never capture RED by temporarily
   mutating the shared worktree.
3. Record:
   - `git rev-parse HEAD`;
   - `git status --porcelain=v1`;
   - `git diff --check`;
   - SHA-256 for every leased writable path and frozen input;
   - Rust toolchain, `cargo-nextest`, Python, `just`, and `typos` versions.
4. Run `cargo nextest list --workspace` once and save the selected inventory. This is the source
   of test counts; do not use the retired TSV.
5. Capture the pre-change baseline below. Failure caused by incomplete unrelated work blocks full
   admission, but does not authorize a workaround in a lane.

## Baseline Commands

```bash
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace --no-tests pass
RUSTDOCFLAGS="-D warnings" cargo test --workspace --doc
python3 tools/test_frontdoor_parity.py
```

The historical counts `1,052` nextest, `5` doctests, and `2` parity cases are comparison data,
not required current counts. Record current selected/executed counts and investigate any delta.

## Lease Rules

- One writer owns all Surface findings because its tests overlap in
  `tests/v3_family_hellgate.rs`.
- Planning, CLI, and Justfile writers are independent of Surface and one another.
- `crates/quanta-gqlang-v2-frontend-contract/**`, `pipeline/**`, and Sema production are outside
  this packet and frozen inputs. The sole Sema exception is the sealed fixture matrix at
  `crates/quanta-gqlang-v2-sema/src/test_support.rs`, leased to Ticket 02.
- A mutation-only path may be changed only in an isolated disposable worktree. Its post-mutation
  hash must exactly equal the frozen pre-mutation hash and it must not appear in a lane commit.
- `Cargo.toml`, `Cargo.lock`, workflow files, language docs, and packet status are integrator-only.
- No dependency addition is expected. A requested manifest change reopens planning.

## Acceptance

- One immutable `base_sha` and one digest manifest exist.
- Each target-owned path is clean at handoff and leased to exactly one writer.
- Baseline results are attached with exit codes and counts.
- Every writer can reproduce its ticket's current behavior from the same base.
- No implementation edit is made by this ticket.

## Reopen Conditions

- admitted SHA changes;
- leased file hash changes before a lane starts;
- a lane needs an unleased path;
- workspace baseline fails at a target-owned file;
- unrelated pipeline/sema work prevents candidate compilation.
