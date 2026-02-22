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

pub mod result;
pub mod query_engine;
pub mod ffi;

use result::{SdkResult, SdkError};
use overdrive_db::storage::Database;
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
    db: Option<Database>,
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
        let db = if Path::new(path).exists() {
            Database::open(path)?
        } else {
            Database::create(path)?
        };
        
        Ok(Self {
            db: Some(db),
            path: path.to_string(),
        })
    }

    /// Create a new database. Returns an error if the file already exists.
    pub fn create(path: &str) -> SdkResult<Self> {
        if Path::new(path).exists() {
            return Err(SdkError::DatabaseAlreadyExists(path.to_string()));
        }
        let db = Database::create(path)?;
        Ok(Self {
            db: Some(db),
            path: path.to_string(),
        })
    }

    /// Open an existing database. Returns an error if the file doesn't exist.
    pub fn open_existing(path: &str) -> SdkResult<Self> {
        if !Path::new(path).exists() {
            return Err(SdkError::DatabaseNotFound(path.to_string()));
        }
        let db = Database::open(path)?;
        Ok(Self {
            db: Some(db),
            path: path.to_string(),
        })
    }

    /// Force sync all data to disk.
    pub fn sync(&self) -> SdkResult<()> {
        let db = self.db()?;
        db.sync()?;
        Ok(())
    }

    /// Close the database and release all resources.
    pub fn close(mut self) -> SdkResult<()> {
        if let Some(db) = self.db.take() {
            db.sync()?;
            drop(db);
        }
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
    pub fn version() -> &'static str {
        "1.0.0"
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // TABLE MANAGEMENT
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Create a new table (schemaless, NoSQL mode).
    pub fn create_table(&mut self, name: &str) -> SdkResult<()> {
        let db = self.db_mut()?;
        if db.list_tables().contains(&name.to_string()) {
            return Err(SdkError::TableAlreadyExists(name.to_string()));
        }
        db.create_table(name)?;
        Ok(())
    }

    /// Drop (delete) a table and all its data.
    pub fn drop_table(&mut self, name: &str) -> SdkResult<()> {
        let db = self.db_mut()?;
        if !db.list_tables().contains(&name.to_string()) {
            return Err(SdkError::TableNotFound(name.to_string()));
        }
        db.drop_table(name)?;
        Ok(())
    }

    /// List all tables in the database.
    pub fn list_tables(&self) -> SdkResult<Vec<String>> {
        let db = self.db()?;
        Ok(db.list_tables())
    }

    /// Check if a table exists.
    pub fn table_exists(&self, name: &str) -> SdkResult<bool> {
        let db = self.db()?;
        Ok(db.list_tables().contains(&name.to_string()))
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
        let db = self.db_mut()?;
        let id = db.insert_json(table, doc)?;
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
        let db = self.db()?;
        let result = db.get_json(table, id)?;
        Ok(result)
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
        let db = self.db_mut()?;
        let result = db.update(table, id, updates)?;
        Ok(result)
    }

    /// Delete a document by its `_id`. Returns `true` if found and deleted.
    pub fn delete(&mut self, table: &str, id: &str) -> SdkResult<bool> {
        let db = self.db_mut()?;
        let result = db.delete(table, id.as_bytes())?;
        Ok(result)
    }

    /// Count all documents in a table.
    pub fn count(&self, table: &str) -> SdkResult<usize> {
        let db = self.db()?;
        let count = db.count(table)?;
        Ok(count)
    }

    /// Scan all documents in a table (no filter).
    pub fn scan(&self, table: &str) -> SdkResult<Vec<Value>> {
        let db = self.db()?;
        let data = db.scan_json(table)?;
        Ok(data)
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
        let mut result = query_engine::execute(self, sql)?;
        result.execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok(result)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // SEARCH
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Full-text search across a table.
    pub fn search(&self, _table: &str, text: &str) -> SdkResult<Vec<Value>> {
        let db = self.db()?;
        let results = db.search_text(text);
        // Convert string results to JSON values
        let values: Vec<Value> = results.iter()
            .filter_map(|r| serde_json::from_str(r).ok())
            .collect();
        Ok(values)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // INDEXING
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Create a secondary index on a table column for faster queries.
    pub fn create_index(&mut self, table: &str, column: &str) -> SdkResult<()> {
        let db = self.db_mut()?;
        let def = overdrive_db::storage::index::IndexDef {
            name: format!("idx_{}_{}", table, column),
            table: table.to_string(),
            columns: vec![column.to_string()],
            index_type: overdrive_db::storage::index::IndexType::BTree,
            unique: false,
        };
        db.create_index(def)?;
        Ok(())
    }

    /// Drop a secondary index by name.
    pub fn drop_index(&mut self, name: &str) -> SdkResult<bool> {
        let db = self.db_mut()?;
        let result = db.drop_index(name)?;
        Ok(result)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // STATISTICS
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Get database statistics.
    pub fn stats(&self) -> SdkResult<Stats> {
        let db = self.db()?;
        let file_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or(0);
        let tables = db.list_tables();
        let mut total_records = 0;
        for table in &tables {
            total_records += db.count(table).unwrap_or(0);
        }
        Ok(Stats {
            tables: tables.len(),
            total_records,
            file_size_bytes: file_size,
            path: self.path.clone(),
        })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // INTERNAL HELPERS
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    pub(crate) fn db(&self) -> SdkResult<&Database> {
        self.db.as_ref().ok_or(SdkError::DatabaseClosed)
    }

    pub(crate) fn db_mut(&mut self) -> SdkResult<&mut Database> {
        self.db.as_mut().ok_or(SdkError::DatabaseClosed)
    }
}

impl Drop for OverDriveDB {
    fn drop(&mut self) {
        if let Some(db) = self.db.take() {
            let _ = db.sync();
            drop(db);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_and_open() {
        let path = format!("test_sdk_{}.odb", uuid::Uuid::new_v4());
        {
            let db = OverDriveDB::create(&path).unwrap();
            assert_eq!(db.path(), path);
            db.close().unwrap();
        }
        {
            let db = OverDriveDB::open_existing(&path).unwrap();
            db.close().unwrap();
        }
        OverDriveDB::destroy(&path).unwrap();
    }

    #[test]
    fn test_crud_operations() {
        let path = format!("test_crud_{}.odb", uuid::Uuid::new_v4());
        let mut db = OverDriveDB::open(&path).unwrap();
        
        // Create table
        db.create_table("users").unwrap();
        assert!(db.table_exists("users").unwrap());
        
        // Insert
        let id = db.insert("users", &json!({
            "name": "Alice",
            "age": 30
        })).unwrap();
        assert!(!id.is_empty());
        
        // Get
        let doc = db.get("users", &id).unwrap();
        assert!(doc.is_some());
        assert_eq!(doc.unwrap()["name"], "Alice");
        
        // Update
        db.update("users", &id, &json!({"age": 31})).unwrap();
        let updated = db.get("users", &id).unwrap().unwrap();
        assert_eq!(updated["age"], 31);
        
        // Count
        assert_eq!(db.count("users").unwrap(), 1);
        
        // Delete
        db.delete("users", &id).unwrap();
        assert_eq!(db.count("users").unwrap(), 0);
        
        // Cleanup
        db.close().unwrap();
        OverDriveDB::destroy(&path).unwrap();
    }

    #[test]
    fn test_batch_insert() {
        let path = format!("test_batch_{}.odb", uuid::Uuid::new_v4());
        let mut db = OverDriveDB::open(&path).unwrap();
        db.create_table("items").unwrap();
        
        let ids = db.insert_batch("items", &[
            json!({"name": "A", "price": 10}),
            json!({"name": "B", "price": 20}),
            json!({"name": "C", "price": 30}),
        ]).unwrap();
        
        assert_eq!(ids.len(), 3);
        assert_eq!(db.count("items").unwrap(), 3);
        
        db.close().unwrap();
        OverDriveDB::destroy(&path).unwrap();
    }

    #[test]
    fn test_list_and_drop_tables() {
        let path = format!("test_tables_{}.odb", uuid::Uuid::new_v4());
        let mut db = OverDriveDB::open(&path).unwrap();
        
        db.create_table("t1").unwrap();
        db.create_table("t2").unwrap();
        
        let tables = db.list_tables().unwrap();
        assert!(tables.contains(&"t1".to_string()));
        assert!(tables.contains(&"t2".to_string()));
        
        db.drop_table("t1").unwrap();
        assert!(!db.table_exists("t1").unwrap());
        
        db.close().unwrap();
        OverDriveDB::destroy(&path).unwrap();
    }

    #[test]
    fn test_sql_query() {
        let path = format!("test_sql_{}.odb", uuid::Uuid::new_v4());
        let mut db = OverDriveDB::open(&path).unwrap();
        db.create_table("products").unwrap();
        
        db.insert("products", &json!({"name": "Laptop", "price": 999})).unwrap();
        db.insert("products", &json!({"name": "Mouse", "price": 29})).unwrap();
        db.insert("products", &json!({"name": "Keyboard", "price": 79})).unwrap();
        
        let result = db.query("SELECT * FROM products WHERE price > 50").unwrap();
        assert_eq!(result.rows.len(), 2);
        assert!(result.execution_time_ms >= 0.0);
        
        db.close().unwrap();
        OverDriveDB::destroy(&path).unwrap();
    }
}
