# INT-02 — 구형 host-adapter의 공개 계약 불일치 처리

상태: **소스 불일치 확인 / 컴파일 판정 전** · 우선순위: P2 · 소유: Semantica workspace/adapter owner.

## 근거

- `S:packages/analysis/quanta-v2/crates/quanta-gqlang-v2-host-adapter/src/lib.rs:3-11`은 구형 lowering 모듈을 무조건 컴파일 대상으로 둔다. `graph_lowering.rs:43-45`, `check_lowering.rs:36-38`은 `GraphQueryHirV1`, `LegalityHirV1` 등 V1 HIR를 `quanta_gqlang_v2_frontend_contract`에서 import한다.
- 현 공개 facade `Q:crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:13-64`는 V3 entry/DTO의 curated allowlist만 export한다. 이 V1 HIR를 내보내지 않는다.
- crate는 `S:packages/analysis/quanta-v2/Cargo.toml:346`의 workspace member다. `S:packages/analysis/quanta-v2`와 `S:crates`에서 Cargo.toml/`.rs` 문자열을 조사한 범위에서는 제품 Cargo 소비자 없이 자체 테스트·manifest 부정 검증만 확인됐다. 텍스트 census는 compiler-resolved dependency graph가 아니다.

## 판단·해결

1. 등록된 QBC owner/member rail로 현 source에서 컴파일 판정을 먼저 얻는다. 이 티켓은 아직 rustc failure를 관찰했다고 주장하지 않는다.
2. 실제 제품/테스트 소비자와 보존해야 할 계약을 확인한다. 소비자가 없으면 crate 및 등록/테스트 참조를 소유자 승인 범위에서 함께 제거한다. 필요하다면 V1 HIR 재-export/compat shim 없이 V3 공개 DTO와 별도 specified runtime payload를 기준으로 재설계한다.
3. workspace manifest, feature-isolation, suite registry, doc 참조를 함께 맞춘다. 제품 V3 경로를 구형 adapter로 재우회시키지 않는다.

## 완료 조건

- 의도한 workspace/member QBC compile·test가 현재 pair source digest에서 통과하고, 삭제 또는 V3 cutover의 선택 이유·남은 소비자가 기록된다.
- 현 QGLang facade의 public allowlist를 adapter 편의 때문에 넓히지 않는다. Graph/CHECK 실행 지원 확대는 별도 기능 계약이다.
