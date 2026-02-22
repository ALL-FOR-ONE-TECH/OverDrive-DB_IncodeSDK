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
*/
import "C"

import (
	"encoding/json"
	"errors"
	"fmt"
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
func Open(path string) (*DB, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	handle := C.overdrive_open(cPath)
	if handle == nil {
		return nil, fmt.Errorf("failed to open database: %w", lastError())
	}
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
