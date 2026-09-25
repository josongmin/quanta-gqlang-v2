# QGLang V3 ↔ Semantica 통합 버그배시 (2026-09-25)

상태: **INT-00 오판 철회 / INT-03 테스트 수정 반영·제품 증명 미완료**. QGLang producer의 이전 clean qualification과 Semantica admission owner PASS는 있으나, 설치형 제품 경로의 현 소스 PASS는 없다.

## 범위와 증거 기준

- QGLang: `quanta-gqlang-v2`의 공개 `quanta-gqlang-v2-frontend-contract`만 downstream 계약이다. 문법·sema·정규화 IR은 QGLang 소유이고, 데이터 세대·provider·budget·실행·SDK/daemon은 Semantica 소유다.
- 2026-09-25 현재 재검증 스냅샷: QGLang `145b66827a8791c22c4df26fcfa11d82592f913f` / tree `88815ae0779e8888cb0ce17661bfd5825a1979d7` / clean, Semantica `9ce5972352ad3503653e7952a9bab17340439898` / 다른 작업의 tracked 변경이 있는 dirty tree. QGLang V3 DTO·digest 변경은 커밋됐고 current clean HEAD qualification을 받았다. Semantica 관련 판정은 현재 dirty checkout의 명시 파일과 아래 exact owner 명령에 한정하며, 통합 완료 전 Semantica 변경을 분리한 고정 source digest가 필요하다.
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
| [INT-01](tickets/INT-01-execution-deadline.md) | P1 후보 | V3 SAMPLE의 admitted timeout이 실행/동기 대기까지 전달되지 않는 소스 경로. 지연 host 재현 전. | 재현 → ingress 절대 deadline·취소·late-success 거부를 한 authority로 연결 |
| [INT-02](tickets/INT-02-legacy-host-adapter.md) | P2 별도 workspace 결함 | 구형 host-adapter가 공개 facade에 없는 V1 HIR를 import한다. raw Cargo check는 실패했고 QBC 판정은 없다. | 제품/등록 소비자 확인 → QBC package 진단 → 유지·제거 결정 |
| [INT-03](tickets/INT-03-installed-process-contract.md) | 코드 수정·증명 대기 | 설치형 테스트를 standalone/named-scope typed reject와 semantic Composition 양성으로 변경했다. 현 소스 타깃 컴파일은 다른 동시 작업의 `SourceAwareCloseout` 비포괄 match(E0004)에서 중단됐다. | 그 변경이 정리된 뒤 같은 QBC compile rail과 receipt-bound process exact/전체 실행 |
| [INT-04](tickets/INT-04-paired-qualification.md) | downstream 증명 공백 | QGLang 이전 clean qualification과 Semantica admission owner 6/6 PASS. SDK→설치형 process의 동일 pair PASS는 없다. | INT-01 판정 후 owner→SDK→설치형 제품 영수증 |

## 현재 증거와 미증명

- QGLang의 `target/qualification/145b66827a8791c22c4df26fcfa11d82592f913f/receipt.json`은 current clean committed source에서 `status=complete`, `exit=0`, Nextest `1248/1248`, skipped/ignored/leaked `0/0/0`이다. 이는 producer와 public external consumer를 증명하지만 Semantica 소비자 또는 제품 실행을 증명하지 않는다.
- Semantica는 QGLang을 sibling path dependency로 소비한다. QBC source closure는 그 sibling 의존성도 digest에 포함한다 (`S:tools/quanta-build-cli/campaign_source_closure_v1.py:171-228`). 즉 pair 봉인 설계가 없는 것이 아니라 **현 소스에서 끝까지 통과한 영수증이 없는 것**이다.
- 이전 raw Cargo exact RED는 현 pair 결함의 증거가 아니다. QGLang과 Semantica의 golden fixture가 서로 다른 authority를 투영한다. 2026-09-26 현재 Semantica 등록 QBC admission owner는 현 QGLang sibling을 사용해 6개를 실제 실행·통과했다. 이 dirty-tree owner PASS는 설치형/clean-HEAD proof가 아니다.
- raw Cargo host-adapter check도 exit 101(`E0432` 5건, 후속 `E0282` 2건)이었다. 이는 소스 불일치와 일치하는 진단이지 QBC qualification은 아니다. catalog·casectl·timing·feature-isolation·boundary policy 참조가 있으므로 무작정 삭제하지 않는다.
- 과거 QBC log의 exec-contract/public-ingress owner PASS는 옛 digest다. SDK GQLang owner 실행은 중단된 기록이 있고, 확인된 `dist-full,gqlang-process-proof` 설치형 실패 영수증은 2026-09-23의 이전 source digest다. 이것들은 current pair PASS가 아니다. 원자료: `/Users/songmin/Library/semantica-codegraph-v2/logs/quanta-build-cli/verification-results.jsonl`.
- 설치형 대상은 `S:tools/quanta-build-cli/product-recipes-v1.toml:114-125`의 filter 없는 `server_socket_daemon_e2e`이다. `S:crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:253-266`에 따라 receipt-bound 설치 바이너리 없이는 process promotion proof가 불가능하다. GitHub Actions는 요구하지 않는다. 로컬 영수증으로 닫는다.
- QGLang `composition_limit_2`와 Semantica pinned golden은 source digest만 같고 contract/qualification digest는 다르다. 계약상 이 두 digest는 DTO 투영을 포함하며 fixture authority가 다르다. 문자열 불일치만으로 pin drift라고 판정하지 않는다.

## 완료 판정 순서

1. QGLang producer 기준은 clean-qualified `145b6682…`로 고정됐다. Semantica 작업은 현재 다른 변경과 분리한 checkout에서 HEAD, dirty/untracked, sibling path, QBC source digest를 다시 고정한다.
2. INT-00의 digest pin은 변경하지 않는다. 현 등록 QBC admission owner PASS를 유지하고 scoped SAMPLE·Graph producer 경계는 별도 runtime owner rail에서 확인한다.
3. INT-03의 standalone 성공 oracle을 현재 V3 계약에 맞춘다. Pattern·dense publication fixture는 보존한다. 소스상 모순을 확인하려고 설치형 RED를 먼저 돌리지 않는다.
4. INT-01은 제어형 지연 host로 먼저 재현한다. RED이면 수정·GREEN을 받고, 재현되지 않으면 enforcement 근거로 후보를 닫는다.
5. INT-04의 owner-local → SDK → 설치형 exact → filter 없는 제품 게이트를 **고정된 동일 pair**에서 실행한다. Semantica QBC는 sibling QGLang을 포함한 source digest에 묶고, QGLang 자체 receipt는 HEAD/소스에 따로 대조한다. target/features/selected·executed count/exit/source digest를 남긴다. GitHub Actions는 요구하지 않는다.
6. INT-02는 제품 경로와 분리해 실제 소비자·등록 gate를 확인하고 QBC package compile을 진단한다. 이 P2를 INT-04의 무조건 선행조건으로 두지 않는다.

문서 작성만으로 어느 항목도 CLOSED가 되지 않는다.
