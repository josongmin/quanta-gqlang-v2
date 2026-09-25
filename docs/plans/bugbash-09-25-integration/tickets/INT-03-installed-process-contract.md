# INT-03 — 설치형 full-provider 테스트와 V3 지원 계약 정합화

상태: **계약 정합화 코드 반영 / 타깃 컴파일·process 실행 미완료** · 우선순위: P1 테스트 게이트 · 소유: Semantica CLI/server + SDK public query owner.

2026-09-25 current-source 재검증에서도 설치형 테스트는 단독 `check ...`와 단독 semantic `sample ...`의 성공·반복결정성을 기대한다. SDK owner tests와 QGLang V3 계약의 standalone-family typed reject 경계는 변하지 않았다. 현재 pair에서 이 exact process test를 실행한 receipt는 없으며, 검색된 최신 GQLang process receipts는 2026-09-23의 이전 source digest 실패다.

2026-09-26 수정: `server_socket_daemon_e2e.rs`의 해당 테스트를 named-scope/standalone CHECK/standalone SAMPLE/Hybrid의 typed reject·receipt 부재와 semantic SAMPLE Composition의 양성·반복결정성·generation/receipt 검증으로 변경했다. 이전 V1 Pattern candidate / standalone dense result oracle helper는 제거했다. 이 변경은 공유 Semantica main의 동시 커밋 `130a7f1f5e9`에 포함됐다. QBC `owner run --lane gqlang-ss-integration --package quanta-cli-server --execution-kind check --target-kind integration --target-name server_socket_daemon_e2e --compile-policy feature-isolation:quanta-cli-server.no-default.61b2510e7d2a`는 exit 101이며, 실패는 변경된 테스트가 아니라 `workspace_journal_operator_runtime_v1.rs:182`의 다른 동시 작업 `SourceAwareCloseout` non-exhaustive match(E0004)다. source snapshot digest `8b1b457dacc5121ea96ff9afa7d67c4e40d0437b4c1fc4abcecb3bebdf5958d4`. 따라서 process PASS는 아직 주장하지 않는다.

## 근거

- 설치형 `daemon_process_full_gqlang_provider_packet_serves_pattern_and_dense_v1`은 CLI `gqlang text`를 통해 단독 `check r.x ...`와 단독 `sample ... mode=semantic`의 성공을 기대한다: `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:4116-4120,4183-4202,4220-4240`. helper의 실제 CLI argv는 같은 파일 `2687-2748`이다.
- 현 SDK V3 테스트는 단독 semantic SAMPLE과 단독 pattern CHECK를 `gqlang_v3_family_unsupported`로 거부한다: `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/tests/group_gqlang_public_path.rs:1961-1981,2576-2624`. QGLang 계약도 Graph/CHECK/Presentation의 standalone 실행 payload를 약속하지 않는다: `Q:docs/contracts/frontend-execution-v3.md:18-22`.
- 이 테스트는 `gqlang-process-proof` feature이고, 제품 recipe `S:tools/quanta-build-cli/product-recipes-v1.toml:114-125`의 filter 없는 `server_socket_daemon_e2e`에 포함된다. 과거 실패 로그만으로 현 source에서 이 케이스의 실패 원인을 확정하지 않는다.

## 확인·해결

1. 현재 V3 계약에서는 단독 CHECK/SAMPLE 성공 oracle이 불가능하므로 이를 보기 위한 비싼 설치형 RED 실행은 생략한다. SDK의 standalone typed reject를 유지하고, process 테스트에서는 해당 요청의 typed reject/no effects를 검증한다.
2. semantic SAMPLE 양성은 `let sampled = sample ... from sampled ... with sampled` V3 Composition으로 교체한다. 기존 persisted dense publication fixture·반복 결정성·generation 검사는 유지한다. lexical Composition 양성은 기존 SDK proof와 역할을 구분해 소켓/daemon 경계에 한 개만 추가한다.
3. Pattern signed packet의 별도 공개 실행 경로가 실제 있다면 그 process test로 이동한다. 없다면 V3 CHECK 성공 주장을 제거하되 packet 생성/검증 자체는 owner rail에 남긴다. 기능을 열기 위해 QGLang의 standalone 실행 계약을 임의 확장하지 않는다.
4. 수정 뒤 `dist-full,gqlang-process-proof` 설치 artifact와 receipt를 생성한다. 이 exact test를 실행한 다음 INT-04의 filter 없는 전체 process recipe를 한 번 실행한다. SDK에 이미 있는 foreign generation/provider 거부를 설치형에서 무조건 복제하지 않고, 소켓 경계에서만 추가로 위험한 identity/receipt 사례를 선택한다.

## 완료 조건

- 소스상 기존 기대 충돌과 수정 diff가 기록되고, post-fix exact-filter process GREEN이 pair source digest 및 설치 artifact receipt에 연결된다. pre-fix process RED 영수증은 필수 아님.
- filter 없는 제품 process recipe에서 모든 GQLang 테스트가 실제 선택·실행되고 통과한다. `#[ignore]`, `required:false`, feature 선택, 환경 누락으로 통과처럼 보이는 결과를 허용하지 않는다.
- 지원하지 않는 family를 정상 결과로 포장하거나 기존 Pattern/dense proof를 삭제해 녹색으로 만들지 않는다.
