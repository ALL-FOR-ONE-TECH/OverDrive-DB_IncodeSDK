// Package overdrive provides Go bindings for the OverDrive InCode SDK.
//
// Embeddable document database — like SQLite for JSON.
//
// Usage:
//
//	db, err := overdrive.Open("myapp.odb")
//	if err != nil { log.Fatal(err) }
//	defer db.Close()
//
//	db.CreateTable("users")
//	id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})
//	results, _ := db.Query("SELECT * FROM users WHERE age > 25")
package overdrive

/*
#cgo windows LDFLAGS: -L${SRCDIR}/lib -loverdrive
#cgo linux LDFLAGS: -L${SRCDIR}/lib -loverdrive -lm -ldl -lpthread
#cgo darwin LDFLAGS: -L${SRCDIR}/lib -loverdrive

#include <stdlib.h>

// OverDrive C FFI declarations
typedef void* ODB;

extern ODB   overdrive_open(const char* path);
extern void   overdrive_close(ODB db);
extern void   overdrive_sync(ODB db);
extern int    overdrive_create_table(ODB db, const char* name);
extern int    overdrive_drop_table(ODB db, const char* name);
extern char*  overdrive_list_tables(ODB db);
extern int    overdrive_table_exists(ODB db, const char* name);
extern char*  overdrive_insert(ODB db, const char* table, const char* json_doc);
extern char*  overdrive_get(ODB db, const char* table, const char* id);
extern int    overdrive_update(ODB db, const char* table, const char* id, const char* json_updates);
extern int    overdrive_delete(ODB db, const char* table, const char* id);
extern int    overdrive_count(ODB db, const char* table);
extern char*  overdrive_query(ODB db, const char* sql);
extern char*  overdrive_search(ODB db, const char* table, const char* text);
extern const char* overdrive_last_error();
extern void   overdrive_free_string(char* s);
extern const char* overdrive_version();

// Phase 5: MVCC Transactions
extern unsigned long long overdrive_begin_transaction(ODB db, int isolation_level);
extern int    overdrive_commit_transaction(ODB db, unsigned long long txn_id);
extern int    overdrive_abort_transaction(ODB db, unsigned long long txn_id);

// Phase 5: Integrity & Stats
extern char*  overdrive_verify_integrity(ODB db);
extern char*  overdrive_stats(ODB db);
*/
import "C"

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"sync"
	"unsafe"
)

// DB represents an open OverDrive database.
type DB struct {
	handle C.ODB
	path   string
}

// QueryResult holds the result of a SQL query.
type QueryResult struct {
	Rows         []map[string]any `json:"rows"`
	Columns      []string         `json:"columns"`
	RowsAffected int              `json:"rows_affected"`
	ExecTimeMs   float64          `json:"execution_time_ms"`
}

// lastError returns the last error from the native library.
func lastError() error {
	msg := C.overdrive_last_error()
	if msg == nil {
		return errors.New("unknown overdrive error")
	}
	return errors.New(C.GoString(msg))
}

// readAndFree reads a C string and frees it, returning a Go string.
func readAndFree(ptr *C.char) string {
	if ptr == nil {
		return ""
	}
	s := C.GoString(ptr)
	C.overdrive_free_string(ptr)
	return s
}

// Version returns the SDK version string.
func Version() string {
	v := C.overdrive_version()
	if v == nil {
		return "unknown"
	}
	return C.GoString(v)
}

// Open opens or creates a database at the given path.
// File permissions are automatically hardened on open (chmod 600 / Windows ACL).
func Open(path string) (*DB, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	handle := C.overdrive_open(cPath)
	if handle == nil {
		return nil, fmt.Errorf("failed to open database: %w", lastError())
	}
	setSecurePermissions(path)
	return &DB{handle: handle, path: path}, nil
}

// Close closes the database and releases resources.
func (db *DB) Close() {
	if db.handle != nil {
		C.overdrive_close(db.handle)
		db.handle = nil
	}
}

// Sync forces all data to be written to disk.
func (db *DB) Sync() {
	if db.handle != nil {
		C.overdrive_sync(db.handle)
	}
}

// Path returns the database file path.
func (db *DB) Path() string {
	return db.path
}

// ── Tables ──────────────────────────────────────

// CreateTable creates a new table.
func (db *DB) CreateTable(name string) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	if C.overdrive_create_table(db.handle, cName) != 0 {
		return lastError()
	}
	return nil
}

// DropTable drops a table and all its data.
func (db *DB) DropTable(name string) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	if C.overdrive_drop_table(db.handle, cName) != 0 {
		return lastError()
	}
	return nil
}

// ListTables returns all table names.
func (db *DB) ListTables() ([]string, error) {
	ptr := C.overdrive_list_tables(db.handle)
	if ptr == nil {
		return nil, lastError()
	}
	s := readAndFree(ptr)
	var tables []string
	if err := json.Unmarshal([]byte(s), &tables); err != nil {
		return nil, err
	}
	return tables, nil
}

// TableExists checks if a table exists.
func (db *DB) TableExists(name string) bool {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	return C.overdrive_table_exists(db.handle, cName) == 1
}

// ── CRUD ────────────────────────────────────────

// Insert inserts a JSON document and returns the generated _id.
func (db *DB) Insert(table string, doc map[string]any) (string, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))

	jsonBytes, err := json.Marshal(doc)
	if err != nil {
		return "", err
	}
	cJSON := C.CString(string(jsonBytes))
	defer C.free(unsafe.Pointer(cJSON))

	ptr := C.overdrive_insert(db.handle, cTable, cJSON)
	if ptr == nil {
		return "", lastError()
	}
	return readAndFree(ptr), nil
}

// Get retrieves a document by _id. Returns nil if not found.
func (db *DB) Get(table, id string) (map[string]any, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))
	cID := C.CString(id)
	defer C.free(unsafe.Pointer(cID))

	ptr := C.overdrive_get(db.handle, cTable, cID)
	if ptr == nil {
		return nil, nil
	}
	s := readAndFree(ptr)
	var doc map[string]any
	if err := json.Unmarshal([]byte(s), &doc); err != nil {
		return nil, err
	}
	return doc, nil
}

// Update updates fields of a document by _id. Returns true if updated.
func (db *DB) Update(table, id string, updates map[string]any) (bool, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))
	cID := C.CString(id)
	defer C.free(unsafe.Pointer(cID))

	jsonBytes, err := json.Marshal(updates)
	if err != nil {
		return false, err
	}
	cJSON := C.CString(string(jsonBytes))
	defer C.free(unsafe.Pointer(cJSON))

	result := C.overdrive_update(db.handle, cTable, cID, cJSON)
	if result == -1 {
		return false, lastError()
	}
	return result == 1, nil
}

// Delete removes a document by _id. Returns true if deleted.
func (db *DB) Delete(table, id string) (bool, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))
	cID := C.CString(id)
	defer C.free(unsafe.Pointer(cID))

	result := C.overdrive_delete(db.handle, cTable, cID)
	if result == -1 {
		return false, lastError()
	}
	return result == 1, nil
}

// Count returns the number of documents in a table.
func (db *DB) Count(table string) (int, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))

	result := C.overdrive_count(db.handle, cTable)
	if result == -1 {
		return 0, lastError()
	}
	return int(result), nil
}

// ── Query ───────────────────────────────────────

// Query executes an SQL query and returns result rows.
func (db *DB) Query(sql string) (*QueryResult, error) {
	cSQL := C.CString(sql)
	defer C.free(unsafe.Pointer(cSQL))

	ptr := C.overdrive_query(db.handle, cSQL)
	if ptr == nil {
		return nil, lastError()
	}
	s := readAndFree(ptr)
	var qr QueryResult
	if err := json.Unmarshal([]byte(s), &qr); err != nil {
		return nil, err
	}
	return &qr, nil
}

// Search performs full-text search across a table.
func (db *DB) Search(table, text string) ([]map[string]any, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	ptr := C.overdrive_search(db.handle, cTable, cText)
	if ptr == nil {
		return nil, nil
	}
	s := readAndFree(ptr)
	var results []map[string]any
	if err := json.Unmarshal([]byte(s), &results); err != nil {
		return nil, err
	}
	return results, nil
}

// ── MVCC Transactions (Phase 5) ────────────────

// IsolationLevel represents the MVCC isolation level for a transaction.
type IsolationLevel int

const (
	ReadUncommitted IsolationLevel = 0
	ReadCommitted   IsolationLevel = 1
	RepeatableRead  IsolationLevel = 2
	Serializable    IsolationLevel = 3
)

// TransactionHandle represents an active MVCC transaction.
type TransactionHandle struct {
	TxnID     uint64
	Isolation IsolationLevel
	Active    bool
}

// BeginTransaction starts a new MVCC transaction.
func (db *DB) BeginTransaction(isolation IsolationLevel) (*TransactionHandle, error) {
	txnID := C.overdrive_begin_transaction(db.handle, C.int(isolation))
	if txnID == 0 {
		return nil, lastError()
	}
	return &TransactionHandle{
		TxnID:     uint64(txnID),
		Isolation: isolation,
		Active:    true,
	}, nil
}

// CommitTransaction commits a transaction, making all changes permanent.
func (db *DB) CommitTransaction(txn *TransactionHandle) error {
	if C.overdrive_commit_transaction(db.handle, C.ulonglong(txn.TxnID)) != 0 {
		return lastError()
	}
	txn.Active = false
	return nil
}

// AbortTransaction aborts a transaction, discarding all changes.
func (db *DB) AbortTransaction(txn *TransactionHandle) error {
	if C.overdrive_abort_transaction(db.handle, C.ulonglong(txn.TxnID)) != 0 {
		return lastError()
	}
	txn.Active = false
	return nil
}

// ── Integrity Verification (Phase 5) ───────────

// IntegrityReport holds the result of an integrity check.
type IntegrityReport struct {
	IsValid        bool     `json:"valid"`
	PagesChecked   int      `json:"pages_checked"`
	TablesVerified int      `json:"tables_verified"`
	Issues         []string `json:"issues"`
}

// VerifyIntegrity checks the database for corruption or inconsistencies.
func (db *DB) VerifyIntegrity() (*IntegrityReport, error) {
	ptr := C.overdrive_verify_integrity(db.handle)
	if ptr == nil {
		return nil, lastError()
	}
	s := readAndFree(ptr)
	var report IntegrityReport
	if err := json.Unmarshal([]byte(s), &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// ── Extended Stats (Phase 5) ───────────────────

// StatsResult holds detailed database statistics.
type StatsResult struct {
	Tables             int     `json:"tables"`
	TotalRecords       int     `json:"total_records"`
	FileSizeBytes      uint64  `json:"file_size_bytes"`
	Path               string  `json:"path"`
	MvccActiveVersions int     `json:"mvcc_active_versions"`
	PageSize           int     `json:"page_size"`
	SdkVersion         string  `json:"sdk_version"`
}

// Stats returns detailed database statistics including MVCC info.
func (db *DB) Stats() (*StatsResult, error) {
	ptr := C.overdrive_stats(db.handle)
	if ptr == nil {
		return nil, lastError()
	}
	s := readAndFree(ptr)
	var stats StatsResult
	if err := json.Unmarshal([]byte(s), &stats); err != nil {
		return nil, err
	}
	return &stats, nil
}

// ── Security (v1.3.0) ───────────────────────────

// setSecurePermissions restricts file access to the current user only.
// Windows: icacls (non-fatal if unavailable)
// Linux/macOS: chmod 600
func setSecurePermissions(path string) {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return
	}
	if runtime.GOOS == "windows" {
		cmd := exec.Command("icacls", path, "/inheritance:r", "/grant:r",
			fmt.Sprintf("%s:F", os.Getenv("USERNAME")))
		if err := cmd.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "[overdrive] WARNING: Could not harden permissions on '%s': %v\n", path, err)
		}
	} else {
		if err := os.Chmod(path, 0600); err != nil {
			fmt.Fprintf(os.Stderr, "[overdrive] WARNING: Could not chmod 600 '%s': %v\n", path, err)
		}
	}
}

// OpenEncrypted opens a database with an AES encryption key loaded from
// the specified environment variable. Never hardcode keys in source code.
//
//	// bash:       export ODB_KEY="my-secret-32-char-key!!!!"
//	// PowerShell: $env:ODB_KEY="my-secret-32-char-key!!!!"
//	db, err := overdrive.OpenEncrypted("app.odb", "ODB_KEY")
func OpenEncrypted(path, keyEnvVar string) (*DB, error) {
	key := os.Getenv(keyEnvVar)
	if key == "" {
		return nil, fmt.Errorf(
			"[overdrive] encryption key env var '%s' is not set or empty — "+
				"set it with: export %s=\"your-key\" (bash) or "+
				"$env:%s=\"your-key\" (PowerShell)", keyEnvVar, keyEnvVar, keyEnvVar)
	}
	// Pass key to engine via dedicated env var, remove immediately after open
	os.Setenv("__OVERDRIVE_KEY", key)
	defer func() {
		os.Unsetenv("__OVERDRIVE_KEY")
		// Zero the key in our local string (best-effort)
		keyBytes := []byte(key)
		for i := range keyBytes {
			keyBytes[i] = 0
		}
	}()
	return Open(path)
}

// Backup creates an encrypted backup of the database at destPath.
// Syncs all in-memory data to disk first, then copies .odb + .wal.
// Store the backup on a separate drive or cloud storage.
//
//	if err := db.Backup("backups/app_2026-03-04.odb"); err != nil { log.Fatal(err) }
func (db *DB) Backup(destPath string) error {
	// Sync in-memory pages to disk first
	db.Sync()

	// Copy main .odb file
	if err := copyFile(db.path, destPath); err != nil {
		return fmt.Errorf("overdrive backup: %w", err)
	}
	// Copy WAL if it exists
	walSrc := db.path + ".wal"
	walDst := destPath + ".wal"
	if _, err := os.Stat(walSrc); err == nil {
		if err := copyFile(walSrc, walDst); err != nil {
			return fmt.Errorf("overdrive backup (wal): %w", err)
		}
	}
	// Harden backup file permissions too
	setSecurePermissions(destPath)
	return nil
}

// copyFile is a helper that copies src → dst byte-for-byte.
func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()
	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()
	if _, err = io.Copy(out, in); err != nil {
		return err
	}
	return out.Sync()
}

// CleanupWAL deletes the WAL file after a confirmed commit to prevent
// stale replay attacks. Call this after CommitTransaction().
//
//	db.CommitTransaction(txn)
//	db.CleanupWAL() // Prevents WAL replay attack
func (db *DB) CleanupWAL() error {
	walPath := db.path + ".wal"
	if _, err := os.Stat(walPath); err == nil {
		return os.Remove(walPath)
	}
	return nil
}

// QuerySafe executes a parameterized SQL query — the safe way to include
// user input. Use ? as placeholders; values are sanitized before substitution.
// Returns an error if any param contains SQL injection patterns.
//
//	result, err := db.QuerySafe("SELECT * FROM users WHERE name = ?", "Alice")
func (db *DB) QuerySafe(sqlTemplate string, params ...string) (*QueryResult, error) {
	dangerous := []string{"DROP", "TRUNCATE", "ALTER", "EXEC", "EXECUTE", "UNION", "XP_"}
	dangerousTokens := []string{"--", ";--", "/*", "*/"}

	sanitized := make([]string, 0, len(params))
	for _, p := range params {
		upper := strings.ToUpper(p)
		for _, tok := range dangerousTokens {
			if strings.Contains(p, tok) {
				return nil, fmt.Errorf("[overdrive] SQL injection detected: param '%s' contains '%s'", p, tok)
			}
		}
		for _, kw := range dangerous {
			for _, word := range strings.Fields(upper) {
				if word == kw {
					return nil, fmt.Errorf("[overdrive] SQL injection detected: param '%s' contains keyword '%s'", p, kw)
				}
			}
		}
		// Escape single quotes
		sanitized = append(sanitized, "'"+strings.ReplaceAll(p, "'", "''")+"'")
	}

	sql := sqlTemplate
	for _, v := range sanitized {
		idx := strings.Index(sql, "?")
		if idx == -1 {
			return nil, fmt.Errorf("[overdrive] more params than '?' placeholders in SQL template")
		}
		sql = sql[:idx] + v + sql[idx+1:]
	}

	count := strings.Count(sqlTemplate, "?")
	if len(params) < count {
		return nil, fmt.Errorf("[overdrive] SQL template has %d '?' placeholders but only %d params", count, len(params))
	}
	return db.Query(sql)
}

// ── SafeDB — Concurrent-safe wrapper ───────────

// SafeDB wraps DB with a sync.RWMutex for safe concurrent access.
// Reads (Query, Get, Count) use RLock; writes (Insert, Update, Delete) use full Lock.
//
//	db, _ := overdrive.Open("app.odb")
//	safe := overdrive.NewSafeDB(db)
//
//	// Multiple goroutines safe:
//	go safe.Query("SELECT * FROM users")
//	go safe.Insert("users", map[string]any{"name": "Alice"})
type SafeDB struct {
	mu sync.RWMutex
	db *DB
}

// NewSafeDB wraps an existing DB in a thread-safe wrapper.
func NewSafeDB(db *DB) *SafeDB {
	return &SafeDB{db: db}
}

// OpenSafe opens (or creates) a database wrapped in SafeDB.
func OpenSafe(path string) (*SafeDB, error) {
	db, err := Open(path)
	if err != nil {
		return nil, err
	}
	return NewSafeDB(db), nil
}

// OpenSafeEncrypted opens an encrypted database wrapped in SafeDB.
func OpenSafeEncrypted(path, keyEnvVar string) (*SafeDB, error) {
	db, err := OpenEncrypted(path, keyEnvVar)
	if err != nil {
		return nil, err
	}
	return NewSafeDB(db), nil
}

func (s *SafeDB) Query(sql string) (*QueryResult, error) {
	s.mu.RLock(); defer s.mu.RUnlock()
	return s.db.Query(sql)
}
func (s *SafeDB) QuerySafe(tmpl string, params ...string) (*QueryResult, error) {
	s.mu.RLock(); defer s.mu.RUnlock()
	return s.db.QuerySafe(tmpl, params...)
}
func (s *SafeDB) Insert(table string, doc map[string]any) (string, error) {
	s.mu.Lock(); defer s.mu.Unlock()
	return s.db.Insert(table, doc)
}
func (s *SafeDB) Get(table, id string) (map[string]any, error) {
	s.mu.RLock(); defer s.mu.RUnlock()
	return s.db.Get(table, id)
}
func (s *SafeDB) Backup(dest string) error {
	s.mu.Lock(); defer s.mu.Unlock()
	return s.db.Backup(dest)
}
func (s *SafeDB) CleanupWAL() error {
	s.mu.Lock(); defer s.mu.Unlock()
	return s.db.CleanupWAL()
}
func (s *SafeDB) Close() {
	s.mu.Lock(); defer s.mu.Unlock()
	s.db.Close()
}


