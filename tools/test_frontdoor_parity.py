#!/usr/bin/env python3
"""Black-box parity gate for the two facade-only shipped front doors."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.environment import sanitized_environment  # noqa: E402


ROOT = Path(__file__).resolve().parents[1]
CLI_BINARY = "quanta-gqlang-v2-cli"
STDIO_BINARY = "quanta-gqlang-v2-diagnostics-stdio"
BUILD_TIMEOUT_S = 600
BINARY_TIMEOUT_S = 20


def run(
    command: list[str], *, env: dict[str, str], stdin: bytes | None = None,
    timeout_s: int = BINARY_TIMEOUT_S,
):
    return run_bounded(
        command,
        cwd=ROOT,
        input_data=stdin,
        capture_output=True,
        timeout_s=timeout_s,
        env=env,
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
            name = target["name"]
            path = Path(executable)
            if name in binaries:
                raise ValueError(f"duplicate Cargo binary artifact: {name}")
            if not path.is_absolute():
                raise ValueError(f"relative Cargo binary artifact: {name}: {executable}")
            binaries[name] = path
    return binaries


def usable_binary(path: Path) -> bool:
    return path.is_file() and not path.is_symlink()


def decode_line_value(value: str) -> str:
    """Independently decode the CLI's one-line value contract."""
    decoded: list[str] = []
    index = 0
    while index < len(value):
        character = value[index]
        if character != "\\":
            codepoint = ord(character)
            if codepoint < 32 or 127 <= codepoint <= 159 or codepoint in (0x2028, 0x2029):
                raise ValueError("raw control character in CLI line value")
            decoded.append(character)
            index += 1
            continue
        index += 1
        if index >= len(value):
            raise ValueError("trailing backslash in CLI line value")
        escape = value[index]
        if escape in ("\\", "n", "r", "t"):
            decoded.append({"\\": "\\", "n": "\n", "r": "\r", "t": "\t"}[escape])
            index += 1
            continue
        if escape == "x":
            digits = value[index + 1 : index + 3]
            if len(digits) != 2:
                raise ValueError("invalid hexadecimal CLI line-value escape")
            index += 3
        elif escape == "u" and value[index + 1 : index + 2] == "{":
            close = value.find("}", index + 2)
            if close == -1:
                raise ValueError("unterminated Unicode escape in CLI line value")
            digits = value[index + 2 : close]
            index = close + 1
        else:
            raise ValueError(f"unknown CLI line-value escape \\{escape}")
        if not digits or not all(digit in "0123456789abcdefABCDEF" for digit in digits):
            raise ValueError("invalid hexadecimal CLI line-value escape")
        codepoint = int(digits, 16)
        if codepoint > 0x10FFFF or 0xD800 <= codepoint <= 0xDFFF:
            raise ValueError("invalid Unicode scalar in CLI line value")
        decoded.append(chr(codepoint))
    return "".join(decoded)


def physical_lines(payload: bytes) -> list[str]:
    if not payload.endswith(b"\n"):
        raise ValueError("front-door output is not LF-terminated")
    lines = payload.decode("utf-8").split("\n")
    if lines.pop() != "" or not lines or any(not line or "\r" in line for line in lines):
        raise ValueError("front-door output has an empty or nonphysical line")
    return lines


def nonnegative_int(value: str, key: str) -> int:
    if not value.isascii() or not value.isdecimal():
        raise ValueError(f"invalid nonnegative integer in {key}: {value!r}")
    return int(value)


def parse_cli(executable: Path, source: str, env: dict[str, str]) -> dict[str, object]:
    result = run([str(executable), "query", source], env=env)
    if bool(result.stdout) == bool(result.stderr):
        raise RuntimeError(
            f"CLI must use exactly one output channel: stdout={result.stdout!r} "
            f"stderr={result.stderr!r}"
        )
    is_failure = bool(result.stderr)
    payload = result.stderr if is_failure else result.stdout
    fields: dict[str, str] = {}
    for raw_line in physical_lines(payload):
        if "=" not in raw_line:
            raise ValueError(f"non-field CLI output row: {raw_line!r}")
        key, value = raw_line.split("=", 1)
        if not key or key in fields:
            raise ValueError(f"empty or duplicate CLI output key: {key!r}")
        fields[key] = value
    expected = {"analysis.status", "analysis.diagnostic_count"}
    if is_failure:
        expected.add("cli.error.class")
        if fields.get("cli.error.class") != "command" or fields.get("analysis.status") != "error":
            raise ValueError("unexpected CLI failure envelope")
    else:
        expected.update(("analysis.source_id", "analysis.item_count", "analysis.unresolved_count"))
        nonnegative_int(fields["analysis.source_id"], "analysis.source_id")
        nonnegative_int(fields["analysis.unresolved_count"], "analysis.unresolved_count")
        item_count = nonnegative_int(fields["analysis.item_count"], "analysis.item_count")
        for index in range(item_count):
            prefix = f"item[{index}]"
            status_key = f"{prefix}.status"
            expected.add(status_key)
            status = fields[status_key]
            if status == "blocked":
                count_key = f"{prefix}.diagnostic_count"
                expected.add(count_key)
                nonnegative_int(fields[count_key], count_key)
            elif status == "not_applicable":
                expected.add(f"{prefix}.no_plan_reason")
            elif status == "complete":
                expected.add(f"{prefix}.kind")
                plan_keys = {
                    f"{prefix}.plan.{name}"
                    for name in (
                        "authority_lane", "determinism", "evidence", "effect",
                        "result_kind", "capabilities",
                    )
                }
                plan_keys.update(
                    f"{prefix}.cost.{name}"
                    for name in (
                        "estimated_cardinality_class", "recursion_risk_class",
                        "witness_explosion_risk_class", "ordering_contract",
                    )
                )
                if set(fields) & plan_keys:
                    expected.update(plan_keys)
            else:
                raise ValueError(f"unknown CLI item status: {status!r}")
    count = nonnegative_int(fields["analysis.diagnostic_count"], "analysis.diagnostic_count")
    diagnostics = []
    for index in range(count):
        prefix = f"diagnostic[{index}]"
        expected.update(f"{prefix}.{name}" for name in (
            "stage", "code", "severity", "message", "source_id", "span",
        ))
        start, end = fields[f"{prefix}.span"].split("..", 1)
        diagnostics.append(
            {
                "stage": fields[f"{prefix}.stage"],
                "code": fields[f"{prefix}.code"],
                "severity": fields[f"{prefix}.severity"],
                "message": decode_line_value(fields[f"{prefix}.message"]),
                "source_id": nonnegative_int(fields[f"{prefix}.source_id"], f"{prefix}.source_id"),
                "span_start": nonnegative_int(start, f"{prefix}.span.start"),
                "span_end": nonnegative_int(end, f"{prefix}.span.end"),
            }
        )
    if set(fields) != expected:
        raise ValueError(
            f"CLI output schema drift: missing={sorted(expected - fields.keys())} "
            f"extra={sorted(fields.keys() - expected)}"
        )
    return {
        "process_status": result.returncode,
        "analysis_status": fields["analysis.status"],
        "diagnostics": diagnostics,
    }


def parse_stdio(
    executable: Path, source: str, env: dict[str, str],
) -> dict[str, object]:
    request = {
        "id": "parity",
        "method": "v3_diagnostics",
        "source_id": 0,
        "source": source,
    }
    result = run(
        [str(executable)],
        env=env,
        stdin=(json.dumps(request, separators=(",", ":")) + "\n").encode("utf-8"),
    )
    if result.returncode != 0 or result.stderr:
        raise RuntimeError(
            f"diagnostics stdio transport failed: status={result.returncode} "
            f"stdout={result.stdout!r} stderr={result.stderr!r}"
        )
    lines = physical_lines(result.stdout)
    if len(lines) != 1:
        raise RuntimeError(f"diagnostics stdio emitted {len(lines)} frames")

    def unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate diagnostics stdio JSON key: {key}")
            result[key] = value
        return result

    response = json.loads(lines[0], object_pairs_hook=unique_object)
    if (
        not isinstance(response, dict)
        or set(response) != {"id", "ok", "method", "analysis_status", "diagnostics"}
        or response.get("id") != "parity"
        or response.get("ok") is not True
        or response.get("method") != "v3_diagnostics"
    ):
        raise RuntimeError(f"diagnostics stdio rejected parity request: {response!r}")
    if not isinstance(response["diagnostics"], list):
        raise ValueError("diagnostics stdio diagnostics is not a list")
    expected_diagnostic = {
        "stage", "code", "severity", "message", "source_id", "span_start", "span_end"
    }
    for diagnostic in response["diagnostics"]:
        if (
            not isinstance(diagnostic, dict)
            or set(diagnostic) != expected_diagnostic
            or any(not isinstance(diagnostic[key], str) for key in (
                "stage", "code", "severity", "message"
            ))
            or any(type(diagnostic[key]) is not int or diagnostic[key] < 0 for key in (
                "source_id", "span_start", "span_end"
            ))
        ):
            raise ValueError(f"invalid diagnostics stdio diagnostic: {diagnostic!r}")
    return {
        "analysis_status": response["analysis_status"],
        "diagnostics": response["diagnostics"],
    }


def main() -> int:
    try:
        env = sanitized_environment()
        env["RUSTUP_TOOLCHAIN"] = read_lock()["rust"]
    except (DoctorError, OSError) as error:
        print(f"front-door parity cannot load locked environment: {error}", file=sys.stderr)
        return 2
    env["CARGO_TARGET_DIR"] = str(ROOT / "target/ci-msrv")
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
            "--locked",
            "--offline",
            "--message-format=json-render-diagnostics",
        ],
        env=env,
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
    if not usable_binary(cli_binary) or not usable_binary(stdio_binary):
        print(
            "front-door parity Cargo artifact path is absent or redirected: "
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
        "unicode-line-separator-message": ("let x = 1;\u2028", "error", 1),
        "esc-message": ("let x = 1;\u001b", "error", 1),
    }
    failures: list[str] = []
    for name, (source, expected_status, minimum_diagnostics) in cases.items():
        cli = parse_cli(cli_binary, source, env)
        stdio = parse_stdio(stdio_binary, source, env)
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
