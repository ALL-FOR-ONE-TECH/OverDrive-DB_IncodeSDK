// Package overdrive provides the OverDrive-DB embedded SDK for Go.
// Version: 2.2.0
//
// Usage:
//   odb, err := overdrive.Open("app.odb")
//   defer odb.Close()
//   id, err := odb.Insert("users", map[string]any{"name": "Alice"})
//   doc, err := odb.Get("users", id)

package overdrive

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
)

// IsolationLevel for MVCC transactions.
type IsolationLevel int32

const (
	ReadUncommitted IsolationLevel = 0
	ReadCommitted   IsolationLevel = 1
	RepeatableRead  IsolationLevel = 2
	Serializable    IsolationLevel = 3
)

// Option configures how a database is opened.
type Option func(*openConfig)

type openConfig struct {
	password string
	engine   string
}

// WithPassword opens an encrypted database.
func WithPassword(pwd string) Option { return func(c *openConfig) { c.password = pwd } }

// WithEngine selects a storage engine: "Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming".
func WithEngine(e string) Option { return func(c *openConfig) { c.engine = e } }

// Transaction represents an active MVCC transaction.
type Transaction struct{ id uint64 }

// OverdriveDb is the main database handle.
// Idiomatic usage: odb, err := overdrive.Open("app.odb")
type OverdriveDb struct {
	handle uintptr
}

// libPath resolves the native library path using platform-arch directories.
func libPath() string {
	// 1. Env override
	if p := os.Getenv("OVERDRIVE_LIB_PATH"); p != "" {
		return p
	}

	// 2. Bundled lib/{os}-{arch}/
	_, file, _, _ := runtime.Caller(0)
	dir := filepath.Dir(file)
	os_name := map[string]string{"windows": "windows", "linux": "linux", "darwin": "macos"}[runtime.GOOS]
	arch := map[string]string{"amd64": "x64", "arm64": "arm64"}[runtime.GOARCH]
	if os_name == "" { os_name = runtime.GOOS }
	if arch == "" { arch = runtime.GOARCH }
	name := libName()
	bundled := filepath.Join(dir, "..", "lib", os_name+"-"+arch, name)
	if _, err := os.Stat(bundled); err == nil {
		return bundled
	}

	// 3. Fallback: system
	return name
}

func libName() string {
	switch runtime.GOOS {
	case "windows": return "overdrive.dll"
	case "darwin":  return "liboverdrive.dylib"
	default:        return "liboverdrive.so"
	}
}

// Version returns the native library version string.
func Version() string { return nativeVersion() }

// Open opens or creates a database at path.
func Open(path string, opts ...Option) (*OverdriveDb, error) {
	cfg := &openConfig{engine: "Disk"}
	for _, o := range opts { o(cfg) }

	var handle uintptr
	var err error

	if cfg.password != "" || cfg.engine != "Disk" {
		optsJSON, _ := json.Marshal(map[string]any{
			"password": cfg.password,
			"auto_create_tables": true,
		})
		handle, err = nativeOpenWithEngine(path, cfg.engine, string(optsJSON))
	} else {
		handle, err = nativeOpen(path)
	}
	if err != nil { return nil, err }
	if handle == 0 { return nil, fmt.Errorf("overdrive: open failed: %s", nativeLastError()) }
	return &OverdriveDb{handle: handle}, nil
}

// Close releases the database handle.
func (odb *OverdriveDb) Close() error {
	nativeClose(odb.handle)
	odb.handle = 0
	return nil
}

// Sync flushes pending writes to disk.
func (odb *OverdriveDb) Sync() error { nativeSync(odb.handle); return nil }

// CreateTable creates a new table.
func (odb *OverdriveDb) CreateTable(name string) error {
	if nativeCreateTable(odb.handle, name) != 0 {
		return fmt.Errorf("overdrive: createTable: %s", nativeLastError())
	}
	return nil
}

// DropTable drops a table.
func (odb *OverdriveDb) DropTable(name string) error {
	if nativeDropTable(odb.handle, name) != 0 {
		return fmt.Errorf("overdrive: dropTable: %s", nativeLastError())
	}
	return nil
}

// ListTables returns all table names.
func (odb *OverdriveDb) ListTables() ([]string, error) {
	s, err := nativeListTables(odb.handle)
	if err != nil { return nil, err }
	var tables []string
	if err := json.Unmarshal([]byte(s), &tables); err != nil { return nil, err }
	return tables, nil
}

// TableExists returns true if the table exists.
func (odb *OverdriveDb) TableExists(name string) bool {
	return nativeTableExists(odb.handle, name) == 1
}

// Insert inserts a document into table. Returns the generated _id.
func (odb *OverdriveDb) Insert(table string, doc map[string]any) (string, error) {
	b, _ := json.Marshal(doc)
	id, err := nativeInsert(odb.handle, table, string(b))
	if err != nil { return "", fmt.Errorf("overdrive: insert: %w", err) }
	return id, nil
}

// InsertBatch inserts multiple documents. Returns a slice of _ids.
func (odb *OverdriveDb) InsertBatch(table string, docs []map[string]any) ([]string, error) {
	ids := make([]string, 0, len(docs))
	for _, doc := range docs {
		id, err := odb.Insert(table, doc)
		if err != nil { return nil, err }
		ids = append(ids, id)
	}
	return ids, nil
}

// Get retrieves a document by _id. Returns nil if not found.
func (odb *OverdriveDb) Get(table, id string) (map[string]any, error) {
	s, err := nativeGet(odb.handle, table, id)
	if err != nil || s == "" { return nil, err }
	var doc map[string]any
	if err := json.Unmarshal([]byte(s), &doc); err != nil { return nil, err }
	return doc, nil
}

// Update merges patch into the document with _id. Returns true if found.
func (odb *OverdriveDb) Update(table, id string, patch map[string]any) (bool, error) {
	b, _ := json.Marshal(patch)
	r := nativeUpdate(odb.handle, table, id, string(b))
	if r < 0 { return false, fmt.Errorf("overdrive: update: %s", nativeLastError()) }
	return r == 1, nil
}

// Delete removes the document with _id. Returns true if deleted.
func (odb *OverdriveDb) Delete(table, id string) (bool, error) {
	r := nativeDelete(odb.handle, table, id)
	if r < 0 { return false, fmt.Errorf("overdrive: delete: %s", nativeLastError()) }
	return r == 1, nil
}

// Count returns the number of documents in table.
func (odb *OverdriveDb) Count(table string) (int, error) {
	n := nativeCount(odb.handle, table)
	if n < 0 { return 0, fmt.Errorf("overdrive: count: %s", nativeLastError()) }
	return int(n), nil
}

// Query executes a SQL statement. Returns rows as a slice of maps.
func (odb *OverdriveDb) Query(sql string) ([]map[string]any, error) {
	s, err := nativeQuery(odb.handle, sql)
	if err != nil { return nil, err }
	var result map[string]any
	if err := json.Unmarshal([]byte(s), &result); err != nil { return nil, err }
	if rows, ok := result["rows"].([]any); ok {
		out := make([]map[string]any, 0, len(rows))
		for _, r := range rows {
			if m, ok := r.(map[string]any); ok { out = append(out, m) }
		}
		return out, nil
	}
	return nil, nil
}

// Search performs full-text search across table.
func (odb *OverdriveDb) Search(table, text string) ([]map[string]any, error) {
	s, err := nativeSearch(odb.handle, table, text)
	if err != nil || s == "" { return nil, err }
	var rows []map[string]any
	json.Unmarshal([]byte(s), &rows)
	return rows, nil
}

// BeginTransaction starts an MVCC transaction.
func (odb *OverdriveDb) BeginTransaction(iso IsolationLevel) (*Transaction, error) {
	id := nativeBeginTransaction(odb.handle, int32(iso))
	if id == 0 { return nil, fmt.Errorf("overdrive: beginTransaction: %s", nativeLastError()) }
	return &Transaction{id: id}, nil
}

// CommitTransaction commits a transaction.
func (odb *OverdriveDb) CommitTransaction(txn *Transaction) error {
	if nativeCommitTransaction(odb.handle, txn.id) != 0 {
		return fmt.Errorf("overdrive: commit: %s", nativeLastError())
	}
	return nil
}

// AbortTransaction rolls back a transaction.
func (odb *OverdriveDb) AbortTransaction(txn *Transaction) error {
	nativeAbortTransaction(odb.handle, txn.id)
	return nil
}

// Transaction runs fn inside a transaction. Auto-commits on nil error.
func (odb *OverdriveDb) Transaction(fn func() error, iso IsolationLevel) error {
	txn, err := odb.BeginTransaction(iso)
	if err != nil { return err }
	if err := fn(); err != nil {
		_ = odb.AbortTransaction(txn)
		return err
	}
	return odb.CommitTransaction(txn)
}
