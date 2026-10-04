"""Read root Rust module attributes from syntax, excluding comments and strings."""

from __future__ import annotations

import re

from tools.lint.semantic_honesty import node_text, parser


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
