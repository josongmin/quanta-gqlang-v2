# 추가 레퍼런스 감사의 정정 기록

날짜: 2026-10-04. 이 문서는 감사 당시 변경한 결정을 기록한다.
현행 [RFC](../rfcs/2026-10-04-code-query-dsl-and-substrate.md) · [액션 리스트](../rfcs/2026-10-04-code-query-action-list.md) · [근거 인덱스](./2026-10-04-reference-source-review.md)

## 범위

Calcite·Differential Dataflow·Kuzu를 독립 병렬 감사하고 egg를 별도 검토했다. 이전 6개와 합쳐 10개 clone을 고정했다. 상세 코드·SHA·행 근거는 근거 인덱스가 연결한다. 이 추가 감사에서는 build/test·benchmark를 실행하지 않았다.

## 채택한 정정

| 이전 표현의 문제 | 최종 결정 |
| --- | --- |
| 비용 planner 존재를 일반 join optimizer로 해석 | 기존 dependency planner와 관계형 접근/join 대안을 구분. 실행 baseline부터 연결 |
| Optimizer cutoff와 실행 불완전을 혼동 | 유효한 best-known plan은 실행 가능; request cancel/resource/incomplete fact는 terminal |
| 통계가 후보 eligibility에 전혀 영향 없다고 표현 | Estimated-budget 선택에는 영향 가능; semantic capability·exactness·coverage는 증명하지 못함 |
| A12가 unrelated recursion/flow까지 기다림 | 바꾸는 연산자의 실행·oracle·계측만 선행 |
| Endpoint BFS로 path 의미를 대신함 | Traversal scope·edge/orientation identity·repetition·multiplicity·대표 정책 명시 |
| 증분을 cache key 변경으로 설명 | Owner별 replay-safe insert/delete epoch·전체 완료·fresh recompute oracle 요구 |
| 유지 결과를 retention gate보다 먼저 공개 가능 | Publication·lease/compaction·replay/delete oracles를 함께 통과해야 공개 |
| 레퍼런스와 구조가 닮으면 dependency 채택 | 동일 workload의 의미·자원·적응 비용 측정 후 결정. Kuzu는 historical 참고 |

## 기록과 현재 상태

감사 당시 `extension-local-source-basis.json`, `extension-verification.json`을 생성했다. 이후 문서 편집으로 과거 document digest는 현행 파일의 검증이 아니다. 이번 정리는 근거 인덱스의 source 재확인과 `document-cleanup-verification.json`을 따른다.

전체 source·제품 qualification은 미실행이다. 최초 installed two-fact join, 이후 재귀·모델 기반 flow/CHECK라는 목표는 유지한다. 구현·완료 조건은 액션 리스트 한 곳에서 관리한다.
