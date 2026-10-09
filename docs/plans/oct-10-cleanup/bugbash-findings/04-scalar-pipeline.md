# Bounded scalar Pipeline 감사 (lane 03)

- 고정 소스: QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`.
- 범위: 공개 scalar source/parsed 진입부터 Pipeline stage 소유자 연결, Execution projection/seal, shared owner와 canonical-files borrowed read. 미커밋 문서와 downstream 실행은 이 소스 판단에 포함하지 않았다.
- **확인된 현재 소스 결함: 없음.** 이는 전체 frontend나 host 저장소 구현의 무결성 판정이 아니다.

## 도달 경로와 적대 대조

| caller → owner | 실패/호환 불변식 | 현재 증거 |
| --- | --- | --- |
| `compile_scalar_source_with_request_owner_v3` → parser-issued owner → checked stages | 단일 bare Query 관계 root만 통과하고, parsed budget 불일치·다른 root/source는 다음 stage debit 전에 거부되어야 한다. | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/scalar_compile.rs:50-109`의 두 진입점과 stage 순서; `tests/v3_compiled_document_assembly/scalar_request_tests.rs:294-373,629-672`의 budget/foreign/unsupported/one-short 회귀. |
| typed Sema/Algebra/Planning → Execution projection | Source→Project의 순서·identity·cost/effect/capability를 보존하고, 지원하지 않는 expression/shape는 typed refusal로 끝나야 한다. | `crates/quanta-gqlang-v2-execution-contract/src/scalar_request.rs:62-160`이 projection 전 유한 shape와 demand를 검사한다. `.../scalar_request_tests.rs:196-239,375-460`은 원본 컴파일과 비교하고 고정된 세 digest 기대값을 검증한다. |
| `project_source_body_with_request_owner_v3` → `seal_scalar_execution_document_with_request_owner_v3` | projection payload와 문서 item storage를 원래 request root에 예약한 뒤 발행한다. 1바이트 부족하면 결과가 발행되지 않고 기존 보관량은 회수된다. | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/sealed/execution.rs:386-466`; `.../scalar_request_tests.rs:462-538`의 exact/one-short debit 및 last-drop 검사. |
| admitted → shared owner → canonical read | DTO deep clone 없이 동일 payload/permit을 공유하며 마지막 clone drop 전에는 charge를 유지한다. canonical authority와 정확한 Scan→Project·RepoPath 형태가 아니면 borrowed read가 없다. | 같은 execution 소유자 `:135-284,302-361`; `.../scalar_request_tests.rs:540-627`의 foreign sink, 공유 identity, last-drop 및 noncanonical 거부. |

## 실행 결과

`cargo test -p quanta-gqlang-v2-pipeline --test v3_compiled_document_assembly scalar_request --locked`를 현재 고정 HEAD에서 실행했다. **10 passed, 0 failed, 0 ignored, 65 filtered out**. 독립 고정 digest, 일반 컴파일 parity, foreign owner, exact/short storage, shared clone 생애주기와 긴 이름 입력을 포함한다. 이 필터의 통과는 미선택 65개나 전체 CI의 통과를 주장하지 않는다.

## 남은 검증 경계

- 저장소 계산은 논리 payload와 유한 capacity 상한이다. allocator metadata/process RSS의 상한이나 성능 개선은 이 테스트로 입증되지 않는다. 별도 대표 workload·peak allocation/RSS 측정 없이는 성능 finding을 닫지 않는다.
- 독립 host의 `FrontendRequestStorageSinkV3` 구현, Semantica admission·provider/lease 결합, 설치 프로세스는 이 레인의 실행 범위 밖이다. 기존 clean-HEAD QGLang 로컬 영수증과 이 focused 테스트를 downstream 증거로 합치지 않는다.
- 이번 레인에서 Rust 소스는 수정하지 않았고 `just ci`·`just qualify-local`은 실행하지 않았다.
