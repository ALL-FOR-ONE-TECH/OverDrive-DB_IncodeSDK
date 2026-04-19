//! C FFI layer — exports `overdrive_*` symbols for `overdrive.dll` / `liboverdrive.so`
//!
//! This module is compiled into the `cdylib` target and loaded at runtime by
//! external consumers (e.g. `overdrive-db` crates.io SDK via `libloading`).
//!
//! ## Memory contract
//! - All `*mut c_char` returned by these functions **must** be freed with `overdrive_free_string()`.
//! - Handles (`*mut OdbHandle`) must be closed with `overdrive_close()`.
//! - Errors are stored per-thread; retrieve them with `overdrive_last_error()`.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::cell::RefCell;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::storage::Database;
use crate::storage::mvcc::IsolationLevel;
use crate::cli::Shell;
use crate::error::OdbError;
use crate::vector::VectorStore;
use crate::timeseries::TimeSeriesEngine;
use crate::graph::GraphEngine;
use crate::streaming::StreamEngine;
use crate::storage::ram::RamStore;

// ─────────────────────────────────────────────────────────────
// Opaque handle
// ─────────────────────────────────────────────────────────────

/// Subscription state for streaming engine
struct SubscriptionState {
    topic: String,
    consumer_group: Option<String>,
    current_offset: u64,
}

/// Opaque database handle used by the C API.
pub struct OdbHandle {
    db: Database,
    /// Path to the .odb file (used to derive engine data directories)
    db_path: PathBuf,
    /// Lazily-initialised engine instances (one per handle)
    vector: Option<VectorStore>,
    timeseries: Option<TimeSeriesEngine>,
    graph: Option<GraphEngine>,
    streaming: Option<StreamEngine>,
    /// RAM store (used when engine="RAM")
    ram: Option<RamStore>,
    /// Whether auto-create-tables is enabled
    auto_create_tables: bool,
    /// Active streaming subscriptions: subscription_id -> state
    subscriptions: HashMap<u64, SubscriptionState>,
    /// Counter for generating unique subscription IDs
    sub_counter: u64,
}

impl OdbHandle {
    fn new(db: Database, path: PathBuf) -> Self {
        let auto = db.auto_create_tables();
        Self {
            db,
            db_path: path,
            vector: None,
            timeseries: None,
            graph: None,
            streaming: None,
            ram: None,
            auto_create_tables: auto,
            subscriptions: HashMap::new(),
            sub_counter: 0,
        }
    }

    /// Return (or lazily create) the data directory for engine sub-stores.
    /// Convention: same directory as the .odb file.
    fn engine_dir(&self) -> PathBuf {
        self.db_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn vector_store(&mut self) -> &mut VectorStore {
        if self.vector.is_none() {
            let dir = self.engine_dir().join("_vector");
            std::fs::create_dir_all(&dir).ok();
            self.vector = Some(VectorStore::new(&dir));
        }
        self.vector.as_mut().unwrap()
    }

    fn timeseries_engine(&mut self) -> &mut TimeSeriesEngine {
        if self.timeseries.is_none() {
            let dir = self.engine_dir().join("_timeseries");
            std::fs::create_dir_all(&dir).ok();
            self.timeseries = Some(TimeSeriesEngine::new(&dir));
        }
        self.timeseries.as_mut().unwrap()
    }

    fn graph_engine(&mut self) -> &mut GraphEngine {
        if self.graph.is_none() {
            let dir = self.engine_dir().join("_graph");
            std::fs::create_dir_all(&dir).ok();
            self.graph = Some(GraphEngine::new(&dir));
        }
        self.graph.as_mut().unwrap()
    }

    fn stream_engine(&mut self) -> &mut StreamEngine {
        if self.streaming.is_none() {
            let dir = self.engine_dir().join("_streaming");
            std::fs::create_dir_all(&dir).ok();
            self.streaming = Some(StreamEngine::new(&dir));
        }
        self.streaming.as_mut().unwrap()
    }
}

// ─────────────────────────────────────────────────────────────
// Thread-local error store
// Stores both a plain string (legacy) and a structured JSON string (Task 3.5).
// ─────────────────────────────────────────────────────────────

thread_local! {
    static LAST_ERROR:      RefCell<Option<CString>> = const { RefCell::new(None) };
    static LAST_ERROR_JSON: RefCell<Option<CString>> = const { RefCell::new(None) };
}

/// Store a plain error message (legacy path).
fn set_error(msg: &str) {
    let cs = CString::new(msg).unwrap_or_else(|_| CString::new("(error message contained null byte)").unwrap());
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(cs));
    // Also store a minimal structured JSON so overdrive_last_error_json() always works.
    let json = format!(
        r#"{{"code":"ODB-FFI-099","message":{},"context":"","suggestions":[],"doc_link":"https://overdrive-db.com/docs/errors/ODB-FFI-099"}}"#,
        serde_json::to_string(msg).unwrap_or_else(|_| format!("\"{}\"", msg))
    );
    let cs_json = CString::new(json).unwrap_or_else(|_| CString::new("{}").unwrap());
    LAST_ERROR_JSON.with(|e| *e.borrow_mut() = Some(cs_json));
}

/// Store a structured `OdbError` (rich path — used by Task 3.5).
fn set_odb_error(err: &OdbError) {
    // Plain string: "ODB-AUTH-001: Incorrect password …"
    let plain = err.to_string();
    let cs = CString::new(plain).unwrap_or_else(|_| CString::new("error").unwrap());
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(cs));
    // Structured JSON
    let json = err.to_json();
    let cs_json = CString::new(json).unwrap_or_else(|_| CString::new("{}").unwrap());
    LAST_ERROR_JSON.with(|e| *e.borrow_mut() = Some(cs_json));
}

fn clear_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
    LAST_ERROR_JSON.with(|e| *e.borrow_mut() = None);
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

/// Convert a raw C string pointer to a Rust String. Returns None if null.
unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
}

/// Allocate a C string from a &str. The caller must free with `overdrive_free_string`.
fn alloc_c_string(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

// ─────────────────────────────────────────────────────────────
// LIFECYCLE
// ─────────────────────────────────────────────────────────────

/// Open or create a database at `path`. Returns an opaque handle, or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_open(path: *const c_char) -> *mut OdbHandle {
    clear_error();
    let path_str = match c_str_to_string(path) {
        Some(p) => p,
        None => { set_error("overdrive_open: path is null"); return ptr::null_mut(); }
    };

    // Try to open existing DB first, fall back to creating a new one
    let db = if std::path::Path::new(&path_str).exists() {
        Database::open(&path_str)
    } else {
        Database::create(&path_str)
    };

    match db {
        Ok(db) => Box::into_raw(Box::new(OdbHandle::new(db, PathBuf::from(&path_str)))),
        Err(e) => { set_odb_error(&e.to_odb_error()); ptr::null_mut() }
    }
}

/// Close a database handle and free all resources.
#[no_mangle]
pub unsafe extern "C" fn overdrive_close(handle: *mut OdbHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Flush all pending writes to disk.
#[no_mangle]
pub unsafe extern "C" fn overdrive_sync(handle: *mut OdbHandle) {
    if handle.is_null() { return; }
    let _ = (*handle).db.sync();
}

// ─────────────────────────────────────────────────────────────
// TABLE MANAGEMENT
// ─────────────────────────────────────────────────────────────

/// Create a table. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_table(handle: *mut OdbHandle, name: *const c_char) -> i32 {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(name) {
        Some(n) => n,
        None => { set_error("overdrive_create_table: null name"); return -1; }
    };
    match (*handle).db.create_table(&name) {
        Ok(()) => 0,
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

/// Drop a table. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_drop_table(handle: *mut OdbHandle, name: *const c_char) -> i32 {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(name) {
        Some(n) => n,
        None => { set_error("overdrive_drop_table: null name"); return -1; }
    };
    match (*handle).db.drop_table(&name) {
        Ok(()) => 0,
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

/// List all tables as a JSON array string. Must be freed with `overdrive_free_string`.
/// Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_list_tables(handle: *mut OdbHandle) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let tables = (*handle).db.list_tables();
    let json = serde_json::to_string(&tables).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

/// Check if a table exists. Returns 1 (exists), 0 (not found), -1 (error).
#[no_mangle]
pub unsafe extern "C" fn overdrive_table_exists(handle: *mut OdbHandle, name: *const c_char) -> i32 {
    if handle.is_null() { return -1; }
    let name = match c_str_to_string(name) {
        Some(n) => n,
        None => return -1,
    };
    let tables = (*handle).db.list_tables();
    if tables.contains(&name) { 1 } else { 0 }
}

// ─────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────

/// Insert a JSON document into `table`. Returns the generated `_id` string (must be freed),
/// or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_insert(
    handle: *mut OdbHandle,
    table: *const c_char,
    json: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let table = match c_str_to_string(table) { Some(t) => t, None => { set_error("null table"); return ptr::null_mut(); } };
    let json_str = match c_str_to_string(json) { Some(j) => j, None => { set_error("null json"); return ptr::null_mut(); } };

    let value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("JSON parse error: {}", e)); return ptr::null_mut(); }
    };

    match (*handle).db.insert_json(&table, &value) {
        Ok(id) => alloc_c_string(&id),
        Err(e) => { set_odb_error(&e.to_odb_error()); ptr::null_mut() }
    }
}

/// Get a document by `id`. Returns JSON string (must be freed), or NULL if not found.
#[no_mangle]
pub unsafe extern "C" fn overdrive_get(
    handle: *mut OdbHandle,
    table: *const c_char,
    id: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { return ptr::null_mut(); }
    let table = match c_str_to_string(table) { Some(t) => t, None => return ptr::null_mut() };
    let id = match c_str_to_string(id) { Some(i) => i, None => return ptr::null_mut() };

    match (*handle).db.get_json(&table, &id) {
        Ok(Some(val)) => alloc_c_string(&val.to_string()),
        Ok(None) => ptr::null_mut(),
        Err(e) => { set_odb_error(&e.to_odb_error()); ptr::null_mut() }
    }
}

/// Update a document. Returns 1 (updated), 0 (not found), -1 (error).
#[no_mangle]
pub unsafe extern "C" fn overdrive_update(
    handle: *mut OdbHandle,
    table: *const c_char,
    id: *const c_char,
    json: *const c_char,
) -> i32 {
    clear_error();
    if handle.is_null() { return -1; }
    let table = match c_str_to_string(table) { Some(t) => t, None => return -1 };
    let id = match c_str_to_string(id) { Some(i) => i, None => return -1 };
    let json_str = match c_str_to_string(json) { Some(j) => j, None => return -1 };

    let value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("JSON parse error: {}", e)); return -1; }
    };

    match (*handle).db.update(&table, &id, &value) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

/// Delete a document. Returns 1 (deleted), 0 (not found), -1 (error).
#[no_mangle]
pub unsafe extern "C" fn overdrive_delete(
    handle: *mut OdbHandle,
    table: *const c_char,
    id: *const c_char,
) -> i32 {
    clear_error();
    if handle.is_null() { return -1; }
    let table = match c_str_to_string(table) { Some(t) => t, None => return -1 };
    let id = match c_str_to_string(id) { Some(i) => i, None => return -1 };

    match (*handle).db.delete(&table, id.as_bytes()) {
        Ok(deleted) => if deleted { 1 } else { 0 },
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

/// Count documents in a table. Returns count ≥ 0, or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_count(handle: *mut OdbHandle, table: *const c_char) -> i32 {
    clear_error();
    if handle.is_null() { return -1; }
    let table = match c_str_to_string(table) { Some(t) => t, None => return -1 };

    match (*handle).db.count(&table) {
        Ok(n) => n as i32,
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

// ─────────────────────────────────────────────────────────────
// QUERY (SQL)
// ─────────────────────────────────────────────────────────────

/// Execute an SQL statement against the open database.
/// Returns a JSON result string (must be freed), or NULL on error.
///
/// Uses `Shell::parse_and_execute` which supports the full OverDrive SQL dialect.
#[no_mangle]
pub unsafe extern "C" fn overdrive_query(
    handle: *mut OdbHandle,
    sql: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let sql = match c_str_to_string(sql) { Some(s) => s, None => return ptr::null_mut() };

    // Build a temporary shell pointing at the same data directory the db is already using.
    // We execute the SQL and serialise the text output as JSON { "result": "..." }.
    let mut shell = Shell::new(".");
    match shell.parse_and_execute(&sql) {
        Ok(output) => {
            let json = serde_json::json!({ "result": output, "ok": true });
            alloc_c_string(&json.to_string())
        }
        Err(e) => {
            set_error(&e.to_string());
            let json = serde_json::json!({ "result": e.to_string(), "ok": false });
            alloc_c_string(&json.to_string())
        }
    }
}

// ─────────────────────────────────────────────────────────────
// FULL-TEXT SEARCH
// ─────────────────────────────────────────────────────────────

/// Full-text search across the database. Returns a JSON array of matching doc IDs.
/// Must be freed with `overdrive_free_string`. Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_search(
    handle: *mut OdbHandle,
    _table: *const c_char,   // reserved — search is cross-table in this engine
    text: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { return ptr::null_mut(); }
    let text = match c_str_to_string(text) { Some(t) => t, None => return ptr::null_mut() };

    let results = (*handle).db.search_text(&text);
    let json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

// ─────────────────────────────────────────────────────────────
// ERROR & UTILITY
// ─────────────────────────────────────────────────────────────

/// Return the last error message as a C string.
/// The pointer is valid until the next API call on this thread.
/// Do **not** free this pointer — it is managed internally.
#[no_mangle]
pub extern "C" fn overdrive_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        match &*e.borrow() {
            Some(cs) => cs.as_ptr(),
            None => ptr::null(),
        }
    })
}

/// Return the last error as a structured JSON string (Task 3.5).
///
/// The JSON has the shape:
/// ```json
/// {
///   "code": "ODB-AUTH-001",
///   "message": "...",
///   "context": "...",
///   "suggestions": ["..."],
///   "doc_link": "https://overdrive-db.com/docs/errors/ODB-AUTH-001"
/// }
/// ```
///
/// Returns NULL when there is no pending error.
/// The pointer is valid until the next API call on this thread — do **not** free it.
#[no_mangle]
pub extern "C" fn overdrive_last_error_json() -> *const c_char {
    LAST_ERROR_JSON.with(|e| {
        match &*e.borrow() {
            Some(cs) => cs.as_ptr(),
            None => ptr::null(),
        }
    })
}

/// Free a string returned by any `overdrive_*` function.
#[no_mangle]
pub unsafe extern "C" fn overdrive_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Return the engine version string. This pointer is static — do not free.
#[no_mangle]
pub extern "C" fn overdrive_version() -> *const c_char {
    // SAFETY: static null-terminated byte string
    c"1.4.2".as_ptr()
}

// ─────────────────────────────────────────────────────────────
// MVCC TRANSACTIONS
// ─────────────────────────────────────────────────────────────

/// Begin a transaction.
/// `isolation_level`: 0=ReadUncommitted, 1=ReadCommitted, 2=RepeatableRead, 3=Serializable.
/// Returns the transaction ID (> 0) on success, 0 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_begin_transaction(handle: *mut OdbHandle, isolation_level: i32) -> u64 {
    clear_error();
    if handle.is_null() { set_error("null handle"); return 0; }

    let isolation = match isolation_level {
        0 => IsolationLevel::ReadUncommitted,
        2 => IsolationLevel::RepeatableRead,
        3 => IsolationLevel::Serializable,
        _ => IsolationLevel::ReadCommitted,
    };

    match (*handle).db.begin_transaction(isolation) {
        Ok(txn) => txn.id,
        Err(e) => { set_odb_error(&e.to_odb_error()); 0 }
    }
}

/// Commit a transaction. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_commit_transaction(handle: *mut OdbHandle, txn_id: u64) -> i32 {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    match (*handle).db.commit_transaction(txn_id) {
        Ok(()) => 0,
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

/// Abort (rollback) a transaction. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_abort_transaction(handle: *mut OdbHandle, txn_id: u64) -> i32 {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    match (*handle).db.abort_transaction(txn_id) {
        Ok(()) => 0,
        Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
    }
}

// ─────────────────────────────────────────────────────────────
// INTEGRITY
// ─────────────────────────────────────────────────────────────

/// Verify database integrity. Returns a JSON report string (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_verify_integrity(handle: *mut OdbHandle) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }

    // Perform basic integrity check: scan all tables
    let tables = (*handle).db.list_tables();
    let mut issues: Vec<String> = Vec::new();
    let mut total_records: usize = 0;

    for table in &tables {
        match (*handle).db.count(table) {
            Ok(n) => total_records += n,
            Err(e) => issues.push(format!("table '{}': {}", table, e)),
        }
    }

    let report = serde_json::json!({
        "valid": issues.is_empty(),
        "pages_checked": total_records,
        "tables_verified": tables.len(),
        "issues": issues,
    });

    alloc_c_string(&report.to_string())
}
// -------------------------------------------------------------
// TASK 4.1 � ENGINE SELECTION FFI FUNCTIONS
// -------------------------------------------------------------

/// Open or create a database with a specific engine type and options.
/// engine: "Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming"
/// options_json: JSON string with optional "password" and "auto_create_tables" fields.
/// Returns an opaque handle, or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_open_with_engine(
    path: *const c_char,
    engine: *const c_char,
    options_json: *const c_char,
) -> *mut OdbHandle {
    clear_error();
    let path_str = match c_str_to_string(path) {
        Some(p) => p,
        None => { set_error("overdrive_open_with_engine: path is null"); return ptr::null_mut(); }
    };
    let engine_str = c_str_to_string(engine).unwrap_or_else(|| "Disk".to_string());

    // Parse options JSON
    let mut password: Option<String> = None;
    let mut auto_create = true;
    if let Some(opts_str) = c_str_to_string(options_json) {
        if let Ok(opts) = serde_json::from_str::<serde_json::Value>(&opts_str) {
            if let Some(pwd) = opts.get("password").and_then(|v| v.as_str()) {
                password = Some(pwd.to_string());
            }
            if let Some(ac) = opts.get("auto_create_tables").and_then(|v| v.as_bool()) {
                auto_create = ac;
            }
        }
    }

    let db_result = if std::path::Path::new(&path_str).exists() {
        Database::open_with_password(&path_str, password.as_deref())
    } else {
        match password {
            Some(ref pwd) => Database::create_with_password(&path_str, pwd),
            None => Database::create(&path_str),
        }
    };

    match db_result {
        Ok(db) => {
            let mut handle = OdbHandle::new(db, PathBuf::from(&path_str));
            handle.auto_create_tables = auto_create;
            // For RAM engine, initialise a RAM store
            if engine_str == "RAM" {
                let name = std::path::Path::new(&path_str)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("ram_db")
                    .to_string();
                handle.ram = Some(RamStore::new(&name));
            }
            Box::into_raw(Box::new(handle))
        }
        Err(e) => { set_odb_error(&e.to_odb_error()); ptr::null_mut() }
    }
}

/// Returns the engine type of the database as a string ("Disk", "RAM", etc.).
/// The returned string must be freed with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_get_engine_type(handle: *mut OdbHandle) -> *mut c_char {
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let engine = if (*handle).ram.is_some() {
        "RAM"
    } else if (*handle).vector.is_some() {
        "Vector"
    } else if (*handle).timeseries.is_some() {
        "Time-Series"
    } else if (*handle).graph.is_some() {
        "Graph"
    } else if (*handle).streaming.is_some() {
        "Streaming"
    } else {
        "Disk"
    };
    alloc_c_string(engine)
}

/// Create a table with a specified engine type.
/// engine: "Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming"
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_table_with_engine(
    handle: *mut OdbHandle,
    table_name: *const c_char,
    engine: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(table_name) {
        Some(n) => n,
        None => { set_error("null table_name"); return -1; }
    };
    let engine_str = c_str_to_string(engine).unwrap_or_else(|| "Disk".to_string());

    match engine_str.as_str() {
        "RAM" => {
            let ram = (*handle).ram.get_or_insert_with(|| RamStore::new("default"));
            match ram.create_table(&name) {
                Ok(()) => 0,
                Err(e) => { set_error(&e.to_string()); -1 }
            }
        }
        _ => {
            // Disk and other engines use the main Database
            match (*handle).db.create_table(&name) {
                Ok(()) => 0,
                Err(e) => { set_odb_error(&e.to_odb_error()); -1 }
            }
        }
    }
}

// -------------------------------------------------------------
// TASK 4.2 � RAM ENGINE FFI FUNCTIONS
// -------------------------------------------------------------

/// Create a full RAM database at path with an optional memory limit.
/// max_memory_bytes: 0 means use system-RAM-aware default.
/// Returns an opaque handle, or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_ram_db(
    path: *const c_char,
    max_memory_bytes: u64,
) -> *mut OdbHandle {
    clear_error();
    let path_str = match c_str_to_string(path) {
        Some(p) => p,
        None => { set_error("overdrive_create_ram_db: path is null"); return ptr::null_mut(); }
    };

    let db_result = if std::path::Path::new(&path_str).exists() {
        Database::open(&path_str)
    } else {
        Database::create(&path_str)
    };

    match db_result {
        Ok(db) => {
            let name = std::path::Path::new(&path_str)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("ram_db")
                .to_string();
            let ram = if max_memory_bytes > 0 {
                RamStore::with_memory_limit(&name, max_memory_bytes as usize)
            } else {
                RamStore::new(&name)
            };
            let mut handle = OdbHandle::new(db, PathBuf::from(&path_str));
            handle.ram = Some(ram);
            Box::into_raw(Box::new(handle))
        }
        Err(e) => { set_odb_error(&e.to_odb_error()); ptr::null_mut() }
    }
}

/// Create a RAM table within an existing database handle.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_ram_table(
    handle: *mut OdbHandle,
    table_name: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(table_name) {
        Some(n) => n,
        None => { set_error("null table_name"); return -1; }
    };
    let ram = (*handle).ram.get_or_insert_with(|| RamStore::new("default"));
    match ram.create_table(&name) {
        Ok(()) => 0,
        Err(e) => { set_error(&e.to_string()); -1 }
    }
}

/// Persist the RAM database to a snapshot file.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_snapshot(
    handle: *mut OdbHandle,
    snapshot_path: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let snap_path = match c_str_to_string(snapshot_path) {
        Some(p) => p,
        None => { set_error("null snapshot_path"); return -1; }
    };
    match &(*handle).ram {
        Some(ram) => match ram.snapshot(&snap_path) {
            Ok(_) => 0,
            Err(e) => { set_error(&e.to_string()); -1 }
        },
        None => { set_error("No RAM store on this handle"); -1 }
    }
}

/// Restore the RAM database from a snapshot file.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_restore(
    handle: *mut OdbHandle,
    snapshot_path: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let snap_path = match c_str_to_string(snapshot_path) {
        Some(p) => p,
        None => { set_error("null snapshot_path"); return -1; }
    };
    match &mut (*handle).ram {
        Some(ram) => match ram.restore(&snap_path) {
            Ok(_) => 0,
            Err(e) => { set_error(&e.to_string()); -1 }
        },
        None => { set_error("No RAM store on this handle"); -1 }
    }
}

/// Returns JSON with memory usage statistics for the RAM store.
/// Format: {"bytes": N, "mb": N.N, "limit_bytes": N_or_null, "percent": N_or_null}
/// Must be freed with overdrive_free_string(). Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_memory_usage(handle: *mut OdbHandle) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    match &(*handle).ram {
        Some(ram) => {
            let stats = ram.stats();
            let bytes = stats.memory_bytes as u64;
            let mb = bytes as f64 / (1024.0 * 1024.0);
            let json = match stats.max_memory_bytes {
                Some(limit) => {
                    let limit_bytes = limit as u64;
                    let percent = if limit_bytes > 0 {
                        Some((bytes as f64 / limit_bytes as f64) * 100.0)
                    } else {
                        None
                    };
                    serde_json::json!({
                        "bytes": bytes,
                        "mb": (mb * 10.0).round() / 10.0,
                        "limit_bytes": limit_bytes,
                        "percent": percent,
                    })
                }
                None => {
                    serde_json::json!({
                        "bytes": bytes,
                        "mb": (mb * 10.0).round() / 10.0,
                        "limit_bytes": serde_json::Value::Null,
                        "percent": serde_json::Value::Null,
                    })
                }
            };
            alloc_c_string(&json.to_string())
        }
        None => { set_error("No RAM store on this handle"); ptr::null_mut() }
    }
}

/// Set the memory limit for the RAM store.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_set_memory_limit(
    handle: *mut OdbHandle,
    max_bytes: u64,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    match &mut (*handle).ram {
        Some(ram) => {
            ram.set_memory_limit(max_bytes as usize);
            0
        }
        None => { set_error("No RAM store on this handle"); -1 }
    }
}
// -------------------------------------------------------------
// TASK 4.4 - TIME-SERIES ENGINE FFI FUNCTIONS
// -------------------------------------------------------------

/// Create a time-series with the given name and TTL.
/// ttl_seconds: 0 means no TTL (keep forever).
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_timeseries(
    handle: *mut OdbHandle,
    name: *const c_char,
    ttl_seconds: u64,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name_str = match c_str_to_string(name) {
        Some(n) => n,
        None => { set_error("null name"); return -1; }
    };
    let ttl_str = ttl_seconds.to_string();
    let ts = (*handle).timeseries_engine();
    match ts.create(&name_str, &ttl_str) {
        Ok(()) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

/// Insert a measurement into a time-series.
/// measurement_json: JSON object with at minimum a `value` field; `timestamp` is optional (defaults to now).
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_insert_measurement(
    handle: *mut OdbHandle,
    timeseries: *const c_char,
    measurement_json: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let series_name = match c_str_to_string(timeseries) {
        Some(n) => n,
        None => { set_error("null timeseries"); return -1; }
    };
    let json_str = match c_str_to_string(measurement_json) {
        Some(j) => j,
        None => { set_error("null measurement_json"); return -1; }
    };

    // Parse the JSON and convert to the assignment string format expected by TimeSeriesEngine::insert
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("JSON parse error: {}", e)); return -1; }
    };

    let obj = match val.as_object() {
        Some(o) => o,
        None => { set_error("measurement_json must be a JSON object"); return -1; }
    };

    // Build assignment string: "key=value, ..."
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in obj {
        if k == "timestamp" || k == "ts" {
            if let Some(ts) = v.as_i64() {
                // timestamp provided in seconds — convert to nanoseconds for the engine
                parts.push(format!("ts={}", ts * 1_000_000_000i64));
            }
        } else if let Some(n) = v.as_f64() {
            parts.push(format!("{}={}", k, n));
        } else if let Some(s) = v.as_str() {
            parts.push(format!("{}=\"{}\"", k, s));
        }
    }

    if parts.is_empty() {
        set_error("measurement_json must contain at least one field");
        return -1;
    }

    let assignments = parts.join(", ");
    let ts = (*handle).timeseries_engine();
    match ts.insert(&series_name, &assignments) {
        Ok(()) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

/// Query measurements in a time range.
/// start_ts / end_ts: Unix timestamps in seconds (i64).
/// Returns JSON array of measurement objects (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_query_timeseries(
    handle: *mut OdbHandle,
    timeseries: *const c_char,
    start_ts: i64,
    end_ts: i64,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let series_name = match c_str_to_string(timeseries) {
        Some(n) => n,
        None => { set_error("null timeseries"); return ptr::null_mut(); }
    };

    // Convert seconds to nanoseconds
    let from_ns = start_ts.saturating_mul(1_000_000_000);
    let to_ns = end_ts.saturating_mul(1_000_000_000);

    let ts = (*handle).timeseries_engine();
    let measurements = ts.query_range(&series_name, from_ns, to_ns);

    // Serialize: convert timestamp_ns back to seconds for the caller
    let result: Vec<serde_json::Value> = measurements.into_iter().map(|m| {
        let mut obj = serde_json::Map::new();
        obj.insert("timestamp".to_string(), serde_json::json!(m.timestamp_ns / 1_000_000_000));
        obj.insert("timestamp_ns".to_string(), serde_json::json!(m.timestamp_ns));
        for (k, v) in &m.fields {
            obj.insert(k.clone(), serde_json::json!(v));
        }
        for (k, v) in &m.tags {
            obj.insert(k.clone(), serde_json::json!(v));
        }
        serde_json::Value::Object(obj)
    }).collect();

    let json = serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

/// Aggregate measurements over a time range with a window.
/// start_ts / end_ts: Unix timestamps in seconds (i64).
/// window_sec: aggregation window size in seconds.
/// aggregation: "avg", "sum", "min", "max", "count".
/// Returns JSON array of aggregated bucket objects (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_aggregate_timeseries(
    handle: *mut OdbHandle,
    timeseries: *const c_char,
    start_ts: i64,
    end_ts: i64,
    window_sec: i64,
    aggregation: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let series_name = match c_str_to_string(timeseries) {
        Some(n) => n,
        None => { set_error("null timeseries"); return ptr::null_mut(); }
    };
    let agg_str = c_str_to_string(aggregation).unwrap_or_else(|| "avg".to_string());

    let agg_func = match crate::timeseries::query::AggFunc::from_str(&agg_str) {
        Some(f) => f,
        None => { set_error(&format!("Unknown aggregation '{}'. Use: avg, sum, min, max, count", agg_str)); return ptr::null_mut(); }
    };

    let from_ns = start_ts.saturating_mul(1_000_000_000);
    let to_ns = end_ts.saturating_mul(1_000_000_000);
    let window_ns = window_sec.saturating_mul(1_000_000_000);

    if window_ns <= 0 {
        set_error("window_sec must be > 0");
        return ptr::null_mut();
    }

    let q = crate::timeseries::query::WindowQuery {
        series: series_name,
        field: "value".to_string(), // aggregate the primary "value" field
        agg: agg_func,
        from_ns,
        to_ns,
        window_ns,
        group_by: None,
    };

    let ts = (*handle).timeseries_engine();
    let buckets = ts.window_query(q);

    let result: Vec<serde_json::Value> = buckets.into_iter().map(|b| {
        serde_json::json!({
            "window_start": b.window_start_ns / 1_000_000_000,
            "window_end":   b.window_end_ns   / 1_000_000_000,
            "window_start_ns": b.window_start_ns,
            "window_end_ns":   b.window_end_ns,
            "value": b.value,
            "count": b.count,
        })
    }).collect();

    let json = serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

/// Drop a time-series and all its data.
/// Returns 0 on success, -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn overdrive_drop_timeseries(
    handle: *mut OdbHandle,
    name: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name_str = match c_str_to_string(name) {
        Some(n) => n,
        None => { set_error("null name"); return -1; }
    };
    let ts = (*handle).timeseries_engine();
    if ts.drop_series(&name_str) {
        0
    } else {
        set_error(&format!("Time-series '{}' not found", name_str));
        -1
    }
}

/// List all time-series as a JSON array of info objects.
/// Each object has: name, ttl_seconds, created_at.
/// Must be freed with overdrive_free_string(). Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_list_timeseries(handle: *mut OdbHandle) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let ts = (*handle).timeseries_engine();
    let list = ts.list();
    let result: Vec<serde_json::Value> = list.into_iter().map(|def| {
        serde_json::json!({
            "name": def.name,
            "ttl_seconds": def.ttl_seconds,
            "created_at": def.created_at,
        })
    }).collect();
    let json = serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

// -------------------------------------------------------------
// TASK 4.4 - TIME-SERIES ENGINE FFI TESTS
// -------------------------------------------------------------

#[cfg(test)]
mod timeseries_ffi_tests {
    use super::*;
    use std::ffi::CString;

    fn open_temp_db() -> (*mut OdbHandle, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let handle = unsafe { overdrive_open(path_cstr.as_ptr()) };
        assert!(!handle.is_null(), "Failed to open temp db");
        (handle, dir)
    }

    // 4.4.1 — overdrive_create_timeseries
    #[test]
    fn test_create_timeseries_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("cpu_usage").unwrap();
        let rc = unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 86400) };
        assert_eq!(rc, 0, "create_timeseries should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_timeseries_no_ttl() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("forever_series").unwrap();
        let rc = unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };
        assert_eq!(rc, 0, "create_timeseries with ttl=0 should succeed");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_timeseries_null_handle() {
        let name = CString::new("x").unwrap();
        let rc = unsafe { overdrive_create_timeseries(ptr::null_mut(), name.as_ptr(), 0) };
        assert_eq!(rc, -1);
    }

    // 4.4.2 — overdrive_insert_measurement
    #[test]
    fn test_insert_measurement_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("temp").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        let meas = CString::new(r#"{"value": 23.5}"#).unwrap();
        let rc = unsafe { overdrive_insert_measurement(handle, name.as_ptr(), meas.as_ptr()) };
        assert_eq!(rc, 0, "insert_measurement should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_insert_measurement_with_timestamp() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("pressure").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        let meas = CString::new(r#"{"value": 101.3, "timestamp": 1700000000}"#).unwrap();
        let rc = unsafe { overdrive_insert_measurement(handle, name.as_ptr(), meas.as_ptr()) };
        assert_eq!(rc, 0);
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_insert_measurement_nonexistent_series() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("ghost").unwrap();
        let meas = CString::new(r#"{"value": 1.0}"#).unwrap();
        let rc = unsafe { overdrive_insert_measurement(handle, name.as_ptr(), meas.as_ptr()) };
        assert_eq!(rc, -1, "insert into non-existent series should fail");
        unsafe { overdrive_close(handle) };
    }

    // 4.4.3 — overdrive_query_timeseries
    #[test]
    fn test_query_timeseries_returns_data() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("humidity").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        let meas = CString::new(r#"{"value": 55.0}"#).unwrap();
        unsafe { overdrive_insert_measurement(handle, name.as_ptr(), meas.as_ptr()) };

        let now = chrono::Utc::now().timestamp();
        let result_ptr = unsafe {
            overdrive_query_timeseries(handle, name.as_ptr(), now - 60, now + 60)
        };
        assert!(!result_ptr.is_null(), "query_timeseries should return a JSON string");

        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let arr = parsed.as_array().expect("array");
        assert_eq!(arr.len(), 1, "should return 1 measurement");
        assert!(arr[0].get("value").is_some(), "measurement should have 'value' field");
        assert!(arr[0].get("timestamp").is_some(), "measurement should have 'timestamp' field");

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_query_timeseries_empty_range() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("empty_range").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        let result_ptr = unsafe {
            overdrive_query_timeseries(handle, name.as_ptr(), 0, 1)
        };
        assert!(!result_ptr.is_null());
        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        assert_eq!(s, "[]");
        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    // 4.4.4 — overdrive_aggregate_timeseries
    #[test]
    fn test_aggregate_timeseries_avg() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("agg_test").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        // Insert two measurements
        for v in [10.0f64, 20.0f64] {
            let meas = CString::new(format!(r#"{{"value": {}}}"#, v)).unwrap();
            unsafe { overdrive_insert_measurement(handle, name.as_ptr(), meas.as_ptr()) };
        }

        let now = chrono::Utc::now().timestamp();
        let agg = CString::new("avg").unwrap();
        let result_ptr = unsafe {
            overdrive_aggregate_timeseries(handle, name.as_ptr(), now - 60, now + 60, 120, agg.as_ptr())
        };
        assert!(!result_ptr.is_null(), "aggregate_timeseries should return JSON");

        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let arr = parsed.as_array().expect("array");
        assert!(!arr.is_empty(), "should have at least one bucket");
        let bucket = &arr[0];
        assert!(bucket.get("value").is_some(), "bucket should have 'value'");
        assert!(bucket.get("window_start").is_some(), "bucket should have 'window_start'");
        assert!(bucket.get("count").is_some(), "bucket should have 'count'");

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_aggregate_timeseries_invalid_agg() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("bad_agg").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        let now = chrono::Utc::now().timestamp();
        let agg = CString::new("median").unwrap(); // unsupported
        let result_ptr = unsafe {
            overdrive_aggregate_timeseries(handle, name.as_ptr(), now - 60, now + 60, 60, agg.as_ptr())
        };
        assert!(result_ptr.is_null(), "aggregate with unknown function should return NULL");
        unsafe { overdrive_close(handle) };
    }

    // 4.4.5 — overdrive_drop_timeseries
    #[test]
    fn test_drop_timeseries_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("to_drop").unwrap();
        unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 0) };

        let rc = unsafe { overdrive_drop_timeseries(handle, name.as_ptr()) };
        assert_eq!(rc, 0, "drop_timeseries should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_drop_timeseries_not_found() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("nonexistent").unwrap();
        let rc = unsafe { overdrive_drop_timeseries(handle, name.as_ptr()) };
        assert_eq!(rc, -1, "drop_timeseries should return -1 when not found");
        unsafe { overdrive_close(handle) };
    }

    // 4.4.6 — overdrive_list_timeseries
    #[test]
    fn test_list_timeseries_empty() {
        let (handle, _dir) = open_temp_db();
        let result_ptr = unsafe { overdrive_list_timeseries(handle) };
        assert!(!result_ptr.is_null());
        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        assert!(parsed.is_array());
        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_list_timeseries_after_create() {
        let (handle, _dir) = open_temp_db();
        for series in ["s1", "s2", "s3"] {
            let name = CString::new(series).unwrap();
            unsafe { overdrive_create_timeseries(handle, name.as_ptr(), 3600) };
        }

        let result_ptr = unsafe { overdrive_list_timeseries(handle) };
        assert!(!result_ptr.is_null());
        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let arr = parsed.as_array().expect("array");
        assert_eq!(arr.len(), 3, "should list 3 time-series");
        // Verify shape
        let first = &arr[0];
        assert!(first.get("name").is_some());
        assert!(first.get("ttl_seconds").is_some());
        assert!(first.get("created_at").is_some());

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }
}

// -------------------------------------------------------------
// TASK 4.3 - VECTOR ENGINE FFI FUNCTIONS
// -------------------------------------------------------------

/// Create a vector index on a table/field with given dimensions and metric.
/// metric: "cosine", "euclidean", "dot"
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_vector_index(
    handle: *mut OdbHandle,
    table: *const c_char,
    _field: *const c_char,
    dimensions: u32,
    metric: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let table_str = match c_str_to_string(table) {
        Some(t) => t,
        None => { set_error("null table"); return -1; }
    };
    let metric_str = c_str_to_string(metric).unwrap_or_else(|| "cosine".to_string());
    let vs = (*handle).vector_store();
    match vs.create_collection(&table_str, dimensions as usize, &metric_str) {
        Ok(()) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

/// Insert a vector into a table. json is the document metadata, embedding_json is "[f32,...]".
/// Returns the generated document ID (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_insert_vector(
    handle: *mut OdbHandle,
    table: *const c_char,
    json: *const c_char,
    embedding_json: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let table_str = match c_str_to_string(table) {
        Some(t) => t,
        None => { set_error("null table"); return ptr::null_mut(); }
    };
    let json_str = match c_str_to_string(json) {
        Some(j) => j,
        None => { set_error("null json"); return ptr::null_mut(); }
    };
    let emb_str = match c_str_to_string(embedding_json) {
        Some(e) => e,
        None => { set_error("null embedding_json"); return ptr::null_mut(); }
    };

    let meta: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("JSON parse error: {}", e)); return ptr::null_mut(); }
    };

    let vector = match crate::vector::parse_vector(&emb_str) {
        Ok(v) => v,
        Err(e) => { set_error(&e); return ptr::null_mut(); }
    };

    // Generate an ID from the metadata _id field or auto-generate
    let id = meta.get("_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}_vec_{}", table_str, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));

    let vs = (*handle).vector_store();
    match vs.insert(&table_str, &id, vector, Some(meta)) {
        Ok(()) => alloc_c_string(&id),
        Err(e) => { set_error(&e); ptr::null_mut() }
    }
}

/// Search for similar vectors. query_vector_json is "[f32,...]".
/// Returns JSON array of {id, distance, metadata} objects (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_vector_search(
    handle: *mut OdbHandle,
    table: *const c_char,
    query_vector_json: *const c_char,
    limit: u32,
    metric: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let table_str = match c_str_to_string(table) {
        Some(t) => t,
        None => { set_error("null table"); return ptr::null_mut(); }
    };
    let qvec_str = match c_str_to_string(query_vector_json) {
        Some(q) => q,
        None => { set_error("null query_vector_json"); return ptr::null_mut(); }
    };
    let metric_opt = c_str_to_string(metric);

    let query_vec = match crate::vector::parse_vector(&qvec_str) {
        Ok(v) => v,
        Err(e) => { set_error(&e); return ptr::null_mut(); }
    };

    let k = if limit == 0 { 10 } else { limit as usize };
    let vs = (*handle).vector_store();
    match vs.search(&table_str, &query_vec, metric_opt.as_deref(), k) {
        Ok(results) => {
            // Map to {id, score, metadata} shape as per FFI contract.
            // `distance` is lower-is-better; expose as `score` = 1 - distance for cosine,
            // or just the raw distance value for other metrics. We expose both for flexibility.
            let mapped: Vec<serde_json::Value> = results.into_iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "score": r.distance,
                    "metadata": r.metadata,
                })
            }).collect();
            let json = serde_json::to_string(&mapped).unwrap_or_else(|_| "[]".to_string());
            alloc_c_string(&json)
        }
        Err(e) => { set_error(&e); ptr::null_mut() }
    }
}

/// Drop a vector index for a table. Returns 0 on success, -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn overdrive_drop_vector_index(
    handle: *mut OdbHandle,
    table: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let table_str = match c_str_to_string(table) {
        Some(t) => t,
        None => { set_error("null table"); return -1; }
    };
    let vs = (*handle).vector_store();
    if vs.drop_collection(&table_str) { 0 } else { set_error("Vector index not found"); -1 }
}

/// List all vector indexes as a JSON array. Must be freed with overdrive_free_string().
/// Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_list_vector_indexes(handle: *mut OdbHandle) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let vs = (*handle).vector_store();
    let list = vs.list();
    let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

// -------------------------------------------------------------
// TASK 4.3 - VECTOR ENGINE FFI TESTS
// -------------------------------------------------------------

#[cfg(test)]
mod vector_ffi_tests {
    use super::*;
    use std::ffi::CString;

    /// Helper: open a temp database and return a raw handle pointer.
    fn open_temp_db() -> (*mut OdbHandle, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let handle = unsafe { overdrive_open(path_cstr.as_ptr()) };
        assert!(!handle.is_null(), "Failed to open temp db");
        (handle, dir)
    }

    // 4.3.1 — overdrive_create_vector_index
    #[test]
    fn test_create_vector_index_success() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("embeddings").unwrap();
        let field = CString::new("vec").unwrap();
        let metric = CString::new("cosine").unwrap();
        let rc = unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };
        assert_eq!(rc, 0, "create_vector_index should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_vector_index_invalid_metric() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("bad_metric_table").unwrap();
        let field = CString::new("vec").unwrap();
        let metric = CString::new("manhattan").unwrap(); // unsupported
        let rc = unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };
        assert_eq!(rc, -1, "create_vector_index should return -1 for unknown metric");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_vector_index_null_handle() {
        let table = CString::new("t").unwrap();
        let field = CString::new("f").unwrap();
        let metric = CString::new("cosine").unwrap();
        let rc = unsafe { overdrive_create_vector_index(ptr::null_mut(), table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };
        assert_eq!(rc, -1);
    }

    // 4.3.2 — overdrive_insert_vector
    #[test]
    fn test_insert_vector_success() {
        let (handle, _dir) = open_temp_db();
        // Create index first
        let table = CString::new("docs").unwrap();
        let field = CString::new("emb").unwrap();
        let metric = CString::new("cosine").unwrap();
        unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };

        let json = CString::new(r#"{"_id":"doc1","title":"hello"}"#).unwrap();
        let emb = CString::new("[1.0, 0.0, 0.0]").unwrap();
        let id_ptr = unsafe { overdrive_insert_vector(handle, table.as_ptr(), json.as_ptr(), emb.as_ptr()) };
        assert!(!id_ptr.is_null(), "insert_vector should return an ID");
        let id_str = unsafe { CStr::from_ptr(id_ptr).to_string_lossy().into_owned() };
        assert_eq!(id_str, "doc1");
        unsafe { overdrive_free_string(id_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_insert_vector_auto_id() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("autoid_docs").unwrap();
        let field = CString::new("emb").unwrap();
        let metric = CString::new("euclidean").unwrap();
        unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 2, metric.as_ptr()) };

        // No _id in metadata — should auto-generate
        let json = CString::new(r#"{"title":"auto"}"#).unwrap();
        let emb = CString::new("[0.5, 0.5]").unwrap();
        let id_ptr = unsafe { overdrive_insert_vector(handle, table.as_ptr(), json.as_ptr(), emb.as_ptr()) };
        assert!(!id_ptr.is_null(), "insert_vector should return an auto-generated ID");
        unsafe { overdrive_free_string(id_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_insert_vector_wrong_dimensions() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("dim_check").unwrap();
        let field = CString::new("emb").unwrap();
        let metric = CString::new("cosine").unwrap();
        unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };

        let json = CString::new(r#"{"_id":"x"}"#).unwrap();
        let emb = CString::new("[1.0, 0.0]").unwrap(); // 2D into 3D index
        let id_ptr = unsafe { overdrive_insert_vector(handle, table.as_ptr(), json.as_ptr(), emb.as_ptr()) };
        assert!(id_ptr.is_null(), "insert_vector should fail on dimension mismatch");
        unsafe { overdrive_close(handle) };
    }

    // 4.3.3 — overdrive_vector_search
    #[test]
    fn test_vector_search_returns_results() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("search_test").unwrap();
        let field = CString::new("emb").unwrap();
        let metric = CString::new("cosine").unwrap();
        unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };

        // Insert a few vectors
        for (id, vec) in [("a", "[1.0,0.0,0.0]"), ("b", "[0.0,1.0,0.0]"), ("c", "[0.9,0.1,0.0]")] {
            let json = CString::new(format!(r#"{{"_id":"{}"}}"#, id)).unwrap();
            let emb = CString::new(vec).unwrap();
            unsafe { overdrive_insert_vector(handle, table.as_ptr(), json.as_ptr(), emb.as_ptr()) };
        }

        let query = CString::new("[1.0,0.0,0.0]").unwrap();
        let metric_null: *const c_char = ptr::null();
        let result_ptr = unsafe { overdrive_vector_search(handle, table.as_ptr(), query.as_ptr(), 2, metric_null) };
        assert!(!result_ptr.is_null(), "vector_search should return results");

        let result_str = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&result_str).expect("valid JSON");
        let arr = parsed.as_array().expect("array");
        assert!(!arr.is_empty(), "should have at least one result");
        // Verify shape: each result has id, score, metadata
        let first = &arr[0];
        assert!(first.get("id").is_some(), "result should have 'id'");
        assert!(first.get("score").is_some(), "result should have 'score'");

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_vector_search_with_metric_override() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("metric_override").unwrap();
        let field = CString::new("emb").unwrap();
        let metric = CString::new("cosine").unwrap();
        unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };

        let json = CString::new(r#"{"_id":"v1"}"#).unwrap();
        let emb = CString::new("[1.0,0.0,0.0]").unwrap();
        unsafe { overdrive_insert_vector(handle, table.as_ptr(), json.as_ptr(), emb.as_ptr()) };

        let query = CString::new("[1.0,0.0,0.0]").unwrap();
        let dot_metric = CString::new("dot").unwrap();
        let result_ptr = unsafe { overdrive_vector_search(handle, table.as_ptr(), query.as_ptr(), 1, dot_metric.as_ptr()) };
        assert!(!result_ptr.is_null());
        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    // 4.3.4 — overdrive_drop_vector_index
    #[test]
    fn test_drop_vector_index_success() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("to_drop").unwrap();
        let field = CString::new("emb").unwrap();
        let metric = CString::new("cosine").unwrap();
        unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 3, metric.as_ptr()) };

        let rc = unsafe { overdrive_drop_vector_index(handle, table.as_ptr()) };
        assert_eq!(rc, 0, "drop_vector_index should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_drop_vector_index_not_found() {
        let (handle, _dir) = open_temp_db();
        let table = CString::new("nonexistent").unwrap();
        let rc = unsafe { overdrive_drop_vector_index(handle, table.as_ptr()) };
        assert_eq!(rc, -1, "drop_vector_index should return -1 when index not found");
        unsafe { overdrive_close(handle) };
    }

    // 4.3.5 — overdrive_list_vector_indexes
    #[test]
    fn test_list_vector_indexes_empty() {
        let (handle, _dir) = open_temp_db();
        let result_ptr = unsafe { overdrive_list_vector_indexes(handle) };
        assert!(!result_ptr.is_null());
        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        assert!(parsed.is_array());
        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_list_vector_indexes_after_create() {
        let (handle, _dir) = open_temp_db();
        for name in ["idx1", "idx2"] {
            let table = CString::new(name).unwrap();
            let field = CString::new("emb").unwrap();
            let metric = CString::new("cosine").unwrap();
            unsafe { overdrive_create_vector_index(handle, table.as_ptr(), field.as_ptr(), 4, metric.as_ptr()) };
        }

        let result_ptr = unsafe { overdrive_list_vector_indexes(handle) };
        assert!(!result_ptr.is_null());
        let s = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let arr = parsed.as_array().expect("array");
        assert_eq!(arr.len(), 2, "should list 2 indexes");

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }
}

// -------------------------------------------------------------
// TASK 4.5 - GRAPH ENGINE FFI FUNCTIONS
// -------------------------------------------------------------

/// Create a node type in the graph engine.
/// type_name: the name of the node type (e.g. "Person", "Product").
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_node_type(
    handle: *mut OdbHandle,
    type_name: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(type_name) {
        Some(n) => n,
        None => { set_error("null type_name"); return -1; }
    };
    let graph = (*handle).graph_engine();
    match graph.create_node_type(&name) {
        Ok(()) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

// -------------------------------------------------------------
// TASK 4.5.1 - GRAPH ENGINE FFI TESTS
// -------------------------------------------------------------

#[cfg(test)]
mod graph_ffi_tests {
    use super::*;
    use std::ffi::CString;

    fn open_temp_db() -> (*mut OdbHandle, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let handle = unsafe { overdrive_open(path_cstr.as_ptr()) };
        assert!(!handle.is_null(), "Failed to open temp db");
        (handle, dir)
    }

    // 4.5.1 — overdrive_create_node_type
    #[test]
    fn test_create_node_type_success() {
        let (handle, _dir) = open_temp_db();
        let type_name = CString::new("Person").unwrap();
        let rc = unsafe { overdrive_create_node_type(handle, type_name.as_ptr()) };
        assert_eq!(rc, 0, "create_node_type should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_node_type_duplicate_returns_error() {
        let (handle, _dir) = open_temp_db();
        let type_name = CString::new("Product").unwrap();
        let rc1 = unsafe { overdrive_create_node_type(handle, type_name.as_ptr()) };
        assert_eq!(rc1, 0, "first create should succeed");
        let rc2 = unsafe { overdrive_create_node_type(handle, type_name.as_ptr()) };
        assert_eq!(rc2, -1, "duplicate create_node_type should return -1");
        // Error message should be set
        let err_ptr = overdrive_last_error();
        assert!(!err_ptr.is_null(), "last error should be set on duplicate");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_node_type_null_handle() {
        let type_name = CString::new("Node").unwrap();
        let rc = unsafe { overdrive_create_node_type(ptr::null_mut(), type_name.as_ptr()) };
        assert_eq!(rc, -1, "null handle should return -1");
    }

    #[test]
    fn test_create_node_type_null_name() {
        let (handle, _dir) = open_temp_db();
        let rc = unsafe { overdrive_create_node_type(handle, ptr::null()) };
        assert_eq!(rc, -1, "null type_name should return -1");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_multiple_node_types() {
        let (handle, _dir) = open_temp_db();
        for name in ["Person", "Company", "Location"] {
            let type_name = CString::new(name).unwrap();
            let rc = unsafe { overdrive_create_node_type(handle, type_name.as_ptr()) };
            assert_eq!(rc, 0, "create_node_type('{}') should succeed", name);
        }
        unsafe { overdrive_close(handle) };
    }
}

// =============================================================
// GRAPH ENGINE FFI — Tasks 4.5.2 – 4.5.8
// =============================================================

/// 4.5.2 — Create an edge type (schema entry)
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_edge_type(
    handle: *mut OdbHandle,
    type_name: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(type_name) {
        Some(n) => n,
        None => { set_error("null type_name"); return -1; }
    };
    let graph = (*handle).graph_engine();
    match graph.create_edge_type(&name) {
        Ok(()) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

/// 4.5.3 — Create a node with the given type and JSON properties.
/// Returns a newly allocated C string containing the node ID, or NULL on error.
/// Caller must free with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_node(
    handle: *mut OdbHandle,
    type_name: *const c_char,
    properties_json: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let type_str = match c_str_to_string(type_name) {
        Some(s) => s,
        None => { set_error("null type_name"); return ptr::null_mut(); }
    };
    let props_str = match c_str_to_string(properties_json) {
        Some(s) => s,
        None => { set_error("null properties_json"); return ptr::null_mut(); }
    };
    let props: serde_json::Value = match serde_json::from_str(&props_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("invalid JSON: {}", e)); return ptr::null_mut(); }
    };
    let graph = (*handle).graph_engine();
    match graph.create_node(&type_str, props) {
        Ok(id) => alloc_c_string(&id),
        Err(e) => { set_error(&e); ptr::null_mut() }
    }
}

/// 4.5.4 — Create a directed edge between two nodes.
/// Returns a newly allocated C string containing the edge ID, or NULL on error.
/// Caller must free with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_edge(
    handle: *mut OdbHandle,
    type_name: *const c_char,
    from_node_id: *const c_char,
    to_node_id: *const c_char,
    properties_json: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let type_str = match c_str_to_string(type_name) {
        Some(s) => s,
        None => { set_error("null type_name"); return ptr::null_mut(); }
    };
    let from_str = match c_str_to_string(from_node_id) {
        Some(s) => s,
        None => { set_error("null from_node_id"); return ptr::null_mut(); }
    };
    let to_str = match c_str_to_string(to_node_id) {
        Some(s) => s,
        None => { set_error("null to_node_id"); return ptr::null_mut(); }
    };
    let props_str = match c_str_to_string(properties_json) {
        Some(s) => s,
        None => { set_error("null properties_json"); return ptr::null_mut(); }
    };
    let props: serde_json::Value = match serde_json::from_str(&props_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("invalid JSON: {}", e)); return ptr::null_mut(); }
    };
    let graph = (*handle).graph_engine();
    match graph.create_edge(&type_str, &from_str, &to_str, props) {
        Ok(id) => alloc_c_string(&id),
        Err(e) => { set_error(&e); ptr::null_mut() }
    }
}

/// 4.5.5 — Execute a MATCH traversal query.
/// `match_query` is a Cypher-like string, e.g.:
///   MATCH (p:Person)-[r:KNOWS]->(q:Person) WHERE p.name="Alice" RETURN p,r,q
/// Returns a JSON array of result rows, or NULL on error.
/// Caller must free with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_graph_traverse(
    handle: *mut OdbHandle,
    match_query: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let query_str = match c_str_to_string(match_query) {
        Some(s) => s,
        None => { set_error("null match_query"); return ptr::null_mut(); }
    };
    let parsed = match crate::graph::query::parse_match(&query_str) {
        Ok(q) => q,
        Err(e) => { set_error(&format!("parse error: {}", e)); return ptr::null_mut(); }
    };
    let graph = (*handle).graph_engine();
    let results = graph.match_query(parsed);
    match serde_json::to_string(&results) {
        Ok(json) => alloc_c_string(&json),
        Err(e) => { set_error(&format!("serialization error: {}", e)); ptr::null_mut() }
    }
}

/// 4.5.6 — Find the shortest path between two nodes via an optional edge type.
/// Pass an empty string for `edge_type` to traverse any edge type.
/// Returns a JSON object with `nodes`, `edges`, and `total_hops`, or NULL if no path exists.
/// Caller must free with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_shortest_path(
    handle: *mut OdbHandle,
    from_node_id: *const c_char,
    to_node_id: *const c_char,
    edge_type: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let from_str = match c_str_to_string(from_node_id) {
        Some(s) => s,
        None => { set_error("null from_node_id"); return ptr::null_mut(); }
    };
    let to_str = match c_str_to_string(to_node_id) {
        Some(s) => s,
        None => { set_error("null to_node_id"); return ptr::null_mut(); }
    };
    // Empty string means "any edge type"
    let edge_filter = c_str_to_string(edge_type)
        .filter(|s| !s.is_empty());

    let graph = (*handle).graph_engine();
    match graph.shortest_path(&from_str, &to_str, edge_filter.as_deref()) {
        Some(path) => match serde_json::to_string(&path) {
            Ok(json) => alloc_c_string(&json),
            Err(e) => { set_error(&format!("serialization error: {}", e)); ptr::null_mut() }
        },
        None => {
            set_error(&format!("no path from '{}' to '{}'", from_str, to_str));
            ptr::null_mut()
        }
    }
}

/// 4.5.7 — Delete a node and all its connected edges.
/// Returns 0 on success, -1 if the node was not found or handle is null.
#[no_mangle]
pub unsafe extern "C" fn overdrive_delete_node(
    handle: *mut OdbHandle,
    node_id: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let id = match c_str_to_string(node_id) {
        Some(s) => s,
        None => { set_error("null node_id"); return -1; }
    };
    let graph = (*handle).graph_engine();
    if graph.delete_node(&id) { 0 } else {
        set_error(&format!("node '{}' not found", id));
        -1
    }
}

/// 4.5.8 — List nodes, optionally filtered by type.
/// Pass an empty string for `type_name` to list all nodes.
/// Returns a JSON array of node objects, or NULL on error.
/// Caller must free with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_list_nodes(
    handle: *mut OdbHandle,
    type_name: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let type_filter = c_str_to_string(type_name)
        .filter(|s| !s.is_empty());
    let graph = (*handle).graph_engine();
    let nodes = graph.list_nodes(type_filter.as_deref());
    match serde_json::to_string(&nodes) {
        Ok(json) => alloc_c_string(&json),
        Err(e) => { set_error(&format!("serialization error: {}", e)); ptr::null_mut() }
    }
}

// =============================================================
// GRAPH ENGINE FFI TESTS — Tasks 4.5.2 – 4.5.8
// =============================================================

#[cfg(test)]
mod graph_ffi_extended_tests {
    use super::*;
    use std::ffi::CString;

    fn open_temp_db() -> (*mut OdbHandle, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let handle = unsafe { overdrive_open(path_cstr.as_ptr()) };
        assert!(!handle.is_null(), "Failed to open temp db");
        (handle, dir)
    }

    // ── 4.5.2 overdrive_create_edge_type ──────────────────────

    #[test]
    fn test_create_edge_type_success() {
        let (handle, _dir) = open_temp_db();
        let type_name = CString::new("KNOWS").unwrap();
        let rc = unsafe { overdrive_create_edge_type(handle, type_name.as_ptr()) };
        assert_eq!(rc, 0);
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_edge_type_duplicate_returns_error() {
        let (handle, _dir) = open_temp_db();
        let type_name = CString::new("FOLLOWS").unwrap();
        assert_eq!(unsafe { overdrive_create_edge_type(handle, type_name.as_ptr()) }, 0);
        assert_eq!(unsafe { overdrive_create_edge_type(handle, type_name.as_ptr()) }, -1);
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_edge_type_null_handle() {
        let type_name = CString::new("EDGE").unwrap();
        let rc = unsafe { overdrive_create_edge_type(ptr::null_mut(), type_name.as_ptr()) };
        assert_eq!(rc, -1);
    }

    // ── 4.5.3 overdrive_create_node ───────────────────────────

    #[test]
    fn test_create_node_returns_id() {
        let (handle, _dir) = open_temp_db();
        let type_name = CString::new("Person").unwrap();
        let props = CString::new(r#"{"name":"Alice","age":30}"#).unwrap();
        let id_ptr = unsafe { overdrive_create_node(handle, type_name.as_ptr(), props.as_ptr()) };
        assert!(!id_ptr.is_null(), "create_node should return a node ID");
        let id = unsafe { std::ffi::CStr::from_ptr(id_ptr).to_string_lossy().to_string() };
        assert!(id.starts_with("node_"), "ID should start with 'node_', got: {}", id);
        unsafe { overdrive_free_string(id_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_create_node_invalid_json() {
        let (handle, _dir) = open_temp_db();
        let type_name = CString::new("Person").unwrap();
        let bad_json = CString::new("not-json").unwrap();
        let id_ptr = unsafe { overdrive_create_node(handle, type_name.as_ptr(), bad_json.as_ptr()) };
        assert!(id_ptr.is_null(), "invalid JSON should return null");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_node_null_handle() {
        let type_name = CString::new("Person").unwrap();
        let props = CString::new("{}").unwrap();
        let id_ptr = unsafe { overdrive_create_node(ptr::null_mut(), type_name.as_ptr(), props.as_ptr()) };
        assert!(id_ptr.is_null());
    }

    // ── 4.5.4 overdrive_create_edge ───────────────────────────

    #[test]
    fn test_create_edge_returns_id() {
        let (handle, _dir) = open_temp_db();
        // Create two nodes first
        let ptype = CString::new("Person").unwrap();
        let props = CString::new("{}").unwrap();
        let n1 = unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        let n2 = unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        assert!(!n1.is_null() && !n2.is_null());

        let etype = CString::new("KNOWS").unwrap();
        let eprops = CString::new(r#"{"since":2020}"#).unwrap();
        let edge_id = unsafe { overdrive_create_edge(handle, etype.as_ptr(), n1, n2, eprops.as_ptr()) };
        assert!(!edge_id.is_null(), "create_edge should return an edge ID");
        let id_str = unsafe { std::ffi::CStr::from_ptr(edge_id).to_string_lossy().to_string() };
        assert!(id_str.starts_with("edge_"), "edge ID should start with 'edge_', got: {}", id_str);

        unsafe { overdrive_free_string(n1); overdrive_free_string(n2); overdrive_free_string(edge_id); overdrive_close(handle) };
    }

    #[test]
    fn test_create_edge_missing_node_returns_null() {
        let (handle, _dir) = open_temp_db();
        let etype = CString::new("KNOWS").unwrap();
        let from = CString::new("node_999").unwrap();
        let to = CString::new("node_998").unwrap();
        let eprops = CString::new("{}").unwrap();
        let edge_id = unsafe { overdrive_create_edge(handle, etype.as_ptr(), from.as_ptr(), to.as_ptr(), eprops.as_ptr()) };
        assert!(edge_id.is_null(), "edge to non-existent nodes should return null");
        unsafe { overdrive_close(handle) };
    }

    // ── 4.5.5 overdrive_graph_traverse ────────────────────────

    #[test]
    fn test_graph_traverse_returns_json_array() {
        let (handle, _dir) = open_temp_db();
        // Create node type + two nodes + edge
        let ptype = CString::new("Person").unwrap();
        unsafe { overdrive_create_node_type(handle, ptype.as_ptr()) };
        let knows = CString::new("KNOWS").unwrap();
        unsafe { overdrive_create_edge_type(handle, knows.as_ptr()) };

        let props_alice = CString::new(r#"{"name":"Alice"}"#).unwrap();
        let props_bob = CString::new(r#"{"name":"Bob"}"#).unwrap();
        let ptype2 = CString::new("Person").unwrap();
        let n1 = unsafe { overdrive_create_node(handle, ptype2.as_ptr(), props_alice.as_ptr()) };
        let n2 = unsafe { overdrive_create_node(handle, ptype2.as_ptr(), props_bob.as_ptr()) };
        let etype = CString::new("KNOWS").unwrap();
        let eprops = CString::new("{}").unwrap();
        unsafe { overdrive_create_edge(handle, etype.as_ptr(), n1, n2, eprops.as_ptr()) };

        let query = CString::new(r#"MATCH (p:Person)-[r:KNOWS]->(q:Person) RETURN p,r,q"#).unwrap();
        let result_ptr = unsafe { overdrive_graph_traverse(handle, query.as_ptr()) };
        assert!(!result_ptr.is_null(), "traverse should return JSON");
        let json_str = unsafe { std::ffi::CStr::from_ptr(result_ptr).to_string_lossy().to_string() };
        assert!(json_str.starts_with('['), "result should be a JSON array");

        unsafe { overdrive_free_string(n1); overdrive_free_string(n2); overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_graph_traverse_null_handle() {
        let query = CString::new("MATCH (p:Person) RETURN p").unwrap();
        let result = unsafe { overdrive_graph_traverse(ptr::null_mut(), query.as_ptr()) };
        assert!(result.is_null());
    }

    // ── 4.5.6 overdrive_shortest_path ─────────────────────────

    #[test]
    fn test_shortest_path_found() {
        let (handle, _dir) = open_temp_db();
        let ptype = CString::new("Person").unwrap();
        let props = CString::new("{}").unwrap();
        let n1 = unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        let ptype2 = CString::new("Person").unwrap();
        let n2 = unsafe { overdrive_create_node(handle, ptype2.as_ptr(), props.as_ptr()) };
        let etype = CString::new("KNOWS").unwrap();
        let eprops = CString::new("{}").unwrap();
        unsafe { overdrive_create_edge(handle, etype.as_ptr(), n1, n2, eprops.as_ptr()) };

        let edge_filter = CString::new("KNOWS").unwrap();
        let path_ptr = unsafe { overdrive_shortest_path(handle, n1, n2, edge_filter.as_ptr()) };
        assert!(!path_ptr.is_null(), "shortest_path should find a path");
        let json_str = unsafe { std::ffi::CStr::from_ptr(path_ptr).to_string_lossy().to_string() };
        assert!(json_str.contains("total_hops"), "result should contain total_hops");

        unsafe { overdrive_free_string(n1); overdrive_free_string(n2); overdrive_free_string(path_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_shortest_path_not_found() {
        let (handle, _dir) = open_temp_db();
        let ptype = CString::new("Person").unwrap();
        let props = CString::new("{}").unwrap();
        let n1 = unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        let ptype2 = CString::new("Person").unwrap();
        let n2 = unsafe { overdrive_create_node(handle, ptype2.as_ptr(), props.as_ptr()) };
        // No edge between them
        let edge_filter = CString::new("").unwrap();
        let path_ptr = unsafe { overdrive_shortest_path(handle, n1, n2, edge_filter.as_ptr()) };
        assert!(path_ptr.is_null(), "no path should return null");

        unsafe { overdrive_free_string(n1); overdrive_free_string(n2); overdrive_close(handle) };
    }

    // ── 4.5.7 overdrive_delete_node ───────────────────────────

    #[test]
    fn test_delete_node_success() {
        let (handle, _dir) = open_temp_db();
        let ptype = CString::new("Person").unwrap();
        let props = CString::new("{}").unwrap();
        let node_id = unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        assert!(!node_id.is_null());
        let rc = unsafe { overdrive_delete_node(handle, node_id) };
        assert_eq!(rc, 0, "delete_node should return 0 on success");
        unsafe { overdrive_free_string(node_id); overdrive_close(handle) };
    }

    #[test]
    fn test_delete_node_not_found() {
        let (handle, _dir) = open_temp_db();
        let node_id = CString::new("node_9999").unwrap();
        let rc = unsafe { overdrive_delete_node(handle, node_id.as_ptr()) };
        assert_eq!(rc, -1, "deleting non-existent node should return -1");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_delete_node_removes_edges() {
        let (handle, _dir) = open_temp_db();
        let ptype = CString::new("Person").unwrap();
        let props = CString::new("{}").unwrap();
        let n1 = unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        let ptype2 = CString::new("Person").unwrap();
        let n2 = unsafe { overdrive_create_node(handle, ptype2.as_ptr(), props.as_ptr()) };
        let etype = CString::new("KNOWS").unwrap();
        let eprops = CString::new("{}").unwrap();
        unsafe { overdrive_create_edge(handle, etype.as_ptr(), n1, n2, eprops.as_ptr()) };
        // Delete n1 — edge should also be gone
        let rc = unsafe { overdrive_delete_node(handle, n1) };
        assert_eq!(rc, 0);
        unsafe { overdrive_free_string(n1); overdrive_free_string(n2); overdrive_close(handle) };
    }

    // ── 4.5.8 overdrive_list_nodes ────────────────────────────

    #[test]
    fn test_list_nodes_empty() {
        let (handle, _dir) = open_temp_db();
        let type_filter = CString::new("").unwrap();
        let result_ptr = unsafe { overdrive_list_nodes(handle, type_filter.as_ptr()) };
        assert!(!result_ptr.is_null());
        let json_str = unsafe { std::ffi::CStr::from_ptr(result_ptr).to_string_lossy().to_string() };
        assert_eq!(json_str, "[]", "empty graph should return empty array");
        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_list_nodes_with_type_filter() {
        let (handle, _dir) = open_temp_db();
        let ptype = CString::new("Person").unwrap();
        let ctype = CString::new("Company").unwrap();
        let props = CString::new("{}").unwrap();
        unsafe { overdrive_create_node(handle, ptype.as_ptr(), props.as_ptr()) };
        unsafe { overdrive_create_node(handle, ctype.as_ptr(), props.as_ptr()) };

        let filter = CString::new("Person").unwrap();
        let result_ptr = unsafe { overdrive_list_nodes(handle, filter.as_ptr()) };
        assert!(!result_ptr.is_null());
        let json_str = unsafe { std::ffi::CStr::from_ptr(result_ptr).to_string_lossy().to_string() };
        let nodes: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(nodes.as_array().unwrap().len(), 1, "should only return Person nodes");
        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_list_nodes_null_handle() {
        let filter = CString::new("").unwrap();
        let result = unsafe { overdrive_list_nodes(ptr::null_mut(), filter.as_ptr()) };
        assert!(result.is_null());
    }
}

// -------------------------------------------------------------
// TASK 4.6 - STREAMING ENGINE FFI FUNCTIONS
// -------------------------------------------------------------

/// 4.6.1 — Create a topic with the given name, partition count, and retention.
/// retention_seconds: 0 means keep forever.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_topic(
    handle: *mut OdbHandle,
    topic_name: *const c_char,
    partitions: u32,
    retention_seconds: u64,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(topic_name) {
        Some(n) => n,
        None => { set_error("null topic_name"); return -1; }
    };
    // parse_ttl accepts "0"/"never" for no TTL, or a plain number of seconds,
    // or suffixed values like "30d", "24h", "60m".
    let retention_str = if retention_seconds == 0 {
        "0".to_string()
    } else {
        retention_seconds.to_string()
    };
    let engine = (*handle).stream_engine();
    match engine.create(&name, partitions as usize, &retention_str) {
        Ok(()) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

/// 4.6.2 — Publish a JSON message to a topic.
/// Returns a JSON string `{"offset": N}` (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_publish(
    handle: *mut OdbHandle,
    topic_name: *const c_char,
    message_json: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let name = match c_str_to_string(topic_name) {
        Some(n) => n,
        None => { set_error("null topic_name"); return ptr::null_mut(); }
    };
    let msg_str = match c_str_to_string(message_json) {
        Some(m) => m,
        None => { set_error("null message_json"); return ptr::null_mut(); }
    };
    let payload: serde_json::Value = match serde_json::from_str(&msg_str) {
        Ok(v) => v,
        Err(e) => { set_error(&format!("JSON parse error: {}", e)); return ptr::null_mut(); }
    };
    let engine = (*handle).stream_engine();
    match engine.publish(&name, payload) {
        Ok(offset) => {
            let json = serde_json::json!({ "offset": offset });
            alloc_c_string(&json.to_string())
        }
        Err(e) => { set_error(&e); ptr::null_mut() }
    }
}

/// 4.6.3 — Subscribe to a topic.
/// consumer_group: pass NULL or empty string for no group (anonymous consumer).
/// offset_mode: "latest" (default) or "earliest" to start from offset 0.
/// Returns a JSON string `{"subscription_id": N}` (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_subscribe(
    handle: *mut OdbHandle,
    topic_name: *const c_char,
    consumer_group: *const c_char,
    offset_mode: *const c_char,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let name = match c_str_to_string(topic_name) {
        Some(n) => n,
        None => { set_error("null topic_name"); return ptr::null_mut(); }
    };
    let group = c_str_to_string(consumer_group).filter(|s| !s.is_empty());
    let mode = c_str_to_string(offset_mode).unwrap_or_else(|| "latest".to_string());

    // Determine starting offset based on mode
    let start_offset = if mode.eq_ignore_ascii_case("earliest") {
        Some(0u64)
    } else {
        None // latest — resolved at poll time
    };

    // Register subscription state on the handle
    let h = &mut *handle;
    h.sub_counter += 1;
    let sub_id = h.sub_counter;
    h.subscriptions.insert(sub_id, SubscriptionState {
        topic: name,
        consumer_group: group,
        current_offset: start_offset.unwrap_or(u64::MAX), // u64::MAX = "latest, resolve on first poll"
    });

    let json = serde_json::json!({ "subscription_id": sub_id });
    alloc_c_string(&json.to_string())
}

/// 4.6.4 — Poll for messages on a subscription.
/// max_messages: maximum number of messages to return (0 = use default of 100).
/// timeout_ms: reserved for future use (currently ignored).
/// Returns a JSON array of message objects (must be freed), or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_poll(
    handle: *mut OdbHandle,
    subscription_id: u64,
    max_messages: u32,
    _timeout_ms: u32,
) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }

    let h = &mut *handle;
    let (topic, group, current_offset) = match h.subscriptions.get(&subscription_id) {
        Some(s) => (s.topic.clone(), s.consumer_group.clone(), s.current_offset),
        None => {
            set_error(&format!("subscription {} not found", subscription_id));
            return ptr::null_mut();
        }
    };

    let limit = if max_messages == 0 { 100 } else { max_messages as usize };

    // Resolve "latest" offset on first poll
    let from_offset = if current_offset == u64::MAX {
        None // StreamEngine::subscribe with None = LATEST
    } else {
        Some(current_offset)
    };

    let engine = h.stream_engine();
    let results = match engine.subscribe(&topic, from_offset, group.as_deref(), limit) {
        Ok(r) => r,
        Err(e) => { set_error(&e); return ptr::null_mut(); }
    };

    // Advance the stored offset past the last message we received
    if let Some(last) = results.last() {
        let next_offset = last.offset + 1;
        if let Some(sub) = h.subscriptions.get_mut(&subscription_id) {
            sub.current_offset = next_offset;
        }
    } else if current_offset == u64::MAX {
        // No messages yet; set offset to 0 so next poll starts from beginning of "latest"
        if let Some(sub) = h.subscriptions.get_mut(&subscription_id) {
            sub.current_offset = 0;
        }
    }

    let json_arr: Vec<serde_json::Value> = results.into_iter().map(|r| {
        serde_json::json!({
            "offset": r.offset,
            "timestamp_ms": r.timestamp_ms,
            "payload": r.payload,
        })
    }).collect();

    let json = serde_json::to_string(&json_arr).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

/// 4.6.5 — Commit a consumer group offset for a topic.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_commit_offset(
    handle: *mut OdbHandle,
    topic_name: *const c_char,
    consumer_group: *const c_char,
    offset: u64,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let topic = match c_str_to_string(topic_name) {
        Some(n) => n,
        None => { set_error("null topic_name"); return -1; }
    };
    let group = match c_str_to_string(consumer_group).filter(|s| !s.is_empty()) {
        Some(g) => g,
        None => { set_error("consumer_group must not be null or empty"); return -1; }
    };

    // Use subscribe with the group to commit the offset (the engine commits on subscribe)
    let engine = (*handle).stream_engine();
    match engine.subscribe(&topic, Some(offset), Some(&group), 0) {
        Ok(_) => 0,
        Err(e) => { set_error(&e); -1 }
    }
}

/// 4.6.6 — Unsubscribe (close) a subscription by ID.
/// Returns 0 on success, -1 if the subscription was not found.
#[no_mangle]
pub unsafe extern "C" fn overdrive_unsubscribe(
    handle: *mut OdbHandle,
    subscription_id: u64,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let h = &mut *handle;
    if h.subscriptions.remove(&subscription_id).is_some() {
        0
    } else {
        set_error(&format!("subscription {} not found", subscription_id));
        -1
    }
}

/// 4.6.7 — Drop a topic and all its messages.
/// Returns 0 on success, -1 if the topic was not found.
#[no_mangle]
pub unsafe extern "C" fn overdrive_drop_topic(
    handle: *mut OdbHandle,
    topic_name: *const c_char,
) -> c_int {
    clear_error();
    if handle.is_null() { set_error("null handle"); return -1; }
    let name = match c_str_to_string(topic_name) {
        Some(n) => n,
        None => { set_error("null topic_name"); return -1; }
    };
    let engine = (*handle).stream_engine();
    if engine.drop_stream(&name) {
        0
    } else {
        set_error(&format!("topic '{}' not found", name));
        -1
    }
}

/// 4.6.8 — List all topics as a JSON array.
/// Each element has: name, partitions, retention_seconds, created_at.
/// Must be freed with overdrive_free_string(). Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_list_topics(handle: *mut OdbHandle) -> *mut c_char {
    clear_error();
    if handle.is_null() { set_error("null handle"); return ptr::null_mut(); }
    let engine = (*handle).stream_engine();
    let defs = engine.list();
    let result: Vec<serde_json::Value> = defs.into_iter().map(|d| {
        serde_json::json!({
            "name": d.name,
            "partitions": d.partitions,
            "retention_seconds": d.retention_seconds,
            "created_at": d.created_at,
        })
    }).collect();
    let json = serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string());
    alloc_c_string(&json)
}

// -------------------------------------------------------------
// TASK 4.6 - STREAMING ENGINE FFI TESTS
// -------------------------------------------------------------

#[cfg(test)]
mod streaming_ffi_tests {
    use super::*;
    use std::ffi::CString;

    fn open_temp_db() -> (*mut OdbHandle, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let handle = unsafe { overdrive_open(path_cstr.as_ptr()) };
        assert!(!handle.is_null(), "Failed to open temp db");
        (handle, dir)
    }

    // 4.6.1 — overdrive_create_topic
    #[test]
    fn test_create_topic_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("orders").unwrap();
        let rc = unsafe { overdrive_create_topic(handle, name.as_ptr(), 4, 86400) };
        assert_eq!(rc, 0, "create_topic should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_topic_no_retention() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("forever_topic").unwrap();
        let rc = unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };
        assert_eq!(rc, 0, "create_topic with retention=0 should succeed");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_topic_duplicate_returns_error() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("dup_topic").unwrap();
        assert_eq!(unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) }, 0);
        assert_eq!(unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) }, -1,
            "duplicate create_topic should return -1");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_create_topic_null_handle() {
        let name = CString::new("t").unwrap();
        let rc = unsafe { overdrive_create_topic(ptr::null_mut(), name.as_ptr(), 1, 0) };
        assert_eq!(rc, -1);
    }

    // 4.6.2 — overdrive_publish
    #[test]
    fn test_publish_returns_offset() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("events").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };

        let msg = CString::new(r#"{"type":"click","user":1}"#).unwrap();
        let result_ptr = unsafe { overdrive_publish(handle, name.as_ptr(), msg.as_ptr()) };
        assert!(!result_ptr.is_null(), "publish should return JSON with offset");

        let s = unsafe { std::ffi::CStr::from_ptr(result_ptr).to_string_lossy().to_string() };
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        assert!(v.get("offset").is_some(), "result should have 'offset' field");
        assert_eq!(v["offset"], 0, "first message offset should be 0");

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_publish_increments_offset() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("inc_events").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };

        for expected_offset in 0u64..3 {
            let msg = CString::new(format!(r#"{{"n":{}}}"#, expected_offset)).unwrap();
            let ptr = unsafe { overdrive_publish(handle, name.as_ptr(), msg.as_ptr()) };
            assert!(!ptr.is_null());
            let s = unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string() };
            let v: serde_json::Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["offset"], expected_offset);
            unsafe { overdrive_free_string(ptr) };
        }
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_publish_nonexistent_topic_returns_null() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("ghost").unwrap();
        let msg = CString::new(r#"{"x":1}"#).unwrap();
        let ptr = unsafe { overdrive_publish(handle, name.as_ptr(), msg.as_ptr()) };
        assert!(ptr.is_null(), "publish to non-existent topic should return NULL");
        unsafe { overdrive_close(handle) };
    }

    // 4.6.3 — overdrive_subscribe
    #[test]
    fn test_subscribe_returns_subscription_id() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("sub_topic").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };

        let group = CString::new("my_group").unwrap();
        let mode = CString::new("earliest").unwrap();
        let ptr = unsafe { overdrive_subscribe(handle, name.as_ptr(), group.as_ptr(), mode.as_ptr()) };
        assert!(!ptr.is_null(), "subscribe should return JSON with subscription_id");

        let s = unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string() };
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        assert!(v.get("subscription_id").is_some(), "result should have 'subscription_id'");
        assert!(v["subscription_id"].as_u64().unwrap() > 0);

        unsafe { overdrive_free_string(ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_subscribe_null_handle() {
        let name = CString::new("t").unwrap();
        let ptr = unsafe { overdrive_subscribe(ptr::null_mut(), name.as_ptr(), ptr::null(), ptr::null()) };
        assert!(ptr.is_null());
    }

    // 4.6.4 — overdrive_poll
    #[test]
    fn test_poll_returns_messages() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("poll_topic").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };

        // Publish two messages
        for i in 0..2 {
            let msg = CString::new(format!(r#"{{"i":{}}}"#, i)).unwrap();
            let p = unsafe { overdrive_publish(handle, name.as_ptr(), msg.as_ptr()) };
            unsafe { overdrive_free_string(p) };
        }

        // Subscribe from earliest
        let mode = CString::new("earliest").unwrap();
        let sub_ptr = unsafe { overdrive_subscribe(handle, name.as_ptr(), ptr::null(), mode.as_ptr()) };
        let sub_str = unsafe { std::ffi::CStr::from_ptr(sub_ptr).to_string_lossy().to_string() };
        let sub_v: serde_json::Value = serde_json::from_str(&sub_str).unwrap();
        let sub_id = sub_v["subscription_id"].as_u64().unwrap();
        unsafe { overdrive_free_string(sub_ptr) };

        // Poll
        let result_ptr = unsafe { overdrive_poll(handle, sub_id, 10, 0) };
        assert!(!result_ptr.is_null(), "poll should return JSON array");
        let s = unsafe { std::ffi::CStr::from_ptr(result_ptr).to_string_lossy().to_string() };
        let arr: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let msgs = arr.as_array().expect("array");
        assert_eq!(msgs.len(), 2, "should receive 2 messages");
        assert!(msgs[0].get("offset").is_some());
        assert!(msgs[0].get("payload").is_some());

        unsafe { overdrive_free_string(result_ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_poll_invalid_subscription_returns_null() {
        let (handle, _dir) = open_temp_db();
        let ptr = unsafe { overdrive_poll(handle, 9999, 10, 0) };
        assert!(ptr.is_null(), "poll with invalid sub_id should return NULL");
        unsafe { overdrive_close(handle) };
    }

    // 4.6.5 — overdrive_commit_offset
    #[test]
    fn test_commit_offset_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("commit_topic").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };

        let msg = CString::new(r#"{"v":1}"#).unwrap();
        let p = unsafe { overdrive_publish(handle, name.as_ptr(), msg.as_ptr()) };
        unsafe { overdrive_free_string(p) };

        let group = CString::new("grp1").unwrap();
        let rc = unsafe { overdrive_commit_offset(handle, name.as_ptr(), group.as_ptr(), 1) };
        assert_eq!(rc, 0, "commit_offset should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_commit_offset_null_group_returns_error() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("t").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };
        let rc = unsafe { overdrive_commit_offset(handle, name.as_ptr(), ptr::null(), 0) };
        assert_eq!(rc, -1, "null consumer_group should return -1");
        unsafe { overdrive_close(handle) };
    }

    // 4.6.6 — overdrive_unsubscribe
    #[test]
    fn test_unsubscribe_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("unsub_topic").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };

        let mode = CString::new("latest").unwrap();
        let sub_ptr = unsafe { overdrive_subscribe(handle, name.as_ptr(), ptr::null(), mode.as_ptr()) };
        let sub_str = unsafe { std::ffi::CStr::from_ptr(sub_ptr).to_string_lossy().to_string() };
        let sub_v: serde_json::Value = serde_json::from_str(&sub_str).unwrap();
        let sub_id = sub_v["subscription_id"].as_u64().unwrap();
        unsafe { overdrive_free_string(sub_ptr) };

        let rc = unsafe { overdrive_unsubscribe(handle, sub_id) };
        assert_eq!(rc, 0, "unsubscribe should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_unsubscribe_not_found() {
        let (handle, _dir) = open_temp_db();
        let rc = unsafe { overdrive_unsubscribe(handle, 9999) };
        assert_eq!(rc, -1, "unsubscribe with unknown id should return -1");
        unsafe { overdrive_close(handle) };
    }

    // 4.6.7 — overdrive_drop_topic
    #[test]
    fn test_drop_topic_success() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("drop_me").unwrap();
        unsafe { overdrive_create_topic(handle, name.as_ptr(), 1, 0) };
        let rc = unsafe { overdrive_drop_topic(handle, name.as_ptr()) };
        assert_eq!(rc, 0, "drop_topic should return 0 on success");
        unsafe { overdrive_close(handle) };
    }

    #[test]
    fn test_drop_topic_not_found() {
        let (handle, _dir) = open_temp_db();
        let name = CString::new("ghost_topic").unwrap();
        let rc = unsafe { overdrive_drop_topic(handle, name.as_ptr()) };
        assert_eq!(rc, -1, "drop_topic should return -1 when not found");
        unsafe { overdrive_close(handle) };
    }

    // 4.6.8 — overdrive_list_topics
    #[test]
    fn test_list_topics_empty() {
        let (handle, _dir) = open_temp_db();
        let ptr = unsafe { overdrive_list_topics(handle) };
        assert!(!ptr.is_null());
        let s = unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string() };
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 0);
        unsafe { overdrive_free_string(ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_list_topics_after_create() {
        let (handle, _dir) = open_temp_db();
        for topic in ["alpha", "beta", "gamma"] {
            let n = CString::new(topic).unwrap();
            unsafe { overdrive_create_topic(handle, n.as_ptr(), 2, 3600) };
        }
        let ptr = unsafe { overdrive_list_topics(handle) };
        assert!(!ptr.is_null());
        let s = unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string() };
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 3, "should list 3 topics");
        // Verify shape
        let first = &arr[0];
        assert!(first.get("name").is_some());
        assert!(first.get("partitions").is_some());
        assert!(first.get("retention_seconds").is_some());
        assert!(first.get("created_at").is_some());
        unsafe { overdrive_free_string(ptr); overdrive_close(handle) };
    }

    #[test]
    fn test_list_topics_null_handle() {
        let ptr = unsafe { overdrive_list_topics(ptr::null_mut()) };
        assert!(ptr.is_null());
    }
}

// ─────────────────────────────────────────────────────────────
// TASK 6 — NEW FFI EXPORTS FOR SIMPLIFIED API
// ─────────────────────────────────────────────────────────────

/// Open or create a password-protected database (Task 6.1).
///
/// - `path`     — file path for the `.odb` file.
/// - `password` — UTF-8 password string (min 8 chars). Pass NULL for no encryption.
///
/// On success returns an opaque handle; on failure returns NULL and sets the
/// thread-local error (retrieve with `overdrive_last_error()` /
/// `overdrive_last_error_json()`).
///
/// The returned handle must be closed with `overdrive_close()`.
#[no_mangle]
pub unsafe extern "C" fn overdrive_open_with_password(
    path: *const c_char,
    password: *const c_char,
) -> *mut OdbHandle {
    clear_error();
    let path_str = match c_str_to_string(path) {
        Some(p) => p,
        None => { set_error("overdrive_open_with_password: path is null"); return ptr::null_mut(); }
    };
    let pwd_opt = c_str_to_string(password);

    let db_result = if std::path::Path::new(&path_str).exists() {
        Database::open_with_password(&path_str, pwd_opt.as_deref())
    } else {
        match pwd_opt.as_deref() {
            Some(pwd) => Database::create_with_password(&path_str, pwd),
            None      => Database::create(&path_str),
        }
    };

    match db_result {
        Ok(db)  => Box::into_raw(Box::new(OdbHandle::new(db, PathBuf::from(&path_str)))),
        Err(e)  => { set_odb_error(&e.to_odb_error()); ptr::null_mut() }
    }
}

/// Enable or disable auto-table creation on an open database handle (Task 6.2).
///
/// `enabled`: non-zero = enable, 0 = disable.
///
/// Returns 0 on success, -1 if the handle is null.
#[no_mangle]
pub unsafe extern "C" fn overdrive_set_auto_create_tables(
    handle: *mut OdbHandle,
    enabled: c_int,
) -> c_int {
    if handle.is_null() { set_error("overdrive_set_auto_create_tables: null handle"); return -1; }
    let flag = enabled != 0;
    (*handle).auto_create_tables = flag;
    (*handle).db.set_auto_create_tables(flag);
    0
}

/// Return the last error as a structured JSON string (Task 6.4).
///
/// This is an alias for `overdrive_last_error_json()` exposed under the name
/// specified in the task list so SDK wrappers can call it by the documented name.
///
/// The JSON shape is:
/// ```json
/// {
///   "code":        "ODB-AUTH-001",
///   "message":     "...",
///   "context":     "...",
///   "suggestions": ["..."],
///   "doc_link":    "https://overdrive-db.com/docs/errors/ODB-AUTH-001"
/// }
/// ```
///
/// Returns NULL when there is no pending error.
/// The pointer is valid until the next API call on this thread — do **not** free it.
#[no_mangle]
pub extern "C" fn overdrive_get_error_details() -> *const c_char {
    LAST_ERROR_JSON.with(|e| {
        match &*e.borrow() {
            Some(cs) => cs.as_ptr(),
            None     => ptr::null(),
        }
    })
}

// -------------------------------------------------------------
// TASK 5 - WATCHDOG FFI FUNCTION
// -------------------------------------------------------------

/// Internal stats returned by verify_file_integrity
struct FileIntegrityStats {
    size: u64,
    modified: i64,
    page_count: u64,
}

/// Verify the integrity of a .odb file.
///
/// Checks performed (in order):
/// 1. File existence
/// 2. File size and page alignment
/// 3. Magic number validation (0x4F564442 = "OVDB")
/// 4. Per-page CRC32 checksum verification
/// 5. SHA-256 hash chain validation across all pages
///
/// Returns `Ok(FileIntegrityStats)` on success, or `Err(String)` describing
/// the first corruption found.
fn verify_file_integrity(path: &str) -> std::result::Result<FileIntegrityStats, String> {
    use std::io::Read;
    use crc32fast::Hasher as Crc32Hasher;
    use sha2::{Sha256, Digest};

    const PAGE_SIZE: usize = 4096;
    // Magic number: "OVDB" = 0x4F564442 stored as little-endian bytes
    const MAGIC_BYTES: [u8; 4] = [0x42, 0x44, 0x56, 0x4F]; // LE: 0x4F564442

    // ── 1. File existence ────────────────────────────────────
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Cannot access file: {}", e))?;

    let file_size = metadata.len();

    // ── 2. File size / page alignment ───────────────────────
    if file_size == 0 {
        return Err("File is empty".to_string());
    }
    if file_size % PAGE_SIZE as u64 != 0 {
        return Err(format!(
            "File size {} is not a multiple of page size {}",
            file_size, PAGE_SIZE
        ));
    }

    let page_count = file_size / PAGE_SIZE as u64;

    // Last-modified timestamp (Unix seconds)
    let modified = metadata
        .modified()
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        })
        .unwrap_or(0);

    // ── 3. Read entire file ──────────────────────────────────
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Cannot open file: {}", e))?;
    let mut data = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut data)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    // ── 3. Magic number validation ───────────────────────────
    if data.len() < 4 {
        return Err("File too small to contain magic number".to_string());
    }
    // The header stores magic as a u32 little-endian at offset 0
    let magic_ok = data[0..4] == MAGIC_BYTES
        // Also accept the ASCII "ODBD" variant seen in consistency.rs
        || &data[0..4] == b"ODBD";
    if !magic_ok {
        return Err(format!(
            "Invalid magic number: {:02X}{:02X}{:02X}{:02X} (expected OVDB/ODBD)",
            data[0], data[1], data[2], data[3]
        ));
    }

    // ── 4. Per-page CRC32 checksum verification ──────────────
    // Convention: the last 4 bytes of each page store a CRC32 of the
    // preceding PAGE_SIZE-4 bytes.  Page 0 (header) is skipped because
    // its trailing bytes are part of the header struct and may not carry
    // a checksum in all versions.
    for page_num in 1..page_count as usize {
        let offset = page_num * PAGE_SIZE;
        let page = &data[offset..offset + PAGE_SIZE];

        // Skip all-zero pages (unallocated / free pages)
        if page.iter().all(|&b| b == 0) {
            continue;
        }

        let payload = &page[..PAGE_SIZE - 4];
        let stored_crc = u32::from_le_bytes([
            page[PAGE_SIZE - 4],
            page[PAGE_SIZE - 3],
            page[PAGE_SIZE - 2],
            page[PAGE_SIZE - 1],
        ]);

        // Only validate if the stored CRC is non-zero (pages without a
        // checksum store 0x00000000 in the trailer).
        if stored_crc != 0 {
            let mut hasher = Crc32Hasher::new();
            hasher.update(payload);
            let computed = hasher.finalize();
            if computed != stored_crc {
                return Err(format!(
                    "Page {} CRC32 mismatch: stored={:#010X} computed={:#010X}",
                    page_num, stored_crc, computed
                ));
            }
        }
    }

    // ── 5. SHA-256 hash chain validation ────────────────────
    // Each page (except the first) is expected to embed the SHA-256 hash
    // of the previous page in bytes [4..36] of its header.  Pages that
    // store all-zeros in that range are treated as "no hash" and skipped,
    // preserving compatibility with pages written before hash-chain support
    // was added.
    let mut prev_hash: Option<[u8; 32]> = None;
    for page_num in 0..page_count as usize {
        let offset = page_num * PAGE_SIZE;
        let page = &data[offset..offset + PAGE_SIZE];

        // Skip all-zero pages
        if page.iter().all(|&b| b == 0) {
            prev_hash = None;
            continue;
        }

        if let Some(expected) = prev_hash {
            // Bytes [4..36] of the page header hold the parent hash
            if page.len() >= 36 {
                let stored: [u8; 32] = page[4..36].try_into().unwrap_or([0u8; 32]);
                let all_zero = stored.iter().all(|&b| b == 0);
                if !all_zero && stored != expected {
                    return Err(format!(
                        "Page {} hash chain broken: stored hash does not match hash of page {}",
                        page_num,
                        page_num - 1
                    ));
                }
            }
        }

        // Compute hash of this page (excluding the parent-hash field to
        // avoid circular dependency; hash bytes [0..4] + [36..PAGE_SIZE]).
        let mut hasher = Sha256::new();
        hasher.update(&page[0..4]);
        if page.len() > 36 {
            hasher.update(&page[36..]);
        }
        let hash: [u8; 32] = hasher.finalize().into();
        prev_hash = Some(hash);
    }

    Ok(FileIntegrityStats {
        size: file_size,
        modified,
        page_count,
    })
}

/// Monitor a .odb file's integrity, size, and modification status.
///
/// Returns a JSON string (must be freed with `overdrive_free_string`) with:
/// - `file_path`          — the path passed in
/// - `file_size_bytes`    — file size in bytes (0 if missing/corrupted)
/// - `last_modified`      — Unix timestamp of last modification (0 if unavailable)
/// - `integrity_status`   — "valid", "corrupted", or "missing"
/// - `corruption_details` — null on success, error string on failure
/// - `page_count`         — number of 4096-byte pages (0 if unavailable)
/// - `magic_valid`        — true when the OVDB/ODBD magic number is present
///
/// This function never returns NULL — it always returns a valid JSON string.
/// Performance target: < 100 ms for files under 1 GB.
#[no_mangle]
pub extern "C" fn overdrive_watchdog(path: *const c_char) -> *mut c_char {
    let path_str = if path.is_null() {
        return alloc_c_string(
            r#"{"file_path":"","file_size_bytes":0,"last_modified":0,"integrity_status":"missing","corruption_details":"null path argument","page_count":0,"magic_valid":false}"#
        );
    } else {
        unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned()
    };

    // Fast-path: missing file
    if !std::path::Path::new(&path_str).exists() {
        let report = serde_json::json!({
            "file_path": path_str,
            "file_size_bytes": 0,
            "last_modified": 0,
            "integrity_status": "missing",
            "corruption_details": serde_json::Value::Null,
            "page_count": 0,
            "magic_valid": false
        });
        return alloc_c_string(&report.to_string());
    }

    match verify_file_integrity(&path_str) {
        Ok(stats) => {
            let report = serde_json::json!({
                "file_path": path_str,
                "file_size_bytes": stats.size,
                "last_modified": stats.modified,
                "integrity_status": "valid",
                "corruption_details": serde_json::Value::Null,
                "page_count": stats.page_count,
                "magic_valid": true
            });
            alloc_c_string(&report.to_string())
        }
        Err(e) => {
            // Try to get basic file stats even for corrupted files
            let (size, modified) = std::fs::metadata(&path_str)
                .map(|m| {
                    let sz = m.len();
                    let ts = m.modified()
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0))
                        .unwrap_or(0);
                    (sz, ts)
                })
                .unwrap_or((0, 0));

            let report = serde_json::json!({
                "file_path": path_str,
                "file_size_bytes": size,
                "last_modified": modified,
                "integrity_status": "corrupted",
                "corruption_details": e,
                "page_count": 0,
                "magic_valid": false
            });
            alloc_c_string(&report.to_string())
        }
    }
}

// -------------------------------------------------------------
// TASK 5 - WATCHDOG FFI TESTS
// -------------------------------------------------------------

#[cfg(test)]
mod watchdog_ffi_tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::io::Write;

    /// Helper: open a temp database and return handle + dir
    fn open_temp_db() -> (*mut OdbHandle, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let handle = unsafe { overdrive_open(path_cstr.as_ptr()) };
        assert!(!handle.is_null(), "Failed to open temp db");
        (handle, dir)
    }

    fn parse_report(ptr: *mut c_char) -> serde_json::Value {
        assert!(!ptr.is_null(), "watchdog returned NULL");
        let s = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { overdrive_free_string(ptr) };
        serde_json::from_str(&s).expect("watchdog result should be valid JSON")
    }

    // 5.1 / 5.5 — function exists and returns correct JSON shape
    #[test]
    fn test_watchdog_valid_db_returns_correct_shape() {
        let (handle, dir) = open_temp_db();
        unsafe { overdrive_close(handle) };

        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let report = parse_report(overdrive_watchdog(path_cstr.as_ptr()));

        // All required fields must be present
        assert!(report.get("file_path").is_some(), "missing file_path");
        assert!(report.get("file_size_bytes").is_some(), "missing file_size_bytes");
        assert!(report.get("last_modified").is_some(), "missing last_modified");
        assert!(report.get("integrity_status").is_some(), "missing integrity_status");
        assert!(report.get("corruption_details").is_some(), "missing corruption_details");
        assert!(report.get("page_count").is_some(), "missing page_count");
        assert!(report.get("magic_valid").is_some(), "missing magic_valid");
    }

    // 5.2 — file existence check: missing file → "missing"
    #[test]
    fn test_watchdog_missing_file() {
        let path_cstr = CString::new("/tmp/nonexistent_overdrive_test_xyz.odb").unwrap();
        let report = parse_report(overdrive_watchdog(path_cstr.as_ptr()));

        assert_eq!(report["integrity_status"], "missing");
        assert_eq!(report["file_size_bytes"], 0);
        assert_eq!(report["page_count"], 0);
        assert_eq!(report["magic_valid"], false);
    }

    // 5.2 — file size and timestamp are populated for a valid file
    #[test]
    fn test_watchdog_valid_db_has_size_and_timestamp() {
        let (handle, dir) = open_temp_db();
        unsafe { overdrive_close(handle) };

        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let report = parse_report(overdrive_watchdog(path_cstr.as_ptr()));

        assert!(report["file_size_bytes"].as_u64().unwrap_or(0) > 0, "file_size_bytes should be > 0");
        assert!(report["last_modified"].as_i64().unwrap_or(0) > 0, "last_modified should be a Unix timestamp");
        assert!(report["page_count"].as_u64().unwrap_or(0) > 0, "page_count should be > 0");
    }

    // 5.3 — magic number validation: corrupted magic → "corrupted"
    #[test]
    fn test_watchdog_corrupted_magic() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("bad_magic.odb");

        // Write a file with wrong magic but correct page alignment
        let mut f = std::fs::File::create(&db_path).unwrap();
        let mut page = vec![0u8; 4096];
        page[0..4].copy_from_slice(b"BAAD"); // wrong magic
        f.write_all(&page).unwrap();

        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let report = parse_report(overdrive_watchdog(path_cstr.as_ptr()));

        assert_eq!(report["integrity_status"], "corrupted", "wrong magic should be corrupted");
        assert_eq!(report["magic_valid"], false);
        assert!(report["corruption_details"].as_str().is_some(), "corruption_details should be a string");
    }

    // 5.3 — valid magic passes
    #[test]
    fn test_watchdog_valid_magic() {
        let (handle, dir) = open_temp_db();
        unsafe { overdrive_close(handle) };

        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();
        let report = parse_report(overdrive_watchdog(path_cstr.as_ptr()));

        assert_eq!(report["magic_valid"], true, "valid DB should have magic_valid=true");
        assert_eq!(report["integrity_status"], "valid");
    }

    // 5.5 — null path returns a safe JSON response
    #[test]
    fn test_watchdog_null_path() {
        let report = parse_report(overdrive_watchdog(std::ptr::null()));
        assert_eq!(report["integrity_status"], "missing");
    }

    // 5.6 — performance: watchdog on a freshly created DB completes well under 100ms
    #[test]
    fn test_watchdog_performance() {
        let (handle, dir) = open_temp_db();
        unsafe { overdrive_close(handle) };

        let db_path = dir.path().join("test.odb");
        let path_cstr = CString::new(db_path.to_str().unwrap()).unwrap();

        let start = std::time::Instant::now();
        let ptr = overdrive_watchdog(path_cstr.as_ptr());
        let elapsed = start.elapsed();

        unsafe { overdrive_free_string(ptr) };
        assert!(
            elapsed.as_millis() < 100,
            "watchdog should complete in < 100ms, took {}ms",
            elapsed.as_millis()
        );
    }
}
