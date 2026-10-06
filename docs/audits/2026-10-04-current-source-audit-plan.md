# 최신 소스 감사와 실행 계획

## Historical proof 경로 안내 — 2026-10-06

이 문서의 과거 checkpoint/qualification checkout 일부는 보관 후 퇴역했다.
현재 상태와 복구 경로는 [최신 액션 계획](/Users/songmin/Documents/code-new/quanta-gqlang-v2/docs/audits/2026-10-05-current-source-audit-action-plan.md)과
[검증 입력 정리 기록](/Users/songmin/Documents/code-new/quanta-gqlang-v2/docs/audits/2026-10-06-proof-input-retirement.md)을 참조한다.
과거 PASS의 source/플랫폼/selector 범위는 그대로이며 새 qualification을 뜻하지 않는다.

## 현재 판정 — 2026-10-05 23:50 KST

이 문서의 2026-10-04 snapshot·계획·실패 기록은 **이력**이다. 현재 source와
잔여 액션의 기준은 [2026-10-05 최신 세션 감사](2026-10-05-current-source-audit-action-plan.md)다.
Producer `7cf36e88739`, consumer `65836a3adfc`의 source/hash를 재확인했다.

- Producer 구조 분할, scratch/request permit, dependency 방향, lint/MSRV,
  facade/pin 및 result-shape 진단 수정은 main에 통합됐다. Clean source867의
  CI1514/1514와 60초 fuzz PASS가 있으며 tracked code/tool input delta는 없다.
- 소비자 **CHECK canonical counting/encoding work admission** 수리가 main에 반영됐다.
  Foundation37/37, Native Match23/23와 producer 로컬 CI1514/1514 PASS다.
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

관측일: **2026-10-04, Asia/Seoul**. 이 갱신은 최신 코드의 병렬 감사와 계획이다.
production 소스, commit, 제품 build는 변경하지 않았다. 아래 현재 판정이 이전 이력보다 우선한다.

## 0. 최신 재감사 — 20:50~20:54 KST

이 절이 아래 b34 및 이전 소스의 판정보다 우선한다. 이번 요청은 최신 코드 감사와 계획이며,
새 production 수정·Rust build·전체 CI·제품 실행은 하지 않았다. 기존 receipt는 원본 source에만 적용한다.

### 소스 기준

| 대상 | 고정한 상태 | 판정 |
| --- | --- | --- |
| 공유 producer | `293bbba5adce471ba456d10f71049718b41dc64b` + dirty | 20:54:21 KST 최종 바이트 capture. owner 작업이 감사 중에도 변경됨 |
| 격리 producer | `a97bd3f5c1746c33a7baabd448119a90debd321e`, clean | 수정 후보. 크기·architecture PASS, 최신 clean qualification exit 101 |
| 공유 consumer | `bfbe6ba4eb144a3510b79b27e4c6676ba13a0534` + dirty | 관련 7파일 capture 전후 동일. 전체 checkout은 실행 증명 대상이 아님 |
| 소유 consumer tuple | consumer `9efc0af1843dc4ed6665e3978210f260c9fd0c4d` + sink fixture/pin 2파일 dirty; producer `49f8735801b63872de9188886385756d3cc2975b`; index `bd65e2f166d87b0eb3fb706a8f3c127a47ec4ecf` | producer sibling·gate pin·a97이 서로 다름. 현재 통합 PASS 없음 |

[producer 소스 capture](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/latest-user-audit-20261004T115043Z/source-state.json),
[감사 마감의 main 바이트](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/latest-user-audit-20261004T115043Z/main-at-audit-close.json),
[consumer 소스 capture](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/latest-user-audit-20261004T115043Z/consumer-source-state.json),
[a97 실패 receipt 사본](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/latest-user-audit-20261004T115043Z/a97-receipt.json).
격리 producer는 병렬 감사 시작·끝 clean이고 관련 바이트도 같았다. 공유 producer 전체는 변경 중이므로
위 마감 snapshot 밖의 후속 변경까지 감사했다고 해석하지 않는다.

### 실제 결함과 검증 공백

1. **공유 main의 production dependency 결함.** pipeline `compile.rs:858,868,883,993`은
   Modules를 일반 코드에서 참조하지만 manifest의 Modules는 dev-dependency다.
   일반 build의 unresolved crate가 예상되는 정적 결함이며 이번 요청에서 compiler 재현은 하지 않았다.
   architecture rail은 불허 `pipeline → HIR` normal edge로 FAIL(27/26).
   a97은 sema opaque session을 통해 Modules owner로 연결하며 허용 normal edges 26/26 PASS다.
2. **공유 main의 nested EXISTS scratch 과소 예약.** 하나의 전체 census stack 예약 아래
   재진입마다 추가 Vec가 동시에 생존한다. a97의 `document/depth.rs:24-75`는 각 생존 stack을
   할당 전에 예약하고 capacity를 검증한다. 독립 exact/short/foreign/drop 회귀와 이전 RED/GREEN 기록은 있다.
   엄격한 예산에서 기존 성공이 typed refusal로 바뀌는 것이 의도된 수정이다.
3. **a97의 current Clippy 실패.** `modules/src/budget.rs:61`의
   `checked_add(...).unwrap_or(usize::MAX)`가 `manual_saturating_arithmetic`에 걸렸다.
   의미가 같은 `saturating_add`로 고칠 수 있다. 도구 버전 차이로 owner Rust 1.81 Clippy PASS가
   locked current 1.95 PASS를 대신하지 못했다. fmt·taplo·size까지 PASS, 뒤의 전체 테스트·정책은 실행되지 않았다.
4. **producer/consumer pin 불일치.** a97의 frontend/pipeline subtree는
   `18be97e09ac2f406b2edc0f5d80b4bf0fa260bd8` / `e23e38be5a3c6037246c45252b3197b056f48f65`다.
   공유 gate는 `1a8c0e9…/8a085b9…`, frozen dirty gate는 b34의 `2ee22bd…/6106bb4…`를 기대한다.
   최종 source가 정해진 뒤 계산해 갱신한다. Vector SHA `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb`와 schema **3/8/8**은 일치한다.

공유 main과 a97의 크기는 frozen 바이트에 Tokei를 적용해 다시 측정했다.
[파일별 hash·측정](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/latest-user-audit-20261004T115043Z/size-six-files.json).

| production 파일 | 공유 main code LOC | a97 code LOC |
| --- | ---: | ---: |
| `modules/src/catalog.rs` | 1,278 | 1,132 |
| `modules/src/graph.rs` | 1,210 | 1,126 |
| `pipeline/src/compile.rs` | 1,244 | 1,065 |
| `sema/src/document.rs` | 1,677 | 574 |
| `surface/src/family.rs` | 1,290 | 1,044 |
| `surface/src/parser.rs` | 1,216 | 872 |

공유 main은 위 6파일로 size rail FAIL, a97은 250파일 PASS다. production/test 한도는 1,200/2,500,
함수 한도는 200이고 파일 예외는 0개다. 한도는 저장소 운영 정책이지 업계 표준 수치가 아니다.

### 최종 액션 리스트

| 순서 | 작업과 목적 | 완료 조건 |
| --- | --- | --- |
| 1 / P0 통합 기준 | a97의 수리와 최신 owner 변경을 하나의 검증 후보로 합침 | 20:36 통합 기준·a97·현재 main의 3-way 비교. 후속 Arc 공유/정렬 Vec/이동 및 새 reference 선예약 작업 보존. manifest/facade는 단일 writer. 미완성 owner API를 제품 연결 완료로 취급하지 않음 |
| 2 / P0 경계·P1 예산 | dependency 방향과 실제 scratch 생존량 보장 | pipeline 일반 build, architecture 26 normal edges, nested EXISTS exact/1-byte-short/foreign/마지막 drop=0. 새 Arc layout 청구까지 독립 bytes oracle에 반영 |
| 3 / P1 구조·lint | 크기 재성장과 도구 차이로 인한 누락 차단 | current Clippy 산술 수정, production 파일 1,200·함수 200 준수, 새 예외 0. 분할 전후 진단 위치/순서·canonical ID·DTO/digest 동일. existing AST/test guard 유지 |
| 4 / P0 계약 | facade·fixture·최종 pin의 일치 | public roots 145 및 negative probes 18에 대한 실제 외부 gate PASS. opaque payload 접근/임의 map 거절. 최종 producer subtree/vector pin 갱신, 동일 source의 sink exact/short/foreign/drop owner 및 M23 gate PASS |
| 5 / 로컬 완료 | 최종 committed source의 전체 검증 | 소유 변경만 clean commit → `just qualify-local`로 내부 `just ci` 1회, exit 0·dirty=false·Nextest selected=executed·skipped/ignored 0. 실패 source 기록 보존. parser/storage 변경은 별도 bounded local `just fuzz-smoke` |
| 6 / 소비자 호환 | 실제 소비자가 같은 의미로 받아들이는지 검증 | 하나의 producer/consumer/index tuple에서 QBC owner로 Graph 지원/거절 및 EXISTS 두 0행 결과. 해당 source 변경에 영향받은 PTA/Stage5/task custody owner만 추가. 다른 tuple의 과거 PASS를 합산하지 않음 |
| 7 / 제품 qualification | public source→admission→SDK/daemon 지원 주장 확정 | serving/release 완료를 주장할 때 위 tuple의 immutable bundle·4개 binary hash로 installed Graph, macOS CHECK typed refusal, Linux CHECK 0/1/multi 및 관련 거절 증명. frontend 정리의 PASS와 별도 기록 |
| 8 / P2 문서 | 검증 범위를 소스와 일치시킴 | qualification README의 negative probe 15→18 정합성 수정. test-only digest oracle 등록은 AST guard를 유지하며 검토; production 기준을 더 엄격히 적용하는 현 상태는 우회 결함이 아님 |

**병렬 배치:** A는 surface/HIR/binder, B는 sema/Modules, C는 types/algebra/lint owner를 맡는다.
pipeline·facade·manifest·pin·최종 source freeze는 통합 담당이 직렬로 처리한다.
최종 owner delta가 확정되기 전에 a97 전체 CI를 통과시켜도 후속 소스의 증명이 아니므로,
affected owner 확인 후 최종 후보에서 전체 rail을 실행한다.

### 재작업하지 않을 항목과 적용할 패턴

- Graph cap=1, Reachability=`Endpoints`, Path=`NodeSequence`는 현재 일치한다.
  EXISTS의 nonempty 입력에서 false conjunct 두 종류를 0행으로 확인하는 회귀도 이미 있다.
- facade는 a97에서도 `FrontendRetainedStoragePermitV3`를 공개한다. 이전 b34의 import 불일치는 현재 a97 결함이 아니다.
- request sink는 consumer의 테스트에서만 생성된다. 일반 compile/serving 경로의 전체 frontend memory custody는 아직 지원 범위 밖이며,
  현재 계약도 partial이라고 명시한다. 확장은 별도 제품 요구가 필요하다.
- authored result-shape 전부와 planned SHOW one-shot은 typed refusal이다. 이를 기능 구현 작업으로 자동 확대하지 않는다.
- conformance는 19 family다. `family.rs`의 18은 그 모듈이 처리하는 범위이고 `Select/from`은 parser가 처리하므로 해당 주석은 stale이 아니다.
- a97 integration 상위 5파일 비중은 19.2%다. 과거 35% 수치로 추가 test 분할을 요구하지 않는다.
- production AST의 `unwrap` 0, `Option::expect` 1(`manifest.rs:567`), `unreachable!` 8은 단순 검색의 591과 다르다.
  `Cursor::expect`는 typed parser diagnostic 메서드다. 8 invariant 분기의 실제 입력 도달성은 이번 감사로 확정하지 않았다.
- local static/ci에는 이미 current 및 MSRV Clippy가 있다. bounded fuzz smoke는 별도 local rail이다.

도구는 기존 Clippy·Tokei·pinned tree-sitter AST·architecture policy를 재사용한다. 새 production dependency는 필요하지 않다.
함수 책임 추출은 [Clippy](https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#too_many_lines),
public 결과 회귀는 [DataFusion](https://datafusion.apache.org/contributor-guide/testing.html#choosing-what-kind-of-test-to-write),
명시적 지원 subset과 producer/consumer 호환은 [Substrait](https://substrait.io/spec/technology_principles/) 공식 패턴을 참고한다.
이 선택은 현재 결함과 운영 비용에 대한 판단이며 특정 도구의 SOTA 순위 주장으로 해석하지 않는다.

아래는 이전 시점의 상세 기록이다.

## 1. 감사 소스와 증거 범위

| 대상 | 확인한 소스 | 현재 상태 |
| --- | --- | --- |
| gqlang 공유 main | `293bbba5adce471ba456d10f71049718b41dc64b` + dirty | 20:31:10 KST에 바이트 스냅샷 고정. 크기 초과 6개, production 의존성 오류와 중첩 scratch 미차감 확인. 이후 binder·inline View 변경은 이 스냅샷 밖 |
| gqlang 격리 refactor | `b34b022db9b8d0ef823b69e45c3d06a104a68db8`, clean | 크기 게이트 247파일 PASS, 구조·scratch 수정 구현 있음. 전체 qualification은 rustfmt에서 exit 1 |
| 이전 clean qualification | `f8991c08002ffc2c10f06c9f0542a74ace4f3aaf` | `just qualify-local` exit 0, Nextest 1,439/1,439. 이후 새 API·단계 변경의 증명이 아님 |
| Semantica 공유 main | 감사 중 `63b4ceb5` → `5571668ddacd4da6cc320cb6da8305793e824743` + dirty | 확인한 Graph/EXISTS/fixture/pin 파일은 두 HEAD 사이 변경 없음. 전체 현재 실행 증명 없음 |
| 소유 consumer proof root | consumer `9efc0af1843dc4ed6665e3978210f260c9fd0c4d` + fixture/pin 2파일 dirty; producer `49f8735801b63872de9188886385756d3cc2975b`; index `bd65e2f166d87b0eb3fb706a8f3c127a47ec4ecf` | 디렉터리 이름의 `baf/293`는 현재 소스가 아님. b34 pin 변경과 producer sibling이 불일치하므로 동일 튜플 qualification으로 사용할 수 없음 |

[불변 소스·크기 검사·capture 전후 hash](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/read-only-current-audit-20261004T113000Z/original-frozen-custody.json),
[격리 스냅샷](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/read-only-current-audit-20261004T113000Z/isolated-frozen-custody.json),
[저장소 상태·dirty hash](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/read-only-current-audit-20261004T112825Z/root-source-basis.json).
원본 manifest SHA-256은 `7dda25d13d322b3f33d8127018873f3e77a91854aed3b0cd759a48ac7612a5c5`,
격리 manifest는 `efa23c95bd2e6c3a926ec4421af358928bce8180af6752d1c58a54aebd802e4d`다.
복사 전후 원본 바이트가 일치했고 frozen 소스에서 크기 결과를 다시 확인했다.
공유 main은 계속 변경 중이므로 이후 작업은 새 바이트를 다시 대조해야 한다.
감사 마감에 sema의 inline View lookup이 BTreeMap→정렬 Vec로 변경되는 중임을 확인했다.
그 후속 바이트는 크기·행 번호 스냅샷 밖이며, 통합 전 조회·중복 진단 순서의 회귀를 함께 검토한다.

## 2. 현재 해결할 문제

### A. P0 — production pipeline의 Modules 참조와 경계

공유 main [compile.rs:858](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-pipeline/src/compile.rs:858)은
Modules validator와 predeclare 오류를 일반 코드에서 직접 참조하지만,
[Cargo.toml:25](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-pipeline/Cargo.toml:25)의 Modules는 dev-dependency다.
일반 빌드에서 unresolved crate가 예상된다. **정적 확인이며 이번 감사에서 컴파일 재현은 하지 않았다.**
`pipeline -> hir` normal edge도 현행 architecture 정책 밖이다.

격리본은 sema의 opaque `ResolvingDocumentSessionV3`로 validator를 운반한다.
그러나 공유 main에 이후 추가된 `PredeclaredRequestSourceV3`, authority 수명 결합,
catalog 입력 수집/attach/predeclare 단계는 b34에 없다. b34 파일을 덮어쓰면 이 작업이 유실된다.
신규 predeclare도 sema helper를 통해 Modules owner에 연결하고 기존 dependency 방향을 유지한다.

### B. P1 — 중첩 EXISTS scratch의 실제 생존량

공유 main [document.rs:437](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-sema/src/document.rs:437)은
전체 census 크기의 stack 하나를 예약한다. [frozen visit_relation:1074](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/read-only-current-audit-20261004T113000Z/frozen/original/crates/quanta-gqlang-v2-sema/src/document.rs:1074)은
Filter/Join predicate 안의 EXISTS에 재진입할 때 또 같은 크기의 Vec를 할당한다.
바깥 Vec가 살아 있으므로 단일 예약은 실제 동시 scratch를 커버하지 못한다.

격리 [depth.rs:31](/Users/songmin/.codex/worktrees/frontend-current-qualification/quanta-gqlang-v2/crates/quanta-gqlang-v2-sema/src/document/depth.rs:31)은
각 생존 stack의 permit을 할당 전에 예약하고 실제 capacity를 확인한다.
[기존 RED/GREEN 증거](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/nested-scratch-red-green-20261004/result.json)는
중첩 예약을 생략한 mutant에서 두 회귀가 실패하고 복원 후 통과함을 기록한다.
이번에는 해당 테스트를 재실행하지 않았다. 통합 시 방문·LIFO·첫 진단 순서를 보존한다.
엄격한 예산에서는 이전의 과소 차감 성공이 typed refusal로 바뀔 수 있다.

### C. P1 — 새 단계로 다시 커진 파일과 함수

Tokei **code LOC** 기준이며 물리 줄 수와 다르다. production 1,200/test 2,500, 예외 0개 정책이다.

| 공유 main 파일 | code LOC | 분리할 책임 |
| --- | ---: | --- |
| `modules/src/catalog.rs` | 1,283 | catalog predeclaration/session bookkeeping |
| `modules/src/graph.rs` | 1,210 | request-owned graph scratch와 기존 topology |
| `pipeline/src/compile.rs` | 1,244 | opaque request stage orchestrator와 기존 closed compile |
| `sema/src/document.rs` | 1,622 | declaration/depth/inline View/session/catalog inputs |
| `surface/src/family.rs` | 1,290 | presentation/composition family |
| `surface/src/parser.rs` | 1,216 | parser entry와 request-owner tests |

격리본은 247파일 PASS이며 기존 production 장함수 예외 17개를 제거했다.
Clippy 함수 기준은 200줄이다. 새 AST gate는 실제 test-only guard를 확인하고 production
`allow`/`expect(too_many_lines)` 및 broad lint 우회를 거절한다.
최신 owner 변경을 합친 결과에서도 검사해야 한다. 파일명만 test로 바꾸거나 예외를 늘려 통과시키지 않는다.
진단·source order·canonical ID·DTO/digest 보존이 분할의 동작 기준이다.

### D. P0 통합 — facade·fixture·pin은 아직 하나의 계약이 아님

공유 facade pin은 **145**, 격리는 **144**다. 차이는 retained permit의 공개 여부다.
공유 consumer [fixture:55](/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/frontend_request_storage_v3.rs:55)은
b34가 export하지 않는 `FrontendRetainedStoragePermitV3`를 import한다.
소유 frozen fixture는 opaque 반환값의 `size_of_val`로 수정했지만 아직 dirty이며 owner 테스트는 NOT_RUN이다.
최종 공개 범위를 선택하고 API oracle·문서·소비자를 같이 맞춘다. 이미 배포된 API라면 호환 또는 명시적 버전 변경이 필요하다.

공유 consumer의 closeout subtree pin은 `1a8c0e9…/8a085b9…`이며 b34의
`2ee22bdc2de40ae932a824d10391b2acee346737/6106bb4914d7bec9f41d5baa408fc11780cc43c1`과 다르다.
Vector SHA-256 `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb`와 schema **3/8/8**은 동일하다.
이를 새 digest 결함으로 재등록하지 않는다. 최종 producer commit 이후 해당 subtree pin을 계산한다.

새 request parse/check/declaration/predeclare 체인은 **staging**이다.
일반 [compile_source_document_v3:1044](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-pipeline/src/compile.rs:1044)는
기존 parse→integrity→sema→IR 경로를 쓴다. 전체 frontend 메모리 custody의 생산 연결 완료로 주장할 수 없다.

### E. 검증 공백 — 최신 qualification은 실패

[b34 receipt](/Users/songmin/.codex/worktrees/frontend-current-qualification/quanta-gqlang-v2/target/qualification/b34b022db9b8d0ef823b69e45c3d06a104a68db8/receipt.json)는
clean source에서 exit **1**이다. `sema/src/lib.rs:53,76` rustfmt 순서로 중단돼 새 전체 테스트·정책 결과가 없다.
기존 집중 테스트·정적 PASS와 f899의 전체 CI PASS를 새 통합 source의 qualification으로 합치지 않는다.
consumer Graph/EXISTS/PTA/Stage5 및 설치 Mac/Linux CHECK의 기존 영수증도 이전 튜플이다.

## 3. 최종 액션 리스트

| 순서 | 작업·목적 | 완료 조건 |
| --- | --- | --- |
| 1 / P0 소스 통합 | 최신 owner overlay와 b34를 바이트 기준으로 통합. 새 predeclare 작업과 기존 수리를 함께 보존 | 새 snapshot 및 변경 소유 경로 확정. base/격리/최신 main 3-way 검토; 공개 request 단계·authority 수명·source 순서 보존. shared main의 무관한 변경 보존 |
| 2 / P0 빌드·P1 정확성 | sema-owned resolution/predeclare 경계와 중첩 stack 예약 수정 통합 | pipeline normal build PASS, 허용 normal edges 26/26. 중첩 EXISTS 독립 2-stack bytes/layout oracle, exact/1-byte-short/foreign-source/foreign-root/마지막 drop=0. 신규 catalog predeclare 정상·예산 부족·foreign 거절 owner 회귀 PASS |
| 3 / P1 구조 | 6파일 초과와 17 장함수 예외의 최신 통합 결과를 책임별 분해 | production/test 1,200/2,500 PASS, 생산 함수 200줄 Clippy PASS, 새 예외 0. parser 테스트 실제 cfg(test), module/impl/cfg_attr/macro 우회 거절. 불필요한 View payload clone 제거·DTO/digest·진단 동등성 |
| 4 / P0 계약 | 최종 facade API·외부 fixture·소비자 import를 일치시킴 | 실제 exporter pin과 positive/negative privacy oracle 일치. partial staging 범위 문서화. 같은 producer에 fixture exact/short/foreign/last-drop 회귀 PASS. 새로운 runtime authority를 임의로 발행하지 않음 |
| 5 / 로컬 마감 | source-bound 전체 검증 | affected owner 집중 테스트 후 rustfmt 수정·소유 경로만 commit. clean 최종 HEAD의 `just qualify-local`이 내부 `just ci`를 **1회** 실행해 exit 0·dirty=false. Nextest 선택/실행 수 일치, skipped/ignored 0. 새 bounded `just fuzz-smoke`는 별도 로컬 기록 |
| 6 / 소비자 owner | producer 변경이 실제 실행 계약과 호환됨을 확인 | 최종 producer+reviewed committed consumer+실제 index path dependency 튜플 고정. subtree/vector pin 계산 및 closeout static 검증. QBC owner로 Graph 지원/거절, EXISTS 0행 두 결과, 변경된 PTA/Stage5/task-custody 범위 PASS. 수정과 receipt를 동일 튜플로 묶음 |
| 7 / 제품 완료 | 실제 public source→admission→SDK/daemon 실행 증명 | 위 owner proof 이후 같은 소스·4바이너리 hash·immutable bundle. Graph witness/empty/budget/foreign/stale, macOS CHECK typed refusal, Linux CHECK 0/1/multi·거절. native Linux/aarch64 결과를 x86_64 qualification으로 해석하지 않음 |

전체 CI를 직접 실행한 뒤 qualification에서 다시 실행하는 중복 단계는 두지 않는다.
실패 후 구체적 수정으로 source가 바뀌면 최종 새 source의 필수 gate를 다시 실행하고 실패 기록도 보존한다.
설치 build는 변경 owner와 소비자 호환이 확인된 뒤 진행한다.

## 4. 병렬 작업과 도구 선택

소스/API snapshot 확정은 통합 담당이 먼저 수행한다. 이후 독립 owner만 병렬로 작업한다.

- **A / surface·HIR:** parser/family 분리와 opaque parse/check·source identity·retained/drop 회귀.
- **B / sema·Modules:** depth scratch·declaration/session/catalog predeclare를 sema 경계에 연결하고 독립 예산/foreign 회귀. root manifest·facade·pipeline API 편집은 통합 담당과 직렬로 조정.
- **C / types·algebra·lint:** 기존 helper refactor의 의미 회귀와 AST/test-module gate; owner 파일 교차 편집 금지.
- **통합 담당:** pipeline/facade/manifest/public docs, source drift 재검토, focused proof 취합, clean commit과 마지막 로컬 qualification. 다음 소비자 snapshot·QBC는 producer 계약 확정 후 착수.

현재 스윗스팟은 **기존 Clippy + Tokei + tree-sitter AST + architecture 정책 + 결과 기반 회귀**다.
기존 pinned `tree-sitter-language-pack==0.9.1`, `tree-sitter==0.23.2`를 재사용한다.
추가 production dependency는 0개다. 1,200/2,500/200은 이 저장소의 운영 기준이며 업계 표준 수치라는 주장은 하지 않는다.
[Clippy의 함수 분할 안내](https://rust-lang.github.io/rust-clippy/master/index.html#too_many_lines),
[DataFusion의 작은 affected 테스트와 사용자 결과 회귀 원칙](https://datafusion.apache.org/contributor-guide/testing.html),
[Substrait의 명시적 subset·producer–consumer 호환 원칙](https://substrait.io/spec/technology_principles/)을 적용한다.

CI는 로컬로 운영한다. 공유 `.circleci/`와 신규 code-query RFC는 별도 owner 변경으로 보존한다.
이 계획의 크기·계약 수리는 엔진 교체나 V4 typed join/재귀/증분 DSL 확장을 요구하지 않는다.
그 기능 설계는 기존 RFC의 별도 완료 조건을 따른다.

## 5. 이미 해결됐거나 별도 기능인 항목

- 과거 document 약 8k/execution 약 5k는 현재 수치가 아니다. frozen main의 물리 줄은 각각 1,766/1,375이며 격리 document는 643줄이다.
- integration 상위 5파일은 현재 **18.9%**다. 큰 테스트의 과거 35% 주장으로 추가 분할을 요구하지 않는다.
- `clippy-msrv`는 이미 local `static`/`ci`에 포함된다. local fuzz smoke는 정규 CI 밖의 bounded 보조 rail이다.
- conformance는 Rename 포함 **19 family**다. 과거 parser 부재·root RED 주석은 현재 없다.
- Graph cap=1, Reachability=`Endpoints`/Path=`NodeSequence`는 일치한다. EXISTS 두 false-conjunct의 nonempty 입력/0행 회귀도 존재한다.
- authored `result_shape`는 leaf·record·nested 모두 typed reject이고 planned SHOW one-shot admission도 typed refusal이다. 지원 확장은 실제 소비자 요구와 별도 계약을 확정한 뒤 진행한다.

이번 감사에서 실행한 것은 source/status/hash·불변 snapshot 비교·Tokei/AST 및 policy fixture의 정적/작은 Python 검사다.
새 Rust build/test, `just ci`, fuzz, 제품 설치 실행은 **NOT_RUN**이다. 이 문서 갱신의 검증은 문서 diff/링크·whitespace에 한정한다.

## 이전 구현·감사 이력

### f899 qualification 이전의 재감사 기록

아래 내용은 이전 관측 소스에 관한 기록이며 현재 판정은 위 1~4절이다.

관측일: 2026-10-04, Asia/Seoul. 다음은 f899 qualification 이전 기록이다.
`293bbba`의 clean qualification은 완료됐지만 이후 공유 작업트리 변경을 포함하지 않는다.

### 이전 재감사: 구현 재개 전의 관측

이번 요청에서는 production 코드를 수정하지 않았다. 진행 중이던 consumer 빌드의
결과를 보존하고 새 빌드를 시작하지 않았다. 공유 main은 계속 변경되므로 아래 SHA와
파일 hash가 감사 범위를 정의한다.

| 대상 | 재감사 snapshot | 판정 |
| --- | --- | --- |
| gqlang | `293bbba5adce471ba456d10f71049718b41dc64b` + Rust 11파일 dirty | request-root parser/storage 공개 API가 추가되는 중. 기존 clean receipt 범위 밖 |
| Semantica | `6d2921b086f6feb6627dd3ee27554b53afcd326f` + 별도 dirty | 새 frontend API를 import하는 consumer 코드가 이미 committed. 이 HEAD의 실행 proof 없음 |
| Index | `538cd805f1def5aebf1cebcebbec8bd8c65b8eff` + 별도 dirty | transitive dependency. 다음 consumer proof에서 함께 source 고정 필요 |

관측 당시 `just size-lint`는 238개 Rust 파일을 검사해 PASS, `git diff --check`도
PASS였다. 두 검사 전후 11개 변경 Rust 파일 hash는 같았다. 이는 동작/API 호환성
검증이 아니다. [최신 manifest와 검사 결과](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/latest-current-audit.json)를 보존했다.

### 확인된 결함과 증거 공백

1. **P0 / committed producer–consumer 재현성:** Semantica의
   `frontend_request_storage_v3.rs`는 `FrontendRequestStorage*V3`를 import하지만,
   gqlang HEAD에는 이 이름이 없고 dirty overlay에만 있다. 해당 admission feature를
   사용하는 두 committed HEAD 조합은 재현되지 않는다. 새 공개 이름 7개는 기존
   `PINNED_ALLOWLIST`에도 없다. 계약 검토 후 API oracle·문서·소유 commit·consumer pin을
   함께 갱신해야 한다. 현재 dirty compile 성공만으로 이를 완료 처리할 수 없다.
2. **P0 / Linux compile:** `quanta-repo-identity-fs/src/pinned_source_v2.rs:189`가
   mutable reference `stack_v2`를 이동한 뒤 193행에서 다시 사용한다. frozen `baf`
   Linux/aarch64 제품 빌드는 E0382로 exit 101이며 현재 main에도 동일한 코드가 있다.
   최소 수정은 `readlinkat_raw(..., &mut *stack_v2)` 재대여다.
3. **P1 / 새 메모리 owner의 검증 범위:** parser/cursor는 source-derived 슬롯을
   예약한 뒤 할당한다. 그러나 projection/key/path-segment 등 일부 nested HIR와
   diagnostics/sema/planning은 아직 이 API의 예약 범위 밖이다. 코드가 partial이라고
   명시하고 있으므로 전체 frontend 메모리 admission 완료로 해석하면 안 된다.
   retained permit의 linked chain에는 기본 재귀 drop이 있으므로 긴 체인·실패 중
   해제의 stack 사용도 검증 대상이다. stack overflow는 이번 감사에서 재현하지 않았다.
4. **P1 / 실제 실행 proof:** Graph cap/dedup 수정, EXISTS 두 false-conjunct oracle,
   Python distinct-assignment admission/index, Stage5와 CHECK 연결은 코드에 있다.
   현재 main의 owner 및 installed process PASS는 없다. 예전 `baf` Python E0433와
   Mac taint E0624는 현재 main에서 수정돼 있으므로 현재 결함으로 재등록하지 않는다.
   Stage5는 E0624 확인 후 중단되어 exit 130이며 통과 증거가 아니다.

### 다음 실행 계획

| 순서 | 작업 / 목적 | 완료 조건 |
| --- | --- | --- |
| 1 / P0 | 새 frontend API와 consumer의 저장소 경계 정리. committed source만으로 재현 가능하게 함 | 소유자의 in-progress 변경 확정; public API 7개와 partial admission 계약 검토; facade positive/negative oracle와 계약 문서 동기화; 소유 경로만 commit; 정확한 consumer subtree pin 및 3개 저장소 source 고정 |
| 2 / P0 | Linux reborrow 한 줄 수정. platform compile 차단 해소 | owner-local Linux QBC와 제품 build exit 0. source/path/relink/deleted refusal 의미 유지 |
| 3 / P1 | request-root parser/storage proof. 예산·authority·해제 보장 | exact/1-byte-short/foreign-source/foreign-root/last-drop oracle; 긴 permit chain의 bounded 해제; raw/admitted parse의 유효 입력 HIR 비교. 작은 예산 malformed 입력의 오류 우선순위는 명시적 계약으로 고정 |
| 4 / P1 | Python·Stage5·EXISTS owner proof. 최근 연결 수정의 의미 검증 | 동일 source tuple에서 Python positive+negative 전체 모듈, Stage5 owner, EXISTS 두 0행 결과 QBC PASS |
| 5 / P1 | 설치된 Graph·CHECK 실행 proof. compiler→host→SDK 결과 보장 | 동일 tuple의 4개 바이너리·immutable bundle; Graph witness/empty/budget/stale/foreign, macOS CHECK typed refusal, Linux CHECK 0/1/multi와 거절 PASS |
| 6 / 마감 | 로컬 frontend CI와 source-bound 증거 정리 | 새 frontend commit에서 `just qualify-local`로 `just ci` 1회; consumer owner/product 기록과 source/artifact/platform 기록; 미실행 항목을 PASS로 처리하지 않음 |

1과 2는 서로 다른 소유 파일에서 병렬 가능하다. 최종 source 고정 후 3과 4,
Linux/Mac build는 각자 lane에서 병렬 수행한다. 5는 해당 플랫폼 product build 뒤에
수행한다. 같은 physical source root/lane의 build 중에는 source를 교체하지 않는다.

완료된 크기 분할, legacy exception 8개 제거, 로컬 `clippy-msrv` 편입,
result-shape 진단 수정, Rename conformance를 다시 backlog로 등록하지 않는다.
CI는 로컬이며 기존 `.circleci/` dirty 파일은 별도 소유 작업이다. 새 엔진이나
외부 production 라이브러리 도입을 요구하는 결함은 이번 재감사에서 확인하지 않았다.



아래 `6be`/`baf` 등의 source tuple과 진행 상태는 이전 관측 기록이다.
최신 완료/미완료 판정에는 위 재감사와 해당 source-bound receipt를 사용한다.

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
  이 수정은 `293bbba`로 commit했다. 해당 clean managed checkout에서
  `just qualify-local`의 전체 CI가 통과했다: Nextest 1,433/1,433(누락·skip 0),
  doctest 8, frontdoor 10, public API 135/negative probe 15 및 모든 정책 gate PASS.
  receipt의 `dirty=false`와 raw log digest를 확인했다.
- 같은 `cc27ff2`의 bounded offline fuzz 60초는 성공했다. corpus 1,240개,
  crash 0개이며 전후 source/HEAD/clean 상태가 일치했다. `cc27ff2`와 `293bbba`
  사이 Rust crate 파일 264개는 동일하므로 이 fuzz 결과의 코드 범위는 유지된다.
- B/C/E의 실제 실행 증명은 후속 결과로 판정한다. 후속 증거는
  [remediation evidence](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-commit-source.json),
  [fuzz 결과](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/fuzz-smoke-result.json),
  [현재 상태](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/remediation-status.json),
  [clean qualification receipt](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/ci-final-qualified/receipt.json)에 보존한다.

목적은 현재 지원하는 gqlang 계약이 실제 소비자 실행까지 유지되는지 확인하고,
큰 함수의 변경 범위를 줄이는 것이다. 최초 감사 단계는 읽기 전용이었고,
후속 구현·검증은 위 기록으로 구분한다. CI는 로컬에서 운영한다.

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

| 순서 / 우선순위 | 작업과 목적 | 완료 조건 | 후속 상태 |
| --- | --- | --- | --- |
| A / P0 | **소유 변경 commit·pin·검증 소스 확정.** 동일한 producer/consumer 조합을 검사할 수 있게 함 | 검토한 producer 변경을 commit → 정확한 subtree/vector pin 확인 → 현재 consumer와 모든 관련 path dependency 동결. 기존 dirty refusal 유지 | 완료: producer `293bbba`, consumer `6be152b`, index `bd65e2f`. 다른 shared-main overlay는 제외 |
| B / P0 | **최신 소스의 installed Graph 증명.** 실제 Path/Reachability 결과와 authority 거절 확인 | 영향받은 Stage5/admission owner 확인 후 새 product artifact와 process test에서 정확한 1개 witness, 빈 결과, 예산 고갈, foreign/stale source 거절 확인 | owner 및 설치 증명 진행 중 |
| C / P1 | **Python assignment production 재현.** 실제 결함 여부와 owning issuer 구분 | 실제 generation에서 assignment가 정확한 source-issued event로 성공하는지 확인. 실패면 issuer owner 수정 및 독립 회귀 | distinct assignment event의 admission 연결 수정, 함수별 index 적용. 전체 owner 검증 진행 중 |
| D / P1 | **`resolve_document_bodies` 책임 분해.** 한 문법 변경의 영향 범위 축소 | typed phase helper 추출, 기존 진단/IR/digest/실패 순서 유지, 함수 크기 `expect` 제거. 해당 owner/conformance 및 마지막 로컬 `just ci` 통과 | 완료·qualification PASS |
| E / P1 | **CHECK 플랫폼별 실행 증명.** 연결된 코드의 지원 범위 확정 | Linux signed catalog 0/1/multi 성공과 relevant 거절, macOS unsupported worker의 typed 거절. 소스/플랫폼/artifact 기록 | 양 플랫폼 product build 및 설치 증명 진행 중 |
| F / P2 | **남은 8개 파일 크기 예외 축소.** 책임 경계를 개선하며 재성장 방지 | 각 owner를 수정할 때 책임별 추출 후 정확한 ceiling을 낮추거나 예외 제거. 예외 추가/상향으로 통과시키지 않음 | 8개 전부 제거·qualification PASS |
| G / P2 | **result-shape 진단 정확화.** 지원 subset과 오류 설명 일치 | leaf/record/list 모두 거절함을 정확히 설명. error code/span 및 typed failure 유지 | 완료·qualification PASS |

A → B는 순차로 수행한다. C와 D는 서로 다른 저장소/owner이므로 독립적으로
진행할 수 있다. E의 구현·test는 현재 consumer 소스에 있으므로 소유 상태를
재확인하고 기존 구현을 검증한다. 같은 QBC lane의 build/test는 동시에 실행하지 않는다.

### A. commit과 pin: 최초 차단 조건과 해결

[closeout gate](/Users/songmin/Documents/code-new/semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh:938)는
frontend/pipeline dirty 상태를 거절한 뒤 **committed subtree**를 비교한다.

| 입력 | consumer 기대값 | 최초 감사 `9060e469`의 committed 값 |
| --- | --- | --- |
| frontend-contract tree | `bd1e06e9ed26a70304aa4cfdc3838436b17d74a6` | `026f736f17820acc0f92028a83aee8d8fc28decb` |
| pipeline tree | `c459d923288e04c575ea4027438465604a4ed07c` | `d9507023127a0af247a5531d6fe1dd575f1b4f82` |
| vector SHA-256 | `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb` | working file 일치 |

이 차단은 `cc27ff2`의 소유 변경 commit으로 해결했다. 현재 `293bbba`의 두 subtree와
vector는 consumer 기대값과 일치하며 M23-120 정적 gate도 PASS다. golden 재생성이나
schema 변경은 필요하지 않았다. 후속 source tuple은 consumer `6be152b`, producer
`293bbba`, index `bd65e2f`로 고정했다.

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

### F/G. 정책과 진단 — 구현 완료

- 현재 파일 정책: production/test 1,200/2,500 code LOC, Clippy 함수 200줄.
  후속 구현에서 legacy 예외 8개를 제거했다. 현재
  [size policy](/Users/songmin/Documents/code-new/quanta-gqlang-v2/tools/lint/size_policy.json)의
  `exceptions=[]`이며 238개 Rust source가 통과한다.
- `static`에 current/MSRV Clippy, `policy`에 architecture gate가 포함된다.
  새 `allow`나 예외 추가로 분할을 우회하지 않는다.
- parser/formatter 변경 및 release 전에는 기존 `just fuzz-smoke`를 사용한다.
  기본 fuzz 시간은 60초, 허용 범위는 1–300초다. source/seed/corpus/crash를 보존한다.
  fuzz는 현재 `ci`/`qualify-local`의 자동 leaf가 아니다.
- [result-shape 거절](/Users/songmin/Documents/code-new/quanta-gqlang-v2/crates/quanta-gqlang-v2-sema/src/document/query_materialization.rs:31)의
  메시지를 전체 authored shape 거절에 맞게 수정했다. scalar/record/nested/Composition
  회귀가 정확한 메시지를 검사한다. 기능 지원 확대는 별도 요구가 있을 때 결정한다.

## 4. 최초 감사의 검증 결과 — 후속 구현 이전 이력

| 체크 | 결과 / 의미 |
| --- | --- |
| `python3 -m tools.lint.size_gate` | PASS, Rust source 파일 225개. 현재 파일 크기 정책 준수 |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m tools.architecture.check_architecture` | PASS, workspace 14개 / normal 26·dev 5·build 0 edges. manifest 의존 방향 검사 |
| frozen producer 대비 crate byte comparison | 251개 동일. 과거 검증과 현재 crate 소스의 일치 |
| vector digest 비교 | 일치. 원본 committed subtree 정합성과는 별개 |
| Graph QBC owner resolve | PASS. selector/feature topology 확인, build/test 아님 |
| selected owner source 재확인 | 31개 변화 없음. 전체 consumer의 source freeze 아님 |
| Rust tests/build, `just ci`, fuzz, `qualify-local`, installed Graph/CHECK, 전체 release qualification | 이번 턴 **NOT_RUN** |

위 표는 최초 읽기 전용 감사의 범위다. 후속 frontend clean qualification은
`293bbba`에서 완료했고, consumer owner/installed process 증명은 별도 source/artifact
tuple로 기록한다. 서로 다른 tuple의 성공 결과를 합쳐 통합 완료로 표시하지 않는다.

## 5. 완료 범위와 별도 기능

이번 액션 리스트는 기존 계약의 정합성, 실행 증명, 구조 분해 및 로컬 게이트에
한정한다. 신규 production 라이브러리 추가는 현재 필요한 근거가 없다.

authored result-shape 지원, specialized `admit_show_planned_source_v2`의 항상
Incomplete인 경로, 새 rule evaluator/물리 optimizer/SMT/증분 실행은 별도 기능이다.
실제 소비자 요구와 의미·예산·거절 계약을 정한 뒤 우선순위를 결정한다.
