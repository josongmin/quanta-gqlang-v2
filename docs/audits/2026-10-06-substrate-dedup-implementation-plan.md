# Substrate 중복 제거 실행 계획

## 1. 목표와 현재 기준

**확인된 D1–D7의 중복 production 계산·변환·manifest reader를 제거하고, 실제 소비자가 선택한 owner를 사용하게 한다.** 모듈을 옮기거나 새 함수로 감싸는 것만으로 완료 처리하지 않는다.

- 날짜: 2026-10-06. 상태: **계획 확정, 구현·Rust/QBC 실행 미완료**.
- 감사 기준: GQLang `cf76e817`, Semantica `6abe4343`, Quanta Index `31828561`.
- 기존 감사의 80 evidence 항목을 현재 bytes와 대조했다. 변경 항목은 감사 Markdown 하나이며, 선택한 중복 owner·caller·gate·Index SDK 소스는 동일하다. F1의 공개 계약 입력도 동일하다.
- Semantica는 당시 dirty 372항목이다. Root Cargo·구조 gate의 foreign 변경을 소유 변경과 분리하고, 매 checkpoint 직전에 같은-file hunks와 실제 입력을 다시 확인한다.
- 코드 변경 저장소: **Semantica**. GQLang은 이 계획과 감사 문서를 보관한다. Index 소스 변경은 현재 D4에 필요하지 않다.
- 원래 판정·caller·계약 비교: [substrate 감사](2026-10-06-substrate-ownership-and-usecase-audit.md). 이번 source tuple과 hash: [계획 기준 입력](2026-10-06-substrate-dedup-plan-source.json).

## 2. 삭제와 owner 선택

| 항목 | 남길 계산·source owner | 없앨 것 | 완료의 핵심 |
|---|---|---|---|
| D3 MemoryIR | `quanta-runtime-memssa-pack`의 build-input ingress | DFG의 미사용 build-input entry·고유 lowering/validation·export·tests | 활성 pack authority 유지, DFG에 같은 build-input producer 없음 |
| D5 UnionFind | base UF와 실제 Steensgaard/pair session | 미사용 Andersen UF module·export·자체 tests | 퇴역 UF 정의/소비자 없음, 실제 PTA solver 정상 |
| D4 RRF | `quanta-retrieval-algo-v1::rrf_contribution_v1` | SCV2의 contribution 수식·중복 상수 | SCV2는 공용 함수 호출, rank/합산/정렬 의미 동일 |
| D7 manifest | 기존 `query_surface_registry`가 captured source 소유 | introspection 별도 resolver/read·분리된 manifest read/cache authority | 목록·schema·mapping·dispatch가 한 version/digest 사용 |
| D1 Shape | 기존 `quanta-adapters-shape` | cost crate의 Shape 구현·고유 export/test | 모든 factory가 standalone 사용, 계산 정의 하나 |
| D6 app projection | 기존 `codegraph_shared.infra.search` 안 공통 current-row 변환 | API/HTTP agent의 중복 field 변환·구형 reader·휴면 converter | F1 계약을 한 변환 owner가 소비, 앱 정책은 각 앱 유지 |
| D2 SCC | 기존 `quanta-graph-utils-v1`의 iterative core | generic/PTA/taint에 중복된 Tarjan state/frame 순회 | 세 선택 owner가 같은 core 사용, domain 정책/순서/중단 의미 보존 |

### D3 — 미사용 MemoryIR build-input 제거

- 삭제 위치: [DFG canonical_ir](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-dfg/src/memssa_adapter/canonical_ir.rs)의 `canonical_memory_ir_from_build_input`, `validate_deletion_input_v1`, `lower_build_input_blocks_v1` 및 이 entry만 사용하는 helper/test.
- 함께 정리: `memssa_adapter/mod.rs` export, 삭제 후 orphan인 error/import, generated inventory의 활성 참조. `DeletionAuthorityViolation`은 fresh caller census 후 orphan일 때만 삭제한다.
- 유지: [활성 runtime pack](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-memssa-pack/src/canonical_memory_ir_ingress.rs), DFG의 PortDFG entry와 공용 producer digest·shape policy·common IR·finalizer·sort helper.
- 확인: real/synthetic load ID, explicit override map, collision, authority 누락, digest 및 typed refusal의 pack regressions; DFG PortDFG와 실제 taint promotion caller의 owning QBC rail.
- 완료: DFG build-input 정의/export/활성 참조 0. 전체 `canonical_ir.rs` 삭제나 DFG→pack forwarding dependency는 만들지 않는다.

### D5 — 미사용 Andersen UF 제거

- 삭제 위치: [Andersen UF](../../../semantica-codegraph-v2/packages/core/pta/engine/src/solver/andersen/union_find.rs), 대응 `union_find_tests.rs`, Andersen `mod.rs` 등록/export, crate root의 해당 `UnionFind` re-export.
- 유지: [base UF](../../../semantica-codegraph-v2/packages/core/pta/engine/src/solver/union_find_base.rs), Steensgaard main/retained pair session, 활성 Andersen solver, Mahjong의 별도 min-root 계약.
- 삭제 전: Semantica·Index·GQLang과 확인 가능한 consumer/feature/target에서 public symbol의 alias·glob import까지 대조한다. 알려진 consumer는 같은 checkpoint에 전환한다. In-repo census가 알려지지 않은 외부 consumer의 부재를 증명하지는 않는다.
- 확인: `codegraph-core-pta-engine`의 실제 Steensgaard/Andersen positive·negative 결과 및 retained pair-session 회귀. Generated target/module selector의 stale 참조도 제거한다.
- 완료: 퇴역 UF 파일·정의·export·소비자 0. Base의 root-returning 계약을 bool-returning 호환 wrapper로 복제하지 않는다.

### D4 — SCV2 RRF 수식 단일화

- 변경 위치: [SCV2 fusion](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-retrieval-kernel/src/scv2_exact_fusion.rs)의 `rrf_score_v1` contribution expression.
- 사용 owner: [기존 RRF helper](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-retrieval-algo-v1/src/rrf.rs). `.iter().map(...).sum()`의 순서와 fusion의 identity·tie-break·top-k·duplicate refusal은 유지한다.
- Cargo: `scv2-exact-owner-surface`에서 helper optional dependency를 직접 활성화한다. Dependency 하나를 쓰기 위해 전체 `identity-collapse-surface`를 켜지 않는다.
- 확인: rank 0/1/2/`u32::MAX`, 다중 contribution의 독립 expected score, 같은 score의 ordering, duplicate conflict. Leaf-only feature도 owning QBC로 검증한다.
- 완료: SCV2의 자체 `1/(k+rank)` 수식·필요 없는 상수 0. Complete fusion pipeline은 서로 다른 책임으로 유지한다. S17/S19 membership/history gate를 RRF의 직접 증명으로 세지 않는다.

### D7 — manifest source를 한 번만 캡처

- 변경 위치: [registry](../../../semantica-codegraph-v2/apps/mcp/mcp/domain/query_surface_registry.py), `handlers/introspection.py`, `handlers/surface_config.py`, `main.py`, import 시 registry를 평가하는 generated SDK adapter.
- 기존 registry가 path resolve → bounded read → decoded validation → private immutable path/digest/전체 entries 캡처를 한 번 수행한다. 한 byte source에서 모든 projections를 만든다.
- metadata·executable·deferred projections는 서로 목적이 다르므로 유지한다. Public dict/list는 private source를 변경할 수 없는 독립 projection으로 반환한다.
- `_initialize_sdk`의 cold/direct/retry 진입 모두 adapter import와 SDK 생성 전에 같은 idempotent preflight를 통과한다. Main-only 검사나 path helper만 공유하는 변경은 미완료다.
- 별도 fallback resolver, `CODEGRAPH_REPO_PATH`/cwd 재탐색, 별도 manifest read·source cache를 제거한다. Explicit override가 잘못되면 typed refusal; packaged/default artifact 선택은 명시한다. Refresh는 같은 captured source를 재투영하며 새 파일 version은 process restart로 적용한다.
- 확인: read/version replacement control, cold import 순서, SDK unavailable retry, override missing/invalid, packaged artifact, same-usecase/alias-normalized duplicate refusal, public copy mutation isolation. 목록 비교는 set뿐 아니라 순서·개수·유일성을 확인한다.
- 완료: 이 경로의 crosswalk byte reader/resolver authority 1개, schema/list/mapping/dispatch의 source identity 일치. Hot reload subsystem은 추가하지 않는다.

### D1 — Shape 계산과 제품 feature를 두 checkpoint로 전환

**D1-A: production 계산 owner cutover**

- 선택: [standalone Shape](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-adapters-shape/src/shape_analyzer_adapter.rs). 기존 `ShapeAnalyzerPort`와 V1/V2 authority·PTA 변환 계약을 유지한다.
- 직접 전환: runtime-surface factory, adapters facade, analysis-facade, sdk-session-backend-owner의 concrete import/export/factory 및 PTA converters.
- Cost의 unique regression을 standalone/적절한 owning test로 옮긴 뒤 cost의 `shape_analyzer_adapter.rs`·전용 tests·mod/export를 물리 삭제한다. Cost/termination/heap 기능은 유지한다.
- 확인: Tree/DAG/cycle·alias cases, V2 authority 누락/불일치 refusal, 실제 SDK factory→port 결과. 두 Shape shell gate의 selector도 standalone owner로 옮기고 실제 선택 test 수를 확인한다.
- D1-A 완료: 활성 concrete factory가 모두 standalone을 만들고 cost에 Shape 계산 정의가 없다. Cost→standalone 호환 wrapper는 남기지 않는다.

**D1-B: shape-only feature closure**

- `shape-analyzer`에서 불필요한 `cost-analyzer` 활성화를 제거한다. Manifest뿐 아니라 selector·builder·capability·public export cfg·overlay·generated honesty 입력을 함께 정리한다.
- 확인 lanes: pure Shape, PTA-derived Shape, Shape disabled typed unavailable, 기존 Cost/Termination/Heap 조합. 두 Shape gate 및 owning feature-policy proof를 갱신한다.
- D1-B 완료: 실제 dependency closure로 shape-only 주장을 증명한다. D1-A 성공을 regex-free/제품 feature 분리 성공으로 세지 않는다.

### D6 — F1 계약 확정 후 API/agent 변환 통합

- **직접 선행은 F1**: Native match/supplemental variants, 실제 identity, optional location, primary/supplemental counts·truncation 의미를 producer/current SDK/소비자가 합의해야 한다. 현재 깨진 reader를 공용 helper로 옮기면 중복 결함만 중앙화된다.
- 기존 shared package에 current SDK row→app hit 공통 변환을 둔다. API strict parser와 HTTP agent `QueryAdapter`의 공통 field mapping을 함께 전환하고 이전 duplicate body를 삭제한다.
- Symbol identity/location 부재는 명시적으로 표현한다. 가짜 ID·line/column을 생성하지 않는다. Authored metadata span이 있으면 유지한다. 앱 DTO에 필요한 변경은 F1의 현재 계약 cutover와 함께 처리한다.
- App별 request admission·HTTP error·anchor·GraphRAG file presentation은 각 앱에서 유지한다. 공개 MCP는 `GeneratedDaemonSdkQueryAdapter`의 별도 envelope 경로로 검증하며 HTTP agent parser와 혼동하지 않는다.
- 휴면 `SymbolRetriever`의 non-test construction을 다시 조사한다. Dormant converter/module을 제거할 때 package export·service exception import·lazy legacy handler를 함께 정리한다. 필요한 live exception의 owner만 옮기고 obsolete facade를 유지하기 위한 shim은 추가하지 않는다.
- 확인: Native-shaped symbol/nonsymbol/mixed/empty/truncated → 실제 SDK projector → 공통 app 변환 → API/HTTP agent; optional span·typed invalid rejection·GraphRAG file IDs; live MCP envelope 별도 control. Converter만 monkeypatch한 smoke로 입증하지 않는다.
- 완료: 현재 SDK row field 변환 owner 1개, 퇴역 converter/import/export 0, 모든 active consumers가 같은 contract를 사용한다.

### D2 — SCC 세 구현을 기존 iterative core로 통합

**D2-P: layer/feature landing 조건**

- 현재 graph-utils는 `tier0-shared`·empty default이며 root mapping도 같다. Root policy를 기준으로 proposed tier1 분류·empty-default·query-HIR dependency default-feature 설정을 함께 수리한다.
- 정책 통과는 최종 landing의 조건이다. SCC 알고리즘의 의미적 선행과 구분하며 unrelated baseline 위반을 이번 변경의 GREEN으로 덮지 않는다.

**D2-K: 기존 core 확장**

- [graph-utils SCC](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-graph-utils-v1/src/scc.rs)에 borrowed indexed iterative traversal을 둔다. 기존 generic entry도 같은 core를 사용한다.
- Caller admission을 graph/index/state allocation 전에 수행한다. Overflow·입력/작업/메모리 bound 및 checkpoint를 연결하며 Compute error와 caller Stop error를 구분한다. Callback 존재만으로 allocation bound를 주장하지 않는다.
- Adapter가 domain을 확정하고 indices를 투영한다. Generic은 closed domain의 unregistered target을 거절; PTA의 declared-key projection과 taint의 successor-only node 처리는 명시적으로 보존한다. Component/member ordering·IDs·caller failure/cancel 계약도 local projection으로 보존한다.
- 새 SCC crate·universal graph trait·third-party engine을 만들지 않는다. Core Andersen/CallGraph/Abstract Worklist/dominance는 이 세 owner cutover의 대상이 아니며 별도 후보로 기록한다.

**D2-PTA / D2-Taint: 각각 소비자 cutover**

- [PTA precise SCC](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-analysis/src/queries/pta/precise_query_scc.rs)의 복제된 Tarjan state/frame 순회를 삭제하고 기존 core 호출과 PTA 정책·출력 투영만 남긴다.
- [Taint recursion SCC](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-taint-solver/src/analysis/taint/interproc/wire_recursion.rs)의 복제 순회/state/sentinel helper를 삭제한다. Opt-in cycle policy·budget·sorted output은 caller owner가 유지한다.
- `gate-tarjan-iterative.sh`를 실제 새 core와 taint adapter tests로 연결한다. Ignored `deep_chain_50k`를 명시적으로 실행하며 empty selection은 통과가 아니다.
- 확인: independent expected partition **및 exact order/ID**, empty/isolated/self-loop/cycle/disconnected/target-only/invalid index, cancellation/typed compute error, admission 전 실패, 50k deep chain. Partition만 같은 결과는 충분하지 않다.
- 완료: generic/PTA/taint의 선택 범위에서 Tarjan 계산 core 1개. PTA만 전환하고 taint 복사본이 남으면 D2는 미완료다. 동일 입력의 작업·allocation/시간을 측정하기 전 성능 개선 수치는 제시하지 않는다.

## 3. 실행 순서와 main checkpoint

단독 개발의 기본 순서는 아래와 같다. 각 checkpoint는 삭제·소비자·해당 gate를 함께 포함하는 독립 commit이다. 비용은 상대적인 코드/검증 범위이며 소요시간 추정이 아니다.

| 순서 | Checkpoint | 크기·핵심 위험 | 직접 선행 |
|---|---|---|---|
| 1 | D3 dormant build-input 삭제 | 작음; PortDFG shared helper 오삭제 | 없음 |
| 2 | D5 dormant UF 삭제 | 작음; public export·남은 consumer | 없음 |
| 3 | D4 pure helper 재사용 | 작음; leaf feature·FP 합산/정렬 | 없음 |
| 4 | D7 captured manifest | 중간; import/retry·public projections | 없음 |
| 5 | D1-A Shape owner cutover | 중간; factories·authority·test 이동 | 없음 |
| 6 | D1-B Shape product feature | 중간 이상; Cargo/cfg/public closure | D1-A |
| 7 | F1 계약 → D6 app 변환 | 중간 이상; 실제 variants·public DTO | D6는 F1 |
| 8 | D2-P/K → PTA → taint | 가장 큼; order/ID·중단·allocation | 각 adapter는 공용 core |

PTA→taint는 권장 검증 순서이며 hard dependency가 아니다. Search 사용자 경로를 우선 복구할 때는 F1→D6를 먼저 수행할 수 있다. 나머지 functional 수리 F2/F4/F5/F6/F7/F9/F10을 중복 작업의 선행으로 추가하지 않는다.

## 4. 매 checkpoint의 통과 조건

1. **착수:** exact HEAD·관련 dirty·caller/feature/known consumer를 확인하고 owned write set을 고정한다. Concurrent 입력이 바뀌면 관련 판정·proof만 다시 확인한다.
2. **결과 증명:** 변경 contract를 independent expected 결과와 실제 owner/caller positive·negative rail로 확인한다. 동작을 보존하는 삭제에는 기존 meaningful rail을 재사용한다. 단순 삭제마다 mirror test를 새로 만들지 않는다.
3. **퇴역 증명:** 퇴역 production 정의·mod·export·factory·reader 및 활성 generated/test selector가 없음을 확인한다. 역사 문서의 이름이나 독립 test oracle까지 blanket text ban으로 제거하지 않는다.
4. **재발 방지:** 기존 구조/feature/manifest gate에 이 7개의 owner invariant를 추가·수정한다. Source 존재/consumer wiring 검사는 runtime proof와 구분한다. 일반 clone detector 하나로 semantic 중복 해결을 주장하지 않는다.
5. **로컬 검증:** Rust는 affected owning QBC rail, Python은 affected owner/consumer tests, 기존 mandatory policy gates를 사용한다. Selector가 실제 cases를 실행했는지 기록한다. 유료 hosted CI는 이 계획의 조건이 아니다.
6. **Main 반영:** 공유 index/commit은 직렬화하고 owned hunks만 commit한다. Consumer cutover와 퇴역 코드 삭제를 같은 merge checkpoint에 포함한다. Dependency가 닫힌 각 checkpoint를 main에 반영한다.
7. **기록:** 각 D 항목에 삭제된 symbols/paths, surviving owner, 연결된 consumers, 실행 command/selector/features, source tuple/digest, 결과·NOT_RUN을 남긴다. Commit·compile·focused rail을 installed product qualification으로 확대하지 않는다.

## 5. 최종 종료 기준

- D1: Shape production 계산 정의 1개, 모든 factory 전환, D1-B 별도 feature proof.
- D2: 선택한 generic/PTA/taint traversal core 1개, 두 adapter cutover 및 order/stop/deep-chain proof.
- D3/D5: 퇴역 entry/module·export·활성 참조 0, 보존한 실제 owner regression 통과.
- D4: SCV2 inline RRF contribution 0, 기존 helper 호출과 standalone leaf proof.
- D6: current row 공통 변환 1개, old converter/import/export 0, 실제 variants/consumers proof.
- D7: captured crosswalk source authority 1개, 모든 projections의 version/digest 일치 및 init/retry/duplicate controls.
- 이 7개가 모두 충족되어야 **D1–D7 중복 제거 완료**다. 현재는 모두 구현 전이며 이 계획 문서 자체를 완료 증거로 세지 않는다.
