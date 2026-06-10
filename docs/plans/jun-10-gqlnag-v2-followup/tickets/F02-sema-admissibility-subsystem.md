# F02 — E05A/B/C/D sema admissibility 서브시스템 신설 (W1)

> **STATUS: NOT STARTED (jun-10 재점검).** external W1. 코드 확인: `sema/src/admissibility/` 디렉터리 부재 — 미착수. quanta-runtime 무관(external 빌드 green)이라 즉시 가능.


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
