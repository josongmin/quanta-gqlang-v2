# F03 — E05E/E06C AI profile sema + planning IR 프로듀서 배선 (W1)

> **STATUS: NOT STARTED (jun-10 재점검).** external W1. `sema/src/ai_profile.rs` 부재, planning IR 프로듀서 미배선 — 미착수.


원천: E-sema-ir-retrieval 감사. E05E = source-landed(unwired), E06C = source-landed(unwired).
타입+밸리데이터+자기시험만 있고 **production 프로듀서가 0개인 섬(island)** 상태.

## 현재 상태 (라이브)

- sema 절반 완전 부재: `crates/quanta-gqlang-v2-sema/src/ai_profile.rs` 없음 (E05E DoD 1차 seam)
- IR 절반 unwired:
  - `ir/src/ai_plan.rs:110` `AiExplorationPlanV1` 정의 + `:120` `validate_v1` 존재
  - `ir/src/planning.rs:91` `PublicPlanV1`, `:98` `ai_plan: Option<AiExplorationPlanV1>`,
    `ir/src/lowering_ready.rs:5` `LoweringReadyIrV1` 존재
  - 그러나 **production 생성자 0** — `tests/public_planning_ir.rs`, `tests/lowering_ready_ir.rs` 만 구성.
    `lower_root_to_ir_v1` 는 plan 을 절대 생산하지 않음. `planning.rs:117` `Some(ai_plan)` 분기는 테스트에서만 도달
- 반례(잘 된 것): `WorkingSetV1` 는 `composition_ir.rs:4` 에 라이브 프로듀서 있음

## 작업

1. `sema/src/ai_profile.rs` 신설 — AI exploration 프로파일 admissibility + reconfirmation 의미론 (E05E DoD)
2. lowering 파이프라인에 planning 프로듀서 연결:
   - `lower_root_to_ir_v1` (또는 후속 `LoweringReadyIrV1` 빌더)가 root family 별 `PublicPlanV1` 을 실제 생산
   - AI 프로파일 요청 경로에서 `AiExplorationPlanV1` 생산 → `ai_plan` 채움
3. CLI/LSP headless 경로 중 최소 1곳이 `LoweringReadyIrV1` 를 소비하도록 배선 (non-test 소비자 확보)
4. DoD 모듈 구조 정리: flat `planning.rs` → `planning/{mod,public_plan,capability_plan}.rs` (E06C 명시 구조) — 구조 미스만으로 block 하지 않되, 이관 시 함께

## DoD

- `rg "PublicPlanV1|AiExplorationPlanV1|LoweringReadyIrV1" crates/*/src` 에 test 외 생성자/소비자 존재
- `planning.rs:117` ai_plan 검증 분기가 production 경로로 도달
- final-truth fence (`planning_ai_final_truth_forbidden`) 의 external 측 대응 테스트 green
