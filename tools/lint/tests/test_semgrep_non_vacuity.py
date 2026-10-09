#!/usr/bin/env python3
"""Negative checks for the single-scan Semgrep evidence contract."""

from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.lint.semgrep_gate import (
    CONFIG, EXPECTED_LINES, EXPECTED_PATH, RULE, SemgrepGateError, expected_paths,
    string_error_public_lines, validate,
)
from tools.lint.semantic_honesty import parser
from tools.process_guard import run_bounded


EXECUTION_SOURCE = "crates/quanta-gqlang-v2-execution-contract/src/lib.rs"


def finding(path: str = EXPECTED_PATH, rule: str = RULE, line: int = 1) -> dict:
    return {"check_id": rule, "path": path, "start": {"line": line}}


def fixtures() -> list[dict]:
    return [finding(line=line) for line in sorted(EXPECTED_LINES)]


class SemgrepNonVacuityTest(unittest.TestCase):
    def test_ast_backstop_checks_outer_error_type_and_visibility(self) -> None:
        rust_parser = parser()
        source = (
            "pub async fn bad() -> Result<Vec<Result<u8, E>>, String> { todo!() }\n"
            "pub(super) fn also_bad() -> Result<(), String> { todo!() }\n"
            "pub fn safe() -> Result<Vec<Result<u8, String>>, E> { todo!() }\n"
            "fn private() -> Result<(), String> { todo!() }\n"
        )
        self.assertEqual(string_error_public_lines(source.encode(), rust_parser), {1, 2})

    def test_exact_positive_fixture_is_accepted(self) -> None:
        validate(
            {"errors": [], "results": fixtures(), "paths": {"scanned": [EXPECTED_PATH]}},
            {EXPECTED_PATH},
        )

    def test_zero_or_production_findings_are_rejected(self) -> None:
        with self.assertRaisesRegex(SemgrepGateError, "positive-fixture findings"):
            validate({"errors": [], "results": [], "paths": {"scanned": [EXPECTED_PATH]}},
                     {EXPECTED_PATH})
        with self.assertRaisesRegex(SemgrepGateError, "unexpected Semgrep finding path"):
            validate({"errors": [], "results": fixtures()[:-1] + [finding("crates/core/src/lib.rs", line=9)],
                      "paths": {"scanned": [EXPECTED_PATH]}}, {EXPECTED_PATH})

    def test_engine_errors_and_rule_drift_are_rejected(self) -> None:
        with self.assertRaisesRegex(SemgrepGateError, "engine errors"):
            validate({"errors": [{"message": "parse failure"}], "results": fixtures(),
                      "paths": {"scanned": [EXPECTED_PATH]}}, {EXPECTED_PATH})
        with self.assertRaisesRegex(SemgrepGateError, "rule identity"):
            validate({"errors": [], "results": fixtures()[:-1] + [finding(rule="other-rule", line=9)],
                      "paths": {"scanned": [EXPECTED_PATH]}}, {EXPECTED_PATH})

    def test_target_inventory_drift_is_rejected(self) -> None:
        with self.assertRaisesRegex(SemgrepGateError, "target inventory drifted"):
            validate({"errors": [], "results": fixtures(), "paths": {"scanned": []}},
                     {EXPECTED_PATH})

    def test_execution_contract_is_required_in_scanned_inventory(self) -> None:
        paths = expected_paths()
        self.assertIn(EXECUTION_SOURCE, paths)
        with self.assertRaisesRegex(SemgrepGateError, "target inventory drifted"):
            validate(
                {"errors": [], "results": fixtures(), "paths": {"scanned": sorted(paths - {EXECUTION_SOURCE})}},
                paths,
            )

    def test_rule_catches_execution_contract_violation(self) -> None:
        executable = ["semgrep"] if shutil.which("semgrep") else ["python3", "-m", "semgrep"]
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / EXECUTION_SOURCE
            source.parent.mkdir(parents=True)
            source.write_text("pub fn bad() -> Result<(), String> { Ok(()) }\n")
            result = run_bounded(
                executable + [
                    "--config", str(CONFIG), "--json", "--quiet", "--metrics=off",
                    "--disable-version-check", EXECUTION_SOURCE,
                ],
                cwd=Path(directory), timeout_s=60, capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            document = json.loads(result.stdout)
            self.assertEqual(document.get("errors"), [])
            self.assertEqual(document.get("paths", {}).get("scanned"), [EXECUTION_SOURCE])
            self.assertEqual(
                [(item.get("path"), item.get("start", {}).get("line")) for item in document.get("results", [])],
                [(EXECUTION_SOURCE, 1)],
            )

if __name__ == "__main__":
    unittest.main()
