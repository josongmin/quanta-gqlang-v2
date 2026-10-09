# QGLang 최근 1주 변경 인벤토리와 전체 회고 입력

상태: **회고 입력 자료**. 결함 판정이나 제품 완료 판정이 아니다.

## 범위와 재현 방법

- 기간: **2026-10-03 00:00 KST ~ 2026-10-10 `25fad76cf5b4510244157c2e009b4dc37aa958ff`**. 기간 내 첫 커밋은 10월 4일의 `cc27ff2`이다.
- 비교 기준: 첫 커밋의 부모 `9060e46997dd9c62080a18bdfae06f8c162442e2` → `25fad76`. `main`의 first-parent 기록과 두 커밋 사이의 파일 상태를 사용했다. 커밋 시간은 committer date다.
- 이 범위에 **216커밋**: 코드·설정 파일을 건드린 커밋 14개, 문서 전용 202개. 커밋 기록에서 한 번 이상 수정된 경로는 **340개**. 기준과 HEAD 사이의 최종 상태가 다른 경로는 rename을 분리해 **299개**(crates 173, docs 82, tools 31, CI/policy 3, 나머지 10). 이동·삭제·되돌림으로 최종 diff에는 없는 작업 경로가 41개다. 아래 인벤토리는 340개 모두 포함한다.
- 조사 시 작업 트리에 다른 작업의 미커밋 문서 변경과 `docs/adr/0009-frontend-owner-dedup-and-execution-contract.md`가 있었다. 이 문서의 **커밋 통계와 의미 변화는 고정된 HEAD만** 대상으로 한다. 그 미커밋 변경은 수정하거나 HEAD 결과로 계산하지 않았다.
- 재현: `git log --first-parent 9060e46..25fad76`; `git diff --no-renames --name-status 9060e46 25fad76`; 각 커밋의 `git diff-tree --no-commit-id --name-only -r <sha>` 합집합. 본문에 적힌 실행 결과는 별도 명시한 영수증에서 읽었으며 이 문서 작업 중 Rust 테스트를 실행한 결과가 아니다.

## 실제 소스·계약 변화

| 주제 / 커밋 | 파일 소유자와 변경된 의미 | 회고에서 확인할 실패·호환 경계 |
| --- | --- | --- |
| Frontend owner 분리와 로컬 정책 (`cc27ff2`, `293bbba`) | Surface, Sema, Pipeline, Types, Modules의 큰 파일을 stage별 모듈과 테스트로 분리했다. [`tools/lint/size_gate.py`](../../tools/lint/size_gate.py), [`tools/lint/rust_module_guards.py`](../../tools/lint/rust_module_guards.py), Semgrep·Clippy·CI 정책에 지역 소유권/크기 검사를 더했다. 다수의 삽입·삭제는 코드 이동이다. | 기존 공개 API, 진단 순서, source span, 파서/타입 거부 결과의 byte-level 동등성; 테스트 분리 후 실제 선택·실행 수; gate가 새 모듈과 삭제된 디렉터리에 fail-closed인지 확인. |
| 요청 단위 보관과 bounded scalar 경로 (`e598a1d`; 일부 `cc27ff2`) | HIR request root/permit, Binder·Modules·Types·Sema·Algebra·Planning·Pipeline의 부분 소유자를 연결했다. 공개 facade에 parse/check request-owner 및 finite Source→Project scalar compile 경로가 생겼다. 기존 소스·권한과 동일한 source-bound owner를 유지하며 실행 DTO를 seal한다. 상세 허용 집합은 [V3 실행 계약](../contracts/frontend-execution-v3.md#bounded-scalar-request-entry)에 있다. | root/source/budget/ledger 불일치가 예약 또는 효과 전에 typed refusal인지; exact/one-short 비용·저장소 예약, 중도 실패·마지막 clone drop에서 회수되는지; 일반 컴파일로의 우회가 없는지; ordinary compile과 결과·세 digest가 같은지. 전체 프런트엔드 heap bound라는 주장은 범위 밖이다. |
| CI·저장소 보조와 문서 정리 (`614d9de`, `cb43052`, `fa4c015`) | CircleCI 설정, 공용 BTree 저장소 보조, 삭제된 CI source 디렉터리를 다루는 검사, ADR/감사 문서 정리와 일부 CI 정책이 들어왔다. `fa4c015`는 제목이 docs여도 `.typos.toml`을 건드렸다. | 로컬 `just ci`와 hosted CI의 실제 job/branch 적용 차이; 파일 삭제 시 policy scan이 조용히 빠지지 않는지; 설정 변경이 어떤 검사 범위를 바꾸는지. 문서 이동은 기능 구현으로 세지 않는다. |
| SCC 공유 (`43d9943`, `c5b27aa`) | Modules의 borrowed iterative [`indexed_scc`](../../crates/quanta-gqlang-v2-modules/src/graph/indexed_scc.rs)를 Logic 재귀 검사와 Modules graph가 사용한다. 순회 커널은 공유하지만 작업량 과금, 위치, witness/first refusal은 각 caller가 보유한다. | 깊고 넓은 그래프의 유한 stack/scratch, malformed index, 그래프 순서·cycle witness, Logic의 polarity 진단, exact/one-short owner 예산. Semantica Core SCC는 별도 소유자다. |
| DAG·Logic·lock 중복 제거 (`77550ba`, `cd9aec0`, `36e3444`, `a6a85d1`) | DAG 공통 preflight/cursor FIFO와 View canonical min-heap으로 인접 목록 복사와 ready-Vec shift를 줄였다. Logic은 SCC membership과 offending minima 선택을 재사용한다. lock 파서는 bounded field slice를 쓰고 이미 검증된 입력으로 순서를 판정한다. 조사 당시 미추적이던 ADR-0009는 이 행의 증거로 사용하지 않았다. | DAG root multiplicity·dangling edge·overflow·minimum-ready order; Logic 모든 edge charge와 negation 우선; lock의 canonical 직렬화 byte, late malformed/duplicate vs ordering 진단 우선순위, 큰 공백 입력의 할당 상한. 성능 개선 수치는 측정 전까지 주장하지 않는다. |
| Execution DTO의 물리적 소유자 이동 (`f7b79e9`, `9abbdc3`, `25fad76`) | [`quanta-gqlang-v2-execution-contract`](../../crates/quanta-gqlang-v2-execution-contract/Cargo.toml)가 DTO·projection·encoding/digest와 Rename Command DTO를 소유한다. Pipeline은 컴파일 orchestration, 진단, request permit을 보유한다. facade는 두 소유자를 직접 reexport한다. [`frontend-execution-v3.md`](../contracts/frontend-execution-v3.md)는 공개 V3 필드/순서, source schema 3 및 contract/qualification schema 8이 그대로임을 규정한다. | Rust 정의 경로 이동으로 downstream source closure 재빌드가 필요하다. facade 밖 owner 직접 의존 금지, DTO 사설 생성, digest golden byte·source order, scalar move-only 예약/해제, Rename의 snapshot/target/token은 의도 정보일 뿐 runtime mutation grant가 아니라는 경계를 확인. |

### 변경을 과대 해석하지 말아야 할 경계

- 위의 대규모 `src`·`tests` 파일 증감은 여러 번의 물리적 분할·이동을 포함한다. `80,189 insertions / 45,961 deletions`라는 최종 diff 크기만으로 기능량이나 품질을 판단할 수 없다.
- Frontend가 표현하는 Graph/CHECK/Composition/Rename과 bounded scalar DTO는 downstream 실행·mutation 허가가 아니다. 실행 가능 여부와 provider/generation/lease/host budget 결합은 downstream 책임이다. [ADR-0003](../adr/0003-frontend-downstream-integration-boundary.md)와 [V3 계약](../contracts/frontend-execution-v3.md)을 기준으로 pair를 리뷰한다.
- `docs/`의 이번 주 변경에는 Semantica substrate/Native/Source/fixture의 계획, handoff, 중간 증거가 많다. **QGLang Rust 구현 변경으로 환산하지 않는다.** 삭제되거나 ADR로 흡수된 문서의 과거 status도 현재 결함 목록이 아니다.
- 문서 수정 횟수 상위 경로는 `docs/plans/oct-6-substrate-dedup/readme.md` 91회, `test-plan.md` 78회, `docs/work/substrate-repair-checkpoints-2026-10-07.md` 71회, `docs/work/remaining.md` 70회다. 앞의 일부 경로는 기간 중 삭제되었다. 회고에서는 커밋 수를 진행률로 읽지 말고 최종 owner·현재 증거만 남긴다.
- private diagnostics stdio는 [ADR-0008](../adr/0008-private-diagnostics-primary-location-contract.md)에 따라 primary location 중심 V2 wire를 유지했다. 이 범위에서 secondary location/structured authority code가 추가된 것으로 보지 않는다.

## 검증 상태와 전체 회고의 기준

| 범위 | 확인된 증거 | 판정 한계 |
| --- | --- | --- |
| 고정 QGLang HEAD `25fad76`, tree `99501ba1` | `target/qualification/25fad76cf5b4510244157c2e009b4dc37aa958ff/receipt.json`: schema 3, SHA/tree 일치, `dirty=false`, `status=complete`, `exit=0`, 명령 `just ci`; Nextest selected/executed/passed **1,529/1,529/1,529**, skipped/leaked 0. | 그 커밋의 repository-local frontend 결과. 현재 미커밋 문서, downstream 빌드, installed process, hosted CI의 증거가 아니다. |
| 개별 owner 결과 | 이번 주 문서와 테스트에 SCC, DAG, Logic, lock, execution-contract/facade의 focused regression이 기록되어 있다. | 각각의 원본 SHA/선택 수/완료 로그를 별도로 확인해야 조합할 수 있다. 현재 clean-HEAD 영수증을 대체하지 않는다. |
| Semantica pair / 제품 | 이번 주 QGLang 커밋만으로는 확인되지 않는다. | execution-contract crate 이동 후 Semantica의 정확한 producer closure·vector·consumer compile, runtime admission, 설치 프로세스와 제품 rail을 한 입력 tuple로 확인해야 한다. |

이 회고 문서 작성으로는 새 테스트·CI·제품 실행을 하지 않았다.

## 후속 전체 회고 리뷰 체크포인트

1. **동일 동작 여부:** 이동 전후 공개 facade API, 정상/거부 진단·span·source order, contract byte/vector를 고정 입력으로 대조한다. 단순 컴파일이나 파일 이동 통계는 근거가 아니다.
2. **자원·결정성:** scalar owner/permit의 모든 실패 경로, SCC/DAG/lock의 exact/short·큰 입력·malformed input을 해당 owner에서 재검토한다. 반복 scan/clone을 제거한 코드에는 실제 대표 workload와 peak allocation/RSS를 따로 측정한다.
3. **게이트 신뢰성:** size/module/architecture/Semgrep/CircleCI가 새 crate, 삭제된 경로, 외부 test module, 수동 예외를 모두 검사하는지 확인한다. 선택 0건·누락된 owner를 green으로 취급하지 않는다.
4. **소비자 pair:** Semantica가 `25fad76`의 facade 및 새 execution-contract source closure를 정확히 가져와 컴파일·계약 vector를 통과하는지 확인한다. 이어서 runtime admission/installed-process 검증을 별도 기록한다.
5. **문서 수렴:** 202개 문서 전용 커밋의 중복 상태표를 현재 source, ADR, [remaining work](../work/remaining.md)로 귀속한다. 특히 조사 중 동시에 수정 중인 Oct-6/Oct-10 plan 파일은 작성자 변경이 완료된 뒤 재고정해서 검토한다.

각 항목은 재현 가능한 현재 caller/owner 반례가 나오면 **결함**, 실행 입력 또는 영수증이 없으면 **검증 공백**, 구현 범위가 다른 경우 **downstream 작업**으로 분리해 기록한다. 이 문서는 그 판정을 미리 내리지 않는다.

## 기간 내 수정 파일 전체 목록

다음 목록은 커밋별 변경 경로 합집합이다. `A/M/D`는 `9060e46`과 `25fad76`의 최종 상태 차이(이동은 삭제+추가로 표기), `T`는 기간에 수정되었으나 두 끝점의 파일 상태가 동일한 경로다. `n`은 그 경로를 건드린 기간 내 커밋 수다. 기준 SHA 이후의 미커밋 파일은 제외했다.

### `crates/quanta-gqlang-v2-algebra` (6)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-algebra/src/lib.rs` |
| M | 1 | `crates/quanta-gqlang-v2-algebra/src/node.rs` |
| M | 2 | `crates/quanta-gqlang-v2-algebra/src/normalize.rs` |
| A | 1 | `crates/quanta-gqlang-v2-algebra/src/normalize/tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-algebra/src/scalar_request.rs` |
| D | 1 | `crates/quanta-gqlang-v2-algebra/tests/RETIRED_LAWS.md` |

### `crates/quanta-gqlang-v2-binder` (12)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-binder/src/diagnostic.rs` |
| M | 1 | `crates/quanta-gqlang-v2-binder/src/lib.rs` |
| M | 1 | `crates/quanta-gqlang-v2-binder/src/module_graph.rs` |
| M | 1 | `crates/quanta-gqlang-v2-binder/src/resolve.rs` |
| A | 1 | `crates/quanta-gqlang-v2-binder/src/resolve/evidence.rs` |
| A | 1 | `crates/quanta-gqlang-v2-binder/src/resolve/request_owner.rs` |
| A | 1 | `crates/quanta-gqlang-v2-binder/src/resolve/request_owner/composition.rs` |
| A | 1 | `crates/quanta-gqlang-v2-binder/src/resolve/request_tests.rs` |
| M | 1 | `crates/quanta-gqlang-v2-binder/src/scope.rs` |
| M | 1 | `crates/quanta-gqlang-v2-binder/tests/v3_binder_hellgate.rs` |
| A | 1 | `crates/quanta-gqlang-v2-binder/tests/v3_composition_request_storage.rs` |
| M | 1 | `crates/quanta-gqlang-v2-binder/tests/v3_relation_source_use.rs` |

### `crates/quanta-gqlang-v2-conformance` (3)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-conformance/Cargo.toml` |
| M | 1 | `crates/quanta-gqlang-v2-conformance/src/lib.rs` |
| M | 1 | `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` |

### `crates/quanta-gqlang-v2-execution-contract` (11)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/Cargo.toml` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/command.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/digest_oracle.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/encoding.rs` |
| A | 2 | `crates/quanta-gqlang-v2-execution-contract/src/lib.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/producer.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/projection.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/scalar_request.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/scalar_request/bounded.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-execution-contract/src/vocabulary.rs` |

### `crates/quanta-gqlang-v2-frontend-contract` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-frontend-contract/Cargo.toml` |
| M | 3 | `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs` |

### `crates/quanta-gqlang-v2-hir` (8)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| A | 1 | `crates/quanta-gqlang-v2-hir/src/btree_storage.rs` |
| M | 1 | `crates/quanta-gqlang-v2-hir/src/build.rs` |
| M | 2 | `crates/quanta-gqlang-v2-hir/src/depth.rs` |
| M | 2 | `crates/quanta-gqlang-v2-hir/src/integrity.rs` |
| M | 2 | `crates/quanta-gqlang-v2-hir/src/lib.rs` |
| A | 1 | `crates/quanta-gqlang-v2-hir/src/request_storage.rs` |
| M | 1 | `crates/quanta-gqlang-v2-hir/src/span.rs` |
| A | 1 | `crates/quanta-gqlang-v2-hir/tests/v3_request_storage.rs` |

### `crates/quanta-gqlang-v2-logic` (5)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-logic/Cargo.toml` |
| M | 2 | `crates/quanta-gqlang-v2-logic/src/recursion.rs` |
| A | 1 | `crates/quanta-gqlang-v2-logic/src/recursion/partition_tests.rs` |
| M | 1 | `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs` |
| M | 1 | `crates/quanta-gqlang-v2-logic/tests/v3_rule_scc_hellgate.rs` |

### `crates/quanta-gqlang-v2-modules` (27)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-modules/src/authority.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/authority/reference_request.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/authority/request_catalog.rs` |
| M | 1 | `crates/quanta-gqlang-v2-modules/src/budget.rs` |
| M | 3 | `crates/quanta-gqlang-v2-modules/src/catalog.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/catalog/predeclaration.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/catalog/request_final_demand.rs` |
| A | 3 | `crates/quanta-gqlang-v2-modules/src/catalog/view_topology.rs` |
| M | 1 | `crates/quanta-gqlang-v2-modules/src/diagnostic.rs` |
| M | 4 | `crates/quanta-gqlang-v2-modules/src/graph.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/graph/catalog_identity.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/graph/cycle_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/graph/indexed_scc.rs` |
| A | 2 | `crates/quanta-gqlang-v2-modules/src/graph/indexed_scc/tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/graph/indexed_walk.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/graph/indexed_walk/tests.rs` |
| M | 2 | `crates/quanta-gqlang-v2-modules/src/lib.rs` |
| M | 3 | `crates/quanta-gqlang-v2-modules/src/lockfile.rs` |
| M | 2 | `crates/quanta-gqlang-v2-modules/src/manifest.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/src/session_identity.rs` |
| M | 1 | `crates/quanta-gqlang-v2-modules/tests/v3_document_predeclaration.rs` |
| M | 1 | `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority/catalog_and_activation_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority/document_and_model_tests.rs` |
| A | 3 | `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority/resolution_and_pack_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-modules/tests/v3_request_reference_storage.rs` |
| A | 2 | `crates/quanta-gqlang-v2-modules/tests/v3_request_topology_final_storage.rs` |

### `crates/quanta-gqlang-v2-pipeline` (26)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-pipeline/Cargo.toml` |
| M | 3 | `crates/quanta-gqlang-v2-pipeline/src/compile.rs` |
| A | 2 | `crates/quanta-gqlang-v2-pipeline/src/compile/assembly.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/scalar_compile.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/sealed.rs` |
| A | 2 | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/sealed/execution.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/session_tests.rs` |
| A | 2 | `crates/quanta-gqlang-v2-pipeline/src/compile/tests.rs` |
| M | 1 | `crates/quanta-gqlang-v2-pipeline/src/diagnostic.rs` |
| M | 3 | `crates/quanta-gqlang-v2-pipeline/src/execution.rs` |
| T | 2 | `crates/quanta-gqlang-v2-pipeline/src/execution/encoding.rs` |
| T | 3 | `crates/quanta-gqlang-v2-pipeline/src/execution/projection.rs` |
| T | 2 | `crates/quanta-gqlang-v2-pipeline/src/execution/scalar_request.rs` |
| T | 2 | `crates/quanta-gqlang-v2-pipeline/src/execution/scalar_request/bounded.rs` |
| T | 2 | `crates/quanta-gqlang-v2-pipeline/src/execution/tests.rs` |
| T | 3 | `crates/quanta-gqlang-v2-pipeline/src/execution/vocabulary.rs` |
| D | 1 | `crates/quanta-gqlang-v2-pipeline/src/execution_digest_oracle.rs` |
| M | 2 | `crates/quanta-gqlang-v2-pipeline/src/lib.rs` |
| M | 2 | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs` |
| A | 2 | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly/assembly_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly/execution_contract_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly/request_body_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly/scalar_request_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_body_types_request.rs` |

### `crates/quanta-gqlang-v2-planning` (4)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 2 | `crates/quanta-gqlang-v2-planning/src/lib.rs` |
| M | 2 | `crates/quanta-gqlang-v2-planning/src/plan.rs` |
| A | 1 | `crates/quanta-gqlang-v2-planning/src/plan/scalar_request.rs` |
| A | 1 | `crates/quanta-gqlang-v2-planning/src/plan/tests.rs` |

### `crates/quanta-gqlang-v2-sema` (43)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 2 | `crates/quanta-gqlang-v2-sema/src/authority.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/authority/budget.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/authority/inputs.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/authority/open_analysis.rs` |
| M | 1 | `crates/quanta-gqlang-v2-sema/src/diagnostic.rs` |
| M | 2 | `crates/quanta-gqlang-v2-sema/src/document.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/bodies.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/bodies/catalog.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/bodies/inline_views.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/bodies/prepare.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/bodies/source_order.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/body_materialization.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/body_types.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/body_types/composition.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/catalog_inputs.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/declaration.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/depth.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/expression_materialization.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/inline_view.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/item_resolution.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/query_materialization.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/relation_materialization.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/request_body.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/scalar_semantic.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/semantic_document.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/session.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/syntax.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/document/tests.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/document/type_inputs.rs` |
| M | 1 | `crates/quanta-gqlang-v2-sema/src/item.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/item/tests.rs` |
| M | 2 | `crates/quanta-gqlang-v2-sema/src/lib.rs` |
| M | 2 | `crates/quanta-gqlang-v2-sema/src/relation.rs` |
| A | 2 | `crates/quanta-gqlang-v2-sema/src/relation/tests.rs` |
| M | 1 | `crates/quanta-gqlang-v2-sema/src/test_support.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/test_support/families.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/test_support/tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/src/test_support/w4d_tests.rs` |
| M | 1 | `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction/aggregation_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction/authority_view_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction/document_rule_tests.rs` |
| A | 1 | `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction/relation_query_tests.rs` |

### `crates/quanta-gqlang-v2-surface` (17)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/cursor.rs` |
| M | 1 | `crates/quanta-gqlang-v2-surface/src/depth.rs` |
| M | 1 | `crates/quanta-gqlang-v2-surface/src/diag.rs` |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/equivalence.rs` |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/expr.rs` |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/family.rs` |
| A | 2 | `crates/quanta-gqlang-v2-surface/src/family/catalog.rs` |
| A | 1 | `crates/quanta-gqlang-v2-surface/src/family/presentation.rs` |
| M | 1 | `crates/quanta-gqlang-v2-surface/src/lexer.rs` |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/lib.rs` |
| A | 2 | `crates/quanta-gqlang-v2-surface/src/materialization.rs` |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/parser.rs` |
| A | 1 | `crates/quanta-gqlang-v2-surface/src/parser/request_owner_tests.rs` |
| M | 2 | `crates/quanta-gqlang-v2-surface/src/query.rs` |
| M | 1 | `crates/quanta-gqlang-v2-surface/src/shape.rs` |
| M | 1 | `crates/quanta-gqlang-v2-surface/src/unit.rs` |
| M | 1 | `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs` |

### `crates/quanta-gqlang-v2-types` (15)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `crates/quanta-gqlang-v2-types/src/builtin.rs` |
| M | 2 | `crates/quanta-gqlang-v2-types/src/check.rs` |
| A | 1 | `crates/quanta-gqlang-v2-types/src/check/inputs.rs` |
| A | 1 | `crates/quanta-gqlang-v2-types/src/check/scalar_request.rs` |
| A | 1 | `crates/quanta-gqlang-v2-types/src/check/scalar_request/composition.rs` |
| M | 2 | `crates/quanta-gqlang-v2-types/src/check/value.rs` |
| M | 1 | `crates/quanta-gqlang-v2-types/src/diag.rs` |
| M | 1 | `crates/quanta-gqlang-v2-types/src/lib.rs` |
| M | 2 | `crates/quanta-gqlang-v2-types/src/op.rs` |
| M | 2 | `crates/quanta-gqlang-v2-types/src/solver.rs` |
| A | 2 | `crates/quanta-gqlang-v2-types/src/solver/engine.rs` |
| M | 1 | `crates/quanta-gqlang-v2-types/src/ty.rs` |
| A | 1 | `crates/quanta-gqlang-v2-types/tests/v3_scalar_projection_request.rs` |
| M | 1 | `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs` |
| M | 2 | `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs` |

### `docs/README.md` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 15 | `docs/README.md` |

### `docs/adr` (9)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `docs/adr/0001-frontend-campaign-consolidation.md` |
| M | 1 | `docs/adr/0002-test-authority-and-adversarial-oracles.md` |
| M | 2 | `docs/adr/0003-frontend-downstream-integration-boundary.md` |
| A | 1 | `docs/adr/0004-sealed-owner-facts-and-bounded-projection.md` |
| A | 5 | `docs/adr/0005-substrate-reuse-and-consumer-projection.md` |
| A | 1 | `docs/adr/0006-code-query-evaluation-boundary.md` |
| A | 1 | `docs/adr/0007-typed-code-query-contract-proposal.md` |
| A | 1 | `docs/adr/0008-private-diagnostics-primary-location-contract.md` |
| M | 2 | `docs/adr/README.md` |

### `docs/audits` (45)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| D | 1 | `docs/audits/2026-09-27-algebra-ownership-index.md` |
| D | 1 | `docs/audits/2026-09-27-artifact-relocation.md` |
| D | 1 | `docs/audits/2026-09-27-blocked-failure-authority.md` |
| D | 1 | `docs/audits/2026-09-27-document-anchor.md` |
| D | 1 | `docs/audits/2026-09-27-execution-symbol-authority.md` |
| D | 1 | `docs/audits/2026-09-27-ir-ssot-parallel-prompt.md` |
| D | 1 | `docs/audits/2026-09-27-join-output-contract.md` |
| D | 1 | `docs/audits/2026-09-27-parse-budget-and-stage-authority.md` |
| D | 1 | `docs/audits/2026-09-27-planning-result-authority.md` |
| D | 1 | `docs/audits/2026-09-27-projection-origin.md` |
| D | 1 | `docs/audits/2026-09-27-string-escape-authority.md` |
| T | 7 | `docs/audits/2026-10-04-current-source-audit-plan.md` |
| T | 4 | `docs/audits/2026-10-04-frontend-consumer-contract-plan.md` |
| T | 2 | `docs/audits/2026-10-04-quanta-index-integration-campaign-prompt.md` |
| T | 2 | `docs/audits/2026-10-04-reference-source-review-extension.md` |
| T | 2 | `docs/audits/2026-10-04-reference-source-review.md` |
| T | 30 | `docs/audits/2026-10-05-current-source-audit-action-plan.md` |
| T | 14 | `docs/audits/2026-10-06-process-fixture-continuation.md` |
| T | 2 | `docs/audits/2026-10-06-proof-input-retirement.md` |
| T | 7 | `docs/audits/2026-10-06-substrate-component-observations.json` |
| T | 3 | `docs/audits/2026-10-06-substrate-dedup-implementation-plan.md` |
| T | 2 | `docs/audits/2026-10-06-substrate-dedup-plan-source.json` |
| T | 2 | `docs/audits/2026-10-06-substrate-frontdoor-inventory.json` |
| T | 8 | `docs/audits/2026-10-06-substrate-ownership-and-usecase-audit.md` |
| D | 1 | `docs/audits/lane-A-20260927/authority-map.md` |
| D | 1 | `docs/audits/lane-A-20260927/lane-A.md` |
| D | 1 | `docs/audits/lane-C.md` |
| D | 1 | `docs/audits/lane-D-20260927/lane-D.md` |
| D | 1 | `docs/audits/lane-e/lane-E.md` |
| D | 1 | `docs/audits/lane-e/lane-E1-assembly-projection.md` |
| D | 1 | `docs/audits/lane-e/lane-E2-facade-front-doors.md` |
| D | 1 | `docs/audits/lane-e/lane-E3-deep-dive.md` |
| D | 1 | `docs/audits/lane-e/lane-E4-adversarial.md` |
| D | 1 | `docs/audits/lane-e/projection-input-E2.md` |
| D | 1 | `docs/audits/lane-e/projection-map.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-calcite.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-codeql.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-datafrog.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-datafusion.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-differential-dataflow.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-egg.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-joern.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-kuzu.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-souffle.md` |
| T | 2 | `docs/audits/reference-source/2026-10-04-substrait.md` |

### `docs/bubgash` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| D | 1 | `docs/bubgash/sep-27-ir/findings-15.md` |
| D | 1 | `docs/bubgash/sep-27-ir/findings-6.md` |

### `docs/bugbash` (25)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| D | 1 | `docs/bugbash/sep-26-test-audit/P2-sema.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P3-digest-oracle-2026-09-27.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P3-format-closure-2026-09-27.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P3-surface.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P4-types.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P5-logic-algebra.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P6-planning-pipeline.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P7-hir-binder.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P8-front-doors-deep.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P8-front-doors.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/P9-tools-python.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/README.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/adversarial-review-2026-09-26.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/final-review-2026-09-26.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/p1-modules.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/parallel-review-2026-09-26.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/reaudit-2026-09-26.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/recheck-2026-09-26.md` |
| D | 1 | `docs/bugbash/sep-26-test-audit/transport-boundary-review-2026-09-26.md` |
| D | 1 | `docs/bugbash/sep-27-ir/declaration-dependency-rca.md` |
| D | 1 | `docs/bugbash/sep-27-ir/findings-2.md` |
| D | 1 | `docs/bugbash/sep-27-ir/findings-3.md` |
| D | 1 | `docs/bugbash/sep-27-ir/findings-4.md` |
| D | 1 | `docs/bugbash/sep-27-ir/remaining-reaudit.md` |
| D | 1 | `docs/bugbash/sep-27-ir/structural-remediation-plan.md` |

### `docs/contracts` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 6 | `docs/contracts/frontend-execution-v3.md` |

### `docs/language` (4)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 5 | `docs/language/README.md` |
| D | 1 | `docs/language/ai-profile.md` |
| M | 1 | `docs/language/examples_index.md` |
| M | 10 | `docs/language/test_hellgate_ledger.md` |

### `docs/misc` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| D | 1 | `docs/misc/checklist-scenariolist-09-25.md` |

### `docs/plans` (23)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| A | 5 | `docs/plans/oct-10-frontend-closeout/README.md` |
| A | 5 | `docs/plans/oct-10-frontend-closeout/actions.md` |
| A | 1 | `docs/plans/oct-10-frontend-closeout/audit.md` |
| A | 8 | `docs/plans/oct-10-frontend-closeout/dedup-audit.md` |
| T | 2 | `docs/plans/oct-6-substrate-dedup/checkpoint-d7-evidence.json` |
| T | 35 | `docs/plans/oct-6-substrate-dedup/checkpoint-generated-evidence.json` |
| T | 44 | `docs/plans/oct-6-substrate-dedup/checkpoint-rust-evidence.json` |
| T | 2 | `docs/plans/oct-6-substrate-dedup/checkpoint-source-state.json` |
| T | 41 | `docs/plans/oct-6-substrate-dedup/code-change-map.md` |
| A | 28 | `docs/plans/oct-6-substrate-dedup/code-map.md` |
| A | 4 | `docs/plans/oct-6-substrate-dedup/completion-dispatch-r356.md` |
| T | 2 | `docs/plans/oct-6-substrate-dedup/detail-source-state.json` |
| T | 50 | `docs/plans/oct-6-substrate-dedup/execution-progress.md` |
| A | 19 | `docs/plans/oct-6-substrate-dedup/execution-r79.md` |
| A | 2 | `docs/plans/oct-6-substrate-dedup/implementation-r356.md` |
| A | 17 | `docs/plans/oct-6-substrate-dedup/native-identity-boundary.md` |
| A | 31 | `docs/plans/oct-6-substrate-dedup/parallel-dispatch.md` |
| A | 9 | `docs/plans/oct-6-substrate-dedup/peer-audit-r343.md` |
| A | 18 | `docs/plans/oct-6-substrate-dedup/rca.md` |
| A | 91 | `docs/plans/oct-6-substrate-dedup/readme.md` |
| T | 2 | `docs/plans/oct-6-substrate-dedup/source-state.json` |
| A | 78 | `docs/plans/oct-6-substrate-dedup/test-plan.md` |
| A | 55 | `docs/plans/oct-6-substrate-dedup/tickets.md` |

### `docs/rfcs` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| T | 2 | `docs/rfcs/2026-10-04-code-query-action-list.md` |
| T | 2 | `docs/rfcs/2026-10-04-code-query-dsl-and-substrate.md` |

### `docs/ssot` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| T | 2 | `docs/ssot/README.md` |
| T | 3 | `docs/ssot/engine-status-v1.md` |

### `docs/work` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| A | 70 | `docs/work/remaining.md` |
| A | 71 | `docs/work/substrate-repair-checkpoints-2026-10-07.md` |

### `tools/architecture` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 2 | `tools/architecture/policy.toml` |

### `tools/ci` (7)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| A | 1 | `tools/ci/circleci-install.sh` |
| M | 1 | `tools/ci/doctor.py` |
| A | 1 | `tools/ci/fuzz_smoke.py` |
| M | 2 | `tools/ci/run_local.py` |
| M | 4 | `tools/ci/tests/test_gates.py` |
| A | 1 | `tools/ci/tests/test_size_policy.py` |
| M | 1 | `tools/ci/toolchain.lock.toml` |

### `tools/frontend_external_consumer` (11)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 2 | `tools/frontend_external_consumer/Cargo.lock` |
| A | 1 | `tools/frontend_external_consumer/negatives/d16_import_generic_request_owner.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d17_read_parsed_request_owner.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d18_map_checked_request_owner.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d19_read_scalar_execution_owner.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d20_read_shared_scalar_payload.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d21_clone_scalar_execution_owner.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d22_read_canonical_files_name.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d23_detach_canonical_files_read.rs` |
| A | 1 | `tools/frontend_external_consumer/negatives/d24_import_execution_projection.rs` |
| M | 2 | `tools/frontend_external_consumer/src/main.rs` |

### `tools/lint` (8)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 3 | `tools/lint/README.md` |
| A | 7 | `tools/lint/rust_module_guards.py` |
| M | 1 | `tools/lint/semantic_honesty.py` |
| M | 1 | `tools/lint/semantic_honesty_policy.json` |
| M | 1 | `tools/lint/semgrep_gate.py` |
| A | 3 | `tools/lint/size_gate.py` |
| A | 1 | `tools/lint/size_policy.json` |
| M | 1 | `tools/lint/tests/test_semantic_honesty.py` |

### `tools/prompt-manager` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `tools/prompt-manager/sources/rust-frontend.md` |

### `tools/qualification` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 2 | `tools/qualification/README.md` |
| M | 1 | `tools/qualification/tests/test_external_consumer_cache.py` |

### `tools/test_frontend_external_consumer.py` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 3 | `tools/test_frontend_external_consumer.py` |

### `.circleci` (2)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| A | 1 | `.circleci/config.yml` |
| A | 1 | `.circleci/info.yml` |

### `.claude` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `.claude/rules/rust-frontend.md` |

### `.cursor` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `.cursor/rules/rust-frontend.mdc` |

### `.semgrep` (1)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `.semgrep/qglang-frontend.yml` |

### `root configuration` (8)

| 최종 상태 | n | 경로 |
| --- | ---: | --- |
| M | 1 | `.gitignore` |
| M | 1 | `.typos.toml` |
| M | 2 | `Cargo.lock` |
| M | 2 | `Cargo.toml` |
| M | 1 | `Justfile` |
| M | 2 | `README.md` |
| M | 1 | `clippy.toml` |
| M | 1 | `fuzz/README.md` |
