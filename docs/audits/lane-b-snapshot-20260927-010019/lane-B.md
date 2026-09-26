# Lane B — Surface / HIR 구조·무결성 감사 보고서

## 0. 실행 경위 (coordinator 기록)

- 병렬 subagent 위임 2회(B1: parser/구조, B2: format/round-trip)를 시도했으나 두 subagent 모두 결과 파일 없이 반복 실패하여, coordinator가 동일 snapshot에서 두 각도의 조사를 직접 수행하고 본 보고서로 통합했다. 감사 방법(정적 조사 + owner 테스트 실행)은 동일하게 유지됐다.

## 1. Snapshot 경계 / coverage

- **Snapshot ID**: `docs/audits/lane-b-snapshot-20260927-010019` (workspace 내 불변 사본)
- **원 HEAD**: `eaeb68ea2711d411616511713fb016d828cd80be` (branch `main`)
- **Dirty 경계**: HEAD 추적 파일 전체 + working-tree dirty 파일 overlay. Lane B 관련 dirty: `surface/src/format.rs`, `surface/src/lib.rs`, `surface/tests/v3_format_hellgate.rs`, `surface/tests/v3_surface_budget.rs`, 신규 `pipeline/tests/v3_format_budget.rs`. snapshot 내 hash가 원본과 일치함을 확인 (format.rs `260341a8…`, lib.rs `4f41e6c5…`, v3_format_budget.rs `4298d504…` MATCH). symlink 없음. manifest: `MANIFEST-laneB.sha256` (surface/hir 67파일).
- **조사 파일**: `surface/src/{lexer,parser,expr,query,unit,shape,family(1-230),format,equivalence,cursor,budget,depth,lib}.rs`, `hir/src/{id,build,expr(1-120),span,depth,visit,integrity,item(참조경로),lib(상수)}.rs`, surface 테스트 17파일 목록 및 `v3_format_hellgate.rs` 케이스 목록.
- **미조사 범위**: `family.rs` 231-1590 (FIND/CHECK/presentation/catalog 세부 clause 파서 — hellgate 테스트가 1304줄로 커버; 정밀 재감사는 별도 필요), `hir/item.rs`·`relation.rs` 전문, `diag.rs` 전문, 테스트 본문 전부. "중복 0개" 주장이 아님 — 조사 경계 내 결론.

## 2. Owner / 경계 표

| 표현 | owner | 생성 | 검증 | 소비 | 실패 동작 |
|---|---|---|---|---|---|
| Token stream | surface/lexer | `lex_with_budget` | byte-exact span, fail-closed | cursor | typed `LexErrorV1` (=ParseDiagnosticV1) |
| ValueExpr/BoolExpr/RelationExpr/ResultShape HIR | hir crate | surface parser → `HirBuilderV1` (single id-gen per document) | `check_item`/`check_depths_v2` at parse; `check_tree`/`check_document_v2` opt-in | sema 이하 전체 | typed ParseDiagnostic / HirIntegrityErrorV1 |
| 문서 node-ID uniqueness | hir/integrity + build | builder가 monotonic mint | `IntegrityCollector.record` (HashMap seen) | downstream side-table keys | `hir.integrity.duplicate_node_id` |
| Span 무결성 (start≤end, end≤source_len) | hir/integrity | parser가 token/merge span 복사 | `record`/`record_auxiliary_span` | diagnostics, formatter | typed inverted/oob error |
| canonical v3 text | surface/format | `format_document`/`format_item` (infallible, parser-produced 전용) / `try_format_item` (checked, raw HIR) | `try_format_item`: representability + reparse + 고정점 + semantic equivalence | CLI fmt, pipeline | FormatDiagnosticV1 (7개 stable code) |
| surface equivalence oracle | surface/equivalence | — (비교기) | stack-safe, ids/spans/paren 무시 payload 완전비교 | `try_format_item`, 테스트 | bool |
| parse budget | surface/budget | `ParseBudgetV2` (양수, depth 상한 ≤ HIR safe max) | tokens/items/nodes/depth/width per-charge | 모든 parse 진입점 | `ResourceExhausted` typed |
| depth 안전 | hir/depth (Frame 스택) + surface 재진입 guard | — | `check_depths_v2`, `with_recursion_depth` | integrity gate 선행 | `hir.resource_limit_exceeded` |

생산 경로 검증: `parse_source_document_v2` → `parse_document_items_with_budget` → 공유 builder → `HirDocumentV1`; `sema/src/document.rs:66`이 `CheckedHirDocumentOwnedV2::try_new`로 문서 단위 무결성을 1회 강제 (production closed path. pipeline 직접 경로는 Lane E 확인 필요 → handoff).

## 3. Findings

### B-F1 — 키워드 어휘의 3중 수동 동기화 (P3, STRUCTURAL_RISK)
- **위반 기준**: repo invariant "schema 추가 시 manual sync 경계 최소화" (구조적 위험, 현재 오동작 아님).
- **위치**: `surface/src/query.rs:108-133` (`is_reserved`), `surface/src/family.rs:46-66` (`is_family_keyword`), `surface/src/parser.rs:352-357` (`is_v2_root_keyword`), `family.rs:133-141` (`at_composition` lookahead {from,sample,find}).
- **증거**: 네 개의 리터럴 키워드 목록이 서로를 참조하지 않고 병렬 유지된다. bare-alias 포획(`parse_bare_alias`, query.rs:91-105)은 `is_reserved`만 방어하므로, 신규 family 키워드 추가 시 `is_reserved`에 빠뜨리면 `from src <newkw>`의 alias 포획·렌더링 규칙이 조용히 달라질 수 있다.
- **현재 발생 조건**: 오늘 코드에서 관측된 misparse 없음 — bare-alias로 잡힌 family 키워드(`sample`, `find` 등)도 format이 동일 테이블로 재직렬화하므로 round-trip이 보존된다(`format.rs:366-371`이 동일 `is_reserved` 사용 → 동기화된 쌍). drift 가능성만 확인 → STRUCTURAL_RISK.
- **최소 fix**: `is_reserved`가 family/v2 어휘를 포함하는 단일 상수 테이블을 참조하게 하거나, 어휘 추가 시 두 테이블을 함께 검사하는 테스트 rail. owning crate: surface. compatibility 없음.
- **regression oracle**: 신규 키워드 k에 대해 `from a k | project x` 파싱/포맷이 k 추가 전후 동일함을 검증하는 테이블 기반 rail.
- **반증 시도**: v3_family_hellgate(1304줄)/v3_mutation_rails가 어휘 drift를 잡는 rail인지 확인 — family별 acceptance는 커버하나 **is_reserved↔is_family_keyword 동기화**를 직접 검사하는 rail은 발견 못 함.

### B-F2 — `at_composition` 고정 오프셋 lookahead가 특정 decl을 표현 불가능하게 만듦 (P3, PROOF_GAP)
- **위치**: `surface/src/family.rs:133-141`, `surface/src/parser.rs:303-304`.
- **증거 (정적)**: `let x = from` / `let x = sample` / `let x = find`는 token offset 3의 producer 키워드 lookahead가 composition으로 오디스패치하여, body가 해당 이름의 bare symbol인 decl로서 파스 불가(`parse_value`는 `Ident("from")`을 symbol로 허용하므로 문법적으로는 유효 형태). parser가 이 HIR를 절대 생산하지 않으므로 parser-produced 경로에는 무영향; caller-built HIR `let x = from`은 formatter가 렌더링한 텍스트가 재파스 거부 → `try_format_item`이 ParseRejected로 거부 (silent loss 아님, 확인: format.rs:169-188).
- **분류 근거**: 실패가 success-shaped가 아니라 typed reject이므로 defect가 아니라 표현력 제한. 문서화된 제한인지 확인 불가 → PROOF_GAP.
- **최소 fix**: 제한을 family.rs doc에 명시하거나, lookahead를 `let` 뒤 형태로 정밀화. owning: surface.
- **반증 시도**: hellgate에서 이 형태를 다루는 케이스 없음 확인; roundtrip 전 경로가 unaffected임은 parser 생산 불가로 증명.

### B-F3 — escape 어휘가 scan과 decode에 이중 구현 (P3, STRUCTURAL_RISK)
- **위치**: `surface/src/lexer.rs:198` (scan_string 허용 목록 `" \ n r t b f u{}`) vs `lexer.rs:287-309` (decode_string 매핑).
- **증거**: 두 함수가 같은 escape 어휘를 독립 유지. scan이 먼저 검증하므로 decode의 관대한 분기(`_ => {}`, `if let Ok`)는 lex 경로에서 도달 불가 — 현재 silent drop 경로 없음. 그러나 scan에 escape를 추가하고 decode를 빠뜨리면 해당 escape가 **내용에서 조용히 소실**된다(정확히 D10 금지 형태).
- **drift 관측**: 없음 (양쪽 목록 일치 확인). 구조적 위험으로만 분류.
- **최소 fix**: 허용 escape 집합을 단일 const로 뽑아 양쪽이 참조. owning: surface/lexer.
- **regression oracle**: `"\x61"` 등 미지원 escape 거부 + 지원 escape 전건 roundtrip(`escape_str` format.rs:715-733 포함 3자 일치)를 검사하는 rail.

### B-F4 — hir 노드의 pub 필드/생성자로 무결성이 옵트인 체크에 의존 (P3, STRUCTURAL_RISK, handoff Lane E)
- **위치**: `hir/src/expr.rs:21-36` (`ValueExprHirV1` pub `id/span/kind` + pub `new`), `hir/src/id.rs:30` (`HirNodeIdV1::from_raw`), `hir/src/span.rs:319` (`SpanV1::new` 비검증).
- **증거**: caller-built HIR이 임의 id/span을 mint 가능. 완화: `NonEmptyPathSegmentsV1`/`CallArgsHirV1`/`NonEmptyBindersV1` 등 일부 불변식은 타입으로 봉인(expr.rs:104-118 등), 나머지는 `check_tree`/`check_document_v2` checked constructor가 담당하며 `integrity.rs:263-269` "Who must call this"로 계약 명시. production 경로는 sema가 `CheckedHirDocumentOwnedV2::try_new`(sema/document.rs:66)을 호출해 봉인함.
- **잔여 위험**: pipeline/CLI가 sema를 우회하는 직접 HIR 경로가 있는지 — Lane E 소유 (여기서는 경로 미확인, SOURCE 범위 밖).
- **최소 fix**: 불필요 (계약이 문서화됨). compile-fail privacy probe가 이미지 존재 여부만 Lane E에서 확인 권고.
- **반증 시도**: fuzz 대상(`fuzz/fuzz_targets/surface_document.rs:31`)이 parse→check_document_v2를 검증; hir hellgate가 duplicate/oob/depth 거부를 직접 pin (v3_hir_hellgate.rs 등).

### B-F5 — formatter의 `%` precedence가 lexer가 생산 불가한 연산자를 포함 (P3, OPTIONAL_REFACTOR)
- **위치**: `surface/src/format.rs:577-580` (`"*" | "/" | "%" => 20`), `surface/src/lexer.rs:376-395` (`%` 부재).
- **증거**: open operator tag 설계상 caller-built HIR의 `%`를 배려한 분기. 오동작 없음, 비대칭 사문. 조치 불필요 (기록용).

## 4. 반증된 후보 (정당한 구조)

1. **Surface↔HIR 이중 carrier 부재**: surface는 자체 expr AST 없이 `hir::ValueExprHirV1` 등을 직접 생산 (lib.rs:5-6, "depends on hir ONLY"). `SurfaceItemV1`의 Query/Decl/Rule/Program 4변체는 parser 반환 경계의 편의 래퍼이고 `to_program_item`/`into_program_item`이 단일 lift 소유 (parser.rs:50-77) — 중복 결정 아님.
2. **`try_format_item`의 reparse**: 경계 재검증이 아니라 **raw HIR 신뢰경계**의 검증 (caller-built 입력, format.rs:164-188). 정당.
3. **`format_source_document_with_parse_budget_v2`의 문서 전체 span 재앵커링** (lib.rs:165-171): 생성 텍스트 오프셋의 원본 귀속 오류 방지 정책, 문서화됨. 정당.
4. **equivalence가 구현 복사 oracle인지**: `semantically_equivalent_items`는 canonical 텍스트와 독립적 필드 완전비교를 명시적으로 설계 (equivalence.rs:79-82), paren/id/span 제외는 문서화된 판단. deep-tree stack-safety + payload 구분 rail 포함 (equivalence.rs:565-581). 원형성 없음.
5. **`paren_opens_bool` 재스캔 비용**: 최악 O(depth²)이나 expression depth 상한 64 (`hir/src/lib.rs:54`)로 봉인 — 무시 가능.
6. **relation depth가 query 단위 복원** (query.rs:74): per-path depth 의미에 부합, 문서 전체는 `document_items` 예산으로 봉인.
7. **지수 표기 float(`1e5`) 미지원**: `scan_number`이 digits+fraction만 허용 → `1e5`는 Int(1)+Ident(e5)로 fail-closed reject. silent 성공 없음 — 문법적 제한, 결함 아님. raw 텍스트 보존으로 `-0`, 선행 0, big-int 정확 보존 확인 (format.rs:687 `raw.clone()`).
8. **Record/shape/composition 중복 필드거부**: expr.rs:565-567, shape.rs:45-49, family.rs:159-168 — 각 owner에서 독립 소유, 동일 정책의 3곳 반복이나 입력·컨테이너가 다른 정당한 경계 검증 (동일 코드 재사용은 선택적 개선).

## 5. 실행한 checks / 한계 / handoff

- **실행**: `cargo test -p quanta-gqlang-v2-surface --target-dir ./target-laneB` (snapshot 내, 격리 target) — **260 passed / 0 failed**, exit 0. (문서화된 deep-child 프로토콜 포함; 상세는 로그를 재실행으로 확인 가능)
- **미실행**: `just ci`(repo-wide, coordinator 판단), family.rs 후반부 정밀 정오 비교, hir crate 단독 테스트(무결성 rail은 surface/hir hellgate가 참조하는 것으로 간접 확인).
- **정적 전용 결론**: B-F2 (misdispatch 증명은 코드 경로 추적).
- **Lane E handoff**: (1) pipeline/CLI가 `CheckedHirDocumentOwnedV2`(sema 경유) 외에 raw HIR를 소비하는 경로 존재 여부 (B-F4 잔여), (2) `format_source_document_with_parse_budget_v2`의 CLI 노출과 예산 프로파일.
- **Lane C handoff**: raw literal 텍스트의 하위 파싱(i128 최솟값, finite float)은 types 소유 — surface는 보존만 확인.

## 6. 평가

조사 경계 내에서 **confirmed defect(P0-P2) 없음** — `증거 범위 내 일관됨`. 생존 finding 5건 모두 P3 (STRUCTURAL_RISK 3, PROOF_GAP 1, OPTIONAL_REFACTOR 1). 구조적으로 눈에 띄는 강점: 단일 shared HIR carrier, 문서 단일 builder에 의한 by-construction ID 유일성, fail-closed 원칙의 철저한 적용(v2 reject, 중복 필드, 빈 statement, trailing tokens, escape 검증), typed checked-constructor 패턴, stack-safe traversal의 일관된 이터레이티브 구현.
