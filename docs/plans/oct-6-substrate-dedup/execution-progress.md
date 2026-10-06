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
| D7 | root final focused 33/33 GREEN; captured source/duplicate/override/init controls | Main `b258f5919a5`; 7 owned paths만 checkout custody 갱신, canonical Ruff/safety PASS |
| D1-A | private `8b35a93b2fe`; frozen sibling exact selected self-loop-tail QBC 1/1 GREEN, earlier module16/16은 diagnostic | 미반영. Full owner/consumer proof와 A/B feature closure 필요 |
| D1-B | cfg/forwarding/SDK/gate cutover diff; metadata/structure checker GREEN. Runtime와 SDK executor Shape-only normal deps에서 Cost 부재 | 미반영. 실제 composition/executor Shape port 전달·trait/injector/snapshot pin 누락도 보완. Runtime/SDK owning QBC 대기 |
| F1 / D6 | private `d0cb3969994` → rebased `5cc4a2c9972`; owned39 inputs unchanged. Affected SDK213 /211 pass /2 skip, rebased consumers142 /141 pass /1 skip, scripted installed SDK wheel1/1·actual Granian HTTP process+scripted daemon1/1 GREEN | Main `c1f821db572`;39 paths,+1434/-1399. Canonical Ruff38/safety PASS; original owned paths clean. Real native daemon/product qualification은 NOT_RUN |
| D2-P/K | private `4715ff4c4e5`; graph-utils QBC 19/19 GREEN; shared borrowed iterative core | 미반영. PTA/Taint cutover/consumer proof 필요 |
| D2 PTA | wrapper와 caller typed Result 전파 diff; owning lib check GREEN | lib-test compilation은 기존 13개 unrelated errors로 RED. SCC case 성공으로 세지 않음 |
| D2 Taint | successor-only adapter / typed resource mapping / contract caller 보완 진행 | public integration QBC 진행 중. 별도 contract crate와 ignored 50k cases 필수 |


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
- F1/D6 main import: `c1f821db572b2b1c1f433ba57bdaea3f6fc9b0f8`. Rebase 후 owned39 파일 bytes 동일; MCP registry가 갱신된 조합에서 consumer142를 재실행했다. SDK/installed scripted receipts는 이전 source 결과로 유지하며 actual native daemon proof로 확대하지 않는다.
- D3/D5/D4 main import: `d88f0a2ed7a64f38ae8ac97badbb9671a759e10c`. 14 paths, +65/-1667. Foreign dirty를 stage하지 않았다. Deleted 파일도 이전 committed bytes를 확인한 후 original checkout에서 퇴역시켰다.
- Search의 CLI는 퇴역 reader를 실제 사용했다. Current registered SDK ingress로 전환하고 logical repo ID와 attached repo path를 구분했다. Unsupported repo/snapshot/type를 attach 전에 거절한다.
- SDK `with_status=True`의 native receipt absence가 중간 projection에서 빠져 호출이 실패했다. Existing ProductResponse metadata/receipt owner로 `receipt=None`를 정확히 보존하고 native degraded reason을 existing receipt pair에 전달한다. Receipt를 발급하거나 owner truth를 합성하지 않는다.
- HTTP search/QueryAdapter/CLI가 native degraded outcome/reason을 보존한다. Shared converter는 data-only `{hits,summary}`만 받으며 transport/envelope parser가 아니다.
- PayloadBuilder는 absent file path를 `unknown` 파일 후보로 만들지 않고 native point/highlight를 full source range로 승격하지 않는다. Authored complete source range만 보존하고 malformed range를 거절한다.
- Search owner의 새 복잡도/크기 증가는 함수 책임을 분리해 제거했다. Rebased commit의14 production paths를 최초 worker base와 동일 AST scanner로 비교: findings44→36, 신규0·growth0. 기존부터 누락된 baseline13개와 퇴역 keys11개는 별도다. 무조건 baseline refresh로 새 위반을 수용하지 않는다.
- Scripted wheel/HTTP fixture는 현재 generated response의 `committed_mutation_v1=None`/receipt absence를 명시한다. HTTP fixture의 Python3.9 hardcode는 SDK minimum3.10과 충돌하므로 실행 test interpreter로 전환한다. Scripted daemon은 actual Rust daemon 실행 증명이 아니다.

## 6. 미해결 검증과 scope

- D1-B actual runtime/SDK pure·PTA·disabled·Cost lanes, 두 Shape gates. Heap feature의 explicit Cost edge는 보존한다.
- D2 PTA existing compile blockers의 committed-main/foreign-dirty 구분 및 owner rail, Taint public contract/lib policy/typed resource rejection/deep50k proof. 첫 Taint iterative5 pass/1 ignored·SCC14 pass 결과는 변경 전후 source별 component proof다.
- D2 core admission의 workspace estimate는 actual frame/output allocation을 충분히 계산하지 않는 결함이 추가 확인됐다. Checked conservative bound와 tiny-limit refusal/positive control로 수정·core owner proof 재실행 후 반영한다. Default 실사용 OOM을 증명한 것은 아니다.
- F1/D6 main import와 rebased affected consumers proof 완료. Generated baseline의 퇴역 keys 정리는 전체 source checkpoint 후 수행한다. Actual native daemon/product proof는 NOT_RUN.
- Shared generated inventories/baselines는 삭제/이동한 실제 keys만 retire/reconcile하고 새로운 scope waiver를 추가하지 않는다.
- Repository/product-wide qualification은 별도다. Affected SDK의 rename/rollback selectors는 이 packet 밖의 기존 failures 때문에 제외한 rail이며 full SDK suite GREEN으로 표시하지 않는다. 별도 실행한 GQLang receipt-wins tests 4개도 기존 profile precheck에서 실패했으며 이 변경의 성공 receipt로 합산하지 않는다.
- F2/F4/F5/F6/F7/F9/F10은 여전히 별도 기능·증명 backlog다.

GQLang local `just ci` 재실행은 **all groups passed**, source snapshot `83f930a4bf86250e925e16386c28d3e8e03e12af315569b293e7a62479e031f9`다. 첫 실행은 historical audit의 SHA/digest를 typos가 단어로 분해한 4개 false positives에서 중단됐다. Hex identifier7–64 chars만 typos 식별에서 제외하고 ordinary prose typo 거절 control을 확인했다. 이 결과는 실행 시점의 GQLang dirty snapshot에 한정하며 Semantica runtime/product qualification 결과가 아니다. 실행 로그: `/tmp/oct6-gqlang-local-ci.log`.
