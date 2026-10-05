# 이 세션의 잔여 작업 — 최신 코드 감사

## 2026-10-06 source·receipt 재확인

- Producer main `2987f54a6d17bf4fe9b5e657d0291cb73422882a`: tracked core/CI 입력은 검증 source `8675bcd`와 같다. Foreign CircleCI/RFC/BTree 파일은 이 변경에 포함하지 않는다.
- Consumer main `33d1ba8ce8ad6c7b476bf7bf8b5c982a196351db`: CHECK work 수리4경로 중3개는 원래 proof bytes와 같고 `canonical_cbor.rs`에는 후속 변경이 있다. Foundation37/37은 원래 source 결과다. 최신 consumer 전체 GREEN이 아니다.
- SDK 검증은 아직 미완료다. 최근 진단은 URI normalizer의 String/&str 컴파일 오류2개에서 멈췄고 담당 작업이 수리·재실행 중이다. PTA indexing/hydration 성공 및 Graph/CHECK actual query는 관측하지 못했다.
- 끝난 FCP checkout24/branch25를 보관 후 정리했다. Dirty overlay4와 복구 ref156을 유지했다. [정리·복구 기록](/Users/songmin/Documents/code-new/quanta-gqlang-v2/docs/audits/2026-10-06-proof-input-retirement.md)을 참조한다. 이전 경로 링크는 historical source 위치이며 현재 실행 입력으로 사용하지 않는다.
- 이번에는 문서/정리만 변경했다. 기존 local CI1514와 owner37/23을 현재 source의 새 product qualification으로 합산하지 않는다.

관측: **2026-10-05 23:50 KST**. 문서 수정 전 producer main `7cf36e887397cfae317fb043118b7aa7e84c4091`, consumer main `65836a3adfcbee70b76a6617c0cec9349610d593`. CHECK work 수리 source와 QBC frozen owner 결과를 대조했다. 공유 main, 미커밋 overlay, 검증 후보를 구분한다. 런타임/provider/SDK 변경은 소비자 소유이며 CI는 로컬 실행이다.

## 현재 남은 코드 작업

| 구분 | 작업·근거 | 완료 조건 |
| --- | --- | --- |
| **원인 확인·수리 여부 판정 / P0** | **Product PTA bootstrap.** 실제 Mac/Linux 실패는 source4501 및 앞선 후보에서 query 이전 `PtaOccurrenceAuthorityRefused`다. 최신 main은 pre-key cohort에 `ProductCallGraphSeedConsumptionV1`을 보유하며 caller도 해당 consumption을 전달한다. 동일 최신 source의 bootstrap 성공은 확인하지 못했다. | 최초 거절 분기를 특정하고 기존 lifetime 수리로 해결되는지 owner QBC/실제 bootstrap으로 확인. 여전히 실패하면 그 owner만 수정. 현재 main의 확정 결함이나 수리 완료로 단정하지 않는다. |
| **테스트 코드 통합 / P1** | **Graph/CHECK process fixture 수리와 Mac Graph refusal 회귀.** 격리 candidate에 있으나 실제 query oracle이 통과하지 않아 main 미통합이다. | 같은 clean tuple의 실제 성공/거절 회귀가 GREEN이면 검증된 test delta만 main checkpoint에 반영. 기존 실패 oracle을 완화하지 않는다. |
| **설치·검증 전제** | Linux CHECK4501은 signed installed trust가 없어 서버 시작에서 거절됐다. Generic aarch64 bundle은 canonical x86_64 installed profile에도 맞지 않는다. 현재 근거로 trust 코드 결함을 확인한 것은 아니다. | 기존 release owner에서 공식 profile·root 소유 immutable slot·독립 approval/key·active generation 확보 후 actual process proof. |

**Producer의 원래 구조·계약 정리에는 추가 production 코드 수정이 남아 있지 않다.**
`8675bcd`와 현재 main의 tracked crates/manifests/Justfile/lint/architecture/external-consumer 입력에 committed delta가 없다.
Foreign untracked `hir/src/btree_storage.rs`, CircleCI 및 RFC 변경은 이 증명에 포함하지 않는다.
22:49 재감사는 읽기 전용이었다. 후속 수리에서는 Foundation QBC37/37 PASS와 producer `just ci`1514/1514 PASS를 새로 실행했다. Producer CI는 보존된 dirty checkout의 로컬 관측이며 clean committed-HEAD qualification을 재발급한 것은 아니다. Native Match23/23도 PASS이며 실제 product/process 결과는 아래에서 구분한다.
감사 source bytes와 상태는 [현재 source capture](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/current-session-source-audit-20261005.json)에 보존한다.

후속 AG5의 전체 행/rollback/qualification 연결, IRC의 일반 Graph·Composition·FunctionRef 확장은 해당 소비자 계획의 별도 작업이다. 이번 bounded Graph/CHECK 유지보수의 미완료 항목과 합산하지 않는다.

## CHECK work 수리와 실행 증거

- 기존 Foundation canonical CBOR owner에 별도 work callback을 연결했다. 신규 라이브러리·wire format·DTO 필드 변경은 없다. Legacy memory-only API는 유지한다.
- Count의 방문·UTF-8 payload, encoder의 출력/child/replacement copy, map 비교·row move를 원래 serving control에 선차감한다. Metered map 정렬은 typed refusal이 가능한 allocation-free heapsort `O(n log n)`이며 동일한 length/byte canonical order를 만든다.
- Native Match는 count→4 MiB cap→bounded encoding→length equality→copy/hash budget→digest equality를 단일 admission owner에서 실행한다. Work 거절은 `WorkLimitExceeded`로 보존되고 transport 전 반환한다.
- Foundation QBC **37/37 PASS**, failed/ignored0, source `05de953c…`, immutable run `20261005T143248.807581Z-ae96b315ef5b`. 독립 UTF-8·scalar 비용, traversal 조기 중단, sort 거절 전 mutation 미실행, duplicate-key 거절, 기존 bytes 동일성과 permit 반환을 검사했다.
- Native Match caller/child QBC **23/23 PASS**, failed/ignored0, source `83679ab9…`, immutable run `20261005T143952.237226Z-56146657b5aa`. 이23개에 canonical admission2개와 transport0 회귀가 포함된다. Count-only·exact·one-short work, forged digest, memory peak/permit 반환 및 기존 Match frame identity를 확인했다. 기존 borrowed graph view의 임시 수명 오류도 view binding으로 수리했다.
- 앞선 live-root QBC는 다른 writer의 source drift로 무효였다. Foundation 첫 frozen attempt의 누락 test import를 수리했다. 잘못된 별도 child-module selector는0건 실행으로 exit86 거절됐으며 증명으로 계산하지 않았다. 실제 child2개는 위23/23 receipt에서 성공을 확인했다. Raw logs와 immutable RED를 보존한다.
- Producer **`just ci` exit0**, nextest1514/1514/skipped0, current/MSRV clippy·size283·architecture26/5/0·external consumer1 positive/23 negative·deny·semantic-honesty PASS. Source snapshot `84581100…`; 기존 foreign dirty를 보존했다.
- 공유 snapshot checkpoints가 수리를 main에 먼저 흡수했다. 해당4경로의 실제 commit/hash는 readback JSON에 기록한다. Frozen owner proof가 최신 소비자 전체 product/설치형 qualification을 대신하지 않는다.

[CHECK work 수리 closeout](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/match-canonical-work/closeout.json) — source/hash·명령·immutable receipt와 무효 시도를 별도로 기록한다.

## 완료한 구조·정책 정리

- **Producer main `e598a1d1debdc68a1c2482f96589ff0c707d05d0`: frontend 수리 134개 파일 통합.** 기존 cc27ff2의 대형 파일 분할에 이어 request owner/permit 인증/Modules custody를 정리했다. One-LET Binder/Types, DTO/digest 의미를 보존했고 normal 의존성 4개와 request 전용 BTree 저장량 추정을 제거했다.
- Size **283 source PASS**, architecture **26 normal/5 dev/0 build PASS**. 다른 작업의 CI/RFC/reference/audit dirty 변경은 보존했다.
- Clean producer **`8675bcd2ba5609c0a80a7e24950edc624aa0c52b`**의 `just qualify-local` 내부 로컬 CI **1514/1514 PASS**, skipped/leaked0. 이 후보와 e598a1d의 committed tree는 `f0e97b3303d2b5ffe54f916d73ad4bd13123a745`로 같다. 공유 dirty overlay 전체의 qualification은 아니다.
- 같은 clean source의 60초 `just fuzz-smoke` **exit0/crash artifacts0**, source unchanged/clean. 정기 스케줄은 설정하지 않았다.

| 원래 요청 | 현재 코드·증거 | 남은 작업 |
|---|---|---|
| CI·clippy-msrv | 로컬 `just ci`; static/ci에 MSRV 편입; clean CI PASS | hosted CI 추가 없음 |
| document.rs 분할 | owner 모듈 분할 및 request owner 추출 통합 | 없음 |
| execution.rs·DTO 경계 | owner 분할; Pipeline 정의 + frontend-contract 공개 seam | 소비자 설치형 증명 |
| mega test3개 분해 | size gate test 위반0 | 없음 |
| fuzz smoke | 동일 clean source 60초 PASS | 정기 실행은 운영 선택 |
| types/check.rs·solver.rs | owner 모듈 분할 및 size PASS | 없음 |
| examples closed-authority |18개:10 planned/7 typed non-plan/1 Export typed refusal; CI PASS | 없음 |
| Execution DTO 계약1:1 | schema3/8/8, 독립 digest oracle; consumer pin 통합 | 없음 |
| sema/test_support |1950 code lines 및 자식 owner 모듈, size PASS | 없음 |

공개 DTO를 일괄 이동하거나 wire/schema를 교체하지 않았다.

## 검증 후 main에 합친 소비자 체크포인트

| main commit | 변경과 목적 | 실제 증거 |
|---|---|---|
| `e7ffc579b5c` | fs2 공유 잠금 명시 호출: Rust1.92 inherent 메서드 충돌 수리 | Linux trust owner6/6 PASS@a48 |
| `82cc45e7bd7` | producer subtree/vector pin 및 budgeted admission lint 동기화 | M23-120 static PASS, Python23+6+30; Rust/product 증명 아님 |
| `37e52c612fb` | canonical storage permit/sink/refusal 공개 seam, runtime import 및 관계 결과 회귀 | Linux relation whole37/37 PASS@8be; kernel 증명 |
| `3b714aa81bb` | IDE failure class를 detail move 전에 보존 | 실제 E0382 수리, Mac Graph11 PASS@27c |
| `14a45cf8eb8` | DuplicateExecutionAuthority→InvalidRequest, authority/count 유지 | 독립 exact1/1 및 Graph11 PASS@27c |
| `00290be4535` | bounded top-level Python Graph census의 parser→adapter→host 연결 | Mac Graph owner11/11 PASS@27c; Linux parser10+adapter9 PASS@8be; Linux Graph owner11/11 PASS@806 |
| `76922f834eb` | 기존 `ServingSourceIssuanceTemporaryPermitV1` canonical trait 재수출 | 실제 SDK 호출부 포함 Mac4-binary product PASS@806 |
| `4305ac65652` | 공용 Match wire 상수4개 `pub(crate)`: library sibling·standalone binary 두 경로 컴파일 | library와 native binary 포함 Mac4-binary product PASS@806; 값/형식 변경 없음 |
| `17bdfd62c2f3` | Linux worker의 unsafe libc 제한 호출을 기존 Rustix 안전 API로 교체 | native worker whole8/8 및 Linux aarch64 4-binary product GREEN@4501; dependency/version·wire 변경 없음 |

모든 scoped 통합은 최신 main descendant에 replay하고 coordinator claim/barrier/source-commit/require-head, boundary before/after, staged hooks를 거쳤다. 다른 작업의 dirty/index를 합치지 않았다.

Relation37은 EXISTS 두0행, 상관/중첩, foreign binding/root, typed source/sort 거절, budget/cancellation, digest/output last-drop를 검증했다. Composition fixture는 실제 frontend DTO와 기존 relation compiler를 사용한다. **공개 Composition 성공 증명으로 확대하지 않는다.**

Graph 변경은 최대16 plain top-level Python 함수의 pass/decimal/direct same-module return call을 bounded census로 연결한다. 미지원·부분 census는 typed Incomplete이며 기존 BFS/identity/root/generation/lease custody를 유지한다. 새로운 Graph engine은 없다.

## 최종 검증 결과와 차단 전제

### Mac — product 완료, 실제 process proof는 bootstrap에서 RED

- 초기 Product 기준 source는 consumer **`7ab464c843466c31cbb8cefac9a49bffb0a12c25`** + producer867 + Index07, 모두 clean. 기존806에서 production 변경 없이 CHECK integration fixture만 고친 후보다. 후속 e0e/2f7도 아래에 source별로 기록한다.
- 기존806 및 새26dd/7ab의 `quanta-cli-product.release-full.build`는 모두 **GREEN/exit0**, immutable4-binary bundle 검증 완료. 7ab product receipt는 `20261005T091528.821620Z-24c696bcc087`이고, 마지막 Mac2f7 product receipt는 `20261005T100319.030844Z-562fa13c7f6c`다.
- 실제 SDK/product 컴파일 전제 E0373/E0583/E0412/E0624/E0753/E0277/E0425는 담당 작업의 committed 수리를 소비했다. Canonical permit seam과 shared wire visibility는 위2개 main checkpoint로 수리했다. 이전 RED는 보존한다.
- **Graph806 actual process RED0/1**, ignored0. `workspace.attach`의 초기 exact-six 실행이 `CallGraphPta/Pta/Execution(PtaOccurrenceAuthorityRefused)`로 실패했다. Graph query 전 단계이며 유한 Graph catalog refusal oracle은 아직 검증되지 않았다. 신규68줄 테스트는 main 미통합이다.
- CHECK 첫 실행은 fixture의 dense configured root와 attach root 불일치로 실패했고, 진단 Python import의 bytecode/cache-directory 생성 때문에 source guard에서도 **INVALID/exit125**였다. 이 시도를 owner proof로 계산하지 않는다. 후속 source probe에는 bytecode 생성을 금지했다.
- CHECK fixture를 source-first/exact configured root/case별freshdaemon으로 고친26dd는 실제 RootScope 검사를 통과한 뒤 누락된 target Python authority로 **RED0/1**. 기존 Graph fixture와 같은 authored Python3.12/src 설정을 추가한7ab는 두 전제를 통과했다.
- **CHECK7ab actual process RED0/1**, ignored0/sourceguard PASS. 초기 Product normalized attribution이 `UnsupportedConstraintKind/unresolved_direct_call_target_v1 caller=app;statement=1;slot=0`로 `print(1)`을 거절했다. CHECK query는 아직 실행되지 않았다. Graph의 PTA authority 실패와 다른 원인이다.
- Signed pattern이 identifier 호출 전체를 매칭함을 확인했다. CHECK 양성 입력을 same-module declared return calls로 바꾼e0e는 독립 AST 기준0/1/2 호출과 모든 target 선언을 만족한다. 서명·query·예산·응답 및 foreign/short oracle은 그대로다. Actual Mac CHECK는 attribution을 통과하고 Graph와 같은 `PtaOccurrenceAuthorityRefused`로 RED0/1이다.
- 추가로2f7의 Mac 거절 입력을 호출0개인 상수 반환 선언으로 한정했다. Linux 양성 Graph/CHECK 본문은 byte-identical이다. Mac2f7의4-binary product는 GREEN이나 Graph/CHECK는 각각 **RED0/1**, failed1/ignored0: 모두 query 이전 initial exact-six Product PTA에서 거절됐다. Immutable run별 stderr와 사후 clean3-repo digest join을 확인했다. 호출0개에서도 같은 오류이므로 단순 미선언 call target이 공통 거절을 설명하지 않는다. 어떤 authority check가 실패했는지는 현재 typed 오류만으로 특정하지 못한다.
- Fixture/input 수리는 boundary before/after와 staged hooks를 통과한 **격리 candidate만**이며 main 미통합이다. 기존0/1/2 결과, foreign/short rejection, WorkerUnavailable 기대값, production RootScope/lowering 거절을 변경하지 않았다.
- 실제 bootstrap 실패와 Linux 설치 authority 전제 부족을 기존 AG5/IRC 담당 작업에 전달했다. 해당 owner의 committed 수리·동일 source proof 및 설치 authority를 확보한 뒤 query를 재검증한다. 새로운 builtin semantics/engine 지원은 이번 maintenance에 추가하지 않는다.
- **남은 통합 조건:** 실제 query까지 도달한 독립1/1 GREEN과 Linux0/1/2 결과 확인 후, 최신 main에 test fixture와 별도 Graph 거절 테스트를 scope별로 합친다.

### Linux — Graph owner 완료, native worker 컴파일 수리 검증

- 과거8GiB Docker 조건의 SIGKILL/OOM3회는 RED/테스트0으로 기록했다. Frontend1/codegen1 시도도 실패했으며 gate나 oracle을 약화하지 않았다.
- 사용자 진행 승인 후 Docker 메모리를 **16GiB**로 변경·재시작했다. 중단 전 proof 컨테이너에 active child가 없음을 확인했고 기존 실행 컨테이너5개를 모두 복원했다. 이미지/volume 삭제는 없다.
- Frozen806/867/07의 native Linux **aarch64** Graph owner는 **GREEN11/11**, failed0/ignored0, jobs1/frontend1. 전체 실행 약79분, peak child RSS9,051,304KiB였다. Immutable receipt: `20261005T080825.880997Z-31332bdffa29`, owner closure digest49e5414b. 이 결과는 설치형 query 증명이 아니다.
- Clean7ab/867/07의 generic product는 **RED/exit101**이다. Linux native worker의 직접 `libc::setrlimit/getrlimit` unsafe4개가 workspace `deny(unsafe_code)`에 걸렸다. OOM0이며 Graph/CHECK 설치 실행은 아직 NOT_RUN이다.
- Parser가 이미 직접 사용하는 pinned **rustix1.1.4**의 `process` feature를 native-worker에 활성화한 candidate4501을 준비했다. 안전한 setter와 fallible `prlimit`의 동일 ceiling 재설정/이전값 확인을 사용한다. AS/DATA 모두 유한한 soft/hard≤요청값, zero/infinity/overflow 거절, syscall 오류의 typed ResourceLimit, allocation 이전 적용, 비Linux 거절을 유지한다. dependency/version 추가나 unsafe lint 예외는 없다.
- **Native worker whole target8/8 GREEN**, failed0/ignored0/filtered0@4501/867/07. 기존 Match3개와 finite/soft/hard/infinite guard 회귀를 포함한다. 이는 OS syscall·실제 Match process의 성공 증명과 구분한다.
- 같은 clean4501/867/07의 generic4-binary product **GREEN/exit0**, 약21분39초. Immutable receipt `20261005T103957.985177Z-b08fc711c0b2`, 전체 primary digest798f152f, owner closure78e56da2. Canonical assembler가 네 바이너리·hash·동일 source/receipt를 검증했다.
- Boundary before/after, staged hooks, coordinator claim/barrier/source-commit/require-head를 통과해 worker **2개 경로만 main17bdfd62c2f3에 통합**했다. Test fixture는 가져오지 않았다. Claim/barrier를 해제했고 두 경로는 clean이다.
- **Linux Graph 실제 process RED0/1**, failed1/ignored0@4501. Immutable receipt `20261005T110342.809277Z-bbf07943cb01` 및 run별 stderr를 확인했다. `workspace.attach` initial exact-six Product가 Mac과 같은 `CallGraphPta/Pta/Execution(PtaOccurrenceAuthorityRefused)`로 거절되어 FIND query는 NOT_REACHED다.
- **Linux CHECK 실제 process RED0/1**, failed1/ignored0@같은4501. Immutable receipt `20261005T113050.261563Z-971dcaa2b43d`다. 서버 시작의 Match worker 준비가 `daemon_gqlang.pattern_provider_config_invalid_v1 / ControlledNativeBudgetUnavailableV3`로 거절되어 attach/query는 NOT_REACHED다.
- CHECK 경로는 `prepare_installed_match_worker_v3`→`from_installed_server_v3`→고정 root의 `verify_installed_product_v1`이다. 컨테이너에 `/etc/semantica-codegraph-v2/trust`가 없으며 canonical profile 검증도 generic bundle을 `copied build receipt differs from requested product build profile`로 거절했다. **정식 설치 authority/환경 전제 부족이며 예산 전달 코드 결함으로 판정하지 않는다.** Worker OS 제한 설치도 이 실행에서는 호출되지 않았다.
- 두 process 종료 후 consumer4501/producer867/Index07의 clean HEAD와 primary798f152f/owner78e56da2를 canonical owner로 재계산했다. Run별 immutable stderr/stdout와 source join을 확인했다. Fixture는 main 미통합이며, 관련2개 claim을 해제해 격리 후보만 보존했다. Active QBC/우리 claim/barrier는 없다.
- 기존 warm lane의 compile-env 변경과 미등록 새 lane은 QBC가 각각 빌드 전에 거절했다. 새 lane을 canonical issuer로 등록했고 기존 lane/cache/receipt를 reset하지 않았다. Product/process는 이미 설치된 Rust toolchain의 LLD shim과 같은 flags를 사용한다. Production source/recipe/feature/profile은 바꾸지 않았다.
- aarch64 generic installed 후보와 공식 **x86_64 installed release profile**을 구분한다. 현재 환경의 PASS는 x86_64 release qualification이 아니다.

## 최신 main과 검증 후보의 차이

- Consumer worker checkpoint HEAD **`17bdfd62c2f3d6624eed9523c6fcef5f72927110`**; 이번 source capture의 main **`3f037ea04eecd80606f045ad9b3a7709c5a17ee7`**. 다른 작업으로 main은 계속 이동한다. Worker2경로와 installed test 경로의 dirty overlay는0개다. Worker2경로는 clean4501의 qualified blob과 같다.
- Source digest 차이는 플랫폼 차이가 아니다. Product806의 전체 primary snapshot은55308640, owner806의 policy/Cargo sibling closure(consumer/producer/Index)는49e5414b다. 기존 canonical owner로 세 저장소의 byte-aware clean HEAD와 두 digest를 독립 재계산해 join을 확인했다. Digest 문자열을 같다고 가정하거나 receipt를 다른 소스로 합성하지 않는다.
- 이후 다른 작업에서 portable parser/adapter의 metering/borrow/staging과 Graph test165줄 등을 수정했다. 원래 Graph owner11 및 parser10/adapter9 receipt는 원래 source에만 유효하다. 과거의 “소유17개 blob 모두 동일” 주장은 더 이상 현재 상태가 아니다.
- 경로별 committed delta와 dirty 상태는 아래 custody JSON에 기록한다. 최신 main 전체 product/설치형 qualification은 **확인하지 못했다**. 검증 후보 결과와 최신 main 결과를 합성하지 않는다.
- Semantic generation carrier3/3은 실제 semantic request 양성 clone/drop proof가 아니다. 기존 SDK source_admission→public ingress에서 Composition→UnsupportedFamily 선거절을 관측했으며, 새 configured-Semantic oracle과의 정합은 활성 소비자 작업에 전달했다. 해당 작업의 최신 source를 덮어쓰거나 새 Composition 기능을 이 maintenance에 추가하지 않는다.
- 최신 main의 `ccd6efbf53` 이후 pre-key native DFG cohort가 `ProductCallGraphSeedConsumptionV1`을 `_seed_lifetime_v1`로 보유한다. `query_impl/query_contract.rs`의 retain 호출→cohort 인자→retained field 연결을 재확인했다. 해당 변경의 Rust/actual bootstrap proof는 확인되지 않았다. Source4501의 receipt를 이 후속 변경이나 최신 main에 합성하지 않는다.
- 다른 소비자 작업의 TS/JS LocalCall assignment 수리는 source64c869에서 owner4/4 GREEN(`20261005T133702.606698Z-f7f4540d8738`)이다. 이는 source4501 Python Product PTA bootstrap의 해소 증명이 아니다.

## 최종 잔여 액션

1. **P0 원인 확인: Product PTA bootstrap 거절.** 기존 AG5/IRC owner가 최초 거절 분기를 특정하고 최신 seed lifetime 수리를 같은 clean source에서 검증한다. 실패가 남으면 해당 owner만 수정한다.
2. **P0 설치 전제: Linux installed authority.** Canonical x86_64 profile, root 소유 immutable slot, AG5 selected phase와 독립 deployment approval/key/active generation을 기존 release owner 경로로 확보한다. Generic aarch64/Mac 결과로 공식 설치 qualification을 대체하지 않는다.
3. **P1 회귀·통합: 동일 tuple의 actual Graph/CHECK와 fixture checkpoint.** FIND PATH/Reachability, empty/budget/stale/foreign, signed CHECK zero/one/multi와 foreign/short를 실제 query까지 검증한다. GREEN인 test scope만 최신 main에 합치고 마지막 source/receipt 정합을 기록한다. Producer core는 입력 변경 없이 CI/fuzz를 반복하지 않는다.

CHECK canonical work 수리는 main 반영과 소유자 proof를 완료해 잔여에서 제거했다.

Fuzz 정기 실행은 로컬 운영 선택으로 남긴다. Hosted CI, 신규 엔진/컴파일러/SMT/library 도입은 잔여 액션에 없다.

## 잘못 남은 작업으로 계산하지 않을 것

- `query.result_shape`: leaf/record/nested 모두 현재 sema typed 거절. 새 지원 구현은 이번 범위가 아니다.
- SHOW를 API별로 구분한다. Pipeline staged syntax→declaration→plan→Execution DTO 양성 경로는 있다. Specialized `admit_show_planned_source_v2` one-shot은 source cap 또는 `IncompleteSourceDiagnostics`로 항상 거절한다. Facade는 SHOW syntax owner만 공개하며 staged owner 성공은 runtime admission이 아니다.
- Rename 포함 conformance19, stale 주석 수리, Graph cap1/dedup 지원 계약, EXISTS 결과 oracle 소스는 이미 반영됐다.
- 물리 optimizer, Graph engine, SMT, 신규 production dependency는 계획에 없다.

## 증거 위치

[Producer checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-main-checkpoint-20261005/checkpoint-result.json)
[Producer clean qualification 원본 log](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-main-checkpoint-qualification-20261005.log) — 관리 checkout/target의 원 receipt 경로는 정리돼 현재 없다. 보존된 log는 source867/1514 PASS/receipt exit0를 기록하며 새 receipt로 재발급하지 않았다.
[Same-source fuzz smoke](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/latest-remediation/producer-checkpoint-fuzz-8675bcd-20261005.json)
[Source별 proof 상태](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/latest-proof-status.json)
[Current-main delta custody](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/current-main-checkpoint-custody.json)
[TemporaryPermit checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/temporary-permit-checkpoint-result.json)
[Shared wire checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/shared-wire-checkpoint-result.json)
[Docker restart recovery](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/docker-memory-restart-result.json)

[Source identity closure join](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/source-identity-domain-join-806.json)
[Graph actual attach RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/installed-mac-graph-attach-failure.json)
[CHECK actual attribution RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/installed-mac-check-attribution-failure.json)
[Mac zero-call bootstrap failures](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/installed-mac-zero-call-bootstrap-failures.json)
[Linux product unsafe compile RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/linux-installed-product-unsafe-compile-failure.json)
[Native safe limit candidate](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/mac-native-owner-8be-20261005/native-safe-rlimit-preparation.json)
[Native safe limit main checkpoint](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/native-safe-rlimit-checkpoint-result.json)
[Linux Graph actual bootstrap RED](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/installed-linux-graph-bootstrap-failure.json)
[Linux same-source process receipts](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/installed-linux-same-source-process-results.json)
[Linux installed trust precondition gap](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/graph-bounded-checkpoint-20261005/linux-installed-trust-precondition-gap.json)
