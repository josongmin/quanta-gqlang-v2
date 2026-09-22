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
