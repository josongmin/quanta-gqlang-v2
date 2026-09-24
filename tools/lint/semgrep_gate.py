#!/usr/bin/env python3
"""Prove one Semgrep rule is non-vacuous and clean in one JSON scan."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".semgrep/qglang-frontend.yml"
RULE = "qglang-v2-no-string-typed-error-results-in-core"
EXPECTED_PATH = "tools/lint/tests/fixtures/semgrep/%s.rs" % RULE
CORE_CRATES = (
    "algebra", "binder", "hir", "logic", "modules", "pipeline", "planning",
    "sema", "surface", "types",
)


class SemgrepGateError(RuntimeError):
    pass


def expected_paths() -> set[str]:
    paths = {
        str(path.relative_to(ROOT))
        for owner in CORE_CRATES
        for path in (ROOT / ("crates/quanta-gqlang-v2-" + owner) / "src").rglob("*.rs")
    }
    paths.discard("crates/quanta-gqlang-v2-sema/src/test_support.rs")
    paths.add(EXPECTED_PATH)
    return paths


def validate(document: object, expected_scanned: set[str]) -> None:
    if not isinstance(document, dict):
        raise SemgrepGateError("Semgrep JSON root is not an object")
    errors = document.get("errors")
    results = document.get("results")
    if not isinstance(errors, list) or not isinstance(results, list):
        raise SemgrepGateError("Semgrep JSON lacks result/error arrays")
    paths = document.get("paths")
    scanned = paths.get("scanned") if isinstance(paths, dict) else None
    if not isinstance(scanned, list) or not all(isinstance(path, str) for path in scanned):
        raise SemgrepGateError("Semgrep JSON lacks scanned-path evidence")
    observed_scanned = set(scanned)
    if observed_scanned != expected_scanned:
        raise SemgrepGateError(
            "Semgrep target inventory drifted: missing=%d extra=%d"
            % (len(expected_scanned - observed_scanned), len(observed_scanned - expected_scanned))
        )
    if errors:
        raise SemgrepGateError("Semgrep reported %d engine errors" % len(errors))
    if len(results) != 1 or not isinstance(results[0], dict):
        raise SemgrepGateError(
            "expected exactly one positive-fixture finding, observed %d" % len(results)
        )
    finding = results[0]
    check_id, path = finding.get("check_id"), finding.get("path")
    if not isinstance(check_id, str) or not check_id.endswith(RULE):
        raise SemgrepGateError("unexpected Semgrep rule identity")
    if path != EXPECTED_PATH:
        raise SemgrepGateError("unexpected Semgrep finding path: %r" % path)


def main() -> int:
    executable = ["semgrep"] if shutil.which("semgrep") else ["python3", "-m", "semgrep"]
    command = executable + [
        "--config", str(CONFIG),
        "--json",
        "--quiet",
        "--metrics=off",
        "--disable-version-check",
        "crates",
        EXPECTED_PATH,
    ]
    try:
        result = run_bounded(
            command, cwd=ROOT, timeout_s=300, capture_output=True, text=True
        )
        if result.returncode:
            raise SemgrepGateError(
                "Semgrep exited %d: %s" % (result.returncode, (result.stderr or "").strip())
            )
        validate(json.loads(result.stdout or ""), expected_paths())
    except (OSError, ValueError, ProcessTimeoutError, SemgrepGateError) as error:
        print("semgrep gate: " + str(error), file=sys.stderr)
        return 2
    print("semgrep gate passed: production=0 positive_fixture=1 rule=" + RULE)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
