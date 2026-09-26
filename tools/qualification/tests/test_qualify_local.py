"""Fast custody and cache-identity tests for the local qualification driver."""

from __future__ import annotations

import fcntl
import hashlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import qualify_local as qualify_local_module  # noqa: E402
from qualify_local import (  # noqa: E402
    QualificationDriver, QualificationError, git, valid_deny_evidence,
    valid_nextest_evidence, validate_recipe_durations, source_status,
)


def run(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, cwd=str(cwd), capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise AssertionError("%s failed: %s" % (" ".join(args), result.stderr))
    return result


def rust_source(version: int) -> str:
    return (
        "pub fn version() -> u32 { %d }\n" % version
        + "#[cfg(test)] mod tests {\n"
        + "  #[test] fn witness() {\n"
        + '    println!("VERSION=%d;DIR={}", env!("CARGO_MANIFEST_DIR"));\n' % version
        + "    assert_eq!(super::version(), %d);\n" % version
        + "  }\n}\n"
    )


class QualificationDriverTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="qglang-qualifier-test-")
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name) / "repo"
        self.repo.mkdir()
        run(["git", "init", "-q"], self.repo)
        run(["git", "config", "user.name", "Qualification Test"], self.repo)
        run(["git", "config", "user.email", "qualification@example.invalid"], self.repo)
        (self.repo / ".gitignore").write_text("/target/\n", encoding="utf-8")
        (self.repo / "src").mkdir()
        (self.repo / "src" / "lib.rs").write_text(rust_source(1), encoding="utf-8")
        run(["git", "add", "."], self.repo)
        run(["git", "commit", "-qm", "fixture-v1"], self.repo)
        self.driver = QualificationDriver(self.repo)
        self.driver.root.mkdir(parents=True)

    def pin(self) -> tuple[str, str]:
        return git(self.repo, "rev-parse", "HEAD"), git(self.repo, "rev-parse", "HEAD^{tree}")

    def test_nextest_evidence_rejects_unsuccessful_or_malformed_custody(self) -> None:
        run_id = "a" * 32
        evidence = {
            "schema": 1,
            "qualification_run_id": run_id,
            "discovered": 1,
            "selected": 1,
            "executed": 1,
            "skipped_ignored": 0,
            "leaked": 0,
            "leaked_tests": [],
            "unsuccessful": {},
            "test_threads": 4,
            "summary_run": 1,
            "summary_passed": 1,
            "summary_failed": 0,
            "inventory_sha256": "b" * 64,
            "events_sha256": "c" * 64,
            "exit": 0,
        }
        self.assertTrue(valid_nextest_evidence(evidence, run_id))
        ignored = dict(evidence, discovered=2, skipped_ignored=1)
        self.assertFalse(valid_nextest_evidence(ignored, run_id))
        for key, value in (
            ("unsuccessful", {"fixture::bin$test": "failed"}),
            ("leaked_tests", ["fixture::bin$test"]),
            ("inventory_sha256", "not-a-digest"),
            ("test_threads", 0),
            ("summary_passed", 0),
            ("summary_failed", 1),
        ):
            malformed = dict(evidence)
            malformed[key] = value
            self.assertFalse(valid_nextest_evidence(malformed, run_id), key)

    def test_deny_evidence_is_bound_to_run_and_stable_revision(self) -> None:
        run_id = "a" * 32
        state = {
            "path": "/db", "origin": "https://github.com/RustSec/advisory-db",
            "revision": "b" * 40,
            "fetched_at_unix": 1, "fetch_age_s": 2,
        }
        evidence = {
            "schema": 1,
            "qualification_run_id": run_id,
            "command": [
                "cargo", "deny", "--frozen", "check", "--disable-fetch",
                "bans", "licenses", "sources", "advisories",
            ],
            "exit": 0,
            "before": state,
            "after": dict(state, fetch_age_s=3),
            "output_sha256": "c" * 64,
        }
        self.assertTrue(valid_deny_evidence(evidence, run_id, state))
        changed = dict(state, revision="d" * 40)
        self.assertFalse(valid_deny_evidence(evidence, run_id, changed))
        self.assertFalse(valid_deny_evidence(evidence, "e" * 32, state))
        for key, value in (("schema", True), ("exit", False)):
            malformed = dict(evidence)
            malformed[key] = value
            self.assertFalse(valid_deny_evidence(malformed, run_id, state), key)

    def test_nextest_raw_streams_are_archived_only_at_matching_hashes(self) -> None:
        self.prepare()
        ci = self.driver.build_cache / "ci"
        ci.mkdir()
        receipt = self.driver.root / self.pin()[0]
        receipt.mkdir()
        streams = {"nextest-list.json": b"{}\n", "nextest-events.jsonl": b'{"type":"test"}\n'}
        for name, raw in streams.items():
            (ci / name).write_bytes(raw)
        evidence = {
            "inventory_sha256": hashlib.sha256(streams["nextest-list.json"]).hexdigest(),
            "events_sha256": hashlib.sha256(streams["nextest-events.jsonl"]).hexdigest(),
        }
        self.assertEqual(self.driver.archive_nextest(receipt, evidence), {
            name: hashlib.sha256(raw).hexdigest() for name, raw in streams.items()
        })
        self.assertEqual((receipt / "nextest-events.jsonl").read_bytes(), streams["nextest-events.jsonl"])
        (ci / "nextest-events.jsonl").write_bytes(b"changed")
        with self.assertRaisesRegex(QualificationError, "hash differs"):
            self.driver.archive_nextest(receipt, evidence)

    def test_successful_receipt_requires_exact_recipe_timing_inventory(self) -> None:
        validate_recipe_durations({"ci": 1.0}, {"ci"})
        for observed in ({}, {"ci": 1.0, "foreign": 2.0}):
            with self.assertRaisesRegex(QualificationError, "timing inventory"):
                validate_recipe_durations(observed, {"ci"})

    def prepare(self) -> None:
        self.driver.prepare_checkout(*self.pin())

    def test_same_sha_keeps_checkout_path_mtime_and_cache(self) -> None:
        self.prepare()
        source = self.driver.checkout / "src" / "lib.rs"
        source_mtime = source.stat().st_mtime_ns
        sentinel = self.driver.build_cache / "sentinel"
        sentinel.write_text("keep", encoding="utf-8")
        self.prepare()
        self.assertEqual(source.stat().st_mtime_ns, source_mtime)
        self.assertEqual(sentinel.read_text(encoding="utf-8"), "keep")
        self.assertEqual(self.driver.checkout.resolve(), self.driver.checkout)
        self.assertEqual(self.driver.checkout.joinpath("target").resolve(), self.driver.build_cache)

    def smoke_changed_source_rebuilds_at_the_same_manifest_path(self) -> None:
        (self.repo / "Cargo.toml").write_text(
            '[package]\nname = "qualifier-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        run(["cargo", "generate-lockfile", "--offline"], self.repo)
        run(["git", "add", "Cargo.toml", "Cargo.lock"], self.repo)
        run(["git", "commit", "-qm", "fixture-cargo"], self.repo)
        self.prepare()

        def cargo_test() -> str:
            env = dict(os.environ)
            env.pop("CARGO_TARGET_DIR", None)
            result = subprocess.run(
                ["cargo", "test", "--locked", "--offline", "--", "--nocapture"],
                cwd=str(self.driver.checkout), env=env, text=True,
                capture_output=True, check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            return result.stdout

        first = cargo_test()
        self.assertIn("VERSION=1;DIR=" + str(self.driver.checkout), first)
        (self.repo / "src" / "lib.rs").write_text(rust_source(2), encoding="utf-8")
        run(["git", "add", "src/lib.rs"], self.repo)
        run(["git", "commit", "-qm", "fixture-v2"], self.repo)
        self.prepare()
        second = cargo_test()
        self.assertIn("VERSION=2;DIR=" + str(self.driver.checkout), second)
        self.assertNotIn("VERSION=1;DIR=", second)

    def test_dirty_managed_checkout_is_preserved_and_rejected(self) -> None:
        self.prepare()
        unrelated = self.driver.checkout / "do-not-delete.txt"
        unrelated.write_text("user data", encoding="utf-8")
        with self.assertRaisesRegex(QualificationError, "managed checkout is dirty"):
            self.prepare()
        self.assertEqual(unrelated.read_text(encoding="utf-8"), "user data")

    def test_post_run_dirty_reports_observed_checkout_state(self) -> None:
        self.prepare()
        self.assertIs(self.driver.post_run_dirty(), False)
        (self.repo / "concurrent-writer.txt").write_text("dirty\n", encoding="utf-8")
        self.assertIs(self.driver.post_run_dirty(), True)

    def test_assume_unchanged_cannot_hide_changed_head_bytes(self) -> None:
        source = self.repo / "src/lib.rs"
        run(["git", "update-index", "--assume-unchanged", "src/lib.rs"], self.repo)
        source.write_text(rust_source(2), encoding="utf-8")
        self.assertIn("HEAD byte/mode drift", source_status(self.repo))
        with self.assertRaisesRegex(QualificationError, "caller checkout is dirty"):
            self.driver.run()

    def test_assume_unchanged_cannot_hide_managed_checkout_edit(self) -> None:
        self.prepare()
        run(["git", "update-index", "--assume-unchanged", "src/lib.rs"], self.driver.checkout)
        (self.driver.checkout / "src/lib.rs").write_text(rust_source(2), encoding="utf-8")
        with self.assertRaisesRegex(QualificationError, "managed checkout is dirty"):
            self.driver.validate_checkout()

    def test_foreign_marker_and_redirected_target_are_rejected(self) -> None:
        self.prepare()
        marker = json.loads(self.driver.marker.read_text(encoding="utf-8"))
        marker["repo_common_dir"] = "/not-this-repository"
        self.driver.marker.write_text(json.dumps(marker), encoding="utf-8")
        with self.assertRaisesRegex(QualificationError, "marker does not match"):
            self.prepare()
        self.driver.marker.write_text(json.dumps(self.driver.marker_payload()), encoding="utf-8")
        target = self.driver.checkout / "target"
        target.unlink()
        target.symlink_to(self.repo / "elsewhere", target_is_directory=True)
        with self.assertRaisesRegex(QualificationError, "does not point"):
            self.prepare()

    def test_concurrent_lock_fails_before_checkout_mutation(self) -> None:
        with self.driver.lock_path.open("a+") as lock:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaisesRegex(QualificationError, "another local qualification"):
                self.driver.run()
        self.assertFalse(self.driver.checkout.exists())

    def test_redirected_repository_target_is_rejected(self) -> None:
        self.driver.root.rmdir()
        target_root = self.repo / "target"
        target_root.rmdir()
        outside = Path(self.temp.name) / "outside"
        outside.mkdir()
        target_root.symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(QualificationError, "target root must not be a symlink"):
            self.driver.run()
        self.assertEqual(list(outside.iterdir()), [])

    def test_redirected_ci_evidence_directory_is_rejected(self) -> None:
        self.prepare()
        outside = Path(self.temp.name) / "outside-evidence"
        outside.mkdir()
        (self.driver.build_cache / "ci").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(QualificationError, "missing or redirected"):
            self.driver.ci_evidence_path("nextest-evidence.json")

    def test_run_records_exact_pin_and_reuses_managed_checkout(self) -> None:
        (self.repo / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.81.0"\n', encoding="utf-8"
        )
        (self.repo / "Justfile").write_text("ci:\n    @true\n", encoding="utf-8")
        run(["git", "add", "Justfile", "rust-toolchain.toml"], self.repo)
        run(["git", "commit", "-qm", "fixture-qualifier"], self.repo)
        sha, tree = self.pin()
        with redirect_stdout(io.StringIO()):
            self.assertEqual(self.driver.run(), 0)
        receipt = json.loads(
            (self.driver.root / sha / "receipt.json").read_text(encoding="utf-8")
        )
        self.assertEqual(set(receipt), {
            "sha", "tree", "cmds", "exit", "duration_s", "schema", "tool_versions",
            "source_hashes", "recipe_duration_s", "ci_log_sha256",
            "qualification_run_id", "nextest", "nextest_raw", "advisory_db", "status", "dirty",
        })
        self.assertEqual((receipt["sha"], receipt["tree"], receipt["cmds"], receipt["exit"]),
                         (sha, tree, ["just ci"], 0))
        self.assertEqual(receipt["schema"], 3)
        self.assertEqual(receipt["status"], "complete")
        self.assertIs(receipt["dirty"], False)
        self.assertIn("rustc", receipt["tool_versions"])
        self.assertEqual(receipt["source_hashes"], {})
        self.assertIsNone(receipt["nextest"])
        self.assertIsNone(receipt["nextest_raw"])
        log = (self.driver.root / sha / "ci.log").read_bytes()
        self.assertEqual(receipt["ci_log_sha256"], hashlib.sha256(log).hexdigest())
        with redirect_stdout(io.StringIO()):
            self.assertEqual(self.driver.run(), 0)
        self.assertEqual(git(self.driver.checkout, "rev-parse", "HEAD"), sha)

    def test_ci_timeout_records_nonpassing_receipt_for_exact_source(self) -> None:
        (self.repo / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.81.0"\n', encoding="utf-8"
        )
        (self.repo / "Justfile").write_text("ci:\n    @sleep 2\n", encoding="utf-8")
        run(["git", "add", "Justfile", "rust-toolchain.toml"], self.repo)
        run(["git", "commit", "-qm", "fixture-timeout"], self.repo)
        sha, tree = self.pin()
        with patch("qualify_local.CI_TIMEOUT_S", 0.1), redirect_stdout(io.StringIO()):
            self.assertEqual(self.driver.run(), 2)
        receipt = json.loads(
            (self.driver.root / sha / "receipt.json").read_text(encoding="utf-8")
        )
        self.assertEqual((receipt["sha"], receipt["tree"], receipt["cmds"], receipt["exit"]),
                         (sha, tree, ["just ci"], 2))
        self.assertEqual(git(self.driver.checkout, "rev-parse", "HEAD"), sha)
        self.assertEqual(git(self.repo, "status", "--porcelain"), "")

    def test_ci_spawn_failure_records_complete_nonpassing_receipt(self) -> None:
        (self.repo / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.81.0"\n', encoding="utf-8"
        )
        (self.repo / "Justfile").write_text("ci:\n    @true\n", encoding="utf-8")
        run(["git", "add", "Justfile", "rust-toolchain.toml"], self.repo)
        run(["git", "commit", "-qm", "fixture-spawn-failure"], self.repo)
        sha, tree = self.pin()
        original = qualify_local_module.run_bounded

        def fail_ci(args, **kwargs):
            if tuple(args) == ("just", "ci"):
                raise OSError("cannot spawn just")
            return original(args, **kwargs)

        with patch("qualify_local.run_bounded", side_effect=fail_ci), \
             redirect_stdout(io.StringIO()):
            self.assertEqual(self.driver.run(), 2)
        receipt = json.loads(
            (self.driver.root / sha / "receipt.json").read_text(encoding="utf-8")
        )
        self.assertEqual((receipt["sha"], receipt["tree"], receipt["exit"]), (sha, tree, 2))
        self.assertEqual(receipt["status"], "complete")
        self.assertIs(receipt["dirty"], False)
        self.assertTrue((self.driver.root / sha / "ci.log").is_file())

    def test_ci_log_symlink_is_replaced_without_writing_external_file(self) -> None:
        (self.repo / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.81.0"\n', encoding="utf-8"
        )
        (self.repo / "Justfile").write_text("ci:\n    @true\n", encoding="utf-8")
        run(["git", "add", "Justfile", "rust-toolchain.toml"], self.repo)
        run(["git", "commit", "-qm", "fixture-log-custody"], self.repo)
        sha, _ = self.pin()
        with redirect_stdout(io.StringIO()):
            self.assertEqual(self.driver.run(), 0)
        outside = Path(self.temp.name) / "outside-log"
        outside.write_text("keep", encoding="utf-8")
        log_path = self.driver.root / sha / "ci.log"
        log_path.unlink()
        log_path.symlink_to(outside)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(self.driver.run(), 0)
        self.assertFalse(log_path.is_symlink())
        self.assertEqual(outside.read_text(encoding="utf-8"), "keep")


if __name__ == "__main__":
    unittest.main()
