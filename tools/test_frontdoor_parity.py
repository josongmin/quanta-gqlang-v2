#!/usr/bin/env python3
"""Black-box parity gate for the two facade-only shipped front doors."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "target" / "debug"
CLI = TARGET / "quanta-gqlang-v2-cli"
STDIO = TARGET / "quanta-gqlang-v2-diagnostics-stdio"


def run(command: list[str], *, stdin: bytes | None = None) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        command,
        cwd=ROOT,
        input=stdin,
        capture_output=True,
        check=False,
    )


def parse_cli(source: str) -> dict[str, object]:
    result = run([str(CLI), "query", source])
    payload = result.stdout if result.stdout else result.stderr
    fields: dict[str, str] = {}
    for raw_line in payload.decode("utf-8").splitlines():
        if "=" in raw_line:
            key, value = raw_line.split("=", 1)
            fields[key] = value
    count = int(fields["analysis.diagnostic_count"])
    diagnostics = []
    for index in range(count):
        prefix = f"diagnostic[{index}]"
        start, end = fields[f"{prefix}.span"].split("..", 1)
        diagnostics.append(
            {
                "stage": fields[f"{prefix}.stage"],
                "code": fields[f"{prefix}.code"],
                "severity": fields[f"{prefix}.severity"],
                "message": fields[f"{prefix}.message"],
                "source_id": int(fields[f"{prefix}.source_id"]),
                "span_start": int(start),
                "span_end": int(end),
            }
        )
    return {
        "process_status": result.returncode,
        "analysis_status": fields["analysis.status"],
        "diagnostics": diagnostics,
    }


def parse_stdio(source: str) -> dict[str, object]:
    request = {
        "id": "parity",
        "method": "v3_diagnostics",
        "source_id": 0,
        "source": source,
    }
    result = run(
        [str(STDIO)],
        stdin=(json.dumps(request, separators=(",", ":")) + "\n").encode("utf-8"),
    )
    lines = result.stdout.splitlines()
    if result.returncode != 0 or len(lines) != 1:
        raise RuntimeError(
            f"diagnostics stdio transport failed: status={result.returncode} "
            f"stdout={result.stdout!r} stderr={result.stderr!r}"
        )
    response = json.loads(lines[0])
    if response.get("id") != "parity" or response.get("ok") is not True:
        raise RuntimeError(f"diagnostics stdio rejected parity request: {response!r}")
    return {
        "analysis_status": response["analysis_status"],
        "diagnostics": response["diagnostics"],
    }


def main() -> int:
    build = run(
        [
            "cargo",
            "build",
            "--quiet",
            "-p",
            "quanta-gqlang-v2-cli",
            "-p",
            "quanta-gqlang-v2-diagnostics-stdio",
            "--bins",
        ]
    )
    if build.returncode != 0:
        sys.stderr.buffer.write(build.stderr)
        return build.returncode

    cases = {
        "incomplete": "show capabilities",
        "error": "totally not a qglang root",
    }
    failures: list[str] = []
    for name, source in cases.items():
        cli = parse_cli(source)
        stdio = parse_stdio(source)
        cli_status = cli.pop("process_status")
        if cli_status == 0:
            failures.append(f"{name}: CLI must exit non-zero for non-complete analysis")
        if cli != stdio:
            failures.append(f"{name}: CLI/stdio semantic payload drift: cli={cli!r} stdio={stdio!r}")

    if failures:
        for failure in failures:
            print(f"front-door parity failed: {failure}", file=sys.stderr)
        return 1
    print(f"front-door parity passed: cases={len(cases)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
