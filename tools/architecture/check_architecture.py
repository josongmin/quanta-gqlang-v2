#!/usr/bin/env python3
#
# Architecture guard for the QGLang v2 workspace.
#
# This script is the `just arch-lint` gate. It now enforces FOUR independent
# guard dimensions:
#
#   1. Crate-dep layering (incl. dev-dependencies).
#      The normal-dep DAG is cli->ir, lsp->ir, ir->sema, sema->syntax.
#      dev-dependencies are checked against the SAME layering, except for an
#      explicit `ALLOWED_DEV_EDGES` allowlist of vetted test-only edges. Any
#      other dev-dep that breaks layering (e.g. a future cli/lsp dev-dep on
#      sema/syntax) fails. Forbidden backend/legacy crate deps still fail too.
#
#   2. Frontend-hub-bypass guard.
#      cli/lsp must consume syntax/sema types ONLY via the
#      `quanta_gqlang_v2_ir::frontend` re-export hub. A direct
#      `use quanta_gqlang_v2_ir::syntax::` / `::sema::` import bypasses the hub
#      and fails. ir-OWNED submodules (root_ir, explain_ir, find_ir, catalog,
#      lower_document_to_ir_v*, ...) remain legitimately consumable directly.
#
#   3. Runtime-truth token denylist.
#      Frontend `src` must not embed runtime-execution-result vocabulary. This
#      is a mix of fixed tokens and word-boundary pattern families chosen so
#      that honest tokens (no_result_reason, root_count,
#      runtime-host-routing-unavailable, ...) do NOT trip.
#
# Single-authority degraded-reason enforcement (one canonical owner of the
# degraded-reason constant) is intentionally NOT added here yet: a separate
# change owns de-duplicating that constant first, and this guard would be
# premature until then.
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
    "quanta-gqlang-v2-conformance",
}
ALLOWED_INTERNAL_DEPENDENCIES: Dict[str, Set[str]] = {
    "quanta-gqlang-v2-syntax": set(),
    "quanta-gqlang-v2-sema": {"quanta-gqlang-v2-syntax"},
    "quanta-gqlang-v2-ir": {"quanta-gqlang-v2-sema"},
    "quanta-gqlang-v2-cli": {"quanta-gqlang-v2-ir"},
    "quanta-gqlang-v2-lsp": {"quanta-gqlang-v2-ir"},
    # Dev-only conformance registry: a leaf that reads the public ir pipeline to
    # self-validate the 17-family front-door table. Consumed by other crates'
    # tests via vetted dev edges (see ALLOWED_DEV_EDGES).
    "quanta-gqlang-v2-conformance": {"quanta-gqlang-v2-ir"},
}
# Vetted dev-dependency edges that are exempt from the layering rule. These are
# test-only (`[dev-dependencies]`) edges that do NOT exist as normal-dep edges
# but are legitimately needed by integration/unit tests:
#   * ir -> syntax: ir already depends on syntax transitively via the normal
#     edge ir->sema->syntax; the dev-dep just lets ir's own tests touch syntax
#     types directly. (Also a normal layering edge in spirit.)
#   * cli -> lsp: cli's integration tests drive the lsp surface end-to-end.
# Anything NOT listed here is held to the normal-dep layering.
ALLOWED_DEV_EDGES: Set[tuple[str, str]] = {
    ("quanta-gqlang-v2-ir", "quanta-gqlang-v2-syntax"),
    ("quanta-gqlang-v2-cli", "quanta-gqlang-v2-lsp"),
}
FORBIDDEN_DEPENDENCY_PREFIXES = (
    "quanta-runtime",
    "quanta-sdk",
    "quanta-index",
)
# Frontend crates whose `src` must consume syntax/sema types ONLY through the
# `quanta_gqlang_v2_ir::frontend` re-export hub.
FRONTEND_HUB_CONSUMER_DIRS = (
    "crates/quanta-gqlang-v2-cli/src",
    "crates/quanta-gqlang-v2-lsp/src",
)
# A direct import of the syntax/sema re-export submodules bypasses the frontend
# hub. ir-OWNED submodules (root_ir, explain_ir, find_ir, catalog,
# lower_document_to_ir_v*, ...) are deliberately NOT matched here.
FRONTEND_HUB_BYPASS_REGEX = re.compile(
    r"\buse\s+quanta_gqlang_v2_ir::(?:syntax|sema)::"
)
# Runtime-truth token pattern families. These are word-boundary-anchored so an
# honest identifier that merely CONTAINS one of these as a substring does not
# trip (e.g. `no_result_reason`, `root_count`,
# `runtime-host-routing-unavailable`). Every pattern here has been verified
# absent from all `crates/*/src`. They are kept SEPARATE from the legacy/leakage
# regexes below so the reason message is specific.
RUNTIME_TRUTH_TOKEN_PATTERNS = (
    "result_count",
    "row_count",
    "rows_returned",
    "match_count",
    "provider_routing",
    "host_dto",
    "backend_result",
    "edit_applied",
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
) + tuple(
    (
        # \b on the trailing side too, so `result_counter`, `host_dtos`, and
        # `no_result_reason` (no boundary before `result`) do not match.
        re.compile(r"\b" + re.escape(token) + r"\b"),
        f"runtime-truth token leakage ({token})",
    )
    for token in RUNTIME_TRUTH_TOKEN_PATTERNS
)
FORBIDDEN_SOURCE_TOKENS = {
    "SearchHost": "frontend crates must stay host-agnostic",
    "IdeReadHost": "frontend crates must stay host-agnostic",
    "ServingQueryRequestV1": "frontend crates must not embed serving carrier DTOs",
    "ServingQueryResultV1": "frontend crates must not embed serving carrier DTOs",
    "native_runtime_provider_registry_v1": "provider registry ownership does not belong here",
}


def is_normal_dependency(dependency: dict) -> bool:
    return dependency.get("kind") in (None, "normal")


def is_dev_dependency(dependency: dict) -> bool:
    return dependency.get("kind") == "dev"


def is_layering_checked_edge(consumer: str, dependency: dict) -> bool:
    """Should this (consumer -> dependency) edge be held to crate layering?

    Normal deps always are. dev-deps are now checked too, EXCEPT the vetted,
    test-only edges in ALLOWED_DEV_EDGES. Other dependency kinds (build, etc.)
    are not layering edges. This is the guard-1 behaviour change: dev-deps used
    to be exempt entirely.
    """
    if is_normal_dependency(dependency):
        return True
    if is_dev_dependency(dependency):
        return (consumer, dependency["name"]) not in ALLOWED_DEV_EDGES
    return False


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
            if is_layering_checked_edge(package_name, dependency)
            and dependency["name"] in EXPECTED_WORKSPACE_MEMBERS
        }
        illegal_internal = sorted(seen_internal - allowed_internal)
        if illegal_internal:
            errors.append(
                f"{package_name} has illegal internal deps: {illegal_internal}; "
                f"allowed={sorted(allowed_internal)} "
                f"(dev-dep exceptions: {sorted(d for c, d in ALLOWED_DEV_EDGES if c == package_name)})"
            )

        for dependency in package["dependencies"]:
            if not (
                is_normal_dependency(dependency) or is_dev_dependency(dependency)
            ):
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

    # Guard 2: frontend-hub-bypass. cli/lsp src must reach syntax/sema types only
    # through quanta_gqlang_v2_ir::frontend, never the ::syntax::/::sema::
    # re-export submodules directly. ir-owned submodules stay legal.
    for hub_dir in FRONTEND_HUB_CONSUMER_DIRS:
        for source_file in sorted((ROOT / hub_dir).glob("**/*.rs")):
            if FRONTEND_HUB_BYPASS_REGEX.search(source_file.read_text()):
                errors.append(
                    f"{source_file}: frontend-hub bypass — import syntax/sema via "
                    "quanta_gqlang_v2_ir::frontend, not "
                    "quanta_gqlang_v2_ir::syntax::/::sema::"
                )

    if errors:
        print("architecture lint failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
