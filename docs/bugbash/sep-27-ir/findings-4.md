# sep-27 IR 감사 — findings-4 (Lane B: Surface / HIR)

- 원본 보고서: `docs/audits/lane-b-snapshot-20260927-010019/lane-B.md`
- 근거 snapshot: 동일 디렉터리 (HEAD `eaeb68ea2711d411616511713fb016d828cd80be` + dirty overlay, manifest 포함)
- 검증: `cargo test -p quanta-gqlang-v2-surface` → 260 passed / 0 failed

## 결론 요약

Lane B 범위에서 **현재 오동작하는 CONFIRMED_DEFECT(P0–P2)는 없다.**
아래는 반증을 거쳐 살아남은, 지금 수정 가치가 있는 실제 문제들만이다.
(사문·취향 후보인 formatter `%` precedence, 정당한 경계 검증 8건은 제외 — 원본 보고서 참고)

---

## F-B1. 키워드 어휘 4곳 수동 동기화 + 동기화 검사 rail 부재

- **심각도/분류**: P3 / STRUCTURAL_RISK (drift 시 실결함으로 승격 가능)
- **문제**: 같은 키워드 어휘 결정이 서로를 참조하지 않는 4개 리터럴 테이블으로 존재:
  - `crates/quanta-gqlang-v2-surface/src/query.rs:108` `is_reserved` (bare-alias 방어 + formatter `as` 선택이 동일 테이블 재사용)
  - `crates/quanta-gqlang-v2-surface/src/family.rs:46` `is_family_keyword`
  - `crates/quanta-gqlang-v2-surface/src/parser.rs:352` `is_v2_root_keyword`
  - `crates/quanta-gqlang-v2-surface/src/family.rs:133` `at_composition` lookahead {from, sample, find}
- **실제 문제인 이유**: 새 family 키워드를 추가할 때 `is_reserved`를 빠뜨리면 `from src <newkw>`의 bare-alias 포획과 canonical 포맷이 조용히 달라진다. 기존 테스트(v3_family_hellgate 등)는 family별 acceptance는 커버하지만 **is_reserved ↔ is_family_keyword 동기화 자체를 검사하는 rail은 없다** (확인됨).
- **현재 상태**: 오늘 코드에서 관측된 misparse는 없음 (formatter가 같은 `is_reserved`를 써서 round-trip 보존).
- **최소 fix**: 키워드 집합을 단일 상수 테이블로 통합하거나, 어휘 추가 시 두 테이블 대응을 검사하는 테이블 기반 rail 추가. owner: surface. compatibility 영향 없음.
- **regression oracle**: 신규 키워드 k에 대해 `from a k | project x`의 parse/format이 k 추가 전후 동일함을 검증.

## F-B2. escape 어휘가 lexer scan/decode에 이중 구현 — drift 시 내용 silent 소실

- **심각도/분류**: P3 / STRUCTURAL_RISK
- **문제**: `crates/quanta-gqlang-v2-surface/src/lexer.rs:198` (`scan_string` 허용 목록)과 `lexer.rs:287-309` (`decode_string` 매핑)가 같은 escape 어휘(`" \ n r t b f \u{}`)를 독립 유지.
- **실제 문제인 이유**: `scan_string`이 먼저 검증하므로 현재 decode의 관대한 분기(`_ => {}`)는 도달 불가하지만, scan에 escape를 추가하고 decode를 빠뜨리면 해당 escape 문자가 **문자열 내용에서 조용히 사라진다** — 정확히 금지된 silent-drop 형태(D10 위반 경로).
- **현재 상태**: 양쪽 목록 일치 확인. 관측된 소실 없음.
- **최소 fix**: 허용 escape 집합을 단일 const로 추출해 양쪽이 참조. owner: surface/lexer.
- **regression oracle**: 미지원 escape(`"\x41"`) 거부 + 지원 escape 전건의 lex→decode→format(`escape_str`) 3자 roundtrip 일치 검사.

## F-B3. `let x = from / sample / find` decl이 표현 불가 (미문서화 제한)

- **심각도/분류**: P3 / PROOF_GAP
- **문제**: `family.rs:133`의 고정 오프셋-3 lookahead가 `let <n> = <producer-kw>`를 composition으로 오디스패치하여, body가 해당 이름 bare symbol인 평범한 decl이 파스 불가. `parse_value`는 `Ident("from")`을 symbol로 허용하므로 문법적으로는 유효 형태다.
- **왜 결함이 아닌가**: 실패가 success-shaped가 아니라 typed reject이고, parser가 해당 HIR을 생산하지 않으므로 parser-produced 경로 무영향. caller-built HIR의 경우 `try_format_item`이 ParseRejected로 거부(silent loss 아님, 확인됨).
- **실제 문제인 이유**: 이 제한이 어디에도 문서화되어 있지 않아 의도인지 우연인지 구분 불가.
- **최소 fix**: family.rs doc에 제한 명시, 또는 lookahead 정밀화. owner: surface.

## F-B4. hir 노드 무결성이 옵트인 체크에 의존 — pipeline 우회 경로 미확인 (Lane E 확인 필요)

- **심각도/분류**: P3 / STRUCTURAL_RISK (소유권: 해소는 Lane E)
- **문제**: `hir/src/expr.rs:21-36`의 pub 필드 + pub `new`, `hir/src/id.rs:30`의 `from_raw`, `hir/src/span.rs:319`의 비검증 `SpanV1::new`로 caller-built HIR이 임의 id/span을 mint 가능. 무결성은 `check_tree`/`check_document_v2` checked constructor가 옵트인으로 담당(`integrity.rs:263` "Who must call this"로 계약 문서화됨).
- **확인된 사실**: production closed path는 `sema/src/document.rs:66`이 `CheckedHirDocumentOwnedV2::try_new`으로 문서 단위 봉인 — 이 경로는 안전.
- **남은 실제 질문**: pipeline/CLI가 sema를 우회해 raw HIR을 직접 소비하는 경로가 있는지. 존재하면 unchecked HIR이 downstream에 도달할 수 있다. Lane B 범위 밖이므로 **Lane E 재확인 항목으로 승계**.

---

## 제외된 항목 (참고)

- B-F5(formatter `%` precedence 사문): 실문제 아님.
- 반증 완료 8건(surface↔HIR 이중 carrier 부재, `try_format_item` 경계 재검증 정당성, equivalence 비순환성, `paren_opens_bool` 비용 봉인 등): `docs/audits/lane-b-snapshot-20260927-010019/lane-B.md` §4.

*작성 경로 참고: 요청 표기 `docs/bubgash/...`는 기존 디렉터리 표기 `docs/bugbash/`의 오타로 판단해 `docs/bugbash/sep-27-ir/findings-4.md`에 작성함.*
