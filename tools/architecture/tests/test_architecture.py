#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from collections import defaultdict
import importlib.util
from pathlib import Path


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
            "feature drift",
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


class ArchitectureCheckerTest(unittest.TestCase):
    def fixture(self, name: str, *, omit: str | None = None) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(prefix="qglang-architecture-")
        root = Path(temporary.name) / "workspace"
        root.mkdir()
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
        self.assertNotEqual(result.returncode, 0)
        verdict = json.loads(result.stdout)
        self.assertEqual(verdict["verdict"], "failed")
        self.assertTrue(verdict["errors"])

    def test_named_mutations_all_exit_1(self) -> None:
        self.assertEqual(len(NAMED_MUTATIONS), 13)
        for name, fixture, omit, markers in NAMED_MUTATIONS:
            with self.subTest(mutation=name):
                temporary, root = self.fixture(fixture, omit=omit)
                with temporary:
                    result = run_checker(root)
                self.assertNotEqual(result.returncode, 0, f"mutation {name} must fail")
                for marker in markers:
                    self.assertIn(marker, result.stderr, f"mutation {name} missing {marker!r}")

    def test_retired_transitional_edge_is_illegal(self) -> None:
        temporary, root = self.fixture("transitional-valid")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "quanta-gqlang-v2-algebra -> quanta-gqlang-v2-hir",
            result.stderr,
        )

    def test_unknown_dependency_kind_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "unknown dependency kind future-kind"):
            architecture_checker.dependency_kind({"kind": "future-kind"})


if __name__ == "__main__":
    unittest.main()
