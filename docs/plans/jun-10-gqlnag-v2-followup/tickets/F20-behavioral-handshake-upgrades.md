# F20 — J08A/J08B/J09A/J09B/J09C mirror-tautology fixture 의 behavioral 승격 (W5, joint)

> **STATUS: F20 FULLY DONE (jun-11) — external(4 fixtures) + sema(J08A/J08B/J09A/J09B/J09C 5 fixtures) all behavioral, green, clippy-clean.**
> sema exec-conformance behavioral 승격(constant-mirror → 실 host 호출+typed neg): `fact_model_handshake`(EntityIndexHost, 4), `autocomplete_handshake`(IdeReadHost.completion suggestion-only, 4), `catalog_autocomplete_handshake`(CapabilityHost, 3), `zero_query_handshake`(Capability+EntityIndex, 4), `explain_path_handshake`(Explain/PathHost, 5), `show_describe_handshake`(CapabilityHost, 6 신설). 각 fixture 실 trait 메서드 호출 + fail-closed reject를 typed `.code`로 검증(vocab을 behavior에 결바). conformance 20 bin green(무관 선행 gate_owner_cutover_guard 1건 제외). J09C ExplainHost/PathHost production impl은 여전히 0(그건 ingress 구현 시) — 본 작업은 conformance 품질 상향.
> (구) **STATUS: J09C sema DONE (`explain_path_handshake` constant-mirror→behavioral, 실 ExplainHost/PathHost 호출+typed neg×4, 5 green) + EXTERNAL FIXTURE DONE (jun-10) — 이 레포(quanta-gqlang-v2) fixture 4/5 behavioral. 잔여: J09C ExplainHost/PathHost impl(semantica exec-host-spi).** workspace 103 bin ok, clippy tests 0.
> - **재점검 정정**: F20 의 "라이브" 품질감사가 일부 stale — J08A `ir/tests/fact_model_handshake.rs` 는 이미 2 test/2 typed-code negative(`entity_field_name_runtime_local`/`fact_schema_source_ref_runtime_local`)+실 `validate_v1` 호출로 DoD 충족 상태였음(승격 불요).
> - **J08B `lsp/tests/catalog_autocomplete_handshake.rs`**: constant-mirror → behavioral 승격. 소유자 pin 은 유지하되 **partition invariant**(catalog≠example owner)로 강화 + **fail-closed negative ×2**(`capability_row_by_id_v1`/`example_row_by_id_v1` 미지 id→None) + 모든 autocomplete row 실 resolution→typed provenance assert. (3 tests)
> - **J09B `lsp/tests/autocomplete_handshake.rs`**: 실 handler 호출(`handle_completion`/`handle_quickfix`)은 기존부터 behavioral. negative 1→2 강화(`describe_target_invalid_shape` + `unsupported_describe_kind` — 거부가 decode 임을 증명). (3 tests)
> - **J09A `cli/tests/zero_query_handshake.rs`**: F08 에서 이미 CLI seam(`zero_query::run`) 경유 behavioral 로 재작성 — group partition + non-mutating fail-closed 속성 assert(constant-mirror 아님). (2 tests)
> - **잔여(semantica)**: J09C — `exec-host-spi`의 ExplainHost/PathHost 는 workspace impl 0. Dummy impl + explain()/find_path() 실호출 + fail-closed 경로 실행은 semantica 워크트리 작업(이 레포 범위 밖). external 측 `cli/tests/explain_path_handshake.rs`(F19 신설)는 front-door `explain::run` 경유로 이미 behavioral.


원천: J-caller 감사. handshake fixture 품질이 3계층으로 갈림 — 상위 3개만 behavioral,
하위 5개는 constant-mirror tautology (상수 리스트를 자기 자신과 비교; 행동 회귀 탐지 불능).

## 품질 계층 (라이브)

- **non-vacuous (기준 모델로 사용)**:
  - J05A `ide_read_handshake.rs:106-155` — read/mutate 분할 invariant (reject∩allow=∅, reject≠∅)
  - J05B `command_bridge_contract.rs:124-182` — 7 tests, typed-code negative 13개
  - J07 `pack_trust_handshake.rs` — 4 tests, negative 8개
- **mirror-only (승격 대상)**:
  - J08A `fact_model_handshake.rs` — 1 test, 0 negative, CONSTANT.contains(hardcoded)
  - J08B `catalog_autocomplete_handshake.rs:17-18,39-42` — 자기 설정 필드 assert + 상수 mirror
  - J09A `zero_query_handshake.rs` — 1 test mirror
  - J09B `autocomplete_handshake.rs` — 1 test mirror
  - J09C `explain_path_handshake.rs:31-35,66-68` — 상수 mirror. **추가로 ExplainHost/PathHost 는
    workspace 전체 impl 0** (`exec-host-spi/src/explain.rs:22`, path.rs) — 테스트가 explain()/find_path() 를
    한 번도 호출 안 함. fail-closed reject 주장이 미실행

## 작업

1. 각 mirror fixture 를 J05A/J05B 패턴으로 승격:
   - 실제 host trait 의 Dummy impl 을 만들고 **메서드를 호출** — 결과/거부를 typed code 로 assert
   - reject-set 은 invariant (분할/비공집합/타입정합) 로 검증 — 상수 복사 비교 금지
   - negative 케이스 ≥2 per fixture (잘못된 입력 → typed error)
2. J09C: ExplainHost/PathHost Dummy impl + explain()/find_path() 실호출 + fail-closed 경로 실행
3. F11 의 8-trait conformance 와 중복 작업 회피 — analysis trait Dummy 는 한 곳에서 공유

## DoD

- 5개 fixture 전부 ≥2 tests / ≥2 negatives / trait 메서드 실호출
- "would only catch an edit to the constant list" 류의 mirror assert 0
- ExplainHost/PathHost impl ≥1 (workspace)
