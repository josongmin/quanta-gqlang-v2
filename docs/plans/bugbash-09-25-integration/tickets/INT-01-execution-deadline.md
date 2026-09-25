# INT-01 — V3 Composition 실행 deadline 검증·수정

상태: **공유 main 수정 반영 / owner·설치형 자격 검증 미완료** · 우선순위: P1 · 소유: Semantica SDK ingress + runtime task/host owner. QGLang frontend 변경은 전제하지 않는다.

초기 감사 시점에는 handler가 dense provider connect에만 별도 deadline을 계산하고 Composition host context에는 `BudgetIntentV1::default()`를 넣었다. provider admission과 SAMPLE 실행도 deadline 없는 동기 runner를 사용했다. 이 문단은 **수정 전 소스의 결함 근거**이며 현재 main의 상태가 아니다.

2026-09-26 검증 시도: 격리 Semantica checkout `c2043a05bae592809ae9dd8b9a16575f52f57f6b`에서 SDK configured semantic SAMPLE 테스트에 이미 만료된 `Instant` 요청과 receipt 부재 검사를 추가했다. 등록 QBC exact owner rail은 proof feature의 누락된 mutation 권한 때문에 처음에는 컴파일 오류였고, 시험용 feature closure를 보강한 다음 재실행에서는 1개 테스트가 선택·실행됐으나 fixture가 요구하는 현재 소스의 `QUANTA_INDEX_SEARCHD_BIN` 부재로 본문 초기에 실패했다(0 pass, 1 fail, 977 filtered; source digest `22ccfbb146eb1ef0401e11fa70c00925b8c41d219f7193a5b346937f06b1a83f`). receipt: `/Users/songmin/Library/Caches/semantica-codegraph-v2-target/quanta-build-cli/_state/execution-roots/semantica-gqlang-ss.6PA8D3-4b9ac41137d5/lanes/gqlang-ss-timeout/verification-results/20260925T162227.342029Z-4a244d867783/receipt.json`. 이는 **타임아웃 동작 RED가 아니다**. 격리 checkout의 시험 수정은 공유 main에 통합하지 않았다.

2026-09-26 공유 Semantica main에는 기존 absolute deadline/lifecycle guard를 보존한 채 `ServingTaskRunnerPortV1::run_sync_serving_task_until_v1` → runtime bounded dispatch → provider admission/SAMPLE 대기 상한을 연결했다. SDK는 기존에 ingress deadline, host time budget, 직렬화 후 lifecycle 검사·observer 순서를 갖고 있었다. 격리 checkout의 SDK handler 수정 전체를 덮어쓰지 않았다. SDK handler가 dispatch worker에서 실행되므로 bounded 재진입은 **현재 worker를 제외한 다른 worker**로 보낸다. 구성된 worker가 1개면 deadline 보장 없는 inline 실행 대신 typed non-success로 거절한다. 동기 host가 비협조적으로 계속 실행해도 caller는 deadline에 typed non-success로 돌아오고 late result는 버리지만, host 자체를 강제 중단하거나 점유 worker를 즉시 회수하는 기능은 아니다. 모든 worker가 외부 요청에 점유되면 중첩 SAMPLE은 capacity timeout으로 실패할 수 있다.

main QBC **manual lane** 실행은 producer 17/17(`20260925T173906.192056Z-d9ec1b579081`), 최종 runtime dispatch exact 1/1(`20260925T181455.961336Z-693b5e3485ce`), 최신 SDK lifecycle 4/4(`20260925T181736.019920Z-77e5ae285e82`) 모두 exit 0이다. 최종 runtime 테스트는 bounded 재진입의 다른 worker 실행도 단언한다. 각 source digest는 `4cada145…`, `b0f024c0…`, `ec097def…`로 **서로 다르다**. receipt의 `authority_kind=manual`, `owner_execution_receipt=false`, `reuse_eligible=false`이므로 owner qualification이나 동일 pair 제품 증명으로 승격하지 않는다. receipt 경로: `/Users/songmin/Library/Caches/semantica-codegraph-v2-target/quanta-build-cli/_state/execution-roots/semantica-codegraph-v2-d0b8b5a985ea/lanes/gqlang-ss-integration/verification-results/<run-id>/receipt.json`.

격리 SDK lexical fixture는 좁은 proof feature에서 published indexing aggregate provider가 없어 `ensure_indexed` 전에 실패했고, default feature SDK lib-test는 별도 테스트 코드 15개 컴파일 오류로 실행 전 중단됐다. semantic fixture는 `QUANTA_INDEX_SEARCHD_BIN` 부재로 본문 전에 실패했다. 따라서 실제 SDK lexical/semantic SAMPLE의 지연 host timeout은 아직 GREEN이 아니다.

## 초기 결함 근거와 현재 경계

- 실행 rail은 `timeout_ms`를 wall-clock/hard deadline 축으로 정의한다: `S:packages/analysis/quanta-v2/crates/quanta-contract-execution-rail-shared/src/execution_rail_contract_v1.rs:603-612,941-962`. admitted request identity에도 이를 hash한다: `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:414-415`.
- 위의 handler/default budget과 무제한 동기 대기 경로는 수정 전 관찰이다. 현재 main의 deadline/lifecycle은 `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs`, bounded worker는 `S:packages/analysis/quanta-v2/crates/quanta-runtime/src/executor/handle/dispatch.rs`, producer 연결은 `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/gqlang_composition_producer_v3.rs`에 있다. 행 번호는 계속 움직이는 main에서 고정하지 않는다.

## 확인할 재현

1. SDK lexical SAMPLE의 지연 host를 두 barrier와 유한 watchdog으로 제어해 deadline 전 정상 성공, deadline 후 typed non-success·receipt 부재, host 해제 후 정상 요청을 같은 source/generation/provider로 증명한다. fixture 인덱싱 권한 오류를 먼저 해결한다. semantic persisted fixture는 검색 daemon binary/revision/SHA/target-dir custody가 확보될 때만 추가한다.
2. owner/profile rail에서 producer → runtime dispatch → SDK를 **동일 고정 source digest**로 재실행한다. manual lane 17/4/1 PASS를 재사용 가능한 owner receipt로 오인하지 않는다.
3. INT-04 설치형 process에서 timeout·late-success·정상 대조군을 확인한다. 취소가 이미 실행 중인 비협조적 host를 즉시 중단해야 하는 제품 요구라면 별도 cooperative cancellation/격리 worker 설계를 정의하고 시험한다.

## 구현 경계 / 완료 조건

- absolute deadline의 dispatch 전달과 late result 폐기는 공유 main에 구현됐다. 단순 `BudgetIntent` 복사나 완료 후 경과시간 비교로 worker 종료까지 증명했다고 주장하지 않는다.
- timeout/cancel의 typed non-success와 receipt 부재, 늦은 host 완료의 성공 덮어쓰기 방지, host 해제 뒤 재사용을 SDK lexical 지연 fixture에서 실제로 확인해야 한다. runtime dispatch 단위 PASS만으로 이를 대체하지 않는다. budget·generation·provider identity를 보존한다.
- 경계시각·취소·동시 실행 및 정상 짧은 host 양성을 owner rail로 실행하고, INT-04 설치형 경로에서 동일 source digest를 재검증해야 CLOSED다.
