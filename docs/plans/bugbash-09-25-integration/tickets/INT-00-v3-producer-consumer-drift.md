# INT-00 — V3 Composition golden digest 동기화

상태: **현재 pair 소스 불일치·raw Cargo 진단 RED / QBC owner 판정 전** · 우선순위: P1 통합 선행조건 · 소유: Semantica composition/ingress integrator. QGLang producer 기준은 clean-qualified `145b66827a8791c22c4df26fcfa11d82592f913f`다.

## 현재 재검증 근거 (2026-09-25)

- QGLang의 committed `ExecutionCompositionProducerV3::Sample`은 `scope: Option<ExecutionGraphTargetV3>`를 포함하고 Graph producer/body는 완전한 query payload를 지닌다. schema-5 contract/qualification encoding과 facade export가 같은 `145b6682…`에 있으며 `just qualify-local`은 `1248/1248`, skip/ignore/leak 0으로 통과했다.
- Semantica `composition_plan_v3.rs`는 이제 `scope: Some(_)`를 `gqlang_v3_composition_sample_scope_unsupported`로 효과 전에 거부하고, `scope: None`만 producer plan으로 내린다. `rejects_scoped_sample_before_producer_plan_or_provider_admission` owner test도 존재한다. 기존 문서의 “구형 전개 패턴/정책 없음” 주장은 stale하다.
- 남은 실제 drift는 golden digest다. QGLang `composition_limit_2`는 source `d47176ffd6ba1ce41f1e6799c4228e3261e634cea9d3413b5c626a4ed5ff4731`, contract `1cfc163f7d8f89f7680fa48c482351e94c4117d30a0431bca09b6bee03d35926`, qualification `444de54f8a520136e3777266e9f94c5187e3ce2280a167f5c2d5fcdc457658b9`다. Semantica는 contract `41d4b8d3…`, qualification `a61355ae…`를 핀한다.
- raw Cargo exact 진단 `cargo test -p quanta-runtime-public-ingress-surface --features gqlang-v3-admission consumer_matches_pinned_frontend_v3_composition_golden_vector --lib -- --nocapture`는 1개를 선택해 `0 passed / 1 failed`였고 contract digest assertion에서 실제 `1cfc163f…` 대 기대 `41d4b8d3…`로 실패했다. feature 없는 두 사전 실행은 해당 cfg 테스트를 0개 선택했으므로 증거로 세지 않는다. Semantica의 최종 Rust verdict는 QBC가 소유한다.

## 해결 경계

1. QGLang producer 기준은 `145b6682…`과 그 clean qualification receipt로 고정한다. 새 producer 변경이 생기지 않는 한 upstream 재설계는 이 티켓의 남은 일이 아니다.
2. Semantica `admitted_execution_request_v3.rs`의 contract/qualification 고정값 두 개를 QGLang `execution-v3-vectors.tsv`의 `composition_limit_2`와 맞춘다. source digest는 이미 일치하므로 바꾸지 않는다. 테스트는 producer를 실제 컴파일해 세 값을 비교하는 기존 구조를 유지한다.
3. 동일 pair의 등록 QBC public-ingress owner에서 golden test가 1개 선택·실행돼 통과하는지 확인한다. runtime-execution owner의 기존 `scope=Some` 선행 typed reject와 Graph producer reject도 선택·실행한다. 새 지원 기능이나 QGLang 수정은 이 티켓에 넣지 않는다.
4. 이후 SDK GQLang owner는 INT-04에서 한 번 실행한다. Semantica QBC source digest에 sibling QGLang 소스가 포함돼야 한다. legacy V1 facade export나 호환 shim으로 우회하지 않는다.

## 완료 조건

- 고정된 양쪽 HEAD/dirty/source digest에 대해 등록 QBC public-ingress·runtime-execution owner 테스트가 통과한다. QGLang producer·external consumer는 `145b6682…` clean receipt로 충족됐다. 실행한 target, features, selected/executed count, exit를 기록한다.
- unscoped SAMPLE의 세 digest가 실제 QGLang producer 벡터와 Semantica consumer 기대값에서 일치하며, scoped SAMPLE과 Graph payload는 현재 미지원 경계에서 효과 전에 typed reject된다.
- 이 owner 영수증이 이후 INT-04의 고정 pair에 묶인다. QGLang receipt와 Semantica QBC digest는 서로 다른 소스 집합의 값이므로 문자열 동일성을 요구하지 않는다. 소스가 이동하면 영향받은 판정만 재검증한다.
