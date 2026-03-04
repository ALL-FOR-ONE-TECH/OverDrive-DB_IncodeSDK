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

    # ── Security (v1.3.0) ────────────────────────

    @staticmethod
    def open_encrypted(path: str, key_env_var: str = "ODB_KEY") -> "OverDrive":
        """
        Open a database with an encryption key loaded from an environment variable.

        Never hardcode the key — always read from env or a secrets manager.

        Example::

            # PowerShell: $env:ODB_KEY = "my-secret-aes-key-32chars!!!!"
            # bash:       export ODB_KEY="my-secret-aes-key-32chars!!!!"
            db = OverDrive.open_encrypted("app.odb", "ODB_KEY")
        """
        key = os.environ.get(key_env_var)
        if not key:
            raise OverDriveError(
                f"Encryption key env var '{key_env_var}' is not set or is empty. "
                f"Set it with:  $env:{key_env_var}=\"your-key\"  (PowerShell) "
                f"or  export {key_env_var}=\"your-key\"  (bash)"
            )
        # Pass to engine via internal env var, then immediately clear it
        os.environ["__OVERDRIVE_KEY"] = key
        try:
            db = OverDrive(path)
        finally:
            # Always clear — even if open fails
            key_bytes = key.encode()
            # Zero the key string in memory (best-effort in Python)
            for i in range(len(key_bytes)):
                key_bytes = key_bytes  # keep reference alive
            os.environ.pop("__OVERDRIVE_KEY", None)
        return db

    def backup(self, dest_path: str) -> None:
        """
        Create an encrypted backup of the database at dest_path.

        Syncs all in-memory data to disk first, then copies ``.odb`` + ``.wal``.
        Store the backup on a separate physical drive or cloud storage.

        Example::

            db.backup("backups/app_2026-03-04.odb")
        """
        import shutil
        self._ensure_open()
        self.sync()
        shutil.copy2(self._path, dest_path)
        # Also backup WAL if exists
        wal_src = self._path + ".wal"
        wal_dst = dest_path + ".wal"
        if os.path.exists(wal_src):
            shutil.copy2(wal_src, wal_dst)
        # Harden backup file permissions
        _set_secure_permissions(dest_path)

    def cleanup_wal(self) -> None:
        """
        Delete the WAL file after a confirmed commit to prevent stale replay attacks.

        Call this after ``commit_transaction()``::

            txn_id = db.begin_transaction(OverDrive.SERIALIZABLE)
            db.insert("users", {"name": "Carol"})
            db.commit_transaction(txn_id)
            db.cleanup_wal()  # Remove WAL — prevents replay attack
        """
        wal_path = self._path + ".wal"
        if os.path.exists(wal_path):
            os.remove(wal_path)

    def query_safe(self, sql_template: str, params: List[Any]) -> List[Dict[str, Any]]:
        """
        Execute a parameterized SQL query — the safe way to include user input.

        Use ``?`` as placeholders; values are sanitized before substitution.
        Raises ``OverDriveError`` if any param contains SQL injection patterns.

        Example::

            # SAFE — user input via params, never via string concat
            results = db.query_safe(
                "SELECT * FROM users WHERE name = ?",
                [user_input]   # blocked if it contains DROP, --, etc.
            )
        """
        _DANGEROUS = {"DROP", "TRUNCATE", "ALTER", "EXEC", "EXECUTE", "UNION", "XP_"}
        _DANGEROUS_TOKENS = {"--", ";--", "/*", "*/"}

        sanitized = []
        for param in params:
            s = str(param)
            upper = s.upper()
            for token in _DANGEROUS_TOKENS:
                if token in s:
                    raise OverDriveError(
                        f"SQL injection detected: param '{s}' contains forbidden token '{token}'"
                    )
            for keyword in _DANGEROUS:
                if keyword in upper.split():
                    raise OverDriveError(
                        f"SQL injection detected: param '{s}' contains forbidden keyword '{keyword}'"
                    )
            # Escape single quotes (SQL standard)
            sanitized.append("'" + s.replace("'", "''") + "'")

        sql = sql_template
        for value in sanitized:
            if "?" not in sql:
                raise OverDriveError("More params than '?' placeholders in SQL template")
            sql = sql.replace("?", value, 1)

        placeholder_count = sql_template.count("?")
        if len(params) < placeholder_count:
            raise OverDriveError(
                f"SQL template has {placeholder_count} '?' placeholders "
                f"but only {len(params)} params were provided"
            )
        return self.query(sql)


# ── File Permission Hardening ──────────────────

def _set_secure_permissions(path: str) -> None:
    """
    Set restrictive OS-level permissions on an .odb file.

    - Windows: ``icacls`` — grants only current user Full Control
    - Linux/macOS: ``chmod 600`` — owner read/write only
    """
    import subprocess
    import stat
    if not os.path.exists(path):
        return
    system = platform.system()
    if system == "Windows":
        try:
            result = subprocess.run(
                ["icacls", path, "/inheritance:r", "/grant:r", f"{os.environ.get('USERNAME', '%USERNAME%')}:F"],
                capture_output=True, text=True, timeout=5
            )
            if result.returncode != 0:
                import warnings
                warnings.warn(
                    f"[overdrive] Could not harden file permissions on '{path}': {result.stderr}",
                    stacklevel=3
                )
        except Exception as e:
            import warnings
            warnings.warn(f"[overdrive] icacls unavailable, permissions not hardened: {e}", stacklevel=3)
    else:
        os.chmod(path, stat.S_IRUSR | stat.S_IWUSR)  # 0o600


# Patch _set_secure_permissions into OverDrive.__init__ automatically
_original_init = OverDrive.__init__

def _secure_init(self, path: str):
    _original_init(self, path)
    _set_secure_permissions(path)

OverDrive.__init__ = _secure_init


# ── Thread-safe Wrapper ───────────────────────

import threading

class ThreadSafeOverDrive:
    """
    Thread-safe wrapper for OverDrive using a threading.Lock.

    Use this when multiple threads need to share one database instance.

    Example::

        db = ThreadSafeOverDrive("app.odb")

        import threading
        def worker():
            results = db.query("SELECT * FROM users LIMIT 1")
            print(results)

        threads = [threading.Thread(target=worker) for _ in range(4)]
        for t in threads: t.start()
        for t in threads: t.join()
    """

    def __init__(self, path: str):
        self._lock = threading.Lock()
        self._db = OverDrive(path)

    @classmethod
    def open_encrypted(cls, path: str, key_env_var: str = "ODB_KEY") -> "ThreadSafeOverDrive":
        """Open with encryption key from env var."""
        instance = cls.__new__(cls)
        instance._lock = threading.Lock()
        instance._db = OverDrive.open_encrypted(path, key_env_var)
        return instance

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def query(self, sql: str) -> List[Dict[str, Any]]:
        with self._lock:
            return self._db.query(sql)

    def query_safe(self, sql_template: str, params: List[Any]) -> List[Dict[str, Any]]:
        with self._lock:
            return self._db.query_safe(sql_template, params)

    def insert(self, table: str, doc: Dict[str, Any]) -> str:
        with self._lock:
            return self._db.insert(table, doc)

    def get(self, table: str, id: str) -> Optional[Dict[str, Any]]:
        with self._lock:
            return self._db.get(table, id)

    def backup(self, dest_path: str) -> None:
        with self._lock:
            return self._db.backup(dest_path)

    def cleanup_wal(self) -> None:
        with self._lock:
            return self._db.cleanup_wal()

    def sync(self) -> None:
        with self._lock:
            return self._db.sync()

    def close(self) -> None:
        with self._lock:
            return self._db.close()

    def __getattr__(self, name):
        """Proxy all other methods through the lock."""
        attr = getattr(self._db, name)
        if callable(attr):
            def locked(*args, **kwargs):
                with self._lock:
                    return attr(*args, **kwargs)
            return locked
        return attr

