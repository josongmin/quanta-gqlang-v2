# F18 — S12B/S13A/S13B잔여/S13C/S13D 삭제 웨이브 실행 (W4, semantica)

> **STATUS: 4/14 DELETED (jun-10, cargo-metadata-verified); 10 BLOCKED by real consumers.**
> 검증 방법: `cargo metadata` 로 crate별 reverse-dep 계산(= runtime 컴파일 불필요) → 0-consumer 만 삭제.
> - **삭제 완료(0 reverse-dep, cargo metadata OK):**
>   - `quanta-gqlang-ranked-proof-contamination-detector-v1` (S13A, member) — workspace member 라인 2곳 제거 + dir 삭제
>   - `quanta-gqlang-sigil-v1` (S13B, orphan) / `quanta-gqlang-planner` (S13C, orphan) / `quanta-gqlang-owasp-matrix-v1` (S13D, orphan) — dir 삭제
> - **BLOCKED (실소비처 존재 — 삭제 전 소비처 재배선 필요 = 진짜 S13B/S12B 마이그레이션):**
>   - catalog-binding-v1(7), validation-v1(8), fragment-v1(6), gates-v1(2: exec-conformance/runtime), core(4: lowering/runtime/execution/query-owner-surface), lowering(1: query-owner-surface), hir-v1(2: runtime/query-owner-surface), catalog-v1(3: adapters-security/runtime/execution), sdk-handlers-v1(2: hir-v1/command-owner-surface), sarif-projection-v1(2: exec-conformance/observation-wire)
>   - 이 소비처 재배선은 quanta-runtime/execution 코드를 건드려야 함(현재 미컴파일) → 빌드 복구 후 안전.
> - **잔여(S13B gate-silent):** planner-v1 이 tooling 11파일(catalog yaml/boundary policy/lint/god-surface-baseline.json)에 잔존. 일부는 baseline/allowlist 라 의도적일 수 있어 일괄삭제 위험 → 파일별 확인 후 정리.

원천: S-public-delete-waves 감사. self-report 는 정직 (planned-only/partial) — 이 티켓은
"미시작 작업의 실행" 패킷. F15(게이트 수리)·F17(alias 드레인) 후 진행 권장.

## 삭제 행 인벤토리 (라이브 확인됨)

| Wave | Crate | 현재 상태 |
|---|---|---|
| S12B | quanta-gqlang-catalog-binding-v1 | PRESENT, member (quanta-v2 Cargo.toml:254) |
| S12B | quanta-gqlang-validation-v1 | PRESENT, member (:253) |
| S12B | quanta-gqlang-fragment-v1 | PRESENT, member (:251) |
| S13A | quanta-gqlang-gates-v1 | PRESENT, member ×2 (:257) |
| S13A | quanta-gqlang-ranked-proof-contamination-detector-v1 | PRESENT, member (:258) |
| S13B | quanta-gqlang-core | PRESENT, member ×2 — 게이트 첫 FAIL 지점 |
| S13B | quanta-gqlang-lowering | PRESENT |
| S13B | quanta-gqlang-hir-v1 | PRESENT, member |
| S13B | quanta-gqlang-catalog-v1 | PRESENT, member |
| S13B | quanta-gqlang-sdk-handlers-v1 | PRESENT, orphan non-member |
| S13B | quanta-gqlang-sigil-v1 | PRESENT, orphan non-member |
| S13B | quanta-gqlang-doctor-v1 / planner-v1 | 이미 ABSENT (2/8 완료) |
| S13C | quanta-gqlang-planner (bare) | PRESENT, orphan non-member dir |
| S13D | quanta-gqlang-owasp-matrix-v1 | PRESENT, orphan + digest alias 운반 |
| S13D | quanta-gqlang-sarif-projection-v1 | PRESENT, orphan + digest alias 운반 |

## S13B 잔여 residue (planner-v1 — dir 삭제됐으나 tooling 이 살아있는 owner 로 참조, 게이트 비커버)

- `tools/catalog/entries/domain_contracts/quanta_gqlang_v1_residual_leaf_stack.yaml:24` (planner-v1, :22 sigil-v1)
- `tools/rust-crate-boundary-guard/policies/quanta-runtime-gqlang-hot-path.toml:369` (부재 dir 로의 path dep)
- `tools/ci/lint/lint-quanta-runtime-planner-feature-lane.sh:35-46` (live dependency 로 요구)
- (S16B 동류) `tools/catalog/entries/domain_contracts/quanta_runtime_gqlang_wire_bundle.yaml` —
  canonical_impl/entrypoint 8/8 DEAD 경로인데 status:stable + wiring:wired 주장 (:18 planner-v1)
- rustdoc 깨진 링크 `request_spec_impl_mutators.rs:837`, 안내 문자열 `testing/tests.rs:895`

## 작업

1. 삭제 순서: 쉬운 orphan 부터 — S13C(bare planner) → S13D(owasp/sarif, digest alias 동반 제거) →
   S13B orphan(sdk-handlers/sigil) → S12B 3종 → S13A 2종 → S13B member 4종(core/lowering/hir/catalog)
2. 각 행: inbound 인벤토리(레저) 확인 → 소비자 재배선 → dir+manifest 삭제 → 레저 regen → 게이트
3. S13B tooling residue 5건 청소 (yaml/policy/lint/rustdoc/문자열) + F15 의 게이트 needle 로 재발 방지
4. wire_bundle.yaml 은 살아있는 owner 경로로 재작성하거나 행 폐기

## DoD

- 15개 잔존 행 전부 dir+manifest absent, `--ticket S12B/S13A/S13B/S13C/S13D` 게이트 PASS
- tooling residue grep 0 (deleted-crate 참조)
- 레저가 전 행을 "already removed" 로 보고
