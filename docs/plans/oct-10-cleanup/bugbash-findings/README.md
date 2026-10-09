# Oct-10 QGLang 병렬 감사 발견사항

**최종 수정 대기열:** [FINAL-TRIAGE.md](FINAL-TRIAGE.md). 활성 6건을 Modules 비용 상한 4건, Semgrep coverage 1건, Semantica locked 소비자 차단 1건의 세 결합 작업으로 묶었다. 아래 전체 표는 제외 항목까지 포함한 감사 기록이며 수정 지시 목록이 아니다.

## 입력과 판정 범위

- QGLang 기준: `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`; 비교 범위 `9060e46..25fad76`. [340개 파일 인벤토리](../../../audits/2026-10-10-weekly-change-inventory.md)와 [9개 레인 계획](../../../audits/2026-10-10-parallel-retrospective-plan.md)에 맞춰 소유권별로 읽었다.
- 이 폴더의 발견은 그 고정 소스와 각 파일의 실행 증거에만 묶인다. 병렬 감사 중 QGLang Rust/정책 소스는 수정하지 않았다. 공유 checkout의 다른 작성자가 진행 중인 ADR·plan 문서는 복원·수정·커밋하지 않았다.
- `CONFIRMED_DEFECT`는 도달 경로와 반례가 있는 현재 소스/정책 문제, `DOWNSTREAM_BLOCKER`는 별도 repo의 재현된 차단, `DOC_HEAD_DEFECT`는 고정 HEAD의 문서 오류다. `PROOF_GAP`은 동작 실패로 세지 않는다.

## 발견 기록 (활성 여부는 최종 triage 기준)

| ID | 분류 | 내용·영향 | 증거 및 후속 소유자 |
| --- | --- | --- | --- |
| [M04-01](M04-01-eager-visible-scan.md) | QGLang 소스, scalability | 로컬 View 대상이 이미 해석되어도 모든 visible binding을 먼저 선형 검색한다. 여러 edge와 큰 visible set에서 불필요한 `E×B` 비교. | Modules `catalog/view_topology.rs`; local-hit에서 visible scan 0회가 되도록 고치되 lookup 우선순위·진단 위치 보존. 실제 지연/RSS 미측정. |
| [M04-02](M04-02-sorted-vec-insert.md) | QGLang 소스, scalability | 역순 고유 dependency를 sorted `Vec`에 한 건씩 삽입하여 `E(E−1)/2` shift. edge charge는 입력 edge당 1회다. | Modules 같은 owner; authored 순서의 duplicate/first error를 유지하며 수집 후 정렬. 실제 지연/RSS 미측정. |
| [MF-01](MF-01-family-edge-insert.md) | QGLang 소스, scalability | 최종 catalog family-reference merge에서도 별도 sorted `Vec::insert`가 역순 `F`건에 `F(F−1)/2` shift를 만든다. | Modules `catalog.rs`; M04-02를 고쳐도 이 경로는 남는다. 입력 순서의 오류 우선순위와 출력 정렬을 보존해야 한다. |
| [MF-02](MF-02-certified-reachability-insert.md) | QGLang 소스, scalability | `REQUIRE CERTIFIED`의 external reachability가 정렬된 seen Vec에 역순으로 삽입해 `N(N−1)/2` shift를 만든다. | Modules `catalog.rs`; 방문 작업에 별도 step charge가 없다. 실제 지연/RSS 미측정. |
| [FF-01](FF-01-retained-chain-owner-validation.md) | QGLang 공개 API custody | head permit만 인정하는 sink에서는 이전 permit이 거절돼도 `validate_owner_v3`와 `try_extend_v3`가 통과한다. 원본 HIR 소스를 사용한 독립 Rust fixture로 재현했다. | HIR `request_storage.rs`; full-chain 검사는 거절한다. 실제 downstream host sink에서의 도달 가능성은 미확인. |
| [CG-01](CG-01-semgrep-execution-owner.md) | QGLang 정책 coverage, P2 | 새 `execution-contract/src`가 Semgrep 규칙과 expected-path 목록에서 모두 누락된다. 동일 위반을 Pipeline에서는 탐지하고 새 owner에서는 누락하는 격리 반례가 있다. | `.semgrep/qglang-frontend.yml`, `tools/lint/semgrep_gate.py`; rule 범위와 새 owner의 typed-error 계약을 함께 고정. 현재 새 crate의 해당 위반은 확인되지 않았다. |
| [CF-01](CF-01-upstream-operator-enum-alias.md) | QGLang 계약 owner 경계, P2 | 공개 V3 `ExecutionOperatorKindV3`가 Types 소유 V2 enum의 alias여서 execution-local closed vocabulary 선언과 충돌한다. Semantica 소비자가 이 타입을 사용한다. | Execution-contract projector와 facade; 현행 실행 결과 오류는 확인되지 않았다. 전용 enum 도입은 소비자 Rust 타입 호환 변경을 수반한다. |
| [DP-01](DP-01-consumer-lock-closure.md) | `DOWNSTREAM_BLOCKER` | Semantica 고정 checkout의 full `cargo metadata --locked --offline`가 exit 101. lock에 새 QGLang execution-contract package가 없지만 다른 lock 변화 요인까지 원인 분리되지는 않았다. | Semantica E13 source/lock/consumer pair. QGLang facade만으로 소비자 compile이나 제품 완료를 주장하지 않는다. |
| [DOC-01](DOC-01-qg-diag-routing.md), [DOC-02](DOC-02-ticket-denominator.md) | `DOC_HEAD_DEFECT` | 고정 HEAD의 QG-DIAG 미완료 표기·깨진 앵커, 활성 티켓 19/18 불일치. | 해당 ADR/README는 다른 작성자의 미커밋 수정에 이미 교정안이 있다. 그 작업의 통합 뒤 새 SHA에서 재확인한다. |

## 레인별 파일과 확인 범위

| 레인 | 결과 파일 | 이번 감사에서 실행한 범위 |
| --- | --- | --- |
| 01–02 Surface/HIR/Binder/Types/Sema | [01-syntax-sema.md](01-syntax-sema.md) | 집중 owner 테스트 23회 실행·23회 통과. 조사 범위에서 확인된 현재 소스 결함 없음. |
| 04–05 Modules/Logic SCC, DAG/View, lock | [02-modules-logic.md](02-modules-logic.md) | 이차 복잡도 2건. 관련 테스트 62회 실행·62회 통과(필터 간 중복 실행 포함). 구조적 연산 횟수 오라클, 지연/RSS 미측정. |
| 06–07 실행 계약/facade/정책/CI | [03-contract-gates.md](03-contract-gates.md) | Semgrep 누락 반례. 집중 Semgrep 5/5, architecture 11/11, size 7/7. 기존 clean-HEAD 영수증의 1,529/1,529 및 24-row digest·facade 증거를 SHA/hash와 대조. |
| 03 bounded scalar Pipeline | [04-scalar-pipeline.md](04-scalar-pipeline.md) | `scalar_request` 필터 10회 실행·10회 통과, 65개 미선택. 조사 범위에서 확인된 현재 소스 결함 없음. |
| 09 Semantica pair | [05-downstream-pair.md](05-downstream-pair.md) | Semantica `58cfa983cf9785ab1752f4d75011fefdfba1285f`의 manifest/lock 대조와 locked metadata 실패. 소비자 compile·V3 vector 미실행. |
| 08 문서·증거 | [06-docs-evidence.md](06-docs-evidence.md) | HEAD에서 살아 있는 관련 문서 링크·상태 대조. 현행 미커밋 문서는 별도 WIP로 취급. CI 미실행. |
| 최종 frontend 교차 감사 | [07-frontend-final.md](07-frontend-final.md) | FF-01 원본 HIR 파일을 직접 컴파일한 외부 fixture 재현. 기존 집중 테스트를 새 실행으로 합산하지 않음. |
| 최종 Modules/Logic 교차 감사 | [08-modules-final.md](08-modules-final.md) | M04-01/02 재확인, 별도 이차 삽입 경로 MF-01/02 발견. 새 Cargo/성능 측정 미실행. |
| 최종 계약/소비자 교차 감사 | [09-contract-downstream-final.md](09-contract-downstream-final.md) | CG-01과 Semantica locked metadata exit 101 재확인; CF-01 source/consumer 경계 추적. 새 소비자 compile 미실행. |
| 최종 문서/증거 재확인 | [10-docs-final.md](10-docs-final.md) | 현재 dirty overlay의 local Markdown 링크 163개, 활성 ID 18개, 삭제 원본 11개 복구 경로 확인. 새 결함 없음; 통합 SHA 미정. |

## 통합 판단과 남은 증거

1. **활성 수정:** [최종 triage](FINAL-TRIAGE.md)의 A/B/C만 작업한다. Modules의 네 경로는 결합해 의미·진단·비용 상한을 확인한다. Semgrep의 실제 scanned path를 고정한다. Semantica lock drift는 원인을 분리한 뒤 소비자 소유자에서 해결한다.
2. **제외·재검토:** FF-01은 현재 Semantica sink 도달성이 확인되지 않았고, CF-01은 계약/소비자 전환 결정이 필요하다. DOC-01/02는 다른 작성자의 미커밋 교정안 통합 뒤 새 SHA에서 재검증한다. 중복 패치를 만들지 않는다.
3. **결합 검증:** 수정 후 affected owner 테스트와 `just ci` 한 번을 실행하고, clean committed HEAD 영수증이 필요하면 `just qualify-local`을 실행한다. 현재 `25fad76`의 clean receipt는 이후 수정의 통과 증거가 아니다. Hosted CircleCI, downstream compile/product, 실제 CPU/peak RSS는 이 감사의 완료 결과가 아니다.

**주의:** 340경로를 소유권 레인과 최종 상태에 매핑한 감사다. 집중 테스트의 selected 수를 합산해 서로 독립적인 테스트 수나 저장소 전체 coverage로 해석하지 않는다.
