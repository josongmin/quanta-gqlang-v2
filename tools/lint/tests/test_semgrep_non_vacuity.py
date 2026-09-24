#!/usr/bin/env python3
"""Negative checks for the single-scan Semgrep evidence contract."""

from __future__ import annotations

import unittest

from tools.lint.semgrep_gate import EXPECTED_PATH, RULE, SemgrepGateError, validate


def finding(path: str = EXPECTED_PATH, rule: str = RULE) -> dict:
    return {"check_id": rule, "path": path}


class SemgrepNonVacuityTest(unittest.TestCase):
    def test_exact_positive_fixture_is_accepted(self) -> None:
        validate(
            {"errors": [], "results": [finding()], "paths": {"scanned": [EXPECTED_PATH]}},
            {EXPECTED_PATH},
        )

    def test_zero_or_production_findings_are_rejected(self) -> None:
        with self.assertRaisesRegex(SemgrepGateError, "exactly one"):
            validate({"errors": [], "results": [], "paths": {"scanned": [EXPECTED_PATH]}},
                     {EXPECTED_PATH})
        with self.assertRaisesRegex(SemgrepGateError, "unexpected Semgrep finding path"):
            validate({"errors": [], "results": [finding("crates/core/src/lib.rs")],
                      "paths": {"scanned": [EXPECTED_PATH]}}, {EXPECTED_PATH})

    def test_engine_errors_and_rule_drift_are_rejected(self) -> None:
        with self.assertRaisesRegex(SemgrepGateError, "engine errors"):
            validate({"errors": [{"message": "parse failure"}], "results": [finding()],
                      "paths": {"scanned": [EXPECTED_PATH]}}, {EXPECTED_PATH})
        with self.assertRaisesRegex(SemgrepGateError, "rule identity"):
            validate({"errors": [], "results": [finding(rule="other-rule")],
                      "paths": {"scanned": [EXPECTED_PATH]}}, {EXPECTED_PATH})

    def test_target_inventory_drift_is_rejected(self) -> None:
        with self.assertRaisesRegex(SemgrepGateError, "target inventory drifted"):
            validate({"errors": [], "results": [finding()], "paths": {"scanned": []}},
                     {EXPECTED_PATH})

if __name__ == "__main__":
    unittest.main()
