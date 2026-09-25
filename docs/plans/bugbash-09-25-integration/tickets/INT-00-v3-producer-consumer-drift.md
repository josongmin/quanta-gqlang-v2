# INT-00 — 현재 V3 producer/consumer DTO·digest 드리프트

상태: **현재 pair owner RED 재현 / scope 거부는 반영됨, digest 동기화 미완료** · 우선순위: P1 통합 선행조건 · 소유: Semantica composition/ingress integrator. QGLang producer 기준은 clean-qualified `39f729171243c07d1f6dc5cdc2d6129731b7c6ad`다.

## 현재 재검증 근거 (2026-09-25)

- QGLang의 committed `ExecutionCompositionProducerV3::Sample`은 `scope: Option<ExecutionGraphTargetV3>`를 포함하고 Graph producer/body는 완전한 query payload를 지닌다. schema-5 contract/qualification encoding과 facade export가 같은 `39f72917…`에 있으며 `just qualify-local`은 `1248/1248`, skip/ignore/leak 0으로 통과했다.
- Semantica `composition_plan_v3.rs`는 이제 `scope: Some(_)`를 `gqlang_v3_composition_sample_scope_unsupported`로 효과 전에 거부하고, `scope: None`만 producer plan으로 내린다. `rejects_scoped_sample_before_producer_plan_or_provider_admission` owner test도 존재한다. 기존 문서의 “구형 전개 패턴/정책 없음” 주장은 stale하다.
- 남은 실제 drift는 golden digest다. QGLang `composition_limit_2`는 source `d47176ffd6ba1ce41f1e6799c4228e3261e634cea9d3413b5c626a4ed5ff4731`, contract `1cfc163f7d8f89f7680fa48c482351e94c4117d30a0431bca09b6bee03d35926`, qualification `444de54f8a520136e3777266e9f94c5187e3ce2280a167f5c2d5fcdc457658b9`다. Semantica는 contract `41d4b8d3…`, qualification `a61355ae…`를 핀한다.
- exact owner 명령 `cargo test -p quanta-runtime-public-ingress-surface --features gqlang-v3-admission consumer_matches_pinned_frontend_v3_composition_golden_vector --lib -- --nocapture`는 1개를 선택해 `0 passed / 1 failed`였고 contract digest assertion에서 실제 `1cfc163f…` 대 기대 `41d4b8d3…`로 실패했다. feature 없는 두 사전 실행은 해당 cfg 테스트를 0개 선택했으므로 증거로 세지 않는다.

## 해결 경계

1. QGLang producer 기준은 `39f72917…`과 그 clean qualification receipt로 고정한다. 새 producer 변경이 생기지 않는 한 upstream 재설계는 이 티켓의 남은 일이 아니다.
2. Semantica 골든 벡터를 확정된 QGLang producer 결과에서 갱신한다. source/contract/qualification 세 digest를 모두 비교하고, 단순 상수 교체만으로 완료 처리하지 않는다.
3. `scope=None` positive, `scope=Some` typed reject, graph producer typed reject가 같은 owner rail에서 유지되는지 확인한다. scoped SAMPLE 또는 Graph 실행 지원을 새로 열지 않는다.
4. 동일한 고정 pair에 대해 등록된 Semantica QBC runtime-execution·public-ingress·SDK owner rail을 실행한 뒤 INT-04의 설치형 process 증명을 다시 받는다. Semantica의 QBC source digest가 sibling QGLang 소스까지 포함해야 한다. 별도 legacy V1 facade export나 호환 shim으로 우회하지 않는다.

## 완료 조건

- 고정된 양쪽 HEAD/dirty/source digest에 대해 Semantica 소비자 compile/owner 테스트가 통과한다. QGLang producer·external consumer는 `39f72917…` clean receipt로 충족됐다. 실행한 target, features, selected/executed count, exit를 기록한다.
- unscoped SAMPLE의 세 digest가 실제 QGLang producer 벡터와 Semantica consumer 기대값에서 일치하며, scoped SAMPLE은 명시된 지원 또는 typed reject로 검증된다. 새 Graph payload는 미지원 경계에서 효과 전에 거부된다.
- 다음 단계 INT-01~04의 영수증이 같은 고정 pair에 묶인다. QGLang 자체 receipt와 Semantica QBC digest는 서로 다른 도구의 값이므로 문자열 동일성을 요구하지 않고, 각자 포함한 소스 집합과 pair identity를 대조한다. 소스가 이동하면 영향받은 판정을 재검증한다.
