# F05 — E11C metric inventory 어휘 복원 + `metric.<name>` 표현식 타겟 (W1)

> **STATUS: DONE (jun-10) — 어휘복원 방향, 헬게이트 3 GREEN, workspace 0 regression.**
> - `sema/src/metric_vocabulary.rs` 신설: `MetricFamilyV1`(9), `EXTERNAL_METRIC_VOCABULARY_V1`(**42 metric, E11C body 자구 1:1**), `metric_descriptor_v1` 조회. 내부 8-counter(`ir/catalog/metric_inventory.rs`)와 **분리**(오염 회피).
> - `metric.<name>` SELECT 타겟: `admit_select_metric_projections_v1` 을 `analyze_root` Select arm 에 배선 → 미지명 metric `select_unknown_metric` fail-closed(문법 무변경). **레이어**: 어휘=sema 소유(ir→sema 의존), ir `metric_inventory.rs` 가 `EXTERNAL_METRIC_VOCABULARY_V1` re-export(catalog plane 노출).
> - hellgate `select_metric_target`(3): 9-family/unique / known admit / unknown fail-closed. workspace 0 fail.
> - **잔여:** docs 재게시(external repo docs 표)는 미반영(코드 const 가 SSOT).


원천: E-shell-pack-catalog 감사 "DECISIVE refutation". 메인 세션 직접 재검증 완료 (0 hits).

## 현재 상태 (라이브)

- `ir/src/catalog/metric_inventory.rs:1-25` `MetricIdV1` = 내부 카운터 정확히 8개
  (QueryResultCount, CandidateSetCount, EvidenceSetCount, CatalogCoverageRatio, CompareAgreementRatio,
  ExternalFindingCount, ExternalHighSeverityCount, SuppressedFindingCount). `:126` rows 배열도 8개
- E11C DoD(:44-52)가 요구한 9개 패밀리 ~50개 metric 어휘(coverage/duplication/complexity/size/
  test-execution/issue-counts/ratings/remediation/hotspot-review)는 **코드에도 docs 에도 없음**
  (`cognitive_complexity|new_coverage|code_smells|security_hotspots|reliability_rating` = crates+docs 0 hits)
- `metric.<name>` typed expression target (SELECT/ASSERT 용) 미존재
- 어휘는 commit `ddee4f9` 에서 docs 로부터 **삭제됨** — 연기(deferral)가 아니라 scope-swap

## 작업

1. 방향 결정 먼저 (둘 중 하나, 기본 권장 = a):
   a. E11C 원래 scope 복원: 9-패밀리 어휘를 `MetricIdV1`(또는 별도 `ExternalMetricIdV1`) 로 추가하고
      catalog rows + docs 재게시
   b. scope 축소를 공식화: E11C ticket body 를 8-counter 로 개정하고 RFC_CROSSWALK/COVERAGE 에 명시
      (이 경우 "landed" 주장 정정 기록 필수)
2. (a 선택 시) `metric.<name>` 을 SELECT/ASSERT 표현식 타겟으로 추가 — syntax 파서 + sema admissibility + IR carrier
3. external findings surface (`ExternalFindingCount` 계열)와 새 어휘의 관계를 metric_inventory 테스트로 고정
4. `ir/tests/metric_inventory.rs` (이미 dirty 상태로 워킹트리에 있음) 를 새 inventory 와 정합화

## DoD

- 결정된 scope 기준으로 ticket body ↔ 코드 ↔ docs 3자 일치
- `metric.<name>` 경로 (a 선택 시) parse→sema→IR round-trip 테스트 green
- 8-counter 와 신규 어휘가 한 registry 에서 충돌 없이 공존

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: `ir/src/catalog/metric_inventory.rs:2 MetricIdV1`(내부 카운터 8개), `:126 METRIC_INVENTORY_ROWS_V1: [MetricDescriptorV1; 8]`, `:201 METRIC_INVENTORY_V1`. 방향 = **(a) 어휘 복원** (티켓 권장안 그대로). 내부 8-counter 와 외부 어휘는 **별도 enum 으로 분리**(섞으면 내부 projection 카운터가 외부 SELECT 타겟이 되는 오염).

### 생성/수정
1. **`ir/src/catalog/metric_inventory.rs` 확장** (같은 파일 — registry 1곳 유지)
   - `pub enum MetricFamilyV1 { Coverage, Duplication, Complexity, Size, TestExecution, IssueCounts, Ratings, Remediation, HotspotReview }` (9-family)
   - `pub struct ExternalMetricDescriptorV1 { pub name: &'static str, pub family: MetricFamilyV1, pub higher_is_better: bool }`
   - `pub const EXTERNAL_METRIC_VOCABULARY_V1: &[ExternalMetricDescriptorV1]` — ~50행: coverage(coverage,new_coverage,line_coverage,branch_coverage,uncovered_lines…), duplication(duplicated_lines_density,duplicated_blocks…), complexity(cyclomatic_complexity,cognitive_complexity), size(ncloc,lines,files,functions,classes,statements…), test-execution(tests,test_errors,test_failures,skipped_tests,test_success_density,test_execution_time), issue-counts(violations,bugs,vulnerabilities,code_smells,open_issues,…), ratings(reliability_rating,security_rating,sqale_rating), remediation(sqale_index,reliability_remediation_effort,security_remediation_effort,effort_to_reach_maintainability_rating_a), hotspot-review(security_hotspots,security_hotspots_reviewed) — **E11C 티켓 body :44-52 의 명명 인벤토리와 자구 대조**해 작성(임의 추가 금지).
   - `pub fn external_metric_by_name_v1(name: &str) -> Option<&'static ExternalMetricDescriptorV1>` + 중복명 0 보장.
2. **`metric.<name>` SELECT 타겟** — 문법 변경 불필요(projection 은 이미 자유 컬럼명): **sema 검증**으로 구현. `sema/src/roots/select.rs` `analyze_select_root_v1` 에서 projection 필드가 `metric.` 접두면 뒷부분을 vocabulary 로 검증 → 미지명은 `select_unknown_metric` fail-closed, 적중이면 통과. ⚠️ 레이어 주의: sema→ir 의존 불가(ir 가 sema 에 의존) → **vocabulary 정의를 `syntax` 또는 별도 공유 위치가 아니라, sema 가 소유**하는 게 맞는지 결정 필요. 권장: vocabulary const 를 **sema** (`sema/src/metric_vocabulary.rs`)에 두고 ir `metric_inventory.rs` 가 re-export(의존방향 ir→sema 는 이미 존재). 내부 8-counter 는 ir 잔류.
3. **docs 재게시**: `docs/`(external repo) 에 metric vocabulary 표 1페이지 (commit ddee4f9 에서 삭제된 것 복원, code 의 const 와 1:1).

### 헬게이트 (red 먼저)
- **`ir/tests/metric_inventory.rs` 확장**(기존 파일): 9 family 전부 ≥1 행 / 이름 중복 0 / `external_metric_by_name_v1("cognitive_complexity")` 적중 / 내부 8-counter 와 이름 교집합 0.
- **`sema/tests/select_metric_target.rs`**(신규): `SELECT metric.cognitive_complexity FROM catalog.files` admit / `SELECT metric.nonexistent ...` → `select_unknown_metric` / 비-metric projection 비파괴.

### 검증
`cargo test -p quanta-gqlang-v2-sema --test select_metric_target && cargo test -p quanta-gqlang-v2-ir --test metric_inventory && cargo test --workspace`.
