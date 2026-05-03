"""
Unit tests for Task 10 — transaction callback pattern in the Python SDK.

Tests validate:
  10.1  transaction(callback, isolation=READ_COMMITTED) method
  10.2  Automatic commit on success
  10.3  Automatic rollback on exception, then re-raise
  10.4  Return callback's return value to caller
  10.5  camelCase aliases: beginTransaction, commitTransaction, abortTransaction
"""

import unittest
from unittest.mock import MagicMock, call, patch

import overdrive as sdk_module
from overdrive import OverDrive, OverDriveError


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_db(txn_id: int = 42):
    """
    Return an OverDrive instance with the native FFI layer fully mocked.
    begin_transaction returns *txn_id*; commit/abort return 0 (success).
    """
    native_mock = MagicMock()
    native_mock.lib.overdrive_open.return_value = 0xBEEF
    native_mock.lib.overdrive_begin_transaction.return_value = txn_id
    native_mock.lib.overdrive_commit_transaction.return_value = 0
    native_mock.lib.overdrive_abort_transaction.return_value = 0
    native_mock.lib.overdrive_last_error.return_value = None

    with patch.object(sdk_module, "_get_native", return_value=native_mock):
        db = object.__new__(OverDrive)
        db._native = native_mock
        db._handle = 0xBEEF
        db._path = ":memory:"

    return db, native_mock


# ===========================================================================
# 10.1  transaction() accepts a callable
# ===========================================================================

class TestTransactionCallableDetection(unittest.TestCase):

    def test_lambda_is_detected_as_callback(self):
        """Passing a lambda triggers callback mode (not context manager)."""
        db, native = _make_db()
        result = db.transaction(lambda txn: "ok")
        self.assertEqual(result, "ok")

    def test_regular_function_is_detected_as_callback(self):
        """Passing a regular function triggers callback mode."""
        db, native = _make_db()

        def my_work(txn):
            return 99

        result = db.transaction(my_work)
        self.assertEqual(result, 99)

    def test_no_arg_returns_context_manager(self):
        """Calling transaction() with no args returns a context manager."""
        db, _ = _make_db()
        ctx = db.transaction()
        self.assertTrue(hasattr(ctx, "__enter__") and hasattr(ctx, "__exit__"))

    def test_isolation_only_returns_context_manager(self):
        """Calling transaction(isolation=...) with an int returns a context manager."""
        db, _ = _make_db()
        ctx = db.transaction(OverDrive.SERIALIZABLE)
        self.assertTrue(hasattr(ctx, "__enter__") and hasattr(ctx, "__exit__"))


# ===========================================================================
# 10.2  Automatic commit on success
# ===========================================================================

class TestTransactionAutoCommit(unittest.TestCase):

    def test_commit_called_after_successful_callback(self):
        """commit_transaction is called when the callback returns normally."""
        db, native = _make_db(txn_id=7)
        db.transaction(lambda txn: None)
        native.lib.overdrive_commit_transaction.assert_called_once_with(0xBEEF, 7)

    def test_abort_not_called_on_success(self):
        """abort_transaction is NOT called when the callback succeeds."""
        db, native = _make_db()
        db.transaction(lambda txn: "done")
        native.lib.overdrive_abort_transaction.assert_not_called()

    def test_begin_called_with_default_isolation(self):
        """begin_transaction defaults to READ_COMMITTED (isolation=1)."""
        db, native = _make_db()
        db.transaction(lambda txn: None)
        native.lib.overdrive_begin_transaction.assert_called_once_with(
            0xBEEF, OverDrive.READ_COMMITTED
        )

    def test_begin_called_with_custom_isolation(self):
        """begin_transaction uses the isolation level passed as second arg."""
        db, native = _make_db()
        db.transaction(lambda txn: None, isolation=OverDrive.SERIALIZABLE)
        native.lib.overdrive_begin_transaction.assert_called_once_with(
            0xBEEF, OverDrive.SERIALIZABLE
        )


# ===========================================================================
# 10.3  Automatic rollback on exception, then re-raise
# ===========================================================================

class TestTransactionAutoRollback(unittest.TestCase):

    def test_abort_called_when_callback_raises(self):
        """abort_transaction is called when the callback raises an exception."""
        db, native = _make_db(txn_id=13)

        def bad_callback(txn):
            raise ValueError("something went wrong")

        with self.assertRaises(ValueError):
            db.transaction(bad_callback)

        native.lib.overdrive_abort_transaction.assert_called_once_with(0xBEEF, 13)

    def test_commit_not_called_when_callback_raises(self):
        """commit_transaction is NOT called when the callback raises."""
        db, native = _make_db()

        with self.assertRaises(RuntimeError):
            db.transaction(lambda txn: (_ for _ in ()).throw(RuntimeError("boom")))

        native.lib.overdrive_commit_transaction.assert_not_called()

    def test_original_exception_is_reraised(self):
        """The original exception propagates out of transaction()."""
        db, _ = _make_db()

        class _CustomError(Exception):
            pass

        with self.assertRaises(_CustomError) as ctx:
            db.transaction(lambda txn: (_ for _ in ()).throw(_CustomError("original")))

        self.assertEqual(str(ctx.exception), "original")

    def test_abort_called_for_overdrive_error(self):
        """abort_transaction is called even when an OverDriveError is raised."""
        db, native = _make_db(txn_id=5)

        def raises_sdk_error(txn):
            raise OverDriveError("native failure")

        with self.assertRaises(OverDriveError):
            db.transaction(raises_sdk_error)

        native.lib.overdrive_abort_transaction.assert_called_once_with(0xBEEF, 5)


# ===========================================================================
# 10.4  Return callback's return value to caller
# ===========================================================================

class TestTransactionReturnValue(unittest.TestCase):

    def test_returns_none_when_callback_returns_none(self):
        db, _ = _make_db()
        result = db.transaction(lambda txn: None)
        self.assertIsNone(result)

    def test_returns_integer(self):
        db, _ = _make_db()
        result = db.transaction(lambda txn: 42)
        self.assertEqual(result, 42)

    def test_returns_string(self):
        db, _ = _make_db()
        result = db.transaction(lambda txn: "hello")
        self.assertEqual(result, "hello")

    def test_returns_dict(self):
        db, _ = _make_db()
        payload = {"_id": "users_1", "name": "Alice"}
        result = db.transaction(lambda txn: payload)
        self.assertEqual(result, payload)

    def test_returns_list(self):
        db, _ = _make_db()
        result = db.transaction(lambda txn: [1, 2, 3])
        self.assertEqual(result, [1, 2, 3])

    def test_txn_id_passed_to_callback(self):
        """The transaction ID returned by begin_transaction is passed to the callback."""
        db, native = _make_db(txn_id=99)
        received = []
        db.transaction(lambda txn: received.append(txn))
        self.assertEqual(received, [99])


# ===========================================================================
# 10.5  camelCase aliases
# ===========================================================================

class TestCamelCaseAliases(unittest.TestCase):

    def test_begin_transaction_alias(self):
        """beginTransaction is an alias for begin_transaction."""
        self.assertIs(OverDrive.beginTransaction, OverDrive.begin_transaction)

    def test_commit_transaction_alias(self):
        """commitTransaction is an alias for commit_transaction."""
        self.assertIs(OverDrive.commitTransaction, OverDrive.commit_transaction)

    def test_abort_transaction_alias(self):
        """abortTransaction is an alias for abort_transaction."""
        self.assertIs(OverDrive.abortTransaction, OverDrive.abort_transaction)

    def test_begin_transaction_callable(self):
        """beginTransaction can be called and delegates to begin_transaction."""
        db, native = _make_db(txn_id=55)
        txn_id = db.beginTransaction(OverDrive.READ_COMMITTED)
        self.assertEqual(txn_id, 55)
        native.lib.overdrive_begin_transaction.assert_called_once_with(
            0xBEEF, OverDrive.READ_COMMITTED
        )

    def test_commit_transaction_callable(self):
        """commitTransaction can be called and delegates to commit_transaction."""
        db, native = _make_db()
        db.commitTransaction(42)
        native.lib.overdrive_commit_transaction.assert_called_once_with(0xBEEF, 42)

    def test_abort_transaction_callable(self):
        """abortTransaction can be called and delegates to abort_transaction."""
        db, native = _make_db()
        db.abortTransaction(42)
        native.lib.overdrive_abort_transaction.assert_called_once_with(0xBEEF, 42)


# ===========================================================================
# Backward compatibility — context manager still works
# ===========================================================================

class TestContextManagerBackwardCompat(unittest.TestCase):

    def test_context_manager_commits_on_success(self):
        """with db.transaction() as txn: still auto-commits."""
        db, native = _make_db(txn_id=3)
        with db.transaction() as txn_id:
            self.assertEqual(txn_id, 3)
        native.lib.overdrive_commit_transaction.assert_called_once_with(0xBEEF, 3)

    def test_context_manager_aborts_on_exception(self):
        """with db.transaction() as txn: still auto-aborts on exception."""
        db, native = _make_db(txn_id=4)
        with self.assertRaises(ValueError):
            with db.transaction() as txn_id:
                raise ValueError("fail")
        native.lib.overdrive_abort_transaction.assert_called_once_with(0xBEEF, 4)

    def test_context_manager_with_isolation(self):
        """with db.transaction(SERIALIZABLE) as txn: passes isolation level."""
        db, native = _make_db()
        with db.transaction(OverDrive.SERIALIZABLE):
            pass
        native.lib.overdrive_begin_transaction.assert_called_once_with(
            0xBEEF, OverDrive.SERIALIZABLE
        )


if __name__ == "__main__":
    unittest.main()
