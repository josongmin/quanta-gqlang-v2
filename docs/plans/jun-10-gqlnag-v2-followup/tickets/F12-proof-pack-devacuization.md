# F12 — S04A/S11/S17–S20 proof-pack registry 탈공허화 + claim_state 강제 (W2, semantica)

> **STATUS: CORE DONE (jun-10) — SSOT exec-contract GREEN (17 tests, 0 warn); conformance rewire static.**
> - **SSOT 신설** `exec-contract/src/proof_pack.rs` (+ lib.rs mod/re-export): `ProofPackFamilyV1`(4 closed families), `ProofPackRegistryV1::ROWS_V1`(가족별 **DISTINCT** parity_criteria/residual_risk_report — 감사가 지적한 동일 보일러플레이트 제거), `validate_v1`(가족 1:1 커버 + non-empty + parity/residual **pairwise distinct** fail-closed), `row_id_v1`(게이트 grep 어휘), `ClaimStateV1`(S11 5-milestone: source_landed→owner_local_green→joint_green→delete_closeout→final_complete) + `can_advance_to_v1`(스킵 금지) + `is_final_v1`.
> - **conformance 탈공허화** `exec-conformance/tests/proof_pack_registry.rs`: test-local `PROOF_PACK_ROWS_V1` 타우톨로지 제거 → SSOT import 후 검증(validate/distinctness/claim-discipline). row-id 리터럴은 게이트 grep 위해 assert 형태로 유지 → **F15 per-ticket S17-20 grep 과 정합**.
> - **검증:** `cargo test -p exec-contract` 17 GREEN(F12 4 + F11-S02 3 + 기존 10), 0 warning. 적대 mutation proof: residual 2개 동일화 시 validate_v1 FAIL(비공허). 게이트 4개 row-id 리터럴 잔존 확인.
> - **Deferred(honest):** (1) `LegacyProofPortV1`(S04B) — 레거시 proof-port 어휘 미상이라 미구현(날조 안 함). (2) 게이트가 per-family `claim_state` 를 출력하는 통합(현재는 코드 enforce 까지). (3) conformance 테스트 빌드검증 — quanta-runtime 블록.



원천: S-contract-spi-proof + S-ledger + S-proofpack 감사. 공통 진단: `proof_pack_registry.rs` 가
**test-local 리터럴을 자기 자신과 비교하는 tautology** 이고, claim discipline 은 prose 로만 존재.

## Gap 목록 (라이브)

- `quanta-gqlang-v2-exec-conformance/tests/proof_pack_registry.rs`:
  - `:7-35` `ProofPackRowV1` struct + `PROOF_PACK_ROWS_V1` 이 **테스트 파일 안에서 정의된 리터럴** —
    production crate 에서 import 하는 named SSOT `ProofPackRegistryV1` 0 hits, `LegacyProofPortV1` 0 hits
  - `:56-67` row_ids == 같은 파일 리터럴 (동일성 tautology — 단 4-id 배열 고정 자체는 유의미)
  - `:42-53` fail_closed 테스트 = 생성자가 설정한 필드를 그 생성자에 대해 assert (constructor tautology)
  - `:38-40` `proof_owner_crate_is_not_empty_shell` = const==own-literal, 3줄짜리 lib.rs 상대 — empty shell 탐지 불능
  - `:79` parity_criteria 검사 = `.contains("parity")` 뿐 — 4 row 의 distinctness/family-정확성 미검증
  - residual_risk_report (`:18,23,28,33`) 4 row 동일 보일러플레이트 — per-class 분리의 정반대
- S11 claim_state discipline (`source_landed != owner_local_green != joint_green != delete_closeout !=
  final_complete`, Core Logic line 46): **코드 어디에도 enforce 없음** (grep 0)
- S17–S20 DoD bullet 2 "Claim discipline reports this family separately": gate/소스 어디에도 미구현

## 작업

1. `ProofPackRegistryV1` 를 production crate (exec-contract 또는 conformance src) 의 SSOT 로 이동 —
   테스트는 import 해서 검증 (리터럴 거울 금지)
2. row 별 실제 proof 파일/테스트 owner 로의 greppable 연결 (DoD line 52): row → 파일 경로 매핑 필드 +
   존재 검사 테스트
3. `LegacyProofPortV1` enum 실체화 (S04B named SSOT)
4. claim_state 5-state enum 도입 + STATUS 보고가 이를 사용하는 검증 레일 (최소: registry 가 state 필드 보유,
   gate 가 state-별 보고 출력)
5. parity_criteria/residual_risk_report 를 family 별 실질 내용으로 분리 + distinctness 테스트
6. F15 의 gate 차별화(S17–20 분리 arm)와 짝 — registry 측은 row 별 단독 검증 함수 제공

## DoD

- registry SSOT 가 production 에 있고 테스트가 import (자기참조 0)
- 4 row 각각 단독으로 fail 가능 (F15 게이트와 합동)
- claim_state 가 코드로 enforce + grep 가능
