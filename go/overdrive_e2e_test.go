//go:build e2e

// End-to-end tests for the OverDrive-DB Go SDK.
// These tests ACTUALLY create real .odb files using the native library.
// A passing test proves the native library loaded and the full stack works.
//
// Run: go test -v -tags e2e .
package overdrive

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

var e2eDir string

func init() {
	e2eDir = filepath.Join(os.TempDir(), "overdrive_e2e_go")
	os.MkdirAll(e2eDir, 0755)
}

func e2ePath(name string) string {
	return filepath.Join(e2eDir, name+".odb")
}

func e2eCleanup(p string) {
	os.Remove(p)
	os.Remove(p + ".wal")
}

// ─────────────────────────────────────────────────────────────
// TEST 1: Open creates a real .odb file on disk
// ─────────────────────────────────────────────────────────────
func TestE2E_OpenCreatesFile(t *testing.T) {
	p := e2ePath("open_creates")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v\n→ Native library may not have loaded correctly", err)
	}
	db.Close()

	info, err := os.Stat(p)
	if err != nil || os.IsNotExist(err) {
		t.Fatalf("❌ .odb file was NOT created at: %s", p)
	}
	if info.Size() == 0 {
		t.Fatal("❌ .odb file exists but is 0 bytes — engine did not initialize")
	}
	t.Logf("✅ .odb file created (%d bytes)", info.Size())
}

// ─────────────────────────────────────────────────────────────
// TEST 2: Insert → Get roundtrip
// ─────────────────────────────────────────────────────────────
func TestE2E_InsertAndGetRoundtrip(t *testing.T) {
	p := e2ePath("crud_roundtrip")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	if err := db.CreateTable("users"); err != nil {
		t.Fatalf("CreateTable failed: %v", err)
	}

	id, err := db.Insert("users", map[string]any{
		"name": "Karthikeyan",
		"role": "engineer",
		"age":  28,
	})
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}
	if id == "" {
		t.Fatal("❌ Insert must return a non-empty _id")
	}

	doc, err := db.Get("users", id)
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if doc == nil {
		t.Fatal("❌ document must exist after insert")
	}

	if doc["name"] != "Karthikeyan" {
		t.Errorf("❌ name mismatch: got %v", doc["name"])
	}
	if doc["role"] != "engineer" {
		t.Errorf("❌ role mismatch: got %v", doc["role"])
	}
	t.Logf("✅ Inserted _id=%s, name=%v", id, doc["name"])
}

// ─────────────────────────────────────────────────────────────
// TEST 3: Count is accurate
// ─────────────────────────────────────────────────────────────
func TestE2E_CountIsAccurate(t *testing.T) {
	p := e2ePath("count_check")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("items")

	before, _ := db.Count("items")
	if before != 0 {
		t.Errorf("❌ empty table count must be 0, got %d", before)
	}

	db.Insert("items", map[string]any{"name": "A"})
	db.Insert("items", map[string]any{"name": "B"})
	db.Insert("items", map[string]any{"name": "C"})

	after, err := db.Count("items")
	if err != nil {
		t.Fatalf("Count failed: %v", err)
	}
	if after != 3 {
		t.Errorf("❌ count must be 3 after 3 inserts, got %d", after)
	}
	t.Logf("✅ count=%d", after)
}

// ─────────────────────────────────────────────────────────────
// TEST 4: SQL query with WHERE returns filtered results
// ─────────────────────────────────────────────────────────────
func TestE2E_SQLQueryWithWhere(t *testing.T) {
	p := e2ePath("sql_query")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("products")
	db.Insert("products", map[string]any{"name": "Apple",  "price": 10})
	db.Insert("products", map[string]any{"name": "Banana", "price": 5})
	db.Insert("products", map[string]any{"name": "Cherry", "price": 25})

	result, err := db.Query("SELECT * FROM products WHERE price > 8")
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}
	if len(result.Rows) < 1 {
		t.Fatalf("❌ query with WHERE price>8 must return >=1 rows, got %d", len(result.Rows))
	}

	names := make([]string, 0)
	for _, row := range result.Rows {
		if n, ok := row["name"].(string); ok {
			names = append(names, n)
		}
	}

	hasApple := false
	hasCherry := false
	for _, n := range names {
		if n == "Apple"  { hasApple = true  }
		if n == "Cherry" { hasCherry = true }
	}
	if !hasApple  { t.Error("❌ Apple (price=10) must be in results") }
	if !hasCherry { t.Error("❌ Cherry (price=25) must be in results") }

	t.Logf("✅ %d rows returned: %v", len(result.Rows), names)
}

// ─────────────────────────────────────────────────────────────
// TEST 5: Update changes a field — verified by Get
// ─────────────────────────────────────────────────────────────
func TestE2E_UpdateChangesField(t *testing.T) {
	p := e2ePath("update_check")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("config")
	id, _ := db.Insert("config", map[string]any{"key": "theme", "value": "light"})

	updated, err := db.Update("config", id, map[string]any{"value": "dark"})
	if err != nil {
		t.Fatalf("Update failed: %v", err)
	}
	if !updated {
		t.Fatal("❌ Update must return true for existing doc")
	}

	doc, _ := db.Get("config", id)
	if doc["value"] != "dark" {
		t.Errorf("❌ value must be updated to 'dark', got %v", doc["value"])
	}
	t.Logf("✅ theme updated: %v → %v", "light", doc["value"])
}

// ─────────────────────────────────────────────────────────────
// TEST 6: Delete removes document — count drops by 1
// ─────────────────────────────────────────────────────────────
func TestE2E_DeleteRemovesDocument(t *testing.T) {
	p := e2ePath("delete_check")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("logs")
	id1, _ := db.Insert("logs", map[string]any{"msg": "event1"})
	id2, _ := db.Insert("logs", map[string]any{"msg": "event2"})

	deleted, err := db.Delete("logs", id1)
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}
	if !deleted {
		t.Fatal("❌ Delete must return true for existing doc")
	}

	count, _ := db.Count("logs")
	if count != 1 {
		t.Errorf("❌ count must be 1 after delete, got %d", count)
	}

	gone, _ := db.Get("logs", id1)
	if gone != nil {
		t.Error("❌ deleted document must return nil on Get")
	}

	stillThere, _ := db.Get("logs", id2)
	if stillThere == nil {
		t.Error("❌ other document must still exist after delete")
	}
	t.Logf("✅ id1 deleted, id2 still present")
}

// ─────────────────────────────────────────────────────────────
// TEST 7: Data persists after Close + Open
// ─────────────────────────────────────────────────────────────
func TestE2E_DataPersistsAfterReopen(t *testing.T) {
	p := e2ePath("persistence")
	e2eCleanup(p)
	defer e2eCleanup(p)

	// Write phase
	{
		db, err := Open(p)
		if err != nil {
			t.Fatalf("first Open() failed: %v", err)
		}
		db.CreateTable("sessions")
		db.Insert("sessions", map[string]any{
			"token": "abc123",
			"user":  "afot_admin",
		})
		db.Sync()
		db.Close()
	}

	// Read phase — fresh open
	{
		db, err := Open(p)
		if err != nil {
			t.Fatalf("second Open() failed: %v", err)
		}
		defer db.Close()

		count, err := db.Count("sessions")
		if err != nil {
			t.Fatalf("Count after reopen failed: %v", err)
		}
		if count != 1 {
			t.Errorf("❌ data must persist after close+reopen. count=%d (expected 1)", count)
		}

		result, err := db.Query("SELECT * FROM sessions")
		if err != nil {
			t.Fatalf("Query after reopen failed: %v", err)
		}
		if len(result.Rows) != 1 {
			t.Fatalf("❌ must have 1 row after reopen, got %d", len(result.Rows))
		}
		if result.Rows[0]["token"] != "abc123" {
			t.Errorf("❌ token must persist, got %v", result.Rows[0]["token"])
		}
		t.Logf("✅ persisted doc: token=%v", result.Rows[0]["token"])
	}
}

// ─────────────────────────────────────────────────────────────
// TEST 8: TableExists returns correct bool
// ─────────────────────────────────────────────────────────────
func TestE2E_TableExists(t *testing.T) {
	p := e2ePath("table_exists")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	if db.TableExists("ghost_table") {
		t.Error("❌ non-existent table must return false")
	}

	db.CreateTable("real_table")

	if !db.TableExists("real_table") {
		t.Error("❌ created table must return true")
	}
	t.Logf("✅ ghost=false, real_table=true")
}

// ─────────────────────────────────────────────────────────────
// TEST 9: ListTables returns created tables
// ─────────────────────────────────────────────────────────────
func TestE2E_ListTables(t *testing.T) {
	p := e2ePath("list_tables")
	e2eCleanup(p)
	defer e2eCleanup(p)

	db, err := Open(p)
	if err != nil {
		t.Fatalf("Open() failed: %v", err)
	}
	defer db.Close()

	db.CreateTable("alpha")
	db.CreateTable("beta")
	db.CreateTable("gamma")

	tables, err := db.ListTables()
	if err != nil {
		t.Fatalf("ListTables failed: %v", err)
	}

	tableSet := make(map[string]bool)
	for _, tbl := range tables {
		tableSet[tbl] = true
	}

	for _, expected := range []string{"alpha", "beta", "gamma"} {
		if !tableSet[expected] {
			t.Errorf("❌ table '%s' must appear in ListTables", expected)
		}
	}
	t.Logf("✅ ListTables: %v", tables)
}

// ─────────────────────────────────────────────────────────────
// TEST 10: Version is a valid string from native lib
// ─────────────────────────────────────────────────────────────
func TestE2E_VersionIsValid(t *testing.T) {
	v := Version()
	if v == "" {
		t.Fatal("❌ Version() must return a non-empty string")
	}
	if v == "unknown" {
		t.Fatal("❌ Version() returned 'unknown' — native lib may not have loaded")
	}
	t.Logf("✅ SDK version: %s", v)
}

// TestE2E_Summary prints a final summary line after all tests
func TestE2E_Summary(t *testing.T) {
	fmt.Println("\n✅ All e2e tests completed — .odb files created and verified")
	os.RemoveAll(e2eDir)
}
