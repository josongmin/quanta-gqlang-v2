#!/usr/bin/env python3
"""Run cargo-deny against one stable, fresh advisory database revision."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.ci.advisory_db import AdvisoryError, advisory_identity, state  # noqa: E402
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.environment import (  # noqa: E402
    EnvironmentSanitizationError, sanitized_environment,
)
from tools.ci.evidence import EvidencePathError, write_json_evidence  # noqa: E402
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "target/ci/deny-evidence.json"
COMMAND = (
    "cargo", "deny", "--frozen", "check", "--disable-fetch",
    "bans", "licenses", "sources", "advisories",
)


class DenyGateError(RuntimeError):
    pass


def write_evidence(payload: dict) -> None:
    write_json_evidence(EVIDENCE, payload, owned_root=ROOT / "target", prefix="deny")


def main() -> int:
    try:
        env = sanitized_environment(preserve_qualification_run_id=True)
        env["RUSTUP_TOOLCHAIN"] = read_lock()["rust"]
        cargo_home = Path(env.get("CARGO_HOME", str(Path.home() / ".cargo")))
        run_id = env.get("QGLANG_QUALIFICATION_RUN_ID")
        before = state(cargo_home)
        result = run_bounded(
            COMMAND, cwd=ROOT, timeout_s=300, capture_output=True, text=True, env=env
        )
        sys.stdout.write(result.stdout or "")
        sys.stderr.write(result.stderr or "")
        after = state(cargo_home)
        output = (result.stdout or "") + "\n" + (result.stderr or "")
        evidence = {
            "schema": 1,
            "qualification_run_id": run_id,
            "command": list(COMMAND),
            "exit": result.returncode,
            "before": before,
            "after": after,
            "output_sha256": hashlib.sha256(output.encode()).hexdigest(),
        }
        write_evidence(evidence)
        if advisory_identity(before) != advisory_identity(after):
            raise DenyGateError("advisory DB changed while cargo-deny was running")
        if result.returncode:
            return result.returncode
    except (
        AdvisoryError, DenyGateError, DoctorError, EnvironmentSanitizationError,
        EvidencePathError, OSError, ProcessTimeoutError,
    ) as error:
        print("deny gate: " + str(error), file=sys.stderr)
        return 2
    print("deny gate: revision=%s exit=0" % after["revision"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
