# Lane F 최종 감사 — 확정 실제 문제 (findings-6)

- **대상**: `/tmp/dsh-laneF-snapshot` — HEAD `eaeb68ea2711d411616511713fb016d828cd80be` (main) + dirty 오버레이 12경로. 원 checkout 미수정.
- **방법**: 동일 snapshot에 독립 2회 병렬 정적 감사(pass1/pass2) → coordinator 교차 반증 → 아래 확정 항목만 남김. cargo/게이트 미실행(audit-only static; 전 항목 file:line 정적 입증).
- **결론 등급**: P0–P2 **없음**. 아래는 양 패스가 교차 확인하고 coordinator 반증 시도 후에도 살아남은 실제 문제(전부 P3).

---

## F-1. "표준" 예산 프로파일 이중 분기 — STRUCTURAL_RISK (P3, 확정)

- **위치**: `crates/quanta-gqlang-v2-surface/src/budget.rs:94-104` (`ParseBudgetV2::standard_v2`: tokens 65,536 / expr_nodes 131,072 / projection 1,024) vs `crates/quanta-gqlang-v2-sema/src/authority.rs:661-686` (`FrontendBudgetV2::standard_v2`: tokens 131,072 / expr_nodes 262,144 / projection 4,096).
- **경로**: pipeline은 `compile.rs:792-794`에서 `ParseBudgetV2::try_new(budget.max_tokens()…)`로 파생 → shipped front doors(CLI/stdio/pipeline)는 131,072로 자기일관적. 그러나 crate 공개 편의 진입점 `surface::parse_expr`(`lib.rs:112`), `parse_predicate`(`lib.rs:137`)는 자체 `standard_v2()`(65,536)를 씀.
- **실제 분기**: 65,537–131,072 토큰 문서가 crate API에선 거부되고 front door에선 통과. 양쪽 모두 fail-closed라 오작용은 아니나, 두 "standard" 사이 동기화 rail이 없어 향후 silent drift 구조.
- **반증 시도**: shipped doors 정합 확인(위 경로), 한도 초과 시 양쪽 모두 typed `resource_exhausted` 확인 — acceptance 범위 불일치 자체는 남음.
- **최소 수정**: surface 기본값을 `FrontendBudgetV2::standard_v2`의 파생값으로 통일하거나, 공통 상수 owner를 한 곳(sema authority 또는 hir)에 두고 양쪽이 참조. regression oracle: 두 프로파일 필드 전수 비교 테스트.
- **책임 crate**: `surface` (+`sema`).

## F-2. stdio seam에서 typed 오류 평탄화 + secondary span 누락 — STRUCTURAL_RISK (P3, 확정)

- **위치**: `crates/quanta-gqlang-v2-diagnostics-stdio/src/handlers/v3_diagnostics.rs:52-55` — `FrontendAuthorityErrorV2`를 `code + message` 문자열 연결 후 generic `"authority_error"`로 평탄화(원 code는 message 안에만 존재). 동일 handler `:73-85` + `server.rs:22-31` — `WireDiagnosticV2`는 `FrontendDiagnosticV2.secondary()`(`sema/src/diagnostic.rs:88`에서 정렬·dedup되어 생산됨)를 전송에서 전부 드롭. primary span/code/severity/message는 보존.
- **영향**: 모호 참조 등 secondary 위치가 필요한 진단이 stdio 소비자에게 정보 손실된 형태로 도달. AGENTS "typed errors with source spans" 위반의 경계 사례 (owner→transport 경계).
- **반증 시도**: primary 필드 전달 정상 확인, protocol 스키마 고정(`deny_unknown_fields`) 확인 — secondary 부재는 wire 스키마 자체에 없어 남음.
- **최소 수정**: `WireDiagnosticV2`에 secondary 배열 추가(버전 담아) + authority error code를 별도 필드로. oracle: secondary 2개를 갖는 진단의 wire 골든.
- **책임 crate**: `diagnostics-stdio`.

## F-3. surface crate 내 source-bytes 차원 부재 — STRUCTURAL_RISK (P3, 확정, 방어심도)

- **위치**: `surface/src/budget.rs:9-24` — 7개 차원에 source bytes 없음. 주석/공백은 lexer가 무차감 skip, 단일 토큰 길이 무상한. 실제 제한은 pipeline ingress(`compile.rs:830-849`)과 stdio(`server.rs:119-128`)에서만.
- **영향**: crate API 직접 호출자(`parse_expr` 등)는 source 길이 상한 없이 파싱 진입. in-repo 우회 호출자는 미발견이므로 현재 오동용 없음 — 방어심도 갭.
- **최소 수정**: `ParseBudgetV2`에 `source_bytes` 차원 추가 후 `parse_*` 진입에서 `source.len()` 선차감. oracle: cap/cap+1 bytes 거부 테스트.
- **책목 crate**: `surface`.

## F-4. 응답·진단 상한이 전량 구성 후에 검사 — STRUCTURAL_RISK (P3, 확정, 순서 갭)

- **위치**: `server.rs:157-168`(`serialize_bounded` — 직렬화 완료 후 bytes 비교), `handlers/v3_diagnostics.rs:27`(`enforce_diagnostic_cap` — 전체 분석 후 count 비교).
- **영향**: fail-closed이고 상한들은 상호 유계(source bytes → tokens → nodes → diagnostics → response)라 무한 자원은 아님. 다만 최악 할당이 상한 도달 한 번에 발생. 측정 없음 — 비용 수치 주장 없음(정적 순서 사실만).
- **최소 수정**: 진단 컬렉션을 limit에서 stop하는 스트리밍/조기 종료. 우선순위 낮음.

## F-5. P3-format-closure rail이 미커밋 — clean-HEAD receipt 부재 (확정, 상태)

- `pipeline/tests/v3_format_budget.rs`(untracked) + ledger 갱신(`test_hellgate_ledger.md:50`)은 `just ci` 경로에서는 유효(cargo 자동 수집 + nextest inventory reconciles, ledger 등재 확인). 그러나 `qualify_local.py`의 clean-HEAD/untracked 검사가 현재 dirty tree를 거부 → 해당 작업은 커밋 전까지 qualification receipt 없음. 원천 결함은 아니고 감사 시점 상태 기록.

---

## 반증되어 제외된 후보 (실제 문제 아님, 근거 요약)

- **parse 재귀 중 stack overflow**: `cursor.rs:51-59 with_recursion_depth`가 모든 재귀 커널(`expr.rs:38,44,84,410`, `shape.rs:20`)에서 body **앞에** `enter_expression_depth` 선차감. 좌결합 체인은 반복 루프. 사후 `check_item`은 `hir/depth.rs:53-256` 명시적 스택 순회(스택 안전).
- **off-by-one**: 컬렉션/projection 차감은 push 전 pre-add incremental; rail이 width-1-admits-1, depth 64/65, relation 128/129 exact (limit, limit+1) 쌍을 고정.
- **오류 시 relation depth 누출**: `query.rs:74`에서 복원.
- **diagnostic_limit 단일 진단 가정 취약**: `compile.rs:873 cap_transaction_error`가 초과 시 진단 목록 전체를 단일 마커로 치환 — handler의 `if let [diagnostic]`는 producer 규약과 정합.
- **HashMap 순서 → 공개 출력**: production src 유일 HashMap은 `hir/integrity.rs:602` seen-set(멤버십 전용). sema/pipeline/frontend-contract src에는 HashMap/HashSet 전무, 정렬은 전부 total-key.
- **float 비결정적 출력**: 원본 소스 텍스트 통과(`format.rs:687`).
- **ledger 드리프트**: 인용 86개 rail 경로 전부 실재, 역방향 커버리지 완전.
- **게이트 공동화**: semgrep non-vacuity(positive fixture/드리프트 거부), architecture(default-deny allowlist), external consumer(exact-count Counter + missing/extra/duplicate), run_local(그룹 간 소스 드리프트 거부 + nextest 스트림 hash 재검증) 모두 실질 오라클.
- **`FrontendBudgetV2::try_new` 22-인자 positional 어긋남**: 22/22 정렬 확인.

## 경계 기록

- snapshot에서 `docs/audits/`(타 lane 산출물)가 누락 — snapshot 생성 스크립트의 미추적 디렉터리 비재귀 복사 한계. Lane F 조사 대상 밖이라 본 findings 영향 없음.
- 미실행: cargo build/test, `just ci`, `just qualify-local`(dirty snapshot이므로 qualification 주장 불가). 모든 결론은 정적 file:line 증거.
- handoff: (1) F-1은 lane A registry/budget authority 목록과 병합 필요. (2) F-2 secondary 생산자는 lane C(sema). (3) logic/planning의 rule/witness/SCC 내부 루프 과금 본문은 lane D 소유 — 본 lane은 caps+테스트 존재만 확인.

## 판정

**증거 범위 내 일관됨** — confirmed P0–P2 결함 없음. P3 구조 위험 4건 + 상태 기록 1건. 전역 "중복 0개" 주장 아님(조사 경계는 위 커버리지 참조, `/tmp/laneF-pass{1,2}*.md` 원문 보존).
