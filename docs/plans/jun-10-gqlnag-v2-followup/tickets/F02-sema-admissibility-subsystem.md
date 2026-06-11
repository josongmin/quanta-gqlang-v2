# F02 — E05A/B/C/D sema admissibility 서브시스템 신설 (W1)

> **STATUS: DONE (jun-10) — 헬게이트 9 tests GREEN, workspace 0 regression.**
> - `sema/src/admissibility.rs` 신설(+ lib.rs `pub mod admissibility` + `analyze_document_v1` 에 doc-level 배선).
> - **E05A** `FindAdmissibilityV1` + `FIND_ADMISSIBILITY_MATRIX_V1[7]`(subkind→capability, 1:1) + `for_subkind_v1`(total). **E05B** `ExecutionProfileRuleV1`+`PROFILE_ADMISSIBILITY_MATRIX_V1`+`admit_for_profile_v1`(CiGate→Sample reject, typed `profile_admissibility_rejected`). **E05C** `GateSemanticsV1` + doc-level `admit_document_v1`: duplicate decl name(`declaration_duplicate_name`) + VIEW reference cycle(DFS, self+다중노드, `declaration_cycle`). **E05D** `CertifiedAbsenceV1::admit_v1`(`uncertified_absence_rejected`) + `BudgetOutcomeV1{RejectedBudget,Degraded}`.
> - SELECT reject 는 이미 parser(`syntax/select.rs:592-636`)에 있어 재구현 안 함(중복 회피).
> - 적대/객관 audit: cycle 다중노드 보강 테스트 추가(green). **잔여(honest):** find/profile matrix 는 queryable named-seam — analyze 파이프라인 reject 로의 enforcement 통합은 profile 운반 도입 후속(현재는 doc-level dup/cycle 만 강제). GateSemanticsV1 declaration 생성자 미배선.
> - 검증: `cargo test -p quanta-gqlang-v2-sema --test admissibility_matrix`(9) + `cargo test --workspace`(0 fail).


원천: E-sema-ir-retrieval 감사. `crates/quanta-gqlang-v2-sema/src/admissibility/` 디렉터리 자체가
디스크에 없음 (메인 세션 직접 확인). E05A/E05B/E05D = missing, E05C = partial.

## 현재 상태 (라이브)

- `admissibility/` 디렉터리 부재. `FindAdmissibilityV1` / `ExecutionProfileRuleV1` / `GateSemanticsV1` /
  `CertifiedAbsence` / `RejectedBudget` 전부 0 hits
- SELECT 의 "reject 로직"은 sema 가 아니라 syntax crate 의 prose 문자열 배열:
  `syntax/src/roots/select.rs:10` `SELECT_REJECTED_FORMS_V1: &[&str]` (JOIN/alias/ORDER BY 문자열)
- E05C: `sema/src/declarations/model_pack.rs:79` `TypedGateDeclarationV1 { name, checks, binding }` 만 존재 —
  DoD 가 요구한 `GateSemanticsV1 { requires_profile: ProfileIdV1, requires_pack: Option<PackRefV1>, requires_certified: bool }` 부재.
  3개 reject 경로(declaration cycle / duplicate name / parameter type mismatch) 전부 grep 0

## 작업

1. `crates/quanta-gqlang-v2-sema/src/admissibility/` 모듈 신설:
   - `select_find.rs` — `FindAdmissibilityV1`: subkind별 capability/admissibility 부착, SELECT reject 를 prose 배열에서 sema typed reject 로 이관
   - `check_explain_profile.rs` — `ExecutionProfileRuleV1`: RFC-002 Appendix J.3 per-root request-field admissibility matrix 를 단일 진실원으로 구현 (interactive/ai/lint/ci_gate 프로파일 matrix 포함)
   - `declarations.rs` — `GateSemanticsV1` (requires_profile/requires_pack/requires_certified) + 3개 reject 경로
   - `bounded_execution.rs` + `absence.rs` — `CertifiedAbsence` typed expectation, `RejectedBudget`/`Degraded` budget-exhaustion surfacing
2. 각 파일에 fail-closed negative 테스트 동반 (positive-only 금지)
3. E05C 보완: `declarations/pack_profile_gate.rs`, `declarations/typed_params.rs` (F10 의 syntax 측과 짝)

## DoD

- 위 named seam 전부 `rg` 로 발견 가능 + non-test 소비자 존재 (analyze 경로에서 실제 호출)
- declaration cycle / duplicate name / param type mismatch 3개 reject 가 각각 negative 테스트로 고정
- `cargo test -p quanta-gqlang-v2-sema` green

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: `sema/src/lib.rs:151 analyze_document_v1` / `:160 analyze_root_v1` 디스패치, `:249 sema_diagnostic_page_v1` 헬퍼(진단 채널 재사용 — 새 에러채널 금지). `TypedRootV1`(lib.rs:85) 9-variant. SELECT reject 산문은 `syntax/src/roots/select.rs:10 SELECT_REJECTED_FORMS_V1`.

### 생성 파일
1. **`sema/src/admissibility/mod.rs`** — 서브시스템 루트.
   - `pub enum SemaExecutionProfileV1 { Interactive, Ai, Lint, CiGate }` (RFC J.3 4-profile)
   - `pub(crate) fn admit_document_v1(roots: &[TypedRootV1]) -> Result<(), SemaDiagnosticPageV1>` — 문서 레벨 패스 집약(아래 declarations/trust 패스 호출).
2. **`sema/src/admissibility/select_find.rs`** (E05A)
   - `pub struct FindAdmissibilityV1 { pub subkind: &'static str, pub required_capability: &'static str, pub seed_shape: FindSeedShapeV1 }` + `pub const FIND_ADMISSIBILITY_MATRIX_V1: [FindAdmissibilityV1; 7]` (reachability/flow/slice/taint/path/points_to/summary — find.rs 의 7 subkind 와 1:1, 불일치시 컴파일타임 길이로 검출).
   - `pub fn admit_find_v1(&TypedFindRootV1) -> Result<(), SemaDiagnosticV1>` — matrix 밖 subkind/시드 fail-closed (`find_admissibility_rejected`).
   - `pub fn admit_select_v1(&TypedSelectRootV1) -> Result<(), SemaDiagnosticV1>` — `SELECT_REJECTED_FORMS_V1` 산문배열을 *여기서* typed reject 로 승격 (`select_rejected_form`, 거부 사유 = 매칭된 form). syntax 배열은 SSOT 로 재사용(중복 정의 금지).
3. **`sema/src/admissibility/check_explain_profile.rs`** (E05B)
   - `pub struct ExecutionProfileRuleV1 { pub profile: SemaExecutionProfileV1, pub root: ProfileRootKindV1, pub admitted: bool, pub reason: &'static str }`
   - `pub const PROFILE_ADMISSIBILITY_MATRIX_V1: &[ExecutionProfileRuleV1]` — RFC-002 Appendix J.3 per-root×profile 단일 진실원 (예: CiGate 는 SAMPLE/AI-lane 비허용, Lint 는 CHECK/SELECT 만).
   - `pub fn admit_for_profile_v1(root: &TypedRootV1, profile: SemaExecutionProfileV1) -> Result<(), SemaDiagnosticV1>` (`profile_admissibility_rejected`).
4. **`sema/src/admissibility/declarations.rs`** (E05C)
   - `pub struct GateSemanticsV1 { pub requires_profile: Option<String>, pub requires_pack: Option<String>, pub requires_certified: bool }` — `TypedGateDeclarationV1{name,checks,binding}`(model_pack.rs:79) + 문서 내 PROFILE/PACK 선언에서 도출(파서 변경 없음).
   - 3대 reject (문서 레벨): `declaration_duplicate_name`(VIEW/RULE/MODEL/PACK/PROFILE/GATE 이름 중복), `declaration_cycle`(VIEW→VIEW InlineSelect 참조 그래프 DFS), `declaration_param_type_mismatch`(F10 typed_params 산출물과 PROFILE capabilities 타입 대조).
5. **`sema/src/admissibility/bounded_execution.rs`** (E05D)
   - `pub enum BudgetOutcomeV1 { RejectedBudget, Degraded }` + `pub struct CertifiedAbsenceV1 { pub claim: String, pub certified: bool }`
   - `pub fn admit_absence_claim_v1(...) -> Result<CertifiedAbsenceV1, SemaDiagnosticV1>` — 미인증 absence 주장 fail-closed (`uncertified_absence_rejected`). F07 trust_policy 가 소비.

### 수정 파일 (배선)
- `sema/src/lib.rs`: `pub mod admissibility;` + `analyze_root_v1` 의 Select arm(:162)에 `admit_select_v1`, Find arm(:165)에 `admit_find_v1` 삽입; `analyze_document_v1`(:151) 마지막에 `admissibility::admit_document_v1(&roots)?` (선언 중복/cycle 은 문서 레벨이라 여기).

### 헬게이트 (red 먼저)
- **`sema/tests/admissibility_matrix.rs`** (신규): ① `SELECT ... JOIN ...` → `select_rejected_form` ② matrix 밖 find seed → reject ③ CiGate profile 로 SAMPLE → `profile_admissibility_rejected` ④ 같은 이름 RULE 2개 → `declaration_duplicate_name` ⑤ VIEW 자기참조 → `declaration_cycle` ⑥ positive: 기존 7 find subkind + 4 root 전부 admit. 전부 typed `.code` assert (positive-only 금지).

### 검증
`cargo test -p quanta-gqlang-v2-sema --test admissibility_matrix` + `cargo test --workspace` (기존 69+ green 유지 — 특히 sema_conformance/let_with_composition 비파괴).
주의: admit 삽입으로 기존 fixture 가 reject 되면 fixture 가 아니라 matrix 의 admitted 칸을 먼저 의심(기존 green 입력은 전부 admitted 가 사양).
