# F13 — S05A/S06A/S06B 어댑터 gap (W2, semantica)

> **STATUS: S05A+S06B+S06A ALL DONE (jun-10) — runtime 빌드 복구 후 전부 cargo test 검증.**
> - **S06A DONE (SOTA++):** `host-adapter/src/catalog_adapter.rs` 신설 — `CatalogBridgeV1` 가 catalog-bridge seam(host_kind=Capability, result_kind=CapabilityCatalog, CapabilityQueryV1 shape)의 단일 owner. `for_category_v1`(SHOW)/`for_capability_v1`(DESCRIBE) 생성자. show_adapter/describe_adapter 는 bridge 로 위임(thin projection) → inline Capability 리터럴 0(중복 라우팅 제거). hellgate `catalog_bridge_routes_every_read_to_the_capability_host_v1` + show/describe lowering 테스트 GREEN.
> - **S05A/S06B 도 재검증 GREEN:** adapter_lowering_matrix 6 / search_projection_ownership 2 (runtime 빌드 후 실행).
>
> 이전 노트: quanta-runtime 블록이라 cargo 미실행 — 타입/전파경로 정적 추적으로 검증.
> - **S05A (reject 커버)** `exec-conformance/tests/adapter_lowering_matrix.rs`: `select_root_fails_closed_on_unsupported_entity_kind_v1` 추가 — `EntityKindV1::Diagnostic`(SELECT surface map 미지원 변종)으로 `lower_request_v1` 호출 → `LoweringErrorV1::Select(SelectLoweringErrorV1::UnsupportedEntityKind(Diagnostic))` fail-closed assert. 전파경로 확인: `request_lowering.rs:50-52` `lower_select_v1(spec).map_err(LoweringErrorV1::Select)`. (이전엔 reject assert 0)
> - **S06B (진짜 ownership proof)** `exec-conformance/tests/search_projection_ownership.rs`: 17줄 tautology(matches! 자기검사) 제거 → `lower_sample_v1` 로 실제 lowering 후 **`runtime_packet.backend_owner == "quanta-index"`** assert (text + symbol 2 케이스). off-rail `host-adapter/tests/search_host_quanta_index.rs` 의 실제 증명을 S06B 게이트 레일(`--test search_projection_ownership`)로 이식. 모든 타입은 그 실제 sibling 에서 검증.
> - **S06A (catalog_adapter.rs)**: DEFERRED — `host-adapter/src/catalog_adapter.rs` 신설은 show/describe catalog 로직 consolidation refactor 라 컴파일러 필요(빌드 블록). thin re-export stub 는 audit 가 싫어하는 placeholder 이므로 회피. quanta-runtime 복구 후 진행.



원천: S-adapter-lowering-bridge 감사. 어댑터 본체는 진짜 — gap 은 (1) create-miss 1건,
(2) vacuous proof 2건.

## Gap 목록 (라이브)

1. **S06A — catalog_adapter.rs create-miss (hard)**
   - S06A.md:16-18 Files-To-Create 가 `src/catalog_adapter.rs` 명시 — repo 전체 0 hits.
     show/describe lowering 은 `show_adapter.rs:13`/`describe_adapter.rs:14` 로 살아있으나 명시된 생성 의무 미이행
2. **S06B — DoD-named proof 가 17줄 tautology**
   - `exec-conformance/tests/search_projection_ownership.rs:9-12` 가 matches! 자기비교, `:13-16` 정적 테이블 —
     `quanta-index` 문자열 자체가 파일에 없음. DoD line 54 는 "SearchHost→quanta-index route correctness 증명" 요구
   - S06B proof rail (line 63) 이 **이 vacuous 파일만 실행** — 게이트가 빈 증명을 자기인증
   - 진짜 증명은 off-rail: `host-adapter/tests/search_host_quanta_index.rs` (`:25` backend_owner=="quanta-index",
     `:101` fail-closed) — exec-conformance 가 아니라 host-adapter 에 있어 레일이 안 돈다
3. **S05A — reject 케이스 미커버**
   - `select_adapter.rs:47` 에 `SelectLoweringErrorV1::UnsupportedEntityKind` Err 경로 실존하나
     DoD-named `adapter_lowering_matrix.rs` 에 is_err/expect_err/unwrap_err 0 — happy-path panic 가드뿐

## 작업

1. S06A: `catalog_adapter.rs` 생성 — show/describe 의 catalog-bridge 공통 로직을 이관 집약
   (혹은 ticket body 개정으로 show/describe_adapter 분담 명문화 — 단 STATUS 정정 필수)
2. S06B: `search_projection_ownership.rs` 를 진짜 증명으로 교체 — `search_host_quanta_index.rs` 의
   backend-owner assert 를 exec-conformance 레일로 이식하거나 proof rail 을 양 파일 실행으로 확장
3. S05A: `adapter_lowering_matrix.rs` 에 UnsupportedEntityKind reject 케이스 추가 (expect_err + typed code)
4. (경미) S05B: find subkind `&'static str` discriminant 의 typed enum 승격 검토 — 선택

## DoD

- catalog_adapter.rs 실재 (또는 개정 명문화) — Complete Rule clause 1 충족
- S06B proof rail 실행이 quanta-index ownership 을 실제 assert
- adapter_lowering_matrix 에 reject 커버리지 ≥1 (expect_err)
