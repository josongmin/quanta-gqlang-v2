#!/usr/bin/env bash
# Stable public entry point; the Python driver owns locking, checkout custody,
# cache reuse, source verification, and the exact-SHA receipt.
set -euo pipefail
exec python3 -I -S "$(dirname "$0")/../ci/python_gate.py" tools/qualification/qualify_local.py
