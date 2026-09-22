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
    return {
        "normal_edges": base["normal_edges"] + topology.get("add_normal_edges", []),
        "dev_edges": base["dev_edges"] + topology.get("add_dev_edges", []),
        "build_edges": base.get("build_edges", []) + topology.get("add_build_edges", []),
        "runtime_edge": topology.get("runtime_edge"),
    }


def materialize_fixture(destination: Path, fixture_name: str, *, omit: str | None = None) -> None:
    topology = (
        final_topology()
        if fixture_name == "valid"
        else read_topology(FIXTURES / fixture_name / "topology.json")
    )
    members = [member for member in MEMBERS if member != omit]
    member_entries = ",\n".join(f'    "crates/{member}"' for member in members)
    excluded = ['"external/quanta-runtime"']
    if omit is not None:
        excluded.append(f'"crates/{omit}"')
    exclude_entries = ", ".join(excluded)
    (destination / "Cargo.toml").write_text(
        f'[workspace]\nmembers = [\n{member_entries},\n]\n'
        f"exclude = [{exclude_entries}]\nresolver = \"2\"\n",
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
        if dependencies:
            lines.extend(["", "[dependencies]"])
            for dependency in sorted(dependencies):
                path = (
                    "../../external/quanta-runtime"
                    if dependency == "quanta-runtime"
                    else f"../{dependency}"
                )
                lines.append(f'{dependency} = {{ path = "{path}" }}')
        dev_dependencies = dev_by_consumer[member]
        if dev_dependencies:
            lines.extend(["", "[dev-dependencies]"])
            for dependency in sorted(dev_dependencies):
                lines.append(f'{dependency} = {{ path = "../{dependency}" }}')
        build_dependencies = build_by_consumer[member]
        if build_dependencies:
            lines.extend(["", "[build-dependencies]"])
            for dependency in sorted(build_dependencies):
                lines.append(f'{dependency} = {{ path = "../{dependency}" }}')
        (crate / "Cargo.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_checker(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(CHECKER), "--root", str(root)],
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

    def test_retired_transitional_edge_is_illegal(self) -> None:
        temporary, root = self.fixture("transitional-valid")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "quanta-gqlang-v2-algebra -> quanta-gqlang-v2-hir",
            result.stderr,
        )

    def test_reverse_edge_rejects_with_exact_edge(self) -> None:
        temporary, root = self.fixture("reverse-edge")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "quanta-gqlang-v2-sema -> quanta-gqlang-v2-planning",
            result.stderr,
        )

    def test_forbidden_runtime_dependency_rejects(self) -> None:
        temporary, root = self.fixture("runtime-edge")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("forbidden normal dependency quanta-runtime", result.stderr)

    def test_workspace_member_drift_rejects(self) -> None:
        temporary, root = self.fixture("valid", omit="quanta-gqlang-v2-types")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("workspace members drifted", result.stderr)

    def test_build_dependency_does_not_inherit_normal_edge_permission(self) -> None:
        temporary, root = self.fixture("build-edge")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "illegal build internal edges: quanta-gqlang-v2-surface -> quanta-gqlang-v2-hir",
            result.stderr,
        )

    def test_dev_dependency_does_not_inherit_normal_edge_permission(self) -> None:
        temporary, root = self.fixture("dev-edge")
        with temporary:
            result = run_checker(root)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "illegal dev internal edges: quanta-gqlang-v2-surface -> quanta-gqlang-v2-hir",
            result.stderr,
        )

    def test_unknown_dependency_kind_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "unknown dependency kind future-kind"):
            architecture_checker.dependency_kind({"kind": "future-kind"})


if __name__ == "__main__":
    unittest.main()
