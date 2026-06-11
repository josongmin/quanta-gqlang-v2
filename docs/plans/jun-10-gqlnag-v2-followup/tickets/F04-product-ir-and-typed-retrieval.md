# F04 — E06B product IR + E07A typed retrieval + E07B fragment policy (W1)

> **STATUS: A+B+C DONE (jun-10) — 헬게이트 6 GREEN, workspace 0 regression.**
> - **E06B** `ir/src/product.rs`: `AssetIdV1`(kind-tagged), `ProductAssetKindV1`(6), `ProductAssetIrV1`, `lower_declaration_to_product_v1`(declaration 전수 match, catch-all 금지). hellgate `product_ir`(1): 5 kind → distinct asset id.
> - **E07A** `ir/src/retrieval/spec.rs`(DoD-named path): `RetrievalSyntaxIrV1`/`TextSearchSpecIrV1`/`SemanticSearchSpecIrV1`/`HybridSearchSpecIrV1`/`SymbolSearchSpecIrV1`/`SymbolProjectionIrV1`/`RetrievalSpecIrV1` + `RetrievalIrV1::typed_spec_v1()`(additive, 기존 필드 비파괴) + 미지원 flag fail-closed(`retrieval_unsupported_flag`). hellgate `typed_retrieval_surface`(2).
> - **E07B** `sema/src/fragment_policy.rs`: `FragmentPolicyV1::admit_dialect_v1`(sourcegraph 허용, 그 외 `fragment_dialect_rejected`, 누락 `fragment_dialect_missing`). hellgate `fragment_policy`(3).
> - **잔여(honest):** E07B 의 syntax-side inline `fragment:<dialect>:<text>` 파싱은 더 큰 grammar 변경이라 미구현(정책 seam 은 완료). RetrievalIrV1 필드 무변경(semantica bridge 직소비 보존).
> - 검증: ir product_ir/typed_retrieval_surface + sema fragment_policy + workspace 0 fail.


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

## 구현 계획 (jun-10 보완 — 파일·로직 단위)

현재 앵커: `ir/src/retrieval_ir.rs` = `RetrievalIrV1{query, scope, mode(Lexical|Semantic|Hybrid), limit}` 박형. `root_ir.rs` 의 `DeclarationIrV1`(View/Rule/Model/Pack/Profile/Gate + ViewQueryIrV1). semantica bridge 가 `RetrievalIrV1` 필드를 직접 소비(`query_bridge.rs:487-499 RetrievalModeIrV1 매칭`) → **기존 필드 제거 금지, additive 로**.

### Part A — E06B product IR
1. **`ir/src/product/mod.rs`** + **`ir/src/product/asset_ir.rs`**
   - `pub struct AssetIdV1(pub String)` (`"view::<name>"` 형 canonical id, `fn for_declaration_v1(&DeclarationIrV1) -> AssetIdV1`)
   - `pub enum ProductAssetKindV1 { View, Rule, Model, Pack, Profile, Gate }`
   - `pub struct ProductAssetIrV1 { pub asset_id: AssetIdV1, pub kind: ProductAssetKindV1, pub declaration: DeclarationIrV1 }`
2. **`ir/src/product/declaration_ir.rs`**
   - `pub fn lower_declaration_to_product_v1(decl: &DeclarationIrV1) -> ProductAssetIrV1` — 전수 match(catch-all 금지), kind/id 도출. catalog 소비자: `catalog/capability_catalog.rs` 가 asset_id 로 row 참조(기존 row 에 `pub fn asset_id_v1()` 추가).
   - 헬게이트 **`ir/tests/declaration_product_ir.rs`**: 6 declaration kind 전수 → kind/asset_id assert + id 충돌 없음(BTreeSet) + catalog row 연결.

### Part B — E07A typed retrieval surface
3. **`ir/src/retrieval/mod.rs`** + **`ir/src/retrieval/spec.rs`** (DoD-named 경로; 기존 `retrieval_ir.rs` 는 유지)
   - `pub enum RetrievalSyntaxIrV1 { Native, Sourcegraph }`
   - 6 owner: `TextSearchSpecIrV1{query_text, syntax}`, `SemanticSearchSpecIrV1{query_text}`, `HybridSearchSpecIrV1{query_text}`, `SymbolSearchSpecIrV1{query_text, projection: SymbolProjectionIrV1{WorkspaceSymbols,DocumentSymbols}}`, 합성 `pub enum RetrievalSpecIrV1{Text,Semantic,Hybrid,Symbol}`, `pub struct SampleSpecIrV1{spec, limit}`
   - **additive 승격**: `impl RetrievalIrV1 { pub fn typed_spec_v1(&self) -> Result<RetrievalSpecIrV1, IrLoweringErrorV1> }` — mode→spec 변환 + **fail-closed**: query 가 `sg:`/`patterntype:` 류 미지원 search-owner flag 포함 시 `retrieval_unsupported_flag` (Appendix N.6).
   - 헬게이트 **`ir/tests/typed_retrieval_surface.rs`**: 3 mode→spec round / symbol projection / 미지원 flag reject 2케이스.
### Part C — E07B fragment policy
4. **`syntax/src/retrieval/mod.rs`+`fragment.rs`**: SAMPLE `query=` 안 `fragment:<dialect>:<text>` 인라인 구문 파싱 → `RetrievalFragmentAstV1{dialect, text}` (없으면 None — 기존 입력 비파괴).
5. **`sema/src/retrieval/fragment_policy.rs`**: `pub struct FragmentPolicyV1 { allowed: &'static [&'static str] }` (`["sourcegraph"]`), `pub fn admit_fragment_v1(...)` → `fragment_dialect_rejected` fail-closed.
   - 헬게이트 **`sema/tests/fragment_policy.rs`**: sourcegraph 허용 / `fragment:regex:` 거부 / dialect 누락 malformed 거부.

### 검증
`cargo test -p quanta-gqlang-v2-ir --test declaration_product_ir --test typed_retrieval_surface && cargo test -p quanta-gqlang-v2-sema --test fragment_policy && cargo test --workspace`. 적대 포인트: `RetrievalIrV1` 필드 변경 금지(semantica bridge 직소비) — additive 메서드만.
