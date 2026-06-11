# F08 — E12A CLI zero-query/scaffold seam 신설 (W1)

> **STATUS: DONE (jun-10) — proof rails `cli --test zero_query`(3), `cli --test zero_query_handshake`(2), `ir --test zero_query_scaffold`(4) GREEN. workspace 100 bin ok, clippy gate+tests 0.**
> - **단계1 (코어 ir 이동, 무손실)**: `lsp/handlers/{starter_scaffold,zero_query}.rs` → `ir/src/catalog/{starter_scaffold,zero_query_surface}.rs` 로 데이터+로직 전부 이동. LSP 핸들러는 `pub use ...::*` thin re-export 化 → server.rs 디스패치/타입 경로 무변경. **기존 LSP suite 22 bin 무변경 green = 무손실 증명.** `zero_query_surface` 에 canonical `build_zero_query_response_v1()` 추가(+ `handle_zero_query()` alias 유지).
> - **단계2 (CLI verb)**: `cli/src/commands/zero_query.rs::run()` = ir 빌더→render. `main.rs` `CliCommandV1::ZeroQuery` + `"zero-query"` parse + `is_nullary_v1()`(입력 비요구·stray arg→usage 2) + 디스패치 + USAGE 갱신. `commands/mod.rs` 등록.
> - **단계3 (vacuous-for-CLI 해소)**: `cli/tests/zero_query.rs`/`zero_query_handshake.rs` 재작성 — **LSP import 0**(rg 검증), CLI seam(`commands::zero_query::run` + 바이너리 smoke) 경유. 헬게이트핵심: 모든 제안 쿼리가 `parse::run` 으로 재파싱(fail-closed 문법 보증). 깊은 scaffold 구조 검증은 적합 레이어(ir test)로 이동.
> - **doc-gate 정합**: `CLI_COMMAND_TOKENS_V1`(7→8, +zero-query), `docs/language/README.md` CLI 목록, usage-contract 2개(cli_help_usage_contract/cli_empty_input_frontdoor) 모두 갱신 — command_doc_gate green.
> - 부수: F07 wildcard 잔재 clippy 정리(admissibility.rs guarded-match) + admissibility_matrix doc-list 경고 정리.


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

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: 로직 전부 LSP 소유 — `lsp/src/handlers/zero_query.rs`(`ZeroQuerySuggestionV1/GroupV1/ResponseV1:4-21`, `render_v1:29`, `handle_zero_query:73`), `lsp/src/handlers/starter_scaffold.rs`(`StarterSuggestionV1:136` 등 — 이미 **ir capability catalog + example registry 소비**). CLI verb 표면: `cli/src/main.rs:28 CliCommandV1` enum + `:41-47` parse + `commands/mod.rs` 디스패치. 레이어 규칙: cli/lsp→ir.

### 단계 1 — 코어를 ir 로 추출 (수평 이동, 무손실)
1. **`ir/src/catalog/starter_scaffold.rs`** (신규): LSP `starter_scaffold.rs` 의 **데이터+로직 전부 이동**(StarterGroupKindV1/StarterRootKindV1/StarterBehaviorV1/StarterReasonCodeV1/StarterReasonV1/StarterSuggestionV1/StarterGroupV1 + 빌더 fn) — LSP 프로토콜 의존이 없으므로 그대로 옮겨짐. `catalog/mod.rs` 에 mod+re-export.
2. **`ir/src/catalog/zero_query_surface.rs`** (신규): `ZeroQuerySuggestionV1/GroupV1/ResponseV1` + `render_v1` + `build_zero_query_response_v1()`(구 handle_zero_query 의 순수부) 이동.
3. **LSP thin wrapper 화**: `lsp/handlers/{zero_query,starter_scaffold}.rs` → `pub use quanta_gqlang_v2_ir::catalog::...` 재수출 + `handle_zero_query()` 는 ir 빌더 호출만. **기존 LSP 테스트(zero_query_handshake/autocomplete 류) 무변경 green 이 이동 무손실 증명.**

### 단계 2 — CLI verb 신설
4. **`cli/src/commands/zero_query.rs`** (신규): `pub fn run() -> Result<String, ...>` = ir `build_zero_query_response_v1()` → `render_v1()` 텍스트.
5. **`cli/src/main.rs`**: `CliCommandV1::ZeroQuery` variant + `"zero-query" => Ok(Self::ZeroQuery)`(:41-47 블록) + 디스패치 + usage 문자열 갱신. `commands/mod.rs` 등록.

### 단계 3 — vacuous-for-CLI 해소
6. **`cli/tests/zero_query.rs` 재작성**: LSP import 제거 → `quanta_gqlang_v2_cli` 경유(또는 binary smoke: 기존 examples_binary_frontdoor 패턴 재사용) 로 zero-query 출력 assert. **`rg "quanta_gqlang_v2_lsp" cli/tests/zero_query.rs` = 0** 이 DoD.
7. J09A 정합: `cli/tests/zero_query_handshake.rs`(기존) 타겟명 유지 — 내용도 CLI 경유로 같이 전환.

### 헬게이트 (red 먼저)
- 신설 CLI 테스트: ① zero-query 출력에 starter group ≥1 + suggestion ≥1 ② 출력의 각 suggestion 쿼리는 `parse_document_v1` 로 재파싱 가능(제안이 항상 valid 문법이라는 fail-closed 보증 — 헬게이트 핵심) ③ usage: 인자 잘못 주면 exit 2.
- 적대 가드: `rg "zero_query|starter_scaffold" cli/src/` ≥1 && `rg "handlers::starter_scaffold" lsp/src` 가 재수출만.

### 검증
`cargo test -p quanta-gqlang-v2-cli --test zero_query --test zero_query_handshake && cargo test -p quanta-gqlang-v2-lsp && cargo test --workspace` (LSP 기존 테스트 무변경 green = 이동 무손실).
