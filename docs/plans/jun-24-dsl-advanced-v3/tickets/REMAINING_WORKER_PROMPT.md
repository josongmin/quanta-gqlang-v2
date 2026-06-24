# QGLang Remaining Worker Prompt

Use this prompt when the worker should close the remaining `quanta-gqlang-v2`
work only, not the semantica execution packet.

```text
/Users/songmin/Documents/code-new/quanta-gqlang-v2 에서 작업해라.

목표:
`quanta-gqlang-v2` 쪽 남은 일만 닫아라.
이번 턴의 범위는 qglang repo-local closeout과 packet SSOT 정리다.
semantica 코드 실행은 하지 말고, semantica D5 상태는 입력 조건으로만 취급해라.

반드시 먼저 읽어라:
1. `docs/plans/jun-24-dsl-advanced-v3/tickets/WORKER_START_HERE.md`
2. `docs/plans/jun-24-dsl-advanced-v3/tickets/10-semantica-serving-contract-handoff.md`
3. `docs/plans/jun-24-dsl-advanced-v3/tickets/16-current-main-proof-and-artifact-hygiene.md`
4. `docs/plans/jun-24-dsl-advanced-v3/tickets/17-downstream-cutover-closeout-and-packet-sync.md`
5. `/Users/songmin/Documents/code-new/semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/README.md`

현재 truth:
1. qglang repo-local proof refresh는 이미 `2026-06-25` current head에서 녹색이다:
   - `git diff --check`
   - `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate --quiet`
   - `cargo test --workspace --quiet`
   - `cargo clippy --workspace --all-targets --quiet`
2. qglang 쪽 남은 로컬 delta는 `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs` 한 파일뿐이다.
3. 이 diff는 기능 확장 lane이 아니라 assert formatting/readability churn이다.
4. semantica 관계 정의는 이미 qglang packet에 박혀 있다.
5. semantica D5가 안 닫히면 ticket `17`은 blocked로 남겨야 한다.

실행 순서:
1. `git status --short --branch`와 `git diff -- <that file>`로 마지막 tracked diff를 재확인
2. `v3_family_hellgate.rs` diff를 keep 또는 revert 중 하나로 정리
   - 새로운 semantics/coverage가 없고 formatting churn이면 revert 우선
   - readability 목적이라면 `cargo fmt` 결과와 함께 intentional change로 남겨라
3. proof refresh를 다시 실행
   - `git diff --check`
   - `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate --quiet`
   - `cargo test --workspace --quiet`
   - `cargo clippy --workspace --all-targets --quiet`
4. ticket `16`을 current-head truth에 맞게 닫아라
5. semantica D5가 아직 open이면:
   - ticket `17`은 blocked 이유를 명시하고 여기서 멈춰라
6. semantica D5가 green이면:
   - `README.md`, `WORKER_START_HERE.md`, `SERVING_HANDOFF.md`, `10`, `16`, `17`을 최종 SSOT로 sync해라

중요 규칙:
- qglang repo에서 semantica runtime ownership을 먹지 마라
- semantica workspace/member 관계 규칙을 다시 흔들지 마라
- false green 금지
- stale dirty-tree 서술 금지
- untracked scratch artifact를 새로 만들지 마라

최종 보고 형식:
1. qglang repo-local closeout 상태 한 줄
2. 마지막 tracked diff 처리 결과
3. proof 명령 pass/fail
4. `17`의 상태:
   - blocked on semantica D5
   - or synced and closed
```
