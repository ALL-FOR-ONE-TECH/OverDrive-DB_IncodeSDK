"""
OverDrive-DB Python SDK v1.0.0

Usage:
    from overdrive import OverdriveDb

    odb = OverdriveDb.open("app.odb")
    odb.create_table("users")
    id = odb.insert("users", {"name": "Alice", "age": 30})
    doc = odb.get("users", id)
    odb.close()
"""

import json
import ctypes
from enum import IntEnum
from typing import Any, Dict, List, Optional
from . import _native


class IsolationLevel(IntEnum):
    READ_UNCOMMITTED = 0
    READ_COMMITTED   = 1
    REPEATABLE_READ  = 2
    SERIALIZABLE     = 3


class Transaction:
    def __init__(self, txn_id: int):
        self._id = txn_id

    @property
    def id(self) -> int:
        return self._id


class OverdriveDb:
    """
    OverDrive-DB embedded database handle.

    Idiomatic usage::

        odb = OverdriveDb.open("app.odb")
        id  = odb.insert("users", {"name": "Alice"})
        doc = odb.get("users", id)
        odb.close()

    Or as a context manager::

        with OverdriveDb.open("app.odb") as odb:
            odb.insert("users", {"name": "Bob"})
    """

    def __init__(self, handle):
        self._lib    = _native.load()
        self._handle = handle

    # ── Context manager ────────────────────────────────────────────────────

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    # ── Static ────────────────────────────────────────────────────────────

    @classmethod
    def open(cls, path: str, *, password: str = None, engine: str = "Disk") -> "OverdriveDb":
        """Open or create a database at path."""
        lib = _native.load()
        if password or engine != "Disk":
            opts = json.dumps({"password": password, "auto_create_tables": True})
            handle = lib.overdrive_open_with_engine(
                _native.encode(path),
                _native.encode(engine),
                _native.encode(opts),
            )
        else:
            handle = lib.overdrive_open(_native.encode(path))
        if not handle:
            raise RuntimeError(f"[overdrive] open failed: {_native.last_error(lib)}")
        return cls(handle)

    @staticmethod
    def version() -> str:
        """Return native library version string."""
        lib = _native.load()
        v = lib.overdrive_version()
        return v.decode('utf-8') if v else ''

    # ── Lifecycle ──────────────────────────────────────────────────────────

    def close(self):
        """Close the database and release resources."""
        if self._handle:
            self._lib.overdrive_close(self._handle)
            self._handle = None

    def sync(self):
        """Flush pending writes to disk."""
        self._lib.overdrive_sync(self._handle)

    # ── Tables ────────────────────────────────────────────────────────────

    def create_table(self, name: str):
        """Create a table."""
        if self._lib.overdrive_create_table(self._handle, _native.encode(name)) != 0:
            raise RuntimeError(f"[overdrive] create_table: {_native.last_error(self._lib)}")

    def drop_table(self, name: str):
        """Drop a table."""
        if self._lib.overdrive_drop_table(self._handle, _native.encode(name)) != 0:
            raise RuntimeError(f"[overdrive] drop_table: {_native.last_error(self._lib)}")

    def list_tables(self) -> List[str]:
        """List all table names."""
        ptr = self._lib.overdrive_list_tables(self._handle)
        if not ptr:
            return []
        s = _native.read_free(self._lib, ptr)
        return json.loads(s)

    def table_exists(self, name: str) -> bool:
        """Return True if table exists."""
        return self._lib.overdrive_table_exists(self._handle, _native.encode(name)) == 1

    # ── CRUD ──────────────────────────────────────────────────────────────

    def insert(self, table: str, doc: Dict[str, Any]) -> str:
        """Insert a document. Returns the generated _id."""
        ptr = self._lib.overdrive_insert(
            self._handle, _native.encode(table), _native.encode(json.dumps(doc))
        )
        if not ptr:
            raise RuntimeError(f"[overdrive] insert: {_native.last_error(self._lib)}")
        return _native.read_free(self._lib, ptr)

    def insert_many(self, table: str, docs: List[Dict[str, Any]]) -> List[str]:
        """Insert multiple documents. Returns list of _ids."""
        return [self.insert(table, doc) for doc in docs]

    def get(self, table: str, id: str) -> Optional[Dict[str, Any]]:
        """Get a document by _id. Returns None if not found."""
        ptr = self._lib.overdrive_get(
            self._handle, _native.encode(table), _native.encode(id)
        )
        if not ptr:
            return None
        s = _native.read_free(self._lib, ptr)
        return json.loads(s) if s else None

    def update(self, table: str, id: str, patch: Dict[str, Any]) -> bool:
        """Update a document by _id. Returns True if found and updated."""
        r = self._lib.overdrive_update(
            self._handle,
            _native.encode(table),
            _native.encode(id),
            _native.encode(json.dumps(patch)),
        )
        if r < 0:
            raise RuntimeError(f"[overdrive] update: {_native.last_error(self._lib)}")
        return r == 1

    def delete(self, table: str, id: str) -> bool:
        """Delete a document by _id. Returns True if deleted."""
        r = self._lib.overdrive_delete(
            self._handle, _native.encode(table), _native.encode(id)
        )
        if r < 0:
            raise RuntimeError(f"[overdrive] delete: {_native.last_error(self._lib)}")
        return r == 1

    def count(self, table: str) -> int:
        """Count documents in a table."""
        n = self._lib.overdrive_count(self._handle, _native.encode(table))
        if n < 0:
            raise RuntimeError(f"[overdrive] count: {_native.last_error(self._lib)}")
        return n

    def get_history(self, table: str, id: str) -> List[Dict[str, Any]]:
        """Return the WAL change history for a document since the last sync/close.

        Each entry: {"lsn": int, "op": "INSERT"|"UPDATE"|"DELETE",
                     "data": dict|None, "prev_data": dict|None}

        Example::

            odb.insert("users", {"name": "Alice", "age": 25})   # id = "users_1"
            odb.update("users", "users_1", {"age": 26})
            history = odb.get_history("users", "users_1")
            # [{"lsn":1,"op":"INSERT","data":{...},"prev_data":None},
            #  {"lsn":2,"op":"UPDATE","data":{...},"prev_data":{...}}]
        """
        ptr = self._lib.overdrive_get_history(
            self._handle, _native.encode(table), _native.encode(id)
        )
        if not ptr:
            return []
        s = _native.read_free(self._lib, ptr)
        return json.loads(s) if s else []

    def query_safe(self, sql: str, params: list = None) -> List[Dict[str, Any]]:
        """Parameterised SQL query — prevents SQL injection.

        Placeholders are ``?1``, ``?2``, … in order matching the ``params`` list.

        Example::

            results = odb.query_safe(
                "SELECT * FROM users WHERE age > ?1 AND name = ?2",
                [25, "Alice"]
            )
        """
        params_json = json.dumps(params or [])
        ptr = self._lib.overdrive_query_safe(
            self._handle, _native.encode(sql), _native.encode(params_json)
        )
        if not ptr:
            raise RuntimeError(f"[overdrive] query_safe: {_native.last_error(self._lib)}")
        s = _native.read_free(self._lib, ptr)
        res = json.loads(s) if s else {}
        return res.get('rows', [res] if res else [])

    def backup(self, dest_path: str) -> int:
        """Flush all pending writes and copy the database file to ``dest_path``.

        Returns the number of bytes copied.  The destination is created
        (including parent directories) if it does not exist.  On Linux/macOS
        the backup is hardened to ``chmod 600``.

        Example::

            bytes_copied = odb.backup("backups/myapp.odb")
        """
        ptr = self._lib.overdrive_backup(self._handle, _native.encode(dest_path))
        if not ptr:
            raise RuntimeError(f"[overdrive] backup: {_native.last_error(self._lib)}")
        s = _native.read_free(self._lib, ptr)
        res = json.loads(s) if s else {}
        return res.get('bytes', 0)

    def cleanup_wal(self) -> None:
        """Truncate the WAL file, removing the crash-replay surface.

        Safe to call any time — only removes recovery data, not committed data.
        For guaranteed durability call ``sync()`` first.

        Example::

            odb.sync()        # flush everything to BTree
            odb.cleanup_wal() # then wipe the WAL
        """
        rc = self._lib.overdrive_cleanup_wal(self._handle)
        if rc < 0:
            raise RuntimeError(f"[overdrive] cleanup_wal: {_native.last_error(self._lib)}")

    # ── Query ─────────────────────────────────────────────────────────────


    def query(self, sql: str) -> List[Dict[str, Any]]:
        """Execute a SQL query. Returns a list of row dicts."""
        ptr = self._lib.overdrive_query(self._handle, _native.encode(sql))
        if not ptr:
            raise RuntimeError(f"[overdrive] query: {_native.last_error(self._lib)}")
        s = _native.read_free(self._lib, ptr)
        res = json.loads(s)
        if 'rows' in res:
            return res['rows']
        return [res]

    def search(self, table: str, text: str) -> List[Dict[str, Any]]:
        """Full-text search across a table."""
        ptr = self._lib.overdrive_search(
            self._handle, _native.encode(table), _native.encode(text)
        )
        if not ptr:
            return []
        s = _native.read_free(self._lib, ptr)
        return json.loads(s) if s else []

    # ── Transactions ──────────────────────────────────────────────────────

    def begin_transaction(self, iso: IsolationLevel = IsolationLevel.READ_COMMITTED) -> Transaction:
        """Begin an MVCC transaction."""
        txn_id = self._lib.overdrive_begin_transaction(self._handle, int(iso))
        if not txn_id:
            raise RuntimeError(f"[overdrive] begin_transaction: {_native.last_error(self._lib)}")
        return Transaction(txn_id)

    def commit_transaction(self, txn: Transaction):
        """Commit a transaction."""
        if self._lib.overdrive_commit_transaction(self._handle, txn.id) != 0:
            raise RuntimeError(f"[overdrive] commit: {_native.last_error(self._lib)}")

    def abort_transaction(self, txn: Transaction):
        """Abort (rollback) a transaction."""
        self._lib.overdrive_abort_transaction(self._handle, txn.id)

    def transaction(self, fn, iso: IsolationLevel = IsolationLevel.READ_COMMITTED):
        """Run fn(odb) inside a transaction. Auto-commits or auto-aborts."""
        txn = self.begin_transaction(iso)
        try:
            result = fn(self)
            self.commit_transaction(txn)
            return result
        except Exception:
            self.abort_transaction(txn)
            raise

    # ── Integrity ─────────────────────────────────────────────────────────

    def verify_integrity(self) -> Dict[str, Any]:
        """Run an integrity check. Returns a report dict."""
        ptr = self._lib.overdrive_verify_integrity(self._handle)
        if not ptr:
            raise RuntimeError(f"[overdrive] verify_integrity: {_native.last_error(self._lib)}")
        return json.loads(_native.read_free(self._lib, ptr))

# Aliases for backward and cross-SDK compatibility
OverDrive = OverdriveDb
__all__ = ["OverdriveDb", "OverDrive", "IsolationLevel"]
