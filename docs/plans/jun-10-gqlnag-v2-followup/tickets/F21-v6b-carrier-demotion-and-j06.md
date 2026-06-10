# F21 — V6B compat carrier demote + J06 최종 closeout (W5, joint)

> **STATUS: NOT STARTED (jun-10 재점검).** semantica V6B. compat carrier 15+ 소비처 cutover = 대형 마이그레이션 — 미착수. 소비처 인벤토리 산출됨.


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
