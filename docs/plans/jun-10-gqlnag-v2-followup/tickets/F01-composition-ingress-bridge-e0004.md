# F01 — ingress-bridge E0004: PublicIrRootV1::Composition arm 누락 (W0, P0)

> **STATUS: DONE (jun-10).** verified-green by main session.
> - `query_bridge.rs`: `lower_composition_query_root_v1` 추가 (body 경유 lowering, working_set seam 재사용, RootSpecV1 미확장) + 2개 match arm(`lower_query_root_v1`, `root_kind_label_v1`).
> - blast radius 수리: `query_plane_bridge.rs` / `query_ingress_full_loop.rs` / `interactive_ingress_full_loop.rs` 의 `capability_for_root_v1` helper 3곳 Composition arm (query-plane 이므로 panic arm 아님, 중립 capability).
> - hellgate `tests/composition_query_bridge.rs`: 4 tests (positive + out-of-range/declaration-body/double-narrowing fail-closed) ALL PASS.
> - V6A full-loop GREEN: query 5 / interactive 4 / declaration_meta 5 / ai_workflow 4 = 18 PASS (real parse→sema→ir→bridge→adapter).
> - adversarial sweep: 잔여 non-exhaustive match 0 (bridge 가 외부 IR enum 의 유일 소비자).
> - 미해결 잔여: 없음. (external workspace 는 본 변경 영향 없음 — 모두 semantica-side)

원천: J01/J02/J03/J04/J09D 감사 GAP-1. 메인 세션 직접 재검증 완료.

## 문제

E03C 작업으로 `PublicIrRootV1` 에 10번째 variant `Composition(CompositionIrV1)` 이 추가됨:

- `quanta-gqlang-v2/crates/quanta-gqlang-v2-ir/src/root_ir.rs:19`

그러나 semantica 쪽 ingress-bridge 의 두 match 는 8 arm 에서 끝나고 catch-all 없음:

- `semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-gqlang-v2-ingress-bridge/src/query_bridge.rs:96-115` (`lower_query_root_v1`, `Meta` 에서 끝남)
- 같은 파일 `:545-553` (`root_kind_label_v1`)

enum 은 `#[non_exhaustive]` 아님 + bridge 는 이 external tree 를 path-dep (`ingress-bridge/Cargo.toml:20`).
→ **정적으로 확정된 E0004 컴파일 에러.** ingress-bridge 가 exec-conformance 의 dev-dep 이므로
`*_ingress_full_loop` 4종 포함 exec-conformance 테스트 타겟 전체가 빌드 불가.
STATUS.md 의 "(jun-09) fresh joint batch proof PASS" 주장은 이 트리에서 성립 불가능.

## 작업

1. `lower_query_root_v1` 에 `Composition` arm 추가. 의미 결정 필요:
   - Composition 은 let/with 합성 root — query-plane `RootSpecV1`(9-variant 고정) 으로 그대로 내릴 수 없음
   - 권장: `Declaration`/`Meta` 와 동일하게 typed reject (`IngressBridgeErrorV1`) 로 fail-closed 처리하고,
     composition 의 body root 만 별도 unwrap 경로로 처리할지는 E03C sema 후속에서 결정
2. `root_kind_label_v1` 에 `Composition(_) => "composition"` 추가
3. bridge 내 `PublicIrRootV1` 를 match 하는 다른 지점 전수 grep (`rg "PublicIrRootV1::" ingress-bridge/src/`) 후 동일 처리
4. `CARGO_TARGET_DIR=/tmp/qgv2-f01 cargo test --manifest-path packages/analysis/quanta-v2/Cargo.toml -p quanta-gqlang-v2-ingress-bridge -- --nocapture` green
5. V6A 4종 fresh 재실행: `query_ingress_full_loop` / `interactive_ingress_full_loop` / `declaration_meta_ingress_full_loop` / `ai_workflow_ingress_full_loop`
6. composition 입력이 query-plane 으로 bypass 하지 않는다는 negative 테스트 1개 이상 추가 (no-RootSpecV1-bypass fence 확장)

## DoD

- exec-conformance 테스트 타겟이 빌드된다 (E0004 해소)
- 4종 full-loop fresh PASS 로그 확보 (prose 아님)
- `RootSpecV1` 는 9 variants 유지 (`exec-contract/src/request.rs:20-29` 불변)
- composition-reject negative 테스트 green
