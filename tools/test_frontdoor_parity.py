#!/usr/bin/env python3
"""Black-box parity gate for the two facade-only shipped front doors."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402


ROOT = Path(__file__).resolve().parents[1]
CLI_BINARY = "quanta-gqlang-v2-cli"
STDIO_BINARY = "quanta-gqlang-v2-diagnostics-stdio"
BUILD_TIMEOUT_S = 600
BINARY_TIMEOUT_S = 20


def run(command: list[str], *, stdin: bytes | None = None, timeout_s: int = BINARY_TIMEOUT_S):
    return run_bounded(
        command,
        cwd=ROOT,
        input_data=stdin,
        capture_output=True,
        timeout_s=timeout_s,
    )


def built_binaries(output: bytes) -> dict[str, Path]:
    """Return exact executable paths emitted by this Cargo build."""
    binaries: dict[str, Path] = {}
    for raw_line in output.splitlines():
        event = json.loads(raw_line)
        if not isinstance(event, dict):
            continue
        target = event.get("target", {})
        if not isinstance(target, dict):
            continue
        executable = event.get("executable")
        if (
            event.get("reason") == "compiler-artifact"
            and "bin" in target.get("kind", [])
            and isinstance(target.get("name"), str)
            and isinstance(executable, str)
        ):
            binaries[target["name"]] = Path(executable)
    return binaries


def parse_cli(executable: Path, source: str) -> dict[str, object]:
    result = run([str(executable), "query", source])
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


def parse_stdio(executable: Path, source: str) -> dict[str, object]:
    request = {
        "id": "parity",
        "method": "v3_diagnostics",
        "source_id": 0,
        "source": source,
    }
    result = run(
        [str(executable)],
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
            "--message-format=json-render-diagnostics",
        ],
        timeout_s=BUILD_TIMEOUT_S,
    )
    if build.returncode != 0:
        sys.stderr.buffer.write(build.stdout)
        sys.stderr.buffer.write(build.stderr)
        return build.returncode

    try:
        binaries = built_binaries(build.stdout)
    except (KeyError, TypeError, ValueError) as error:
        print(f"front-door parity could not parse Cargo artifacts: {error}", file=sys.stderr)
        return 1
    missing = sorted({CLI_BINARY, STDIO_BINARY} - binaries.keys())
    if missing:
        print(f"front-door parity build omitted binaries: {missing}", file=sys.stderr)
        return 1
    cli_binary = binaries[CLI_BINARY]
    stdio_binary = binaries[STDIO_BINARY]
    if not cli_binary.is_file() or not stdio_binary.is_file():
        print(
            "front-door parity Cargo artifact path is absent: "
            f"cli={cli_binary} stdio={stdio_binary}",
            file=sys.stderr,
        )
        return 1

    deep = "let x = " + ("(" * 70) + "1" + (")" * 70)
    cases = {
        # name: (source, expected shared status, minimum diagnostic count)
        "multi-diagnostic-incomplete": ("show capabilities", "incomplete", 3),
        "invalid-root": ("totally not a qglang root", "error", 1),
        "unicode-byte-span": ("let 이름 = 1", "error", 1),
        "multi-item": ("let x = 1; show capabilities", "incomplete", 3),
        "mixed-valid-invalid": ("let x = 1; totally not a qglang root", "error", 1),
        "expression-depth-resource": (deep, "error", 1),
        "composition": (
            "let suspects = find taint a.b -> c.d | depth=2 "
            "find taint api.entry -> db.write | depth=3 with suspects",
            "incomplete",
            3,
        ),
        "rule": ("rule adult : x if x = 1", "incomplete", 3),
    }
    failures: list[str] = []
    for name, (source, expected_status, minimum_diagnostics) in cases.items():
        cli = parse_cli(cli_binary, source)
        stdio = parse_stdio(stdio_binary, source)
        cli_status = cli.pop("process_status")
        if cli_status != 1:
            failures.append(
                f"{name}: CLI must exit 1 for non-complete analysis, got {cli_status}"
            )
        if cli["analysis_status"] != expected_status:
            failures.append(
                f"{name}: expected shared status {expected_status!r}, got {cli['analysis_status']!r}"
            )
        if len(cli["diagnostics"]) < minimum_diagnostics:
            failures.append(
                f"{name}: expected at least {minimum_diagnostics} diagnostics, "
                f"got {len(cli['diagnostics'])}"
            )
        if cli != stdio:
            failures.append(f"{name}: CLI/stdio semantic payload drift: cli={cli!r} stdio={stdio!r}")

    if failures:
        for failure in failures:
            print(f"front-door parity failed: {failure}", file=sys.stderr)
        return 1
    print(f"front-door parity passed: cases={len(cases)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ProcessTimeoutError as error:
        print(f"front-door parity infrastructure failure: {error}", file=sys.stderr)
        raise SystemExit(2) from error
