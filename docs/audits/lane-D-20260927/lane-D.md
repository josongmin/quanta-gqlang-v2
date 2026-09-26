# Lane D — Algebra / Logic / Planning 감사 보고서

대상: `/Users/songmin/Documents/code-new/quanta-gqlang-v2`
범위: `crates/quanta-gqlang-v2-algebra`, `-logic`, `-planning` (의미 보존, owner 경계, SSOT 중복, boundedness)
비변경 원칙 준수: 구현 소스 수정 0건. 산출물은 `docs/audits/` 하위에만 작성.

## 0. 수행 방식과 한계

- 계획은 "Lane D 2-pass 병렬 subagent"였으나 이 세션에서 백그라운드 subagent 3회가 모두 종료 메시지 없이 실패했다(부분 산출물 0). 이후 조정자(본 보고서 작성자)가 **직접 정적 감사 + owner rail 실행 + pass-2 반증/재현**으로 대체 수행했다.
- pass-2에서 D-01의 도달성을 격리 snapshot(HEAD `git archive`, 원 트리 불변)에서 실제로 재현했고, 후보 fix 2종 중 부분 패치를 적용·실행해 한계까지 확인했다. 재현물은 snapshot 안에만 존재한다.
- 재현·패치 실험은 모두 `docs/audits/lane-d-scratch/snapshot/`에서 수행했고, 실험이 끝난 뒤 snapshot의 `logic/src/source.rs`는 HEAD 내용과 hash가 동일함을 확인했다(패치 잔존 없음).
- 그 외 findings는 정적 입증이며 각 항목에 "재현 미실행"을 명시했다. 교차 lane 검증(Lane A/C/E/F 대조)은 §6 handoff로 남긴다.

## 1. 소스 경계

| 항목 | 값 |
|---|---|
| HEAD | `eaeb68ea2711d411616511713fb016d828cd80be` (branch `main`) |
| Lane D crate 상태 | algebra / logic / planning **clean at HEAD** (dirty 없음) |
| dirty 경계(타 lane) | `cli/src/main.rs`, `cli/tests/v3_cli_fmt_idempotence.rs`, `pipeline/src/compile.rs`, `surface/{format,lib}.rs`, `surface/tests/*`, `docs/language/*`, `docs/audits/`(untracked) |
| hash manifest | `docs/audits/lane-d-scratch/manifest-lane-d.sha256` — Lane D 26개 소스 SHA-256 |
| 감사 전/후 drift | `shasum -a 256` 재계산 후 `diff` → **NO_DRIFT** (감사 중 Lane D 소스 불변) |
| symlink | Lane D 경로에 symlink 없음 |
| 실행 로그 | `docs/audits/lane-d-scratch/lane-d-tests.log` |

경계에 대한 정확한 진술:

- 원 checkout을 수정·stash·reset·clean하지 않았다. Lane D 스코프가 HEAD에서 clean이므로 **원본 트리 + 전후 hash 비교**로 고정했다(전체 복사 snapshot은 사용하지 않음).
- `crates/quanta-gqlang-v2-pipeline/src/compile.rs`는 다른 lane의 dirty 파일이다. 이 보고서에서 인용한 compile.rs line은 **working tree 기준**이며, `git diff -U0`로 확인한 dirty hunk는 line 62 부근 import 2줄 치환뿐이고(line 수 불변) 인용 구간(1148, 1330–1450, 1490–1600, 1614–1624)에는 hunk가 없다. 따라서 인용 구간 내용은 HEAD와 동일하다.
- 파일 snapshot hash를 frontend authority/provenance로 사용하지 않았다. `ExecutionDigestV3`와는 무관하다.

조사 파일: Lane D 3 crate 11,141줄 전부(node.rs 615, normalize.rs 1607, diagnostic.rs 100, lib.rs 19 / source.rs 530, recursion.rs 1095, graph.rs 752, witness.rs 496, diagnostic.rs 361, aggregate.rs 63, lib.rs 54 / plan.rs 1864, composition.rs 1206, diagnostic.rs 577, policy.rs 274, cost.rs 237, result_kind.rs 258, check.rs 249, effect.rs 187, item.rs 184, presentation.rs 143, capability.rs 134, explain.rs 63, lib.rs 73).
경계 교차 확인: sema `rule.rs`(전문), `document.rs`(1709–1735, 2608–2734, 3150–3180, 3900–4030, 5870–5940, 6040–6090, 6540–6600), `item.rs`(200–360, 463–525), types `builtin.rs`(15–110, 240–420), `check.rs`(1587–1700, 1850–1870, 2353–2536), `solver.rs`(150–215, 316–340), surface `unit.rs`/`parser.rs`/`family.rs`(rule·FIND 문법), pipeline `compile.rs`(1148–1630), `execution.rs`(560–610), 각 crate `Cargo.toml`.
미조사: Lane D integration test 본문 전체(실행만 함, mutant rail 일부·composition transplant rail만 정독), `plan.rs` test 1501–1864 및 `composition.rs` test 861–1206의 나머지, diagnostics-stdio/CLI/conformance(→ Lane E/F), modules/lockfile(→ Lane C).

## 2. Owner / ingress / invariant 요약

| IR · 계약 | owner (생성) | ingress | 검증 시점 · 불변식 | 실패 동작 | 소비자 |
|---|---|---|---|---|---|
| `CheckedAlgebraNodeV2`, `LoweredRelationItemV2` | algebra `normalize.rs` assemble → `node.rs:179` `new` | `AlgebraSessionV2::new(&SemanticDocumentV2)` (normalize.rs:46) | 생성자에서 item/source/stage-key 일치 + family별 output 일치(node.rs:482–599); foreign doc 거부(normalize.rs:73–79,129–149,353–361) | typed `AlgebraDiagnosticV2` | planning `walk`, pipeline `project_relation` |
| `CorrelationSetV2` | algebra `node.rs:56` `try_new` | `TypedCorrelationEdgeV2` (sema 봉인) | reference 중복 금지; 호출측이 source 소속·expression 존재 확인(normalize.rs:788–798) | `DuplicateCorrelationReference` / `ForeignSemanticFact` | planning policy, projection |
| `RuleProgramContractV2` | logic `source.rs:72` `from_document` | 문서 item 집합 자체 열거(caller Vec 없음) | 멤버/콜/타깃/polarity/registry/span 재검증 + **1회** stratification | `LogicDiagnosticV1::RuleProgramInvalid*`, `RuleGraphBudgetExceeded` | planning, pipeline `compile_rule_program` |
| `StratificationProofV1` | logic `recursion.rs:323` `stratify` | `RecursiveProgramV1`(공개 구성 가능) | Tarjan SCC + canonical offending edge + strata fixpoint | `NegationCycle` / `RecursiveAggregate` / budget | logic source(1회), planning `rule_facts`, pipeline DTO |
| `GraphQueryContractV2` | logic `graph.rs:263` `from_resolved` | `ResolvedGraphQueryV2`(sema 봉인) | endpoint 필수/금지 역할, depth 필수·양수, witness bound ≤ sealed budget | `GraphEndpoint*`, `UnboundedWitness` | planning `graph_facts`, pipeline projection |
| `WitnessContractV2` | logic `witness.rs:125/161` (`pub(crate)`) | `WitnessBindingV2`(crate-private) | cap/path bound 존재·비퇴화·예산 이하, 결정적 거절 순서 | `UnboundedWitness{reason}` | graph contract, planning retention |
| `SemanticPlanInputV2` | planning `item.rs:31` | 봉인 carrier만(7 arm) | arm별 `validate_identity` (plan.rs:351) | `InputDocumentMismatch{Absent,WrongKind,IdentityMismatch}` | planning session |
| `SemanticPlanV2` (9 field) | planning `plan.rs:145` `from_parts` (**private**) | `PlanningSessionV2::plan_outcome` | identity → non-plan → budgeted walk → classify | `PlanningDiagnosticV1` | pipeline, facade 재수출 |
| `PlanPolicyV2` / `CompositionPolicyV2` | planning `policy.rs:236` / `composition.rs:493` | composition policy는 `CheckedCompositionPlanInputV2::policy()`만 | family narrowing, body family 일치 | `InvalidComposition{BodyKindMismatch}` | pipeline projection |
| `SemanticCostV2` | planning `cost.rs:133` | 4개 고정 field, `FIELD_NAMES` | schema drift rail | — | CLI/LSP |
| 예산 회계 | sema `DownstreamBudgetReceiptV2` | algebra `LoweringBudgetV2`(normalize.rs:267), planning `ChargeV2`(plan.rs:657), logic `RuleGraphChargeV1`(recursion.rs:424) | charge-before-work, exact limit 성공 / +1 거절 | `TraversalBudgetExceeded` / `SemanticStepBudgetExceeded` / `RuleGraphBudgetExceeded` | 각 단계 |

구조 판정(요약): **정당한 단계별 IR** (HIR→typed semantic→algebra→plan→execution DTO), **정당한 projection** (algebra→plan→V3 DTO, logic 계약→planning retention), **정당한 경계 재검증** (algebra의 foreign/nested 소속 검사, logic의 규칙 재검증, planning의 identity 재봉인). **의미 결정의 중복**은 §3 D-01 1건이며 재현으로 **CONFIRMED_DEFECT**(거짓 거절, fail-closed 방향)로 확정됐다. **경쟁 authority**는 관측되지 않았다.

## 3. Findings

### D-01 · P2 · CONFIRMED_DEFECT (재현 완료: sema-legal 문서가 logic에서 거짓 거절)

- **대상**: rule body의 call polarity/aggregation 전파 판정이 두 곳에서 독립 구현되어 있고 규칙이 다르다.
  - 생산자(sema): `crates/quanta-gqlang-v2-sema/src/document.rs:2620–2733` `RuleDependencyCollectorV2::visit_value` — `self.aggregated |= intrinsic.aggregation == AggregationClassV1::Aggregate`(:2730), 즉 **호출된 함수의 고유(intrinsic) 효과**만 사용하고, 그 호출의 인자 walk 동안만 전파한 뒤 복원(:2732–2733).
  - 검증자(logic): `crates/quanta-gqlang-v2-logic/src/source.rs:361–384` `derive_body_calls` — `let child_aggregated = aggregated || expression.effects().aggregation == Aggregate;`(:371–372), 즉 봉인된 **composed 효과**를 사용해 그 Call의 **모든** 인자에 전파.
- **왜 다른가(정적 입증)**: types가 Call 노드 효과를 `call_effects.compose(arg_effects)`로 합성한다(`crates/quanta-gqlang-v2-types/src/check.rs:1862`), `compose`는 `join` fold이고(`types/src/builtin.rs:95–98`) `join`은 `aggregation.max(other.aggregation)`(`builtin.rs:90`), `Scalar < Aggregate`(`builtin.rs:21–24`)이다. 반면 source predicate의 intrinsic 효과는 `AggregationClassV1::Scalar`다(`types/src/solver.rs:186–200`, `builtin.rs:66–74`). 따라서 **인자에 aggregate call을 가진 source-predicate call 노드**는 `effects().aggregation == Aggregate`가 되고, logic은 그 call의 모든 값 인자를 aggregated로 표시한다.
- **reachable caller → owner**: surface/HIR → sema `resolve_checked_document_v2` → pipeline `compile_rule_program`(`pipeline/src/compile.rs:1500`) → `RuleProgramContractV2::from_document` → `validate_rule`(source.rs:212) → `derive_body_calls`; 불일치 시 `PolarityFlip`(:300–305) 반환 → **rule program ingress 거절**.
- **기대/실제**: 기대 = logic의 재유도가 sema가 봉인한 `call.polarity()`와 항상 일치(문서화: 두 walk가 mirror). 실제 = aggregate call이 predicate/builtin call의 인자 subtree 안에 있고 그 위에 또 다른 call이 있으면, logic이 aggregate scope를 그 call의 **모든** 인자로 확장해 하위 predicate call의 polarity를 `Aggregated`로 오판 → sema가 봉인한 `Positive`와 충돌해 `logic.rule_program.polarity_flip`으로 **거절**. 불일치는 양 방향 모두 `!=` 비교에 걸리므로 **거짓 수용은 발생하지 않고 오직 거짓 거절만** 발생한다(fail-closed).
- **재현(실행 완료)**: 격리 snapshot(HEAD export)의 standalone facade 소비자에 테스트를 작성하고 실행했다.
  - 명령: `cd docs/audits/lane-d-scratch/snapshot/tools/frontend_external_consumer && CARGO_TARGET_DIR=.../lane-d-scratch/target-laneD-repro cargo test --test lane_d_polarity_repro -- --nocapture`
  - 로그: `docs/audits/lane-d-scratch/lane-d-repro.log`, 테스트 소스: `docs/audits/lane-d-scratch/snapshot/tools/frontend_external_consumer/tests/lane_d_polarity_repro.rs`
  - 결과 행렬(모두 `compile_source_document_v3` 경유):

    | 변형 | rule body | 결과 |
    |---|---|---|
    | A_plain | `y = 1 and s(t(1))` | **OK** |
    | V2_builtin_layer | `y = 1 and s(is_null(count(u(1))))` | **OK** |
    | V4_single_predicate_layer | `y = 1 and t(count(u(1)))` | **OK** |
    | V6_record_control | `y = 1 and is_null({a: t(1), b: 1})` | **OK** |
    | **B_agg_nested** | `y = 1 and s(t(count(u(1))))` | **ERR** `logic.rule_program.polarity_flip [113..127]` |
    | **V5_record_under_scalar_builtin** | `y = 1 and is_null({a: t(1), b: count(u(1))})` | **ERR** `logic.rule_program.polarity_flip [123..127]` |

  - 오류 code가 `logic.*`이므로 sema/types는 문서를 봉인했고(그 전 단계 실패 없음), **오직 logic의 규칙 계약 ingress가 거절**했다. 오류 span은 각각 중간 predicate call `t(count(u(1)))`(113..127)과 `t(1)`(123..127)을 가리킨다.
- **trigger 조건(재현으로 확정)**: (1) 해당 predicate call(`t`)의 인자 subtree 안에 aggregate call이 있고, (2) 그 predicate call 위에 또 다른 call(`s` 또는 scalar builtin `is_null`, 심지어 record literal)이 있어 logic이 scope를 확장하는 위치에 있을 때. predicate 계층이 없으면(V2/V4) 두 유도가 일치한다.
- **중복 후보 비교**: 두 구현의 입력(sema: HIR bool/value tree + intrinsic registry / logic: 봉인 typed expression registry), 판단 규칙(intrinsic-only vs composed), 호출 경로(문서 sealing vs 규칙 계약 ingress), 동기화 방식(문서화된 "mirror" 주석뿐, 공유 코드·공유 테스트 없음), 관측된 차이(B/V5 거절). 소비자 증상(`PolarityFlip`)은 별도 root finding이 아니라 이 root의 증상이므로 중복 등록하지 않는다.
- **최소 root fix(권장)**: aggregate scope의 authority를 **intrinsic**으로 되돌린다. sema는 이미 call 노드마다 `intrinsic`을 계산하므로(`document.rs:2708–2730`) 그 값을 봉인된 call fact(예: `ResolvedRuleCallV2` 또는 typed call expression)에 실어 보내고, logic의 `derive_body_calls`는 composed `effects().aggregation` 대신 **그 봉인된 intrinsic 사실**로 scope를 확장한다. owning crate: **sema(사실 추가) + logic(소비)**. compatibility: 봉인 carrier에 private field + additive accessor — 공개 API 파괴 없음.
- **부분 fix(검증됨, 불충분)**: `source.rs:371–372`에서 Predicate target만 scope 확장에서 제외하는 1줄 패치는 B를 고치지만 V5는 고치지 못한다. 격리 snapshot에서 실제로 패치·실행해 확인했다(B → OK, V5 → 여전히 `polarity_flip`). builtin/host call의 intrinsic을 logic이 알 수 없기 때문이다. 따라서 이 패치는 stopgap으로만 유효하다.
- **regression oracle**: 위 행렬을 그대로 rail로 옮긴다 — B/V5는 컴파일 성공을, A/V2/V4/V6은 계속 성공을 단언하고, 별도 rail로 fixture matrix에서 `derive_body_calls`의 유도 polarity가 `call.polarity()`와 전부 일치함을 단언(현재 이런 대조 rail이 없다).
- **반증 시도**: (a) Bool `Not` 처리: sema는 bool tree에서 토글(`document.rs:2610–2613`)하고 typed tree에도 `TypedExprShapeV2::Not`이 materialize된다(`document.rs:3158–3173`) → 일치. (b) aggregated∧negated는 sema가 선거절(`document.rs:2640–2645`)하므로 logic의 우선순위 분기는 도달 불가. (c) `validate_rule`은 retained call의 target/polarity를 expression fact와 대조하므로 target transplant는 별도로 막힌다. (d) 값 문법·builtin 표를 확인해 다인자 call은 문자열 builtin뿐임을 확인했고, 그래서 처음에는 도달 불가로 의심했으나 재현이 성립했다(record/중첩 predicate 인자가 경로). (e) fix 후보를 실제로 패치해 부분 패치의 한계까지 확인했다. 반증되지 않고 재현되었으므로 CONFIRMED_DEFECT로 승격한다.

### D-02 · P3 · PROOF_GAP (fail-closed 계약이 문서화된 범위보다 좁음, production trigger 미발견)

- **문서화된 불변식**: `algebra/src/node.rs:170–178` — "the operator must agree with the exact sealed output (schema, provenance, cardinality, partition, ordering, and family-specific identity …)".
- **실제 검사**:
  - `Project`(node.rs:497): `output.provenance() == input.output().provenance()` **만** 비교. `expressions`는 `..`로 무시되어 **column 수·column origin↔expression key 정렬이 검증되지 않는다**(Aggregate 분기는 :579–586에서 정확히 그 검사를 한다).
  - `Join`(node.rs:498–520): mode/predicate/column 연결/provenance는 비교하지만 **cardinality·row는 비교하지 않는다**.
  - `Aggregate`(node.rs:562–587): partition/cardinality/ordering/effects/columns↔expressions는 비교하지만 **row는 비교하지 않는다**.
  - `SemiJoin`/`AntiJoin`(node.rs:595–597): left frame 보존만 검사하고 `correlations`와 right subtree의 정합은 검사하지 않는다(문서 소속·중복은 normalize.rs:788–798에서 별도 확인).
- **trigger/input**: 봉인 문서가 내부 불일치(예: Project stage의 output columns가 expressions와 어긋남)를 가질 때. 현재 이 형태는 sema 정상 경로에서 만들어지지 않는다 — Project output은 같은 loop에서 `columns`와 `keys`를 함께 push하므로(`sema/src/document.rs:3903–3947`) 항상 정렬된다. 즉 **sema test seam(`forged_*_document_v2`)으로만 주입 가능**하다.
- **expected/actual**: 기대 = `InvalidStageContract` 거절. 실제 = Project column/origin 불일치는 통과해 planning/pipeline에 모순된 node가 전달된다.
- **재현**: 미실행(해당 forged fixture 없음). 정적 입증만. 기존 mutant rail(`algebra/tests/v3_algebra_mutants.rs`)은 mixed scalar/aggregate, wrong nested root, capture mutation만 덮는다.
- **최소 root fix**: Project 분기에 column 수 + `origin == Expression(expression.key())` 정렬 검사를 추가(Aggregate와 동일 패턴), Join/Aggregate에 sema가 정의한 범위의 cardinality/row 비교 추가. owning crate: **algebra**. compatibility: `pub(crate)` 생성자 — 공개 API 영향 없음.
- **oracle**: `sema::test_support`에 `forged_project_column_mismatch_document_v2`(및 Join cardinality 변조)를 추가하고 `lower_relation_item`이 `InvalidStageContract`로 거절하는 rail.
- **반증 시도**: 정상 경로 도달 불가를 `document.rs:3903–3947`로 확인했고, 그래서 P3 PROOF_GAP으로 분류했다(현재 오동작 아님). "검사 폭이 문서 주장보다 좁다"는 사실 자체는 남는다.

### D-03 · P3 · STRUCTURAL_RISK (공개 API 우회 가능, production 미도달)

- **대상**: `planning/src/composition.rs:212–218` `nested_root_matches` — relational composition body는 `body.root().item_id() == owner_item_id && body.root().output() == sealed.output()`만 검사한다(:612–620에서 호출).
- **문서화된 불변식**: composition.rs:609–611 — "the root must be the canonical lowering of the SEALED body relation, not an arbitrary subtree carrying the sealed identity". 실제 검사는 identity(owner item) + output 계약뿐이라 **동일 output을 가진 다른 nested relation root**(예: 같은 composition의 producer root)로 대체 가능하다.
- **reachable 경로**: 외부 소비자가 `algebra::lower_nested_relation_v2(composition_item, producer_root)`(공개) + `CompositionRelationalBodyPlanInputV2::try_new(body_item_id, producer_node)`(공개) + `CheckedCompositionPlanInputV2::new(composition, body)`(공개)를 조합. producer root의 item_id도 owner composition item이므로 identity 검사를 통과한다. pipeline은 올바른 root(`body.root()`)를 넘기므로(`compile.rs:1547–1559`) **production trigger 없음**.
- **expected/actual**: 기대 = `BodyIdentityMismatch`. 실제 = 동일 output이면 수용되어 다른 query를 기술하는 plan/policy가 만들어진다.
- **재현**: 미실행. 기존 rail `composition_body_transplants_fail_closed_at_the_public_constructor`(`planning/tests/v3_planning_hellgate.rs:1119`)는 (a) family 불일치, (b) 다른 item_id의 standalone graph만 덮고 **동일 output의 relational root 대체는 덮지 않는다**.
- **최소 root fix**: (i) `CheckedCompositionPlanInputV2::new`에 `document: &SemanticDocumentV2`를 추가하고 sealed body root를 재-lowering한 결과와 `==` 비교(가장 정확, 공개 생성자 signature 변경 → contract versioning 필요), 또는 (ii) algebra nested lowering이 sealed relation key를 node에 보존하고 planning이 `sealed.key()`와 비교(필드 추가, 공개 API 영향 없음). owning crate: planning(+algebra 선택). oracle: 동일 output producer root를 body로 넘겨 `BodyIdentityMismatch`를 기대하는 rail 확장.
- **반증 시도**: sema `ResolvedCompositionBodyV2::Query(sealed)`가 body relation 전문을 보유하므로 authority는 존재하나 planning이 사용하지 않는다는 점을 확인했다(그래서 "증거 부족"이 아니라 구조적 위험). production 경로가 올바른 root를 넘긴다는 사실도 확인했다.

### D-04 · P3 · PROOF_GAP (fail-closed가 조건부)

- **대상**: `logic/src/source.rs:96–101` — `document.order()`가 item map에 없는 identity를 지목했을 때 `membership_anchor(&members, document)`가 `None`이면(문서에 rule 멤버도, located item도 없음) 에러 없이 루프를 계속하고, 결과적으로 빈 rule program 계약이 만들어져 `Ok`로 반환된다.
- **문서 주장**: source.rs:97–99 — "the rule program cannot prove that it enumerated every member. Fail closed on the surviving anchor."
- **expected/actual**: 기대 = `RuleProgramInvalid{MissingMember}`; 실제(anchor 부재 시) = 빈 program 성공 → planning은 `NoPlanReasonV2::EmptyRuleProgram`을 낸다(증상). 문서 정합성(order ⊆ items)은 `SemanticDocumentV2` 봉인이 보장하므로 정상 경로 도달 불가, test seam `SemanticDocumentV2::forged_membership`(`sema/document.rs:457–465`, `#[cfg(any(test, feature="test"))]`)로만 만들 수 있다.
- **재현**: 미실행. 정적 입증.
- **최소 root fix**: anchor 유무와 무관하게 거절하고, anchor가 없으면 문서 자체의 start-of-source류 checked span(planning의 `document_anchor`와 동일 방식)을 사용. owning crate: logic. oracle: `forged_membership`으로 (rule 0개 + order에 orphan id) fixture를 만들어 `MissingMember`를 기대.
- **반증 시도**: 이 경로가 정상 문서에서 불가능함을 확인했지만, "fail closed"라는 계약 문장이 조건부라는 사실은 남는다(테스트 전용 위조 가능성을 공개 API 결함으로 승격하지 않고 P3로 유지).

### D-05 · P3 · PROOF_GAP (sealed authority 없는 다중도/순서 결정, handoff → Lane E)

- **대상**: `planning/src/plan.rs:1183–1323`.
  - `graph_facts`: `cardinality: CardinalityV1::Set`(:1195), `check_facts`: `Set`(:1238), `presentation_facts`: `Set`(:1316).
  - `witness_selection`(:1406–1412)은 sealed cap==1 → `CanonicalOne`, 그 외 → `AllWitness`.
  - `MultiplicityOrderContractV2::from_cardinality`(result_kind.rs:87) → Set이면 duplicates 제거·unordered로 **공개 계약에 확정**된다(cost.rs:105–128 `OrderingContractV2`).
- **문제**: FIND/CHECK/EXPLAIN/SHOW/DESCRIBE/SAMPLE/COMPARE/EXPORT 결과의 다중도·순서를 정의하는 sealed fact가 sema에 없다(`ResolvedCheckV2`, `ResolvedPresentationKindV2`는 payload만 보유 — `sema/src/item.rs:463–493`). 즉 "alert는 set이다", "compare는 unordered다"라는 언어 의미가 planning 상수로 **사후 정의**된다. FIND은 witness 계약(dedup identity)이 있어 근거가 있지만(:1193–1195), CHECK/COMPARE의 `Set`/`Unordered`는 근거 fact가 없다.
- **expected/actual**: 기대 = 문서화된 언어 의미 또는 sealed fact. 실제 = planning의 암묵 선택. 잘못된 값이라는 증거는 없고(현재 동작 일관), 의미 정의의 출처가 코드 상수라는 점이 gap이다.
- **재현**: 정적. (`docs/language/*`, `docs/contracts/frontend-execution-v3.md`에서 CHECK/COMPARE 다중도 정의를 찾지 못했다.)
- **최소 root fix**: (i) 언어/계약 문서에 arm별 다중도·순서를 명시하고 rail로 고정하거나, (ii) 언어가 정의한다면 sema가 sealed fact로 방출. owning crate: planning + docs(의미 정의는 Lane C/E와 합의 필요). compatibility: 값을 바꾸는 수정이 아니라 **정의를 고정**하는 수정.
- **반증 시도**: `Item contract matrix`가 planning 소유라고 여러 문서 주석이 주장하므로 의도된 소유일 수 있다. 그러나 "sealed authority 없음"은 사실이며, V3 DTO가 `result`/`semantic_cost`를 투영하지 않으므로(`execution.rs:576–610`은 authority/determinism/evidence/effect/policy_family만) 소비자가 이 결정을 검증할 공개 경로도 없다 → Lane E와 함께 판단할 handoff로 남긴다.

### D-06 · P3 · OPTIONAL_REFACTOR (비용만, 오동작 없음, 측정 없음)

- **대상**: `algebra/src/normalize.rs:124–151` `validate_nested_owner` → `item_owns_nested_relation`(:160–191) → `relation_references_nested`(:195–265).
- **구조적 반복(정적 입증)**: Composition의 경우 ownership 검사가 `composition.bindings()`를 순회하며 각 producer root에 대해 `relation_references_nested`를 수행한다. pipeline은 producer마다 `lower_nested_relation`을 호출하고(`compile.rs:1401–1403`) body에 대해 한 번 더 호출하므로(`compile.rs:1548–1550`), producer 수 P에 대해 최대 P+1회의 ownership walk가 발생하고 각 walk는 최대 O(P·T) relation/expression 방문이다(T = 각 tree 크기) → **O(P²·T)**. 이 walk는 `max_semantic_steps`/`max_expression_nodes`에 **청구되지 않는다**(방문 집합으로 종료만 보장, normalize.rs:202–203, 816).
- **영향**: 정확성 문제 없음(방문 집합으로 유한). 다만 budget refusal이 이 비용을 통제하지 못한다. 실제 runtime/peak-memory 측정은 하지 않았으므로 성능 수치는 주장하지 않는다.
- **최소 root fix**: `AlgebraSessionV2`가 owner별 owned nested-root key 집합을 1회 구축(또는 lazy memo)하여 P+1회 재검증을 O(document) 1회로 축소. owning crate: algebra. oracle: producer 수를 늘린 합성 문서로 방문 횟수를 세는 계측 rail(선택).
- **반증 시도**: 각 호출이 독립 공개 API이므로 매 호출 재검증이 보안상 정당하다는 해석이 가능하다. 다만 같은 session 안에서 동일 문서를 반복 검증하는 것은 중복이며, memo는 그 보안 성질을 유지한다. 취향 문제가 아니라 **비용 구조** 문제로만 남긴다.

### D-07 · P3 · DOCUMENTATION_DRIFT (의미 혼용, 동작 오류 아님)

- **대상**: `planning/src/capability.rs:36–38` — `RecursiveClosure` 문서: "Recursive closure (a document rule program with a recursive member). Derived from the sealed logic proof, never from the algebra walk."
- **실제**: `plan.rs:863–879` `visit_leaf`는 Graph 계약에서 `contract.shape() == Cyclic`이면 `RecursiveClosure`를 삽입하고, CHECK의 nested graph도 동일하다. 그리고 `witness_binding`은 **모든 FIND를 Cyclic으로 bind**한다(`logic/src/graph.rs:220–235`) → 모든 FIND/CHECK(nested) plan이 `RecursiveClosure` capability를 갖는다.
- **동시에**: `ExplainNoteV2::recursive` 문서는 "admitted recursive closure"(`explain.rs:26–27`)인데 `graph_facts`/`check_facts`는 `recursive: cyclic`(`plan.rs:1207, 1252`)을 넣어 모든 FIND이 `recursive=true`가 된다.
- **영향**: 같은 keyword가 fixpoint recursion과 bounded witness enumeration을 함께 뜻한다. 구분 정보는 `semantic_cost.recursion_risk_class`(`BoundedFixpoint` vs `BoundDependent`, plan.rs:1201–1205, 1350–1354)에만 있다. 능력 keyword로 host negotiation을 하는 소비자는 구분할 수 없다. 값 자체가 틀렸다는 증거는 없다.
- **최소 root fix**: variant 문서를 실제 의미로 좁히고, 필요하면 `BoundedWitnessTraversal` 같은 별도 capability를 추가(공개 enum variant 추가 → host contract versioning 검토), `ExplainNoteV2::recursive` 문서를 "cyclic/bounded witness를 포함"으로 정정. owning crate: planning + docs. oracle: FIND plan의 capability 집합과 `recursion_risk_class`를 함께 단언하는 rail.
- **반증 시도**: FIND traversal이 실제로 cycle-safe closure를 요구하므로 "capability가 필요하다"는 해석도 가능하다. 다만 rule fixpoint와 동일 keyword를 쓰는 이상 소비자 구분이 불가능하다는 사실은 남는다.

### D-08 · P3 · OPTIONAL_REFACTOR (authority 우회 아님)

- **대상**: `planning/src/policy.rs:199–213` `RuleProgramPolicyV2::new(contract, stratum_count, recursive_members)`.
- **문제**: `contract.stratification()`이 `stratum_count`/`recursive`를 이미 보유하는데도 caller가 요약값을 넘긴다. production은 proof에서 읽어 넘기므로 drift 없음(`plan.rs:1333–1361`). 외부 caller가 불일치 값을 넣어 `PlanPolicyV2::Rule`을 만들 수 있지만 `SemanticPlanV2::from_parts`가 private(`plan.rs:145`)이므로 plan authority가 되지 못한다.
- **최소 root fix**: `new(contract)`로 좁혀 proof에서 직접 읽거나, 새 생성자를 추가하고 기존 것을 deprecated. owning crate: planning. compatibility: 공개 API 축소 → minor version.
- **반증 시도**: 의도적 "carrier는 structural only" 설계일 수 있으나, 같은 crate 안에서 proof를 재읽기만 하면 되는 정보를 caller에게 요구할 이유는 없다.

### D-09 · P3 · OPTIONAL_REFACTOR (할당, 측정 없음)

- **대상**: `planning/src/plan.rs:803` `let children = node.children();` — `CheckedAlgebraNodeV2::children()`(`algebra/src/node.rs:243–255`)은 매 호출 `Vec`을 새로 만든다. 방문 node마다 1회, Aggregate에서는 `exit_class`(:905–909)가 `node.children()`을 **다시** 호출해 1회 추가 할당. reachable 경로: 모든 relational item plan. 반복 횟수 = 방문 node 수(+Aggregate 수).
- **영향**: 구조적으로 반복됨은 입증되지만 runtime 측정은 없다. 최소 fix: `children()`을 `impl Iterator`/`SmallVec`로 바꾸거나 `exit_class`가 이미 받은 자식 정보를 재사용. owning crate: algebra(접근자) + planning(호출).
- **반증 시도**: 문서 크기 대비 무시할 수준일 수 있어 P3·OPTIONAL_REFACTOR로만 둔다.

## 4. 의심했지만 반증된 후보

| 후보 | 반증 근거 |
|---|---|
| Project의 `output.provenance == input.provenance` 검사가 과도(부분 projection에서 실패할 것) | sema가 Project output provenance를 **입력 상속**으로 정의(`sema/src/document.rs:3972–3988`), ordering만 비우고 cardinality는 상속 → 검사 방향은 맞다. |
| algebra가 HIR/text를 재해석하는 shadow path | algebra/logic/planning `Cargo.toml`에 hir/surface/binder/modules edge 0건, 소스에 raw text 식별자 0건. |
| `normalize`가 fixed-point canonicalization/optimization | `normalize.rs`는 stage→node 1:1 필수 lowering만 수행하고 rewrite/수렴 loop가 없다(optional canonicalization 부재). 이름과 실제가 다르지 않다. |
| `CorrelationSetV2`/`LoweredRelationItemV2`/`SemanticPlanV2` 위조 | 필드 private + 생성자 `pub(crate)`/private(`node.rs:56,88,179`, `plan.rs:145`). plan은 `PlanningSessionV2`로만 생성. |
| 공개 `stratify`/`RuleDefV1`/`BodyLiteralV1`/`RecursiveProgramV1`가 문서 authority를 위조 | 어떤 소비자도 caller-built proof를 문서 authority로 받지 않는다(planning은 `contract.stratification()`, pipeline은 계약에서 clone: `compile.rs:1502`). |
| sema `test` feature forging seam이 production에 컴파일 | `#[cfg(any(test, feature = "test"))]` + `test` feature는 algebra/logic/planning의 **dev-dependencies**에서만 활성(`Cargo.toml`). |
| logic의 builtin/host call 분기(project_rules/validate_rule)가 실제 경로 | `ResolvedRuleV2::try_new`가 retained call의 target을 Predicate로 제한(`sema/src/rule.rs:111–115`) → sealed 문서에서 도달 불가한 방어 코드. 결함이 아니라 주석 정확도 이슈. |
| catalog `NoPlanReasonV2` 분류가 pipeline과 planning에 중복 | `item.rs:10–13`이 pipeline 소유로 명시, mapping은 exhaustive match(`compile.rs:1614–1624`)로 compile-time 동기 → adapter. |
| FIND endpoint legality가 sema와 logic에 중복 | sema에는 endpoint 존재 검사가 없다(역할/스팬만 resolve) → logic 단일 owner. |
| planning `validate_identity`의 `GraphQueryContractV2::from_resolved` 재실행이 중복 legality 결정 | transplant 검출용 **경계 재검증**이며 동일 owner 함수를 소비(두 번째 resolver 아님). |
| `assign_strata` relaxation이 수렴 실패를 조용히 Ok로 반환 | loop bound는 최대 C+1 pass이고 DAG condensation의 최장 경로는 C-1 edge → Bellman-Ford 한계 내 수렴. `changed==true`로 루프를 빠져나오는 경로는 도달 불가(정적). |
| `WitnessSelectionPolicyV2::TopK` dead vocabulary가 의미 손실 | 언어에 authored per-query witness cap이 없고(max_witnesses는 sealed budget; `sema/src/item.rs:212–230`), 문서도 TopK를 주장하지 않는다 → `MutationForbidden`과 같은 retained vocabulary. 언어에 top-k 기능이 계획되어 있다면 그때 미표현 결함이 된다(현재 아님). |
| V3 DTO가 planning 필드를 잃는다 | `execution.rs:576–610`이 authority/determinism/evidence/effect/policy_family를 투영하고, 나머지는 facade가 `SemanticPlanV2`를 재수출(`frontend-contract/src/lib.rs:46`)하므로 소비 경로가 존재 → Lane E 계약 문제로 handoff. |

## 5. 실행한 checks와 결과

- 명령 1(owner rail): `CARGO_TARGET_DIR=docs/audits/lane-d-scratch/target-laneD cargo test -p quanta-gqlang-v2-algebra -p quanta-gqlang-v2-logic -p quanta-gqlang-v2-planning`
- 결과 1: **exit 0**, 실패·무시·필터 0. 집계 **321 passed / 0 failed** (algebra 62: lib 21 + hellgate 11 + mutants 7 + normalization 15 + probe 8 / logic 111: lib 23 + endpoint_conflict 2 + depth_waypoint 2 + graph_query_hellgate 8 + logic_hellgate 4 + mutation_rails 9 + recursion 16 + source 25 + witness 6 + rule_aggregate_policy 5 + rule_scc_hellgate 11 / planning 148: lib 29 + capability 14 + cost 32 + explain 13 + item_hellgate 11 + cost_schema 5 + planning_hellgate 30 + planning_truth 14). 로그: `docs/audits/lane-d-scratch/lane-d-tests.log`.
- 명령 2(D-01 재현, pass-2): `cd docs/audits/lane-d-scratch/snapshot/tools/frontend_external_consumer && CARGO_TARGET_DIR=.../lane-d-scratch/target-laneD-repro cargo test --test lane_d_polarity_repro -- --nocapture`
- 결과 2: 테스트 **FAILED(의도된 회귀 확인)** — A/V2/V4/V6 OK, **B/V5 `logic.rule_program.polarity_flip`**. 로그: `docs/audits/lane-d-scratch/lane-d-repro.log`. fence code가 `logic.*`이므로 sema는 봉인했고 logic ingress가 거절한 것이 실행으로 확정됐다.
- 명령 3(부분 fix 실험, pass-2): snapshot의 `logic/src/source.rs` Predicate-target scope 제외 1줄 패치 후 명령 2 재실행 → **B OK / V5 여전히 ERR**(부분 패치의 불충분성 실행 확인). 실험 후 원본 복원, snapshot `source.rs` hash가 감사 트리·manifest와 동일함을 확인(`930708e1…`).
- 추가 실행 check: 감사 전/후 SHA-256 manifest diff(NO_DRIFT), `git diff -U0`로 dirty hunk 범위 확인, Lane D production 소스의 HashMap/HashSet·float 사용 0건(rg).
- **미실행**: `just ci`(조정자 1회 실행 몫), `just qualify-local`(요청 없음·dirty tree), 권장(비파괴) fix의 구현·검증, runtime·peak-memory 측정, conformance/privacy/architecture gate 직접 실행(→ Lane F).
- 정적 결론과 실행 결론의 분리: D-01은 **실행 재현된 CONFIRMED_DEFECT**(거짓 거절, 부분 fix 한계까지 확인)이고, 나머지 finding은 **정적 입증**이다(D-02/D-03/D-04/D-05는 재현 미실행). 본 lane 최종 집계: **CONFIRMED_DEFECT 1건(D-01, P2), STRUCTURAL_RISK 1건(D-03), PROOF_GAP 3건(D-02, D-04, D-05), DOCUMENTATION_DRIFT 1건(D-07), OPTIONAL_REFACTOR 3건(D-06, D-08, D-09)**.

## 6. 다른 lane handoff

- **Lane C (sema/types)**: D-01은 이제 재현된 CONFIRMED_DEFECT다. 권장 fix는 sema가 call 노드별 **intrinsic aggregation**을 봉인하고(`document.rs:2708–2730`에서 이미 계산됨) logic이 composed `effects().aggregation` 대신 그것을 소비하는 것이다. C는 (a) `ResolvedRuleCallV2`/typed call expression에 그 사실을 실을 최소 형태, (b) rule head symbol 타입 추론(`types/src/solver.rs:316–340`)과 Bool/record 파라미터 합법성, (c) `ResolvedRuleV2::try_new`의 Predicate-only 제약(`sema/src/rule.rs:111–115`)을 확정해 fix 범위를 닫아야 한다. D-05의 CHECK/COMPARE 다중도도 sema sealed fact 유무 확인이 필요하다.
- **Lane E (assembly/projection/facade)**: `execution.rs:576–610` `project_plan`이 `capabilities`/`semantic_cost`/`explain`/`result`를 V3 DTO에 투영하지 않는다 → D-07의 소비자 영향은 DTO가 아니라 facade의 `SemanticPlanV2` 경로에서 발생한다. V3 `GraphQuery` 인코딩(`docs/contracts/frontend-execution-v3.md:106`)은 relation-shape/dedup/max-witnesses/path-length/tie-break를 담지만 **witness_selection은 담지 않는다** — D-05와 함께 확인 필요. composition은 planning body 검사(D-03)와 pipeline `project_composition`의 body projection이 짝을 이룬다.
- **Lane F (boundedness/coverage)**: D-06(algebra ownership walk 무예산·2차)과 rule-graph 예산이 문서당 1회만 청구되는지(production caller는 `compile.rs:1500` 단 1곳임을 `rg`로 확인)를 coverage matrix에 반영. D-02의 생성자 일치 검사 공백과 **D-01의 sema↔logic polarity 일치 rail 부재**(현재 fixture는 두 유도가 일치하는 형태만 덮는다)를 coverage gap으로 등록해야 한다. D-01 재현물은 `docs/audits/lane-d-scratch/snapshot/tools/frontend_external_consumer/tests/lane_d_polarity_repro.rs`이며, fix 후에는 정상 회귀 rail로 승격 가능하다.
- **Lane A (authority map)**: Lane D의 IR/authority 항목은 §2 표를 그대로 사용 가능. `CapabilityV2`(planning owner)와 `NoPlanReasonV2`(planning 어휘, pipeline 분류)의 owner 분리는 authority-map에 명시할 가치가 있다.
