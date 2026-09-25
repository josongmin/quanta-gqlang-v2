# INT-00 — 현재 V3 producer/consumer DTO·digest 드리프트

상태: **현재 파일 간 소스 불일치 / 컴파일·실행 판정 전** · 우선순위: P1 통합 선행조건 · 소유: QGLang frontend-contract/pipeline + Semantica composition/ingress integrator.

## 재감사 근거 (2026-09-25, 이동 중인 작업 트리)

- QGLang의 미커밋 `Q:crates/quanta-gqlang-v2-pipeline/src/execution.rs:1042-1055`에서 `ExecutionCompositionProducerV3::Sample`에 `scope: Option<ExecutionGraphTargetV3>`가 추가됐다. Graph producer/body도 단순 kind 대신 완전한 query payload를 지닌다 (`:998-1003,1153-1195`). 공개 facade가 새 graph DTO를 내보낸다: `Q:crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:30-45`. 계약 문서와 digest 인코더도 함께 변경 중이다: `Q:docs/contracts/frontend-execution-v3.md:20,99`; `Q:crates/quanta-gqlang-v2-pipeline/src/execution.rs:1901-1945`.
- Semantica의 현재 `S:packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/composition_plan_v3.rs:871-896`은 `Sample { query_text, mode, limit }`를 `..` 없이 분해한다. 현재 sibling path dependency의 새 필드를 처리하지 않는 **정적 패턴 불일치**다. 해당 파일에 scope admission/실행 정책도 아직 없다. QGLang이 이 변경을 그대로 확정한다면 Semantica는 새 필드를 명시 처리해야 한다. 이 감사에서는 rustc/QBC 결과를 실행하지 않았다.
- QGLang의 현재 미커밋 `Q:crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv:9`에서 `composition_limit_2`는 source digest `d47176ff...`가 그대로지만 contract digest `0e47a752bd6e47564879e41e98aea388a4383e613f8f8fc9f3bf8ddeae30b104`, qualification digest `4b131ed5ea49f0f150bb3cd731a780235afa22c03f1c507307b5af445d375f80`으로 바뀌었다. Semantica의 고정 기대값은 각각 `41d4b8d3c6484cf3506bc198273dc095dc7aa0a9222e9500abaccc08958d6803`, `a61355aefbb950f331a70bf8aaffcd41f61c6c19d4f677d158524dbcce894bab`이다 (`S:packages/analysis/quanta-v2/crates/quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs:627-650`). 따라서 기존 문서의 “골든 벡터 일치” 판정은 현 파일 기준으로 무효다.
- 이전 QGLang clean-HEAD qualification과 Semantica의 과거 QBC PASS는 이 미커밋 pair를 증명하지 않는다. 변경 중인 소스의 컴파일 실패나 런타임 오결과를 관찰했다고 주장하지 않는다.

## 해결 경계

1. QGLang owner가 V3 DTO·digest 인코딩·문서·producer 벡터를 같은 스냅샷에서 확정하고 owner-local 테스트 및 공개 external-consumer 검증을 받는다. 중간 상태의 벡터를 Semantica에 성급히 핀하지 않는다.
2. Semantica owner가 `Sample` 전개 패턴을 새 shape에 맞추고 `scope`의 정책을 명시한다. 현재 지원하지 않으면 **타입이 있는 admission 거부**로 효과 전에 막는다. `..`만 넣어 scope를 조용히 무시하지 않는다. 지원한다면 target 종류·binding capability·provider/generation authority·rowset 경계를 명세하고 양성/음성 테스트를 붙인다. Graph query는 기존 미지원 경계를 유지하며 새 DTO만으로 실행 가능하다고 선언하지 않는다.
3. Semantica 골든 벡터는 확정된 QGLang producer 결과에서 갱신한다. source/contract/qualification 세 digest를 모두 비교하고, 단순 상수 교체만으로 완료 처리하지 않는다. `scope=None`과 적어도 하나의 `scope=Some`에 대해 지원 또는 typed reject가 테스트로 드러나야 한다.
4. 동일한 고정 pair에 대해 등록된 Semantica QBC runtime-execution·public-ingress·SDK owner rail을 실행한 뒤 INT-04의 설치형 process 증명을 다시 받는다. Semantica의 QBC source digest가 sibling QGLang 소스까지 포함해야 한다. 별도 legacy V1 facade export나 호환 shim으로 우회하지 않는다.

## 완료 조건

- 고정된 양쪽 HEAD/dirty/source digest에 대해 QGLang producer·external consumer와 Semantica 소비자 컴파일/owner 테스트가 통과한다. 실행한 target, features, selected/executed count, exit를 기록한다.
- unscoped SAMPLE의 세 digest가 실제 QGLang producer 벡터와 Semantica consumer 기대값에서 일치하며, scoped SAMPLE은 명시된 지원 또는 typed reject로 검증된다. 새 Graph payload는 미지원 경계에서 효과 전에 거부된다.
- 다음 단계 INT-01~04의 영수증이 같은 고정 pair에 묶인다. QGLang 자체 receipt와 Semantica QBC digest는 서로 다른 도구의 값이므로 문자열 동일성을 요구하지 않고, 각자 포함한 소스 집합과 pair identity를 대조한다. 소스가 이동하면 영향받은 판정을 재검증한다.
