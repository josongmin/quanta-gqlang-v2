#!/usr/bin/env python3
"""Cargo-metadata architecture guard for the QGLang v2 frontend workspace."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Iterable


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
EXPECTED_WORKSPACE_MEMBERS = {
    "quanta-gqlang-v2-algebra",
    "quanta-gqlang-v2-binder",
    "quanta-gqlang-v2-cli",
    "quanta-gqlang-v2-conformance",
    "quanta-gqlang-v2-diagnostics-stdio",
    "quanta-gqlang-v2-frontend-contract",
    "quanta-gqlang-v2-hir",
    "quanta-gqlang-v2-logic",
    "quanta-gqlang-v2-modules",
    "quanta-gqlang-v2-pipeline",
    "quanta-gqlang-v2-planning",
    "quanta-gqlang-v2-sema",
    "quanta-gqlang-v2-surface",
    "quanta-gqlang-v2-types",
}

FINAL_NORMAL_EDGES = {
    ("quanta-gqlang-v2-surface", "quanta-gqlang-v2-hir"),
    ("quanta-gqlang-v2-binder", "quanta-gqlang-v2-hir"),
    ("quanta-gqlang-v2-types", "quanta-gqlang-v2-hir"),
    ("quanta-gqlang-v2-modules", "quanta-gqlang-v2-hir"),
    ("quanta-gqlang-v2-modules", "quanta-gqlang-v2-types"),
    ("quanta-gqlang-v2-sema", "quanta-gqlang-v2-hir"),
    ("quanta-gqlang-v2-sema", "quanta-gqlang-v2-binder"),
    ("quanta-gqlang-v2-sema", "quanta-gqlang-v2-types"),
    ("quanta-gqlang-v2-sema", "quanta-gqlang-v2-modules"),
    ("quanta-gqlang-v2-algebra", "quanta-gqlang-v2-sema"),
    ("quanta-gqlang-v2-algebra", "quanta-gqlang-v2-types"),
    ("quanta-gqlang-v2-logic", "quanta-gqlang-v2-sema"),
    ("quanta-gqlang-v2-logic", "quanta-gqlang-v2-types"),
    ("quanta-gqlang-v2-planning", "quanta-gqlang-v2-sema"),
    ("quanta-gqlang-v2-planning", "quanta-gqlang-v2-algebra"),
    ("quanta-gqlang-v2-planning", "quanta-gqlang-v2-logic"),
    ("quanta-gqlang-v2-planning", "quanta-gqlang-v2-types"),
    ("quanta-gqlang-v2-pipeline", "quanta-gqlang-v2-surface"),
    ("quanta-gqlang-v2-pipeline", "quanta-gqlang-v2-sema"),
    ("quanta-gqlang-v2-pipeline", "quanta-gqlang-v2-algebra"),
    ("quanta-gqlang-v2-pipeline", "quanta-gqlang-v2-logic"),
    ("quanta-gqlang-v2-pipeline", "quanta-gqlang-v2-planning"),
    ("quanta-gqlang-v2-pipeline", "quanta-gqlang-v2-modules"),
    ("quanta-gqlang-v2-frontend-contract", "quanta-gqlang-v2-pipeline"),
    ("quanta-gqlang-v2-cli", "quanta-gqlang-v2-frontend-contract"),
    (
        "quanta-gqlang-v2-diagnostics-stdio",
        "quanta-gqlang-v2-frontend-contract",
    ),
    ("quanta-gqlang-v2-conformance", "quanta-gqlang-v2-frontend-contract"),
}

# Development dependencies may follow an edge in the final DAG. Any additional
# test-only exception must be declared independently instead of weakening the
# normal graph.
FINAL_DEV_EDGES = FINAL_NORMAL_EDGES

# Build scripts execute code during compilation and therefore do not inherit
# either normal-link or test-only dependency authority.  The current frontend
# workspace admits no internal build edge; additions require an explicit edge
# here with an architecture review.
FINAL_BUILD_EDGES: set[tuple[str, str]] = set()

FORBIDDEN_DEPENDENCY_PREFIXES = (
    "quanta-runtime",
    "quanta-sdk",
    "quanta-index",
    "quanta-provider",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=REPOSITORY_ROOT,
        help="workspace root containing Cargo.toml",
    )
    parser.add_argument(
        "--require-final",
        action="store_true",
        help="compatibility spelling; the checker always enforces the final DAG",
    )
    return parser.parse_args()


def cargo_metadata(root: Path) -> dict:
    result = subprocess.run(
        [
            "cargo",
            "metadata",
            "--manifest-path",
            str(root / "Cargo.toml"),
            "--format-version",
            "1",
            "--no-deps",
        ],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("architecture lint failed: cargo metadata failed", file=sys.stderr)
        print(result.stderr.rstrip(), file=sys.stderr)
        raise SystemExit(1)
    return json.loads(result.stdout)


def dependency_kind(dependency: dict) -> str:
    kind = dependency.get("kind")
    if kind is None or kind == "normal":
        return "normal"
    if kind == "dev":
        return "dev"
    if kind == "build":
        return "build"
    raise ValueError(f"unknown dependency kind {kind}")


def render_edge(edge: tuple[str, str]) -> str:
    return f"{edge[0]} -> {edge[1]}"


def render_edges(edges: Iterable[tuple[str, str]]) -> str:
    return ", ".join(render_edge(edge) for edge in sorted(edges))


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    metadata = cargo_metadata(root)
    workspace_ids = set(metadata["workspace_members"])
    workspace_packages = {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in workspace_ids
    }
    errors: list[str] = []

    package_names = set(workspace_packages)
    if package_names != EXPECTED_WORKSPACE_MEMBERS:
        errors.append(
            "workspace members drifted: expected "
            f"{sorted(EXPECTED_WORKSPACE_MEMBERS)}, got {sorted(package_names)}"
        )

    normal_edges: set[tuple[str, str]] = set()
    dev_edges: set[tuple[str, str]] = set()
    build_edges: set[tuple[str, str]] = set()
    for consumer, package in sorted(workspace_packages.items()):
        for dependency in package["dependencies"]:
            try:
                kind = dependency_kind(dependency)
            except ValueError as error:
                errors.append(f"{consumer} has {error}")
                continue
            dependency_name = dependency["name"]
            if dependency_name.startswith(FORBIDDEN_DEPENDENCY_PREFIXES):
                errors.append(
                    f"{consumer} has forbidden {kind} dependency {dependency_name}"
                )
            if dependency_name.startswith("quanta-gqlang-") and not dependency_name.startswith(
                "quanta-gqlang-v2-"
            ):
                errors.append(
                    f"{consumer} has forbidden legacy {kind} dependency {dependency_name}"
                )
            if dependency_name.startswith("quanta-gqlang-v2-"):
                if dependency_name not in EXPECTED_WORKSPACE_MEMBERS:
                    errors.append(
                        f"{consumer} has undeclared frontend {kind} dependency {dependency_name}"
                    )
                    continue
                edge = (consumer, dependency_name)
                {
                    "normal": normal_edges,
                    "dev": dev_edges,
                    "build": build_edges,
                }[kind].add(edge)

    illegal_normal = normal_edges - FINAL_NORMAL_EDGES
    illegal_dev = dev_edges - FINAL_DEV_EDGES
    illegal_build = build_edges - FINAL_BUILD_EDGES
    if illegal_normal:
        errors.append(f"illegal normal internal edges: {render_edges(illegal_normal)}")
    if illegal_dev:
        errors.append(f"illegal dev internal edges: {render_edges(illegal_dev)}")
    if illegal_build:
        errors.append(f"illegal build internal edges: {render_edges(illegal_build)}")

    verdict = "failed" if errors else "passed"
    stream = sys.stderr if errors else sys.stdout
    print(f"architecture lint {verdict}", file=stream)
    print(f"workspace_members={len(package_names)}", file=stream)
    print(
        f"normal_edges={len(normal_edges)} "
        f"final_present={len(normal_edges & FINAL_NORMAL_EDGES)}",
        file=stream,
    )
    print(
        f"dev_edges={len(dev_edges)} "
        f"final_dev_present={len(dev_edges & FINAL_DEV_EDGES)}",
        file=stream,
    )
    print(
        f"build_edges={len(build_edges)} "
        f"final_build_present={len(build_edges & FINAL_BUILD_EDGES)}",
        file=stream,
    )
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
