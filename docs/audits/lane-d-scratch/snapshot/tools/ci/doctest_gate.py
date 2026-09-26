#!/usr/bin/env python3
"""Run all workspace doctests and reject a vacuous successful run."""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.environment import sanitized_environment  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
RESULT = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;(?:.*)$"
)


class DoctestEvidenceError(RuntimeError):
    pass


def count_executed(output: str) -> int:
    summaries = [line for line in output.splitlines() if line.startswith("test result:")]
    if not summaries:
        raise DoctestEvidenceError("missing doctest result summaries")
    executed = 0
    for line in summaries:
        match = RESULT.fullmatch(line)
        if match is None:
            raise DoctestEvidenceError("invalid doctest result summary")
        status, passed, failed, ignored = match.groups()
        if status != "ok" or int(failed) != 0 or int(ignored) != 0:
            raise DoctestEvidenceError(
                "non-passing doctest summary: status=%s failed=%s ignored=%s"
                % (status, failed, ignored)
            )
        executed += int(passed)
    return executed


def main() -> int:
    env = sanitized_environment()
    env["CARGO_TARGET_DIR"] = str(ROOT / "target/ci-msrv")
    env["RUSTDOCFLAGS"] = "-D warnings"
    try:
        env["RUSTUP_TOOLCHAIN"] = read_lock()["rust"]
        result = run_bounded(
            ("cargo", "test", "--workspace", "--doc", "--locked", "--offline"),
            cwd=ROOT, env=env, capture_output=True, text=True, timeout_s=600,
        )
    except (DoctorError, ProcessTimeoutError, OSError) as error:
        print("doctest gate: " + str(error), file=sys.stderr)
        return 2
    sys.stdout.write(result.stdout or "")
    sys.stderr.write(result.stderr or "")
    if result.returncode:
        return result.returncode
    try:
        executed = count_executed((result.stdout or "") + "\n" + (result.stderr or ""))
    except DoctestEvidenceError as error:
        print("doctest gate: " + str(error), file=sys.stderr)
        return 2
    if executed == 0:
        print("doctest gate: no doctests executed", file=sys.stderr)
        return 2
    print("doctest gate: executed=%d ignored=0" % executed)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
