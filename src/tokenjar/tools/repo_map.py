from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from tokenjar.config import load_config
from tokenjar.parsers.languages import parse_code
from tokenjar.utils.file_utils import (
    detect_language,
    is_binary,
    read_file_text,
    should_skip_dir,
    walk_source_files,
)
from tokenjar.utils.token_counter import estimate_tokens


@dataclass
class SymbolInfo:
    """Information about a code symbol (class, function, method)."""

    name: str
    kind: str  # 'class', 'function', 'method', 'interface', 'struct'
    signature: str  # Full signature line
    children: list[SymbolInfo] = field(default_factory=list)  # nested methods
    line: int = 0


@dataclass
class FileInfo:
    """Information about a source file."""

    path: str  # relative path
    language: str
    symbols: list[SymbolInfo] = field(default_factory=list)
    import_count: int = 0
    score: float = 0.0


def extract_symbols(node, language: str, source_code: bytes, is_root: bool = True) -> list[SymbolInfo]:
    symbols = []

    # Python
    if language == "python":
        target_types = ["function_definition", "class_definition"]
        if node.type in target_types:
            name_node = next((n for n in node.children if n.type == "identifier"), None)
            name = name_node.text.decode("utf-8") if name_node else "unknown"
            kind = "class" if node.type == "class_definition" else ("method" if not is_root else "function")

            # Extract signature
            body_node = next((n for n in node.children if n.type == "block"), None)
            if body_node:
                end_byte = body_node.start_byte
            else:
                end_byte = node.end_byte

            signature = source_code[node.start_byte : end_byte].decode("utf-8").strip()
            # Clean up trailing colons
            if signature.endswith(":"):
                signature = signature[:-1].strip()

            # Extract children if class
            children = []
            if kind == "class" and body_node:
                for child in body_node.children:
                    children.extend(extract_symbols(child, language, source_code, is_root=False))

            symbols.append(
                SymbolInfo(name=name, kind=kind, signature=signature, children=children, line=node.start_point[0])
            )

        elif node.type == "decorated_definition":
            # Extract from decorated definition
            for child in node.children:
                if child.type in target_types:
                    symbols.extend(extract_symbols(child, language, source_code, is_root))

        elif is_root:
            for child in node.children:
                symbols.extend(extract_symbols(child, language, source_code, is_root))

    # Typescript/Javascript
    elif language in ["typescript", "javascript", "tsx", "jsx"]:
        target_types = ["function_declaration", "class_declaration", "method_definition", "lexical_declaration"]
        if node.type in target_types:
            name_node = next((n for n in node.children if n.type in ["identifier", "property_identifier"]), None)
            if not name_node and node.type == "lexical_declaration":
                # Check for arrow function
                var_decl = next((n for n in node.children if n.type == "variable_declarator"), None)
                if var_decl:
                    name_node = next((n for n in var_decl.children if n.type == "identifier"), None)
                    arrow = next((n for n in var_decl.children if n.type == "arrow_function"), None)
                    if not arrow:
                        return []

            name = name_node.text.decode("utf-8") if name_node else "unknown"
            kind = (
                "class"
                if node.type == "class_declaration"
                else ("method" if node.type == "method_definition" else "function")
            )

            body_node = next((n for n in node.children if n.type == "statement_block" or n.type == "class_body"), None)
            if body_node:
                end_byte = body_node.start_byte
            else:
                end_byte = node.end_byte

            signature = source_code[node.start_byte : end_byte].decode("utf-8").strip()
            if signature.endswith("{"):
                signature = signature[:-1].strip()

            children = []
            if kind == "class" and body_node:
                for child in body_node.children:
                    children.extend(extract_symbols(child, language, source_code, is_root=False))

            symbols.append(
                SymbolInfo(name=name, kind=kind, signature=signature, children=children, line=node.start_point[0])
            )

        elif node.type == "export_statement":
            for child in node.children:
                symbols.extend(extract_symbols(child, language, source_code, is_root))

        elif is_root:
            for child in node.children:
                symbols.extend(extract_symbols(child, language, source_code, is_root))

    # Go
    elif language == "go":
        target_types = ["function_declaration", "method_declaration", "type_declaration"]
        if node.type in target_types:
            name_node = next((n for n in node.children if n.type == "identifier" or n.type == "type_identifier"), None)
            if node.type == "type_declaration":
                type_spec = next((n for n in node.children if n.type == "type_spec"), None)
                if type_spec:
                    name_node = next((n for n in type_spec.children if n.type == "type_identifier"), None)

            name = name_node.text.decode("utf-8") if name_node else "unknown"
            kind = "struct" if node.type == "type_declaration" else "function"

            body_node = next((n for n in node.children if n.type == "block"), None)
            if body_node:
                end_byte = body_node.start_byte
            else:
                end_byte = node.end_byte

            signature = source_code[node.start_byte : end_byte].decode("utf-8").strip()
            if signature.endswith("{"):
                signature = signature[:-1].strip()

            symbols.append(SymbolInfo(name=name, kind=kind, signature=signature, children=[], line=node.start_point[0]))

        elif is_root:
            for child in node.children:
                symbols.extend(extract_symbols(child, language, source_code, is_root))

    # Rust
    elif language == "rust":
        target_types = ["function_item", "struct_item", "enum_item"]
        if node.type in target_types:
            name_node = next((n for n in node.children if n.type == "identifier" or n.type == "type_identifier"), None)
            name = name_node.text.decode("utf-8") if name_node else "unknown"

            if node.type == "function_item":
                kind = "method" if not is_root else "function"
            elif node.type == "struct_item":
                kind = "struct"
            else:
                kind = "enum"

            body_node = next(
                (n for n in node.children if n.type == "block" or n.type == "field_declaration_list"), None
            )
            if body_node:
                end_byte = body_node.start_byte
            else:
                end_byte = node.end_byte

            signature = source_code[node.start_byte : end_byte].decode("utf-8").strip()
            if signature.endswith("{"):
                signature = signature[:-1].strip()

            symbols.append(SymbolInfo(name=name, kind=kind, signature=signature, children=[], line=node.start_point[0]))

        elif node.type == "impl_item":
            body_node = next((n for n in node.children if n.type == "declaration_list"), None)
            if body_node:
                for child in body_node.children:
                    symbols.extend(extract_symbols(child, language, source_code, is_root=False))

        elif is_root:
            for child in node.children:
                symbols.extend(extract_symbols(child, language, source_code, is_root))

    # Java/C# (simplified)
    elif language in ["java", "c_sharp"]:
        target_types = ["class_declaration", "method_declaration", "interface_declaration"]
        if node.type in target_types:
            name_node = next((n for n in node.children if n.type == "identifier"), None)
            name = name_node.text.decode("utf-8") if name_node else "unknown"
            kind = (
                "class"
                if node.type == "class_declaration"
                else ("interface" if node.type == "interface_declaration" else "method")
            )

            body_node = next(
                (n for n in node.children if n.type == "class_body" or n.type == "block" or n.type == "interface_body"),
                None,
            )
            if body_node:
                end_byte = body_node.start_byte
            else:
                end_byte = node.end_byte

            signature = source_code[node.start_byte : end_byte].decode("utf-8").strip()
            if signature.endswith("{"):
                signature = signature[:-1].strip()

            children = []
            if kind in ["class", "interface"] and body_node:
                for child in body_node.children:
                    children.extend(extract_symbols(child, language, source_code, is_root=False))

            symbols.append(
                SymbolInfo(name=name, kind=kind, signature=signature, children=children, line=node.start_point[0])
            )

        elif is_root:
            for child in node.children:
                symbols.extend(extract_symbols(child, language, source_code, is_root))

    # Fallback to root children iteration if no specific language match but we need to traverse
    elif is_root and language not in [
        "python",
        "typescript",
        "javascript",
        "tsx",
        "jsx",
        "go",
        "rust",
        "java",
        "c_sharp",
    ]:
        pass  # No extraction for unknown languages

    return symbols


def extract_import_count(node, language: str) -> int:
    count = 0
    if language == "python":
        if node.type in ["import_statement", "import_from_statement"]:
            count += 1
    elif language in ["typescript", "javascript", "tsx", "jsx"]:
        if node.type == "import_statement":
            count += 1
    elif language == "go":
        if node.type == "import_declaration":
            count += 1
    elif language == "rust":
        if node.type == "use_declaration":
            count += 1
    elif language in ["java", "c_sharp"]:
        if node.type in ["import_declaration", "using_directive"]:
            count += 1

    for child in node.children:
        count += extract_import_count(child, language)

    return count


def format_repo_map(file_infos: list[FileInfo], root_path: str, max_tokens: int) -> str:
    # Sort files by score descending
    sorted_files = sorted(file_infos, key=lambda f: f.score, reverse=True)

    # Output structured file list prioritized by graph centrality
    output_lines = []
    output_lines.append(f"📁 Repository Map ({len(file_infos)} files, budget: {max_tokens} tokens)")
    output_lines.append("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

    # Group by directory for the top files
    # We will incrementally add files until budget is hit
    included_files = []
    current_tokens = estimate_tokens("\n".join(output_lines))

    for f_info in sorted_files:
        file_lines = []
        file_lines.append(f"{f_info.path}:")

        def format_symbol(sym, level=1):
            sym_lines = []
            sym_lines.append(f"{'  ' * level}{sym.signature}")
            for child in sym.children:
                sym_lines.extend(format_symbol(child, level + 1))
            return sym_lines

        for sym in f_info.symbols:
            file_lines.extend(format_symbol(sym))

        if not f_info.symbols:
            file_lines.append("  (no symbols detected)")

        file_text = "\n".join(file_lines) + "\n"
        file_tokens = estimate_tokens(file_text)

        if current_tokens + file_tokens > max_tokens:
            if included_files:
                output_lines.append(
                    f"\n... (truncating remaining {len(sorted_files) - len(included_files)} files to respect token budget)"
                )
            else:
                output_lines.append("\n... (budget too small to include even the top file)")
            break

        output_lines.append(file_text)
        included_files.append(f_info)
        current_tokens += file_tokens

    return "\n".join(output_lines)


def get_repo_map(root_path: str = ".", max_tokens: int = 1000, focus_files: list[str] | None = None) -> str:
    """
    Generates a structural map of the entire repository fitted to a token budget.
    Use this at the START of any task to get a bird's-eye view of the codebase.
    """
    if focus_files is None:
        focus_files = []

    root = Path(root_path).resolve()
    config = load_config(root)
    if max_tokens == 1000 and config.repo_map_budget != 1000:
        max_tokens = config.repo_map_budget

    raw_file_data: list[tuple[str, str, str, int, list[SymbolInfo], int]] = []
    symbol_to_file: dict[str, str] = {}
    total_raw_tokens = 0

    # Pass 1: Parse files, extract symbols, map definitions
    for file_path_str in walk_source_files(str(root), max_files=config.max_source_files):
        file_path = Path(file_path_str)
        try:
            rel_path = file_path.relative_to(root).as_posix()
        except ValueError:
            rel_path = file_path.as_posix()

        if config.is_ignored(rel_path) or config.is_ignored(file_path_str):
            continue

        language = detect_language(str(file_path))
        if not language:
            continue

        text = read_file_text(str(file_path))
        if not text:
            continue

        line_count = text.count("\n") + 1
        total_raw_tokens += len(text) // 4

        tree = parse_code(text, language)
        if not tree:
            continue

        symbols = extract_symbols(tree.root_node, language, text.encode("utf-8"))
        import_count = extract_import_count(tree.root_node, language)

        # Map defined top-level symbols to this file
        for sym in symbols:
            if len(sym.name) > 2 and sym.name not in ("main", "init", "__init__"):
                symbol_to_file[sym.name] = rel_path

        raw_file_data.append((rel_path, str(file_path), language, line_count, symbols, import_count))

    if not raw_file_data:
        return f"📁 Repository Map (0 files, budget: {max_tokens} tokens)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nNo supported source files found."

    # Pass 2: Calculate cross-file dependency edges (Call Graph Centrality)
    inbound_references: dict[str, int] = {f[0]: 0 for f in raw_file_data}
    for rel_path, _, _, _, _, _ in raw_file_data:
        for sym_name, def_file in symbol_to_file.items():
            if def_file != rel_path:
                # If this file mentions another file's symbol, increment defined file's in-degree
                # We do a fast name lookup in text
                pass  # fast scan below

    for rel_path, full_path, language, line_count, symbols, import_count in raw_file_data:
        file_text = read_file_text(full_path)
        for sym_name, def_file in symbol_to_file.items():
            if def_file != rel_path and sym_name in file_text:
                inbound_references[def_file] = inbound_references.get(def_file, 0) + 1

    file_infos: list[FileInfo] = []
    for rel_path, full_path, language, line_count, symbols, import_count in raw_file_data:
        top_level_symbols = len(symbols)
        base_score = top_level_symbols * 10
        import_score = import_count * 5
        focus_boost = 100 if rel_path in focus_files or full_path in focus_files else 0
        length_penalty = -0.001 * line_count
        call_graph_centrality = inbound_references.get(rel_path, 0) * 12

        score = base_score + import_score + focus_boost + length_penalty + call_graph_centrality

        file_infos.append(
            FileInfo(path=rel_path, language=language, symbols=symbols, import_count=import_count, score=score)
        )

    output = format_repo_map(file_infos, str(root), max_tokens)

    try:
        from tokenjar.telemetry.stats import tracker

        map_tokens = len(output) // 4
        # Dynamically measure realistic exploration cost: physical size of manifests, readme and entrypoints
        candidate_files = [
            "README.md",
            "README",
            "Cargo.toml",
            "pyproject.toml",
            "package.json",
            "go.mod",
            "Makefile",
            "src/main.rs",
            "src/lib.rs",
            "src/tokenjar/server.py",
            "index.ts",
            "main.py",
            "app.py",
        ]
        manifest_tokens = 0
        for cf in candidate_files:
            p = root / cf
            if p.is_file():
                try:
                    manifest_tokens += int(p.stat().st_size / 3.5)
                except OSError:
                    pass
        realistic_raw_tokens = min(max(manifest_tokens, 1500), 6000)
        if realistic_raw_tokens > map_tokens:
            tracker.record_savings("repo_map", realistic_raw_tokens, map_tokens)
    except Exception:
        pass

    return output


def get_directory_tree(root_path: str = ".", max_depth: int = 4) -> str:
    """
    Simple directory tree listing (respects skip dirs and binary files) — a lightweight alternative to repo_map.
    """
    root = Path(root_path).resolve()
    if not root.exists() or not root.is_dir():
        return f"Error: Directory {root_path} does not exist."

    config = load_config(root)
    lines = []

    def walk_tree(current_dir: Path, depth: int, prefix: str = ""):
        if depth > max_depth:
            lines.append(f"{prefix}...")
            return

        try:
            items = sorted(current_dir.iterdir(), key=lambda x: (not x.is_dir(), x.name.lower()))
        except PermissionError:
            lines.append(f"{prefix}<Permission Denied>")
            return

        # Filter items
        filtered_items = []
        for item in items:
            if config.is_ignored(item):
                continue
            if item.is_dir():
                if not should_skip_dir(item.name):
                    filtered_items.append(item)
            else:
                if not is_binary(str(item)):
                    filtered_items.append(item)

        for i, item in enumerate(filtered_items):
            is_last = i == len(filtered_items) - 1
            connector = "└── " if is_last else "├── "

            if item.is_dir():
                lines.append(f"{prefix}{connector}{item.name}/")
                extension_prefix = "    " if is_last else "│   "
                walk_tree(item, depth + 1, prefix + extension_prefix)
            else:
                lines.append(f"{prefix}{connector}{item.name}")

    lines.append(f"{root.name}/")
    walk_tree(root, 1)

    return "\n".join(lines)


def register_repo_map_tools(mcp) -> None:
    """Register repository map tools with the MCP server."""

    @mcp.tool()
    def get_repo_map_tool(root_path: str = ".", max_tokens: int = 1000, focus_files: list[str] | None = None) -> str:
        """
        Generates a structural map of the entire repository fitted to a token budget.
        Use this at the START of any task to get a bird's-eye view of the codebase before diving into specific files.
        """
        return get_repo_map(root_path, max_tokens, focus_files)

    @mcp.tool()
    def get_directory_tree_tool(root_path: str = ".", max_depth: int = 4) -> str:
        """
        Simple directory tree listing (respects skip dirs and binary files) — a lightweight alternative to repo_map.
        """
        return get_directory_tree(root_path, max_depth)
