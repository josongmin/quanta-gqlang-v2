# CF-01: Execution operator vocabulary aliases Types owner

**Final triage:** [not in the active repair queue](FINAL-TRIAGE.md). The alias contradicts the execution-local vocabulary statement, but a current execution/compatibility failure is unproven and replacement changes the Semantica Rust type. Require a public contract decision first.

- 판정: `CONFIRMED_BOUNDARY_DEFECT` (P2, source-level contract coupling). QGLang HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`.
- 계약: `crates/quanta-gqlang-v2-execution-contract/src/lib.rs:5-10`은 execution-local enum과 제한된 inert owner type 재사용을 선언한다. 실제 `:49`는 `ExecutionOperatorKindV3`을 Sema의 `OperatorSemanticKindV2`로 `pub use`한다. Sema도 `crates/quanta-gqlang-v2-sema/src/lib.rs:32-37`에서 Types의 `crates/quanta-gqlang-v2-types/src/op.rs:21-40` enum을 재수출한다. 별도 V3 enum이나 exhaustively checked projection은 없다.
- 도달 경로: `execution-contract/src/vocabulary.rs:14-30,42-58` descriptor → `frontend-contract/src/lib.rs:65-87` 공개 facade → Semantica `packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/relation_plan_v3.rs:13,857-889,916-949` 및 `relation_exec_v3.rs:8,549-564`. Facade public API root allowlist도 `tools/test_frontend_external_consumer.py:121-125`에서 이 이름을 고정한다.
- 영향: Types owner enum의 variant나 의미가 바뀌면 V3 실행 API의 실제 타입과 vocabulary가 projection/version decision 없이 함께 바뀐다. 현재 Semantica는 알 수 없는 종류를 거부하므로 **현행 잘못된 실행 결과는 확인하지 않았다**. 본 결함은 소유권 및 호환성 경계 위반이다.
- 보완 수락: execution-contract 전용 `ExecutionOperatorKindV3`를 정의하고 owner enum을 exhaustive mapping한다. 모든 기존 variant와 unknown/unsupported refusal을 facade-only consumer test로 고정한다. Rust 타입 정체성 변경은 Semantica source pair와 함께 조정한다. 인코딩에 해당 enum이 없다면 wire digest schema 변경 여부는 실제 encoded-field 차이를 기준으로 결정한다.
- 증거·한계: [최종 계약 레인](09-contract-downstream-final.md). Static source/consumer trace만 수행했다. 수정 후 Cargo, full CI, Semantica compile은 `NOT_RUN`.
