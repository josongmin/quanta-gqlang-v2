# Lane E 최종 어딧 — 실제 문제만

대상: IR/SSOT 병렬 감사 Lane E (pipeline assembly / V3 DTO / facade / front doors), 2바퀴 병렬 + 적대적 반증 완료.
근거 보고서: `docs/audits/lane-e/lane-E.md` 및 하위 4개 서브레인 보고서.
Snapshot: `docs/audits/lane-b-snapshot-20260927-010019` @ HEAD `eaeb68ea2711d411616511713fb016d828cd80be` (dirty 포함, 정적 감사).

아래는 **반증을 통과한 실제 문제만**. 반증된 항목(E2-03/E2-07/E2-13/E1-04), 의도된 제한으로 강등된 항목(E2-02), 취향/선택 리팩터(E3-7, E1-03)는 제외.

---

## P2 — 구조적 위험 (수정 권장)

### F-01. 독립 digest oracle 부재 + mutation rail 커버리지 구멍
- **분류**: STRUCTURAL_RISK · **Owner**: pipeline tooling
- **문제**: whole-stream golden vector(`pipeline/tests/fixtures/ln.tsv`)이 producer-derived다. `tools/` 전체에 두 번째 독립 encoder나 `qglang.execution`/`contract.v5` 도메인을 참조하는 oracle이 없다(`run_local.py`의 sha256은 tree-cleanliness 지문일 뿐). encoder+TSV가 함께 drift해도 모든 gate를 통과한다.
- **커버리지 구멍**: field-mutation rail이 item_id, item family tag, root stage key, children 순서, top-level item 순서를 커버하지 않는다(E-4 확인).
- **최소 수정**: `tools/`에 out-of-tree whole-row digest oracle 추가 + 누락 5개 필드 mutation rail 확장.
- **회귀 oracle**: oracle이 현재 TSV 값과 독립적으로 일치하는지, 의도된 인코딩 변경 시 양쪽이 함께 움직이는지.

## P3 — 테스트/검증 갭

### F-02. sentinel fallback이 digest 정체성에서 조용한 충돌 가능
- **분류**: STRUCTURAL_RISK · **Owner**: pipeline (`execution.rs:536`, `896-899`)
- **문제**: `"invalid_closed_builtin"` / `"invalid_closed_operator"` sentinel fallback이 존재하고, builtin 이름이 digest에 encoding되는 유일한 정체성(`put_call_target`, execution.rs:2591-2594)이라 invalid id끼리 조용히 충돌한다 — fail-closed 대신. None arm을 커버하는 테스트 없음.
- **최소 수정**: None arm을 typed 실패로 바꾸거나(불가능 조건 주장이 맞다면) 최소한 `debug_assert`/테스트로 sentinel 도달 불가를 pin.
- **회귀 oracle**: sentinel 문자열이 digest에 등장하면 실패하는 테스트.

### F-03. algebra/planning bridge-code parity 미주장
- **분류**: PROOF_GAP · **Owner**: pipeline tests (`diagnostic.rs:578-619`, `751-921`)
- **문제**: logic은 `logic_code(x) == x.code()` parity를 주장하지만 algebra/planning은 리터럴만 확인한다. 모듈 doc(diagnostic.rs:26-27)은 "every reachable variant"를 과대 주장. owner에 variant 추가 시 bridge 매핑 drift가 컴파일이 아닌 런타임 코드로 발견될 수 있는 영역이 있다(match는 exhaustive하지만 코드 문자열 대응은 미검증).
- **최소 수정**: logic과 동일한 parity 테스트를 algebra/planning에 추가, doc 문장 수정.

### F-04. parsable-but-unsupported family의 사전-authority 거부 seam 미검증
- **분류**: PROOF_GAP · **Owner**: tests (facade seam)
- **문제**: contract doc이 문서화한 classify-then-reject 소비 패턴(Rule/Declaration/Catalog root)을 facade 어디에서도 실행하지 않는다.
- **최소 수정**: external consumer 또는 facade 테스트에 unsupported-root → typed reject 케이스 추가.

## P3 — 코드/문서 불일치 (실제 drift, 수정 비용 작음)

### F-05. `pipeline.analysis.*` code가 closed enum 밖에서 발행됨
- **Owner**: pipeline (`compile.rs:82/91/92` + inline literals `940/945/950`)
- **문제**: 6개 code가 const/inline 문자열로 발행되며 `diagnostic.rs:15-16`은 이 모듈이 소유한다고 잘못 서술. `DocumentAssemblyCodeV2`처럼 closed enum으로 묶지 않아 rename/typo drift에 구조적 보호 없음.
- **최소 수정**: `pipeline.analysis.*` 전용 enum(또는 const 모듈)로 이전, doc 수정. E2-03 잔여(exported constant 사용)도 함께 해소.

### F-06. 죽은 공개 어휘 클러스터
- **Owner**: pipeline/docs
- **내용**:
  - `ExecutionFamilyV3::RuleProgram` variant + digest tag 6 + contract doc 어휘 표(L114)가 production에서 한 번도 생성되지 않음(문서 L19는 제외를 명시 — 표와 모순). 예약 어휘라는 문장도 없음.
  - `PipelineStageV2` export가 production 경로에서 미생성, 근거 doc 미구현, code-prefix 유도 주장이 planning→logic relay 2곳에서 깨짐.
- **최소 수정**: 예약 표시 또는 다음 schema bump 때 제거; `PipelineStageV2` 제거하거나 실제 stage reporting을 연결.

### F-07. 진단 정규화의 sema owner와의 비대칭 미문서화
- **Owner**: pipeline (`diagnostic.rs:169-175`)
- **문제**: `canonicalize_diagnostics`가 sema의 sort_key와 필드별 동일한 key를 쓰지만(검증됨) sema에 없는 full-struct dedup를 추가하고, 이를 "canonical Sema order"로 귀속 서술. equal-key tie는 삽입 순(결정적이나 미문서).
- **최소 수정**: 주석에 dedup/tie 규칙의 pipeline 소유권 명시.

### F-08. `empty_blocked_failure`의 무관 조건에 code 재사용 (`diagnostic.rs:478-490`)
- **문제**: 불가능-빈-컬렉션 조각에 `missing_item_result` code를 재사용 — 진단 의미가 어긋남.
- **최소 수정**: 별도 code 또는 해당 경로 제거 검토.

### F-09. 문장 수준 문서 수정 2건
- `execution.rs:136-138` rustdoc: "any DTO change moves it"가 rule-program 제외를 과대 포괄(공개 contract doc :99는 정확함).
- contract doc: TSV digest 도출 provenance 미기재(F-01과 쌍).

---

## 제외한 것 (반증 완료 — 실제 문제 아님)

| 항목 | 반증 근거 |
|---|---|
| E2-03 하드코딩 code | `resource_boundaries.rs:200-226` 행동 pin — rename 시 rail 실패 |
| E2-07 cross-doc `item()` hit | external consumer `main.rs:385-402`가 pin |
| E2-13 family별 payload 부재 | `self_validation.rs:252-307` 전 레지스트리 주장 |
| E1-04 diagnostic cap 교체 | pipeline+stdio rail이 pin (설계된 동작) |
| E2-02 ingress 우회 | `policy.toml` exact-edge 게이트가 내부 강제; 잔여는 외부 repo 영역 |
| E1-03 format 이중 parse / E3-7 이중 encoding | 비용/취향 — 의미 문제 없음 |

## 총평

**증거 범위 내 일관됨**: CONFIRMED_DEFECT 0. 실제 문제는 F-01(P2, oracle 부재) 하나가 실질적이고, 나머지는 fail-closed 격의를 높이는 테스트 갭(F-02~F-04)과 저비용 코드/문서 정합(F-05~F-09)이다. 핵심 무결성(assembly exact-key, projection 무재판단, digest field coverage 완전, unforgeability, 결정성, production panic 0)은 2바퀴 감사에서 반복 검증됨.
