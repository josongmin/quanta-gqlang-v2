#!/usr/bin/env python3
"""Prove one Semgrep rule is non-vacuous and clean in one JSON scan."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402
from tools.lint.semantic_honesty import node_text, parser, production_nodes  # noqa: E402
from tools.lint.rust_module_guards import module_has_attributes  # noqa: E402
from tools.lint.size_gate import TEST_SUPPORT_ATTRIBUTES  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".semgrep/qglang-frontend.yml"
RULE = "qglang-v2-no-string-typed-error-results-in-core"
EXPECTED_PATH = "tools/lint/tests/fixtures/semgrep/%s.rs" % RULE
EXPECTED_LINES = {1, 5, 9}
CORE_CRATES = (
    "algebra", "binder", "execution-contract", "hir", "logic", "modules", "pipeline", "planning",
    "sema", "surface", "types",
)


class SemgrepGateError(RuntimeError):
    pass


def expected_paths() -> set[str]:
    paths = {
        str(path.relative_to(ROOT))
        for owner in CORE_CRATES
        for path in (ROOT / ("crates/quanta-gqlang-v2-" + owner) / "src").rglob("*.rs")
    }
    guard = (ROOT / "crates/quanta-gqlang-v2-sema/src/lib.rs").read_bytes()
    if not module_has_attributes(guard, "test_support", TEST_SUPPORT_ATTRIBUTES):
        raise SemgrepGateError("test_support exclusion lost its test-only module guard")
    paths = {
        path for path in paths
        if path != "crates/quanta-gqlang-v2-sema/src/test_support.rs"
        and not path.startswith("crates/quanta-gqlang-v2-sema/src/test_support/")
    }
    paths.add(EXPECTED_PATH)
    return paths


def validate(document: object, expected_scanned: set[str]) -> None:
    if not isinstance(document, dict):
        raise SemgrepGateError("Semgrep JSON root is not an object")
    errors = document.get("errors")
    results = document.get("results")
    if not isinstance(errors, list) or not isinstance(results, list):
        raise SemgrepGateError("Semgrep JSON lacks result/error arrays")
    paths = document.get("paths")
    scanned = paths.get("scanned") if isinstance(paths, dict) else None
    if not isinstance(scanned, list) or not all(isinstance(path, str) for path in scanned):
        raise SemgrepGateError("Semgrep JSON lacks scanned-path evidence")
    observed_scanned = set(scanned)
    if observed_scanned != expected_scanned:
        raise SemgrepGateError(
            "Semgrep target inventory drifted: missing=%d extra=%d"
            % (len(expected_scanned - observed_scanned), len(observed_scanned - expected_scanned))
        )
    if errors:
        raise SemgrepGateError("Semgrep reported %d engine errors" % len(errors))
    if len(results) != len(EXPECTED_LINES) or not all(isinstance(item, dict) for item in results):
        raise SemgrepGateError(
            "expected exactly %d positive-fixture findings, observed %d" % (len(EXPECTED_LINES), len(results))
        )
    lines = set()
    for finding in results:
        check_id, path = finding.get("check_id"), finding.get("path")
        if not isinstance(check_id, str) or not check_id.endswith(RULE):
            raise SemgrepGateError("unexpected Semgrep rule identity")
        if path != EXPECTED_PATH:
            raise SemgrepGateError("unexpected Semgrep finding path: %r" % path)
        start = finding.get("start")
        line = start.get("line") if isinstance(start, dict) else None
        if type(line) is not int:
            raise SemgrepGateError("Semgrep finding lacks a line number")
        lines.add(line)
    if lines != EXPECTED_LINES:
        raise SemgrepGateError("positive fixture coverage changed: %s" % sorted(lines))


def string_error_public_lines(source: bytes, rust_parser) -> set[int]:
    """Classify the outer Result error type, including nested generic returns."""
    root = rust_parser.parse(source).root_node
    lines = set()
    for node, _ in production_nodes(root, source):
        if node.type != "function_item":
            continue
        if not any(child.type == "visibility_modifier" for child in node.named_children):
            continue
        result = node.child_by_field_name("return_type")
        if result is None or result.type != "generic_type":
            continue
        children = result.named_children
        if len(children) != 2 or node_text(source, children[0]).split("::")[-1] != "Result":
            continue
        arguments = children[1].named_children
        if len(arguments) == 2 and node_text(source, arguments[1]).split("::")[-1] == "String":
            lines.add(node.start_point[0] + 1)
    return lines


def validate_ast(expected_scanned: set[str]) -> None:
    rust_parser = parser()
    for path in sorted(expected_scanned):
        lines = string_error_public_lines((ROOT / path).read_bytes(), rust_parser)
        expected = EXPECTED_LINES if path == EXPECTED_PATH else set()
        if lines != expected:
            raise SemgrepGateError(
                "Rust AST return-type inventory drifted at %s: expected=%s observed=%s"
                % (path, sorted(expected), sorted(lines))
            )


def main() -> int:
    executable = ["semgrep"] if shutil.which("semgrep") else ["python3", "-m", "semgrep"]
    command = executable + [
        "--config", str(CONFIG),
        "--json",
        "--quiet",
        "--metrics=off",
        "--disable-version-check",
        "crates",
        EXPECTED_PATH,
    ]
    try:
        result = run_bounded(
            command, cwd=ROOT, timeout_s=300, capture_output=True, text=True
        )
        if result.returncode:
            raise SemgrepGateError(
                "Semgrep exited %d: %s" % (result.returncode, (result.stderr or "").strip())
            )
        paths = expected_paths()
        validate(json.loads(result.stdout or ""), paths)
        validate_ast(paths)
    except (OSError, ValueError, ProcessTimeoutError, SemgrepGateError) as error:
        print("semgrep gate: " + str(error), file=sys.stderr)
        return 2
    print("semgrep gate passed: production=0 positive_fixture=%d rule=%s" % (len(EXPECTED_LINES), RULE))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
