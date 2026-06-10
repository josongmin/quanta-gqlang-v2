# F07 — E10B REQUIRE certified: admission/trust 의미론 구현 (W1)

> **STATUS: NOT STARTED (jun-10 재점검).** external W1. `trust_policy.rs` + 3대 admission reject 부재 — 미착수.


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
