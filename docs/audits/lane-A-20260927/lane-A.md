# Lane A — IR 목록 / 의존성 아키텍처 / SSOT authority map

감사 실행 보고서. 구현 변경 없음.

## 1. 소스 경계

- Snapshot: `/tmp/quanta-audit-laneA-snapshot` (불변 사본; 원 checkout 미수정)
- 원 HEAD: `eaeb68ea2711d411616511713fb016d828cd80be`, branch `main`
- Dirty 경로 12개(오버레이 포함): `crates/quanta-gqlang-v2-cli/src/main.rs`, `cli/tests/v3_cli_fmt_idempotence.rs`, `pipeline/src/compile.rs`, `surface/src/format.rs`, `surface/src/lib.rs`, `surface/tests/v3_format_hellgate.rs`, `surface/tests/v3_surface_budget.rs`, `docs/language/README.md`, `docs/language/test_hellgate_ledger.md`, untracked `pipeline/tests/v3_format_budget.rs`, `docs/audits/`, `docs/bugbash/sep-26-test-audit/P3-format-closure-2026-09-27.md`
- 파일 내용 hash manifest: `MANIFEST.sha256` (372 files), manifest sha256 `473a504a4bea4d057d666b9fce3e907bbd6639645dbbd74d84eac6d664e78ebd` — 조사 전후 동일(SOURCE_STABLE). escaping symlink 없음.
- snapshot 식별자는 공개 `ExecutionDigestV3`와 무관하며 frontend authority/runtime provenance로 사용하지 않았다.

## 2. 실행 구조

- 병렬 2개 sub-lane: A1(DAG vs policy — coordinator가 subagent 실패 3회 후 직접 수행), A2(IR/registry/authority — subagent 완료).
- 실행한 check: `python3 tools/architecture/check_architecture.py --root <snapshot> --format json` → `passed`, normal_edges=26/26, dev_edges=5/5, build_edges=0, members=14, exit 0. cargo build/test 미실행(정적 감사; 공유 target 경쟁 회피).
- A2 핵심 주장(`<>` lexer/registry 불일치, 사중 `%`/`++` 불가, stdio 하드코드 const)은 coordinator가 grep으로 독립 재확인하여 일치.

## 3. 실제 의존성 DAG (A1)

Normal edges (26) — policy `tools/architecture/policy.toml:41-68`와 정확히 일치, 위반 0, 사이클 없음, 문서화된 stage 방향(surface→hir→binder/types/modules→sema→algebra/logic→planning→pipeline→frontend-contract)과 역전 없음:

```
surface→hir, types→hir, binder→hir, modules→{hir,types},
sema→{binder,hir,modules,types},
algebra→{sema,types}, logic→{sema,types},
planning→{algebra,logic,sema,types},
pipeline→{algebra,logic,planning,sema,surface},
frontend-contract→pipeline,
cli→frontend-contract, conformance→frontend-contract, diagnostics-stdio→frontend-contract
```

Dev edges (5): algebra/logic/pipeline/planning→sema(feature `test`), pipeline→modules. Build edges 0. External: modules/pipeline `sha2=0.10.9`, cli `libc=0.2.186`, diagnostics-stdio `serde`/`serde_json`, surface/pipeline dev `proptest`, surface dev `sha2`.

### Policy 실효성 (check_architecture.py)

| 규칙 | 시행 지점 | 실제 DAG 상태 | 우회 가능성 |
|---|---|---|---|
| internal edge exact-allowlist (kind별 독립) | check_architecture.py:477-485 (집합 차집합) | 일치 | 낮음 — metadata+manifest 이중 검사 |
| external per-crate/kind allowlist + version/feature pin | :251-269, :414-440 | 일치 | 낮음 |
| forbidden prefixes(legacy/runtime) | :185-194 | 해당 없음 | 낮음 |
| target/rename/optional/default-features 거부 | :195-215, :353-369 | 해당 없음 | 낮음 |
| custom build 거부(build default-deny) | :245-249, :298-309, :466-472 | 해당 없음 | 낮음 |
| feature forwarding drift | :328-341 | 해당 없음 | 중간 — forwarding만 검사, feature 정의 자체의 의미는 미검사(설계상 한계) |
| workspace member drift | :457-462 | 일치 14 | 낮음 |
| **edge 제거(누락) 감지** | 없음 — `final_present`는 summary 표기만 | — | **없음이 에러가 아님** (아래 A1-1) |
| crate 경계 통과 ≠ 의미 중복 | (구조상 불가) | — | DAG gate는 의미 복제를 못 잡음 — A2가 담당 |

CI 연결: `Justfile:46-48 arch-lint`가 `check_architecture.py` + 자체 unittest를 실행, `just ci`(:101)에 포함. empty scan 불가(member drift 검사가 policy 목록과 대조).

## 4. 대표 경로 및 진입점 (A2, 심볼 검증)

`compile_source_document_v3` (pipeline/src/compile.rs:618) → `parse_source`(:818, budget 강제 + `project_parse_budget`:788) → `CheckedSyntaxDocumentV2::try_from_hir`(sema/src/document.rs:64 → HIR 유일 integrity checker `CheckedHirDocumentOwnedV2::try_new`, hir/src/integrity.rs:324) → `resolve_checked_document_v2`(sema/src/document.rs:1430; Declared→Resolved→modules catalog 사전 검증→typed bodies) → `run_document`(:1136, `DocumentMembershipV2::of`:1769 중복/누락/외래/wrong-kind 검증) → `run_item`(:1346) owner dispatch: `AlgebraSessionV2.lower_relation_item`(algebra/src/normalize.rs:38), `GraphQueryContractV2::from_resolved`(logic/src/graph.rs:244), planning `CheckContractV2`/`PresentationContractV2`, `RuleProgramContractV2::from_document`(logic/src/source.rs:47, stratification 1회 과금·proof 보존) → `PlanningSessionV2::plan_outcome`(planning/src/plan.rs:286) → `validate_produced_membership`(:1252) exact-key → `CompiledDocumentV2::try_new`(:127) → `ExecutionDocumentV3::new`(closure Dangling/Foreign 검사).

별도 진입점: `analyze_source_document_v2`(:688, open authority; `incomplete_open_analysis`:903는 위조 없음), `classify_single_source_root_v3`(:592, pre-authority route hint), `format_source_document_v2`(:751, 출력 재파싱 + span 재앵커링). CLI/diagnostics-stdio는 facade 순수 projection(재유도 없음).

## 5. Findings (통합)

### A2-1 | P3 | CONFIRMED_DEFECT (drift; 동작은 fail-closed)
연산자 어휘가 surface 문법과 `types::OperatorRegistryV2`에서 독립 열거되어 양방향 drift.
- (a) `<>`가 lexer에서 토큰화되고(lexer.rs:368) 파서가 비교로 수용(expr.rs:195)하나 registry에는 `!=`만 존재 → types 단계 `UnknownOperator`(check.rs:2304-2312)로 사망. parser 단계의 명시적 v2-reject 정책경로가 아님.
- (b) `%`(id 105), `++`(id 118) overload가 op.rs:24,28에 pinning 테스트(:60-100, count-67)로 고정되어 있으나 이를 만들 lexer 규칙·precedence level이 없음(lexer.rs:352-394 대조) → 도달 불가능한 사중 코드 + 테스트가 죽은 항목을 '정상'으로 제도화.
- 경로: 임의 source → parse_source → HIR Binary 심볼 문자열 → types check. trigger: `a <> b` 입력 / 신규 연산자 추가.
- 최소 fix: 연산자 spelling 단일 테이블을 lexer/expr/registry가 공유하거나, `<>`→`!=` HIR lowering 정규화 + 사중 overload 삭제. owner: types+surface. regression oracle: `<>`에 대한 parser-단계 거절 진단 + `%`/`++` 부재를 강제하는 registry-surface 동기화 테스트.
- 반증 시도: `"<>"` 정규화 지점 전역 탐색(lexer.rs:368, expr.rs:195 외 없음), wildcard match 없음 확인.

### A2-2 | P3 | STRUCTURAL_RISK
`"pipeline.analysis.diagnostic_limit_exceeded"`가 pipeline private const(compile.rs:92)인데 diagnostics-stdio가 문자열로 재하드코딩(v3_diagnostics.rs:35)하고, handler가 pipeline이 이미 시행하는 cap(compile.rs:858/883)을 재구현(:68-83). pipeline 측 rename 시 stdio가 같은 protocol code·다른 메시지로 조용히 미스매치. fix: const를 frontend-contract로 export하고 재검사 삭제 또는 단일 authority 유지. owner: pipeline+diagnostics-stdio.

**2차 강화(근거 추가)**: 같은 파일에서 `OPEN_AUTHORITY_INCOMPLETE_CODE`(compile.rs:82)와 `PARSE_BUDGET_PROJECTION_FAILED_CODE`(:91)는 `pub`이고 pipeline lib.rs:36-39를 통해 재수출되며 facade도 `OPEN_AUTHORITY_INCOMPLETE_CODE`를 노출한다(frontend-contract/lib.rs:42). 즉 crate 내 const export 정책이 이미 존재하는데 diagnostic-limit code만 private로 남아 소비자가 문자열 복사를 강제당한다 — 의도가 아니라 누락이라는 근거.

### A2-3 | P3 | STRUCTURAL_RISK
budget 차원이 4곳에 수동 병열 나열: sema/authority.rs:366 `FrontendBudgetV2`(23차원), :400 `DownstreamBudgetReceiptV2`(복사, guard :693 존재), surface/budget.rs:44 `ParseBudgetV2`(7차원), compile.rs:793-801 `project_parse_budget`(7개 명시 getter). compile guard 없는 drift 방향: `FrontendBudgetV2`에 parse-relevant 차원 추가 시 surface/pipeline 미수정이면 파싱이 조용히 무제한. fix: 공유 차원 enum/테이블 또는 getter 기반 projection. owner: sema(authority).

### A1-1 | P3 | PROOF_GAP
architecture gate는 edge 추가만 감지하고 제거(누락)는 감지하지 않음(check_architecture.py:477-485 집합 차집산 방향; `final_present`는 :490 요약 표기). policy에서 edge를 지우고 manifest에서도 지우면 통과 — 의도된 최대치 정책일 수 있으나 "exact-allowlist" 주석(policy.toml:4-5)과의 정합 근거가 없다. fix: `FINAL_*_EDGES - actual`을 경고 또는 에러로 보고. owner: tools/architecture.

### A2-4 | P3 | OPTIONAL_REFACTOR (guarded sync surface)
catalog payload kind variant 추가 시 ~6개 exhaustive match 통과 필요: hir/item.rs:651, sema/document.rs:1455(+1094-1131), modules catalog.rs:720, planning/item.rs:83(+code/name matches), pipeline compile.rs:1607. wildcard 없음이 확인되어 빌드 강제 — 결함이 아니라 add-variant 체크리스트로 문서화 권고.

### A2-5 | P3 | PROOF_GAP (handoff → Lane F)
`ExecutionDocumentV3` contract digest(execution.rs:1930-1958)는 plan tag + relation/composition projection을 hash하고 `SemanticPlanV2` 전체 payload(capability, cost, explain, policy 상세)는 제외. 미태그 필드만 다른 두 문서가 동일 digest. schema version v5로 의도일 가능성 있으나 downstream integrity 소비자 관점 명시 필요.

### A2-6 | P3 | DOCUMENTATION_DRIFT
format.rs:579 precedence 표의 `"%"` — lexer가 만들 수 없는 사중 항목. A2-1(b)와 동일 root.

### A1-2 | P3 | DOCUMENTATION_DRIFT (handoff → Lane E)
facade 재수출 체인: frontend-contract(lib.rs:16-64) → pipeline(lib.rs:28-59) → algebra/sema/planning 타입 재노출. **2차에서 확정**: facade 모듈 doc은 "owner internals (HIR, Binder, Types, Modules, Algebra, Logic, Planning, Pipeline) stay unreachable"(frontend-contract/lib.rs:7-9)고 선언하지만, 같은 파일 :65가 algebra 소유 타입 `AlgebraFamilyV2`(algebra/src/node.rs:12)를 "inert value/location closure types"로 재수출한다. 선언과 실제 노출이 모순이며, 큐레이션 의도(닫힘 타입 허용)와 문서 문구 중 무엇이 authority인지 미결정. `SemanticPlanV2`/`NoPlanReasonV2`(:46)도 planning 소유 타입이다. fix: 문구를 실제 allowlist 기준으로 정정(owner internals 중 닫힘 값 타입은 노출) 또는 노출 축소. owner: frontend-contract.

## 5b. 2차 이터레이션 신규 findings

### A3-1 | P2-P3 | CONFIRMED_DEFECT (중복 결정 + 비대칭 검증)
pipeline이 상류 4개 crate의 진단 code 어휘를 **독립 리터럴 열거로 재구현**하고 공개 재수출한다: `algebra_code`, `logic_code`, `no_plan_code`, `planning_code`(pipeline/src/diagnostic.rs:201,255,343 + lib.rs:36-39). owner 측 대응 구현은 algebra `AlgebraFailureKindV2::code`(suffix)/`AlgebraDiagnosticV2::code`(full)(algebra/src/diagnostic.rs:33,71), planning `NoPlanReasonV2::code`(planning/src/item.rs:129)·`CatalogDeclarationKindV2::code`(:101)·`PlanningDiagnosticV1::code`(조합형 `format!`, planning/src/diagnostic.rs:227), logic `LogicDiagnosticV1::code`(logic/src/diagnostic.rs:235).

검증 비대칭(4개 브리지 중 1개만 oracle 보유):
- `every_logic_bridge_code_matches_an_independent_literal`(:622)는 브리지와 owner를 **둘 다** 단언 — `.code()` 단언 4건(:643,677,719,746).
- `every_algebra_bridge_code_is_an_independent_literal`(:578)는 리터럴만 단언, `.code()` 단언 0건.
- `every_planning_bridge_code_matches_an_independent_literal`(:750)는 리터럴만 단언, `.code()` 단언 0건(테스트 본문 750-921 전수).
- `no_plan_code`도 리터럴만(:879) + coarse 동작을 명시적으로 pin(:913).

즉 algebra/planning 어휘는 **양측이 독립 규칙을 유지하고 drift를 감지할 oracle이 없다**. planning 쪽은 생성 방식 자체가 다르다(owner는 접두어+`format!` 조합, pipeline은 variant별 완전 리터럴 30개). trigger: owner가 suffix/접두어를 rename하면 pipeline 리터럴은 그대로 남아 같은 진단이 경로에 따라 다른 code로 노출. 현재 관측된 drift는 A3-2 1건(public API 한정). 최소 fix: logic 선례(:745-746)대로 algebra/planning 브리지 테스트에 owner `.code()` 교차 단언을 추가하거나, bridge를 owner 함수 위임으로 축소. owner: pipeline(+algebra/planning), regression oracle: 3개 브리지 전부에 owner-side 동등 단언.

### A3-2 | P3 | STRUCTURAL_RISK (public API 내부 모순, production 미도달 확인)
`no_plan_code(NoPlanReasonV2::CatalogDeclaration { kind: _ })`는 kind를 버리고 `"planning.no_plan.catalog_declaration"`을 반환한다(pipeline/src/diagnostic.rs:345). 그러나 owner `PlanningDiagnosticV1::code()`는 `format!("planning.no_plan.{}.{}", reason.code(), kind.code())`로 kind 접미어를 붙이고(planning/src/diagnostic.rs:247), pipeline 자신의 `planning_code()`도 kind별 6개 리터럴을 열거한다(:327-333). 동일 사실에 대해 pipeline 내부 두 함수가 서로 다른 답을 준다(:345 coarse vs :327-333 full).
반증 시도 및 결과: production 경로는 `match reason.catalog_kind() { Some(kind) => <kind별>, None => no_plan_code(...) }`(:326-336)이므로 CatalogDeclaration은 항상 Some 분기를 타 coarse 리터럴에 **도달하지 않는다** — 따라서 현재 오동작이 아니라 public helper의 계약 모순 + 테스트가 그 모순을 pin(:913). fix: `no_plan_code`에서 unreachable한 catalog 분기를 제거하고 kind를 요구하도록 시그니처를 좁히거나 `None` 전용으로 재정의. owner: pipeline.

### A3-3 | P3 | STRUCTURAL_RISK (front-door 정보 손실, handoff → Lane E)
동일 사실(NoPlanReasonV2)에 대해 서로 다른 3개 spelling이 존재한다: (1) owner suffix `NoPlanReasonV2::code()`(planning/src/item.rs:129, catalog kind 무시) (2) owner full `PlanningDiagnosticV1::code()`(kind 포함) (3) pipeline `no_plan_code`/`planning_code`. CLI v3_plan.rs:44는 (1)을 그대로 출력해 `item[i].no_plan_reason=catalog_declaration`이 되고, catalog 6종(view/model/pack/profile/gate/rule_binding)이 한 문자열로 붕괴한다. 계약 문서는 DTO 필드만 규정하고 CLI 텍스트 형식은 규정하지 않으므로, 이것이 의도된 축약인지 미결정. owner 판단 필요: front-door 출력에 kind를 복원할지, 축약을 계약으로 문서화할지. handoff: Lane E(front door), 근거는 Lane A authority map.

### A3-4 | P3 | STRUCTURAL_RISK (test feature가 public forge seam)
`quanta-gqlang-v2-sema`는 `publish = false`가 아니며(전 crate 중 conformance만 publish=false), `test = []` feature를 공개한다. 이 feature 하에서 **`pub`** forging API가 열린다: `SemanticDocumentV2::forged_membership`(sema/src/document.rs:457, `#[doc(hidden)] pub`)과 `ResolvedRuleItemV2::forged_with_calls`(sema/src/rule.rs:159, `#[doc(hidden)] pub`), 그리고 `pub mod test_support`(sema/src/lib.rs:18). 즉 "never part of a normal build"(주석 :454-456)라는 서술과 달리, 임의 소비자가 `features = ["test"]` 한 줄로 정상 빌드에서 봉인 해제 seam을 얻는다.
- 현재 workspace 내부에서는 도달 불가: policy가 `sema|dev|*` 4개 edge에만 feature `test`를 허용(policy.toml:78-81)하고 실 DAG 검사가 통과한다. 외부 소비자는 facade를 통해 sema에 도달할 수 없다(facade allowlist에 sema 없음).
- 위조된 문서는 하류 validator가 거부하도록 설계되어 있어(주석의 adversarial rail 목적) 실행 문서 위조로 직결되지 않는다 — **테스트 전용 위조 가능성을 공개 API 결함으로 과장하지 않는다**.
- 잔여 위험은 미래 변경/외부 직접 의존: feature 활성화를 막는 gate가 없고, method-level seam 인벤토리를 감시하는 rail도 없다. 검증: `tools/`에서 `feature = "test"`를 언급하는 파일은 semgrep_gate.py 하나뿐이고, 그 gate는 `test_support` **모듈 가드 정규식만** pin한다(:37-43) — `forged_*` method 추가/`doc(hidden)` 누락은 어떤 gate도 잡지 않는다.
최소 fix: (a) forge seam을 `pub` 대신 `pub(crate)` + test_support 경유로 축소하거나 (b) semgrep/lint gate의 seam 인벤토리에 `#[cfg(any(test, feature = "test"))]` + `pub` 조합을 정확히 열거해 추가·누락 시 실패시키기. owner: sema(+tools/lint), regression oracle: feature 활성화 시 존재하는 public forge 심볼 집합을 정확값으로 pin.

## 6. 의심했으나 반증된 후보

- BuiltinIdV2::name match ↔ registry: owner 테스트가 모든 entry의 id/name·count 21 pinning.
- OperatorIdV2::symbol ↔ overloads: 테스트 보호 — 단, 그 보호가 죽은 `%`/`++`를 제도화(A2-1).
- plan-requiredness 3중 열거(plan_required_ids / run_item / non_plan_reason): 양방향 cross-validation + fail-closed(validate_produced_membership :1252, PlanOutcomeMismatch :1666) — 경쟁 authority가 아닌 의도된 defense-in-depth.
- formatter precedence ↔ parser precedence: Paren 노드 보존 + 출력 재파싱 + idempotence 테스트 pinning.
- classify_parsed_root_v3: 문서화된 pre-authority route hint(compile.rs:587-591), 의미 authority 아님.
- docs/language/README.md stage 목록 ↔ 코드: 심볼 검증 결과 실제 crate 흐름과 일치.

### 2·3차에서 반증·정정된 후보

- **A2-1이 "silent drop"이라는 가설 → 반증**: `<>`는 types에서 `TypeDiagnosticV2::UnknownOperator`(check.rs:2306, code `types.unknown_operator` diag.rs:282)로 typed error가 되고 sema가 `type_failure`(document.rs:1622)로 transaction 실패시킨다. 성공 형태 기본값·부분 성공 없음. 따라서 A2-1은 P3 drift로 유지(격상하지 않음).
- **`no_plan_code` divergence가 현재 오동작이라는 가설 → 반증**: production 호출 경로가 `catalog_kind()==Some` 분기를 먼저 타므로 coarse 리터럴은 미도달(A3-2는 구조 위험으로만 기록).
- **진단 code 문자열 중복 다수(`planning.budget.*` ×4, `pipeline.document.*` ×4) → 대부분 반증**: 발생 지점은 owner 정의 + pipeline bridge + 양측 테스트 pin이며, 각각 단일 owner 규칙을 서술한다. 실제 위험은 A3-1이 지적한 bridge 알고리즘 차이와 oracle 부재에 한정.
- **`forged_*`가 실행 문서 위조로 이어진다는 가설 → 반증**: 위조 seam은 `SemanticDocumentV2`/rule item 수준이며 하류 validator가 거부하도록 설계된 adversarial rail 입력이다. A3-4는 "정상 빌드에서 forge seam이 공개 활성화 가능"이라는 사실과 gate 부재로 한정.
- **format.rs:579 `"%"` precedence 항목이 즉시 오동작이라는 가설 → 반증**: pipeline format 경로는 소스 텍스트를 재파싱하므로 `%` 토큰이 생기지 않는다. caller-built HIR를 직접 format하는 비-파이프라인 경로에서만 의미가 있으며, 도달 경로 미확인이므로 DOCUMENTATION_DRIFT로 유지.

## 7. 이터레이션 로그 (2·3차)

| 라운드 | 수행 | 결과 |
|---|---|---|
| 1차 | DAG vs policy(A1), IR/registry 인벤토리(A2), gate 실행 | normal 26/26·dev 5/5·build 0 `passed`; findings A2-1..A2-6, A1-1, A1-2 |
| 2차 | forge seam(`feature = "test"` 6곳), 진단 code 어휘 중복 전수, facade 선언 vs 실제, const export 정책 대조 | 신규 A3-1(A2-P3), A3-2~A3-4(P3), A1-2 확정(DOCUMENTATION_DRIFT), A2-2 근거 강화 |
| 3차 | 각 신규 finding 반증 시도: `<>` 실패 모드 추적(check.rs:2296-2312 → document.rs:1622), `no_plan_code` 도달 분기, bridge 테스트 4종 `.code()` 단언 계수, semgrep gate 커버리지, CLI no_plan 출력 | A2-1 격상 기각, A3-2 production 미도달 확정, A3-1 비대칭 확정(algebra 0·planning 0·logic 4), A3-4 gate 부재 확정 |

추가 실행 check: `tools/architecture/check_architecture.py --root <snapshot>` 1회(exit 0). 빌드/테스트는 미실행 — 정적 감사 원칙 유지(공유 target 경쟁 회피). A3-1/A3-2의 행동 재현은 pipeline 단위 테스트 추가로 검증 가능하며 본 감사에서는 미실행.

## 8. Coverage / 한계 / handoff

- 조사: 14개 crate manifest 전체, policy/checker/Justfile, IR 생성·소비 경로 정적 추적, registry 7종, 진단 code 어휘 4개 브리지 + 3개 owner 구현, forge seam 6곳, facade allowlist 67줄.
- 미실행: cargo build/test 전체(정적 감사). A3-1 drift는 구조적 위험이며 실제 rename 재현은 미실행.
- 미심화(다른 lane 소유): sema body 의미(C), stratification 알고리즘(D), digest 충돌 분석(F), format 바이트 정규형(B/E), modules lockfile/manifest 내부(C), DTO 노출 상세(E), front-door 텍스트 계약(E).
- 평가: **confirmed defect 2건**(A2-1 P3 fail-closed drift, A3-1 P2-P3 진단 어휘 중복+비대칭 oracle), 나머지는 구조 위험·증명 갭·문서 drift. DAG는 policy와 정확히 일치하며 IR 단계 책임 소재는 선언과 부합 — 전역적 "중복 0개"는 주장하지 않는다(조사 경계는 위 미심화 목록).
