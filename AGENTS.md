# Repository instructions

## Scope

- Be concise, direct, and technical.
- This repo owns syntax, sema, normalized IR, CLI, and diagnostics stdio only.
- Runtime substrate, provider registry, SDK/backend wiring, and runtime truth claims stay downstream-owned.
- Public downstream contracts remain host-agnostic and enter through `quanta-gqlang-v2-frontend-contract`.
- Preserve deterministic grammar, diagnostics, and IR. Do not lift legacy `quanta-gqlang-*` crates verbatim.

## Context routing

- Start with this file and the files named by the task. Do not preload `README.md`, the full `docs/` tree, or plan packets.
- Public language or owner-stack changes: search `docs/language/README.md`, then read only the relevant heading.
- Public frontend contract changes: use `docs/contracts/frontend-execution-v3.md`.
- Examples or release claims: use `docs/language/release_discipline.md` and `docs/language/examples_index.md`.
- Test coverage changes: use `docs/language/test_hellgate_ledger.md`. Qualification tooling: use `tools/qualification/README.md`.
- Plan or ticket work: read only the explicitly named packet and its direct prerequisites. Historical plans and receipts are evidence, not current implementation authority.
- Use `rg -n "<symbol|term>" <candidate paths>` before opening a long document.

## Verification

- During iteration, run the smallest affected owner test or policy rail.
- Use `just ci` once when the requested completion claim requires the repository-wide local gate; do not pre-run every command that `just ci` repeats.
- Use `just qualify-local` only for a clean committed-HEAD qualification receipt.
- Prompt-management changes must pass `just pm-check`.
