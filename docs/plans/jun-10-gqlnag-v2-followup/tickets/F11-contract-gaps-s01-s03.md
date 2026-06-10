# F11 — S01/S02/S03 계약 gap 보수 (W2, semantica)

> **STATUS: S01+S02+S03 ALL DONE (jun-10) — quanta-runtime 빌드 복구 후 전부 compile+test 검증.**
> - **S01 finding plane DONE (SOTA++):** `exec-contract/src/diagnostics.rs` 신설 — 전체 finding plane: `SeverityV1`/`LocationV1`/`PathWitnessRefV1`/`MetricSnapshotRefV1`/`IssueKindV1`/`IssueWorkflowStateV1`(read-vs-mutate `is_triage_mutation_v1`)/`HotspotStatusV1`/`DeltaStateV1`/`ExternalOriginV1`/`SuppressionReasonV1`/`CandidateBasisV1{Heuristic,Inferred,Certified}`(RFC 교정셋)/`RankingMetadataV1`/`DiagnosticProjectionV1`(18필드, truth.rs 의 TruthClass/Outcome/CoverageStatus 재사용). RFC 불변식 코드 인코딩: `admits_through_gate_v1()==false`(ranking 은 gate-allow 아님), `derived_truth_class_v1()==None`(ranking 은 truth_class 안 올림), RankingMetadata 에 truth_class 필드 부재(구조적 분리). `validate_v1` fail-closed(외부origin은 ExternalIssue 일때만).
> - **CandidateBasisV1 충돌 해소:** `result.rs` 의 잘못된 `CandidateBasisV1{Lexical,Semantic,Hybrid,Fragment}`(실제론 match basis) → `CandidateMatchBasisV1` 로 rename + 소비처 2곳(ai_workflow) 갱신. finding-plane CandidateBasisV1 가 canonical.
> - **conformance** `diagnostics_issue_ranking_contract.rs` 신설: shared projection / external provenance fail-closed / read-vs-mutate split / ranking-never-admits-nor-raises-truth — **4 tests GREEN**.
> - **검증:** exec-contract 17 + diagnostics conformance 4 GREEN.
> - (S03 의 seam 테스트는 빌드 검증 중 plane 가정 오류 발견·수정: Resolve=Retrieval, PatternLint=ProductWorkflow, 나머지 6=Analysis 로 정밀화.)
>
> 이전 노트:
> - **S03 DONE (static-verified):** `host_spi_contract.rs` 에 8개 analysis-trait(Resolve/Relation/Path/TaintAnalysis/PointsTo/Summary/Explain/PatternLint) DummyHost impl 추가(기존 serving-plane 10개와 동일 compile-only 패턴) + 비공허 테스트 `seam_inventory_covers_all_analysis_hosts_v1`(8개 Analysis-plane seam 존재 assert). 적대검증: 8 trait + 8 ServingHostKindV1 variant + 23 contract type 전부 소스에 존재 확인(시그니처는 trait body 에서 직접 추출). cargo 는 conformance→runtime 블록이라 미실행, 타입 전수 resolve 정적확인.
>
> - **S01 finding-plane — DEFER (날조 회피):** S01 티켓이 `DiagnosticProjectionV1` 의 top-level + enum(IssueKindV1/IssueWorkflowStateV1/HotspotStatusV1/RankingMetadataV1/`CandidateBasisV1{Heuristic,Inferred,Certified}`)은 명세하나, `DiagnosticProjectionV1` 가 참조하는 leaf 7종(`SeverityV1`/`LocationV1`/`PathWitnessRefV1`/`MetricSnapshotRefV1`/`DeltaStateV1`/`ExternalOriginV1`/`SuppressionReasonV1`)은 shape 미명세 → RFC-002 Appendix 확인 필요(현재 exec-contract 부재). 3종(TruthClassV1/OutcomeV1/CoverageStatusV1)만 존재. leaf shape 확보 후 `exec-contract/src/diagnostics.rs` 신설(빌드 가능, 검증 가능).
> - **S01 CandidateBasisV1 교정 — DEFER:** spec 은 `{Heuristic,Inferred,Certified}` 인데 `result.rs` 의 현행 `CandidateBasisV1{Lexical,Semantic,Hybrid,Fragment}`(실제론 retrieval-mode 의미)와 이름 충돌. 올바른 fix 는 result.rs 것을 `CandidateRetrievalBasisV1` 등으로 rename + 필드/소비처 재배선인데, 소비처가 차단된 ai_workflow bridge/conformance 테스트라 검증 불가(ripple). 빌드 복구 후.
>
> 이하 원래 S02 노트:
> - **S02 DONE (exec-contract, verified green):** `ExportKindV1` 에 wire-mapping SSOT 추가(`request.rs` `as_wire_str_v1`/`from_wire_str_v1`), `BridgeReceiptV1::validate_v1`(`command_bridge.rs`)에 closed-set 검사 추가 → 비공집합 `export_kind`("garbage")가 `command_bridge_receipt_invalid_export_kind` 로 fail-closed. 신규 테스트 3개(positive/negative/round-trip) + 기존 13개 `cargo test -p quanta-gqlang-v2-exec-contract` GREEN. 적대검증: 차단된 producer 들이 쓰는 "command_bridge"/"working_set" 는 모두 수용 → 무리플.
> - **환경 블로커:** semantica 워크트리의 미커밋 `quanta-runtime` WIP(`taint/types.rs` → `quanta_contract_types::security` 미해결, `pta_solver::PointsToResult` 필드 드리프트)가 host-adapter/ingress-bridge/exec-conformance 빌드를 막음. 따라서:
>   - **S03 (8 analysis-trait conformance coverage)**: host_spi_contract.rs 가 exec-conformance(→bridge→quanta-runtime) 빌드를 끌어 **테스트 실행 차단**. 코드 추가는 가능하나 green 증명 불가 → 블로커 해소 후 진행.
>   - **S01 finding plane (DiagnosticProjectionV1 등) 추가**: exec-contract 에 추가 자체는 가능(빌드됨)하나, 의미있는 conformance 배선이 차단 → 블로커 후.
>   - **S01 CandidateBasisV1 variant 교정({Lexical,Semantic,Hybrid,Fragment}→{Heuristic,Inferred,Certified})**: 소비처가 차단된 ai_workflow bridge/conformance 테스트 → variant 변경 시 검증 불가(ripple). 블로커 후.
> - exec-contract 만 단독 빌드/테스트 가능(leaf wire crate 의존) — S02 는 그 안에서 완결.



원천: S-contract-spi-proof 감사. 셋 다 claimed=complete → verified=partial.
대상 repo: `semantica-codegraph-v2/packages/analysis/quanta-v2/crates/`.

## Gap 목록 (라이브)

1. **S01 — CandidateBasisV1 variant 불일치 + finding plane 부재**
   - `quanta-gqlang-v2-exec-contract/src/result.rs:275-280` `CandidateBasisV1` =
     `{Lexical, Semantic, Hybrid, Fragment}` — 티켓이 의무화한 `{Heuristic, Inferred, Certified}` 0 hits
   - finding/diagnostics plane 전체 미구축: `DiagnosticProjectionV1`/`IssueKindV1`/`IssueWorkflowStateV1`/
     `HotspotStatusV1`/`RankingMetadataV1` workspace 0 hits. contract src 에 diagnostics/issue/ranking 파일 없음
   - `request_result_contract.rs:40,136` residue-rejection 이 provider_backed flag 만 assert —
     `authority_carrier_v2`/`planner_carrier_v1`/`GqlangBoundExecuteRequestV1` 부재 assert 없음
2. **S02 — export_kind 닫힌집합 깨짐**
   - `result.rs:188` `BridgeReceiptV1.export_kind: String`, `command_bridge.rs:29-33` non-empty 검사만 —
     `request.rs:251` 에 닫힌 enum `ExportKindV1{WorkingSet,ReviewProjection,CommandBridge}` 가 60줄 옆에 있는데
     receipt 경계에서 `"garbage"` 가 validate_v1() 통과
3. **S03 — 8개 analysis trait 의 conformance 0**
   - Resolve/Relation/Path/Taint/PointsTo/Summary/Explain/PatternLint Host 가 소스에 존재하나
     tests/ 전체에서 참조 0. `host_spi_contract.rs` DummyHost 는 serving-plane subset 만 impl

## 작업

1. S01: basis variant 정합 (스펙 따라 `{Heuristic,Inferred,Certified}` 로 교정 또는 티켓 개정 — 교정 권장,
   기존 4-variant 는 retrieval mode 로 분리), finding plane 타입 신설, residue-rejection negative 강화
2. S02: `export_kind` 를 `ExportKindV1` enum 으로 교체(또는 validate 에서 enum-parse 강제) + illegal 값 거부
   negative 테스트
3. S03: 8개 trait 각각 DummyHost impl + 호출 + fail-closed 거동 assert 를 host_spi_contract(또는 신규
   analysis_host_contract.rs) 에 추가

## DoD

- 위 named 타입 전부 실재 + negative 테스트 green
- `export_kind:"garbage"` 가 typed code 로 거부됨
- 19개 host trait 전부 conformance 테스트에서 참조됨 (`rg "<TraitName>" tests/` 전수 ≥1)
