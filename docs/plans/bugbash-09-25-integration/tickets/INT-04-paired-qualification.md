# INT-04 — 동일 pair source의 통합 qualification

상태: **QGLang producer qualification 완료 / Semantica 통합·제품 증명 공백** · 우선순위: release blocker · 소유: 두 저장소 integrator / Semantica product proof owner.

## 현재 증거

- QGLang `145b66827a8791c22c4df26fcfa11d82592f913f` / tree `88815ae0779e8888cb0ce17661bfd5825a1979d7`의 historical clean committed-source qualification receipt가 있다: `status=complete`, `exit=0`, Nextest `1248/1248`, skipped/ignored/leaked `0/0/0`, duration 28s. 경로는 `target/qualification/145b66827a8791c22c4df26fcfa11d82592f913f/receipt.json`이다. 2026-09-26 문서 갱신 직전 current clean HEAD는 `a01eecc5ff2d255aabc9eb09011cecd3895218a7` / tree `c2745f2c67d8060dace3cb4e0cc59aea1bbf5dd8`이며, 제품 코드는 동일하지만 exact-HEAD receipt는 없다.
- Semantica는 sibling `Q:crates/quanta-gqlang-v2-frontend-contract`를 path dependency로 소비한다: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:17-20,37`. QBC campaign source closure에는 sibling path dependency가 포함된다: `S:tools/quanta-build-cli/campaign_source_closure_v1.py:171-228`.
- INT-00의 raw Cargo RED는 다른 authority fixture의 digest를 동일하게 요구한 오판으로 철회했다. 2026-09-26 Semantica 등록 QBC admission owner는 현 QGLang sibling을 사용해 6/6 PASS다. INT-02 host-adapter raw check exit 101은 별도 P2 진단이며 제품 V3 pair PASS는 여전히 확인되지 않았다.
- `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:3819-3910`은 V3 from-files의 실제 소켓/authority drift를 검사한다. lexical·semantic V3 Composition은 SDK owner 테스트가 있지만 설치형 process 양성은 확인되지 않았다. SDK 등록: `S:tools/testing/suite-registry-v1.yaml:646-775`.
- 2026-09-26 설치형 테스트 계약 수정은 공유 Semantica 커밋 `130a7f1f5e9`에 포함됐다. 그 뒤 `dist-full,gqlang-process-proof` 타깃 QBC compile은 현 소스의 다른 동시 변경 `workspace_journal_operator_runtime_v1.rs:182`에서 E0004(`SourceAwareCloseout` 미처리)로 중단됐다. GQLang process 실행이나 테스트 컴파일 PASS로 세지 않는다.
- 같은 타깃 QBC `check` 재실행은 source digest `03b688c325ee8d70be9afcd6375aa00fe20e495eaa2b4b42847b9f9ed852e809`에서 exit 0이다. 이는 컴파일 증거이며 설치형 process 실행이나 pair qualification PASS가 아니다.
- 격리 Semantica SDK exact semantic SAMPLE 재현 시도는 source digest `22ccfbb146eb1ef0401e11fa70c00925b8c41d219f7193a5b346937f06b1a83f`에서 테스트 1개 선택·실행 후 현재 소스 `QUANTA_INDEX_SEARCHD_BIN` 미설정으로 실패했다. 타임아웃 동작 판정이나 SDK PASS가 아니며, 공유 main에 통합되지 않은 격리 checkout 결과다.
- 공유 main의 lifecycle + bounded dispatch 통합 뒤 QBC **manual lane** 실행은 producer 17/17(`20260925T173906.192056Z-d9ec1b579081`, source `4cada145…`), 최종 runtime dispatch 1/1(`20260925T181455.961336Z-693b5e3485ce`, source `b0f024c0…`), 최신 SDK lifecycle 4/4(`20260925T181736.019920Z-77e5ae285e82`, source `ec097def…`) 모두 exit 0이다. receipt 위치: `/Users/songmin/Library/Caches/semantica-codegraph-v2-target/quanta-build-cli/_state/execution-roots/semantica-codegraph-v2-d0b8b5a985ea/lanes/gqlang-ss-integration/verification-results/<run-id>/receipt.json`. 각 receipt는 `authority_kind=manual`, `owner_execution_receipt=false`, `reuse_eligible=false`이며 digest도 다르므로 동일 pair qualification이 아니다.
- QGLang `2ba085bc…` dirty 문서 스냅샷의 `just ci` exit 0, Nextest 1248/1248, external consumer positive 1/1·negative 15/15는 historical local gate다. 이후 HEAD와 계획 문서가 바뀌었으므로 current exact-HEAD qualification으로 재사용하지 않는다.

## 실행 순서와 증거

1. QGLang `145b6682…` clean receipt는 그 소스의 producer 증거로만 재사용한다. 현재 기준 HEAD는 `a01eecc5…`이며 이 문서 수정 뒤 worktree는 dirty다. Semantica는 격리 owner overlay를 사용하고, 양쪽 HEAD·tree·dirty/untracked를 다시 고정한다. QBC source digest에 sibling QGLang이 포함됐는지 기록한다.
2. 등록 QBC owner/profile rail에서 exec-contract → public-ingress → runtime-execution → SDK GQLang을 동일 Semantica source digest로 실행한다. SDK에는 `S:tools/testing/suite-registry-v1.yaml:646-775`의 files/lexical/semantic positive와 foreign-scope/generation negatives를 선택한다. 변경 없는 owner PASS는 같은 digest·feature·selector에서만 재사용한다.
3. `dist-full,gqlang-process-proof` 설치 bundle/receipt에서 `quanta`, `quanta-cli-server`, `quanta-filesystem-scan-worker`, `quanta-sql-rulepack-worker` 네 아티팩트의 exact path/SHA를 확인한다. INT-03 수정 뒤 exact process test를 먼저 실행하고, 성공하면 filter 없는 product recipe를 한 번 실행한다. 이전 소스 모순을 보기 위한 pre-fix process RED는 요구하지 않는다.
4. pair HEAD, QBC source digest, toolchain, features, target, selected/executed/ignored count, exit, 설치 binary/receipt identity를 남긴다. 실패·중단·skip·미선택은 PASS가 아니다. 소스가 움직이면 영향받은 단계만 재판정한다. INT-02는 이 경로가 package/workspace gate를 실제 선택하지 않는 한 별도 P2다.

## 완료 조건

- 두 저장소 producer/consumer + SDK + 설치형 daemon/process가 동일한 고정 소스에서 통과하고, V3 files/lexical/semantic Composition 및 권한 drift가 실제 선택돼 실행됐음이 확인된다.
- QBC의 sibling source closure와 네 설치 artifact의 path/SHA/build-receipt custody가 서로 일치하고, 별도 filesystem worker가 실제 launch된다. 로컬 증거로 충분하며 GitHub Actions 활성화는 요구하지 않는다.
- Graph/CHECK/Presentation의 일반 실행, Rust/Java lexical SAMPLE, 15개 모든 producer 벡터의 downstream 복제는 이 티켓의 묵시적 완료 조건이 아니다. 범위 확대 시 별도 계약/티켓을 만든다.
