#!/usr/bin/env bash
# Pinned local qualification driver (invoked by `just qualify-local`).
#
# Steps:
#   1. Clean-tree check: refuse to qualify a dirty working tree.
#   2. HEAD SHA pin: the receipt directory is target/qualification/<sha>/.
#   3. Toolchain record: print the pinned toolchain and active tool versions.
#   4. Run `just ci` (the single composition entry point over the
#      static/test/policy groups) while timing it.
#   5. Write target/qualification/<sha>/receipt.json containing ONLY the
#      keys: sha, tree, cmds, exit, duration_s.
#
# The receipt lives under target/ (gitignored), so it stays uncommitted.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"

# 1. Clean-tree check.
if [ -n "$(git status --porcelain)" ]; then
    echo "qualify-local refused: working tree is dirty; commit or stash first:" >&2
    git status --porcelain >&2
    exit 1
fi

# 2. HEAD SHA pin.
sha="$(git rev-parse HEAD)"
echo "qualify-local: HEAD sha=$sha"

# 3. Toolchain record (stdout only; never added to receipt.json).
echo "qualify-local: pinned toolchain:"
cat rust-toolchain.toml
echo "qualify-local: active tool versions:"
rustc --version
cargo --version
command -v just >/dev/null 2>&1 || { echo "qualify-local: just is required but not installed" >&2; exit 1; }
just --version

# 4. Run the single composition entry point.
start="$(date +%s)"
set +e
just ci
exit_code=$?
set -e
end="$(date +%s)"
duration_s=$((end - start))

# 5. Receipt with exactly the keys sha/tree/cmds/exit/duration_s.
receipt_dir="target/qualification/$sha"
mkdir -p "$receipt_dir"
python3 - "$receipt_dir/receipt.json" "$sha" "$duration_s" "$exit_code" <<'PY'
import json
import sys

receipt_path, sha, duration_s, exit_code = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
receipt = {
    "sha": sha,
    "tree": "clean",
    "cmds": ["just ci"],
    "exit": exit_code,
    "duration_s": duration_s,
}
with open(receipt_path, "w", encoding="utf-8") as handle:
    json.dump(receipt, handle, indent=2)
    handle.write("\n")
PY

echo "qualify-local: receipt written to $receipt_dir/receipt.json (exit=$exit_code, duration_s=$duration_s)"
exit "$exit_code"
