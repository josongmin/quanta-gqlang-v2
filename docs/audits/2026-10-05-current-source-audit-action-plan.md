# 이 세션의 잔여 작업 — 최신 코드 감사

관측: 2026-10-05 16:02 KST. 공유 main, 미커밋 overlay, 격리 후보와 실제 검증 소스를 구분한다.

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
| Execution DTO 계약1:1 | schema3/8/8 및 독립 digest oracle, CI PASS; consumer pin/lint 반영 | 없음 |
| sema/test_support 정리 |1950 code lines 및 자식 owner 모듈, size PASS | 없음 |

초기 큰 파일 분할 자체는 cc27ff2에 이미 committed였다. 이번 통합은 그 뒤 공유 dirty request 변경에서 재발한 크기·의존성 위반과 소유권 경계를 정리한 것이다. 공개 DTO를 다른 crate로 일괄 이동하지 않았다.

## 필수 잔여 액션

### 1. P0 — Graph owner checkpoint 및 나머지 소비자 통합

- **Consumer pin/lint main `82cc45e7bd763c0651b753df7c9d3b9b1706fe3f` 통합 완료.** Producer subtree/vector pin과 현행 budgeted admission 호출 marker를 맞췄다. 기존 provider-before-selection 거절을 유지하고 실제 mutation 회귀를 보완했다. `M23-120` static gate PASS: Python23+6+30, locked/offline metadata. Rust나 설치형 제품 증명은 아니다.
- **Execution seam 및 회귀 main `37e52c612fbbabdadae376deefa8bca2f7eefeb6` 통합 완료.** Storage permit/sink/refusal을 기존 exec-contract seam으로 노출하고 runtime import를 연결했다. Composition output-field borrow는 다른 checkpoint가 main에 먼저 반영해 중복 변경을 제외했다.
- **Graph parser/adapter/host 11개 파일은 main `00290be45350259f8fd9d379d7406e9526a2dc0d`에 통합 완료.** Native macOS candidate `27c247841f5ef5d87331e056bfecee6cfd17bccc`의 Graph owner11/11 GREEN 뒤 최신 main descendant에 경로를 제한해 replay했다. Scoped boundary before/after와 staged rustfmt/safety fence PASS. 통합 blob은 검증 후보와 같으며 다른 작업의 dirty/index를 보존했다.
- **IDE 오류 전달 수리는 main `3b714aa81bb83797128246f958fd01d2950f8868`, duplicate authority host 분류는 main `14a45cf8eb8c4b6a80e95373be395eae8234ff30`에 통합 완료.** 실제 E0382/E0004 이후 소유 consumer를 수정했고 Graph11 및 duplicate InvalidRequest 회귀1/1 GREEN이다. 새 오류의 authority/count를 유지한다.
- SDK·ingress generation 전달과 registry는 활성 작업과 invariant/proof 조정이 필요하다. Graph 결과/selector authority는 최신 main과 검증 후보가 byte-identical임을 재확인해 추가 패치에서 제외했다. 사용자 진행 지시 후 AG5와 IRC 소비자 작업에 설치형 컴파일 전제와 Semantic 지원 정합을 전달했다. 기존 lock·registry·SDK·새 Semantic process oracle을 candidate 전체로 덮어쓰지 않는다.
- **완료 조건:** owner GREEN 뒤 최신 main descendant의 정확한 변경 경로만 coordinator claim/barrier/source-commit/require-head로 통합한다. 겹치는 후속 묶음은 조정 후 재검토한다.

### 2. P0 — Graph host 최종 owner 검증

- Candidate8be + producer867 + Index07의 관계 실행 전체 **37/37 GREEN**, filtered0/ignored0. Source digest `0ea3af40e2452f0703d81df7f05877ea051e3dad9810358688f329c9c1cc3782`. EXISTS 두0행, 상관/중첩, foreign binding/root, typed source/sort 거절, budget/cancellation, digest 및 output last-drop를 포함한다.
- 초기 테스트 private API 및 unowned Composition fixture는 현재 계약에 맞게 고쳤다. 실제 frontend DTO를 기존 relation compiler에서 검증하는 **커널 증명**이며 공개 Composition 성공을 주장하지 않는다. 별도 public selection refusal도 GREEN이다. Long rename 기대값에는 wire logical demand 외 sized Arc owner demand를 포함했다; production charge를 제거하지 않았다.
- Graph host 최신 8be 실행은 **RED/테스트0**: Rust1.92 Linux aarch64, Docker7.65GiB, jobs1/frontend thread1에서도 runtime-analysis가 SIGKILL. Compiler의 dev codegen-units256과 Docker OOM 기록을 확인했다. Source guard는 정상 최종화해 receipt를 발행했다. 이전 bind-metadata inode 문제는 native source로 재발하지 않았다.
- Linux frontend1/codegen1 재시도도 **OOM/테스트0**으로 종료됐다. Peak child RSS5,551,480KiB, Docker guest OOM counter3. 다른 Index 컨테이너가 약2GiB를 사용 중이며 Docker7.65GiB 조건은 유지됐다. 다른 컨테이너를 중단하거나 Docker 설정을 바꾸지 않았다. 16GiB 변경·재시작은 다른 작업을 잠시 중단시키므로 사용자 확인을 요청했다.
- **Native macOS48GiB Graph owner11/11 GREEN**: source27c, digest `314645e4299aa59e7f8b833fcf8601e2b287a5d51fbbc0579c3034d4d33e6126`, failed/ignored0, filtered163. 체인/순환/정확한 witness, foreign/partial census/short budget 거절을 포함한다. Duplicate authority exact 회귀1/1도 같은 digest GREEN이다. Mac 결과는 Linux 설치형 증명이 아니다.
- Candidate8be + producer867 + Index07의 **Linux parser10/10 및 adapter9/9를 새로 확인**했다. 동일 source digest0ea3af40, ignored0. 종전 a48 proof와 구분하며 SDK/product completion으로 확대하지 않는다.
- **완료 조건:** Graph host 실제 선택/실행 수와 receipt 확인, main replay의 proof-input delta 기록. 이후 동일 committed tuple의 process proof를 별도로 확인한다.

### 3. P0 — 같은 소스의 설치형 Graph·CHECK 실행 증명

- `quanta-cli-product.release-full.build`의 immutable4-binary bundle, build receipt, binary hashes와 source tuple을 고정해야 한다.
- Linux 설치형 Graph: PATH/Reachability 성공, unreachable 빈 결과, budget/stale/foreign 거절. CHECK: signed catalog zero/one/multi 실제 matcher 결과.
- macOS는 parser/worker 미설치의 typed Unsupported 및 결과/receipt 미발급을 검증한다.
- **Linux 설치형 NOT_RUN.** Linux 환경은 aarch64이며 x86_64 증명이 아니다. 반복 OOM 조건이 바뀌어야 설치형 bundle/process 검증을 진행한다.
- Mac 설치형 제품 빌드에서 발견한 E0373 async capture, E0583 모듈 경로, E0412 타입 경로, E0624 kernel 부모 접근, E0753 included-file doc 주석은 담당 작업의 committed main 수리 `d806bc6230c`·`e685a7d0895`·`5156a674411`·`1c25e469e62`를 소비했다. 이 작업은 해당 경로를 중복 수정하거나 main으로 재수입하지 않았다.
- Frozen consumer `1bba0ba893c4aa4eddddd8c6498f7ffd19b21df8` + producer867 + Index07의 최신 product run은 **RED101**: `executor/handle/dispatch.rs:1252`의 `Vec<u8> != Arc<[u8]>` 비교 E0277. Source digest `147b007efd8fbb2223c5167cad827fd67ae7f41acedadea1efb11dd0dbab0fd2`, immutable receipt `20261005T065605.129997Z-7fd794eff04f`. 최신 main에도 같은 비교가 있었으며 AG5 소유 claim이 활성이라 오류와 exact source를 해당 작업에 전달했다. Immutable bundle 미생성, process 테스트0; 이 결과는 메모리 실패가 아니다.
- Product/test가 같은 physical target을 쓰도록 기존 Mac warm lane의 linker-native symbol-strip 플래그를 제품 빌드에도 명시했다. Guard와 oracle은 유지했으며 target reset은 없다.
- 별도 `daemon_native_gqlang_graph_unsupported_worker_refuses_v3`는 현재 frozen candidate `1bba0ba893c4aa4eddddd8c6498f7ffd19b21df8`에 준비됐다. 기존 Path/Reachability 양성 함수와 registry oracle을 유지한다. PATH와 Reachability 모두 catalog refusal의 유한 public reason, 빈 payload와 receipt 미발급을 검증하도록 작성했다. **NOT_RUN/미통합**, AG5 바이트 비교 수리의 committed source가 전제다. 테스트 예상값은 Catalog→public reason→SDK 매핑을 추적해 정했으며 실행 출력에서 복사한 값이 아니다.
- **완료 조건:** 최종 committed producer/consumer/Index 조합, 동일 physical build root/lane의 bundle와 process rail, source drift 및 cleanup 확인.

### 4. 완료 — Consumer closeout pin; 후속 증거 기록

- Producer main의 frontend subtree `325a744628a282f875fc5e080ffc639672f14d9b`, pipeline subtree `be0572f3bfa2ed28ee2a369a3379f564dd3407e9`는 후보718/867과 같다.
- Consumer main `82cc45e7`의 closeout gate에 이 pin을 반영했고 static gate를 통과했다.
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
- Consumer shared 관측 HEAD: `d0b896735ac1ba492ddb5401cb88d9a9f300cfa4`; 공유 HEAD는 다른 작업으로 계속 이동한다. Trust e7ffc579, pin/lint82cc45e7, execution37e52c6, IDE3b714aa, error14a45cf, Graph00290be는 그 main 계열에 반영됐다.
- Full consumer 후보 branch: `codex/fcp-consumer-de040-upstream-first-20261005`의 a48. Host 작업트리는 trust checkpoint를 위해 최신 main 기반 별도 branch로 이동했다. 현재 Linux native frozen proof 조합은8be/867/07이며, 초기 a48/718/07 proof와 구분한다. 최신 관측 main에서 execution·IDE·map·Graph 소유17개 blob은 해당 GREEN source와 동일하고 경로별 dirty0다.

[Producer main checkpoint 기록](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-main-checkpoint-20261005/checkpoint-result.json)
[Producer clean qualification](/Users/songmin/.codex/worktrees/fcp-producer-main-integration-oct5/quanta-gqlang-v2/target/qualification/8675bcd2ba5609c0a80a7e24950edc624aa0c52b/receipt.json)
[최신 fuzz smoke](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-checkpoint-fuzz-8675bcd-20261005.json)
[Trust main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/installed-trust-checkpoint-20261005/checkpoint-result.json)
[Consumer 통합 감사](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/checkpoint-integration-audit-20261005.json)
[Linux adapter QBC](</Volumes/Extreme SSD 1/gqlang-linux-check-proof-20261004/final-tuple-d15d6a-20261005/qbc-controlled-cache-bridge-a48a3ca6b79d-result.json>)

[Consumer pin/lint checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/producer-pin-checkpoint-20261005/checkpoint-result.json)
[Execution main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/exec-checkpoint-result.json)
[Relation whole37 QBC](</Volumes/Extreme SSD 1/gqlang-linux-check-proof-20261004/qbc-relation-whole-native-8be8cd2f23b-20261005-result.json>)
[Graph OOM disposition](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/graph-host-oom-disposition.json)

[Graph main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/graph-checkpoint-result.json)
[IDE main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/ide-checkpoint-result.json)
[Duplicate authority main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/map-checkpoint-result.json)
[최신 source별 proof 상태](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/latest-proof-status.json)
