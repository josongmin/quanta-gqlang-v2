#!/usr/bin/env python3
"""AST policy for first-party Rust outcome and diagnostic projections.

This checks deliberately local transformations. It does not claim to prove
arbitrary interprocedural data flow; owner and front-door tests cover that.
"""

from __future__ import annotations

import json
from importlib.metadata import PackageNotFoundError, version
import re
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
POLICY = Path(__file__).with_name("semantic_honesty_policy.json")
EMPTY_SUCCESS = frozenset({"None", "Ok(None)", "Ok(())", "Ok(false)", "Ok(0)", "Ok(Vec::new())", "Ok(Default::default())"})
PARSER_VERSIONS = {"tree-sitter-language-pack": "0.9.1", "tree-sitter": "0.23.2"}
RETURN_SCOPE_BOUNDARIES = frozenset({"closure_expression", "function_item", "async_block"})
UNSUPPORTED_EXIT_FORMS = frozenset({"loop_expression", "while_expression", "for_expression", "try_block", "try_expression", "await_expression", "break_expression", "macro_invocation"})


@dataclass(frozen=True)
class Finding:
    rule: str
    path: str
    line: int
    detail: str

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.rule}: {self.detail}"


def parser():
    for package, expected in PARSER_VERSIONS.items():
        try:
            actual = version(package)
        except PackageNotFoundError as error:
            raise RuntimeError(f"{package} is required; install tools/lint/requirements.txt") from error
        if actual != expected:
            raise RuntimeError(f"{package} version {actual} does not match pinned {expected}")
    try:
        from tree_sitter_language_pack import get_parser
    except ImportError as error:
        raise RuntimeError("tree-sitter-language-pack is required; install tools/lint/requirements.txt") from error
    return get_parser("rust")


def read_policy(path: Path = POLICY) -> dict:
    data = json.loads(path.read_text())
    if set(data) != {"schema_version", "enum_families", "semantic_fields", "allowed_transitions", "wire_structs"} or data["schema_version"] != 1:
        raise ValueError("semantic honesty policy schema is invalid")
    families = data["enum_families"]
    if not isinstance(families, list) or not families:
        raise ValueError("enum_families must be a nonempty list")
    seen = set()
    for family in families:
        required = {"type", "source", "negative", "positive", "empty_is_success"}
        if not required <= set(family) or set(family) - required - {"forbidden_outputs", "ban_untyped_ok"}:
            raise ValueError("enum family fields are invalid")
        name = family["type"]
        source = family["source"]
        if not isinstance(name, str) or not name.isidentifier() or name in seen:
            raise ValueError(f"invalid or repeated enum family: {name}")
        seen.add(name)
        if not isinstance(source, str) or not source.startswith("crates/") or not source.endswith(".rs") or ".." in Path(source).parts:
            raise ValueError(f"invalid source for {name}")
        for field in ("negative", "positive", "forbidden_outputs"):
            values = family.get(field, [])
            if not isinstance(values, list) or any(not isinstance(value, str) or not value for value in values) or len(values) != len(set(values)):
                raise ValueError(f"invalid {field} for {name}")
        if not family["negative"] or not family["positive"] or set(family["negative"]) & set(family["positive"]):
            raise ValueError(f"invalid polarity for {name}")
        if type(family["empty_is_success"]) is not bool:
            raise ValueError(f"invalid empty_is_success for {name}")
        if type(family.get("ban_untyped_ok", False)) is not bool:
            raise ValueError(f"invalid ban_untyped_ok for {name}")
    for field in ("semantic_fields", "wire_structs"):
        values = data[field]
        if not isinstance(values, list) or not values or any(not isinstance(value, str) or not value for value in values) or len(values) != len(set(values)):
            raise ValueError(f"invalid {field}")
    transitions = data["allowed_transitions"]
    if not isinstance(transitions, list) or any(not isinstance(pair, list) or len(pair) != 2 or any(not isinstance(item, str) for item in pair) for pair in transitions) or len(transitions) != len({tuple(pair) for pair in transitions}):
        raise ValueError("invalid allowed_transitions")
    by_name = {family["type"]: family for family in families}
    for before, after in transitions:
        source_type, sep1, source_variant = before.partition("::")
        target_type, sep2, target_variant = after.partition("::")
        if not sep1 or not sep2 or source_type not in by_name or target_type not in by_name or source_variant not in by_name[source_type]["negative"] or target_variant not in by_name[target_type]["positive"]:
            raise ValueError(f"invalid allowed transition {before} -> {after}")
    return data


def node_text(source: bytes, node) -> str:
    return source[node.start_byte:node.end_byte].decode("utf-8", errors="replace")


def syntax_compact(source: bytes, node) -> str:
    """Return syntax without trivia while preserving every significant token."""
    parts = []
    stack = [node]
    while stack:
        current = stack.pop()
        if current.type in {"line_comment", "block_comment"}:
            continue
        children = current.children
        if children:
            stack.extend(reversed(children))
        else:
            parts.append(re.sub(r"\s+", "", node_text(source, current)))
    return "".join(parts)


def significant_named_children(node):
    return tuple(child for child in node.named_children if child.type not in {"line_comment", "block_comment"})


def unwrap_terminal(node):
    """Remove value-neutral wrappers and select a block's returned expression."""
    current = node
    while current is not None:
        if current.type in {"parenthesized_expression", "expression_statement", "return_expression", "else_clause"}:
            children = significant_named_children(current)
            if len(children) != 1:
                break
            current = children[0]
            continue
        if current.type == "block":
            children = significant_named_children(current)
            if not children:
                break
            current = children[-1]
            continue
        break
    return current


def unwrap_neutral_value(node):
    """Unwrap syntax that cannot hide statements or control flow."""
    current = node
    while current is not None and current.type in {"parenthesized_expression", "expression_statement", "block", "else_clause"}:
        children = significant_named_children(current)
        if len(children) != 1:
            break
        current = children[0]
    return current


def terminal_compact(source: bytes, node) -> str:
    terminal = unwrap_terminal(node)
    return syntax_compact(source, terminal) if terminal is not None else ""


def neutral_compact(source: bytes, node) -> str:
    value = unwrap_neutral_value(node)
    return syntax_compact(source, value) if value is not None else ""


def rust_string_value(source: bytes, node) -> str | None:
    if node.type == "raw_string_literal":
        return "".join(node_text(source, child) for child in node.children if child.type == "string_content")
    if node.type != "string_literal":
        return None
    value = []
    strip_continuation_indent = False
    escapes = {r"\n": "\n", r"\r": "\r", r"\t": "\t", r"\\": "\\", r'\"': '"', r"\'": "'", r"\0": "\0"}
    for child in node.children:
        text = node_text(source, child)
        if child.type == "string_content":
            if strip_continuation_indent:
                text = text.lstrip()
                strip_continuation_indent = False
            value.append(text)
        elif child.type == "escape_sequence":
            if text in escapes:
                value.append(escapes[text])
            elif match := re.fullmatch(r"\\x([0-9a-fA-F]{2})", text):
                value.append(chr(int(match.group(1), 16)))
            elif match := re.fullmatch(r"\\u\{([0-9a-fA-F_]{1,8})\}", text):
                try:
                    value.append(chr(int(match.group(1).replace("_", ""), 16)))
                except ValueError:
                    return None
            elif re.fullmatch(r"\\\r?\n[ \t]*", text):
                strip_continuation_indent = True
                continue
            else:
                return None
    return "".join(value)


def terminal_scalar(source: bytes, node) -> str | None:
    """Evaluate only literal terminals and tuple-index projections."""
    terminal = unwrap_terminal(node)
    if terminal is None:
        return None
    if terminal.type == "integer_literal":
        literal = syntax_compact(source, terminal)
        match = re.fullmatch(
            r"(0[xX][0-9a-fA-F_]+|0[oO][0-7_]+|0[bB][01_]+|[0-9][0-9_]*)(?:_?(?:[iu](?:8|16|32|64|128|size)))?",
            literal,
        )
        if match is None:
            return None
        digits = match.group(1).replace("_", "")
        base = 16 if digits.lower().startswith("0x") else 8 if digits.lower().startswith("0o") else 2 if digits.lower().startswith("0b") else 10
        try:
            return str(int(digits, base))
        except ValueError:
            return None
    if terminal.type in {"string_literal", "raw_string_literal"}:
        value = rust_string_value(source, terminal)
        return json.dumps(value, ensure_ascii=False) if value is not None else None
    if terminal.type in {"boolean_literal", "unit_expression"}:
        return syntax_compact(source, terminal)
    if terminal.type == "unary_expression":
        children = significant_named_children(terminal)
        if len(children) == 1 and node_text(source, terminal).lstrip().startswith("-"):
            value = terminal_scalar(source, children[0])
            if value is not None and re.fullmatch(r"-?[0-9]+", value):
                return str(-int(value))
    if terminal.type == "type_cast_expression":
        value = terminal.child_by_field_name("value")
        target = terminal.child_by_field_name("type")
        scalar = terminal_scalar(source, value) if value is not None else None
        target_name = syntax_compact(source, target) if target is not None else ""
        if target_name == "_":
            return scalar
        match = re.fullmatch(r"([iu])(8|16|32|64|128)", target_name)
        if scalar is not None and re.fullmatch(r"-?[0-9]+", scalar) and match is not None:
            bits = int(match.group(2))
            integer = int(scalar) % (1 << bits)
            if match.group(1) == "i" and integer >= 1 << (bits - 1):
                integer -= 1 << bits
            return str(integer)
    if terminal.type == "field_expression":
        value = unwrap_terminal(terminal.child_by_field_name("value"))
        field = terminal.child_by_field_name("field")
        if value is not None and value.type == "tuple_expression" and field is not None:
            try:
                index = int(node_text(source, field))
            except ValueError:
                return None
            elements = significant_named_children(value)
            return terminal_scalar(source, elements[index]) if index < len(elements) else None
    return None


def call_argument(node):
    terminal = unwrap_terminal(node)
    if terminal is None or terminal.type != "call_expression":
        return None
    arguments = terminal.child_by_field_name("arguments")
    if arguments is None:
        return None
    values = significant_named_children(arguments)
    return values[0] if len(values) == 1 else None


def empty_success(source: bytes, node) -> str | None:
    compact = terminal_compact(source, node)
    if compact in EMPTY_SUCCESS:
        return compact
    if call_name(source, node) != "Ok":
        return None
    argument = call_argument(node)
    if argument is None:
        return None
    inner = terminal_compact(source, argument)
    normalized = f"Ok({inner})"
    return normalized if normalized in EMPTY_SUCCESS else None


def walk(node):
    stack = [node]
    while stack:
        current = stack.pop()
        yield current
        stack.extend(reversed(current.named_children))


def wildcard_pattern(node) -> bool:
    if node.type == "_":
        return True
    if node.type not in {"match_pattern", "parenthesized_pattern", "captured_pattern"}:
        return False
    return any(wildcard_pattern(child) for child in node.children)


def cfg_operands(expression: str) -> tuple[str, list[str]] | None:
    match = re.fullmatch(r"(all|any)\s*\((.*)\)", expression.strip(), re.S)
    if match is None:
        return None
    parts, start, depth = [], 0, 0
    for index, char in enumerate(match.group(2)):
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif char == "," and depth == 0:
            parts.append(match.group(2)[start:index].strip())
            start = index + 1
    parts.append(match.group(2)[start:].strip())
    return match.group(1), parts


def cfg_implies_test(expression: str) -> bool:
    if expression.strip() == "test":
        return True
    parsed = cfg_operands(expression)
    if parsed is None:
        return False
    operation, parts = parsed
    return any(map(cfg_implies_test, parts)) if operation == "all" else bool(parts) and all(map(cfg_implies_test, parts))


def production_nodes(root, source: bytes):
    stack = [iter(root.named_children)]
    pending = [[]]
    while stack:
        child = next(stack[-1], None)
        if child is None:
            stack.pop()
            pending.pop()
            continue
        if child.type == "attribute_item":
            pending[-1].append(node_text(source, child))
            continue
        attrs = pending[-1]
        pending[-1] = []
        test_only = False
        for attr in attrs:
            cfg = re.fullmatch(r"#\s*\[\s*cfg\s*\((.*)\)\s*\]", attr, re.S)
            if cfg and cfg_implies_test(cfg.group(1)):
                test_only = True
                break
        if test_only:
            continue
        yield child, attrs
        stack.append(iter(child.named_children))
        pending.append([])


def owner_type(source: bytes, node) -> str | None:
    current = node.parent
    while current is not None:
        if current.type == "impl_item":
            target = current.child_by_field_name("type")
            return node_text(source, target).split("<", 1)[0].split("::")[-1] if target else None
        current = current.parent
    return None


def references(source: bytes, root, aliases: dict[str, str], known: set[str]) -> set[tuple[str, str]]:
    found = set()
    for node in walk(root):
        if node.type == "identifier" and node.parent is not None and node.parent.type in {"tuple_struct_pattern", "struct_pattern"}:
            direct = aliases.get(node_text(source, node), "").split("::")
            if len(direct) == 2 and direct[0] in known:
                found.add((direct[0], direct[1]))
        if node.type not in {"scoped_identifier", "scoped_type_identifier"}:
            continue
        parts = node_text(source, node).split("::")
        if len(parts) < 2:
            continue
        family = parts[-2]
        if family == "Self":
            family = owner_type(source, node) or family
        family = aliases.get(family, family)
        if family in known:
            found.add((family, parts[-1]))
    return found


def call_name(source: bytes, node) -> str | None:
    node = unwrap_terminal(node)
    if node is None:
        return None
    if node.type != "call_expression":
        return None
    function = node.child_by_field_name("function")
    name = node_text(source, function) if function else None
    if name and name.startswith(("std::result::Result::", "core::result::Result::")):
        return name.rsplit("::", 1)[-1]
    return name


class UnsupportedExit(ValueError):
    """The local AST proof cannot classify a governed return path."""


def nested_returns(node):
    """Find function returns without treating closure-local returns as arm exits."""
    stack = [node]
    while stack:
        current = stack.pop()
        if current.type in RETURN_SCOPE_BOUNDARIES:
            continue
        if current.type == "return_expression":
            yield current
            continue
        stack.extend(reversed(current.named_children))


def enclosing_return_type_node(node):
    owner = node.parent
    while owner is not None and owner.type != "function_item":
        owner = owner.parent
    if owner is None:
        return None
    return owner.child_by_field_name("return_type")


def generic_return_arguments(source: bytes, node, constructor: str):
    result = enclosing_return_type_node(node)
    if result is None or result.type != "generic_type":
        return None
    children = significant_named_children(result)
    if len(children) != 2 or children[0].type != "type_identifier" or node_text(source, children[0]) != constructor or children[1].type != "type_arguments":
        return None
    return significant_named_children(children[1])


def is_render_vec_return(source: bytes, node) -> bool:
    arguments = generic_return_arguments(source, node, "Vec")
    return arguments is not None and len(arguments) == 1 and arguments[0].type == "type_identifier" and node_text(source, arguments[0]) == "String"


def is_analysis_payload_return(source: bytes, node) -> bool:
    arguments = generic_return_arguments(source, node, "Result")
    return arguments is not None and len(arguments) == 2 and arguments[0].type == "type_identifier" and node_text(source, arguments[0]) == "AnalysisPayloadV2"


def exit_expressions(source: bytes, node):
    """Enumerate each syntactic arm exit, including branch and early-return paths.

    Ordinary calls and identifiers remain opaque values. Unsupported control
    forms fail closed because selecting their final value requires more than
    the local branch grammar modeled here.
    """
    if node.type in {"parenthesized_expression", "expression_statement", "else_clause", "match_pattern"}:
        children = significant_named_children(node)
        if len(children) != 1:
            raise UnsupportedExit(f"{node.type} does not have one value")
        yield from exit_expressions(source, children[0])
        return
    if node.type == "return_expression":
        children = significant_named_children(node)
        if not children:
            yield node  # Rust unit return.
            return
        if len(children) != 1:
            raise UnsupportedExit("return has multiple values")
        yield from exit_expressions(source, children[0])
        return
    if node.type == "block":
        children = significant_named_children(node)
        for statement in children[:-1]:
            for early in nested_returns(statement):
                yield from exit_expressions(source, early)
        if not children:
            yield node
            return
        final = children[-1]
        if final.type == "let_declaration" or (final.type == "expression_statement" and any(child.type == ";" for child in final.children)):
            for early in nested_returns(final):
                yield from exit_expressions(source, early)
            final_value = significant_named_children(final)
            if not (len(final_value) == 1 and final_value[0].type == "return_expression"):
                yield node
            return
        yield from exit_expressions(source, final)
        return
    if node.type == "if_expression":
        condition = node.child_by_field_name("condition")
        for early in nested_returns(condition) if condition is not None else ():
            yield from exit_expressions(source, early)
        consequence = node.child_by_field_name("consequence")
        alternative = node.child_by_field_name("alternative")
        if consequence is None:
            raise UnsupportedExit("if has no auditable consequence")
        yield from exit_expressions(source, consequence)
        if alternative is None:
            yield node  # The other path is Rust's unit value.
        else:
            yield from exit_expressions(source, alternative)
        return
    if node.type == "match_expression":
        scrutinee = node.child_by_field_name("value")
        body = node.child_by_field_name("body")
        if (
            scrutinee is None
            or scrutinee.is_missing
            or scrutinee.has_error
            or body is None
            or body.is_missing
            or body.has_error
        ):
            raise UnsupportedExit("match has no auditable scrutinee or body")
        for early in nested_returns(scrutinee):
            yield from exit_expressions(source, early)
        arms = significant_named_children(body)
        if any(arm.type != "match_arm" for arm in arms):
            raise UnsupportedExit("match body has an unauditable member")
        # A complete empty match has no ordinary exit. Rust's type checker
        # permits it only when the scrutinee is uninhabited.
        for arm in arms:
            value = arm.child_by_field_name("value")
            if value is None:
                raise UnsupportedExit("match arm has no auditable value")
            yield from exit_expressions(source, value)
        return
    if node.type in UNSUPPORTED_EXIT_FORMS:
        if node.type != "macro_invocation" or not is_render_vec_return(source, node):
            raise UnsupportedExit(f"unsupported exit form {node.type}")
    for early in nested_returns(node):
        yield from exit_expressions(source, early)
    yield node


def terminal_variant(source: bytes, expression, aliases: dict[str, str], families: dict[str, dict]) -> tuple[str, str] | None:
    """Classify only the returned constructor, never constructors in arguments."""
    terminal = unwrap_terminal(expression)
    if terminal is None:
        return None
    target = terminal
    if terminal.type == "call_expression":
        target = terminal.child_by_field_name("function")
    elif terminal.type == "struct_expression":
        target = terminal.child_by_field_name("name")
    if target is None:
        return None
    if target.type == "identifier":
        direct = aliases.get(node_text(source, target), "").split("::")
        if len(direct) == 2 and direct[0] in families:
            return direct[0], direct[1]
        return None
    if target.type not in {"scoped_identifier", "scoped_type_identifier"}:
        return None
    refs = references(source, target, aliases, set(families))
    return next(iter(refs)) if len(refs) == 1 else None


def has_negative_terminal(source: bytes, expression, aliases: dict[str, str], families: dict[str, dict]) -> bool:
    terminal = unwrap_terminal(expression)
    if terminal is None:
        return False
    if call_name(source, terminal) == "Err":
        return True
    variant = terminal_variant(source, terminal, aliases, families)
    return variant is not None and variant[1] in families[variant[0]]["negative"]


def is_error_pattern(source: bytes, pattern) -> bool:
    current = pattern
    while current is not None and current.type in {"match_pattern", "parenthesized_pattern", "reference_pattern"}:
        children = significant_named_children(current)
        if len(children) != 1:
            return False
        current = children[0]
    if current is None or current.type != "tuple_struct_pattern":
        return False
    children = significant_named_children(current)
    return bool(children) and syntax_compact(source, children[0]) in {
        "Err", "std::result::Result::Err", "core::result::Result::Err"
    }


def pattern_bindings(source: bytes, pattern) -> set[str]:
    """Collect bound values from pattern grammar, independent of their names."""
    result = set()
    for current in walk(pattern):
        if current.type == "shorthand_field_identifier":
            name = node_text(source, current)
            if not name.startswith("_"):
                result.add(name)
        elif current.type == "identifier":
            parent = current.parent
            if parent is not None and parent.type in {"scoped_identifier", "scoped_type_identifier"}:
                continue
            if parent is not None and parent.type in {"tuple_struct_pattern", "struct_pattern"} and significant_named_children(parent)[0] == current:
                continue
            name = node_text(source, current)
            if not name.startswith("_"):
                result.add(name)
    return result


def aliases_in(source: bytes, location, known: set[str]) -> dict[str, str]:
    """Resolve imports from the lexical containers of one governed expression."""
    aliases = {}
    scopes = []
    scope = location.parent
    while scope is not None:
        scopes.append(scope)
        scope = scope.parent
    for scope in reversed(scopes):
        for child in scope.named_children:
            if child.type != "use_declaration":
                continue
            for clause in walk(child):
                if clause.type == "scoped_use_list":
                    parts = significant_named_children(clause)
                    if len(parts) == 2:
                        family = syntax_compact(source, parts[0]).split("::")[-1]
                        if family in known:
                            for member in significant_named_children(parts[1]):
                                if member.type == "identifier":
                                    variant = node_text(source, member)
                                    aliases[variant] = f"{family}::{variant}"
                                elif member.type == "use_as_clause":
                                    names = significant_named_children(member)
                                    if len(names) == 2:
                                        aliases[node_text(source, names[1])] = f"{family}::{node_text(source, names[0])}"
                if clause.type != "use_as_clause":
                    continue
                parts = significant_named_children(clause)
                if len(parts) != 2 or parts[1].type != "identifier":
                    continue
                original = syntax_compact(source, parts[0]).split("::")[-1]
                if original in known:
                    aliases[node_text(source, parts[1])] = original
    return aliases


def analysis_status_findings(source: bytes, path: str, function) -> list[Finding]:
    """Check the two priority branches that classify open analysis."""
    name = function.child_by_field_name("name")
    if name is None or node_text(source, name) != "status" or owner_type(source, function) != "AnalysisDocumentV2":
        return []
    body = function.child_by_field_name("body")
    body_children = significant_named_children(body) if body is not None else ()
    if len(body_children) != 1 or body_children[0].type != "expression_statement":
        return [Finding("SH-09-status-priority", path, function.start_point[0] + 1, "analysis status must have one conditional classifier")]
    outer = unwrap_neutral_value(body_children[0])
    if outer is None or outer.type != "if_expression":
        return [Finding("SH-09-status-priority", path, function.start_point[0] + 1, "analysis status has no conditional classifier")]
    first_condition = outer.child_by_field_name("condition")
    first_value = outer.child_by_field_name("consequence")
    alternative = outer.child_by_field_name("alternative")
    inner = unwrap_neutral_value(alternative) if alternative else None

    def branch_is(value, expected: str) -> bool:
        return value is not None and neutral_compact(source, value) == expected

    checks = (
        first_condition is not None and neutral_compact(source, first_condition) == "!self.unresolved.is_empty()",
        branch_is(first_value, "AnalysisStatusV2::Incomplete"),
        inner is not None
        and inner.type == "if_expression"
        and neutral_compact(source, inner.child_by_field_name("condition"))
        == "self.items.iter().any(|item|matches!(item,AnalysisItemV2::BlockedByDiagnostics(_)))",
        inner is not None and inner.type == "if_expression" and branch_is(inner.child_by_field_name("consequence"), "AnalysisStatusV2::Error"),
        inner is not None and inner.type == "if_expression" and branch_is(inner.child_by_field_name("alternative"), "AnalysisStatusV2::Complete"),
    )
    return [] if all(checks) else [Finding("SH-09-status-priority", path, function.start_point[0] + 1, "unresolved, blocked, and complete branches drifted")]


def is_document_error_variant(source: bytes, node) -> bool:
    if node.type not in {"scoped_identifier", "scoped_type_identifier"}:
        return False
    return syntax_compact(source, node).split("::")[-2:] == ["FrontendTransactionErrorV2", "Document"]


def document_error_binding(source: bytes, pattern) -> str | None:
    for node in walk(pattern):
        if node.type != "tuple_struct_pattern":
            continue
        children = significant_named_children(node)
        if len(children) != 2 or not is_document_error_variant(source, children[0]):
            continue
        binding = children[1]
        identifiers = [child for child in walk(binding) if child.type == "identifier"]
        if len(identifiers) == 1:
            return node_text(source, identifiers[0])
    return None


def document_error_payload_findings(source: bytes, path: str, arm) -> list[Finding]:
    if not is_analysis_payload_return(source, arm):
        return []
    pattern = arm.child_by_field_name("pattern")
    value = arm.child_by_field_name("value")
    document_pattern = pattern is not None and any(is_document_error_variant(source, node) for node in walk(pattern))
    if not document_pattern:
        return []
    binding = document_error_binding(source, pattern) if pattern is not None else None
    if binding is None or value is None:
        return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, "document error pattern must retain one auditable failure binding")]
    try:
        exits = tuple(exit_expressions(source, value))
    except UnsupportedExit as error:
        return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, str(error))]
    if not exits:
        return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, "document error arm has no auditable exit")]
    success_exits = 0
    for expression in exits:
        if call_name(source, expression) == "Err":
            continue
        if call_name(source, expression) != "Ok":
            return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, "document error arm has an untyped success exit")]
        success_exits += 1
        payload = unwrap_terminal(call_argument(expression))
        name = payload.child_by_field_name("name") if payload is not None and payload.type == "struct_expression" else None
        if name is None or syntax_compact(source, name) != "AnalysisPayloadV2":
            return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, "document error success must construct an analysis payload")]
        fields = {}
        body = payload.child_by_field_name("body")
        for child in body.named_children if body else ():
            if child.type == "field_initializer":
                field = child.child_by_field_name("field")
                field_value = child.child_by_field_name("value")
                if field and field_value:
                    fields[node_text(source, field)] = node_text(source, field_value)
                status = terminal_compact(source, field_value) if field and node_text(source, field) == "analysis_status" else None
                diagnostics = terminal_compact(source, field_value) if field and node_text(source, field) == "diagnostics" else None
                if status is not None:
                    fields["analysis_status"] = status
                if diagnostics is not None:
                    fields["diagnostics"] = diagnostics
        if (
            fields.get("analysis_status") != "AnalysisStatusV2::Error.keyword()"
            or fields.get("diagnostics") != f"{binding}.diagnostics().iter().map(WireDiagnosticV2::from).collect()"
        ):
            return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, "document error payload must report error and retain failure diagnostics")]
    if success_exits == 0:
        return [Finding("SH-10-document-error-payload", path, arm.start_point[0] + 1, "document error arm has no analysis payload exit")]
    return []


def scan_source(source: bytes, path: str, policy: dict, rust_parser) -> tuple[Finding, ...]:
    root = rust_parser.parse(source).root_node
    families = {family["type"]: family for family in policy["enum_families"]}
    candidates = tuple(production_nodes(root, source))
    findings = []
    targets = tuple(families) + tuple(policy["semantic_fields"]) + tuple(policy["wire_structs"])
    for node, attrs in candidates:
        if node.type == "function_item":
            findings.extend(analysis_status_findings(source, path, node))
        if node.type == "ERROR" or node.is_missing:
            context = node
            while context.parent is not None and context.type not in {"match_arm", "match_expression", "struct_item", "field_initializer"}:
                context = context.parent
            if any(token.encode() in source[context.start_byte:context.end_byte] for token in targets) or b"Err(" in source[context.start_byte:context.end_byte]:
                findings.append(Finding("SH-00-parse-error", path, node.start_point[0] + 1, "parse error overlaps governed source"))
        if node.type == "match_arm":
            aliases = aliases_in(source, node, set(families))
            findings.extend(document_error_payload_findings(source, path, node))
            pattern = node.child_by_field_name("pattern")
            value = node.child_by_field_name("value")
            if pattern is None or value is None:
                continue
            input_refs = references(source, pattern, aliases, set(families))
            negative = {(family, variant) for family, variant in input_refs if variant in families[family]["negative"]}
            try:
                exits = tuple(exit_expressions(source, value))
            except UnsupportedExit as error:
                if negative or is_error_pattern(source, pattern):
                    findings.append(Finding("SH-11-unsupported-exit", path, node.start_point[0] + 1, str(error)))
                exits = ()
            for expression in exits:
                output = terminal_variant(source, expression, aliases, families)
                positive = {output} if output is not None and output[1] in families[output[0]]["positive"] else set()
                forbidden = {(src, dst) for src in negative for dst in positive if [f"{src[0]}::{src[1]}", f"{dst[0]}::{dst[1]}"] not in policy["allowed_transitions"]}
                if forbidden:
                    findings.append(Finding("SH-01-outcome-laundering", path, node.start_point[0] + 1, f"{sorted(forbidden)}"))
                if negative and not has_negative_terminal(source, expression, aliases, families):
                    if call_name(source, expression) == "Ok" and any(families[family].get("ban_untyped_ok", False) for family, _ in negative):
                        findings.append(Finding("SH-02-untyped-success", path, node.start_point[0] + 1, f"{sorted(negative)} reaches untyped Ok"))
                    empty = empty_success(source, expression)
                    if any(families[family]["empty_is_success"] and empty is not None for family, _ in negative):
                        findings.append(Finding("SH-02-empty-success", path, node.start_point[0] + 1, f"{sorted(negative)} reaches {empty}"))
                    scalar = terminal_scalar(source, expression)
                    if any(scalar in families[family].get("forbidden_outputs", []) for family, _ in negative):
                        findings.append(Finding("SH-03-negative-exit", path, node.start_point[0] + 1, f"{sorted(negative)} reaches {scalar}"))
                    if scalar is None and any(families[family].get("forbidden_outputs") for family, _ in negative):
                        findings.append(Finding("SH-11-unsupported-exit", path, node.start_point[0] + 1, "scalar output cannot be classified from local AST"))
            # Pattern grammar identifies payload bindings. An explicit `_` bind
            # is a Rust discard and therefore outside this preservation rule.
            if negative:
                used = {node_text(source, child) for child in walk(value) if child.type == "identifier"}
                for name in pattern_bindings(source, pattern) - used:
                    findings.append(Finding("SH-04-binding-loss", path, node.start_point[0] + 1, f"negative payload binding {name} is unused"))
            document_payload_projection = document_error_binding(source, pattern) is not None and is_analysis_payload_return(source, node)
            if is_error_pattern(source, pattern) and not document_payload_projection:
                for expression in exits:
                    if call_name(source, expression) == "Ok" and not has_negative_terminal(source, expression, aliases, families):
                        findings.append(Finding("SH-05-error-to-ok", path, node.start_point[0] + 1, "Err is converted to Ok without a typed negative terminal"))
        elif node.type == "match_expression":
            aliases = aliases_in(source, node, set(families))
            body = node.child_by_field_name("body")
            if body is None:
                continue
            arms = [child for child in body.named_children if child.type == "match_arm"]
            governed = set()
            wildcard = None
            for arm in arms:
                pattern = arm.child_by_field_name("pattern")
                if pattern is None:
                    continue
                governed.update(family for family, _ in references(source, pattern, aliases, set(families)))
                if wildcard_pattern(pattern):
                    wildcard = arm
            if governed and wildcard is not None:
                value = wildcard.child_by_field_name("value")
                if value is not None:
                    try:
                        exits = tuple(exit_expressions(source, value))
                    except UnsupportedExit as error:
                        findings.append(Finding("SH-11-unsupported-exit", path, wildcard.start_point[0] + 1, str(error)))
                    else:
                        if not all(has_negative_terminal(source, expression, aliases, families) for expression in exits):
                            findings.append(Finding("SH-06-success-wildcard", path, wildcard.start_point[0] + 1, f"wildcard hides future variants of {sorted(governed)}"))
        elif node.type == "field_initializer":
            field = node.child_by_field_name("field")
            value = node.child_by_field_name("value")
            if field is not None and value is not None and node_text(source, field) in policy["semantic_fields"]:
                if any(call_name(source, child) and call_name(source, child).endswith(".ok") for child in walk(value)):
                    findings.append(Finding("SH-07-semantic-ok", path, node.start_point[0] + 1, "semantic field erases an error with .ok()"))
        elif node.type == "field_declaration" and any(re.search(r"#\s*\[\s*serde\s*\([^]]*\bdefault\b", attr) for attr in attrs):
            field = node.child_by_field_name("name")
            owner = node.parent.parent if node.parent else None
            owner_name = owner.child_by_field_name("name") if owner is not None else None
            if field is not None and owner_name is not None and node_text(source, owner_name) in policy["wire_structs"] and node_text(source, field) in policy["semantic_fields"]:
                findings.append(Finding("SH-08-semantic-serde-default", path, node.start_point[0] + 1, "wire semantic field defaults missing data"))
        elif node.type == "struct_item" and any(re.search(r"#\s*\[\s*serde\s*\([^]]*\bdefault\b", attr) for attr in attrs):
            name = node.child_by_field_name("name")
            if name is not None and node_text(source, name) in policy["wire_structs"]:
                for field in walk(node):
                    if field.type == "field_declaration" and (field_name := field.child_by_field_name("name")) is not None and node_text(source, field_name) in policy["semantic_fields"]:
                        findings.append(Finding("SH-08-semantic-serde-default", path, field.start_point[0] + 1, "wire struct defaults a semantic field"))
    return tuple(sorted(set(findings), key=lambda finding: (finding.path, finding.line, finding.rule, finding.detail)))


def validate_enum_definitions(root: Path, policy: dict) -> None:
    rust_parser = parser()
    for family in policy["enum_families"]:
        source_path = root / family["source"]
        source = source_path.read_bytes()
        tree = rust_parser.parse(source).root_node
        declarations = [node for node, _ in production_nodes(tree, source) if node.type == "enum_item" and (name := node.child_by_field_name("name")) is not None and node_text(source, name) == family["type"]]
        if len(declarations) != 1:
            raise ValueError(f"expected exactly one production enum {family['type']} in {family['source']}")
        body = declarations[0].child_by_field_name("body")
        variants = {node_text(source, name) for variant in body.named_children if variant.type == "enum_variant" and (name := variant.child_by_field_name("name")) is not None}
        configured = set(family["negative"]) | set(family["positive"])
        if configured != variants:
            raise ValueError(f"enum policy drift for {family['type']}: missing={sorted(variants - configured)}, stale={sorted(configured - variants)}")


def rust_paths(root: Path) -> tuple[str, ...]:
    result = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "crates/**/src/*.rs"], cwd=root, capture_output=True, check=True)
    return tuple(sorted({path for raw in result.stdout.split(b"\0") if raw and (path := raw.decode())}))


def scan_repository(root: Path = ROOT) -> tuple[Finding, ...]:
    # Imported after this module is initialized: the guard parser reuses the
    # Rust syntax helpers defined here, and test-only status needs owner syntax.
    from tools.lint.rust_module_guards import guarded_test_module_paths

    policy = read_policy()
    validate_enum_definitions(root, policy)
    local = threading.local()

    def scan(path: str) -> tuple[Finding, ...]:
        if not hasattr(local, "rust_parser"):
            local.rust_parser = parser()
        return scan_source((root / path).read_bytes(), path, policy, local.rust_parser)

    test_modules = guarded_test_module_paths(root)
    paths = tuple(path for path in rust_paths(root) if path not in test_modules)
    with ThreadPoolExecutor(max_workers=min(8, max(1, len(paths)))) as pool:
        return tuple(sorted((finding for batch in pool.map(scan, paths) for finding in batch), key=lambda finding: (finding.path, finding.line, finding.rule, finding.detail)))


def main() -> int:
    try:
        findings = scan_repository()
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"semantic honesty lint failed: {error}", file=sys.stderr)
        return 2
    for finding in findings:
        print(finding, file=sys.stderr)
    if findings:
        print(f"semantic honesty lint failed: {len(findings)} finding(s)", file=sys.stderr)
        return 1
    print("semantic honesty lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
