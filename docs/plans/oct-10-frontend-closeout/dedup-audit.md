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

D01~03 source 수정과 focused owner 검증을 묶어 로컬 main에 커밋한다.
commit SHA를 문서에 다시 삽입해 qualification 입력을 바꾸지 않는다. 현재 checkout의
`git rev-parse HEAD`로 `target/qualification/<HEAD>/receipt.json`을 조회하고 schema3, exit0,
SHA/tree, clean pre/post custody 및 같은 run의 raw Nextest evidence를 확인한 경우에만
**현재 HEAD local qualified**로 판정한다. receipt가 없거나 실패하면 NOT_QUALIFIED다.
이 packet의 체크박스와 이전 dirty-source CI1514는 receipt를 대체하지 않는다.
paired bootstrap/consumer와 Semantica runtime/product는 별도 OPEN이다.
