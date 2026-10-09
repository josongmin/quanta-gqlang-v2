# Production 중복 구현 최종 대조와 수정

Updated: 2026-10-10. 입력: QGLang main `5e5164dbcb1493a09dceecab4e1f3a23b1c09b3c`
+ 이전 문서 WIP. Rust 수정 전 source는 HEAD와 같았다. 이 파일은 초기
[구조 감사](audit.md)를 production implementation 대조로 보완한다.

## 조사 방법과 한계

- repository가 고정한 AST parser와 test-module guard를 재사용했다.
- `crates/*/src/**/*.rs`에서 guarded test modules/test_support를 제외한160개 파일,
  production function body2870개를 읽었다. parse error0.
- 주석을 제외한 leaf-token 본문80개 이상 exact 비교: duplicate2묶음.
- identifier를 정규화한 본문160개 이상 후보 비교: Record/Union 생성자1묶음.
- production 함수 안의120-token fragment를12-token 간격으로 표본 비교:
  같은 catalog namespace pair1묶음. 추가 표본 후보 없음.
- macro expansion,80-token 미만 본문, stride 사이 fragment, 다른 구조로 작성된 의미상 중복,
  test fixture 중복은 이 census의 전수 증명 대상이 아니다. 전체 중복0을 주장하지 않는다.
- `duplicate-census.json`은 source file SHA와 exact token hash, `fragment-candidates.json`은
  표본 pair를 `.audit/oct10-frontend-closeout/`에 보관한다. 이 디렉터리는 기존 ignore 대상이다.

## D01 — catalog namespace vocabulary: SOURCE_REPAIRED

기존 [catalog.rs](../../../crates/quanta-gqlang-v2-modules/src/catalog.rs)의
`ExternalCatalogPayloadV2::namespace_kind`와
`ExternalCatalogPayloadInputV2::namespace_kind`는 동일한15-arm 매핑을 각각 소유했다.
수정 전 AST 본문156 tokens/hash
`f8b81bf12ff10ea0fcec646145edcb023f59d30bd5e42ed4ffcb4e129587575b`.

### 실제 caller와 남긴 owner

- unchecked input: `PrecheckedExternalCatalogAssetV2::try_admit`의 kind-token/payload 대조.
- checked output: catalog/binding 소비자의 `payload().namespace_kind()`.
- Modules catalog의 private `catalog_payload_namespace_kind!`에 매핑 표를 한 번 선언하고
  두 기존 메서드가 사용한다. macro는 두 **서로 다른 payload type**의 exhaustive match만 생성한다.
  새 public API, runtime dispatch, allocation, dynamic validator는 추가하지 않는다.
- input의 `SignatureInputV2`와 checked `SignatureV2`, signature/callable-kind validation,
  prevalidation 순서와 budget/error 처리는 각각 기존 owner에 유지한다.

### 수락 조건

public checked method의 `const`, return type와 visibility가 같고15개 variant mapping을 유지한다.
input→checked는 기존 검증을 계속 통과해야 하며 잘못된 payload/foreign registry/budget 거절을 유지한다.
기존 `external_catalog_accepts_exact_closed_ten_kind_payload_matrix` 및 rejection/budget owner가 실행됐다.

## D02 — canonical lock text: SOURCE_REPAIRED

기존 [lockfile.rs](../../../crates/quanta-gqlang-v2-modules/src/lockfile.rs)의
`LockfileV2::to_text`와 `VerifiedLockSetV2::to_text`가 같은 BTreeMap pin 직렬화를 반복했다.
수정 전 AST 본문87 tokens/hash
`7278e586b73e736f20a7b0df4d48e0f4fa3b120579619bfea936acc90c460cea`.

### 실제 caller와 남긴 owner

- parse canonicality: `LockfileV2::parse`의 `lock.to_text() != text` 판정.
- verified output: `ValidatedPackAuthorityV2`가 보유한 verified lock의 serialization.
- 기존 lockfile owner의 private `canonical_lock_text(&BTreeMap<PackIdV2, LockPinV2>)`가
  하나의 byte writer를 소유하고 두 public method는 위임한다.
- sorted iteration, single spaces, version/digest encoding, final newline과 empty output을 유지한다.
  pin payload clone, reserve policy, verification/source identity 변경은 없다.

### 수락 조건

독립 literal 두 pin의 `canonical_lock_roundtrip_is_byte_exact`,
`noncanonical_lock_order_points_at_first_out_of_order_line`, duplicate/missing/stale/version drift,
separator/field/final-newline rejection과 기존 budget tests가 실행됐다.
parser canonicality와 verified output이 같은 writer를 사용한다.

## 유지한 유사 코드 — 중복 결함으로 발급하지 않음

[ty.rs](../../../crates/quanta-gqlang-v2-types/src/ty.rs)의 `RecordTypeV1::try_new`와
`UnionTypeV1::try_new`는 identifier-normalized AST172 tokens가 같다.
Record는 field name/`DuplicateRecordField`/`RecordFieldV1`, Union은 tag/
`DuplicateUnionTag`/`UnionVariantV1`를 소유한다. 첫 중복 위치·diagnostic와
payload type이 다르며 hot path/authority 오류가 재현되지 않았다.
이 둘을 erased validator나 새 generic engine으로 묶는 작업은 발급하지 않는다.

## D03 — Modules/Logic SCC 계산 책임: SOURCE_REPAIRED / FOCUSED_PASS

AST 동일 본문 비교와 별도로 algorithm/caller를 추적해 Modules Kosaraju와 Logic Tarjan의
SCC partition 책임 겹침을 확인했다. 재현된 결과 오류는 아니며 두 production traversal을 제거했다.

| 실제 경로 | 공통화 후 owner와 보존 계약 |
| --- | --- |
| [Modules graph](../../../crates/quanta-gqlang-v2-modules/src/graph.rs) `minimal_cycle_witness` | 공통 partition labels → 기존 lexicographically minimum actual simple cycle. module import/reexport, manifest pack, external catalog, document View topology caller 및 legacy charge-only walk 유지 |
| [Logic recursion](../../../crates/quanta-gqlang-v2-logic/src/recursion.rs) `stratify`→`rule_components` | 공통 partition → members/components canonical sort → 기존 negation/aggregate rejection, strata. discovery 및 SCC emission의 incoming literal/root location별 work charge 유지 |
| [공통 kernel](../../../crates/quanta-gqlang-v2-modules/src/graph/indexed_scc.rs) `partition_v1` | 반복형 Tarjan, borrowed adjacency/context, fallible before-discovery/emission callback. missing/out-of-range edge는 typed `InvalidEdge`, caller refusal는 첫 `Rejected(E)`로 즉시 종료 |

### 의존·호환·실패 경계

- Logic→Modules normal edge 하나를 Cargo, 두 lockfile 및 exact architecture policy에 함께 추가했다.
  이미 Logic→Sema→Modules 방향이므로 역방향·cycle·새 외부 dependency는 없다.
- kernel은 Modules graph child이며 Modules에서 doc-hidden workspace API로만 노출한다.
  curated frontend-contract facade root, DTO/schema/digest 계약은 변경하지 않았다.
- Logic frame은 incoming checked location을 빌린다. emission을 defining-rule location으로
  바꾸지 않는다. 두 정점 chain의 수작업 charge 순서6개에서 limit1~5의 정확한 refusal span을 검사했다.
- Modules는 모든 target을 사전 검사한 뒤 Infallible callback으로 kernel을 호출한다.
  malformed graph의 기존 `None` 계약과 canonical witness, charge/error precedence를 유지한다.
- partition label 번호는 내부 값이다. Logic은 정점 오름차순으로 component를 재구성하고
  component 목록을 정렬해 기존 출력 order를 유지한다.

### scratch accounting

- kernel은 index/lowlink/pending/output의 usize buffer4개, bool buffer1개,
  `(node, next_edge, borrowed context)` frame buffer1개를 정점 수로 선할당한다.
  `scratch_bytes_v1`은 모든 live Vec capacity를 checked arithmetic으로 계산한다.
  64-bit target에서 정점당57 bytes이며 allocator overhead/실측 RSS 주장이 아니다.
- kernel scratch는 반환 labels 외에는 함수 반환 시 drop된다. Modules는 이후 reverse adjacency와
  witness scratch를 할당하므로 `minimal_cycle_scratch_bytes_v3`는 두 phase bound의 max다.
  기존 reverse adjacency는 witness에 필요해 유지했다. global cap/allowance를 올리지 않았다.
- manifest `cycle_scratch_bytes_v3`와 request `request_topology_bytes_v3`는 이 기존 helper를 사용하므로
  새 bound를 함께 소비한다. topology 테스트는 정상/DAG와 실제 self-cycle witness 양쪽에서
  sink reservation peak를 관측 후 exact/one-byte-short로 재실행하고 failure/drop custody를 검사한다.
  이것은 admission 경계 증명이며 allocator 측정이나 demand 공식의 독립 실측 검증은 아니다.

### 실행한 owner 증거

- Modules/Logic lib 및 `v3_logic_recursion`, `v3_rule_scc_hellgate`: **71/71 PASS**.
  independent Logic343 graphs, Modules exhaustive cycle oracle, permutation/equal-label witness,
  first-cause callback, malformed adjacency, 50k deep-chain512-KiB stack 및 exact199998-step 포함.
- Modules authority120 + document predeclaration25 + topology storage8: **153/153 PASS**.
  topology 최종 assertion 정리 후8/8 재실행 PASS.
- 실제 Logic caller의 discovery/emission located charge regression: **1/1 PASS** (16 filtered).
- 새 kernel oracle는3정점 전체512 graphs를 양쪽 edge order로 대조한다.
- 추가 caller census에서 `lowlink` traversal은 공통 kernel에만 존재한다.
  old `TarjanStateV1` 및 Modules Kosaraju production traversal은 없다.
- 후속 AST census: **161 production files /2866 functions**, parse error0,
 80-token 이상 동일 body group0. macro expansion/작은 body/다르게 작성한 의미 중복은 이 수치의 대상 밖이다.

증거는 `.audit/oct10-frontend-closeout/scc-{focused,storage,storage-final,charge-final}.log`,
`scc-duplicate-postflight.json`, `scc-source-state.json`에 보관한다.
새 source의 repository qualification은 아래 current-HEAD receipt로 별도 판정한다.

## D04 — DAG preflight 중복과 ready frontier 반복 이동: SOURCE_REPAIRED

후속 감사 입력은 clean main `c5b27aa59e2f76f6844d9f74ce434803050e05af`다.
이 SHA의 qualification은 exit0/Nextest1520이며 아래 successor source의 증명으로 재사용하지 않는다.

### 확인한 원인과 실제 caller

- Modules `ModuleGraphBuilderV2::detect_cycle`는 `topological_order`와 visits DP를,
  manifest `cycle_with_legacy_steps`는 별도 indexed Kahn/ways DP를 소유했다.
  서로 다르게 작성된 같은 legacy DAG edge-visit 계산이며 AST exact-body census로는 검출되지 않았다.
- manifest ready Vec의 `remove(0)`와 sorted `insert`는 넓은 frontier에서 반복해서 원소를 이동한다.
  isolated N roots만 있어도 제거에 O(N²) 이동이 발생하고 charged edge work는0일 수 있다.
  이는 source로 확인한 구조적 work bound 문제다. 기존 production latency/회귀 수치를 측정한 주장은 아니다.
- View `dependency_first_view_order`에도 같은 이동 패턴이 있지만 최소 ready ID 선택은 sealed output이다.
  이 경로를 private preflight FIFO와 단순 통합하면 canonical order가 바뀌므로 별도 정책을 유지한다.
- 기존 `dag_fast_path_matches_legacy_steps_for_all_five_node_forward_graphs`는 cfg(test) map 기반
  compatibility helper만 호출했다. 이름과 달리 production indexed path의 직접 증명이 부족했다.

### 변경과 보존 조건

- [indexed_walk.rs](../../../crates/quanta-gqlang-v2-modules/src/graph/indexed_walk.rs)의
  `acyclic_steps_v1`이 두 production caller의 DAG 계산을 소유한다. borrowed adjacency,
  root multiset, dangling terminal edge count, overflow flag와 cyclic fallback 의미를 보존한다.
  잘못된 root/target은 typed error이며 acyclic success로 바꾸지 않는다.
- ready Vec는 append와 cursor만 사용한다. 각 정점은 한 번 enqueue되며 이 **preflight 순회**는
  O(V+E+roots), scratch는 usize buffer3개다. 전체 caller의 ID indexing/sorting까지 선형이라고 주장하지 않는다.
- manifest의 중복 `adjacency_ids`를 제거하고 labeled adjacency를 빌린다.
  `cycle_scratch_bytes_v3`에서 제거된 Vec headers/target capacity를 함께 빼며 global cap은 올리지 않는다.
- [View topology](../../../crates/quanta-gqlang-v2-modules/src/catalog/view_topology.rs)는 표준
  `BinaryHeap<Reverse<usize>>`로 최소 ready ID를 계속 선택한다. heap push/pop은 O(log V),
  capacity는 Views 수로 선할당하며 request bound도 `Reverse<usize>` 원소 크기를 사용한다.
  기존 순서/진단/step charge와 cycle witness 의미를 변경하지 않는다. 새 외부 dependency는 없다.

### 실제 검증과 한계

- DAG 공통 owner:3정점512 graphs × dangling patterns8 × root multisets4 = **16384 cases**를
  독립 reachability/path-enumeration oracle와 대조했다. malformed root/target, reachable overflow,
  empty-root unreachable branching과 dangling edge 과금도 검사했다.
- 기존5정점 forward DAG1024 graphs × root sets2에서 **실제 `cycle_with_legacy_steps`**를 호출해
  독립 legacy path count의 exact/one-short refusal를 검사하도록 수정했다.
- actual production wrapper에서 isolated roots50000, wide-star50000 vertices의 exact/one-short를 검사했다.
  wall-clock threshold나 속도 향상 배수로 판정하지 않는다.
- Modules lib25 + authority120 + request topology8 = **153/153 PASS**.
  heap 전환 후 document predeclaration25 + topology storage8 = **33/33 PASS**;
  기존 unrelated-ready diamond의 literal canonical order, asset/edge permutation과 duplicate-edge controls 포함.
- 최종 typed diagnostic projection 정리 후 Modules lib25/25 재검증 PASS.
- production 재대조 **162 files /2866 functions**, parse error0,80-token 이상 exact-body group0.
  old production `topological_order`, `acyclic_cycle_walk_steps_indexed`, `adjacency_ids`와
  Modules ready frontier의 `remove(0)`/Vec insert는 제거됐다. map compatibility oracle는 test-only다.
- 증거: `.audit/oct10-frontend-closeout/{dag-focused,dag-final-unit,view-heap-focused}.log`,
  `dag-duplicate-postflight.json`. 최종 full qualification은 현재 HEAD receipt로 별도 판정한다.

## D05 — Logic SCC membership 재구성과 offending payload 복제: SOURCE_REPAIRED

입력은 clean main `77550baafe77dc8798ee666aeb8d9271736f3a50` (qualification exit0,
Nextest1524)다. 이는 아래 변경 후 source의 qualification이 아니며 현재 HEAD receipt가 필요하다.

### 원인·실제 caller·변경

- [Logic recursion](../../../crates/quanta-gqlang-v2-logic/src/recursion.rs)의 `rule_components`는
  공통 kernel의 dense labels를 component 목록 생성 후 폐기했다. `scan_offending_edges`는
  component별 BTreeSet, `component_members`는 return-path용 BTreeSet, `assign_strata`는
  별도의 `Vec<Option<usize>>` membership을 다시 만들었다.
- private `RuleComponentsV1`이 canonical sorted members와 dense `component_of`를 한 번 소유한다.
  kernel labels buffer를 canonical component sorting 후 제자리 relabel하며 scan/BFS/strata가 재사용한다.
  각 domain의 graph ordering/stratification 정책을 다른 stage로 옮기지 않는다.
- `offending_order`는 기존 `edge_order`의 같은 source/target/polarity/literal/rule key를 다시 투영했다.
  이를 제거하고 graph construction과 offending selection이 하나의 ordering owner를 쓴다.
- scan은 positive intra-SCC edge까지 임시 payload를 복제했고, negated/aggregate 후보 전부를 Vec에
  보관한 뒤 두 최소값만 사용했다. 이제 graph payload를 빌린 minimum Option2개만 유지한다.
  selection 추가 보관은 O(1), shared membership index는 O(V)다. graph 자체나 전체 RSS가 O(1)이라는 주장은 아니다.
- canonical component/member/adjacency scan order와 edge별 before-work charge는 유지한다.
  최소값 발견 뒤에도 scan을 끝내며 negation-before-aggregate와 마지막 scan의 budget refusal가 유지된다.
  equal canonical key는 기존처럼 첫 payload를 유지한다. 최종 diagnostic만 필요한 checked facts를 clone한다.

### 실패 경계

- 누락/범위 밖 membership을 빈 집합이나 stratum0으로 해석하던 fallback을 제거했다.
  공통 lookup은 exact checked location의 typed `MembershipMismatch`로 거절한다.
- return-path의 self-cycle도 초기 source/target membership 검증을 통과해야 한다.
- 이 index는 private constructor가 완전하게 만든다. 이번 감사에서 외부 입력으로 잘못된 성공을
  재현한 것은 아니다. malformed-index 검증은 **내부 fault injection**이며 invariant 회귀에 대비한 proof다.
- public DTO/schema/digest, source identity, diagnostic selection/span, step totals와 global limits는 변경하지 않는다.

### 실행 증거

- `cargo test --locked --offline -p quanta-gqlang-v2-logic --lib --test v3_logic_recursion
  --test v3_rule_scc_hellgate --test v3_logic_source`: **84/84 PASS**
  (lib24/recursion17/SCC13/source30, failed0/ignored0/filtered0).
- 기존 independent343 graph/strata oracle,50k narrow-stack/exact-step, canonical actual cycle,
  permutation, source-sealed polarity/identity 및 discovery/emission located charges를 포함한다.
- 새 actual stratify regression:8194 body edges의 exact8196 steps에서는 최소 negated cycle,
  one-short에서는 마지막 aggregate literal의 step refusal. 조기 rejection이나 과금 생략을 거절한다.
- equal-key aggregate candidate는 두 입력 순서 각각의 첫 function과 정확한 span을 보존한다.
- private fault injection: missing/out-of-range component index 각각에서 scan, self-cycle BFS,
  strata가 exact literal/rule location의 `MembershipMismatch`로 거절한다.
- 후속 production AST: **162 files /2865 functions**, parse error0,80-token 이상 exact-body group0.
  old `offending_order`, `component_members`, owned candidate Vec와 별도 Option membership map은 없다.
- `.audit/oct10-frontend-closeout/logic-index-final.log`, `logic-index-duplicate-postflight.json`에 보관한다.
  peak allocator/RSS/latency 비교는 실행하지 않았다. full qualification은 current-HEAD receipt로 판정한다.

### 추가 대조 후 유지한 경계

- Types `solver.rs`→`Engine::is_recursive`는 unresolved rule-head domain의 진단 분기를 위한
  pre-sema predicate reachability이며 자체 type-work budget을 사용한다. Logic의 sealed document
  SCC/strata proof와 consumer/stage가 다르다. Types→Modules/Logic 의존을 만들어 공통 SCC로
  강제 연결하는 변경은 하지 않는다. 이번 source audit를 그 경로의 새 runtime/performance proof로 해석하지 않는다.
- Pipeline `execution/vocabulary.rs::registered_execution_symbol`→`project_call_target`는 absent owner
  spelling을 `UnregisteredSymbol`로 거절하고 compile assembly가 이를 진단으로 투영한다.
  `projection.rs::close_expression_ids`는 Foreign/Dangling을 typed rejection으로 처리한다.
  frontend-contract는 curated re-export seam이며 DTO declaration은 Pipeline에 있다.
  확인한 경로에서 새 placeholder success나 두 번째 execution owner는 발견하지 않았다.

## D06 — lock parsing의 중복 canonical 작업과 입력 비례 임시 할당: SOURCE_REPAIRED / FOCUSED_PASS

입력은 clean main `cd9aec00c43dd89998b934054319f75c3ca05247`이다.
해당 입력의 qualification은 exit0/Nextest1527이며 아래 수정 후 source의 증거는 아니다.

### RCA·caller·수정

- `ModuleAuthorityBuilderV2::try_set_lockfile`은 path/raw bytes를 과금한 뒤 pending text를 보관한다.
  `finish`는 graph/catalog/manifest를 검증하고 nonempty pack graph에만 `LockfileV2::parse`를 호출한다.
  empty graph에 명시한 empty lock도 기존처럼 `modules.unexpected_lockfile`이다.
- parser의 `line.split(' ').collect::<Vec<_>>()`는 정확히3필드인지 확인하기 전에 모든 slice를
  보관했다. 예를 들어131072 spaces는 raw bytes 한도 안에서도 공백 수에 비례하는 slice Vec를
  만든다. 이는 **정적 allocation amplification 확인**이며 OOM/잘못된 외부 성공을 재현한 것은 아니다.
- 이제 split iterator에서 최대4필드만 확인한다. field-shape 검사용 동적 Vec는 없으며
  blank/CR/tab, 정확히3개의 nonempty single-space fields, missing EOF LF의 거절을 유지한다.
- 정렬 검사만을 위해 각 `PackId` 문자열을 다시 clone하던 order list는 입력 `&str`을 빌린다.
  `LockfileV2::to_text()`로 전체 canonical String을 만든 뒤 원문과 비교하던 경로는 제거했다.
- `PackIdV2::try_new`, `VersionV2::try_parse`, `ContentDigestV2::try_parse`가 각 canonical spelling을
  이미 검증한다. unique pin count와 parsed count가 같으므로 마지막 검사는 borrowed names와
  BTreeMap keys의 순서 대조만 필요하다. public writer는 기존 `canonical_lock_text` 하나를 유지한다.

### acceptance·compatibility·실패 경계

- 소유 stage는 Modules lock authority, consumer는 `finish`의 verified pack authority다.
  DTO/schema/digest/API를 바꾸지 않으며 새 외부 library나 stage dependency는 없다.
- 전체 줄 검증을 끝낸 뒤 정렬을 확인한다. 뒤쪽 malformed field/duplicate/budget 거절이
  앞쪽 정렬 오류보다 우선하고 primary는 sorted sequence와 처음 다른 입력 줄이다.
- raw byte charge, field-width check, pin node charge 및 duplicate 위치/secondary는 그대로다.
  공통 field owner를 우회하는 permissive parse나 global allowance 증가는 없다.
- 제거한 부가 저장만 field-shape O(1), pack-name extra clones0, canonical-check full String0이다.
  pending raw text, pin map 및 위치를 보관하는 order list는 남는다. 전체 parser의 O(1) 메모리나
  request-owned allocator custody, 측정 latency/RSS 개선을 주장하지 않는다.

### 실행 증거와 추가 대조

- `cargo test --locked --offline -p quanta-gqlang-v2-modules --test v3_modules_v2_authority`:
  **122/122 PASS**, failed0/ignored0/filtered0.
- literal `[A,a,a-b,a.b]`의24 permutations: canonical bytes positive와 모든 noncanonical 순열의
  first-mismatch exact span/secondary 없음. canonical expected를 production serializer로 만들지 않는다.
- 기존 separator/field-shape rail에 missing field와131072-space line 및 exact whole-line span을
  추가했다. 비정렬 prefix 뒤 invalid version/overflow/uppercase digest/extra field/duplicate pin의
  exact primary/duplicate secondary oracle를 추가했다. 기존 source-length/EOF/field-width/resource
  및 module/catalog/pack graph exact-budget controls도 같은 owner target에서 실행했다.
- 초기 focused 실행의 새 test accessor 오기와 fixture digest mismatch는 수정했다.
  각각 `lock-parser-initial-compile.log`, `lock-parser-initial-fixture.log`를 보존하며
  최종 `lock-parser-focused.log`가122/122 성공 증거다. full qualification은 current-HEAD receipt로 판정한다.
- 최초 full qualification `36e3444bb99ebda9bcf16583d1b5d78fd5589c4c`는 새 permutation test의
  배열 인덱싱2곳에 대한 `clippy::indexing_slicing`에서 exit101로 중단했다. 해당 SHA의
  `target/qualification/<SHA>/receipt.json`과 `ci.log`를 보존한다. `.get().expect()`로 수정하며
  lint allowance/정책은 바꾸지 않는다. 수정 HEAD의 same-source receipt가 필요하다.
- 추가 source 대조: Version parse의 `.ok()`는 overflow를 typed invalid-version으로 거절한다.
  lock source-length mismatch의 `.ok()`는 error 진단 위치 부착에만 쓰이며 성공으로 바꾸지 않는다.
  Types candidate generation은 finite-domain shape cost를 mutation 전에 charge하며 propagation의
  tuple arity는 `Engine::add`가 검증한다. saturating cost는 u32 charge conversion/checked-add에서
  거절된다. 이 선택 경로에서 새로운 잘못된 성공은 확인하지 않았으며 Types 전체 증명은 아니다.
- Pipeline scalar bounded projector의 unsupported shapes는 typed refusal이며 source-only relation
  subset을 유지한다. generic relation expression closure는 Foreign/Dangling을 거절한다.
  이번에는 public execution contract를 변경하지 않았으며 새 consumer/runtime proof는 NOT_RUN이다.

## D07 — Execution DTO의 계약 owner 물리 분리: SOURCE_REPAIRED / FOCUSED_PASS

입력은 clean main `a6a85d125d87d421632034fab8c38fe70b59fcd3`이다.
사용자가 보류했던 DTO 이동을 이번 범위에 명시적으로 추가했다.

### RCA·선택한 구조·acceptance

- 기존 dependency는 facade→Pipeline이었다. Pipeline의 DTO를 facade crate로 직접 옮기면
  compiler→facade→compiler cycle이 생긴다. DTO의 private field/constructor는 projection 및
  encoder와 같은 crate에 있어야 하므로 계약-side 내부 `quanta-gqlang-v2-execution-contract`로
  함께 이동했다. 새 외부 library는 없다. 기존 sha2 dependency도 해당 owner로 이동했다.
- 방향은 facade→Pipeline+execution-contract, Pipeline→execution-contract,
  execution-contract→Algebra/Logic/Planning/Sema다. contract→Pipeline 및 normal Surface 의존은 없다.
  facade 공개 Execution roots는 새 contract crate에서 직접 import한다. 외부 seam은 기존 facade 하나다.
- consumer는 기존 facade-only sibling/compiler entry와 staged SHOW/scalar continuations다.
  acceptance는 public roots156 및 private constructor 유지, 동일 source→DTO/diagnostic/byte digests,
  모든 plan-required item 유지, 기존 request exact/short/refund custody, backward edge 없음이다.
- Pipeline `execution.rs`는 compatibility imports만 남는다. DTO/encoding/projector 선언의 두 번째
  사본이나 include/path mount는 없다. Pipeline은 compile orchestration과 error bridge/permits를 유지한다.

### 발행 경계·자원·호환성

- public facade에 raw constructor, producer module 또는 mutable field를 추가하지 않는다.
  `ExecutionDocumentV3::new`와 `ExecutionItemV3::new`는 contract crate 내부로 유지한다.
- 내부 `producer::ExecutionProjectionV3`는 move-only item Vec를 소유한다. checked plan과 stage
  projections로 item을 만들고 자신의 buffer를 document로 넘긴다. raw item Vec/외부 digest를 받지 않는다.
  내부 owner crate에 직접 의존하는 코드는 지원하는 downstream surface가 아니다.
- 일반 source와 lexical source compile은 이 owner를 사용한다. SHOW는 original count의 capacity를
  사전 확보하고 기존 semantic/plan agreement check 후 같은 buffer에 삽입한다.
- scalar는 `repr(transparent)` move-only issued item wrapper를 쓴다. Pipeline이 기존 request root로
  projection과2개 item slots를 예약한 다음 contract가 exact1 Vec를 만들고 document를 seal한다.
  중간 변환 buffer/extra allocation은 추가하지 않는다. permit chain과 drop 시점은 Pipeline 소유다.
- public names/accessors와 source schema3/contract·qualification schema8/field order/shared vectors는
  유지한다. Rust definition path는 이동하므로 consumer는 새 source closure로 재빌드해야 한다.
  allocator/RSS/latency 비교 및 Semantica consumer/runtime 실행은 NOT_RUN이다.

### 검증·gate·문서

- `cargo test --offline -p quanta-gqlang-v2-execution-contract -p quanta-gqlang-v2-pipeline`:
  **171 Rust tests +3 doctests PASS**, failed0/ignored0. contract unit40, Pipeline unit30,
  integration101이며 기존 exact-byte/stream/digest, source assembly/command, SHOW와 scalar custody 포함.
- 외부 gate는 facade의 dependency2개를 정확히 검사하고 Execution roots가 contract에서,
  compile/authority roots가 Pipeline에서 오는지 검사한다. owner drift/missing/extra/path/feature mutation
  controls를 추가했다. scoped Python owner24/24 PASS (`dto-consumer-policy-tests.log`).
- 새 d24 compile-fail probe는 facade에서 내부 `ExecutionProjectionV3` import를 거절해야 한다.
  기존 d15 private document constructor probe와 함께 full qualification에서 실행한다.
- architecture exact15 members/새 normal·dev edges/sha2 owner, root+fixture lockfiles,
  moved test-module guards 및 fake-guard refusal fixture를 갱신했다. 이동한 contract unit tests를
  production으로 잘못 세지 않도록 cfg(test) owner/path inventory도 옮겼다.
- 생성 instruction은 source에서 dependency DAG를 수정하고 `just pm-sync`, `just pm-check` PASS.
  최초 prompt byte ratchet growth는 내용을 줄여 해결했으며 budget/ratchet 상한은 올리지 않았다.
- 최초 compile 단계의 이동된 lib name/unused exports/dev dependencies, 새 opaque carrier의 accessor와
  test Debug, Python workspace count assertion을 수정했다. 초기 로그는 ignored audit 디렉터리에
  보존한다. 현재 owner proof는 `dto-owner-tests.log`; full 성공은 현재 HEAD receipt로만 판정한다.
- 최초 full qualification `f7b79e9d82485411c0ffc3535bd01008eff52678`는 새 contract `lib.rs`의
  1202 code lines에서 size gate exit1로 중단했다. 해당 SHA의 receipt/log를 보존한다.
  Rename Command DTO와 owner projection을 private `command.rs`로 분리하고 root에서 기존
  public 이름을 재공개한다. field visibility는 contract crate 내부이며1200 상한/예외는 바꾸지 않았다.
- normative frontend contract, ADR-0003, language owner stack, test ledger와 QFC-03를 함께 갱신했다.
  초기 F01 subtree match는 과거 입력 증거다. D07은 facade/pipeline subtree와 dependency closure를
  바꾸므로 QFC-02/E13에서 paired source pins·consumer proof를 갱신해야 한다. Semantica 파일은 수정하지 않았다.

## 이전 D01/D02 실행과 증거 (D03 전환 전 snapshot)

| 검증 | 현재 결과 |
| --- | --- |
| Modules 변경2개 AST parse | PASS |
| `cargo test --locked --offline -p quanta-gqlang-v2-modules --test v3_modules_v2_authority` | PASS120/120, failed0/ignored0/filtered0 |
| production AST census 재대조 |160 files/2871 functions,80-token 이상 exact body groups2→0. macro expansion 제외; 이 시점 D03 의미상 겹침은 별도 OPEN이었다 |
| repository `just ci` | PASS, exit0. Nextest1514/1514, failed0/skipped0/leaked0; doctest10; frontdoor10; facade positive1/negative23; size283 files 및 current/MSRV Clippy·정책 통과 |
| clean committed `just qualify-local` | NOT_RUN |
| Semantica bootstrap/owner/product | NOT_RUN; audit F02 및 기존 E13 owner |

두 수정의 동작 변경 의도는 없다. 이는 구현 공통화와 focused regression 성공이며,
source commit/clean qualification 또는 downstream product 완료를 뜻하지 않는다.
최신 남은 실행과 종료 상태는 [actions.md](actions.md)의 QFC-03~05에서 관리한다.

Local CI source snapshot: `a41b0fa64adc3c3725e14efe3ec634705a54629fc64d6eb06b9a34e842518f42`;
qualification run ID: `b2d1b5fb9762ed9cc5c4d510f799f418`.
`.audit/oct10-frontend-closeout/ci-summary.json`은 CI log hash와 당시 source478개 file hash,
nextest 원문/metadata 복사본을 보관한다. 원문 inventory/events hash readback은 일치했다.
이후 D03과 실행 결과를 문서에 추가했다. Rust2개 postimage는 동일하지만 최종 문서 source는
위 CI snapshot과 다르다. 따라서 최종 checkout 전체/committed HEAD qualification 성공으로 기록하지 않는다.

## 최종 source와 qualification 판정

D01~07 source 수정과 focused owner 검증을 묶어 로컬 main에 커밋한다.
commit SHA를 문서에 다시 삽입해 qualification 입력을 바꾸지 않는다. 현재 checkout의
`git rev-parse HEAD`로 `target/qualification/<HEAD>/receipt.json`을 조회하고 schema3, exit0,
SHA/tree, clean pre/post custody 및 같은 run의 raw Nextest evidence를 확인한 경우에만
**현재 HEAD local qualified**로 판정한다. receipt가 없거나 실패하면 NOT_QUALIFIED다.
이 packet의 체크박스와 이전 dirty-source CI1514는 receipt를 대체하지 않는다.
paired bootstrap/consumer와 Semantica runtime/product는 별도 OPEN이다.

### 최초 전체 gate 실패와 수정

`43d99433264e8e197eb368e6649141de950f82bd`의 qualification은 size285 통과 후
current Clippy의 새 kernel oracle `needless_range_loop`에서 exit101로 중단됐다.
`target/qualification/43d99433264e8e197eb368e6649141de950f82bd/receipt.json`과
`ci.log`를 보존했다. 해당 초기화 반복을 `iter_mut().enumerate()`로 수정했고
standalone `just clippy-current`는 exit0이다 (`.audit/oct10-frontend-closeout/scc-clippy-repair.log`).
이 focused policy 성공은 전체 gate 성공이 아니며 수정된 최종 HEAD의 receipt가 필요하다.
