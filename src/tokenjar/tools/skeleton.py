from __future__ import annotations

import os
from typing import Any

from tokenjar.parsers.languages import parse_code
from tokenjar.utils.file_utils import detect_language, read_file_text
from tokenjar.utils.token_counter import estimate_tokens


def _build_skeleton(source_code: str, language: str) -> str:
    """Builds a skeleton of the source code by replacing function/method bodies with '...'."""
    tree = parse_code(source_code, language)
    if not tree:
        return source_code

    root_node = tree.root_node

    # We will collect byte ranges to replace with '...'
    replace_ranges = []

    def walk(node):
        node_type = node.type

        # Identify function-like bodies to replace
        if language == "python":
            if node_type == "function_definition":
                body = None
                for child in node.children:
                    if child.type == "block":
                        body = child
                        break
                if body:
                    # Check for docstring
                    start_byte = body.start_byte
                    first_stmt = body.children[0] if body.children else None
                    if first_stmt and first_stmt.type == "expression_statement":
                        if first_stmt.children and first_stmt.children[0].type == "string":
                            start_byte = first_stmt.end_byte

                    if start_byte < body.end_byte:
                        replace_ranges.append((start_byte, body.end_byte, " ...\n"))

        elif language == "ruby":
            if node_type in ["method", "singleton_method"]:
                body = None
                for child in node.children:
                    if child.type == "body_statement":
                        body = child
                        break
                if body and body.start_byte < body.end_byte:
                    replace_ranges.append((body.start_byte, body.end_byte, "\n    ...\n  "))

        elif language in [
            "javascript",
            "typescript",
            "tsx",
            "jsx",
            "go",
            "java",
            "c",
            "cpp",
            "rust",
            "c_sharp",
            "php",
            "kotlin",
        ]:
            # Generic body replacement for C-family and block-based languages
            if node_type in [
                "function_declaration",
                "method_definition",
                "arrow_function",
                "method_declaration",
                "function_definition",
                "function_item",
            ]:
                body = None
                for child in node.children:
                    if child.type in ["statement_block", "block", "compound_statement", "function_body"]:
                        body = child
                        break

                if body and body.start_byte < body.end_byte - 1:
                    # Keep braces if possible
                    replace_ranges.append((body.start_byte + 1, body.end_byte - 1, "\n  ...\n"))

        # Traverse children
        for child in node.children:
            walk(child)

    walk(root_node)

    if not replace_ranges:
        return source_code

    # Sort ranges by start_byte
    replace_ranges.sort(key=lambda x: x[0])

    # Resolve overlapping ranges (keep the outermost)
    filtered_ranges = []
    for r in replace_ranges:
        if not filtered_ranges:
            filtered_ranges.append(r)
        else:
            last_r = filtered_ranges[-1]
            if r[0] < last_r[1]:
                continue
            else:
                filtered_ranges.append(r)

    # Reconstruct the string
    source_bytes = source_code.encode("utf-8")
    result_bytes = bytearray()
    last_end = 0

    for start, end, replacement in filtered_ranges:
        result_bytes.extend(source_bytes[last_end:start])
        result_bytes.extend(replacement.encode("utf-8"))
        last_end = end

    result_bytes.extend(source_bytes[last_end:])

    return result_bytes.decode("utf-8")


def _find_symbol_range(source_code: str, language: str, symbol_name: str) -> tuple[int, int, str] | None:
    """Finds and returns (start_line, end_line, implementation) of a symbol by name."""
    tree = parse_code(source_code, language)
    if not tree:
        return None

    root_node = tree.root_node
    source_bytes = source_code.encode("utf-8")

    def get_node_name(node) -> str | None:
        for child in node.children:
            if child.type in [
                "identifier",
                "type_identifier",
                "property_identifier",
                "name",
                "simple_identifier",
                "constant",
                "field_identifier",
            ]:
                return source_bytes[child.start_byte : child.end_byte].decode("utf-8", errors="replace")
            elif child.type in ["function_declarator", "declarator"]:
                nested = get_node_name(child)
                if nested:
                    return nested
        return None

    def walk(node):
        if node.type in [
            "function_definition",
            "class_definition",
            "function_declaration",
            "class_declaration",
            "class_specifier",
            "struct_specifier",
            "method_definition",
            "method_declaration",
            "function_item",
            "struct_item",
            "method",
            "singleton_method",
            "class",
            "module",
            "interface_declaration",
        ]:
            name = get_node_name(node)
            if name == symbol_name:
                impl = source_bytes[node.start_byte : node.end_byte].decode("utf-8", errors="replace")
                return (node.start_point[0] + 1, node.end_point[0] + 1, impl)

        for child in node.children:
            res = walk(child)
            if res:
                return res
        return None

    return walk(root_node)


def _find_symbol(content: str, language: str, symbol_name: str) -> str:
    res = _find_symbol_range(content, language, symbol_name)
    if res:
        return res[2]
    return f"Symbol '{symbol_name}' not found."


def get_code_skeleton(file_path: str) -> str:
    """
    Extracts a structural skeleton from a source file.
    Use this INSTEAD of reading full files when you need to understand a file's API/structure.
    It keeps imports, classes, function signatures, and docstrings, but replaces bodies with '...'.
    """
    if not os.path.exists(file_path):
        return f"Error: File {file_path} not found."

    content = read_file_text(file_path)
    language = detect_language(file_path)

    if not language:
        return content

    skeleton = _build_skeleton(content, language)

    orig_tokens = estimate_tokens(content)
    skel_tokens = estimate_tokens(skeleton)
    savings_pct = 0
    if orig_tokens > 0:
        savings_pct = int(((orig_tokens - skel_tokens) / orig_tokens) * 100)

    skeleton += f"\n# TokenJar: {orig_tokens} → {skel_tokens} tokens ({savings_pct}% saved)"
    try:
        from tokenjar.telemetry.stats import tracker

        tracker.record_savings("skeleton", orig_tokens, skel_tokens)
    except Exception:
        pass
    return skeleton


def get_symbol(file_path: str, symbol_name: str) -> str:
    """
    Extracts the FULL implementation of a specific function, method, or class from a file by name.
    Useful after viewing a skeleton to fetch only the specific function needed.
    """
    if not os.path.exists(file_path):
        return f"Error: File {file_path} not found."

    content = read_file_text(file_path)
    language = detect_language(file_path)

    if not language:
        return f"Error: Could not detect language for {file_path}."

    return _find_symbol(content, language, symbol_name)


def register_skeleton_tools(mcp: Any) -> None:
    """Registers skeleton tools with the FastMCP application."""

    @mcp.tool(
        description="Extracts a structural skeleton from a source file, replacing bodies with '...'. Use this INSTEAD of reading full files to understand structure."
    )
    def tool_get_code_skeleton(file_path: str) -> str:
        return get_code_skeleton(file_path)

    @mcp.tool(
        description="Extracts the FULL implementation of a specific function, method, or class from a file by name."
    )
    def tool_get_symbol(file_path: str, symbol_name: str) -> str:
        return get_symbol(file_path, symbol_name)
