# F05 — E11C metric inventory 어휘 복원 + `metric.<name>` 표현식 타겟 (W1)

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
