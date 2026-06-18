# jun-10 gqlang-v2 followup — 작업 패킷 + 진행 대시보드

> **2026-06-15 authority update**
>
> 이 문서는 external coordination/historical packet이다.
> live closeout truth는 더 이상 이 파일이 아니다.
>
> current authority:
>
> 1. semantica live code
> 2. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/STATUS.md`
> 3. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh`
> 4. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/docs/operation/jun-15-qglang-v2-authoritative-closeout-state.md`
>
> do not treat `F18b` / `F21` in this file as semantica gate IDs.
> use them only as historical coordination labels.

기원: RFC `2026-06-05-rfc-qglang-v2-semantic-host-dx-platform` 전체를 2-라운드 적대적→객관적 감사로
검증 → 21개 follow-up 티켓(F01–F21). 아래 대시보드는 **jun-10 재점검(코드 상태 직접 확인)** 기준.

## 현재 상태 대시보드 (jun-10 재점검)

**✅ DONE — cargo test/metadata 로 검증됨 (12 티켓 + 부분)**

| 티켓 | 검증 |
|---|---|
| F01 composition ingress arm | cargo green (V6A 18 + 외부 grammar drift 대응) |
| F09 CHECK 문법 | external workspace green |
| F10 parser foundation (agent) | external green, proof-rail 4종 PASS, typed_ast 삭제 |
| F11 contract (S01+S02+S03 전체) | exec-contract 17 + diagnostics 4 + host_spi 7 green |
| F12 proof-pack (SSOT+claim_state+S17-20+S04B) | proof_pack_registry 7 green |
| F13 adapter (S05A+S06B+S06A 전체) | host-adapter green |
| F14-S07C residue guard (+drain) | bash-replicate + compile green (S08C=문서화 결정) |
| F15 closeout gate 3 fixes | hellgate 11 + mutation proof |
| F16 generator alias + ledger regen | pytest + regen |
| F17 digest alias drain | compile-verified (13 manifest + 17 .rs) |
| F18 삭제 4/14 | cargo-metadata-verified |

빌드 복구 후 검증이 실제 이슈 4건 포착·수정: bridge API-drift 5곳, seam-plane, composition grammar drift, CandidateBasisV1 충돌.

**✅ External W1 코어 전부 DONE (jun-10)** — F02·F03·F04·F05·F06·F07·F08 모두 헬게이트 드리븐(e2e RED → TDD → 적대/객관 어딧 → 보강)으로 완료. workspace 100 test-bin green, clippy gate(`--lib/bins/examples`) + `--tests` 0 warn.

| 티켓 | proof rail | 핵심 |
|---|---|---|
| F02 | `sema --test admissibility_matrix`(9) | E05A/B/C/D admissibility 서브시스템 |
| F03 | `ir planning_producer`(5)+`sema ai_profile`(3)+`cli query_plan_summary`(4) | E06C planning producer + E05E AI-lane admissibility + CLI plan 소비자 |
| F04 | `ir --test product_ir`/`typed_retrieval_surface`, `sema --test fragment_policy`(6) | E06B product IR + E07 typed retrieval/fragment |
| F05 | `sema --test select_metric_target`(3) | E11C metric inventory 어휘 + `metric.<name>` |
| F06 | `ir/cli/lsp --test fact_*`(4+2+2) | E11A fact schema → DESCRIBE catalog root 배선 |
| F07 | `sema --test trust_policy`(5) | E10B REQUIRE CERTIFIED fail-closed admission |
| F08 | `cli zero_query`(3)+`handshake`(2)+`ir zero_query_scaffold`(4) | E12A scaffold core→ir + CLI zero-query verb |

**🔲 REMAINING — 미착수**

| 묶음 | 티켓 | 비고 |
|---|---|---|
| Joint (external 측 DONE) | F19·F20 | ✅ external proof-rail 타겟명 정합 + fixture behavioral 승격 완료. 잔여: F19 semantica dry-run, F20 J09C ExplainHost/PathHost impl(semantica) |
| Semantica 삭제 마이그레이션 | F18-10 | 10개 crate 의 실소비처 재배선 후 삭제 = 진짜 S13B/S12B |
| Semantica V6B cutover | F21 | compat carrier 15+ 소비처 cutover (대형) |
| 자투리 | planner-v1 tooling 11파일, F14-S08C feature 레일 | 파일별 확인 필요 |

**이 레포(quanta-gqlang-v2/external) 측은 전부 DONE.** 잔여 작업은 전부 semantica-codegraph-v2 워크트리.

### Semantica 최종 어딧 (jun-10, 4영역 실코드 검증)

**🔴 stale claim 정정**: F18·F21 의 "quanta-runtime 미컴파일 → 빌드 복구 후" 전제는 **틀림**. `quanta-runtime`(cargo check EXIT=0)·memssa·exec-contract·exec-host-spi·host-adapter **전부 클린 컴파일**. 유일 블로커는 `codeagent-quanta-bridge` 바이너리(feature flag 누락, gqlang 무관). **→ F18 재배선·F21 컷오버 지금 unblocked.**

| 영역 | 실제 상태 (검증) |
|---|---|
| F19-sema | 42 test 중 J03A `show_describe_handshake.rs` **1개 부재** → rail hard-fail. 나머지 18 OK |
| F20-J09C | ExplainHost(1)/PathHost(4) trait 존재, **production impl 0**, test엔 `unreachable!` Dummy뿐. `explain_path_handshake.rs`(sema) constant-mirror — 메서드 미호출 |
| F21 | carrier 3종 존재(dual #[path]). 실 prod 생성사이트 **4개**(15+ 아님). 목적지 `ServingQueryRequestV1`+`lower_public_ir_root_to_request_v1` 존재. `#[deprecated]`/변환어댑터 미착수. "17 test err" 결함 해소됨 |
| F18 | **5 ABSENT**(이미삭제 0소비처) + **12 PRESENT**(실소비처: catalog-binding 21·validation 18·fragment 13·core 10·catalog-v1 5·hir-v1 4·sdk-handlers 4·gates-v1 2·sarif-proj 2·lowering 1·planner/contract 12×2). 안전삭제 0 — 재배선 필요. 게이트/lint/catalog residue 다수 |

### Semantica 작업 계획 (의존순 — 보강판)

> **🎯 잔여 작업(F18b·F21·J06)의 확정 SSOT: [FINAL-PLAN.md](FINAL-PLAN.md)** (jun-11, Track A/B/C).
> 핵심: F21 carrier demotion 은 별도 티켓이 아니라 Track C(v2 프런트엔드 입구 전환)에 흡수됨.
> 아래 표의 #4(F21)·#5(F18b)·#6(J06) 항목은 FINAL-PLAN 의 Track 구조로 대체.
>
> **📋 구체 실행계획(파일·타입·시그니처·헬게이트·검증커맨드 단위): [SEMANTICA-EXECUTION-PLAN.md](SEMANTICA-EXECUTION-PLAN.md)** (4영역 병렬 어딧 기반 — 배치 A 는 완료, F18b/F21/J06 은 FINAL-PLAN 참조).

| # | 작업 | 크기 | 의존 | 병렬군 |
|---|---|---|---|---|
| 1 | **F19/J03A**: `show_describe_handshake` **sema(exec-conformance)+ir(external)** 신설 (CapabilityHost behavioral) | S | — | A |
| 2 | **F20-J09C**: `explain_path_handshake`(sema) behavioral — Dummy impl 실호출 + neg≥2 | S–M | — | A |
| 3 | **F18a**: 삭제된 5크레이트 tooling residue 청소(baseline/lint/catalog yaml/gate needle) | S–M | — | A |
| 4 | **F21**: carrier 강등 — adapter `from_gqlang_execution_request_v1`+`#[deprecated]`+**진짜 컷오버 2사이트**(나머지 pre-planning/subquery는 allow)+guard+round-trip | M | — | B |
| 5 | **F18b**: **10 gqlang 크레이트**(planner 제외) 재배선 후 삭제 — topo: leaf(validation/fragment/gates/sdk-handlers/sarif-proj)→catalog-binding/hir→catalog-v1→core/lowering | **L** | 빌드 OK | C |
| 6 | **J06 closeout**: ledger regen→S-ticket complete→joint bundle(J03A 포함)→`gate --all` | M | 1·3·4·5 | D |

**정정**: F18b는 **10** 크레이트(bare planner/planner-contract는 gqlang-* 아님 → 범위 밖). F21 실 컷오버는 **2사이트**(15+ 아님). F19 J03A는 **external+sema 양쪽** 파일 부재.

**실행 순서:** 배치A(1+2+3 병렬, quick win) → B(4 F21) → C(5 F18b, critical path, 단계별 `cargo check -p`) → D(6 J06).

⚠️ **착수 전 결정**: semantica `main` 에 uncommitted WIP(memssa/taint) 존재 → 작업 브랜치 분리 방침 필요.

**권장 다음 순서:** ~~External W1~~ ✅ → ~~F19/F20 external~~ ✅ → **Semantica 배치 A(1·2·3) 병렬** → B(F21) → C(F18b) → D(J06).

---

## Wave 순서 (의존성 기준 — 원본 구조)

### W0 — 빌드 차단 해제 (최우선, 다른 모든 joint proof 의 전제)
- [F01](tickets/F01-composition-ingress-bridge-e0004.md) — ingress-bridge E0004 (Composition arm 누락) 수정 + V6A full-loop 재증명

### W1 — External 미구현 코어 (quanta-gqlang-v2)
- [F02](tickets/F02-sema-admissibility-subsystem.md) — E05A/B/C/D sema admissibility 서브시스템 신설
- [F03](tickets/F03-ai-profile-and-planning-ir-wiring.md) — E05E/E06C AI profile sema + planning IR 프로듀서 배선
- [F04](tickets/F04-product-ir-and-typed-retrieval.md) — E06B product IR + E07A/E07B typed retrieval/fragment policy
- [F05](tickets/F05-metric-inventory-vocabulary.md) — E11C metric inventory 어휘 + `metric.<name>` 표현식 타겟
- [F06](tickets/F06-catalog-fact-schema-wiring.md) — E11A fact schema 를 SHOW/DESCRIBE catalog root 에 배선
- [F07](tickets/F07-trust-admission-semantics.md) — E10B REQUIRE certified admission/trust 의미론
- [F08](tickets/F08-cli-zero-query-seam.md) — E12A CLI zero-query/scaffold seam
- [F09](tickets/F09-check-grammar-repair.md) — E02C CHECK implicit class inference 거부 + `emit` + typed shape
- [F10](tickets/F10-parser-foundation-debt.md) — E01/E03B/E03D/E04C proof-rail 부채 + typed_ast dead-code 해소

### W2 — Semantica 계약/어댑터/SDK 보수
- [F11](tickets/F11-contract-gaps-s01-s03.md) — S01/S02/S03 계약 gap (CandidateBasisV1, finding plane, export_kind, 8 trait conformance)
- [F12](tickets/F12-proof-pack-devacuization.md) — S04A/S11/S17–S20 proof-pack registry 탈공허화 + claim_state 강제
- [F13](tickets/F13-adapter-gaps-s05-s06.md) — S05A/S06A/S06B 어댑터 gap (reject 테스트, catalog_adapter.rs, 진짜 ownership proof)
- [F14](tickets/F14-sdk-tail-s07c-s08c.md) — S07C named guard + S08C default-feature command path

### W3 — 게이트/레저 수리 (semantica)
- [F15](tickets/F15-closeout-gate-repairs.md) — closeout gate 3대 결함 (S15/S16 survivor 모순, S17–20 비차별, staleness 세탁)
- [F16](tickets/F16-ledger-resync-and-alias-awareness.md) — S09B/S09C/J06 레저 재동기화 + generator alias 인지

### W4 — 삭제 웨이브 실행 (semantica)
- [F17](tickets/F17-digest-alias-drain.md) — digest-v1 package-rename alias 30파일 드레인 + orphan dir 정리
- [F18](tickets/F18-delete-waves-s12b-s13.md) — S12B/S13A/S13B잔여/S13C/S13D 삭제 실행

### W5 — Joint closeout
- [F19](tickets/F19-proof-rail-target-repairs.md) — J01/J02A/J09C proof-rail 타겟명/touch-file 수리
- [F20](tickets/F20-behavioral-handshake-upgrades.md) — J08A/J08B/J09A/J09B/J09C mirror-tautology → behavioral 승격
- [F21](tickets/F21-v6b-carrier-demotion-and-j06.md) — V6B compat carrier demote + J06 최종 closeout

## 병렬화 규칙
- W0 는 단독 선행 (J* 전부의 빌드 전제)
- W1 내부는 전부 상호 독립 (F02–F10 병렬 가능). 단 F03 은 F02 의 sema 모듈 골격을 공유하므로 같은 worker 권장
- W2/W3 은 W0 와 독립이라 즉시 병렬 시작 가능. F15(게이트 수리)는 F12·F18 의 검증 레일이므로 W4 전에 완료
- W4 의 F17 은 F16(generator alias 인지)과 교차 검증 관계 — F17 완료 후 F16 의 ledger regen 재실행
- W5 는 W0 완료 후 착수

## 완료 판정
각 티켓 DoD + 아래 프로그램 레벨 조건:
1. external `cargo test --workspace` green (이 repo)
2. semantica `bash tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh --all` PASS — 단 F15 의 게이트 수리가 선행되어야 이 PASS 가 의미를 가짐
3. V6A 4종 full-loop + V6B handshake bundle fresh green
