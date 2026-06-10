# F15 — closeout gate 3대 구조 결함 수리 (W3, semantica)

원천: S-runtime-fold / S-proofpack / S-ledger 감사. 메인 세션에서 survivor-모순과 S17–20 collapse
직접 재검증 완료. **이 티켓이 끝나야 다른 모든 "gate PASS" 가 증거 능력을 가짐.**

대상: `tools/ci/gates/gate-gqlang-v2-semantica-closeout.sh`

## 결함 목록 (라이브)

1. **S15/S16 survivor 자기모순**
   - `:120-126` `SHALLOW_FOLD_DELETE_CRATES` 에 `quanta-runtime-query-preset-wire` 포함 (`:122`) —
     이 crate 는 **살아있는 survivor owner** (dir 존재 + quanta-v2 Cargo.toml:150,370 멤버)
   - `:836-839` `check_fold_closeout` 이 이 배열로 `require_deleted_crates_absent` 실행,
     `:1061-1063`(S15) / `:1088-1090`(S16) 부모 arm 이 이를 호출 → **부모 게이트 통과 불가능**
   - S15A.md:68 "wrong-target bug 수정됨" 주장은 child arm(`:1044-1045`)에만 반영, 공유 배열엔 미반영 — 거짓 evidence
2. **S17–S20 비차별 arm**
   - `:1091-1093` `S17|S18|S19|S20)` 가 한 arm — `check_proof_pack_rows`(`:851-857`) 가 4 row 무조건 grep.
     단일 티켓 단독 fail 불가 → per-ticket PASS 의 증거 가치 0
3. **staleness 세탁 (S09)**
   - `:744-759` `check_snapshot_generator_stability` 가 committed 레저를 in-place regen 후
     **fresh 두 번을 서로 diff** — committed-vs-HEAD 비교 없음, dirty-tree 가드 없음
     (`rg "git diff|git status|porcelain|dirty"` = 0). CI 가 stale 아티팩트를 매 실행마다 세탁
4. **(부수) 게이트 사각지대 기록**
   - S06 gap (catalog_adapter/search ownership) 을 게이트가 구조적으로 못 봄 (`:531,953` 이 tautology 파일을
     실행); S13B planner-v1 tooling 잔존(yaml/policy/lint)도 게이트 비커버 — F13/F18 에서 소스 수정 후
     게이트 needle 추가

## 작업

1. `SHALLOW_FOLD_DELETE_CRATES:122` 를 legacy shim `quanta-runtime-gqlang-query-preset-wire` 로 교정
2. S17–S20 arm 분리: row 별 단독 검사 함수 (F12 의 registry 분리와 짝) — 각 `--ticket SNN` 이 해당 row 만으로 fail 가능
3. snapshot stability 검사에 regen-vs-committed `git diff --exit-code` 단계 추가 (stale 커밋 아티팩트 검출)
4. 신규 needle: catalog_adapter 존재(F13 후), search ownership 실제 assert 파일, S13B tooling 잔존
   (catalog yaml/boundary policy/lint 의 deleted-crate 참조 금지)
5. 수리 후 전체 재실행: `--ticket S15`, `--ticket S16`, `--ticket S17..S20`, `--ticket S09B/S09C` 및 `--all`

## DoD

- 부모 S15/S16 게이트가 현재 head 에서 정당하게 PASS (survivor 모순 해소)
- S17–S20 각각 단독 fail 가능함을 인위 결손으로 검증
- stale 레저 커밋 시 게이트가 FAIL 함을 검증 (negative 시험)
