#!/usr/bin/env python3
"""Exact manifest-policy guard for the QGLang v2 frontend workspace.

Enforces tools/architecture/policy.toml exactly: workspace membership,
per-kind internal edges, per-crate per-kind external allowlists (build is
default-deny everywhere), workspace-external path rejection, custom-build
rejection, and rename/optional/target/feature drift rejection, plus a
``[lints] workspace = true`` requirement on every member manifest.

Exit 0 with a passing verdict, 1 with a failing verdict. ``--format json``
emits a machine-readable verdict object on stdout.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Iterable

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11 (pinned toolchain ships tomli)
    import tomli as tomllib


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
POLICY_PATH = Path(__file__).resolve().parent / "policy.toml"

DEP_SECTION_KINDS = (
    ("dependencies", "normal"),
    ("dev-dependencies", "dev"),
    ("build-dependencies", "build"),
)


def load_policy(path: Path) -> dict:
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except OSError as error:
        print(f"architecture lint failed: cannot read policy {path}: {error}", file=sys.stderr)
        raise SystemExit(1)
    except (tomllib.TOMLDecodeError, ValueError) as error:
        print(f"architecture lint failed: invalid policy {path}: {error}", file=sys.stderr)
        raise SystemExit(1)


_POLICY = load_policy(POLICY_PATH)

EXPECTED_WORKSPACE_MEMBERS = set(_POLICY["workspace"]["members"])


def parse_edge(raw: str) -> tuple[str, str]:
    consumer, _, dependency = raw.partition(" -> ")
    return (consumer.strip(), dependency.strip())


FINAL_NORMAL_EDGES = {parse_edge(edge) for edge in _POLICY["internal"]["normal"]}
FINAL_DEV_EDGES = {parse_edge(edge) for edge in _POLICY["internal"]["dev"]}
FINAL_BUILD_EDGES = {parse_edge(edge) for edge in _POLICY["internal"]["build"]}
INTERNAL_VERSION = _POLICY["internal"]["version"]
INTERNAL_ALLOWED_FEATURES = dict(_POLICY["internal"].get("allowed_features", {}))
EXTERNAL_ALLOW = {
    kind: {
        crate: {
            name: {"version": spec["version"], "features": list(spec.get("features", []))}
            for name, spec in crates.items()
        }
        for crate, crates in _POLICY.get("external", {}).get(kind, {}).items()
    }
    for kind in ("normal", "dev", "build")
}
HYGIENE = _POLICY.get("hygiene", {})

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
        "--format",
        choices=("human", "json"),
        default="human",
        help="verdict rendering (default: human)",
    )
    return parser.parse_args()


def cargo_metadata(root: Path) -> dict:
    try:
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
            timeout=60,
        )
    except subprocess.TimeoutExpired:
        print("architecture lint failed: cargo metadata timed out after 60 s", file=sys.stderr)
        raise SystemExit(1) from None
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


def feature_key(consumer: str, kind: str, dependency: str) -> str:
    return f"{consumer}|{kind}|{dependency}"


def cargo_normalized_req(version: str) -> str:
    """Normalize a manifest version requirement the way cargo metadata does.

    A bare requirement (``1.0``) is reported back as a caret requirement
    (``^1.0``); operator-prefixed requirements (``=0.10.9``) round-trip.
    """
    if version[:1].isdigit():
        return f"^{version}"
    return version


def check_metadata_edges(
    workspace_packages: dict, errors: list[str]
) -> tuple[set[tuple[str, str]], set[tuple[str, str]], set[tuple[str, str]]]:
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
            if dependency.get("target") is not None:
                errors.append(
                    f"{consumer} has target-conditional {kind} dependency "
                    f"{dependency_name} on {dependency['target']} "
                    "(target-conditional dependencies are denied)"
                )
            if dependency.get("rename") is not None:
                errors.append(
                    f"{consumer} has renamed {kind} dependency {dependency_name} "
                    f"(renames are denied)"
                )
            if dependency.get("optional"):
                errors.append(
                    f"{consumer} has optional {kind} dependency {dependency_name} "
                    "(optional dependencies are denied)"
                )
            if dependency.get("uses_default_features") is False:
                errors.append(
                    f"{consumer} has default-features-off {kind} dependency "
                    f"{dependency_name} (default-features drift is denied)"
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
            elif kind == "build":
                errors.append(
                    f"{consumer} has denied external build dependency {dependency_name} "
                    "(build dependencies are default-deny)"
                )
            else:
                allowed = EXTERNAL_ALLOW.get(kind, {}).get(consumer, {})
                spec = allowed.get(dependency_name)
                if spec is None:
                    errors.append(
                        f"{consumer} has undeclared external {kind} dependency {dependency_name}"
                    )
                else:
                    if dependency.get("req") != cargo_normalized_req(spec["version"]):
                        errors.append(
                            f"{consumer} has version drift on external {kind} dependency "
                            f"{dependency_name}: expected {spec['version']}, "
                            f"got {dependency.get('req')}"
                        )
                    if sorted(dependency.get("features", [])) != sorted(spec["features"]):
                        errors.append(
                            f"{consumer} has feature drift on external {kind} dependency "
                            f"{dependency_name}: expected {sorted(spec['features'])}, "
                            f"got {sorted(dependency.get('features', []))}"
                        )
    return normal_edges, dev_edges, build_edges


def check_member_manifest(
    root: Path, consumer: str, manifest_path: Path, errors: list[str]
) -> None:
    try:
        with open(manifest_path, "rb") as handle:
            manifest = tomllib.load(handle)
    except OSError as error:
        errors.append(f"{consumer} has unreadable manifest: {error}")
        return
    except (tomllib.TOMLDecodeError, ValueError) as error:
        errors.append(f"{consumer} has invalid manifest: {error}")
        return
    crate_dir = manifest_path.parent

    if HYGIENE.get("require_lints_workspace", True):
        lints = manifest.get("lints")
        if not isinstance(lints, dict) or lints.get("workspace") is not True:
            errors.append(
                f"{consumer} must declare `[lints] workspace = true` in its manifest"
            )

    if HYGIENE.get("deny_custom_build", True):
        package = manifest.get("package", {})
        build_key = package.get("build") if isinstance(package, dict) else None
        if build_key not in (None, False):
            errors.append(
                f"{consumer} declares a custom build script via `build = {build_key!r}` "
                "(custom builds are denied)"
            )
        if (crate_dir / "build.rs").is_file():
            errors.append(
                f"{consumer} has a custom build script build.rs (custom builds are denied)"
            )

    if HYGIENE.get("deny_target_sections", True) and "target" in manifest:
        errors.append(
            f"{consumer} has [target.*] dependency sections "
            "(target-conditional dependencies are denied)"
        )

    for section, kind in DEP_SECTION_KINDS:
        section_table = manifest.get(section, {})
        if not isinstance(section_table, dict):
            errors.append(f"{consumer} has invalid [{section}] table")
            continue
        for name, entry in sorted(section_table.items()):
            spec = {"version": entry} if isinstance(entry, str) else dict(entry)
            if not isinstance(spec, dict):
                errors.append(f"{consumer} has invalid {kind} dependency {name}")
                continue
            if HYGIENE.get("deny_renames", True) and "package" in spec:
                errors.append(
                    f"{consumer} has renamed {kind} dependency {name} "
                    f"(package = {spec['package']!r}; renames are denied)"
                )
            if HYGIENE.get("deny_optional", True) and spec.get("optional") is True:
                errors.append(
                    f"{consumer} has optional {kind} dependency {name} "
                    "(optional dependencies are denied)"
                )
            if HYGIENE.get("deny_default_features_drift", True) and spec.get(
                "default-features"
            ) is False:
                errors.append(
                    f"{consumer} has default-features-off {kind} dependency {name} "
                    "(default-features drift is denied)"
                )
            raw_path = spec.get("path")
            if raw_path is not None:
                resolved = (crate_dir / raw_path).resolve()
                try:
                    resolved.relative_to(root)
                except ValueError:
                    errors.append(
                        f"{consumer} has workspace-external path {kind} dependency {name} "
                        f"(path = {raw_path!r} escapes the workspace)"
                    )
                    continue
                if name not in EXPECTED_WORKSPACE_MEMBERS:
                    errors.append(
                        f"{consumer} has undeclared frontend {kind} dependency {name}"
                    )
                    continue
            if name in EXPECTED_WORKSPACE_MEMBERS:
                version = spec.get("version")
                if version is not None and version != INTERNAL_VERSION:
                    errors.append(
                        f"{consumer} has version drift on internal {kind} dependency "
                        f"{name}: expected {INTERNAL_VERSION}, got {version}"
                    )
                allowed_features = INTERNAL_ALLOWED_FEATURES.get(
                    feature_key(consumer, kind, name), []
                )
                features = spec.get("features", [])
                drifted = sorted(set(features) - set(allowed_features))
                if drifted:
                    errors.append(
                        f"{consumer} has feature drift on internal {kind} dependency "
                        f"{name}: unexpected features {drifted} "
                        f"(allowed: {sorted(allowed_features)})"
                    )
            elif raw_path is None:
                allowed = EXTERNAL_ALLOW.get(kind, {}).get(consumer, {})
                if name not in allowed:
                    if kind == "build":
                        errors.append(
                            f"{consumer} has denied external build dependency {name} "
                            "(build dependencies are default-deny)"
                        )
                    else:
                        errors.append(
                            f"{consumer} has undeclared external {kind} dependency {name}"
                        )
                else:
                    want = allowed[name]
                    if spec.get("version") != want["version"]:
                        errors.append(
                            f"{consumer} has version drift on external {kind} dependency "
                            f"{name}: expected {want['version']}, got {spec.get('version')}"
                        )
                    drifted = sorted(set(spec.get("features", [])) - set(want["features"]))
                    missing = sorted(set(want["features"]) - set(spec.get("features", [])))
                    if drifted or missing:
                        errors.append(
                            f"{consumer} has feature drift on external {kind} dependency "
                            f"{name}: expected {sorted(want['features'])}, "
                            f"got {sorted(spec.get('features', []))}"
                        )


def run_check(root: Path) -> tuple[dict, list[str]]:
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

    normal_edges, dev_edges, build_edges = check_metadata_edges(workspace_packages, errors)

    for consumer, package in sorted(workspace_packages.items()):
        for target in package.get("targets", []):
            if "custom-build" in target.get("kind", []):
                errors.append(
                    f"{consumer} has a custom build script {target.get('src_path')} "
                    "(custom builds are denied)"
                )
        check_member_manifest(root, consumer, Path(package["manifest_path"]), errors)

    illegal_normal = normal_edges - FINAL_NORMAL_EDGES
    illegal_dev = dev_edges - FINAL_DEV_EDGES
    illegal_build = build_edges - FINAL_BUILD_EDGES
    if illegal_normal:
        errors.append(f"illegal normal internal edges: {render_edges(illegal_normal)}")
    if illegal_dev:
        errors.append(f"illegal dev internal edges: {render_edges(illegal_dev)}")
    if illegal_build:
        errors.append(f"illegal build internal edges: {render_edges(illegal_build)}")

    summary = {
        "workspace_members": len(package_names),
        "normal_edges": len(normal_edges),
        "final_present": len(normal_edges & FINAL_NORMAL_EDGES),
        "dev_edges": len(dev_edges),
        "final_dev_present": len(dev_edges & FINAL_DEV_EDGES),
        "build_edges": len(build_edges),
        "final_build_present": len(build_edges & FINAL_BUILD_EDGES),
    }
    return summary, errors


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    summary, errors = run_check(root)

    verdict = "failed" if errors else "passed"
    if args.format == "json":
        print(json.dumps({"verdict": verdict, **summary, "errors": errors}, indent=2, sort_keys=True))
        return 1 if errors else 0

    stream = sys.stderr if errors else sys.stdout
    print(f"architecture lint {verdict}", file=stream)
    print(f"workspace_members={summary['workspace_members']}", file=stream)
    print(
        f"normal_edges={summary['normal_edges']} "
        f"final_present={summary['final_present']}",
        file=stream,
    )
    print(
        f"dev_edges={summary['dev_edges']} "
        f"final_dev_present={summary['final_dev_present']}",
        file=stream,
    )
    print(
        f"build_edges={summary['build_edges']} "
        f"final_build_present={summary['final_build_present']}",
        file=stream,
    )
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
