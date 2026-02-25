"""
OverDrive InCode SDK — Python Wrapper
Embeddable document database. Like SQLite for JSON.

Usage:
    from overdrive import OverDrive

    db = OverDrive("myapp.odb")
    db.create_table("users")
    db.insert("users", {"name": "Alice", "age": 30})
    results = db.query("SELECT * FROM users WHERE age > 25")
    db.close()
"""

import ctypes
import json
import os
import platform
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Union


def _find_library() -> str:
    """Find the overdrive shared library."""
    system = platform.system()
    if system == "Windows":
        lib_name = "overdrive.dll"
    elif system == "Darwin":
        lib_name = "liboverdrive.dylib"
    else:
        lib_name = "liboverdrive.so"

    # Search paths
    search_paths = [
        Path(__file__).parent / lib_name,
        Path(__file__).parent / "lib" / lib_name,
        Path(__file__).parent.parent / "target" / "release" / lib_name,
        Path(__file__).parent.parent.parent / "target" / "release" / lib_name,
    ]

    for path in search_paths:
        if path.exists():
            return str(path)

    # Try system path
    return lib_name


class _Native:
    """Low-level ctypes bindings to liboverdrive."""

    def __init__(self):
        lib_path = _find_library()
        self.lib = ctypes.cdll.LoadLibrary(lib_path)

        # Lifecycle
        self.lib.overdrive_open.argtypes = [ctypes.c_char_p]
        self.lib.overdrive_open.restype = ctypes.c_void_p

        self.lib.overdrive_close.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_close.restype = None

        self.lib.overdrive_sync.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_sync.restype = None

        # Tables
        self.lib.overdrive_create_table.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.overdrive_create_table.restype = ctypes.c_int

        self.lib.overdrive_drop_table.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.overdrive_drop_table.restype = ctypes.c_int

        self.lib.overdrive_list_tables.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_list_tables.restype = ctypes.c_void_p

        self.lib.overdrive_table_exists.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.overdrive_table_exists.restype = ctypes.c_int

        # CRUD
        self.lib.overdrive_insert.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        self.lib.overdrive_insert.restype = ctypes.c_void_p

        self.lib.overdrive_get.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        self.lib.overdrive_get.restype = ctypes.c_void_p

        self.lib.overdrive_update.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
        self.lib.overdrive_update.restype = ctypes.c_int

        self.lib.overdrive_delete.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        self.lib.overdrive_delete.restype = ctypes.c_int

        self.lib.overdrive_count.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.overdrive_count.restype = ctypes.c_int

        # Query
        self.lib.overdrive_query.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.overdrive_query.restype = ctypes.c_void_p

        self.lib.overdrive_search.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        self.lib.overdrive_search.restype = ctypes.c_void_p

        # Utility
        self.lib.overdrive_last_error.argtypes = []
        self.lib.overdrive_last_error.restype = ctypes.c_char_p

        self.lib.overdrive_free_string.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_free_string.restype = None

        self.lib.overdrive_version.argtypes = []
        self.lib.overdrive_version.restype = ctypes.c_char_p

        # Phase 5: MVCC Transactions
        self.lib.overdrive_begin_transaction.argtypes = [ctypes.c_void_p, ctypes.c_int]
        self.lib.overdrive_begin_transaction.restype = ctypes.c_uint64

        self.lib.overdrive_commit_transaction.argtypes = [ctypes.c_void_p, ctypes.c_uint64]
        self.lib.overdrive_commit_transaction.restype = ctypes.c_int

        self.lib.overdrive_abort_transaction.argtypes = [ctypes.c_void_p, ctypes.c_uint64]
        self.lib.overdrive_abort_transaction.restype = ctypes.c_int

        # Phase 5: Integrity & Stats
        self.lib.overdrive_verify_integrity.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_verify_integrity.restype = ctypes.c_void_p

        self.lib.overdrive_stats.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_stats.restype = ctypes.c_void_p


_native = None


def _get_native() -> _Native:
    global _native
    if _native is None:
        _native = _Native()
    return _native


def _check_error(native: _Native):
    """Check for and raise the last error."""
    err = native.lib.overdrive_last_error()
    if err:
        raise OverDriveError(err.decode("utf-8"))


def _read_and_free(native: _Native, ptr) -> Optional[str]:
    """Read a C string and free it."""
    if not ptr:
        return None
    try:
        s = ctypes.cast(ptr, ctypes.c_char_p).value
        return s.decode("utf-8") if s else None
    finally:
        native.lib.overdrive_free_string(ptr)


class OverDriveError(Exception):
    """Exception raised by OverDrive operations."""
    pass


class OverDrive:
    """
    OverDrive InCode SDK — Embeddable Document Database

    Usage:
        db = OverDrive("myapp.odb")
        db.create_table("users")
        db.insert("users", {"name": "Alice", "age": 30})
        results = db.query("SELECT * FROM users WHERE age > 25")
        db.close()

    Context manager:
        with OverDrive("myapp.odb") as db:
            db.insert("logs", {"event": "startup"})
    """

    def __init__(self, path: str):
        self._native = _get_native()
        self._handle = self._native.lib.overdrive_open(path.encode("utf-8"))
        if not self._handle:
            _check_error(self._native)
            raise OverDriveError(f"Failed to open database: {path}")
        self._path = path

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def __del__(self):
        try:
            self.close()
        except Exception:
            pass

    # ── Lifecycle ───────────────────────────────

    def close(self):
        """Close the database and release resources."""
        if self._handle:
            self._native.lib.overdrive_close(self._handle)
            self._handle = None

    def sync(self):
        """Force sync data to disk."""
        self._ensure_open()
        self._native.lib.overdrive_sync(self._handle)

    @property
    def path(self) -> str:
        return self._path

    @staticmethod
    def version() -> str:
        native = _get_native()
        v = native.lib.overdrive_version()
        return v.decode("utf-8") if v else "unknown"

    # ── Tables ──────────────────────────────────

    def create_table(self, name: str):
        """Create a new table."""
        self._ensure_open()
        result = self._native.lib.overdrive_create_table(self._handle, name.encode("utf-8"))
        if result != 0:
            _check_error(self._native)

    def drop_table(self, name: str):
        """Drop a table."""
        self._ensure_open()
        result = self._native.lib.overdrive_drop_table(self._handle, name.encode("utf-8"))
        if result != 0:
            _check_error(self._native)

    def list_tables(self) -> List[str]:
        """List all tables."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_list_tables(self._handle)
        if not ptr:
            _check_error(self._native)
            return []
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else []

    def table_exists(self, name: str) -> bool:
        """Check if a table exists."""
        self._ensure_open()
        result = self._native.lib.overdrive_table_exists(self._handle, name.encode("utf-8"))
        return result == 1

    # ── CRUD ────────────────────────────────────

    def insert(self, table: str, doc: Dict[str, Any]) -> str:
        """Insert a document. Returns the _id."""
        self._ensure_open()
        json_str = json.dumps(doc)
        ptr = self._native.lib.overdrive_insert(
            self._handle,
            table.encode("utf-8"),
            json_str.encode("utf-8"),
        )
        if not ptr:
            _check_error(self._native)
            raise OverDriveError("Insert failed")
        result = _read_and_free(self._native, ptr)
        return result or ""

    def insert_many(self, table: str, docs: List[Dict[str, Any]]) -> List[str]:
        """Insert multiple documents. Returns list of _ids."""
        return [self.insert(table, doc) for doc in docs]

    def get(self, table: str, id: str) -> Optional[Dict[str, Any]]:
        """Get a document by _id."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_get(
            self._handle,
            table.encode("utf-8"),
            id.encode("utf-8"),
        )
        if not ptr:
            return None
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else None

    def update(self, table: str, id: str, updates: Dict[str, Any]) -> bool:
        """Update a document by _id."""
        self._ensure_open()
        json_str = json.dumps(updates)
        result = self._native.lib.overdrive_update(
            self._handle,
            table.encode("utf-8"),
            id.encode("utf-8"),
            json_str.encode("utf-8"),
        )
        if result == -1:
            _check_error(self._native)
        return result == 1

    def delete(self, table: str, id: str) -> bool:
        """Delete a document by _id."""
        self._ensure_open()
        result = self._native.lib.overdrive_delete(
            self._handle,
            table.encode("utf-8"),
            id.encode("utf-8"),
        )
        if result == -1:
            _check_error(self._native)
        return result == 1

    def count(self, table: str) -> int:
        """Count documents in a table."""
        self._ensure_open()
        result = self._native.lib.overdrive_count(self._handle, table.encode("utf-8"))
        if result == -1:
            _check_error(self._native)
        return max(0, result)

    # ── Query ───────────────────────────────────

    def query(self, sql: str) -> List[Dict[str, Any]]:
        """Execute SQL query. Returns list of result rows."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_query(self._handle, sql.encode("utf-8"))
        if not ptr:
            _check_error(self._native)
            return []
        result = _read_and_free(self._native, ptr)
        if not result:
            return []
        parsed = json.loads(result)
        return parsed.get("rows", [])

    def query_full(self, sql: str) -> Dict[str, Any]:
        """Execute SQL query. Returns full result with metadata."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_query(self._handle, sql.encode("utf-8"))
        if not ptr:
            _check_error(self._native)
            return {"rows": [], "columns": [], "rows_affected": 0}
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else {}

    def search(self, table: str, text: str) -> List[Dict[str, Any]]:
        """Full-text search."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_search(
            self._handle,
            table.encode("utf-8"),
            text.encode("utf-8"),
        )
        if not ptr:
            return []
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else []

    # ── Internal ────────────────────────────────

    def _ensure_open(self):
        if not self._handle:
            raise OverDriveError("Database is closed")

    # ── MVCC Transactions (Phase 5) ────────────

    READ_UNCOMMITTED = 0
    READ_COMMITTED = 1
    REPEATABLE_READ = 2
    SERIALIZABLE = 3

    def begin_transaction(self, isolation: int = 1) -> int:
        """Begin a new MVCC transaction. Returns transaction ID."""
        self._ensure_open()
        txn_id = self._native.lib.overdrive_begin_transaction(self._handle, isolation)
        if txn_id == 0:
            _check_error(self._native)
            raise OverDriveError("Failed to begin transaction")
        return txn_id

    def commit_transaction(self, txn_id: int):
        """Commit a transaction."""
        self._ensure_open()
        result = self._native.lib.overdrive_commit_transaction(self._handle, txn_id)
        if result != 0:
            _check_error(self._native)

    def abort_transaction(self, txn_id: int):
        """Abort (rollback) a transaction."""
        self._ensure_open()
        result = self._native.lib.overdrive_abort_transaction(self._handle, txn_id)
        if result != 0:
            _check_error(self._native)

    class _TransactionCtx:
        """Context manager for MVCC transactions."""
        def __init__(self, db, isolation):
            self.db = db
            self.isolation = isolation
            self.txn_id = None

        def __enter__(self):
            self.txn_id = self.db.begin_transaction(self.isolation)
            return self.txn_id

        def __exit__(self, exc_type, exc_val, exc_tb):
            if exc_type is not None:
                self.db.abort_transaction(self.txn_id)
            else:
                self.db.commit_transaction(self.txn_id)
            return False

    def transaction(self, isolation: int = 1):
        """
        Context manager for MVCC transactions.

        Usage:
            with db.transaction() as txn_id:
                db.insert("users", {"name": "Alice"})
                # auto-commits on success, auto-aborts on exception
        """
        return self._TransactionCtx(self, isolation)

    # ── Integrity Verification (Phase 5) ───────

    def verify_integrity(self) -> Dict[str, Any]:
        """Verify database integrity. Returns a report dict."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_verify_integrity(self._handle)
        if not ptr:
            _check_error(self._native)
            return {"valid": False, "issues": ["verification failed"]}
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else {}

    # ── Extended Stats (Phase 5) ───────────────

    def stats(self) -> Dict[str, Any]:
        """Get extended database statistics including MVCC info."""
        self._ensure_open()
        ptr = self._native.lib.overdrive_stats(self._handle)
        if not ptr:
            _check_error(self._native)
            return {}
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else {}
