# 이 세션의 잔여 작업 — 최신 코드 감사

관측: 2026-10-05 17:31 KST. 공유 main, 미커밋 overlay, clean 검증 후보를 구분한다. 런타임/provider/SDK 변경은 소비자 소유이며 신규 엔진·컴파일러·SMT·production library 추가는 없다. CI는 로컬 실행이다.

## 완료한 구조·정책 정리

- **Producer main `e598a1d1debdc68a1c2482f96589ff0c707d05d0`: frontend 수리 134개 파일 통합.** 기존 cc27ff2의 대형 파일 분할에 이어 request owner/permit 인증/Modules custody를 정리했다. One-LET Binder/Types, DTO/digest 의미를 보존했고 normal 의존성 4개와 request 전용 BTree 저장량 추정을 제거했다.
- Size **283 source PASS**, architecture **26 normal/5 dev/0 build PASS**. 다른 작업의 CI/RFC/reference/audit dirty 변경은 보존했다.
- Clean producer **`8675bcd2ba5609c0a80a7e24950edc624aa0c52b`**의 `just qualify-local` 내부 로컬 CI **1514/1514 PASS**, skipped/leaked0. 이 후보와 e598a1d의 committed tree는 `f0e97b3303d2b5ffe54f916d73ad4bd13123a745`로 같다. 공유 dirty overlay 전체의 qualification은 아니다.
- 같은 clean source의 60초 `just fuzz-smoke` **exit0/crash artifacts0**, source unchanged/clean. 정기 스케줄은 설정하지 않았다.

| 원래 요청 | 현재 코드·증거 | 남은 작업 |
|---|---|---|
| CI·clippy-msrv | 로컬 `just ci`; static/ci에 MSRV 편입; clean CI PASS | hosted CI 추가 없음 |
| document.rs 분할 | owner 모듈 분할 및 request owner 추출 통합 | 없음 |
| execution.rs·DTO 경계 | owner 분할; Pipeline 정의 + frontend-contract 공개 seam | 소비자 설치형 증명 |
| mega test3개 분해 | size gate test 위반0 | 없음 |
| fuzz smoke | 동일 clean source 60초 PASS | 정기 실행은 운영 선택 |
| types/check.rs·solver.rs | owner 모듈 분할 및 size PASS | 없음 |
| examples closed-authority |18개:10 planned/7 typed non-plan/1 Export typed refusal; CI PASS | 없음 |
| Execution DTO 계약1:1 | schema3/8/8, 독립 digest oracle; consumer pin 통합 | 없음 |
| sema/test_support |1950 code lines 및 자식 owner 모듈, size PASS | 없음 |

공개 DTO를 일괄 이동하거나 wire/schema를 교체하지 않았다.

## 검증 후 main에 합친 소비자 체크포인트

| main commit | 변경과 목적 | 실제 증거 |
|---|---|---|
| `e7ffc579b5c` | fs2 공유 잠금 명시 호출: Rust1.92 inherent 메서드 충돌 수리 | Linux trust owner6/6 PASS@a48 |
| `82cc45e7bd7` | producer subtree/vector pin 및 budgeted admission lint 동기화 | M23-120 static PASS, Python23+6+30; Rust/product 증명 아님 |
| `37e52c612fb` | canonical storage permit/sink/refusal 공개 seam, runtime import 및 관계 결과 회귀 | Linux relation whole37/37 PASS@8be; kernel 증명 |
| `3b714aa81bb` | IDE failure class를 detail move 전에 보존 | 실제 E0382 수리, Mac Graph11 PASS@27c |
| `14a45cf8eb8` | DuplicateExecutionAuthority→InvalidRequest, authority/count 유지 | 독립 exact1/1 및 Graph11 PASS@27c |
| `00290be4535` | bounded top-level Python Graph census의 parser→adapter→host 연결 | Mac Graph owner11/11 PASS@27c; Linux parser10+adapter9 PASS@8be |
| `76922f834eb` | 기존 `ServingSourceIssuanceTemporaryPermitV1` canonical trait 재수출 | 실제 SDK 호출부 포함 Mac4-binary product PASS@806 |
| `4305ac65652` | 공용 Match wire 상수4개 `pub(crate)`: library sibling·standalone binary 두 경로 컴파일 | library와 native binary 포함 Mac4-binary product PASS@806; 값/형식 변경 없음 |

모든 scoped 통합은 최신 main descendant에 replay하고 coordinator claim/barrier/source-commit/require-head, boundary before/after, staged hooks를 거쳤다. 다른 작업의 dirty/index를 합치지 않았다.

Relation37은 EXISTS 두0행, 상관/중첩, foreign binding/root, typed source/sort 거절, budget/cancellation, digest/output last-drop를 검증했다. Composition fixture는 실제 frontend DTO와 기존 relation compiler를 사용한다. **공개 Composition 성공 증명으로 확대하지 않는다.**

Graph 변경은 최대16 plain top-level Python 함수의 pass/decimal/direct same-module return call을 bounded census로 연결한다. 미지원·부분 census는 typed Incomplete이며 기존 BFS/identity/root/generation/lease custody를 유지한다. 새로운 Graph engine은 없다.

## 현재 실행 중인 최종 검증

### Mac — product 완료, 설치형 거절 proof 진행 중

- Frozen tuple: consumer **`806f4a9da70bd902e0b61561115e78bbdcd46092`** + producer867 + Index **`07d212e4c89eb7e2ed1b712ffd4a5f16c5f44324`**, 모두 clean.
- `quanta-cli-product.release-full.build` **GREEN/exit0**, immutable CLI/server/sql worker/native worker **4개**. 기존 owner assembler로 source/receipt/hash를 검증한 설치 bundle을 생성했다.
- Mac source digest **`553086404e21f19272e689595f2b47464a9bd22829a6770bf9362cc8ee425ec8`**. Receipt run **`20261005T080826.848905Z-76730bdae5c7`**.
- 실제 product 빌드에서 발견한 E0373/E0583/E0412/E0624/E0753/E0277/E0425는 담당 작업의 committed main 수리를 소비했다. 이 세션은 해당 파일을 중복 수정하거나 전체 dirty 후보를 합치지 않았다. E0432 canonical permit seam과 E0433/E0603 shared wire visibility는 위2개 checkpoint로 수리했다. 이전 RED receipt는 보존한다.
- **Graph typed refusal process RUNNING**, CHECK typed refusal pending. Graph 신규 독립 테스트는 PATH/Reachability의 finite catalog rejection reason·빈 payload·receipt 미발급을 검증한다. 기존 양성 Graph 함수와 registry oracle을 변경하지 않았다.
- **남은 통합:** 두 actual process rail의 exact1/1 GREEN, ignored0, bundle와 동일 source digest 확인 후 신규 Graph 거절 테스트68줄만 main에 합친다.

### Linux — 메모리 복구 후 Graph owner 재실행

- 과거8GiB Docker 조건의 SIGKILL/OOM3회는 RED/테스트0으로 기록했다. Frontend1/codegen1 시도도 실패했으며 gate나 oracle을 약화하지 않았다.
- 사용자 진행 승인 후 Docker 메모리를 **16GiB**로 변경·재시작했다. 중단 전 proof 컨테이너에 active child가 없음을 확인했고 기존 실행 컨테이너5개를 모두 복원했다. 이미지/volume 삭제는 없다.
- 위와 같은 Git tuple의 native Linux **aarch64** source에서 Graph owner 재실행 **RUNNING**, jobs1/frontend1. Linux source digest **`49e5414ba161ad85ca74f284072bf368ba690131d9dc08c60db4888039cc1df3`**로 Mac digest와 구분한다.
- **설치형 Graph/CHECK 양성 proof NOT_RUN.** Graph owner GREEN 후 같은 frozen source·physical target/lane에서 기존 generic product4-binary recipe→bundle→Graph PATH/Reachability/empty/budget/stale/foreign 및 signed CHECK zero/one/multi 실제 결과를 실행한다.
- aarch64 generic installed 후보와 공식 **x86_64 installed release profile**을 구분한다. 현재 환경의 PASS는 x86_64 release qualification이 아니다.

## 최신 main과 검증 후보의 차이

- Consumer 관측 HEAD **`dc2e2cb1a6e0b1886e0a0785a1fe9a4d63e4068f`**. 다른 작업으로 main은 계속 이동한다.
- 이후 다른 작업에서 portable parser/adapter의 metering/borrow/staging과 Graph test165줄 등을 수정했다. 원래 Graph owner11 및 parser10/adapter9 receipt는 원래 source에만 유효하다. 과거의 “소유17개 blob 모두 동일” 주장은 더 이상 현재 상태가 아니다.
- 경로별 committed delta와 dirty 상태는 아래 custody JSON에 기록한다. 최신 main 전체 product/설치형 qualification은 **없다**. 검증 후보 결과와 최신 main 결과를 합성하지 않는다.
- Semantic generation carrier3/3은 실제 semantic request 양성 clone/drop proof가 아니다. 기존 SDK source_admission→public ingress에서 Composition→UnsupportedFamily 선거절을 관측했으며, 새 configured-Semantic oracle과의 정합은 활성 소비자 작업에 전달했다. 해당 작업의 최신 source를 덮어쓰거나 새 Composition 기능을 이 maintenance에 추가하지 않는다.

## 잘못 남은 작업으로 계산하지 않을 것

- `query.result_shape`: leaf/record/nested 모두 현재 sema typed 거절. 새 지원 구현은 이번 범위가 아니다.
- SHOW staged 양성 경로가 있으므로 “항상 실패”라는 주장은 틀리다.
- Rename 포함 conformance19, stale 주석 수리, Graph cap1/dedup 지원 계약, EXISTS 결과 oracle 소스는 이미 반영됐다.
- 물리 optimizer, Graph engine, SMT, 신규 production dependency는 계획에 없다.

## 증거 위치

[Producer checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-main-checkpoint-20261005/checkpoint-result.json)
[Producer clean qualification](/Users/songmin/.codex/worktrees/fcp-producer-main-integration-oct5/quanta-gqlang-v2/target/qualification/8675bcd2ba5609c0a80a7e24950edc624aa0c52b/receipt.json)
[Same-source fuzz smoke](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-checkpoint-fuzz-8675bcd-20261005.json)
[Source별 proof 상태](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/latest-proof-status.json)
[Current-main delta custody](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/current-main-checkpoint-custody.json)
[TemporaryPermit checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/temporary-permit-checkpoint-result.json)
[Shared wire checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/shared-wire-checkpoint-result.json)
[Docker restart recovery](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/docker-memory-restart-result.json)
