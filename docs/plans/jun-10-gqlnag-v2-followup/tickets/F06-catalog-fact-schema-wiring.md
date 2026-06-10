# F06 — E11A public fact/relation schema 를 catalog root 에 배선 (W1)

> **STATUS: NOT STARTED (jun-10 재점검).** external W1. fact schema 가 SHOW/DESCRIBE catalog root 에 미배선 — 미착수.


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
