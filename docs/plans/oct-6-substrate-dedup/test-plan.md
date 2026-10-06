# 테스트 플랜 — fixtures, expected results, selectors

상위 계획: [readme.md](readme.md). Write set / Before·After: [code-change-map.md](code-change-map.md).

**이 문서는 required proof matrix다.** 아래 `기존`은 source에서 확인한 test 이름/파일이며 목록 자체는 통과 증거가 아니다. 구현 착수 후 실제 실행/receipt/main 상태는 [execution-progress.md](execution-progress.md)에 기록한다. `추가`는 계획상 새 회귀이며 진행 상태는 별도 ledger를 따른다. 기존 meaningful cases로 요구를 충족하면 동일 test를 다시 만들지 않는다.

## 1. 실행 계약과 증거 단계

| 단계 | 하는 일 | 무엇을 증명하는가 | 완료 판정 |
|---|---|---|---|
| T0 source / retirement | related hashes, caller/export/feature census, deleted path 확인 | 실제 연결·삭제 범위 | runtime 결과 증명 아님 |
| T1 owner | independent expected positive/negative cases | owner 계산·거절·control | Rust는 owning QBC receipt 필요 |
| T2 consumer | 실제 projector/factory/converter/route 경로 | owner 선택과 consumer 의미 보존 | patched converter만 실행하면 불충분 |
| T3 policy / mandatory gates | feature/layer/boundary/generated selector | dependency와 재발 방지 | nonzero selection, baseline weakening 없음 |
| T4 product, 필요할 때 | actual daemon/HTTP/stdio/installed artifact | 해당 제품 경계 | source tuple/installed identity 별도 기록 |

동작을 보존하는 dead-code 삭제는 T1 기존 regression + T0/T3를 사용한다. D1/D6/D7의 진입점 전환에는 T2가 필요하다. T4를 실행하지 않은 checkpoint는 installed qualification을 주장하지 않는다.

## 2. Rust 실행 경로 — 실제 CLI 문법

실행 cwd는 `S = /Users/songmin/Documents/code-new/semantica-codegraph-v2`다. Syntax authority: [QBC request grammar](../../../../semantica-codegraph-v2/tools/quanta-build-cli/qbc_request_scope_v1.py)와 [QBC 안내](../../../../semantica-codegraph-v2/tools/quanta-build-cli/README.md).

### 2.1 기존 owner/unit selector의 resolve → run

현재 CLI에 `--compile-policy`가 필수다. `--features`를 임의로 붙인 owner command를 만들지 않는다.

```bash
cd /Users/songmin/Documents/code-new/semantica-codegraph-v2

# Noncompiling preview. Package/features/scope/source attestation을 먼저 검토한다.
scripts/quanta-build-cli owner resolve \
  --package quanta-graph-utils-v1 --execution-kind test --target-kind lib \
  --selector-mode module --selector scc \
  --compile-policy package-all-features --json

# 위 preview의 owner scope가 허용 가능할 때 final owner run.
scripts/quanta-build-cli owner run \
  --package quanta-graph-utils-v1 --execution-kind test --target-kind lib \
  --selector-mode module --selector scc \
  --compile-policy package-all-features
```

`package-all-features`는 실제 CLI policy다. Leaf/Shape-only proof에는 적합하지 않다. 아래 table의 package/selector를 같은 grammar에 대입하되, 최소 의미 있는 isolation policy가 있으면 `feature-isolation:<resolved-class-id>`를 선택한다. Isolation ID는 content-bound authority이므로 임의 문자열이나 과거 ID를 발급된 class처럼 문서에 고정하지 않는다.

| checkpoint | package | target / selector candidate | compile/feature proof 조건 |
|---|---|---|---|
| D3 pack | `quanta-runtime-memssa-pack` | lib / module `canonical_memory_ir_ingress` | resolved MemSSA runtime-pack authority 확인 |
| D3 DFG | `quanta-adapters-dfg` | lib / module `memssa_adapter` | PortDFG cases 실제 선택, 필요한 DFG closure |
| D5 | `codegraph-core-pta-engine` | lib / module `solver::steensgaard`; module `solver::andersen` | base/active solver와 retained-pair cases 확인 |
| D4 | `quanta-runtime-retrieval-kernel` | lib / module `scv2_exact_fusion` | **no-default + scv2-exact-owner-surface only** isolation owner proof 필요 |
| D1 standalone | `quanta-adapters-shape` | lib / module `shape_analyzer_adapter` | pure lane와 `pta-rail` lane를 구분 |
| D1 SDK | `quanta-runtime` | integration / `sdk_python_shape_analysis_test` | D1-B pure/PTA/disabled/Cost combinations의 각각 resolved recipe |
| D1 PTA converter | `quanta-adapters` | integration / `pta_to_shape_input_contract_test` | test `required-features`와 inner cfg 함께 갱신 |
| D2 generic/indexed | `quanta-graph-utils-v1` | lib / module `scc` | generic + indexed controls 모두 실행 |
| D2 PTA | `quanta-runtime-analysis` | lib / selector matching `precise_query_scc` | current compiled module path·PTA feature closure를 resolve에서 확인 |
| D2 taint | `quanta-taint-solver` | integration `tarjan_iterative` / SCC properties + lib `wire_recursion` | `cfg-dfg` 포함 owning closure; policy/resource unit과 ignored deep case 별도 |
| D2 taint contract | `quanta-taint-solver-contract-tests` | integration `tarjan_scc_contract_test` | public Result caller와 successor-only domain proof |

위 selector는 source namespace 또는 기존 gate substring에서 확인한 후보다. 이번 상세화에서 `owner resolve`/test enumeration은 실행하지 않았다. **resolve refusal, cfg로 0 cases, class 미등록이면 DoR 미충족**이다. 착수 시 registry/recipe owner에서 isolation class를 정리한 뒤 exact resolved command·test case count를 ledger에 저장한다. Broad all-features 성공으로 leaf closure를 대체하지 않는다.

### 2.2 feature closure를 위한 비컴파일 diagnostic

이미 등록된 lane을 사용한다. 실행 전에 `DEDUP_EXISTING_LANE`을 실제 task lane 이름으로 설정한다. 이 문서 작업에서 새 lane을 만들지 않는다.

```bash
scripts/quanta-build-cli cargo --lane "$DEDUP_EXISTING_LANE" -- tree --locked \
  -p quanta-runtime-retrieval-kernel --no-default-features \
  --features scv2-exact-owner-surface -e features
```

D4 expected: algo helper dependency는 활성, `identity-collapse-surface`는 비활성. 실제 package graph를 함께 확인한다. 이 tree 결과는 feature 진단이며 Rust test receipt가 아니다. D1-B도 해당 resolved Shape-only recipe의 same-feature dependency tree와 SDK tests를 함께 기록한다.

## 3. D3 — MemoryIR

Existing location: [pack ingress tests](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-memssa-pack/src/canonical_memory_ir_ingress/tests.rs). DFG의 PortDFG controls는 [canonical_ir.rs](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-dfg/src/memssa_adapter/canonical_ir.rs)에 남긴다.

| ID | 기존 / 추가 | 입력 | independent expected / 실패 검출 |
|---|---|---|---|
| M1 | 기존 `canonical_memory_ir_from_build_input_bare_entry_preserves_real_load_value_def_id_v1` | valid real load ID | 실제 ID 유지, surrogate로 교체 금지 |
| M2 | 기존 `canonical_memory_ir_from_build_input_bare_entry_auto_remaps_synthetic_load_below_floor_v1` | synthetic load | permitted synthetic surrogate 및 현재 typed reference/digest 유지 |
| M3 | 기존 `canonical_memory_ir_from_build_input_bare_entry_skips_used_real_id_for_synthetic_surrogate_v1` | used real ID와 synthetic 후보 충돌 | real ID를 침범하지 않는 surrogate, duplicate value identity 없음 |
| M4 | 기존 `canonical_memory_ir_from_build_input_with_load_value_def_ids_allows_synthetic_load_v1` | explicit override map | authored map 적용, fabricated fallback 없음 |
| M5 | 기존 `canonical_memory_ir_from_build_input_with_load_value_def_ids_rejects_missing_synthetic_override_v1` | 필요한 override 없음 | typed error, partial/success IR 없음 |
| M6 | 기존 missing store/load derivation tests | authority 없는 store/load | 각각 typed refusal |
| M7 | 기존 `canonical_memory_ir_mem14_producer_permutation_preserves_identity_v1` | 같은 facts를 순서만 변경 | canonical identity/digest 동일 |
| M8 | 기존 DFG `deletion_adapter_bridge_preserves_kill_and_rejects_incoherent_map` | PortDFG deletion valid/invalid | 별도 PortDFG contract 보존; shared helper 오삭제 검출 |

T2: 실제 `memssa_input_promotion` consumer의 owning cases로 pack 경로 확인. T0: retired symbols는 **DFG scope**에서 definition/export 0, pack의 같은 이름은 정상이다. `DeletionAuthorityViolation` 다른 caller가 남으면 error 삭제를 강제하지 않는다. 새로운 mirror test는 필요 없다.

## 4. D5 — active PTA 보존과 dormant 삭제

Existing sources: [base tests](../../../../semantica-codegraph-v2/packages/core/pta/engine/src/solver/union_find_base_tests.rs), [Steensgaard soundness](../../../../semantica-codegraph-v2/packages/core/pta/engine/src/solver/steensgaard_tests/basic_and_soundness_v1.rs), [retained pair tests](../../../../semantica-codegraph-v2/packages/core/pta/engine/src/solver/steensgaard_retained_pairs_tests.rs).

| ID | 기존 control | expected |
|---|---|---|
| U1 | `test_copy_unification`, `test_chain_copy`, `test_no_alias` | 같은 allocation의 copies는 alias, 독립 allocation은 분리 |
| U2 | `test_load_store`, unknown load/store soundness cases | field-insensitive active solver의 sound over-approximation 유지 |
| U3 | `retained_ten_thousand_plus_one_publishes_only_one_pair_v1` | 기존 10k pairs 다음 append에서 새로운 pair만 한 번 publish |
| U4 | `retained_shared_allocation_updates_every_owner_without_reemitting_old_pairs_v1` | 모든 affected owners 갱신, old pair 재발행 없음 |
| U5 | `retained_projection_checks_control_before_member_and_pair_work_v1` | control refusal 후 member/pair work 진행 없음 |
| U6 | `retained_alloc_copy_prefixes_match_legacy_native_oracle_v1` | retained prefix projection이 기존 native independent oracle과 일치 |

T0: deleted UF files 없는지, Andersen module/root export/alias/glob/feature targets의 retired consumer 0인지 확인. Current Andersen solving tests도 실행한다. Dormant UF를 위해 새 test나 bool→root 호환 layer를 추가하지 않는다.

## 5. D4 — score golden과 fusion invariants

기존 controls는 [SCV2 tests](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-retrieval-kernel/src/scv2_exact_fusion.rs)에 있다. 아래 expected는 helper를 호출해서 계산하지 않는다.

| ID | 기존 / 추가 | 입력 | expected |
|---|---|---|---|
| R1 | 추가 rank golden | rank 0 / 1 / 2 / `u32::MAX` | 각각 `1/61`, `1/61`, `1/62`, `1/4_294_967_355`; rank 0 clamp 보존 |
| R2 | 추가 mixed contribution | stable sequence ranks `[1,2,1]` | sequential binary64 expected `0.048915917503966164`, bits `0x3fa90b81d3dde27e`; 임의 fold/order 교체 검출 |
| R3 | 기존 `exact_fusion_equal_score_cross_owner_tie_uses_canonical_identity_order_v1` | 같은 score, 다른 owner identity | 기존 exact identity tie order |
| R4 | 기존 `contribution_permutation_is_byte_stable_v1` | 동일 facts permutation | admitted/canonicalized input 결과 bytes 동일 |
| R5 | 기존 `conflicting_duplicate_contribution_fails_closed_v1`, `exact_duplicate_contribution_collapses_once_v1` | conflicting / identical duplicate | conflict refusal / single contribution |
| R6 | 기존 `exact_fusion_truncates_and_renumbers_final_rank_v1` | result count > top-k | 기존 truncation·dense rank 보존 |
| R7 | 추가 또는 existing isolation owner | no-default + only `scv2-exact-owner-surface` | helper available, no accidental identity-collapse activation, nonzero R1–R6 |

Floating-point expected는 허용할 현재 precision을 명시한다. Bit preservation 주장은 기존 expression과 같은 f64 operation/accumulation order를 고정한 cases로 확인한다. Broad relative epsilon 하나로 tie-break 변화까지 허용하지 않는다.

## 6. D7 — captured source와 init/retry

Reuse/modify existing files: [test_introspection.py](../../../../semantica-codegraph-v2/apps/mcp/tests/test_introspection.py), [test_surface_config.py](../../../../semantica-codegraph-v2/apps/mcp/tests/test_surface_config.py), [test_mcp_capability_parity.py](../../../../semantica-codegraph-v2/apps/mcp/tests/test_mcp_capability_parity.py), [test_query_surface_contract.py](../../../../semantica-codegraph-v2/apps/mcp/tests/test_query_surface_contract.py), [test_mcp_sdk_di_lifecycle.py](../../../../semantica-codegraph-v2/apps/mcp/tests/test_mcp_sdk_di_lifecycle.py).

새 focused file 제안: `S/apps/mcp/tests/test_crosswalk_source_snapshot.py`. Pure registry fixture와 cold-process controls를 넣고 기존 mega surface tests의 동일 assertion을 복제하지 않는다.

| ID | 입력·제어 | expected | 검출할 faulty behavior |
|---|---|---|---|
| C1 | explicit manifest A 로드 후 file을 B로 교체; names→mapping→config→schema 호출 | 한 process에서 A version/digest/entries, restart 뒤 B | 같은 path 재읽기로 A names/B mapping 혼합 |
| C2 | 동일 tool/usecase entry 두 번 | duplicate typed refusal | dict overwrite/set 비교로 duplicate 숨김 |
| C3 | `search_symbol`/`find_symbol` 같은 canonical alias + colliding entry | canonical collision refusal | raw spelling만 검사 |
| C4 | public executable / deferred public / metadata-only entries | projections별 올바른 subset, source digest 동일 | metadata 삭제 또는 deferred dispatch 승격 |
| C5 | public dict/list를 caller가 mutation | 다음 read/private capture는 불변 | 공유 mutable payload 변조 |
| C6 | explicit missing path + 다른 cwd/env의 valid manifest | explicit override refusal, SDK factory calls 0 | fallback으로 다른 authority 성공 |
| C7 | invalid JSON / missing required entries / invalid schema/deps | typed preflight refusal, SDK calls 0 | fake empty registry 또는 late init failure |
| C8 | fresh subprocess direct `_initialize_sdk`; adapter module 미수입 상태 | capture/validate 먼저, 그 뒤 adapter import·factory | main-only preflight 우회 |
| C9 | 첫 SDK construction unavailable, `call_tool` retry | 같은 captured source 재사용; idempotent preflight | retry의 split capture 또는 preflight 생략 |
| C10 | packaged mode의 명시 artifact; repo checkout/cwd 없음 | chosen artifact로 projections 성공 | repo-root 탐색만 가능한 packaging |
| C11 | configured bytes/entry limit의 boundary / +1 | limit 내 성공, 초과 typed refusal before oversized read/decode | `read_text()` 뒤 길이 검사로 allocation bound 주장 |
| C12 | actual ordered public inventory | exact order·count·unique cardinality, list/schema/dispatch 대응 | set equality만 통과 |

Existing controls: `test_mcp_list_tools_matches_filtered_surface_config_ssot`, `test_explicit_mcp_public_inventory_matches_schema_inventory`, `test_surface_config_load_raises_when_crosswalk_unavailable`를 ordered/version controls로 보완한다. Config refresh가 same capture를 재투영하는 것도 확인한다. Test cache reset은 fixture/private process lifecycle에만 두고 production hot-reload API를 새로 만들지 않는다.

현재 `test_introspection.py`에는 `CODEGRAPH_REPO_PATH`의 `target/quanta-evidence/crosswalk.json` 탐색과 missing load의 empty diagnostic dict를 기대하는 cases가 있다. D7의 explicit source/typed refusal 계약에 맞춰 override fixture·expected error를 함께 전환한다. Minimal metadata-only test fixture도 captured full manifest schema를 충족하도록 갱신한다. 새 validation을 우회하거나 fallback을 재도입해서 old assertions를 통과시키지 않는다.

## 7. F1/D6 — native-shaped inputs부터 consumer까지

### 7.1 fixture 작성 위치와 oracle

- Existing SDK [fixture owner](../../../../semantica-codegraph-v2/packages/sdk/python/codegraph_sdk/tests/_current_product_response.py), [test_current_product_response.py](../../../../semantica-codegraph-v2/packages/sdk/python/codegraph_sdk/tests/test_current_product_response.py)를 current native writer 기준으로 수정한다.
- 추가 제안: `S/packages/sdk/python/codegraph_sdk/tests/test_search_product_current_variants.py`, `fixtures/search_primary_file_product_v1.json`, `fixtures/search_supplemental_product_v1.json`, `fixtures/search_mixed_product_v1.json`.
- App shared test 제안: `S/packages/analysis/codegraph-shared/tests/test_current_search_projection.py`.
- Golden fields는 실제 native ProductResponse typed writer/serializer에서 확인한 variant/type/tag로 만든다. 예상 결과는 hard-coded identity/span/summary assertion으로 검증한다. Consumer가 요구하는 `symbol_id`를 producer fixture에 임의로 추가하지 않는다.
- F1의 writer 확인으로 current SDK `data={hits,summary}`를 확정했다. Native field 이름/optional values/variants를 보존한다. `page_count=match_count=primary returned rows`, supplemental은 별도 count다. 전체 상세 계약과 현재 diff 상태는 [checkpoint §3](execution-progress.md#3-f1에서-확인한-구체적인-계약)를 따른다.

### 7.2 required fixture matrix

| ID | native/current input | expected SDK + shared/app result |
|---|---|---|
| S1 | primary match, 실제 symbol metadata 있음 | authored `symbol_fqn/symbol_kind/runtime_symbol_locator/symbol_discriminator` 보존, score·count 의미 일치. Runtime locator를 canonical symbol ID로 바꾸지 않음 |
| S2 | primary file/text match, symbol 없음 | 정상 nonsymbol variant; fake symbol ID 없음, 허용된 file/content presentation |
| S3 | supplemental-only | supplemental variant와 실제 identity 보존; primary로 오분류 없음 |
| S4 | mixed primary+supplemental, distinct counts | native total/page/primary/supplemental count 정의 유지; 근거 없는 total>=all-row assertion 금지 |
| S5 | page rows empty but overall total nonzero; result_truncated true | SDK/app summary에 authored total/truncation 보존; `len(rows)`/false fallback 금지 |
| S6 | line/range 없음 | missing/optional 상태 보존; line 0/end_line/start_column fabrication 없음 |
| S7 | native `line=10,column=3`, optional `highlights=[{start:4,end:9}]`; 별도 app metadata full span `{start:10:3,end:12:7}` control | native coordinates/byte offsets 정확 보존, end coordinates 생성 없음; 이미 authored된 app full span은 PayloadBuilder까지 정확 보존. Native writer는 full source span을 발급하지 않음 |
| S8 | wrong operation/status/field variant/type; malformed required identity | typed SDK/app rejection; 정상 empty success로 전환 없음 |
| S9 | degraded/not_computed/failed native response | 기존 status/reason/failure projection 보존 |
| S10 | GraphRAG seed file IDs, configured no-graph / graph failure | seed identity/file presentation·existing typed degradation 보존 |
| S11 | live public MCP generated-envelope request | 기존 native authority envelope 보존; HTTP agent helper를 우회하는 별도 control |

### 7.3 actual boundary chain

```text
native-shaped current ProductResponse fixture
  → generated decode / public SDK search operation projector
  → shared current SDK row→app conversion
  → API actual route response assembly / HTTP agent actual QueryAdapter
  → metadata span / summary / GraphRAG file presentation assertions
```

Fake transport는 첫 입력 seam에만 둘 수 있다. SDK projector/shared converter/_execute_search를 대체한 test를 이 chain의 proof로 세지 않는다. Actual HTTP/daemon fixture가 아니면 T2 component receipt로 명시한다.

Existing consumer locations:

| 파일 | 기존 test / 보완 |
|---|---|
| [API canonical search](../../../../semantica-codegraph-v2/apps/api/tests/test_search_route_canonical_surface.py) | `test_search_hybrid_uses_canonical_hybrid_surface`, `test_search_route_rejects_removed_hits_projection`, request rejection/anchor controls; current variants를 route boundary로 주입 |
| [QueryAdapter](../../../../semantica-codegraph-v2/apps/mcp/tests/test_query_adapter.py) | `test_hybrid_search_accepts_canonical_symbol_projection_from_sdk`; nonsymbol/mixed/summary refusal를 추가 |
| [span controls](../../../../semantica-codegraph-v2/apps/mcp/tests/test_query_adapter_span.py) | `test_to_dict_preserves_nested_metadata_structure`, `test_to_dict_without_span_metadata`, `test_fake_span_is_forbidden` |
| [GraphRAG](../../../../semantica-codegraph-v2/apps/mcp/tests/test_query_adapter_graphrag.py) | `test_graphrag_expansion_with_no_graph_returns_seed_hits`, `test_graphrag_expansion_failure_surfaces_degraded_reason` |
| [public MCP](../../../../semantica-codegraph-v2/apps/mcp/tests/test_query_surface_contract.py) | `test_mcp_adapter_preserves_runtime_authority_envelope_without_local_authority` |

Delete `SymbolRetriever`-only cases when module is retired; move a useful live exception/consumer control to its surviving owner. Do not keep dormant module just to keep its old tests green.

## 8. D2 — partition, exact output, control

Existing sources: [generic/indexed tests](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-graph-utils-v1/src/scc.rs), [PTA tests](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-analysis/src/queries/pta/precise_query_scc_tests.rs), [taint tests](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-taint-solver/src/analysis/taint/interproc/wire_recursion.rs).

| ID | graph/control | independent expected |
|---|---|---|
| G1 | empty / isolated `a` / self-loop `a→a` | respectively 0 / 1 / 1 components; self-loop cycle policy distinguishes cyclic |
| G2 | `a↔b`, `b→c`, closed keys `a,b,c` | partition `{a,b},{c}`. Generic/PTA emission `[{c},{a,b}]`; taint sorted output `[[a,b],[c]]`. Exact vectors required |
| G3 | indexed chain `[[1],[2],[]]` | same current dense IDs `[2,1,0]`, not merely three distinct IDs |
| G4 | independent cycles plus bridge/disconnected nodes | hard-coded partition + exact emission order/IDs, not only normalize/sort |
| G5 | generic `{a:{b}}` with undeclared b | `UnregisteredNeighbor`; no partial components |
| G6 | PTA `{a:{b}}`, only a declared | preserve PTA declared-key projection → singleton a, no synthesized b. Caller authority admission failures remain typed |
| G7 | taint `{a:{b}}`, successor-only b | admitted node set includes a,b → sorted singletons a,b |
| G8 | indexed neighbor >= node_count | typed compute error, no partial IDs/panic |
| G9 | unsorted/duplicate neighbor permutations | current public indexed wrapper’s sort/dedup semantics and dense IDs preserved; borrowed core must not expose arbitrary edge order |
| G10 | stop at each admitted indexing/traversal/materialization checkpoint | caller's exact typed Stop, no partial or late success; control observed before affected work |
| G11 | node/edge bound +1, checked-size overflow, tiny workspace cap와 exact boundary | typed refusal before oversized dense traversal/result buffer allocation. Actual frame/header/member capacity를 계산; borrowed input·generic payload·allocator metadata/RSS까지 보장한다고 주장하지 않음 |
| G12 | chain with 50,000 nodes | 50k singleton components/valid IDs, iterative completion without recursion; explicit ignored test execution |
| G13 | taint cycle-policy fail_closed off/on and size/edge limits | original opt-in policy, independent size/edge refusal; output sort unchanged |

G2/G3 expected vectors are static traces of current deterministic root/neighbor ordering; bind them as independent literal goldens in the implementation rail. A test that calls the old/new implementation to manufacture expected values alone is insufficient.

Existing cases to keep/extend:

- generic: `unregistered_generic_neighbor_is_not_silently_removed`, `indexed_neighbor_outside_domain_is_an_error`, `deterministic_across_input_order`, `indexed_dag_yields_distinct_sccs`. Extend the indexed DAG assertion with exact `[2,1,0]`.
- PTA: `controlled_scc_every_checkpoint_stops_without_partial_result_v1`, `controlled_scc_handles_empty_and_deep_graph_boundaries_v1`. Existing normalized partition controls remain; add unnormalized G2/G4/G6.
- taint: `tarjan_iterative_successor_only`, `tarjan_iterative_disjoint_multi_root_determinism`, `recursion_policy_preserves_independent_cycle_budgets`, `tarjan_iterative_deep_chain_50k`.

Before gate는 taint `tarjan_iterative`와 ignored `tarjan_iterative_deep_chain_50k`만 선택했다. After gate는 core, PTA, taint iterative integration, SCC properties, `wire_recursion` lib policy/resource units, 별도 contract crate, ignored 50k의 7개 owner/selector를 선택한다. Integration target만 실행해 lib unit을 검증했다고 표시하지 않는다. Nonzero regular and ignored case counts are mandatory. Script comment의 test count는 실행 증거가 아니다.

## 9. D1 — Shape semantics와 feature closure

| ID | existing / new fixture | expected |
|---|---|---|
| H1 | standalone `single_node_tree`, `chain_tree`, `dag_two_incoming`, `cycle_detected` | exact Tree/DAG/Cyclic classification |
| H2 | cost `self_loop_remains_cyclic_when_the_node_also_points_to_a_tail` | singleton self-loop + tail stays Cyclic; unique regression moves before cost deletion |
| H3 | existing converter authority/overflow tests | canonical object authority preferred, digest-leak forbidden, overflow/context-stack mismatch typed refusal |
| H4 | `pta_to_shape_input_tree_chain`, `pta_to_shape_input_dag_two_parents`, `pta_to_shape_input_cyclic`, `pta_to_shape_input_hard_orphan_field_loc` | actual PTA graph/classification. Orphan field fixture는 base node 유지·가짜 edge 없음·다른 유효 edge 유지·orphan Tree. 허용된 empty points-to를 missing authority 오류로 바꾸지 않음 |
| H5 | actual runtime overlay + SDK session factory fixture; V2 valid/missing/mismatch authority | existing gateway core producer → standalone concrete owner → SDK port; correct result or typed refusal |
| H6 | pure Shape / PTA-derived Shape | both intended paths available with actual required features |
| H7 | Shape disabled | typed unavailable, no fallback Cost Shape construction |
| H8 | Cost / Termination / Heap active; Cost-only public Shape API control | original responsibilities/results remain reachable; existing Cost-only Shape API/capability preserved |
| H9 | Shape-only resolved Cargo closure | no unnecessary cost-analyzer activation; public SDK cfg/capability matches actual available implementation |

Owning locations: [standalone](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-shape/src/shape_analyzer_adapter.rs), [PTA contract](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters/tests/pta_to_shape_input_contract_test.rs), [SDK Shape](../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime/tests/sdk_python_shape_analysis_test.rs). Cost의 unique case는 최초 worker base의 `quanta-adapters-cost/src/shape_analyzer_adapter_tests.rs`에서 standalone owner로 이동한다.

Gate before/after:

| current selection | required after selection |
|---|---|
| cost `--lib shape_analyzer_adapter --features cost-analyzer` | standalone Shape unit tests; pta-rail converter cases separately selected |
| SDK tests with cost-analyzer feature | D1-B exact Shape-only + PTA-derived recipe; feature-required cfg no empty pass |
| Cost `gate_cost_shape_integration`, `extract_traversal_kind` | remain Cost owner checks with port supplied by surviving Shape where applicable |
| query-execution Heap integration | remain Heap owner checks; not deleted as Shape clone |

Cost-only actual overlay control은 기존 `runtime_ingress_shape_overlay_uses_standalone_adapter_v1` body를 재사용한다. Guard는 `any(cost-analyzer, shape-analyzer)`이며 command는 QBC를 통해 `cargo test -p quanta-runtime --lib runtime_ingress_shape_overlay_uses_standalone_adapter_v1 --no-default-features --features gateway-ingress-runtime-core,cost-analyzer --no-fail-fast --locked --offline`이다. 실제 1 case가 실행되어 Tree classification까지 도달해야 한다. Gate에 lane이 있다는 정적 확인은 이 행동 증명을 대신하지 않는다.

D1-A requires H1–H5 and relevant H8. D1-B additionally requires H6–H9, two retargeted Shape gates, feature policy/honesty regeneration. Factory cutover cannot close D1-B.

H5의 producer classification은 `runtime_ingress_shape_overlay_uses_standalone_adapter_v1`에서 factory가 반환한 실제 `overlay.shape_analyzer`를 사용한다. SDK `session_shape_factory_selects_runtime_overlay_v1`는 selected feature의 factory reachability만 증명한다. Capability/compatibility는 `shape_analysis_builder_public_under_cost_or_shape_v1`, `shape_capability_requires_compiled_builder_and_installed_port_v1`로 구분한다. `shape_pin_failure_is_typed_and_operation_attributed_v1`는 missing authority를 성공으로 만들지 않는 control이다. 이 결과들은 별도 boundary proofs이며 하나의 end-to-end session 실행 결과가 아니다. `session-shape-analyzer-runtime`이 실제 선택하는 기존 `gateway-ingress-runtime-core` producer를 사용하며 넓은 coordination feature를 추가하지 않는다. 이미 주입한 Shape port의 단순 forwarding만으로 producer proof를 대신하지 않는다.

## 10. Python execution commands

These commands target existing files. They are runnable selectors, **not executed results**. Use the installed Semantica SDK Python environment; if imports/toolchain/plugins fail, report that blocker instead of changing tests to mocks.

```bash
cd /Users/songmin/Documents/code-new/semantica-codegraph-v2

# D7: existing owner/consumer controls; add snapshot-focused file after authoring.
.venv/bin/python -m pytest -q \
  apps/mcp/tests/test_introspection.py \
  apps/mcp/tests/test_surface_config.py \
  apps/mcp/tests/test_mcp_capability_parity.py \
  apps/mcp/tests/test_query_surface_contract.py \
  apps/mcp/tests/test_mcp_sdk_di_lifecycle.py

# F1: existing SDK fixture/projection control; extend to actual native variants.
.venv/bin/python -m pytest -q \
  packages/sdk/python/codegraph_sdk/tests/test_current_product_response.py

# D6: actual consumer + span + GraphRAG + separate public MCP control.
.venv/bin/python -m pytest -q \
  apps/api/tests/test_search_route_canonical_surface.py \
  apps/mcp/tests/test_query_adapter.py \
  apps/mcp/tests/test_query_adapter_span.py \
  apps/mcp/tests/test_query_adapter_graphrag.py \
  apps/mcp/tests/test_query_surface_contract.py
```

Execute new focused files after they exist: `test_crosswalk_source_snapshot.py`, SDK `test_search_product_current_variants.py`, shared `test_current_search_projection.py`. Run selectors in the affected owner environment; do not assume unrelated apps' conftests can always compose into one pytest session. Preserve existing fixtures/setup when separating runs.

## 11. Retirement / guard acceptance matrix

| item | zero checks | preserved positive checks |
|---|---|---|
| D1 | cost Shape definition/module/export/factory 0 | standalone definition 1, actual consumers use it |
| D2 | selected PTA/taint local Tarjan state/frame traversal 0 | generic/indexed/PTA/taint call same core; domain/output policies local |
| D3 | DFG build-input producer/export/exclusive helpers 0 | pack producer + PortDFG entry remain |
| D4 | SCV2 inline contribution/K constant 0 | existing RRF helper + SCV2 leaf dependency |
| D5 | dormant UF files/module/export/consumers 0 | base UF·Steensgaard/Andersen/Mahjong contracts remain |
| D6 | API/agent duplicate field conversion and retired legacy converter/imports 0 | shared conversion and actual current consumers |
| D7 | alternative path resolver/byte reader/source cache 0 | one registry capture, separate projections maintained |

Use symbol-aware existing structural gates and fresh caller census. `rg` zero-match exit 1 is normal search output; exit 2 is a search error and cannot prove absence. Blanket identifier bans would incorrectly remove pack symbols, port types, historical docs, or independent test oracles.

Generated retirement도 확인한다. Final clean source에서 topology inventory/roster를 owning generator로 갱신하고 GOD baseline은 reviewed owned paths만 reconcile한다. Honesty의 `shape-only`, `positive-negative-pair`, `relationship`에는 삭제한 Cost Shape/DFG tests의 stale keys가 남아 있으므로 owning `tools/ci/honesty/ratchet.sh`로 감소·line-drift 갱신한다. `evidence-grounding`의 scoped keys는 live test 이름이므로 regression 결과로 구분하며 일괄 삭제하지 않는다. Ratchet이 신규 위반을 발견하면 실패를 유지하며 threshold·scope·escape를 완화하지 않는다.

## 12. Required checkpoint record

For each C1–C8: source root/HEAD/dirty digest, resolved command and feature closure, fixture provenance, independent expected, actual selected test names/count (including ignored), exit/receipt, retirement/generator/policy result, main commit, NOT_RUN/T4 gaps. Link to the record from the readme ledger.

Stop optional testing when the concrete risk and required gates are covered. Compile-only, fake converter smoke, source presence, document validation, commit/push are separate observations; none substitutes for the required behavior proof.
