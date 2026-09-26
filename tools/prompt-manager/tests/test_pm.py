from __future__ import annotations

import importlib.util
import copy
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

    def test_ratchet_rejects_removed_startup_member_and_moved_output(self) -> None:
        config = copy.deepcopy(self.config)
        config["context_groups"]["claude_startup"]["targets"].remove("claude")
        with self.assertRaisesRegex(PM.PromptManagerError, "group_members changed"):
            PM.validate_ratchet(config, PM.current_sizes(config, self.rendered))
        config = copy.deepcopy(self.config)
        config["targets"]["agents"]["output"] = "MOVED.md"
        with self.assertRaisesRegex(PM.PromptManagerError, "outputs changed"):
            PM.validate_ratchet(config, PM.current_sizes(config, self.rendered))

    def test_cursor_rules_are_mdc_and_conditionally_attached(self) -> None:
        verified = 0
        for name, spec in self.config["targets"].items():
            if spec["kind"] != "cursor_rule":
                continue
            verified += 1
            self.assertTrue(spec["output"].endswith(".mdc"), name)
            self.assertIn("\nalwaysApply: false\n", self.rendered[name])
            self.assertIn("\nglobs: ", self.rendered[name])
        self.assertGreater(verified, 0, "no cursor_rule target was verified")

    def test_claude_rules_are_path_scoped(self) -> None:
        verified = 0
        for name, spec in self.config["targets"].items():
            if spec["kind"] != "claude_rule":
                continue
            verified += 1
            self.assertTrue(self.rendered[name].startswith("---\npaths:\n"), name)
        self.assertGreater(verified, 0, "no claude_rule target was verified")

    def test_scoping_rejects_justfile_case_regression_for_both_clients(self) -> None:
        for target in ("claude_rust", "cursor_rust"):
            config = copy.deepcopy(self.config)
            spec = config["targets"][target]
            if target == "claude_rust":
                spec["paths"] = ["justfile"]
            else:
                spec["globs"] = "justfile"
            with self.subTest(target=target):
                with self.assertRaisesRegex(PM.PromptManagerError, "wrong-case scope path"):
                    PM.validate_scoping(config, PM.render_all(config))

    def test_literal_scopes_check_every_path_component_and_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / "tools").mkdir()
            (root / "tools" / "gate.py").write_text("pass\n", encoding="utf-8")
            with patch.object(PM, "ROOT", root):
                PM.validate_scope_paths("rule", ["tools/gate.py", "tools/**/*.py"])
                for path in ("Tools/gate.py", "tools/Gate.py", "tools/missing.py"):
                    with self.subTest(path=path):
                        with self.assertRaisesRegex(PM.PromptManagerError, "scope path"):
                            PM.validate_scope_paths("rule", [path])

    def test_startup_files_do_not_import_document_stack(self) -> None:
        startup = self.rendered["agents"] + self.rendered["claude"]
        self.assertNotIn("@README", startup)
        self.assertNotIn("@docs/", startup)
        self.assertEqual(self.rendered["claude"].count("@AGENTS.md"), 1)

    def test_claude_bootstrap_cannot_drop_duplicate_or_quote_required_import(self) -> None:
        for body in ("No shared instructions\n", "@AGENTS.md\n@AGENTS.md\n", "`@AGENTS.md`\n"):
            rendered = self.rendered.copy()
            rendered["claude"] = body
            with self.subTest(body=body):
                with self.assertRaisesRegex(PM.PromptManagerError, "import contract changed"):
                    PM.validate_imports(self.config, rendered)

    def test_all_scoped_rule_files_are_managed(self) -> None:
        PM.validate_managed_rule_directories(self.config)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            stray = root / ".cursor/rules/stray.mdc"
            stray.parent.mkdir(parents=True)
            stray.write_text("stray\n", encoding="utf-8")
            with patch.object(PM, "ROOT", root):
                with self.assertRaisesRegex(
                    PM.PromptManagerError, "unmanaged scoped prompt file"
                ):
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
            config = {
                "targets": {
                    "agents": {"output": "AGENTS.md"},
                    "claude": {"output": "CLAUDE.md"},
                },
                "context_groups": {"startup": {"targets": ["agents", "claude"]}},
            }
            with patch.object(PM, "ROOT", root):
                PM.validate_imports(config, {"agents": "local\n", "claude": "@AGENTS.md\n"})
                for imported in ("../foreign.md", "linked.md"):
                    with self.subTest(imported=imported):
                        with self.assertRaisesRegex(PM.PromptManagerError, "external import"):
                            PM.validate_imports(config, {"agents": "local\n", "claude": f"@{imported}\n"})

    def test_imports_cannot_bypass_budgets_or_group_membership(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / "extra.md").write_text("large context\n" * 1000, encoding="utf-8")
            config = copy.deepcopy(self.config)
            rendered = self.rendered.copy()
            with patch.object(PM, "ROOT", root):
                rendered["claude"] = "@extra.md\n"
                with self.assertRaisesRegex(PM.PromptManagerError, "unbudgeted import"):
                    PM.validate_imports(config, rendered)
                rendered["claude"] = "@.claude/rules/rust-frontend.md\n"
                with self.assertRaisesRegex(PM.PromptManagerError, "import escapes context group"):
                    PM.validate_imports(config, rendered)
                rendered["claude"] = self.rendered["claude"]
                rendered["claude_rust"] = "@../../AGENTS.md\n"
                with self.assertRaisesRegex(PM.PromptManagerError, "import without context group"):
                    PM.validate_imports(config, rendered)

    def test_import_cycles_are_rejected(self) -> None:
        config = copy.deepcopy(self.config)
        del config["context_groups"]["cursor_root_startup"]
        rendered = self.rendered.copy()
        rendered["agents"] = "@CLAUDE.md\n"
        with self.assertRaisesRegex(PM.PromptManagerError, "import cycle"):
            PM.validate_imports(config, rendered)

    def test_imports_resolve_relative_to_output_and_ignore_code_examples(self) -> None:
        config = {
            "targets": {
                "agents": {"output": "AGENTS.md"},
                "claude": {"output": "nested/CLAUDE.md"},
            },
            "context_groups": {"startup": {"targets": ["agents", "claude"]}},
        }
        PM.validate_imports(config, {
            "agents": "Shared instructions\n",
            "claude": (
                "@../AGENTS.md\n`@missing-inline.md`\n``@missing-double.md``\n"
                "```md\n@missing-fenced.md\n````\n"
                "~~~md\n@missing-tilde.md\n~~~\n"
                "```md\n@missing-unclosed.md\n"
            ),
        })


if __name__ == "__main__":
    unittest.main()
