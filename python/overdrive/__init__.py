"""
OverDrive InCode SDK — Python Wrapper
Embeddable document database. Like SQLite for JSON.

Usage:
    from overdrive import OverDrive

    # Simplified API (v1.4+)
    db = OverDrive.open("myapp.odb")
    db.insert("users", {"name": "Alice", "age": 30})  # table auto-created
    results = db.query("SELECT * FROM users WHERE age > 25")
    db.close()

    # Password-protected database
    db = OverDrive.open("secure.odb", password="my-secret-pass")

    # RAM engine for high-performance caching
    db = OverDrive.open("cache.odb", engine="RAM")

    # Legacy API (still fully supported)
    db = OverDrive("myapp.odb")
    db.create_table("users")
    db.insert("users", {"name": "Alice", "age": 30})
    db.close()
"""

import ctypes
import json
import os
import platform
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, TypeVar, Union

_T = TypeVar("_T")


def _find_library() -> str:
    """Find the overdrive shared library, downloading if necessary."""
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

    # Not found locally — try to download from GitHub Releases
    try:
        from overdrive.download import ensure_binary
        downloaded = ensure_binary(target_dir=str(Path(__file__).parent))
        if downloaded and Path(downloaded).exists():
            return downloaded
    except Exception:
        pass

    # Try system path as last resort
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

        # Phase 5: Integrity & Stats — bound lazily to avoid crash if symbol missing
        # These are bound on first use in _bind_optional_symbols()
        self._optional_bound = False

    def _bind_optional_symbols(self):
        """Lazily bind symbols that may not exist in older native library versions."""
        if self._optional_bound:
            return
        self._optional_bound = True
        try:
            self.lib.overdrive_verify_integrity.argtypes = [ctypes.c_void_p]
            self.lib.overdrive_verify_integrity.restype = ctypes.c_void_p
        except AttributeError:
            self.lib.overdrive_verify_integrity = None
        try:
            self.lib.overdrive_stats.argtypes = [ctypes.c_void_p]
            self.lib.overdrive_stats.restype = ctypes.c_void_p
        except AttributeError:
            self.lib.overdrive_stats = None

        # v1.4: Simplified open with engine + options
        self.lib.overdrive_open_with_engine.argtypes = [
            ctypes.c_char_p,  # path
            ctypes.c_char_p,  # engine
            ctypes.c_char_p,  # options_json
        ]
        self.lib.overdrive_open_with_engine.restype = ctypes.c_void_p

        # v1.4: Password-based open (Argon2id key derivation in native lib)
        self.lib.overdrive_open_with_password.argtypes = [
            ctypes.c_char_p,  # path
            ctypes.c_char_p,  # password (nullable)
        ]
        self.lib.overdrive_open_with_password.restype = ctypes.c_void_p

        # v1.4: Toggle auto-table creation on an open handle
        self.lib.overdrive_set_auto_create_tables.argtypes = [
            ctypes.c_void_p,  # handle
            ctypes.c_int,     # enabled (non-zero = True)
        ]
        self.lib.overdrive_set_auto_create_tables.restype = ctypes.c_int

        # v1.4: Structured error details (JSON)
        self.lib.overdrive_get_error_details.argtypes = []
        self.lib.overdrive_get_error_details.restype = ctypes.c_char_p

        # v1.4: RAM engine — per-table engine selection
        self.lib.overdrive_create_table_with_engine.argtypes = [
            ctypes.c_void_p,  # handle
            ctypes.c_char_p,  # table_name
            ctypes.c_char_p,  # engine
        ]
        self.lib.overdrive_create_table_with_engine.restype = ctypes.c_int

        # v1.4: RAM engine — snapshot / restore
        self.lib.overdrive_snapshot.argtypes = [
            ctypes.c_void_p,  # handle
            ctypes.c_char_p,  # snapshot_path
        ]
        self.lib.overdrive_snapshot.restype = ctypes.c_int

        self.lib.overdrive_restore.argtypes = [
            ctypes.c_void_p,  # handle
            ctypes.c_char_p,  # snapshot_path
        ]
        self.lib.overdrive_restore.restype = ctypes.c_int

        # v1.4: RAM engine — memory usage (returns JSON string)
        self.lib.overdrive_memory_usage.argtypes = [ctypes.c_void_p]
        self.lib.overdrive_memory_usage.restype = ctypes.c_void_p

        # v1.4: Watchdog — file integrity monitoring (no open handle required)
        self.lib.overdrive_watchdog.argtypes = [ctypes.c_char_p]
        self.lib.overdrive_watchdog.restype = ctypes.c_void_p


_native = None


def _get_native() -> _Native:
    global _native
    if _native is None:
        _native = _Native()
    return _native


class OverDriveError(Exception):
    """
    Base exception for all OverDrive SDK errors.

    Attributes
    ----------
    code:
        Machine-readable error code (e.g. ``"ODB-AUTH-001"``), or ``""``
        when not available.
    message:
        Human-readable description of the error.
    context:
        Additional context string from the native library, or ``""`` when
        not available.
    suggestions:
        List of actionable suggestions to resolve the error.
    doc_link:
        URL to the relevant documentation page, or ``""`` when not
        available.
    """

    def __init__(
        self,
        message: str = "",
        *,
        code: str = "",
        context: str = "",
        suggestions: Optional[List[str]] = None,
        doc_link: str = "",
    ):
        self.code = code
        self.message = message
        self.context = context
        self.suggestions = suggestions or []
        self.doc_link = doc_link
        # Build a rich string representation
        parts = [f"Error {code}: {message}" if code else message]
        if context:
            parts.append(f"Context: {context}")
        if self.suggestions:
            parts.append("Suggestions:\n" + "\n".join(f"  • {s}" for s in self.suggestions))
        if doc_link:
            parts.append(f"For more help: {doc_link}")
        super().__init__("\n".join(parts))


class AuthenticationError(OverDriveError):
    """Raised for authentication / encryption errors (ODB-AUTH-*)."""


class TableError(OverDriveError):
    """Raised for table operation errors (ODB-TABLE-*)."""


class QueryError(OverDriveError):
    """Raised for query execution errors (ODB-QUERY-*)."""


class TransactionError(OverDriveError):
    """Raised for transaction / MVCC errors (ODB-TXN-*)."""


class OverDriveIOError(OverDriveError):
    """Raised for file I/O errors (ODB-IO-*).

    Named ``OverDriveIOError`` to avoid shadowing the Python builtin
    ``IOError``.
    """


class FFIError(OverDriveError):
    """Raised for native-library / FFI errors (ODB-FFI-*)."""


# Mapping from error-code prefix → exception class
_ERROR_CLASS_MAP = {
    "ODB-AUTH": AuthenticationError,
    "ODB-TABLE": TableError,
    "ODB-QUERY": QueryError,
    "ODB-TXN": TransactionError,
    "ODB-IO": OverDriveIOError,
    "ODB-FFI": FFIError,
}


def _make_error(
    message: str,
    code: str = "",
    context: str = "",
    suggestions: Optional[List[str]] = None,
    doc_link: str = "",
) -> OverDriveError:
    """Instantiate the most specific OverDriveError subclass for *code*."""
    cls = OverDriveError
    if code:
        # Match on the category prefix (e.g. "ODB-AUTH" from "ODB-AUTH-001")
        # Code format: ODB-{CATEGORY}-{number}, so split and take first 2 segments
        parts = code.split("-")
        if len(parts) >= 3:
            prefix = f"{parts[0]}-{parts[1]}"  # e.g. "ODB-AUTH"
        else:
            prefix = code
        cls = _ERROR_CLASS_MAP.get(prefix, OverDriveError)
    return cls(
        message,
        code=code,
        context=context,
        suggestions=suggestions,
        doc_link=doc_link,
    )


def _check_error(native: "_Native"):
    """
    Check for and raise the last error from the native library.

    Tries ``overdrive_get_error_details()`` first for structured JSON;
    falls back to ``overdrive_last_error()`` for a plain string.
    """
    # 1. Try structured error details (v1.4+)
    try:
        details_ptr = native.lib.overdrive_get_error_details()
        if details_ptr:
            raw = details_ptr.decode("utf-8") if isinstance(details_ptr, bytes) else details_ptr
            if raw:
                try:
                    data = json.loads(raw)
                    code = data.get("code", "")
                    msg = data.get("message", "")
                    context = data.get("context", "")
                    suggestions = data.get("suggestions", [])
                    doc_link = data.get("doc_link", "")
                    if msg or code:
                        raise _make_error(msg, code=code, context=context,
                                          suggestions=suggestions, doc_link=doc_link)
                except (json.JSONDecodeError, UnicodeDecodeError):
                    # Malformed JSON — fall through to plain-string path below
                    pass
    except OverDriveError:
        raise
    except Exception:
        pass  # overdrive_get_error_details not available — fall through

    # 2. Fall back to plain string error
    err = native.lib.overdrive_last_error()
    if err:
        raise OverDriveError(err.decode("utf-8") if isinstance(err, bytes) else err)


def _read_and_free(native: "_Native", ptr) -> Optional[str]:
    """Read a C string and free it."""
    if not ptr:
        return None
    try:
        s = ctypes.cast(ptr, ctypes.c_char_p).value
        return s.decode("utf-8") if s else None
    finally:
        native.lib.overdrive_free_string(ptr)


@dataclass
class WatchdogReport:
    """
    Result of a :meth:`OverDrive.watchdog` call.

    Fields
    ------
    file_path:
        Absolute or relative path that was inspected.
    file_size_bytes:
        Size of the ``.odb`` file in bytes (0 if missing).
    last_modified:
        Unix timestamp of the last modification (0 if missing).
    integrity_status:
        One of ``"valid"``, ``"corrupted"``, or ``"missing"``.
    corruption_details:
        Human-readable description of the corruption, or ``None`` when
        the file is healthy.
    page_count:
        Number of pages found in the file (0 if missing or unreadable).
    magic_valid:
        ``True`` when the file's magic number matches the expected value.
    """

    file_path: str
    file_size_bytes: int
    last_modified: int
    integrity_status: str
    corruption_details: Optional[str]
    page_count: int
    magic_valid: bool


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

    def create_table(self, name: str, engine: str = "Disk"):
        """
        Create a new table.

        Parameters
        ----------
        name:
            Table name.
        engine:
            Storage engine for this table.  One of ``"Disk"`` (default),
            ``"RAM"``, ``"Vector"``, ``"Time-Series"``, ``"Graph"``,
            ``"Streaming"``.  When ``engine="RAM"`` the table is stored
            entirely in memory with O(1) HashMap operations.

        Examples
        --------
        Disk table (default)::

            db.create_table("users")

        RAM table for hot-cache data::

            db.create_table("sessions", engine="RAM")
        """
        self._ensure_open()
        if engine == "Disk":
            result = self._native.lib.overdrive_create_table(
                self._handle, name.encode("utf-8")
            )
        else:
            result = self._native.lib.overdrive_create_table_with_engine(
                self._handle,
                name.encode("utf-8"),
                engine.encode("utf-8"),
            )
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

    def transaction(self, callback_or_isolation=None, isolation: int = READ_COMMITTED):
        """
        Dual-mode transaction helper.

        Callback pattern (Task 10):
            result = db.transaction(lambda txn: db.insert("users", {"name": "Alice"}))
            # auto-commits on success, auto-aborts on exception, returns callback value

        Context manager pattern (backward compat):
            with db.transaction() as txn_id:
                db.insert("users", {"name": "Alice"})
                # auto-commits on success, auto-aborts on exception
        """
        if callable(callback_or_isolation):
            # Callback pattern
            callback = callback_or_isolation
            txn_id = self.begin_transaction(isolation)
            try:
                result = callback(txn_id)
                self.commit_transaction(txn_id)
                return result
            except Exception:
                self.abort_transaction(txn_id)
                raise
        else:
            # Context manager pattern
            iso = callback_or_isolation if callback_or_isolation is not None else isolation
            return self._TransactionCtx(self, iso)

    # camelCase aliases for cross-SDK API consistency (Task 10.5)
    beginTransaction = begin_transaction
    commitTransaction = commit_transaction
    abortTransaction = abort_transaction

    def transaction_with_retry(
        self,
        callback: Callable[[int], _T],
        isolation: int = READ_COMMITTED,
        max_retries: int = 3,
    ) -> _T:
        """
        Execute a transaction with automatic exponential-backoff retry on
        :class:`TransactionError`.

        Parameters
        ----------
        callback:
            A callable that receives the transaction ID and performs
            database operations.  It is called inside a transaction that
            is automatically committed on success or aborted on exception.
        isolation:
            MVCC isolation level.  Defaults to :attr:`READ_COMMITTED`.
        max_retries:
            Maximum number of attempts (including the first).  Defaults
            to ``3``.  After all attempts are exhausted the last
            :class:`TransactionError` is re-raised.

        Returns
        -------
        Any
            The return value of *callback*.

        Raises
        ------
        TransactionError
            If all retry attempts fail due to transaction conflicts.
        Exception
            Any non-:class:`TransactionError` exception raised by
            *callback* is propagated immediately without retrying.

        Example
        -------
        ::

            result = db.transaction_with_retry(
                lambda txn: db.insert("orders", {"item": "widget"}),
                isolation=OverDrive.SERIALIZABLE,
                max_retries=5,
            )
        """
        import time

        last_exc: Optional[TransactionError] = None
        for attempt in range(max_retries):
            try:
                return self.transaction(callback, isolation)
            except TransactionError as exc:
                last_exc = exc
                if attempt < max_retries - 1:
                    delay = min(0.1 * (2 ** attempt), 2.0)
                    time.sleep(delay)
        # All attempts exhausted — re-raise the last conflict error
        raise last_exc  # type: ignore[misc]

    # ── Helper Methods (Task 11) ────────────────

    def findOne(self, table: str, where: str = "") -> Optional[Dict[str, Any]]:
        """
        Return the first document matching *where*, or ``None`` if no match.

        Parameters
        ----------
        table:
            Table to query.
        where:
            Optional SQL WHERE clause (e.g. ``"age > 25"``).  When empty,
            returns the first document in the table.

        Example
        -------
        ::

            user = db.findOne("users", "name = 'Alice'")
            first = db.findOne("users")  # first doc in table
        """
        if where:
            sql = f"SELECT * FROM {table} WHERE {where} LIMIT 1"
        else:
            sql = f"SELECT * FROM {table} LIMIT 1"
        rows = self.query(sql)
        return rows[0] if rows else None

    def findAll(
        self,
        table: str,
        where: str = "",
        order_by: str = "",
        limit: int = 0,
    ) -> List[Dict[str, Any]]:
        """
        Return all documents matching *where*.

        Parameters
        ----------
        table:
            Table to query.
        where:
            Optional SQL WHERE clause (e.g. ``"age > 25"``).
        order_by:
            Optional ORDER BY expression (e.g. ``"age DESC"``).
        limit:
            Maximum number of rows to return.  ``0`` means no limit.

        Example
        -------
        ::

            adults = db.findAll("users", where="age >= 18", order_by="name", limit=100)
        """
        sql = f"SELECT * FROM {table}"
        if where:
            sql += f" WHERE {where}"
        if order_by:
            sql += f" ORDER BY {order_by}"
        if limit > 0:
            sql += f" LIMIT {limit}"
        return self.query(sql)

    def updateMany(self, table: str, where: str, updates: Dict[str, Any]) -> int:
        """
        Update all documents matching *where* with the given *updates*.

        Parameters
        ----------
        table:
            Table to update.
        where:
            SQL WHERE clause that selects the rows to update (required).
        updates:
            Dictionary of field → new value pairs.

        Returns
        -------
        int
            Number of documents updated.

        Example
        -------
        ::

            count = db.updateMany("users", "active = 0", {"status": "inactive"})
        """
        set_clauses = ", ".join(
            f"{k} = {json.dumps(v)}" for k, v in updates.items()
        )
        sql = f"UPDATE {table} SET {set_clauses} WHERE {where}"
        result = self.query_full(sql)
        return int(result.get("rows_affected", 0))

    def deleteMany(self, table: str, where: str) -> int:
        """
        Delete all documents matching *where*.

        Parameters
        ----------
        table:
            Table to delete from.
        where:
            SQL WHERE clause that selects the rows to delete (required).

        Returns
        -------
        int
            Number of documents deleted.

        Example
        -------
        ::

            count = db.deleteMany("logs", "created_at < 1700000000")
        """
        sql = f"DELETE FROM {table} WHERE {where}"
        result = self.query_full(sql)
        return int(result.get("rows_affected", 0))

    def countWhere(self, table: str, where: str = "") -> int:
        """
        Count documents matching *where*.

        Parameters
        ----------
        table:
            Table to count in.
        where:
            Optional SQL WHERE clause.  When empty, counts all documents.

        Returns
        -------
        int
            Number of matching documents.

        Example
        -------
        ::

            n = db.countWhere("users", "age > 25")
            total = db.countWhere("users")
        """
        if where:
            sql = f"SELECT COUNT(*) FROM {table} WHERE {where}"
        else:
            sql = f"SELECT COUNT(*) FROM {table}"
        rows = self.query(sql)
        if not rows:
            return 0
        row = rows[0]
        # The COUNT(*) column may be named differently depending on the engine
        for key in ("COUNT(*)", "count(*)", "count", "COUNT"):
            if key in row:
                return int(row[key])
        # Fallback: take the first value in the row
        return int(next(iter(row.values()), 0))

    def exists(self, table: str, id: str) -> bool:
        """
        Check whether a document with the given *id* exists.

        Parameters
        ----------
        table:
            Table to look in.
        id:
            The ``_id`` value of the document.

        Returns
        -------
        bool
            ``True`` if the document exists, ``False`` otherwise.

        Example
        -------
        ::

            if db.exists("users", "users_1"):
                print("found")
        """
        return self.get(table, id) is not None

    # ── Integrity Verification (Phase 5) ───────

    def verify_integrity(self) -> Dict[str, Any]:
        """Verify database integrity. Returns a report dict."""
        self._ensure_open()
        self._native._bind_optional_symbols()
        if not self._native.lib.overdrive_verify_integrity:
            return {"valid": True, "issues": [], "pages_checked": 0, "tables_verified": 0}
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
        self._native._bind_optional_symbols()
        if not self._native.lib.overdrive_stats:
            # Fallback: compute basic stats without the native symbol
            tables = self.list_tables()
            total = sum(self.count(t) for t in tables)
            import os
            size = os.path.getsize(self._path) if os.path.exists(self._path) else 0
            return {
                "tables": len(tables),
                "total_records": total,
                "file_size_bytes": size,
                "path": self._path,
                "sdk_version": OverDrive.version(),
            }
        ptr = self._native.lib.overdrive_stats(self._handle)
        if not ptr:
            _check_error(self._native)
            return {}
        result = _read_and_free(self._native, ptr)
        return json.loads(result) if result else {}

    # ── RAM Engine Methods (v1.4.0) ────────────

    def snapshot(self, path: str) -> None:
        """
        Persist the current RAM database (or RAM tables) to a snapshot file.

        The snapshot can later be loaded with :meth:`restore`.  This is a
        no-op for pure Disk databases but is safe to call regardless of
        engine type.

        Parameters
        ----------
        path:
            Destination file path for the snapshot (e.g. ``"./backup.odb"``).

        Raises
        ------
        OverDriveError
            If the snapshot cannot be written (permission error, disk full,
            etc.).

        Example
        -------
        ::

            db = OverDrive.open("cache.odb", engine="RAM")
            db.insert("sessions", {"user": "alice"})
            db.snapshot("./backups/cache_snapshot.odb")
        """
        self._ensure_open()
        result = self._native.lib.overdrive_snapshot(
            self._handle, path.encode("utf-8")
        )
        if result != 0:
            _check_error(self._native)

    def restore(self, path: str) -> None:
        """
        Load a previously saved snapshot into the current database handle.

        All existing in-memory data is replaced by the snapshot contents.

        Parameters
        ----------
        path:
            Path to the snapshot file created by :meth:`snapshot`.

        Raises
        ------
        OverDriveError
            If the snapshot file is missing, corrupted, or cannot be read.

        Example
        -------
        ::

            db = OverDrive.open("cache.odb", engine="RAM")
            db.restore("./backups/cache_snapshot.odb")
            sessions = db.query("SELECT * FROM sessions")
        """
        self._ensure_open()
        result = self._native.lib.overdrive_restore(
            self._handle, path.encode("utf-8")
        )
        if result != 0:
            _check_error(self._native)

    def memory_usage(self) -> Dict[str, Any]:
        """
        Return current RAM consumption statistics for this database.

        Returns
        -------
        dict
            A dictionary with the following keys:

            * ``bytes`` (int) — bytes currently used by the RAM engine.
            * ``mb`` (float) — same value expressed in megabytes.
            * ``limit_bytes`` (int) — configured memory limit in bytes.
            * ``percent`` (float) — utilisation as a percentage of the limit.

        Raises
        ------
        OverDriveError
            If the native library cannot retrieve memory statistics.

        Example
        -------
        ::

            usage = db.memory_usage()
            print(f"RAM usage: {usage['mb']:.1f} MB / "
                  f"{usage['limit_bytes'] // 1024 // 1024} MB "
                  f"({usage['percent']:.1f}%)")
        """
        self._ensure_open()
        ptr = self._native.lib.overdrive_memory_usage(self._handle)
        if not ptr:
            _check_error(self._native)
            return {"bytes": 0, "mb": 0.0, "limit_bytes": 0, "percent": 0.0}
        result = _read_and_free(self._native, ptr)
        if not result:
            return {"bytes": 0, "mb": 0.0, "limit_bytes": 0, "percent": 0.0}
        data = json.loads(result)
        return {
            "bytes": int(data.get("bytes", 0)),
            "mb": float(data.get("mb", 0.0)),
            "limit_bytes": int(data.get("limit_bytes", 0)),
            "percent": float(data.get("percent", 0.0)),
        }

    # camelCase alias for API consistency across SDKs
    memoryUsage = memory_usage

    # ── Simplified API (v1.4.0) ──────────────────

    #: Valid storage engine names.
    ENGINES = frozenset({"Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming"})

    @staticmethod
    def open(
        path: str = "./app.odb",
        password: Optional[str] = None,
        engine: str = "Disk",
        auto_create_tables: bool = True,
    ) -> "OverDrive":
        """
        Open or create a database — the simplified v1.4 entry point.

        Parameters
        ----------
        path:
            File path for the database (created if it doesn't exist).
            Defaults to ``./app.odb``.
        password:
            Optional password for AES-256-GCM encryption.  The native library
            derives the key with Argon2id (64 MB memory, 3 iterations,
            parallelism 4).  Must be at least 8 characters.
        engine:
            Storage engine to use.  One of ``"Disk"`` (default), ``"RAM"``,
            ``"Vector"``, ``"Time-Series"``, ``"Graph"``, ``"Streaming"``.
        auto_create_tables:
            When ``True`` (default), tables are created automatically on the
            first ``insert()`` call so you don't need explicit
            ``create_table()`` calls.

        Returns
        -------
        OverDrive
            An open database instance.

        Examples
        --------
        Basic usage (3 lines)::

            db = OverDrive.open("myapp.odb")
            db.insert("users", {"name": "Alice"})   # table auto-created
            db.close()

        Password-protected::

            db = OverDrive.open("secure.odb", password="my-secret-pass")

        RAM engine for caching::

            db = OverDrive.open("cache.odb", engine="RAM")

        Raises
        ------
        OverDriveError
            If the engine name is invalid, the password is too short, or the
            native library returns an error.
        """
        if engine not in OverDrive.ENGINES:
            raise OverDriveError(
                f"Invalid engine '{engine}'. "
                f"Valid options: {', '.join(sorted(OverDrive.ENGINES))}"
            )
        if password is not None and len(password) < 8:
            raise OverDriveError(
                "Password must be at least 8 characters long"
            )

        native = _get_native()

        # Build options JSON for the native call
        options: Dict[str, Any] = {"auto_create_tables": auto_create_tables}
        if password is not None:
            options["password"] = password

        options_json = json.dumps(options).encode("utf-8")

        handle = native.lib.overdrive_open_with_engine(
            path.encode("utf-8"),
            engine.encode("utf-8"),
            options_json,
        )
        if not handle:
            # Try to get structured error details first
            details_ptr = native.lib.overdrive_get_error_details()
            if details_ptr:
                try:
                    details = json.loads(details_ptr.decode("utf-8"))
                    msg = details.get("message", "")
                    suggestions = details.get("suggestions", [])
                    code = details.get("code", "")
                    parts = [f"Error {code}: {msg}"] if code else [msg]
                    if suggestions:
                        parts.append("Suggestions: " + "; ".join(suggestions))
                    raise OverDriveError("\n".join(parts))
                except (json.JSONDecodeError, UnicodeDecodeError):
                    pass
            _check_error(native)
            raise OverDriveError(f"Failed to open database: {path}")

        # Bypass __init__ — we already have a handle from the native call
        instance = object.__new__(OverDrive)
        instance._native = native
        instance._handle = handle
        instance._path = path
        _set_secure_permissions(path)
        return instance

    # ── File Watchdog (v1.4.0) ──────────────────

    @staticmethod
    def watchdog(file_path: str) -> "WatchdogReport":
        """
        Inspect a ``.odb`` file for integrity, size, and modification status.

        This is a **static method** — it does not require an open database
        handle and can be called on any path at any time.

        Parameters
        ----------
        file_path:
            Path to the ``.odb`` file to inspect.

        Returns
        -------
        WatchdogReport
            A dataclass with file size, last-modified timestamp, integrity
            status (``"valid"``, ``"corrupted"``, or ``"missing"``), optional
            corruption details, page count, and magic-number validity.

        Raises
        ------
        OverDriveError
            If the native library returns a NULL pointer or an unparseable
            response.

        Example
        -------
        ::

            report = OverDrive.watchdog("./app.odb")
            if report.integrity_status == "corrupted":
                print(f"Corruption detected: {report.corruption_details}")
            else:
                print(f"Healthy — {report.file_size_bytes} bytes, "
                      f"{report.page_count} pages")
        """
        native = _get_native()
        ptr = native.lib.overdrive_watchdog(file_path.encode("utf-8"))
        if not ptr:
            _check_error(native)
            raise OverDriveError(f"watchdog() returned NULL for path: {file_path}")
        raw = _read_and_free(native, ptr)
        if not raw:
            raise OverDriveError(f"watchdog() returned empty response for path: {file_path}")
        data = json.loads(raw)
        return WatchdogReport(
            file_path=data.get("file_path", file_path),
            file_size_bytes=int(data.get("file_size_bytes", 0)),
            last_modified=int(data.get("last_modified", 0)),
            integrity_status=data.get("integrity_status", "missing"),
            corruption_details=data.get("corruption_details"),
            page_count=int(data.get("page_count", 0)),
            magic_valid=bool(data.get("magic_valid", False)),
        )

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

