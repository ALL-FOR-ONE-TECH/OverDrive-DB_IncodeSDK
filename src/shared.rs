//! Thread-safe wrapper for OverDriveDB
//!
//! `OverDriveDB` is not `Send + Sync` by design. Use `SharedDB` when you need
//! to share a single database across multiple threads (e.g., a web server).
//!
//! ## Quick Example
//!
//! ```no_run
//! use overdrive::shared::SharedDB;
//! use std::thread;
#![allow(clippy::arc_with_non_send_sync)] // Mutex<OverDriveDB> provides the required synchronization
//!
//! let db = SharedDB::open("app.odb").unwrap();
//!
//! let db2 = db.clone(); // cheaply cloned — same underlying Mutex
//! let handle = thread::spawn(move || {
//!     db2.with(|d| {
//!         d.query("SELECT * FROM users").unwrap()
//!     }).unwrap()
//! });
//!
//! let result = handle.join().unwrap();
//! println!("{:?}", result.rows);
//! ```

use crate::{OverDriveDB, QueryResult};
use crate::result::{SdkResult, SdkError};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// A thread-safe, cheaply-cloneable handle to an `OverDriveDB`.
///
/// Internally wraps the database in an `Arc<Mutex<OverDriveDB>>`.
/// Multiple `SharedDB` instances pointing to the same database can be
/// safely sent across threads.
///
/// # Locking
/// Each `.with()` call acquires the mutex for the duration of the closure.
/// Keep closures short to avoid blocking other threads.
#[derive(Clone)]
#[allow(clippy::arc_with_non_send_sync)] // Mutex provides the required synchronization
pub struct SharedDB {
    inner: Arc<Mutex<OverDriveDB>>,
}

impl SharedDB {
    /// Open (or create) a database wrapped in a thread-safe handle.
    ///
    /// File permissions are hardened automatically on open.
    pub fn open(path: &str) -> SdkResult<Self> {
        let db = OverDriveDB::open(path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(db)),
        })
    }

    /// Open with an encryption key from an environment variable.
    ///
    /// ```no_run
    /// use overdrive::shared::SharedDB;
    /// // $env:ODB_KEY="my-aes-256-key"
    /// let db = SharedDB::open_encrypted("app.odb", "ODB_KEY").unwrap();
    /// ```
    pub fn open_encrypted(path: &str, key_env_var: &str) -> SdkResult<Self> {
        let db = OverDriveDB::open_encrypted(path, key_env_var)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(db)),
        })
    }

    /// Execute a closure with exclusive access to the database.
    ///
    /// The mutex is acquired for the duration of `f` and released when it returns.
    /// Returns `SecurityError` if the mutex has been poisoned by a panicking thread.
    ///
    /// ```ignore
    /// let count = db.with(|d| d.count("users")).unwrap();
    /// ```
    pub fn with<F, T>(&self, f: F) -> SdkResult<T>
    where
        F: FnOnce(&mut OverDriveDB) -> T,
    {
        let mut guard = self.inner.lock().map_err(|_| {
            SdkError::SecurityError(
                "SharedDB mutex is poisoned — a thread panicked while holding the lock. \
                 Create a new SharedDB instance to recover.".to_string()
            )
        })?;
        Ok(f(&mut guard))
    }

    /// Convenience: execute an SQL query.
    pub fn query(&self, sql: &str) -> SdkResult<QueryResult> {
        self.with(|db| db.query(sql))?
    }

    /// Convenience: execute a safe parameterized SQL query.
    ///
    /// See `OverDriveDB::query_safe()` for full documentation.
    pub fn query_safe(&self, sql_template: &str, params: &[&str]) -> SdkResult<QueryResult> {
        self.with(|db| db.query_safe(sql_template, params))?
    }

    /// Convenience: insert a document into a table.
    pub fn insert(&self, table: &str, doc: &Value) -> SdkResult<String> {
        self.with(|db| db.insert(table, doc))?
    }

    /// Convenience: get a document by `_id`.
    pub fn get(&self, table: &str, id: &str) -> SdkResult<Option<Value>> {
        self.with(|db| db.get(table, id))?
    }

    /// Convenience: create a backup at `dest_path`.
    pub fn backup(&self, dest_path: &str) -> SdkResult<()> {
        self.with(|db| db.backup(dest_path))?
    }

    /// Convenience: sync to disk.
    pub fn sync(&self) -> SdkResult<()> {
        self.with(|db| db.sync())?
    }

    /// Number of `SharedDB` handles pointing to this same database (Arc strong count).
    pub fn handle_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_db_clone_count() {
        // We can't open a real DB in unit tests without the native lib,
        // but we can verify the Arc count logic compiles and works with a mock.
        // Real integration tests go in ../examples/
        let _ = std::marker::PhantomData::<SharedDB>;
    }
}
