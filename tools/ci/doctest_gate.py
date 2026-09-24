#!/usr/bin/env python3
"""Run all workspace doctests and reject a vacuous successful run."""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
RESULT = re.compile(r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", re.MULTILINE)


def count_executed(output: str) -> int:
    return sum(int(passed) for passed, failed, ignored in RESULT.findall(output))


def main() -> int:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(ROOT / "target/ci-msrv")
    env["RUSTDOCFLAGS"] = "-D warnings"
    try:
        result = run_bounded(
            ("cargo", "test", "--workspace", "--doc", "--locked", "--offline"),
            cwd=ROOT, env=env, capture_output=True, text=True, timeout_s=600,
        )
    except (ProcessTimeoutError, OSError) as error:
        print("doctest gate: " + str(error), file=sys.stderr)
        return 2
    sys.stdout.write(result.stdout or "")
    sys.stderr.write(result.stderr or "")
    if result.returncode:
        return result.returncode
    executed = count_executed((result.stdout or "") + (result.stderr or ""))
    if executed == 0:
        print("doctest gate: no doctests executed", file=sys.stderr)
        return 2
    print("doctest gate: executed=%d" % executed)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
