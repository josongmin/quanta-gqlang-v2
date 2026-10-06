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

Sibling dependency symlink는 live 원래 checkout을 가리킨다. Rust 결과는 해당 QBC source snapshot/외부 dependency topology에 묶인다. Python 실행은 `PYTHONPATH`와 module `__file__`로 구현 worktree 사용을 확인한다. 공유 `.venv`가 외부 작업으로 삭제된 뒤에는 그 이전 결과와 복구 환경의 새 실행을 구분한다.

## 2. 관찰 상태 — 2026-10-06 진행 중

| 항목 | 구현 / 실제 실행 관찰 | main 상태 / 잔여 |
|---|---|---|
| D3 | DFG 삭제 diff; DFG owning QBC 91/91 GREEN. Pack owning QBC 16/16 GREEN. Pack의 기존 borrowed/owned E0308를 exact-file `.clone()`로 선행 수리 | 미반영. 실제 taint promotion owner proof, checkpoint diff review/commit 대기 |
| D5 / D4 | leaf에서 이어서 작업 예정 | NOT_RUN / 미반영 |
| D7 | `7fa7d57b5ee6a0c6e3fb2cfa26cd0f5e51a9856f` private commit. Bounded capture + immutable source + metadata/dispatch/init 연결. Agent가 전체 MCP suite GREEN, 새 focused file 19 cases를 보고. Final 1-case 추가 후 dependency-light 재검증 보고 | 미반영. Root diff review·canonical commit fences·fresh main delta 검증 대기. Agent report를 installed qualification으로 세지 않음 |
| D1-A | standalone factories/direct dependencies와 고유 tests 이동 diff. Metadata 443 packages, runtime structure checker GREEN 보고 | QBC는 새 Cargo edge에 따른 locked admission 실패. Cargo.lock 갱신 후 owner run 필요. 미반영 |
| D1-B | cfg/forwarding/SDK/gate 영향 분석 | 구현·feature closure proof 미완료 |
| F1 | projector/native fixtures diff. Current fixture+variants 21 cases GREEN. Typed LexicalSearch DTO와 기존 public SDK cases 전환 중 | 마지막 DTO diff 이후 전체 affected proof NOT_RUN. 미반영 |
| D6 | F1의 current schema 확정 후 소비자 전환 | 구현·owner/consumer proof 미완료 |
| D2 | D7 checkpoint 다음 배정 | 공통 core/PTA/Taint 모두 미완료 |

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
