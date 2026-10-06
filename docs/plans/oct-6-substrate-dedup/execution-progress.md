# 구현 checkpoint — Oct 6 substrate dedup

계획: [readme.md](readme.md), [파일·Before/After](code-change-map.md), [테스트 플랜](test-plan.md).

## 1. 작업 custody

구현 시작 base는 Semantica `5cb96ba45c8ed933e6511647ecd2ceb3a5d4e03a`다. 네 checkout은 같은 committed source에서 생성했다. 원래 main의 foreign dirty는 복사하지 않았다. Main은 다른 작업으로 계속 이동하므로 아래 base를 현재 main이라고 부르지 않는다.

공통 경로: `/Users/songmin/.codex/worktrees/` 아래 각 이름의 `semantica-codegraph-v2/`.

| checkout 이름 / branch | 담당 | owned diff |
|---|---|---|
| `oct6-substrate-leaf` / `codex/oct6-substrate-leaf` | D3 → D5 → D4 | DFG build-input 삭제, dormant UF 삭제, SCV2 contribution reuse |
| `oct6-substrate-manifest` / `codex/oct6-substrate-manifest` | D7 → D2 | manifest capture; 공통 SCC core와 PTA/taint cutover |
| `oct6-substrate-shape` / `codex/oct6-substrate-shape` | D1-A → D1-B | standalone Shape factory 전환, cost Shape 삭제, feature/cfg/SDK/gate 정리 |
| `oct6-substrate-search` / `codex/oct6-substrate-search` | F1 → D6 | native search 계약, typed SDK DTO, 공통 app converter와 실제 consumers |

초기 sibling dependency symlink는 live 원래 checkout을 가리켰다. D1 exact snapshot은 symlink topology 거절 후 별도 physical detached siblings를 사용했다. Rust 결과는 해당 QBC source snapshot/외부 dependency topology에 묶인다. Python 실행은 `PYTHONPATH`와 module `__file__`로 구현 worktree 사용을 확인한다. 공유 `.venv`가 외부 작업으로 삭제된 뒤에는 그 이전 결과와 복구 환경의 새 실행을 구분한다.

## 2. 관찰 상태 — 2026-10-06 진행 중

| 항목 | 구현 / 실제 실행 관찰 | main 상태 / 잔여 |
|---|---|---|
| D3 | private `fd69cb7686d`; DFG QBC 91/91, pack QBC 16/16, actual taint promotion 3/3 GREEN | Main `d88f0a2ed7a`; source imported. Worker receipt는 그 source/dependency snapshot에 한정 |
| D5 | private `49e532799c9`; active PTA solver QBC 227/227 GREEN | Main `d88f0a2ed7a`; dormant module/export 삭제, base/active solver 보존 |
| D4 | private `c0396ef17c9`; no-default SCV2 owning QBC 34/34 GREEN; rank bits/mixed sum 및 algo-only closure 확인 | Main `d88f0a2ed7a`; actual canonical classifier/rustfmt/safety fences PASS |
| D7 | root final clean 6-file owner matrix70/70 GREEN; captured source/duplicate/override/init/parity/surface/DI controls | Main `b258f5919a5`; 7 owned paths만 checkout custody 갱신, canonical Ruff/safety PASS |
| D1-A | current private `a63825b598e`, base `0fd79e4`; owner rerun17/17 GREEN 후 selected closure delta0 | 미반영. Runtime/SDK consumer proof 필요 |
| D1-B | current private `36c0cfeb17a`; locked metadata GREEN. Runtime와 SDK session/executor Shape-only normal deps에서 Cost 부재 | 미반영. Parser 선행 수정은 main `39edfc1ca5b`에 반영. Actual runtime consumer QBC는 공통 analysis API compile13 errors/0 cases로 BLOCKED; Cost-only 실제 overlay·SDK/gates도 미증명 |
| F1 / D6 | private `d0cb3969994` → rebased `5cc4a2c9972`; owned39 inputs unchanged. Affected SDK213 /211 pass /2 skip, rebased consumers142 /141 pass /1 skip, scripted installed SDK wheel1/1·actual Granian HTTP process+scripted daemon1/1 GREEN | Main `c1f821db572`;39 paths,+1434/-1399. Canonical Ruff38/safety PASS; original owned paths clean. Real native daemon/product qualification은 NOT_RUN |
| D2-P/K | final shared borrowed iterative core; actual frame/result bound 보완 후 QBC 20/20 GREEN. Rebase의 Cargo/lib/scc bytes 동일, deps 없음·toolchain1.92 동일 | Main `df2a867ad70`; core5 paths 반영. PTA/Taint cutover/consumer proof는 별도 pending |
| D2 PTA | current private `1ea51f05dc5`, base `7e9447923d6`; wrapper/caller typed Result 전파와 최소 test prerequisites | 이전 `300ba0df508` QBC는 local owner/consumer skew로 lib-test compile54 errors/0 cases. New main에서 일부 해결됐으나 누락 export/API 남음; current nonzero owner proof BLOCKED |
| D2 Taint | successor-only adapter / typed resource mapping / contract caller 보완. Current private `1ea51f05dc5`에서 iterative5 pass/1 ignored·SCC property14/14·policy units13 pass/1 ignored QBC 최종 GREEN | Contract3/3 GREEN; 실제 ignored50k·기존 gate6단계 proof와 fresh-main import pending |


숫자는 서로 다른 owner/snapshot의 component 결과다. 합산해 하나의 repository/product qualification 결과로 쓰지 않는다. Private commit의 main import는 exact fresh main의 owned path delta를 확인한 뒤 직렬 수행한다. Private branch commit에서 우회된 hook은 root import 전에 해당 canonical fence의 실제 결과를 확보한다.

## 3. F1에서 확인한 구체적인 계약

Native authority: `crates/quanta-cli-ingress-runtime/src/project/search_projection.rs`의 `project_search_page` / `search_match_row` / `search_supplemental_row_v1`와 gateway `DaemonSearchResultPageV1`.

현재 writer에는 `symbol_id/name/kind/location`이 없다. 기본 row는 `row_kind=match`, `snippet`, optional `file_path/line/column`, optional `symbol_fqn/symbol_kind/runtime_symbol_locator/symbol_discriminator`, score·lane·provenance·truth metadata를 발급한다. Supplemental은 `row_kind=supplemental`, `entity_id/entity_kind`, score·optional projection/provenance다. Runtime locator는 source offset을 포함하는 opaque value이며 canonical symbol ID로 이름을 바꾸지 않는다.

After SDK schema는 하나의 `data={"hits": [...native row fields...], "summary": {...native scalar fields...}}`로 전환 중이다. Typed `LexicalSearchResult`도 같은 discriminated row variants와 summary를 사용한다. Old `symbols/total/truncated` reader와 fabricated full-range DTO를 보존하지 않는다.

| summary | native 정의 | After 검증 |
|---|---|---|
| `total_count` | primary 결과 전체 count | primary returned count 이상; supplemental count와 합치지 않음 |
| `page_count` | `matches_v1.len()` | returned primary count와 동일 |
| `match_count` | `matches_v1.len()` | returned primary count와 동일 |
| `supplemental_match_count` | supplemental row count | returned supplemental count와 동일 |
| `result_truncated` | runtime metadata boolean | 그대로 보존; false 기본값 없음 |

이 writer는 end coordinates를 발급하지 않는다. `line/column`만 보존하고 highlight의 byte offsets를 source end coordinates로 바꾸지 않는다. S7 full source span 보존은 기존 app metadata 경로의 독립 control이며 native search가 그 span을 발급한다는 proof가 아니다.

## 4. 다음 checkpoint에서 기록할 것

- exact worker commit/parent, 실제 변경 파일과 symbols, source digest와 실행 root.
- owner command/package/target/features/selector, 선택 case 이름·수, outcome/receipt.
- retired definitions/exports/imports, actual consumer path, generated/feature gate 결과.
- main import SHA와 import 전후 selected input delta. 재사용한 receipt는 `GREEN@old + unchanged-input delta`로 표시.
- 미실행 installed/process/performance 범위, 다른 F backlog와 이 계획 밖의 기능 수리.


## 5. 실제 checkpoint와 추가 발견

- D7 main import: `b258f5919a589611d21674c4e388e7ec70d8ed60`. 정식 hook의 SIM117 거절을 수정한 뒤 재실행·반영했다. Root focused33는 rebased integration source에서 실행했다.
- D7 추가 consumer matrix: integration `8304a0098a6`에서 기존 parity/surface/DI 3 files 37 selected, 35 pass/2 skip. 두 startup skip은 test `PYTHONPATH=...:apps/mcp:...`가 설치된 `mcp.server`를 가린 실행 환경 문제였다. SDK/shared/repo roots만 사용한 exact startup selectors 재실행은 2/2 PASS. Source/dependency 변경 없음; D7 owned bytes는 첫33 proof 이후 동일. Commands/cases/skip reason/JUnit digest는 [checkpoint-d7-evidence.json](checkpoint-d7-evidence.json)에 보존했다.
- Final D7 owner matrix는 SDK/shared/repo roots만 사용해 focused controls와 기존 5 files를 함께 실행했다: `8304a0098a6` clean, 70 selected/70 passed/0 skipped. Source tree는 core checkpoint `df2a867ad70`과 동일하다. 기록 시 main은 외부 commit `c00d98f63d4`로 이동했지만 relevant MCP/SDK/shared Python paths·pyproject·uv.lock delta는 0이다. 전체 current tree qualification으로 확대하지 않는다. 이전33의 original argv를 보존 log에서 복원할 수 없어 replay command와 executed final70 command를 구분했다.
- F1/D6 main import: `c1f821db572b2b1c1f433ba57bdaea3f6fc9b0f8`. Rebase 후 owned39 파일 bytes 동일; MCP registry가 갱신된 조합에서 consumer142를 재실행했다. SDK/installed scripted receipts는 이전 source 결과로 유지하며 actual native daemon proof로 확대하지 않는다.
- D3/D5/D4 main import: `d88f0a2ed7a64f38ae8ac97badbb9671a759e10c`. 14 paths, +65/-1667. Foreign dirty를 stage하지 않았다. Deleted 파일도 이전 committed bytes를 확인한 후 original checkout에서 퇴역시켰다.
- D2 core main import: `df2a867ad70dd1e1a71fea1abd4397bd9bcb2174`. 5 paths, +481/-200; integration source `8304a0098a6`, base `3c6a04ebc47`. Classifier/rustfmt2/safety PASS. Original root Cargo의 foreign QMP rev 변경은 three-way replay로 보존했다. PTA/Taint local 순회 제거는 이 checkpoint에 포함하지 않았다.
- Parser prerequisite main import: `39edfc1ca5bf143a09b66f0738c5f2b3987c7572`, base `b42b84465fb`, source `bb2b88dd137`. Unconditional Go branch와 shared helper의 `parser-java` cfg 불일치 1줄만 수정했다. Canonical rustfmt1/safety PASS; original owned path clean, claim/barrier 해제. QBC Cargo artifact는 Java·Go feature 없이 parser lib를 실제 새로 빌드했다(`fresh=false`). 이는 전체 runtime 빌드 성공 증명이 아니다.
- Search의 CLI는 퇴역 reader를 실제 사용했다. Current registered SDK ingress로 전환하고 logical repo ID와 attached repo path를 구분했다. Unsupported repo/snapshot/type를 attach 전에 거절한다.
- SDK `with_status=True`의 native receipt absence가 중간 projection에서 빠져 호출이 실패했다. Existing ProductResponse metadata/receipt owner로 `receipt=None`를 정확히 보존하고 native degraded reason을 existing receipt pair에 전달한다. Receipt를 발급하거나 owner truth를 합성하지 않는다.
- HTTP search/QueryAdapter/CLI가 native degraded outcome/reason을 보존한다. Shared converter는 data-only `{hits,summary}`만 받으며 transport/envelope parser가 아니다.
- PayloadBuilder는 absent file path를 `unknown` 파일 후보로 만들지 않고 native point/highlight를 full source range로 승격하지 않는다. Authored complete source range만 보존하고 malformed range를 거절한다.
- Search owner의 새 복잡도/크기 증가는 함수 책임을 분리해 제거했다. Rebased commit의14 production paths를 최초 worker base와 동일 AST scanner로 비교: findings44→36, 신규0·growth0. 기존부터 누락된 baseline13개와 퇴역 keys11개는 별도다. 무조건 baseline refresh로 새 위반을 수용하지 않는다.
- Scripted wheel/HTTP fixture는 현재 generated response의 `committed_mutation_v1=None`/receipt absence를 명시한다. HTTP fixture의 Python3.9 hardcode는 SDK minimum3.10과 충돌하므로 실행 test interpreter로 전환한다. Scripted daemon은 actual Rust daemon 실행 증명이 아니다.

## 6. 미해결 검증과 scope

- D1-B actual runtime/SDK pure·PTA·disabled·Cost lanes, 두 Shape gates. Heap feature의 explicit Cost edge는 보존한다.
- D2 PTA existing compile blockers의 committed-main/foreign-dirty 구분 및 owner rail, Taint public contract/deep50k·기존 gate6단계 proof. 첫 Taint iterative5 pass/1 ignored·SCC14 pass 결과는 변경 전후 source별 component proof다.
- D2 core admission의 workspace estimate가 actual frame/output allocation을 충분히 계산하지 않는 결함은 checked conservative bound와 tiny-limit refusal/positive control로 수정했다. Final core20 proof 후 main `df2a867ad70`에 반영됐다. Default 실사용 OOM을 증명한 것은 아니다.
- Final core owner QBC `20261006T022541.599406Z-8544c76c92a4`: 20/20 GREEN. Materialization checkpoint는 해당 member buffer allocation 전에 호출한다. 이 결과는 worker final dirty source에 묶이며 main import/PTA/Taint proof를 대신하지 않는다.
- D2 private 7-step gate 확장의 taint contract selector에서 `cfg-dfg` 누락으로 0 tests가 선택됨을 확인하고 보완했다. Main의 기존 gate는 iterative/ignored50k 두 단계이며 contract step이 없다. Private 확장 오류를 현재 main 결함으로 세지 않는다. 별도 contract QBC에 feature를 명시하고 nonzero proof를 확보한다; 전체 7-step gate 반영은 PTA 선행 이후다.
- Shape full gate의 `heap_shape`도 기본 feature/selector 조합에서 0 tests였다. 실제 generated `shape_analyzer_` port suite와 `analysis-contract-q2-analyzers`를 선택하도록 수정했다. SDK factory reachability와 runtime returned overlay의 실제 classification은 별도 proofs다.
- D1 standalone pre-rebase owner QBC `20261006T022722.633931Z-9671477d7d91`: 16/16 GREEN. Main `c929c932` rebase가 resolved local closure의22 파일과 lock을 바꿨으므로 이 receipt를 현재 source GREEN으로 재사용하지 않는다. Rebased own source는 동일하지만 local dependencies는 달라 owner rail을 재실행한다.
- Final rebased standalone owner QBC `20261006T024432.465548Z-ed4f5be512a1`: 17/17 GREEN. 별도 `acedbd2a491`은 frozen Index manifest가 요구하는 기존 `memchr`/`ciborium-ll` lock edge 2개만 보완한다. 새 package version이나 library 선택을 추가한 것이 아니다. Locked metadata PASS; actual runtime/SDK consumer proof는 진행 중이다.
- Runtime Shape-only QBC `20261006T024835.289983Z-31fce94661f2`는 parser compile E0425로 실패했다. `core_native_aug.rs`의 unconditional Go branch가 `parser-java`에만 정의된 import-binding helper를 호출한다. Helper의 잘못된 cfg를 제거하는 owner prerequisite를 최소 수정하며 Java feature를 추가하지 않는다. Runtime/SDK 결과는 아직 GREEN이 아니다.
- Shape range는 core main `df2a867ad70`에 rebase했다. 기존 Index lock edge는 이미 main에 있어 별도 lock commit이 skip됐다. Locked metadata PASS. 실제 compiled `quanta-runtime-context-shared/src/fallible_sort_v1.rs`가 달라 owner17 receipt는 이전 source로 유지하고 재실행한다. Parser prerequisite는 helper cfg 1줄 삭제이며 기존 diagnostic context 문자열을 보존했다.
- Core dependency 변경 후 standalone owner QBC `20261006T031048.999998Z-0bacfa6f1868` 재실행17/17 GREEN. df2a→0fd의 resolved Shape owner closure delta는 0이다. Runtime/SDK consumer closure는 별도 변했으므로 current Shape range를 0fd에 rebase한 뒤 consumer QBC를 실행한다.
- PTA QBC `20261006T030029.078651Z-4e0d283d3eed`는 lib-test compile54 errors/0 cases, exit101이다. Worker와 committed main의 QMP pin14cf는 같고 foreign dirty472048은 해당 tuple 밖이다. 독립 감사 두 건은 local owner/consumer migration의 missing definitions/exports/signature skew를 확인했다. Rebased7e에서 CloneLogicalCostCore와 wire Ref 정의는 해결됐지만 facade/rebind/role exports 등은 남아 있다. Old54를 최신 main의 오류 수로 주장하지 않는다. Exact commands/snapshots/cases·RCA는 [checkpoint-rust-evidence.json](checkpoint-rust-evidence.json)에 보존했다.
- Parser 수정 후 runtime Shape-only QBC `20261006T031607.551813Z-7570fb8c0144`, source digest `70bf23ecf022b6e5c453a2ecbabbb1b5fe5def6a970517d9cffc15beff85ab52`도 exit101이다. Parser는 compile artifact를 발급했으나 `quanta-runtime-analysis` lib의 missing rebind/export/method·argument 오류13개에서 중단됐다. Runtime Shape selector는 0 cases; D1 A/B는 private에 보존한다. Main `39edfc1ca5b` 정적 재확인에서도 execution rebind 함수·`ProductRuntimeRebindErrorV1`·structural wire provider가 없어 upstream owner 작업의 committed source tuple을 기다린다. Foreign dirty를 가져오거나 compatibility shim을 추가하지 않는다.
- Committed main `dbffb79db15`의 별도 정적 재확인에서도 PTA/DFG execution rebind owner와 caller/export, Product rebind 오류형·export chain, wire structural DTO·공개 field list가 어긋난다. Foreign dirty에는 대응 구현이 있지만 이 작업에 채택하지 않았다. 현재 main의 오류 개수는 새 빌드 없이 추정하지 않는다. 이 owner/consumer tuple의 동일 HEAD 정합화가 D1 runtime 및 D2 PTA 재검증의 선행 조건이다.
- 이후 main `8f560b0178c`에서 wire structural DTO·공개 field list·serde/root export는 연결됐다. PTA/DFG execution rebind와 Product 오류형은 여전히 정의/export/caller가 어긋난다. 해당 files는 dbff→8f 사이 변경도 없어 정적 차단 근거가 유지된다. D1 branch는 clean `4402f61`로 보존하며 rebase/expensive consumer QBC를 시작하지 않았다. 상세 재확인과 과거 결과는 [Rust evidence](checkpoint-rust-evidence.json)에 구분했다.
- Current Taint QBC `20261006T033315.108639Z-d20018367b1e`는 iterative5 pass/1 ignored, `20261006T040300.462942Z-245ef474119e`는 SCC property14/14 pass다. 둘 다 private `1ea51f05dc5`, source digest `fb2e1b963212894428f3a2595fe57dd3f8d01427814f667f1becba9e378e7390`, launcher drain 후 finished/exit0이다. Property의 after_run 중간 관찰 exit125는 최종 판정이 아니다. Lib/contract/ignored50k와 main import는 각각 별도 proof로 추적하며 [Rust evidence](checkpoint-rust-evidence.json)에 commands/log/receipt digest를 보존했다.
- 같은 private source/digest의 policy unit QBC `20261006T040639.717643Z-38f0918b7a0b`도 finished/exit0이다. `wire_recursion` selector14 selected 중13 pass/1 ignored,5113 filtered로 typed budget reason과 독립 cycle policy를 확인했다. Ignored deep case를 통과한 것으로 세지 않으며 별도 명시적 integration50k 실행을 수행한다. Core/Taint가 닫힌 뒤 기존 Tarjan gate의6개 selector를 먼저 반영하고 PTA proof 후7번째를 추가한다; 새 임시 gate나 empty-selection 성공은 없다.
- QBC 종료 후 D1-B의 기존 runtime overlay classification test guard를 `any(cost-analyzer, shape-analyzer)`로 넓히고 full gate에 같은 test의 Cost-only lane을 추가했다. Test body/API 복제 없이 public Cost-only 연결을 직접 검증하도록 보완했다. Current private range는 A `a63825b598e`, B `36c0cfeb17a`, parser `4402f61fc18`; 이전 failed receipt source를 새 range의 결과로 바꾸지 않는다. Rustfmt/ShellCheck/safety PASS, actual Cost-only 실행은 NOT_RUN이다.
- F1/D6 main import와 rebased affected consumers proof 완료. Generated baseline의 퇴역 keys 정리는 전체 source checkpoint 후 수행한다. Actual native daemon/product proof는 NOT_RUN.
- Shared generated inventories/baselines는 삭제/이동한 실제 keys만 retire/reconcile하고 새로운 scope waiver를 추가하지 않는다.
- Frozen main `39edfc1ca5b`에서 topology owning generate/check PASS(1089 targets,1008 census rows). 이후 main source가 이동해 두 JSON은 보존했고 final source에서 재생성한 뒤 반영한다. GOD은 D3/D5/Search/core scope의17 obsolete keys를 제거했다: current findings21, 신규0·growth0·outside entries 동일. CI barrier 해제 후 base `eff71929856`에 rebase하고 actual CLI19 files PASS(stale0)를 다시 확인했다. Main `0a86de35c282195bed0a69456bf81a63e6b62bc7`, source `b869e104972`; exact1 JSON만 반영하고 owned path clean·claim/barrier 해제. SDK `client.py`/`_receipt.py`와 MemSSA summary의 기존 unbaselined/grown debt는 포함하지 않았다.
- 같은 frozen source의 Honesty full-sweep ratchet는 positive-negative-pair와 relationship 둘 다 exit1, ratcheted0이다. Evidence-grounding regression도 exit1이다. Raw new/rewritten findings를 이 작업의 실제 runtime 결함 수로 세지 않는다. D3의 preserved producer test74 lines는 base/D3 parent/D3/current 모두 byte-identical이고 `def_a`를 `vec!`→builder에 전달한다. Relationship scanner가 이 macro use를 놓친 false positive를 확인했다. Historical baseline key와 current source line은 구분한다. Positive-negative의8 obsolete keys에는 적용 경로 정책·이전 rename 영향도 있어 D3 test8개 삭제나 negative controls 추가로 주장하지 않는다. 상세 commands/log digests/keys/body hash는 [checkpoint-generated-evidence.json](checkpoint-generated-evidence.json)에 보존했다. Honesty baseline은 수정하지 않았다.
- Repository/product-wide qualification은 별도다. Affected SDK의 rename/rollback selectors는 이 packet 밖의 기존 failures 때문에 제외한 rail이며 full SDK suite GREEN으로 표시하지 않는다. 별도 실행한 GQLang receipt-wins tests 4개도 기존 profile precheck에서 실패했으며 이 변경의 성공 receipt로 합산하지 않는다.
- F2/F4/F5/F6/F7/F9/F10은 여전히 별도 기능·증명 backlog다.

GQLang local `just ci` 재실행은 **all groups passed**, source snapshot `83f930a4bf86250e925e16386c28d3e8e03e12af315569b293e7a62479e031f9`다. 첫 실행은 historical audit의 SHA/digest를 typos가 단어로 분해한 4개 false positives에서 중단됐다. Hex identifier7–64 chars만 typos 식별에서 제외하고 ordinary prose typo 거절 control을 확인했다. 이 결과는 실행 시점의 GQLang dirty snapshot에 한정하며 Semantica runtime/product qualification 결과가 아니다. 실행 로그: `/tmp/oct6-gqlang-local-ci.log`.

- 재개 시 committed main `8454fbf7812` 독립 정적 재확인: wire tuple는 닫혔고 PTA/DFG execution rebind 및 Product 오류형의 정의/export/caller 불일치는 유지된다. 8f 이후 관련 변경은 durable borrowed lookup이며 해당 경로는 미변경이다. 새 runtime 빌드·현재 오류 수 주장은 없다.
- Taint public contract QBC `20261006T042220.648000Z-9ec7d6f82c40`: private `1ea51f05dc5`/기존 digest에서3/3 PASS. 최종 receipt finished/exit0과 launcher PID96905 종료를 root가 확인했다. 명시적 ignored50k와 최신 main의6단계 gate는 별도이며 과거 source 결과를 최신 main proof로 재사용하지 않는다.

- 명시적 ignored50k QBC `20261006T045031.586428Z-c83fd7d0cf12`:1/1 PASS,0 ignored, source `1ea51f05dc5`의 동일 digest, launcher 최종 exit0. Cargo의 Index lock 들여쓰기16행 변경만 독립 확인 후 원상복구했다.
- 최신 committed base `05f755e1d31`에서 Taint9-path candidate `c3cb1918d02`를 구성했다. 원래 PTA 포함24-path source/gate7은 `codex/oct6-substrate-manifest`의 `1ea51f05dc5`에 보존한다. Candidate의7개 Taint source bytes는 이전 증거와 동일, lock은 Taint dependency1 hunk만, gate는 existing6 selectors다. Normal hooks Rustfmt6/safety PASS. Shipping toolchain을 고정하는 기존 gate6단계 실행 중이며 최종 QBC 증거 전 import하지 않는다.

- 최신 후보 live gate의 첫 실행은 외부 `quanta-index` 변경으로 closure capture에서 종료(exit1/0 Rust tests), 재시도는 core20 Cargo tests 뒤 verification source changed로 immutable completion locator를 발급하지 못했다(exit74). Cargo receipt exit0을 gate GREEN으로 세지 않는다. 두 번째 closure 변경의 단일 member 귀속은 미확정이며 candidate에서 Index lock 들여쓰기16행 side effect를 별도 확인했다.
- Candidate `9fe9fe4d5f7`는 해당16행만 Cargo의 canonical 형식으로 커밋했다(`--ignore-all-space` semantic delta0, normal safety fence PASS). `ensure_snapshot_worktree`/snapshot lease의 기존 owning API와 PLAIN_DIRECTORY 모드로 외부 sibling의 committed HEAD만 고정한다. Foreign dirty overlays는 제외한다. 같은 기존 gate6단계 실행 결과를 확보한 뒤 checkpoint한다; gate 우회·source drift 허용은 없다.

- Frozen snapshot은 candidate `9fe9fe4d5f7`, gqlang `c402321`, Index `498cd46`의 committed HEAD를 고정했다: topology digest `9fe9fe4d5f7977b687a8e5e467eadc0857d9a86d+ext-0ab3db4d28be5675435a`, sibling dirty overlay0. 새 실행 root lane 미등록으로 첫 frozen gate는 exit74/0 tests/dirty0이었다. 정식 `lane-token issue` 후 같은 snapshot/gate를 재실행했다. Snapshot discovery는 private sibling symlink를 허용하지 않으므로 plain primary sibling layout에서 candidate SHA를 선택해 owning API로 생성했다.
- 당시 committed `26ca0079ff2e` 독립 감사에서도 rebind/Product provider tuple는 미완성이었다. PTA16-file 변경은 DFG/constructor custody이며 execution helper·Product enum을 제공하지 않았다. 이후 연결 복구는 다음 절의 `fbb0583` 관찰로 갱신한다.

### Frozen gate 종료와 선행 조건 갱신

- Registered frozen gate는 최종 exit101, source dirty0으로 종료했다. Core QBC `20261006T051350.373538Z-6612abbd93ec`는20/20 PASS와 immutable suite completion을 발급했다. 2단계 `20261006T052441.019294Z-a7a791d54c79`는 CallGraph wire lib compile29 diagnostics에서 중단됐다: sibling module1, canonical helper2, unused import3, trait scope 연쇄22, tuple 참조1. Taint 테스트0개이며 3–6단계는 NOT_RUN이다. Compiler 선행 실패를 SCC 결과 회귀로 분류하지 않는다.
- Wire 최소 수정은 기존 모듈/함수/trait의 import를 소유 child scope에 배치하고 encoder tuple의 불필요한 borrow를 제거한다. 공개 DTO/schema/visibility/authority 정책 변화는 없다. Private 최초 source `6e2af852a1e` normal Rustfmt5/safety PASS; committed base `ae423eac4b5`로 rebase한 fix `7d611039d0f`와 기존 lock16행 whitespace 정규화만 합친 clean source `060a2f2b426`에서 default 및 `call-graph-authority-serde` owner tests를 canonical frozen snapshot/QBC로 실행 중이다. Main의 foreign Java witness dirty는 채택하지 않았다.
- Committed `fbb0583`은 기존26ca 관찰 뒤 PTA/DFG `_under_execution_v1` owner 및 Product 오류형을 제공했다. 독립 감사2건은 `45eec3966f7`에서 definition→export→caller 인자 수와 `interprocedural-engine` export chain을 확인했다. 이전 정적 차단 근거는 해당3경로에서 해소됐고, 최신 runtime compile 성공은 아직 관찰하지 않았다. Old13/54를 현재 오류 수로 세지 않는다.
- D1 A/B는 정확한 committed base `45eec3966f7`로 rebase했다: `de072b6c563`/`734aab87d0a`, clean, range-diff 두 patch 동일. 기존 parser patch는 upstream에 있어 drop됐다. Private backup `4402f61`을 보존하며 runtime/SDK QBC는 Wire 선행 수정 검증·committed tuple 반영 후 재개한다. PTA-only 후보도 별도 worktree에서 준비한다.
- Wire default owner QBC `20261006T054939.888945Z-ff446fdae925`는 source `060a2f2b426`, digest `96bbde716e197e4513cb588510658665af6b4ceb1260344589e2f8843951f5e4`, exit101/0tests/dirty0이다. Production29 diagnostics는 없어졌으나 serde block에서만 쓰는 test counter import1개가 default lib-test에서 미사용으로 거절됐다. Serde 단계는 아직 실행되지 않았다. Counter import를 `all(test, call-graph-authority-serde)`로 정합화했고 기존 tests의 `ciborium::de` 직접 호출에 빠졌던 workspace dev dependency와 양쪽 lock edge를 연결했다. Version/production dependency/schema 변화 없이 clean source `fae420827a4`, exact8paths, normal Rustfmt1/classifier/safety PASS. 재검증은 serde→default이며 둘 다 actual nonzero QBC 결과가 필요하다.
- PTA-only clean candidate `680305490fd`는 exact base `45eec3966f7`,12paths다: SCC/caller/tests9 source + runtime-analysis manifest1 + 기존 outer/inner lock의 직접 graph-utils edge2. Local Tarjan state/frame을 제거하고 declared-key domain·emission order·typed Stop/QueryFailed/Incomplete 계약을 유지했다. 기존 under_execution와 Product 연결 파일은 미변경이다. Normal staged integrity/classifier/Rustfmt9/safety PASS, Rust build NOT_RUN. Actual owner/gate는 Wire가 연결된 고정 source에서 실행한다.

### Wire serde scope repair and combined candidate

- Wire serde QBC `20261006T060427.100331Z-d650cf957933`: source `fae420827a4`, digest `c5503ecbebe8d07703e1a11fffa9f150ffdae05d53b7380f10eb5e8b033e9a8a`, exit101/0tests/dirty0. 기존 projection test 블록의 `canonical_encode` 직접 import가 빠졌으며 형제 tests module의 private import는 전달되지 않는다. Production 진단과 구분한다. 해당 attempt의 default는 NOT_RUN.
- 기존 serde 블록에 직접 import한 clean Wire source `b0dbe01cd35`는9paths다: production import/tuple5Rust, test scope1Rust, 기존 CBOR dev edge와 outer/inner lock3files. Public schema/visibility/authority 정책 변화0. Normal Rustfmt1/safety PASS. 새 frozen snapshot에서 serde→default 두 owner lib targets 검증 중이다.
- 통합 clean candidate `4bdbdf87ae32aae94b3709b1665423249b3c031e`, base `ae423eac4b5`,81paths에 Wire·PTA·Taint·D1-A/B와 기존 Tarjan7 selectors를 합쳤다. 독립 정적 감사는 D1 range-diff 동일, PTA9Rust와 Taint6Rust+manifest bytes 동일, lock version/source replacement0을 확인했다. Rust GREEN으로 세지 않는다. Wire owner 통과·main checkpoint 뒤 같은 frozen tuple에서 Tarjan7→Shape full15→typed pin→Termination→Shape-only trees를 실행한다.
- 사용하지 않은 이전 통합 `835a334d91e` snapshot은 Wire 실패 뒤 lease를 해제했으며 Rust 테스트0개다. 수정된 통합 source의 snapshot을 다시 준비한다.

### Current-base refresh after concurrent owner checkpoint

- Frozen Wire `b0dbe01cd35` 두 owner modes는 sealed QBC PASS: serde64 (`20261006T061734.096713Z-46bca73b89d7`), default45 (`20261006T062347.176031Z-b3a7bddfa23b`), dirty0, launcher exit0. 총109 cases는 해당 정확한 source 증거다.
- 검증 중 committed main `db2f1461a5a`는 같은 production import/tuple 수리를 제공했고 `486964fbf22`는 native clone 계산을 변경했다. 이를 포함한 base `2a25c00cef2`로 Wire·통합 branch를 rebase했다. Native owner 입력이 바뀌었으므로 b0 GREEN을 현재 입력에 재사용하지 않는다. 사용 전4b snapshot도 Rust0/lease released로 취소했다.
- 남은 Wire private source `213799a85fe`는5paths—test counter cfg, projection-test import, 기존 CBOR dev dependency, 양쪽 lock이다. Rebase 중 기존 standalone import와 옛 inner import가 함께 생긴 `projection_validation_v1` 중복은 upstream bytes로 복원했다. Production 기능/schema/visibility 변화0, normal Rustfmt1/safety PASS.
- 실제 base→candidate GOD 비교는 stale baseline12건과 이번 신규2건을 분리했다. PTA SCC 실패정책과 SDK Shape port field 전달을 owner helper로 분리하며 Stop 전파/Compute incomplete/복제 없는 move를 유지한다. 이 수리의 Rust QBC는 아직 실행하지 않았다. 기존 field-refinement caller controls는 Tarjan의 SCC 필터 밖이므로 별도 owning rail을 추가한다.

### Wire checkpoint and refreshed remaining implementation

- 통합 `482a1846f62`의 Wire serde65/default46은 sealed PASS였으나 main의 공통 CBOR 입력4개가 바뀌어 import 전에 중단했다. Main 수정0, lease released, consumer tests0. 이후 `f6c2907bc5e`/base `0d8bffc115b`에서 두 owner를 다시 실행해 serde65/default46을 sealed PASS로 확인했다.
- Wire 5개 파일은 main `fddfb1e175e48f5cf1c3d18d30630777d533d1fd`에 반영했다. Typed test import/cfg와 기존 Ciborium dev edge만 추가하며 schema/API/production authority 정책 변화0. Foreign dirty/index를 stage하지 않고 three-way replay로 보존했다. Own claim/barrier released.
- 같은 f6c snapshot의 Tarjan gate는 core20 sealed PASS까지 완료됐다. Main에서 PTA lib-test의 `BTreeSet` import·lifecycle fixture/lifetime 수리, native evidence owner lifetime, CBOR authority와 QMP pin이 추가돼 나머지 실행을 취소했다(exit130/lease released). PTA/Taint/Shape의 새 consumer GREEN으로 세지 않는다.
- 최신 통합 후보 `813857b73fc738ab74d83ce5c0f07970ff49c093`, committed base `8b44be652250f483296bd1c3f8898ad5b06580fc`, clean75paths다. Wire patch3개는 upstream에 이미 있어 rebase에서 drop됐다. Fresh Wire baseline은 같은8b main이며 새 engine/library/version 선택은 없다. 기존 main의 QMP pin 변경은 substrate dedup 변경과 구분한다.
- 남은 검증은 Wire serde/default → Tarjan7 → PTA field-refinement caller → Shape full15 → typed pin·Termination·SDK authority controls → shape-only feature tree2. Generated retirement6keys/topology2와 final main import는 해당 proof 뒤 진행한다. Live main의 후속 변화는 reviewed delta와 영향받는 owner proof로 구분한다.

### Selected test and owner lint follow-up

- Private `01abc49b4e6` strengthens two existing standalone Shape error tests: exact `EngineError` variant/reason and valid native fixture before ContextStack carrier rejection. Overflow test name now matches its private checked-index call. Owner shape-only/relationship measurement0, normal hooks PASS; Rust/QBC NOT_RUN on this new source. The two historical standalone Honesty keys were existing debt, not new findings.
- Private `bc1a10dd07b` extracts PTA file-identity owner child and SCC adjacency helper. Production file metric1255→1150 (baseline1188), recursive summary function84→73 (baseline83). Existing32 test names/order and moved bodies preserved. Normal rustfmt3/integrity/safety PASS; fresh owning QBC required.
- Static original-source GOD delta8 was separated from actual baseline:6 exceeded baseline,2 remained below. SDK/runtime repairs also address same edited capability file's two pre-existing function overruns. The first SDK commit attempt was rejected for wildcard reexport; explicit cfg/public-path-preserving repair follows, no allowlist change.
- `WorkerCfgDfgCacheV1` has a pre-existing unbaselined11-public-method finding. All methods have callers outside the impl; arbitrary impl splitting or deleting required methods would not repair ownership. It remains explicitly reported debt. Global Honesty ratchet still refuses a scoped baseline write and global new/rewritten violations; baseline writes0.

### Fresh owner failure and source repair

- Frozen `813857b73fc`/base8b Wire serde65/default46 and core20 are sealed PASS. Tarjan stage2 QBC `20261006T065950.339205Z-cc17c655c30a` fails lib-test compile87 errors/52 warnings/0 tests, exit101; snapshot clean and lease released after launcher drain. Stages3–7, field-refinement and all Shape consumer gates are NOT_RUN.
- Exact first-location comparison:2 diagnostics in changed D2 field owner (missing child module path plus likely inferred-type consequence),85 in files byte-identical to8b base. This is source attribution, not a base compilation result; newly active dependency paths remain possible. The direct D2 path defect is repaired with an explicit nested child path, not a diagnostic suppression.
- All private changes rebase cleanly to committed main `8539ca554ae318a5ee75a2e25815fff4b5e1002a`, producing `4bb1e21af9b` before further prerequisites. Latest main supplies several export/visibility/fixture repairs. Three remaining nested authority mod declarations are corrected; the wire-fields constant already follows the canonical export chain and is not duplicated. Borrowed lookup/cost owner links and remaining test fixtures are checked against actual87 diagnostics before the next owning QBC.
- SDK wildcard commit rejection is resolved by explicit cfg-preserving reexports64 public names, a same-path gateway module and private test-support helper. SDK/runtime structural commit8d91b46 passed normal rustfmt9/integrity/safety and rebased with preserved behavior. Rust verification remains pending.

### Frozen source after compiler prerequisite reconciliation

- Current private source `60eaba860404a2d1bcf4611d03140d97c4fbf55e`, base `10c902e74af63d054c8824c2ab5c8c4af0a19be7`, clean92 paths. Six fixture files retain their original positive/rejection assertions while using current admitted/retained APIs; normal hooks passed.
- The live resume logical-cost method reuses the portable walk through a borrowed view, with the same field/order census and typed failure propagation. The borrowed matcher now provided by main is reused; our duplicate proposal was removed. Existing role source axes are exported through the engine-gated public facade. No new engine/library/schema or authority policy was added.
- New owning QBC is running on the canonical archived-sibling snapshot: Wire2, Tarjan7, ten existing prerequisite selectors, field-refinement, Shape15, typed pin/Termination/SDK authority controls, then two feature diagnostics. No runtime success or current-main qualification is claimed before completion.
- Read-only scanner witness now contains exactly7 retired keys across4 owner paths (14 baseline entries→7 current findings, allowance/new/growth0). Generated writes still wait for owning proof and a reviewed main checkpoint. Global Honesty and the pre-existing WorkerCache11 debt remain separate.

### External Index compiler prerequisite and preserved main fixtures

- Frozen `60eaba860404` / archived Index `318023c06f3c` proves Wire serde65/default46 and SCC core20 with sealed receipts. Tarjan PTA `20261006T073146.648037Z-38f871aec9ae` fails before RuntimeAnalysis compilation: one direct nested `Visitor` impl in Index activation-token owner is rejected by `non_local_definitions` under deny(warnings). PTA tests0, later gates NOT_RUN; launcher exit101/snapshot clean/lease released. This is not a derive macro issue or evidence that the older87 RuntimeAnalysis errors are all repaired.
- Index private `93c85cf2963c` moves that exact impl to module scope, one file26insert/25delete, no new library/schema/authority policy/lint waiver. Native-feature canonical `cargow` owner test hit admission timeout124 after300seconds; tests0. Default owner selector also hit admission timeout124 after60seconds; tests0. Both canonical resource-admission attempts drained without compiling repaired source. Index main import is pending owner proof.
- S candidate rebases to main `6078f9282cb5`, now `1ac35525239a92ea7d875bbd991337d68891454f`, clean87paths. Five fixture repairs were already on main with preserved/stronger split controls; those main bytes and two children are retained. Only the alias borrowed-refs fixture remains a candidate change. Current main borrowed Java identity path is reviewed independently of its foreign dirty overlay.
- New paired source proof begins after the Index owner checkpoint. Old receipts remain attached to60e/Index318. Exact scoped GOD witness is refreshed to1ac (7keys/4paths/14→7/new0/growth0); generated writes and final S import remain NOT_RUN.

### Main-first prerequisite checkpoint and current combined proof

- Index local main `a72a85df2f70e1e2ca943afb5f62ad62dcae9b6d` imports exactly `activation_token.rs` from private `93c85cf2`: module-scope direct Visitor impl, body tokens unchanged, no API/schema/dependency/lint waiver. Exact one-path staged diff and foreign doc hashes preserved. Static checkpoint, not compile/test qualification; native owner retry waits at the canonical resource gate with 1800-second wait, default old timeout remains historical. No push.
- S private source `03ce6dec1eddd7dcd5684b459d7394f626251899` rebases all14 commits without conflict onto `7a887ec6cc0a43c751225983a3dd3fa20eff6297`;87 owned paths. Both independent reviews found zero committed path intersections with main-since-base29 paths. Main retained constructor custody/DFG RHS/module-identity changes survive. Primary foreign overlay has11 overlapping paths and is excluded from the proof; importer must replay/preserve it.
- New frozen QBC root `/Users/songmin/.cache/semantica-codegraph-v2/casectl-snapshots/03ce6dec1eddd7dcd5684b459d7394f626251899_ext-be67ee1a4a9443242c3b_mat-plain-v1/semantica-codegraph-v2`, topology `03ce6dec1eddd7dcd5684b459d7394f626251899+ext-be67ee1a4a9443242c3b`, archived G `2e8693e` + Index `a72a85df2f70e1e2ca943afb5f62ad62dcae9b6d`. Wire2→Tarjan7→ten PTA prerequisite selectors→field→Shape15/additional typed controls→feature diagnostics. Currently `running selected proof wire-serde`; no current runtime/SDK success or main qualification asserted.
- Main checkpoint policy is to land reviewed prerequisites while completing owner proof, then import dependency-closed D1/D2 changes with preserved foreign overlays and publish exact remaining proof gaps. Scoped GOD witness rebound to current source:7 retired/4paths/14→7/new0/worse0; no generated write yet.
- Current03ce frozen Wire owners sealed PASS: serde65/default46, run IDs `20261006T081758.131557Z-cc16a9f659d0` and `20261006T081908.256738Z-352411ecb3ef`; same archived tuple, immutable receipt hashes checked. Tarjan7 now running, no current PTA/Taint/Shape success yet. Overlay assembly rehearsal on main `3465e9730ac9` succeeded for all87 owned paths and preserved11 foreign dirty paths by clean-or-same/three-way merge; this is preparation only, no S main import.
- Additional S main prerequisite `65d1d94ed96b6b1a471cd01c5a728a45a6713aa3`: one-line existing `RecoveredCallGraphRoleSourceAxesV1` facade export, producer/caller/type reused. Normal coordinator source import with two advisory claim reconciliations, exact one path; overlay replay and own claim/barrier release succeeded. Boundary preflight11 policies PASS on pre-change7a and post-change03ce source states (executed after checkpoint). Current03ce core20 sealed PASS (`20261006T082006.802284Z-0446edc8ea7b`, digest0829b7c9); remaining Tarjan PTA stage compiling. Main has this1 candidate path; other86 are not imported yet.

### Current-source custody refresh after Index corpus Visitor

- Index `38eda80a5688c6b8de6ccd0faef886179c9ddb0f` already moved `CorpusKeySeedV1`'s direct `Visitor` impl from `DeserializeSeed::deserialize` to module scope. The `visit_str` field loop and ordinary/native key policy remain the same in source. This is an upstream Index fix, not a new substrate implementation. Index `8fdaa95115e03a721376c25e63d7df77d1cf7f66` differs from `38eda` only in two documentation files. The earlier private `93c85cf2` base-native run waited 1,028 seconds and was interrupted (130) before admission; tests 0. Current Index owner state `/tmp/oct6-index-current-owner-proof-state.json` is waiting/running base-native with results 0. Neither source move nor old private wait is an owner-test PASS.
- Previous frozen attempt `/tmp/oct6-current-substrate-proof-attempt5.json` used S `03ce6dec1eddd7dcd5684b459d7394f626251899`/base `7a887ec6cc0a43c751225983a3dd3fa20eff6297`, G `2e8693e82049a5d545f2045b382c6fb149a5c977`, Index `a72a85df2f70e1e2ca943afb5f62ad62dcae9b6d`. Wire serde 65 and default 46 plus SCC core 20 have sealed PASS on that tuple. Tarjan PTA failed in Index `native_decode_v1.rs:190` due to a nested `CorpusKeySeedV1` Visitor impl (`non_local_definitions`); PTA cases 0, RuntimeAnalysis not reached, later Shape/SDK gates NOT_RUN. This is failure history, not proof for the refreshed tuple.
- Clean S candidate `5fe8de9549a9d4349117b9d01010eeddd9f8f6b0`/base `2d439969e494071779648a2b48a29e477c57d62f` has 86 owned paths. The role-source-axes facade prerequisite `65d1d94ed96b6b1a471cd01c5a728a45a6713aa3` is already in main, so the candidate drops that one path. The current frozen sibling tuple is G `614d9de1dfb304d1721d59d6f8a1f0c124574176` and Index `8fdaa95115e03a721376c25e63d7df77d1cf7f66`, no sibling dirty overlay, topology `5fe8de9549a9d4349117b9d01010eeddd9f8f6b0+ext-2b460d8597fc36d4df41`.
- The `5fe8de` Wire serde attempt ended exit 74 after QBC reported `verification source changed during QBC execution`; the snapshot shows ` M Cargo.lock`, and no immutable completion locator was published. The observed lock delta is the Index unicode vendored dependency closure prerequisite; root is repairing canonical committed lock inputs before a fresh source freeze. This attempt has **0 accepted test verdicts**, despite reaching receipt finalization. Wire default, Tarjan/PTA, Shape/runtime/SDK, generated retirement, and full product gates are NOT_RUN on this tuple. Do not compose the old `03ce` PASS receipts with this failed snapshot.
- Semantica main subsequently imported the canonical unicode Index lock prerequisite as `968e57361399c2066c165479222189bbdfb40a46` (eight dependency identity qualifications and path package identity; package versions/checksums unchanged). The clean candidate was rebased as `a342228343d437a5eea1c7ff0715d29b0b70b19e`/base `968e57361399`, retaining 86 paths; its selected owner/consumer proof is pending. The old `5fe8de` exit 74 remains historical and supplies no accepted test result. S main import of the remaining candidate and current-main qualification are pending. No new engine, SDK library, schema, or public DTO is introduced by this custody repair. Global Honesty retirement remains blocked by the owning full-sweep ratchet; no baseline waiver/write is claimed. Native installed product/HTTP/stdio verification remains NOT_RUN.

### Current combined candidate after Wire capacity checkpoint

- GQLang의 마지막 확인된 committed source는 `809bf17bc7e70259b2d9dc46b537291213be4380`이다. 현재 문서 편집은 별도 dirty overlay다. S main `3ce7f396346b6a532a2229c6ffe36b79cd351380` fixed local `session_v1` name shadowing in the session fixture constructor, then `3188fe799218cff227e7cfc4ae36598eb2c7eefa` retained the owned Java parameter `Vec` in the Wire census and reused `selected_java_parameter_spare_capacity_is_retained_but_not_cloned_v1`. No second prototype test remains.
- S `a342228343d437a5eea1c7ff0715d29b0b70b19e`/base `968e57361399c2066c165479222189bbdfb40a46` Wire serde failed E0599 at `clone_logical_cost_v1.rs:483` (`capacity()` on `&[String]`), exit 101 and 0 tests. Receipt `20261006T084744.146218Z-4588ee831ad8`, SHA-256 `edc710bded333e82197d6fdb9af2ea58622b7dfe8798ce634a178aed0ec7a721`, source digest `528d7853e1c932411954db747ce8d77f271c44df6ee9ae31b1aa032a26528918`; clean source, immutable failed completion. Record `/tmp/oct6-current-substrate-proof-attempt7.json` is failure history, not a PASS.
- The current private candidate `d14a8760c0397a78765636728d38ce9c4cd3d82c`/base `3188fe799218cff227e7cfc4ae36598eb2c7eefa` is clean with 86 paths. Three-file resume-cost rebase kept main owned `String`/`Vec` capacity and inline enum accounting while preserving separate clone-birth Arc and original-session work controls. Twelve impacted boundary policies passed after conflict resolution; no Rust behavior verdict follows from that preflight. Fresh full QBC materialized topology `d14a8760c0397a78765636728d38ce9c4cd3d82c+ext-a5f1840e0a7b23f04d4c` with archived G `809bf17bc7e7` and Index `a80dc1ca5834`; Wire serde is RUNNING and selected results remain 0 until an immutable receipt.
- Index `38eda80a5688c6b8de6ccd0faef886179c9ddb0f` base native owner completed 63 passed, 0 failed/ignored/filtered (`/tmp/oct6-index-current-base-native.log`); corpus controls remain waiting without verdict. D1/D2 runtime, SDK, PTA, Taint, main qualification and native installed product checks remain unproven on the current combined source. Global Honesty retirement remains BLOCKED.

### Current source-only witness and main import delta

- At frozen `d14a8760` topology, Wire serde sealed 68 passed/0 failed, receipt `20261006T091201.973825Z-7fead918cc19` SHA-256 `f115ffdf9ba28e84f092636c36c992a59f7746681c1ccc1e5dbc6c1f7568e553`, source digest `f5d1baca4b21fa86c9595f110ad8c617eeb050934c4de0a8e2a9a46aed39966a`; it includes the retained Java parameter capacity regression. Wire default is running. No downstream verdict follows yet.
- `/tmp/oct6-final-retirement-witness.json` is read-only d14/base3188 GOD evidence: four selected paths, baseline14→7, retired7/new0/worse0, active Cost Shape symbol0, allowances/outside-baseline entries unchanged. No generated-file write or global Honesty pass.
- `/tmp/oct6-final-main-delta-review.json` examined S main `03c9bfa8e926a36278c93a95cf3a0e9804e9800f` against candidate base3188: 76 changed main paths and 5 THREE_WAY overlaps in runtime ingress, SDK exports/injector and composition Cargo/port impls. Main subsequently moved to `0c823f3f68c81e11688056d8e9124660a310cc57`; refresh the exact delta and reconcile stronger main behavior before remaining86-path import. The d14 frozen result remains valid for its own tuple only; merged source needs affected owner/consumer requalification.
