from __future__ import annotations

from tokenjar.cache.session_cache import CacheStatus, SessionCache
from tokenjar.config import load_config
from tokenjar.utils.file_utils import is_binary, read_file_text
from tokenjar.utils.token_counter import format_savings

_cache = SessionCache()
_config = load_config()

MAX_OUTPUT_LINES = 40
MAX_OUTPUT_BYTES = 2500


def read_file_smart(
    file_path: str,
    force_full: bool = False,
    query: str | None = None,
    start_line: int | None = None,
    end_line: int | None = None,
    symbol: str | None = None,
) -> str:
    """Intelligently read a file with session-level caching, targeted line slicing, and lockfile protection.

    Use this tool instead of native file reading for iterative editing workflows.
    It returns the full file content on the first read. On subsequent reads, if the
    file is unchanged, it returns a very short cached message. If changed, it returns
    a unified diff of the modifications, saving thousands of tokens.

    Supports start_line and end_line (1-indexed, inclusive) to surgically inspect specific
    code ranges without dumping entire large files into context.

    Supports symbol to extract a specific function, class, or method directly with line numbers,
    avoiding 2-step lookups and preventing large output truncation.

    Auto-generated lockfiles (package-lock.json, Cargo.lock, poetry.lock, yarn.lock, etc.)
    and minified assets are automatically shielded to protect context windows from compaction.

    Args:
        file_path: Absolute or relative path to the file.
        force_full: If True, bypasses cache and lockfile shielding, returning raw full content.
        query: Optional package name or keyword to surgically query inside lockfiles or large assets.
        start_line: Optional starting line number (1-indexed, inclusive).
        end_line: Optional ending line number (1-indexed, inclusive).
        symbol: Optional name of a function, class, or method to extract directly.

    Returns:
        The full content, a short cached message, a unified diff, or a shielded summary.
    """
    import os

    if os.path.isdir(file_path):
        return (
            f"[TOKENJAR] '{file_path}' is a directory, not a file. "
            f"Use 'get_directory_tree_tool' or 'get_repo_map_tool' to explore directory contents."
        )

    if not force_full:
        try:
            if os.path.exists(file_path) and os.path.getsize(file_path) > _config.max_cacheable_bytes:
                size_mb = os.path.getsize(file_path) / (1024 * 1024)
                limit_mb = _config.max_cacheable_bytes / (1024 * 1024)
                return (
                    f"[TOKENJAR] File '{file_path}' ({size_mb:.2f} MB) exceeds maximum cacheable limit ({limit_mb:.2f} MB). "
                    f"Reading this entirely into context would consume massive tokens. "
                    f"Pass force_full=True if you explicitly need the raw content."
                )
        except Exception:
            pass

    if not force_full and _config.is_ignored(file_path):
        base_name = os.path.basename(file_path)
        return (
            f"[TOKENJAR SECURITY] '{base_name}' matches security ignore patterns "
            f"(credentials/secrets/exclusions). Pass force_full=True if you explicitly "
            f"need to read this file."
        )

    if not force_full and is_binary(file_path):
        return f"[TOKENJAR] Binary file '{file_path}' skipped to prevent context window corruption."

    try:
        content = read_file_text(file_path)
    except Exception as e:
        return f"Error reading file {file_path}: {e}"

    # Direct symbol slicing support (targeted AST extraction)
    if symbol and symbol.strip():
        sym = symbol.strip()
        lines = content.splitlines()
        total = len(lines)
        from tokenjar.utils.file_utils import detect_language

        language = detect_language(file_path)
        if language:
            from tokenjar.tools.skeleton import _find_symbol_range

            found = _find_symbol_range(content, language, sym)
            if found:
                start, end, _impl = found
                full_slice = [f"{i}: {line}" for i, line in enumerate(lines[start - 1 : end], start=start)]
                slice_key = f"{file_path}#sym:{sym}"
                header = f"[TOKENJAR] Symbol '{sym}' found at lines {start}-{end} of {total} in '{file_path}':\n"

                raw_sliced = header + "\n".join(full_slice)
                cached = _cache.get(slice_key, raw_sliced)
                if cached.status == CacheStatus.UNCHANGED:
                    savings = format_savings(content, cached.content)
                    try:
                        from tokenjar.telemetry.stats import tracker

                        tracker.record_savings("cache", len(content) // 4, len(cached.content) // 4)
                    except Exception:
                        pass
                    return f"{cached.content}\n\nToken savings: {savings}"

                take_count = 0
                byte_accum = 0
                for line_item in full_slice:
                    if take_count >= MAX_OUTPUT_LINES or (take_count > 0 and byte_accum + len(line_item) > MAX_OUTPUT_BYTES):
                        break
                    byte_accum += len(line_item) + 1
                    take_count += 1
                if take_count == 0:
                    take_count = min(1, len(full_slice))

                if take_count < len(full_slice) and not force_full:
                    p_end = start + take_count - 1
                    display = full_slice[:take_count]
                    notice = (
                        f"\n\n[TOKENJAR PAGINATION] Showing lines {start}-{p_end} of {total}. (Remaining lines truncated to protect context window)\n"
                        f"👉 To read next slice, call read_file_smart with start_line={p_end + 1}, end_line={min(end, p_end + MAX_OUTPUT_LINES)}."
                    )
                    sliced_content = header + "\n".join(display) + notice
                else:
                    sliced_content = raw_sliced

                orig_tok = len(content) // 4
                opt_tok = len(sliced_content) // 4
                if orig_tok > opt_tok:
                    savings = format_savings(content, sliced_content)
                    try:
                        from tokenjar.telemetry.stats import tracker

                        tracker.record_savings("slice", orig_tok, opt_tok)
                    except Exception:
                        pass
                    return f"{sliced_content}\n\nToken savings: {savings}"
                return sliced_content

        # Fallback for non-AST files or if symbol not found by AST: search line containing sym
        match_line = None
        for i, line in enumerate(lines):
            if sym in line:
                match_line = i + 1
                break

        if match_line is not None:
            start = match_line
            end = min(total, start + 40)
            full_slice = [f"{i}: {line}" for i, line in enumerate(lines[start - 1 : end], start=start)]
            slice_key = f"{file_path}#sym:{sym}"
            header = f"[TOKENJAR] Symbol '{sym}' matched at line {start} (showing lines {start}-{end} of {total}) in '{file_path}':\n"
            raw_sliced = header + "\n".join(full_slice)

            cached = _cache.get(slice_key, raw_sliced)
            if cached.status == CacheStatus.UNCHANGED:
                savings = format_savings(content, cached.content)
                try:
                    from tokenjar.telemetry.stats import tracker

                    tracker.record_savings("cache", len(content) // 4, len(cached.content) // 4)
                except Exception:
                    pass
                return f"{cached.content}\n\nToken savings: {savings}"

            orig_tok = len(content) // 4
            opt_tok = len(raw_sliced) // 4
            if orig_tok > opt_tok:
                savings = format_savings(content, raw_sliced)
                try:
                    from tokenjar.telemetry.stats import tracker

                    tracker.record_savings("slice", orig_tok, opt_tok)
                except Exception:
                    pass
                return f"{raw_sliced}\n\nToken savings: {savings}"
            return raw_sliced

        return (
            f"[TOKENJAR] Symbol '{sym}' was not found in '{file_path}'. "
            f"Tip: use 'tool_get_code_skeleton' to see available functions/classes or pass start_line and end_line."
        )

    # Line slicing support (targeted range extraction)
    if start_line is not None or end_line is not None:
        lines = content.splitlines()
        total = len(lines)
        if total == 0:
            return f"[TOKENJAR] File '{file_path}' is empty (0 lines)."

        start = max(1, start_line) if start_line is not None else 1
        end = min(total, end_line) if end_line is not None else total

        if start > total:
            return f"[TOKENJAR] Requested start_line {start} exceeds total line count ({total}) in '{file_path}'."
        if start > end:
            return f"[TOKENJAR] Invalid line range: start_line ({start}) cannot be greater than end_line ({end})."

        full_slice = [f"{i}: {line}" for i, line in enumerate(lines[start - 1 : end], start=start)]
        slice_key = f"{file_path}#L{start}-{end}"
        header = f"[TOKENJAR] Lines {start}-{end} of {total} in '{file_path}':\n"
        raw_sliced = header + "\n".join(full_slice)

        cached = _cache.get(slice_key, raw_sliced)
        if cached.status == CacheStatus.UNCHANGED:
            savings = format_savings(content, cached.content)
            try:
                from tokenjar.telemetry.stats import tracker

                tracker.record_savings("cache", len(content) // 4, len(cached.content) // 4)
            except Exception:
                pass
            return f"{cached.content}\n\nToken savings: {savings}"

        take_count = 0
        byte_accum = 0
        for line_item in full_slice:
            if take_count >= MAX_OUTPUT_LINES or (take_count > 0 and byte_accum + len(line_item) > MAX_OUTPUT_BYTES):
                break
            byte_accum += len(line_item) + 1
            take_count += 1
        if take_count == 0:
            take_count = min(1, len(full_slice))

        if take_count < len(full_slice) and not force_full:
            p_end = start + take_count - 1
            display = full_slice[:take_count]
            notice = (
                f"\n\n[TOKENJAR PAGINATION] Showing lines {start}-{p_end} of {total}. (Remaining lines truncated to protect context window)\n"
                f"👉 To read next slice, call read_file_smart with start_line={p_end + 1}, end_line={min(end, p_end + MAX_OUTPUT_LINES)}."
            )
            sliced_content = header + "\n".join(display) + notice
        else:
            sliced_content = raw_sliced

        orig_tok = len(content) // 4
        opt_tok = len(sliced_content) // 4
        if orig_tok > opt_tok:
            savings = format_savings(content, sliced_content)
            try:
                from tokenjar.telemetry.stats import tracker

                tracker.record_savings("slice", orig_tok, opt_tok)
            except Exception:
                pass
            return f"{sliced_content}\n\nToken savings: {savings}"
        return sliced_content

    if force_full:
        return content

    # Lockfile & giant asset protection
    if _config.is_lockfile(file_path):
        from tokenjar.filters.lockfile import process_lockfile

        return process_lockfile(file_path, content, query=query)

    result = _cache.get(file_path, content)

    if result.status == CacheStatus.FIRST_READ:
        lines = content.splitlines()
        total = len(lines)
        is_lock = any(
            file_path.endswith(s)
            for s in (
                "package-lock.json",
                "yarn.lock",
                "pnpm-lock.yaml",
                "Cargo.lock",
                "poetry.lock",
                "composer.lock",
                "Gemfile.lock",
            )
        )
        take_count = 0
        byte_accum = 0
        for line_item in lines:
            if take_count >= MAX_OUTPUT_LINES or (take_count > 0 and byte_accum + len(line_item) > MAX_OUTPUT_BYTES):
                break
            byte_accum += len(line_item) + 1
            take_count += 1

        if take_count < total and not force_full and not is_lock:
            p_end = take_count
            display = [f"{i}: {line}" for i, line in enumerate(lines[:p_end], start=1)]
            header = f"[TOKENJAR] Showing lines 1-{p_end} of {total} in '{file_path}':\n"
            notice = (
                f"\n\n[TOKENJAR PAGINATION] Showing lines 1-{p_end} of {total}. (Remaining lines truncated to protect context window)\n"
                f"👉 To read next slice, call read_file_smart with start_line={p_end + 1}, end_line={min(total, p_end + MAX_OUTPUT_LINES)} (or pass force_full=true)."
            )
            paginated = header + "\n".join(display) + notice
            orig_tok = len(content) // 4
            opt_tok = len(paginated) // 4
            if orig_tok > opt_tok:
                savings = format_savings(content, paginated)
                try:
                    from tokenjar.telemetry.stats import tracker

                    tracker.record_savings("slice", orig_tok, opt_tok)
                except Exception:
                    pass
                return f"{paginated}\n\nToken savings: {savings}"
            return paginated
        return result.content

    savings = format_savings(content, result.content)
    try:
        from tokenjar.telemetry.stats import tracker

        tracker.record_savings("cache", result.original_tokens, result.optimized_tokens)
    except Exception:
        pass
    return f"{result.content}\n\nToken savings: {savings}"


def cache_stats() -> str:
    """Get a summary of the smart reader cache performance.

    Returns:
        A string detailing cache hits, misses, diffs, and overall reads.
    """
    return _cache.get_stats().summary()


def register_smart_reader_tools(mcp) -> None:
    """Register smart reader tools with the MCP server."""

    @mcp.tool()
    def read_file_smart(
        file_path: str,
        force_full: bool = False,
        query: str | None = None,
        start_line: int | None = None,
        end_line: int | None = None,
        symbol: str | None = None,
    ) -> str:
        """Intelligently read a file with session-level caching, targeted line slicing, lockfile protection, and instant symbol extraction."""
        return _cache_read_impl(
            file_path,
            force_full=force_full,
            query=query,
            start_line=start_line,
            end_line=end_line,
            symbol=symbol,
        )

    @mcp.tool()
    def cache_stats() -> str:
        """Get a summary of the smart reader cache performance."""
        return _cache_stats_impl()


_cache_read_impl = read_file_smart
_cache_stats_impl = cache_stats
