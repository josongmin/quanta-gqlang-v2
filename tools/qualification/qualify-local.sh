#!/usr/bin/env bash
# Stable public entry point; the Python driver owns locking, checkout custody,
# cache reuse, source verification, and the exact-SHA receipt.
set -euo pipefail
exec python3 "$(dirname "$0")/qualify_local.py"
