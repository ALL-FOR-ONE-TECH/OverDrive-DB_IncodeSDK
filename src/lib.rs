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
pub mod ffi;
mod dynamic;

use result::{SdkResult, SdkError};
use dynamic::NativeDB;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;

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

/// Database statistics
#[derive(Debug, Clone)]
pub struct Stats {
    pub tables: usize,
    pub total_records: usize,
    pub file_size_bytes: u64,
    pub path: String,
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
    /// This is the main entry point for the SDK. If the file exists,
    /// it opens the existing database. If not, it creates a new one.
    /// 
    /// ```no_run
    /// # use overdrive::OverDriveDB;
    /// let mut db = OverDriveDB::open("myapp.odb").unwrap();
    /// ```
    pub fn open(path: &str) -> SdkResult<Self> {
        let native = NativeDB::open(path)?;
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

    /// Get database statistics.
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
        })
    }
}

impl Drop for OverDriveDB {
    fn drop(&mut self) {
        self.native.close();
    }
}
