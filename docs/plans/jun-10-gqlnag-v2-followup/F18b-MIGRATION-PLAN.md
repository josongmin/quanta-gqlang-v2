# F18b 전체 마이그레이션 계획 (jun-11, 4-클러스터 심층조사 기반)

> **⚠️ SUPERSEDED — 확정판은 [FINAL-PLAN.md](FINAL-PLAN.md).** 본 문서는 조사 원자료(크레이트별 실측 표)로만 유효.
> 주요 정정: Phase C "6~18개월 재작성"은 과대 — v2 프런트엔드 파이프라인(syntax→sema→PublicIrRoot→ingress-bridge)이
> 양 repo 에 이미 존재함이 실측 확인되어, C 는 "입구 전환+패리티" 프로젝트로 재정의됨(F21 흡수).

> 대상: `semantica-codegraph-v2` 의 10개 `quanta-gqlang-*` v1 크레이트 삭제.
> 검증: 4개 병렬 Explore 어딧 (export/소비처/v2-대체재/전략).

## 🔴 헤드라인 (가정 정정)

**"v1 지우고 v2로 옮기면 됨"은 대부분 성립하지 않습니다.** 실측 결론:

- v2 표면(`exec-contract`/`host-adapter`/`ingress-bridge`)은 **이미 파싱·타입체크된 spec(`RootSpecV1`)을 받아 plan으로 lower 하는 다운스트림**입니다.
- v1 크레이트들은 **그 위(업스트림)** — 텍스트 파싱→AST→typecheck→HIR→catalog binding→spec — 인데, **이 스택의 v2 대체재가 semantica repo 안에 없습니다.**
- v2 파서(`quanta-gqlang-v2-syntax`)는 **external repo(quanta-gqlang-v2)에 있고 semantica에 통합 안 됨.** `RootSpecV1`은 실행 spec이지 parse AST가 아님.
- → "v2로 옮긴다" = 대부분 **새 v2 home에 로직을 PORT(복사 이관)** 하거나, core/lowering의 경우 **v2 파서를 semantica에 통합 + gateway/execution 재작성**(6~18개월 재아키텍처). 단순 repoint 아님.

## 크레이트별 실측 (LoC / v2대체재 / 전략 / 리스크)

| 크레이트 | LoC | v2 대체재 | 전략 | 리스크 | Phase |
|---|---|---|---|---|---|
| **gates-v1** | 5908 | 없음 | **DROP-DEAD** (runtime prod 미사용 — Cargo dep만, 실제 import 0; conformance dev-dep만) | 매우낮음 | **A** |
| **catalog-v1** | 5960 | 없음 | **DROP-DEAD-ish** (소비 대부분 test/bench/feature-gate; `operator_blocker_surface_rows()` 1개 함수만 v2 home로 이동 + execution embedded fixture 제거) | 낮음 | **A** |
| **fragment-v1** | 281 | 없음 | **PORT** (FragmentIdV1/RejectionV1 enum → 새 v2 home, 5소비처 repoint) | 낮음 | **B** |
| **validation-v1** | 1355 | 없음 | **PORT** (10 boundary newtype → v2 home, 8 소비처, serde-wire 결합) | **높음**(serde 직렬화 호환) | **B** |
| **sarif-projection-v1** | 1601 | 없음 | **PORT** (NegativePhraseRegistry + project_envelope_to_sarif → v2 home, observation-wire+conformance repoint) | 중간(LIVE observation-wire) | **B** |
| **sdk-handlers-v1** | 6155 | 없음 | **PORT or KEEP** (외부 SDK API 스키마; command-owner-surface가 재공표. hir-v1 삭제 후 단일소비. HANDLER_SCHEMA_VERSION 외부계약이면 유지 후보) | 중간 | **B/C** |
| **catalog-binding-v1** | 2245 | **부분**(envelope-wire가 이미 재공표→v2 exec-contract) | **PROMOTE** (PrecisionClassV1/EvidenceClassV1/UnsupportedReasonCodeV1를 authority-support로 승격 + 7소비처 repoint) | **높음**(wire 직렬화 호환 — envelope에 serialize됨) | **C** |
| **hir-v1** | 14638 | 없음 | **재아키텍처 필요** (Epic2 HIR SSOT — node taxonomy/type universe/predicate truth/stratification/lowering. planner-wire가 전체 재공표. v2 등가물 0) | **매우높음** | **C** |
| **lowering** | 2673 | 없음(파서) | **재아키텍처 필요** (유일한 v1 텍스트 파서, nom 기반. gateway surface_factory가 LIVE 소비) | **치명적**(텍스트 입력 전부 깨짐) | **C** |
| **core** | 11881 | 없음(AST) | **재아키텍처 필요** (v1 parse AST+typecheck+builtins. runtime/execution/query-owner-surface LIVE 소비) | **치명적** | **C** |

## 의존 순서 (topo)
```
fragment-v1, validation-v1 (leaf)
   ↓
catalog-binding-v1 ──→ catalog-v1
sdk-handlers-v1 ──→ hir-v1
core ──→ lowering
```
삭제는 leaf→상위 순. 단 상위(core/lowering/hir/catalog-binding)는 Phase C(재아키텍처)라 실제론 막힘.

## 단계별 계획

### Phase A — 진짜 지금 삭제 가능 (저위험, 즉시)
1. **gates-v1 DROP**: `quanta-runtime/Cargo.toml`에서 미사용 dep 제거 → `cargo check -p quanta-runtime` → dir+member 삭제. conformance dev-dep는 유지(또는 conformance가 자체 fixture로). **gate-registry 5908줄은 실 prod 미사용 = dead.**
2. **catalog-v1 DROP**: `operator_blocker_surface_rows()`(language_claim_surface.rs)를 v2 home(authority-support 등)으로 이동 → adapters-security/bench repoint → execution의 `embedded-catalog-quarantine` fixture 제거 → dir 삭제.
- 각 단계 `cargo check -p <소비처>` green 후 진행.

### Phase B — PORT (중간, 라이브 wire 건드림 → 충돌주의)
3. **fragment-v1**: enum 2개를 `quanta-gqlang-v2-ir`(또는 신규 v2 vocab crate)로 복사 → 5소비처 use 경로 변경 → 삭제.
4. **validation-v1**: 10 newtype를 v2 home로 PORT, **serde round-trip 테스트 필수**(envelope에 직렬화됨) → 8소비처 repoint → 삭제.
5. **sarif-projection-v1**: NegativePhraseRegistry+SARIF projection을 v2 diagnostic home로 PORT → observation-wire+conformance repoint → 삭제.

### Phase C — 재아키텍처 (삭제 아님, 프로젝트)
6. **core/lowering**: external repo의 `quanta-gqlang-v2-syntax` 파서를 semantica에 통합 → gateway surface_factory를 v2 파서로 전환 → execution/typecheck를 v2 IR로 재작성 → 그 다음에야 lowering→core 삭제. **6~18개월.**
7. **hir-v1**: Epic2 HIR 의미론(14.6k줄)을 v2로 재구현 + planner-wire 재배선. v2 등가물 전무.
8. **catalog-binding-v1**: PrecisionClass/EvidenceClass를 authority-support로 승격 + wire 호환성 검증(envelope 직렬화).

## J06 closeout 영향
J06 DoD "전 S-티켓 complete = 전 크레이트 삭제"는 **Phase C(재아키텍처) 완료 전엔 불가**. → J06은 Phase A/B로 일부 진전 가능하나, 완전 closeout은 core/lowering/hir cutover(대형)에 종속.

## 충돌 주의
Phase B/C는 `quanta-runtime`/gateway/wire 크레이트를 건드리는데, **동시 에이전트가 지금 그 영역(taint/xss/memssa)에서 실시간 커밋 중** → 정면 충돌. Phase A(gates/catalog-v1 dep 제거)는 비교적 격리됨.

## 권고
- **지금 안전하게 가능**: Phase A (gates-v1 dead-drop + catalog-v1 정리) — 2개 크레이트 실삭제.
- **Phase B**: 동시작업 정리 후 (serde 호환 검증 동반).
- **Phase C**: 별도 대형 프로젝트로 계획 (core/lowering/hir = "삭제"가 아니라 v2 파서 통합 + 스택 재작성).
