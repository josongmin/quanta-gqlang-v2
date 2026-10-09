# Final frontend owner audit: Surface/HIR/Binder/Types/Sema and scalar Pipeline

- Pinned source: `25fad76cf5b4510244157c2e009b4dc37aa958ff` (tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`). The Rust files audited here are clean in the shared checkout. Existing dirty documentation was not changed.
- Scope: independent second pass over [Surface/HIR and Binder/Types/Sema](01-syntax-sema.md) and [bounded scalar Pipeline](04-scalar-pipeline.md), including the public request-storage sink and scalar execution projection. This report adds one finding; it does not reclassify earlier lane tests as newly executed.

## Finding

| ID | Classification | Result |
| --- | --- | --- |
| [FF-01](FF-01-retained-chain-owner-validation.md) | Confirmed public API invariant failure under a selective sink; production reachability depends on host sink behavior | `FrontendRetainedStoragePermitV3::validate_owner_v3` checks only the current permit. A second root whose sink recognizes that permit but rejects an earlier retained permit passes validation and can extend the chain. Full-chain `validate_issued_sink_v3` rejects the resulting chain. |

The public caller path is `FrontendRequestMaterializationRootV3::from_issued_sink_v3` in `crates/quanta-gqlang-v2-hir/src/request_storage.rs:60-69` → request parse/continuation in `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner.rs:217-229,260-289` → declaration admission in `crates/quanta-gqlang-v2-sema/src/document/declaration.rs:340-352,121-129`. Scalar compilation also accepts the caller-supplied root at `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/scalar_compile.rs:50-59,63-76`. The chain check and extension are in `request_storage.rs:140-150,172-212`. A host sink that authenticates every permit from a stable issuer does not exhibit this counterexample; no production host was shown to construct a selective sink.

## Rechecked claims and boundaries

- The prior reports' source paths still describe the current HEAD: parser admission (`crates/quanta-gqlang-v2-surface/src/parser.rs:641-794`), HIR/source check (`crates/quanta-gqlang-v2-pipeline/src/compile/request_owner.rs:260-289`), scalar stage sequence (`.../scalar_compile.rs:63-105`), execution demand/projection (`crates/quanta-gqlang-v2-execution-contract/src/scalar_request.rs:62-160,164-220`), and execution seal/share (`crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/sealed/execution.rs:303-327,440-500`). This is a static recheck, not a rerun of their focused Cargo tests.
- The original parser request owner explicitly excludes some nested HIR and diagnostic allocations (`parser.rs:641-644`). It cannot establish a whole-frontend memory bound. Scalar projection counts source inventory and rejects unsupported expression shapes before projection; allocator/RSS and non-scalar families remain unmeasured.
- The prior 01 report records 23 focused test executions, and the prior 04 report records 10 scalar integration tests at the pinned HEAD. This lane ran **no Cargo test**, `just ci`, `just qualify-local`, downstream consumer build, or installed/runtime rail. No new claim about those tests' current execution is made.

## Independent counterexample executed in this lane

An external `/tmp/qglang-ff01-owner-chain.rs` fixture imported the **actual** `crates/quanta-gqlang-v2-hir/src/request_storage.rs` with Rust `#[path]`; only `SourceIdV2` and two host sinks were supplied by the fixture. A issues two reservations. B accepts A's current permit but rejects A's earlier retained permit, and can issue its own permit. With the same source and source ID, `validate_owner_v3(root_b)` and `try_extend_v3(root_b)` both returned `Ok(())`; `validate_issued_sink_v3(&b)` returned `ForeignRequestRoot` before and after extension.

Command: `rustc --edition=2021 -Awarnings /tmp/qglang-ff01-owner-chain.rs -o /tmp/qglang-ff01-owner-chain && /tmp/qglang-ff01-owner-chain` → exit `0`, stdout `reproduced: mixed-issuer chain admitted by validate_owner and try_extend; full-chain validation rejects`.

The temporary fixture is outside the repository. No Rust source, test, or policy file was edited. A repo regression should use the real owner API and reject the same counterexample before the next debit. Full-chain validation on every extension would make long chains repeatedly linear; a constant-time chain issuer witness or stage-boundary check needs separate design and measurement.
