from __future__ import annotations

from unittest.mock import patch

from tokenjar.tools.skeleton import _build_skeleton, _find_symbol, get_code_skeleton, get_symbol

PYTHON_SAMPLE = """
import os

class MyClass:
    def __init__(self, x: int):
        self.x = x
        self.y = 0
        self.z = "hello"
        for i in range(10):
            print(i)

    def do_something(self, y: str) -> bool:
        \"\"\"
        This does something.
        With a multiline docstring.
        \"\"\"
        print(y)
        if len(y) > 5:
            print("long string")
        else:
            print("short string")
        return True

def outer_func():
    a = 1
    b = 2
    c = a + b
    d = c * 2
    return d
"""


def test_build_skeleton_python():
    skeleton = _build_skeleton(PYTHON_SAMPLE, "python")
    assert "class MyClass:" in skeleton
    assert "def __init__(self, x: int):" in skeleton
    assert "def do_something(self, y: str) -> bool:" in skeleton
    assert "This does something." in skeleton
    assert "print(y)" not in skeleton
    assert "return True" not in skeleton
    assert "def outer_func():" in skeleton
    assert "return d" not in skeleton
    assert "..." in skeleton


def test_find_symbol_python():
    symbol = _find_symbol(PYTHON_SAMPLE, "python", "do_something")
    assert "def do_something(self, y: str) -> bool:" in symbol
    assert "print(y)" in symbol

    symbol_class = _find_symbol(PYTHON_SAMPLE, "python", "MyClass")
    assert "class MyClass:" in symbol_class
    assert "def __init__" in symbol_class

    symbol_typo = _find_symbol(PYTHON_SAMPLE, "python", "do_somethng")
    assert "Did you mean: do_something" in symbol_typo


def test_unknown_language_fallback():
    skeleton = _build_skeleton(PYTHON_SAMPLE, "unknown")
    assert skeleton == PYTHON_SAMPLE


@patch("tokenjar.tools.skeleton.os.path.exists")
def test_nonexistent_file(mock_exists):
    mock_exists.return_value = False
    res = get_code_skeleton("missing.py")
    assert "not found" in res.lower()

    res2 = get_symbol("missing.py", "foo")
    assert "not found" in res2.lower()


@patch("tokenjar.tools.skeleton.os.path.exists")
@patch("tokenjar.tools.skeleton.read_file_text")
@patch("tokenjar.tools.skeleton.detect_language")
def test_token_savings(mock_detect, mock_read, mock_exists):
    mock_exists.return_value = True
    mock_read.return_value = PYTHON_SAMPLE
    mock_detect.return_value = "python"

    res = get_code_skeleton("test.py")

    # We just want to check the token savings line is in output and it's > 0 (ideally >= 50 but our sample is small)
    assert "# TokenJar:" in res
    assert "saved" in res

    # Check savings > 50%
    # Extract pct
    pct_str = res.split("(")[-1].split("%")[0]
    pct = int(pct_str)
    assert pct >= 50
