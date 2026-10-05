from __future__ import annotations

from tokenjar.tools.symbol_index import SymbolIndexer, find_symbol_global


def test_symbol_extraction():
    python_code = """
class UserService:
    def authenticate(self, user, token):
        pass

def global_helper():
    pass
"""
    symbols = SymbolIndexer.extract_symbols_from_code(python_code, "python", "src/user.py")
    names = [s.name for s in symbols]
    assert "UserService" in names
    assert "authenticate" in names
    assert "global_helper" in names


def test_symbol_extraction_multi_lang():
    ruby_code = """
class BillingManager
  def process_invoice(id)
    puts id
  end
end
"""
    symbols_rb = SymbolIndexer.extract_symbols_from_code(ruby_code, "ruby", "lib/billing.rb")
    names_rb = [s.name for s in symbols_rb]
    assert "BillingManager" in names_rb
    assert "process_invoice" in names_rb

    kotlin_code = """
class PushNotifier {
    fun sendAlert(msg: String) {
        println(msg)
    }
}
"""
    symbols_kt = SymbolIndexer.extract_symbols_from_code(kotlin_code, "kotlin", "app/Notifier.kt")
    names_kt = [s.name for s in symbols_kt]
    assert "PushNotifier" in names_kt
    assert "sendAlert" in names_kt

    go_code = """
package main
type Server struct{}
func (s *Server) Start() error { return nil }
func NewServer() *Server { return &Server{} }
"""
    symbols_go = SymbolIndexer.extract_symbols_from_code(go_code, "go", "main.go")
    names_go = [s.name for s in symbols_go]
    assert "Start" in names_go
    assert "NewServer" in names_go

    cpp_code = """
class DatabaseConnection {
public:
    void connect() {}
};
int query(int q) { return q; }
"""
    symbols_cpp = SymbolIndexer.extract_symbols_from_code(cpp_code, "cpp", "db.cpp")
    names_cpp = [s.name for s in symbols_cpp]
    assert "DatabaseConnection" in names_cpp
    assert "connect" in names_cpp
    assert "query" in names_cpp

    java_code = """
package com.example;
public class AuthProvider {
    public boolean verifyToken(String token) {
        return true;
    }
}
"""
    symbols_java = SymbolIndexer.extract_symbols_from_code(java_code, "java", "AuthProvider.java")
    names_java = [s.name for s in symbols_java]
    assert "AuthProvider" in names_java
    assert "verifyToken" in names_java


def test_find_symbol_global(tmp_path):
    src_file = tmp_path / "app.py"
    src_file.write_text(
        """
class OrderProcessor:
    def process_order(self, order_id):
        pass

def calculate_tax(amount):
    return amount * 0.20
""",
        encoding="utf-8",
    )

    out1 = find_symbol_global("OrderProcessor", str(tmp_path))
    assert "OrderProcessor" in out1
    assert "CLASS" in out1
    assert "app.py" in out1

    out2 = find_symbol_global("calculate_tax", str(tmp_path), exact=True)
    assert "calculate_tax" in out2
    assert "FUNCTION" in out2

    out3 = find_symbol_global("NonExistentSymbol", str(tmp_path))
    assert "No symbols found" in out3

    # Fuzzy suggestion when exact/substring match fails due to typo
    out_fuzzy = find_symbol_global("OrderProcesor", str(tmp_path))
    assert "Did you mean one of these symbols?" in out_fuzzy
    assert "OrderProcessor" in out_fuzzy


def test_compute_similarity():
    from tokenjar.tools.symbol_index import compute_similarity

    assert compute_similarity("OrderProcessor", "OrderProcessor") == 1.0
    assert compute_similarity("order_processor", "OrderProcessor") >= 0.95
    assert compute_similarity("OrderProcesor", "OrderProcessor") >= 0.85
    assert compute_similarity("validate_tokn", "validate_token") >= 0.85
    assert compute_similarity("something_completely_different", "OrderProcessor") < 0.40


def test_find_symbol_references(tmp_path):
    from tokenjar.tools.symbol_index import find_symbol_references

    services_dir = tmp_path / "services"
    services_dir.mkdir()
    billing_file = services_dir / "billing.py"
    billing_file.write_text(
        """
def calculate_discount(price, rate=0.1):
    return price * (1.0 - rate)
""",
        encoding="utf-8",
    )

    checkout_file = services_dir / "checkout.py"
    checkout_file.write_text(
        """
from services.billing import calculate_discount

def process_checkout(cart):
    discounted = calculate_discount(cart.total)
    return discounted
""",
        encoding="utf-8",
    )

    api_file = tmp_path / "main.py"
    api_file.write_text(
        """
from services.checkout import process_checkout

def run_app():
    pass
""",
        encoding="utf-8",
    )

    result = find_symbol_references("calculate_discount", str(tmp_path))
    assert "[REFERENCES]" in result
    assert "calculate_discount" in result
    assert "services/billing.py" in result
    assert "services/checkout.py" in result
    assert "[IMPORT]" in result or "[CALL]" in result

    # Nonexistent reference
    nonexistent = find_symbol_references("unknown_method_xyz", str(tmp_path))
    assert "No references or definitions found" in nonexistent


def test_persistent_symbol_cache_project_isolation(tmp_path):
    from tokenjar.cache.persistent_cache import PersistentCache

    db_file = tmp_path / "test_cache.db"
    cache = PersistentCache(db_path=db_file)

    proj_a = str(tmp_path / "project_a")
    proj_b = str(tmp_path / "project_b")

    cache.set_file_symbols(
        project_root=proj_a,
        file_path="src/auth.py",
        file_hash="hash_a",
        mtime=12345.0,
        symbols=[{"name": "login_user", "kind": "function", "line": 10, "signature": "def login_user():"}],
    )

    cache.set_file_symbols(
        project_root=proj_b,
        file_path="src/auth.py",
        file_hash="hash_b",
        mtime=54321.0,
        symbols=[{"name": "login_user", "kind": "function", "line": 20, "signature": "def login_user(token):"}],
    )

    # Search in proj_a
    matches_a = cache.search_symbols(proj_a, "login_user", exact=True)
    assert len(matches_a) == 1
    assert matches_a[0]["line"] == 10
    assert matches_a[0]["signature"] == "def login_user():"

    # Search in proj_b
    matches_b = cache.search_symbols(proj_b, "login_user", exact=True)
    assert len(matches_b) == 1
    assert matches_b[0]["line"] == 20
    assert matches_b[0]["signature"] == "def login_user(token):"

    # Search something only in proj_a
    cache.set_file_symbols(
        project_root=proj_a,
        file_path="src/secret.py",
        file_hash="hash_s",
        mtime=100.0,
        symbols=[{"name": "vault_key", "kind": "variable", "line": 1, "signature": "vault_key = 'x'"}],
    )
    assert len(cache.search_symbols(proj_a, "vault_key")) == 1
    assert len(cache.search_symbols(proj_b, "vault_key")) == 0
