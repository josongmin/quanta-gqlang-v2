#!/usr/bin/env python3
"""Bound Rust source size with Tokei code lines and explicit legacy ceilings."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from tools.ci.doctor import DoctorError, read_lock
from tools.lint.rust_module_guards import module_has_attributes


ROOT = Path(__file__).resolve().parents[2]
POLICY = Path(__file__).with_name("size_policy.json")
TEST_SUPPORT = "crates/quanta-gqlang-v2-sema/src/test_support.rs"
TEST_SUPPORT_OWNER = "crates/quanta-gqlang-v2-sema/src/lib.rs"
TEST_SUPPORT_ATTRIBUTES = {'#[cfg(any(test,feature="test"))]', '#[doc(hidden)]'}
TEST_MODULES = {
    "crates/quanta-gqlang-v2-algebra/src/normalize/tests.rs":
        ("crates/quanta-gqlang-v2-algebra/src/normalize.rs", "normalize/tests.rs"),
    "crates/quanta-gqlang-v2-modules/src/graph/cycle_tests.rs":
        ("crates/quanta-gqlang-v2-modules/src/graph.rs", "graph/cycle_tests.rs"),
    "crates/quanta-gqlang-v2-pipeline/src/compile/tests.rs":
        ("crates/quanta-gqlang-v2-pipeline/src/compile.rs", "compile/tests.rs"),
    "crates/quanta-gqlang-v2-pipeline/src/execution/tests.rs":
        ("crates/quanta-gqlang-v2-pipeline/src/execution.rs", "execution/tests.rs"),
    "crates/quanta-gqlang-v2-planning/src/plan/tests.rs":
        ("crates/quanta-gqlang-v2-planning/src/plan.rs", "plan/tests.rs"),
    "crates/quanta-gqlang-v2-sema/src/document/tests.rs":
        ("crates/quanta-gqlang-v2-sema/src/document.rs", "document/tests.rs"),
    "crates/quanta-gqlang-v2-sema/src/item/tests.rs":
        ("crates/quanta-gqlang-v2-sema/src/item.rs", "item/tests.rs"),
    "crates/quanta-gqlang-v2-sema/src/relation/tests.rs":
        ("crates/quanta-gqlang-v2-sema/src/relation.rs", "relation/tests.rs"),
    "crates/quanta-gqlang-v2-sema/src/test_support/tests.rs":
        ("crates/quanta-gqlang-v2-sema/src/test_support.rs", "test_support/tests.rs"),
    "crates/quanta-gqlang-v2-sema/src/test_support/w4d_tests.rs":
        ("crates/quanta-gqlang-v2-sema/src/test_support.rs", "test_support/w4d_tests.rs"),
}
FEATURE_TEST_MODULES = {
    "crates/quanta-gqlang-v2-sema/src/test_support/families.rs": "families",
}


class SizeGateError(RuntimeError):
    pass


def source_paths(root: Path) -> set[str]:
    paths: set[str] = set()
    for pattern in ("*/src/**/*.rs", "*/tests/**/*.rs"):
        for path in (root / "crates").glob(pattern):
            if path.is_symlink() or not path.is_file():
                raise SizeGateError("Rust source is redirected or not a file: " + str(path))
            paths.add(path.relative_to(root).as_posix())
    if TEST_SUPPORT not in paths:
        raise SizeGateError("test support source is missing")
    owner = (root / TEST_SUPPORT_OWNER).read_bytes()
    if not module_has_attributes(owner, "test_support", TEST_SUPPORT_ATTRIBUTES):
        raise SizeGateError("test support is not guarded by cfg(any(test, feature = \"test\"))")
    test_support = (root / TEST_SUPPORT).read_bytes()
    for name, module_name in FEATURE_TEST_MODULES.items():
        if name not in paths or not module_has_attributes(test_support, module_name, set()):
            raise SizeGateError("feature-test module is missing from test support: " + name)
    for name, (owner_name, relative) in TEST_MODULES.items():
        if name not in paths:
            continue
        owner = (root / owner_name).read_bytes()
        module_name = Path(name).stem
        required = {'#[cfg(test)]', f'#[path="{relative}"]'}
        if not module_has_attributes(owner, module_name, required):
            raise SizeGateError("test module is not cfg(test) guarded: " + name)
    return paths


def read_policy(path: Path = POLICY) -> tuple[dict[str, int], dict[str, int]]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise SizeGateError("cannot read size policy: " + str(error)) from error
    if set(data) != {"schema_version", "limits", "exceptions"} or data["schema_version"] != 1:
        raise SizeGateError("invalid size policy schema")
    limits = data["limits"]
    if set(limits) != {"production", "test"} or any(
        type(value) is not int or value <= 0 for value in limits.values()
    ):
        raise SizeGateError("invalid size limits")
    exceptions = data["exceptions"]
    if not isinstance(exceptions, list):
        raise SizeGateError("size exceptions must be a list")
    ceilings: dict[str, int] = {}
    for entry in exceptions:
        if not isinstance(entry, dict) or set(entry) != {"path", "max_code_lines"}:
            raise SizeGateError("invalid size exception")
        name, ceiling = entry["path"], entry["max_code_lines"]
        if not isinstance(name, str) or not name.startswith("crates/") or \
           not name.endswith(".rs") or ".." in Path(name).parts or \
           name in ceilings or type(ceiling) is not int or ceiling <= 0:
            raise SizeGateError("invalid or duplicate size exception: " + repr(name))
        ceilings[name] = ceiling
    if list(ceilings) != sorted(ceilings):
        raise SizeGateError("size exceptions must be sorted by path")
    return limits, ceilings


def tokei_code_lines(root: Path, expected_version: str) -> dict[str, int]:
    try:
        version = subprocess.run(
            ("tokei", "--version"), cwd=root, capture_output=True, text=True,
            check=True, timeout=30,
        ).stdout.strip()
        if version != expected_version and not version.startswith(expected_version + " "):
            raise SizeGateError("tokei version mismatch: " + repr(version))
        result = subprocess.run(
            ("tokei", "crates", "--files", "--output", "json"), cwd=root,
            capture_output=True, text=True, check=True, timeout=30,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise SizeGateError("tokei failed: " + str(error)) from error
    try:
        rust = json.loads(result.stdout)["Rust"]
        if rust["inaccurate"] is not False:
            raise SizeGateError("tokei reported inaccurate Rust counts")
        reports = rust["reports"]
        if not isinstance(reports, list):
            raise SizeGateError("tokei Rust reports are invalid")
        counts: dict[str, int] = {}
        for report in reports:
            name, count = report["name"], report["stats"]["code"]
            if not isinstance(name, str) or type(count) is not int or count < 0 or name in counts:
                raise SizeGateError("tokei Rust file report is invalid")
            counts[name] = count
        return counts
    except (KeyError, TypeError, ValueError) as error:
        raise SizeGateError("cannot parse tokei Rust reports: " + str(error)) from error


def evaluate(paths: set[str], counts: dict[str, int], limits: dict[str, int],
             ceilings: dict[str, int]) -> list[str]:
    findings = []
    for name in sorted(paths):
        if name not in counts:
            findings.append(name + ": tokei omitted source")
            continue
        test_only = (
            "/tests/" in name or name == TEST_SUPPORT or name in TEST_MODULES
            or name in FEATURE_TEST_MODULES
        )
        limit = limits["test" if test_only else "production"]
        count = counts[name]
        ceiling = ceilings.get(name)
        if ceiling is not None and ceiling <= limit:
            findings.append(name + ": exception ceiling is not above the global limit")
        elif count > limit and ceiling is None:
            findings.append(f"{name}: {count} code lines exceeds {limit}; split the file")
        elif ceiling is not None and count > ceiling:
            findings.append(f"{name}: {count} code lines exceeds legacy ceiling {ceiling}")
        elif ceiling is not None and count <= limit:
            findings.append(name + ": legacy exception is stale; remove it")
    for name in sorted(ceilings.keys() - paths):
        findings.append(name + ": legacy exception names a missing source")
    return findings


def main() -> int:
    try:
        limits, ceilings = read_policy()
        paths = source_paths(ROOT)
        counts = tokei_code_lines(ROOT, read_lock()["tokei"])
        findings = evaluate(paths, counts, limits, ceilings)
    except (DoctorError, OSError, SizeGateError) as error:
        print("size gate: " + str(error), file=sys.stderr)
        return 2
    if findings:
        for finding in findings:
            print("size gate: " + finding, file=sys.stderr)
        return 1
    print(f"size gate: {len(paths)} Rust source files within policy")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
