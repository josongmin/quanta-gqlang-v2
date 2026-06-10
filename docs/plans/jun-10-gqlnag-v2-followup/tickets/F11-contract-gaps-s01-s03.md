# F11 — S01/S02/S03 계약 gap 보수 (W2, semantica)

> **STATUS: PARTIAL (jun-10) — S02 DONE/verified; S01·S03·CandidateBasisV1 BLOCKED/deferred.**
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
