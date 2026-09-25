# INT-01 — V3 Composition 실행 deadline 검증·수정

상태: **결함 후보 / 재현 필요** · 우선순위: P1 · 소유: Semantica SDK ingress + runtime task/host owner. QGLang frontend 변경은 전제하지 않는다.

2026-09-25 current-source 재검증에서도 상태는 그대로다. handler는 dense provider connect에만 별도 deadline을 계산하고, 이후 Composition host context에는 여전히 `BudgetIntentV1::default()`를 넣는다. provider admission과 SAMPLE 실행은 deadline 입력이 없는 `run_sync_serving_task_v1`을 사용한다. GQLang delayed-host RED/GREEN 테스트는 검색 범위에서 확인되지 않았다.

## 근거

- 실행 rail은 `timeout_ms`를 wall-clock/hard deadline 축으로 정의한다: `S:packages/analysis/quanta-v2/crates/quanta-contract-execution-rail-shared/src/execution_rail_contract_v1.rs:603-612,941-962`. admitted request identity에도 이를 hash한다: `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:414-415`.
- V3 handler는 SAMPLE host context에 `BudgetIntentV1::default()`를 넣는다: `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:1020-1026`. 이 기본값의 `time_budget_ms`는 `None`: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/src/profile.rs:26-31`.
- provider admission과 SAMPLE producer는 동기 task를 사용한다: `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/gqlang_composition_producer_v3.rs:102-120,227-230`. dispatch sync closure는 요청 deadline control 없이 결과를 기다린다: `S:packages/analysis/quanta-v2/crates/quanta-runtime/src/executor/handle/dispatch.rs:2191-2216`.
- dense store/connect에만 별도 `start + timeout`이 있다: `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:801-825`. 이후 SAMPLE 작업의 timeout 증명은 아니다.

## 확인할 재현

1. 기존 owner fixture의 lexical SAMPLE host를 동기 barrier로 멈추고, admitted timeout 경과 후 release한다. 1ms sleep 경합 대신 두 barrier와 유한 watchdog으로 순서를 고정한다. 같은 source/generation/provider/budget의 정상 host를 대조군으로 둔다.
2. timeout 전/후의 응답·receipt 유무, late host 완료의 성공 덮어쓰기 여부, worker 점유/해제 여부를 따로 기록한다. 첫 재현에는 semantic SAMPLE·canonical-files까지 늘리지 않는다.
3. 재현되지 않으면 실제 enforcement 지점과 선택·실행된 테스트를 기록해 이 후보를 닫는다. RED면 lexical 경로를 먼저 수정하고 그 뒤 semantic/canonical 경계를 확장 검증한다.

## 수정 원칙 / 완료 조건

- RED일 때 ingress 시작 시점의 절대 deadline을 provider admission, producer task, host 호출, 응답/receipt 발행에 전달한다. 단순 `BudgetIntent` 복사나 완료 후 경과시간 비교로 worker 종료까지 증명했다고 주장하지 않는다.
- timeout/cancel 시 typed non-success와 receipt 부재가 유지되고, 늦게 완료한 작업이 성공을 덮어쓰지 않는다. 동기 host의 강제 중단이 불가능하면 응답 차단과 worker 회수를 별도 결과로 표시한다. budget·generation·provider identity는 보존한다.
- 지연 host RED→GREEN, 정상 짧은 host 양성, 경계시각·취소·동시 실행 테스트를 owner rail로 실행한다. 이후 INT-04 설치형 경로에서 동일 source digest를 재검증한다.
