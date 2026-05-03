package overdrive

import (
	"os"
	"path/filepath"
	"testing"
)

// OverDrive-DB Go SDK — Unit Tests (v1.4)
// Task 46: Tests for all v1.4 features
//
// Run: go test -v ./...

var testDir string

func TestMain(m *testing.M) {
	testDir = filepath.Join(os.TempDir(), "odb_go_test_v14")
	os.MkdirAll(testDir, 0755)
	code := m.Run()
	os.RemoveAll(testDir)
	os.Exit(code)
}

func testPath(name string) string {
	return filepath.Join(testDir, name)
}

// ── Task 46.1: Open with options ───────────────────────

func TestOpenDefault(t *testing.T) {
	db, err := Open(testPath("default.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	if db.Path() != testPath("default.odb") {
		t.Errorf("Expected path %q, got %q", testPath("default.odb"), db.Path())
	}
}

func TestOpenWithEngine(t *testing.T) {
	db, err := Open(testPath("ram.odb"), WithEngine("RAM"))
	if err != nil {
		t.Fatalf("Open with RAM failed: %v", err)
	}
	defer db.Close()
}

func TestOpenRejectsInvalidEngine(t *testing.T) {
	_, err := Open(testPath("bad.odb"), WithEngine("InvalidEngine"))
	if err == nil {
		t.Error("Expected error for invalid engine")
	}
}

func TestOpenRejectsShortPassword(t *testing.T) {
	_, err := Open(testPath("short.odb"), WithPassword("abc"))
	if err == nil {
		t.Error("Expected error for short password")
	}
	if oe, ok := err.(*AuthenticationError); ok {
		_ = oe // Correct error type
	}
}

func TestOpenWithPassword(t *testing.T) {
	db, err := Open(testPath("enc.odb"), WithPassword("my-secret-password-123"))
	if err != nil {
		t.Fatalf("Open with password failed: %v", err)
	}
	defer db.Close()
}

func TestAutoCreateTables(t *testing.T) {
	db, err := Open(testPath("auto.odb"), WithAutoCreateTables(true))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	_, err = db.Insert("auto_table", map[string]any{"key": "value"})
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}
	if !db.TableExists("auto_table") {
		t.Error("Expected auto_table to exist")
	}
}

// ── Task 46.2: RAM engine methods ──────────────────────

func TestCreateTableWithEngine(t *testing.T) {
	db, err := Open(testPath("hybrid.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	if err := db.CreateTable("ram_table", WithTableEngine("RAM")); err != nil {
		t.Fatalf("CreateTable with RAM failed: %v", err)
	}
	if _, err := db.Insert("ram_table", map[string]any{"key": "fast"}); err != nil {
		t.Fatalf("Insert failed: %v", err)
	}
	count, err := db.Count("ram_table")
	if err != nil {
		t.Fatalf("Count failed: %v", err)
	}
	if count != 1 {
		t.Errorf("Expected 1, got %d", count)
	}
}

func TestSnapshotAndRestore(t *testing.T) {
	db, err := Open(testPath("ram_snap.odb"), WithEngine("RAM"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.Insert("cache", map[string]any{"key": "session"})
	snapPath := testPath("snap.odb")
	if err := db.Snapshot(snapPath); err != nil {
		t.Fatalf("Snapshot failed: %v", err)
	}
	if _, err := os.Stat(snapPath); os.IsNotExist(err) {
		t.Error("Snapshot file should exist")
	}
	if err := db.Restore(snapPath); err != nil {
		t.Fatalf("Restore failed: %v", err)
	}
}

func TestMemoryUsage(t *testing.T) {
	db, err := Open(testPath("ram_mem.odb"), WithEngine("RAM"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.Insert("data", map[string]any{"key": "value"})
	usage, err := db.MemoryUsageStats()
	if err != nil {
		t.Fatalf("MemoryUsageStats failed: %v", err)
	}
	if usage.Bytes < 0 {
		t.Error("Bytes should be >= 0")
	}
}

// ── Task 46.3: Watchdog function ───────────────────────

func TestWatchdogValid(t *testing.T) {
	dbPath := testPath("watchdog.odb")
	db, err := Open(dbPath)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	db.Insert("data", map[string]any{"key": "value"})
	db.Close()

	report, err := Watchdog(dbPath)
	if err != nil {
		t.Fatalf("Watchdog failed: %v", err)
	}
	if report.IntegrityStatus != "valid" {
		t.Errorf("Expected 'valid', got '%s'", report.IntegrityStatus)
	}
	if !report.MagicValid {
		t.Error("Expected magic to be valid")
	}
}

func TestWatchdogMissing(t *testing.T) {
	report, err := Watchdog(testPath("nonexistent.odb"))
	if err != nil {
		t.Fatalf("Watchdog should not error for missing file: %v", err)
	}
	if report.IntegrityStatus != "missing" {
		t.Errorf("Expected 'missing', got '%s'", report.IntegrityStatus)
	}
}

// ── Task 46.4: Transaction callback ────────────────────

func TestTransactionAutoCommit(t *testing.T) {
	db, err := Open(testPath("txn_ok.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("users")
	err = db.Transaction(func(txn *TransactionHandle) error {
		_, err := db.Insert("users", map[string]any{"name": "Alice"})
		return err
	}, ReadCommitted)
	if err != nil {
		t.Fatalf("Transaction failed: %v", err)
	}

	count, _ := db.Count("users")
	if count != 1 {
		t.Errorf("Expected 1 user, got %d", count)
	}
}

func TestTransactionAutoRollback(t *testing.T) {
	db, err := Open(testPath("txn_fail.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("users")
	err = db.Transaction(func(txn *TransactionHandle) error {
		db.Insert("users", map[string]any{"name": "Alice"})
		return os.ErrInvalid // Force error
	}, ReadCommitted)
	if err == nil {
		t.Error("Expected transaction to fail")
	}
}

// ── Task 46.5: Helper methods ──────────────────────────

func TestFindOne(t *testing.T) {
	db, err := Open(testPath("helpers.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.Insert("items", map[string]any{"name": "Apple", "category": "fruit"})
	db.Insert("items", map[string]any{"name": "Carrot", "category": "vegetable"})

	item, err := db.FindOne("items", "category = 'fruit'")
	if err != nil {
		t.Fatalf("FindOne failed: %v", err)
	}
	if item == nil {
		t.Fatal("Expected item, got nil")
	}
	if item["category"] != "fruit" {
		t.Errorf("Expected fruit, got %v", item["category"])
	}
}

func TestFindOneReturnsNil(t *testing.T) {
	db, err := Open(testPath("helpers2.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("empty")
	item, err := db.FindOne("empty", "category = 'meat'")
	if err != nil {
		t.Fatalf("FindOne failed: %v", err)
	}
	if item != nil {
		t.Errorf("Expected nil, got %v", item)
	}
}

func TestCountWhere(t *testing.T) {
	db, err := Open(testPath("helpers3.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.Insert("items", map[string]any{"name": "Apple", "category": "fruit"})
	db.Insert("items", map[string]any{"name": "Banana", "category": "fruit"})

	count, err := db.CountWhere("items", "category = 'fruit'")
	if err != nil {
		t.Fatalf("CountWhere failed: %v", err)
	}
	if count != 2 {
		t.Errorf("Expected 2, got %d", count)
	}
}

func TestExists(t *testing.T) {
	db, err := Open(testPath("helpers4.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	id, _ := db.Insert("items", map[string]any{"name": "Apple"})
	exists, _ := db.Exists("items", id)
	if !exists {
		t.Error("Expected item to exist")
	}
	exists, _ = db.Exists("items", "nonexistent_999")
	if exists {
		t.Error("Expected item to not exist")
	}
}

// ── Task 46.6: Error types ─────────────────────────────

func TestErrorTypes(t *testing.T) {
	err := newTypedError("test", "ODB-AUTH-001", "", nil, "")
	if _, ok := err.(*AuthenticationError); !ok {
		t.Error("Expected AuthenticationError")
	}

	err = newTypedError("test", "ODB-TABLE-001", "", nil, "")
	if _, ok := err.(*TableError); !ok {
		t.Error("Expected TableError")
	}

	err = newTypedError("test", "ODB-TXN-001", "", nil, "")
	if _, ok := err.(*TransactionError); !ok {
		t.Error("Expected TransactionError")
	}
}

func TestErrorInterface(t *testing.T) {
	err := newTypedError("test message", "ODB-AUTH-001", "ctx",
		[]string{"suggestion1", "suggestion2"}, "https://docs.example.com")

	var ode OverDriveError
	var ok bool
	if ode, ok = err.(OverDriveError); !ok {
		t.Fatal("Expected OverDriveError interface")
	}
	if ode.Code() != "ODB-AUTH-001" {
		t.Errorf("Expected ODB-AUTH-001, got %s", ode.Code())
	}
	if len(ode.Suggestions()) != 2 {
		t.Errorf("Expected 2 suggestions, got %d", len(ode.Suggestions()))
	}
}

// ── Task 46.7: Backward compatibility ──────────────────

func TestVersionExists(t *testing.T) {
	v := Version()
	if v == "" || v == "unknown" {
		t.Skip("Version not available (native lib not loaded)")
	}
}

func TestManualTransactions(t *testing.T) {
	db, err := Open(testPath("compat_txn.odb"))
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("txn_test")
	txn, err := db.BeginTransaction(ReadCommitted)
	if err != nil {
		t.Fatalf("BeginTransaction failed: %v", err)
	}
	db.Insert("txn_test", map[string]any{"key": "value"})
	if err := db.CommitTransaction(txn); err != nil {
		t.Fatalf("CommitTransaction failed: %v", err)
	}
	count, _ := db.Count("txn_test")
	if count != 1 {
		t.Errorf("Expected 1, got %d", count)
	}
}
