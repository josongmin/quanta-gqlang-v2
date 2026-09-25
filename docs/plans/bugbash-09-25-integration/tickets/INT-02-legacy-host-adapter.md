# INT-02 — 구형 host-adapter의 공개 계약 불일치 처리

상태: **현재 pair compile RED 확인** · 우선순위: P2 · 소유: Semantica workspace/adapter owner.

## 근거

- `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-host-adapter/src/lib.rs:3-11`은 구형 lowering 모듈을 무조건 컴파일 대상으로 둔다. `graph_lowering.rs:43-45`, `check_lowering.rs:36-38`은 `GraphQueryHirV1`, `LegalityHirV1` 등 V1 HIR를 `quanta_gqlang_v2_frontend_contract`에서 import한다.
- 현 공개 facade `Q:crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:13-64`는 V3 entry/DTO의 curated allowlist만 export한다. 이 V1 HIR를 내보내지 않는다.
- crate는 `S:packages/analysis/quanta-v2/Cargo.toml:346`의 workspace member다. `S:packages/analysis/quanta-v2`와 `S:crates`에서 Cargo.toml/`.rs` 문자열을 조사한 범위에서는 제품 Cargo 소비자 없이 자체 테스트·manifest 부정 검증만 확인됐다. 텍스트 census는 compiler-resolved dependency graph가 아니다.

## 현재 재현

- Semantica `01c699be4d26f3d80e378c8906dfd2ee9b39af41`의 dirty checkout에서 `cargo check -p quanta-gqlang-v2-host-adapter --lib --no-default-features`를 실행했다.
- 결과는 exit 101이다. `algebra_lowering.rs`, `check_lowering.rs`, `graph_lowering.rs`, `presentation_lowering.rs`의 제거된 frontend/exec-contract import에서 `E0432` 5건이 발생했고, 그 후속 타입 추론 `E0282` 2건이 발생했다.
- 이는 현재 sibling QGLang `39f72917…`과의 실제 compile defect다. Semantica checkout의 다른 dirty 변경 때문에 repository qualification으로 승격할 수는 없지만, “컴파일 판정 전” 상태는 아니다.

## 판단·해결

1. 실제 제품/테스트 소비자와 보존해야 할 계약을 확인한다. 소비자가 없으면 crate와 root/nested workspace 등록, runtime/executor census, timing scenario, feature-isolation 및 readiness 참조를 함께 제거한다.
2. 필요하다면 V1 HIR 재-export/compat shim 없이 V3 공개 DTO와 별도 specified runtime payload를 기준으로 재설계한다.
3. 수정 후 위 exact check와 등록된 QBC owner/member rail을 같은 source digest에서 통과시킨다. 제품 V3 경로를 구형 adapter로 재우회시키지 않는다.

## 완료 조건

- 의도한 workspace/member QBC compile·test가 현재 pair source digest에서 통과하고, 삭제 또는 V3 cutover의 선택 이유·남은 소비자가 기록된다.
- 현 QGLang facade의 public allowlist를 adapter 편의 때문에 넓히지 않는다. Graph/CHECK 실행 지원 확대는 별도 기능 계약이다.
