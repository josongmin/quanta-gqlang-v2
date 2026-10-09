# 01–02 Surface/HIR and Binder/Types/Sema audit

- Source: `25fad76cf5b4510244157c2e009b4dc37aa958ff` (tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`). Audited committed Rust source only. The shared checkout had pre-existing dirty documentation; no Rust source was edited.
- Scope: the `cc27ff2`/`e598a1d` owner split and bounded request continuation through Surface, HIR, Binder, Modules session/catalog predeclaration, Types and Sema. The `9060e46..25fad76` file inventory is in `docs/audits/2026-10-10-weekly-change-inventory.md`.
- **Confirmed current-source defects: none in this scoped investigation.** This is not a claim that all changed paths or the full frontend are defect-free.

## Reachable path and decisions

| Caller → owner | Invariant and independently expected rejection | Current-source observation |
| --- | --- | --- |
| Pipeline parse → Surface lexer/parser → HIR | Different source allocation with identical bytes and source ID must fail before a host debit; a root with one byte less than peak demand must fail and release all earlier debits. | `crates/quanta-gqlang-v2-hir/src/request_storage.rs:71-92` binds the root to the original source address/length and ID; `:172-186` checks the live issuer. `crates/quanta-gqlang-v2-surface/src/parser.rs:673-794` reserves lexical, parser vector, HIR vector and root box charges. `parser/request_owner_tests.rs:184-324` exercises exact/short, foreign source, scratch refusal, and last drop. |
| Pipeline check/declaration → HIR integrity/Sema | Foreign root or different source must fail before issuing checked/declaration state; item and node identity remain source-issued. | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner.rs:260-309` consumes the parse owner and validates its permit before integrity; `crates/quanta-gqlang-v2-sema/src/document/declaration.rs:341-389` checks root, source length, item cap and depth, then admits declaration vectors. |
| Sema session → Modules predeclaration | Same source/root but another document session must not authorize catalog facts. | `crates/quanta-gqlang-v2-sema/src/document/session.rs:35-91` retains the validator; `crates/quanta-gqlang-v2-modules/src/session_identity.rs:10-34` carries issuer custody; `crates/quanta-gqlang-v2-modules/src/authority/request_catalog.rs:32-70` compares session witnesses by pointer identity. `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/session_tests.rs:155-220` rejects a mixed session before a new reservation. |
| Binder declaration/resolution → Sema → Types | Unsupported relation/call shapes must remain typed failures; foreign Binder, builtin, session or parent permits must not yield a typed body. | `crates/quanta-gqlang-v2-binder/src/resolve/request_owner.rs:86-106,129-149,174-225,429-497` authenticates retained declarations/resolution and rejects unadmitted shapes. `crates/quanta-gqlang-v2-sema/src/document/body_types.rs:212-264` validates four owners and preserves `UnsupportedShape`, storage, allocation and diagnostic variants. `crates/quanta-gqlang-v2-types/src/check/scalar_request.rs:132-173,188-239` independently recomputes the finite shape/demand and retains both type and builtin permits. |

The independent behavior oracle for the finite scalar example is one `Source → Project` query with two typed projection expressions; malformed/unsupported shapes must not produce an execution document. Existing `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly/scalar_request_tests.rs:198-289` compares the bounded result's three digests with the ordinary compiler and checks the source bytes and last-drop ledger. That integration test was read but not rerun in this lane.

## Executed checks at the pinned HEAD

| Command | Result |
| --- | --- |
| `cargo test --locked -p quanta-gqlang-v2-surface --lib request_owner_tests --quiet` | 5 passed, 0 failed |
| `cargo test --locked -p quanta-gqlang-v2-binder --lib request_tests --quiet` | 5 passed, 0 failed |
| `cargo test --locked -p quanta-gqlang-v2-pipeline --lib request_ --quiet` | 11 passed, 0 failed |
| `cargo test --locked -p quanta-gqlang-v2-types --lib request_ --quiet` | 1 passed, 0 failed |
| `cargo test --locked -p quanta-gqlang-v2-sema --lib request_ --quiet` | 1 passed, 0 failed |

## Proof and coverage gaps, not confirmed defects

- The request parse owner explicitly remains partial: `crates/quanta-gqlang-v2-surface/src/parser.rs:641-644` excludes other nested HIR constructors and diagnostics from full request custody. Do not treat a successful parse or these focused tests as proof of a fully bounded frontend. The scalar continuation has separate stage admission.
- This lane did not run the scalar integration test, `just ci`, `just qualify-local`, a downstream consumer build, or an installed/runtime rail. A committed-HEAD `just ci` receipt is listed in the weekly inventory, but it was not revalidated as part of this audit.
- The tests above use implementation-hosted sink mocks. A downstream host must satisfy the `FrontendRequestStorageSinkV3` contract in `crates/quanta-gqlang-v2-hir/src/request_storage.rs:6-15` by charging the concrete boxed carrier and implementing issuer-specific `owns_permit_v3`; these owner tests do not prove an external host implementation.
