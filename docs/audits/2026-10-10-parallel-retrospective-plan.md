# QGLang 최근 1주 변경: 병렬 적대 감사·보완 계획

상태: **실행 계획**. 감사·수정·재검증을 완료했다는 문서가 아니다.
입력: [2026-10-10 변경 인벤토리](2026-10-10-weekly-change-inventory.md)의 340개 touched path, 216커밋 및 계약·증거 경계.

## 목표와 고정 입력

- 기준: QGLang `main`의 `25fad76cf5b4510244157c2e009b4dc37aa958ff` (tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`), 비교 시작 `9060e46997dd9c62080a18bdfae06f8c162442e2`. 이 SHA의 로컬 `just ci` 영수증은 Nextest 1,529 selected/executed/passed, skip/leak 0이다. 이는 **frontend-local** 증거다.
- 현재 공유 checkout에는 별도 작성 중인 ADR/plan 문서의 수정·삭제가 있다. 각 감사자는 시작 시 HEAD, `git status`, 작업 경로 소유자를 기록하고 **고정 SHA 읽기 또는 별도 격리 checkout**에서 조사한다. 공유 미커밋 변경을 되돌리거나 자신의 완료 상태에 포함하지 않는다.
- 소비자: 공개 `quanta-gqlang-v2-frontend-contract` facade와 정확히 짝지어진 Semantica consumer. QGLang 소유는 syntax, Binder/Types/Sema, normalized IR, Planning, Pipeline/Execution 계약, CLI/diagnostics다. runtime admission, provider, SDK/backend 및 설치 제품은 downstream 소유다.
- 수락 기준: 기간 중 코드·설정 변경 **14커밋 전부**와 인벤토리의 **340경로 전부**를 아래 레인에 매핑한다. 각 동작 변화는 정상·거부·경계 입력, 독립 기대값, 결정성/자원/호환 영향으로 판정한다. 실행하지 않은 검증은 `NOT_RUN`; downstream 주장에는 별도 source tuple과 소비자 증거를 요구한다.
- 파일군 배정: `crates/`의 touched 179경로는 01–06(그중 `conformance` 테스트 이동은 07과 공동 검토), `tools/` 31경로와 CI/policy 3경로·root/기타 10경로는 07, `docs/` 117경로는 08이 완주한다. 공개 계약 문서는 06, Semantica handoff 문서는 09가 교차 검토한다. `T`로 표시된 최종 상태 동일 경로도 제외하지 않는다.

## 작업 방식: 9개 번호 레인, 최대 4개 동시 실행

분석은 병렬로 시작한다. 레인별 source 수정은 소유 파일이 겹치지 않을 때만 병렬로 허용한다. Cargo build/test는 각 레인의 격리 target을 쓰거나 admission을 직렬화한다. 통합자는 공통 파일과 최종 증거를 한 번만 합친다. 아래 번호는 추후 독립 작업 채팅/checkout에 바로 배정할 수 있는 단위이며, 이 계획 작성만으로 새 채팅이나 worktree를 생성하지 않는다.

| 레인 | 주 소유 경로·커밋 | 적대 감사 질문과 독립 오라클 | 완료 산출물 |
| --- | --- | --- | --- |
| **01 Surface/HIR·parse custody** | `surface`, `hir`, Pipeline parse/check owner (`cc27ff2`, `e598a1d`) | 같은 bytes라도 다른 source/root/ledger의 permit을 거부하는가? byte cap, escape, malformed/unsupported root, nested parser scratch, exact/one-short 예약을 독립 입력으로 검증. 이동 전후 parse/진단/span을 비교. | 도달 가능한 caller→owner 경로, 거부 매트릭스, 선택된 owner 테스트와 잔여 비소유 allocation 목록. |
| **02 Binder/Types/Sema authority** | `binder`, `types`, `sema`, `modules` declaration/session (`e598a1d`; `cc27ff2`) | 선언·LET·builtin·type·catalog가 정확히 같은 발행 세션과 root를 쓰는가? 한 단계의 `Incomplete`/typed failure가 성공, 빈 결과 또는 다른 진단으로 바뀌는가? 필드·표현식 identity/order를 일반 compile 결과 및 독립 타입 기대값과 대조. | stage별 issuer/consumer 지도, 정상/거부 fixture, 원인 소유자와 공개 호환 영향. |
| **03 bounded scalar end-to-end** | `algebra`, `planning`, Pipeline `compile/request_owner`, sealed execution (`e598a1d`) | 허용된 단일 Query Source→Project만 통과하는가? unsupported shape가 unrestricted compile로 우회하지 않는가? Scan→Project, plan effect/cost/capability, DTO 세 digest가 ordinary compile과 일치하는가? shared clone의 charge가 마지막 drop까지 유지되는가? | source-bound reservation 생애주기, exact/short·foreign-root·last-drop 반례, 독립 byte/digest oracle. |
| **04 SCC·Logic** | Modules `graph/indexed_scc.rs`, Logic `recursion.rs`, Modules `graph.rs`의 cycle projection (`43d9943`, `c5b27aa`, `cd9aec0`) | 작은 그래프의 상호 도달성 oracle과 partition이 일치하는가? deep/wide·permuted graph에서 stack/scratch, cycle witness, 첫 typed refusal과 위치, 모든 edge charge, negation 우선이 유지되는가? | Modules/Logic 독립 graph oracle, malformed index·exact/short·50k 규모 증거, 공유 `graph.rs` 수정 요청. |
| **05 DAG/View·lock** | Modules `graph/indexed_walk.rs`, `catalog/view_topology.rs`, `manifest.rs`, `lockfile.rs` (`77550ba`, `36e3444`, `a6a85d1`) | repeated/empty/subset roots, dangling edge, cycle, overflow에서 caller 정책이 유지되는가? View minimum-ready 순서가 입력 순열에도 고정되는가? lock canonical bytes와 late malformed/duplicate vs ordering 진단·span이 보존되는가? 큰 공백 줄의 할당 상한은? | 독립 DAG/lock oracle, 1-short 예산, 깊이·폭 입력의 비용/peak allocation 측정 조건, `graph.rs` 변경 충돌 표시. |
| **06 Execution contract·facade** | 새 `execution-contract`, Pipeline bridge, `frontend-contract`, golden vectors (`f7b79e9`, `9abbdc3`, `25fad76`) | DTO field/order/source schema 3·contract/qualification schema 8 byte가 이전 owner 및 독립 oracle과 같은가? 외부에서 owner 직접 의존·사설 DTO 생성·digest 주입을 차단하는가? Rename token/target/snapshot은 의도만 표현하고 runtime grant로 읽히지 않는가? | byte/vector·외부 consumer positive/negative 결과, public API diff, downstream source-path 이동 목록. |
| **07 테스트·정책·CI 신뢰성** | `tools/`, `.semgrep/`, `.circleci/`, `Justfile`, `Cargo.*`, `clippy.toml` (`cc27ff2`, `293bbba`, `614d9de`, `cb43052`, `fa4c015`) | 새 모듈/삭제 경로/외부 test module을 gate가 빠뜨리지 않는가? 선택 0건·누락 source가 green이 되는가? Semgrep `--severity ERROR` 적용과 실제 스캔 파일을 확인. CircleCI `just ci` job과 로컬 receipt의 입력 차이를 기록. | fail-closed mutation fixture, gate 대상 목록, hosted CI `NOT_RUN` 또는 실제 run URL/commit, dependency/lock 변경 설명. |
| **08 문서·증거 수렴** | 202개 docs-only 커밋; `docs/adr`, `docs/contracts`, `docs/language`, `docs/work`, `docs/plans`의 생성·삭제·이동 | 현재 source/API와 살아 있는 문서 주장이 충돌하는가? 옛 SHA의 focused proof나 Semantica 계획을 QGLang 현재 완료로 오독하는가? 340경로 중 문서와 touched-only 파일의 최종 소유자는 어디인가? | 중복/깨진 링크/낡은 상태표 목록, 보존해야 할 ID·원본 증거, 정리 제안. 현재 미커밋 문서는 작성자 소유 확인 후만 수정. |
| **09 Semantica pair·실행 경계** | QGLang facade/contract source closure ↔ Semantica consumer (별도 repo) | 새 execution-contract package와 V3 vector를 정확한 QGLang SHA로 잠그고 consumer가 빌드/계약 테스트를 통과하는가? frontend digest와 runtime generation/provider/lease/grant를 별도로 결합하는가? | 두 repo의 SHA/tree/lock/feature tuple, selected/executed/pass, consumer 실패의 실제 원인. 설치·제품 증거는 별도 rail. |

### 조사 순서와 의존성

1. **준비 (통합자, 직렬):** 고정 SHA/dirty 소유권/활성 프로세스와 기존 영수증을 기록한다. 인벤토리 340경로를 번호별로 할당한다. 사용 중인 checkout과 target을 다른 레인에 재할당하지 않는다.
2. **Wave A (읽기 병렬):** 01, 04, 06, 07. 09의 별도 repo source/lock tuple 읽기는 이 중 슬롯이 비면 시작한다. 코드 변경·광범위 테스트는 아직 하지 않는다.
3. **Wave B (읽기 병렬):** 02, 03, 05, 08. 02는 01의 parser-issued carrier 경계를, 03은 02의 typed facts를 입력으로 받되 현재 코드의 독립 반례는 기다리지 않고 조사한다. 05와 04는 `modules/src/graph.rs`를 동시에 수정하지 않는다.
4. **공격적 교차 검토:** 레인마다 다른 레인이 최소 하나의 정상 주장과 하나의 거부/예산 주장을 독립 입력으로 반박 시도한다. 단순 기존 테스트 재실행이나 문서 인용만으로는 `PASS`하지 않는다.
5. **수정 파동:** 확인된 결함만 원인 소유자에서 고치고 독립 expected result와 회귀 테스트를 더한다. 01/02/03의 public API·공유 owner 파일, 04/05의 `graph.rs`·budget, 06/07의 facade·manifest·lock·정책은 통합자 승인 순서로 한 번에 결합한다. 제거된 계획 문서나 무관한 dirty hunk는 편집하지 않는다.
6. **최종 통합 (직렬):** 소스 tuple을 재고정하고 충돌/중복 구현/계약 byte를 다시 본다. 가장 작은 owner rail만 반복한 뒤 결합 소스에서 `just ci` **한 번** 실행한다. clean committed HEAD의 로컬 receipt가 필요하면 그 뒤 `just qualify-local`을 실행한다. 09의 Semantica pair와 installed/hosted rail은 별도 성공 조건이다.

## 레인 공통 보고 형식과 판정 규칙

각 레인은 다음을 한 표로 제출한다: `ID`, 고정 SHA/tree·dirty 소유권, 파일/줄·reachable caller→owner, 불변식과 consumer, 최소 입력, **독립 예상 결과 / 실제 결과**, 분류(`CONFIRMED_DEFECT`, `COVERAGE_GAP`, `PROOF_GAP`, `DOWNSTREAM`, `NO_ACTION`), 영향·호환성, 수정 소유자, 실행 명령과 selected/executed/passed/skipped, 증거 경로. 실행하지 않은 것은 0이 아니라 `NOT_RUN`이다.

- **결함:** 현재 도달 가능한 source에서 독립 기대값과 다르게 성공·누락·잘못된 거부·비결정성·무제한 작업이 재현된다. 오래된 계획의 finding 제목만으로 재발급하지 않는다.
- **커버리지 공백:** 실제 규칙 대상이 스캔되지 않거나 regression이 실패를 거부하지 못한다. 예: 현재 `.semgrep/qglang-frontend.yml`과 `tools/lint/semgrep_gate.py::CORE_CRATES`는 새 `execution-contract/src`를 열거하지 않는다. 해당 typed-error 규칙이 새 owner에도 적용되는지 정책 의도를 확인하고, 적용된다면 누락 경로를 넣은 negative fixture로 증명한다. 아직 전체 policy 결함으로 확정하지 않는다.
- **증거 공백:** source는 구현됐지만 현재 입력의 test/receipt/benchmark가 없다. old green, compile-only, source-only, 미완료 runner는 통과 증거가 아니다.
- **Downstream:** facade 발행 이후 admission, provider, SDK, 설치 제품의 문제는 정확한 두 repo의 source tuple과 함께 해당 owner로 보낸다. QGLang DTO만으로 제품 실행을 주장하지 않는다.
- **성능:** 회귀 입력 크기, SHA, toolchain, 선택된 경로, work/alloc/RSS 측정법을 고정한다. Big-O 개선 추론과 실제 처리량/peak 개선을 구분한다.

## 완료 조건과 중지 조건

- 모든 340경로가 레인/retired-doc/touched-only 중 하나로 설명되고, 14개 코드·설정 커밋의 계약 영향이 각 owner와 소비자에 연결된다.
- 확인된 결함의 source 수정·독립 regression·거부 경로가 완료되고, 합친 QGLang HEAD의 `just ci`/필요한 clean-HEAD 영수증이 동일 source에 결합된다. 새 public DTO byte/의미 변화가 있으면 계약·digest schema/골든 vector와 downstream pair를 함께 version한다.
- 남은 항목은 **결함 / coverage / proof / downstream / 제안**으로 나누어 소유자, 조건, 다음 실행 입력을 남긴다. 검증 공백을 결함으로 세거나 focused pass로 제품 완료를 선언하지 않는다.
- 기준 HEAD/dirty ownership이 이동하거나 다른 레인의 파일을 동시에 고쳐야 하면 해당 범위의 수정·검증을 멈추고 소스/소유권을 다시 고정한다. 임의의 force push, 공유 checkout reset, 다른 작업자의 dirty 문서 삭제는 하지 않는다.

이 문서 작성 시점에는 레인 실행, 새 worktree/채팅 생성, 코드 수정 또는 테스트 실행을 하지 않았다.
