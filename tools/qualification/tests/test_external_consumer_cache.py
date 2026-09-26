"""Custody and semantic-contract checks for the external-consumer gate."""

import json
import io
import os
import subprocess
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from unittest import mock

from tools import test_frontend_external_consumer as consumer_gate
from tools.test_frontend_external_consumer import (
    FIXTURE_DIR,
    FIXTURE_LOCKFILE,
    GLOB_RE,
    MODULE_RE,
    REPOSITORY_ROOT,
    InfrastructureError,
    exact_path_dependency,
    lock_cache,
    manifest_dependencies,
    materialize_fixture,
    materialize_negative_bins,
    negative_observations,
    offline_dependency_preflight,
    consume_preflight_stamp,
    positive_executable,
    positive_outcome,
    public_api_roots,
    public_api_inventory,
    write_preflight_stamp,
)


class ExternalConsumerCacheTest(unittest.TestCase):
    def setUp(self) -> None:
        # These tests may run under the repository's real QGLANG_CI_RUN_ID.
        # Never let a mocked preflight overwrite the same-run production stamp.
        self.stamp_directory = tempfile.TemporaryDirectory(
            prefix="qglang-external-preflight-test-"
        )
        self.addCleanup(self.stamp_directory.cleanup)
        evidence_root = Path(self.stamp_directory.name) / "target"
        stamp = evidence_root / "ci/external-preflight.json"
        stamp_patch = mock.patch.object(consumer_gate, "PREFLIGHT_STAMP", stamp)
        root_patch = mock.patch.object(consumer_gate, "CI_EVIDENCE_ROOT", evidence_root)
        stamp_patch.start()
        root_patch.start()
        self.addCleanup(stamp_patch.stop)
        self.addCleanup(root_patch.stop)

    def test_preflight_inventory_covers_root_fixture_and_all_workspace_members(self):
        paths = consumer_gate.preflight_manifests()
        self.assertEqual(len(paths), 16)
        self.assertIn(REPOSITORY_ROOT / "Cargo.toml", paths)
        self.assertIn(FIXTURE_DIR / "Cargo.toml", paths)
        self.assertEqual(len([path for path in paths if path.parent.parent == REPOSITORY_ROOT / "crates"]), 14)

    def test_negative_cargo_json_is_bound_to_exact_probe_source(self):
        name = "d01_read_diagnostic_stage"
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory)
            source = fixture / "src" / "bin" / (name + ".rs")
            source.parent.mkdir(parents=True)
            source.write_text("fn main() {}\n", encoding="utf-8")
            event = {
                "reason": "compiler-message",
                "target": {"name": name, "src_path": str(source)},
                "message": {
                    "level": "error",
                    "code": {"code": "E0616"},
                    "message": "private field",
                    "rendered": "error[E0616]: field `stage` is private",
                    "spans": [{"file_name": str(source), "is_primary": True}],
                },
            }
            result = subprocess.CompletedProcess(
                ["cargo", "check"], 101, json.dumps(event), "cargo summary"
            )
            observation = negative_observations(result, fixture)[name]
            self.assertTrue(observation["attempted"])
            self.assertFalse(observation["compiled"])
            self.assertEqual(observation["codes"], {"E0616"})
            self.assertIn("field `stage` is private", observation["rendered"][0])

            artifact = subprocess.CompletedProcess(
                ["cargo", "check"], 0,
                json.dumps({
                    "reason": "compiler-artifact",
                    "target": {"name": name, "src_path": str(source)},
                }),
                "",
            )
            self.assertTrue(negative_observations(artifact, fixture)[name]["compiled"])

            event["target"]["src_path"] = str(fixture / "src" / "main.rs")
            bad = subprocess.CompletedProcess(
                ["cargo", "check"], 101, json.dumps(event), "cargo summary"
            )
            with self.assertRaisesRegex(InfrastructureError, "wrong source path"):
                negative_observations(bad, fixture)

            event["target"]["src_path"] = str(source)
            event["message"]["spans"][0]["file_name"] = str(fixture / "src" / "main.rs")
            unanchored = subprocess.CompletedProcess(
                ["cargo", "check"], 101, json.dumps(event), "cargo summary"
            )
            with self.assertRaisesRegex(InfrastructureError, "not anchored to its probe source"):
                negative_observations(unanchored, fixture)

    def test_negative_bins_are_stable_and_reject_foreign_cache_entries(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                fixture = materialize_fixture(root)
                bin_dir = materialize_negative_bins(fixture)
                paths = sorted(bin_dir.glob("*.rs"))
                self.assertEqual(
                    {path.name for path in paths},
                    {name + ".rs" for name in consumer_gate.NEGATIVE_CASES},
                    "negative bin cache must expose the registered probe inventory",
                )
                before = {path: path.stat().st_mtime_ns for path in paths}
                materialize_negative_bins(fixture)
                self.assertEqual({path: path.stat().st_mtime_ns for path in paths}, before)
                (bin_dir / "foreign.rs").write_text("fn main() {}\n", encoding="utf-8")
                with self.assertRaisesRegex(InfrastructureError, "unexpected paths"):
                    materialize_negative_bins(fixture)

    def test_preflight_stamp_is_run_bound_hash_bound_and_single_use(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence_root = Path(directory) / "target"
            stamp = evidence_root / "ci/external-preflight.json"
            with mock.patch.object(consumer_gate, "PREFLIGHT_STAMP", stamp), \
                 mock.patch.object(consumer_gate, "CI_EVIDENCE_ROOT", evidence_root), \
                 mock.patch.object(consumer_gate, "preflight_manifests", return_value=[
                     REPOSITORY_ROOT / "Cargo.toml", FIXTURE_DIR / "Cargo.toml",
                     REPOSITORY_ROOT / "crates/quanta-gqlang-v2-modules/Cargo.toml",
                 ]):
                run_id = "a" * 32
                write_preflight_stamp(run_id)
                with self.assertRaisesRegex(InfrastructureError, "does not match"):
                    consume_preflight_stamp("b" * 32)
                self.assertTrue(stamp.exists())
                original_hash = consumer_gate.sha256_of
                with mock.patch.object(consumer_gate, "sha256_of", side_effect=lambda path: (
                    "changed" if path == consumer_gate.LOCKFILE else original_hash(path)
                )):
                    with self.assertRaisesRegex(InfrastructureError, "does not match"):
                        consume_preflight_stamp(run_id)
                with mock.patch.object(consumer_gate, "sha256_of", side_effect=lambda path: (
                    "changed" if path == consumer_gate.FIXTURE_LOCKFILE else original_hash(path)
                )):
                    with self.assertRaisesRegex(InfrastructureError, "does not match"):
                        consume_preflight_stamp(run_id)
                original_bytes = Path.read_bytes
                changed_manifest = REPOSITORY_ROOT / "crates/quanta-gqlang-v2-modules/Cargo.toml"
                with mock.patch.object(Path, "read_bytes", autospec=True, side_effect=lambda path: (
                    b"changed manifest" if path == changed_manifest else original_bytes(path)
                )):
                    with self.assertRaisesRegex(InfrastructureError, "does not match"):
                        consume_preflight_stamp(run_id)
                consume_preflight_stamp(run_id)
                self.assertFalse(stamp.exists())
                with self.assertRaisesRegex(InfrastructureError, "missing"):
                    consume_preflight_stamp(run_id)

    def test_offline_preflight_uses_locked_cargo_fetch_without_build(self):
        result = subprocess.CompletedProcess(["cargo", "fetch"], 0, "", "")
        with mock.patch.object(consumer_gate, "run_bounded", return_value=result) as command, \
             mock.patch.object(consumer_gate, "preflight_manifests", return_value=[REPOSITORY_ROOT / "Cargo.toml"]), \
             mock.patch.object(consumer_gate, "git_status_bytes", return_value=b""), \
             mock.patch.object(consumer_gate, "gate_writable_fingerprint", return_value={}):
            self.assertEqual(offline_dependency_preflight(), consumer_gate.EXIT_OK)
        self.assertEqual(command.call_count, 3)
        nightly = consumer_gate.read_lock()["public_api_nightly"]
        for call, manifest, toolchain in zip(
            command.call_args_list,
            (
                FIXTURE_DIR / "Cargo.toml",
                REPOSITORY_ROOT / "Cargo.toml",
                REPOSITORY_ROOT / "Cargo.toml",
            ),
            (
                consumer_gate.repo_toolchain_channel(),
                consumer_gate.repo_toolchain_channel(),
                nightly,
            ),
        ):
            args, kwargs = call
            self.assertEqual(
                args[0],
                ["cargo", "fetch", "--locked", "--offline", "--manifest-path", str(manifest)],
            )
            self.assertEqual(kwargs["cwd"], REPOSITORY_ROOT)
            self.assertEqual(kwargs["timeout_s"], 30)
            self.assertEqual(kwargs["env"]["CARGO_NET_OFFLINE"], "true")
            self.assertEqual(kwargs["env"]["RUSTUP_TOOLCHAIN"], toolchain)

    def test_offline_preflight_distinguishes_unavailable_sources_from_contract_failure(self):
        unavailable = subprocess.CompletedProcess(["cargo", "fetch"], 101, "", "source unavailable")
        with mock.patch.object(consumer_gate, "run_bounded", return_value=unavailable), \
             mock.patch.object(consumer_gate, "preflight_manifests", return_value=[REPOSITORY_ROOT / "Cargo.toml"]), \
             mock.patch.object(consumer_gate, "git_status_bytes", return_value=b""), \
             mock.patch.object(consumer_gate, "gate_writable_fingerprint", return_value={}):
            self.assertEqual(consumer_gate.main(["--preflight"]), consumer_gate.EXIT_INFRASTRUCTURE)

    def test_offline_preflight_catches_workspace_sources_after_fixture_passes(self):
        ready = subprocess.CompletedProcess(["cargo", "fetch"], 0, "", "")
        unavailable = subprocess.CompletedProcess(["cargo", "fetch"], 101, "", "workspace source unavailable")
        diagnostics = io.StringIO()
        with mock.patch.object(consumer_gate, "run_bounded", side_effect=[ready, unavailable]) as command, \
             mock.patch.object(consumer_gate, "preflight_manifests", return_value=[REPOSITORY_ROOT / "Cargo.toml"]), \
             mock.patch.object(consumer_gate, "git_status_bytes", return_value=b""), \
             mock.patch.object(consumer_gate, "gate_writable_fingerprint", return_value={}), \
             redirect_stderr(diagnostics):
            self.assertEqual(consumer_gate.main(["--preflight"]), consumer_gate.EXIT_INFRASTRUCTURE)
        self.assertEqual(command.call_count, 2)
        self.assertIn("locked workspace dependency graph", diagnostics.getvalue())
        self.assertIn("workspace source unavailable", diagnostics.getvalue())

    def test_offline_preflight_rejects_checkout_mutation_even_when_cargo_succeeds(self):
        result = subprocess.CompletedProcess(["cargo", "fetch"], 0, "", "")
        with mock.patch.object(consumer_gate, "run_bounded", return_value=result), \
             mock.patch.object(consumer_gate, "preflight_manifests", return_value=[REPOSITORY_ROOT / "Cargo.toml"]), \
             mock.patch.object(consumer_gate, "git_status_bytes", side_effect=[b"", b" M Cargo.lock\n"]), \
             mock.patch.object(consumer_gate, "gate_writable_fingerprint", return_value={}):
            self.assertEqual(offline_dependency_preflight(), consumer_gate.EXIT_INFRASTRUCTURE)

    def test_positive_artifact_uses_cargo_emitted_exact_binary_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / "fixture"
            target = root / "build"
            (fixture / "src").mkdir(parents=True)
            (fixture / "src" / "main.rs").write_text("fn main() {}\n")
            binary = target / "custom-target" / "debug" / "consumer"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"built artifact")
            artifact = {
                "reason": "compiler-artifact",
                "target": {
                    "name": "quanta-gqlang-v2-frontend-external-consumer",
                    "kind": ["bin"],
                    "src_path": str(fixture / "src" / "main.rs"),
                },
                "executable": str(binary.resolve()),
            }
            unrelated = dict(artifact, target=dict(artifact["target"], name="dependency"))
            build = subprocess.CompletedProcess(
                ["cargo", "build"], 0, json.dumps(unrelated) + "\n" + json.dumps(artifact), ""
            )
            self.assertEqual(positive_executable(build, fixture, target), binary.resolve())

            physical_target = root / "managed-build-cache"
            physical_target.mkdir()
            linked_target = root / "managed-checkout-target"
            linked_target.symlink_to(physical_target, target_is_directory=True)
            linked_cache = linked_target / "external-consumer"
            linked_binary = linked_cache / "debug" / "consumer"
            linked_binary.parent.mkdir(parents=True)
            linked_binary.write_bytes(b"managed artifact")
            linked_artifact = dict(artifact, executable=str(linked_binary))
            linked_build = subprocess.CompletedProcess(
                ["cargo", "build"], 0, json.dumps(linked_artifact), ""
            )
            self.assertEqual(
                positive_executable(linked_build, fixture, linked_cache),
                linked_binary.resolve(),
            )

            foreign = root / "foreign" / "consumer"
            foreign.parent.mkdir()
            foreign.write_bytes(b"stale artifact")
            relative = dict(artifact, executable="debug/consumer")
            missing = dict(artifact, executable=str(target / "debug" / "missing"))
            redirected = target / "debug" / "redirected"
            redirected.parent.mkdir(exist_ok=True)
            redirected.symlink_to(binary)
            symlink = dict(artifact, executable=str(redirected))
            wrong_source = dict(
                artifact,
                target=dict(artifact["target"], src_path=str(fixture / "src" / "other.rs")),
            )
            without_executable = dict(artifact, executable=None)
            cases = {
                "absent": (json.dumps(unrelated), "0 fixture binary artifacts"),
                "duplicate": (
                    json.dumps(artifact) + "\n" + json.dumps(artifact),
                    "2 fixture binary artifacts",
                ),
                "foreign": (
                    json.dumps(dict(artifact, executable=str(foreign))),
                    "outside the owned build cache",
                ),
                "relative": (json.dumps(relative), "relative fixture binary path"),
                "missing": (json.dumps(missing), "missing or redirected"),
                "symlink": (json.dumps(symlink), "missing or redirected"),
                "wrong-source": (json.dumps(wrong_source), "wrong source path"),
                "no-executable": (
                    json.dumps(without_executable),
                    "without an executable path",
                ),
                "invalid-json": ("not-json", "invalid JSON"),
            }
            for name, (stdout, expected) in cases.items():
                with self.subTest(name=name):
                    rejected = subprocess.CompletedProcess(
                        ["cargo", "build"], 0, stdout, ""
                    )
                    with self.assertRaisesRegex(InfrastructureError, expected):
                        positive_executable(rejected, fixture, target)

    def test_cargo_manifest_detects_every_dependency_form(self):
        extras = [
            '[dependencies.extra]\nversion = "1"\n',
            '[build-dependencies.extra]\nversion = "1"\n',
            '[dev-dependencies.extra]\nversion = "1"\n',
            '[target.\'cfg(unix)\'.dependencies]\nextra = "1"\n',
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            facade = root / "facade"
            manifest = root / "Cargo.toml"
            (root / "src").mkdir()
            (root / "src" / "main.rs").write_text("fn main() {}\n")
            baseline = (
                '[package]\nname = "external-probe"\nversion = "0.1.0"\n'
                'edition = "2021"\n[dependencies]\n'
                'quanta-gqlang-v2-frontend-contract = { path = "facade" }\n'
            )
            manifest.write_text(baseline)
            self.assertTrue(
                exact_path_dependency(
                    manifest_dependencies(manifest),
                    "quanta-gqlang-v2-frontend-contract",
                    facade,
                )
            )
            for extra in extras:
                with self.subTest(extra=extra):
                    manifest.write_text(baseline + extra)
                    self.assertFalse(
                        exact_path_dependency(
                            manifest_dependencies(manifest),
                            "quanta-gqlang-v2-frontend-contract",
                            facade,
                        )
                    )

    def test_positive_panic_is_a_contract_violation(self):
        built = subprocess.CompletedProcess(["cargo", "build"], 0, "", "")
        panicked = subprocess.CompletedProcess(["consumer"], 101, "", "thread 'main' panicked")
        self.assertEqual(
            positive_outcome(built, panicked),
            ("violation", "positive consumer program failed during execution"),
        )

    @staticmethod
    def compile_rust_contract(source: str) -> subprocess.CompletedProcess:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_path = root / "probe.rs"
            output_path = root / "probe.rlib"
            source_path.write_text(source)
            return subprocess.run(
                [
                    "rustc",
                    "--crate-name",
                    "semantic_contract_probe",
                    "--crate-type",
                    "lib",
                    "--edition",
                    "2021",
                    "-o",
                    str(output_path),
                    str(source_path),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

    def test_public_api_parser_accepts_only_curated_root_reexports(self):
        output = "\n".join(
            [
                "pub mod quanta_gqlang_v2_frontend_contract",
                "pub use quanta_gqlang_v2_frontend_contract::AnalysisDocumentV2",
                "pub use quanta_gqlang_v2_frontend_contract::compile_source_document_v3",
            ]
        )
        roots, direct = public_api_roots(output)
        self.assertEqual(roots, ["AnalysisDocumentV2", "compile_source_document_v3"])
        self.assertEqual(direct, [])

    def test_public_api_parser_rejects_direct_public_item_mutations(self):
        prefix = "pub mod quanta_gqlang_v2_frontend_contract\n"
        for mutation in [
            "pub fn rogue()",
            "pub struct Rogue",
            "pub const ROGUE: u8",
            "pub mod leak",
        ]:
            roots, direct = public_api_roots(prefix + mutation + "\n")
            self.assertEqual(roots, [], mutation)
            self.assertEqual(direct, [mutation], mutation)

    def test_duplicate_public_root_is_an_extra_exposure(self):
        self.assertEqual(
            public_api_inventory(["FrontendBudgetV2", "FrontendBudgetV2"], ["FrontendBudgetV2"]),
            ([], [], ["FrontendBudgetV2"]),
        )

    def test_source_shape_rejects_indented_glob_and_module_mutations(self):
        source = "  pub use owner::*;\n\tpub mod leak;\n"
        self.assertEqual(len(GLOB_RE.findall(source)), 1)
        self.assertEqual(len(MODULE_RE.findall(source)), 1)

    def test_rust_semantic_probe_kills_method_variant_and_return_mutations(self):
        baseline = """
pub struct Document;
impl Document {
    pub fn count(&self) -> u32 { 0 }
}
pub enum Mode { A, B }
pub fn method_signature(document: &Document) -> u32 {
    let method: fn(&Document) -> u32 = Document::count;
    method(document)
}
pub fn exhaustive_variant(mode: Mode) {
    match mode { Mode::A | Mode::B => () }
}
"""
        baseline_result = self.compile_rust_contract(baseline)
        self.assertEqual(baseline_result.returncode, 0, baseline_result.stderr)

        mutations = {
            "method removed": baseline.replace("pub fn count(&self) -> u32 { 0 }", ""),
            "return type drifted": baseline.replace(
                "pub fn count(&self) -> u32 { 0 }",
                "pub fn count(&self) -> u64 { 0 }",
            ),
            "variant removed": baseline.replace("pub enum Mode { A, B }", "pub enum Mode { A }"),
            "variant added": baseline.replace("pub enum Mode { A, B }", "pub enum Mode { A, B, C }"),
        }
        for name, source in mutations.items():
            with self.subTest(name=name):
                result = self.compile_rust_contract(source)
                self.assertNotEqual(result.returncode, 0, result.stderr)

    def test_rust_denies_a_curated_root_that_is_only_imported(self):
        source = """
#![deny(unused_imports)]
mod facade { pub struct Used; pub struct ImportOnly; }
use facade::{ImportOnly, Used};
pub fn semantic_receipt(_: Used) {}
"""
        result = self.compile_rust_contract(source)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertIn("unused import: `ImportOnly`", result.stderr)

    def test_replay_restores_source_at_the_same_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                fixture = materialize_fixture(root)
                files = [
                    fixture / "Cargo.toml",
                    fixture / "Cargo.lock",
                    fixture / "src" / "main.rs",
                ]
                stable_mtime = 1_700_000_000_000_000_000
                for path in files:
                    os.utime(path, ns=(stable_mtime, stable_mtime))
                self.assertEqual(materialize_fixture(root), fixture)
                self.assertTrue(all(path.stat().st_mtime_ns == stable_mtime for path in files))
                expected_contents = {path: path.read_bytes() for path in files}
                (fixture / "Cargo.toml").write_text("[package]\nname = 'tampered'\n")
                (fixture / "Cargo.lock").write_text("tampered\n")
                (fixture / "src" / "main.rs").write_text("fn main() { panic!() }\n")
            with lock_cache(root, REPOSITORY_ROOT):
                replay = materialize_fixture(root)
                self.assertEqual(replay, fixture)
                self.assertEqual({path: path.read_bytes() for path in files}, expected_contents)
                self.assertEqual(
                    (replay / "src" / "main.rs").read_bytes(),
                    (FIXTURE_DIR / "src" / "main.rs").read_bytes(),
                )
                self.assertEqual(
                    (replay / "Cargo.lock").read_bytes(),
                    FIXTURE_LOCKFILE.read_bytes(),
                )

    def test_unowned_or_foreign_cache_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            root.mkdir()
            (root / "foreign").write_text("do not adopt")
            with self.assertRaisesRegex(InfrastructureError, "unowned"):
                lock_cache(root, REPOSITORY_ROOT)
            (root / "foreign").unlink()
            with lock_cache(root, REPOSITORY_ROOT):
                pass
            (root / ".owner").write_text("other checkout\n")
            with self.assertRaisesRegex(InfrastructureError, "another checkout"):
                lock_cache(root, REPOSITORY_ROOT)

    def test_second_runner_cannot_interleave_privacy_probes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                with self.assertRaisesRegex(InfrastructureError, "another external-consumer gate"):
                    lock_cache(root, REPOSITORY_ROOT)

    def test_redirected_build_cache_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                pass
            (root / "build").symlink_to(Path(directory) / "foreign-build")
            with self.assertRaisesRegex(InfrastructureError, "build cache is redirected"):
                lock_cache(root, REPOSITORY_ROOT)


if __name__ == "__main__":
    unittest.main()
