# Semantica 잔여 작업

> ## ⚠️ 재점검 갱신 (jun-10, 빌드 복구 후) — 아래 표는 빌드차단 시점 스냅샷, 다수 항목 DONE
> `quanta-runtime` 빌드가 복구되어 정적 추정이 cargo 검증으로 승격됐고, 표의 상당수가 완료됐습니다.
> 권위 있는 현재 상태는 [INDEX.md 대시보드](INDEX.md) 입니다. 요약:
> - **DONE(검증):** F11(S01/S02/S03), F12(S04A/S11/S17–20/S04B), F13(S05A/S06A/S06B), F14-S07C, F15, F16, F17, F18(4/14).
> - **남은 semantica:** F18-10(실소비처 재배선 후 삭제), F21(V6B carrier 15+ cutover), planner-v1 tooling 11파일, F14-S08C feature 레일.
> - **참고:** semantica 가 아닌 **External W1(F02–F08)** 이 최대 미착수 덩어리 — 이 문서 범위 밖, INDEX 참조.
> 아래 원본 표는 빌드차단 당시(정적) 기록으로 보존.

---

# (원본) Semantica 잔여 작업 — 최종 확정 (jun-10, 라이브 코드 정적 검증)

메인 세션이 현재 `semantica-codegraph-v2` HEAD(`7cd0c4aed4`) 라이브 소스를 직접 grep/read/git 으로 검증.
빌드는 PTA/DFG 캠페인(quanta-runtime 미컴파일)으로 막혀 있어 **정적 검증**. 이미 끝낸 항목(F01/F09/F11-S02/F16-generator) 제외.

## 결론: 9개 유닛 전부 still-open (조용히 끝난 것 없음, 일부는 더 커짐)

| 티켓 | 항목 | 검증된 현재 상태 (file:line) | 액션 | 공수 | 빌드검증 |
|---|---|---|---|---|---|
| **F11** | S01 CandidateBasisV1 | `exec-contract/src/result.rs:275-280` 여전히 `{Lexical,Semantic,Hybrid,Fragment}` (spec: `{Heuristic,Inferred,Certified}`) | variant 교정 + 소비처(ai_workflow bridge/conformance) 재배선 | M | ⚠️ ripple→블록 |
| **F11** | S01 finding plane | `DiagnosticProjectionV1/IssueKindV1/IssueWorkflowStateV1/HotspotStatusV1/RankingMetadataV1` = **0 hits** | exec-contract 에 타입 신설 + 검증 | M | ✅ exec-contract |
| **F11** | S03 trait coverage | `host_spi_contract.rs` DummyHost 가 8 analysis trait(Resolve/Relation/Path/Taint/PointsTo/Summary/Explain/PatternLint) **0 coverage** | DummyHost impl + 호출 + assert 추가 | M | ❌ conformance→runtime 블록 |
| **F12** | proof-pack SSOT | `ProofPackRegistryV1`/`LegacyProofPortV1` = **0 hits**; `proof_pack_registry.rs` 여전히 89줄 tautology | production SSOT 신설 + 테스트 import | M | ✅부분(SSOT 는 contract) / ❌(test) |
| **F12** | claim_state 규율 | `claim_state`/`final_complete` = **0 hits** (코드 enforce 없음) | 5-state enum + 보고 레일 | M | ❌ |
| **F13** | S06A catalog_adapter | `host-adapter/src/catalog_adapter.rs` **부재** | 파일 생성(show/describe catalog 로직 집약) | S | ❌ host-adapter→runtime 블록 |
| **F13** | S06B vacuous proof | `search_projection_ownership.rs` 여전히 17줄, `quanta-index` assert **0** | 진짜 ownership proof 로 교체 | S | ❌ |
| **F13** | S05A reject 커버 | `adapter_lowering_matrix.rs` `is_err/expect_err` = **0** | UnsupportedEntityKind reject 테스트 추가 | S | ❌ |
| **F14** | S07C named guard | `FORBIDDEN_QUERY_BOUNDARY_IMPORT` **0 hits**, `v2_guard.rs` 부재 | const+guard 구현, `wire_to_boundary_v1.rs` 가드 포함 | S | ❌ sdk→runtime 블록 |
| **F14** | S08C default stub | `Cargo.toml:30 default=["binding-core"]`, `gqlang_command_handler.rs:302 #[cfg(not(mutation-backend-runtime))]→:309 CommandProviderFailed` | 결정: feature 승격 vs 명문화+CI 레일 | S | ❌ |
| **F15** | gate survivor 모순 | `gate:122 SHALLOW_FOLD_DELETE_CRATES` 에 LIVE survivor `quanta-runtime-query-preset-wire` 잔존 | `:122` → legacy shim `-gqlang-query-preset-wire` 로 교정 | S | ⚠️ gate 실행=cargo 메타 필요 |
| **F15** | gate S17-20 비차별 | `S17\|S18\|S19\|S20` 단일 arm → `check_proof_pack_rows`(gate:1092) | row 별 단독 검사 분리 (F12 와 짝) | M | ⚠️ |
| **F15** | staleness 세탁 | `git diff/status/dirty/porcelain` 가드 = **0** (gate:744-759 fresh×2 diff) | regen-vs-committed `git diff --exit-code` 추가 | S | ⚠️ |
| **F16** | S09C orphan stale | `current-orphan-source-readiness.md:7,9` public-entry-v1 가 active+absent 양쪽에; `r-arch-11-wildcard-baseline.txt:72` 는 이제 sarif-projection-v1 | F17 후 `generate_dependency_snapshot.py` regen + 커밋 | S | ⚠️ regen=cargo 메타 필요 |
| **F17** | digest 별칭 드레인 | `quanta-gqlang-digest-v1` = **39 Cargo.toml + 20 .rs** (감사 추정 13/17 보다 증가); `hir-v1/Cargo.toml:27` package-rename 잔존 | canonical `codegraph-core-ir-foundation` 으로 일괄 재배선 | L | ❌ 빌드 검증 필요 |
| **F18** | 삭제 14행 | 14개 crate dir 잔존(doctor-v1/planner-v1 만 absent): catalog-binding/validation/fragment/gates/ranked-proof-contamination-detector/core/lowering/hir-v1/catalog-v1/sdk-handlers-v1/sigil-v1/planner(bare)/owasp-matrix-v1/sarif-projection-v1 | 소비자 재배선 → dir+manifest 삭제 → gate | L | ❌ |
| **F18** | planner-v1 tooling residue | `planner-v1` 가 ~10개 tooling 파일에 잔존 (catalog yaml, boundary policy:6, planner-lane lint:6, god-surface-baseline.json:306 등) | 삭제+게이트 needle | M | ⚠️ 정적 |
| **F21** | V6B carriers | `request_types.rs:21 GqlangExecutionRequestV1`, `:52 GqlangBoundExecuteRequestV1`, `surface_types.rs:67 ExistingQuerySurfaceV1` 잔존 | demote(삭제/`#[deprecated]`+가드) + 소비자 cutover | M | ❌ |
| **F21** | J06 closeout | 위 carrier+digest+planner blocker 존재 → `delete_rows_zero` false | F16/F17/F18 후 기계 검증 | S | ⚠️ |

## 하드 전제조건 (모든 ❌ 항목의 게이트)
- **`quanta-runtime` 가 다시 컴파일되어야** host-adapter/ingress-bridge/exec-conformance/sdk 가 빌드되고, F11-S03/F12-test/F13/F14/F17/F18/F21 의 green 검증이 가능. 현재 사용자 팀의 PTA/DFG 캠페인이 mid-flight(미컴파일).
- ✅ = `exec-contract`(leaf wire 의존) 단독으로 검증 가능: F11-finding-plane, F12-SSOT(contract 측).
- ⚠️ = 정적 편집은 가능하나 완전 검증(gate 실행/ledger regen)은 cargo metadata 필요.

## 권장 순서 (의존성)
1. **(전제) quanta-runtime 컴파일 복구** ← 사용자 PTA 캠페인. 이게 풀리면 ❌ 전부 verifiable.
2. **F15 게이트 수리** — F12/F18 의 검증 레일이므로 먼저.
3. **F11/F12/F13/F14** — 계약/proof/adapter/sdk 수리 (병렬 가능, crate 분리).
4. **F17 digest 드레인 → F16 ledger regen** (순서 의존).
5. **F18 삭제 웨이브 → F21 carrier demote + J06 최종 closeout** (마지막).

## 지금(빌드 블록 중) 안전하게 가능한 것
- F11-finding-plane 타입 + F12 ProofPackRegistryV1 SSOT 를 **exec-contract** 에 추가(단독 빌드/테스트 가능).
- F15 게이트 3종 수리(셸 — 정적; 실행 검증만 보류).
- F16 orphan 문서 정정(정적; regen 은 보류).
나머지(F13/F14/F17/F18/F21, F11-S03)는 quanta-runtime 컴파일 복구 후 진행이 안전.
