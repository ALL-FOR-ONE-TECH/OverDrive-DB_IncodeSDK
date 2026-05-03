"""
OverDrive-DB Python SDK — Unit Tests (v1.4)
Task 43: Tests for all v1.4 features

Run: pytest tests/test_v14.py -v
"""
import os
import shutil
import tempfile
import pytest

# These tests require the native library to be available
# Mark as integration tests if native lib is not present
try:
    from overdrive import OverDrive, OverDriveError, AuthenticationError, TableError
    NATIVE_AVAILABLE = True
except (ImportError, OSError):
    NATIVE_AVAILABLE = False

pytestmark = pytest.mark.skipif(not NATIVE_AVAILABLE, reason="Native library not available")

TEST_DIR = None

@pytest.fixture(autouse=True)
def setup_teardown():
    global TEST_DIR
    TEST_DIR = tempfile.mkdtemp(prefix="odb_test_")
    yield
    shutil.rmtree(TEST_DIR, ignore_errors=True)

def test_path(name):
    return os.path.join(TEST_DIR, name)

# ── Task 43.1: open() with password and engine ─────────

class TestOpen:
    def test_open_default(self):
        db = OverDrive.open(test_path("default.odb"))
        assert db is not None
        db.close()

    def test_open_with_ram_engine(self):
        db = OverDrive.open(test_path("ram.odb"), engine="RAM")
        assert db is not None
        db.close()

    def test_open_rejects_invalid_engine(self):
        with pytest.raises(OverDriveError):
            OverDrive.open(test_path("bad.odb"), engine="InvalidEngine")

    def test_open_rejects_short_password(self):
        with pytest.raises(OverDriveError, match="at least 8 characters"):
            OverDrive.open(test_path("short.odb"), password="abc")

    def test_open_with_password(self):
        db = OverDrive.open(test_path("enc.odb"), password="my-secret-password-123")
        assert db is not None
        db.close()

    def test_auto_create_tables(self):
        db = OverDrive.open(test_path("auto.odb"))
        db.insert("auto_table", {"key": "value"})
        assert db.tableExists("auto_table")
        db.close()

# ── Task 43.2: RAM engine methods ──────────────────────

class TestRAMEngine:
    def test_snapshot_and_restore(self):
        db = OverDrive.open(test_path("ram_snap.odb"), engine="RAM")
        db.insert("cache", {"key": "session", "value": "abc123"})
        snap_path = test_path("snap.odb")
        db.snapshot(snap_path)
        assert os.path.exists(snap_path)
        db.restore(snap_path)
        db.close()

    def test_memory_usage(self):
        db = OverDrive.open(test_path("ram_mem.odb"), engine="RAM")
        db.insert("data", {"key": "value"})
        usage = db.memoryUsage()
        assert "bytes" in usage
        assert "mb" in usage
        assert "limit_bytes" in usage
        assert "percent" in usage
        db.close()

# ── Task 43.3: watchdog function ───────────────────────

class TestWatchdog:
    def test_valid_database(self):
        dbpath = test_path("watchdog.odb")
        db = OverDrive.open(dbpath)
        db.insert("data", {"key": "value"})
        db.close()

        report = OverDrive.watchdog(dbpath)
        assert report.file_path == dbpath
        assert report.file_size_bytes > 0
        assert report.integrity_status == "valid"
        assert report.magic_valid is True

    def test_missing_database(self):
        report = OverDrive.watchdog(test_path("nonexistent.odb"))
        assert report.integrity_status == "missing"

# ── Task 43.4: transaction callback ────────────────────

class TestTransaction:
    def test_auto_commit_on_success(self):
        db = OverDrive.open(test_path("txn_ok.odb"))
        db.createTable("users")
        result = db.transaction(lambda txn: db.insert("users", {"name": "Alice"}))
        assert result is not None
        assert db.count("users") == 1
        db.close()

    def test_auto_rollback_on_exception(self):
        db = OverDrive.open(test_path("txn_fail.odb"))
        db.createTable("users")
        with pytest.raises(ValueError):
            def fail(txn):
                db.insert("users", {"name": "Alice"})
                raise ValueError("test rollback")
            db.transaction(fail)
        db.close()

    def test_returns_callback_value(self):
        db = OverDrive.open(test_path("txn_val.odb"))
        db.createTable("data")
        result = db.transaction(lambda txn: "hello")
        assert result == "hello"
        db.close()

# ── Task 43.5: helper methods ──────────────────────────

class TestHelpers:
    @pytest.fixture(autouse=True)
    def setup_db(self):
        self.db = OverDrive.open(test_path("helpers.odb"))
        self.db.createTable("items")
        self.db.insert("items", {"name": "Apple", "price": 1.50, "category": "fruit"})
        self.db.insert("items", {"name": "Banana", "price": 0.75, "category": "fruit"})
        self.db.insert("items", {"name": "Carrot", "price": 0.50, "category": "vegetable"})
        yield
        self.db.close()

    def test_find_one(self):
        item = self.db.findOne("items", "category = 'fruit'")
        assert item is not None
        assert item["category"] == "fruit"

    def test_find_one_returns_none(self):
        item = self.db.findOne("items", "category = 'meat'")
        assert item is None

    def test_find_all(self):
        items = self.db.findAll("items", "category = 'fruit'")
        assert len(items) == 2

    def test_update_many(self):
        count = self.db.updateMany("items", "category = 'fruit'", {"organic": True})
        assert count == 2

    def test_count_where(self):
        count = self.db.countWhere("items", "category = 'fruit'")
        assert count == 2

    def test_exists_true(self):
        items = self.db.findAll("items")
        assert self.db.exists("items", items[0]["_id"])

    def test_exists_false(self):
        assert not self.db.exists("items", "nonexistent_999")

# ── Task 43.6: error handling ──────────────────────────

class TestErrorHandling:
    def test_error_hierarchy(self):
        assert issubclass(AuthenticationError, OverDriveError)
        assert issubclass(TableError, OverDriveError)

    def test_error_has_code(self):
        try:
            OverDrive.open(test_path("bad.odb"), password="short")
        except OverDriveError as e:
            assert hasattr(e, "code") or "8 characters" in str(e)

# ── Task 43.7: backward compatibility ──────────────────

class TestBackwardCompat:
    def test_constructor_still_works(self):
        db = OverDrive(test_path("compat.odb"))
        assert db is not None
        db.close()

    def test_explicit_create_table(self):
        db = OverDrive.open(test_path("compat_tbl.odb"))
        db.createTable("explicit")
        assert db.tableExists("explicit")
        db.close()

    def test_manual_transactions(self):
        db = OverDrive.open(test_path("compat_txn.odb"))
        db.createTable("txn_test")
        txn_id = db.begin_transaction()
        db.insert("txn_test", {"key": "value"})
        db.commit_transaction(txn_id)
        assert db.count("txn_test") == 1
        db.close()
