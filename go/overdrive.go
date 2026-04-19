// Package overdrive provides Go bindings for the OverDrive InCode SDK (v1.4.3).
//
// Embeddable document database — like SQLite for JSON.
//
// This package uses platform-native dynamic loading (no CGo required!):
//   - Windows: LoadLibraryW / GetProcAddress (syscall)
//   - Linux/macOS: dlopen / dlsym (purego)
//
// Install:
//
//	go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest
//
// Usage:
//
//	db, err := overdrive.Open("myapp.odb")
//	if err != nil { log.Fatal(err) }
//	defer db.Close()
//
//	id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})
//	results, _ := db.Query("SELECT * FROM users WHERE age > 25")
//	fmt.Println(results.Rows)
//
// Password-protected:
//
//	db, _ := overdrive.Open("secure.odb", overdrive.WithPassword("my-secret-pass"))
//
// RAM engine for sub-microsecond caching:
//
//	db, _ := overdrive.Open("cache.odb", overdrive.WithEngine("RAM"))
package overdrive

import (
	"encoding/json"
	"fmt"
	"io"
	"math"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"
	"unsafe"
)

// ─────────────────────────────────────────────
// Platform loader interface
// ─────────────────────────────────────────────

// libHandle is the platform-native library handle abstraction.
type libHandle interface {
	sym(name string) (unsafe.Pointer, error)
	close()
}

// ─────────────────────────────────────────────
// Error Types
// ─────────────────────────────────────────────

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

// ─────────────────────────────────────────────
// Core Types
// ─────────────────────────────────────────────

// DB represents an open OverDrive database.
type DB struct {
	handle uintptr // opaque ODB* from native lib
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

// ─────────────────────────────────────────────
// OpenOption (functional options)
// ─────────────────────────────────────────────

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

// ─────────────────────────────────────────────
// Native library loader (no CGo)
// ─────────────────────────────────────────────

const (
	releaseVersion = "v1.4.3"
	releaseRepo    = "ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK"
)

var (
	globalLib     libHandle
	globalLibOnce sync.Once
	globalLibErr  error
)

// libName returns the platform-specific native library filename.
func libName() string {
	switch runtime.GOOS {
	case "windows":
		return "overdrive.dll"
	case "darwin":
		return "liboverdrive.dylib"
	default:
		return "liboverdrive.so"
	}
}

// releaseAssetName returns the GitHub Release asset name for this platform.
func releaseAssetName() string {
	switch runtime.GOOS {
	case "windows":
		return "overdrive.dll"
	case "darwin":
		if runtime.GOARCH == "arm64" {
			return "liboverdrive-macos-arm64.dylib"
		}
		return "liboverdrive-macos-x64.dylib"
	default: // linux
		if runtime.GOARCH == "arm64" {
			return "liboverdrive-linux-arm64.so"
		}
		return "liboverdrive-linux-x64.so"
	}
}

// downloadLibrary downloads the native library from GitHub Releases.
func downloadLibrary(dest string) error {
	asset := releaseAssetName()
	url := fmt.Sprintf("https://github.com/%s/releases/download/%s/%s",
		releaseRepo, releaseVersion, asset)

	fmt.Fprintf(os.Stderr, "overdrive: Downloading %s from %s...\n", asset, releaseVersion)

	resp, err := http.Get(url) //nolint:noctx
	if err != nil {
		return fmt.Errorf("HTTP GET failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		return fmt.Errorf("server returned %d for %s", resp.StatusCode, url)
	}

	f, err := os.Create(dest)
	if err != nil {
		return fmt.Errorf("cannot create %s: %w", dest, err)
	}
	defer f.Close()

	n, err := io.Copy(f, resp.Body)
	if err != nil {
		return fmt.Errorf("download write failed: %w", err)
	}
	if n < 100_000 {
		return fmt.Errorf("downloaded file too small (%d bytes) — may be corrupt", n)
	}

	fmt.Fprintf(os.Stderr, "overdrive: Downloaded %s (%.1f MB)\n", libName(), float64(n)/1_048_576)
	return nil
}

// loadLibrary finds and loads the native library, downloading if necessary.
// Called once via sync.Once.
func loadLibrary() (libHandle, error) {
	name := libName()

	// Search directories (in order of preference)
	execDir, _ := os.Executable()
	execDir = filepath.Dir(execDir)

	cwd, _ := os.Getwd()

	searchDirs := []string{
		cwd,
		filepath.Join(cwd, "lib"),
		execDir,
		filepath.Join(execDir, "lib"),
	}

	// Also check the directory of the caller's binary (cross-platform)
	for _, dir := range searchDirs {
		p := filepath.Join(dir, name)
		info, err := os.Stat(p)
		if err == nil && info.Size() > 100_000 {
			lib, err := openLib(p)
			if err == nil {
				return lib, nil
			}
		}
	}

	// Not found locally — download to cwd
	downloadPath := filepath.Join(cwd, name)
	if err := downloadLibrary(downloadPath); err != nil {
		return nil, fmt.Errorf(
			"native library '%s' not found and auto-download failed: %w\n"+
				"Download manually from: https://github.com/%s/releases/tag/%s\n"+
				"Place '%s' in your project directory or add it to PATH.",
			name, err, releaseRepo, releaseVersion, name,
		)
	}

	lib, err := openLib(downloadPath)
	if err != nil {
		return nil, fmt.Errorf("downloaded '%s' but failed to load: %w", name, err)
	}
	return lib, nil
}

// getLib returns the global library handle, loading if necessary.
func getLib() (libHandle, error) {
	globalLibOnce.Do(func() {
		globalLib, globalLibErr = loadLibrary()
	})
	return globalLib, globalLibErr
}

// ─────────────────────────────────────────────
// Helper: C string ↔ Go string (unsafe, no CGo)
// ─────────────────────────────────────────────

// cstring allocates a null-terminated C string in Go-managed memory.
// Returns a pointer and a cleanup func (call deferred).
func cstring(s string) (uintptr, func()) {
	bs := append([]byte(s), 0)
	ptr := uintptr(unsafe.Pointer(&bs[0]))
	return ptr, func() { runtime.KeepAlive(bs) }
}

// gostring reads a null-terminated C string from ptr.
func gostring(ptr uintptr) string {
	if ptr == 0 {
		return ""
	}
	var buf []byte
	for i := 0; ; i++ {
		b := *(*byte)(unsafe.Pointer(ptr + uintptr(i)))
		if b == 0 {
			break
		}
		buf = append(buf, b)
	}
	return string(buf)
}

// ─────────────────────────────────────────────
// Native function wrappers
// ─────────────────────────────────────────────

// callFunc1 calls a native function with 1 uintptr arg, returns uintptr.
type fn1 func(a1 uintptr) uintptr
type fn2 func(a1, a2 uintptr) uintptr
type fn3 func(a1, a2, a3 uintptr) uintptr
type fn4 func(a1, a2, a3, a4 uintptr) uintptr
type fn0 func() uintptr
type fn0v func()
type fn1v func(a1 uintptr)
type fn2v func(a1, a2 uintptr)
type fn1i2 func(a1 uintptr, a2 int32) uintptr
type fn1u64 func(a1 uintptr, a2 uint64) uintptr

// ─────────────────────────────────────────────
// Error helpers
// ─────────────────────────────────────────────

func lastError(lib libHandle) error {
	sym, err := lib.sym("overdrive_last_error")
	if err != nil {
		return newError("unknown overdrive error (last_error symbol missing)")
	}
	ptr := callFn0(sym)
	if ptr == 0 {
		return newError("unknown overdrive error")
	}
	return newError(gostring(ptr))
}

func checkErrorStructured(lib libHandle) error {
	sym, err := lib.sym("overdrive_get_error_details")
	if err == nil {
		ptr := callFn0(sym)
		if ptr != 0 {
			ds := gostring(ptr)
			if ds != "" {
				var data map[string]any
				if jsonErr := json.Unmarshal([]byte(ds), &data); jsonErr == nil {
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
	}
	return lastError(lib)
}

func readAndFree(lib libHandle, ptr uintptr) string {
	if ptr == 0 {
		return ""
	}
	s := gostring(ptr)
	if sym, err := lib.sym("overdrive_free_string"); err == nil {
		callFn1v(sym, ptr)
	}
	return s
}

// ─────────────────────────────────────────────
// Open / Close
// ─────────────────────────────────────────────

// Version returns the SDK version string.
func Version() string {
	lib, err := getLib()
	if err != nil {
		return "unknown"
	}
	sym, err := lib.sym("overdrive_version")
	if err != nil {
		return "unknown"
	}
	ptr := callFn0(sym)
	return gostring(ptr)
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

	if options.Password != "" && len(options.Password) < 8 {
		return nil, newTypedError(
			"Password must be at least 8 characters long",
			"ODB-AUTH-002", "",
			[]string{"Use a password with at least 8 characters", "Consider using 12+ characters for better security"},
			"https://overdrive-db.com/docs/errors/ODB-AUTH-002")
	}

	lib, err := getLib()
	if err != nil {
		return nil, fmt.Errorf("failed to load native library: %w", err)
	}

	// Resolve to absolute path — avoids issues when CWD changes later
	absPath, err := filepath.Abs(path)
	if err != nil {
		absPath = path
	}

	var handle uintptr

	// Simple open for default options
	if options.Engine == "Disk" && options.Password == "" && options.AutoCreateTables {
		sym, symErr := lib.sym("overdrive_open")
		if symErr != nil {
			return nil, fmt.Errorf("symbol overdrive_open not found: %w", symErr)
		}
		cpath, keep := cstring(absPath)
		handle = callFn1(sym, cpath)
		keep()
	} else {
		sym, symErr := lib.sym("overdrive_open_with_engine")
		if symErr != nil {
			return nil, fmt.Errorf("symbol overdrive_open_with_engine not found: %w", symErr)
		}
		optMap := map[string]any{"auto_create_tables": options.AutoCreateTables}
		if options.Password != "" {
			optMap["password"] = options.Password
		}
		optJSON, _ := json.Marshal(optMap)

		cpath, keepP := cstring(absPath)
		cengine, keepE := cstring(options.Engine)
		copts, keepO := cstring(string(optJSON))
		handle = callFn3(sym, cpath, cengine, copts)
		keepP()
		keepE()
		keepO()
	}

	if handle == 0 {
		return nil, fmt.Errorf("failed to open database: %w", checkErrorStructured(lib))
	}

	setSecurePermissions(absPath)
	return &DB{handle: handle, path: absPath}, nil
}

// Close closes the database and releases resources.
func (db *DB) Close() {
	if db.handle == 0 {
		return
	}
	lib, err := getLib()
	if err != nil {
		return
	}
	sym, err := lib.sym("overdrive_close")
	if err != nil {
		return
	}
	callFn1v(sym, db.handle)
	db.handle = 0
}

// Sync forces all data to be written to disk.
func (db *DB) Sync() {
	if db.handle == 0 {
		return
	}
	lib, err := getLib()
	if err != nil {
		return
	}
	sym, err := lib.sym("overdrive_sync")
	if err != nil {
		return
	}
	callFn1v(sym, db.handle)
}

// Path returns the database file path.
func (db *DB) Path() string {
	return db.path
}

// ─────────────────────────────────────────────
// Tables
// ─────────────────────────────────────────────

// CreateTable creates a new table with optional engine selection.
//
//	db.CreateTable("users")
//	db.CreateTable("cache", overdrive.WithTableEngine("RAM"))
func (db *DB) CreateTable(name string, opts ...TableOption) error {
	to := &tableOpts{engine: "Disk"}
	for _, opt := range opts {
		opt(to)
	}
	lib, err := getLib()
	if err != nil {
		return err
	}

	cname, keep := cstring(name)
	defer keep()

	if to.engine == "Disk" {
		sym, err := lib.sym("overdrive_create_table")
		if err != nil {
			return err
		}
		if callFn2(sym, db.handle, cname) != 0 {
			return checkErrorStructured(lib)
		}
	} else {
		sym, err := lib.sym("overdrive_create_table_with_engine")
		if err != nil {
			return err
		}
		cengine, keepE := cstring(to.engine)
		defer keepE()
		if callFn3(sym, db.handle, cname, cengine) != 0 {
			return checkErrorStructured(lib)
		}
	}
	return nil
}

// DropTable drops a table and all its data.
func (db *DB) DropTable(name string) error {
	lib, err := getLib()
	if err != nil {
		return err
	}
	sym, err := lib.sym("overdrive_drop_table")
	if err != nil {
		return err
	}
	cname, keep := cstring(name)
	defer keep()
	if callFn2(sym, db.handle, cname) != 0 {
		return checkErrorStructured(lib)
	}
	return nil
}

// ListTables returns all table names.
func (db *DB) ListTables() ([]string, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_list_tables")
	if err != nil {
		return nil, err
	}
	ptr := callFn1(sym, db.handle)
	if ptr == 0 {
		return nil, checkErrorStructured(lib)
	}
	s := readAndFree(lib, ptr)
	var tables []string
	if err := json.Unmarshal([]byte(s), &tables); err != nil {
		return nil, err
	}
	return tables, nil
}

// TableExists checks if a table exists.
func (db *DB) TableExists(name string) bool {
	lib, err := getLib()
	if err != nil {
		return false
	}
	sym, err := lib.sym("overdrive_table_exists")
	if err != nil {
		return false
	}
	cname, keep := cstring(name)
	defer keep()
	return callFn2(sym, db.handle, cname) == 1
}

// ─────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────

// Insert inserts a JSON document and returns the generated _id.
func (db *DB) Insert(table string, doc map[string]any) (string, error) {
	lib, err := getLib()
	if err != nil {
		return "", err
	}
	sym, err := lib.sym("overdrive_insert")
	if err != nil {
		return "", err
	}
	jsonBytes, err := json.Marshal(doc)
	if err != nil {
		return "", err
	}
	ctable, keepT := cstring(table)
	cjson, keepJ := cstring(string(jsonBytes))
	defer keepT()
	defer keepJ()
	ptr := callFn3(sym, db.handle, ctable, cjson)
	if ptr == 0 {
		return "", checkErrorStructured(lib)
	}
	return readAndFree(lib, ptr), nil
}

// Get retrieves a document by _id. Returns nil if not found.
func (db *DB) Get(table, id string) (map[string]any, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_get")
	if err != nil {
		return nil, err
	}
	ctable, keepT := cstring(table)
	cid, keepI := cstring(id)
	defer keepT()
	defer keepI()
	ptr := callFn3(sym, db.handle, ctable, cid)
	if ptr == 0 {
		return nil, nil
	}
	s := readAndFree(lib, ptr)
	var doc map[string]any
	if err := json.Unmarshal([]byte(s), &doc); err != nil {
		return nil, err
	}
	return doc, nil
}

// Update updates fields of a document by _id. Returns true if updated.
func (db *DB) Update(table, id string, updates map[string]any) (bool, error) {
	lib, err := getLib()
	if err != nil {
		return false, err
	}
	sym, err := lib.sym("overdrive_update")
	if err != nil {
		return false, err
	}
	jsonBytes, err := json.Marshal(updates)
	if err != nil {
		return false, err
	}
	ctable, keepT := cstring(table)
	cid, keepI := cstring(id)
	cjson, keepJ := cstring(string(jsonBytes))
	defer keepT()
	defer keepI()
	defer keepJ()
	result := int64(callFn4(sym, db.handle, ctable, cid, cjson))
	if result == -1 {
		return false, checkErrorStructured(lib)
	}
	return result == 1, nil
}

// Delete removes a document by _id. Returns true if deleted.
func (db *DB) Delete(table, id string) (bool, error) {
	lib, err := getLib()
	if err != nil {
		return false, err
	}
	sym, err := lib.sym("overdrive_delete")
	if err != nil {
		return false, err
	}
	ctable, keepT := cstring(table)
	cid, keepI := cstring(id)
	defer keepT()
	defer keepI()
	result := int64(callFn3(sym, db.handle, ctable, cid))
	if result == -1 {
		return false, checkErrorStructured(lib)
	}
	return result == 1, nil
}

// Count returns the number of documents in a table.
func (db *DB) Count(table string) (int, error) {
	lib, err := getLib()
	if err != nil {
		return 0, err
	}
	sym, err := lib.sym("overdrive_count")
	if err != nil {
		return 0, err
	}
	ctable, keep := cstring(table)
	defer keep()
	result := int64(callFn2(sym, db.handle, ctable))
	if result == -1 {
		return 0, checkErrorStructured(lib)
	}
	return int(result), nil
}

// ─────────────────────────────────────────────
// Query
// ─────────────────────────────────────────────

// Query executes an SQL query and returns result rows.
func (db *DB) Query(sql string) (*QueryResult, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_query")
	if err != nil {
		return nil, err
	}
	csql, keep := cstring(sql)
	defer keep()
	ptr := callFn2(sym, db.handle, csql)
	if ptr == 0 {
		return nil, checkErrorStructured(lib)
	}
	s := readAndFree(lib, ptr)
	var qr QueryResult
	if err := json.Unmarshal([]byte(s), &qr); err != nil {
		return nil, err
	}
	return &qr, nil
}

// Search performs full-text search across a table.
func (db *DB) Search(table, text string) ([]map[string]any, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_search")
	if err != nil {
		return nil, err
	}
	ctable, keepT := cstring(table)
	ctext, keepTx := cstring(text)
	defer keepT()
	defer keepTx()
	ptr := callFn3(sym, db.handle, ctable, ctext)
	if ptr == 0 {
		return nil, nil
	}
	s := readAndFree(lib, ptr)
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

// ─────────────────────────────────────────────
// RAM Engine Methods
// ─────────────────────────────────────────────

// Snapshot persists the current RAM database to a snapshot file.
func (db *DB) Snapshot(path string) error {
	lib, err := getLib()
	if err != nil {
		return err
	}
	sym, err := lib.sym("overdrive_snapshot")
	if err != nil {
		return err
	}
	cpath, keep := cstring(path)
	defer keep()
	if callFn2(sym, db.handle, cpath) != 0 {
		return checkErrorStructured(lib)
	}
	return nil
}

// Restore loads a previously saved snapshot into the current database handle.
func (db *DB) Restore(path string) error {
	lib, err := getLib()
	if err != nil {
		return err
	}
	sym, err := lib.sym("overdrive_restore")
	if err != nil {
		return err
	}
	cpath, keep := cstring(path)
	defer keep()
	if callFn2(sym, db.handle, cpath) != 0 {
		return checkErrorStructured(lib)
	}
	return nil
}

// MemoryUsageStats returns current RAM consumption statistics.
func (db *DB) MemoryUsageStats() (*MemoryUsage, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_memory_usage")
	if err != nil {
		return nil, err
	}
	ptr := callFn1(sym, db.handle)
	if ptr == 0 {
		return nil, checkErrorStructured(lib)
	}
	s := readAndFree(lib, ptr)
	var usage MemoryUsage
	if err := json.Unmarshal([]byte(s), &usage); err != nil {
		return nil, err
	}
	return &usage, nil
}

// ─────────────────────────────────────────────
// Watchdog
// ─────────────────────────────────────────────

// Watchdog inspects a .odb file for integrity, size, and modification status.
// Static function — does not require an open database handle.
//
//	report, err := overdrive.Watchdog("app.odb")
//	if report.IntegrityStatus == "corrupted" {
//	    log.Printf("Corrupted: %s", *report.CorruptionDetails)
//	}
func Watchdog(filePath string) (*WatchdogReport, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_watchdog")
	if err != nil {
		return nil, err
	}
	cpath, keep := cstring(filePath)
	defer keep()
	ptr := callFn1(sym, cpath)
	if ptr == 0 {
		return nil, fmt.Errorf("watchdog() returned NULL for path: %s", filePath)
	}
	s := readAndFree(lib, ptr)
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

// ─────────────────────────────────────────────
// MVCC Transactions
// ─────────────────────────────────────────────

// BeginTransaction starts a new MVCC transaction.
func (db *DB) BeginTransaction(isolation IsolationLevel) (*TransactionHandle, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_begin_transaction")
	if err != nil {
		return nil, err
	}
	txnID := callFn1i2(sym, db.handle, int32(isolation))
	if txnID == 0 {
		return nil, checkErrorStructured(lib)
	}
	return &TransactionHandle{
		TxnID:     uint64(txnID),
		Isolation: isolation,
		Active:    true,
	}, nil
}

// CommitTransaction commits a transaction, making all changes permanent.
func (db *DB) CommitTransaction(txn *TransactionHandle) error {
	lib, err := getLib()
	if err != nil {
		return err
	}
	sym, err := lib.sym("overdrive_commit_transaction")
	if err != nil {
		return err
	}
	if callFn1u64(sym, db.handle, txn.TxnID) != 0 {
		return checkErrorStructured(lib)
	}
	txn.Active = false
	return nil
}

// AbortTransaction aborts a transaction, discarding all changes.
func (db *DB) AbortTransaction(txn *TransactionHandle) error {
	lib, err := getLib()
	if err != nil {
		return err
	}
	sym, err := lib.sym("overdrive_abort_transaction")
	if err != nil {
		return err
	}
	if callFn1u64(sym, db.handle, txn.TxnID) != 0 {
		return checkErrorStructured(lib)
	}
	txn.Active = false
	return nil
}

// Transaction executes a function inside a transaction with automatic commit/rollback.
//
//	err := db.Transaction(func(txn *overdrive.TransactionHandle) error {
//	    db.Insert("users", map[string]any{"name": "Alice"})
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

// ─────────────────────────────────────────────
// Helper Methods
// ─────────────────────────────────────────────

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
	var parts []string
	parts = append(parts, "SELECT * FROM "+table)
	if where != "" {
		parts = append(parts, "WHERE "+where)
	}
	if orderBy != "" {
		parts = append(parts, "ORDER BY "+orderBy)
	}
	if limit > 0 {
		parts = append(parts, fmt.Sprintf("LIMIT %d", limit))
	}
	qr, err := db.Query(strings.Join(parts, " "))
	if err != nil {
		return nil, err
	}
	return qr.Rows, nil
}

// CountWhere counts documents matching a WHERE clause.
func (db *DB) CountWhere(table, where string) (int, error) {
	var sql string
	if where != "" {
		sql = fmt.Sprintf("SELECT COUNT(*) FROM %s WHERE %s", table, where)
	} else {
		sql = fmt.Sprintf("SELECT COUNT(*) FROM %s", table)
	}
	qr, err := db.Query(sql)
	if err != nil {
		return 0, err
	}
	if len(qr.Rows) > 0 {
		for _, v := range qr.Rows[0] {
			switch n := v.(type) {
			case float64:
				return int(n), nil
			case int:
				return n, nil
			}
		}
	}
	return 0, nil
}

// Exists returns true if the document with the given _id exists.
func (db *DB) Exists(table, id string) (bool, error) {
	doc, err := db.Get(table, id)
	if err != nil {
		return false, err
	}
	return doc != nil, nil
}

// UpdateMany updates all documents matching a WHERE clause with the given updates.
// Returns the count of updated documents.
func (db *DB) UpdateMany(table, where string, updates map[string]any) (int, error) {
	jsonBytes, err := json.Marshal(updates)
	if err != nil {
		return 0, err
	}
	sql := fmt.Sprintf("UPDATE %s SET %s WHERE %s", table, string(jsonBytes), where)
	qr, err := db.Query(sql)
	if err != nil {
		return 0, err
	}
	return qr.RowsAffected, nil
}

// DeleteMany deletes all documents matching a WHERE clause.
// Returns the count of deleted documents.
func (db *DB) DeleteMany(table, where string) (int, error) {
	sql := fmt.Sprintf("DELETE FROM %s WHERE %s", table, where)
	qr, err := db.Query(sql)
	if err != nil {
		return 0, err
	}
	return qr.RowsAffected, nil
}

// VerifyIntegrity verifies database integrity (B-Tree, page checksums, MVCC chains).
func (db *DB) VerifyIntegrity() (map[string]any, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_verify_integrity")
	if err != nil {
		return nil, err
	}
	ptr := callFn1(sym, db.handle)
	if ptr == 0 {
		return nil, checkErrorStructured(lib)
	}
	s := readAndFree(lib, ptr)
	var result map[string]any
	if err := json.Unmarshal([]byte(s), &result); err != nil {
		return nil, err
	}
	return result, nil
}

// Stats returns extended database statistics.
func (db *DB) Stats() (map[string]any, error) {
	lib, err := getLib()
	if err != nil {
		return nil, err
	}
	sym, err := lib.sym("overdrive_stats")
	if err != nil {
		return nil, err
	}
	ptr := callFn1(sym, db.handle)
	if ptr == 0 {
		return nil, checkErrorStructured(lib)
	}
	s := readAndFree(lib, ptr)
	var result map[string]any
	if err := json.Unmarshal([]byte(s), &result); err != nil {
		return nil, err
	}
	return result, nil
}

// ─────────────────────────────────────────────
// Security
// ─────────────────────────────────────────────

// setSecurePermissions sets restrictive permissions on the database file.
func setSecurePermissions(path string) {
	if runtime.GOOS != "windows" {
		_ = os.Chmod(path, 0600)
	}
	// Windows: file is opened with exclusive access by default
}

// ─────────────────────────────────────────────
// Platform call stubs — implemented in platform files
// ─────────────────────────────────────────────

// These wrapper functions call native symbols via platform-specific mechanism.
// Implementations are in overdrive_windows.go and overdrive_unix.go.
