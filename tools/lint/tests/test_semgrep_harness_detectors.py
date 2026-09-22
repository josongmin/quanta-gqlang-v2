#!/usr/bin/env python3
"""Mutation tests for the semgrep gate's own fail-closed detectors.

`test_semgrep_non_vacuity.py` proves the *rules* are non-vacuous (every
configured rule fires on its positive fixture, the clean control stays clean).
This file proves the *harness* is non-vacuous: each detector that is supposed to
reject a broken gate is driven with a deliberately broken config and must reject
with its own message.

Why it matters: a detector that has never fired is indistinguishable from a
detector that cannot fire. Every assertion below checks the specific message, not
merely a non-zero exit status, so a mutation cannot pass because some unrelated
check happened to trip first.

Scope of what can be mutated from outside: `run_semgrep.sh` takes only the config
path from the environment (`QGLANG_SEMGREP_CONFIG`); the repo root and fixture
directory are derived from the script's own location. Every mutation below is
therefore config-side, which is exactly the surface a rule author edits.

These tests require a runnable `semgrep`. In environments where the scanner
cannot start (missing system trust store, unwritable log directory) they fail the
same way the production gate does; see the W6-C audit report for the environment
workaround used to verify them.
"""

from __future__ import annotations

import copy
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "tools/lint/run_semgrep.sh"
CONFIG = ROOT / ".semgrep/qglang-frontend.yml"

# A rule whose fixture is a single line, so narrowing/broadening the pattern has
# an unambiguous expected match count (the harness expects exactly 1 per rule,
# except the runtime-truth vocabulary rule which expects 27).
NARROWED_RULE = "qglang-v2-no-silent-err-discard"
NARROWED_PATTERN = r"Err\(_\)\s*=>"
UNMATCHABLE_PATTERN = r"Err\(_\)\s*=>\s*this_token_appears_nowhere"
# The fixture has one `Err(_) =>` arm and one `Ok(()) =>` arm, so `=>` matches
# twice: enough to prove the exact-count assertion without touching the file.
OVERBROAD_PATTERN = r"=>"


class SemgrepHarnessDetectorTest(unittest.TestCase):
    """Each test mutates one rule and asserts the harness rejects it by name."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.base = yaml.safe_load(CONFIG.read_text(encoding="utf-8"))

    def run_gate_with(self, document: dict) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory(prefix="qglang-semgrep-mutation-") as temporary:
            config = Path(temporary) / "qglang-frontend.yml"
            config.write_text(yaml.safe_dump(document), encoding="utf-8")
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

    def rule(self, document: dict, rule_id: str) -> dict:
        for rule in document["rules"]:
            if rule["id"] == rule_id:
                return rule
        raise AssertionError(f"fixture rule is missing from the config: {rule_id}")

    def assert_rejected(self, result: subprocess.CompletedProcess[str], message: str) -> None:
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(message, result.stderr)

    def test_baseline_config_is_accepted(self) -> None:
        """Control: the unmutated config must pass, or every other case is noise."""
        result = self.run_gate_with(copy.deepcopy(self.base))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("configured=7 executed=7", result.stdout)

    def test_duplicate_rule_id_is_rejected(self) -> None:
        document = copy.deepcopy(self.base)
        document["rules"].append(copy.deepcopy(document["rules"][0]))
        self.assert_rejected(
            self.run_gate_with(document),
            "duplicate configured rule IDs",
        )

    def test_zero_rules_is_rejected(self) -> None:
        document = copy.deepcopy(self.base)
        document["rules"] = []
        self.assert_rejected(
            self.run_gate_with(document),
            "zero configured rules",
        )

    def test_removed_rule_is_rejected_as_inventory_drift(self) -> None:
        """A rule deleted from the config must not silently drop its fixture."""
        document = copy.deepcopy(self.base)
        document["rules"] = [r for r in document["rules"] if r["id"] != NARROWED_RULE]
        self.assert_rejected(
            self.run_gate_with(document),
            "fixture inventory drift",
        )

    def test_renamed_rule_is_rejected_as_inventory_drift(self) -> None:
        """A renamed rule leaves both a missing and a stale fixture entry."""
        document = copy.deepcopy(self.base)
        self.rule(document, NARROWED_RULE)["id"] = NARROWED_RULE + "-renamed"
        self.assert_rejected(
            self.run_gate_with(document),
            "fixture inventory drift",
        )

    def test_rule_that_stops_matching_is_rejected(self) -> None:
        """Proves the per-rule 'every rule executed' assertion is not vacuous."""
        document = copy.deepcopy(self.base)
        rule = self.rule(document, NARROWED_RULE)
        self.assertEqual(rule["pattern-regex"], NARROWED_PATTERN)
        rule["pattern-regex"] = UNMATCHABLE_PATTERN
        result = self.run_gate_with(document)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        # The fixture no longer maps to its own rule, so the harness reports an
        # unexpected/absent match rather than an unexecuted rule.
        self.assertIn(NARROWED_RULE, result.stderr)

    def test_rule_that_over_matches_is_rejected(self) -> None:
        """Proves the exact-match-count assertion (expected == 1) is not vacuous."""
        document = copy.deepcopy(self.base)
        self.rule(document, NARROWED_RULE)["pattern-regex"] = OVERBROAD_PATTERN
        result = self.run_gate_with(document)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("expected", result.stderr)

    def test_rule_scoped_away_from_its_fixture_is_rejected(self) -> None:
        """Proves the include-path check rejects an unreachable rule."""
        document = copy.deepcopy(self.base)
        rule = self.rule(document, NARROWED_RULE)
        rule["paths"]["include"] = ["/crates/quanta-gqlang-v2-missing/src/**/*.rs"]
        self.assert_rejected(
            self.run_gate_with(document),
            NARROWED_RULE,
        )


if __name__ == "__main__":
    unittest.main()
