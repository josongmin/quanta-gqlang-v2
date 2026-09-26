# F-04 / F-05 / F-09 재감사

기준 HEAD: `3b403bbb3d2d02a03d0d1125d770467841d0d640`. 시작 working tree는 clean.
범위는 직전 상태 확인에서 남았던 세 항목이다. 최초 보고서 전체나 downstream
runtime qualification을 다시 수행한 결과가 아니다.

## 판정과 수용 기준

| 항목 | 재판정 | 처리 / 수용 기준 |
| --- | --- | --- |
| F-04 | frontend 동작 결함은 반증. 공개 facade 분류의 제한된 검증 공백 | Declaration/Rule/Catalog 원문을 closed authority 없이 facade에서 분류하고 정확한 enum을 독립 리터럴로 검증한다. runtime 지원 정책이나 가짜 typed-rejection adapter는 추가하지 않는다. |
| F-05 | 현재 SSOT 결함은 반증. enum/const 모듈 추출은 선택적 리팩터링 | production 발행처와 기존 독립 wire-code oracle을 확인한다. 공개 코드 문자열과 API는 유지한다. |
| F-09 | digest 동작 결함은 반증. rustdoc의 범위 모호성 | execution projection과 원문 bytes가 digest 입력이며 projection 밖의 compiled disposition/rule program은 별도 입력이 아님을 명시한다. encoder/schema/vector는 변경하지 않는다. |

## F-04: 분류와 admission의 소유권

`docs/contracts/frontend-execution-v3.md`의 Public boundary는 consumer가
지원하지 않는 family를 거부하는 데 classifier를 **사용할 수 있다**고 한다.
frontend가 Rule/Declaration/Catalog 자체를 거부해야 한다는 계약은 없다.
지원 여부와 실행 전 typed rejection은 downstream admission owner 책임이다.

호출 경로는 facade re-export → pipeline `classify_single_source_root_v3` →
`parse_source` → surface `classify_parsed_root_v3`다. 반환값은 root enum이고
closed authority 입력이나 execution document 발급이 없다. surface의
`parsed_root_classification_matches_every_family`는 해당 세 family도 이미
검증한다. 따라서 거부 구현 부재를 frontend 결함으로 볼 수 없다.

다만 standalone external consumer는 Graph/Composition 및 zero/multiple
입력만 분류해 해당 세 family의 공개 경로를 직접 pin하지 않았다.
`tools/frontend_external_consumer/src/main.rs`에 아래 원문과 기대값을 추가했다.

- `let x = 1` → Declaration.
- `rule r : x if a` → Rule.
- `view v | query-ref=r.x` → Catalog.

후자의 이름들은 resolve하지 않아도 분류되어야 한다. 세 검사는 closed builder
생성 전에 실행한다. 기존 Graph/Composition positive control과 zero/multiple
rejection 검사는 유지한다. 이 검사는 family 오분류와 authority에 잘못 의존한
분류를 검출하지만 실제 downstream 지원 정책이나 실행을 증명하지 않는다.

## F-05: inline 문자열과 경쟁 authority는 다르다

`pipeline/src/compile.rs::incomplete_open_analysis`의 missing-module/relation/
graph code는 각각 production 발행처가 하나다. 공개 incomplete/diagnostic-limit
코드도 해당 모듈에서 소유한다. 최초 보고서의 여섯 코드 중
`pipeline.parse_budget_projection_failed`는 `pipeline.analysis.*`가 아니다.

기존 독립 oracle:

- `pipeline/tests/v3_analysis_document_assembly.rs::all_open_authority_masks_have_exact_status_and_missing_facts`:
  8개 authority mask별 정확한 status와 missing code 목록·순서를 리터럴로 검증.
- `diagnostics-stdio/tests/ndjson_contract.rs`: transport의 정확한 missing code 검증.
- `conformance/tests/self_validation.rs`와 standalone external consumer:
  공개 분석 결과와 literal wire vocabulary 검증.

테스트의 리터럴은 독립 expected value이며 production SSOT 복제가 아니다.
`pipeline/src/diagnostic.rs`도 open-analysis 코드 소유자가 sibling compile
모듈임을 이미 명시한다. enum 추가는 문자열 값의 정합을 자동 증명하지 않으며,
현재 잘못된 값이나 경쟁 producer는 발견되지 않았다. 필수 수정에서 제외한다.

## F-09: qualification digest의 범위

`pipeline/src/execution.rs::ExecutionDocumentV3::qualification_digest`는
source digest와 projected contract digest를 결합한다. 문서 전체 rule program과
compiled disposition은 V3 execution projection 밖에 있다. 다만 이들을 바꾸는
원문 편집은 source digest를 통해 계속 반영된다.

기존 rustdoc의 “Any DTO”는 해당 execution DTO라는 문맥으로 읽을 수 있으므로
현행 구현 모순으로 단정할 수 없다. 호출자가 compiled DTO 전체에 대한 digest로
오해하지 않도록 입력 범위를 명시했다. 기존 독립 digest-stream oracle,
source-only change sensitivity, facade vector replay를 그대로 사용한다.
주석을 위한 구현 복제 테스트는 추가하지 않는다.

## 검증 경계

production 동작, 공개 API, 진단 code, schema, digest 값은 변경하지 않았다.
변경은 공개 consumer assertion 3개, digest rustdoc, coverage ledger와 감사 기록이다.
검증 command는 `just ci`; 실행 로그는
`target/sep27-ir-remaining-reaudit/ci.log`에 보관한다. 실제 실행 결과는 해당 로그와
`target/ci/nextest-evidence.json`으로 확인한다. 이 dirty 변경에 기존 clean-HEAD
qualification receipt를 재사용하지 않는다.
