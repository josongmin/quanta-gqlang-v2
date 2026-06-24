# 최종 계획 — live-head 기준 semantica closeout (2026-06-11)

> **2026-06-15 authority update**
>
> 이 문서는 external repo에 남겨둔 coordination note다.
> semantica live truth를 대체하지 않는다.
>
> authoritative closeout state:
>
> - `/Users/songmin/Documents/code-new/semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/STATUS.md`
> - `/Users/songmin/Documents/code-new/semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh`
> - `/Users/songmin/Documents/code-new/semantica-codegraph-v2/docs/operation/jun-15-qglang-v2-authoritative-closeout-state.md`
>
> current live status at 2026-06-24:
>
> - `S21`, `S21A`, `S21B`, `S21C`, `S21D`, `S21E`, `J06` are all `complete` in semantica `STATUS.md`
> - the sections below preserve the 2026-06-11 open-work plan; they are not current completion claims
>
> use this file for historical coordination only.

이 문서는 `quanta-gqlang-v2` 쪽 coordination note 이다.
최종 권위는 항상 `semantica-codegraph-v2` 현재 코드, gate, ticket ledger 에 있다.

목적은 하나다.

- `F18b`, `F21`, `J06` 를 과장 없이 닫는 실제 순서를 고정한다.

---

## 1. 목표

최종 목표는 다음 두 조건을 동시에 만족하는 것이다.

1. `semantica-codegraph-v2` production runtime/gateway 가 legacy GQLang support/frontend carrier 권위에 더 이상 의존하지 않는다.
2. 그 상태에서만 `gate-gqlang-v2-semantica-closeout.sh` 가 요구하는 삭제/ledger/joint-bundle 조건을 만족시켜 `J06` 를 닫는다.

즉, 목표는 "external frontend 가 있으니 텍스트 ingress 만 repoint 하자" 가 아니다.
목표는 **live runtime owner cutover + 삭제 closeout** 이다.

---

## 2. 검증된 현재 상태

### 2.1 closeout gate 가 실제로 요구하는 것

`semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh` 는 현재도 다음을 강제한다.

- mandatory joint ticket bundle 존재 및 완결
- legacy support/frontend/product crate 부재
- `J06` 포함 mandatory joint row 확인

특히 삭제 대상으로 아직 명시된 crate 군은 다음과 같다.

- support: `quanta-gqlang-catalog-binding-v1`, `quanta-gqlang-validation-v1`, `quanta-gqlang-fragment-v1`
- frontend: `quanta-gqlang-core`, `quanta-gqlang-lowering`, `quanta-gqlang-hir-v1`, `quanta-gqlang-catalog-v1`, `quanta-gqlang-sdk-handlers-v1` 포함
- product: `quanta-gqlang-sarif-projection-v1`

즉 이 문서 작성 시점의 `J06` 는 gate 수준에서도 "crate absence + mandatory joint bundle" 전제가 살아 있었다.

### 2.2 production ingress truth

이 문서 작성 시점의 semantica production ingress 는 여전히 rich legacy carrier 를 만들었다.

- `GqlangExecutionRequestV1` 는 `normalized_ast`, `typed_query_v1`, `planner_carrier_v1`, `analysis_invocation_source_v1`, authority digest sidecar 를 들고 있다.
- native ingress 생성은 여전히 normalized AST 에서 planner carrier 와 authority bundle 을 직접 민트한다.
- `surface_lowering.rs` 도 여전히 parse -> admit -> typecheck -> normalize 경로를 타서 native request 를 만든다.

따라서 현재 구조는 `ServingQueryRequestV1` 직접소비형이 아니다.

### 2.3 bridge truth

`PublicIrRootV1 -> ServingQueryRequestV1` bridge 는 이미 있었지만 범위가 좁았다.

- query-plane root 만 lower 한다.
- `Declaration` 과 `Meta` 는 reject 한다.
- 이 bridge 존재를 곧바로 production gateway cutover 로 해석하면 오판이다.

즉 external repo 쪽 "public contract ready" 와 semantica 쪽 "runtime cutover ready" 는 다른 상태다.

### 2.4 현재 삭제 후보의 live consumer 규모

아래 숫자는 2026-06-11 시점 `cargo metadata --no-deps` reverse dependency 기준이다.

| crate | cargo consumer 수 | 현재 consumer |
|---|---:|---|
| `quanta-gqlang-sarif-projection-v1` | 2 | `quanta-gqlang-v2-exec-conformance`, `quanta-runtime-observation-wire` |
| `quanta-gqlang-fragment-v1` | 6 | `quanta-gqlang-catalog-binding-v1`, `quanta-gqlang-hir-v1`, `quanta-runtime-authority-support`, `quanta-runtime-execution`, `quanta-runtime-observation-wire`, `quanta-runtime-query-owner-surface` |
| `quanta-gqlang-validation-v1` | 8 | `quanta-gqlang-catalog-binding-v1`, `quanta-gqlang-sarif-projection-v1`, `quanta-runtime-absence-wire`, `quanta-runtime-authority-support`, `quanta-runtime-envelope-wire`, `quanta-runtime-execution`, `quanta-runtime-observation-wire`, `quanta-runtime-query-owner-surface` |
| `quanta-gqlang-catalog-binding-v1` | 6 | `quanta-gqlang-catalog-v1`, `quanta-runtime-absence-wire`, `quanta-runtime-envelope-wire`, `quanta-runtime-execution`, `quanta-runtime-observation-wire`, `quanta-runtime-query-owner-surface` |
| `quanta-gqlang-gates-v1` | 2 | `quanta-gqlang-v2-exec-conformance`, `quanta-runtime` |
| `quanta-gqlang-catalog-v1` | 3 | `quanta-adapters-security`, `quanta-runtime`, `quanta-runtime-execution` |
| `quanta-gqlang-sdk-handlers-v1` | 2 | `quanta-gqlang-hir-v1`, `quanta-runtime-command-owner-surface` |
| `quanta-gqlang-core` | 4 | `quanta-gqlang-lowering`, `quanta-runtime`, `quanta-runtime-execution`, `quanta-runtime-query-owner-surface` |
| `quanta-gqlang-lowering` | 1 | `quanta-runtime-query-owner-surface` |
| `quanta-gqlang-hir-v1` | 2 | `quanta-runtime`, `quanta-runtime-query-owner-surface` |

이 표가 말하는 건 단순하다.

- `sarif-projection-v1` 는 작은 삭제 파동 후보다.
- `fragment-v1`, `validation-v1`, `catalog-binding-v1` 는 live wire/support owner 다.
- `core`, `lowering`, `hir-v1` 는 `F21` 완료 전에 dead-crate cleanup 으로 취급하면 안 된다.

### 2.5 오늘 확보한 compile truth

아래 owner-local `check` 는 2026-06-11 시점 green 이었다.

- `scripts/quanta-build-cli cargo --lane <explicit closeout lane> -- check -p quanta-runtime`
- `scripts/quanta-build-cli cargo --lane <explicit closeout lane> -- check -p quanta-runtime-observation-wire`
- `scripts/quanta-build-cli cargo --lane <explicit closeout lane> -- check -p quanta-runtime-command-owner-surface`
- `scripts/quanta-build-cli cargo --lane <explicit closeout lane> -- check -p quanta-gqlang-v2-exec-conformance`

이전 `gqlang-closeout-p*-*` 표기는 historical shorthand 였다. `zsh`에서는 quoting 없이 복붙 실행되지 않는다.
`quanta-runtime` 는 당시 unused variable warning 3개가 있었지만 compile 실패는 아니었다.

---

## 3. As-Is

- external repo owner scope 는 syntax, sema, IR, CLI, doctor, LSP, public execution contract 안정성이다.
- semantica production runtime/gateway 는 아직 legacy admitted carrier rich field 를 직접 읽는다.
- closeout gate 는 여전히 legacy crate 부재와 mandatory joint bundle 을 요구한다.
- 삭제 후보 다수는 dead residue 가 아니라 현재 runtime/wire/frontend owner seam 이다.

정리하면 현재 상태는 다음 한 줄이다.

**external frontend 는 준비됐지만, semantica runtime/gateway closeout 은 아직 deletion wave + carrier cutover 프로그램 상태다.**

---

## 4. To-Be

- small leaf/support/product crate 는 순차적으로 owner seam 으로 흡수되어 삭제된다.
- runtime/gateway 는 legacy request 전용 field 직접소비를 끊고 runtime-owned replacement context 로 이행한다.
- 그 이후에만 `quanta-gqlang-core`, `quanta-gqlang-lowering`, `quanta-gqlang-hir-v1` 를 포함한 frontend delete row 가 진짜 삭제 가능 상태가 된다.
- 마지막에만 `J06`, mandatory joint bundle, regenerated snapshot, closeout gate 를 한 번에 닫는다.

---

## 5. 최종 실행 계획

### P0. 동결과 inventory

먼저 해야 할 일은 삭제가 아니라 동결이다.

- 전용 브랜치/워크트리로 gateway/wire 작업을 다른 동시 세션과 분리
- 각 대상 crate 의 cargo consumer + source consumer inventory 저장
- `GqlangExecutionRequestV1`, `GqlangBoundExecuteRequestV1`, `ExistingQuerySurfaceV1` 생성자와 reader cluster 고정
- 실제 cutover edit surface 는 root `quanta-runtime` owner path 로 제한:
  `packages/analysis/quanta-v2/crates/quanta-runtime/src/orchestration/gateway/surface_factory.rs`
  및 `surface_factory/*`
- `packages/analysis/quanta-v2/crates/quanta-runtime-orchestration/src/gateway/surface_factory/*`
  아래 동명 파일은 relocation/reference copy 로만 취급하고 V6B 수정면으로 사용하지 않는다
- compile rail 은 `scripts/quanta-build-cli` 로 통일

P0 완료 기준:

- crate별 reverse dependency 표 확보
- carrier field별 reader 목록 확보
- live compile owner 와 non-owner copy 경계 고정
- owner-local check lane 기준선 확보

### P1. 작은 삭제 파동

순서는 아래로 고정한다.

1. `quanta-gqlang-sarif-projection-v1`
2. `quanta-gqlang-fragment-v1`
3. `quanta-gqlang-validation-v1`

이 순서인 이유:

- `sarif-projection-v1` 가 소비자 2개로 가장 작다.
- `fragment-v1` 와 `validation-v1` 는 support/wire owner 이지만, `catalog-binding-v1` 보다 먼저 정리하지 않으면 뒤 파동이 안 열린다.

P1 작업 원칙:

- crate 하나 끝나기 전 다음 crate 로 넘어가지 않는다.
- 각 crate 마다 `consumer repoint -> owner-local check/test -> crate 삭제 -> residue 삭제` 순서를 강제한다.
- wire bytes/golden drift 가 나오면 즉시 중단한다.

P1 완료 기준:

- support/product 1차 대상 3개가 실제로 metadata/source 양쪽에서 사라진다.
- 관련 owner-local rails 가 green 이다.

### P2. 중간 삭제 파동

순서는 아래로 고정한다.

1. `quanta-gqlang-catalog-binding-v1`
2. `quanta-gqlang-gates-v1`
3. `quanta-gqlang-catalog-v1`
4. `quanta-gqlang-sdk-handlers-v1`

이 순서인 이유:

- `catalog-binding-v1` 는 P1 support seam 의 후속 owner 이다.
- `gates-v1` 는 conformance/runtime baseline 권위를 같이 정리해야 한다.
- `catalog-v1` 는 runtime + adapters-security + runtime-execution 3방향 소비가 있다.
- `sdk-handlers-v1` 는 command owner와 HIR state를 같이 건드리므로 P3 직전까지만 밀어둔다.

P2 완료 기준:

- support/proof/catalog/handler 중간 파동 대상이 source + metadata 에서 제거된다.
- runtime/wire/command owner-local rail 이 모두 green 이다.

### P3. `F21` carrier cutover 프로그램

여기가 핵심이다.
이 Phase 는 삭제 파동이 아니라 runtime/gateway 권위 cutover 다.

작업 축은 네 개다.

1. `legacy request 생성 사이트 축소`
   - 새 production code 가 `GqlangExecutionRequestV1` / `GqlangBoundExecuteRequestV1` 를 직접 추가 생성하지 못하게 막는다.

2. `reader cluster 별 replacement context 도입`
   - `normalized_ast`
   - `typed_query_v1`
   - `planner_carrier_v1`
   - `analysis_invocation_source_v1`
   - authority/digest sidecar

3. `runtime/gateway direct field read 제거`
   - public boundary
   - semantic subset
   - bound-plan materialization
   - IDE/helper surface
   - runtime execution helper

4. `frontend crate retirement prerequisite 충족`
   - `quanta-gqlang-core`
   - `quanta-gqlang-lowering`
   - `quanta-gqlang-hir-v1`

P3 완료 기준:

- production reader 가 legacy request 전용 필드 직접의존 0
- legacy request 신규 생성 site 0
- replacement context 로 동일 동작이 재현됨
- 그 다음에만 `core/lowering/hir-v1` 삭제 시작

### P4. `J06` closeout

마지막에만 아래를 실행한다.

- `S21` hostile-closeout 과 `S21A`~`S21E` child row 가 먼저 `complete` 여야 한다
- Semantica ticket row close
- mandatory joint bundle 재검증
- generated dependency snapshot regen
- `gate-gqlang-v2-semantica-closeout.sh` full run

P4 완료 기준:

1. mandatory joint bundle 통과
2. `S21` / `S21A`~`S21E` / `J06` row complete
3. 삭제 대상 crate 부재 확인
4. regenerated snapshot 과 committed snapshot 일치
5. closeout gate green

---

## 6. 금지 사항

다음은 false green 이다.

- `ServingQueryRequestV1` bridge 가 있으니 `F21` 완료라고 말하는 것
- cargo consumer 가 줄었다고 live owner crate 를 bulk delete 하는 것
- owner-local compile green 을 `J06` complete 로 승격하는 것
- `core/lowering/hir-v1` 를 support/product leaf 와 같은 deletion class 로 다루는 것
- external repo 계획 문서를 semantica gate 권위보다 앞세우는 것

---

## 7. 최종 결론

최종 순서는 이것으로 고정했다.

1. `P0` inventory 동결
2. `P1` 작은 삭제 파동: `sarif-projection-v1 -> fragment-v1 -> validation-v1`
3. `P2` 중간 삭제 파동: `catalog-binding-v1 -> gates-v1 -> catalog-v1 -> sdk-handlers-v1`
4. `P3` `F21` carrier cutover + `core/lowering/hir-v1` retirement prerequisite 충족
5. `P4` `J06` closeout

한 줄로 줄이면 이렇다.

**이 문서의 As-Is 는 2026-06-11 시점 external frontend ready + semantica runtime cutover pending 이고, To-Be 는 runtime/gateway owner cutover 뒤 delete-wave 와 `J06` closeout 이 완료된 상태였다. 현재 live truth는 상단 authority 문서를 따른다.**
