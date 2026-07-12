package overdrive_test

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	overdrive "github.com/ALL-FOR-ONE-TECH/overdrive-db-go"
)

func tmpDB(t *testing.T, name string) string {
	t.Helper()
	dir := filepath.Join(os.TempDir(), "overdrive_sdk_e2e_go")
	os.MkdirAll(dir, 0755)
	p := filepath.Join(dir, name+".odb")
	os.Remove(p)
	os.Remove(p + ".wal")
	t.Cleanup(func() { os.Remove(p); os.Remove(p + ".wal") })
	return p
}

// TEST 1
func TestOpen_CreatesFile(t *testing.T) {
	p := tmpDB(t, "t01_open")
	odb, err := overdrive.Open(p)
	if err != nil { t.Fatalf("open: %v", err) }
	odb.Close()
	info, err := os.Stat(p)
	if err != nil { t.Fatal("❌ .odb file not created") }
	if info.Size() == 0 { t.Fatal("❌ .odb is 0 bytes") }
	fmt.Printf("✅ TEST 1 — .odb created (%d bytes)\n", info.Size())
}

// TEST 2
func TestInsertGet_Roundtrip(t *testing.T) {
	p := tmpDB(t, "t02_crud")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	odb.CreateTable("users")
	id, err := odb.Insert("users", map[string]any{"name": "Karthikeyan", "role": "engineer", "age": 28})
	if err != nil || id == "" { t.Fatalf("insert: %v", err) }
	doc, err := odb.Get("users", id)
	if err != nil || doc == nil { t.Fatalf("get: %v", err) }
	if doc["name"] != "Karthikeyan" { t.Fatalf("❌ name mismatch: %v", doc["name"]) }
	fmt.Printf("✅ TEST 2 — odb.Insert+Get: _id=%s, name=%v\n", id, doc["name"])
}

// TEST 3
func TestCount_Accurate(t *testing.T) {
	p := tmpDB(t, "t03_count")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	odb.CreateTable("items")
	n, _ := odb.Count("items")
	if n != 0 { t.Fatalf("❌ expected 0, got %d", n) }
	odb.Insert("items", map[string]any{"n": "A"})
	odb.Insert("items", map[string]any{"n": "B"})
	odb.Insert("items", map[string]any{"n": "C"})
	n, _ = odb.Count("items")
	if n != 3 { t.Fatalf("❌ expected 3, got %d", n) }
	fmt.Printf("✅ TEST 3 — odb.Count=3\n")
}

// TEST 4
func TestMultiGet_Fields(t *testing.T) {
	p := tmpDB(t, "t04_multi")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	odb.CreateTable("products")
	id1, _ := odb.Insert("products", map[string]any{"name": "Apple",  "price": 10})
	id2, _ := odb.Insert("products", map[string]any{"name": "Banana", "price": 5})
	id3, _ := odb.Insert("products", map[string]any{"name": "Cherry", "price": 25})
	d1, _ := odb.Get("products", id1)
	d2, _ := odb.Get("products", id2)
	d3, _ := odb.Get("products", id3)
	if d1["name"] != "Apple" || d2["name"] != "Banana" || d3["name"] != "Cherry" {
		t.Fatal("❌ field mismatch")
	}
	fmt.Printf("✅ TEST 4 — odb.Get per-doc fields correct\n")
}

// TEST 5
func TestUpdate_ChangesField(t *testing.T) {
	p := tmpDB(t, "t05_update")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	odb.CreateTable("cfg")
	id, _ := odb.Insert("cfg", map[string]any{"key": "theme", "val": "light"})
	ok, err := odb.Update("cfg", id, map[string]any{"val": "dark"})
	if err != nil || !ok { t.Fatalf("update failed: %v", err) }
	doc, _ := odb.Get("cfg", id)
	if doc["val"] != "dark" { t.Fatalf("❌ expected dark, got %v", doc["val"]) }
	fmt.Printf("✅ TEST 5 — odb.Update: light→dark\n")
}

// TEST 6
func TestDelete_RemovesDoc(t *testing.T) {
	p := tmpDB(t, "t06_delete")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	odb.CreateTable("logs")
	id1, _ := odb.Insert("logs", map[string]any{"msg": "e1"})
	id2, _ := odb.Insert("logs", map[string]any{"msg": "e2"})
	ok, _ := odb.Delete("logs", id1)
	if !ok { t.Fatal("❌ delete must return true") }
	n, _ := odb.Count("logs")
	if n != 1 { t.Fatalf("❌ count must be 1, got %d", n) }
	d, _ := odb.Get("logs", id1)
	if d != nil { t.Fatal("❌ deleted doc must be nil") }
	d2, _ := odb.Get("logs", id2)
	if d2 == nil { t.Fatal("❌ remaining doc must exist") }
	fmt.Printf("✅ TEST 6 — odb.Delete: count=1, deleted=nil\n")
}

// TEST 7
func TestPersist_AfterReopen(t *testing.T) {
	p := tmpDB(t, "t07_persist")
	var storedID string
	{
		odb, _ := overdrive.Open(p)
		odb.CreateTable("sessions")
		storedID, _ = odb.Insert("sessions", map[string]any{"token": "abc123", "user": "afot"})
		odb.Sync()
		odb.Close()
	}
	{
		odb, _ := overdrive.Open(p)
		defer odb.Close()
		n, _ := odb.Count("sessions")
		if n != 1 { t.Fatalf("❌ data must persist, count=%d", n) }
		doc, _ := odb.Get("sessions", storedID)
		if doc == nil { t.Fatal("❌ doc must exist after reopen") }
		if doc["token"] != "abc123" { t.Fatalf("❌ token mismatch: %v", doc["token"]) }
		fmt.Printf("✅ TEST 7 — persisted: token=%v, _id=%s\n", doc["token"], storedID)
	}
}

// TEST 8
func TestInsertBatch(t *testing.T) {
	p := tmpDB(t, "t08_batch")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	odb.CreateTable("orders")
	ids, err := odb.InsertBatch("orders", []map[string]any{
		{"order_id": "ORD-001", "amount": 150},
		{"order_id": "ORD-002", "amount": 200},
		{"order_id": "ORD-003", "amount": 75},
	})
	if err != nil || len(ids) != 3 { t.Fatalf("insertBatch: %v", err) }
	n, _ := odb.Count("orders")
	if n != 3 { t.Fatalf("❌ count must be 3, got %d", n) }
	for _, id := range ids {
		doc, _ := odb.Get("orders", id)
		if doc == nil { t.Fatalf("❌ inserted doc %s not found", id) }
	}
	fmt.Printf("✅ TEST 8 — odb.InsertBatch: %v\n", ids)
}

// TEST 9
func TestTableExists(t *testing.T) {
	p := tmpDB(t, "t09_tables")
	odb, _ := overdrive.Open(p)
	defer odb.Close()
	if odb.TableExists("ghost") { t.Fatal("❌ ghost should not exist") }
	odb.CreateTable("real")
	if !odb.TableExists("real") { t.Fatal("❌ real should exist") }
	fmt.Printf("✅ TEST 9 — odb.TableExists: ghost=false, real=true\n")
}

// TEST 10
func TestVersion(t *testing.T) {
	v := overdrive.Version()
	if v == "" { t.Fatal("❌ version empty") }
	if v == "unknown" { t.Fatal("❌ native lib not loaded") }
	if v != "2.3.0" { t.Fatalf("❌ expected 2.3.0, got %s", v) }
	fmt.Printf("✅ TEST 10 — odb version: %s\n", v)
}

// Suppress unused import warning
var _ = json.Marshal
