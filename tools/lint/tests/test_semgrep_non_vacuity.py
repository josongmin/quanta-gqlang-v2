#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "tools/lint/run_semgrep.sh"
CONFIG = ROOT / ".semgrep/qglang-frontend.yml"


class SemgrepNonVacuityTest(unittest.TestCase):
    def run_gate(self, config: Path) -> subprocess.CompletedProcess[str]:
        environment = os.environ.copy()
        environment["QGLANG_SEMGREP_CONFIG"] = str(config)
        environment["QGLANG_SEMGREP_SKIP_PRODUCTION"] = "1"
        return subprocess.run(
            [str(RUNNER)],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_every_configured_rule_has_one_positive_and_clean_control(self) -> None:
        result = self.run_gate(CONFIG)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("configured=7 executed=7", result.stdout)

    def test_runtime_truth_fixture_covers_the_complete_policy_vocabulary(self) -> None:
        fixture = (
            ROOT
            / "tools/lint/tests/fixtures/semgrep"
            / "qglang-v2-no-runtime-truth-tokens.rs"
        )
        expected = {
            "result_count",
            "row_count",
            "rows_returned",
            "match_count",
            "matches_found",
            "results_found",
            "num_results",
            "execution_time",
            "elapsed_ms",
            "executed_at",
            "ran_at",
            "provider_response",
            "provider_endpoint",
            "routed_to",
            "host_dto",
            "host_response",
            "backend_result",
            "edit_applied",
            "applied_edit",
            "workspace_edit_applied",
            "materialized_result",
            "result_materialized",
            "runtime_truth",
            "confirmed_true",
            "actual_result",
            "rows=",
            "verdict=",
        }
        actual = set(fixture.read_text(encoding="utf-8").splitlines())
        self.assertEqual(actual, expected)

    def test_missing_include_path_rejects_unexecuted_rule(self) -> None:
        original = CONFIG.read_text(encoding="utf-8")
        mutated = original.replace(
            "/crates/quanta-gqlang-v2-hir/src/**/*.rs",
            "/crates/quanta-gqlang-v2-missing/src/**/*.rs",
            1,
        )
        self.assertNotEqual(original, mutated)
        with tempfile.TemporaryDirectory(prefix="qglang-semgrep-config-") as temporary:
            config = Path(temporary) / "qglang-frontend.yml"
            config.write_text(mutated, encoding="utf-8")
            result = self.run_gate(config)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "qglang-v2-no-string-typed-error-results-in-core",
            result.stderr,
        )


if __name__ == "__main__":
    unittest.main()
