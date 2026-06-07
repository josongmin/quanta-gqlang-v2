Be concise, direct, and technical.

Prioritize:
- public language boundary correctness
- frontend-only ownership
- deterministic grammar and IR
- no runtime substrate leakage

Rules:

1. This repo owns only syntax, sema, IR, CLI, and LSP.
2. Runtime substrate, provider registry, and SDK backend wiring do not belong here.
3. Public frontend contracts must stay host-agnostic.
4. Do not lift-and-shift legacy `quanta-gqlang-*` crates verbatim.

