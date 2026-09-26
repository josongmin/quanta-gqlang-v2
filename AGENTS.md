# Repository instructions

## Scope

- Be concise, direct, and technical.
- Own syntax, sema, normalized IR, CLI, and diagnostics stdio.
- Runtime substrate, providers, SDK/backend wiring, and runtime claims belong downstream.
- Public downstream contracts are host-agnostic through `quanta-gqlang-v2-frontend-contract`.
- Keep grammar, diagnostics, and IR deterministic; do not copy legacy `quanta-gqlang-*` crates.

## Context routing

- Start with task-named files; do not preload `README.md`, all `docs/`, or plan packets.
- Language or stage ownership changes: search `docs/language/README.md`; read the relevant heading.
- Public frontend contract changes: use `docs/contracts/frontend-execution-v3.md`.
- Examples or release claims: use `docs/language/release_discipline.md` and `docs/language/examples_index.md`.
- Test coverage changes: use `docs/language/test_hellgate_ledger.md`. Qualification tooling: use `tools/qualification/README.md`.
- Plan/ticket work: read the named packet and direct prerequisites. Current source/tests govern behavior; distinguish planned, focused proof, and qualified states.
- Use `rg -n "<symbol|term>" <candidate paths>` before opening a long document.
- For Rust changes, read `tools/prompt-manager/sources/rust-frontend.md`; Codex does not auto-load Cursor/Claude rules.

## Engineering decisions

- For nontrivial changes, define acceptance criteria and identify the consumer, owning stage, invariants, failure behavior, and compatibility impact before choosing the design.
- Trace the reachable caller-to-owner path. Fix the root cause at its owner, reuse existing contracts, and keep abstractions proportional to demonstrated needs.
- Missing authority stays typed `Incomplete`; malformed or unsupported input must not become success, a fabricated plan, or silently omitted work.
- Preserve stable ordering, identities, diagnostics, and IR. State intended behavior changes; version public contracts when compatibility requires it.
- For affected hot paths, bound input size, recursion, allocation, and repeated work. Base performance claims on comparable measurements with workload and source recorded.
- Inspect HEAD and relevant dirty changes before editing; preserve others' work. Continue authorized implementation and checks until acceptance criteria pass or a concrete blocker remains.

## Verification

- Run the smallest affected owner test or policy rail.
- For behavior fixes, use an independent expected result and a regression that rejects the faulty behavior. Exercise relevant edge/rejection paths; never weaken a gate just to pass.
- Run `just ci` once for repository-wide local completion; avoid pre-running its leaves.
- Use `just qualify-local` only for a clean committed-HEAD receipt.
- Instructions are generated: edit `tools/prompt-manager/sources/` or `tools/prompt-manager/targets.json`; run `just pm-sync` and `just pm-check`.
- Report changed behavior, executed checks/results, source state, and remaining gaps. Compile success, focused tests, and stale receipts do not establish product qualification; distinguish defects from untested claims.
