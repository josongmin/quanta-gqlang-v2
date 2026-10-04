# 이 세션의 잔여 작업 — 최신 코드 감사

관측: 2026-10-05 05:57 KST. 공유 main, 미커밋 overlay, 격리 후보와 실제 검증 소스를 구분한다.

## 이번 체크포인트

- **gqlang main `e598a1d1debdc68a1c2482f96589ff0c707d05d0`에 frontend 수리 134개 파일 통합 완료.** 기존 one-LET Binder/Types 동작, 전체 permit 체인 인증, Modules 세션 custody, DTO/digest 의미를 보존했다. 금지된 normal 의존성 4개와 request 전용 BTree 저장량 추정을 제거했다.
- 공유 producer에서 size **283 source PASS**, architecture **26 normal/5 dev/0 build PASS**. 이전 dirty overlay의 size11/architecture4 실패는 해소됐다. 다른 작업의 CI/RFC/reference/audit 등 23개 dirty 파일은 보존했다.
- Clean 후보 `8675bcd2ba5609c0a80a7e24950edc624aa0c52b`의 `just qualify-local` 내부 로컬 CI **1514/1514 PASS**, skipped/leaked0. `e598a1d`의 committed tree는 이 후보와 동일한 `f0e97b3303d2b5ffe54f916d73ad4bd13123a745`다. 공유 dirty overlay 전체를 qualification한 것은 아니다.
- 같은 clean 후보의 60초 `just fuzz-smoke` **exit0, crash artifacts0**, source unchanged/clean. 자동 스케줄은 설정하지 않았다.
- **Semantica main `e7ffc579b5cb84bf56e8f077ecb7d38a3a9f43cc`에 Linux file-lock 컴파일 수리 통합 완료.** Rust1.92 inherent `File::try_lock_shared`와 기존 fs2 메서드 충돌을 fs2 명시 호출로 해결했다. QBC owner **6/6 PASS**에는 signed admission 및 rotation/foreign/mutated-image 거절이 포함된다.
- Consumer 격리 조합 `a48a3ca6b79d95fd4229ed2594bb463847a6931b` + producer718 + Index07에서 Linux adapter **9/9 PASS**. 설치 fixture의 top-level 3함수 Graph와 source-root/foreign/budget/last-drop를 검증했다.

## 원래 P0/P1 목록 대조

| 원래 항목 | 현재 상태 | 남은 것 |
|---|---|---|
| CI 호스트화 | 사용자 결정에 따라 로컬 `just ci` | hosted CI 추가 없음 |
| clippy-msrv → static/ci | main 구현 및 clean 후보 CI PASS | 없음 |
| document.rs 분할 | 기존 분할 cc27ff2 및 이번 request owner 추출 통합 | 없음 |
| execution.rs 분할·DTO 경계 | main owner 분할; Pipeline 정의 + frontend-contract 공개 seam | consumer 통합 및 실행 증명 |
| mega test3개 분해 | main 크기 게이트 test 위반0 | 없음 |
| fuzz smoke | 최신 동일 tree의 60초 smoke PASS | 정기 로컬 실행은 선택적 운영 설정 |
| types/check.rs·solver.rs 분할 | main owner 모듈 분할, size PASS | 없음 |
| examples closed-authority 확대 |18개:10 planned/7 typed non-plan/1 Export typed refusal; CI PASS | 없음 |
| Execution DTO 계약1:1 | schema3/8/8 및 독립 digest oracle, CI PASS | consumer closeout pin 반영 |
| sema/test_support 정리 |1950 code lines 및 자식 owner 모듈, size PASS | 없음 |

초기 큰 파일 분할 자체는 cc27ff2에 이미 committed였다. 이번 통합은 그 뒤 공유 dirty request 변경에서 재발한 크기·의존성 위반과 소유권 경계를 정리한 것이다. 공개 DTO를 다른 crate로 일괄 이동하지 않았다.

## 필수 잔여 액션

### 1. P0 — Consumer 나머지 수리 main 통합

- 전체 격리 후보는 common base de040 대비30개 파일. Linux trust1개는 위 체크포인트로 반영했고, **나머지29개**를 최신 main에 통합해야 한다.
- Graph parser/adapter/host의 top-level16함수 bounded projection, canonical IDs, chain/cycle/depth/work 처리; SDK·ingress generation owner 전달; request 회귀 및 source pin이 포함된다.
- Provider alias/clone/last-drop 및 foreign-root **3/3**, Graph 결과 owner **8/8**, generation carrier **3/3**은 `082640af`/source digest `1936608667ea97c33ab4a93685b38e47eba1db05a3589381a1a96d844bac986c`에서 PASS다. 최신 a48의 추가 변화는 Linux trust1개다. 이 결과를 최신 main의 전체 제품 증명으로 읽지 않는다.
- 재확인 HEAD `ed3abaf7f0754561aa3d83ecca4e7f6ece1971b4`에서는 committed 교집합이5개다: lock, suite registry, daemon E2E, SDK handler, dense factory. Dense factory에는 새 dirty 변경도 있다. SDK의 boxed permit 선예약·control 전달과 새 process oracle을 보존해 재조정해야 한다. SDK·ingress·lock·registry의 활성 claim은 다른 실제 작업에서도 진행 중임을 확인했다.
- **완료 조건:** 최신 main descendant에 정확한 owner 변경만 재적용, 양측 변경 보존, coordinator의 claim/barrier/source-commit/require-head로 checkpoint 생성. 다른 dirty/index 변경은 포함하지 않는다.

### 2. P0 — Graph host·EXISTS·SDK 최종 owner 검증

- Linux parser portable HIR **10/10 PASS**는 b41 소스에서의 증거다. 뒤 변경은 provider test 및 trust1개로 parser 입력은 그대로다.
- Linux adapter **9/9**, trust **6/6**은 a48의 동일 source digest `5fb8ca024533fd917ba75a9fe4ce38a1a7d49f3fc8240493189efbe5bbbc7f95`다.
- **Graph host QBC 종료, 테스트0:** broad dist-full 컴파일에서 runtime-analysis rustc SIGKILL 및 Docker OOM 기록, request-spec 모듈 경로 E0433이 확인됐다. 경로 오류는 최신 공유 main에 이미 수정돼 있어 오래된 후보에 중복 수리하지 않는다.
- 최종화에서 추가로 약4만 소스 항목의 inode 변경을 감지해 QBC가 결과 JSON을 발행하지 않았다. Tuple은 현재 clean이고 샘플의 내용·시간 정보는 유지됐지만 최초 전체 inode/byte 상태와의 동등성은 입증되지 않았다. Docker bind metadata 불안정 가능성이 있으며 가드를 유지해야 한다. **이 실행의 qualified verdict는 없다.**
- 후속 EXISTS exact·SDK lib check는 실행되지 않았다. 최신 소스를 통합하고 Linux native filesystem에 소스를 고정한 뒤, Docker7.65GiB에 맞춰 jobs1/낮은 rustc frontend thread 수로 검증을 재개한다. Cache는 호환성 범위에서 보존한다.
- EXISTS의 비어 있지 않은 입력 두0행 oracle은 이미 구현됐다. 새 테스트 작성보다 실제 실행 결과 확인이 남았다.
- **완료 조건:** 실제 선택/실행 수와 receipt 확인; main replay의 proof 입력 delta 검토 및 바뀐 proof closure 재검증. 컴파일-only SDK check를 SDK 제품 실행 증명으로 대신하지 않는다.

### 3. P0 — 같은 소스의 설치형 Graph·CHECK 실행 증명

- `quanta-cli-product.release-full.build`의 immutable4-binary bundle, build receipt, binary hashes와 source tuple을 고정해야 한다.
- Linux 설치형 Graph: PATH/Reachability 성공, unreachable 빈 결과, budget/stale/foreign 거절. CHECK: signed catalog zero/one/multi 실제 matcher 결과.
- macOS는 parser/worker 미설치의 typed Unsupported 및 결과/receipt 미발급을 검증한다.
- **현재 NOT_RUN.** Linux owner PASS를 설치형 제품 완료로 계산하지 않는다. Linux 환경은 aarch64이며 x86_64 증명이 아니다. 새 실행은 위 native-source/메모리 선행 조건을 먼저 해결해야 한다.
- **완료 조건:** 최종 committed producer/consumer/Index 조합, 동일 physical build root/lane의 bundle와 process rail, source drift 및 cleanup 확인.

### 4. P0 — Consumer closeout pin과 최종 증거 기록

- Producer main의 frontend subtree `325a744628a282f875fc5e080ffc639672f14d9b`, pipeline subtree `be0572f3bfa2ed28ee2a369a3379f564dd3407e9`는 후보718/867과 같다.
- Candidate consumer pin은 이 값이다. 공유 closeout gate는 이전 `1a8c…`/`8a085…` 값이라 아직 반영이 필요하다.
- Vector SHA `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb`, schema3/8/8. 새 wire/schema 변경 근거는 없다.
- **완료 조건:** checkpoint와 실행 증거를 exact SHA/source digest/selector/features/toolchain별로 남긴다. 과거 source의 GREEN과 현재 main qualified 상태를 구별한다.

## 완료나 신규 기능으로 잘못 계산하지 않을 것

- Semantic generation carrier3/3은 **실제 semantic request의 양성 clone/drop 증명이 아니다**. 현재 SDK source_admission_v3→compile_issued_gqlang_source_with_generation_v3 경로는 public ingress의 Composition→UnsupportedFamily 선거절에 도달하며 Relation의 Some generation도 거절된다. 공유 main의 새 configured-Semantic process oracle은 양성 결과를 기대하나, 그 테스트의 PASS는 확인되지 않았다. Oracle과 선거절 경로의 정합은 해당 작업과 통합 조정해야 한다. 현재 공개 성공 기능을 이 maintenance의 새 구현 범위로 추가하지 않는다.
- `query.result_shape`는 leaf/record/nested 모두 현재 sema에서 typed 거절한다. 신규 지원은 이번 잔여가 아니다.
- SHOW staged 양성 경로가 있어 SHOW 전체 불능이라는 주장은 틀리다.
- Rename 포함 conformance19 corpus, stale 주석 수리, Graph cap1/dedup 계약, EXISTS oracle 소스는 이미 구현된 항목이다.
- 물리 optimizer, 새 Graph engine, SMT, 신규 production library는 이번 계획에 없다.
- 다른 작업의 Index benchmark 및 CI/RFC 변경은 이 checkpoint의 소유 범위가 아니다.

## 소스·증거

- Producer core checkpoint: e598a1d; committed tree=f0e97b3. 이후 이 소유 보고서의 문서 checkpoint는 별도다.
- Consumer shared 관측 HEAD: `824c70ecb68efc8102f71d92866ef347b587e1e4`; 공유 HEAD는 다른 작업으로 계속 이동한다. Trust checkpoint e7ffc579는 그 main 계열에 반영됐다.
- Full consumer 후보 branch: `codex/fcp-consumer-de040-upstream-first-20261005`의 a48. Host 작업트리는 trust checkpoint를 위해 최신 main 기반 별도 branch로 이동했다. Linux frozen 조합은 a48/718/07 clean이다.

[Producer main checkpoint 기록](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-main-checkpoint-20261005/checkpoint-result.json)
[Producer clean qualification](/Users/songmin/.codex/worktrees/fcp-producer-main-integration-oct5/quanta-gqlang-v2/target/qualification/8675bcd2ba5609c0a80a7e24950edc624aa0c52b/receipt.json)
[최신 fuzz smoke](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-checkpoint-fuzz-8675bcd-20261005.json)
[Trust main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/installed-trust-checkpoint-20261005/checkpoint-result.json)
[Consumer 통합 감사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/checkpoint-integration-audit-20261005.json)
[Linux adapter QBC](</Volumes/Extreme SSD 1/gqlang-linux-check-proof-20261004/final-tuple-d15d6a-20261005/qbc-controlled-cache-bridge-a48a3ca6b79d-result.json>)
