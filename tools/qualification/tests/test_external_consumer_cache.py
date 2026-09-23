"""Custody and semantic-contract checks for the external-consumer gate."""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

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
    positive_executable,
    positive_outcome,
    public_api_roots,
)


class ExternalConsumerCacheTest(unittest.TestCase):
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
                (fixture / "src" / "main.rs").write_text("fn main() { panic!() }\n")
            with lock_cache(root, REPOSITORY_ROOT):
                replay = materialize_fixture(root)
                self.assertEqual(replay, fixture)
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
