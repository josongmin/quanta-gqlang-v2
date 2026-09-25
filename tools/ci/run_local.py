#!/usr/bin/env python3
"""Run local CI groups against one immutable source snapshot and run identity."""

from __future__ import annotations

import hashlib
import os
import secrets
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.environment import sanitized_environment, valid_run_id  # noqa: E402

GROUPS = ("doctor", "external-consumer-preflight", "static", "test", "policy")
TIMEOUTS = {
    "doctor": 120,
    "external-consumer-preflight": 180,
    "static": 900,
    "test": 900,
    "policy": 900,
}
ROOT = Path(__file__).resolve().parents[2]


class LocalCiError(RuntimeError):
    pass


def source_snapshot() -> str:
    """Hash every tracked or non-ignored untracked path without following links."""
    try:
        inventory = run_bounded(
            ("git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"),
            cwd=ROOT,
            timeout_s=30,
            capture_output=True,
        )
    except (OSError, ProcessTimeoutError) as error:
        raise LocalCiError("cannot enumerate source: %s" % error) from error
    if inventory.returncode:
        raise LocalCiError("git source inventory failed (exit=%d)" % inventory.returncode)
    names = (inventory.stdout or b"").split(b"\0")
    digest = hashlib.sha256()
    for raw_name in names:
        if not raw_name:
            continue
        name = os.fsdecode(raw_name)
        path = ROOT / name
        digest.update(raw_name)
        digest.update(b"\0")
        try:
            metadata = path.lstat()
        except FileNotFoundError:
            # A tracked deletion is valid dirty-checkout input. Keep it in the
            # snapshot as an explicit absence rather than refusing to test it.
            digest.update(b"MISSING\0")
            continue
        except OSError as error:
            raise LocalCiError("cannot inspect source path %s: %s" % (name, error)) from error
        try:
            digest.update(str(metadata.st_mode).encode("ascii"))
            digest.update(b"\0")
            if path.is_symlink():
                digest.update(os.fsencode(os.readlink(path)))
            elif path.is_file():
                with path.open("rb") as handle:
                    for block in iter(lambda: handle.read(1024 * 1024), b""):
                        digest.update(block)
            else:
                digest.update(b"NON_FILE")
        except OSError as error:
            raise LocalCiError("cannot hash source path %s: %s" % (name, error)) from error
        digest.update(b"\0")
    return digest.hexdigest()


def main() -> int:
    run_id = os.environ.get("QGLANG_QUALIFICATION_RUN_ID") or secrets.token_hex(16)
    if not valid_run_id(run_id):
        print("local CI: invalid qualification run identity", file=sys.stderr)
        return 2
    try:
        env = sanitized_environment()
        env["RUSTUP_TOOLCHAIN"] = read_lock()["rust"]
    except (DoctorError, OSError) as error:
        print("local CI: cannot load locked environment: %s" % error, file=sys.stderr)
        return 2
    env["QGLANG_CI_RUN_ID"] = run_id
    env["QGLANG_QUALIFICATION_RUN_ID"] = run_id
    try:
        frozen = source_snapshot()
        print("local CI: source snapshot=" + frozen, flush=True)
        for group in GROUPS:
            print("local CI: running " + group, flush=True)
            result = run_bounded(
                ("just", group), cwd=ROOT, timeout_s=TIMEOUTS[group], env=env
            )
            if result.returncode:
                print(
                    "local CI: %s failed (exit=%d)" % (group, result.returncode),
                    file=sys.stderr,
                )
                return result.returncode
            if source_snapshot() != frozen:
                print(
                    "local CI: source changed while running " + group,
                    file=sys.stderr,
                )
                return 2
    except (LocalCiError, ProcessTimeoutError, OSError) as error:
        print("local CI: " + str(error), file=sys.stderr)
        return 2
    print("local CI: all groups passed; source snapshot=" + frozen, flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
