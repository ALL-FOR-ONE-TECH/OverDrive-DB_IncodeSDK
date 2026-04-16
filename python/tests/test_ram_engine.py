"""
Unit tests for Task 8 — RAM engine methods in the Python SDK.

These tests validate the SDK-layer logic (argument handling, return-type
shaping, error propagation) without requiring the native library to be
present.  The native FFI calls are patched via unittest.mock so the tests
run in any CI environment.
"""

import json
import ctypes
import unittest
from unittest.mock import MagicMock, patch, PropertyMock

# ---------------------------------------------------------------------------
# Helpers to build a minimal OverDrive instance with a mocked native layer
# ---------------------------------------------------------------------------

def _make_db(handle=0xDEADBEEF):
    """Return an OverDrive instance whose native layer is fully mocked."""
    from overdrive import OverDrive, _Native

    native = MagicMock(spec=_Native)
    native.lib = MagicMock()

    # Default: every FFI call succeeds (returns 0 / non-null pointer)
    native.lib.overdrive_create_table.return_value = 0
    native.lib.overdrive_create_table_with_engine.return_value = 0
    native.lib.overdrive_snapshot.return_value = 0
    native.lib.overdrive_restore.return_value = 0
    native.lib.overdrive_last_error.return_value = None

    db = object.__new__(OverDrive)
    db._native = native
    db._handle = handle
    db._path = "./test.odb"
    return db


def _memory_usage_ptr(native, data: dict):
    """
    Configure native.lib.overdrive_memory_usage to return a fake pointer
    whose value is decoded by _read_and_free into *data*.
    """
    json_bytes = json.dumps(data).encode("utf-8")
    # Use a non-zero sentinel so the SDK doesn't treat it as NULL
    fake_ptr = 0xCAFEBABE

    def _read_and_free_side_effect(n, ptr):
        return json.dumps(data)

    native.lib.overdrive_memory_usage.return_value = fake_ptr
    native.lib.overdrive_free_string.return_value = None

    # Patch _read_and_free at the module level so the SDK uses our version
    return json_bytes, fake_ptr


# ===========================================================================
# 8.1  createTable — engine parameter
# ===========================================================================

class TestCreateTableEngine(unittest.TestCase):

    def test_default_engine_calls_overdrive_create_table(self):
        """create_table(name) with default engine calls the plain FFI."""
        db = _make_db()
        db.create_table("users")
        db._native.lib.overdrive_create_table.assert_called_once_with(
            db._handle, b"users"
        )
        db._native.lib.overdrive_create_table_with_engine.assert_not_called()

    def test_disk_engine_calls_overdrive_create_table(self):
        """create_table(name, engine='Disk') also uses the plain FFI."""
        db = _make_db()
        db.create_table("logs", engine="Disk")
        db._native.lib.overdrive_create_table.assert_called_once_with(
            db._handle, b"logs"
        )
        db._native.lib.overdrive_create_table_with_engine.assert_not_called()

    def test_ram_engine_calls_create_table_with_engine(self):
        """create_table(name, engine='RAM') calls the engine-aware FFI."""
        db = _make_db()
        db.create_table("sessions", engine="RAM")
        db._native.lib.overdrive_create_table_with_engine.assert_called_once_with(
            db._handle, b"sessions", b"RAM"
        )
        db._native.lib.overdrive_create_table.assert_not_called()

    def test_non_disk_engine_passes_engine_string(self):
        """Any non-Disk engine name is forwarded verbatim to the FFI."""
        for engine in ("Vector", "Time-Series", "Graph", "Streaming"):
            with self.subTest(engine=engine):
                db = _make_db()
                db.create_table("t", engine=engine)
                db._native.lib.overdrive_create_table_with_engine.assert_called_once_with(
                    db._handle, b"t", engine.encode("utf-8")
                )

    def test_ffi_error_raises_overdrive_error(self):
        """A non-zero return from the FFI propagates as OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._native.lib.overdrive_create_table_with_engine.return_value = 1
        db._native.lib.overdrive_last_error.return_value = b"engine not supported"
        with self.assertRaises(OverDriveError):
            db.create_table("bad", engine="RAM")

    def test_backward_compat_no_engine_arg(self):
        """Existing callers that pass only a name still work."""
        db = _make_db()
        db.create_table("legacy")  # no engine kwarg
        db._native.lib.overdrive_create_table.assert_called_once()

    def test_closed_db_raises(self):
        """Calling create_table on a closed handle raises OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._handle = None
        with self.assertRaises(OverDriveError):
            db.create_table("t")


# ===========================================================================
# 8.2  snapshot(path)
# ===========================================================================

class TestSnapshot(unittest.TestCase):

    def test_snapshot_calls_ffi_with_encoded_path(self):
        """snapshot() forwards the path as bytes to overdrive_snapshot."""
        db = _make_db()
        db.snapshot("./backups/snap.odb")
        db._native.lib.overdrive_snapshot.assert_called_once_with(
            db._handle, b"./backups/snap.odb"
        )

    def test_snapshot_returns_none_on_success(self):
        """snapshot() returns None when the FFI succeeds."""
        db = _make_db()
        result = db.snapshot("./snap.odb")
        self.assertIsNone(result)

    def test_snapshot_raises_on_ffi_error(self):
        """A non-zero FFI return raises OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._native.lib.overdrive_snapshot.return_value = 1
        db._native.lib.overdrive_last_error.return_value = b"disk full"
        with self.assertRaises(OverDriveError):
            db.snapshot("./snap.odb")

    def test_snapshot_closed_db_raises(self):
        """snapshot() on a closed handle raises OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._handle = None
        with self.assertRaises(OverDriveError):
            db.snapshot("./snap.odb")


# ===========================================================================
# 8.3  restore(path)
# ===========================================================================

class TestRestore(unittest.TestCase):

    def test_restore_calls_ffi_with_encoded_path(self):
        """restore() forwards the path as bytes to overdrive_restore."""
        db = _make_db()
        db.restore("./backups/snap.odb")
        db._native.lib.overdrive_restore.assert_called_once_with(
            db._handle, b"./backups/snap.odb"
        )

    def test_restore_returns_none_on_success(self):
        """restore() returns None when the FFI succeeds."""
        db = _make_db()
        result = db.restore("./snap.odb")
        self.assertIsNone(result)

    def test_restore_raises_on_ffi_error(self):
        """A non-zero FFI return raises OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._native.lib.overdrive_restore.return_value = 1
        db._native.lib.overdrive_last_error.return_value = b"file not found"
        with self.assertRaises(OverDriveError):
            db.restore("./missing.odb")

    def test_restore_closed_db_raises(self):
        """restore() on a closed handle raises OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._handle = None
        with self.assertRaises(OverDriveError):
            db.restore("./snap.odb")


# ===========================================================================
# 8.4  memoryUsage() / memory_usage()
# ===========================================================================

class TestMemoryUsage(unittest.TestCase):

    def _setup_memory_usage(self, db, data: dict):
        """Patch overdrive_memory_usage + _read_and_free for *data*."""
        import overdrive as sdk_module

        fake_ptr = 0xCAFEBABE
        db._native.lib.overdrive_memory_usage.return_value = fake_ptr

        original_raf = sdk_module._read_and_free

        def patched_raf(native, ptr):
            if ptr == fake_ptr:
                return json.dumps(data)
            return original_raf(native, ptr)

        return patch.object(sdk_module, "_read_and_free", side_effect=patched_raf)

    def test_returns_dict_with_required_keys(self):
        """memory_usage() always returns a dict with bytes/mb/limit_bytes/percent."""
        db = _make_db()
        payload = {"bytes": 1048576, "mb": 1.0, "limit_bytes": 2147483648, "percent": 0.05}
        with self._setup_memory_usage(db, payload):
            usage = db.memory_usage()
        self.assertIn("bytes", usage)
        self.assertIn("mb", usage)
        self.assertIn("limit_bytes", usage)
        self.assertIn("percent", usage)

    def test_bytes_is_int(self):
        """The 'bytes' key must be an int."""
        db = _make_db()
        payload = {"bytes": 2097152, "mb": 2.0, "limit_bytes": 4294967296, "percent": 0.05}
        with self._setup_memory_usage(db, payload):
            usage = db.memory_usage()
        self.assertIsInstance(usage["bytes"], int)
        self.assertEqual(usage["bytes"], 2097152)

    def test_mb_is_float(self):
        """The 'mb' key must be a float."""
        db = _make_db()
        payload = {"bytes": 1048576, "mb": 1.0, "limit_bytes": 2147483648, "percent": 0.05}
        with self._setup_memory_usage(db, payload):
            usage = db.memory_usage()
        self.assertIsInstance(usage["mb"], float)

    def test_limit_bytes_is_int(self):
        """The 'limit_bytes' key must be an int."""
        db = _make_db()
        payload = {"bytes": 0, "mb": 0.0, "limit_bytes": 8589934592, "percent": 0.0}
        with self._setup_memory_usage(db, payload):
            usage = db.memory_usage()
        self.assertIsInstance(usage["limit_bytes"], int)
        self.assertEqual(usage["limit_bytes"], 8589934592)

    def test_percent_is_float(self):
        """The 'percent' key must be a float."""
        db = _make_db()
        payload = {"bytes": 524288, "mb": 0.5, "limit_bytes": 1073741824, "percent": 0.049}
        with self._setup_memory_usage(db, payload):
            usage = db.memory_usage()
        self.assertIsInstance(usage["percent"], float)

    def test_null_ptr_returns_zero_dict(self):
        """A NULL pointer from the FFI returns a zeroed dict (no exception)."""
        db = _make_db()
        db._native.lib.overdrive_memory_usage.return_value = 0  # NULL
        db._native.lib.overdrive_last_error.return_value = None
        usage = db.memory_usage()
        self.assertEqual(usage["bytes"], 0)
        self.assertEqual(usage["mb"], 0.0)
        self.assertEqual(usage["limit_bytes"], 0)
        self.assertEqual(usage["percent"], 0.0)

    def test_camelcase_alias_works(self):
        """memoryUsage() is an alias for memory_usage()."""
        from overdrive import OverDrive
        self.assertIs(OverDrive.memoryUsage, OverDrive.memory_usage)

    def test_closed_db_raises(self):
        """memory_usage() on a closed handle raises OverDriveError."""
        from overdrive import OverDriveError
        db = _make_db()
        db._handle = None
        with self.assertRaises(OverDriveError):
            db.memory_usage()


if __name__ == "__main__":
    unittest.main()
