# CG-01: Execution contract가 Semgrep typed-error 규칙에서 빠짐

- 판정: `CONFIRMED_DEFECT` (P2 정책 커버리지). QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`.
- 소유자: `.semgrep/qglang-frontend.yml:10-27`의 include 목록과 `tools/lint/semgrep_gate.py:22-35`의 `CORE_CRATES`. 새 `crates/quanta-gqlang-v2-execution-contract/src`가 양쪽에서 빠진다. CircleCI → `just ci` → `just semgrep`은 이 목록을 사용한다.
- 독립 반례: 격리된 임시 Git repo에 같은 `pub fn bad() -> Result<(), String> { Ok(()) }`를 Pipeline과 execution-contract의 `src/lib.rs`에 둔 뒤 현재 Semgrep 설정으로 스캔했다. Pipeline 위반 1건만 보고되고 새 crate 경로는 scan 대상에도 없었다. 게이트의 expected-path 생성도 실제 새 crate `src/lib.rs`를 제외해 누락을 green으로 수용한다.
- 기대/실제: DTO·projection·encoding과 typed failure를 소유하는 새 crate에도 같은 typed-error 규칙을 적용한다면 위반을 탐지해야 한다. 실제로는 적용되지 않는다. 현재 새 crate에서 공개 `Result<_, String>` 위반은 발견되지 않았으므로 runtime 버그로 분류하지 않는다.
- 보완 수락: 규칙 include와 `CORE_CRATES`를 함께 확장하고, 경로를 한쪽에서만 빼거나 새 crate에 위반을 넣으면 실패하는 non-vacuity/negative fixture를 추가한다. `just semgrep` 후 통합 source에서 `just ci`를 실행한다.
- 상세 실행 결과와 제한: [레인 보고](03-contract-gates.md). Hosted CircleCI 실행과 수정 후 CI는 `NOT_RUN`.
