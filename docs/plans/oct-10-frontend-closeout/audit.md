# 최종 정적 감사 — 기존 frontend 마감

이 파일은 초기 구조/설정 감사의 고정 입력과 결과다. 후속 production AST 감사에서
확인·수정한 Modules 중복2묶음과 실행 결과는 [dedup-audit.md](dedup-audit.md)를 따른다.
아래 초기 감사의 Rust/CI NOT_RUN을 후속 실행 결과와 혼합하지 않는다.

## 입력과 증거 경계

| 항목 | 관찰값 |
| --- | --- |
| 일자 | 2026-10-10, Asia/Seoul |
| QGLang branch / HEAD | `main` / `5e5164dbcb1493a09dceecab4e1f3a23b1c09b3c` |
| QGLang HEAD tree | `44cc7206a581cfeba750a17452919ea6601e294a` |
| QGLang 시작 상태 | `git status --short` 출력 없음, clean |
| Semantica 관찰 HEAD | `a126366c202367b95bc1bb52ec249739fcc4802c` + shared WIP |
| Semantica 상태 | 해당 관찰에서 porcelain 항목 853개; committed HEAD와 WIP를 같은 증명으로 취급하지 않음 |
| 감사 후 변경 | 이 패킷 및 remaining.md 링크; Rust/계약/Git index 변경 없음 |

실행한 것은 Git 조회, 현재 소스/정책/테스트 본문 읽기, 물리 행수,
subtree ID와 SHA-256 및 선언된 pin 대조다. paired 값 대조는 Python 3.12로 수행했다.
Semantica의 mutable WIP는 read-only point-in-time 관찰이며 전체 snapshot freeze가 아니다.

## 확인한 완료 소스 — 재구현에서 제외

| 기존 요구 | 현재 근거 | 판정 |
| --- | --- | --- |
| 로컬 CI와 MSRV | [Justfile](../../../Justfile)의 static에 size-lint, clippy-current, clippy-msrv; ci는 기존 run_local composition 사용 | 구현 존재, 최신 실행 NOT_RUN |
| God 파일 게이트 | [size_policy.json](../../../tools/lint/size_policy.json): production 1200/test 2500 **Tokei code lines**, exceptions 없음. [size_gate.py](../../../tools/lint/size_gate.py)는 test mount와 production function-size lint 우회도 검사 | 구현 존재. 물리 행수와 gate 판정 구분 |
| document 분할 | [document.rs](../../../crates/quanta-gqlang-v2-sema/src/document.rs): 672 physical lines, semantic/materialization/inputs/session 등의 child mount 존재 | 구조 분할 존재 |
| execution 분할 | [execution.rs](../../../crates/quanta-gqlang-v2-pipeline/src/execution.rs): 1375 physical lines, encoding/projection/vocabulary/scalar_request/test child 사용 | 구조 분할 존재. 1375를 곧바로 1200-code-line gate 실패로 해석하지 않음 |
| types 분할 | [check.rs](../../../crates/quanta-gqlang-v2-types/src/check.rs) 1197 physical lines, inputs/value/scalar_request; [solver.rs](../../../crates/quanta-gqlang-v2-types/src/solver.rs) 1136, engine child | 구조 분할 존재 |
| test support 정리 | [test_support.rs](../../../crates/quanta-gqlang-v2-sema/src/test_support.rs) 2348 physical lines, child 테스트/fixture 책임 분리; size gate의 feature/test guard 검사 | 구조 분할 존재 |
| mega test 분할 | [document transaction](../../../crates/quanta-gqlang-v2-sema/tests/v3_document_transaction.rs), [compiled assembly](../../../crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs)의 child suites | 분할 존재. 모든 테스트가 작다는 주장은 아님 |
| examples closed authority | [self_validation.rs](../../../crates/quanta-gqlang-v2-conformance/tests/self_validation.rs): 실제 파일 10 planned + 7 typed non-plan + export1 exact rejection; shipped examples18 | 기존 coverage 존재, 이번 실행 NOT_RUN |
| DTO 문서 대응 | [public DTO field map](../../contracts/frontend-execution-v3.md#public-dto-field-map), producer golden vectors와 외부 facade 소비 테스트 | 기존 계약/회귀 존재. 전체 field drift 실행 판정은 qualification에 남음 |

`ExecutionDocumentV3` 선언 owner는 Pipeline이고 frontend-contract는 curated facade다.
물리적 DTO 이동은 완료된 것으로 기록하지 않는다. authored `result_shape`는 현재 typed rejection이
명시된 미지원 기능이며 이번 마감에서 새 기능으로 구현하지 않는다.

## F01 — closeout 계약 pin 일치: STATIC_MATCH

Semantica의 [closeout gate](../../../../semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh)
`sibling_frontend_tree`, `sibling_pipeline_tree`, `sibling_vector_sha256` 비교와 실제 producer를 대조했다.

| 입력 | producer 관찰값 = consumer 기대값 |
| --- | --- |
| frontend-contract subtree | `325a744628a282f875fc5e080ffc639672f14d9b` |
| pipeline subtree | `be0572f3bfa2ed28ee2a369a3379f564dd3407e9` |
| execution-v3-vectors.tsv SHA-256 | `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb` |

실제 runtime-execution Cargo 의존성은 sibling frontend-contract path를 사용한다.
tree/vector 세 값의 일치는 정적 custody 대조다. full closeout gate, ABI compile,
SDK/runtime/installed 결과의 성공 증명은 아니다. 위 두 subtree가 frontend 전체 의존 closure를 대신하지도 않는다.

## F02 — external-workspace 전체 revision 불일치: OPEN_PAIRING

- [external-rust-workspaces-v1.toml](../../../../semantica-codegraph-v2/tools/ci/contracts/external-rust-workspaces-v1.toml)
  `workspaces.gqlang_v2.revision`은 `9060e46997dd9c62080a18bdfae06f8c162442e2`다.
- [bootstrap verify_checkout](../../../../semantica-codegraph-v2/tools/ci/external_rust_workspace_bootstrap_v1.py)
  는 전체 HEAD가 이 revision과 같고 clean이어야 한다.
- 현재 sibling HEAD `5e5164d…`는 다르다. old pin→현재 frontend/pipeline scoped diff도 비어 있지 않다
  (25 files). 단순 문서 SHA 변경만으로 설명되는 차이가 아니다.
- 따라서 현재 sibling을 그 verifier에 넘기면 HEAD 비교 조건을 만족하지 않는다.
  bootstrap 전체 실행이나 CI failure를 새로 재현한 것은 아니다. 고정된 과거 checkout을 쓰는
  bootstrap 자체가 잘못됐다는 뜻도 아니다.
- 원인: closeout은 subtree/vector pin을, bootstrap은 과거 전체 revision을 사용한다.
  최신 paired acceptance의 입력 정책을 Semantica E13 owner와 맞춰야 한다(QFC-02).
  기대값만 바꾸거나 다른 source의 결과를 합쳐 완료 처리하지 않는다.

## F03 — 최신 clean qualification 증거: PENDING

`target/qualification/5e5164dbcb1493a09dceecab4e1f3a23b1c09b3c/receipt.json`은 관찰 시 없었다.
다른 위치의 receipt까지 없다는 주장은 아니다. 기존 R334 CI1514 등은 역사적 source 결과다.
이 패킷 작성 후 문서가 dirty가 되므로 최종 committed SHA를 다시 고정해야 한다.

| 검증 | 이번 감사 결과 |
| --- | --- |
| source/module/policy/explicit test oracle readback | 수행 |
| paired subtree2/vector1 비교 | MATCH |
| bootstrap revision 비교 | MISMATCH, 원인·owner 기록 |
| size-lint / Clippy / Rust tests / external-consumer | NOT_RUN |
| `just ci` / `just qualify-local` | NOT_RUN |
| Semantica QBC / Graph/CHECK / installed Product | NOT_RUN, downstream owner 범위 |

## 중복 작업 경계

Semantica factory guard5 통합은 기존 `E01-DUP-GUARD`, legacy Source/Exact·Shared·CAS 및
HIR/ItemIndex 완료 정책은 기존 E01/E03/E04/E11, V1/V3 common-owner 검토는 E18로 연결돼 있다.
이 패킷은 그 구현을 QGLang에 발급하지 않는다. Shape/SCC canonical source를 다시 구현하지 않는다.
기존 Parent18·original25를 더하거나 새로운 acceptance 분모로 바꾸지 않는다.

## 문서 작성 후 검사

패킷3개+remaining.md의 local link77개, QFC-01~05 heading 유일성, actions.md 외
중복 체크박스 없음, whitespace를 검사했다(PASS). 최종 postflight에서 QGLang HEAD는
동일하고 Rust/tooling diff는 없으며 subtree2/vector1 일치를 다시 확인했다.
stage/commit/push 및 CI/qualification 실행은 하지 않았다.

## 후속 source 변경 안내

위 NOT_RUN 및 Rust/tooling diff 없음은 초기 snapshot의 결과다. 후속 D01/D02 공통화와
D03 SCC common-owner 전환, actual caller 검증 및 현재 HEAD receipt 판정은
[dedup-audit.md](dedup-audit.md)와 [actions.md](actions.md)가 소유한다.
초기 구조 감사의 상태를 최신 source qualification으로 재사용하지 않는다.
