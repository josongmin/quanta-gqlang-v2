# Graph/CHECK process fixture continuation

재관측: 2026-10-06. Producer `af5c5b84bdc`, fixture checkpoint `98141eff6e5`, local lane checkpoint `d224b7ead54`, ACK reader checkpoint `0b418d11b65`. 공유 consumer HEAD와 foreign dirty는 계속 변경된다. 실행 증명은 아래의 frozen source별로 구분한다.

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

선행 전체 registry 검사는 공용 Rust unit roster의 `native_allocation_demand_v1`2개 unresolved에서 중단됐다. 명시적 `#[path]` 수리가 반영된 frozen `a1d91184`에서 다시 실행한 전체 검사는 정상적으로 끝났지만90 findings로 RED다. GQLang1건은 required signed CHECK의 literal 실행 진입점 누락이며, 나머지89건은 별도 소비자 범위다.

## Local lane와 ACK reader 수리

- `d224b7ead54`: `Justfile`9줄로 `test-gqlang-v3-check-linux-process`와 `test-gqlang-v3-unsupported-host-process`를 연결했다. Canonical suite consumer 검사에서 GQLang findings0, Just dry-run/scoped diff-check/pre-commit patch·safety fence PASS다. Linux 필수 환경·artifact·서명 조건과 expected count는 유지한다.
- 해당 commit은 생성됐지만 coordinator의 post-commit index 복원이 실패했다. 자신의 serial barrier 아래 `Justfile`의 역방향 staged diff만 HEAD와 맞췄고, commit이 main에 retained됨과 경로 clean을 확인했다. 해당 claim/barrier는 해제했다.
- Mac product run `20261005T171304.672815Z-ad6080a60fc0`: frozen `a1d91184`, canonical source `f1b29bd4a9eb3b2f0844ebe3904531892cd026620ed86de252d293a14b659d85`, jobs1, RED/exit101. `aggregate_publication_authority.rs:571/679`의 ordinary observer가 crash-test feature 전용 `checkpoint_ack_v2`를 호출해 `E0599`2건이다. 네 바이너리 bundle과 actual process는 생성·실행하지 못했다.
- `0b418d11b65`: admitted reader의 ACK field/getter/construct cfg를 ordinary reader와 맞췄다. 기존 durable row·artifact·최종 row·manifest pin 검증 뒤 ACK를 move하며 새 String clone을 추가하지 않는다. WrongPhase와 identity drift 거절은 유지한다. Private `reader_ack.rs`는 phase 소비만 맡는다. Public observer API·wire·frontend schema·dependency 변경은 없다.
- ACK 수리는 boundary17, staged rustfmt2, changed safety fence PASS다. Coordinator GOD finding은 변경 전후26개로 동일하고 새·악화0, child0이다. 기존 whole-file baseline RED를 PASS로 바꾸거나 baseline을 완화하지 않았다.
- Frozen `0b418d1`을 canonical casectl lease로 고정했다. 새 lane의 단순 cache clone은 command identity가 없어 거절됐고, canonical fork의 RED donor cache도 `cross_root_source_last_exit_nonzero`로 거절됐다. Cache gate를 유지한 채 해당 source의 재빌드를 진행한다. 이것은 actual process GREEN이 아니다.

## 남은 실행 전제

1. SDK/PTA 공통 compile closure와 ordinary ACK reader: 선행 SDK borrow 수리 및 이번 ACK cfg 수리를 소비하는 같은 frozen source에서 compiler와 actual attach를 확인한다. 선행 `E0515`·`E0599`는 원래 source에 보존하며 최신 main 전체 결함으로 합산하지 않는다. SDK owner의 검증을 중복하지 않는다.
2. Mac non-Linux Graph/CHECK typed refusal: 최신 frozen source의 receipt-bound CLI/server/worker bundle과 executing test target을 맞춰 실행한다. 실패는 native boundary·empty payload·no receipt까지 관측해야 한다.
3. Linux Graph Path/Reachability와 signed CHECK0/1/2: 로컬 Docker는 `aarch64`; `/etc/semantica-codegraph-v2/trust`는 없다. 공식 `quanta-cli-product.release-full.linux-installed-build`는 `x86_64-unknown-linux-gnu` 및 installed authority features를 요구한다. 공식 bundle/대상 환경·deployment approval·root protected key/active generation 입력을 요청했다. 입력은 아직 제공되지 않았다.
4. Source checkpoint `98141eff6e5`의 회귀를 같은 frozen source에서 actual 검증하고 receipt를 기록한다. Consumer main 반영을 actual GREEN으로 계산하지 않는다. Graph Linux cfg 들여쓰기2줄도 후속 공유 snapshot에 흡수됐고 소유2경로는 clean이다. Source checkpoint 뒤 claim을 해제했다.

## 증거

[후보·소스 검토](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/fixture-source-review.json), [소유 patch](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/current-owned-candidate.patch), [scoped registry 검증](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/scoped-registry-validation.json).

[전체 registry90 findings](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/global-registry.json), [local lane 검증](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/local-lane-validation.json), [local lane main readback](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/lane-main-checkpoint-reconciled.json), [product RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/product-result.json), [ACK GOD delta](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/ack-reader-god-delta.json), [수리 source 재검증 요청](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v3/request.json).

기존 owner/kernel/local CI 결과는 원래 source에 유지한다. 이 후보의 actual process 또는 공식 installed qualification으로 합성하지 않는다.
