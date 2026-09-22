#!/usr/bin/env bash
# Pinned local qualification driver (invoked by `just qualify-local`).
#
# Steps:
#   1. Clean-tree check: refuse to qualify a dirty working tree.
#   2. Pin HEAD and run in an isolated detached worktree at that commit.
#   3. Toolchain record: print the pinned toolchain and active tool versions.
#   4. Run `just ci` (the single composition entry point over the
#      static/test/policy groups) while timing it; verify both trees afterwards.
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

# 2. Pin the source commit and tree, then materialize that source separately.
sha="$(git rev-parse HEAD)"
tree="$(git rev-parse HEAD^{tree})"
echo "qualify-local: HEAD sha=$sha"
echo "qualify-local: source tree=$tree"

work_dir="$(mktemp -d)"
checkout="$work_dir/checkout"
cleanup() {
    if [ -d "$checkout" ]; then
        git -C "$repo_root" worktree remove --force "$checkout" >&2 || true
    fi
    rmdir "$work_dir" 2>/dev/null || true
}
trap cleanup EXIT
git worktree add --detach "$checkout" "$sha" >&2

# 3. Toolchain record (stdout only; never added to receipt.json).
echo "qualify-local: pinned toolchain:"
cat rust-toolchain.toml
echo "qualify-local: active tool versions:"
rustc --version
cargo --version
command -v just >/dev/null 2>&1 || { echo "qualify-local: just is required but not installed" >&2; exit 1; }
just --version

# 4. Run the single composition entry point against the pinned source. Shared
# Cargo outputs are outside the checkout; they cannot change its source tree.
start="$(date +%s)"
set +e
(cd "$checkout" && CARGO_TARGET_DIR="$repo_root/target" just ci)
exit_code=$?
set -e
end="$(date +%s)"
duration_s=$((end - start))

if [ "$(git -C "$checkout" rev-parse HEAD)" != "$sha" ] || \
   [ "$(git -C "$checkout" rev-parse HEAD^{tree})" != "$tree" ] || \
   [ -n "$(git -C "$checkout" status --porcelain --untracked-files=all)" ]; then
    echo "qualify-local: pinned checkout changed during qualification" >&2
    exit_code=1
fi
if [ "$(git -C "$repo_root" rev-parse HEAD)" != "$sha" ] || \
   [ -n "$(git -C "$repo_root" status --porcelain)" ]; then
    echo "qualify-local: caller checkout changed during qualification" >&2
    exit_code=1
fi

# 5. Receipt with exactly the keys sha/tree/cmds/exit/duration_s.
receipt_dir="$repo_root/target/qualification/$sha"
mkdir -p "$receipt_dir"
python3 - "$receipt_dir/receipt.json" "$sha" "$tree" "$duration_s" "$exit_code" <<'PY'
import json
import os
import sys
import tempfile

receipt_path, sha, tree, duration_s, exit_code = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
receipt = {
    "sha": sha,
    "tree": tree,
    "cmds": ["just ci"],
    "exit": exit_code,
    "duration_s": duration_s,
}
fd, temporary_path = tempfile.mkstemp(dir=os.path.dirname(receipt_path), prefix=".receipt-")
try:
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        json.dump(receipt, handle, indent=2)
        handle.write("\n")
    os.replace(temporary_path, receipt_path)
finally:
    if os.path.exists(temporary_path):
        os.unlink(temporary_path)
PY

echo "qualify-local: receipt written to $receipt_dir/receipt.json (exit=$exit_code, duration_s=$duration_s)"
exit "$exit_code"
