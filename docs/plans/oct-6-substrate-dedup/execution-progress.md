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
| D2 Taint | successor-only adapter / typed resource mapping / contract caller 보완. Current private `1ea51f05dc5`에서 iterative5 pass/1 ignored·SCC property14/14·policy units13 pass/1 ignored QBC 최종 GREEN | Contract·실제 ignored50k·기존 gate6단계 proof와 fresh-main import pending |


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
