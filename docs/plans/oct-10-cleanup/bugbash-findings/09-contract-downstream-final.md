# Final contract, gate, and downstream pair audit

- QGLang source: HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`. Rust, manifests, policy, and lockfiles were clean; unrelated documentation was dirty. This audit did not edit source or Semantica.
- Semantica source: HEAD moved from `170736bc5512f5d766306ccb5756aa87f783a0ef` to `65a8bd80c6f11a220a228b2ec936cd50eba9238d` during this audit. The intervening commit changed documentation, not the named manifests or lockfile. Its unrelated documentation had staged changes; `Cargo.toml`, `Cargo.lock`, and `packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml` were clean before and after the read-only checks.

## Findings

### CG-01 — Semgrep misses the new execution owner (reconfirmed)

- `.semgrep/qglang-frontend.yml:10-22` targets ten crates but omits `quanta-gqlang-v2-execution-contract`; `tools/lint/semgrep_gate.py:22-47` independently expects only those same ten owners. The gate therefore regards omission as the complete scan set. `Justfile:107-117` includes that gate in `just ci`, which CircleCI calls at `.circleci/config.yml:61-67`.
- Independent current-source check: importing `semgrep_gate.expected_paths()` returned 157 expected paths and `crates/quanta-gqlang-v2-execution-contract/src/lib.rs in expected_paths() == False`. The earlier isolated same-function mutation found a Pipeline violation but did not scan the new crate; see [CG-01](CG-01-semgrep-execution-owner.md). No current public `Result<_, String>` in the execution owner was established by this finding.
- Status: **confirmed policy coverage defect**, not a demonstrated current runtime typed-error defect.

### DP-01 — Semantica locked dependency closure fails (reconfirmed)

- Reachable path: Semantica `packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:19-21,35-38` enables the path dependency on the QGLang facade; QGLang `crates/quanta-gqlang-v2-frontend-contract/Cargo.toml:14-17` now depends on both Pipeline and execution-contract. Semantica `Cargo.lock:6830-6834` lists only Pipeline under the facade, and no `quanta-gqlang-v2-execution-contract` package entry exists.
- In Semantica root at both `170736bc` and `65a8bd80`, `cargo metadata --locked --offline --format-version 1` exited **101**: `the lock file ... needs to be updated but --locked was passed`. The command used full dependency resolution, without `--no-deps`. Manifest and lock remained clean afterward.
- Status: **confirmed consumer integration blocker**. The missing package is consistent with the new dependency, but this check does not isolate all possible lock changes. Consumer compilation, V3 vectors, and runtime admission remain `NOT_RUN`.

### CF-01 — Execution enum aliases an upstream Types owner (new boundary defect)

- `crates/quanta-gqlang-v2-execution-contract/src/lib.rs:5-10` promises execution-local closed vocabularies and permits reuse only of inert identity types and `AlgebraFamilyV2`. At line 49, `ExecutionOperatorKindV3` is instead a `pub use` alias of `quanta_gqlang_v2_sema::OperatorSemanticKindV2`, itself a re-export of the concrete `quanta-gqlang-v2-types/src/op.rs:21-40` enum (`crates/quanta-gqlang-v2-sema/src/lib.rs:32-37`). The descriptor returns this alias at `execution-contract/src/vocabulary.rs:14-30,42-58`; the facade exports it at `frontend-contract/src/lib.rs:65-87` and pins its root name in `tools/test_frontend_external_consumer.py:121-125`.
- Reachable consumer: Semantica re-exports and uses `ExecutionOperatorKindV3` in `packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/relation_plan_v3.rs:13,857-889,916-949` and `relation_exec_v3.rs:8,549-564`. A future Types enum variant/behavior change therefore changes this public execution vocabulary without an explicit projection or V3 version decision. Existing consumers reject unsupported kinds; no current wrong result was reproduced.
- Status: **confirmed source-level owner-boundary defect** under the repository's stated contract. Proposed repair: define a local `ExecutionOperatorKindV3` enum, exhaustively map the owner kind in the descriptor projector, and pin every supported variant in a facade-only test. Coordinate the intentional Rust type-identity break with the Semantica pair; preserve the wire schema if encoded fields do not change.

## Checks and proof boundary

- Read-only QGLang source/manifest/policy/API trace and Semantica consumer call-path trace. The alias and target omission are source facts; neither required a fresh Cargo build.
- Rehashed the existing exact-HEAD clean qualification receipt: `sha=25fad76`, `tree=99501ba`, `dirty=false`, `status=complete`, `exit=0`, Nextest selected/executed/passed `1529/1529/1529`, skipped/leaked 0. Its `ci.log` SHA-256 and all three source-lock/toolchain hashes still match. This is prior local proof for that HEAD, not a new run of `just ci`.
- `NOT_RUN` here: fresh Cargo tests, full CI, consumer build/vector, hosted CircleCI, performance measurements, and installed-product execution. Current shared QGLang checkout is dirty, so no new `just qualify-local` receipt was attempted.
