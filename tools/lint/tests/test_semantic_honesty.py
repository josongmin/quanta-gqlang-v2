#!/usr/bin/env python3
"""Executable positive and negative proof for the semantic honesty policy."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LINTER = ROOT / "tools/lint/semantic_honesty.py"
spec = importlib.util.spec_from_file_location("semantic_honesty", LINTER)
assert spec is not None and spec.loader is not None
lint = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = lint
spec.loader.exec_module(lint)


class SemanticHonestyTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.policy = lint.read_policy()
        cls.parser = lint.parser()

    def rules(self, source: str) -> set[str]:
        findings = lint.scan_source(source.encode(), "crates/fixture/src/lib.rs", self.policy, self.parser)
        return {finding.rule for finding in findings}

    def test_enum_policy_matches_exact_current_variants(self) -> None:
        lint.validate_enum_definitions(ROOT, self.policy)

    def test_negative_to_positive_and_empty_success(self) -> None:
        self.assertIn("SH-01-outcome-laundering", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => PlanOutcomeV2::Plan(plan), _ => Err(Error::Bad) } }"))
        self.assertIn("SH-02-empty-success", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => Ok(None), _ => Err(Error::Bad) } }"))
        self.assertIn("SH-02-untyped-success", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => Ok(plan), _ => Err(Error::Bad) } }"))
        self.assertIn("SH-02-empty-success", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => (Ok(None)), _ => Err(Error::Bad) } }"))
        self.assertIn("SH-02-untyped-success", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => { Ok(plan) }, _ => Err(Error::Bad) } }"))
        self.assertIn("SH-02-empty-success", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => (/* c */ Ok(None)), _ => Err(Error::Bad) } }"))
        self.assertIn("SH-02-empty-success", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => { Ok(None) /* c */ }, _ => Err(Error::Bad) } }"))

    def test_reason_loss_and_terminal_error_retention(self) -> None:
        self.assertIn("SH-04-binding-loss", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => Err(Error::Bad), _ => Err(Error::Bad) } }"))
        self.assertNotIn("SH-04-binding-loss", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => Err(Error::from(reason)), _ => Err(Error::Bad) } }"))
        self.assertIn("SH-04-binding-loss", self.rules("fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(payload) => Err(Error::Bad), _ => Err(Error::Bad) } }"))

    def test_branch_and_return_paths_are_checked_independently(self) -> None:
        for value in (
            "if cond { Ok(None) } else { Err(reason) }",
            "match flag { true => Err(reason), false => Ok(None) }",
            "{ if cond { return Ok(None); } Err(reason) }",
        ):
            source = f"fn f(x: PlanOutcomeV2, cond: bool, flag: bool) {{ match x {{ PlanOutcomeV2::NoPlan(reason) => {value}, _ => Err(Error::Bad) }} }}"
            self.assertIn("SH-02-empty-success", self.rules(source), value)

    def test_nested_constructor_is_not_the_returned_constructor(self) -> None:
        source = "fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => Err(E::from(PlanOutcomeV2::Plan(reason))), _ => Err(E::Bad) } }"
        self.assertNotIn("SH-01-outcome-laundering", self.rules(source))
        source = "fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => PlanOutcomeV2::Plan(Err(reason)), _ => Err(E::Bad) } }"
        self.assertIn("SH-01-outcome-laundering", self.rules(source))

    def test_closure_return_is_not_arm_exit(self) -> None:
        source = "fn f(x: PlanOutcomeV2) { match x { PlanOutcomeV2::NoPlan(reason) => { let closure = || return Ok(None); Err(reason) }, _ => Err(E::Bad) } }"
        self.assertNotIn("SH-02-empty-success", self.rules(source))

    def test_unsupported_macro_exit_is_type_scoped(self) -> None:
        governed = "fn f(x: PlanOutcomeV2) -> Result<PlanOutcomeV2, E> { match x { PlanOutcomeV2::NoPlan(reason) => some_macro!(reason), _ => Err(E::Bad) } }"
        self.assertIn("SH-11-unsupported-exit", self.rules(governed))
        rendered = "fn f(x: AnalysisItemV2) -> Vec<String> { match x { AnalysisItemV2::BlockedByDiagnostics(diagnostics) => vec![format!(\"{}\", diagnostics.len())], _ => vec![] } }"
        self.assertNotIn("SH-11-unsupported-exit", self.rules(rendered))

    def test_status_exit_code_and_wildcard(self) -> None:
        self.assertIn("SH-03-negative-exit", self.rules("fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => 0, AnalysisStatusV2::Complete => 0 } }"))
        self.assertIn("SH-06-success-wildcard", self.rules("fn f(s: PlanOutcomeV2) { match s { PlanOutcomeV2::Plan(p) => Ok(p), _ => Ok(None) } }"))
        self.assertNotIn("SH-06-success-wildcard", self.rules("fn f(s: PlanOutcomeV2) { match s { PlanOutcomeV2::Plan(p) => Ok(p), _ => Err(Error::Bad) } }"))
        self.assertNotIn("SH-06-success-wildcard", self.rules("fn f(s: PlanOutcomeV2) { match s { PlanOutcomeV2::Plan(p) => Ok(p), _ => { return Err(Error::Bad); } } }"))
        self.assertIn("SH-03-negative-exit", self.rules("fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => ((0)), AnalysisStatusV2::Complete => 0 } }"))
        self.assertIn("SH-03-negative-exit", self.rules("fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => (0, AnalysisStatusV2::Error).0, AnalysisStatusV2::Complete => 0 } }"))
        self.assertIn("SH-03-negative-exit", self.rules("fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => ((0, AnalysisStatusV2::Error)).0, AnalysisStatusV2::Complete => 0 } }"))
        for zero in ("0_i32", "0x0", "0o0", "0b0"):
            self.assertIn("SH-03-negative-exit", self.rules(f"fn f(s: AnalysisStatusV2) {{ match s {{ AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => {zero}, AnalysisStatusV2::Complete => 0 }} }}"))
        for zero in ("-0", "0 as i32", "0 as _", "0x1_0000_0000_u64 as i32"):
            self.assertIn("SH-03-negative-exit", self.rules(f"fn f(s: AnalysisStatusV2) {{ match s {{ AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => {zero}, AnalysisStatusV2::Complete => 0 }} }}"))
        for complete in ('r"complete"', '"\\u{63}omplete"', '"com\\\n    plete"'):
            self.assertIn("SH-03-negative-exit", self.rules(f"impl AnalysisStatusV2 {{ fn f(self) {{ match self {{ Self::Error => {complete}, Self::Complete => \"complete\", Self::Incomplete => \"incomplete\" }} }} }}"))
        self.assertIn("SH-11-unsupported-exit", self.rules("fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Error => 1 - 1, _ => 1 } }"))

    def test_error_to_ok_and_serde_default(self) -> None:
        self.assertIn("SH-05-error-to-ok", self.rules("fn f(x: Result<u8, E>) { match x { Err(_error) => Ok(()), Ok(value) => Ok(value) } }"))
        self.assertIn("SH-05-error-to-ok", self.rules("fn f(x: Result<u8, E>) { match x { Err(error) => Ok(0), Ok(value) => Ok(value) } }"))
        self.assertIn("SH-05-error-to-ok", self.rules("fn f(x: Result<u8, E>) { match x { Err(ref error) => if cond { Err(error.into()) } else { Ok(0) }, Ok(value) => Ok(value) } }"))
        self.assertIn("SH-08-semantic-serde-default", self.rules("struct RequestV2 { #[serde(default)] status: String }"))
        self.assertNotIn("SH-08-semantic-serde-default", self.rules("struct Other { #[serde(default)] status: String }"))
        self.assertIn("SH-05-error-to-ok", self.rules("fn f(x: Result<u8, E>) { match x { Err(_error) => { (Ok(())) }, Ok(value) => Ok(value) } }"))
        self.assertIn("SH-05-error-to-ok", self.rules("fn f(x: Result<u8, E>) { match x { Err(/* c */ _error) => (/* c */ Ok({ let _ = AnalysisStatusV2::Error; () })), Ok(value) => Ok(value) } }"))

    def test_semantic_field_ok_and_safe_local_ok(self) -> None:
        self.assertIn("SH-07-semantic-ok", self.rules("fn f() { let p = AnalysisPayloadV2 { analysis_status: parse().ok(), diagnostics: vec![] }; }"))
        self.assertNotIn("SH-07-semantic-ok", self.rules("fn f() { let version = source.parse::<u8>().ok(); }"))

    def test_cfg_test_and_parse_error_scope(self) -> None:
        self.assertEqual(self.rules("#[cfg(test)] mod tests { fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete => 0, _ => 0 } } }"), set())
        self.assertEqual(self.rules("#[cfg(all(feature = \"x\", test))] mod tests { fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete => 0, _ => 0 } } }"), set())
        self.assertIn("SH-03-negative-exit", self.rules("#[cfg(any(test, feature = \"x\"))] fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete => 0, _ => 1 } }"))
        self.assertIn("SH-00-parse-error", self.rules("fn f(s: AnalysisStatusV2) { match s { AnalysisStatusV2::Incomplete => "))

    def test_new_unclassified_enum_variant_fails_policy(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            family = self.policy["enum_families"][0]
            source = root / family["source"]
            source.parent.mkdir(parents=True)
            source.write_text("pub enum PlanOutcomeV2 { Plan(u8), NoPlan(u8), NewFailure }\n")
            with self.assertRaisesRegex(ValueError, "enum policy drift"):
                lint.validate_enum_definitions(root, {"enum_families": [family]})

    def test_allowed_nonplan_projection(self) -> None:
        source = "fn f(x: ItemRunV2) { match x { ItemRunV2::NoPlan { reason, .. } => AnalysisItemV2::NotApplicable(reason), _ => Err(Error::Bad) } }"
        self.assertNotIn("SH-01-outcome-laundering", self.rules(source))

    def test_real_status_mutants(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-pipeline/src/compile.rs").read_text()
        for old, new in (
            ("AnalysisStatusV2::Incomplete\n", "AnalysisStatusV2::Complete\n"),
            ("AnalysisStatusV2::Error\n", "AnalysisStatusV2::Complete\n"),
        ):
            self.assertIn("SH-09-status-priority", self.rules(source.replace(old, new, 1)))
        for old, new in (
            ("if !self.unresolved.is_empty()", "if false && !self.unresolved.is_empty()"),
            ("} else if self\n", "} else if false && self\n"),
        ):
            self.assertIn("SH-09-status-priority", self.rules(source.replace(old, new, 1)))

    def test_status_classifier_allows_value_neutral_wrappers(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-pipeline/src/compile.rs").read_text()
        for variant in ("Incomplete", "Error", "Complete"):
            mutant = source.replace(f"AnalysisStatusV2::{variant}\n", f"(AnalysisStatusV2::{variant})\n", 1)
            self.assertNotIn("SH-09-status-priority", self.rules(mutant))
        mutant = source.replace("if !self.unresolved.is_empty()", "if /* priority */ (!self.unresolved.is_empty())", 1)
        self.assertNotIn("SH-09-status-priority", self.rules(mutant))
        mutant = source.replace("if !self.unresolved.is_empty()", "if (/* priority */ !self.unresolved.is_empty())", 1)
        self.assertNotIn("SH-09-status-priority", self.rules(mutant))
        mutant = source.replace("AnalysisStatusV2::Incomplete\n", "{ AnalysisStatusV2::Incomplete /* c */ }\n", 1)
        self.assertNotIn("SH-09-status-priority", self.rules(mutant))

    def test_status_classifier_rejects_hidden_control_flow(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-pipeline/src/compile.rs").read_text()
        mutant = source.replace(
            "if !self.unresolved.is_empty()",
            "if { return AnalysisStatusV2::Complete; !self.unresolved.is_empty() }",
            1,
        )
        self.assertIn("SH-09-status-priority", self.rules(mutant))

    def test_early_success_return_in_status_is_rejected(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-pipeline/src/compile.rs").read_text()
        mutant = source.replace("pub fn status(&self) -> AnalysisStatusV2 {", "pub fn status(&self) -> AnalysisStatusV2 { return AnalysisStatusV2::Complete;", 1)
        self.assertIn("SH-09-status-priority", self.rules(mutant))

    def test_real_cli_exit_mutant(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-cli/src/commands/v3_query.rs").read_text()
        mutant = source.replace("AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => 1", "AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => 0", 1)
        self.assertIn("SH-03-negative-exit", self.rules(mutant))
        mutant = source.replace("AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => 1", "AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => { (0) }", 1)
        self.assertIn("SH-03-negative-exit", self.rules(mutant))
        for zero in ("0_i32", "0x0"):
            mutant = source.replace("AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => 1", f"AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => {zero}", 1)
            self.assertIn("SH-03-negative-exit", self.rules(mutant))

    def test_real_stdio_payload_mutants(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-diagnostics-stdio/src/handlers/v3_diagnostics.rs").read_text()
        mutants = (
            source.replace("analysis_status: AnalysisStatusV2::Error.keyword()", "analysis_status: AnalysisStatusV2::Complete.keyword()", 1),
            source.replace(
                "diagnostics: failure\n                    .diagnostics()\n                    .iter()\n                    .map(WireDiagnosticV2::from)\n                    .collect(),",
                "diagnostics: Vec::new(),",
                1,
            ),
            source.replace(
                "diagnostics: failure\n                    .diagnostics()\n                    .iter()\n                    .map(WireDiagnosticV2::from)\n                    .collect(),",
                "diagnostics: { let _discard = failure; Vec::new() },",
                1,
            ),
            source.replace(
                "Ok(AnalysisPayloadV2 {\n                analysis_status: AnalysisStatusV2::Error.keyword(),",
                "if failure.diagnostics().is_empty() { Ok(Default::default()) } else { Ok(AnalysisPayloadV2 {\n                analysis_status: AnalysisStatusV2::Error.keyword(),",
                1,
            ).replace(".collect(),\n            })\n        }\n        Err(FrontendTransactionErrorV2::Authority", ".collect(),\n            }) }\n        }\n        Err(FrontendTransactionErrorV2::Authority", 1),
        )
        for mutant in mutants:
            self.assertIn("SH-10-document-error-payload", self.rules(mutant))

    def test_document_error_pattern_wrappers_do_not_hide_payload_loss(self) -> None:
        source = (ROOT / "crates/quanta-gqlang-v2-diagnostics-stdio/src/handlers/v3_diagnostics.rs").read_text()
        empty = source.replace(
            "diagnostics: failure\n                    .diagnostics()\n                    .iter()\n                    .map(WireDiagnosticV2::from)\n                    .collect(),",
            "diagnostics: Vec::new(),",
            1,
        )
        for pattern in (
            "Err(/* c */ FrontendTransactionErrorV2::Document(failure))",
            "Err((FrontendTransactionErrorV2::Document(failure)))",
            "Err(FrontendTransactionErrorV2::Document(failure @ _))",
            "Err(FrontendTransactionErrorV2::Document(ref failure))",
            "Err(FrontendTransactionErrorV2::Document(mut failure))",
            "Err(FrontendTransactionErrorV2::Document(((failure))))",
            "Err(quanta_gqlang_v2_frontend_contract::FrontendTransactionErrorV2::Document(failure))",
        ):
            mutant = empty.replace("Err(FrontendTransactionErrorV2::Document(failure))", pattern, 1)
            self.assertIn("SH-10-document-error-payload", self.rules(mutant))

        renamed = source.replace("failure", "document_failure")
        mutant = renamed.replace(
            "diagnostics: document_failure\n                    .diagnostics()\n                    .iter()\n                    .map(WireDiagnosticV2::from)\n                    .collect(),",
            "diagnostics: Vec::new(),",
            1,
        )
        self.assertIn("SH-10-document-error-payload", self.rules(mutant))

        no_payload = "fn f(x: Result<(), FrontendTransactionErrorV2>) -> Result<AnalysisPayloadV2, E> { match x { Err(FrontendTransactionErrorV2::Document(failure)) => Ok(Default::default()), _ => Err(Error::Bad) } }"
        self.assertIn("SH-10-document-error-payload", self.rules(no_payload))
        all_error = "fn f(x: Result<(), FrontendTransactionErrorV2>) -> Result<AnalysisPayloadV2, E> { match x { Err(FrontendTransactionErrorV2::Document(failure)) => Err(E::from(failure)), _ => Err(Error::Bad) } }"
        self.assertIn("SH-10-document-error-payload", self.rules(all_error))


if __name__ == "__main__":
    unittest.main()
