#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

if command -v semgrep >/dev/null 2>&1; then
  exec semgrep --config .semgrep/qglang-frontend.yml --error
fi

if python3 -m semgrep --version >/dev/null 2>&1; then
  exec python3 -m semgrep --config .semgrep/qglang-frontend.yml --error
fi

echo "semgrep is required but not installed" >&2
exit 1
