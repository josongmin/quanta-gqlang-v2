# Jun 9 Repo Startup

Status: `Done`

## Goal

This plan records the actual remaining work inside `quanta-gqlang-v2` based on
the original RFC ticket program and the current repo-local base.

It is a repo-local recovery packet:

1. restore the local ticket-backed surfaces that were removed or narrowed
2. restore the named proof rails that those tickets require
3. close repo-local integration and `just ci`
4. only then do structural cleanup

## Closure Snapshot

1. ticket-backed frontend surfaces for `E05E`, `E06C`, `E11C`, and `E12C` are restored
2. named proof rails are present and green
3. repo-local composition closes on `just ci`
4. Wave 3 cleanup is landed:
   - shared LSP read-only analysis is extracted out of `hover.rs`
   - broad crate-root test-time allow blocks are removed
   - test-only lint exceptions are localized to dedicated test modules

## Historical Note

1. per-ticket `Current Base` sections record the startup baseline that this packet repaired

## Scope

In scope:

1. `quanta-gqlang-v2` only
2. local docs under `docs/`
3. local tests and proof rails
4. repo-local cleanup after closure

Out of scope:

1. `semantica-codegraph-v2` code changes
2. cross-repo joint handshake closure
3. Semantica `V6A/V6B/V6C` closeout claims

## Read Order

1. [tickets/README.md](./tickets/README.md)
2. [tickets/WORKER_START_HERE.md](./tickets/WORKER_START_HERE.md)
3. assigned ticket only
