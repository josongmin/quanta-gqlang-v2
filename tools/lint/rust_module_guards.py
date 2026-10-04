"""Read root Rust module attributes from syntax, excluding comments and strings."""

from __future__ import annotations

import re
from pathlib import Path

from tools.lint.semantic_honesty import node_text, parser


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

def module_has_attributes(source: bytes, name: str, required: set[str]) -> bool:
    root = parser().parse(source).root_node
    if root.has_error:
        return False
    matches: list[set[str]] = []
    pending: set[str] = set()
    for node in root.named_children:
        if node.type == "attribute_item":
            pending.add(re.sub(r"\s+", "", node_text(source, node)))
            continue
        if node.type in {"line_comment", "block_comment"}:
            continue
        if node.type == "mod_item":
            identifier = node.child_by_field_name("name")
            if identifier is not None and node_text(source, identifier) == name:
                matches.append(pending)
        pending = set()
    return len(matches) == 1 and required <= matches[0]


def guarded_test_module_paths(root: Path) -> set[str]:
    """Admit only exact external modules whose owner has real cfg(test) syntax."""
    paths: set[str] = set()
    for name, (owner_name, relative) in TEST_MODULES.items():
        if not (root / name).is_file():
            continue
        owner = root / owner_name
        required = {'#[cfg(test)]', f'#[path="{relative}"]'}
        if not owner.is_file() or not module_has_attributes(
            owner.read_bytes(), Path(name).stem, required
        ):
            raise ValueError("test module is not cfg(test) guarded: " + name)
        paths.add(name)
    return paths
