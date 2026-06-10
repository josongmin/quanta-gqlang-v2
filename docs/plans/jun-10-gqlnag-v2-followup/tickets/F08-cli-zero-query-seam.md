# F08 — E12A CLI zero-query/scaffold seam 신설 (W1)

원천: E-shell-pack-catalog 감사. scaffold 로직은 진짜지만 LSP 소유 — CLI seam 부재를
CLI-디렉터리 테스트가 가리고(masking) 있음.

## 현재 상태 (라이브)

- 진짜 로직: `lsp/src/handlers/starter_scaffold.rs`, `handlers/zero_query.rs` (LSP 소유)
- CLI 부재: `cli/src/main.rs:39-47` command enum 에 zero-query/scaffold verb 없음.
  `rg "zero_query|starter_scaffold" cli/src/` = EMPTY
- 가림 구조: `cli/tests/zero_query.rs:9` 가 `quanta_gqlang_v2_lsp::handlers::{starter_scaffold, zero_query}` 만
  import — `-p quanta-gqlang-v2-cli --test zero_query` green 이 CLI 에 대해 아무것도 증명 못 함
- J09A 측 named touch `cli/src/starter_scaffold.rs` 부재 (joint ticket line 22 미충족)

## 작업

1. zero-query/scaffold 공용 코어를 중립 위치로 추출 (lsp handler → ir 또는 공용 모듈;
   layering 규칙상 cli/lsp → ir 만 허용이므로 ir 측 또는 별도 공유 crate)
2. CLI verb 추가: `main.rs` command enum 에 zero-query(또는 scaffold) 진입 + `cli/src/` 측 owner 파일
3. `cli/tests/zero_query.rs` 를 CLI 진입점 경유로 재작성 (LSP import 제거) — vacuous-for-CLI 해소
4. J09A handshake 와 타겟명 정합 유지 (`zero_query_handshake.rs` 는 양쪽 다 존재 — 타겟명 OK)

## DoD

- CLI 에서 zero-query 실행 가능 (process-level smoke 포함)
- `cli/tests/zero_query.rs` 의 모든 assert 가 CLI 코드 경유
- LSP 측 기존 테스트 비파괴
