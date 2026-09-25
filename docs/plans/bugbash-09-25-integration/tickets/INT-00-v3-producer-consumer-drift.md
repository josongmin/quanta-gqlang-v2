# INT-00 — V3 Composition golden digest 비교 재판정

상태: **CLOSED — 원래 결함 판정 철회**. 두 golden 테스트는 같은 source bytes와 SourceId를 사용하지만 서로 다른 authority fixture로 DTO를 구성한다. contract/qualification digest의 문자열을 직접 같다고 요구한 것이 잘못이었다. Semantica 소유 테스트는 현재 소스에서 통과한다.

## 현재 재검증 근거 (2026-09-25)

- QGLang의 committed `ExecutionCompositionProducerV3::Sample`은 `scope: Option<ExecutionGraphTargetV3>`를 포함하고 Graph producer/body는 완전한 query payload를 지닌다. schema-5 contract/qualification encoding과 facade export가 같은 `145b6682…`에 있으며 `just qualify-local`은 `1248/1248`, skip/ignore/leak 0으로 통과했다.
- Semantica `composition_plan_v3.rs`는 이제 `scope: Some(_)`를 `gqlang_v3_composition_sample_scope_unsupported`로 효과 전에 거부하고, `scope: None`만 producer plan으로 내린다. `rejects_scoped_sample_before_producer_plan_or_provider_admission` owner test도 존재한다. 기존 문서의 “구형 전개 패턴/정책 없음” 주장은 stale하다.
- QGLang `composition_limit_2`의 `minimal_authority()`는 `ModuleAuthorityBuilderV2`를 직접 사용한다. Semantica `vector_authority()`는 `ClosedFrontendAuthorityBuilderV2`의 `finish()`를 사용한다. 계약상 contract/qualification digest는 source bytes만이 아니라 **투영된 DTO**의 fingerprint다 (`docs/contracts/frontend-execution-v3.md`). 같은 source digest만으로 나머지 두 digest의 동일성을 요구할 수 없다.
- 과거 raw Cargo RED는 이전 source/feature 조합의 진단이며 현재 pair의 판정으로 승격하지 않는다. 2026-09-26 현재 Semantica `dc91f9b8…` dirty snapshot + QGLang `7c1289ee…`에서 등록 QBC owner 명령 `scripts/quanta-build-cli owner run --lane gqlang-ss-integration --package quanta-runtime-public-ingress-surface --execution-kind test --target-kind lib --selector-mode module --selector admitted_execution_request_v3::tests --compile-policy feature-isolation:quanta-runtime-public-ingress-surface.no-default.fec264993b3e --ignored-policy exclude --max-test-threads 1`이 exit 0, **6 passed / 0 failed / 0 ignored**였다. 이 모듈에는 pinned golden 테스트가 포함된다. QBC source snapshot digest는 receipt에서 확인해야 하며 이 dirty-tree PASS를 clean-HEAD qualification으로 부르지 않는다.

## 해결 경계

1. QGLang producer 기준은 `145b6682…`과 그 clean qualification receipt로 고정한다. 새 producer 변경이 생기지 않는 한 upstream 재설계는 이 티켓의 남은 일이 아니다.
2. **두 pin을 서로 복사하지 않는다.** 서로 다른 authority fixture를 동일화할 제품 요구가 생기면 각 DTO 입력을 먼저 고정하고 새 공유 벡터를 정의한다.
3. scoped SAMPLE과 Graph producer의 현 미지원 경계 검증, SDK·설치형 증명은 INT-04에서 별도로 수행한다. legacy V1 facade export나 호환 shim은 만들지 않는다.

## 완료 조건

- 현재 Semantica fixture의 pinned golden 테스트가 등록 QBC owner에서 실제 선택·통과한다. QGLang의 별도 fixture digest와 문자열 동일성은 완료 조건이 아니다.
- downstream 전체 동작·설치형 제품 검증은 이 티켓이 아니라 INT-04의 별도 증명이다.
