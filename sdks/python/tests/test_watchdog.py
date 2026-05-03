"""
Unit tests for Task 9 — watchdog function in the Python SDK.

Tests validate:
  9.1  OverDrive.watchdog(file_path) static method
  9.2  WatchdogReport dataclass fields and types
  9.3  overdrive_watchdog() FFI call and JSON parsing

The native FFI layer is mocked so tests run without the native library.
"""

import json
import unittest
from unittest.mock import MagicMock, patch

from overdrive import OverDrive, OverDriveError, WatchdogReport


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_VALID_PAYLOAD = {
    "file_path": "./app.odb",
    "file_size_bytes": 8192,
    "last_modified": 1700000000,
    "integrity_status": "valid",
    "corruption_details": None,
    "page_count": 2,
    "magic_valid": True,
}

_CORRUPTED_PAYLOAD = {
    "file_path": "./bad.odb",
    "file_size_bytes": 4096,
    "last_modified": 1699999999,
    "integrity_status": "corrupted",
    "corruption_details": "page checksum mismatch at page 1",
    "page_count": 1,
    "magic_valid": False,
}

_MISSING_PAYLOAD = {
    "file_path": "./missing.odb",
    "file_size_bytes": 0,
    "last_modified": 0,
    "integrity_status": "missing",
    "corruption_details": None,
    "page_count": 0,
    "magic_valid": False,
}


def _patch_watchdog(payload: dict):
    """
    Return a context manager that patches the native layer so that
    overdrive_watchdog returns *payload* as a JSON string.
    """
    import overdrive as sdk_module

    fake_ptr = 0xDEADC0DE

    def _fake_read_and_free(native, ptr):
        if ptr == fake_ptr:
            return json.dumps(payload)
        return None

    native_mock = MagicMock()
    native_mock.lib.overdrive_watchdog.return_value = fake_ptr
    native_mock.lib.overdrive_last_error.return_value = None
    native_mock.lib.overdrive_free_string.return_value = None

    get_native_patch = patch.object(sdk_module, "_get_native", return_value=native_mock)
    read_free_patch = patch.object(sdk_module, "_read_and_free", side_effect=_fake_read_and_free)

    class _CM:
        def __enter__(self):
            self._p1 = get_native_patch.__enter__()
            self._p2 = read_free_patch.__enter__()
            return native_mock

        def __exit__(self, *args):
            get_native_patch.__exit__(*args)
            read_free_patch.__exit__(*args)

    return _CM()


# ===========================================================================
# 9.2  WatchdogReport dataclass
# ===========================================================================

class TestWatchdogReportDataclass(unittest.TestCase):

    def test_can_be_constructed_directly(self):
        """WatchdogReport can be instantiated with all required fields."""
        report = WatchdogReport(
            file_path="./app.odb",
            file_size_bytes=8192,
            last_modified=1700000000,
            integrity_status="valid",
            corruption_details=None,
            page_count=2,
            magic_valid=True,
        )
        self.assertEqual(report.file_path, "./app.odb")
        self.assertEqual(report.file_size_bytes, 8192)
        self.assertEqual(report.last_modified, 1700000000)
        self.assertEqual(report.integrity_status, "valid")
        self.assertIsNone(report.corruption_details)
        self.assertEqual(report.page_count, 2)
        self.assertTrue(report.magic_valid)

    def test_corruption_details_can_be_string(self):
        """corruption_details accepts a non-None string."""
        report = WatchdogReport(
            file_path="./bad.odb",
            file_size_bytes=4096,
            last_modified=1699999999,
            integrity_status="corrupted",
            corruption_details="page checksum mismatch at page 1",
            page_count=1,
            magic_valid=False,
        )
        self.assertEqual(report.corruption_details, "page checksum mismatch at page 1")

    def test_is_dataclass(self):
        """WatchdogReport is a proper dataclass (has __dataclass_fields__)."""
        import dataclasses
        self.assertTrue(dataclasses.is_dataclass(WatchdogReport))

    def test_field_names(self):
        """WatchdogReport exposes exactly the seven required fields."""
        import dataclasses
        field_names = {f.name for f in dataclasses.fields(WatchdogReport)}
        expected = {
            "file_path", "file_size_bytes", "last_modified",
            "integrity_status", "corruption_details", "page_count", "magic_valid",
        }
        self.assertEqual(field_names, expected)


# ===========================================================================
# 9.1  OverDrive.watchdog() — static method
# ===========================================================================

class TestWatchdogStaticMethod(unittest.TestCase):

    def test_is_static_method(self):
        """watchdog() can be called on the class without an instance."""
        # Should not raise AttributeError
        self.assertTrue(callable(OverDrive.watchdog))

    def test_returns_watchdog_report(self):
        """watchdog() returns a WatchdogReport instance."""
        with _patch_watchdog(_VALID_PAYLOAD):
            report = OverDrive.watchdog("./app.odb")
        self.assertIsInstance(report, WatchdogReport)

    def test_valid_file_fields(self):
        """watchdog() correctly maps all JSON fields for a valid file."""
        with _patch_watchdog(_VALID_PAYLOAD):
            report = OverDrive.watchdog("./app.odb")
        self.assertEqual(report.file_path, "./app.odb")
        self.assertEqual(report.file_size_bytes, 8192)
        self.assertEqual(report.last_modified, 1700000000)
        self.assertEqual(report.integrity_status, "valid")
        self.assertIsNone(report.corruption_details)
        self.assertEqual(report.page_count, 2)
        self.assertTrue(report.magic_valid)

    def test_corrupted_file_fields(self):
        """watchdog() correctly maps fields when the file is corrupted."""
        with _patch_watchdog(_CORRUPTED_PAYLOAD):
            report = OverDrive.watchdog("./bad.odb")
        self.assertEqual(report.integrity_status, "corrupted")
        self.assertEqual(report.corruption_details, "page checksum mismatch at page 1")
        self.assertFalse(report.magic_valid)

    def test_missing_file_fields(self):
        """watchdog() correctly maps fields when the file is missing."""
        with _patch_watchdog(_MISSING_PAYLOAD):
            report = OverDrive.watchdog("./missing.odb")
        self.assertEqual(report.integrity_status, "missing")
        self.assertEqual(report.file_size_bytes, 0)
        self.assertEqual(report.last_modified, 0)
        self.assertEqual(report.page_count, 0)
        self.assertFalse(report.magic_valid)


# ===========================================================================
# 9.3  FFI binding — overdrive_watchdog call and JSON parsing
# ===========================================================================

class TestWatchdogFFIBinding(unittest.TestCase):

    def test_ffi_called_with_encoded_path(self):
        """watchdog() encodes the path to bytes before calling the FFI."""
        with _patch_watchdog(_VALID_PAYLOAD) as native_mock:
            OverDrive.watchdog("./app.odb")
        native_mock.lib.overdrive_watchdog.assert_called_once_with(b"./app.odb")

    def test_ffi_called_with_unicode_path(self):
        """watchdog() correctly encodes a unicode path."""
        payload = dict(_VALID_PAYLOAD, file_path="./données.odb")
        with _patch_watchdog(payload) as native_mock:
            OverDrive.watchdog("./données.odb")
        native_mock.lib.overdrive_watchdog.assert_called_once_with(
            "./données.odb".encode("utf-8")
        )

    def test_null_ptr_raises_overdrive_error(self):
        """A NULL pointer from the FFI raises OverDriveError."""
        import overdrive as sdk_module

        native_mock = MagicMock()
        native_mock.lib.overdrive_watchdog.return_value = 0  # NULL
        native_mock.lib.overdrive_last_error.return_value = None

        with patch.object(sdk_module, "_get_native", return_value=native_mock):
            with self.assertRaises(OverDriveError):
                OverDrive.watchdog("./app.odb")

    def test_null_ptr_with_error_message_raises(self):
        """A NULL pointer with a native error message propagates the message."""
        import overdrive as sdk_module

        native_mock = MagicMock()
        native_mock.lib.overdrive_watchdog.return_value = 0  # NULL
        native_mock.lib.overdrive_last_error.return_value = b"permission denied"

        with patch.object(sdk_module, "_get_native", return_value=native_mock):
            with self.assertRaises(OverDriveError) as ctx:
                OverDrive.watchdog("./app.odb")
        self.assertIn("permission denied", str(ctx.exception))

    def test_file_size_bytes_is_int(self):
        """file_size_bytes is cast to int even if the JSON contains a float."""
        payload = dict(_VALID_PAYLOAD, file_size_bytes=8192.0)
        with _patch_watchdog(payload):
            report = OverDrive.watchdog("./app.odb")
        self.assertIsInstance(report.file_size_bytes, int)
        self.assertEqual(report.file_size_bytes, 8192)

    def test_last_modified_is_int(self):
        """last_modified is cast to int."""
        payload = dict(_VALID_PAYLOAD, last_modified=1700000000.9)
        with _patch_watchdog(payload):
            report = OverDrive.watchdog("./app.odb")
        self.assertIsInstance(report.last_modified, int)

    def test_page_count_is_int(self):
        """page_count is cast to int."""
        payload = dict(_VALID_PAYLOAD, page_count=4.0)
        with _patch_watchdog(payload):
            report = OverDrive.watchdog("./app.odb")
        self.assertIsInstance(report.page_count, int)
        self.assertEqual(report.page_count, 4)

    def test_magic_valid_is_bool(self):
        """magic_valid is cast to bool."""
        payload = dict(_VALID_PAYLOAD, magic_valid=1)
        with _patch_watchdog(payload):
            report = OverDrive.watchdog("./app.odb")
        self.assertIsInstance(report.magic_valid, bool)
        self.assertTrue(report.magic_valid)

    def test_missing_json_fields_use_defaults(self):
        """watchdog() uses safe defaults when optional JSON fields are absent."""
        minimal = {"integrity_status": "missing"}
        with _patch_watchdog(minimal):
            report = OverDrive.watchdog("./app.odb")
        self.assertEqual(report.file_size_bytes, 0)
        self.assertEqual(report.last_modified, 0)
        self.assertEqual(report.page_count, 0)
        self.assertFalse(report.magic_valid)
        self.assertIsNone(report.corruption_details)


if __name__ == "__main__":
    unittest.main()
