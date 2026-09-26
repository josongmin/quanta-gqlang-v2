#!/usr/bin/env python3
from __future__ import annotations

import json
import io
import subprocess
import tempfile
import unittest
from collections import defaultdict
from contextlib import redirect_stderr
import importlib.util
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[3]
CHECKER = ROOT / "tools/architecture/check_architecture.py"
FIXTURES = ROOT / "tools/architecture/tests/fixtures"

_checker_spec = importlib.util.spec_from_file_location("architecture_checker", CHECKER)
assert _checker_spec and _checker_spec.loader
architecture_checker = importlib.util.module_from_spec(_checker_spec)
_checker_spec.loader.exec_module(architecture_checker)
MEMBERS = sorted(architecture_checker.EXPECTED_WORKSPACE_MEMBERS)

# The thirteen named mutations. Each must make the checker exit 1. The entry
# is (mutation name, fixture name, omit member, required stderr substrings).
NAMED_MUTATIONS: tuple[tuple[str, str, str | None, tuple[str, ...]], ...] = (
    (
        "illegal-normal-edge",
        "reverse-edge",
        None,
        ("illegal normal internal edges", "quanta-gqlang-v2-sema -> quanta-gqlang-v2-planning"),
    ),
    (
        "illegal-dev-edge",
        "dev-edge",
        None,
        ("illegal dev internal edges", "quanta-gqlang-v2-surface -> quanta-gqlang-v2-hir"),
    ),
    (
        "illegal-build-edge",
        "build-edge",
        None,
        ("illegal build internal edges", "quanta-gqlang-v2-surface -> quanta-gqlang-v2-hir"),
    ),
    (
        "forbidden-runtime-dep",
        "runtime-edge",
        None,
        ("forbidden normal dependency quanta-runtime",),
    ),
    (
        "member-drift",
        "valid",
        "quanta-gqlang-v2-types",
        ("workspace members drifted",),
    ),
    (
        "undeclared-external-dep",
        "external-drift",
        None,
        ("undeclared external normal dependency regex",),
    ),
    (
        "custom-build-and-build-deny",
        "custom-build",
        None,
        ("custom build script", "default-deny"),
    ),
    (
        "lints-drift",
        "lints-drift",
        None,
        ("[lints] workspace = true",),
    ),
    (
        "dep-attribute-drift",
        "dep-attr-drift",
        None,
        (
            "rename",
            "optional",
            "target",
            "feature drift on internal normal dependency",
            "workspace-external path",
            "default-features",
        ),
    ),
    (
        "external-version-drift",
        "external-version-drift",
        None,
        ("version drift on external normal dependency", "sha2"),
    ),
    (
        "external-feature-drift",
        "external-feature-drift",
        None,
        ("feature drift on external normal dependency", "serde"),
    ),
    (
        "internal-version-drift",
        "internal-version-drift",
        None,
        ("version drift on internal normal dependency", "quanta-gqlang-v2-sema"),
    ),
    (
        "legacy-crate-dep",
        "legacy-edge",
        None,
        ("forbidden legacy normal dependency", "quanta-gqlang-v1-legacy"),
    ),
)


def final_topology() -> dict:
    """Materialize the checker's one authoritative final DAG for mutation tests."""
    return {
        "normal_edges": [
            list(edge) for edge in sorted(architecture_checker.FINAL_NORMAL_EDGES)
        ],
        "dev_edges": [list(edge) for edge in sorted(architecture_checker.FINAL_DEV_EDGES)],
        "build_edges": [
            list(edge) for edge in sorted(architecture_checker.FINAL_BUILD_EDGES)
        ],
    }


def read_topology(path: Path) -> dict:
    topology = json.loads(path.read_text(encoding="utf-8"))
    if "base" not in topology:
        return topology
    base_name = topology["base"]
    base = (
        final_topology()
        if base_name == "final"
        else read_topology((path.parent / base_name).resolve())
    )
    merged = {
        "normal_edges": base["normal_edges"] + topology.get("add_normal_edges", []),
        "dev_edges": base["dev_edges"] + topology.get("add_dev_edges", []),
        "build_edges": base.get("build_edges", []) + topology.get("add_build_edges", []),
        "runtime_edge": topology.get("runtime_edge"),
    }
    for key in (
        "add_external_deps",
        "drop_lints",
        "add_build_rs",
        "add_build_deps",
        "rename_dep",
        "make_optional",
        "add_target_section",
        "add_internal_features",
        "external_path_dep",
        "legacy_edge",
        "default_features_off",
        "internal_version",
    ):
        if key in topology:
            merged[key] = topology[key]
    return merged


def internal_dep_line(consumer: str, kind: str, dependency: str, topology: dict) -> str:
    optional = [list(item) for item in topology.get("make_optional", [])]
    features = {
        (item[0], item[1], item[2]): item[3]
        for item in topology.get("add_internal_features", [])
    }
    default_off = {
        tuple(item) for item in topology.get("default_features_off", [])
    }
    versions = {
        (item[0], item[1], item[2]): item[3]
        for item in topology.get("internal_version", [])
    }
    extra = ""
    if (consumer, kind, dependency) in {tuple(item) for item in optional}:
        extra += ", optional = true"
    if (consumer, kind, dependency) in features:
        rendered = ", ".join(f'"{feature}"' for feature in features[(consumer, kind, dependency)])
        extra += f", features = [{rendered}]"
    if (consumer, kind, dependency) in default_off:
        extra += ", default-features = false"
    if (consumer, kind, dependency) in versions:
        extra += f', version = "{versions[(consumer, kind, dependency)]}"'
    if extra:
        return f'{dependency} = {{ path = "../{dependency}"{extra} }}'
    return f'{dependency} = {{ path = "../{dependency}" }}'


def external_dep_line(name: str, version: str, features: list[str] | None) -> str:
    if features is None:
        return f'{name} = "{version}"'
    rendered = ", ".join(f'"{feature}"' for feature in features)
    return f'{name} = {{ version = "{version}", features = [{rendered}] }}'


def materialize_fixture(destination: Path, fixture_name: str, *, omit: str | None = None) -> None:
    destination = destination.resolve()
    topology = (
        final_topology()
        if fixture_name == "valid"
        else read_topology(FIXTURES / fixture_name / "topology.json")
    )
    members = [member for member in MEMBERS if member != omit]
    member_entries = ",\n".join(f'    "crates/{member}"' for member in members)
    excluded = ['"external/quanta-runtime"', '"external/quanta-gqlang-v1-legacy"']
    if omit is not None:
        excluded.append(f'"crates/{omit}"')
    exclude_entries = ", ".join(excluded)
    (destination / "Cargo.toml").write_text(
        f'[workspace]\nmembers = [\n{member_entries},\n]\n'
        f"exclude = [{exclude_entries}]\nresolver = \"2\"\n"
        "[workspace.lints.rust]\nunsafe_code = \"forbid\"\n",
        encoding="utf-8",
    )
    shared = destination / "shared/lib.rs"
    shared.parent.mkdir(parents=True)
    shared.write_text("#![forbid(unsafe_code)]\n", encoding="utf-8")

    normal_by_consumer: dict[str, list[str]] = defaultdict(list)
    dev_by_consumer: dict[str, list[str]] = defaultdict(list)
    build_by_consumer: dict[str, list[str]] = defaultdict(list)
    for consumer, dependency in topology["normal_edges"]:
        normal_by_consumer[consumer].append(dependency)
    for consumer, dependency in topology["dev_edges"]:
        dev_by_consumer[consumer].append(dependency)
    for consumer, dependency in topology.get("build_edges", []):
        build_by_consumer[consumer].append(dependency)
    runtime_edge = topology.get("runtime_edge")
    if runtime_edge:
        normal_by_consumer[runtime_edge[0]].append(runtime_edge[1])
        runtime_dir = destination / "external/quanta-runtime"
        runtime_dir.mkdir(parents=True)
        (runtime_dir / "Cargo.toml").write_text(
            '[package]\nname = "quanta-runtime"\nversion = "0.1.0"\nedition = "2021"\n'
            '[lib]\npath = "../../shared/lib.rs"\n',
            encoding="utf-8",
        )
    legacy_edge = topology.get("legacy_edge")
    if legacy_edge:
        normal_by_consumer[legacy_edge[0]].append(legacy_edge[1])
        legacy_dir = destination / f"external/{legacy_edge[1]}"
        legacy_dir.mkdir(parents=True)
        (legacy_dir / "Cargo.toml").write_text(
            f'[package]\nname = "{legacy_edge[1]}"\nversion = "0.1.0"\nedition = "2021"\n'
            '[lib]\npath = "../../shared/lib.rs"\n',
            encoding="utf-8",
        )

    external_by_consumer: dict[
        tuple[str, str], list[tuple[str, str, list[str] | None]]
    ] = defaultdict(list)
    for entry in topology.get("add_external_deps", []):
        consumer, kind, name, version = entry[:4]
        features = list(entry[4]) if len(entry) > 4 else None
        external_by_consumer[(consumer, kind)].append((name, version, features))

    rename = topology.get("rename_dep")
    target_sections: dict[str, list[str]] = defaultdict(list)
    for consumer, dependency in topology.get("add_target_section", []):
        target_sections[consumer].append(dependency)

    evil_crates: dict[str, Path] = {}
    for consumer, kind, name in topology.get("external_path_dep", []):
        evil_dir = destination.parent / f"evil-{name}"
        evil_dir.mkdir(parents=True, exist_ok=True)
        (evil_dir / "Cargo.toml").write_text(
            f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n'
            '[lib]\npath = "lib.rs"\n',
            encoding="utf-8",
        )
        (evil_dir / "lib.rs").write_text("#![forbid(unsafe_code)]\n", encoding="utf-8")
        evil_crates[name] = evil_dir

    drop_lints = set(topology.get("drop_lints", []))
    build_rs_members = set(topology.get("add_build_rs", []))
    build_deps: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for consumer, name, version in topology.get("add_build_deps", []):
        build_deps[consumer].append((name, version))

    for member in MEMBERS:
        crate = destination / "crates" / member
        crate.mkdir(parents=True)
        lines = [
            "[package]",
            f'name = "{member}"',
            'version = "0.1.0"',
            'edition = "2021"',
            "",
            "[lib]",
            'path = "../../shared/lib.rs"',
        ]
        dependencies = normal_by_consumer[member]
        if member in target_sections:
            dependencies = [dep for dep in dependencies if dep not in target_sections[member]]
        if dependencies or external_by_consumer.get((member, "normal")) or (
            rename and rename[0] == member and rename[1] == "normal"
        ):
            lines.extend(["", "[dependencies]"])
            for dependency in sorted(dependencies):
                if dependency in ("quanta-runtime", "quanta-gqlang-v1-legacy"):
                    lines.append(f'{dependency} = {{ path = "../../external/{dependency}" }}')
                else:
                    lines.append(internal_dep_line(member, "normal", dependency, topology))
            for name, version, features in sorted(external_by_consumer.get((member, "normal"), [])):
                lines.append(external_dep_line(name, version, features))
            if rename and rename[0] == member and rename[1] == "normal":
                _, _, alias, package, version = rename
                lines.append(f'{alias} = {{ package = "{package}", version = "{version}" }}')
            for consumer, kind, name in topology.get("external_path_dep", []):
                if consumer == member and kind == "normal":
                    # Emit a path that escapes the workspace root.
                    escape = Path(*([".."] * (len(crate.resolve().relative_to(destination).parts) + 1)))
                    lines.append(f'{name} = {{ path = "{escape / evil_crates[name].name}" }}')
        dev_dependencies = dev_by_consumer[member]
        if dev_dependencies or external_by_consumer.get((member, "dev")):
            lines.extend(["", "[dev-dependencies]"])
            for dependency in sorted(dev_dependencies):
                lines.append(internal_dep_line(member, "dev", dependency, topology))
            for name, version, features in sorted(external_by_consumer.get((member, "dev"), [])):
                lines.append(external_dep_line(name, version, features))
        internal_build_deps = build_by_consumer[member]
        member_build_deps = sorted(build_deps.get(member, []))
        if internal_build_deps or member_build_deps:
            lines.extend(["", "[build-dependencies]"])
            for dependency in sorted(internal_build_deps):
                lines.append(internal_dep_line(member, "build", dependency, topology))
            for name, version in member_build_deps:
                lines.append(f'{name} = "{version}"')
        if member in target_sections:
            lines.extend(["", "[target.'cfg(unix)'.dependencies]"])
            for dependency in sorted(target_sections[member]):
                lines.append(f'{dependency} = {{ path = "../{dependency}" }}')
        if member not in drop_lints:
            lines.extend(["", "[lints]", "workspace = true"])
        (crate / "Cargo.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")
        if member in build_rs_members:
            (crate / "build.rs").write_text('fn main() {}\n', encoding="utf-8")


def run_checker(root: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(CHECKER), "--root", str(root), *extra],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def fixture_metadata(root: Path) -> dict:
    """Project the deliberately simple mutation manifests without invoking Cargo."""
    with (root / "Cargo.toml").open("rb") as handle:
        workspace = architecture_checker.tomllib.load(handle)
    member_paths = [root / path for path in workspace["workspace"]["members"]]
    packages = []
    workspace_members = []
    for crate_dir in member_paths:
        manifest_path = crate_dir / "Cargo.toml"
        with manifest_path.open("rb") as handle:
            manifest = architecture_checker.tomllib.load(handle)
        package = manifest["package"]
        package_id = str(manifest_path.resolve())
        workspace_members.append(package_id)
        dependencies = []

        def add_dependencies(table: dict, kind: str | None, target: str | None = None) -> None:
            for alias, raw in table.items():
                spec = {"version": raw} if isinstance(raw, str) else dict(raw)
                name = spec.get("package", alias)
                raw_path = spec.get("path")
                version = spec.get("version", "*")
                dependencies.append({
                    "name": name,
                    "kind": kind,
                    "target": target,
                    "rename": alias if name != alias else None,
                    "optional": spec.get("optional", False),
                    "uses_default_features": spec.get("default-features", True),
                    "features": list(spec.get("features", [])),
                    "req": architecture_checker.cargo_normalized_req(version),
                    "path": str((crate_dir / raw_path).resolve()) if raw_path else None,
                    "source": None if raw_path else "registry+fixture",
                })

        for section, kind in architecture_checker.DEP_SECTION_KINDS:
            add_dependencies(manifest.get(section, {}), None if kind == "normal" else kind)
        for target, target_table in manifest.get("target", {}).items():
            for section, kind in architecture_checker.DEP_SECTION_KINDS:
                add_dependencies(
                    target_table.get(section, {}),
                    None if kind == "normal" else kind,
                    target,
                )
        targets = [{"kind": ["lib"], "src_path": str((crate_dir / "../../shared/lib.rs").resolve())}]
        if (crate_dir / "build.rs").is_file() or package.get("build") not in (None, False):
            targets.append({"kind": ["custom-build"], "src_path": str(crate_dir / "build.rs")})
        packages.append({
            "id": package_id,
            "name": package["name"],
            "manifest_path": str(manifest_path.resolve()),
            "dependencies": dependencies,
            "targets": targets,
        })
    return {"workspace_members": workspace_members, "packages": packages}


class ArchitectureCheckerTest(unittest.TestCase):
    def fixture(self, name: str, *, omit: str | None = None) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(prefix="qglang-architecture-")
        root = Path(temporary.name) / "workspace"
        root.mkdir()
        root = root.resolve()
        materialize_fixture(root, name, omit=omit)
        return temporary, root

    def test_valid_final_dag(self) -> None:
        temporary, root = self.fixture("valid")
        with temporary:
            result = run_checker(root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"workspace_members={len(MEMBERS)}", result.stdout)
        self.assertIn(
            f"normal_edges={len(architecture_checker.FINAL_NORMAL_EDGES)}",
            result.stdout,
        )
        self.assertIn(f"dev_edges={len(architecture_checker.FINAL_DEV_EDGES)}", result.stdout)

    def test_valid_final_dag_json_verdict(self) -> None:
        temporary, root = self.fixture("valid")
        with temporary:
            result = run_checker(root, "--format", "json")
        self.assertEqual(result.returncode, 0, result.stderr)
        verdict = json.loads(result.stdout)
        self.assertEqual(verdict["verdict"], "passed")
        self.assertEqual(verdict["workspace_members"], len(MEMBERS))
        self.assertEqual(verdict["errors"], [])

    def test_json_verdict_reports_failure(self) -> None:
        temporary, root = self.fixture("reverse-edge")
        with temporary:
            result = run_checker(root, "--format", "json")
            human = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        verdict = json.loads(result.stdout)
        self.assertEqual(verdict["verdict"], "failed")
        self.assertTrue(verdict["errors"])
        self.assertNotEqual(human.returncode, 0)
        self.assertIn("architecture lint failed", human.stderr)

    def test_named_mutations_all_exit_1(self) -> None:
        self.assertEqual(len(NAMED_MUTATIONS), 13)
        for name, fixture, omit, markers in NAMED_MUTATIONS:
            with self.subTest(mutation=name):
                temporary, root = self.fixture(fixture, omit=omit)
                with temporary:
                    with patch.object(
                        architecture_checker, "cargo_metadata", return_value=fixture_metadata(root)
                    ):
                        _summary, errors = architecture_checker.run_check(root)
                rendered = "\n".join(errors)
                self.assertTrue(errors, f"mutation {name} must fail")
                for marker in markers:
                    self.assertIn(marker, rendered, f"mutation {name} missing {marker!r}")

    def test_retired_transitional_edge_is_illegal(self) -> None:
        temporary, root = self.fixture("transitional-valid")
        with temporary:
            with patch.object(
                architecture_checker, "cargo_metadata", return_value=fixture_metadata(root)
            ):
                _summary, errors = architecture_checker.run_check(root)
        rendered = "\n".join(errors)
        self.assertTrue(errors)
        self.assertIn(
            "quanta-gqlang-v2-algebra -> quanta-gqlang-v2-hir",
            rendered,
        )

    def test_unknown_dependency_kind_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "unknown dependency kind future-kind"):
            architecture_checker.dependency_kind({"kind": "future-kind"})

    def test_internal_manifest_dependency_must_use_exact_member_path(self) -> None:
        temporary, root = self.fixture("valid")
        with temporary:
            root = root.resolve()
            consumer = "quanta-gqlang-v2-cli"
            dependency = "quanta-gqlang-v2-frontend-contract"
            manifest_path = root / "crates" / consumer / "Cargo.toml"
            original = manifest_path.read_text(encoding="utf-8")
            member_dirs = {name: root / "crates" / name for name in MEMBERS}
            for replacement in (
                f'{dependency} = "0.1.0"',
                f'{dependency} = {{ path = "../quanta-gqlang-v2-hir" }}',
            ):
                with self.subTest(replacement=replacement):
                    manifest_path.write_text(
                        original.replace(
                            f'{dependency} = {{ path = "../{dependency}" }}', replacement
                        ),
                        encoding="utf-8",
                    )
                    errors: list[str] = []
                    architecture_checker.check_member_manifest(
                        root, consumer, manifest_path, member_dirs, errors
                    )
                    self.assertTrue(
                        any("internal edges require the exact member path" in error for error in errors),
                        errors,
                    )

    def test_internal_metadata_dependency_rejects_registry_source(self) -> None:
        temporary, root = self.fixture("valid")
        with temporary:
            metadata = fixture_metadata(root)
            workspace_ids = set(metadata["workspace_members"])
            packages = {
                package["name"]: package
                for package in metadata["packages"]
                if package["id"] in workspace_ids
            }
            dependency = next(
                item
                for item in packages["quanta-gqlang-v2-cli"]["dependencies"]
                if item["name"] == "quanta-gqlang-v2-frontend-contract"
            )
            dependency["source"] = "registry+https://github.com/rust-lang/crates.io-index"
            errors: list[str] = []
            architecture_checker.check_metadata_edges(packages, errors)
            self.assertTrue(
                any("internal edges require the member path" in error for error in errors),
                errors,
            )

    def test_cargo_metadata_timeout_fails_closed(self) -> None:
        stderr = io.StringIO()
        with patch.object(
            architecture_checker.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(["cargo", "metadata"], 60),
        ) as runner, redirect_stderr(stderr):
            with self.assertRaises(SystemExit) as failure:
                architecture_checker.cargo_metadata(ROOT)
        self.assertEqual(failure.exception.code, 1)
        self.assertIn("cargo metadata timed out after 60 s", stderr.getvalue())
        self.assertEqual(runner.call_args.kwargs["timeout"], 60)

    def test_cargo_metadata_freezes_workspaces_with_a_lockfile(self) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo", "metadata"], 0, '{"packages": [], "workspace_members": []}', ""
        )
        with tempfile.TemporaryDirectory(prefix="qglang-metadata-lock-") as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            with patch.object(
                architecture_checker.subprocess, "run", return_value=completed
            ) as run:
                architecture_checker.cargo_metadata(root)
                command = run.call_args.args[0]
                self.assertNotIn("--locked", command)
                self.assertNotIn("--offline", command)

                (root / "Cargo.lock").write_text("version = 3\n", encoding="utf-8")
                architecture_checker.cargo_metadata(root)
                command = run.call_args.args[0]
                self.assertIn("--locked", command)
                self.assertIn("--offline", command)


if __name__ == "__main__":
    unittest.main()
