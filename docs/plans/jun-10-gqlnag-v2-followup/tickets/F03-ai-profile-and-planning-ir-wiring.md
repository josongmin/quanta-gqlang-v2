# F03 — E05E/E06C AI profile sema + planning IR 프로듀서 배선 (W1)

> **STATUS: DONE (jun-10) — proof rails `ir --test planning_producer`(5), `sema --test ai_profile`(3), `cli --test query_plan_summary`(4) GREEN. workspace 102 bin ok, clippy gate+tests 0.**
> - **E06C (IR planning producer)**: `ir/src/planning_producer.rs` — `plan_for_root_v1`(root-family별 capability/truth 단일진실원 매핑), `plan_for_ai_retrieval_v1`(Retrieval→`Some(ai_plan)` 첫 production 경로 = planning.rs:117 `Some(ai_plan)` 검증분기 도달), `lowering_ready_for_root_v1`.
> - **E05E (sema AI lane, neutral half)**: `sema/src/ai_profile.rs` — `TypedAiProfileV1{reconfirmation_required}` + `ai_lane_admissibility_v1`. F02 `SemaExecutionProfileV1::Ai` matrix 를 구조적 base 로 경유(단일소스) 후, final/certified policy truth 요구 root(`REQUIRE CERTIFIED`, confirm=require CHECK)를 fail-closed(`ai_lane_final_truth_forbidden`) — IR `planning_ai_final_truth_forbidden` fence 의 sema 미러. admit 된 root 는 항상 reconfirmation-required. **과차단 가드: advisory(confirm=skip) CHECK·plain SELECT 는 admit 유지(테스트 고정).**
> - **non-test 소비자(섬 탈출)**: `cli/src/commands/query.rs` 가 `lowering_ready_for_root_v1` 소비 → query summary 에 `query.root[N].plan.capability=` / `.plan.truth=` 라인. Declaration/Meta 는 plan 없음(fabricate 안 함). → DoD `rg PublicPlanV1|...|LoweringReadyIrV1 crates/*/src` 에 creator(producer)+consumer(cli) 양쪽 hit.
> - 아키텍처 메모: `ai_lane_admissibility_v1`/`admit_for_profile_v1` 는 analyze_document 자동실행이 아닌 **profile-parameterized 게이트**(host 가 lane 선택 시 호출) — F02 프로파일 게이트와 동일 패턴. 정직성: jun-10 세션 초기 INDEX 에 F03 DONE 으로 잘못 표기했다가 E05E/소비자 부재를 재점검에서 포착해 마저 구현.


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

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: `ir/src/planning.rs:91 PublicPlanV1`(root/required_capabilities/truth_expectation/bridge_mode/export_mode/working_set_kind/ai_plan), `:5 PublicPlanRootKindV1`(10-variant), `ir/src/lowering_ready.rs:5 LoweringReadyIrV1{root, planning}` + `:11 new_v1` + `:20 validate_v1`. `ai_plan.rs:110 AiExplorationPlanV1` + 하위 plan 타입들. **전부 production 생산자 0** — capability/truth 매핑의 유일한 기존 구현은 conformance 테스트 헬퍼(`capability_for_root_v1`/`plan_v1`)에 있으니 그걸 production 으로 승격(테스트 헬퍼는 이후 이걸 import).

### 생성 파일
1. **`ir/src/planning_producer.rs`** (E06C 프로듀서)
   - `pub fn plan_for_root_v1(root: &PublicIrRootV1) -> Option<PublicPlanV1>` — 단일 진실원 매핑:
     - capability: Select|Retrieval→Search, Find→Path, Check→PatternLint, Explain→Explain, Interactive→CatalogIntrospection, Composition→(body 위임), Declaration|Meta→None(plan 없음 — query-plane 아님)
     - truth: Explain→ExplainEvidence, Retrieval→CandidateSet, 그 외→Observational
     - bridge/export/working_set: Check→PolicyBridge, Explain→ExplainBridge, Retrieval→RetrievalBridge+CandidateSet 의 기존 헬퍼 로직 이관
   - `pub fn lowering_ready_for_root_v1(root: PublicIrRootV1) -> Result<LoweringReadyIrV1, LoweringReadyIrErrorV1>` = `LoweringReadyIrV1::new_v1(root, plan_for_root_v1(&root))` — **이게 planning.rs:117 `Some(ai_plan)` 검증 분기의 첫 production 도달 경로**.
2. **`sema/src/ai_profile.rs`** (E05E sema-half)
   - `pub struct TypedAiProfileV1 { pub reconfirmation_required: bool }` + `pub fn ai_lane_admissibility_v1(root: &TypedRootV1) -> Result<TypedAiProfileV1, SemaDiagnosticV1>` — AI lane 에서 허용 root(SAMPLE/SELECT-ranked/COMPOSITION)와 금지 root(CHECK apply 류) 분리, 결과는 항상 `reconfirmation_required=true` (final-truth 금지 = RFC fence). F02 의 `SemaExecutionProfileV1::Ai` matrix 와 연동.

### 수정 파일 (배선)
- `ir/src/lib.rs`: `pub mod planning_producer;` + re-export. `lower_document_to_ir_v1`(:126) 은 그대로(plan 은 opt-in 산출).
- **AI plan 생산**: `planning_producer.rs` 에 `pub fn plan_for_root_with_profile_v1(root, profile: PlanProfileV1{Default,Ai}) -> Option<PublicPlanV1>` — `Ai`+Retrieval 이면 `ai_plan: Some(AiExplorationPlanV1{ objective: FindCandidateSet, discovery_plan: Some(DiscoveryPlanV1{ seed_query: retrieval.query, target_kind: CandidateSet, basis: AiDiscovery }), narrowing/reconfirmation: ai_plan.rs 기존 타입으로 구성 })`.
- **non-test 소비자**: `cli/src/commands/query.rs` summary 에 `query.root[N].plan.capability=...` / `.truth=...` 라인 추가 (`lowering_ready_for_root_v1` 호출) — LSP 아닌 CLI 가 소비.
- conformance 중복 제거(후속): `query_plane_bridge.rs`/`*_full_loop.rs` 의 로컬 `plan_v1` 헬퍼는 semantica 측이라 이 티켓 범위 밖 — external `ir/tests` 의 로컬 헬퍼만 producer import 로 교체.

### 헬게이트 (red 먼저)
- **`ir/tests/planning_producer.rs`**: ① 9 root-kind 각각 `plan_for_root_v1` 산출검증(capability/truth 표 그대로) ② Declaration/Meta→None ③ `lowering_ready_for_root_v1` 가 validate green ④ **Ai-lane**: Retrieval+Ai → `ai_plan.is_some()` + `validate_v1` green (planning.rs:117 분기 production 도달 증명) ⑤ negative: Ai+Check→ai_plan None/또는 reject.
- **`cli/tests/cli_headless_read.rs` 확장**: query summary 에 plan 라인 존재 assert.

### 검증
`cargo test -p quanta-gqlang-v2-ir --test planning_producer && cargo test --workspace`. 적대 포인트: producer 매핑이 conformance 헬퍼(semantica 측 3곳)와 어긋나면 V6A full-loop 가 깨짐 — 매핑 값은 그 헬퍼와 자구 일치시킬 것.
