"""Negative proof for local CI discovery and tool custody."""

from __future__ import annotations

import io
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

from tools.ci import (
    advisory_db, clippy_gate, deny_gate, doctor, doctest_gate, nextest_gate, run_local,
    unittest_gate,
)
from tools.ci.evidence import EvidencePathError, read_json_evidence, write_json_evidence
from tools.ci.environment import (
    EnvironmentSanitizationError, sanitized_environment,
)


class LocalGateTest(unittest.TestCase):
    def test_rust_cargo_and_nextest_semantic_overrides_are_removed(self) -> None:
        source = {
            "PATH": "/usr/bin",
            "RUSTC_BOOTSTRAP": "1",
            "RUSTFLAGS": "--cfg poisoned",
            "RUSTUP_TOOLCHAIN": "poisoned",
            "CARGO_BUILD_TARGET": "poisoned-target",
            "CARGO_PROFILE_DEV_OPT_LEVEL": "0",
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER": "false",
            "NEXTEST_PROFILE": "poisoned",
            "PROPTEST_CASES": "1",
            "QGLANG_DEEP_HIR_CHILD": "1",
            "QGLANG_QUALIFICATION_RUN_ID": "a" * 32,
        }
        self.assertEqual(sanitized_environment(source), {"PATH": "/usr/bin"})
        self.assertEqual(
            sanitized_environment(source, preserve_qualification_run_id=True),
            {"PATH": "/usr/bin", "QGLANG_QUALIFICATION_RUN_ID": "a" * 32},
        )
        source["QGLANG_QUALIFICATION_RUN_ID"] = "not-a-run"
        with self.assertRaisesRegex(
            EnvironmentSanitizationError, "invalid qualification run identity"
        ):
            sanitized_environment(source, preserve_qualification_run_id=True)

    def test_nextest_leak_timeout_is_bounded_and_failing(self) -> None:
        config = (run_local.ROOT / ".config/nextest.toml").read_text(encoding="utf-8")
        self.assertEqual(
            len(re.findall(
                r'^leak-timeout\s*=\s*\{ period = "1s", result = "fail" \}$',
                config, re.MULTILINE,
            )),
            1,
        )
        self.assertEqual(
            nextest_gate.SELECTION_ARGS,
            ("--profile", "default", "--ignore-default-filter"),
        )

    def test_cargo_heavy_composition_is_an_explicit_waterfall(self) -> None:
        text = (run_local.ROOT / "Justfile").read_text(encoding="utf-8")
        expected = {
            "static": ["fmt", "taplo", "clippy-current", "typos", "pm-check", "pm-test"],
            "test": ["nextest", "doctest", "frontdoor-parity"],
            "policy": [
                "ci-rail-test", "qualifier-custody", "arch-lint", "external-consumer",
                "deny", "semgrep", "semantic-honesty",
            ],
        }
        for recipe, leaves in expected.items():
            match = re.search(
                r"^" + re.escape(recipe) + r":\n((?:    [^\n]+\n)+)", text, re.MULTILINE
            )
            self.assertIsNotNone(match, recipe)
            commands = [line.strip() for line in match.group(1).splitlines()]
            self.assertEqual(commands, ["just " + leaf for leaf in leaves])

    def test_local_runner_rejects_invalid_inherited_identity(self) -> None:
        with patch.dict(run_local.os.environ, {"QGLANG_QUALIFICATION_RUN_ID": "not-a-run"}), \
             redirect_stderr(io.StringIO()) as errors:
            self.assertEqual(run_local.main(), 2)
        self.assertIn("invalid qualification run identity", errors.getvalue())

    def test_local_runner_rejects_source_mutation_between_groups(self) -> None:
        completed = subprocess.CompletedProcess(["just", "doctor"], 0, None, None)
        with patch.dict(run_local.os.environ, {}, clear=True), \
             patch.object(run_local, "GROUPS", ("doctor",)), \
             patch.object(run_local, "source_snapshot", side_effect=["before", "after"]), \
             patch.object(run_local, "run_bounded", return_value=completed), \
             redirect_stderr(io.StringIO()) as errors, redirect_stdout(io.StringIO()):
            self.assertEqual(run_local.main(), 2)
        self.assertIn("source changed while running doctor", errors.getvalue())

    def test_source_snapshot_represents_a_tracked_deletion(self) -> None:
        with tempfile.TemporaryDirectory(prefix="qglang-source-snapshot-") as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            tracked = root / "tracked.txt"
            tracked.write_text("present\n", encoding="utf-8")
            subprocess.run(["git", "add", "tracked.txt"], cwd=root, check=True)
            with patch.object(run_local, "ROOT", root):
                present = run_local.source_snapshot()
                tracked.unlink()
                deleted = run_local.source_snapshot()
        self.assertNotEqual(present, deleted)

    def test_unittest_zero_discovery_fails(self) -> None:
        with patch.object(sys, "argv", ["unittest_gate.py", "irrelevant", "test_*.py"]), \
             patch.object(unittest_gate.unittest.defaultTestLoader, "discover", return_value=unittest.TestSuite()), \
             redirect_stderr(io.StringIO()) as errors:
            self.assertEqual(unittest_gate.main(), 2)
        self.assertIn("no tests discovered", errors.getvalue())

    def test_doctest_zero_execution_fails(self) -> None:
        output = "Doc-tests empty\nrunning 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;"
        result = subprocess.CompletedProcess(["cargo", "test"], 0, output, "")
        with patch.object(doctest_gate, "run_bounded", return_value=result), \
             redirect_stderr(io.StringIO()) as errors, redirect_stdout(io.StringIO()):
            self.assertEqual(doctest_gate.main(), 2)
        self.assertIn("no doctests executed", errors.getvalue())
        self.assertEqual(doctest_gate.count_executed(
            "test result: ok. 2 passed; 0 failed; 0 ignored;\n"
            "test result: ok. 3 passed; 0 failed; 0 ignored;\n"
        ), 5)

    def test_doctest_rejects_contradictory_or_ignored_summaries(self) -> None:
        for output in (
            "test result: ok. 1 passed; 1 failed; 0 ignored;",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored;",
            "test result: ok. 1 passed; 0 failed; 1 ignored;",
            "test result: unknown",
        ):
            result = subprocess.CompletedProcess(["cargo", "test"], 0, output, "")
            with patch.object(doctest_gate, "run_bounded", return_value=result), \
                 redirect_stderr(io.StringIO()) as errors, redirect_stdout(io.StringIO()):
                self.assertEqual(doctest_gate.main(), 2, output)
            self.assertIn("doctest gate:", errors.getvalue())

    def test_rust_test_gates_override_inherited_toolchain(self) -> None:
        lock = doctor.read_lock()
        doctest_result = subprocess.CompletedProcess(
            ["cargo", "test"], 0,
            "test result: ok. 1 passed; 0 failed; 0 ignored;", "",
        )
        with patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "poisoned"}), \
             patch.object(doctest_gate, "run_bounded", return_value=doctest_result) as run, \
             redirect_stdout(io.StringIO()):
            self.assertEqual(doctest_gate.main(), 0)
        self.assertEqual(run.call_args.kwargs["env"]["RUSTUP_TOOLCHAIN"], lock["rust"])

    def test_clippy_gate_uses_lock_instead_of_caller_or_justfile_literal(self) -> None:
        completed = subprocess.CompletedProcess(list(clippy_gate.COMMAND), 0, None, None)
        with patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "poisoned"}), \
             patch.object(clippy_gate, "run_bounded", return_value=completed) as run:
            self.assertEqual(clippy_gate.main(), 0)
        self.assertEqual(
            run.call_args.kwargs["env"]["RUSTUP_TOOLCHAIN"], doctor.read_lock()["clippy"]
        )
        justfile = (run_local.ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn("clippy-current:\n    python3 tools/ci/clippy_gate.py\n", justfile)
        self.assertNotRegex(justfile, r"cargo \+\d+\.\d+\.\d+ clippy")

    def test_advisory_db_requires_clean_head_at_fetched_revision(self) -> None:
        with tempfile.TemporaryDirectory(prefix="qglang-advisory-") as directory:
            cargo_home = Path(directory)
            database = cargo_home / "advisory-dbs/advisory-db-fixture"
            database.mkdir(parents=True)
            subprocess.run(["git", "init", "-q"], cwd=database, check=True)
            subprocess.run(
                ["git", "config", "user.name", "Advisory Test"], cwd=database, check=True
            )
            subprocess.run(
                ["git", "config", "user.email", "advisory@example.invalid"],
                cwd=database, check=True,
            )
            subprocess.run(
                ["git", "remote", "add", "origin", "https://github.com/RustSec/advisory-db"],
                cwd=database, check=True,
            )
            tracked = database / "advisory.md"
            tracked.write_text("fixture\n", encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=database, check=True)
            subprocess.run(["git", "commit", "-qm", "fixture"], cwd=database, check=True)
            revision = subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=database, check=True,
                capture_output=True, text=True,
            ).stdout.strip()
            fetched = database / ".git/FETCH_HEAD"
            fetched.write_text(revision + "\t\tbranch 'main' of fixture\n", encoding="utf-8")
            self.assertEqual(advisory_db.state(cargo_home)["revision"], revision)

            fetched.write_text("0" * 40 + "\t\tbranch 'main' of fixture\n", encoding="utf-8")
            with self.assertRaisesRegex(advisory_db.AdvisoryError, "does not match"):
                advisory_db.state(cargo_home)
            fetched.write_text(revision + "\t\tbranch 'main' of fixture\n", encoding="utf-8")
            tracked.write_text("dirty\n", encoding="utf-8")
            with self.assertRaisesRegex(advisory_db.AdvisoryError, "checkout is dirty"):
                advisory_db.state(cargo_home)
            tracked.write_text("fixture\n", encoding="utf-8")
            subprocess.run(
                ["git", "remote", "set-url", "origin", "https://example.invalid/advisory-db"],
                cwd=database, check=True,
            )
            with self.assertRaisesRegex(advisory_db.AdvisoryError, "official RustSec"):
                advisory_db.state(cargo_home)

    def test_deny_gate_rejects_database_change_during_scan(self) -> None:
        before = {
            "path": "/db", "origin": "https://github.com/RustSec/advisory-db",
            "revision": "a" * 40,
            "fetched_at_unix": 1, "fetch_age_s": 2,
        }
        after = dict(before, revision="b" * 40)
        completed = subprocess.CompletedProcess(list(deny_gate.COMMAND), 0, "ok", "")
        with patch.dict(os.environ, {
                 "RUSTFLAGS": "--cfg poisoned",
                 "RUSTUP_TOOLCHAIN": "poisoned",
             }), \
             patch.object(deny_gate, "state", side_effect=[before, after]), \
             patch.object(deny_gate, "run_bounded", return_value=completed) as invoked, \
             patch.object(deny_gate, "write_evidence"), \
             redirect_stderr(io.StringIO()) as errors, redirect_stdout(io.StringIO()):
            self.assertEqual(deny_gate.main(), 2)
        self.assertIn("changed while cargo-deny", errors.getvalue())
        self.assertNotIn("RUSTFLAGS", invoked.call_args.kwargs["env"])
        self.assertEqual(
            invoked.call_args.kwargs["env"]["RUSTUP_TOOLCHAIN"], doctor.read_lock()["rust"]
        )

    def test_evidence_gates_reject_invalid_inherited_identity_before_execution(self) -> None:
        for gate in (nextest_gate, deny_gate):
            with self.subTest(gate=gate.__name__), \
                 patch.dict(os.environ, {
                     "QGLANG_QUALIFICATION_RUN_ID": "not-a-run",
                 }, clear=True), \
                 patch.object(gate, "run_bounded") as invoked, \
                 redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(gate.main(), 2)
                invoked.assert_not_called()
            self.assertIn("invalid qualification run identity", errors.getvalue())

    def test_evidence_writer_rejects_redirected_directory(self) -> None:
        with tempfile.TemporaryDirectory(prefix="qglang-evidence-") as directory:
            root = Path(directory) / "target"
            outside = Path(directory) / "outside"
            root.mkdir()
            outside.mkdir()
            (root / "ci").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(EvidencePathError, "redirected"):
                write_json_evidence(
                    root / "ci/evidence.json", {"pass": False},
                    owned_root=root, prefix="test",
                )
            self.assertEqual(list(outside.iterdir()), [])

    def test_evidence_consume_is_atomic_and_single_use(self) -> None:
        with tempfile.TemporaryDirectory(prefix="qglang-evidence-") as directory:
            root = Path(directory) / "target"
            path = root / "ci/evidence.json"
            write_json_evidence(path, {"run": "one"}, owned_root=root, prefix="test")
            observed, raw = read_json_evidence(path, owned_root=root, consume=True)
            self.assertEqual(observed, {"run": "one"})
            self.assertIn(b'"run": "one"', raw)
            self.assertFalse(path.exists())
            with self.assertRaisesRegex(EvidencePathError, "missing or redirected"):
                read_json_evidence(path, owned_root=root, consume=True)

    def test_tool_lock_rejects_duplicate_missing_and_unknown_keys(self) -> None:
        original = doctor.LOCK.read_text(encoding="utf-8")
        missing_just = "\n".join(
            line for line in original.splitlines() if not line.startswith("just ")
        ) + "\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "toolchain.lock.toml"
            for altered in (
                original + '\njust = "duplicate"\n',
                missing_just,
                original + '\nunknown = "value"\n',
            ):
                path.write_text(altered, encoding="utf-8")
                with self.assertRaises(doctor.DoctorError):
                    doctor.read_lock(path)

    def test_default_rust_toolchain_must_match_the_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "rust-toolchain.toml"
            path.write_text('[toolchain]\nchannel = "1.80.0"\n', encoding="utf-8")
            self.assertEqual(doctor.rust_toolchain_channel(path), "1.80.0")
            with patch.object(doctor, "RUST_TOOLCHAIN", path), \
                 self.assertRaisesRegex(doctor.DoctorError, "sources disagree"):
                doctor.report()
            path.write_text(
                '[toolchain]\nchannel = "1.80.0"\nchannel = "1.81.0"\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(doctor.DoctorError, "exactly one"):
                doctor.rust_toolchain_channel(path)

    def test_tool_doctor_rejects_installed_version_drift(self) -> None:
        installed = "1.81.0-aarch64-apple-darwin\n1.95.0-aarch64-apple-darwin\nnightly-2026-09-23-aarch64-apple-darwin\n"
        result = subprocess.CompletedProcess(["rustup"], 0, installed, "")
        with patch.object(doctor.subprocess, "run", return_value=result), \
             patch.object(doctor, "version", return_value="unexpected"):
            with self.assertRaisesRegex(doctor.DoctorError, "rust version mismatch"):
                doctor.report()

    def test_nextest_inventory_reconciles_only_selected_terminal_tests(self) -> None:
        listing = {
            "test-count": 2,
            "rust-suites": {"binary-id": {
                "package-name": "fixture", "binary-name": "fixture",
                "testcases": {
                    "works": {"ignored": False, "filter-match": {"status": "matches"}},
                    "ignored": {"ignored": True, "filter-match": {
                        "status": "mismatch", "reason": "ignored",
                    }},
                },
            }},
        }
        selected, skipped = nextest_gate.inventory(listing)
        self.assertEqual((len(selected), len(skipped)), (1, 1))
        events = '\n'.join((
            json.dumps({"type": "test", "event": "started", "name": "fixture::fixture$ignored"}),
            json.dumps({"type": "test", "event": "ok", "name": "fixture::fixture$works"}),
        ))
        self.assertEqual(nextest_gate.terminal_events(events), (selected, {}, set()))
        self.assertEqual(nextest_gate.summary_counts(
            "Summary [ 0.1s] 1 test run: 1 passed, 1 skipped"
        ), (1, 1, 0, 0))
        self.assertEqual(nextest_gate.summary_counts(
            "Summary [ 0.1s] 1 test run: 1 passed (1 leaky), 1 skipped"
        ), (1, 1, 0, 1))
        self.assertEqual(
            nextest_gate.reported_leaks(
                "        LEAK [   0.154s] fixture::binary module::test_name"
            ),
            {"fixture::binary$module::test_name"},
        )
        self.assertEqual(
            nextest_gate.reported_leaks(
                "   LEAK-FAIL [   1.004s] fixture::binary module::test_name"
            ),
            {"fixture::binary$module::test_name"},
        )
        self.assertEqual(nextest_gate.TEST_THREADS, 4)
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "missing=1"):
            nextest_gate.reconcile(selected, set(), 0)
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "extra=1"):
            nextest_gate.reconcile(selected, selected | {"foreign"}, 0)
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "leaked=1"):
            nextest_gate.reconcile(selected, selected, 1)
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "names=fixture::leaky"):
            nextest_gate.reconcile(selected, selected, 1, {"fixture::leaky"})
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "unsuccessful=.*failed"):
            nextest_gate.reconcile(
                selected, selected, 0, unsuccessful={"fixture::fixture$works": "failed"}
            )
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "duplicate terminal"):
            nextest_gate.terminal_events(events + "\n" + events.splitlines()[1])
        listing["test-count"] = 3
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "inconsistent"):
            nextest_gate.inventory(listing)
        listing["test-count"] = True
        with self.assertRaisesRegex(nextest_gate.NextestEvidenceError, "schema"):
            nextest_gate.inventory(listing)

    def test_nextest_rejects_failed_event_with_zero_child_exit(self) -> None:
        listing = json.dumps({
            "test-count": 1,
            "rust-suites": {"binary-id": {
                "package-name": "fixture", "binary-name": "fixture",
                "testcases": {"fails": {
                    "filter-match": {"status": "matches"},
                }},
            }},
        })
        failed = json.dumps({
            "type": "test", "event": "failed", "name": "fixture::fixture$fails",
        })
        results = [
            subprocess.CompletedProcess(["cargo", "nextest", "list"], 0, listing, ""),
            subprocess.CompletedProcess(
                ["cargo", "nextest", "run"], 0, failed,
                "Summary [ 0.1s] 1 test run: 0 passed, 1 failed\n",
            ),
        ]
        with patch.object(nextest_gate, "run_bounded", side_effect=results) as run, \
             patch.object(nextest_gate, "write_evidence"), \
             redirect_stderr(io.StringIO()) as errors:
            self.assertEqual(nextest_gate.main(), 2)
        self.assertIn("unsuccessful=fixture::fixture$fails=failed", errors.getvalue())
        list_command = run.call_args_list[0].args[0]
        run_command = run.call_args_list[1].args[0]
        self.assertIn("--ignore-default-filter", list_command)
        self.assertIn("--ignore-default-filter", run_command)
        self.assertEqual(run_command[run_command.index("--retries") + 1], "0")
        self.assertEqual(
            run.call_args_list[1].kwargs["env"]["RUSTUP_TOOLCHAIN"], doctor.read_lock()["rust"]
        )

    def test_nextest_rejects_ignored_inventory_after_running_selected_tests(self) -> None:
        listing = json.dumps({
            "test-count": 2,
            "rust-suites": {"binary-id": {
                "package-name": "fixture", "binary-name": "fixture",
                "testcases": {
                    "works": {"filter-match": {"status": "matches"}},
                    "disabled": {"filter-match": {
                        "status": "mismatch", "reason": "ignored",
                    }},
                },
            }},
        })
        succeeded = json.dumps({
            "type": "test", "event": "ok", "name": "fixture::fixture$works",
        })
        results = [
            subprocess.CompletedProcess(["cargo", "nextest", "list"], 0, listing, ""),
            subprocess.CompletedProcess(
                ["cargo", "nextest", "run"], 0, succeeded,
                "Summary [ 0.1s] 1 test run: 1 passed, 1 skipped\n",
            ),
        ]
        with patch.object(nextest_gate, "run_bounded", side_effect=results), \
             patch.object(nextest_gate, "write_evidence"), \
             redirect_stderr(io.StringIO()) as errors:
            self.assertEqual(nextest_gate.main(), 2)
        self.assertIn("ignored=1", errors.getvalue())
        self.assertIn("fixture::fixture$disabled", errors.getvalue())

    def test_nextest_rejects_summary_that_contradicts_ok_events(self) -> None:
        listing = json.dumps({
            "test-count": 1,
            "rust-suites": {"binary-id": {
                "package-name": "fixture", "binary-name": "fixture",
                "testcases": {"works": {"filter-match": {"status": "matches"}}},
            }},
        })
        succeeded = json.dumps({
            "type": "test", "event": "ok", "name": "fixture::fixture$works",
        })
        results = [
            subprocess.CompletedProcess(["cargo", "nextest", "list"], 0, listing, ""),
            subprocess.CompletedProcess(
                ["cargo", "nextest", "run"], 0, succeeded,
                "Summary [ 0.1s] 1 test run: 0 passed, 1 failed, 0 skipped\n",
            ),
        ]
        with patch.object(nextest_gate, "run_bounded", side_effect=results), \
             patch.object(nextest_gate, "write_evidence"), \
             redirect_stderr(io.StringIO()) as errors:
            self.assertEqual(nextest_gate.main(), 2)
        self.assertIn("summary differs from terminal events", errors.getvalue())


if __name__ == "__main__":
    unittest.main()
