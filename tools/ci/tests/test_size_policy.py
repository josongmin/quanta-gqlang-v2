"""Independent positive/negative fixtures for production function-size custody."""

from pathlib import Path
import tempfile
import unittest

from tools.lint import size_gate


class FunctionSizePolicyTest(unittest.TestCase):
    def findings(self, source: str, name: str = "crates/fixture/src/lib.rs") -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / name
            path.parent.mkdir(parents=True)
            path.write_text(source)
            return size_gate.function_size_overrides(root, {name})

    def test_exact_lint_and_covering_groups_are_rejected(self) -> None:
        for level in ("allow", "expect"):
            for lint in ("clippy::too_many_lines", "clippy::pedantic", "clippy::all", "warnings"):
                with self.subTest(level=level, lint=lint):
                    result = self.findings(f"#[{level}({lint})] fn f() {{}}")
                    self.assertEqual(len(result), 1)
                    self.assertIn(f"{level}({lint})", result[0])

    def test_module_impl_inner_and_conditional_overrides_are_rejected(self) -> None:
        for source in (
            "#![allow(clippy::too_many_lines)] fn f() {}",
            "#[expect(clippy::too_many_lines)] mod owner { fn f() {} }",
            "struct S; #[allow(clippy::pedantic)] impl S { fn f() {} }",
            '#[cfg_attr(feature="large",allow(clippy::too_many_lines))] fn f() {}',
            '#[cfg_attr(test,expect(clippy::too_many_lines))] fn f() {}',
        ):
            with self.subTest(source=source):
                self.assertEqual(len(self.findings(source)), 1)

    def test_literal_macro_attributes_cannot_bypass_the_gate(self) -> None:
        self.assertEqual(len(self.findings(
            "macro_rules! emit { () => { #[allow(clippy::too_many_lines)] fn f() {} }; }"
        )), 1)
        self.assertEqual(len(self.findings(
            "macro_rules! emit { ($lint:meta) => { #[allow($lint)] fn f() {} }; }"
        )), 1)
        self.assertEqual(self.findings(
            'macro_rules! text { () => { "#[allow(clippy::too_many_lines)]" }; }'
        ), [])

    def test_comments_literals_and_unrelated_lints_are_not_overrides(self) -> None:
        source = r'''
            // #[allow(clippy::too_many_lines)]
            const MESSAGE: &str = "#[expect(clippy::too_many_lines)]";
            #[allow(clippy::unreadable_literal, reason="clippy::too_many_lines")]
            fn f() { let _ = r#"allow(clippy::all)"#; }
        '''
        self.assertEqual(self.findings(source), [])

    def test_test_only_scope_is_allowed_with_comment_trivia(self) -> None:
        for source in (
            "#[cfg(test)] // comment between guard and item\n#[expect(clippy::too_many_lines)] fn f() {}",
            "#[cfg(all(test,unix))] mod tests { #[allow(warnings)] fn f() {} }",
            "#![cfg(test)] #![allow(clippy::too_many_lines)] fn f() {}",
            "#[cfg(test)] macro_rules! test { () => { #[allow(warnings)] fn f() {} }; }",
        ):
            with self.subTest(source=source):
                self.assertEqual(self.findings(source), [])

    def test_guard_must_exclude_all_production_configurations(self) -> None:
        self.assertEqual(len(self.findings(
            '#[cfg(any(test,feature="production"))] #[expect(clippy::too_many_lines)] fn f() {}'
        )), 1)
        self.assertEqual(len(self.findings(
            "#[cfg(not(test))] #[allow(clippy::too_many_lines)] fn f() {}"
        )), 1)

    def test_invalid_rust_is_not_silently_accepted(self) -> None:
        with self.assertRaisesRegex(size_gate.SizeGateError, "invalid Rust"):
            self.findings("fn broken( {")


if __name__ == "__main__":
    unittest.main()
