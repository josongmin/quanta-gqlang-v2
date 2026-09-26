# QGLang V3 ↔ Semantica 통합 버그배시 (2026-09-25)

상태: **INT-00 오판 철회 / INT-01·03 코드 반영·제품 증명 미완료**. QGLang producer의 이전 clean qualification과 Semantica admission owner PASS는 있으나, 설치형 제품 경로의 현 소스 PASS는 없다.

## 범위와 증거 기준

- QGLang: `quanta-gqlang-v2`의 공개 `quanta-gqlang-v2-frontend-contract`만 downstream 계약이다. 문법·sema·정규화 IR은 QGLang 소유이고, 데이터 세대·provider·budget·실행·SDK/daemon은 Semantica 소유다.
- 2026-09-26 문서 갱신 직전 QGLang은 `a01eecc5ff2d255aabc9eb09011cecd3895218a7` / tree `c2745f2c67d8060dace3cb4e0cc59aea1bbf5dd8` / clean이었다. `145b6682…` qualification 뒤 제품 코드는 바뀌지 않았지만 `.typos.toml`과 통합 계획 문서는 바뀌었으므로 그 receipt는 역사적·코드동일성 증거이지 `a01eecc5…` exact-HEAD qualification이 아니다. Semantica 구현·제품 증명은 격리 branch `codex/gqlang-integration-sep25`, base `038405343fe50ebc1216dbc52de9148a4cb52710`, dirty owner overlay에서 진행 중이다. clean committed-HEAD qualification과 동일 pair 최종 제품 receipt는 별도다.
- 표기: `Q:`는 QGLang 저장소 루트, `S:`는 sibling `semantica-codegraph-v2` 루트에 대한 경로다. 행 번호는 이 감사 시점의 근거이며 소스가 바뀌면 다시 확인한다.
- 판정 수준: **소스 불일치**는 파일 간 계약 충돌, **결함 후보**는 실행 경로상 위험이 보이나 재현 전, **증명 공백**은 테스트/영수증 부족이다. 과거 스냅샷의 RED/GREEN을 현재 결함이나 완료 증거로 승격하지 않는다.

## 확인된 통합 경로와 지원 경계

`Q:crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:13-18` → `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:17-20,37` → ingress bridge → `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/lib.rs:93-149` → `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:854-1115` → Composition producer/execution → SDK/daemon 응답·receipt. `dist-full`이 SDK GQLang substrate를 여는 경로는 `S:crates/quanta-cli-server/Cargo.toml:41-75`, `S:packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/Cargo.toml:137-152`에 있다.

현재 공개 V3 실행은 제한된 Composition이다. QGLang 계약 `Q:docs/contracts/frontend-execution-v3.md:18-22`는 Graph/CHECK/Presentation 단독 실행 payload 부재와 표현 가능한 Composition 변형의 runtime 지원 비보장을 명시한다. Semantica ingress는 Composition 외 root를 거부한다 (`S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/lib.rs:122-126`). 따라서 **Graph/CHECK 일반 실행 미지원, Hybrid 거부, Python 전용 lexical provider는 이 버그배시에서 독립 결함으로 세지 않는다.** 지원 확대는 별도 제품 요구·DTO 계약·provider proof가 선행돼야 한다.

Authority/TOCTOU 재감사에서는 새로운 오결과 경로를 찾지 못했다. 정확한 item/qualification 선택, 발행 source closure와 committed generation 대조, grant의 전체 request identity 바인딩, producer의 path/provider/generation/rowset 검사 경로가 존재한다. 이는 **소스 검사 결과**이지 프로세스 통과 증거가 아니다. 주요 소스: `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/src/frontend_execution_selection_v3.rs:60-93`, `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:119-149,352-415`, `S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/gqlang_composition_producer_v3.rs:272-329,362-428`.

## 남길 항목

| ID | 분류 | 판정 | 바로 필요한 일 |
| --- | --- | --- | --- |
| [INT-00](tickets/INT-00-v3-producer-consumer-drift.md) | CLOSED — 오판 철회 | 서로 다른 authority fixture의 DTO digest를 직접 비교했다. 현 Semantica 등록 QBC admission owner 6/6 PASS. | pin 변경 없음; pair 제품 증명은 INT-04 |
| [INT-01](tickets/INT-01-execution-deadline.md) | P1 수정 반영·자격 미완료 | 공유 Semantica main에서 기존 deadline/lifecycle guard를 bounded sync dispatch에 연결했다. manual QBC producer 17/17·SDK lifecycle 4/4·runtime dispatch 1/1 PASS이나 source digest가 서로 다르고 owner receipt가 아니다. | SDK lexical 지연 host 양·음성, 동일 digest owner rail, 설치형 timeout 증명 |
| [INT-02](tickets/INT-02-legacy-host-adapter.md) | P2 별도 workspace 결함 | 구형 host-adapter가 공개 facade에 없는 V1 HIR를 import한다. 등록 QBC package check도 E0432로 실패했고 Rust 역의존자는 0개다. | catalog/registry의 stable·wired 주장과 유지·제거 결정 |
| [INT-03](tickets/INT-03-installed-process-contract.md) | 코드 수정·제품 증명 대기 | 설치형 테스트를 standalone/named-scope typed reject와 semantic Composition 양성으로 변경했다. QBC 타깃 `check`는 현 소스에서 통과했다. | 동일 소스 설치 bundle·SDK wheel 영수증으로 exact/전체 process 실행 |
| [INT-04](tickets/INT-04-paired-qualification.md) | downstream 증명 공백 | QGLang 이전 clean qualification과 Semantica admission owner 6/6 PASS. 최신 manual lane 17/4/1은 서로 다른 digest이며 설치형 process의 동일 pair PASS는 없다. | 동일 digest owner→SDK→설치형 제품 영수증 |

## 현재 증거와 미증명

- QGLang의 `target/qualification/145b66827a8791c22c4df26fcfa11d82592f913f/receipt.json`은 해당 historical clean committed source에서 `status=complete`, `exit=0`, Nextest `1248/1248`, skipped/ignored/leaked `0/0/0`이다. 이는 당시 producer와 public external consumer를 증명하지만 현재 `a01eecc5…` exact HEAD, Semantica 소비자, 제품 실행을 증명하지 않는다.
- Semantica는 QGLang을 sibling path dependency로 소비한다. QBC source closure는 그 sibling 의존성도 digest에 포함한다 (`S:tools/quanta-build-cli/campaign_source_closure_v1.py:171-228`). 즉 pair 봉인 설계가 없는 것이 아니라 **현 소스에서 끝까지 통과한 영수증이 없는 것**이다.
- 이전 raw Cargo exact RED는 현 pair 결함의 증거가 아니다. QGLang과 Semantica의 golden fixture가 서로 다른 authority를 투영한다. 2026-09-26 현재 Semantica 등록 QBC admission owner는 현 QGLang sibling을 사용해 6개를 실제 실행·통과했다. 이 dirty-tree owner PASS는 설치형/clean-HEAD proof가 아니다.
- host-adapter 등록 QBC package `check`도 exit 101(`E0432`)이었다. Cargo metadata의 Rust 역의존자는 0개지만 catalog는 여전히 stable/wired/authoritative로 표기한다. catalog·casectl·timing·feature-isolation·boundary policy 참조가 있으므로 제품 계약 선택 없이 무작정 삭제하지 않는다.
- 과거 QBC log의 exec-contract/public-ingress owner PASS는 옛 digest다. SDK GQLang owner 실행은 중단된 기록이 있고, 확인된 `dist-full,gqlang-process-proof` 설치형 실패 영수증은 2026-09-23의 이전 source digest다. 이것들은 current pair PASS가 아니다. 원자료: `/Users/songmin/Library/semantica-codegraph-v2/logs/quanta-build-cli/verification-results.jsonl`.
- 2026-09-26 공유 main 통합 뒤 manual QBC producer 17/17, SDK lifecycle 4/4, runtime dispatch 1/1은 모두 exit 0이다. 각 receipt의 `authority_kind=manual`, `owner_execution_receipt=false`, `reuse_eligible=false`이고 source digest가 다르다. 이는 좁은 실행 진단이지 owner qualification/제품 증명이 아니다. 상세 run ID는 INT-01/04에 있다.
- 설치형 대상은 `S:tools/quanta-build-cli/product-recipes-v1.toml`의 filter 없는 `server_socket_daemon_e2e`이다. 현 QBC 타깃 `check`는 exit 0이지만 테스트 실행 수는 0이다. process promotion proof는 동일 build receipt에 묶인 네 아티팩트 `quanta`, `quanta-cli-server`, `quanta-filesystem-scan-worker`, `quanta-sql-rulepack-worker` 각각의 exact 설치 경로·SHA-256과 실제 worker launch를 요구한다. GitHub Actions는 요구하지 않는다. 로컬 영수증으로 닫는다.
- QGLang `composition_limit_2`와 Semantica pinned golden은 source digest만 같고 contract/qualification digest는 다르다. 계약상 이 두 digest는 DTO 투영을 포함하며 fixture authority가 다르다. 문자열 불일치만으로 pin drift라고 판정하지 않는다.

## 완료 판정 순서

1. QGLang의 `145b6682…` receipt는 historical baseline으로만 유지한다. 현재 HEAD `a01eecc5…`를 완료 근거로 쓰려면 자체 exact-HEAD receipt를 새로 받는다. Semantica 작업은 격리 checkout에서 HEAD, dirty/untracked, sibling path, QBC source digest를 다시 고정한다.
2. INT-00의 digest pin은 변경하지 않는다. 현 등록 QBC admission owner PASS를 유지하고 scoped SAMPLE·Graph producer 경계는 별도 runtime owner rail에서 확인한다.
3. INT-03의 standalone 성공 oracle은 현재 V3 typed reject로 수정됐다. Pattern·dense publication fixture를 보존한 동일 소스 설치 bundle·SDK wheel을 준비하고, 네 제품 아티팩트의 path/SHA/build receipt를 고정해 exact 및 filter 없는 process 실행을 받는다.
4. INT-01의 runtime bounded dispatch 수정은 반영됐다. SDK lexical 지연 host fixture의 실제 양·음성과 receipt 부재, 정상 대조군을 증명하고 동일 digest owner rail로 재실행한다. 동기 host 즉시 강제 중단은 별도 제품 요구로 판정한다.
5. INT-04의 owner-local → SDK → 설치형 exact → filter 없는 제품 게이트를 **고정된 동일 pair**에서 실행한다. Semantica QBC는 sibling QGLang을 포함한 source digest에 묶고, QGLang 자체 receipt는 HEAD/소스에 따로 대조한다. target/features/selected·executed count/exit/source digest를 남긴다. GitHub Actions는 요구하지 않는다.
6. INT-02는 제품 경로와 분리해 실제 소비자·등록 gate를 확인하고 QBC package compile을 진단한다. 이 P2를 INT-04의 무조건 선행조건으로 두지 않는다.

문서 작성만으로 어느 항목도 CLOSED가 되지 않는다.
