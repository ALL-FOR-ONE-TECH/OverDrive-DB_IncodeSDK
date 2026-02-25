//! C FFI Layer for OverDrive InCode SDK
//! 
//! Provides C-compatible functions for cross-language bindings.
//! All functions use `extern "C"` with C-compatible types.
//! 
//! ## Memory Rules
//! - All `*mut c_char` returned must be freed with `overdrive_free_string()`
//! - `*mut ODB` must be closed with `overdrive_close()`
//! - Errors are stored thread-locally, retrieve with `overdrive_last_error()`

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::cell::RefCell;

use crate::OverDriveDB;

/// Opaque database handle for C API
pub struct ODB {
    inner: OverDriveDB,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_error(msg: String) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

fn clear_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

unsafe fn from_c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LIFECYCLE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Open (or create) a database. Returns NULL on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_open(path: *const c_char) -> *mut ODB {
    clear_error();
    let path = match from_c_str(path) {
        Some(p) => p,
        None => { set_error("Invalid path".into()); return ptr::null_mut(); }
    };
    match OverDriveDB::open(&path) {
        Ok(db) => Box::into_raw(Box::new(ODB { inner: db })),
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

/// Close a database and free resources.
#[no_mangle]
pub unsafe extern "C" fn overdrive_close(db: *mut ODB) {
    if !db.is_null() {
        let odb = Box::from_raw(db);
        let _ = odb.inner.close();
    }
}

/// Sync data to disk.
#[no_mangle]
pub unsafe extern "C" fn overdrive_sync(db: *mut ODB) {
    if db.is_null() { return; }
    let _ = (*db).inner.sync();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TABLE MANAGEMENT
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a table. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_create_table(db: *mut ODB, name: *const c_char) -> i32 {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return -1; }
    let name = match from_c_str(name) {
        Some(n) => n,
        None => { set_error("Invalid table name".into()); return -1; }
    };
    match (*db).inner.create_table(&name) {
        Ok(()) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

/// Drop a table. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_drop_table(db: *mut ODB, name: *const c_char) -> i32 {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return -1; }
    let name = match from_c_str(name) {
        Some(n) => n,
        None => { set_error("Invalid table name".into()); return -1; }
    };
    match (*db).inner.drop_table(&name) {
        Ok(()) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

/// List all tables. Returns JSON array string. Must be freed with overdrive_free_string().
#[no_mangle]
pub unsafe extern "C" fn overdrive_list_tables(db: *mut ODB) -> *mut c_char {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return ptr::null_mut(); }
    match (*db).inner.list_tables() {
        Ok(tables) => {
            let json = serde_json::to_string(&tables).unwrap_or_else(|_| "[]".into());
            to_c_string(&json)
        }
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

/// Check if table exists. Returns 1 if exists, 0 if not, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_table_exists(db: *mut ODB, name: *const c_char) -> i32 {
    clear_error();
    if db.is_null() { return -1; }
    let name = match from_c_str(name) {
        Some(n) => n,
        None => { return -1; }
    };
    match (*db).inner.table_exists(&name) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CRUD OPERATIONS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Insert a JSON document. Returns the _id string. Must be freed.
#[no_mangle]
pub unsafe extern "C" fn overdrive_insert(db: *mut ODB, table: *const c_char, json: *const c_char) -> *mut c_char {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return ptr::null_mut(); }
    let table = match from_c_str(table) { Some(t) => t, None => { set_error("Invalid table".into()); return ptr::null_mut(); } };
    let json_str = match from_c_str(json) { Some(j) => j, None => { set_error("Invalid JSON".into()); return ptr::null_mut(); } };
    
    let value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => { set_error(format!("JSON parse error: {}", e)); return ptr::null_mut(); }
    };
    
    match (*db).inner.insert(&table, &value) {
        Ok(id) => to_c_string(&id),
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

/// Get a document by ID. Returns JSON string. Must be freed.
#[no_mangle]
pub unsafe extern "C" fn overdrive_get(db: *mut ODB, table: *const c_char, id: *const c_char) -> *mut c_char {
    clear_error();
    if db.is_null() { return ptr::null_mut(); }
    let table = match from_c_str(table) { Some(t) => t, None => return ptr::null_mut() };
    let id = match from_c_str(id) { Some(i) => i, None => return ptr::null_mut() };
    
    match (*db).inner.get(&table, &id) {
        Ok(Some(val)) => to_c_string(&val.to_string()),
        Ok(None) => ptr::null_mut(),
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

/// Update a document. Returns 1 if updated, 0 if not found, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_update(db: *mut ODB, table: *const c_char, id: *const c_char, json: *const c_char) -> i32 {
    clear_error();
    if db.is_null() { return -1; }
    let table = match from_c_str(table) { Some(t) => t, None => return -1 };
    let id = match from_c_str(id) { Some(i) => i, None => return -1 };
    let json_str = match from_c_str(json) { Some(j) => j, None => return -1 };
    
    let value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => { set_error(format!("JSON parse error: {}", e)); return -1; }
    };
    
    match (*db).inner.update(&table, &id, &value) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

/// Delete a document. Returns 1 if deleted, 0 if not found, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_delete(db: *mut ODB, table: *const c_char, id: *const c_char) -> i32 {
    clear_error();
    if db.is_null() { return -1; }
    let table = match from_c_str(table) { Some(t) => t, None => return -1 };
    let id = match from_c_str(id) { Some(i) => i, None => return -1 };
    
    match (*db).inner.delete(&table, &id) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

/// Count documents in a table. Returns count or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_count(db: *mut ODB, table: *const c_char) -> i32 {
    clear_error();
    if db.is_null() { return -1; }
    let table = match from_c_str(table) { Some(t) => t, None => return -1 };
    
    match (*db).inner.count(&table) {
        Ok(n) => n as i32,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// QUERY
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Execute SQL query. Returns JSON result string. Must be freed.
#[no_mangle]
pub unsafe extern "C" fn overdrive_query(db: *mut ODB, sql: *const c_char) -> *mut c_char {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return ptr::null_mut(); }
    let sql = match from_c_str(sql) { Some(s) => s, None => return ptr::null_mut() };
    
    match (*db).inner.query(&sql) {
        Ok(result) => {
            let json = serde_json::json!({
                "rows": result.rows,
                "columns": result.columns,
                "rows_affected": result.rows_affected,
                "execution_time_ms": result.execution_time_ms,
            });
            to_c_string(&json.to_string())
        }
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

/// Full-text search. Returns JSON array. Must be freed.
#[no_mangle]
pub unsafe extern "C" fn overdrive_search(db: *mut ODB, table: *const c_char, text: *const c_char) -> *mut c_char {
    clear_error();
    if db.is_null() { return ptr::null_mut(); }
    let table = match from_c_str(table) { Some(t) => t, None => return ptr::null_mut() };
    let text = match from_c_str(text) { Some(t) => t, None => return ptr::null_mut() };
    
    match (*db).inner.search(&table, &text) {
        Ok(results) => {
            let json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".into());
            to_c_string(&json)
        }
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ERROR & UTILITY
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Get the last error message. Returns NULL if no error.
/// The returned string is valid until the next API call.
#[no_mangle]
pub unsafe extern "C" fn overdrive_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        match &*e.borrow() {
            Some(msg) => {
                // Leak a CString — caller should not free this
                let cs = CString::new(msg.as_str()).unwrap_or_default();
                cs.into_raw() as *const c_char
            }
            None => ptr::null(),
        }
    })
}

/// Free a string returned by the SDK.
#[no_mangle]
pub unsafe extern "C" fn overdrive_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Get SDK version string.
#[no_mangle]
pub extern "C" fn overdrive_version() -> *const c_char {
    // Static string, no need to free
    b"1.2.0\0".as_ptr() as *const c_char
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MVCC TRANSACTIONS (Phase 5)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Begin a transaction. isolation_level: 0=ReadUncommitted, 1=ReadCommitted, 2=RepeatableRead, 3=Serializable.
/// Returns a transaction ID (>0) on success, 0 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_begin_transaction(db: *mut ODB, isolation_level: i32) -> u64 {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return 0; }
    let isolation = crate::IsolationLevel::from_i32(isolation_level);
    match (*db).inner.begin_transaction(isolation) {
        Ok(txn) => txn.txn_id,
        Err(e) => { set_error(e.to_string()); 0 }
    }
}

/// Commit a transaction. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_commit_transaction(db: *mut ODB, txn_id: u64) -> i32 {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return -1; }
    let txn = crate::TransactionHandle {
        txn_id,
        isolation: crate::IsolationLevel::ReadCommitted,
        active: true,
    };
    match (*db).inner.commit_transaction(&txn) {
        Ok(()) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

/// Abort (rollback) a transaction. Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn overdrive_abort_transaction(db: *mut ODB, txn_id: u64) -> i32 {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return -1; }
    let txn = crate::TransactionHandle {
        txn_id,
        isolation: crate::IsolationLevel::ReadCommitted,
        active: true,
    };
    match (*db).inner.abort_transaction(&txn) {
        Ok(()) => 0,
        Err(e) => { set_error(e.to_string()); -1 }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// INTEGRITY VERIFICATION (Phase 5)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Verify database integrity. Returns JSON report string. Must be freed.
#[no_mangle]
pub unsafe extern "C" fn overdrive_verify_integrity(db: *mut ODB) -> *mut c_char {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return ptr::null_mut(); }
    match (*db).inner.verify_integrity() {
        Ok(report) => {
            let json = serde_json::json!({
                "valid": report.is_valid,
                "pages_checked": report.pages_checked,
                "tables_verified": report.tables_verified,
                "issues": report.issues,
            });
            to_c_string(&json.to_string())
        }
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

/// Get extended database stats. Returns JSON string. Must be freed.
#[no_mangle]
pub unsafe extern "C" fn overdrive_stats(db: *mut ODB) -> *mut c_char {
    clear_error();
    if db.is_null() { set_error("Null db handle".into()); return ptr::null_mut(); }
    match (*db).inner.stats() {
        Ok(stats) => {
            let json = serde_json::json!({
                "tables": stats.tables,
                "total_records": stats.total_records,
                "file_size_bytes": stats.file_size_bytes,
                "path": stats.path,
                "mvcc_active_versions": stats.mvcc_active_versions,
                "page_size": stats.page_size,
                "sdk_version": stats.sdk_version,
            });
            to_c_string(&json.to_string())
        }
        Err(e) => { set_error(e.to_string()); ptr::null_mut() }
    }
}

