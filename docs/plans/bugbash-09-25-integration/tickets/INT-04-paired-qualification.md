# INT-04 — 동일 pair source의 통합 qualification

상태: **QGLang producer qualification 완료 / Semantica 통합·제품 증명 공백** · 우선순위: release blocker · 소유: 두 저장소 integrator / Semantica product proof owner.

## 현재 증거

- QGLang `145b66827a8791c22c4df26fcfa11d82592f913f` / tree `88815ae0779e8888cb0ce17661bfd5825a1979d7`의 current clean committed-source qualification receipt가 생성됐다: `status=complete`, `exit=0`, Nextest `1248/1248`, skipped/ignored/leaked `0/0/0`, duration 28s. 경로는 `target/qualification/145b66827a8791c22c4df26fcfa11d82592f913f/receipt.json`이다.
- Semantica는 sibling `Q:crates/quanta-gqlang-v2-frontend-contract`를 path dependency로 소비한다: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:17-20,37`. QBC campaign source closure에는 sibling path dependency가 포함된다: `S:tools/quanta-build-cli/campaign_source_closure_v1.py:171-228`.
- INT-00의 raw Cargo RED는 다른 authority fixture의 digest를 동일하게 요구한 오판으로 철회했다. 2026-09-26 Semantica 등록 QBC admission owner는 현 QGLang sibling을 사용해 6/6 PASS다. INT-02 host-adapter raw check exit 101은 별도 P2 진단이며 제품 V3 pair PASS는 여전히 확인되지 않았다.
- `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:3819-3910`은 V3 from-files의 실제 소켓/authority drift를 검사한다. lexical·semantic V3 Composition은 SDK owner 테스트가 있지만 설치형 process 양성은 확인되지 않았다. SDK 등록: `S:tools/testing/suite-registry-v1.yaml:646-775`.
- 2026-09-26 설치형 테스트 계약 수정은 공유 Semantica 커밋 `130a7f1f5e9`에 포함됐다. 그 뒤 `dist-full,gqlang-process-proof` 타깃 QBC compile은 현 소스의 다른 동시 변경 `workspace_journal_operator_runtime_v1.rs:182`에서 E0004(`SourceAwareCloseout` 미처리)로 중단됐다. GQLang process 실행이나 테스트 컴파일 PASS로 세지 않는다.

## 실행 순서와 증거

1. QGLang `145b6682…` clean receipt는 그 소스의 producer 증거로만 재사용한다. 현재 QGLang HEAD `7c1289ee…`은 이후 통합 문서만 변경됐다. Semantica는 INT-03 수정과 INT-01 판정 뒤 다른 dirty 작업을 분리하고, 양쪽 HEAD·tree·dirty/untracked를 고정한다. QBC source digest에 sibling QGLang이 포함됐는지 기록한다.
2. 등록 QBC owner/profile rail에서 exec-contract → public-ingress → runtime-execution → SDK GQLang을 동일 Semantica source digest로 실행한다. SDK에는 `S:tools/testing/suite-registry-v1.yaml:646-775`의 files/lexical/semantic positive와 foreign-scope/generation negatives를 선택한다. 변경 없는 owner PASS는 같은 digest·feature·selector에서만 재사용한다.
3. `dist-full,gqlang-process-proof` 설치 bundle/receipt의 binary SHA를 확인한다. INT-03 수정 뒤 exact process test를 먼저 실행하고, 성공하면 filter 없는 `S:tools/quanta-build-cli/product-recipes-v1.toml:114-125` recipe를 한 번 실행한다. 이전 소스 모순을 보기 위한 pre-fix process RED는 요구하지 않는다.
4. pair HEAD, QBC source digest, toolchain, features, target, selected/executed/ignored count, exit, 설치 binary/receipt identity를 남긴다. 실패·중단·skip·미선택은 PASS가 아니다. 소스가 움직이면 영향받은 단계만 재판정한다. INT-02는 이 경로가 package/workspace gate를 실제 선택하지 않는 한 별도 P2다.

## 완료 조건

- 두 저장소 producer/consumer + SDK + 설치형 daemon/process가 동일한 고정 소스에서 통과하고, V3 files/lexical/semantic Composition 및 권한 drift가 실제 선택돼 실행됐음이 확인된다.
- QBC의 sibling source closure와 설치 artifact custody가 서로 일치한다. 로컬 증거로 충분하며 GitHub Actions 활성화는 요구하지 않는다.
- Graph/CHECK/Presentation의 일반 실행, Rust/Java lexical SAMPLE, 15개 모든 producer 벡터의 downstream 복제는 이 티켓의 묵시적 완료 조건이 아니다. 범위 확대 시 별도 계약/티켓을 만든다.
