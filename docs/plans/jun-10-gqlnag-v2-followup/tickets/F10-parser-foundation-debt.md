# F10 — E01/E03B/E03D/E04C parser-foundation 부채 일괄 해소 (W1)

원천: E-parser-grammar 감사 CHALLENGE 1/3/4. 공통 패턴: **티켓이 명명한 proof-rail 테스트 파일이
존재하지 않아 DoD 검증이 한 번도 실행된 적 없음** + dead-code seam.

## 부채 목록 (라이브)

1. **E01 named seams 부재**
   - `syntax/src/root.rs`, `src/error.rs` 부재 (Files-To-Create line 20, DoD line 73)
   - `RootKindV1` / `ParseErrorV1` 0 hits (DoD line 74 가 parser 테스트 backing 요구)
   - `tests/parser_foundation.rs` 부재 → proof rail `--test parser_foundation` 실행 불가
2. **E01 typed_ast = dead code**
   - `syntax/src/typed_ast.rs:4,9` `TypedDocumentAstV1`/`TypedRootAstV1` 정의만 존재,
     workspace 전체 inbound 0 (lib.rs:13 mod 선언만, :17-22 재수출 없음). `TypedNodeIdV1` 미존재
3. **E03B proof rail 부재**
   - `tests/sample_compare_export_grammar.rs` 부재. DoD nouns `compare_target`/`export_target`/`bridge_receipt`
     workspace 0 hits
4. **E03D context builtins 미구축**
   - `current_diff`/`changed_files`/`changed_symbols`/`ContextBuiltin` 0 hits (E03D line 44 가 explicit
     syntax node + fail-closed validation 요구). `tests/meta_commands.rs` 부재
5. **E04C typed_params 부재**
   - `syntax/src/declarations/` 에 mod/pack/view 만 — `typed_params.rs` 부재 (Files-To-Create line 19).
     `tests/typed_params.rs` 부재

## 작업

1. E01: `RootKindV1`(17-family 거울 또는 conformance registry 재사용) + `ParseErrorV1` typed error 도입,
   `root.rs`/`error.rs` 로 재배치, `tests/parser_foundation.rs` 신설
2. typed_ast: **wire-or-delete 결정** — sema handoff 에 실제 사용하면 재수출+프로듀서+테스트,
   사용 계획 없으면 삭제하고 E01 body 개정 (dead code 방치 금지)
3. E03B: `tests/sample_compare_export_grammar.rs` 신설, DoD nouns 를 필드/assert 로 실체화
4. E03D: context builtin 3종을 explicit syntax node 로 동결 + fail-closed validation + `tests/meta_commands.rs`
5. E04C: `declarations/typed_params.rs` (syntax) + F02 의 sema 측 + `tests/typed_params.rs`

## DoD

- 5개 named proof-rail 테스트 파일이 전부 실존하고 green (`--test parser_foundation`,
  `--test sample_compare_export_grammar`, `--test meta_commands`, `--test typed_params` + typed_ast 결정 반영)
- dead-code seam 0 (typed_ast wire-or-delete 완료)
- DoD nouns grep 가능
