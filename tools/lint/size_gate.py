#!/usr/bin/env python3
"""Bound Rust source size with Tokei code lines and explicit legacy ceilings."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

from tools.ci.doctor import DoctorError, read_lock
from tools.lint.rust_module_guards import TEST_MODULES, module_has_attributes
from tools.lint.semantic_honesty import cfg_implies_test, node_text, parser


ROOT = Path(__file__).resolve().parents[2]
POLICY = Path(__file__).with_name("size_policy.json")
TEST_SUPPORT = "crates/quanta-gqlang-v2-sema/src/test_support.rs"
TEST_SUPPORT_OWNER = "crates/quanta-gqlang-v2-sema/src/lib.rs"
TEST_SUPPORT_ATTRIBUTES = {'#[cfg(any(test,feature="test"))]', '#[doc(hidden)]'}
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
    for name, (owner_name, relative, module_name) in TEST_MODULES.items():
        if name not in paths:
            continue
        owner = (root / owner_name).read_bytes()
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



# These groups can suppress the function-size warning just as its exact lint can.
FUNCTION_SIZE_OVERRIDE_LINTS = {
    "warnings", "clippy::all", "clippy::pedantic", "clippy::too_many_lines",
}
TRIVIA = {"line_comment", "block_comment"}
LITERALS = {"string_literal", "raw_string_literal", "char_literal"}


def _test_attribute(source: bytes, node) -> bool:
    text = node_text(source, node)
    match = re.fullmatch(r"#\s*!?\s*\[\s*cfg\s*\((.*)\)\s*\]", text, re.S)
    return bool(match and cfg_implies_test(match.group(1)))


def _production_attributes(source: bytes, root):
    """Keep actual attributes and macro-emitted attributes outside test scopes."""
    stack = [root]
    while stack:
        scope = stack.pop()
        children = scope.named_children
        if any(child.type == "inner_attribute_item" and _test_attribute(source, child)
               for child in children):
            continue
        pending = []
        for child in children:
            if child.type in TRIVIA:
                continue
            if child.type == "attribute_item":
                pending.append(child)
                continue
            if child.type == "inner_attribute_item":
                yield child
                continue
            attrs, pending = pending, []
            if any(_test_attribute(source, attr) for attr in attrs):
                continue
            yield from attrs
            if child.type == "macro_definition":
                # Rust attributes in macro token trees are not attribute_item AST
                # nodes. Check their literal syntax; ignore quoted documentation.
                yield child
            else:
                stack.append(child)


def _syntax_tokens(source: bytes, node):
    tokens = []
    stack = [node]
    while stack:
        current = stack.pop()
        if current.type in TRIVIA | LITERALS:
            continue
        if current.children:
            stack.extend(reversed(current.children))
        else:
            tokens.append((node_text(source, current), current.start_point.row + 1))
    return tokens


def _overrides(source: bytes, node):
    tokens = _syntax_tokens(source, node)
    values = [value for value, _line in tokens]
    # Macro token trees contain literal attributes as tokens rather than AST
    # attribute_item nodes. Delimit exact #[...] / #![...] syntax first.
    index = 0
    while index < len(values):
        bracket = index + 1
        if values[index] != "#":
            index += 1
            continue
        if values[bracket:bracket + 1] == ["!"]:
            bracket += 1
        if values[bracket:bracket + 1] != ["["]:
            index += 1
            continue
        depth = 1
        end = bracket + 1
        while end < len(values) and depth:
            depth += values[end] == "["
            depth -= values[end] == "]"
            end += 1
        for cursor in range(bracket + 1, end - 1):
            level = values[cursor]
            if level not in {"allow", "expect"} or values[cursor + 1:cursor + 2] != ["("]:
                continue
            call_depth = 1
            call_end = cursor + 2
            while call_end < end and call_depth:
                call_depth += values[call_end] == "("
                call_depth -= values[call_end] == ")"
                call_end += 1
            body = values[cursor + 2:call_end - 1]
            # Reason strings and comments were removed before lint parsing.
            for name in "".join(body).split(","):
                if name in FUNCTION_SIZE_OVERRIDE_LINTS:
                    yield tokens[cursor][1], level, name
            if node.type == "macro_definition" and any(token.startswith("$") for token in body):
                yield tokens[cursor][1], level, "dynamic macro lint"
        index = end


def function_size_overrides(root: Path, paths: set[str]) -> list[str]:
    findings = []
    rust_parser = parser()
    test_paths = {TEST_SUPPORT, *TEST_MODULES, *FEATURE_TEST_MODULES}
    for name in sorted(paths):
        if "/tests/" in name or name in test_paths:
            continue
        source = (root / name).read_bytes()
        tree = rust_parser.parse(source)
        if tree.root_node.has_error:
            raise SizeGateError("cannot inspect function-size policy in invalid Rust: " + name)
        for node in _production_attributes(source, tree.root_node):
            for line, level, lint in _overrides(source, node):
                findings.append(f"{name}:{line}: production {level}({lint}) function-size override is forbidden; split the function")
    return sorted(set(findings))

def main() -> int:
    try:
        limits, ceilings = read_policy()
        paths = source_paths(ROOT)
        counts = tokei_code_lines(ROOT, read_lock()["tokei"])
        findings = evaluate(paths, counts, limits, ceilings)
        findings.extend(function_size_overrides(ROOT, paths))
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
