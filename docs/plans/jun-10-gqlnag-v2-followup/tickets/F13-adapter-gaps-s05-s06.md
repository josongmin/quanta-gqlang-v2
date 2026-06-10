# F13 — S05A/S06A/S06B 어댑터 gap (W2, semantica)

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
