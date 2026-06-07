#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Set


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_MANIFEST = ROOT / "Cargo.toml"
EXPECTED_WORKSPACE_MEMBERS: Set[str] = {
    "quanta-gqlang-v2-syntax",
    "quanta-gqlang-v2-sema",
    "quanta-gqlang-v2-ir",
    "quanta-gqlang-v2-cli",
    "quanta-gqlang-v2-lsp",
}
ALLOWED_INTERNAL_DEPENDENCIES: Dict[str, Set[str]] = {
    "quanta-gqlang-v2-syntax": set(),
    "quanta-gqlang-v2-sema": {"quanta-gqlang-v2-syntax"},
    "quanta-gqlang-v2-ir": {"quanta-gqlang-v2-sema"},
    "quanta-gqlang-v2-cli": {"quanta-gqlang-v2-ir"},
    "quanta-gqlang-v2-lsp": {"quanta-gqlang-v2-ir"},
}
FORBIDDEN_DEPENDENCY_PREFIXES = (
    "quanta-runtime",
    "quanta-sdk",
    "quanta-index",
)
FORBIDDEN_SOURCE_REGEXES = (
    (
        re.compile(r"\b(quanta_runtime|quanta_sdk|quanta_index)(::|_)"),
        "runtime/sdk/index source import leakage",
    ),
    (
        re.compile(r"\bquanta_gqlang_(?!v2_)\w+"),
        "legacy quanta_gqlang_* source import leakage",
    ),
)
FORBIDDEN_SOURCE_TOKENS = {
    "SearchHost": "frontend crates must stay host-agnostic",
    "IdeReadHost": "frontend crates must stay host-agnostic",
    "ServingQueryRequestV1": "frontend crates must not embed serving carrier DTOs",
    "ServingQueryResultV1": "frontend crates must not embed serving carrier DTOs",
    "native_runtime_provider_registry_v1": "provider registry ownership does not belong here",
}


def is_runtime_dependency(dependency: dict) -> bool:
    return dependency.get("kind") in (None, "normal")


def cargo_metadata() -> dict:
    result = subprocess.run(
        [
            "cargo",
            "metadata",
            "--manifest-path",
            str(WORKSPACE_MANIFEST),
            "--format-version",
            "1",
        ],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def main() -> int:
    metadata = cargo_metadata()
    packages = {
        package["name"]: package
        for package in metadata["packages"]
        if package["name"] in EXPECTED_WORKSPACE_MEMBERS
    }
    errors: List[str] = []

    package_names = set(packages)
    if package_names != EXPECTED_WORKSPACE_MEMBERS:
        errors.append(
            "workspace members drifted: expected "
            f"{sorted(EXPECTED_WORKSPACE_MEMBERS)}, got {sorted(package_names)}"
        )

    for package_name, allowed_internal in ALLOWED_INTERNAL_DEPENDENCIES.items():
        package = packages.get(package_name)
        if package is None:
            continue

        seen_internal = {
            dependency["name"]
            for dependency in package["dependencies"]
            if is_runtime_dependency(dependency)
            and dependency["name"] in EXPECTED_WORKSPACE_MEMBERS
        }
        illegal_internal = sorted(seen_internal - allowed_internal)
        if illegal_internal:
            errors.append(
                f"{package_name} has illegal internal deps: {illegal_internal}; "
                f"allowed={sorted(allowed_internal)}"
            )

        for dependency in package["dependencies"]:
            if not is_runtime_dependency(dependency):
                continue
            dep_name = dependency["name"]
            if dep_name.startswith(FORBIDDEN_DEPENDENCY_PREFIXES):
                errors.append(
                    f"{package_name} depends on forbidden backend crate {dep_name}"
                )
            if dep_name.startswith("quanta-gqlang-") and not dep_name.startswith(
                "quanta-gqlang-v2-"
            ):
                errors.append(
                    f"{package_name} depends on forbidden legacy crate {dep_name}"
                )

    for source_file in sorted(ROOT.glob("crates/**/src/**/*.rs")):
        text = source_file.read_text()
        for regex, reason in FORBIDDEN_SOURCE_REGEXES:
            if regex.search(text):
                errors.append(f"{source_file}: {reason}")
        for token, reason in FORBIDDEN_SOURCE_TOKENS.items():
            if token in text:
                errors.append(f"{source_file}: found forbidden token {token} ({reason})")

    if errors:
        print("architecture lint failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
