# Oct-10 bugbash 최종 수정 대상

## 고정 입력과 판정 기준

- QGLang `main` HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`. Rust, 정책, manifest, lock은 clean; 별도 작성자의 문서 통합은 미커밋 상태다. 그 문서는 이 감사에서 수정하지 않았다.
- Semantica 별도 checkout `65a8bd80c6f11a220a228b2ec936cd50eba9238d`: 관련 Cargo manifest/lock clean. 이 SHA에서 full `cargo metadata --locked --offline --format-version 1`은 exit 101 (`Cargo.lock needs to be updated`)로 재현됐다.
- 활성 목록에는 현재 소스의 도달 경로와 구체적 실패/무제한 작업, 또는 실제 실패한 정책·소비자 명령이 있는 항목만 둔다. 구성 가능한 반례만 있고 현행 host에서 도달하지 못한 경우와 미래 변경 위험만 있는 경우는 수정 대기열에서 제외한다. 실행하지 않은 레일은 성공으로 계산하지 않는다.

## 활성: 6건, 3개 결합 작업

| 묶음 / 소유자 | 확정된 현상 | 필수 수락 증거 |
| --- | --- | --- |
| **A. Modules 비용 상한** — [M04-01](M04-01-eager-visible-scan.md), [M04-02](M04-02-sorted-vec-insert.md), [MF-01](MF-01-family-edge-insert.md), [MF-02](MF-02-certified-reachability-insert.md) | 로컬 View 대상에도 eager visible-binding 전수 검색(`E×B`), 직접 topology와 family merge의 정렬 Vec 앞삽입(`E(E−1)/2`, `F(F−1)/2`), certified reachability의 visited Vec 앞삽입(`N(N−1)/2`). 모두 현재 caller→owner 경로에 있고 입력 edge/방문당 비용 청구가 내부 반복 작업을 제한하지 않는다. 이는 **정적 연산 수**이며 지연/RSS 실측은 아니다. | 각 경로의 작은 정상·거부 오라클과 역순/visible-large 입력에서 반복 작업이 선형 또는 `O(n log n)`인 증거. 기존 duplicate/kind 진단의 첫 위치, local→external→visible 우선순위, 정렬 결과, exact/one-short budget 보존. 대표 workload·소스 SHA를 적은 CPU/peak RSS 측정 후 affected Modules 테스트와 결합 `just ci`. |
| **B. QGLang 정책 coverage** — [CG-01](CG-01-semgrep-execution-owner.md) | 새 `execution-contract/src`가 Semgrep include와 게이트 expected-path 양쪽에서 빠져 있다. 동일 `Result<_, String>` 위반을 Pipeline에서는 잡고 새 crate에서는 스캔하지 않는 격리 fixture가 있다. 현재 새 crate의 해당 위반은 발견되지 않았다. | 규칙·expected-path를 함께 확장; 새 경로 위반과 한쪽만 누락한 변형을 각각 실패시키는 fixture; 실제 scanned-path 집합 및 `just semgrep` 확인. 결합 변경 후 `just ci`. |
| **C. Semantica 소비자 source/lock closure** — [DP-01](DP-01-consumer-lock-closure.md) | QGLang facade는 새 execution-contract crate에 의존하지만 Semantica lock에는 그 package가 없고 full locked metadata가 exit 101이다. **정확한 모든 lock drift 원인은 미분리**이므로 이 사실만으로 한 줄 lock 수정안을 확정하지 않는다. QGLang 소스 수정 소유가 아닌 별도 Semantica E13 통합 차단이다. | 동일 QGLang/Semantica SHA·subtree/vector/feature tuple에서 lock diff 원인 분리 → `cargo metadata --locked --offline` 성공 → consumer compile → V3 contract vector. Runtime/installed product는 별도 rail. |

## 활성 수정 대상에서 제외

| 기록 | 제외 이유 / 재진입 조건 |
| --- | --- |
| [FF-01](FF-01-retained-chain-owner-validation.md) | 원본 HIR 코드로 head-only 선택 sink의 API 불변식 반례는 재현했다. 현재 Semantica `IssuedFrontendStorageSinkV3::owns_permit_v3`는 live permit의 **동일 budget identity와 kind**를 검사한다. 이 host에서 선택 sink의 재현 경로는 확인되지 않았다. 공개 API hardening 후보로 보존하되, 실제 host 도달성 또는 지원 계약상 임의 ledger 부분 승인 요구를 확정한 뒤 수정한다. |
| [CF-01](CF-01-upstream-operator-enum-alias.md) | execution-local vocabulary라는 소스 주석과 실제 Types enum alias의 불일치는 사실이다. 현행 실행 결과·바이트·호환 실패는 확인되지 않았고 독립 enum 도입 자체가 Semantica의 Rust 타입을 바꾼다. 명시적 공개 계약 소유권 결정과 소비자 전환 계획이 나오기 전에는 bug fix로 실행하지 않는다. |
| [DOC-01](DOC-01-qg-diag-routing.md), [DOC-02](DOC-02-ticket-denominator.md) | 고정 HEAD의 오류이지만 다른 작성자의 현행 미커밋 ADR/README 통합안이 QG-DIAG 링크와 18개 분모를 교정한다. 중복 패치를 만들지 않는다. 그 문서가 통합되면 새 SHA에서 링크·ID를 다시 검사한다. |

기존 레인 보고와 제외 카드들은 판정 근거로 유지한다. 제외는 반례를 삭제하거나 `PASS`로 바꾸는 뜻이 아니다.

## 증거 경계

- QGLang 기존 clean-HEAD local receipt는 `25fad76`에서 Nextest selected/executed/passed `1529/1529/1529`, skip/leak 0이었다. 이번 최종 triage에서 `just ci`나 `just qualify-local`을 새로 실행하지 않았다. 이후 수정의 증거로 재사용하지 않는다.
- 이번 triage의 새 실행은 Semantica full locked metadata 재현과 소스/문서 readback이다. Modules 대표 부하 측정, 수정 후 owner 테스트, consumer compile/vector, hosted CI, runtime/product는 `NOT_RUN`이다.
- 소스·정책·Semantica 파일 수정, 커밋, push는 하지 않았다. 활성 6건은 **수정 작업 범위**이지 완료 판정이 아니다.
