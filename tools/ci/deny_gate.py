#!/usr/bin/env python3
"""Run cargo-deny against one stable, fresh advisory database revision."""

from __future__ import annotations

import hashlib
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.ci.advisory_db import AdvisoryError, state  # noqa: E402
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


def stable_identity(observed: dict) -> tuple:
    return tuple(observed.get(key) for key in ("path", "revision", "fetched_at_unix"))


def write_evidence(payload: dict) -> None:
    write_json_evidence(EVIDENCE, payload, owned_root=ROOT / "target", prefix="deny")


def main() -> int:
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    run_id = os.environ.get("QGLANG_QUALIFICATION_RUN_ID")
    try:
        before = state(cargo_home)
        result = run_bounded(
            COMMAND, cwd=ROOT, timeout_s=300, capture_output=True, text=True
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
        if stable_identity(before) != stable_identity(after):
            raise DenyGateError("advisory DB changed while cargo-deny was running")
        if result.returncode:
            return result.returncode
    except (
        AdvisoryError, DenyGateError, EvidencePathError, OSError, ProcessTimeoutError,
    ) as error:
        print("deny gate: " + str(error), file=sys.stderr)
        return 2
    print("deny gate: revision=%s exit=0" % after["revision"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
