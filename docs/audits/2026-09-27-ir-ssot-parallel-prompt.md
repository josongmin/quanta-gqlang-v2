# IR / SSOT / frontend architecture 병렬 감사 프롬프트

아래 실행 프롬프트 전체를 조정 에이전트에 전달한다. 독립 세션에 분배할 때는 공통 규칙·출력 계약과 해당 lane을 함께 전달한다. 이 문서는 감사 실행 지시이며 감사 결과가 아니다.

## 실행 프롬프트

`/Users/songmin/Documents/code-new/quanta-gqlang-v2`의 현재 소스를 대상으로 IR, 의미 결정의 SSOT, 중복 구현, 단계별 책임, 공개 계약, 검증 구조를 적대적·객관적으로 감사하라. 실제 수정이 필요한 결함과 증거 부족을 분리하라. 구현 변경은 하지 말고 병렬 조사와 최종 통합 보고서까지 완료하라.

### 1. 먼저 소스 경계를 고정하라

- 현재 `AGENTS.md`, `tools/prompt-manager/sources/rust-frontend.md`를 읽는다. named workflow skill은 호출하지 않는다.
- `git rev-parse HEAD`, branch, `git status --short`, 관련 diff를 기록한다. HEAD만으로 dirty 소스가 고정됐다고 주장하지 않는다.
- 조정자는 감사에 필요한 추적 파일과 미추적 파일을 포함한 불변 snapshot을 하나 만든다. 원 checkout을 수정하거나 stash/reset/clean하지 않는다. snapshot의 경로, 원 HEAD, 포함된 dirty 경로, 파일 내용 hash manifest를 기록하고 모든 lane이 같은 snapshot을 읽게 한다. snapshot 밖으로 연결되는 symlink는 기록하고 소스 고정 증거로 사용하지 않는다.
- 파일 snapshot 식별자와 공개 `ExecutionDigestV3`는 다른 개념이다. 소스 고정 hash를 frontend authority나 runtime provenance로 사용하지 않는다.
- snapshot 생성이 불가능하면 원본에서 조사하되 읽은 파일의 hash를 조사 전후 비교한다. 변경된 파일에 의존한 결론은 재검증 전까지 `SOURCE_UNSTABLE`로 남긴다.
- 2026-09-27 프롬프트 작성 시 확인한 HEAD는 `eaeb68ea2711d411616511713fb016d828cd80be`였다. 이는 실행 시점의 HEAD를 대신하지 않는다. 작성 중에도 외부 dirty 변경이 관찰됐다.

### 2. 감사의 의미와 범위

이 레포는 frontend-only다. runtime/provider/SDK/backend 구현 부재를 결함으로 신고하지 않는다. downstream 실행 보장은 별도의 소스와 실행 증거가 있어야 한다.

현재 조사 시작점:

- `surface`: parse/format → 공유 HIR.
- `hir`: 문서·노드·좌표·무결성 검사.
- `binder`, `types`, `modules`: 이름, 타입, 외부 authority의 각 owner.
- `sema`: 해당 owner를 조합하여 문서 단위의 checked semantic transaction과 typed carriers를 만든다.
- `algebra`, `logic`: sealed semantic document에서 관계 의미와 rule/graph 계약을 만든다.
- `planning`: capability, truth/evidence/effect, result-kind, cost를 소유한다.
- `pipeline`: 공유 컴파일 경로, 문서 조립, V3 projection/encoding.
- `frontend-contract`: downstream 공개 facade.
- CLI, diagnostics stdio, conformance, external consumer는 facade 경계를 사용해야 한다.

이 목록은 조사 가설이다. 실제 함수 호출과 Cargo dependency DAG로 확인하라. 문서의 선형 단계 목록을 실제 의존성 그래프로 간주하지 않는다.

대표 경로는 `compile_source_document_v3` → source parse → `CheckedSyntaxDocumentV2::try_from_hir` → `resolve_checked_document_v2` → compiled assembly/owner stages → `ExecutionDocumentV3`다. 별도로 open analysis, formatting, classifier, owner-level 직접 호출 경로를 추적하라.

SSOT는 모든 단계를 하나의 enum/struct로 합치는 요구가 아니다. 다음을 구분하라.

1. **정당한 단계별 IR**: 서로 다른 불변식·consumer·추상화 수준을 가진 표현.
2. **정당한 projection/adapter**: 이미 확정된 사실을 별도 계약으로 옮기는 변환.
3. **정당한 경계 재검증**: 다른 입력 신뢰 경계나 새 불변식을 검사하는 경우.
4. **의미 결정의 중복**: 같은 사실을 두 owner가 독립적으로 판단하고 별도 규칙을 유지하는 경우.
5. **경쟁 authority**: 서로 다를 수 있는 표현·registry·캐시 중 무엇이 진실인지 결정되지 않은 경우.
6. **불필요한 복사/순회**: 의미 중복과 별개인 비용 문제. reachable 경로와 반복 횟수를 제시한다.

유사한 이름, enum variant, match, struct 필드, validation 존재만으로 중복 결함을 만들지 않는다. 중복 후보마다 두 구현, 입력, 판단 규칙, 호출 경로, 동기화 방식, 실제 drift 가능성 또는 관측된 차이를 제시하라. drift 가능성만 확인되면 구조적 위험으로 분류하고 현재 오동작과 분리한다.

### 3. 병렬 배치

6개 lane을 분리된 subagent에 맡긴다. 가용 슬롯보다 많이 동시에 실행하지 않는다. 조정자는 소스 경계, handoff, 통합, 최종 검증을 소유한다. lane별 출력 파일만 작성하고 공통 소스·테스트·registry를 수정하지 않는다.

- A: IR 목록 / dependency architecture / SSOT authority map.
- B: Surface / HIR / format의 구조와 무결성.
- C: Binder / Types / Modules / Sema의 의미 결정.
- D: Algebra / Logic / Planning의 의미 보존과 owner 경계.
- E: Pipeline assembly / V3 DTO / facade / front doors.
- F: 자원 상한 / 결정성 / diagnostics / proof rail의 실효성.

다른 lane의 owner가 루트 원인이면 경계 증거를 handoff하고 그 lane이 본 finding을 소유하게 한다. consumer 증상은 별도 root finding으로 중복 등록하지 않는다. lane별 1차 완료 후 조정자가 교차 경계를 재감사한다.

### 4. 공통 감사 기준

모든 lane은 필요한 범위에서 다음 기준을 적용한다.

- consumer → caller → 의미 결정 owner → checked representation → projection을 추적한다.
- 각 IR의 생성자, 검증 시점, 읽기/쓰기 경로, 식별자 범위, 유지해야 하는 불변식, 실패 동작을 기록한다.
- 외부 입력과 caller-built HIR에서 검증 우회, forged/cross-document 참조, 잘못된 span, 중복/누락 ID를 조사한다.
- raw/public constructor, mutable access, trait default, test feature/helper가 production 경로의 불변식을 우회하는지 확인한다. 테스트 전용 위조 가능성을 공개 API 결함으로 혼동하지 않는다.
- malformed/unsupported 입력이 success-shaped default, partial success, fabricated plan, silent drop으로 바뀌는지 확인한다.
- open analysis의 `Incomplete`, `Error`, `Complete`와 closed compilation 및 execution document 발급 조건을 구분한다. open analysis는 완전한 authority가 있으면 Complete일 수 있으나 execution document를 발급하지 않는다.
- source order, stable diagnostic order, tie-break, identity, digest encoding, null/float/string semantics를 확인한다.
- 단계 간 정보 소실이 의도된 abstraction인지, consumer에 필요한 의미의 누락인지 구분한다.
- parsing/formatting의 구조적 round trip과 semantic equivalence를 구분한다. round trip만으로 타입·관계·rule 의미 보존을 증명하지 않는다.
- lowering, mandatory normalization, optional canonicalization, optimization을 구분한다. 이름이 normalize라고 해서 고정점 rewrite라고 가정하지 않는다.
- 불필요한 generic framework, wrapper, duplicated traversal/registry, 과도한 결합은 실제 변경 파급·오동작·비용과 함께 제시한다. 파일 길이와 취향만으로 결함을 만들지 않는다.
- 전면 리팩터링·SSA·arena·interning·incremental query system·MLIR 도입을 기본 답으로 내놓지 않는다. 필요성, 최소 해결책, compatibility impact를 먼저 제시한다.

### 5. 일반적인 관행을 적용하는 방법

단일한 보편적 compiler/query frontend 표준이 있다고 가정하지 않는다. 아래는 비교 기준이며 이 레포에 그대로 적용되는 규격이 아니다.

- Rust compiler의 여러 IR은 단계별 표현이 정상적인 설계임을 보여준다: https://rustc-dev-guide.rust-lang.org/overview.html
- MLIR의 verifier/IR/표현 경계는 불변식·의미 보존 감사의 비교 기준이다: https://mlir.llvm.org/docs/LangRef/
- MLIR의 canonicalization은 best-effort이며 반복 수렴과 선택적 최적화의 correctness 분리를 설명한다: https://mlir.llvm.org/docs/Canonicalization/

특히 MLIR의 선택적 canonicalization 규칙을 이 레포의 필수 lowering에 그대로 강요하지 않는다. 관계/논리 변환은 이 레포가 명시한 null, bag/set, 순서, 상관, effect, authority 의미를 먼저 찾아서 판단한다. 정의가 없으면 proof/spec gap으로 기록한다.

각 기준은 `repo invariant / external contract / common practice / optional tradeoff` 중 하나로 분류한다. 위반 보고에는 적용 근거와 실제 failure mode가 필요하다. 외부 관행을 추가 인용하면 공식 문서나 원 논문을 확인하고 링크·해당 설명·적용 한계를 기록한다.

### 6. Lane A — authority와 구조

시작 파일: `Cargo.toml`, 각 crate manifest, `tools/architecture/policy.toml`, `tools/architecture/check_architecture.py`, owner `src/lib.rs`.

- production/dev/build/feature/target 의존성을 분리하여 실제 DAG와 policy를 비교한다.
- HIR, typed semantic facts, algebra, logic contracts, plans, execution DTO의 목록과 생성·소비 경계를 만든다.
- operator/builtin/family/schema/catalog/source/diagnostic/command registry 및 budget/digest schema authority를 검색한다. 실질적으로 소비되는 복제 registry와 retired compatibility 경로를 찾는다.
- crate 경계 통과 자체와 내부 의미 구현 복제를 구분한다. DAG gate가 통과해도 의미 중복이 있을 수 있는지 조사한다.
- schema/variant 추가 시 수정해야 하는 실제 위치를 추적하여 누락되기 쉬운 수동 동기화 경계를 기록한다.

산출물: `lane-A.md`, `authority-map.md`.

### 7. Lane B — Surface와 HIR

시작 파일: `surface/src/{parser,expr,query,family,shape,format,equivalence}.rs`, `hir/src/{build,expr,relation,item,shape,id,span,integrity,depth,visit}.rs`.

- expr/bool/relation/result-shape의 단일 carrier와 builder/visitor owner를 확인한다.
- precedence, associativity, literal boundary, keyword dispatch, escaping, UTF-8 byte spans가 parser/formatter에서 일치하는지 조사한다.
- 전체 문서 node-ID uniqueness, auxiliary spans, caller-built HIR, representability rejection을 확인한다.
- formatter가 의미를 추측하거나 지원하지 않는 구조를 출력에서 제거하는지 확인한다.
- parse/format/parse의 비대칭 payload fixture, structurally different children, 정확한 순서 oracle을 살펴본다.

산출물: `lane-B.md`.

### 8. Lane C — 의미 결정 owner

시작 파일: `binder/src/{resolve,scope,module_graph}.rs`, `types/src/{check,solver,op,builtin,env,ty}.rs`, `modules/src/{authority,catalog,graph,visibility,signature,extension,manifest,lockfile}.rs`, `sema/src/{document,authority,expr,relation,names,item,rule}.rs`.

- Sema가 Binder/Types/Modules를 조합하는지, 동일한 resolution/type legality/catalog validation을 독립 재구현하는지 확인한다.
- scope/capture/shadowing, import/re-export/visibility/ambiguity, overload/arity/type solving, rule symbol/type closure를 추적한다.
- relation row schema, provenance, nullable/type coercion, aggregation context와 output shape의 authority를 찾는다.
- `TypedLiteralFactV2`부터 semantic literal까지 i128 최솟값, finite float, -0, escape decoding의 사실이 보존되는지 조사한다.
- `SemanticDocumentV2` sealing, atomic document failure, item-key/source registry 연결과 open authority의 unresolved fact 처리 경계를 확인한다.

산출물: `lane-C.md`.

### 9. Lane D — Algebra / Logic / Planning

시작 파일: `algebra/src/{normalize,node}.rs`, `logic/src/{source,recursion,aggregate,graph,witness}.rs`, `planning/src/{plan,item,policy,effect,cost,result_kind,composition,check,presentation}.rs`.

- `AlgebraSessionV2`의 ingress가 sealed semantic fact를 소비하는지 확인한다. raw HIR/text를 재해석하는 shadow path가 있는지 찾는다.
- Filter/Project/Join/Group/Aggregate/Sort/SemiJoin/AntiJoin 변환에서 입력·predicate·join mode·ON·correlation·output/provenance·ordering을 추적한다.
- relation exists, nested query, quantifier, composition binding/body를 비상관·상관·다중 binding·동명 collision로 구분하여 조사한다.
- null semantics, duplicate/bag/set, ordering, non-pure operation이 rewrite legality에 미치는 영향은 명시된 계약 기준으로 판단한다.
- rule program closure/SCC/negation/aggregate legality, graph witness/dedup/tie-break가 해당 owner에 있는지 확인한다.
- plan-required, NoPlanReason, rule membership의 분류와 capability/effect/evidence/cost 결정이 다른 단계에서 중복되는지 조사한다.
- bounded lowering와 optional rewrite가 있다면 종료·수렴·repeated work를 구분하여 확인한다.

산출물: `lane-D.md`.

### 10. Lane E — assembly / projection / facade

시작 파일: `pipeline/src/{compile,execution,diagnostic}.rs`, `frontend-contract/src/lib.rs`, `cli/src/{main,v3_plan,line_value}.rs`, `cli/src/commands/`, `diagnostics-stdio/src/handlers/v3_diagnostics.rs`, `conformance/src/lib.rs`, `tools/frontend_external_consumer/`, `tools/test_frontend_external_consumer.py`, `docs/contracts/frontend-execution-v3.md`.

- source compilation, open analysis, format, classifier, CLI, diagnostics, conformance가 선언된 ingress를 사용하는지 확인한다. parse-only 경로에 semantic validation을 요구하지 않는다.
- compiled assembly의 order와 item map이 exact-key로 대응하는지 확인한다. missing/extra/duplicate/foreign/wrong-kind 및 partial-document 처리를 조사한다.
- Sema/Algebra/Plan → `ExecutionDocumentV3` field mapping을 작성한다. projection에서 binding/type/literal/source/schema/operator/policy를 새로 판단하거나 재파싱하지 않는지 확인한다.
- public constructor/accessor/mutability와 compile-fail privacy probe가 unforgeability를 실제로 지키는지 확인한다.
- plan-required items, NoPlanReason, document-wide rule program, 독립 executable payload의 지원 범위를 구분한다. 일부 family의 standalone payload 부재가 문서화된 제한이면 곧바로 결함으로 신고하지 않는다.
- V3 지원 여부와 downstream 실행 가능성을 혼동하지 않는다. 소비할 수 없는 payload의 producer/consumer 책임을 명시한다.
- deterministic encoding의 version/domain separation/tag/length framing/field coverage/순서와 golden vector provenance를 확인한다.
- source/contract/qualification digest, raw ID와 transaction-local scope, item lookup과 runtime provenance를 구분한다.

산출물: `lane-E.md`, `projection-map.md`.

### 11. Lane F — boundedness / determinism / proof

시작 파일: `surface/src/{budget,depth}.rs`, `hir/src/depth.rs`, `sema/src/authority.rs`, `modules/src/budget.rs`, `diagnostics-stdio/src/{server,handlers/v3_diagnostics}.rs`, `docs/language/test_hellgate_ledger.md`, `docs/adr/0002-test-authority-and-adversarial-oracles.md`, `tools/qualification/README.md`, `tools/ci/run_local.py`, `Justfile`, architecture/lint/consumer/qualification gates.

- source bytes, expression/relation/shape depth, node/item/rule counts, graph size, catalog/import expansion, diagnostics/response bytes가 첫 위험 순회·할당보다 먼저 제한되는지 확인한다.
- recursion, long left-associated chains, clone/collect, repeated subtree walks, SCC 및 resolution 반복의 입력 크기와 상한을 분석한다.
- 측정 없는 성능 결론은 만들지 않는다. 구조적으로 반복됨을 입증한 결과와 실제 runtime/peak-memory 측정을 분리한다.
- unordered collections, traversal order, sorting tie-break, source IDs, float bits가 public 출력/identity에 영향을 주는지 조사한다.
- owner diagnostics가 transport에서 code/span/status를 잃거나 typed error를 문자열로 평탄화하는지 조사한다.
- 테스트가 independent oracle인지, 구현과 같은 규칙을 복사하는지, variant/Ok만 보고 payload를 놓치는지 확인한다.
- conformance/registry/privacy/semantic-honesty/architecture gates의 검출 범위와 비어 있는 scan·누락 대상·same-count fixture drift 가능성을 확인한다.
- local owner test, binary process, external consumer, local CI, clean committed-HEAD qualification, downstream product proof를 구분한다.

산출물: `lane-F.md`, `coverage-matrix.md`.

### 12. 검증과 실행 비용

- 정적 source 증거가 충분한 조사에 무조건 전체 테스트를 실행하지 않는다. 의심되는 behavior는 최소 owner rail 또는 재현으로 확인한다.
- 신규 재현 파일이나 mutation은 lane별 scratch/isolated checkout에만 만들고 원본을 수정하지 않는다. 필요할 때만 작성한다.
- 공유 Cargo target directory에서 여러 빌드/테스트를 경쟁시키지 않는다. coordinator가 실행 순서를 조정하거나 lane별 target을 분리하며 build 자원은 별도로 제한한다.
- 실패 재현에는 valid positive neighbor와 independent expected result를 포함한다. guard/default/mapping 변경 시 해당 regression이 실패할 근거를 제시한다.
- command, toolchain, snapshot ID, exit status, selected/executed/pass/fail/skip counts와 로그 위치를 기록한다. 부분 실행·timeout·중단·compile success를 PASS로 쓰지 않는다.
- 조정자는 repo-wide local completion이 필요할 때 `just ci`를 한 번 실행한다. 그 leaf 전체를 사전 중복 실행하지 않는다. audit-only 정적 결론과 CI 실행 여부를 별도로 표기한다.
- `just qualify-local`은 clean committed-HEAD receipt 요청에만 사용한다. dirty snapshot을 committed-HEAD qualification이라고 부르지 않는다.

### 13. 모든 lane의 출력 계약

`lane-X.md`에는 다음을 작성한다.

1. snapshot ID/HEAD/dirty 경계, 조사 파일·API·variant·caller coverage, 미조사 범위.
2. owner와 consumer, 입력/출력 IR, 불변식, 생성/검증/실패 경계 표.
3. findings. 각 항목은 다음 필드를 가진다.
   - ID, severity(P0–P3), classification.
   - classification은 `CONFIRMED_DEFECT`, `STRUCTURAL_RISK`, `PROOF_GAP`, `DOCUMENTATION_DRIFT`, `OPTIONAL_REFACTOR` 중 하나.
   - 위반한 repo invariant/공개 계약 또는 common practice와 적용 이유.
   - 정확한 파일:줄·symbol, reachable caller → owner 경로.
   - trigger/input, expected, actual, 영향 및 현재 발생에 필요한 조건.
   - 재현 command/result 또는 정적 입증. 미실행은 명시한다.
   - 중복 후보는 두 구현의 비교와 divergence/synchronization 근거.
   - 최소 root fix, owning lane/crate, compatibility impact, regression oracle.
   - 반증 시도: 관련 guard/기존 테스트/의도된 제한을 확인했고도 finding이 남는 이유.
4. 의심했지만 반증된 후보: 정당한 단계별 IR, adapter, boundary validation 등의 근거.
5. 실행한 checks와 결과, 미실행 checks 및 한계, 다른 lane handoff.

`authority-map.md`는 `fact/invariant | owner | representation | producer | consumers | duplicated decision? | evidence`를 사용한다. `projection-map.md`는 `source fact/field | transform owner | destination | intentionally omitted? | semantic/digest coverage | evidence`를 사용한다. `coverage-matrix.md`는 `invariant | source owner | owner test | rejection/edge oracle | facade/process/gate proof | gap`을 사용한다.

### 14. 최종 통합과 종료 조건

조정자는 모든 lane 결과를 읽고 다음을 수행하라.

- snapshot이 다른 결과는 합성하지 않는다. 해당 finding을 같은 snapshot에서 다시 확인하거나 pending으로 둔다.
- `owning invariant + root decision + reachable failure`로 중복 findings를 합친다. 같은 root fix로 해결되는 consumer 증상을 분리 티켓으로 늘리지 않는다.
- A의 전체 IR/authority 목록과 E의 projection mapping, F의 coverage를 대조하여 빠진 stage/family/variant/entry point를 재조사한다.
- B↔C, C↔D, D↔E, E↔F의 불변식 handoff를 확인한다. 각 표현의 책임·생성·검증·소비가 연결되지 않는 부분을 남긴다.
- surviving findings를 다시 반증한다. unreachable code, test-only forgery, downstream owner 책임, 문서화된 지원 제한, 취향만 남은 항목은 confirmed defect에서 제거한다.
- 결과는 무근거 점수 대신 `증거 범위 내 일관됨 / confirmed defect 있음 / evidence insufficient`로 평가한다. 전역적 '중복 0개'는 exhaustive inventory와 조사 경계 없이 주장하지 않는다.

최종 산출물:

- `final-audit.md`: 현재 결론, 소스 경계, 우선순위별 confirmed defects, 구조적 위험, proof gaps, 문서 drift, optional refactor를 분리한 보고서.
- `remediation-union.md`: 실제 필요한 root fix의 합집합. 각 항목에 owner, prerequisites, disjoint 수정 영역, acceptance criteria, 검증 rail을 적는다. Optional refactor는 별도 목록이다.
- `authority-map.md`, `projection-map.md`, `coverage-matrix.md`, `lane-A.md` … `lane-F.md`.

종료 조건은 6개 lane 보고와 통합·중복 제거·누락 감사를 완료하고, 각 결론에 source/test 증거 또는 명시적인 미검증 한계를 붙이는 것이다. 실제 결함 수정·release qualification은 이번 감사의 완료 조건에 포함하지 않는다.
