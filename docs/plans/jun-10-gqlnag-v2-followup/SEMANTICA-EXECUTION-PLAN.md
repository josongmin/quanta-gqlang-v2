# Semantica 최종 실행계획 (jun-10, 코드기준 어딧 보강판)

> **2026-06-15 authority update**
>
> 이 문서는 semantica residual work를 탐색하던 시점의 planning note다.
> live closeout truth는 아래를 우선한다.
>
> 1. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/STATUS.md`
> 2. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh`
> 3. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/docs/operation/jun-15-qglang-v2-authoritative-closeout-state.md`
>
> 특히 본 문서의 old `F18b`/`F21` framing, consumer 규모, 단계 이름은
> current semantica row/gate authority보다 앞설 수 없다.
>
> **⚠️ 부분 SUPERSEDED**: 배치 A(#1·#2·#3)는 완료. 잔여 #4(F21)·#5(F18b)·#6(J06)의 확정 계획은 **[FINAL-PLAN.md](FINAL-PLAN.md)** — 본 문서의 해당 절(§2의 #4/#5/#6)은 조사 원자료로만 유효.
> 대상 레포: `/Users/songmin/Documents/code-new/semantica-codegraph-v2`
> 검증: 4개 병렬 적대적 어딧(Explore) + spot-check + `cargo check -p quanta-runtime` EXIT=0.
> external(quanta-gqlang-v2) 측은 전부 DONE — 본 문서는 semantica 잔여 작업 전부.

## 0. 헤드라인 정정 (stale claim → 실측)

- **빌드 됨**: `quanta-runtime`·memssa·exec-contract·exec-host-spi·host-adapter 전부 클린 컴파일. F18/F21 의 "runtime 미컴파일 → 빌드 복구 후" 전제 **무효**. 유일 블로커 `codeagent-quanta-bridge` 바이너리(feature flag, gqlang 무관) → 작업 전부 unblocked.
- **F18 범위**: bare `quanta-planner`/`quanta-planner-contract` 는 **gqlang-* 아님** → F18 밖. **F18b = 10 gqlang 크레이트** (첫 어딧의 12는 planner over-match).
- **F21 규모**: carrier "15+ 소비처"는 과대계상 — 실 prod 생성 4사이트 중 **진짜 변환 2개**(나머지 pre-planning/subquery → `#[allow(deprecated)]`). test ~28.
- **F19 J03A**: `show_describe_handshake` 가 **external(ir) + semantica(exec-conformance) 양쪽 부재**. (F19 원 gap목록 J01/J02A/J09C 에 J03A 추가.)
- **J06**: gate 자체 뷰는 S04A/S04B/S12B/S13A-D 미완 → S 완료는 F18 삭제에 종속.

---

## 1. 작업 목록·의존·순서

| # | 작업 | 크기 | 의존 | 병렬군 |
|---|---|---|---|---|
| 1 | F19 J03A `show_describe_handshake` (sema + ir) | S | — | A |
| 2 | F20-J09C `explain_path_handshake`(sema) behavioral | S–M | — | A |
| 3 | F18a 삭제된 5크레이트 tooling residue 청소 | S–M | — | A |
| 4 | F21 carrier 강등 (adapter+deprecate+2사이트+guard) | M | — | B |
| 5 | F18b 10 gqlang 크레이트 재배선 후 삭제 | **L** | 빌드OK | C |
| 6 | J06 closeout (ledger regen + S완료 + joint bundle + `gate --all`) | M | 1·3·4·5 | D |

**순서**: 배치 A(1+2+3 병렬) → B(4) → C(5, critical path) → D(6). 각 단계 `cargo check -p <crate>`로 half-migrated 금지.

---

## 2. 항목별 구체 계획

### #1 — F19/J03A: show_describe_handshake (sema + ir)
- **sema**: 신설 `packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-conformance/tests/show_describe_handshake.rs`.
  - surface: `CapabilityHost` trait (`exec-host-spi/src/capability.rs`) — `list_capabilities(CapabilityQueryV1, HostCallCtxV1) -> Result<CapabilityCatalogV1, HostErrorV1>`. seam 상수 `CAPABILITY_HOST_SEAM_V1`(Analysis/P0), `CAPABILITY_HANDSHAKE_SURFACES_V1`(show_capabilities/describe_capability/…), `CAPABILITY_FAIL_CLOSED_REJECTS_V1`.
  - 모델: `tests/host_spi_contract.rs`(DummyHost 패턴), `catalog_sampling_handshake.rs`.
  - 테스트: behavioral DummyHost(`Ok` 반환, `unreachable!` 금지) + `list_capabilities` 실호출 positive + fail-closed reject ≥2(command_bridge/truth_claim_surface) + seam plane assert.
  - ⚠️ 구현시 실측 필요: `CapabilityCatalogV1`/`CapabilityEntryV1`/`HostCallCtxV1` 정확 필드(agent 추정).
- **ir(external)**: 신설 `crates/quanta-gqlang-v2-ir/tests/show_describe_handshake.rs` (J03A 티켓 named file). show/describe lowering round-trip — 기존 external describe 경로(F06 `fact_catalog`/show_describe) 재사용.
- **검증**: `cargo test -p quanta-gqlang-v2-exec-conformance --test show_describe_handshake` + external `cargo test -p quanta-gqlang-v2-ir --test show_describe_handshake`.

### #2 — F20/J09C: explain_path_handshake(sema) behavioral 승격
- **파일**: `exec-conformance/tests/explain_path_handshake.rs` 수정.
- traits: `ExplainHost`(`explain.rs:22`, 1메서드), `PathHost`(`path.rs:29`, find_path/find_flow/find_slice/find_reachability).
- 현재 constant-mirror 블록(EXPLAIN_HOST_SEAM/PATH_HOST_SEAM/FAIL_CLOSED_REJECTS 자기비교) 제거.
- behavioral DummyHost: `unreachable!` → 실제 `Ok(...)` 반환. 각 메서드 실호출 positive + typed reject ≥2(ranked_retrieval_truth_promotion / unsupported_retrieval_shape). seam plane assert 1.
- ⚠️ 실측 필요: `ExplainPayloadV1`/`PathQueryResultV1`/`SliceQueryResultV1`/`ReachabilityResultV1`/`HostCallCtxV1`/`HostErrorV1` 필드.
- **검증**: `cargo test -p quanta-gqlang-v2-exec-conformance --test explain_path_handshake`.

### #3 — F18a: 삭제된 5크레이트 tooling residue 청소
대상 크레이트(이미 dir 부재, 0 cargo 소비처): ranked-proof-contamination-detector-v1, doctor-v1, planner-v1, sigil-v1, owasp-matrix-v1.
- residue 위치(실측): `tools/ci/gates/baselines/core/{clone-budget,pub-surface-budget,cargo-dep-budget}.v0.json`, `tools/ci/lint/{r-test-33-stringly-error-baseline.txt,r-arch-11-wildcard-baseline.txt,pyo3-thin-adapter-dependency-keys-allowlist.txt,god-surface-baseline.json}`, `tools/ci/honesty/baselines/*.json`, `tools/rust-crate-boundary-guard/policies/{quanta-runtime-gqlang-hot-path.toml,…}`, `tools/catalog/entries/{domain_contracts/quanta_runtime_gqlang_wire_bundle.yaml,domain_contracts/quanta_gqlang_v1_residual_leaf_stack.yaml,adapters/gqlang_doctor_v1.yaml}`, `tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh`(목록행), `tools/ci/gqlang-v2/generate_dependency_snapshot.py`.
- 행동: baseline(허용치)는 행 삭제, lint allowlist 는 행 삭제, gate 목록행은 deleted-crate needle 로 재발 방지(F15 패턴), wire_bundle.yaml 은 살아있는 owner 경로로 재작성 또는 행 폐기.
- ⚠️ baseline vs 의도적 항목 구분 후 삭제 (god-surface-baseline.json 등은 변동 가능). 삭제 후 해당 게이트 재실행.

### #4 — F21: V6B carrier 강등
- carrier 정의(dual #[path]): `quanta-runtime/src/orchestration/gateway/surface_factory/{request_types.rs(GqlangExecutionRequestV1:21, GqlangBoundExecuteRequestV1:52), surface_types.rs(ExistingQuerySurfaceV1)}` + orchestration 미러.
- 목적지: `ServingQueryRequestV1`(exec-contract/request.rs:5) + `lower_public_ir_root_to_request_v1`(ingress-bridge/query_bridge.rs:47).
- **계획**:
  1. 변환 어댑터 신설 `surface_factory/carrier_to_serving_adapter.rs`: `from_gqlang_execution_request_v1(&GqlangExecutionRequestV1) -> Result<ServingQueryRequestV1>` (필드매핑: source→source_scope, budget→budget, normalized_ast→root via lower, options+proof_level+solver→execution_profile, analysis_invocation_source→required_capabilities). 미러 crate 도 동일.
  2. 3 carrier 에 `#[deprecated(note="V6B compat — adapter 경유만")]`.
  3. **컷오버**: Site2 `request_constructors.rs:45`(+미러 Site4) 만 어댑터 경유로 전환(post-planning, ServingQueryRequest 구성가능). **Site1 native_lowering.rs:460(pre-planning, execution_profile 부재)·Site3 semantic_subset_admission:638(subquery context)** 는 변환 불가 → `#[allow(deprecated)]`+exempt 주석. `ExistingQuerySurfaceV1→RequestSpec` cross-domain 변환은 defer(별 PR).
  4. residue guard: `scripts/gate-f21-carrier-demotion-residue.sh` — `GqlangExecutionRequestV1 {`/`ExistingQuerySurfaceV1 {` 직생성 검출(어댑터/new_v1/impl 제외). mutation-proof: 직생성 주입 시 FAIL.
  5. round-trip 헬게이트: carrier == `from_serving_request_v1(...)` 필드동치.
- **검증**: 각 단계 `cargo check -p quanta-runtime --all-features` + `-p quanta-runtime-orchestration` + round-trip test + guard script.
- ⚠️ test 생성사이트 ~28 은 `#![allow(deprecated)]` 또는 어댑터 헬퍼로 일괄 처리(가드는 prod 만).

### #5 — F18b: 10 gqlang 크레이트 재배선 후 삭제 (critical path)
**DAG topo 삭제순서(leaf 먼저)**:
```
P1(leaf):   validation-v1, fragment-v1, gates-v1, sdk-handlers-v1(orphan), sarif-projection-v1(orphan)
P2:         catalog-binding-v1, hir-v1
P3:         catalog-v1
P4:         core, lowering
```
의존: catalog-binding-v1→{fragment-v1,validation-v1}; hir-v1→{fragment-v1,sdk-handlers-v1}; sarif-projection-v1→validation-v1; catalog-v1→catalog-binding-v1; lowering→core.

**소비자 클러스터**: wire(absence/envelope/observation-wire), execution(load-bearing), query-owner-surface(load-bearing), runtime/adapters-security/bench(optional/feature).

**난이도·전략(쉬운것 먼저)**:
- 매우쉬움: gates-v1(dev-only 1소비), sdk-handlers-v1·sarif-projection-v1(orphan path-dep → 단일소비처 inline).
- 쉬움: fragment-v1(2타입 FragmentIdV1/FragmentRejectionV1), validation-v1(순수 re-export 컨테이너), catalog-v1(feature-gated).
- 중간: hir-v1(self-contained, 1주소비), lowering(parser, core AST 의존).
- **어려움(클러스터)**: catalog-binding-v1(wire 5소비, 순환위험 — trait object 분리), **core(100+ AST 타입, 최대)**.

**각 행 절차**: 소비자가 쓰는 심볼 inventory(`rg "use quanta_gqlang_<x>::"`) → 재배선(re-export shim inline / dead dep drop / 로직 이전) → dir+manifest 삭제 → ledger regen → `cargo check -p <소비자클러스터>`.

⚠️ 리스크: wire 순환의존(catalog-binding 분리시) / core AST 산재 / feature flag(serde-wire,cbor-digest) 보존.

### #6 — J06 closeout
gate: `tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh`.
- **3 조건**: (a) 전 S-ticket `complete` (현재 S04A/S04B source-landed, S12B/S13A-D planned/partial — F18b 삭제로 해소) (b) mandatory joint bundle J01~J09D green(`check_mandatory_joint_bundle`: 4 owner-local host-adapter test + 11 conformance test + J03A 신설 필요) (c) `gate --all` PASS.
- **ledger**: `python3 tools/ci/gqlang-v2/generate_dependency_snapshot.py` → `git diff --exit-code HEAD -- <snapshot dir>`(F15 staleness guard) → 재생성 2회 동일성. `delete_rows_zero`/readiness md 가 전 행 "already removed" 보고.
- **순서**: F18b·F21 land 후 → ledger regen+commit → F15 헬게이트(`test-gate-...-f15.sh`) → S-ticket 별 `--ticket Sxx` → joint bundle(J03A 포함) → `--all`.
- F15/F16/F17 산출(gate needle/ledger alias-awareness/digest drain)은 DONE 상태 확인됨.

---

## 3. 오픈 이슈 / 결정 필요

1. **semantica 작업 브랜치 전략**: 현재 `main` 에 uncommitted WIP(memssa rename.rs/tests.rs, taint assign.rs). 본격 착수 전 작업 브랜치 분리 + WIP 처리 방침 필요.
2. **F18b core 삭제 깊이**: core(100+ AST) 삭제는 최대 작업. "삭제" vs "소비자 0으로 줄이고 보류" 중 J06 DoD(전 행 absent)는 완전삭제 요구 → 완주 필요.
3. **J03A external ir 파일**: gate-blocking 은 sema(exec-conformance) 쪽. external ir 파일은 J03A 티켓 named-touch — 동시 생성 권장(외부 "done" 보강).
4. **agent 추정 필드명**: #1/#2 의 result/ctx 타입 필드는 구현시 실파일로 재확인(본 계획의 ⚠️ 표시).

---

## 3.5 진행 현황 (jun-10 구현)

- ✅ **#1 F19/J03A** — `show_describe_handshake`: sema `exec-conformance`(CapabilityHost behavioral, 6 green) + external `ir`(fact_catalog SHOW/DESCRIBE, 3 green). 둘 다 clippy-clean. SHOW=list-all / DESCRIBE=narrow / fail-closed(forbidden surface + unknown) typed-code negative.
- ✅ **#2 F20/J09C** — `explain_path_handshake`(sema) constant-mirror → **behavioral**: 실 ExplainHost/PathHost impl, 전 메서드 실호출, typed-code negative 4(no-result/ranked-promotion/degenerate-path/empty-slice). 전체 surface-list mirror 제거, plane-ownership invariant 유지. 5 green clippy-clean.
- 🔁 **#3 F18a** — J06 regen 으로 re-sequence. 사유: god-surface/honesty/r-test-33 baseline 은 auto-gen이고 F18b 삭제로 더 늘어남 → 단일 regen pass 가 정답(수기 JSON 편집은 fragile + 낭비).
- 🔬 **#4 F21 구조 발견(중요)**: carrier(`GqlangExecutionRequestV1`)는 runtime 내부 rich 요청(QueryId/AnalysisOptions/ProofLevel/native AST/PlannerCarrier/digests)으로 public `ServingQueryRequestV1`의 **하류**. "ServingQueryRequest→carrier 변환 어댑터"는 **데이터흐름 역행** + 1/4 사이트만 커버하는 대형 AST→spec lowering → **기각**. 구조적 정답 = **freeze-guard + deprecation**(DoD 옵션 b): 신규 직접생성 0 가드 + compat-only 격리. (full-absorb 는 ingress 대형 리팩터와 함께여야 함.)

## 3.55 진행 현황 (jun-11 추가 — 안전·커밋된 작업)

semantica 브랜치 `gqlang-v2-semantica-closeout` 에 커밋(동시 작업 에이전트의 reset 으로부터 보호):
- `91a31d2e1b` — F19/F20: 6 handshake constant-mirror → behavioral (show_describe·explain_path·fact_model·autocomplete·catalog_autocomplete·zero_query). 실 host trait 호출 + typed `.code` fail-closed.
- `d479da386c` — J04 `catalog_sampling_handshake` behavioral (SamplingHost+CatalogIntrospectionHost, ranked candidate/include_examples/budget reject).
- `9a2b59fd2a` — J03A `show_describe_handshake` 를 gate `check_mandatory_joint_bundle` 에 배선(문서화된 게이트 gap 해소).
- 전부 green + clippy-clean + 적대적 어딧(non-vacuous: impl+호출+expect_err) 통과. 7 fixture / 30 tests.
- ⚠️ **동시 작업 충돌**: 같은 브랜치에 다른 에이전트가 taint/ssa/memssa 를 실시간 커밋 중 → 미커밋 작업이 reset 으로 1회 클로버됨(복구+커밋으로 보호). F18b/F21 은 runtime/gateway 를 건드려야 해 **이 동시 작업이 끝나기 전엔 충돌 위험**.

## 3.6 🔴🔴 자율런 스코프 정정 (jun-11 실코드 검증 — F18b/F21 premise 무효)

이전 어딧(agent)의 크레이트 크기·소비처 추정이 **심각하게 과소**였음. 실측:

**F18b 크레이트는 dead leaf 가 아니라 LIVE 적재(load-bearing) v1 구현 — v2 surface 가 re-export 로 의존 중:**
- `gqlang-gates-v1` **5908 LoC** — 살아있는 gate-registry SSOT(published baseline gate IDs). 소비: `quanta-runtime`/`exec-conformance` 의 epic8 baseline 계약 테스트. 삭제 = registry 를 v2 로 이관하는 마이그레이션.
- `gqlang-sdk-handlers-v1` **6155 LoC** — `quanta-runtime-command-owner-surface` 가 `command_handler`/`error`/`HANDLER_SCHEMA_VERSION` 를 **pub use 재공표**, `hir-v1/environment.rs` 가 `CommandOutcomeKindV1`/`OutcomeStateV1` 사용. orphan 아님(실소비).
- `gqlang-sarif-projection-v1` **1601 LoC** — `observation-wire/diagnostic_surface_v1` + `exec-conformance/doctor_surface_port`(NegativePhraseRegistryV1) 사용.
- 나머지(catalog-binding 21소비·validation 18·fragment 13·core 10·…) 동일 — 전부 live.
- **결론**: "delete waves" 는 dead-crate 청소가 아니라 **v1→v2 로직 이관(멀티위크, 어텐디드)**. 한 크레이트씩 (소비처 전부 v2 등가물로 재배선 → green → 삭제) 점진 진행해야 하며, **무인 일괄삭제 금지**(349-crate 워크스페이스 깨짐).

**F21 carrier 는 residual compat 가 아니라 LIVE 게이트웨이 적재 타입:**
- `GqlangExecutionRequestV1`/`GqlangBoundExecuteRequestV1`/`ExistingQuerySurfaceV1` 가 **~40 게이트웨이 파일**에서 참조/전달(universal_cutover·data_fetching·semantic_subset_*·ide_subset_*·public_boundary·native_execution·…). "4 생성사이트" 는 오산.
- freeze-guard(40 활성파일 allowlist) 무의미, `#[deprecated]` 는 40파일 경고폭발. **demotion = 게이트웨이 ingress 전면 리팩터**(ServingQueryRequestV1 직접소비로 전환 후 carrier 삭제) — 대형 어텐디드.

**자율런에서 한 일**: 깨질 위험 0 인 안전·가산적 작업만 — 배치 A(완료) + 본 스코프 정정 문서화. **F18b/F21/J06 은 무인 실행 부적합**(빌드 파괴 위험) → 어텐디드 세션에서 점진 진행.

**다음 세션 권장 진입점**: F18b 를 **가장 작은+명확한 v2 목적지가 있는 크레이트 1개**부터(예: sarif-projection-v1 1601 LoC → observation-wire 로 흡수). 소비처 재배선 → `cargo check -p <소비처>` green → 삭제 → 반복. F21 은 ingress 리팩터 설계 먼저.

## 4. 권장 착수
~~배치 A 병렬~~ ✅ 완료(J03A sema+ir, J09C behavioral, 14 green). 잔여(F18b/F21/J06)는 **어텐디드 점진 마이그레이션** — 위 3.6 진입점 참조. 무인 일괄삭제 금지.
