#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CONFIG = ROOT / ".semgrep/qglang-frontend.yml"
FIXTURES = ROOT / "tools/lint/tests/fixtures/semgrep"
RULE = "qglang-v2-no-string-typed-error-results-in-core"


class SemgrepNonVacuityTest(unittest.TestCase):
    def semgrep(self) -> list[str]:
        if shutil.which("semgrep"):
            return ["semgrep"]
        return ["python3", "-m", "semgrep"]

    def scan(self, config: Path, root: Path) -> list[dict]:
        result = subprocess.run(
            self.semgrep()
            + [
                "--config",
                str(config),
                "--json",
                "--metrics=off",
                "--disable-version-check",
                ".",
            ],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)["results"]

    def fixture_root(self, temporary: Path, *names: str) -> Path:
        root = temporary / "workspace"
        source = root / "crates/quanta-gqlang-v2-hir/src"
        source.mkdir(parents=True)
        for name in names:
            shutil.copyfile(FIXTURES / name, source / name)
        return root

    def test_rule_matches_only_the_positive_fixture(self) -> None:
        with tempfile.TemporaryDirectory(prefix="qglang-semgrep-") as temporary:
            root = self.fixture_root(
                Path(temporary),
                f"{RULE}.rs",
                "clean.rs",
            )
            findings = self.scan(CONFIG, root)
        self.assertEqual(len(findings), 1)
        self.assertTrue(findings[0]["check_id"].endswith(RULE))
        self.assertEqual(Path(findings[0]["path"]).name, f"{RULE}.rs")

if __name__ == "__main__":
    unittest.main()
