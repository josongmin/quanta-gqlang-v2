# Graph/CHECK process fixture continuation

관측: 2026-10-06 01:48 KST. Producer `57313ae17ab`, consumer fixture 대조 `98141eff6e5`. 공유 consumer HEAD와 foreign dirty는 계속 변경된다. 이 문서는 소스 수리 후보와 실행 증명을 구분한다.

## 적용한 후보

Consumer write-set:

- `crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs`
- `tools/testing/suite-registry-v1.yaml`

`FCP-PROCESS-FIXTURES-20261006` claim으로 기존 AG5 integration claim과 경로 overlap을 기록했다. 기존 private capture harness와 M23-120 raw observation을 보존했다. Production API, wire, dependency, engine 변경은 없다. 이후 공유 snapshot `98141eff6e57f6557e1360b6b7161d5c00054ca8`가 소유2경로를 main에 흡수했다. 이 checkpoint에 actual query proof는 없다.

- CHECK source와 Python3.12 build context를 daemon 시작 전에 materialize한다.
- CHECK configured workspace root와 실제 attach root를 일치시키고 case마다 fresh daemon을 사용한다. Dense state는 별도 디렉터리다.
- CHECK 양성 source는 같은 모듈의 선언된 함수 호출을 사용한다. Python AST 독립 oracle은 호출0/1/2와 모든 target 선언을 확인했다. Signed pattern/query/budget/response/foreign/short 단언은 유지한다.
- Graph parser를 실제 설치하는 production caller는 `shared_execution_authority_adapter_v1.rs`의 Linux cfg다. Path/Reachability 성공 selector를 Linux에 한정하고 non-Linux native catalog 거절 selector를 추가했다. Registry의 required Linux 성공 행을 유지하며 non-Linux 거절을 별도 행으로 등록한다. 실행0건은 GREEN이 아니다.

## 실행한 검사

| 검사 | 결과·범위 |
| --- | --- |
| 독립 Python AST | PASS: CHECK0/1/2 호출 및 선언된 target |
| Rustfmt parse | PASS: Rust 구문만 확인, compile 아님 |
| `git diff --check` | PASS: 소유2경로 |
| Boundary preflight | PASS: `quanta-workspace.child-exact-spawn-ssot.v1` |
| Scoped test topology | PASS |
| Scoped suite registry | PASS: Graph/CHECK4행의 schema·owner·target·filter·feature class·expected count |
| GOD surface | 해당 없음: production-file gate가 integration test를 제외한다. PASS로 계산하지 않는다. |

System Python3.9로 시도한 boundary 검사는 `zip(strict=True)`에서 중단됐으며 project `.venv/bin/python`으로 다시 실행한 위 결과만 유효하다. Actual process proof와 Rust compile은 이번 후보에서 아직 NOT_RUN이다.

전체 registry 검사는 별도 RED다: 공용 Rust unit roster가 PTA의 `dfg_target_ordinal_index`와 `worker_dfg_symbol_index`에서 child `native_allocation_demand_v1`2개를 unresolved로 판정했다. 두 child 파일은 각 모듈 하위에 존재한다. Cargo 실행 증거가 없으므로 runtime 코드 결함으로 단정하지 않는다. 이 결과는 소유4행의 scoped 검증과 합산하지 않는다.

## 남은 실행 전제

1. SDK/PTA 공통 compile closure: 최신 직접 확인 run `20261005T164051.089530Z-b7cb2b38d81c`, source `1c25c311e1d750e2ab255ecfdbbed73c32a1785de616a17067175abf4614c2db`는 quarantine owner unused import로 RED, 실행0건이다. 담당 작업의 해당 수정과 재검증을 중복하지 않는다. 이것은 actual PTA bootstrap 실패의 해소 증명이 아니다.
2. Mac non-Linux Graph/CHECK typed refusal: 최신 frozen source의 receipt-bound CLI/server/worker bundle과 executing test target을 맞춰 실행한다. 실패는 native boundary·empty payload·no receipt까지 관측해야 한다.
3. Linux Graph Path/Reachability와 signed CHECK0/1/2: 로컬 Docker는 `aarch64`; `/etc/semantica-codegraph-v2/trust`는 없다. 공식 `quanta-cli-product.release-full.linux-installed-build`는 `x86_64-unknown-linux-gnu` 및 installed authority features를 요구한다. 공식 bundle/대상 환경·deployment approval·root protected key/active generation 입력을 요청했다. 입력은 아직 제공되지 않았다.
4. Source checkpoint `98141eff6e5`의 회귀를 같은 frozen source에서 actual 검증하고 receipt를 기록한다. Consumer main 반영을 actual GREEN으로 계산하지 않는다. Graph Linux cfg 들여쓰기2줄도 후속 공유 snapshot에 흡수됐고 소유2경로는 clean이다. Source checkpoint 뒤 claim을 해제했다.

## 증거

[후보·소스 검토](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/fixture-source-review.json), [소유 patch](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/current-owned-candidate.patch), [scoped registry 검증](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/scoped-registry-validation.json).

기존 owner/kernel/local CI 결과는 원래 source에 유지한다. 이 후보의 actual process 또는 공식 installed qualification으로 합성하지 않는다.
