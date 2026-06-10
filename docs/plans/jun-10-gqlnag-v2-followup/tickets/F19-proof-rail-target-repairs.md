# F19 — J01/J02A/J09C proof-rail 타겟명·touch-file 수리 (W5, joint)

원천: J-ingress-handshake-matrix + J-caller 감사 CH2. 티켓이 명명한 cargo test 타겟이
실제 파일명과 달라 **acceptance 커맨드 자체가 hard fail** 하는 케이스들.

## Gap 목록 (라이브)

1. **J01** — Proof Rail(J01:61) `--test select_vertical_slice` vs 실제 ir-side 파일
   `quanta-gqlang-v2-ir/tests/j01_select_vertical_slice.rs` → "no test target named" hard fail.
   (semantica-side `exec-conformance/tests/select_vertical_slice.rs` 는 존재 — external 쪽만 불일치)
2. **J02A** — Proof Rail(J02A:56) `--test j02a_handshake` vs 실제 `tests/j02a_find_handshake.rs` → 동일 hard fail
3. **J09C** — 최악 케이스:
   - Proof Rail(:56) `-p quanta-gqlang-v2-cli --test explain_path_handshake` — CLI 에는 `explain_path_ux.rs` 만 존재
   - named create file `cli/tests/explain_path_handshake.rs`(:17) 부재
   - named touch `ir/src/retrieval/spec.rs`(:23) 부재 (flat retrieval_ir.rs 구조 — F04 가 spec.rs 신설 예정)
4. **J09A** — named touch `cli/src/starter_scaffold.rs`(:22) 부재 (F08 에서 해결)

## 작업

1. 정합 방향 결정 (titcket-rename vs file-rename) — 권장: **파일명을 티켓 타겟명으로 rename**
   (acceptance 커맨드가 SSOT; 티켓 개정은 이력 오염):
   - `j01_select_vertical_slice.rs` → `select_vertical_slice.rs`
   - `j02a_find_handshake.rs` → `j02a_handshake.rs`
   - CLI: `explain_path_ux.rs` 를 `explain_path_handshake.rs` 로 rename 하거나 handshake 전용 테스트 신설
2. J09C 의 ir touch-file 은 F04 의 `retrieval/spec.rs` 신설로 충족 — F04 완료 후 본 티켓에서 확인만
3. 수리 후 각 티켓의 Proof Rail 커맨드를 **그대로 복붙 실행**해서 green 로그 확보
4. 전 joint 티켓의 Proof Rail 커맨드 전수 dry-run (`cargo test --no-run --test <name>`) 으로
   동류 불일치 추가 발굴 (이번 감사는 J01/J02A/J09C 만 확인)

## DoD

- 모든 joint Proof Rail 커맨드가 타겟-부재 에러 없이 실행됨
- F01 완료 전제 하에 J01/J02A rail green
