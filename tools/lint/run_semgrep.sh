#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
config="${QGLANG_SEMGREP_CONFIG:-$repo_root/.semgrep/qglang-frontend.yml}"
args=(--config "$config" --error --quiet --metrics=off --disable-version-check --exclude "tools/lint/tests/fixtures/semgrep/**" .)

cd "$repo_root"
if command -v semgrep >/dev/null 2>&1; then
    semgrep "${args[@]}"
else
    python3 -m semgrep "${args[@]}"
fi
printf 'semgrep gate passed: rule=qglang-v2-no-string-typed-error-results-in-core\n'
