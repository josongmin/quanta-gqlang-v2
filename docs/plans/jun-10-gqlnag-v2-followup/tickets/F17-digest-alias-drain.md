# F17 — quanta-gqlang-digest-v1 alias 드레인 + orphan dir 정리 (W4, semantica)

> **STATUS: DRAIN DONE (jun-10) — COMPILE-VERIFIED.** (사용자 지적이 옳았음: runtime 무관, package-rename alias = 동일 crate = 기계적 안전 rename, cargo metadata + leaf compile 로 검증)
> - **13 manifests**: dep key `quanta-gqlang-digest-v1 = { package = "codegraph-core-ir-foundation", ... }` → `codegraph-core-ir-foundation = { ... }` + feature ref(`"quanta-gqlang-digest-v1/..."` → `"codegraph-core-ir-foundation/..."`). 충돌검사 0(alias+직접 동시보유 없음), workspace.dependencies 간접참조 없음.
> - **17 .rs**: `quanta_gqlang_digest_v1` → `codegraph_core_ir_foundation` (use/pub use/path 전부).
> - **잔여 0**: toml/rs 모두 `quanta.gqlang.digest.v1` grep 0.
> - **검증:** `cargo metadata` OK(그래프 resolve) + `cargo check -p quanta-gqlang-hir-v1 --lib` **Finished GREEN** — 그 과정에서 drained crate(authority-support/absence-wire/envelope-wire) 도 함께 컴파일됨 = 실제 빌드 검증. package-rename 이라 타입 동일(의미상 null).
> - **잔여(orphan dir)**: owasp-matrix-v1 / sarif-projection-v1 은 S13D 삭제행 → F18 에서 dir 삭제(드레인은 이미 canonical 로 끝남).

원천: S-public-delete-waves 감사. 메인 세션 직접 재검증: `hir-v1/Cargo.toml:27,35` 에서
`quanta-gqlang-digest-v1 = { package = "codegraph-core-ir-foundation", path = ".../core/ir/foundation" }`
확인, 참조 파일 30개.

## 문제

S12A 가 "digest-v1 crate 삭제 완료" 로 보고했으나 실체는 **삭제가 아니라 package-rename alias 재배선**:
- donor 식별자 `quanta-gqlang-digest-v1` / `quanta_gqlang_digest_v1::` 가 13 Cargo.toml + 17 .rs 의
  살아있는 빌드 엣지로 잔존 (예: `quanta-runtime-execution/Cargo.toml:93`)
- S12A.md:75 는 source/manifest/docs/ledger 4자 일치 요구 — alias 는 manifest/source 불일치
- orphan non-member dir 이 alias 를 추가 운반:
  - `quanta-gqlang-owasp-matrix-v1/Cargo.toml:22` + `src/lib.rs:17` `pub use quanta_gqlang_digest_v1 as digest`
  - `quanta-gqlang-sarif-projection-v1/Cargo.toml:27`
- 이 alias 가 S09B metadata-레그 사각지대(F16)와 J06 17-blocker 의 원인

## 작업

1. 13개 Cargo.toml 의 alias dep 를 정식 명칭 `codegraph-core-ir-foundation` 직접 의존으로 교체
2. 17개 .rs 의 `use quanta_gqlang_digest_v1::` → `use codegraph_core_ir_foundation::` 일괄 재배선
   (re-export 경유 시 facade 경로 유지 검토)
3. orphan dir 처리: owasp-matrix-v1 / sarif-projection-v1 은 S13D 삭제 행 — F18 에서 삭제 예정이므로
   삭제가 먼저면 이 단계 생략 가능 (F18 과 순서 조율; 권장: F18 의 S13D 먼저 → 남는 alias 만 드레인)
4. 드레인 후 `rg "quanta.gqlang.digest.v1|quanta_gqlang_digest_v1"` workspace = 0 확인
5. 레저 regen (F16) + readiness 문서에서 digest-v1 blocker 0 확인

## DoD

- donor 식별자 grep 0 (Cargo.toml + .rs)
- 영향 crate 전체 `cargo check` green
- readiness 레저에서 digest-v1 행이 "already removed / blocker 0" 으로 전이
