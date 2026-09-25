# QGLang V3 ↔ Semantica 통합 버그배시 (2026-09-25)

상태: **현재 pair 재검증 완료 / downstream 결함·제품 증명 미완료**. QGLang producer는 clean qualification을 받았지만 Semantica consumer와 설치형 제품 경로는 아직 닫히지 않았다.

## 범위와 증거 기준

- QGLang: `quanta-gqlang-v2`의 공개 `quanta-gqlang-v2-frontend-contract`만 downstream 계약이다. 문법·sema·정규화 IR은 QGLang 소유이고, 데이터 세대·provider·budget·실행·SDK/daemon은 Semantica 소유다.
- 2026-09-25 현재 재검증 스냅샷: QGLang `39f729171243c07d1f6dc5cdc2d6129731b7c6ad` / tree `cddb605117b7719d459877fd2b4046458a3800e2` / clean, Semantica `01c699be4d26f3d80e378c8906dfd2ee9b39af41` / 다른 작업의 tracked 변경이 있는 dirty tree. QGLang V3 DTO·digest 변경은 이제 커밋됐고 clean qualification을 받았다. Semantica 관련 판정은 현재 dirty checkout의 명시 파일과 아래 exact owner 명령에 한정하며, 통합 완료 전 Semantica 변경을 분리한 고정 source digest가 필요하다.
- 표기: `Q:`는 QGLang 저장소 루트, `S:`는 sibling `semantica-codegraph-v2` 루트에 대한 경로다. 행 번호는 이 감사 시점의 근거이며 소스가 바뀌면 다시 확인한다.
- 판정 수준: **소스 불일치**는 파일 간 계약 충돌, **결함 후보**는 실행 경로상 위험이 보이나 재현 전, **증명 공백**은 테스트/영수증 부족이다. 과거 스냅샷의 RED/GREEN을 현재 결함이나 완료 증거로 승격하지 않는다.

## 확인된 통합 경로와 지원 경계

`Q:crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:13-18` → `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:17-20,37` → ingress bridge → `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/lib.rs:93-149` → `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:854-1115` → Composition producer/execution → SDK/daemon 응답·receipt. `dist-full`이 SDK GQLang substrate를 여는 경로는 `S:crates/quanta-cli-server/Cargo.toml:41-75`, `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/Cargo.toml:137-152`에 있다.

현재 공개 V3 실행은 제한된 Composition이다. QGLang 계약 `Q:docs/contracts/frontend-execution-v3.md:18-22`는 Graph/CHECK/Presentation 단독 실행 payload 부재와 표현 가능한 Composition 변형의 runtime 지원 비보장을 명시한다. Semantica ingress는 Composition 외 root를 거부한다 (`S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/lib.rs:122-126`). 따라서 **Graph/CHECK 일반 실행 미지원, Hybrid 거부, Python 전용 lexical provider는 이 버그배시에서 독립 결함으로 세지 않는다.** 지원 확대는 별도 제품 요구·DTO 계약·provider proof가 선행돼야 한다.

Authority/TOCTOU 재감사에서는 새로운 오결과 경로를 찾지 못했다. 정확한 item/qualification 선택, 발행 source closure와 committed generation 대조, grant의 전체 request identity 바인딩, producer의 path/provider/generation/rowset 검사 경로가 존재한다. 이는 **소스 검사 결과**이지 프로세스 통과 증거가 아니다. 주요 소스: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/src/frontend_execution_selection_v3.rs:60-93`, `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:119-149,352-415`, `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/gqlang_composition_producer_v3.rs:272-329,362-428`.

## 남길 항목

| ID | 분류 | 판정 | 바로 필요한 일 |
| --- | --- | --- | --- |
| [INT-00](tickets/INT-00-v3-producer-consumer-drift.md) | P1 재현된 consumer RED | Semantica의 scoped SAMPLE 선행 거부는 이미 구현됐지만, pinned golden digest가 QGLang schema-5 producer 벡터보다 오래됐다. exact owner test가 `0/1`로 실패한다. | Semantica digest pin 갱신 → scope/graph 거부 회귀 포함 owner rail → 동일 pair proof |
| [INT-01](tickets/INT-01-execution-deadline.md) | P1 후보 | V3 SAMPLE의 admitted timeout이 실행/동기 대기까지 전달되지 않는 소스 경로. 지연 host 재현 전. | 재현 → ingress 절대 deadline·취소·late-success 거부를 한 authority로 연결 |
| [INT-02](tickets/INT-02-legacy-host-adapter.md) | P2 확인된 compile defect | 별도 workspace member인 구형 host-adapter가 공개 facade에 없는 V1 HIR를 import하며 exact owner check가 7개 오류로 실패한다. | 소비자/존치 결정 → 제거 또는 V3 계약 재설계 → workspace/QBC 정합화 |
| [INT-03](tickets/INT-03-installed-process-contract.md) | P1 테스트 계약 충돌 후보 | 설치형 full-provider 테스트는 단독 CHECK·semantic SAMPLE 성공을 기대하지만 현 V3 SDK는 단독 family를 거부. 현재 설치형 실행 전. | 테스트가 통과해야 할 제품 계약 결정; V3 Composition·typed reject·provider proof를 각각 올바른 경로에 배치 |
| [INT-04](tickets/INT-04-paired-qualification.md) | downstream 증명 공백 | QGLang `39f72917…` clean qualification은 PASS다. 그러나 같은 pair의 Semantica ingress/SDK→설치형 process E2E PASS는 없다. | INT-00~03 수정 후 Semantica owner→SDK→설치형 process→전체 제품 게이트 영수증 수집 |

## 현재 증거와 미증명

- QGLang의 `target/qualification/39f729171243c07d1f6dc5cdc2d6129731b7c6ad/receipt.json`은 current clean committed source에서 `status=complete`, `exit=0`, Nextest `1248/1248`, skipped/ignored/leaked `0/0/0`이다. 이는 producer와 public external consumer를 증명하지만 Semantica 소비자 또는 제품 실행을 증명하지 않는다.
- Semantica는 QGLang을 sibling path dependency로 소비한다. QBC source closure는 그 sibling 의존성도 digest에 포함한다 (`S:tools/quanta-build-cli/campaign_source_closure_v1.py:171-228`). 즉 pair 봉인 설계가 없는 것이 아니라 **현 소스에서 끝까지 통과한 영수증이 없는 것**이다.
- 현재 Semantica checkout에서 `cargo test -p quanta-runtime-public-ingress-surface --features gqlang-v3-admission consumer_matches_pinned_frontend_v3_composition_golden_vector --lib -- --nocapture`를 실행했고 1개가 선택되어 `0 passed / 1 failed`였다. 실제 contract digest `1cfc163f…`와 pinned `41d4b8d3…`가 불일치한다. 이는 INT-00의 current-source RED다.
- 같은 checkout에서 `cargo check -p quanta-gqlang-v2-host-adapter --lib --no-default-features`는 exit 101, `E0432` 5건과 후속 `E0282` 2건으로 실패했다. 이는 INT-02의 current-source compile RED다.
- 과거 QBC log의 exec-contract/public-ingress owner PASS는 옛 digest다. SDK GQLang owner 실행은 중단된 기록이 있고, 확인된 `dist-full,gqlang-process-proof` 설치형 실패 영수증은 2026-09-23의 이전 source digest다. 이것들은 current pair PASS가 아니다. 원자료: `/Users/songmin/Library/semantica-codegraph-v2/logs/quanta-build-cli/verification-results.jsonl`.
- 설치형 대상은 `S:tools/quanta-build-cli/product-recipes-v1.toml:114-125`의 filter 없는 `server_socket_daemon_e2e`이다. `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:253-266`에 따라 receipt-bound 설치 바이너리 없이는 process promotion proof가 불가능하다. GitHub Actions는 요구하지 않는다. 로컬 영수증으로 닫는다.
- 현재 QGLang `Q:crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv:9`의 `composition_limit_2`는 source `d47176ff…`, contract `1cfc163f…`, qualification `444de54f…`다. Semantica `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs`는 source만 같고 contract `41d4b8d3…`, qualification `a61355ae…`를 기대한다. QGLang의 모든 producer 벡터를 Semantica에 복제하는 것은 별도 근거 없이 요구하지 않는다.

## 완료 판정 순서

1. QGLang producer 기준은 clean-qualified `39f72917…`로 고정됐다. Semantica 작업은 현재 다른 변경과 분리한 checkout에서 HEAD, dirty/untracked, sibling path, QBC source digest를 다시 고정한다.
2. INT-00의 Semantica golden digest를 schema-5 producer에 맞추고, 이미 존재하는 scoped SAMPLE typed reject와 graph producer reject를 owner rail에 함께 유지한다. exact RED를 GREEN으로 바꾸기 전 source 비교만으로 완료 선언하지 않는다.
3. INT-01은 지연 host로 RED를 재현하고 수정 뒤 같은 조건 GREEN을 받는다. INT-02는 존치 결정과 workspace/member compile 결과로 닫는다.
4. INT-03의 현행 제품 기대를 정합화한다. 단독 미지원 family를 성공으로 포장하지 않고, Pattern·dense provider proof를 무단 삭제하지 않는다.
5. INT-04의 owner-local → SDK → 설치형 바이너리+receipt → filter 없는 제품 게이트를 **고정된 동일 pair**에서 실행한다. Semantica QBC 단계들은 sibling QGLang까지 포함한 동일 source digest에 묶고, QGLang 자체 receipt는 그 producer HEAD/소스에 따로 대조한다. 각 단계의 target/features/selected test count/exit/source digest를 남긴다. 소스가 움직이면 영향받은 단계만 다시 판단한다.

문서 작성만으로 어느 항목도 CLOSED가 되지 않는다.
