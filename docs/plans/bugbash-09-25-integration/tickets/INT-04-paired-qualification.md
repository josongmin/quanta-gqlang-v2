# INT-04 — 동일 pair source의 통합 qualification

상태: **증명 공백** · 우선순위: release blocker · 소유: 두 저장소 integrator / Semantica product proof owner.

## 현재 증거

- QGLang `14e2d126...`의 clean committed-source qualification receipt는 complete/exit 0, Nextest `1239/1239`다. 현재 QGLang dirty 변경과 Semantica 실행까지 증명하지 않는다.
- Semantica는 sibling `Q:crates/quanta-gqlang-v2-frontend-contract`를 path dependency로 소비한다: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:17-20,37`. QBC campaign source closure에는 sibling path dependency가 포함된다: `S:tools/quanta-build-cli/campaign_source_closure_v1.py:171-228`.
- 과거 QBC log에는 exec-contract/public-ingress의 옛 digest PASS, SDK GQLang 중단, 설치형 process target의 옛 digest 실패가 있다. 감사 당시 현재 digest에 대한 제품 PASS는 확인되지 않았다. 중단은 test RED가 아니다.
- `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:3819-3910`은 V3 from-files의 실제 소켓/authority drift를 검사한다. lexical·semantic V3 Composition은 SDK owner 테스트가 있지만 설치형 process 양성은 확인되지 않았다. SDK 등록: `S:tools/testing/suite-registry-v1.yaml:646-775`.

## 실행 순서와 증거

1. INT-00~03의 판정/수정을 반영한 뒤 양쪽 HEAD·tree·dirty/untracked를 고정한다. QGLang의 현 수정이 남으면 이전 clean receipt를 재사용하지 않는다. Semantica QBC source digest에 sibling QGLang이 포함됐는지 기록한다. QGLang 자체 receipt와 QBC digest의 문자열 동일성을 요구하지 않는다. INT-00의 새 DTO/digest가 확정되기 전에는 현 pair를 승인하지 않는다.
2. QGLang은 clean committed HEAD의 `just ci` / `just qualify-local` 범위로 producer proof를 갱신한다. Semantica는 등록된 QBC owner/profile rail에서 exec-contract → ingress → SDK GQLang 테스트를 동일 digest로 실행한다. 새로운 임의 QBC lane을 조용히 만들지 않는다.
3. `dist-full,gqlang-process-proof` 설치 bundle/receipt를 생성하고, `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:253-266`의 환경 입력과 binary SHA를 대조한다. INT-03의 exact test 후 filter 없는 `S:tools/quanta-build-cli/product-recipes-v1.toml:114-125` process target을 실행한다.
4. 각 영수증에 pair HEAD, dirty/source digest, toolchain, feature, target, selected/executed test count, exit, 설치 binary/receipt identity를 남긴다. 실패·중단·skip·미선택을 PASS로 계산하지 않는다. 범위가 변하면 영향받은 단계부터 재실행한다.

## 완료 조건

- 두 저장소 producer/consumer + SDK + 설치형 daemon/process가 동일한 고정 소스에서 통과하고, V3 files/lexical/semantic Composition 및 권한 drift가 실제 선택돼 실행됐음이 확인된다.
- QBC의 sibling source closure와 설치 artifact custody가 서로 일치한다. 로컬 증거로 충분하며 GitHub Actions 활성화는 요구하지 않는다.
- Graph/CHECK/Presentation의 일반 실행, Rust/Java lexical SAMPLE, 15개 모든 producer 벡터의 downstream 복제는 이 티켓의 묵시적 완료 조건이 아니다. 범위 확대 시 별도 계약/티켓을 만든다.
