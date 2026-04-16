// Package overdrive provides Go bindings for the OverDrive InCode SDK (v1.4).
//
// Embeddable document database — like SQLite for JSON.
//
// Simplified API (v1.4):
//
//	db, err := overdrive.Open("myapp.odb", overdrive.WithAutoCreateTables(true))
//	if err != nil { log.Fatal(err) }
//	defer db.Close()
//
//	id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})
//	results, _ := db.Query("SELECT * FROM users WHERE age > 25")
//
// Password-protected:
//
//	db, _ := overdrive.Open("secure.odb", overdrive.WithPassword("my-secret-pass"))
//
// RAM engine:
//
//	db, _ := overdrive.Open("cache.odb", overdrive.WithEngine("RAM"))
package overdrive

/*
#cgo windows LDFLAGS: -L${SRCDIR}/lib -loverdrive
#cgo linux LDFLAGS: -L${SRCDIR}/lib -loverdrive -lm -ldl -lpthread
#cgo darwin LDFLAGS: -L${SRCDIR}/lib -loverdrive

#include <stdlib.h>

// OverDrive C FFI declarations
typedef void* ODB;

// Core (v1.3)
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

// v1.4: Transactions
extern unsigned long long overdrive_begin_transaction(ODB db, int isolation_level);
extern int    overdrive_commit_transaction(ODB db, unsigned long long txn_id);
extern int    overdrive_abort_transaction(ODB db, unsigned long long txn_id);

// v1.4: Integrity & Stats
extern char*  overdrive_verify_integrity(ODB db);
extern char*  overdrive_stats(ODB db);

// v1.4: Engine & auto-create
extern ODB    overdrive_open_with_engine(const char* path, const char* engine, const char* options_json);
extern int    overdrive_set_auto_create_tables(ODB db, int enabled);
extern const char* overdrive_get_error_details();
extern int    overdrive_create_table_with_engine(ODB db, const char* name, const char* engine);

// v1.4: RAM engine
extern int    overdrive_snapshot(ODB db, const char* path);
extern int    overdrive_restore(ODB db, const char* path);
extern char*  overdrive_memory_usage(ODB db);

// v1.4: Watchdog
extern char*  overdrive_watchdog(const char* path);
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"io"
	"math"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"sync"
	"time"
	"unsafe"
)

// ── Error Types (Task 30) ──────────────────────

// OverDriveError is the base interface for all OverDrive errors.
type OverDriveError interface {
	error
	Code() string
	Context() string
	Suggestions() []string
	DocLink() string
}

type odbError struct {
	code        string
	message     string
	context     string
	suggestions []string
	docLink     string
}

func (e *odbError) Error() string {
	parts := []string{}
	if e.code != "" {
		parts = append(parts, fmt.Sprintf("Error %s: %s", e.code, e.message))
	} else {
		parts = append(parts, e.message)
	}
	if e.context != "" {
		parts = append(parts, "Context: "+e.context)
	}
	if len(e.suggestions) > 0 {
		lines := []string{"Suggestions:"}
		for _, s := range e.suggestions {
			lines = append(lines, "  • "+s)
		}
		parts = append(parts, strings.Join(lines, "\n"))
	}
	if e.docLink != "" {
		parts = append(parts, "For more help: "+e.docLink)
	}
	return strings.Join(parts, "\n")
}

func (e *odbError) Code() string         { return e.code }
func (e *odbError) Context() string       { return e.context }
func (e *odbError) Suggestions() []string { return e.suggestions }
func (e *odbError) DocLink() string       { return e.docLink }

// Specific error types
type AuthenticationError struct{ odbError }
type TableError struct{ odbError }
type QueryError struct{ odbError }
type TransactionError struct{ odbError }
type IOError struct{ odbError }
type FFIError struct{ odbError }

func newError(msg string) error {
	return &odbError{message: msg}
}

func newTypedError(msg, code, ctx string, suggestions []string, docLink string) error {
	base := odbError{
		code:        code,
		message:     msg,
		context:     ctx,
		suggestions: suggestions,
		docLink:     docLink,
	}
	if strings.HasPrefix(code, "ODB-AUTH") {
		return &AuthenticationError{base}
	} else if strings.HasPrefix(code, "ODB-TABLE") {
		return &TableError{base}
	} else if strings.HasPrefix(code, "ODB-QUERY") {
		return &QueryError{base}
	} else if strings.HasPrefix(code, "ODB-TXN") {
		return &TransactionError{base}
	} else if strings.HasPrefix(code, "ODB-IO") {
		return &IOError{base}
	} else if strings.HasPrefix(code, "ODB-FFI") {
		return &FFIError{base}
	}
	return &base
}

// ── Core Types ─────────────────────────────────

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

// WatchdogReport holds the result of a watchdog integrity check.
type WatchdogReport struct {
	FilePath          string  `json:"file_path"`
	FileSizeBytes     int64   `json:"file_size_bytes"`
	LastModified      int64   `json:"last_modified"`
	IntegrityStatus   string  `json:"integrity_status"`
	CorruptionDetails *string `json:"corruption_details"`
	PageCount         int     `json:"page_count"`
	MagicValid        bool    `json:"magic_valid"`
}

// MemoryUsage holds RAM engine memory statistics.
type MemoryUsage struct {
	Bytes      int64   `json:"bytes"`
	Mb         float64 `json:"mb"`
	LimitBytes int64   `json:"limit_bytes"`
	Percent    float64 `json:"percent"`
}

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

// ── OpenOption (Task 25) ───────────────────────

// OpenOptions holds configuration for opening a database.
type OpenOptions struct {
	Password         string
	Engine           string
	AutoCreateTables bool
}

// OpenOption is a functional option for Open().
type OpenOption func(*OpenOptions)

// WithPassword sets the encryption password (minimum 8 characters).
func WithPassword(pwd string) OpenOption {
	return func(o *OpenOptions) { o.Password = pwd }
}

// WithEngine sets the storage engine: "Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming".
func WithEngine(engine string) OpenOption {
	return func(o *OpenOptions) { o.Engine = engine }
}

// WithAutoCreateTables enables/disables auto-table creation on first insert.
func WithAutoCreateTables(enabled bool) OpenOption {
	return func(o *OpenOptions) { o.AutoCreateTables = enabled }
}

// TableOption is a functional option for CreateTable().
type TableOption func(*tableOpts)

type tableOpts struct {
	engine string
}

// WithTableEngine sets the storage engine for a specific table.
func WithTableEngine(engine string) TableOption {
	return func(o *tableOpts) { o.engine = engine }
}

// ── Internal helpers ───────────────────────────

func lastError() error {
	msg := C.overdrive_last_error()
	if msg == nil {
		return newError("unknown overdrive error")
	}
	return newError(C.GoString(msg))
}

func checkErrorStructured() error {
	detailsRaw := C.overdrive_get_error_details()
	if detailsRaw != nil {
		ds := C.GoString(detailsRaw)
		if ds != "" {
			var data map[string]any
			if err := json.Unmarshal([]byte(ds), &data); err == nil {
				code, _ := data["code"].(string)
				msg, _ := data["message"].(string)
				ctx, _ := data["context"].(string)
				docLink, _ := data["doc_link"].(string)
				var suggestions []string
				if rawSugg, ok := data["suggestions"].([]any); ok {
					for _, s := range rawSugg {
						if str, ok := s.(string); ok {
							suggestions = append(suggestions, str)
						}
					}
				}
				if msg != "" || code != "" {
					return newTypedError(msg, code, ctx, suggestions, docLink)
				}
			}
		}
	}
	return lastError()
}

func readAndFree(ptr *C.char) string {
	if ptr == nil {
		return ""
	}
	s := C.GoString(ptr)
	C.overdrive_free_string(ptr)
	return s
}

// ── Lifecycle ──────────────────────────────────

// Version returns the SDK version string.
func Version() string {
	v := C.overdrive_version()
	if v == nil {
		return "unknown"
	}
	return C.GoString(v)
}

// Open opens or creates a database with functional options (v1.4 API).
//
//	db, err := overdrive.Open("app.odb")
//	db, err := overdrive.Open("app.odb", overdrive.WithPassword("secret123"))
//	db, err := overdrive.Open("app.odb", overdrive.WithEngine("RAM"))
func Open(path string, opts ...OpenOption) (*DB, error) {
	options := &OpenOptions{
		Engine:           "Disk",
		AutoCreateTables: true,
	}
	for _, opt := range opts {
		opt(options)
	}

	// Validate engine
	validEngines := map[string]bool{
		"Disk": true, "RAM": true, "Vector": true,
		"Time-Series": true, "Graph": true, "Streaming": true,
	}
	if !validEngines[options.Engine] {
		return nil, newTypedError(
			fmt.Sprintf("Invalid engine '%s'. Valid options: Disk, Graph, RAM, Streaming, Time-Series, Vector", options.Engine),
			"", "", nil, "")
	}

	// Validate password
	if options.Password != "" && len(options.Password) < 8 {
		return nil, newTypedError(
			"Password must be at least 8 characters long",
			"ODB-AUTH-002", "",
			[]string{"Use a password with at least 8 characters", "Consider using 12+ characters for better security"},
			"https://overdrive-db.com/docs/errors/ODB-AUTH-002")
	}

	// If no special options, use simple open for backward compat
	if options.Engine == "Disk" && options.Password == "" && options.AutoCreateTables {
		cPath := C.CString(path)
		defer C.free(unsafe.Pointer(cPath))
		handle := C.overdrive_open(cPath)
		if handle == nil {
			return nil, fmt.Errorf("failed to open database: %w", checkErrorStructured())
		}
		setSecurePermissions(path)
		return &DB{handle: handle, path: path}, nil
	}

	// Build options JSON
	optMap := map[string]any{"auto_create_tables": options.AutoCreateTables}
	if options.Password != "" {
		optMap["password"] = options.Password
	}
	optJSON, _ := json.Marshal(optMap)

	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))
	cEngine := C.CString(options.Engine)
	defer C.free(unsafe.Pointer(cEngine))
	cOpts := C.CString(string(optJSON))
	defer C.free(unsafe.Pointer(cOpts))

	handle := C.overdrive_open_with_engine(cPath, cEngine, cOpts)
	if handle == nil {
		return nil, fmt.Errorf("failed to open database: %w", checkErrorStructured())
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

// CreateTable creates a new table with optional engine selection.
//
//	db.CreateTable("users")
//	db.CreateTable("cache", overdrive.WithTableEngine("RAM"))
func (db *DB) CreateTable(name string, opts ...TableOption) error {
	to := &tableOpts{engine: "Disk"}
	for _, opt := range opts {
		opt(to)
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	if to.engine == "Disk" {
		if C.overdrive_create_table(db.handle, cName) != 0 {
			return checkErrorStructured()
		}
	} else {
		cEngine := C.CString(to.engine)
		defer C.free(unsafe.Pointer(cEngine))
		if C.overdrive_create_table_with_engine(db.handle, cName, cEngine) != 0 {
			return checkErrorStructured()
		}
	}
	return nil
}

// DropTable drops a table and all its data.
func (db *DB) DropTable(name string) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	if C.overdrive_drop_table(db.handle, cName) != 0 {
		return checkErrorStructured()
	}
	return nil
}

// ListTables returns all table names.
func (db *DB) ListTables() ([]string, error) {
	ptr := C.overdrive_list_tables(db.handle)
	if ptr == nil {
		return nil, checkErrorStructured()
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
		return "", checkErrorStructured()
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
		return false, checkErrorStructured()
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
		return false, checkErrorStructured()
	}
	return result == 1, nil
}

// Count returns the number of documents in a table.
func (db *DB) Count(table string) (int, error) {
	cTable := C.CString(table)
	defer C.free(unsafe.Pointer(cTable))

	result := C.overdrive_count(db.handle, cTable)
	if result == -1 {
		return 0, checkErrorStructured()
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
		return nil, checkErrorStructured()
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

// QuerySafe executes a parameterized SQL query — SQL injection safe.
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

// ── RAM Engine Methods (Task 26) ────────────────

// Snapshot persists the current RAM database to a snapshot file.
func (db *DB) Snapshot(path string) error {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	if C.overdrive_snapshot(db.handle, cPath) != 0 {
		return checkErrorStructured()
	}
	return nil
}

// Restore loads a previously saved snapshot into the current database handle.
func (db *DB) Restore(path string) error {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	if C.overdrive_restore(db.handle, cPath) != 0 {
		return checkErrorStructured()
	}
	return nil
}

// MemoryUsageStats returns current RAM consumption statistics.
func (db *DB) MemoryUsageStats() (*MemoryUsage, error) {
	ptr := C.overdrive_memory_usage(db.handle)
	if ptr == nil {
		return nil, checkErrorStructured()
	}
	s := readAndFree(ptr)
	var usage MemoryUsage
	if err := json.Unmarshal([]byte(s), &usage); err != nil {
		return nil, err
	}
	return &usage, nil
}

// ── Watchdog (Task 27) ──────────────────────────

// Watchdog inspects a .odb file for integrity, size, and modification status.
// Static function — does not require an open database handle.
//
//	report, err := overdrive.Watchdog("app.odb")
//	if report.IntegrityStatus == "corrupted" {
//	    log.Printf("Corrupted: %s", *report.CorruptionDetails)
//	}
func Watchdog(filePath string) (*WatchdogReport, error) {
	cPath := C.CString(filePath)
	defer C.free(unsafe.Pointer(cPath))

	ptr := C.overdrive_watchdog(cPath)
	if ptr == nil {
		return nil, fmt.Errorf("watchdog() returned NULL for path: %s", filePath)
	}
	s := readAndFree(ptr)
	if s == "" {
		return nil, fmt.Errorf("watchdog() returned empty response for path: %s", filePath)
	}
	var report WatchdogReport
	if err := json.Unmarshal([]byte(s), &report); err != nil {
		return nil, err
	}
	if report.FilePath == "" {
		report.FilePath = filePath
	}
	return &report, nil
}

// ── MVCC Transactions (Task 28) ────────────────

// BeginTransaction starts a new MVCC transaction.
func (db *DB) BeginTransaction(isolation IsolationLevel) (*TransactionHandle, error) {
	txnID := C.overdrive_begin_transaction(db.handle, C.int(isolation))
	if txnID == 0 {
		return nil, checkErrorStructured()
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
		return checkErrorStructured()
	}
	txn.Active = false
	return nil
}

// AbortTransaction aborts a transaction, discarding all changes.
func (db *DB) AbortTransaction(txn *TransactionHandle) error {
	if C.overdrive_abort_transaction(db.handle, C.ulonglong(txn.TxnID)) != 0 {
		return checkErrorStructured()
	}
	txn.Active = false
	return nil
}

// Transaction executes a function inside a transaction with automatic commit/rollback.
//
//	err := db.Transaction(func(txn *overdrive.TransactionHandle) error {
//	    db.Insert("users", map[string]any{"name": "Alice"})
//	    db.Insert("logs", map[string]any{"event": "user_created"})
//	    return nil
//	}, overdrive.ReadCommitted)
func (db *DB) Transaction(fn func(*TransactionHandle) error, isolation IsolationLevel) error {
	txn, err := db.BeginTransaction(isolation)
	if err != nil {
		return err
	}
	if err := fn(txn); err != nil {
		_ = db.AbortTransaction(txn)
		return err
	}
	return db.CommitTransaction(txn)
}

// TransactionWithRetry executes a transaction with exponential-backoff retry on TransactionError.
func (db *DB) TransactionWithRetry(fn func(*TransactionHandle) error, isolation IsolationLevel, maxRetries int) error {
	var lastErr error
	for attempt := 0; attempt < maxRetries; attempt++ {
		err := db.Transaction(fn, isolation)
		if err == nil {
			return nil
		}
		if _, ok := err.(*TransactionError); ok {
			lastErr = err
			if attempt < maxRetries-1 {
				delay := time.Duration(math.Min(float64(100*int(1<<uint(attempt))), 2000)) * time.Millisecond
				time.Sleep(delay)
			}
		} else {
			return err
		}
	}
	return lastErr
}

// ── Helper Methods (Task 29) ────────────────────

// FindOne returns the first document matching where, or nil if no match.
func (db *DB) FindOne(table, where string) (map[string]any, error) {
	var sql string
	if where != "" {
		sql = "SELECT * FROM " + table + " WHERE " + where + " LIMIT 1"
	} else {
		sql = "SELECT * FROM " + table + " LIMIT 1"
	}
	qr, err := db.Query(sql)
	if err != nil {
		return nil, err
	}
	if len(qr.Rows) == 0 {
		return nil, nil
	}
	return qr.Rows[0], nil
}

// FindAll returns all documents matching where.
func (db *DB) FindAll(table, where, orderBy string, limit int) ([]map[string]any, error) {
	sql := "SELECT * FROM " + table
	if where != "" {
		sql += " WHERE " + where
	}
	if orderBy != "" {
		sql += " ORDER BY " + orderBy
	}
	if limit > 0 {
		sql += fmt.Sprintf(" LIMIT %d", limit)
	}
	qr, err := db.Query(sql)
	if err != nil {
		return nil, err
	}
	return qr.Rows, nil
}

// UpdateMany updates all documents matching where. Returns count of updated documents.
func (db *DB) UpdateMany(table, where string, updates map[string]any) (int, error) {
	setClauses := []string{}
	for k, v := range updates {
		valJSON, _ := json.Marshal(v)
		setClauses = append(setClauses, fmt.Sprintf("%s = %s", k, string(valJSON)))
	}
	sql := "UPDATE " + table + " SET " + strings.Join(setClauses, ", ") + " WHERE " + where
	qr, err := db.Query(sql)
	if err != nil {
		return 0, err
	}
	return qr.RowsAffected, nil
}

// DeleteMany deletes all documents matching where. Returns count of deleted documents.
func (db *DB) DeleteMany(table, where string) (int, error) {
	sql := "DELETE FROM " + table + " WHERE " + where
	qr, err := db.Query(sql)
	if err != nil {
		return 0, err
	}
	return qr.RowsAffected, nil
}

// CountWhere counts documents matching where.
func (db *DB) CountWhere(table, where string) (int, error) {
	var sql string
	if where != "" {
		sql = "SELECT COUNT(*) FROM " + table + " WHERE " + where
	} else {
		sql = "SELECT COUNT(*) FROM " + table
	}
	qr, err := db.Query(sql)
	if err != nil {
		return 0, err
	}
	if len(qr.Rows) == 0 {
		return 0, nil
	}
	row := qr.Rows[0]
	for _, key := range []string{"COUNT(*)", "count(*)", "count", "COUNT"} {
		if val, ok := row[key]; ok {
			if n, ok := val.(float64); ok {
				return int(n), nil
			}
		}
	}
	// Fallback: first numeric value
	for _, val := range row {
		if n, ok := val.(float64); ok {
			return int(n), nil
		}
	}
	return 0, nil
}

// Exists checks whether a document with the given id exists.
func (db *DB) Exists(table, id string) (bool, error) {
	doc, err := db.Get(table, id)
	if err != nil {
		return false, err
	}
	return doc != nil, nil
}

// ── Integrity Verification ─────────────────────

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
		return nil, checkErrorStructured()
	}
	s := readAndFree(ptr)
	var report IntegrityReport
	if err := json.Unmarshal([]byte(s), &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// ── Extended Stats ─────────────────────────────

// StatsResult holds detailed database statistics.
type StatsResult struct {
	Tables             int    `json:"tables"`
	TotalRecords       int    `json:"total_records"`
	FileSizeBytes      uint64 `json:"file_size_bytes"`
	Path               string `json:"path"`
	MvccActiveVersions int    `json:"mvcc_active_versions"`
	PageSize           int    `json:"page_size"`
	SdkVersion         string `json:"sdk_version"`
}

// Stats returns detailed database statistics including MVCC info.
func (db *DB) Stats() (*StatsResult, error) {
	ptr := C.overdrive_stats(db.handle)
	if ptr == nil {
		return nil, checkErrorStructured()
	}
	s := readAndFree(ptr)
	var stats StatsResult
	if err := json.Unmarshal([]byte(s), &stats); err != nil {
		return nil, err
	}
	return &stats, nil
}

// ── Security ───────────────────────────────────

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
// the specified environment variable.
func OpenEncrypted(path, keyEnvVar string) (*DB, error) {
	key := os.Getenv(keyEnvVar)
	if key == "" {
		return nil, fmt.Errorf(
			"[overdrive] encryption key env var '%s' is not set or empty — "+
				"set it with: export %s=\"your-key\" (bash) or "+
				"$env:%s=\"your-key\" (PowerShell)", keyEnvVar, keyEnvVar, keyEnvVar)
	}
	os.Setenv("__OVERDRIVE_KEY", key)
	defer func() {
		os.Unsetenv("__OVERDRIVE_KEY")
		keyBytes := []byte(key)
		for i := range keyBytes {
			keyBytes[i] = 0
		}
	}()
	return Open(path)
}

// Backup creates an encrypted backup of the database.
func (db *DB) Backup(destPath string) error {
	db.Sync()
	if err := copyFile(db.path, destPath); err != nil {
		return fmt.Errorf("overdrive backup: %w", err)
	}
	walSrc := db.path + ".wal"
	walDst := destPath + ".wal"
	if _, err := os.Stat(walSrc); err == nil {
		if err := copyFile(walSrc, walDst); err != nil {
			return fmt.Errorf("overdrive backup (wal): %w", err)
		}
	}
	setSecurePermissions(destPath)
	return nil
}

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

// CleanupWAL deletes the WAL file after a confirmed commit.
func (db *DB) CleanupWAL() error {
	walPath := db.path + ".wal"
	if _, err := os.Stat(walPath); err == nil {
		return os.Remove(walPath)
	}
	return nil
}

// ── SafeDB — Concurrent-safe wrapper ───────────

// SafeDB wraps DB with a sync.RWMutex for safe concurrent access.
type SafeDB struct {
	mu sync.RWMutex
	db *DB
}

// NewSafeDB wraps an existing DB in a thread-safe wrapper.
func NewSafeDB(db *DB) *SafeDB {
	return &SafeDB{db: db}
}

// OpenSafe opens (or creates) a database wrapped in SafeDB.
func OpenSafe(path string, opts ...OpenOption) (*SafeDB, error) {
	db, err := Open(path, opts...)
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
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.db.Query(sql)
}
func (s *SafeDB) QuerySafe(tmpl string, params ...string) (*QueryResult, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.db.QuerySafe(tmpl, params...)
}
func (s *SafeDB) Insert(table string, doc map[string]any) (string, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.db.Insert(table, doc)
}
func (s *SafeDB) Get(table, id string) (map[string]any, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.db.Get(table, id)
}
func (s *SafeDB) Backup(dest string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.db.Backup(dest)
}
func (s *SafeDB) CleanupWAL() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.db.CleanupWAL()
}
func (s *SafeDB) Close() {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.db.Close()
}
