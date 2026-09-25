# INT-02 — 구형 host-adapter의 공개 계약 불일치 처리

상태: **소스 불일치·raw Cargo compile 진단 RED / QBC 판정 전** · 우선순위: P2 독립 workspace 위생 · 소유: Semantica workspace/adapter owner.

## 근거

- `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-host-adapter/src/lib.rs:3-11`은 구형 lowering 모듈을 무조건 컴파일 대상으로 둔다. `graph_lowering.rs:43-45`, `check_lowering.rs:36-38`은 `GraphQueryHirV1`, `LegalityHirV1` 등 V1 HIR를 `quanta_gqlang_v2_frontend_contract`에서 import한다.
- 현 공개 facade `Q:crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:13-64`는 V3 entry/DTO의 curated allowlist만 export한다. 이 V1 HIR를 내보내지 않는다.
- crate는 `S:packages/analysis/quanta-v2/Cargo.toml:346`의 workspace member다. `S:packages/analysis/quanta-v2`와 `S:crates`에서 Cargo.toml/`.rs` 문자열을 조사한 범위에서는 제품 Cargo 소비자 없이 자체 테스트·manifest 부정 검증만 확인됐다. 텍스트 census는 compiler-resolved dependency graph가 아니다.

## 현재 재현

- Semantica `9ce5972352ad3503653e7952a9bab17340439898`의 dirty checkout에서 `cargo check -p quanta-gqlang-v2-host-adapter --lib --no-default-features`를 실행했다.
- 결과는 exit 101이다. `algebra_lowering.rs`, `check_lowering.rs`, `graph_lowering.rs`, `presentation_lowering.rs`의 제거된 frontend/exec-contract import에서 `E0432` 5건이 발생했고, 그 후속 타입 추론 `E0282` 2건이 발생했다.
- 이는 현재 sibling QGLang `145b6682…`과의 소스 불일치를 확인하는 진단이다. Semantica 규칙상 raw Cargo 결과는 권위 있는 Rust verdict가 아니며, checkout도 다른 작업으로 dirty하다. 등록 QBC 판정은 아직 없다.

## 판단·해결

1. 제품 Rust 소비자는 조사 범위에서 미발견이지만 `tools/catalog`, `tools/casectl/registry/ext-gqlang-serving.yaml`, QBC timing/feature-isolation, boundary policy 참조는 존재한다. 이 참조가 실제 gate·제품 보증인지 먼저 분류한다.
2. QBC에서 package 단위 compile을 진단한다. 제품 소비자가 없고 구형 owner rail도 더 이상 유효하지 않으면 crate와 workspace/registry/catalog/policy 참조를 함께 제거한다. 제품 요구가 실제 있다면 이 티켓에서 V1 shim을 만들지 말고 별도 V3 payload/consumer 설계 승인을 받는다.
3. 선택한 경로의 package/workspace QBC compile과 해당 registry/policy 검증을 현재 source digest에서 실행한다. 이 P2는 제품 V3 pair proof의 무조건 선행조건이 아니다.

## 완료 조건

- 의도한 package/workspace QBC compile·test가 현재 pair source digest에서 통과하고, 유지/삭제 선택 이유와 남은 registry·제품 소비자가 기록된다.
- 현 QGLang facade의 public allowlist를 adapter 편의 때문에 넓히지 않는다. Graph/CHECK 실행 지원 확대는 별도 기능 계약이다.
