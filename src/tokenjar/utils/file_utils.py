"""File utility functions for TokenJar.

Handles file reading, encoding detection, gitignore respect,
and language detection from file extensions.
"""

from __future__ import annotations

import os
from pathlib import Path

# Map file extensions to Tree-sitter language names
EXTENSION_TO_LANGUAGE: dict[str, str] = {
    ".py": "python",
    ".pyi": "python",
    ".ts": "typescript",
    ".tsx": "tsx",
    ".js": "javascript",
    ".jsx": "javascript",
    ".mjs": "javascript",
    ".cjs": "javascript",
    ".go": "go",
    ".rs": "rust",
    ".java": "java",
    ".kt": "kotlin",
    ".kts": "kotlin",
    ".cs": "c_sharp",
    ".c": "c",
    ".h": "c",
    ".cpp": "cpp",
    ".cc": "cpp",
    ".cxx": "cpp",
    ".hpp": "cpp",
    ".hxx": "cpp",
    ".rb": "ruby",
    ".php": "php",
    ".swift": "swift",
    ".scala": "scala",
    ".r": "r",
    ".R": "r",
    ".lua": "lua",
    ".zig": "zig",
    ".ex": "elixir",
    ".exs": "elixir",
    ".erl": "erlang",
    ".hs": "haskell",
    ".ml": "ocaml",
    ".mli": "ocaml",
    ".pl": "perl",
    ".pm": "perl",
    ".sh": "bash",
    ".bash": "bash",
    ".zsh": "bash",
    ".dart": "dart",
    ".jl": "julia",
    ".sql": "sql",
    ".html": "html",
    ".htm": "html",
    ".css": "css",
    ".scss": "scss",
    ".yaml": "yaml",
    ".yml": "yaml",
    ".toml": "toml",
    ".json": "json",
    ".xml": "xml",
    ".md": "markdown",
    ".rst": "rst",
    ".tf": "hcl",
    ".hcl": "hcl",
    ".proto": "proto",
    ".cmake": "cmake",
    ".Makefile": "make",
    ".mk": "make",
    ".dockerfile": "dockerfile",
    ".Dockerfile": "dockerfile",
}

# Binary/non-text extensions to skip
BINARY_EXTENSIONS: set[str] = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".bmp",
    ".ico",
    ".svg",
    ".webp",
    ".mp3",
    ".mp4",
    ".wav",
    ".avi",
    ".mov",
    ".mkv",
    ".webm",
    ".zip",
    ".tar",
    ".gz",
    ".bz2",
    ".xz",
    ".7z",
    ".rar",
    ".exe",
    ".dll",
    ".so",
    ".dylib",
    ".bin",
    ".o",
    ".a",
    ".pdf",
    ".doc",
    ".docx",
    ".xls",
    ".xlsx",
    ".ppt",
    ".pptx",
    ".woff",
    ".woff2",
    ".ttf",
    ".otf",
    ".eot",
    ".pyc",
    ".pyo",
    ".class",
    ".jar",
    ".db",
    ".sqlite",
    ".sqlite3",
}

# Directories to always skip
SKIP_DIRS: set[str] = {
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".nox",
    ".venv",
    "venv",
    "env",
    ".env",
    "dist",
    "build",
    "target",
    "out",
    "bin",
    "obj",
    ".next",
    ".nuxt",
    ".output",
    "coverage",
    "htmlcov",
    ".coverage",
    ".idea",
    ".vscode",
    ".vs",
    ".terraform",
    "data",
    "logs",
}


def detect_language(file_path: str) -> str | None:
    """Detect the programming language from a file's extension.

    Returns the Tree-sitter language name, or None if unknown.
    """
    ext = Path(file_path).suffix.lower()
    # Handle special filenames
    basename = Path(file_path).name.lower()
    if basename in ("makefile", "gnumakefile"):
        return "make"
    if basename == "dockerfile" or basename.startswith("dockerfile."):
        return "dockerfile"
    if basename in ("cmakelists.txt",):
        return "cmake"

    return EXTENSION_TO_LANGUAGE.get(ext)


def is_binary(file_path: str) -> bool:
    """Check if a file is binary based on extension."""
    ext = Path(file_path).suffix.lower()
    return ext in BINARY_EXTENSIONS


def should_skip_dir(dir_name: str) -> bool:
    """Check if a directory should be skipped during traversal."""
    return dir_name in SKIP_DIRS or dir_name.startswith(".")


def read_file_text(file_path: str) -> str:
    """Read a text file with encoding fallback.

    Tries UTF-8 first, then falls back to latin-1 (which never fails).
    """
    path = Path(file_path)
    if not path.exists():
        raise FileNotFoundError(f"File not found: {file_path}")
    if not path.is_file():
        raise IsADirectoryError(f"Not a file: {file_path}")

    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="latin-1")


def walk_source_files(
    root: str,
    max_files: int | None = None,
) -> list[str]:
    """Walk a directory tree and collect source code file paths.

    Respects SKIP_DIRS and BINARY_EXTENSIONS filters.
    Returns absolute paths, limited to max_files or configured max_source_files.
    """
    root_path = Path(root).resolve()
    files: list[str] = []

    if max_files is not None:
        effective_limit = max_files
    else:
        try:
            from tokenjar.config import load_config

            effective_limit = load_config(root_path).max_source_files
        except Exception:
            effective_limit = 5000

    for dirpath, dirnames, filenames in os.walk(root_path):
        # Filter out directories we should skip (modifies in-place)
        dirnames[:] = [d for d in dirnames if not should_skip_dir(d)]

        for filename in sorted(filenames):
            if len(files) >= effective_limit:
                return files

            file_path = os.path.join(dirpath, filename)
            if is_binary(file_path):
                continue

            # Only include files we can detect a language for, or common text files
            ext = Path(filename).suffix.lower()
            if ext in EXTENSION_TO_LANGUAGE or ext in {".txt", ".cfg", ".ini"}:
                files.append(file_path)

    return files
