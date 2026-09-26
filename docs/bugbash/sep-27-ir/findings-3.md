# Lane C 감사 — 실제 문제 확정 목록 (findings-3)

기준: snapshot `lane-c-snapshot-20260927010022`, HEAD `eaeb68ea2711d411616511713fb016d828cd80be`, binder/types/modules/sema clean.
2-pass 독립 병렬 감사 + 조정자 원문 반증 검증 완료. 전체 분석·반증 후보는 `docs/audits/lane-C.md` 참조.

## CONFIRMED_DEFECT

### 1. `-(i128::MIN)` 괄호 형태 거짓 거절 — P2

- **위치**: `crates/quanta-gqlang-v2-types/src/check.rs:1775-1785` (발견: C-P1-01 / C-P2 i128 항 병합, 조정자 확정)
- **내용**: `min_case` 특수 접기는 단항 `-`가 `Literal(Int("170141183460469231731687303715884105728"))`와 **직접 인접**할 때만 성립. `ValueExprKindV1::Paren` 분기(check.rs:1775-1778)는 `deferred` 플래그를 전파하지 않아 `-(<2^127 리터럴>)` 괄호 철자에서 min_case 미성립 → 일반 parse 경로(check.rs:2081)에서 i128::MAX 초과로 `IntegerOutOfRange` **거짓 거절**. 무괄호 `-(1701...)`는 수용(비대칭).
- **경로**: surface parser → `CheckedSyntaxDocumentV2::try_from_hir` → types kernel `type_expr`.
- **Expected**: 괄호/무괄호 동일하게 수용, `TypedLiteralValueV2::Int(i128::MIN)`.
- **Actual**: 괄호 형태만 `IntegerOutOfRange` (fail-closed 방향의 유효 프로그램 거부).
- **Root fix (owner: quanta-gqlang-v2-types)**: `Paren` 통과 시 deferred 전파, 또는 min_case 판정에서 operand의 Paren을 strip 후 매치. 부수: 매직스트링 비교의 trim 불일치(1782 `text.trim()` vs 2081 무trim) 통일.
- **Regression oracle**: 괄호/무괄호 양 철자가 동일 `TypedLiteralFactV2`를 산출하는 독립 회귀 (기존 테스트는 무괄호만 커버).
- **입증**: 정적 (재현 미실행). pass1 병렬 감사자 cargo test all-pass 상태에서 신규 회귀 부재 확인.

### 2. catalog edge 비교의 정렬 키 불완전 → 결정성 위반 오탐 — P2

- **위치**: `crates/quanta-gqlang-v2-sema/src/document.rs:697-701` (발견: C-P2-01, 조정자 확정)
- **내용**: `semantic_edges`/`validated_edges`를 `sort_by`로 (source, target) **2-튜플 키**로 정렬한 뒤 5-튜플(kind, family, location 포함) `Vec` equality로 비교. 동일 (source,target)에 서로 다른 5-튜플 edge가 2건 이상이면 정렬이 원 입력 순서를 보존한 채 끝나 비교 결과가 입력 순서에 의존 → 모듈 검증 그래프와 실제로 일치하는 문서도 `document_validation_failed`("semantic catalog references differ") 오탐 거부 가능.
- **Trigger 조건**: modules catalog와 sema 재수집이 같은 key 쌍에 상이한 5-튜플 edge를 상이한 순서로 생산하는 문서(동일 자산 쌍에 참조 family가 2개 이상).
- **Expected**: 다른 edge들이면 순서 무관하게 동일 판정.
- **Actual**: 정렬 키에 포함되지 않은 필드 값의 순서가 결과를 좌우.
- **Root fix (owner: quanta-gqlang-v2-sema)**: 정렬 키를 비교에 사용하는 전체 5-튜플로 확장해 총순서 확보.
- **Regression oracle**: 동일 (source,target) 2-edge를 서로 반전된 순서로 제공하는 fixture 양방 수용 회귀.
- **입증**: 정적.

## DOCUMENTATION_DRIFT (실제 계약-코드 불일치, 수정 대상)

### 3. partition candidate push vs 계약 문서 모순 — P2

- **위치**: `crates/quanta-gqlang-v2-binder/src/resolve.rs:1131-1136` vs 필드 계약 `resolve.rs:82-85`
- **내용**: `partition_candidates` 계약은 "syntactic call position alone is never a successful semantic resolution"이나, 해당 라인은 candidate를 성공 resolution 벡터에 push. 동작 자체는 types owner의 exact aggregate-argument proof가 최종 gate라 위험 없음 — 계약 문구/주석과 코드가 모순되어 다음 독자가 잘못된 불변식을 믿게 됨.
- **Fix (owner: binder)**: 문구를 "candidate 등록"으로 정정하거나 push 대상을 분리. 회귀: partition candidate가 resolution으로 승격되지 않는 부정 테스트 존재 확인만.

## 제외된 후보 (참고)

- Join/Cross가 `checked_relation_frame`(outer-shadow 무검사)을 쓰는 것 — Join은 새 이름을 mint하지 않고 각 입력 프레임 mint 시점에 이미 shadow 검사를 통과했으므로 의도된 설계 (반증 상세 `docs/audits/lane-C.md` §3).
- 나머지 전 항목은 STRUCTURAL_RISK / PROOF_GAP / OPTIONAL_REFACTOR로, 현재 오동작 아님 — `docs/audits/lane-C.md` 참조.
