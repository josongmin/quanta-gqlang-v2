# 04 — Test Entrypoint Authority

Status: `FOCUSED_GREEN (see receipts/04.md)`

Finding owned: `TEST-01`.

## Goal

Make `just test` mean all repository test layers that currently define the GitHub Actions
`rust-test` job: nextest, doctests, and shipped CLI↔stdio front-door parity.

## Write Set

- `Justfile`

`.github/workflows/lint.yml` and `tools/test_frontdoor_parity.py` are frozen. CI intentionally
keeps explicit commands because its runner does not install `just`; the local recipe must compose
the same three layers without creating a CI tooling dependency.

## Required Change

1. Add `just frontdoor-parity` to the `test` recipe after doctests.
2. Remove the separate `frontdoor-parity` dependency from `lint`; `lint` reaches it only through
   `test`.
3. Keep the dedicated `frontdoor-parity` recipe for focused execution.
4. Do not replace the Python rail with a Rust test: it compares two shipped binaries and their
   payloads at the process boundary.

Resulting composition:

```text
just test -> nextest -> doc-test -> frontdoor-parity
just lint -> ... -> test -> ...
GitHub rust-test -> same three commands explicitly
```

## Non-Vacuity Proof

Run `just --dry-run test` and `just --dry-run lint` and inspect the expanded commands:

- parity appears exactly once in each expanded path;
- nextest and doctests remain present;
- `lint` does not invoke parity twice.

Mutation: temporarily remove parity from `test`; the dry-run/assertion evidence must detect the
missing command. Do not add a permanent test that parses arbitrary Just syntax unless a stable
repository-owned Justfile parser already exists.

## Exact Verification

```bash
just --dry-run test
just --dry-run lint
just test
just lint
```

Record each subcommand and exit status. `just lint` is broad and may wait until Ticket 05 if other
lanes are active; the focused dry-run contract and `just test` must still be captured here.

## Definition Of Done

- Local `test` and hosted `rust-test` contain the same three semantic layers.
- `lint` reaches parity through exactly one composition path.
- The cross-binary parity script remains independently runnable.
- No workflow/toolchain churn is introduced for a one-line authority repair.

