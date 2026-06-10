# F09 — E02C CHECK root: implicit class inference 거부 + `emit` + typed shape (W1)

> **STATUS: CORE DONE (jun-10).** external workspace 전체 GREEN (69 ok / 0 fail).
> - `decode_legacy` 제거 → 미접두 scope 는 `invalid_check_payload` "implicit class inference rejected" 로 fail-closed (E02C line 48 핵심 위반 해소).
> - `CheckEmitIntentV1 { Findings, Sarif }` + `emit` 필드 추가 → typed shape `{rule_id, class, scope, pattern, confirm, emit}` 완성. `emit` 절 부재 시 `Findings` 기본값(하위호환), `emit=apply` 는 `check_command_bridge_apply_rejected` 로 거부(E02C line 48 "direct command-bridge apply").
> - hellgate `tests/check_root_acceptance.rs` (E02C `check_root_acceptance_*`): 5 tests (explicit class+emit / emit 기본값 / emit sarif / implicit-class reject / command-bridge-apply reject) GREEN.
> - 다운스트림 정합: sema/ir re-export + 7개 `new()` call site + formatter(자동 emit) + CLI headless test(explicit form 으로 갱신) + shipped example `examples/check/hybrid_rule.qglang`(canonical 재생성).
> - cross-repo 안전성: 적대 grep 결과 semantica 측 `CheckRootIntentV1` full-destructure 0 (field-access only) → emit 필드 추가 non-breaking. 기존 `check:v1|...` 문자열은 emit 기본값으로 decode → bridge 동작 보존.
> - **환경 블로커(F09 무관):** semantica 워크트리의 미커밋 WIP `quanta-runtime/src/queries/taint/types.rs`(`quanta_contract_types::security` 미해결) + `pta_solver::PointsToResult` 필드 드리프트가 semantica 풀빌드를 막아 V6A cross-repo 재실행은 보류. F09 변경과 무관(외부 CHECK 만 수정).
>
> **Deferred (honest):** (1) E02C line 47 의 `CLASS=`/`SCOPE=` surface-clause 문법(현재는 `check:v1|` 페이로드 유지) — 더 큰 문법 변경; typed shape+reject 계약은 페이로드로 충족. (2) sema/ir 측 `check_root_acceptance_*` named fixture(line 61, 현재 syntax 만; sema/ir 는 동등 테스트 다른 이름). (3) "unscoped workspace mutation" reject 의 sema 측 심화(F02 admissibility).

원천: E-parser-grammar 감사 CHALLENGE 2. 메인 세션 직접 재검증 완료.

## 현재 상태 (라이브)

- `syntax/src/roots/check.rs:7-10` `CheckRootAstV1 { rule_id, scope }` — String 2필드 carrier
- `:103-111` `encode_scope_payload` 가 class|scope|pattern|confirm 을 구분자 문자열 codec 으로 packing
- `:113-116` `decode()` — **접두사 없는 입력 전부 `decode_legacy` 로 기본 라우팅**
- `:167-176` `decode_legacy` 가 `class="legacy"` 하드코딩, `:394` 테스트가 이를 의도로 고정
- E02C DoD line 48 "reject implicit class inference" **정면 위반** — 거부가 아니라 기본 경로
- 필수 `emit` 필드 0 hits. DoD typed shape `{ rule_id, class, scope, pattern, confirm, emit }` 미구현

## 작업

1. `CheckRootAstV1` 를 typed shape 로 승격: `{ rule_id, class, scope, pattern, confirm, emit }` —
   string-codec packing 제거 (formatter round-trip 은 typed 필드 기반으로 재구성)
2. implicit class inference 거부: 접두사 없는/class 누락 입력은 fail-closed typed parse error
   (`check_missing_class` 류 코드) — `decode_legacy` 삭제 또는 명시적 `legacy:` 접두 입력에만 격리
3. `:394` 의 `class=="legacy"` 고정 테스트를 거부-assert 로 교체
4. `emit` 절 파싱 + sema admissibility (F02 의 check_explain_profile 과 연결)
5. downstream 정합: ir `CheckIrV1`, semantica `check_adapter.rs` / `rule_class_v1` 매핑
   (`query_bridge.rs:388-391` 의 class 매핑이 "legacy" 를 어떻게 받는지 확인 — bridge 거부 코드 추가)

## DoD

- 접두사-없는 class 입력이 parse 단계에서 typed code 로 거부됨 (negative 테스트)
- `emit` 필드가 AST→IR→adapter 까지 관통
- round-trip (parse→format→reparse) green 유지
- `check_root_acceptance_*` fixture (DoD line 61) 정비
