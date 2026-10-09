# B. CG-01 Semgrep execution-contract coverage

## Source state and acceptance

- Starting QGLang HEAD: `25fad76cf5b4510244157c2e009b4dc37aa958ff`. This patch changes only the Semgrep rule, its path inventory gate, and the gate's focused test. Shared dirty documents were not modified.
- Acceptance: `crates/quanta-gqlang-v2-execution-contract/src/**/*.rs` is scanned by the typed-error rule; the gate rejects a missing scanned path or any production `Result<_, String>` finding; an isolated violation in the new owner is detected.

## Change

- `.semgrep/qglang-frontend.yml`: added the execution-contract source glob to the rule's include list.
- `tools/lint/semgrep_gate.py`: added `execution-contract` to `CORE_CRATES`, so the actual scanned-path set must contain all its Rust sources. The existing AST backstop and zero-production-finding rule now cover those files too.
- `tools/lint/tests/test_semgrep_non_vacuity.py`: added an explicit new-owner inventory assertion, a missing-path rejection, and an isolated temporary Rust file containing `pub fn bad() -> Result<(), String>` at the execution-contract path. The test runs the real Semgrep rule and requires one finding at that exact path and line.

## Evidence

- Before patch, an isolated temporary execution-contract file containing the violation produced scanned paths `[]` and findings `[]` under the existing rule.
- After patch, `just semgrep` exited 0: focused unit gate selected/executed 7/7, skipped 0; production Semgrep gate reported 0 findings and 3 expected positive fixture findings. The path inventory and AST backstop ran on the current source.
- The isolated new-owner probe returned one scanned path and one violation at line 1. Removing only the new glob from a temporary copy of the rule changed that probe to scanned paths `[]` and findings `[]`; the added test therefore fails if the rule path is dropped. Its explicit inventory assertion fails if `CORE_CRATES` drops the owner, and the missing-path case fails if the rule omits that path while the inventory retains it.

## Remaining boundary

- No Rust source behavior changed. The current execution-contract source has no finding under this rule.
- Combined `just ci`, clean-HEAD qualification, hosted CI, consumer compile, and product runtime are not claimed here. The root integration lane owns the single combined `just ci` after all parallel patches are assembled.
