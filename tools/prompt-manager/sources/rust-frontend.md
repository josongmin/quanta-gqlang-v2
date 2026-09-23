# Rust frontend boundaries

- Preserve the stage direction `surface -> hir -> binder/types/modules -> algebra/logic -> planning -> pipeline -> frontend-contract`.
- Only `quanta-gqlang-v2-frontend-contract` is a public downstream seam. Do not expose pipeline internals as a second public API.
- Keep open analysis distinct from closed compilation. Missing external authority must remain typed `Incomplete`, never a fabricated plan or success.
- Do not add provider, backend, runtime execution, host DTO, or SDK wiring to this repository.
- For a local change, run the narrow crate/test target that proves the behavior. Use the root gate according to `AGENTS.md`.
- Read `docs/language/README.md` only when the language surface or stage ownership changes. Read `docs/contracts/frontend-execution-v3.md` only when the public contract changes.
