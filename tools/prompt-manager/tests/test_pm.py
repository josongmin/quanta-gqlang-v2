from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


PM_PATH = Path(__file__).resolve().parents[1] / "pm.py"
SPEC = importlib.util.spec_from_file_location("prompt_manager", PM_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load prompt-manager")
PM = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PM)


class PromptManagerTest(unittest.TestCase):
    def setUp(self) -> None:
        self.config = PM.load_config()
        self.rendered = PM.render_all(self.config)

    def test_all_candidates_fit_declared_budgets_and_scopes(self) -> None:
        sizes = PM.validate_all(self.config, self.rendered, ratchet=False)
        self.assertEqual(set(self.config["targets"]), set(sizes["targets"]))

    def test_cursor_rules_are_mdc_and_conditionally_attached(self) -> None:
        for name, spec in self.config["targets"].items():
            if spec["kind"] != "cursor_rule":
                continue
            self.assertTrue(spec["output"].endswith(".mdc"), name)
            self.assertIn("\nalwaysApply: false\n", self.rendered[name])
            self.assertIn("\nglobs: ", self.rendered[name])

    def test_claude_rules_are_path_scoped(self) -> None:
        for name, spec in self.config["targets"].items():
            if spec["kind"] != "claude_rule":
                continue
            self.assertTrue(self.rendered[name].startswith("---\npaths:\n"), name)

    def test_startup_files_do_not_import_document_stack(self) -> None:
        startup = self.rendered["agents"] + self.rendered["claude"]
        self.assertNotIn("@README", startup)
        self.assertNotIn("@docs/", startup)
        self.assertEqual(self.rendered["claude"].count("@AGENTS.md"), 1)

    def test_all_scoped_rule_files_are_managed(self) -> None:
        PM.validate_managed_rule_directories(self.config)

    def test_imports_cannot_escape_repository_through_parent_or_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repo"
            root.mkdir()
            root = root.resolve()
            (root / "AGENTS.md").write_text("local\n", encoding="utf-8")
            (parent / "foreign.md").write_text("foreign\n", encoding="utf-8")
            (root / "linked.md").symlink_to(parent / "foreign.md")
            config = {"targets": {"agents": {"output": "AGENTS.md"}}}
            with patch.object(PM, "ROOT", root):
                PM.validate_imports(config, {"claude": "@AGENTS.md\n"})
                for imported in ("../foreign.md", "linked.md"):
                    with self.subTest(imported=imported):
                        with self.assertRaisesRegex(PM.PromptManagerError, "external import"):
                            PM.validate_imports(config, {"claude": f"@{imported}\n"})


if __name__ == "__main__":
    unittest.main()
