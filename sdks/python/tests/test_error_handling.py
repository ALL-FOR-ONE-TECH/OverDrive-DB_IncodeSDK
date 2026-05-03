"""
Tests for Task 12: Enhanced error handling in Python SDK.

Covers:
  12.1  OverDriveError base class
  12.2  Specific error subclasses
  12.3  Structured JSON parsing in _check_error()
  12.4  Error attributes (code, message, context, suggestions, doc_link)
  12.5  transaction_with_retry() with exponential backoff
"""

import json
import time
import unittest
from unittest.mock import MagicMock, patch, PropertyMock

from overdrive import (
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
    _make_error,
    _check_error,
)


# ---------------------------------------------------------------------------
# 12.1 – OverDriveError base class
# ---------------------------------------------------------------------------

class TestOverDriveErrorBase(unittest.TestCase):

    def test_is_exception_subclass(self):
        self.assertTrue(issubclass(OverDriveError, Exception))

    def test_plain_message(self):
        err = OverDriveError("something went wrong")
        self.assertIn("something went wrong", str(err))
        self.assertEqual(err.message, "something went wrong")

    def test_default_attributes(self):
        err = OverDriveError("msg")
        self.assertEqual(err.code, "")
        self.assertEqual(err.context, "")
        self.assertEqual(err.suggestions, [])
        self.assertEqual(err.doc_link, "")

    def test_rich_attributes(self):
        err = OverDriveError(
            "bad password",
            code="ODB-AUTH-001",
            context="app.odb",
            suggestions=["Check your password", "Try again"],
            doc_link="https://overdrive-db.com/docs/errors/ODB-AUTH-001",
        )
        self.assertEqual(err.code, "ODB-AUTH-001")
        self.assertEqual(err.context, "app.odb")
        self.assertEqual(err.suggestions, ["Check your password", "Try again"])
        self.assertEqual(err.doc_link, "https://overdrive-db.com/docs/errors/ODB-AUTH-001")

    def test_str_includes_code_and_message(self):
        err = OverDriveError("bad password", code="ODB-AUTH-001")
        s = str(err)
        self.assertIn("ODB-AUTH-001", s)
        self.assertIn("bad password", s)

    def test_str_includes_suggestions(self):
        err = OverDriveError("oops", suggestions=["Do this", "Do that"])
        s = str(err)
        self.assertIn("Do this", s)
        self.assertIn("Do that", s)

    def test_str_includes_doc_link(self):
        err = OverDriveError("oops", doc_link="https://example.com/err")
        self.assertIn("https://example.com/err", str(err))


# ---------------------------------------------------------------------------
# 12.2 – Specific error subclasses
# ---------------------------------------------------------------------------

class TestErrorSubclasses(unittest.TestCase):

    def _assert_subclass(self, cls):
        self.assertTrue(issubclass(cls, OverDriveError))
        instance = cls("test")
        self.assertIsInstance(instance, OverDriveError)
        self.assertIsInstance(instance, cls)

    def test_authentication_error(self):
        self._assert_subclass(AuthenticationError)

    def test_table_error(self):
        self._assert_subclass(TableError)

    def test_query_error(self):
        self._assert_subclass(QueryError)

    def test_transaction_error(self):
        self._assert_subclass(TransactionError)

    def test_overdrive_io_error(self):
        self._assert_subclass(OverDriveIOError)

    def test_ffi_error(self):
        self._assert_subclass(FFIError)

    def test_overdrive_io_error_does_not_shadow_builtin(self):
        # The class must NOT be named IOError to avoid shadowing the builtin
        self.assertNotEqual(OverDriveIOError.__name__, "IOError")
        self.assertEqual(OverDriveIOError.__name__, "OverDriveIOError")

    def test_subclasses_inherit_attributes(self):
        err = AuthenticationError(
            "wrong password",
            code="ODB-AUTH-001",
            suggestions=["Check password"],
        )
        self.assertEqual(err.code, "ODB-AUTH-001")
        self.assertEqual(err.suggestions, ["Check password"])


# ---------------------------------------------------------------------------
# 12.3 / 12.4 – _make_error() prefix routing + attributes
# ---------------------------------------------------------------------------

class TestMakeError(unittest.TestCase):

    def _make(self, code, **kw):
        return _make_error("msg", code=code, **kw)

    def test_auth_prefix_gives_authentication_error(self):
        self.assertIsInstance(self._make("ODB-AUTH-001"), AuthenticationError)

    def test_table_prefix_gives_table_error(self):
        self.assertIsInstance(self._make("ODB-TABLE-001"), TableError)

    def test_query_prefix_gives_query_error(self):
        self.assertIsInstance(self._make("ODB-QUERY-001"), QueryError)

    def test_txn_prefix_gives_transaction_error(self):
        self.assertIsInstance(self._make("ODB-TXN-001"), TransactionError)

    def test_io_prefix_gives_overdrive_io_error(self):
        self.assertIsInstance(self._make("ODB-IO-001"), OverDriveIOError)

    def test_ffi_prefix_gives_ffi_error(self):
        self.assertIsInstance(self._make("ODB-FFI-001"), FFIError)

    def test_unknown_prefix_gives_base_error(self):
        self.assertIsInstance(self._make("ODB-UNKNOWN-001"), OverDriveError)
        self.assertNotIsInstance(self._make("ODB-UNKNOWN-001"), AuthenticationError)

    def test_empty_code_gives_base_error(self):
        err = _make_error("plain error")
        self.assertIsInstance(err, OverDriveError)
        self.assertEqual(err.code, "")

    def test_attributes_propagated(self):
        err = _make_error(
            "table missing",
            code="ODB-TABLE-001",
            context="users",
            suggestions=["Create the table"],
            doc_link="https://overdrive-db.com/docs/errors/ODB-TABLE-001",
        )
        self.assertEqual(err.code, "ODB-TABLE-001")
        self.assertEqual(err.context, "users")
        self.assertEqual(err.suggestions, ["Create the table"])
        self.assertEqual(err.doc_link, "https://overdrive-db.com/docs/errors/ODB-TABLE-001")


# ---------------------------------------------------------------------------
# 12.3 – _check_error() structured JSON parsing
# ---------------------------------------------------------------------------

def _make_native_mock(details_json=None, last_error_str=None):
    """Build a minimal mock _Native object."""
    native = MagicMock()
    if details_json is not None:
        native.lib.overdrive_get_error_details.return_value = (
            details_json.encode("utf-8") if isinstance(details_json, str) else details_json
        )
    else:
        native.lib.overdrive_get_error_details.return_value = None

    if last_error_str is not None:
        native.lib.overdrive_last_error.return_value = (
            last_error_str.encode("utf-8") if isinstance(last_error_str, str) else last_error_str
        )
    else:
        native.lib.overdrive_last_error.return_value = None

    return native


class TestCheckError(unittest.TestCase):

    def test_no_error_does_not_raise(self):
        native = _make_native_mock(details_json=None, last_error_str=None)
        # Should not raise
        _check_error(native)

    def test_plain_string_fallback_raises_base_error(self):
        native = _make_native_mock(details_json=None, last_error_str="something failed")
        with self.assertRaises(OverDriveError) as ctx:
            _check_error(native)
        self.assertIn("something failed", str(ctx.exception))

    def test_structured_json_raises_correct_subclass(self):
        payload = json.dumps({
            "code": "ODB-AUTH-001",
            "message": "Incorrect password",
            "context": "app.odb",
            "suggestions": ["Check your password"],
            "doc_link": "https://overdrive-db.com/docs/errors/ODB-AUTH-001",
        })
        native = _make_native_mock(details_json=payload)
        with self.assertRaises(AuthenticationError) as ctx:
            _check_error(native)
        err = ctx.exception
        self.assertEqual(err.code, "ODB-AUTH-001")
        self.assertEqual(err.message, "Incorrect password")
        self.assertEqual(err.context, "app.odb")
        self.assertIn("Check your password", err.suggestions)
        self.assertIn("overdrive-db.com", err.doc_link)

    def test_structured_json_table_error(self):
        payload = json.dumps({
            "code": "ODB-TABLE-001",
            "message": "Table 'users' not found",
            "context": "users",
            "suggestions": ["Create the table first"],
            "doc_link": "",
        })
        native = _make_native_mock(details_json=payload)
        with self.assertRaises(TableError):
            _check_error(native)

    def test_structured_json_txn_error(self):
        payload = json.dumps({
            "code": "ODB-TXN-002",
            "message": "Transaction conflict",
            "context": "",
            "suggestions": ["Retry the transaction"],
            "doc_link": "",
        })
        native = _make_native_mock(details_json=payload)
        with self.assertRaises(TransactionError):
            _check_error(native)

    def test_structured_json_io_error(self):
        payload = json.dumps({
            "code": "ODB-IO-001",
            "message": "File not found",
            "context": "/tmp/missing.odb",
            "suggestions": [],
            "doc_link": "",
        })
        native = _make_native_mock(details_json=payload)
        with self.assertRaises(OverDriveIOError):
            _check_error(native)

    def test_structured_json_ffi_error(self):
        payload = json.dumps({
            "code": "ODB-FFI-001",
            "message": "Library not found",
            "context": "overdrive.dll",
            "suggestions": ["Reinstall the package"],
            "doc_link": "",
        })
        native = _make_native_mock(details_json=payload)
        with self.assertRaises(FFIError):
            _check_error(native)

    def test_malformed_json_falls_back_to_plain_string(self):
        native = _make_native_mock(
            details_json="{not valid json",
            last_error_str="fallback error",
        )
        with self.assertRaises(OverDriveError) as ctx:
            _check_error(native)
        # Should have fallen back to the plain string
        self.assertIn("fallback error", str(ctx.exception))

    def test_structured_json_preferred_over_plain_string(self):
        payload = json.dumps({
            "code": "ODB-QUERY-001",
            "message": "Syntax error",
            "context": "",
            "suggestions": [],
            "doc_link": "",
        })
        native = _make_native_mock(
            details_json=payload,
            last_error_str="plain fallback",
        )
        with self.assertRaises(QueryError) as ctx:
            _check_error(native)
        # Must use the structured error, not the plain string
        self.assertEqual(ctx.exception.code, "ODB-QUERY-001")


# ---------------------------------------------------------------------------
# 12.5 – transaction_with_retry()
# ---------------------------------------------------------------------------

class TestTransactionWithRetry(unittest.TestCase):
    """
    Tests for OverDrive.transaction_with_retry().

    We mock the underlying transaction() method so no real database is needed.
    """

    def _make_db(self):
        """Return a minimal OverDrive-like object with a mocked transaction()."""
        from overdrive import OverDrive
        db = object.__new__(OverDrive)
        db._handle = MagicMock()
        db._native = MagicMock()
        db._path = ":memory:"
        return db

    def test_success_on_first_attempt(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(return_value=42)

        result = db.transaction_with_retry(lambda txn: None)
        self.assertEqual(result, 42)
        self.assertEqual(db.transaction.call_count, 1)

    def test_returns_callback_value(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(return_value={"id": "users_1"})

        result = db.transaction_with_retry(lambda txn: None)
        self.assertEqual(result, {"id": "users_1"})

    def test_retries_on_transaction_error(self):
        from overdrive import OverDrive
        db = self._make_db()
        # Fail twice, succeed on third attempt
        db.transaction = MagicMock(
            side_effect=[
                TransactionError("conflict"),
                TransactionError("conflict"),
                "ok",
            ]
        )

        with patch("time.sleep"):  # don't actually sleep in tests
            result = db.transaction_with_retry(lambda txn: None, max_retries=3)

        self.assertEqual(result, "ok")
        self.assertEqual(db.transaction.call_count, 3)

    def test_raises_after_max_retries_exhausted(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(side_effect=TransactionError("conflict"))

        with patch("time.sleep"):
            with self.assertRaises(TransactionError):
                db.transaction_with_retry(lambda txn: None, max_retries=3)

        self.assertEqual(db.transaction.call_count, 3)

    def test_does_not_retry_non_transaction_errors(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(side_effect=QueryError("bad sql"))

        with self.assertRaises(QueryError):
            db.transaction_with_retry(lambda txn: None, max_retries=3)

        # Should have stopped after the first attempt
        self.assertEqual(db.transaction.call_count, 1)

    def test_default_max_retries_is_three(self):
        from overdrive import OverDrive
        import inspect
        sig = inspect.signature(OverDrive.transaction_with_retry)
        self.assertEqual(sig.parameters["max_retries"].default, 3)

    def test_default_isolation_is_read_committed(self):
        from overdrive import OverDrive
        import inspect
        sig = inspect.signature(OverDrive.transaction_with_retry)
        self.assertEqual(
            sig.parameters["isolation"].default,
            OverDrive.READ_COMMITTED,
        )

    def test_exponential_backoff_delays(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(
            side_effect=[
                TransactionError("c1"),
                TransactionError("c2"),
                "done",
            ]
        )

        sleep_calls = []
        with patch("time.sleep", side_effect=lambda d: sleep_calls.append(d)):
            db.transaction_with_retry(lambda txn: None, max_retries=3)

        # Two sleeps: after attempt 0 and attempt 1
        self.assertEqual(len(sleep_calls), 2)
        # First delay: 0.1 * 2^0 = 0.1
        self.assertAlmostEqual(sleep_calls[0], 0.1, places=5)
        # Second delay: 0.1 * 2^1 = 0.2
        self.assertAlmostEqual(sleep_calls[1], 0.2, places=5)

    def test_backoff_capped_at_two_seconds(self):
        from overdrive import OverDrive
        db = self._make_db()
        # Fail many times
        db.transaction = MagicMock(
            side_effect=[TransactionError("c")] * 9 + ["ok"]
        )

        sleep_calls = []
        with patch("time.sleep", side_effect=lambda d: sleep_calls.append(d)):
            db.transaction_with_retry(lambda txn: None, max_retries=10)

        # All delays must be ≤ 2.0 seconds
        for delay in sleep_calls:
            self.assertLessEqual(delay, 2.0)

    def test_no_sleep_after_last_attempt(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(side_effect=TransactionError("conflict"))

        sleep_calls = []
        with patch("time.sleep", side_effect=lambda d: sleep_calls.append(d)):
            with self.assertRaises(TransactionError):
                db.transaction_with_retry(lambda txn: None, max_retries=3)

        # 3 attempts → 2 sleeps (no sleep after the final failure)
        self.assertEqual(len(sleep_calls), 2)

    def test_passes_isolation_level_to_transaction(self):
        from overdrive import OverDrive
        db = self._make_db()
        db.transaction = MagicMock(return_value=None)

        db.transaction_with_retry(
            lambda txn: None,
            isolation=OverDrive.SERIALIZABLE,
        )

        db.transaction.assert_called_once()
        _, kwargs = db.transaction.call_args
        # isolation may be passed as positional or keyword
        call_args = db.transaction.call_args
        # Check that SERIALIZABLE was forwarded
        all_args = list(call_args.args) + list(call_args.kwargs.values())
        self.assertIn(OverDrive.SERIALIZABLE, all_args)


if __name__ == "__main__":
    unittest.main()
