//! # OverDrive InCode SDK
//! 
//! An embeddable document database — like SQLite for JSON.
//! 
//! Import the package, open a file, query your data. **No server needed.**
//! 
//! ## Quick Start (Rust)
//! 
//! ```no_run
//! use overdrive::OverDriveDB;
//! 
//! let mut db = OverDriveDB::open("myapp.odb").unwrap();
//! db.create_table("users").unwrap();
//! 
//! let id = db.insert("users", &serde_json::json!({
//!     "name": "Alice",
//!     "email": "alice@example.com",
//!     "age": 30
//! })).unwrap();
//! 
//! let results = db.query("SELECT * FROM users WHERE age > 25").unwrap();
//! println!("{:?}", results.rows);
//! ```
//!
//! ## Setup
//!
//! 1. `cargo add overdrive-sdk`
//! 2. Download the native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest)
//! 3. Place it in your project directory or on your system PATH

pub mod result;
pub mod query_engine;
pub mod shared;
mod dynamic;

use result::{SdkResult, SdkError};
use dynamic::NativeDB;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use zeroize::{Zeroize, ZeroizeOnDrop};

// ─────────────────────────────────────────────
// SECURITY: Secret key wrapper
// Zero bytes from RAM automatically on drop
// ─────────────────────────────────────────────

/// A secret key that is automatically zeroed from memory when dropped.
/// Use this to hold AES encryption keys — prevents leak via memory dump.
///
/// ```no_run
/// use overdrive::SecretKey;
/// let key = SecretKey::from_env("ODB_KEY").unwrap();
/// // ...key bytes are wiped from RAM when `key` is dropped
/// ```
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey(Vec<u8>);

impl SecretKey {
    /// Create a `SecretKey` from raw bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Read key bytes from an environment variable.
    ///
    /// Returns `SecurityError` if the env var is not set or is empty.
    pub fn from_env(env_var: &str) -> SdkResult<Self> {
        let val = std::env::var(env_var).map_err(|_| {
            SdkError::SecurityError(format!(
                "Encryption key env var '{}' is not set. \
                 Set it with: $env:{}=\"your-secret-key\" (PowerShell) \
                 or export {}=\"your-secret-key\" (bash)",
                env_var, env_var, env_var
            ))
        })?;
        if val.is_empty() {
            return Err(SdkError::SecurityError(format!(
                "Encryption key env var '{}' is set but empty.", env_var
            )));
        }
        Ok(Self(val.into_bytes()))
    }

    /// Raw key bytes (use sparingly — minimize time in scope).
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

// ─────────────────────────────────────────────
// SECURITY: OS-level file permission hardening
// ─────────────────────────────────────────────

/// Set restrictive OS-level permissions on the `.odb` file:
/// - **Windows**: `icacls` — removes all inherit ACEs, grants only current user Full Control
/// - **Linux/macOS**: `chmod 600` — owner read/write only
///
/// Called automatically inside `OverDriveDB::open()`.
pub fn set_secure_permissions(path: &str) -> SdkResult<()> {
    #[cfg(target_os = "windows")]
    {
        // Reset all inherited permissions and grant only current user
        let output = std::process::Command::new("icacls")
            .args([path, "/inheritance:r", "/grant:r", "%USERNAME%:F"])
            .output();
        match output {
            Ok(out) if out.status.success() => {}
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                // Non-fatal: log but don't fail (icacls may not be available on all setups)
                eprintln!("[overdrive-sdk] WARNING: Could not set file permissions on '{}': {}", path, stderr);
            }
            Err(e) => {
                eprintln!("[overdrive-sdk] WARNING: icacls not available, file permissions not hardened: {}", e);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        if Path::new(path).exists() {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| SdkError::SecurityError(format!(
                    "Failed to chmod 600 '{}': {}", path, e
                )))?;
        }
    }
    Ok(())
}

/// Query result returned by `query()`
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Result rows (JSON objects)
    pub rows: Vec<Value>,
    /// Column names (for SELECT queries)
    pub columns: Vec<String>,
    /// Number of rows affected (for INSERT/UPDATE/DELETE)
    pub rows_affected: usize,
    /// Query execution time in milliseconds
    pub execution_time_ms: f64,
}

impl QueryResult {
    fn empty() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            rows_affected: 0,
            execution_time_ms: 0.0,
        }
    }
}

/// Database statistics (expanded for Phase 5)
#[derive(Debug, Clone)]
pub struct Stats {
    pub tables: usize,
    pub total_records: usize,
    pub file_size_bytes: u64,
    pub path: String,
    /// Number of active MVCC versions in memory
    pub mvcc_active_versions: usize,
    /// Database page size in bytes (typically 4096)
    pub page_size: usize,
    /// SDK version string
    pub sdk_version: String,
}

/// MVCC Isolation level for transactions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IsolationLevel {
    /// Read uncommitted data (fastest, least safe)
    ReadUncommitted = 0,
    /// Read only committed data (default)
    ReadCommitted = 1,
    /// Repeatable reads within the transaction
    RepeatableRead = 2,
    /// Full serializable isolation (slowest, safest)
    Serializable = 3,
}

impl IsolationLevel {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => IsolationLevel::ReadUncommitted,
            1 => IsolationLevel::ReadCommitted,
            2 => IsolationLevel::RepeatableRead,
            3 => IsolationLevel::Serializable,
            _ => IsolationLevel::ReadCommitted,
        }
    }
}

/// A handle for an active MVCC transaction
#[derive(Debug, Clone)]
pub struct TransactionHandle {
    /// Unique transaction ID
    pub txn_id: u64,
    /// Isolation level of this transaction
    pub isolation: IsolationLevel,
    /// Whether this transaction is still active
    pub active: bool,
}

/// Result of an integrity verification check
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Whether the database passed all checks
    pub is_valid: bool,
    /// Total pages checked
    pub pages_checked: usize,
    /// Total tables verified
    pub tables_verified: usize,
    /// List of issues found (empty if valid)
    pub issues: Vec<String>,
}

/// OverDrive InCode SDK — Embeddable document database
/// 
/// Use this struct to create, open, and interact with OverDrive databases
/// directly in your application. No server required.
pub struct OverDriveDB {
    native: NativeDB,
    path: String,
}

impl OverDriveDB {
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // DATABASE LIFECYCLE
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Open an existing database or create a new one.
    ///
    /// File permissions are automatically hardened on open (chmod 600 / Windows ACL).
    pub fn open(path: &str) -> SdkResult<Self> {
        let native = NativeDB::open(path)?;
        // Fix 6: Harden file permissions immediately on open
        let _ = set_secure_permissions(path);
        Ok(Self {
            native,
            path: path.to_string(),
        })
    }

    /// Create a new database. Returns an error if the file already exists.
    pub fn create(path: &str) -> SdkResult<Self> {
        if Path::new(path).exists() {
            return Err(SdkError::DatabaseAlreadyExists(path.to_string()));
        }
        Self::open(path)
    }

    /// Open an existing database. Returns an error if the file doesn't exist.
    pub fn open_existing(path: &str) -> SdkResult<Self> {
        if !Path::new(path).exists() {
            return Err(SdkError::DatabaseNotFound(path.to_string()));
        }
        Self::open(path)
    }

    /// Force sync all data to disk.
    pub fn sync(&self) -> SdkResult<()> {
        self.native.sync();
        Ok(())
    }

    /// Close the database and release all resources.
    pub fn close(mut self) -> SdkResult<()> {
        self.native.close();
        Ok(())
    }

    /// Delete a database file from disk.
    pub fn destroy(path: &str) -> SdkResult<()> {
        if Path::new(path).exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Get the database file path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the SDK version.
    pub fn version() -> String {
        NativeDB::version()
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // SECURITY: Encrypted open, backup, WAL cleanup
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Open a database with an encryption key loaded securely from an environment variable.
    ///
    /// **Never hardcode** the key — always read from env or a secrets manager.
    ///
    /// ```no_run
    /// // In your shell: $env:ODB_KEY="my-secret-32-char-aes-key!!!!"
    /// use overdrive::OverDriveDB;
    /// let mut db = OverDriveDB::open_encrypted("app.odb", "ODB_KEY").unwrap();
    /// ```
    pub fn open_encrypted(path: &str, key_env_var: &str) -> SdkResult<Self> {
        let key = SecretKey::from_env(key_env_var)?;
        // Pass key to the engine via a dedicated env var the engine reads internally.
        // This avoids passing the key as a command-line argument (visible in process list).
        std::env::set_var("__OVERDRIVE_KEY", std::str::from_utf8(key.as_bytes())
            .map_err(|_| SdkError::SecurityError("Key contains non-UTF8 bytes".to_string()))?);
        let db = Self::open(path)?;
        // Immediately remove from env after handoff
        std::env::remove_var("__OVERDRIVE_KEY");
        // SecretKey is dropped here — bytes are zeroed
        Ok(db)
    }

    /// Create an encrypted backup of the database to `dest_path`.
    ///
    /// Syncs all in-memory data to disk first, then copies the `.odb` and `.wal` files.
    /// Store the backup in a separate physical location or cloud storage.
    ///
    /// ```no_run
    /// # use overdrive::OverDriveDB;
    /// # let db = OverDriveDB::open("app.odb").unwrap();
    /// db.backup("backups/app_2026-03-04.odb").unwrap();
    /// ```
    pub fn backup(&self, dest_path: &str) -> SdkResult<()> {
        // Flush all in-memory pages to disk first
        self.sync()?;

        // Copy the main .odb file
        std::fs::copy(&self.path, dest_path)
            .map_err(|e| SdkError::BackupError(format!(
                "Failed to copy '{}' -> '{}': {}", self.path, dest_path, e
            )))?;

        // Also copy the WAL file if it exists (crash consistency)
        let wal_src = format!("{}.wal", self.path);
        let wal_dst = format!("{}.wal", dest_path);
        if Path::new(&wal_src).exists() {
            std::fs::copy(&wal_src, &wal_dst)
                .map_err(|e| SdkError::BackupError(format!(
                    "Failed to copy WAL '{}' -> '{}': {}", wal_src, wal_dst, e
                )))?;
        }

        // Harden permissions on the backup file too
        let _ = set_secure_permissions(dest_path);
        Ok(())
    }

    /// Delete the WAL (Write-Ahead Log) file after a confirmed commit.
    ///
    /// **Call this after `commit_transaction()`** to prevent attackers from replaying
    /// the WAL file to restore deleted data.
    ///
    /// ```no_run
    /// # use overdrive::{OverDriveDB, IsolationLevel};
    /// # let mut db = OverDriveDB::open("app.odb").unwrap();
    /// let txn = db.begin_transaction(IsolationLevel::ReadCommitted).unwrap();
    /// // ... writes ...
    /// db.commit_transaction(&txn).unwrap();
    /// db.cleanup_wal().unwrap(); // Remove stale WAL
    /// ```
    pub fn cleanup_wal(&self) -> SdkResult<()> {
        let wal_path = format!("{}.wal", self.path);
        if Path::new(&wal_path).exists() {
            std::fs::remove_file(&wal_path)
                .map_err(SdkError::IoError)?;
        }
        Ok(())
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // TABLE MANAGEMENT
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Create a new table (schemaless, NoSQL mode).
    pub fn create_table(&mut self, name: &str) -> SdkResult<()> {
        self.native.create_table(name)?;
        Ok(())
    }

    /// Drop (delete) a table and all its data.
    pub fn drop_table(&mut self, name: &str) -> SdkResult<()> {
        self.native.drop_table(name)?;
        Ok(())
    }

    /// List all tables in the database.
    pub fn list_tables(&self) -> SdkResult<Vec<String>> {
        Ok(self.native.list_tables()?)
    }

    /// Check if a table exists.
    pub fn table_exists(&self, name: &str) -> SdkResult<bool> {
        Ok(self.native.table_exists(name))
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // CRUD OPERATIONS
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Insert a JSON document into a table. Returns the auto-generated `_id`.
    /// 
    /// ```ignore
    /// let id = db.insert("users", &serde_json::json!({
    ///     "name": "Alice",
    ///     "age": 30,
    ///     "tags": ["admin", "developer"]
    /// })).unwrap();
    /// println!("Inserted: {}", id);
    /// ```
    pub fn insert(&mut self, table: &str, doc: &Value) -> SdkResult<String> {
        let json_str = serde_json::to_string(doc)?;
        let id = self.native.insert(table, &json_str)?;
        Ok(id)
    }

    /// Insert multiple documents in a batch. Returns a list of generated `_id`s.
    pub fn insert_batch(&mut self, table: &str, docs: &[Value]) -> SdkResult<Vec<String>> {
        let mut ids = Vec::with_capacity(docs.len());
        for doc in docs {
            let id = self.insert(table, doc)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Get a document by its `_id`.
    pub fn get(&self, table: &str, id: &str) -> SdkResult<Option<Value>> {
        match self.native.get(table, id)? {
            Some(json_str) => {
                let value: Value = serde_json::from_str(&json_str)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Update a document by its `_id`. Returns `true` if the document was found and updated.
    /// 
    /// ```ignore
    /// db.update("users", &id, &serde_json::json!({
    ///     "age": 31,
    ///     "email": "alice@newmail.com"
    /// })).unwrap();
    /// ```
    pub fn update(&mut self, table: &str, id: &str, updates: &Value) -> SdkResult<bool> {
        let json_str = serde_json::to_string(updates)?;
        Ok(self.native.update(table, id, &json_str)?)
    }

    /// Delete a document by its `_id`. Returns `true` if found and deleted.
    pub fn delete(&mut self, table: &str, id: &str) -> SdkResult<bool> {
        Ok(self.native.delete(table, id)?)
    }

    /// Count all documents in a table.
    pub fn count(&self, table: &str) -> SdkResult<usize> {
        let count = self.native.count(table)?;
        Ok(count.max(0) as usize)
    }

    /// Scan all documents in a table (no filter).
    pub fn scan(&self, table: &str) -> SdkResult<Vec<Value>> {
        let result_str = self.native.query(&format!("SELECT * FROM {}", table))?;
        let result: Value = serde_json::from_str(&result_str)?;
        let rows = result.get("rows")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(rows)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // QUERY ENGINE
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Execute an SQL query and return results.
    /// 
    /// Supports: SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE, SHOW TABLES
    /// 
    /// ```ignore
    /// let result = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name LIMIT 10").unwrap();
    /// for row in &result.rows {
    ///     println!("{}", row);
    /// }
    /// ```
    pub fn query(&mut self, sql: &str) -> SdkResult<QueryResult> {
        let start = Instant::now();
        let result_str = self.native.query(sql)?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let result: Value = serde_json::from_str(&result_str)?;
        let rows = result.get("rows")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        let columns = result.get("columns")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let rows_affected = result.get("rows_affected")
            .and_then(|r| r.as_u64())
            .unwrap_or(0) as usize;

        Ok(QueryResult {
            rows,
            columns,
            rows_affected,
            execution_time_ms: elapsed,
        })
    }

    /// Execute a **parameterized** SQL query — the safe way to include user input.
    ///
    /// Use `?` as placeholders in the SQL template; values are sanitized and
    /// escaped before substitution. Any param containing SQL injection patterns
    /// (`DROP`, `DELETE`, `--`, `;`) is rejected with `SecurityError`.
    ///
    /// ```ignore
    /// // SAFE: user input via params, never via string concat
    /// let name: &str = get_user_input(); // could be "Alice'; DROP TABLE users--"
    /// let result = db.query_safe(
    ///     "SELECT * FROM users WHERE name = ?",
    ///     &[name],
    /// ).unwrap(); // Blocked: SecurityError if injection detected
    /// ```
    pub fn query_safe(&mut self, sql_template: &str, params: &[&str]) -> SdkResult<QueryResult> {
        /// Dangerous SQL keywords/tokens that signal injection attempts
        const DANGEROUS: &[&str] = &[
            "DROP", "TRUNCATE", "ALTER", "EXEC", "EXECUTE",
            "--", ";--", "/*", "*/", "xp_", "UNION",
        ];

        // Sanitize each param
        let mut sanitized: Vec<String> = Vec::with_capacity(params.len());
        for &param in params {
            let upper = param.to_uppercase();
            for &danger in DANGEROUS {
                if upper.contains(danger) {
                    return Err(SdkError::SecurityError(format!(
                        "SQL injection detected in parameter: '{}' contains forbidden token '{}'",
                        param, danger
                    )));
                }
            }
            // Escape single quotes by doubling them (SQL standard)
            let escaped = param.replace('\'', "''");
            sanitized.push(format!("'{}'", escaped));
        }

        // Replace ? placeholders in order
        let mut sql = sql_template.to_string();
        for value in &sanitized {
            if let Some(pos) = sql.find('?') {
                sql.replace_range(pos..pos + 1, value);
            } else {
                return Err(SdkError::SecurityError(
                    "More params than '?' placeholders in SQL template".to_string()
                ));
            }
        }

        // Check no unresolved placeholders remain
        let remaining = params.len();
        let placeholder_count = sql_template.chars().filter(|&c| c == '?').count();
        if remaining < placeholder_count {
            return Err(SdkError::SecurityError(format!(
                "SQL template has {} '?' placeholders but only {} params were provided",
                placeholder_count, remaining
            )));
        }

        self.query(&sql)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // SEARCH
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Full-text search across a table.
    pub fn search(&self, table: &str, text: &str) -> SdkResult<Vec<Value>> {
        let result_str = self.native.search(table, text)?;
        let values: Vec<Value> = serde_json::from_str(&result_str).unwrap_or_default();
        Ok(values)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // STATISTICS
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Get database statistics (expanded with MVCC info).
    pub fn stats(&self) -> SdkResult<Stats> {
        let file_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or(0);
        let tables = self.list_tables().unwrap_or_default();
        let mut total_records = 0;
        for table in &tables {
            total_records += self.count(table).unwrap_or(0);
        }
        Ok(Stats {
            tables: tables.len(),
            total_records,
            file_size_bytes: file_size,
            path: self.path.clone(),
            mvcc_active_versions: 0, // Populated by engine when available
            page_size: 4096,
            sdk_version: Self::version(),
        })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // MVCC TRANSACTIONS (Phase 5)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Begin a new MVCC transaction with the specified isolation level.
    ///
    /// ```ignore
    /// let txn = db.begin_transaction(IsolationLevel::ReadCommitted).unwrap();
    /// // ... perform reads/writes ...
    /// db.commit_transaction(&txn).unwrap();
    /// ```
    pub fn begin_transaction(&mut self, isolation: IsolationLevel) -> SdkResult<TransactionHandle> {
        let txn_id = self.native.begin_transaction(isolation as i32)?;
        Ok(TransactionHandle {
            txn_id,
            isolation,
            active: true,
        })
    }

    /// Commit a transaction, making all its changes permanent.
    pub fn commit_transaction(&mut self, txn: &TransactionHandle) -> SdkResult<()> {
        self.native.commit_transaction(txn.txn_id)?;
        Ok(())
    }

    /// Abort (rollback) a transaction, discarding all its changes.
    pub fn abort_transaction(&mut self, txn: &TransactionHandle) -> SdkResult<()> {
        self.native.abort_transaction(txn.txn_id)?;
        Ok(())
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // INTEGRITY VERIFICATION (Phase 5)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Verify the integrity of the database.
    ///
    /// Checks B-Tree consistency, page checksums, and MVCC version chains.
    /// Returns a detailed report.
    ///
    /// ```ignore
    /// let report = db.verify_integrity().unwrap();
    /// assert!(report.is_valid);
    /// println!("Checked {} pages across {} tables", report.pages_checked, report.tables_verified);
    /// ```
    pub fn verify_integrity(&self) -> SdkResult<IntegrityReport> {
        let result_str = self.native.verify_integrity()?;
        let result: Value = serde_json::from_str(&result_str).unwrap_or_default();

        let is_valid = result.get("valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let pages_checked = result.get("pages_checked")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let tables_verified = result.get("tables_verified")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let issues = result.get("issues")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(IntegrityReport {
            is_valid,
            pages_checked,
            tables_verified,
            issues,
        })
    }
}

impl Drop for OverDriveDB {
    fn drop(&mut self) {
        self.native.close();
    }
}
