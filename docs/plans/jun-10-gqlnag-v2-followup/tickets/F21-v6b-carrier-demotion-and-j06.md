# F21 — V6B compat carrier demote + J06 최종 closeout (W5, joint)

> **STATUS: NOT STARTED (jun-10 재점검 + 어딧 정정).** semantica V6B carrier demotion — 미착수.
> **🔴 어딧 정정**: (1) "15+ 소비처"는 과대계상 — 실 **production 생성사이트 4개**(native_lowering:460, semantic_subset_admission:638, request_constructors:45×2) + 테스트 ~28. (2) 목적지 `ServingQueryRequestV1`(exec-contract/request.rs:5)+`lower_public_ir_root_to_request_v1`(ingress-bridge/query_bridge.rs:47) **존재 확인**. (3) carrier 에 `#[deprecated]`/`from_serving_request_v1` **미착수**(강등 0). (4) 본문 §4 "orchestration 17 test err" 결함 **미발견(해소됨)**. (5) 빌드 OK → 컷오버 지금 가능. INDEX 계획 #4 참조.


원천: J-caller 감사 + STATUS V6B(33%) 자기보고. V6B 의 남은 코드작업과 J06 클레임 규율 마감.

## Gap 목록 (라이브)

1. **compat carrier 잔존** (STATUS jun-09 정정에서 인정, 라이브 확인):
   - `quanta-runtime-orchestration/src/gateway/surface_factory/request_types.rs:21`
     `GqlangExecutionRequestV1` (+ `GqlangBoundExecuteRequestV1`)
   - `surface_types.rs:67` `ExistingQuerySurfaceV1` (`RequestSpec` join 유지)
   - 구조 노트: 이 파일들은 orchestration 이 컴파일하지 않고 `#[path]` 로 root `quanta-runtime` 에 forward —
     실제 cutover 는 runtime/SDK tail 에서 발생
2. **S08B residue-형 alias**: `quanta-sdk-session-backend-owner/src/lib.rs:276` `BoundarySourceSpecV1` —
   티켓이 명령한 재공표라 위반은 아니나 소비자가 테스트뿐 — demote 대상 후보로 기록
3. **J06 closeout 조건** (F16/F17/F18 완료 후):
   - delete_rows_zero == true (readiness 문서 기준)
   - generated 레저 fresh == committed
   - program closeout rule: 전체 S row complete + mandatory J bundle green + `--all` PASS
4. **선행 외부 결함 기록** (V6B 와 별개, 트래킹만): `quanta-runtime-orchestration` lib-`--tests` 17 errs —
   `#[path]`-forward 된 `planner_context_tests` 가 미링크 `quanta_contract_analysis_wire` 사용

## 작업

1. carrier demotion 설계: `GqlangExecutionRequestV1`/`GqlangBoundExecuteRequestV1`/`ExistingQuerySurfaceV1` 를
   (a) canonical ingress 경로로 흡수 후 삭제, 또는 (b) `#[deprecated]` + compat-only 모듈 격리 + 신규 소비 금지
   가드. 소비자 인벤토리 먼저 (`rg` + cargo metadata)
2. 데모션 증명 테스트: compat carrier 가 신규 canonical 경로의 변환으로만 생성됨을 assert
3. J05A/J05B/J07~J09 bundle fresh 재실행 (F19/F20 후) + V6B 체크포인트 마감
4. J06: F16 레저 동기화 확인 → program closeout rule 3조건 기계 검증 → 최종 클레임 기록
   (claim_state 규율은 F12 산출물 사용)

## DoD

- compat carrier 가 demote(삭제 또는 deprecated+가드) 완료, 신규 소비 0
- mandatory joint bundle (J01~J09D 19종) green 로그
- `gate-gqlang-v2-semantica-closeout.sh --all` PASS (F15 수리된 게이트 기준)
- J06 클레임이 prose 아닌 게이트/레저 산출물로 닫힘

## 구현 계획 보완 (jun-10 — 소비처 인벤토리 + 컷오버 순서)

### 라이브 소비처 인벤토리 (rg 산출, 컷오버 대상 전수)
`GqlangExecutionRequestV1`/`GqlangBoundExecuteRequestV1`/`ExistingQuerySurfaceV1` 소비 파일:
- **정의부**: `quanta-runtime-orchestration/src/gateway/surface_factory/{request_types.rs:21,52, surface_types.rs:67}` (#[path]-forward → root quanta-runtime 이 컴파일)
- **SDK 경로**: `quanta-sdk-query-backend-owner/src/{gqlang_ingress_v1.rs, lib.rs, public_query_surface_v1.rs}` (admitted-request 체인) / `quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs`
- **runtime 내부**: `quanta-runtime/src/boundary_gqlang_runtime_v1/{public_autocomplete_v1.rs, public_explain_v1.rs(3곳), orchestration_real.rs}` / `quanta-runtime/src/test_support_v1.rs`
- **테스트 지원**: `quanta-runtime-test-support-gateway/src/lib.rs` / `quanta-runtime/tests/{gqlang_semantic_subset_predicates_e2e_test.rs, gqlang_native_ide_vertical_test.rs}`
- 참고 주석: `quanta-runtime-query-owner-surface/src/public_ingress_surface_v1.rs`

### 컷오버 순서 (단계별 cargo check, half-migrated 금지)
1. **canonical 목적지 확정**: `ServingQueryRequestV1`(exec-contract) + ingress-bridge `lower_public_ir_root_to_request_v1` 가 신규 유일 진입. carrier 는 변환 전용으로 강등.
2. **변환 어댑터 신설**(1단계): `surface_factory` 에 `fn from_serving_request_v1(ServingQueryRequestV1) -> GqlangExecutionRequestV1` 단방향 변환 + carrier 구조체에 `#[deprecated(note="V6B compat — construct via from_serving_request_v1 only")]` 부착.
3. **소비처 그룹별 컷오버**(각 단계 후 `cargo check -p` 해당 crate):
   a. test-support 2 crate (저위험) → b. boundary_gqlang_runtime_v1 3파일 → c. sdk-query-backend-owner 체인 → d. runtime-executor handler.
4. **신규 소비 금지 가드**: F14 패턴 재사용 — `tests/` residue guard 에 carrier 직접 생성(`GqlangExecutionRequestV1 {`) 검출 needle 추가(변환 fn 경유만 허용).
5. **J06 마감**: 컷오버 후 ledger regen → `delete_rows_zero` 기계 검증 → mandatory joint bundle(J01~J09D) fresh 일괄 실행 → `gate --all`.

### 헬게이트
- carrier 가 `from_serving_request_v1` 산출과 필드 단위 동치임을 고정하는 변환 round-trip 테스트(신규, surface_factory 단위).
- residue guard needle 의 mutation proof(직접 생성 코드 주입 시 가드 FAIL).
