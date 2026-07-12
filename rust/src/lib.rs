//! OverDrive-DB Rust SDK
//!
//! Zero-config embedded document database with SQL, MVCC transactions,
//! and 6 storage engines.
//!
//! # Quick Start
//! ```rust,no_run
//! use overdrive::OverdriveDb;
//! use serde_json::json;
//!
//! let mut odb = OverdriveDb::open("app.odb").unwrap();
//! odb.create_table("users").unwrap();
//! let id = odb.insert("users", &json!({"name":"Alice","age":30})).unwrap();
//! let doc = odb.get("users", &id).unwrap();
//! println!("{}", doc.unwrap());
//! odb.close().unwrap();
//! ```

pub mod ffi;
pub mod error;

pub use error::{Result, SdkError};

use std::os::raw::c_char;
use serde_json::Value;
use libloading::Symbol;

/// MVCC transaction isolation levels.
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    ReadUncommitted = 0,
    ReadCommitted   = 1,
    RepeatableRead  = 2,
    Serializable    = 3,
}

/// A live MVCC transaction handle.
pub struct Transaction {
    pub id: u64,
}

/// Options for opening a database.
#[derive(Default)]
pub struct OpenOptions {
    pub password: Option<String>,
    pub engine:   Option<String>,
    pub auto_create_tables: bool,
}

impl OpenOptions {
    pub fn new() -> Self { Self { auto_create_tables: true, ..Default::default() } }
    pub fn password(mut self, p: &str) -> Self { self.password = Some(p.into()); self }
    pub fn engine(mut self, e: &str)   -> Self { self.engine   = Some(e.into()); self }
}

// ── Internal helper ──────────────────────────────────────────────────────────
//
// BEFORE (whole-file issue): every single method below did
//     let lib = ffi::load();                              // panics if lib missing
//     let func: Symbol<...> = lib.get(b"...").unwrap();    // panics if symbol missing
// i.e. TWO independent panic sources on every call, duplicated ~20 times.
// A stale/mismatched native .so/.dll (a very real scenario: SDK crate
// updated via `cargo update` while the bundled native binary lags behind,
// or OVERDRIVE_LIB_PATH pointing at an old build) turned every DB call into
// an `unwrap` panic that took the whole process down — including inside
// library code embedded in someone else's server, where a panic in a
// non-`catch_unwind`-wrapped thread is fatal.
//
// AFTER: `last_error()` and every method return `Result`, using
// `ffi::try_load()` / `ffi::try_sym()` / `ffi::try_to_cstr()` and the `?`
// operator instead of `.unwrap()`.

fn last_error() -> String {
    unsafe {
        match ffi::try_sym::<unsafe extern "C" fn() -> *const c_char>(b"overdrive_last_error\0") {
            Ok(func) => ffi::read_static(func()),
            Err(e) => e.0,
        }
    }
}

// ── Main struct ───────────────────────────────────────────────────────────────

/// OverDrive-DB embedded database handle.
pub struct OverdriveDb {
    handle: *mut std::ffi::c_void,
}

unsafe impl Send for OverdriveDb {}

impl Drop for OverdriveDb {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                // Best-effort on drop: if the native lib/symbol can't be
                // resolved here there's nothing more we can do (Drop can't
                // return Result), but we no longer panic during unwind —
                // panicking inside Drop while already unwinding aborts the
                // process.
                if let Ok(func) = ffi::try_sym::<unsafe extern "C" fn(*mut std::ffi::c_void)>(b"overdrive_close\0") {
                    func(self.handle);
                }
            }
            self.handle = std::ptr::null_mut();
        }
    }
}

impl OverdriveDb {
    // ── Lifecycle ─────────────────────────────────────────────────────────

    /// Open or create a database at `path`.
    pub fn open(path: &str) -> Result<Self> {
        let c_path = ffi::try_to_cstr(path)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*const c_char) -> *mut std::ffi::c_void> =
                ffi::try_sym(b"overdrive_open\0")?;
            let handle = func(c_path.as_ptr());
            if handle.is_null() {
                return Err(SdkError(last_error()));
            }
            Ok(Self { handle })
        }
    }

    /// Open with engine and/or password options.
    pub fn open_with_options(path: &str, opts: OpenOptions) -> Result<Self> {
        let c_path   = ffi::try_to_cstr(path)?;
        let engine   = opts.engine.as_deref().unwrap_or("Disk");
        let c_engine = ffi::try_to_cstr(engine)?;
        let options  = serde_json::json!({
            "password":           opts.password,
            "auto_create_tables": opts.auto_create_tables,
        });
        let c_opts = ffi::try_to_cstr(&options.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> *mut std::ffi::c_void> =
                ffi::try_sym(b"overdrive_open_with_engine\0")?;
            let handle = func(c_path.as_ptr(), c_engine.as_ptr(), c_opts.as_ptr());
            if handle.is_null() {
                return Err(SdkError(last_error()));
            }
            Ok(Self { handle })
        }
    }

    /// Flush all pending writes to disk.
    pub fn sync(&self) -> Result<()> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)> =
                ffi::try_sym(b"overdrive_sync\0")?;
            func(self.handle);
        }
        Ok(())
    }

    /// Close the database explicitly (also called on Drop).
    pub fn close(mut self) -> Result<()> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)> =
                ffi::try_sym(b"overdrive_close\0")?;
            func(self.handle);
            self.handle = std::ptr::null_mut();
        }
        Ok(())
    }

    /// Return the native library version string.
    pub fn version() -> Result<String> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *const c_char> =
                ffi::try_sym(b"overdrive_version\0")?;
            Ok(ffi::read_static(func()))
        }
    }

    // ── Tables ────────────────────────────────────────────────────────────

    /// Create a table.
    pub fn create_table(&mut self, name: &str) -> Result<()> {
        let c_name = ffi::try_to_cstr(name)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                ffi::try_sym(b"overdrive_create_table\0")?;
            if func(self.handle, c_name.as_ptr()) != 0 {
                return Err(SdkError(last_error()));
            }
        }
        Ok(())
    }

    /// Drop a table.
    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        let c_name = ffi::try_to_cstr(name)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                ffi::try_sym(b"overdrive_drop_table\0")?;
            if func(self.handle, c_name.as_ptr()) != 0 {
                return Err(SdkError(last_error()));
            }
        }
        Ok(())
    }

    /// List all tables.
    pub fn list_tables(&self) -> Result<Vec<String>> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> *mut c_char> =
                ffi::try_sym(b"overdrive_list_tables\0")?;
            let ptr = func(self.handle);
            if ptr.is_null() { return Err(SdkError(last_error())); }
            let s = ffi::read_and_free(ptr)?;
            Ok(serde_json::from_str(&s)?)
        }
    }

    /// Check if a table exists.
    pub fn table_exists(&self, name: &str) -> Result<bool> {
        let c_name = ffi::try_to_cstr(name)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                ffi::try_sym(b"overdrive_table_exists\0")?;
            Ok(func(self.handle, c_name.as_ptr()) == 1)
        }
    }

    // ── CRUD ─────────────────────────────────────────────────────────────

    /// Insert a JSON document. Returns the generated `_id`.
    pub fn insert(&mut self, table: &str, doc: &Value) -> Result<String> {
        let c_table = ffi::try_to_cstr(table)?;
        let c_json  = ffi::try_to_cstr(&doc.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> *mut c_char> =
                ffi::try_sym(b"overdrive_insert\0")?;
            let ptr = func(self.handle, c_table.as_ptr(), c_json.as_ptr());
            if ptr.is_null() { return Err(SdkError(last_error())); }
            ffi::read_and_free(ptr)
        }
    }

    /// Insert multiple documents. Returns a Vec of generated `_id`s.
    ///
    /// NOTE (pre-existing, still true): this is NOT atomic — it loops and
    /// calls `insert()` per document. If document N fails, documents
    /// 0..N-1 are already committed to the table with no rollback. Wrap in
    /// `transaction()` if you need all-or-nothing semantics.
    pub fn insert_batch(&mut self, table: &str, docs: &[Value]) -> Result<Vec<String>> {
        docs.iter().map(|doc| self.insert(table, doc)).collect()
    }

    /// Get a document by `_id`. Returns `None` if not found.
    pub fn get(&self, table: &str, id: &str) -> Result<Option<Value>> {
        let c_table = ffi::try_to_cstr(table)?;
        let c_id    = ffi::try_to_cstr(id)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> *mut c_char> =
                ffi::try_sym(b"overdrive_get\0")?;
            let ptr = func(self.handle, c_table.as_ptr(), c_id.as_ptr());
            if ptr.is_null() { return Ok(None); }
            let s = ffi::read_and_free(ptr)?;
            Ok(Some(serde_json::from_str(&s)?))
        }
    }

    /// Update a document by `_id`. Returns `true` if updated.
    pub fn update(&mut self, table: &str, id: &str, patch: &Value) -> Result<bool> {
        let c_table = ffi::try_to_cstr(table)?;
        let c_id    = ffi::try_to_cstr(id)?;
        let c_json  = ffi::try_to_cstr(&patch.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char, *const c_char) -> i32> =
                ffi::try_sym(b"overdrive_update\0")?;
            match func(self.handle, c_table.as_ptr(), c_id.as_ptr(), c_json.as_ptr()) {
                1  => Ok(true),
                0  => Ok(false),
                _  => Err(SdkError(last_error())),
            }
        }
    }

    /// Delete a document by `_id`. Returns `true` if deleted.
    pub fn delete(&mut self, table: &str, id: &str) -> Result<bool> {
        let c_table = ffi::try_to_cstr(table)?;
        let c_id    = ffi::try_to_cstr(id)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> i32> =
                ffi::try_sym(b"overdrive_delete\0")?;
            match func(self.handle, c_table.as_ptr(), c_id.as_ptr()) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(SdkError(last_error())),
            }
        }
    }

    /// Count documents in a table.
    pub fn count(&self, table: &str) -> Result<usize> {
        let c_table = ffi::try_to_cstr(table)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                ffi::try_sym(b"overdrive_count\0")?;
            let n = func(self.handle, c_table.as_ptr());
            if n < 0 { return Err(SdkError(last_error())); }
            Ok(n as usize)
        }
    }

    // ── Query ─────────────────────────────────────────────────────────────

    /// Execute a SQL query. Returns rows as a Vec of JSON Values.
    pub fn query(&mut self, sql: &str) -> Result<Vec<Value>> {
        let c_sql = ffi::try_to_cstr(sql)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> *mut c_char> =
                ffi::try_sym(b"overdrive_query\0")?;
            let ptr = func(self.handle, c_sql.as_ptr());
            if ptr.is_null() { return Err(SdkError(last_error())); }
            let s = ffi::read_and_free(ptr)?;
            let v: Value = serde_json::from_str(&s)?;
            // Response: {"rows":[...], "ok": true}  or  {"result":"...", "ok": true}
            if let Some(rows) = v.get("rows").and_then(|r| r.as_array()) {
                return Ok(rows.clone());
            }
            Ok(vec![v])
        }
    }

    /// Full-text search across a table.
    pub fn search(&self, table: &str, text: &str) -> Result<Vec<Value>> {
        let c_table = ffi::try_to_cstr(table)?;
        let c_text  = ffi::try_to_cstr(text)?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> *mut c_char> =
                ffi::try_sym(b"overdrive_search\0")?;
            let ptr = func(self.handle, c_table.as_ptr(), c_text.as_ptr());
            if ptr.is_null() { return Ok(vec![]); }
            let s = ffi::read_and_free(ptr)?;
            Ok(serde_json::from_str(&s).unwrap_or_default())
        }
    }

    // ── Transactions ─────────────────────────────────────────────────────

    /// Begin an MVCC transaction.
    pub fn begin_transaction(&mut self, iso: IsolationLevel) -> Result<Transaction> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, i32) -> u64> =
                ffi::try_sym(b"overdrive_begin_transaction\0")?;
            let id = func(self.handle, iso as i32);
            if id == 0 { return Err(SdkError(last_error())); }
            Ok(Transaction { id })
        }
    }

    /// Commit a transaction.
    pub fn commit_transaction(&mut self, txn: &Transaction) -> Result<()> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, u64) -> i32> =
                ffi::try_sym(b"overdrive_commit_transaction\0")?;
            if func(self.handle, txn.id) != 0 {
                return Err(SdkError(last_error()));
            }
        }
        Ok(())
    }

    /// Abort (rollback) a transaction.
    pub fn abort_transaction(&mut self, txn: &Transaction) -> Result<()> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, u64) -> i32> =
                ffi::try_sym(b"overdrive_abort_transaction\0")?;
            if func(self.handle, txn.id) != 0 {
                return Err(SdkError(last_error()));
            }
        }
        Ok(())
    }

    /// Run a closure inside a transaction — auto commits on Ok, aborts on Err.
    pub fn transaction<F, T>(&mut self, iso: IsolationLevel, f: F) -> Result<T>
    where F: FnOnce(&mut Self) -> Result<T>
    {
        let txn = self.begin_transaction(iso)?;
        match f(self) {
            Ok(v)  => { self.commit_transaction(&txn)?; Ok(v) }
            Err(e) => { let _ = self.abort_transaction(&txn); Err(e) }
        }
    }

    // ── Integrity ─────────────────────────────────────────────────────────

    /// Run an integrity check. Returns a JSON report.
    pub fn verify_integrity(&self) -> Result<Value> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> *mut c_char> =
                ffi::try_sym(b"overdrive_verify_integrity\0")?;
            let ptr = func(self.handle);
            if ptr.is_null() { return Err(SdkError(last_error())); }
            let s = ffi::read_and_free(ptr)?;
            Ok(serde_json::from_str(&s)?)
        }
    }
}
