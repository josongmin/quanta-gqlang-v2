# F06 — E11A public fact/relation schema 를 catalog root 에 배선 (W1)

> **STATUS: DONE (jun-10) — proof rails `ir --test fact_catalog_registry`(4), `cli --test fact_schema_describe`(2), `lsp --test fact_schema_describe`(2) GREEN. workspace 0 regression, clippy `--lib/bins/examples` + `--tests` 0 warn.**
> - 신규 `ir/src/catalog/fact_catalog.rs` — catalog-facing 단일 registry. canonical 인스턴스(`catalog.deployments`/`catalog.artifacts` 엔티티, `catalog.deployment_owns_artifact` 릴레이션 — fact_model_handshake fixture 와 이름 일치). `entity_schema_for_v1`/`relation_schema_for_v1`/`describe_schema_rows_v1`(`<kind>::<name>` prefix 정규화)/`field_type_label_v1`/`published_entity_names_v1`.
> - **소비자 #1 (CLI)**: `cli/src/commands/query.rs` DESCRIBE summary → `describe.schema.field[N]=name:type` 라인. **소비자 #2 (LSP)**: `lsp/.../read_only_analysis/summary.rs` DESCRIBE arm → hover card `schema.field=name:type`. → 섬 탈출 증명(`rg PublicEntitySchemaV1 cli/src lsp/src`는 간접이지만 `describe_schema_rows_v1` 직접 hit ≥2).
> - **fail-closed**: 미지 target 은 필드 invent 안 함 — bare `target=` 유지(테스트로 고정). registry integrity test 가 모든 published schema `validate_v1`-clean 강제(런타임 아닌 빌드시 실패).
> - 설계: DESCRIBE 문법에 entity/relation kind 부재(capability/model/view/rule/pack/profile/gate) → entity 는 `model::<name>` 로 describe(provenance 가 `model::` 라 정합). relation 은 문법상 DESCRIBE 도달 불가 → registry 직접 API + IR 테스트가 소비자.
> - 이름 드리프트(EntitySchemaV1 vs Public*): 코드 rename 대신 본문 표기 정정(아래 §현재상태 기록 유지).


원천: E-shell-pack-catalog 감사. "root-referencing 은 affirmatively REFUTED" — schema 타입이
정의만 있고 catalog-facing root 가 하나도 참조 안 함.

## 현재 상태 (라이브)

- `ir/src/facts/entity_schema.rs` (`PublicEntitySchemaV1` 등), `relation_schema.rs:88`
  (`PublicRelationSchemaV1`) 정의 존재
- 참조자는 IR 테스트 3개뿐 (fact_model_handshake / fact_schema_rowset_guard / public_fact_schema).
  **syntax/sema/cli/lsp src 에서 0 hits** — defined-but-unreferenced IR island
- E11A Files-To-Touch 가 명시한 `syntax/src/interactive/describe.rs` 부재 (show.rs 에 통합돼 있음)
- 이름 드리프트: DoD 는 `EntitySchemaV1`/`RelationSchemaV1`, 라이브는 `Public*` 접두

## 작업

1. SHOW/DESCRIBE lowering 경로가 schema 타입을 실제 소비하도록 배선:
   - DESCRIBE <entity/relation> 결과 페이로드를 `PublicEntitySchemaV1`/`PublicRelationSchemaV1` 기반으로 구성
   - LSP completion/quickfix 의 catalog surface 와 연결 (completion.rs 가 이미 ir catalog 사용 — 같은 패턴)
2. `describe.rs` 분리 여부 결정: 분리 비용이 낮으면 Files-To-Touch 대로 분리, 아니면 ticket body 를 show.rs 통합으로 개정
3. 이름 드리프트 해소: ticket body 를 `Public*` 표기로 정정 (코드 rename 보다 문서 정정 권장)
4. catalog-root 배선을 검증하는 non-IR 테스트 추가 (sema 또는 lsp 측)

## DoD

- `rg "PublicEntitySchemaV1|PublicRelationSchemaV1" crates/quanta-gqlang-v2-{syntax,sema,cli,lsp}/src` ≥ 1 hit (배선 증거)
- DESCRIBE 경로 round-trip 테스트가 schema 필드를 실제 assert
- E11A DoD "referenced by catalog-facing roots" 충족

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: 스키마 타입은 `ir/src/facts/{entity_schema,relation_schema}.rs`(PublicEntitySchemaV1 등) — IR 테스트만 참조하는 섬. DESCRIBE 경로는 `sema/src/interactive/show_describe.rs`(`TypedDescribeTargetKindV1:74` — Rule 등) → ir Interactive lowering. **레이어 제약: 스키마는 ir 소유, sema 는 ir 에 의존 불가** → 배선은 ir/lsp/cli 층에서.

### 생성/수정
1. **`ir/src/catalog/fact_catalog.rs`** (신규 — catalog-facing 단일 registry)
   - `pub fn entity_schema_for_v1(entity: &str) -> Option<&'static PublicEntitySchemaV1>` / `pub fn relation_schema_for_v1(rel: &str) -> Option<&'static PublicRelationSchemaV1>` — facts/ 의 canonical 인스턴스들을 `LazyLock`/const 로 등록 (Files/Symbols/Definitions/References/Calls/… 기존 fact_model_handshake fixture 의 엔티티명과 일치).
   - `pub fn describe_schema_rows_v1(target: &str) -> Option<Vec<(String, String)>>` — DESCRIBE 렌더용 (field name, field type) 행 추출.
2. **LSP 배선** — `lsp/src/handlers/` 의 describe/completion 핸들러(완성: `completion.rs` 가 이미 ir catalog 소비하는 패턴)에서 DESCRIBE 대상이 entity/relation 이면 `describe_schema_rows_v1` 로 스키마 필드 렌더 → **non-IR-test 소비자 #1**.
3. **CLI 배선** — `cli/src/commands/query.rs` summary: DESCRIBE root 일 때 `describe.schema.field[N]=<name>:<type>` 라인 추가 → **소비자 #2** (E11A DoD "referenced by catalog-facing roots" 충족).
4. **이름 드리프트 처리**: 코드 rename 대신 E11A 티켓 body 의 `EntitySchemaV1`→`PublicEntitySchemaV1` 표기 정정 주석 (이미 F06 본문에 권장 기록됨).

### 헬게이트 (red 먼저)
- **`lsp/tests/fact_schema_describe.rs`**(신규): DESCRIBE entity 응답에 스키마 필드 존재 / 미지 entity → 스키마 없음 graceful(fail-closed 메시지) / relation 케이스 1개.
- **`cli/tests/cli_headless_read.rs` 확장**: DESCRIBE query summary 에 `describe.schema.field` 라인 assert.
- 적대 가드: `rg "PublicEntitySchemaV1|PublicRelationSchemaV1" cli/src lsp/src` ≥1 (섬 탈출 증명 — F06 DoD 그대로).

### 검증
`cargo test -p quanta-gqlang-v2-lsp --test fact_schema_describe && cargo test --workspace`.
