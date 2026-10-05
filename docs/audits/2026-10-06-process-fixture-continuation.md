# Graph/CHECK process fixture continuation

재관측: 2026-10-06. Producer `b7b443f`(이번 문서 수정 전), fixture checkpoint `98141eff6e5`, local lane checkpoint `d224b7ead54`, ACK reader checkpoint `0b418d11b65`, curated binding checkpoint `f5028ee6c85`, ingress caller checkpoint `b576c57feec`, PTA caller checkpoint `62fc21be4fe`. 최종 완료한 product attempt의 frozen consumer는 `693cd7e7eced7d0e82d11c3eaaa7268d9cacc2d7`이며 RED다. 공유 consumer HEAD와 foreign dirty는 계속 변경된다. 실행 증명은 아래의 frozen source별로 구분한다.

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

System Python3.9로 시도한 boundary 검사는 `zip(strict=True)`에서 중단됐으며 project `.venv/bin/python`으로 다시 실행한 위 결과만 유효하다. 아래 product compile은 실행했으나 RED이며, 해당 fixture의 actual process proof는 아직 NOT_RUN이다.

선행 전체 registry 검사는 공용 Rust unit roster의 `native_allocation_demand_v1`2개 unresolved에서 중단됐다. 명시적 `#[path]` 수리가 반영된 frozen `a1d91184`에서 다시 실행한 전체 검사는 정상적으로 끝났지만90 findings로 RED다. GQLang1건은 required signed CHECK의 literal 실행 진입점 누락이며, 나머지89건은 별도 소비자 범위다.

## Local lane와 ACK reader 수리

- `d224b7ead54`: `Justfile`9줄로 `test-gqlang-v3-check-linux-process`와 `test-gqlang-v3-unsupported-host-process`를 연결했다. Canonical suite consumer 검사에서 GQLang findings0, Just dry-run/scoped diff-check/pre-commit patch·safety fence PASS다. Linux 필수 환경·artifact·서명 조건과 expected count는 유지한다.
- 해당 commit은 생성됐지만 coordinator의 post-commit index 복원이 실패했다. 자신의 serial barrier 아래 `Justfile`의 역방향 staged diff만 HEAD와 맞췄고, commit이 main에 retained됨과 경로 clean을 확인했다. 해당 claim/barrier는 해제했다.
- Mac product run `20261005T171304.672815Z-ad6080a60fc0`: frozen `a1d91184`, canonical source `f1b29bd4a9eb3b2f0844ebe3904531892cd026620ed86de252d293a14b659d85`, jobs1, RED/exit101. `aggregate_publication_authority.rs:571/679`의 ordinary observer가 crash-test feature 전용 `checkpoint_ack_v2`를 호출해 `E0599`2건이다. 네 바이너리 bundle과 actual process는 생성·실행하지 못했다.
- `0b418d11b65`: admitted reader의 ACK field/getter/construct cfg를 ordinary reader와 맞췄다. 기존 durable row·artifact·최종 row·manifest pin 검증 뒤 ACK를 move하며 새 String clone을 추가하지 않는다. WrongPhase와 identity drift 거절은 유지한다. Private `reader_ack.rs`는 phase 소비만 맡는다. Public observer API·wire·frontend schema·dependency 변경은 없다.
- ACK 수리는 boundary17, staged rustfmt2, changed safety fence PASS다. Coordinator GOD finding은 변경 전후26개로 동일하고 새·악화0, child0이다. 기존 whole-file baseline RED를 PASS로 바꾸거나 baseline을 완화하지 않았다.
- Frozen `0b418d1`을 canonical casectl lease로 고정했다. 새 lane의 단순 cache clone은 command identity가 없어 거절됐고, canonical fork의 RED donor cache도 `cross_root_source_last_exit_nonzero`로 거절됐다. Cache gate를 유지한 product-shaped compile `20261005T174340.091365Z-164545f5c223`은 ACK 지점을 통과한 뒤 `product_metadata_v3.rs:19`의 curated `AggregateQueryBindingErrorV3` 누락 `E0412`1건으로 종료했다. Exit101, 1,447,555ms, campaign source `df33221afb732fadd503630ecfd6290932db9f3f97c11e9974e8975b705c13c0`, actual process0건. Canonical product publication이나 four-role bundle 성공을 주장하지 않는다.

## Curated binding error 수리

- Consumer는 GQLang SDK executor이며 owning type은 기존 search aggregate owner의 `AggregateQueryBindingErrorV3`다. 오류 cause·reason code·feature cfg·wire 의미는 바꾸지 않는다.
- Acceptance: ordinary product에서 타입을 resolve하고, executor의 runtime SDK 직접 import 금지와 exact curated export gate를 함께 통과한다. 기존 `index-sdk-ingress` cfg를 유지하고 누락 export를 검사기가 거절해야 한다.
- 중간 main `84bf7329d8d`의 직접 `sdk::search_builder_facade` import는 canonical boundary preflight에서 RED였다. Curated `boundary_gqlang_runtime_v1/search_builder.rs`에 타입을 명시 export하고 caller를 기존 curated 경로로 연결했다.
- Fanin inventory의 exact gated block에 해당 오류 타입만 추가했다. Guard의 export 삭제 음성 사례와 기존 wildcard/gate/owner drift 거절 검사를 유지했다. Boundary24 PASS, Python guard28/28 PASS, Rustfmt2 PASS, 해당 production2파일 GOD findings0 PASS. Public/frontend schema·dependency·engine 변경은 없다.
- 작업 중 공유 snapshot `f5028ee6c85a18700e28eb6a5954a4c9e9548863`가 수리4경로를 main에 흡수했다. Commit readback과 경로 clean을 확인하고 claim을 해제했다. ACK claim도 해제했다.
- 같은 committed source의 canonical product run `20261005T181406.687054Z-9d7618d60b31`: source `5f73fb398bfea4a5ebd21e0b864f742e5f1268bf37988e60e88245e3ad9015c2`, exit101, 2,025,911ms. ACK·curated export·SDK executor 지점은 통과했고 CLI ingress에서 후속6개 compiler 오류로 종료했다. 네 바이너리 publication과 actual process proof는 없다.

## Ingress borrowed-to-owned caller 수리

- `ProjectedDaemonRequestV1`의 기존 read-only API는 `core_v1()`이며 core의 `request_id`를 제공한다. `DaemonSearchRequestCopyViewV1`은 `&str`, 실행 요청은 owned `String`을 요구한다.
- Frozen `f5028ee6c85`와 live main 모두 `code_agent.rs`·`search.rs`에서 nonexistent `request_id()`4개와 code-agent query의 반대 ownership2개가 남아 있었다. Compile error6개는 `E0599`4개와 `E0308`2개다.
- `b576c57feec1e24e6b5af60e829e2c0680cb3864`: 기존 core identity를 root issuance 전 빌리고, root admission 성공 후 실행 요청에 복사한다. Code-agent query도 copy view에서 빌리고 owned request 구성 시 복사한다. Deadline·cancel·root 거절 순서와 원래 source authority를 유지한다. 새 accessor/API/engine/dependency는 없다.
- Acceptance: ordinary product profile에서 두 caller의 타입 계약이 맞고, root admission 이전 owned request 복사를 만들지 않는다. Boundary1·GOD2파일0·staged Rustfmt2·staged patch/safety fence PASS. 소유2경로·6줄만 private index로 커밋했고 다른 staged 변경을 보존했다. Claim/barrier 해제, 소유2경로 clean.
- 같은 committed source를 canonical casectl snapshot/lease로 고정해 product·actual Graph/CHECK를 다시 실행한다. Jobs1, frontend1, canonical cache admission 유지. 이 단계의 실제 query 결과는 아직 관측하지 못했다.
- `b576c57feec`의 product run `20261005T185316.385788Z-8aab9e9b04e7`: source `011363e104251994124642c1c81e76a4bb027dba5e58351a937aaa7af1674759`, exit101, 222,441ms. Kernel `context_core/mod.rs:81`의 unused `CanonicalAdmissionFailureV1`·`CanonicalLogicalBytePermitV1` import가 `-D unused-imports`에 걸린1건이다. Ingress caller·actual query 재검증까지 도달하지 못했다.
- 최신 main `27de476a8fa57459478b322aacd2a0a6398fbf20`에 해당 import 제거가 이미 있었다. 중복 source 수정 없이 그 변경과 `b576c57feec`를 포함한 `fc6a554735db67ba1ebb92ab844344852b530e5d`를 다음 canonical product·process 입력으로 고정했다. Lint 설정은 유지한다.

## PTA borrowed target와 admitted key handoff

- Frozen `fc6a554735d`의 product `20261005T190036.472946Z-eb6ce819547d`: source `c5b455e5e69edbc20741704f5b081bb4b2769764a6ffffe01def3f3e292d762d`, exit101, 343,343ms. `quanta-adapters-pta`의 facade 타입 누락3개와 private key field 접근1개로 종료했다. Kernel import 수리는 통과했다. Actual Graph/CHECK0건이다.
- `PtaExactFunctionRefTargetRefV1`은 기존 constraint extractor port의 borrowed 타입이다. 같은 `foundation-cfg-dfg-build-result-floor` cfg를 가진 PTA facade에 명시 export해 기존3caller를 연결했다. Foreign dirty attribution artifact는 편집하지 않았다.
- Accumulator는 admitted requested key set을 private `take_requested_dfg_keys_v1`에서 move하고 sibling handoff가 그 method를 사용한다. BTreeSet clone/추가 allocation 없이 payload·node permits는 accumulator에 유지한다. Keys traversal과 drop·finish 순서, stable BTreeSet ordering, 원래 root authority는 유지한다.
- Boundary6·GOD3파일0·Rustfmt3 PASS. 작업 중 공유 snapshot `62fc21be4fef90766ce236b2976e11b25a27070a`가 수리3개 변경을 흡수했다. Exact source commit 준비는 이미 흡수된 변경을 발견해 main mutation 전에 중단했고 빈·중복 commit을 만들지 않았다. Readback에서 세 변경을 확인했다. Foreign accumulator allocation·formatting 변경을 보존했으며 claim/barrier를 해제했다.
- 다음 frozen product·process source는 `5be58c12f90f19c47cf190a236d7f1e831d076db`다. Source checkpoint와 static gate를 actual process GREEN으로 계산하지 않는다.
- 해당 product run `20261005T191654.905048Z-2a5df62f802b`: source `e30b0f45e5944d756cc8dbc53d642aa2a26ed511af7cc16f29122649e37dec08`, exit101, 188,112ms. DFG synthetic publication child3개에서 `QueryId`·`REASON_CODE_QUERY_FAILED` imports가 빠져 compiler 오류15개로 종료했다. Actual Graph/CHECK0건이다.
- 기존 main `c8e025531c2b0ce2982db63ec9e0670d6e9c65ae`가 `slot_assignment_error.rs`·`statement_membership.rs`·`typed_publication.rs`의 explicit owner imports를 이미 수리했다. 중복 source 수정 없이 모두 포함한 `14cd42b7ed9b2562f1971cc38c5ebb83a4b8ace7`를 다음 canonical product·process source로 고정한다.
- `14cd42b7ed9`의 product run `20261005T192331.381715Z-22b70874a114`: source `24e319d755be82e926bf8490122434f38c99c267c8a00142845a9afe03e91c27`, exit101, 254,450ms. `native_generation_binding_index_v1.rs`가 exact FunctionRef index를 옛 parent에서 import해 `E0432`와 후속 `E0282`로 종료했다. Actual Graph/CHECK0건이다.
- 기존 main `a384b1e6df87cb9c145c6b34c4ee648205a4a862`가 private `native_extracted_constraint_binding_v1` owner에서 기존 alias를 직접 import하도록 수리했다. 중복 source 수정·shim·type annotation을 추가하지 않았다. 그 수리를 포함한 `cbbfe47ce212a38ffb2137592065bab631400070`에서 canonical product·process를 재검증 중이다.
- `cbbfe47ce21`의 product run `20261005T193141.649451Z-f2c5ebb1d4c1`: source `98d6a1f43d718cc00f0b22e3dcd22fe19718496e32cf93bffcec2883e6a95291`, exit101, 678,658ms. Runtime-analysis의 missing lease field1, delta helper generic mismatch2, packet view mismatch1, opaque snapshot field/후속 inference7, FunctionRef binding move-after-borrow1로 종료했다. Actual Graph/CHECK0건이다.
- 기존 main `9a05e53cd37`의 delta helper generic, `99234f0bca1`의 snapshot `.as_ref()`·packet `.bytes_v1()`, `6be819ecd08`의 lease field·original source owner retention 수리를 해당6경로 readback으로 확인했다. 새 source 수정 없이 모두 포함한 `693cd7e7eced7d0e82d11c3eaaa7268d9cacc2d7`를 다음 canonical product·process 입력으로 고정했다.

## 현재 product compile 차단

- Frozen `693cd7e7ece`의 canonical product run `20261005T194447.127472Z-bb8fa5a945cc`: source `5c43ace395588f9dd138bf662ee9729155afe716d528e007fd8c87751fa803cf`, exit101, 718,745ms. 선행 runtime-analysis 오류 지점을 통과한 뒤 `quanta-runtime`11경로에서 compiler 오류31건으로 종료했다. Four-role bundle 및 actual Graph/CHECK0건이다.
- Transcript 연결14건: session-id에서 `SessionTranscriptHandleV1`으로의 전환 중 ports·analysis-session·injector 계약이 서로 다르다. 기존 foreign dirty4경로와 QMP/host cutover가 해당 부분을 변경 중이다.
- Resume/publication 연결17건: 제거된 rootless prepare export, child reason/type import, original execution-session argument, Arc registry borrow, layout admission error mapping, hydration refusal exhaustiveness, temporary manifest view lifetime이다. 기존 IRC recovery/integration claim과 겹친다. Main `7afbb088133` 대조에서11경로의 committed delta0, foreign dirty5경로를 확인했다. Source를 통합하거나 typed 원래 session을 가짜/default authority로 대체하지 않았다.
- AG5 SDK의 별도 immutable run `20261005T193555.409816Z-33b000ebfa9b`도 exit101/actual0이다. Source `735039e37ad2774bd414298cae01f7026377030544599ecbac5948e7011f695b`에서 QMP pin `14cf0345`에 없는 `SessionHandle`·`open_session`·`retire_session`3개가 원인이다. 이 결과를 위 product31오류와 합산하지 않는다.
- 기존 QMP 담당 작업은 source checkpoint `8c85dae`와 workspace900/feature18/document selector353 결과를 보고했고 추출 검증·host pin 통합을 진행 중이다. 이는 담당 작업의 보고이며 이번 Graph/CHECK 실행 proof가 아니다. Consumer manifest의 actual pin 갱신과 동일 source product를 확인한 뒤 다음 실행을 한다.

## 남은 실행 전제

1. 소비자 compile closure: 선행 수리에 더해 transcript/QMP pin 및 resume/publication caller·export·typed admission 연결을 기존 소유자가 main에 통합한다. 원래 session handle·refusal을 유지하고 같은 frozen source에서 compiler와 actual attach를 확인한다. 과거 compiler 실패는 원래 source에 보존하며 최신 main 전체 결함으로 합산하지 않는다. SDK owner의 검증을 중복하지 않는다.
2. Mac non-Linux Graph/CHECK typed refusal: 최신 frozen source의 receipt-bound CLI/server/worker bundle과 executing test target을 맞춰 실행한다. 실패는 native boundary·empty payload·no receipt까지 관측해야 한다.
3. Linux Graph Path/Reachability: generic aarch64 four-role product bundle로 별도 actual query proof를 실행할 수 있다. 기존 컨테이너에 lockfile의 `architecture-tooling`21개 package를 Python3.12 환경으로 오프라인 복원했다. Primary Git mount가 읽기 전용이어서 canonical worktree 생성은 거절됐다. 전용 native clone3개에서 consumer `693cd7e7ece`/producer `b7b443f`/Index `66f92d2`의 clean 상태를 확인하고 canonical snapshot/lease 준비를 통과했다. Product/bundle/Graph exact selector 실행은 준비 상태이며 시작하지 않았다. 공식 설치형 qualification으로 확대하지 않는다.
4. Signed Linux CHECK0/1/2: 2026-10-06 live Docker 재확인에서도 `aarch64`이고 `/etc/semantica-codegraph-v2/trust`는 없다. 공식 `quanta-cli-product.release-full.linux-installed-build`는 `x86_64-unknown-linux-gnu` 및 installed authority features를 요구한다. 공식 bundle/대상 환경·deployment approval·root protected key/active generation 입력을 요청했다. 입력은 아직 제공되지 않았다.
5. Source checkpoint `98141eff6e5`의 회귀를 같은 frozen source에서 actual 검증하고 receipt를 기록한다. Consumer main 반영을 actual GREEN으로 계산하지 않는다. Graph Linux cfg 들여쓰기2줄도 후속 공유 snapshot에 흡수됐고 소유2경로는 clean이다. Source checkpoint 뒤 claim을 해제했다.

## 증거

[후보·소스 검토](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/fixture-source-review.json), [소유 patch](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/current-owned-candidate.patch), [scoped registry 검증](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/scoped-registry-validation.json).

[전체 registry90 findings](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/global-registry.json), [local lane 검증](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/local-lane-validation.json), [local lane main readback](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/lane-main-checkpoint-reconciled.json), [product RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/product-result.json), [ACK GOD delta](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/ack-reader-god-delta.json), [수리 source 재검증 요청](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v3/request.json).

기존 owner/kernel/local CI 결과는 원래 source에 유지한다. 이 후보의 actual process 또는 공식 installed qualification으로 합성하지 않는다.

[ACK 이후 compile RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v3/terminal-compile-result.json), [curated boundary 검사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/curated-binding-error-boundary.log), [guard28개 검사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/curated-binding-error-tests.log), [main 흡수 readback](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/curated-binding-error-absorbed.json), [frozen product 요청](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v4/request.json).

[CLI ingress compile RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v4/terminal-compile-result.json), [ingress main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v4/ingress-copy-view-main-checkpoint.json), [ingress boundary 검사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v4/ingress-copy-view-boundary.log), [curated GOD 검사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final/curated-binding-error-god.log), [live Linux 설치 전제](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/linux-installed-preconditions-live-20261006.json).

[Kernel import compile RED와 기존 main 수리](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v5/terminal-compile-result.json).

[PTA compile RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v6/terminal-compile-result.json), [PTA 수리 main readback](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v6/pta-caller-main-absorbed.json), [PTA boundary 검사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v6/pta-caller-boundary.log), [foreign source 보존](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v6/pta-caller-foreign-preservation.json).

[DFG child import RED와 기존 main 수리](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v7/terminal-compile-result.json).

[PTA private index import RED와 기존 main 수리](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v8/terminal-compile-result.json), [다음 frozen product 요청](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v9/request.json).

[Runtime-analysis 타입/borrow compile RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v9/terminal-compile-result.json).

[현재 product compiler31 RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v10/terminal-compile-result.json), [고정 source 요청](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/mac-final-v10/request.json), [Linux native base 입력](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/current-process-preparation-20261006/linux-native-base-source-inputs.json).
