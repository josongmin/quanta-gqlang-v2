# IR/SSOT/frontend 병렬 감사 — 확정 실제 문제 통합 (findings-15)

- **대상**: `/Users/songmin/Documents/code-new/quanta-gqlang-v2`, 원 HEAD `eaeb68ea2711d411616511713fb016d828cd80be` (main).
- **출처**: lane A–F 산출물(`docs/audits/lane-A-20260927/lane-A.md`, `docs/audits/lane-b-snapshot-20260927-010019/lane-B.md`, `docs/audits/lane-C.md`, `docs/audits/lane-D-20260927/lane-D.md`, `docs/audits/lane-e/lane-E.md`, `docs/bubgash/sep-27-ir/findings-6.md`) 통합.
- **검증 상태 표기**: `[재확인]` = 본 문서 작성자(Lane D)가 원 트리에서 인용 file:line을 직접 다시 읽어 확인. `[실행 재현]` = 격리 snapshot에서 실행. `[lane 보고]` = 해당 lane의 정적/실행 증거를 그대로 relay(본 문서에서 재실행하지 않음).
- **필터 기준**: 실제 오동작·거짓 거절·정보 손실·우회 가능한 무결성 경계만 남긴다. PROOF_GAP 단독, DOCUMENTATION_DRIFT, OPTIONAL_REFACTOR, 취향/중복 서술 항목은 §3에 축약 제외한다.
- **한계**: lane별 snapshot이 서로 다르다(dirty 오버레이 상이). 아래 항목은 원 HEAD 동일 + Lane D 스코프 clean을 확인한 상태이며, `just ci`/`qualify-local`은 미실행이다. 실행 재현은 IR-01 하나뿐이다.

---

## 1. 확정 결함 (실제 오동작 / 거짓 거절)

### IR-01 · P2 · rule body polarity 이중 유도 → sema-legal 문서가 logic에서 거짓 거절 `[실행 재현]` (Lane D)

- **위치**: 생산자 `crates/quanta-gqlang-v2-sema/src/document.rs:2620–2733` (`self.aggregated |= intrinsic.aggregation == Aggregate`, :2730) vs 검증자 `crates/quanta-gqlang-v2-logic/src/source.rs:361–384` (`child_aggregated = aggregated || expression.effects().aggregation == Aggregate`, :371–372).
- **메커니즘**: types가 Call 노드 효과를 `call_effects.compose(arg_effects)`로 합성하고(`types/src/check.rs:1862`) `join`이 aggregation을 max로 전파하므로(`types/src/builtin.rs:90`), 인자에 aggregate를 가진 predicate/builtin call의 **composed** 효과는 `Aggregate`가 된다. sema는 **intrinsic**만 보고 scope를 확장하지만 logic은 composed를 보고 그 call의 모든 인자로 확장 → 하위 predicate call의 polarity가 `Aggregated`로 오판되어 sema가 봉인한 `Positive`와 충돌, `logic.rule_program.polarity_flip` 반환(`source.rs:300–305`) → **컴파일 실패**.
- **재현(실행)**: `cd docs/audits/lane-d-scratch/snapshot/tools/frontend_external_consumer && CARGO_TARGET_DIR=.../lane-d-scratch/target-laneD-repro cargo test --test lane_d_polarity_repro -- --nocapture` (facade `compile_source_document_v3`, 로그 `docs/audits/lane-d-scratch/lane-d-repro.log`).
  - 실제 소스: `let answer = 1; rule u : x if x = 1; rule t : x if x = 1; rule s : x if x or false; ` + 아래 main 규칙 (`t`/`u`는 Int 파라미터, `s`는 Bool 파라미터).
    | 변형 | body | 결과 |
    |---|---|---|
    | A | `y = 1 and s(t(1))` | OK |
    | V2 | `y = 1 and s(is_null(count(u(1))))` | OK |
    | V4 | `y = 1 and t(count(u(1)))` | OK |
    | V6 | `y = 1 and is_null({a: t(1), b: 1})` | OK |
    | **B** | `y = 1 and s(t(count(u(1))))` | **ERR `logic.rule_program.polarity_flip [113..127]`** |
    | **V5** | `y = 1 and is_null({a: t(1), b: count(u(1))})` | **ERR `logic.rule_program.polarity_flip [123..127]`** |
  - 오류 code가 `logic.*` = sema/types는 봉인 성공, logic ingress만 거절. 불일치는 `!=` 비교라 **거짓 수용은 없고 오직 거짓 거절**만 발생(fail-closed).
- **트리거**: aggregate를 인자 subtree에 가진 predicate call 위에 또 다른 call 계층이 있을 때(B: predicate 계층, V5: scalar builtin+record). 계층이 없으면(V2/V4) 두 유도가 일치.
- **최소 수정(권장)**: sema가 call 노드별 **intrinsic aggregation**(`document.rs:2708–2730`에서 이미 계산)을 봉인된 사실로 싣고, logic이 composed `effects().aggregation` 대신 그 사실로 scope를 확장. owner: sema(+logic). compatibility: private field + additive accessor.
- **부분 수정(검증됨, 불충분)**: Predicate target만 scope 확장에서 제외하는 1줄 패치는 B만 고치고 V5는 남긴다(snapshot에서 패치·실행으로 확인).
- **oracle**: 위 행렬 rail(B/V5 컴파일 성공 단언) + fixture matrix에서 유도 polarity와 `call.polarity()` 전수 일치 단언(현재 없음).

### IR-02 · P2 · `i128::MIN` 괄호 철자가 거짓 거절 `[재확인]` (Lane C, C-01)

- **위치**: `crates/quanta-gqlang-v2-types/src/check.rs:1780–1785` (`min_case`는 unary operand가 **직접** `Literal(Int(text))`일 때만 성립), `:1777–1780` (`Paren`은 `type_expr(inner, false)`로 재귀 — deferred 소실), `:2078–2081` (일반 경로는 `deferred && text == "<2^127>"` 요구).
- **영향**: `-(170141183460469231731687303715884105728)`은 수용되지만 `-(  (…))`처럼 괄호가 끼면 `deferred=false`가 되어 `IntegerOutOfRange` **거짓 거절**(무괄호/괄호 비대칭).
- **최소 수정**: Paren 통과 시 deferred 전파 또는 operand에서 Paren strip 후 판정. owner: types. **oracle**: 괄호/무괄호 두 철자가 동일 `TypedLiteralFactV2`를 갖는 독립 회귀.
- **반증 시도**: 두 경로(1777–1785 vs 2078–2081)를 직접 대조했고, deferred 플래그가 괄호에서 false로 떨어지는 것을 확인 → 유지.

### IR-03 · P2 · catalog edge 정렬 키 불완전 → 입력 순서 의존 오탐 `[재확인]` (Lane C, C-02)

- **위치**: `crates/quanta-gqlang-v2-sema/src/document.rs:690–699` — 5-튜플(source, target, kind, family, location)을 수집하지만 `sort_by` 키는 `(&left.0, &left.1)` = **(source, target) 2-튜플**뿐이고, 바로 아래 `semantic_edges != validated_edges`는 5-튜플 **완전 비교**.
- **영향**: 같은 (source, target)에 서로 다른 5-튜플 edge가 2건 이상이면 정렬이 stable이라 수집 순서가 결과 순서를 좌우 → modules-validated 그래프와 순서가 어긋나면 `document_validation_failed`("semantic catalog references differ") **오탐 거절**.
- **최소 수정**: 정렬 키를 전체 5-튜플로(총순서 확보). owner: sema. **oracle**: 동일 (source,target) 2-edge를 서로 반전된 입력 순서로 구성한 fixture가 양쪽 모두 수용.

### IR-04 · P3 · `<>` 어휘가 파서에서 수용되고 types에서 사망 + 도달 불가 overload `[재확인]` (Lane A, A2-1)

- **위치**: `surface/src/lexer.rs:368` (`<>` 토큰화), `surface/src/expr.rs:195` (`is_compare_op`에 `<>` 포함), `types/src/op.rs:75` (registry spelling 목록에 `<>` **없음**), `types/src/check.rs:2304–2312` (`binary_overloads` 비면 `UnknownOperator`).
- **영향**: `a <> b`는 문법적으로 수용되어 **types 단계에서** 죽는다(명시적 v2-reject 정책 경로가 아님). 추가로 `%`(id 105)/`++`(id 118) overload가 registry에 pin되어 있으나(`op.rs:24,28`) lexer에 해당 규칙이 없어 **도달 불가 코드 + 이를 정상으로 고정한 테스트**.
- **최소 수정**: 연산자 spelling 단일 테이블을 lexer/expr/registry가 공유하거나 `<>`→`!=` HIR lowering + 사문 overload 삭제. owner: types+surface. **oracle**: `<>`의 parser-단계 진단 + `%`/`++` 부재를 강제하는 동기화 테스트.

---

## 2. 실제 영향이 있는 경계·무결성 위험

### IR-05 · P2 · execution digest의 독립 oracle 부재 + mutation rail 필드 공백 `[lane 보고]` (Lane E, E2-01)

- **요지**: `tools/` 어디에도 전체 스트림 digest를 **독립 재인코딩**해 대조하는 oracle이 없다(TSV vector도 동일 encoder가 생산). 필드 mutation rail이 `item_id`, family tag, root stage key, children 순서, top-level item 순서를 비껴가므로 encoder와 TSV가 함께 drift하면 모든 gate를 통과한다.
- **영향**: 공개 bit-level 계약(`ExecutionDigestV3`, schema v5)의 무결성 증명 공백. owner: pipeline tooling. **oracle**: 두 번째 encoder 또는 해당 필드별 mutation rail 추가.

### IR-06 · P2–P3 · pipeline 진단 code 이중 authority + 브리지 oracle 비대칭 `[lane 보고]` (Lane A A3-1/A3-2 + Lane E E3-1)

- **요지**: pipeline이 상류 4개 crate의 진단 code를 독립 리터럴로 재열거(`pipeline/src/diagnostic.rs:201,255,327–345`)하고 공개 재수출한다. logic 브리지만 owner `.code()`와 교차 단언하고 **algebra/planning 브리지는 리터럴만 단언** → owner rename 시 경로별로 다른 code가 조용히 노출될 수 있다. 같은 파일에서 `no_plan_code(CatalogDeclaration)`은 kind를 버린 coarse 문자열(:345)을, `planning_code()`는 kind별 6개(:327–333)를 반환해 동일 사실에 두 답이 존재.
- **영향**: 진단 identity/drift. producer 경로는 kind 분기를 타 coarse에 도달하지 않으므로 **현재 오동작은 아님**(public helper 계약 모순 + 감시 부재). 최소 수정: logic 선례대로 algebra/planning 브리지에 owner `.code()` 교차 단언 추가, `no_plan_code` 시그니처 축소. owner: pipeline(+algebra/planning).

### IR-07 · P3 · "standard" 예산 프로파일 이중 분기 `[재확인]` (Lane F, F-1)

- **위치**: `surface/src/budget.rs:94–104` (`ParseBudgetV2::standard_v2`: tokens 65,536 / expr_nodes 131,072) vs `sema/src/authority.rs:661–686` (`FrontendBudgetV2::standard_v2`: tokens 131,072 / expr_nodes 262,144).
- **영향**: pipeline은 sema 예산에서 parse 예산을 파생해(`compile.rs:792–794`) front door는 131,072로 일관적이지만, crate 공개 편의 진입점(`surface::parse_expr`/`parse_predicate`)은 자체 65,536을 쓴다 → **같은 문서가 경로에 따라 수용/거절**. 양쪽 fail-closed이므로 오작용은 아니나 동기화 rail 부재. owner: surface(+sema).

### IR-08 · P3 · stdio transport의 secondary span 드롭 + authority code 평탄화 `[재확인]` (Lane F, F-2)

- **위치**: `crates/quanta-gqlang-v2-diagnostics-stdio/src/server.rs:22–31` (`WireDiagnosticV2`에 secondary 필드 없음), `handlers/v3_diagnostics.rs:71–85` (`From<&FrontendDiagnosticV2>`가 primary만 전송, `secondary()` 누락), `:52–55` (`FrontendAuthorityErrorV2`를 `code + message` 문자열로 합쳐 generic `"authority_error"`로 평탄화).
- **영향**: 모호 참조 등 secondary 위치가 필요한 진단이 stdio 소비자에게 **정보 손실**된 형태로 도달; authority 원 code는 message 안에만 존재. owner: diagnostics-stdio. 최소 수정: wire에 secondary 배열(버전) + authority code 별도 필드.

### IR-09 · P3 · surface 키워드/escape 어휘 이중 구현 `[lane 보고]` (Lane B, B-F1/B-F3)

- **요지**: `query.rs:108–133 is_reserved`, `family.rs:46–66 is_family_keyword`, `parser.rs:352–357 is_v2_root_keyword`가 서로를 참조하지 않고 병렬 유지(신규 키워드 누락 시 bare-alias 포획 규칙이 조용히 달라짐). escape도 `lexer.rs:198`(scan 허용목록) vs `:287–309`(decode 매핑)로 이중이며, scan에만 추가하면 해당 escape 내용이 **조용히 소실**(D10 금지 형태, 현재 drift 없음).
- **영향**: 잠재적 silent misparse/loss. owner: surface. 최소 수정: 단일 const 테이블 공유 + 동기화 rail.

### IR-10 · P3 · sema `test` feature가 public forge seam을 노출 `[lane 보고]` (Lane A, A3-4)

- **요지**: `quanta-gqlang-v2-sema`는 `publish=false`가 아니고 `test = []` feature를 공개한다. 이 feature에서 `pub` forging API(`document.rs:457 forged_membership`, `rule.rs:159 forged_with_calls`, `pub mod test_support`)가 열려 "never part of a normal build" 서술과 어긋난다. workspace 내부는 policy로 차단(`policy.toml:78–81`)되고 facade 경유로도 도달 불가하지만, **feature 활성화를 막는 gate도, seam 인벤토리 감시 rail도 없다**(semgrep gate는 `test_support` 모듈 가드 정규식만 검사).
- **영향**: 미래 변경/외부 직접 의존 시 봉인 해제 seam. owner: sema(+tools/lint). 최소 수정: seam을 `pub(crate)`+`test_support` 경유로 축소하거나 `#[cfg(any(test, feature="test"))] + pub` 조합 인벤토리를 gate로 고정.

---

## 3. 실제 문제 아님으로 제외 (요약)

- **PROOF_GAP 단독 / 방어심도**: algebra 생성자 출력 일치 검사 폭(D-02), logic membership anchor 조건부(D-04), CHECK/COMPARE 다중도 authority 부재(D-05), `assign_strata` 수렴 guard, planning `RuleProgramPolicyV2::new` caller 요약값(D-08), pipeline `invalid_closed_builtin/operator` sentinel, E3-1/E3-3~E3-6의 doc/drift 항목.
- **DOCUMENTATION_DRIFT**: facade 노출 문구 모순(A1-2), `%` precedence(A2-6), capability `RecursiveClosure` 의미 혼용(D-07), `PipelineStageV2`/`ExecutionFamilyV3::RuleProgram` 미생성 어휘(E1-01/E3-3).
- **OPTIONAL_REFACTOR / 비용**: algebra ownership walk 무예산·2차(D-06), planning `children()` Vec 재할당(D-09), `format_source_document_v2` 이중 파싱(E1-03), `%`/`++` 사문 정리(IR-04에 병합), B-F2 표현력 제한, B-F4 raw HIR pub 필드(계약 문서화됨), hir/surface depth·예산 정합 항목.
- **반증 완료**: parse 재귀 stack overflow, off-by-one, HashMap 순서, float 비결정, 골든 벡터 provenance, gate 공동화, `no_plan_code` production 도달 등은 각 lane 원문의 "반증된 후보" 절 참조.
- 원문: `docs/audits/lane-A-20260927/lane-A.md`, `docs/audits/lane-b-snapshot-20260927-010019/lane-B.md`, `docs/audits/lane-C.md`, `docs/audits/lane-D-20260927/lane-D.md`, `docs/audits/lane-e/lane-E.md`, `docs/bubgash/sep-27-ir/findings-6.md`.

## 4. 판정

- **확정 결함**: IR-01(P2, 실행 재현), IR-02(P2), IR-03(P2), IR-04(P3) — 4건. 모두 거절/수용 오류이며 fail-closed 방향이라 거짓 수용은 없다.
- **실제 영향 있는 위험**: IR-05~IR-10 (P2 1건, P3 5건). 현재 오동작 관측 없음(IR-07은 경로별 수용 불일치가 실제 차이).
- **미실행/한계**: `just ci`, `just qualify-local`, 권장 fix 구현·검증, runtime/peak-memory 측정. IR-05/IR-06/IR-09/IR-10은 해당 lane 보고 relay이며 본 문서에서 재실행하지 않았다. IR-01만 실행 재현이고, IR-02/IR-03/IR-04/IR-07/IR-08은 본 문서에서 정적으로 재확인했다.
