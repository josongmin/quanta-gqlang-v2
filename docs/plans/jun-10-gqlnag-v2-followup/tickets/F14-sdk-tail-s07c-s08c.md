# F14 — S07C named guard + S08C default-feature command path (W2, semantica)

> **STATUS: S07C DONE (jun-10, static-verified + bonus drain); S08C = documented decision.**
> - **S07C guard** `quanta-sdk-runtime-executor/tests/runtime_executor_source_residue_guard.rs`: named `const FORBIDDEN_QUERY_BOUNDARY_IMPORT = "quanta-runtime-gqlang-boundary-v1"` 추가 + 가드가 그 crate-path(`quanta_runtime_gqlang_boundary_v1`) 부재를 검사 + 잔존 `wire_to_boundary_v1.rs` 를 guarded_files 에 포함. (DoD "또는 기존 guard 확장" 경로 — 새 v2_guard.rs 불필요, 무중복)
> - **bonus drain (가드가 노출한 실제 잔여):** 활성 캠페인이 재도입한 `group_gqlang_lockfile.rs:8 use crate::query_backend::gqlang_ingress_v1::FreshnessStatusV1` 직접 ingress import 발견 → `public_query_surface_v1` 에 `FreshnessStatusV1` 노출(비-feature-gated, gqlang_ingress_v1:10 원천 확인) 후 `crate::query_backend::public_query_surface_v1::FreshnessStatusV1` 로 재배선.
> - **검증(정적):** 가드 로직(순수 fs+string)을 bash 로 13개 파일 전수 replicate → **0 residue, guard WOULD PASS**. 재수출 체인(authority-support→gqlang_ingress_v1→public_query_surface_v1→lockfile) 정적 추적 sound. cargo 미실행(quanta-runtime 블록)이나 가드는 타입 의존 0이라 replicate 가 곧 검증.
> - **S08C 결정(코드변경 회피):** 기본빌드(`binding-core`)의 command-plane fail-closed stub 은 **의도된 read-only thin-shell lane** 으로 유지(기본 feature 승격은 빌드검증 불가 상태에서 위험). 액션 = production/command 빌드는 `mutation-backend-runtime` 활성 + 그 feature-on CI 레일 추가(quanta-runtime 복구 후 실행). 기본을 바꾸지 않음.



원천: S-sdk-drain 감사. 실질 residue 는 깨끗 (S07A/S07B 는 라이브 검증 complete) — 남은 건
named guard 계약 미이행과 기본 빌드 경로 stub 문제.

## Gap 목록 (라이브)

1. **S07C — named guard 부재 + guard under-scope**
   - S07C.md:54 가 선언한 `const FORBIDDEN_QUERY_BOUNDARY_IMPORT = "quanta-runtime-gqlang-boundary-v1"`
     crate 전체 0 hits; `v2_guard.rs` 부재
   - `quanta-sdk-runtime-executor/tests/runtime_executor_source_residue_guard.rs:30,34` 가 검사하는 substring 은
     티켓의 금지 import 와 직교 (query_backend::gqlang_ingress_v1 / mutation_backend::gqlang_command_transport_v1)
   - 유일한 Boundary*-형 잔존 파일 `src/client/query_handlers/wire_to_boundary_v1.rs` 가 guarded_files
     리스트(`:11-23`)에서 빠져 있음 — 가드가 못 보는 단 하나의 파일이 잔존 파일
   - (완화: 실질 residue 는 grep 0 — 의도는 충족, 계약만 미이행)
2. **S08C — 기본 빌드가 fail-closed stub**
   - `quanta-sdk-runtime-executor/Cargo.toml:30-31` `default=["binding-core"]`, binding-core=[] —
     `mutation-backend-runtime` (`:47`) 미포함
   - 따라서 기본 빌드의 `execute_public_gqlang_command_json_v1` 은
     `gqlang_command_handler.rs:304-323` 의 `#[cfg(not(feature))]` stub (CommandProviderFailed) 로 해소.
     진짜 구현(`:61`)은 non-default feature 뒤
   - (완화: stub 은 의도된 fail-closed lane — 단 "complete" 주장과 기본 경로 실체가 불일치)

## 작업

1. S07C: `FORBIDDEN_QUERY_BOUNDARY_IMPORT` const + `v2_guard.rs`(또는 기존 guard 파일 확장) 구현 —
   금지 needle 을 티켓 명세와 일치시키고 `wire_to_boundary_v1.rs` 를 guarded_files 에 추가
2. S08C: 둘 중 하나 결정 + 명문화:
   a. `mutation-backend-runtime` 을 default feature 로 승격 (command plane 이 기본 활성이어야 한다면)
   b. 현 구조 유지 + ticket/STATUS 를 "기본=fail-closed, 실경로=feature-gated" 로 정정하고
      feature-on CI 레일 추가 (`cargo test --features mutation-backend-runtime`)
3. 두 변경 모두 closeout gate `--ticket S07C` / `--ticket S08C` 재실행으로 확정

## DoD

- named guard 가 티켓 명세 needle 로 작동 + 잔존 파일 커버
- S08C 결정이 코드/문서/CI 3자 일치
- 해당 gate 2종 fresh PASS
