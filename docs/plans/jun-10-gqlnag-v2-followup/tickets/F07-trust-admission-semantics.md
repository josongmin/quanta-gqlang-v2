# F07 — E10B REQUIRE certified: admission/trust 의미론 구현 (W1)

> **STATUS: DONE (jun-10) — proof rail `--test trust_policy` 5 GREEN, workspace 0 regression.**
> - `admissibility.rs` 에 `enforce_trust_policy_v1` 추가(+ `admit_document_v1` 에서 호출). `REQUIRE CERTIFIED` 활성 시 3대 fail-closed: ① 임의 SAMPLE(`require_certified_rejects_sampling`) ② CHECK confirm≠Require(`require_certified_rejects_unconfirmed_check`) ③ PROFILE admission≠CertifiedOnly(`require_certified_profile_incompatible`).
> - **과차단 가드(rule#5)**: REQUIRE CERTIFIED 없는 문서는 sampling/unconfirmed check 자유 admit — 테스트로 고정.
> - 설계 메모: SAMPLE mode 는 typed root 에 미보존이라 reject① 을 "임의 SAMPLE 금지"(candidate-set=비인증)로 정의(더 단순·정확). J07 pack_trust_handshake 와 코드값 별개(이건 external require-certified 의미론).
> - 검증: `--test trust_policy`(5) + workspace 0 fail.


원천: E-shell-pack-catalog 감사. RequireCertified 는 파서 문법만 있고 sema-time admission 의미론 0.

## 현재 상태 (라이브)

- `sema/src/lib.rs:72-117` — RequireCertified 는 고정 단일-variant enum (`CertifiedOnly`/`FailClosed`) 의
  standalone meta-command. **admission/trust 의미론 없음**
- DoD 3대 fail-closed admission rejection 전부 부재:
  1. heuristic candidate basis 거부 — 없음
  2. uncertified absence 거부 — 없음
  3. profile-incompatible trust 거부 — 없음 (PROFILE coupling 이 어떤 assert 에도 없음)
- DoD-named `trust_policy.rs` / proof rail `--test trust_policy` 부재. 대체물 `pack_trust_handshake.rs:49-77` 은
  문법적 tail-clause 거부만 검증 (use_pack_missing_lockfile / unsupported_use_pack_clause / unsupported_require_meta)
- `rg "heuristic|uncertified.absence|candidate.basis"` sema = 무관한 주석 1건뿐

## 작업

1. sema admission 단계 신설 (F02 의 admissibility 모듈과 같은 layer):
   - REQUIRE certified 활성 시 heuristic-basis candidate 거부 (typed code)
   - uncertified absence 거부 (F02 의 `CertifiedAbsence` 와 연결)
   - 활성 PROFILE 과 trust 요구 불일치 거부
2. `sema/tests/trust_policy.rs` 신설 — 3대 거부 각각 negative 테스트로 고정, proof rail `--test trust_policy` 성립
3. 기존 `pack_trust_handshake.rs` 는 문법 레일로 유지하고 의미론 레일과 역할 분리 명시

## DoD

- 3대 fail-closed rejection 이 각각 typed error code 로 거부되고 테스트로 고정
- `cargo test -p quanta-gqlang-v2-sema --test trust_policy` green (DoD-named rail 복원)
- semantica S06/J07 측 require-certified handshake 와 코드명 정합 (J07 fixtures 는 이미 non-vacuous — 그쪽 코드와 동일 코드 사용)

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: `sema/src/lib.rs` 의 `TypedMetaCommandV1::RequireCertified{truth_requirement, admission_policy}`(이미 파싱됨 — **문법은 done, 의미론 0 이 gap**). PROFILE 선언의 `TypedProfileAdmissionPolicyV1`(model_pack.rs). CHECK 의 `CheckConfirmIntentV1{Skip,Prompt,Require}`. SAMPLE 의 `RetrievalModeIrV1{Lexical,Semantic,Hybrid}` 상응 sema 타입(sample_compare_export.rs). F02 의 `admit_document_v1` 패스에 합류(별도 채널 만들지 말 것).

### 생성 파일
1. **`sema/src/admissibility/trust_policy.rs`** (F02 모듈 안 — admission 계열 응집)
   - `pub fn enforce_trust_policy_v1(roots: &[TypedRootV1]) -> Result<(), SemaDiagnosticPageV1>` — 문서에 `RequireCertified{admission_policy: FailClosed}` meta 가 있으면 활성:
     - **reject ① heuristic candidate basis**: SAMPLE root 의 retrieval mode ∈ {Lexical, Hybrid} → `require_certified_rejects_heuristic_basis` (Semantic 만 inferred 로 통과? — no: RFC 상 certified-only 면 Semantic 도 candidate 일 뿐. 정확히는: SAMPLE 자체가 CandidateSet truth → require-certified 문서에서 SAMPLE 은 *reconfirm 동반 시만* 허용. 단순화 1차: Lexical/Hybrid 거부 + Semantic 은 통과(레졸루션 주석 명기).)
     - **reject ② uncertified absence**: CHECK root 의 `confirm != Require` → `require_certified_rejects_unconfirmed_check` (F02 `bounded_execution::admit_absence_claim_v1` 와 연동 — absence 류 주장은 certified 만).
     - **reject ③ profile-incompatible trust**: 문서 내 PROFILE 선언의 `TypedProfileAdmissionPolicyV1` 가 certified-only 보다 약하면 → `require_certified_profile_incompatible`.
   - 각 reject 는 typed `.code` + 어떤 root(index)가 위반인지 message 에 포함.

### 수정 파일 (배선)
- `sema/src/admissibility/mod.rs` 의 `admit_document_v1` → `trust_policy::enforce_trust_policy_v1(roots)?` (RequireCertified 부재 시 no-op — 기존 문서 전부 비파괴).
- semantica 측 J07(`pack_trust_handshake` — 이미 non-vacuous 4 tests)과 코드값 정합: reject code 문자열을 그쪽과 자구 일치 확인.

### 헬게이트 (red 먼저) — proof rail 이름 = DoD 의 `--test trust_policy`
- **`sema/tests/trust_policy.rs`**: ① `REQUIRE CERTIFIED` + `SAMPLE query=x ;; mode=lexical ...` → reject ① ② `REQUIRE CERTIFIED` + CHECK(confirm=skip) → reject ② ③ `REQUIRE CERTIFIED` + `PROFILE lax ;; admission=heuristic-allowed`(해당 variant 가 문법에 있으면; 없으면 admission=certified-only 아닌 형) → reject ③ ④ positive: `REQUIRE CERTIFIED` + semantic SAMPLE + CHECK(confirm=require) + certified-only PROFILE 전부 통과 ⑤ RequireCertified 없는 문서는 lexical SAMPLE 도 통과(게이트 비활성 증명 — 과차단 가드).

### 검증
`cargo test -p quanta-gqlang-v2-sema --test trust_policy && cargo test --workspace`. 적대 포인트: ⑤ 가 없으면 모든 기존 SAMPLE fixture 가 깨지는 과차단을 못 잡음 — 반드시 포함.
