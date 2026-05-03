"""
Unit tests for Task 11 — helper methods in the Python SDK.

Tests validate:
  11.1  findOne(table, where="")  — returns first matching doc or None
  11.2  findAll(table, where="", order_by="", limit=0)  — returns list
  11.3  updateMany(table, where, updates)  — returns rows_affected count
  11.4  deleteMany(table, where)  — returns rows_affected count
  11.5  countWhere(table, where)  — returns int count
  11.6  exists(table, id)  — returns bool
"""

import unittest
from unittest.mock import MagicMock, patch, call

import overdrive as sdk_module
from overdrive import OverDrive, OverDriveError


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_db():
    """Return an OverDrive instance with the native FFI layer fully mocked."""
    native_mock = MagicMock()
    native_mock.lib.overdrive_open.return_value = 0xDEAD
    native_mock.lib.overdrive_last_error.return_value = None

    db = object.__new__(OverDrive)
    db._native = native_mock
    db._handle = 0xDEAD
    db._path = ":memory:"
    return db, native_mock


def _mock_query(db, native, rows):
    """
    Patch db.query() to return *rows* and record the SQL that was passed.
    Returns the list of SQL strings captured.
    """
    captured = []

    def fake_query(sql):
        captured.append(sql)
        return rows

    db.query = fake_query
    return captured


def _mock_query_full(db, rows_affected):
    """Patch db.query_full() to return a result with rows_affected."""
    captured = []

    def fake_query_full(sql):
        captured.append(sql)
        return {"rows": [], "columns": [], "rows_affected": rows_affected}

    db.query_full = fake_query_full
    return captured


# ===========================================================================
# 11.1  findOne
# ===========================================================================

class TestFindOne(unittest.TestCase):

    def test_returns_first_row_when_match(self):
        db, native = _make_db()
        doc = {"_id": "users_1", "name": "Alice"}
        _mock_query(db, native, [doc])
        result = db.findOne("users", "name = 'Alice'")
        self.assertEqual(result, doc)

    def test_returns_none_when_no_match(self):
        db, native = _make_db()
        _mock_query(db, native, [])
        result = db.findOne("users", "name = 'Nobody'")
        self.assertIsNone(result)

    def test_sql_includes_where_clause(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findOne("users", "age > 25")
        self.assertIn("WHERE age > 25", captured[0])

    def test_sql_includes_limit_1(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findOne("users", "age > 25")
        self.assertIn("LIMIT 1", captured[0])

    def test_no_where_omits_where_clause(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findOne("users")
        self.assertNotIn("WHERE", captured[0])

    def test_no_where_still_has_limit_1(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findOne("users")
        self.assertIn("LIMIT 1", captured[0])

    def test_table_name_in_sql(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findOne("orders")
        self.assertIn("orders", captured[0])

    def test_returns_only_first_of_multiple_rows(self):
        db, native = _make_db()
        docs = [{"_id": "u1"}, {"_id": "u2"}]
        _mock_query(db, native, docs)
        result = db.findOne("users")
        self.assertEqual(result["_id"], "u1")


# ===========================================================================
# 11.2  findAll
# ===========================================================================

class TestFindAll(unittest.TestCase):

    def test_returns_all_rows(self):
        db, native = _make_db()
        docs = [{"_id": "u1"}, {"_id": "u2"}, {"_id": "u3"}]
        _mock_query(db, native, docs)
        result = db.findAll("users")
        self.assertEqual(result, docs)

    def test_returns_empty_list_when_no_rows(self):
        db, native = _make_db()
        _mock_query(db, native, [])
        result = db.findAll("users")
        self.assertEqual(result, [])

    def test_where_clause_included(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users", where="age >= 18")
        self.assertIn("WHERE age >= 18", captured[0])

    def test_no_where_omits_where_keyword(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users")
        self.assertNotIn("WHERE", captured[0])

    def test_order_by_included(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users", order_by="name ASC")
        self.assertIn("ORDER BY name ASC", captured[0])

    def test_no_order_by_omits_order_by_keyword(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users")
        self.assertNotIn("ORDER BY", captured[0])

    def test_limit_included_when_nonzero(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users", limit=50)
        self.assertIn("LIMIT 50", captured[0])

    def test_zero_limit_omits_limit_clause(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users", limit=0)
        self.assertNotIn("LIMIT", captured[0])

    def test_all_clauses_combined(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("users", where="active = 1", order_by="name", limit=10)
        sql = captured[0]
        self.assertIn("WHERE active = 1", sql)
        self.assertIn("ORDER BY name", sql)
        self.assertIn("LIMIT 10", sql)

    def test_table_name_in_sql(self):
        db, native = _make_db()
        captured = _mock_query(db, native, [])
        db.findAll("products")
        self.assertIn("products", captured[0])


# ===========================================================================
# 11.3  updateMany
# ===========================================================================

class TestUpdateMany(unittest.TestCase):

    def test_returns_rows_affected_count(self):
        db, native = _make_db()
        _mock_query_full(db, rows_affected=3)
        count = db.updateMany("users", "active = 0", {"status": "inactive"})
        self.assertEqual(count, 3)

    def test_returns_zero_when_no_rows_affected(self):
        db, native = _make_db()
        _mock_query_full(db, rows_affected=0)
        count = db.updateMany("users", "age < 0", {"flag": True})
        self.assertEqual(count, 0)

    def test_sql_is_update_statement(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=1)
        db.updateMany("users", "name = 'Bob'", {"verified": True})
        self.assertTrue(captured[0].strip().upper().startswith("UPDATE"))

    def test_table_name_in_sql(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=0)
        db.updateMany("orders", "status = 'pending'", {"status": "shipped"})
        self.assertIn("orders", captured[0])

    def test_where_clause_in_sql(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=0)
        db.updateMany("users", "age > 30", {"senior": True})
        self.assertIn("WHERE age > 30", captured[0])

    def test_set_clause_in_sql(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=0)
        db.updateMany("users", "active = 0", {"status": "inactive"})
        self.assertIn("SET", captured[0])
        self.assertIn("status", captured[0])

    def test_multiple_update_fields(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=2)
        db.updateMany("users", "role = 'guest'", {"role": "member", "verified": True})
        sql = captured[0]
        self.assertIn("role", sql)
        self.assertIn("verified", sql)


# ===========================================================================
# 11.4  deleteMany
# ===========================================================================

class TestDeleteMany(unittest.TestCase):

    def test_returns_rows_affected_count(self):
        db, native = _make_db()
        _mock_query_full(db, rows_affected=5)
        count = db.deleteMany("logs", "created_at < 1700000000")
        self.assertEqual(count, 5)

    def test_returns_zero_when_no_rows_deleted(self):
        db, native = _make_db()
        _mock_query_full(db, rows_affected=0)
        count = db.deleteMany("logs", "created_at < 0")
        self.assertEqual(count, 0)

    def test_sql_is_delete_statement(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=0)
        db.deleteMany("logs", "level = 'debug'")
        self.assertTrue(captured[0].strip().upper().startswith("DELETE"))

    def test_table_name_in_sql(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=0)
        db.deleteMany("sessions", "expired = 1")
        self.assertIn("sessions", captured[0])

    def test_where_clause_in_sql(self):
        db, native = _make_db()
        captured = _mock_query_full(db, rows_affected=0)
        db.deleteMany("users", "banned = 1")
        self.assertIn("WHERE banned = 1", captured[0])


# ===========================================================================
# 11.5  countWhere
# ===========================================================================

class TestCountWhere(unittest.TestCase):

    def _make_count_row(self, n):
        return [{"COUNT(*)": n}]

    def test_returns_count_integer(self):
        db, native = _make_db()
        _mock_query(db, native, self._make_count_row(7))
        result = db.countWhere("users", "age > 25")
        self.assertEqual(result, 7)

    def test_returns_zero_when_no_rows(self):
        db, native = _make_db()
        _mock_query(db, native, [])
        result = db.countWhere("users", "age > 999")
        self.assertEqual(result, 0)

    def test_sql_includes_count_star(self):
        db, native = _make_db()
        captured = _mock_query(db, native, self._make_count_row(0))
        db.countWhere("users", "active = 1")
        self.assertIn("COUNT(*)", captured[0].upper())

    def test_where_clause_included(self):
        db, native = _make_db()
        captured = _mock_query(db, native, self._make_count_row(0))
        db.countWhere("users", "role = 'admin'")
        self.assertIn("WHERE role = 'admin'", captured[0])

    def test_no_where_omits_where_keyword(self):
        db, native = _make_db()
        captured = _mock_query(db, native, self._make_count_row(10))
        db.countWhere("users")
        self.assertNotIn("WHERE", captured[0])

    def test_table_name_in_sql(self):
        db, native = _make_db()
        captured = _mock_query(db, native, self._make_count_row(0))
        db.countWhere("products", "in_stock = 1")
        self.assertIn("products", captured[0])

    def test_handles_lowercase_count_key(self):
        """Engine may return 'count(*)' in lowercase."""
        db, native = _make_db()
        _mock_query(db, native, [{"count(*)": 4}])
        result = db.countWhere("users", "active = 1")
        self.assertEqual(result, 4)

    def test_handles_count_key_without_parens(self):
        """Engine may return just 'count' as the key."""
        db, native = _make_db()
        _mock_query(db, native, [{"count": 9}])
        result = db.countWhere("users")
        self.assertEqual(result, 9)


# ===========================================================================
# 11.6  exists
# ===========================================================================

class TestExists(unittest.TestCase):

    def test_returns_true_when_doc_found(self):
        db, native = _make_db()
        db.get = MagicMock(return_value={"_id": "users_1", "name": "Alice"})
        result = db.exists("users", "users_1")
        self.assertTrue(result)

    def test_returns_false_when_doc_not_found(self):
        db, native = _make_db()
        db.get = MagicMock(return_value=None)
        result = db.exists("users", "users_999")
        self.assertFalse(result)

    def test_calls_get_with_correct_args(self):
        db, native = _make_db()
        db.get = MagicMock(return_value=None)
        db.exists("orders", "orders_42")
        db.get.assert_called_once_with("orders", "orders_42")

    def test_returns_bool_type(self):
        db, native = _make_db()
        db.get = MagicMock(return_value={"_id": "x"})
        result = db.exists("t", "x")
        self.assertIsInstance(result, bool)

    def test_returns_false_bool_type(self):
        db, native = _make_db()
        db.get = MagicMock(return_value=None)
        result = db.exists("t", "missing")
        self.assertIsInstance(result, bool)
        self.assertFalse(result)


if __name__ == "__main__":
    unittest.main()
