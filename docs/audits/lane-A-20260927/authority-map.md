# Lane A — SSOT authority map

Snapshot: `/tmp/quanta-audit-laneA-snapshot` (HEAD `eaeb68ea` + 12 dirty overlay, manifest sha256 `473a504a…`). 정적 증거만.

형식: `fact/invariant | owner | representation | producer | consumers | duplicated decision? | evidence`

| fact/invariant | owner | representation | producer | consumers | duplicated decision? | evidence |
|---|---|---|---|---|---|---|
| HIR 문서 무결성(중복 ID, span 경계, depth) | hir | `CheckedHirDocumentOwnedV2` | surface `HirBuilderV1` → `try_new` | binder, sema, format | 아니요 — 유일 checker | hir/src/integrity.rs:324, sema/src/document.rs:64 |
| Token/파싱 구조 | surface | `TokenV1`/`ParsedDocumentV2` | lexer/parser | pipeline `parse_source`, format | 아니요 (연산자 어휘는 예외 — 아래) | surface/src/lexer.rs:20, parser.rs:96 |
| 연산자 적법성 | types | `OperatorRegistryV2` (string-keyed overloads) | op.rs 등록 | sema check/solver, pipeline projection | **예** — surface 문법이 spelling을 독립 결정 (A2-1: `<>`↔`!=`, 사중 `%`/`++`) | types/src/op.rs:17-42,57,652 vs surface/lexer.rs:352-394, expr.rs:194-196,382-404 |
| Builtin 적법성/효과 | types | `BuiltinRegistryV2` | builtin.rs 등록 | sema, execution | 아니요 — id/name·count pinning 테스트 | types/src/builtin.rs:176,223, test :547 |
| 이름/스코프 해석 | binder | resolved carriers | binder | sema | 아니요 (Lane C가 심화 확인) | binder/src/resolve.rs |
| 모듈/catalog/가시성 | modules | catalog/graph | manifest/lockfile/catalog | sema, pipeline(dev) | 아니요 (Lane C 소관) | modules/src/catalog.rs:720 |
| 문서 단위 checked semantic transaction | sema | `SemanticDocumentV2`(sealed), `TypedLiteralFactV2` | `resolve_checked_document_v2` | algebra, logic, planning, pipeline | 아니요 — sealing 후 재해석 경로 없음 확인 | sema/src/document.rs:270,315,428 |
| 관계 대수 lowering | algebra | `CheckedAlgebraNodeV2` | `AlgebraSessionV2` | pipeline, planning | 아니요 (Lane D 소관) | algebra/src/normalize.rs:38, node.rs:12-118 |
| Rule program/stratification | logic | `RuleProgramContractV2`+proof | `from_document` | pipeline `compile_rule_program` | 아니요 — pipeline에서 의도적 미반복(주석 명시) | logic/src/source.rs:47, pipeline compile.rs:1486-1491 |
| Graph witness | logic | `WitnessContractV2` | witness.rs | planning/pipeline | 아니요 (Lane D) | logic/src/witness.rs:113 |
| plan-required-ness / NoPlanReason | planning | `NoPlanReasonV2`, `PlanOutcomeV2` | `plan_outcome` | pipeline assembly | 3중 열거이나 cross-validated fail-closed — 의도된 방어 | planning/src/item.rs:56, plan.rs:286,296; pipeline compile.rs:1882,1906,1666 |
| capability/effect/evidence/cost/result-kind | planning | `SemanticPlanV2`(private fields) | `PlanningSessionV2` | pipeline projection | 아니요 — 단일 소유지 | planning/src/plan.rs:131,265 |
| item membership/assembly | pipeline | `DocumentMembershipV2` (BTreeMap) | `run_document` | assembly | 아니요 — 단일 검증지 + exact-key | pipeline compile.rs:1140,1252,1769 |
| catalog payload kind | hir→planning | `CatalogPayloadV1`→`CatalogDeclarationKindV2` | parser | sema→modules→planning→pipeline chain | 열거 복제 6곳 — 전부 exhaustive match로 빌드 강제(A2-4) | hir/item.rs:651, sema/document.rs:1455, modules catalog.rs:720, planning/item.rs:83, compile.rs:1607 |
| 루트 family 분류 | surface | `SourceRootFamilyV3` | `classify_parsed_root_v3` | pipeline route 선택 | 아니요 — pre-authority hint, 문서화됨 | surface/parser.rs:155-181, compile.rs:587-592 |
| 자원 상한 | sema | `FrontendBudgetV2` (NonZeroU32) | `try_new` receipt | 전 단계 | **부분** — 4곳 수동 병열 나열, 무가드 drift 방향 존재 (A2-3) | sema/authority.rs:366,400; surface/budget.rs:44; compile.rs:793-801 |
| 분석 상태(Incomplete/Error/Complete) | pipeline | `AnalysisDocumentV2::status` | pipeline | CLI, stdio | 아니요 — front doors에 재유도 금지 주석 | compile.rs:512-516, v3_diagnostics.rs:26 |
| diagnostic code registry | 각 crate | diagnostic.rs `as_str`/`code()` | 각 owner | facade callers | **예**(2건) — (i) stdio가 pipeline private const를 문자열 재하드코딩 + cap 재구현 (ii) pipeline이 algebra/logic/planning code 어휘를 독립 열거(A3-1) | pipeline/compile.rs:82,91,92,858,883; diagnostics-stdio v3_diagnostics.rs:35,68-83; pipeline/diagnostic.rs:201,255,343 + lib.rs:36-39 |
| bridge code 검증 oracle | pipeline tests | 4개 테스트 | — | — | **예** — logic만 owner `.code()` 교차 단언(:643,677,719,746), algebra(:578)·planning(:750)은 리터럴 자기확인, `no_plan_code`는 coarse 동작을 pin(:913) | pipeline/src/diagnostic.rs:578,622,750,879,913 |
| NoPlanReasonV2 spelling | planning(+pipeline) | suffix / full / full-kindless 3종 | planning item.rs:129, diagnostic.rs:247; pipeline diagnostic.rs:343 | CLI(stdout), assembly 진단 | **예** — CLI는 suffix만 출력해 catalog kind 소실(v3_plan.rs:44); `no_plan_code`는 kind 무시(:345, production 미도달) | planning/src/item.rs:129, planning/src/diagnostic.rs:247, pipeline/src/diagnostic.rs:327-345, cli/src/v3_plan.rs:44 |
| sema forge seam(테스트 전용 봉인 해제) | sema | `pub` + `feature = "test"` | sema | workspace dev edges만 (tools/architecture/policy.toml:78-81) | **예** — gate 없는 public forge 표면: sema는 `publish = false` 아님, `forged_membership`/`forged_with_calls`/`test_support`가 정상 빌드에서 활성화 가능(A3-4) | sema/src/document.rs:457, sema/src/rule.rs:159, sema/src/lib.rs:18; gate 커버리지는 tools/lint/semgrep_gate.py:37-43 뿐 |
| facade 노출 범위 선언 | frontend-contract | 모듈 doc vs allowlist | lib.rs | 외부 소비자 | **예** — doc은 Algebra/Planning unreachable 주장(:7-9)이나 :46,:65가 `NoPlanReasonV2`/`SemanticPlanV2`/`AlgebraFamilyV2` 재수출(A1-2) | frontend-contract/src/lib.rs:7-9,46,65; algebra/src/node.rs:12 |
| digest schema | pipeline | tag fns + `qglang.execution.contract.v5` | `ExecutionDocumentV3` | downstream | 경쟁 아님 — 단 plan payload 전체 미커버 (A2-5, handoff F) | pipeline/execution.rs:1917,1930-1958 |
| 공개 facade | frontend-contract | 큐레이션 재수출 | lib.rs allowlist | CLI, stdio, conformance, external | 소비자 전부 순수 projection — 단 pipeline `pub use` 체인으로 내부 타입 도달 가능 (A1-2, handoff E) | frontend-contract/src/lib.rs:16-64, pipeline/src/lib.rs:28-59 |
| workspace 의존성 DAG | tools/architecture | policy.toml exact-allowlist | manifests | CI gate | 아니요 — 실제 DAG와 26+5+0 정확히 일치; 단 edge 제거는 비감지 (A1-1) | policy.toml:32-72, check_architecture.py:477-485 |
