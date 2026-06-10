# jun-10 gqlang-v2 followup — 작업 패킷 + 진행 대시보드

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

**🔲 REMAINING — 미착수 (코드 상태로 확인)**

| 묶음 | 티켓 | 비고 |
|---|---|---|
| **External W1 코어 (최대 미착수 덩어리)** | F02·F03·F04·F05·F06·F07·F08 | sema admissibility/AI profile/product IR/metric/fact wiring/trust/cli zero-query 전부 부재. **external 빌드 green → runtime 무관, 즉시 착수 가능.** |
| Semantica 삭제 마이그레이션 | F18-10 | 10개 crate 의 실소비처(runtime/execution/query-owner-surface) 재배선 후 삭제 = 진짜 S13B/S12B |
| Semantica V6B cutover | F21 | compat carrier 15+ 소비처 cutover (대형) |
| Joint | F19·F20 | proof-rail 타겟명 정합 / mirror-tautology 행동승격 (runtime green → 검증 가능) |
| 자투리 | planner-v1 tooling 11파일, F14-S08C feature 레일 | 파일별 확인 필요 |

**권장 다음 순서:** External W1(F02→F04→F05→F03/F06/F07/F08, 빌드빠름·즉시 가능) → Joint F19/F20 → Semantica 대형 마이그레이션 F21 → F18-10.

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
