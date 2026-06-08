# Jun 9 Repo Startup

Status: `Ready`

## Goal

This plan records the actual remaining work inside `quanta-gqlang-v2` based on
the original RFC ticket program and the current repo-local base.

It is a repo-local recovery packet:

1. restore the local ticket-backed surfaces that were removed or narrowed
2. restore the named proof rails that those tickets require
3. close repo-local integration and `just ci`
4. only then do structural cleanup

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

