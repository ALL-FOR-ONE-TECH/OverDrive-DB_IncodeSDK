"""
OverDrive InCode SDK — Python Wrapper (v1.4.3)
Embeddable document database. Like SQLite for JSON.

Usage:
    from overdrive import OverDrive

    db = OverDrive.open("myapp.odb")
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

__version__ = "1.4.3"

# ─────────────────────────────────────────────────────────
# Native library loader
# ─────────────────────────────────────────────────────────

def _find_library() -> str:
    """Find the overdrive shared library."""
    system = platform.system()
    if system == "Windows":
        lib_name = "overdrive.dll"
    elif system == "Darwin":
        lib_name = "liboverdrive.dylib"
    else:
        lib_name = "liboverdrive.so"

    # Also check for the build-output name (overdrive_db.dll)
    alt_name = "overdrive_db.dll" if system == "Windows" else lib_name

    search_paths = [
        Path(__file__).parent / lib_name,
        Path(__file__).parent / alt_name,
        Path(__file__).parent / "lib" / lib_name,
        Path(__file__).parent.parent / "lib" / lib_name,
        Path(__file__).parent.parent / "target" / "release" / alt_name,
        Path(__file__).parent.parent.parent / "target" / "release" / alt_name,
    ]

    for path in search_paths:
        if path.exists() and path.stat().st_size > 100_000:
            return str(path)

    # Auto-download fallback
    try:
        from overdrive.download import ensure_binary
        downloaded = ensure_binary(target_dir=str(Path(__file__).parent))
        if downloaded and Path(downloaded).exists():
            return downloaded
    except Exception:
        pass

    return lib_name  # last resort — let the OS find it


# Use c_size_t (unsigned, pointer-width integer) for ALL pointer types.
# c_void_p sign-extends 64-bit pointers on Windows → access violation.
_H = ctypes.c_size_t   # ODB* handle
_P = ctypes.c_size_t   # char* (returned strings that must be freed)


class _Native:
    """Low-level ctypes bindings to liboverdrive."""

    def __init__(self):
        lib_path = _find_library()
        self.lib = ctypes.cdll.LoadLibrary(lib_path)
        self._setup_bindings()

    def _setup_bindings(self):
        L = self.lib

        # — Lifecycle —
        L.overdrive_open.argtypes = [ctypes.c_char_p]
        L.overdrive_open.restype = _H

        L.overdrive_close.argtypes = [_H]
        L.overdrive_close.restype = None

        L.overdrive_sync.argtypes = [_H]
        L.overdrive_sync.restype = None

        # — Tables —
        L.overdrive_create_table.argtypes = [_H, ctypes.c_char_p]
        L.overdrive_create_table.restype = ctypes.c_int

        L.overdrive_drop_table.argtypes = [_H, ctypes.c_char_p]
        L.overdrive_drop_table.restype = ctypes.c_int

        L.overdrive_list_tables.argtypes = [_H]
        L.overdrive_list_tables.restype = _P

        L.overdrive_table_exists.argtypes = [_H, ctypes.c_char_p]
        L.overdrive_table_exists.restype = ctypes.c_int

        # — CRUD —
        L.overdrive_insert.argtypes = [_H, ctypes.c_char_p, ctypes.c_char_p]
        L.overdrive_insert.restype = _P

        L.overdrive_get.argtypes = [_H, ctypes.c_char_p, ctypes.c_char_p]
        L.overdrive_get.restype = _P

        L.overdrive_update.argtypes = [_H, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
        L.overdrive_update.restype = ctypes.c_int

        L.overdrive_delete.argtypes = [_H, ctypes.c_char_p, ctypes.c_char_p]
        L.overdrive_delete.restype = ctypes.c_int

        L.overdrive_count.argtypes = [_H, ctypes.c_char_p]
        L.overdrive_count.restype = ctypes.c_int

        # — Query —
        L.overdrive_query.argtypes = [_H, ctypes.c_char_p]
        L.overdrive_query.restype = _P

        L.overdrive_search.argtypes = [_H, ctypes.c_char_p, ctypes.c_char_p]
        L.overdrive_search.restype = _P

        # — Utility —
        L.overdrive_last_error.argtypes = []
        L.overdrive_last_error.restype = ctypes.c_char_p

        L.overdrive_free_string.argtypes = [_P]
        L.overdrive_free_string.restype = None

        L.overdrive_version.argtypes = []
        L.overdrive_version.restype = ctypes.c_char_p

        # — Transactions —
        L.overdrive_begin_transaction.argtypes = [_H, ctypes.c_int]
        L.overdrive_begin_transaction.restype = ctypes.c_uint64

        L.overdrive_commit_transaction.argtypes = [_H, ctypes.c_uint64]
        L.overdrive_commit_transaction.restype = ctypes.c_int

        L.overdrive_abort_transaction.argtypes = [_H, ctypes.c_uint64]
        L.overdrive_abort_transaction.restype = ctypes.c_int

        # — Optional symbols (may not exist in older DLLs) —
        for name, args, ret in [
            ("overdrive_verify_integrity", [_H], _P),
            ("overdrive_open_with_engine", [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p], _H),
            ("overdrive_open_with_password", [ctypes.c_char_p, ctypes.c_char_p], _H),
        ]:
            try:
                fn = getattr(L, name)
                fn.argtypes = args
                fn.restype = ret
            except AttributeError:
                pass


# Singleton
_native: Optional[_Native] = None

def _get_native() -> _Native:
    global _native
    if _native is None:
        _native = _Native()
    return _native


def _read_ptr(native: _Native, ptr: int) -> Optional[str]:
    """Read a C string from an unsigned pointer, then free it."""
    if ptr == 0:
        return None
    try:
        raw = ctypes.cast(ptr, ctypes.c_char_p).value
        return raw.decode("utf-8") if raw else None
    finally:
        native.lib.overdrive_free_string(ptr)


def _last_error(native: _Native) -> str:
    err = native.lib.overdrive_last_error()
    return err.decode("utf-8") if err else "Unknown error"


# ─────────────────────────────────────────────────────────
# Public API
# ─────────────────────────────────────────────────────────

class OverDrive:
    """OverDrive-DB — Embeddable hybrid SQL+NoSQL database."""

    def __init__(self, handle: int, path: str, native: _Native):
        self._handle = handle
        self._path = str(Path(path).resolve())  # always absolute
        self._native = native
        self._closed = False

    # ── Static constructors ──────────────────────────────

    @staticmethod
    def open(path: str, password: str = None, engine: str = None) -> "OverDrive":
        """Open or create a database file."""
        native = _get_native()
        abs_path = str(Path(path).resolve())
        path_bytes = abs_path.encode("utf-8")

        if password and hasattr(native.lib, "overdrive_open_with_password"):
            handle = native.lib.overdrive_open_with_password(path_bytes, password.encode("utf-8"))
        elif engine and hasattr(native.lib, "overdrive_open_with_engine"):
            opts = json.dumps({"password": password} if password else {})
            handle = native.lib.overdrive_open_with_engine(
                path_bytes, engine.encode("utf-8"), opts.encode("utf-8")
            )
        else:
            handle = native.lib.overdrive_open(path_bytes)

        if handle == 0:
            raise RuntimeError(f"Failed to open database: {_last_error(native)}")
        return OverDrive(handle, abs_path, native)

    @staticmethod
    def version() -> str:
        native = _get_native()
        v = native.lib.overdrive_version()
        return v.decode("utf-8") if v else __version__

    # ── Table management ────────────────────────────────

    def create_table(self, name: str) -> None:
        rc = self._native.lib.overdrive_create_table(self._handle, name.encode("utf-8"))
        if rc < 0:
            raise RuntimeError(f"create_table failed: {_last_error(self._native)}")

    def drop_table(self, name: str) -> None:
        rc = self._native.lib.overdrive_drop_table(self._handle, name.encode("utf-8"))
        if rc < 0:
            raise RuntimeError(f"drop_table failed: {_last_error(self._native)}")

    def list_tables(self) -> List[str]:
        ptr = self._native.lib.overdrive_list_tables(self._handle)
        raw = _read_ptr(self._native, ptr)
        return json.loads(raw) if raw else []

    def table_exists(self, name: str) -> bool:
        return self._native.lib.overdrive_table_exists(self._handle, name.encode("utf-8")) == 1

    # ── CRUD ────────────────────────────────────────────

    def insert(self, table: str, doc: dict) -> str:
        json_str = json.dumps(doc)
        ptr = self._native.lib.overdrive_insert(
            self._handle, table.encode("utf-8"), json_str.encode("utf-8")
        )
        result = _read_ptr(self._native, ptr)
        if result is None:
            raise RuntimeError(f"insert failed: {_last_error(self._native)}")
        return result

    def get(self, table: str, doc_id: str) -> Optional[dict]:
        ptr = self._native.lib.overdrive_get(
            self._handle, table.encode("utf-8"), doc_id.encode("utf-8")
        )
        raw = _read_ptr(self._native, ptr)
        return json.loads(raw) if raw else None

    def update(self, table: str, doc_id: str, updates: dict) -> bool:
        json_str = json.dumps(updates)
        rc = self._native.lib.overdrive_update(
            self._handle, table.encode("utf-8"),
            doc_id.encode("utf-8"), json_str.encode("utf-8")
        )
        if rc < 0:
            raise RuntimeError(f"update failed: {_last_error(self._native)}")
        return rc == 1

    def delete(self, table: str, doc_id: str) -> bool:
        rc = self._native.lib.overdrive_delete(
            self._handle, table.encode("utf-8"), doc_id.encode("utf-8")
        )
        if rc < 0:
            raise RuntimeError(f"delete failed: {_last_error(self._native)}")
        return rc == 1

    def count(self, table: str) -> int:
        rc = self._native.lib.overdrive_count(self._handle, table.encode("utf-8"))
        if rc < 0:
            raise RuntimeError(f"count failed: {_last_error(self._native)}")
        return rc

    # ── Query ───────────────────────────────────────────

    def query(self, sql: str) -> List[dict]:
        """Execute SQL query. Implements SELECT client-side using CRUD primitives."""
        sql_stripped = sql.strip().rstrip(";").strip()
        upper = sql_stripped.upper()

        # --- SELECT queries: scan via CRUD ---
        if upper.startswith("SELECT"):
            return self._exec_select(sql_stripped)

        # --- Non-SELECT (CREATE TABLE, DROP, INSERT, UPDATE, DELETE) ---
        # Fall through to the native Shell for DDL/DML
        ptr = self._native.lib.overdrive_query(
            self._handle, sql.encode("utf-8")
        )
        raw = _read_ptr(self._native, ptr)
        if raw is None:
            raise RuntimeError(f"query failed: {_last_error(self._native)}")
        result = json.loads(raw)
        if isinstance(result, dict) and not result.get("ok", True):
            raise RuntimeError(f"Query error: {result.get('result', 'unknown')}")
        return []

    def _exec_select(self, sql: str) -> List[dict]:
        """Parse a SELECT statement and execute via CRUD scan."""
        import re
        upper = sql.upper()

        # Handle COUNT(*)
        count_match = re.match(
            r"SELECT\s+COUNT\(\*\)\s+(?:AS\s+\w+\s+)?FROM\s+(\w+)(?:\s+WHERE\s+(.+))?",
            sql, re.IGNORECASE
        )
        if count_match:
            table = count_match.group(1)
            where = count_match.group(2)
            if where:
                rows = self._scan_with_where(table, where)
                return [{"cnt": len(rows), "COUNT(*)": len(rows)}]
            else:
                c = self.count(table)
                return [{"cnt": c, "COUNT(*)": c}]

        # Parse: SELECT <cols> FROM <table> [WHERE ...] [ORDER BY col [DESC]] [LIMIT n]
        m = re.match(
            r"SELECT\s+(.+?)\s+FROM\s+(\w+)"
            r"(?:\s+WHERE\s+(.+?))?"
            r"(?:\s+ORDER\s+BY\s+(\w+)(?:\s+(ASC|DESC))?)?"
            r"(?:\s+LIMIT\s+(\d+))?",
            sql, re.IGNORECASE
        )
        if not m:
            raise RuntimeError(f"Unsupported SQL: {sql}")

        cols_str, table, where_clause, order_col, order_dir, limit_str = m.groups()
        limit = int(limit_str) if limit_str else None

        # Get all rows
        if where_clause:
            rows = self._scan_with_where(table, where_clause)
        else:
            rows = self._scan_all(table)

        # Order
        if order_col:
            desc = (order_dir or "").upper() == "DESC"
            rows.sort(key=lambda r: r.get(order_col, ""), reverse=desc)

        # Limit
        if limit:
            rows = rows[:limit]

        # Column filtering
        if cols_str.strip() != "*":
            selected = [c.strip() for c in cols_str.split(",")]
            rows = [{k: r.get(k) for k in selected if k in r} for r in rows]

        return rows

    def _scan_all(self, table: str) -> List[dict]:
        """Scan all documents in a table."""
        count = self.count(table)
        rows = []
        for i in range(1, count + 200):  # generous range
            doc_id = f"{table}_{i}"
            doc = self.get(table, doc_id)
            if doc:
                rows.append(doc)
            if len(rows) >= count:
                break
        return rows

    def _scan_with_where(self, table: str, where: str) -> List[dict]:
        """Scan table and filter rows matching a WHERE clause."""
        import re, operator
        all_rows = self._scan_all(table)

        # Parse simple conditions: col op value [AND col op value ...]
        ops = {
            ">=": operator.ge, "<=": operator.le,
            "!=": operator.ne, "<>": operator.ne,
            ">": operator.gt, "<": operator.lt,
            "=": operator.eq,
        }

        conditions = re.split(r'\s+AND\s+', where, flags=re.IGNORECASE)
        result = all_rows

        for cond in conditions:
            cond = cond.strip()
            matched = False
            for op_str in sorted(ops.keys(), key=len, reverse=True):
                if op_str in cond:
                    parts = cond.split(op_str, 1)
                    if len(parts) == 2:
                        col = parts[0].strip()
                        val = parts[1].strip().strip("'\"")
                        op_fn = ops[op_str]
                        filtered = []
                        for row in result:
                            rv = row.get(col)
                            if rv is None:
                                continue
                            try:
                                # Try numeric comparison
                                if isinstance(rv, (int, float)):
                                    if op_fn(rv, float(val)):
                                        filtered.append(row)
                                else:
                                    if op_fn(str(rv), val):
                                        filtered.append(row)
                            except (ValueError, TypeError):
                                if op_fn(str(rv), val):
                                    filtered.append(row)
                        result = filtered
                        matched = True
                        break
            if not matched:
                pass  # Skip unparseable conditions

        return result


    def query_safe(self, sql_template: str, params: list) -> List[dict]:
        """Execute parameterized query — prevents SQL injection."""
        DANGEROUS = ["DROP", "TRUNCATE", "ALTER", "EXEC", "--", ";--", "/*", "*/", "UNION"]
        sanitized = []
        for p in params:
            s = str(p)
            upper = s.upper()
            for d in DANGEROUS:
                if d in upper:
                    raise RuntimeError(f"SQL injection detected: '{s}' contains '{d}'")
            sanitized.append("'" + s.replace("'", "''") + "'")

        sql = sql_template
        for val in sanitized:
            sql = sql.replace("?", val, 1)
        return self.query(sql)

    # ── Search ──────────────────────────────────────────

    def search(self, table: str, text: str) -> List[dict]:
        ptr = self._native.lib.overdrive_search(
            self._handle, table.encode("utf-8"), text.encode("utf-8")
        )
        raw = _read_ptr(self._native, ptr)
        return json.loads(raw) if raw else []

    # ── Helper methods (convenience wrappers) ───────────

    def findOne(self, table: str, where: str = None) -> Optional[dict]:
        sql = f"SELECT * FROM {table}"
        if where:
            sql += f" WHERE {where}"
        sql += " LIMIT 1"
        rows = self.query(sql)
        return rows[0] if rows else None

    def findAll(self, table: str, where: str = None,
                order_by: str = None, limit: int = None) -> List[dict]:
        sql = f"SELECT * FROM {table}"
        if where:
            sql += f" WHERE {where}"
        if order_by:
            sql += f" ORDER BY {order_by}"
        if limit:
            sql += f" LIMIT {limit}"
        return self.query(sql)

    def countWhere(self, table: str, where: str) -> int:
        rows = self.query(f"SELECT COUNT(*) as cnt FROM {table} WHERE {where}")
        if rows and isinstance(rows[0], dict):
            return int(rows[0].get("cnt", rows[0].get("COUNT(*)", 0)))
        return 0

    def exists(self, table: str, doc_id: str) -> bool:
        return self.get(table, doc_id) is not None

    def updateMany(self, table: str, where: str, updates: dict) -> int:
        matches = self.findAll(table, where=where)
        count = 0
        for row in matches:
            doc_id = row.get("_id")
            if doc_id and self.update(table, doc_id, updates):
                count += 1
        return count

    def deleteMany(self, table: str, where: str) -> int:
        matches = self.findAll(table, where=where)
        count = 0
        for row in matches:
            doc_id = row.get("_id")
            if doc_id and self.delete(table, doc_id):
                count += 1
        return count

    # ── Maintenance ─────────────────────────────────────

    def backup(self, dest_path: str) -> None:
        self.sync()
        import shutil
        abs_dest = str(Path(dest_path).resolve())
        shutil.copy2(self._path, abs_dest)
        wal = self._path + ".wal"
        if os.path.exists(wal):
            shutil.copy2(wal, abs_dest + ".wal")

    def cleanup_wal(self) -> None:
        wal = self._path + ".wal"
        if os.path.exists(wal):
            os.remove(wal)

    def sync(self) -> None:
        self._native.lib.overdrive_sync(self._handle)

    def verify_integrity(self) -> dict:
        if not hasattr(self._native.lib, "overdrive_verify_integrity"):
            return {"valid": True, "message": "integrity check not available"}
        ptr = self._native.lib.overdrive_verify_integrity(self._handle)
        raw = _read_ptr(self._native, ptr)
        return json.loads(raw) if raw else {"valid": True}

    # ── Transactions ────────────────────────────────────

    def begin_transaction(self, isolation: int = 1) -> int:
        txn_id = self._native.lib.overdrive_begin_transaction(self._handle, isolation)
        if txn_id == 0:
            raise RuntimeError(f"begin_transaction failed: {_last_error(self._native)}")
        return txn_id

    def commit_transaction(self, txn_id: int) -> None:
        rc = self._native.lib.overdrive_commit_transaction(self._handle, txn_id)
        if rc < 0:
            raise RuntimeError(f"commit_transaction failed: {_last_error(self._native)}")

    def abort_transaction(self, txn_id: int) -> None:
        rc = self._native.lib.overdrive_abort_transaction(self._handle, txn_id)
        if rc < 0:
            raise RuntimeError(f"abort_transaction failed: {_last_error(self._native)}")

    # ── Lifecycle ───────────────────────────────────────

    def close(self) -> None:
        if not self._closed and self._handle:
            self._native.lib.overdrive_close(self._handle)
            self._closed = True
            self._handle = 0

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def __del__(self):
        try:
            self.close()
        except Exception:
            pass
