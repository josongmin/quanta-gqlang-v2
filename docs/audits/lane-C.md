# Lane C — 의미 결정 owner 감사 (Binder / Types / Modules / Sema) 통합 보고서

병렬 2-pass 독립 감사 + 조정자 교차 검증 결과. 원본 패스 보고서 전문:
`/tmp/lane-c-snapshot-20260927010022/lane-C-pass1.md`, `.../lane-C-pass2.md` (snapshot 소멸 시 이 파일이 요약 원본).

## 1. 소스 경계

- Snapshot: `/tmp/lane-c-snapshot-20260927010022` (rsync, .git/target 제외, symlink 0건, manifest 756파일 SHA-256)
- 원 HEAD: `eaeb68ea2711d411616511713fb016d828cd80be` (main)
- dirty 경계: Lane C 4 crate(binder/types/modules/sema)는 **clean**. dirty는 cli/pipeline/surface/docs로 타 lane(B/E) 영역 — 본 lane 결론은 snapshot 기준.
- Coverage: binder(1,920줄), types(8,354), modules(6,903), sema(19,129 incl. test_support) 전체. 미조사: modules lockfile/manifest/signature/extension/version 세부 대칭, sema/document.rs 후반 일부 극히 일부 (pass2가 보완, residual은 §5).
- 실행 checks: pass1 병렬 감사자 `cargo test`(binder+types, 격리 target `/tmp/lane-c-target-p1`) all-pass; pass2 정적 감사(cargo 미실행). 조정자는 핵심 finding 4건 원문 대조 반증 검증.

## 2. Owner / 경계 요약 (상세 표는 pass1 §2)

| fact | owner | 소비 | 비고 |
|---|---|---|---|
| 이름 해결/scope/capture | binder `LexicalScopeV1` | sema | 단일 owner 확인 |
| 타입 합법성/overload/리터럴 | types kernel(`check.rs`) | sema, solver | solver는 hint 복제(구조 리스크 C-08) |
| module graph/catalog/visibility | modules `ResolvedGraphV2` | sema(`ModuleImportAdapterV2`) | binder `ModuleGraphV1`은 placeholder(구조 리스크 C-06) |
| 문서 트랜잭션/sealing | sema `SemanticDocumentV2` | algebra/logic/planning | 필드 private, 생성자 검증형, BTreeMap 순서 |
| 위조 seam (`forged_membership` 등) | sema | 테스트 only | `#[cfg(any(test, feature="test"))]`, `test` feature는 4개 downcrate **dev-dependencies**에서만 — production 미컴파일. 반증 완료. |

핵심 반증 (양 패스 일치, 조정자 재확인): Sema의 의미 결정 중복 **CONFIRMED 0건** — resolution/type legality/catalog validation은 각 owner 위임 + sema 교차검증 구조. 4 crate에서 HashMap/HashSet production 사용 0건, 정렬 총순서(유일 예외 C-02). 리터럴 극값(NaN/inf/underflow/i128::MIN/-0.0/escape) 처리 정상 경로 확인. 트랜잭션 원자성(값 소비, 프레임 로컬, per-transaction validator) 준수.

## 3. Findings (통합·중복 제거 후)

### CONFIRMED_DEFECT

**C-01 · P2** — i128::MIN 접기가 단항-리터럴 직접 인접만 인정.
`types/check.rs:1779-1785` — `min_case`는 `unary.operand.kind == Literal(Int(...))` 직접 매치 + `text.trim()` 비교. `ValueExprKindV1::Paren`(:1775-1778)은 `deferred=false`로 재귀하여 `-(<2^127>)` 괄호 형태에서 min_case 미성립 → 일반 parse 경로(:2081, `text == ...` 무trim)에서 i128::MAX 초과 `IntegerOutOfRange` **거짓 거절**. 무괄호는 수용(비대칭). Caller: surface parser → `CheckedSyntaxDocumentV2::try_from_hir` → types kernel. Expected: 두 철자 동일 수용. Fix: Paren 통과 deferred 전파 또는 operand에서 Paren strip 후 판정. Oracle: 괄호/무괄호 양 철자 동일 TypedLiteralFactV2를 갖는 독립 회귀. (pass1+pass2 독립 발견, 조정자 원문 확정. regression은 미실행 — 정적 입증.)

**C-02 · P2** — catalog edge 비교의 정렬 키 불완전.
`sema/document.rs:698-699` — `sort_by` 키는 `(source,target)` 2-튜플이지만 equality는 5-튜플(kind/family/location 포함) `Vec` 비교. 동일 (source,target)에 서로 다른 5-튜플 edge 2건이 병존하면 정렬 후 순서가 입력 순서에 의존 → 오탐 `document_validation_failed`("semantic catalog references differ"). Trigger: modules catalog와 sema 재수집이 같은 key 쌍에 상이한 순서로 edge를 생산하는 문서. Fix: 정렬 키를 전체 5-튜플로(총순서 확보). Oracle: 동일 key 2-edge + 서로 반전된 입력 순서 fixture 양방 수용. (pass2 발견, 조정자 원문 확정. 정적 입증.)

### STRUCTURAL_RISK

- **C-03 · P3** — 어휘 스코프 이중 구현: binder `scope.rs`(shadow→거절) vs types `solver.rs:248-270`(shadow 허용). 규칙 의도 상이하나 sema 교차검증이 guard, 관측 drift 없음.
- **C-04 · P3** — binder 상호재귀(`resolve_relation/bool/value/shape`)의 crate 내부 depth 상한 부재 — 상류 hir/sema 예산에만 의존. 공개 API가 불변식을 스스로 인코딩하지 않음.
- **C-05 · P3** — `ModuleGraphV1::with_import` 공개 위조 표면 + binder placeholder graph가 modules `ResolvedGraphV2`와 병존(production은 sema adapter만 사용). 같은 root: 미사용/테스트 전용 표면 정리 또는 gate.
- **C-06 · P3** — solver의 리터럴 분류 무검증 복제(`solver.rs:1133-1139,1443-1451` vs `check.rs:2072-2161`) — kernel 후행 검증으로 누출 없음.
- **C-07 · P3** — kernel/solver overload 판정 규칙 상이(assignability vs exact domain) — 닫힌 registry에서 무해.
- **C-08 · P3** — 평행 예산 회계(sema `FrontendBudgetV2` vs modules budget) — 수치 동기 수동.
- **C-09 · P3** — 미검증 pub carrier/생성자: `ResolvedDocumentV2::from_declared`, `IncompleteSemanticDocumentV2` 인접 등 — sema 내부 신뢰 경계용이나 문서화 부족.
- **C-10 · P3** — kind_tag 문자열 결합, family→kind 표 이중 인코딩 — schema 확장 시 수동 동기 지점.

### PROOF_GAP

- **C-11 · P3** — `- -2^127`(=+2^127, i128 초과) 타입 단계 수용, 런타임 checked-overflow 거절은 downstream/Lane B·E 미증명.
- **C-12 · P3** — relation-predicate capture owner 재검증 관대(`check.rs:1074-1107`, EXISTS 외 노드도 owner 승인) — sema 유일 생산자라 현재 결함 아님.
- **C-13 · P3** — Presentation/SourceField sealing, 선언체인 depth의 명시적 불변식 문서 부재.
- **C-14 · P3** — `size` builtin 도메인 `list<int>` 고정(`builtin.rs:371`) — 의도 확인 필요.

### DOCUMENTATION_DRIFT

- **C-15 · P2** — binder `resolve.rs:1131-1136`: partition candidate를 성공 resolution 후보로 push하나 필드 계약(:82-85 "never a successful semantic resolution")과 모순. 문서 또는 코드 주석 정리.
- **C-16 · P3** — `FiniteF64V2` -0.0→+0.0 정규화(`ty.rs:726`) vs sign-aware `Ord`(`ty.rs:757`) 잠재 모순 및 -0.0 canonical화 미문서 — 현재 손실 경로 없음.
- **C-17 · P3** — usize::MAX fail-open, Host provenance 형식 미검증 등 경계 문서 부재.

### OPTIONAL_REFACTOR

- **C-18 · P3** — `document.rs:5782/5796` HIR 아이템 전체 clone(비용만, 의미 영향 없음).
- **C-19 · P3** — canonical-name 검증기 3중 복제(`ty.rs:711`/`check.rs:2589`/`env.rs:327`) — 현재 일치, 공유 함수 도입 여지.
- **C-20 · P3** — `[null,1]`→HeterogeneousCollection(optional 미채택)은 결정적 합법성 선택, 결함 아님(기록용).
- 기타 P3: binding-kind 어휘 2중 match, type fact 무음 덮어쓰기(`check.rs:2018`), 순서의존 초기화/O(n²) 소규모 4건 — pass2 보고서 참조.

### 반증되어 제외된 후보 (근거)

- **Join/Cross의 `checked_relation_frame`(shadow 검사 없음)** — pass2 C-P2-05. 반증: 해당 검사는 "**newly introduced**" binding 대상이며 Join/Cross는 새 이름을 만들지 않고 각 Source/Project/GroupBy mint 시점에 이미 outer-shadow 검사를 통과한 프레임의 chain이므로 재검사는 이중 보고가 된다. mint 이후에야 가시성이 변하는 경로 없음. 의도된 설계로 판정, 결함 제외.
- 위조 seam/`test` feature, 트랜잭션 원자성, HashMap 결정성, DeferredI128MinMagnitude 이중부정 봉인(`document.rs:2838`), -0.0 정규화(계약 명시), sema 의미결정 중복 — 모두 반증 완료(§2).

## 4. Cross-lane handoff

- **Lane B**: `-(...)` 괄호 철자 수용 범위(parser가 Paren 산출 조건), escape 디코딩 경계, surface budget 상한이 binder 재귀 깊이 상한(C-04)을 실제로 커버하는지.
- **Lane D**: operator id 1의 `-(i128::MIN)` checked 산술, resolved_call None 정책, rule polarity 소비.
- **Lane E**: `- -2^127` 런타임 overflow 거절(C-11), projection에서 type fact 무음 덮어쓰기(C-20 계열) 미발생 확인.
- **Lane A**: C-05 placeholder graph, C-08 평행 예산, C-10 표 이중 인코딩을 authority map에 반영.

## 5. 평가

- 증거 범위 내 일관됨: **이름/타입/catalog 의미 결정의 SSOT는 단일 owner 위임 + 교차검증 구조로 확인**, 의미 결정 중복 CONFIRMED 0건(조사 경계: 4 crate 전체 + 호출 지점; "전역 중복 0" 주장 아님).
- confirmed defect 있음: **C-01(유허용 거짓 거절, fail-closed 방향), C-02(결정성 위반 오탐)** — 둘 다 유효 프로그램에 대한 거짓 거부형이며 fabricated success/silent drop은 없음.
- 미검증 한계: modules lockfile/manifest 세부, 실행 기반 재현(양 defect 정적 입증 + 회귀 oracle 제안만), downstream 런타임 동작(C-11).
