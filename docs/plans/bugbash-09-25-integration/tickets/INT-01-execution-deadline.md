# INT-01 — V3 Composition 실행 deadline 검증·수정

상태: **결함 후보 / 재현 필요** · 우선순위: P1 · 소유: Semantica SDK ingress + runtime task/host owner. QGLang frontend 변경은 전제하지 않는다.

## 근거

- 실행 rail은 `timeout_ms`를 wall-clock/hard deadline 축으로 정의한다: `S:packages/analysis/quanta-v2/crates/quanta-contract-execution-rail-shared/src/execution_rail_contract_v1.rs:603-612,941-962`. admitted request identity에도 이를 hash한다: `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:414-415`.
- V3 handler는 SAMPLE host context에 `BudgetIntentV1::default()`를 넣는다: `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:1020-1026`. 이 기본값의 `time_budget_ms`는 `None`: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/src/profile.rs:26-31`.
- provider admission과 SAMPLE producer는 동기 task를 사용한다: `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/gqlang_composition_producer_v3.rs:102-120,227-230`. dispatch sync closure는 요청 deadline control 없이 결과를 기다린다: `S:packages/analysis/quanta-v2/crates/quanta-runtime/src/executor/handle/dispatch.rs:2191-2216`.
- dense store/connect에만 별도 `start + timeout`이 있다: `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:801-825`. 이후 SAMPLE 작업의 timeout 증명은 아니다.

## 확인할 재현

1. 기존 owner fixture에서 lexical SAMPLE host를 `Bounded(1ms)` 이후까지 의도적으로 지연한다. 같은 source/generation/provider/budget으로 완료된 host 결과를 돌려준다.
2. 현 경로가 deadline 뒤에도 성공·receipt를 발행하는지, 혹은 worker가 응답 없이 대기하는지 측정한다. semantic SAMPLE과 canonical-files 물질화도 분리해서 확인한다. 외부 I/O·scheduler 지연을 포함하되 테스트는 bounded/fail-fast여야 한다.
3. 재현되지 않으면 다른 실제 deadline enforcement 지점을 정확한 caller/테스트로 문서화하고 이 후보를 닫는다. 코드만 보고 확정 버그로 승격하지 않는다.

## 수정 원칙 / 완료 조건

- 단일 ingress 시작 시점에서 절대 deadline을 산출하고 provider admission, producer task, host 호출, 결과/receipt 발행에 같은 authority를 전달한다. 단순 `BudgetIntent` 값 복사나 완료 후 경과시간 비교만으로 worker 점유·늦은 성공을 해결했다고 보지 않는다.
- timeout/cancel 시 typed non-success와 receipt 부재가 유지되고, 늦게 완료한 작업이 성공을 덮어쓰지 않는다. budget·generation·provider identity는 기존처럼 보존한다.
- 지연 host RED→GREEN, 정상 짧은 host 양성, 경계시각·취소·동시 실행 테스트를 owner rail로 실행한다. 이후 INT-04 설치형 경로에서 동일 source digest를 재검증한다.
