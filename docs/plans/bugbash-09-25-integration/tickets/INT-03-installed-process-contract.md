# INT-03 — 설치형 full-provider 테스트와 V3 지원 계약 정합화

상태: **소스상 테스트 기대 충돌 / 현재 process 실행 전** · 우선순위: P1 테스트 게이트 · 소유: Semantica CLI/server + SDK public query owner.

2026-09-25 current-source 재검증에서도 설치형 테스트는 단독 `check ...`와 단독 semantic `sample ...`의 성공·반복결정성을 기대한다. SDK owner tests와 QGLang V3 계약의 standalone-family typed reject 경계는 변하지 않았다. 현재 pair에서 이 exact process test를 실행한 receipt는 없으며, 검색된 최신 GQLang process receipts는 2026-09-23의 이전 source digest 실패다.

## 근거

- 설치형 `daemon_process_full_gqlang_provider_packet_serves_pattern_and_dense_v1`은 CLI `gqlang text`를 통해 단독 `check r.x ...`와 단독 `sample ... mode=semantic`의 성공을 기대한다: `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:4116-4120,4183-4202,4220-4240`. helper의 실제 CLI argv는 같은 파일 `2687-2748`이다.
- 현 SDK V3 테스트는 단독 semantic SAMPLE과 단독 pattern CHECK를 `gqlang_v3_family_unsupported`로 거부한다: `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/tests/group_gqlang_public_path.rs:1961-1981,2576-2624`. QGLang 계약도 Graph/CHECK/Presentation의 standalone 실행 payload를 약속하지 않는다: `Q:docs/contracts/frontend-execution-v3.md:18-22`.
- 이 테스트는 `gqlang-process-proof` feature이고, 제품 recipe `S:tools/quanta-build-cli/product-recipes-v1.toml:114-125`의 filter 없는 `server_socket_daemon_e2e`에 포함된다. 과거 실패 로그만으로 현 source에서 이 케이스의 실패 원인을 확정하지 않는다.

## 확인·해결

1. receipt-bound 설치 artifact로 이 테스트를 먼저 exact-filter 실행하여 첫 실제 실패 위치/응답 reason을 기록한다. 단순 소스 모순을 실행 판정으로 오인하지 않는다.
2. 현 제품 계약이 V3 Composition뿐이면 단독 CHECK/SAMPLE 기대를 성공으로 유지하지 않는다. 단독 family는 typed reject/no effects로 검증하고, semantic SAMPLE 양성은 `let sampled = sample ... from sampled ... with sampled`의 V3 Composition으로 옮긴다.
3. Pattern signed packet에 독립적인 실제 공개 실행 경로가 있다면 그 경로의 process proof를 분리해 유지한다. 없으면 지원 주장/테스트를 정확히 축소한다. provider packet 검증 자체를 조용히 삭제하지 않는다.
4. lexical·semantic Composition 모두 socket/daemon/receipt, 반복 결정성, foreign generation/provider·scope 거부를 추가한다. 이미 있는 SDK owner 테스트와 중복 책임을 분리한다.

## 완료 조건

- exact-filter process RED 원인과 post-fix GREEN이 동일 pair source digest 및 설치 artifact receipt에 연결된다.
- filter 없는 제품 process recipe에서 모든 GQLang 테스트가 실제 선택·실행되고 통과한다. `#[ignore]`, `required:false`, feature 선택, 환경 누락으로 통과처럼 보이는 결과를 허용하지 않는다.
- 지원하지 않는 family를 정상 결과로 포장하거나 기존 Pattern/dense proof를 삭제해 녹색으로 만들지 않는다.
