#!/usr/bin/env python3
"""Run workspace Clippy with the independently locked current toolchain."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.environment import sanitized_environment  # noqa: E402
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
COMMAND = (
    "cargo", "clippy", "--workspace", "--all-targets", "--locked", "--offline",
    "--", "-D", "clippy::unused_result_ok", "-D", "warnings",
)


def main() -> int:
    try:
        channel = read_lock()["clippy"]
        env = sanitized_environment()
        env["RUSTUP_TOOLCHAIN"] = channel
        env["CARGO_TARGET_DIR"] = str(ROOT / "target/ci-clippy-current")
        result = run_bounded(COMMAND, cwd=ROOT, env=env, timeout_s=900)
    except (DoctorError, OSError, ProcessTimeoutError) as error:
        print("clippy gate: " + str(error), file=sys.stderr)
        return 2
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
