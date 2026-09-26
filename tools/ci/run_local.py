#!/usr/bin/env python3
"""Run local CI groups against one immutable source snapshot and run identity."""

from __future__ import annotations

import hashlib
import fcntl
import os
import secrets
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.evidence import EvidencePathError, read_bytes_evidence, read_json_evidence  # noqa: E402
from tools.ci.environment import (  # noqa: E402
    EnvironmentSanitizationError, isolated_cargo_home, sanitized_environment, valid_run_id,
)

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
    """Hash source bytes and change times, including containing directories."""
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
    directories: set[Path] = set()
    for raw_name in names:
        if not raw_name:
            continue
        name = os.fsdecode(raw_name)
        path = ROOT / name
        if name == "target" and path.is_symlink():
            expected = ROOT.parent / "managed-build-cache"
            if ROOT.name != "managed-checkout" or path.resolve() != expected.resolve():
                raise LocalCiError("target symlink is not the managed qualification cache")
            # The qualifier owns and verifies this generated cache link.
            continue
        directories.update(path.parents)
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
            digest.update(str(metadata.st_ctime_ns).encode("ascii"))
            digest.update(b"\0")
            if path.is_symlink():
                digest.update(os.fsencode(os.readlink(path)))
                target = path.resolve(strict=True)
                if ROOT.resolve() not in target.parents or not target.is_file():
                    raise LocalCiError("tracked symlink leaves source tree or is not a file: " + name)
                digest.update(target.read_bytes())
            elif path.is_file():
                with path.open("rb") as handle:
                    for block in iter(lambda: handle.read(1024 * 1024), b""):
                        digest.update(block)
            else:
                digest.update(b"NON_FILE")
        except OSError as error:
            raise LocalCiError("cannot hash source path %s: %s" % (name, error)) from error
        digest.update(b"\0")
    for directory in sorted(path for path in directories if path == ROOT or ROOT in path.parents):
        digest.update(os.fsencode(str(directory.relative_to(ROOT))))
        digest.update(str(directory.stat().st_ctime_ns).encode("ascii"))
    return digest.hexdigest()


def validate_nextest_custody(run_id: str) -> None:
    evidence_dir = ROOT / "target/ci"
    evidence, _ = read_json_evidence(
        evidence_dir / "nextest-evidence.json", owned_root=ROOT / "target"
    )
    if not isinstance(evidence, dict) or evidence.get("qualification_run_id") != run_id or \
       type(evidence.get("exit")) is not int or evidence["exit"] != 0 or \
       type(evidence.get("selected")) is not int or evidence["selected"] <= 0 or \
       type(evidence.get("executed")) is not int or \
       evidence["selected"] != evidence["executed"]:
        raise LocalCiError("current-run Nextest evidence is missing or inconsistent")
    for name, field in (
        ("nextest-list.json", "inventory_sha256"),
        ("nextest-events.jsonl", "events_sha256"),
    ):
        raw = read_bytes_evidence(evidence_dir / name, owned_root=ROOT / "target")
        if hashlib.sha256(raw).hexdigest() != evidence.get(field):
            raise LocalCiError("current-run Nextest raw stream differs from evidence: " + name)


def _run_locked() -> int:
    run_id = os.environ.get("QGLANG_QUALIFICATION_RUN_ID") or secrets.token_hex(16)
    if not valid_run_id(run_id):
        print("local CI: invalid qualification run identity", file=sys.stderr)
        return 2
    try:
        env = sanitized_environment()
        env["RUSTUP_TOOLCHAIN"] = read_lock()["rust"]
        cargo_source = Path(env.get("CARGO_HOME", str(Path.home() / ".cargo")))
        env["CARGO_HOME"] = str(isolated_cargo_home(ROOT, cargo_source))
    except (DoctorError, EnvironmentSanitizationError, OSError) as error:
        print("local CI: cannot load locked environment: %s" % error, file=sys.stderr)
        return 2
    env["QGLANG_CI_RUN_ID"] = run_id
    env["QGLANG_QUALIFICATION_RUN_ID"] = run_id
    try:
        (ROOT / "target").mkdir(exist_ok=True)
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
        if "test" in GROUPS:
            validate_nextest_custody(run_id)
    except (EvidencePathError, LocalCiError, ProcessTimeoutError, OSError) as error:
        print("local CI: " + str(error), file=sys.stderr)
        return 2
    print("local CI: all groups passed; source snapshot=" + frozen, flush=True)
    return 0


def main() -> int:
    ci_root = ROOT / "target/ci"
    if ci_root.is_symlink():
        print("local CI: output directory is redirected", file=sys.stderr)
        return 2
    ci_root.mkdir(parents=True, exist_ok=True)
    lock_path = ci_root / ".run-local.lock"
    if lock_path.is_symlink():
        print("local CI: lock path is redirected", file=sys.stderr)
        return 2
    with lock_path.open("a+") as handle:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("local CI: another run owns this checkout", file=sys.stderr)
            return 2
        return _run_locked()


if __name__ == "__main__":
    raise SystemExit(main())
