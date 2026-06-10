# F04 — E06B product IR + E07A typed retrieval + E07B fragment policy (W1)

> **STATUS: NOT STARTED (jun-10 재점검).** external W1. `ir/src/product/`, `ir/src/retrieval/spec.rs`, fragment policy 전부 부재 — 미착수.


원천: E-sema-ir-retrieval 감사. 셋 다 missing (high confidence).

## 현재 상태 (라이브)

- E06B: `ir/src/product/` 디렉터리 없음. `ProductAssetIrV1`/`AssetIdV1` 0 hits.
  `DeclarationIrV1` 는 root_ir.rs 에 있으나 재사용 product/catalog-asset IR 부재.
  DoD 파일 `product/declaration_ir.rs`, `product/asset_ir.rs`, `tests/declaration_product_ir.rs` 전부 부재
- E07A: 6종 typed surface owner (`SampleSpecV1`/`RetrievalSpecV1`/`TextSearchSpecV1`/`SemanticSearchSpecV1`/
  `HybridSearchSpecV1`/`SymbolSearchSpecV1`) 전부 0 hits. `ir/src/retrieval_ir.rs:2` `RetrievalIrV1` 는
  `{query, scope, mode, limit}` 박형 carrier. unsupported search-owner flag 의 typed-reject (Appendix N.6
  fail-closed) 없음. `tests/typed_retrieval_surface.rs` 부재
- E07B: fragment policy seam 전체 부재 — `syntax/src/retrieval/fragment.rs`,
  `sema/src/retrieval/fragment_policy.rs`, `tests/fragment_policy.rs` 없음. `FragmentPolicy|DialectRestriction` 0 hits

## 작업

1. `ir/src/product/` 모듈: `ProductAssetIrV1` + `AssetIdV1` + declaration→product lowering, `tests/declaration_product_ir.rs`
2. `ir/src/retrieval/spec.rs`: 6종 SpecV1 typed surface + `RetrievalIrV1` 를 typed spec 합성으로 승격,
   unsupported flag fail-closed typed-reject 경로, `tests/typed_retrieval_surface.rs`
3. fragment policy: syntax fragment 노드 + sema `fragment_policy.rs` (dialect restriction, fail-closed adapter contract) + `tests/fragment_policy.rs`
4. semantica 측 `sample_adapter.rs` (S06B) 가 새 typed spec 을 소비하도록 후속 연동 포인트 기록 (J03B 재핸드셰이크)

## DoD

- 위 named seam 전부 정의 + non-test 소비자(lowering 경로 또는 CLI/LSP) 존재
- 3개 proof-rail 테스트 파일이 실제 존재하고 negative 케이스 포함 green
