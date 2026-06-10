# F20 — J08A/J08B/J09A/J09B/J09C mirror-tautology fixture 의 behavioral 승격 (W5, joint)

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
