# 최신 소스 감사와 실행 계획

## 현재 판정 — 2026-10-05 22:49 KST

이 문서의 2026-10-04 snapshot·계획·실패 기록은 **이력**이다. 현재 source와
잔여 액션의 기준은 [2026-10-05 최신 세션 감사](2026-10-05-current-source-audit-action-plan.md)다.
Producer `1ddeaac`, consumer `3f037ea04ee`의 source/hash를 재확인했다.

- Producer 구조 분할, scratch/request permit, dependency 방향, lint/MSRV,
  facade/pin 및 result-shape 진단 수정은 main에 통합됐다. Clean source867의
  CI1514/1514와 60초 fuzz PASS가 있으며 tracked code/tool input delta는 없다.
- 확정 코드 잔여는 소비자 **CHECK canonical counting/encoding work admission**이다.
  Product PTA의 과거 bootstrap 거절은 최신 seed-lifetime 변경 이후 해결 여부를
  아직 검증하지 못했다. 실제 거절 분기를 특정한 뒤 수리 필요 여부를 판정한다.
- Graph/CHECK fixture 통합은 same-source actual-query GREEN 이후다.
  Linux signed installed profile/trust/slot/approval 부족은 설치 전제로 구분한다.
- SHOW Pipeline staged plan/Execution DTO는 live다. Specialized one-shot
  `admit_show_planned_source_v2`의 typed refusal는 여전히 의도된 제한이다.

이 갱신은 문서/source 대조이며 새 Rust/QBC/CI/product 실행 결과를 만들지 않았다.
아래에서 “현재”, “최신”, “진행 중”은 각 **당시 관측 소스**를 뜻한다.
후속 완료 항목을 다시 backlog로 등록하거나 이전 receipt를 현재 main에 합성하지 않는다.

## 2026-10-04 관측·구현 이력

관측일: 2026-10-04, Asia/Seoul. 상태: 소스 감사 및 계획 작성.

## 구현 진행 기록

같은 날 후속 구현에서 아래 항목을 처리했다. 아래의 최초 감사 source tuple은
이력이며, 후속 commit/CI/consumer 실행 증거는 별도의 source-bound manifest로 기록한다.

- D: `resolve_document_bodies`를 preparation / dependency-first inline View /
  source-order item materialization으로 분리했다. typed phase output을 사용하고
  진단·IR·digest·step charge 순서를 유지했다. 기존 함수 크기 예외를 제거했다.
- F: 8개 legacy 파일 예외를 전부 제거했다. 5개 파일의 내부 테스트를 cfg(test)
  자식 모듈로 옮겼고, 나머지 3개는 catalog topology / authority inputs·open analysis /
  catalog family parsing 책임으로 분리했다. 현재 238개 Rust source가 예외 없이
  production/test 1,200/2,500 code LOC 정책을 통과한다.
- G: authored result-shape 전체 미지원 상태에 맞게 진단 메시지를 수정했다.
  기존 scalar/record/nested/Composition 거절 회귀에 정확한 메시지 oracle를 추가했다.
- 집중 검증: sema Clippy, document transaction 67/67, semantic budget 7/7;
  Surface lib/family 25/66, Modules lib/predeclaration 16/22,
  Sema lib/authority builder 60/7, result-shape conformance 1 PASS.
- A의 producer 소유 변경은 `cc27ff2`로 commit했고 contract subtree/vector pin은
  기존 consumer 기대값과 일치했다. 소비자 M23-120 정적 closeout gate도 통과했다.
  이 gate에서 발견한 오래된 compile API 검사를 generation-aware owner에 맞추고
  generation authority 제거를 거절하는 회귀 2개를 추가했다.
- clean snapshot의 첫 local CI: Nextest 1,433/1,433, doctest 8, frontdoor 10,
  external consumer 135 API/15 negative 및 앞선 정책 gate는 통과했다. 마지막
  semantic-honesty가 추출된 test-only 파일을 production으로 분류해 실패했다.
  실제 부모 AST의 cfg(test)/path를 확인하는 공통 inventory로 수정했고, 실제 guard
  / 누락 / 주석 / 잘못된 path 회귀를 포함한 focused gate 22개가 통과했다.
  전체 clean-HEAD CI는 이 수정이 포함된 commit에서 재실행한다.
- 같은 `cc27ff2`의 bounded offline fuzz 60초는 성공했다. corpus 1,240개,
  crash 0개이며 전후 source/HEAD/clean 상태가 일치했다.
- B/C/E의 실제 실행 증명은 후속 결과로 판정한다. 후속 증거는
  [remediation evidence](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-commit-source.json),
  [fuzz 결과](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/fuzz-smoke-result.json)에 보존한다.

목적은 현재 지원하는 gqlang 계약이 실제 소비자 실행까지 유지되는지 확인하고,
큰 함수의 변경 범위를 줄이는 것이다. 이번 턴은 production 수정이나 Rust 실행
검증을 하지 않았다. CI는 로컬에서 운영한다.

## 1. 감사한 소스와 증거 경계

| 대상 | 관측한 HEAD / 상태 | 감사 범위 |
| --- | --- | --- |
| gqlang | `9060e46997dd9c62080a18bdfae06f8c162442e2` + dirty | 기존 구조 분할, 로컬 게이트, frontend 계약, conformance |
| Semantica | 감사 시작 `d100b6bf…`, 파일 manifest `ffb2d18d…`, 후속 확인 `0f14cb9c7593cc4247946d3def4ddec2fafcd7b6` + dirty | Graph ingress → host → SDK/process, Stage5 proof, Python occurrence, CHECK |
| Quanta Index | 이번 턴 재감사하지 않음 | 소비자 실행 증명 전에 transitive path dependency 소스를 함께 동결해야 함 |

manifest 관측 당시 gqlang은 tracked 변경 62개 / untracked 파일 54개,
Semantica는 86개 / 4개였다. 공유 main에서 다른 작업이 계속되고 있으므로 이 수치는
전체 저장소 동결을 뜻하지 않는다. 선택한 owner 파일 31개(13 producer / 18 consumer)는 후속 확인에서
바이트 변화가 없었다. 완료 시점의 상태는 after manifest에 기록한다.

gqlang의 crate 파일 251개는 기존 검증 소스
`1b55f6233b578d1be351bb926dfbeac2a0262b70`과 동일하다. 기존 snapshot의 tracked
파일 비교에서 차이는 문서 2개였다. 이것으로 현재 dirty main의 clean-HEAD receipt나
Semantica 설치 실행 성공을 주장하지 않는다.

증거:

- [감사 전 파일 manifest](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-audit/source-manifest-before.json)
- [감사 후 파일 manifest](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-audit/source-manifest-after.json)
- [이번 감사의 체크 결과](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-audit/audit-checks.json)
- [이전 구현·검증 이력](2026-10-04-frontend-consumer-contract-plan.md)

## 2. 이전 주장 중 수정해야 하는 것

| 항목 | 최신 소스에서 확인한 사실 | 판정 |
| --- | --- | --- |
| Graph cap / dedup 불일치 | ingress가 serving witness cap을 1로 재봉인. host는 Reachability=`Endpoints`, Path=`NodeSequence` 허용 | 기존 수정 보존. 설치 실행 증명은 남음 |
| schema 7/8 golden 불일치 | consumer golden은 source/contract/qualification `3/8/8`, vector digest도 현재 working source와 일치 | 기존 수정 보존. 원본 main의 committed tree pin이 문제 |
| 복수 EXISTS 증명 | 두 false-conjunct 사례가 비어 있지 않은 입력에서 0행을 기대 | 회귀 증명 코드 보존 |
| 18 family 전체 증명 | 현재 registry는 Rename을 포함한 19개. preview/apply와 authored result-shape 거절 corpus 존재 | 이전 누락 보완됨 |
| CHECK 미연결 | SDK가 `execute_issued_check_v3`를 호출하고 command owner까지 연결됨 | 이전 미연결 주장은 현재 코드에 맞지 않음 |
| 일반 Python assignment가 실패 | 현재 negative test는 **unsealed assignment의 거절**을 요구 | 일반 assignment 결함으로 단정할 수 없음. production generation 재현 필요 |
| nested result-shape만 미지원 | sema는 authored `result_shape`를 모두 거절 | 기능 subset은 명확함. 진단의 “nested” 표현은 부정확 |
| 큰 파일이 그대로 남음 | document/execution/types/test harness 분할 반영, 현재 size gate 통과 | 남은 큰 함수와 정확한 legacy 예외를 대상으로 계획 변경 |

주요 소스:

- [serving Graph budget](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/lib.rs:639),
  [host Graph policy](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-host-spi/src/analysis_host.rs:438),
  [admission 회귀](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/public_source_family_contract_v3.rs:441).
- [schema-8 golden](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:1816),
  [EXISTS 독립 결과](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-execution/tests/gqlang_relation_execution_v3.rs:177),
  [facade conformance](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-conformance/tests/self_validation.rs:324).
- [CHECK SDK dispatch](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs:2085),
  [CHECK lifecycle / output admission](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler/check_dispatch_v3.rs:35).

## 3. 최종 액션 리스트

| 순서 / 우선순위 | 작업과 목적 | 완료 조건 |
| --- | --- | --- |
| A / P0 | **소유 변경 commit·pin·검증 소스 확정.** 동일한 producer/consumer 조합을 검사할 수 있게 함 | 검토한 producer 변경을 commit → 정확한 subtree/vector pin 확인 → 현재 consumer와 모든 관련 path dependency 동결. 기존 dirty refusal 유지 |
| B / P0 | **최신 소스의 installed Graph 증명.** 실제 Path/Reachability 결과와 authority 거절 확인 | 영향받은 Stage5/admission owner 확인 후 새 product artifact와 process test에서 정확한 1개 witness, 빈 결과, 예산 고갈, foreign/stale source 거절 확인 |
| C / P1 | **Python assignment production 재현.** 실제 결함 여부와 owning issuer 구분 | 실제 generation에서 assignment가 정확한 sealed occurrence로 성공하는지 확인. 성공이면 결함 항목 종료; 실패면 issuer owner 수정 및 독립 회귀 |
| D / P1 | **`resolve_document_bodies` 책임 분해.** 한 문법 변경의 영향 범위 축소 | typed phase helper 추출, 기존 진단/IR/digest/실패 순서 유지, 함수 크기 `expect` 제거. 해당 owner/conformance 및 마지막 로컬 `just ci` 통과 |
| E / P1 | **CHECK 플랫폼별 실행 증명.** 연결된 코드의 지원 범위 확정 | Linux signed catalog 0/1/multi 성공과 relevant 거절, macOS unsupported worker의 typed 거절. 소스/플랫폼/artifact 기록 |
| F / P2 | **남은 8개 파일 크기 예외 축소.** 책임 경계를 개선하며 재성장 방지 | 각 owner를 수정할 때 책임별 추출 후 정확한 ceiling을 낮추거나 예외 제거. 예외 추가/상향으로 통과시키지 않음 |
| G / P2 | **result-shape 진단 정확화.** 지원 subset과 오류 설명 일치 | leaf/record/list 모두 거절함을 정확히 설명. error code/span 및 typed failure 유지 |

A → B는 순차로 수행한다. C와 D는 서로 다른 저장소/owner이므로 독립적으로
진행할 수 있다. E의 구현·test는 현재 consumer 소스에 있으므로 소유 상태를
재확인하고 기존 구현을 검증한다. 같은 QBC lane의 build/test는 동시에 실행하지 않는다.

### A. commit과 pin: 현재의 실제 차단 조건

[closeout gate](/Users/songmin/Documents/code-new/semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh:938)는
frontend/pipeline dirty 상태를 거절한 뒤 **committed subtree**를 비교한다.

| 입력 | consumer 기대값 | 원본 gqlang main의 committed 값 |
| --- | --- | --- |
| frontend-contract tree | `bd1e06e9ed26a70304aa4cfdc3838436b17d74a6` | `026f736f17820acc0f92028a83aee8d8fc28decb` |
| pipeline tree | `c459d923288e04c575ea4027438465604a4ed07c` | `d9507023127a0af247a5531d6fe1dd575f1b4f82` |
| vector SHA-256 | `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb` | working file 일치 |

핵심은 검증된 overlay를 검토된 commit으로 정리하는 것이다. 현재 근거로 golden
재생성이나 schema 변경은 필요하지 않다. 실제 commit tree가 기대 pin과 같으면
pin을 다시 바꾸지 않는다. 다르면 차이를 먼저 검토한다.

gqlang의 기존 `.circleci/`와 `tools/ci/circleci-install.sh`는 별도 작업이다.
이 로컬 CI 계획의 commit에 섞지 않는다. Semantica의 unrelated taint/PTA/parser/index
변경도 별도로 보존한다.

### B. Graph와 Stage5: 남은 것은 현재 소스의 실행 증명

[Stage5 admission](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime/src/executor/registered_child_proof_v1.rs:135)은
typed canonical bytes, fresh request, cache miss, nonempty output, 기존 overlay 부재,
persist/wire/full-trace 정책을 검사한다. typed-only 허용 수정은 유지돼 있다.
현재 helper import는 `quanta-runtime-execution` owner로 이동했다. 이전 snapshot의
7-test PASS는 이 owner 이동 이후의 main 실행 증명이 아니다.

실행 순서:

1. 현재 source tuple과 QBC selector/feature profile 확정.
2. Stage5 proof 및 source/runtime projection, Graph admission의 영향받은 owner 확인.
3. 새 product binary build/receipt/bundle 생성.
4. [installed Graph test](/Users/songmin/Documents/code-new/semantica-codegraph-v2/crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:4955) 실행.
5. `alpha→beta`의 정확한 node sequence 1개, `alpha→gamma` 0개, budget=1 거절,
   foreign/stale source의 성공 receipt 부재 확인.

이번 감사에서 QBC owner resolution은 성공했다. compile policy는
`feature-isolation:quanta-cli-server.no-default.3c0734102bd7`, request ID는
`99e81b931fd46cbaa717ffcf34d7ad22`이다. **resolution은 실행 PASS가 아니다.**
확인한 기록에 현재 소스의 installed Graph PASS는 없었다.

### C. Python: authority를 완화하기 전에 production issuer를 재현

[현재 negative test](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-pta/src/pta_constraint_extract_v2/native_constraint_extraction_transaction_v1_tests/python_direct_return.rs:568)는
sealed row가 없는 `value = beta(); return value` 호출을 거절한다.
[direct-return fallback](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-pta/src/native_python_direct_return_call_admission_v1.rs:30)은
`__return__` 및 정확한 return SSA/occurrence에 한정된다.
[normal binding](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-pta/src/native_normalized_attribution_generation_artifact_v2.rs:1109)은
먼저 owner-issued occurrence를 소비한다.

- 실제 captured generation의 ordinary assignment로 positive oracle를 만든다.
- source/DFG/generation/callee/return/statement/slot identity가 일치해야 성공한다.
- missing/foreign/duplicate occurrence는 계속 거절한다.
- 정상 assignment에 row가 발행되지 않는 것이 확인되면 occurrence issuer/join owner를
  수정한다. direct-return guard를 넓혀 authority 부재를 숨기지 않는다.

### D. 큰 함수: 단계별 typed output으로 추출

[bodies.rs](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-sema/src/document/bodies.rs:26)는
Tokei 기준 1,094 code LOC이며 `resolve_document_bodies`에 함수 크기 예외가 있다.
이 파일에는 **정확한 legacy 파일 ceiling 예외가 없다**. 일반 production 1,200줄
정책은 유지하고, 이 함수의 예외를 제거하는 것이 직접 목표다.

추출 순서:

1. declaration/callable binding과 type input 준비.
2. sealed topology 순서의 inline-view 준비.
3. source 순서의 family dispatch.

각 phase의 소비자와 typed output을 명시한다. authority/validator/step counter의
소유와 debit 순서, 첫 실패와 진단 순서, item ID/IR/digest를 보존한다.
거대한 공유 context나 새로운 generic pass framework를 만들지 않는다.
기존 document transaction/conformance oracle로 검증하고 크기 예외를 제거한다.
frontend-contract는 기존 facade를 유지한다. 현재 facade가 pipeline에 의존하므로
DTO를 그대로 옮기고 pipeline이 이를 참조하게 하면 cycle이 생긴다.

현재 주요 파일 code LOC: document root 1,179, execution root 1,199,
types/check root 1,061 / inputs 1,104, solver root 1,022, sema/test_support 1,950.
대형 root/harness 분해는 이미 반영됐다. 다음 구조 작업은 기능 owner별로 정한다.

### E. CHECK: 플랫폼을 구분해 증명

`gqlang-check-process-proof` feature와 owner isolation profile, process regression이
현재 consumer 소스에 추가돼 있다. 신규 구현부터 시작할 항목이 아니다.

- [Linux positive matrix](/Users/songmin/Documents/code-new/semantica-codegraph-v2/crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:5203): signed catalog 0/1/multi 결과.
- [non-Linux refusal](/Users/songmin/Documents/code-new/semantica-codegraph-v2/crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs:5316): unsupported worker typed 거절.

macOS 거절 PASS로 Linux execution PASS를 대신하지 않는다. Linux 지원을
출시 주장에 포함하려면 대응하는 로컬 Linux 환경에서 positive matrix를 수행한다.
현재 연결된 CHECK가 모든 규칙 실행이나 SMT 지원을 뜻하지는 않는다.

### F/G. 정책과 진단

- 현재 파일 정책: production/test 1,200/2,500 code LOC, Clippy 함수 200줄.
  legacy 예외 8개는 [정확한 path/ceiling](/Users/songmin/Documents/code-new/quanta-gqlang-v2/tools/lint/size_policy.json)으로 고정된다.
- `static`에 current/MSRV Clippy, `policy`에 architecture gate가 포함된다.
  새 `allow`나 예외 추가로 분할을 우회하지 않는다.
- parser/formatter 변경 및 release 전에는 기존 `just fuzz-smoke`를 사용한다.
  기본 fuzz 시간은 60초, 허용 범위는 1–300초다. source/seed/corpus/crash를 보존한다.
  fuzz는 현재 `ci`/`qualify-local`의 자동 leaf가 아니다.
- [result-shape 거절](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-sema/src/document/query_materialization.rs:31)의
  메시지를 전체 authored shape 거절에 맞게 고친다. 기능 지원 확대는 별도 요구가 있을 때 결정한다.

## 4. 이번 감사의 검증 결과

| 체크 | 결과 / 의미 |
| --- | --- |
| `python3 -m tools.lint.size_gate` | PASS, Rust source 파일 225개. 현재 파일 크기 정책 준수 |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m tools.architecture.check_architecture` | PASS, workspace 14개 / normal 26·dev 5·build 0 edges. manifest 의존 방향 검사 |
| frozen producer 대비 crate byte comparison | 251개 동일. 과거 검증과 현재 crate 소스의 일치 |
| vector digest 비교 | 일치. 원본 committed subtree 정합성과는 별개 |
| Graph QBC owner resolve | PASS. selector/feature topology 확인, build/test 아님 |
| selected owner source 재확인 | 31개 변화 없음. 전체 consumer의 source freeze 아님 |
| Rust tests/build, `just ci`, fuzz, `qualify-local`, installed Graph/CHECK, 전체 release qualification | 이번 턴 **NOT_RUN** |

이전 producer local CI와 consumer owner PASS는 해당 snapshot의 역사적 증거다.
현재 소스에 영향을 주는 owner 변경을 검증한 뒤 같은 source/artifact tuple의
installed process 결과를 기록해야 통합 완료로 표시한다.

## 5. 완료 범위와 별도 기능

이번 액션 리스트는 기존 계약의 정합성, 실행 증명, 구조 분해 및 로컬 게이트에
한정한다. 신규 production 라이브러리 추가는 현재 필요한 근거가 없다.

authored result-shape 지원, specialized `admit_show_planned_source_v2`의 항상
Incomplete인 경로, 새 rule evaluator/물리 optimizer/SMT/증분 실행은 별도 기능이다.
실제 소비자 요구와 의미·예산·거절 계약을 정한 뒤 우선순위를 결정한다.
