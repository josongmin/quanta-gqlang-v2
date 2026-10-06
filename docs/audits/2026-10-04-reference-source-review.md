# Code-query 소스 감사: 근거 인덱스

갱신: 2026-10-04. 이 문서는 source 관찰·고정 upstream 근거를 관리한다.
현행 설계는 [RFC](../rfcs/2026-10-04-code-query-dsl-and-substrate.md), 실행 순서·완료 조건은 [액션 리스트](../rfcs/2026-10-04-code-query-action-list.md)를 따른다. 아래 기록만으로 제품 기능·성능이 검증된 것은 아니다.

## 로컬 source 재확인과 stale 정정

| 확인된 source | 관찰과 문서 반영 |
| --- | --- |
| [compiled rule program](../../crates/quanta-gqlang-v2-pipeline/src/compile.rs), [logic projection](../../crates/quanta-gqlang-v2-logic/src/source.rs) | Membership·plan·stratification proof는 존재한다. 이를 generic typed rule body의 공개 실행 완료로 해석하지 않는다. |
| [V3 relation admission](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/relation_plan_v3.rs) | 단일 `RepoPath`; Scan/Filter/Project/SemiJoin/AntiJoin/ExistsFilter subset. “join 전혀 없음”과 “일반 typed multicolumn join 완료” 모두 부정확하다. |
| [SDK CHECK caller](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler.rs), [CHECK dispatch](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/gqlang_handler/check_dispatch_v3.rs) | Issued admission→gateway grant→실행→read-only completion 경로가 있다. **과거의 CHECK 항상 미연결 주장은 폐기한다.** 새 모델 DSL 전체 지원이나 설치 proof를 뜻하지 않는다. |
| [physical lowering](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/physical_plan_v1/lowering.rs), [planner enumeration](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-planner/src/enumerate.rs) | 기존 실행 bridge와 비용 planner가 있다. 관찰한 enumerator는 순차·병렬 dependency 후보이며 관계형 join-order memo와 다르다. |
| [recursive executor](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-kernel-contract/src/domain/values/recursive_executor.rs) | Full-rule sweep과 최종 no-new-tuples sweep의 shared budget을 구현한다. “수렴 sweep이 무료” 또는 “실행기 없음”으로 기술하지 않는다. Typed public 경로·delta·자원 확장은 별도 gate다. |
| [relation rows](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/src/result.rs) | `RowPageV1`의 cell은 문자열이다. 다른 graph/finding payload까지 문자열이라고 일반화하지 않는다. |
| [registered search](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/search_handler.rs), [generation-bound adapter](../../../semantica-codegraph-v2/packages/analysis/quanta-v2/crates/quanta-runtime-retrieval-kernel/src/index_sdk_ingress/generation_bound_search_index_v1.rs) | Capture-once pin·응답 pin 검증이 이미 있다. 직접 capture caller의 required generation은 `None`; 이것은 analysis/corpus 공동 발행 검증 완료의 증거가 아니며 실제 generation 혼합 사고의 증거도 아니다. |

이번 관찰 중 QGLang은 `9060e469`+dirty, Semantica는 `c24cc86e`→`3ef1368b`, Index는 `52a766de`였다. Source가 움직이는 shared checkout이므로 frozen qualification tuple이 아니다. 명시 파일 hash/status는 `document-cleanup-source-basis.json`에 기록한다. [기존 V3 수리 계획](./2026-10-04-frontend-consumer-contract-plan.md)은 해당 owner의 상태·receipt를 보존하며 이 문서가 덮어쓰지 않는다.

## 실제 clone한 레퍼런스

Clone 위치: `/Users/songmin/Documents/code-new/.reference-audits/2026-10-04-code-query/`.
[전체 SHA·origin·checkout manifest](/Users/songmin/Documents/code-new/.reference-audits/2026-10-04-code-query/manifest.json)

| 고정 소스 / 상세 감사 | 확인한 패턴 | 적용 판단 |
| --- | --- | --- |
| [CodeQL](./reference-source/2026-10-04-codeql.md) `f7617abe` | Typed modules·models·flow/path query | 언어·모델 참고. 공개 evaluator 내부는 범위 밖 |
| [DataFusion](./reference-source/2026-10-04-datafusion.md) `9547b091` | Provider/pushdown·physical stream·memory/cancel | 실행 계약 참고; 동일 typed workload 후 채택 판단 |
| [Soufflé](./reference-source/2026-10-04-souffle.md) `f53dab8b` | Full clauses·SCC·delta/index/provenance | 재귀 참고. Size cutoff를 완료로 복사하지 않음 |
| [Joern](./reference-source/2026-10-04-joern.md) `bf4f3434` | Context·외부 model·endpoint/path·solver cap | Flow 참고. 이 pin의 대표 선택은 universal shortest가 아님 |
| [Substrait](./reference-source/2026-10-04-substrait.md) `ab7eff6b` | Versioned schema·required semantics·consumer subset | 계약 검증 참고; 전체 IR 교체는 채택하지 않음 |
| [Datafrog](./reference-source/2026-10-04-datafrog.md) `edd7cfc0` | Rust finite tuple/delta kernel | 선택적 재귀 prototype; hard memory/cancel 보강 필요 |
| [Calcite](./reference-source/2026-10-04-calcite.md) `fe1c46d2` | Guarded rewrites·memo/traits·cost·optimizer stop | 작은 합법적 대안부터. Java/full memo 도입 보류 |
| [egg](./reference-source/2026-10-04-egg.md) `73975c98` | Rust equality saturation·cost extraction | 선택적 optimizer prototype; 사용자 제공 equality가 의미 증명은 아님 |
| [Differential Dataflow](./reference-source/2026-10-04-differential-dataflow.md) `aa8745f9` | Signed updates·frontiers·arrangements·compaction | 이후 증분 후보; source delta/lease/resource 계약 필요 |
| [Kuzu](./reference-source/2026-10-04-kuzu.md) `89f0263c` | Adjacency·recursive expand·path multiplicity | Archived 역사적 참고. 현행 dependency로 추천하지 않음 |

신규 production dependency는 선택하지 않았다. [추가 감사의 정정 기록](./2026-10-04-reference-source-review-extension.md)은 유지하되 설계·액션을 다시 복제하지 않는다.

## 계약 보강에 사용한 공개 문헌

- [FlowLog, PVLDB 19(3), 361–374](https://www.vldb.org/pvldb/vol19/p361-zhao.pdf): rule별 relational IR과 recursive control 분리, robustness·SIPS/early filtering, batch/incremental 실행 참고. 논문은 magic sets를 후속 연구로 둔다. SIPS와 demand specialization을 같은 구현으로 취급하지 않는다. 이번에는 논문을 확인했으며 FlowLog artifact clone·실행 감사는 하지 않았다.
- [CodeQL diff-informed hook](https://codeql.github.com/codeql-standard-libraries/java/codeql/dataflow/DataFlow.qll/predicate.DataFlow%24Configs%24ConfigSig%24observeDiffInformedIncrementalMode.html): 변경 관련 source/sink 필터. Full incremental evaluator의 공개 근거가 아니다.
- [Soufflé provenance](https://souffle-lang.github.io/provenance): lazy tuple proof와 사용자가 rule/binding을 지정하는 부재 설명. 일반 자동 why-not을 첫 계약에 넣을 근거로 사용하지 않는다.

이 자료는 신규 계약의 설계 근거이며 QGLang의 SOTA·CodeQL 우위·성능 또는 “6개 보강으로 충분함”을 증명하지 않는다.

## 증거의 범위

- Upstream 문서는 **각 고정 SHA의 감사 기록**이다. 로컬 구현·계획 언급은 그때의 관찰이며 현재 상태로 재사용하려면 source를 다시 확인한다.
- 기존 Datafrog probe는 Rust 직접 빌드·테스트 3개를 통과했다. 3노드 그래프 512개의 linear/nonlinear closure, static antijoin, indistinct cycle을 확인했다. 일반 DSL·성능·제품 통합 증명이 아니다.
- 문서 정리의 검증은 링크·고정 SHA/파일/행 범위·whitespace와 diagram fence다. 기존 결과는 `document-cleanup-verification.json`, 이번 계약 보강 결과는 `rfc-contract-revision-verification.json`에 남긴다.
- 추가 upstream build/test, CodeQL CLI, 제품 실행, 성능 비교, `just ci`는 이번 작업에서 실행하지 않았다. 문서는 미커밋 상태다.
