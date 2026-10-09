# 기존 작업 마감: source 상태와 남은 acceptance

Updated: 2026-10-10. QGLang D01~06 source 수정 및 focused proof 완료.
전체 저장소 검증 상태는 QFC-04의 **현재 HEAD receipt**로 판정한다.
[감사](audit.md)는 초기 snapshot, [후속 감사](dedup-audit.md)는 실제 source 변경과 owner 검증을 기록한다.

## QFC-01 — 입력 고정

- [x] QGLang main baseline `5e5164dbcb1493a09dceecab4e1f3a23b1c09b3c`, tree
  `44cc7206a581cfeba750a17452919ea6601e294a`, 자신의 dirty paths와 file hashes를 기록했다.
  `.audit/oct10-frontend-closeout/scc-source-state.json`은 커밋 전 입력이며 이후 문서 변경을 포함한
  최종 committed 입력은 qualification driver가 SHA/tree와 clean pre/post custody로 다시 고정한다.
- [x] downstream selected source만 readback: Semantica HEAD `a126366c202367b95bc1bb52ec249739fcc4802c`,
  bootstrap contract/driver/closeout gate hash를 기록했다. sibling 전체 WIP/product 증명으로 해석하지 않는다.

Owner: QGLang; downstream 전체 입력은 기존 Semantica E13 owner.

## QFC-02 — bootstrap/producer paired 입력 조정: OPEN_DOWNSTREAM

- [ ] 전체 revision `9060e469…`와 현재 producer subtree/vector 경로의 입력 정책을 조정한다.
  Semantica E13-T04/T08/T11의 기존 source/verification 작업에 연결한다.
- [ ] pin 변경이 필요하면 compatible bundle과 actual V3 consumer compile/contract-vector owner proof를
  함께 확보한다. 과거 pin 유지 시 최신 producer acceptance가 별도 OPEN임을 명시한다.

Owner: Semantica E13. QGLang standalone qualification의 선행조건은 아니다.
새 runner/pin 정책/중복 ticket은 만들지 않는다. QGLang CI만으로 완료 처리하지 않는다.

## QFC-06 — SCC 공통 partition owner: SOURCE_DONE / FOCUSED_PASS

- [x] actual caller, canonical ordering/witness, discovery/emission located charge와 typed first refusal를 대조했다.
- [x] Modules graph child의 iterative borrowed Tarjan kernel 하나로 Modules/Logic 양쪽을 전환했다.
  old Kosaraju와 domain-private Tarjan traversal을 제거했다. Logic→Modules edge는 Cargo/두 lockfile/
  exact architecture policy에 함께 반영했다. 새 외부 library 및 backward edge는 없다.
- [x] Modules legacy charge/witness 및 Logic negation-before-aggregate/strata/error ordering을 caller에 유지했다.
- [x] kernel live buffers와 witness phase를 구분해 checked scratch bound를 갱신했다.
  manifest/request topology는 동일 helper를 소비한다. global budget/allowance는 올리지 않았다.
- [x] actual Modules cycle/Logic strata independent graph oracles, canonical witness/permutation,
  exact/one-short work, located first-cause, 50k512-KiB stack, request topology 정상/cycle reservation
  exact/one-byte-short 및 drop custody owner 검증을 실행했다. 결과와 한계는 dedup-audit.md 참조.
- [x] old traversal caller census와 production AST 재대조 완료. 새 source의 full gate는 QFC-04로 판정한다.

- [x] D04 후속 감사의 DAG preflight 중복을 공통 owner로 통합하고 duplicate adjacency를 제거했다.
  cursor FIFO와 canonical View min-heap으로 ready frontier 반복 이동을 제거했다.
- [x] compatibility helper만 검사하던5정점 DAG oracle에 production exact/one-short 호출을 추가했다.
  DAG16384 oracle cases, production wide/isolated50000 및 View canonical/resource owner 검증 PASS.

- [x] D05의 canonical component index를 scan/BFS/strata가 재사용하도록 통합했다.
  duplicate edge-order projection과 모든 offending payload를 보관하던 Vec를 제거했다.
- [x] Logic84 owner 검증 PASS: 대량 후보의 exact/one-short/first-tie, source-sealed semantics,
  malformed internal index의 scan/self-cycle/strata typed refusal와 기존 independent/deep-chain proof 포함.

Owner: QGLang Modules graph + Logic recursion. Semantica Core SCC와는 별개다.

## QFC-03 — 필요한 수정과 로컬 main

- [x] catalog namespace mapping과 canonical lock text writer를 기존 owner에서 공통화했다.
  public const/signature, canonical bytes와 validation/budget/error ordering을 유지했다.
- [x] D01~06 동반 변경과 문서/원장의 중복 배정을 대조했다. 신규 동작 결함은 재현하지 않았다.
  Record/Union validator 유사 코드는 서로 다른 payload/diagnostic owner이므로 추가 generic engine을 발급하지 않았다.
- [x] D06 lock parser의 가변 field Vec, 추가 pack-name 복제와 canonical text 재생성을 제거했다.
  기존 field validator/serializer owner를 유지하며 late-field/duplicate-before-order와 exact span을 보존한다.
  Modules authority122 tests PASS:24 permutations,131072-space malformed line 및 기존 resource rails 포함.
- [x] test ledger와 guarded-test-module inventory를 새 SCC oracle에 맞춰 갱신했다.
- source 반영 결과는 `git log -1`과 caller clean status로 확인한다. 이 packet 자체를 포함한
  하나의 변경 단위를 로컬 main에 커밋하고, 해당 HEAD에서 QFC-04를 실행한다.

Owner: QGLang. Semantica coupled bundle 또는 타 작업 dirty hunk를 stage하지 않는다.

## QFC-04 — 현재 committed HEAD 로컬 qualification: RECEIPT_DERIVED

실행 명령은 **`just qualify-local`** 하나다. 이 명령이 `just ci`를 실행하므로 full gate와
그 leaves를 선행 중복 실행하지 않는다. 이전 D01/D02 dirty-source CI1514는 historical evidence다.

| 현재 HEAD의 결과 | 판정 |
| --- | --- |
| `target/qualification/<git rev-parse HEAD>/receipt.json` 없음 또는 exit≠0 | NOT_QUALIFIED; first failing command/source/log를 보관하고 원인 수정 |
| schema3, matching SHA/tree, exit0, clean pre/post, same-run raw Nextest inventory/events/count/hash 및 required gate evidence 일치 | LOCAL_QUALIFIED |

receipt와 `ci.log`가 source 밖에서 실행 상태를 소유한다. 그 결과를 tracked 문서에 다시 커밋해
SHA를 바꾼 뒤 이전 receipt를 최신 qualification으로 제시하지 않는다.
실패 시 threshold 확대/테스트 제외/source drift 무시로 통과시키지 않는다.

Owner: QGLang. 로컬 gate만 사용하며 새 hosted/remote CI는 필수조건이 아니다.

## QFC-05 — 범위별 종료

- source/ownership: D01~03 공통화와 양쪽 caller 전환 완료, focused owner 검증 PASS.
- QGLang standalone: QFC-04의 현재 HEAD receipt로 완료 여부를 판정한다.
- paired contract/compile: QFC-02 **OPEN_DOWNSTREAM**.
- Semantica runtime/product: 기존 Oct-10 compiled required exits의 owner 결과가 필요하며 이 작업에서 **NOT_RUN**.
- push/remote CI: 별도 상태이며 이 packet의 필수조건이 아니다.

완료율 분모에 downstream product나 V4를 섞지 않는다. 현재 HEAD receipt와 source 완료,
paired integration 및 실제 product acceptance를 각각 보고한다.
