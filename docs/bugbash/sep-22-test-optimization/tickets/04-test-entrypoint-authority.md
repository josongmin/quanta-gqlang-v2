# 04 — Test Entrypoint Authority

Status: `FOCUSED_GREEN (see receipts/04.md)`

Finding owned: `TEST-01`.

## Goal

Make `just test` own all repository test layers locally: nextest, doctests, and shipped
CLI↔stdio front-door parity.

## Write Set

- `Justfile`

`tools/test_frontdoor_parity.py` is frozen. GitHub Actions is intentionally absent; the local
recipe is the only composition authority for these three layers.

## Required Change

1. Include `frontdoor-parity` in the `test` dependency group with nextest and doctests.
2. Keep a single route from the top-level `ci` recipe through `test` to parity. The later W1-C
   gate graph replaced the historical `lint` recipe with `static`, `test`, and `policy` groups.
3. Keep the dedicated `frontdoor-parity` recipe for focused execution.
4. Do not replace the Python rail with a Rust test: it compares two shipped binaries and their
   payloads at the process boundary.

Resulting composition:

```text
just test -> nextest + doctest + frontdoor-parity
just ci -> static + test + policy (one parity invocation through test)
```

## Non-Vacuity Proof

Run `just --dry-run test` and `just --dry-run ci` and inspect the expanded commands:

- parity appears exactly once in each expanded path;
- nextest and doctests remain present;
- `ci` does not invoke parity twice.

Mutation: temporarily remove parity from `test`; the dry-run/assertion evidence must detect the
missing command. Do not add a permanent test that parses arbitrary Just syntax unless a stable
repository-owned Justfile parser already exists.

## Exact Verification

```bash
just --dry-run test
just --dry-run ci
just test
just static
```

Record each subcommand and exit status. Full `just ci` includes policy gates and
is qualified separately; the focused dry-run contract and `just test` remain required here.

## Definition Of Done

- Local `test` contains all three semantic layers.
- `ci` reaches parity through exactly one composition path.
- The cross-binary parity script remains independently runnable.
- No hosted workflow is required or retained.
