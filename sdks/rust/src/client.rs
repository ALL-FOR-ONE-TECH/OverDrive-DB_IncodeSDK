//! OverDrive-DB Rust Client

use crate::loader::NativeLoader;
use crate::types::{QueryResult, TransactionOptions, StorageEngine};
use crate::errors::{OverDriveError, Result};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::ptr;

/// OverDrive-DB database client
pub struct OverDrive {
    handle: *mut std::ffi::c_void,
    loader: NativeLoader,
}

impl OverDrive {
    /// Open database with default settings
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use overdrive::OverDrive;
    /// 
    /// let odb = OverDrive::open("myapp.odb")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let loader = NativeLoader::new()?;
        let path_cstr = CString::new(path.as_ref())?;
        
        let handle = unsafe {
            loader.overdrive_open(path_cstr.as_ptr())
        };
        
        if handle.is_null() {
            return Err(OverDriveError::DatabaseError("Failed to open database".into()));
        }
        
        Ok(Self { handle, loader })
    }
    
    /// Open database with password encryption
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use overdrive::OverDrive;
    /// 
    /// let odb = OverDrive::open_with_password("secure.odb", "secret123")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open_with_password<P: AsRef<str>, S: AsRef<str>>(path: P, password: S) -> Result<Self> {
        let loader = NativeLoader::new()?;
        let path_cstr = CString::new(path.as_ref())?;
        let password_cstr = CString::new(password.as_ref())?;
        
        let handle = unsafe {
            loader.overdrive_open_with_password(path_cstr.as_ptr(), password_cstr.as_ptr())
        };
        
        if handle.is_null() {
            return Err(OverDriveError::DatabaseError("Failed to open encrypted database".into()));
        }
        
        Ok(Self { handle, loader })
    }
    
    /// Open database with specific storage engine
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use overdrive::{OverDrive, StorageEngine};
    /// 
    /// // In-memory database
    /// let odb = OverDrive::open_with_engine(":memory:", StorageEngine::RAM)?;
    /// 
    /// // Vector search database
    /// let odb = OverDrive::open_with_engine("vectors.odb", StorageEngine::Vector)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open_with_engine<P: AsRef<str>>(path: P, engine: StorageEngine) -> Result<Self> {
        let loader = NativeLoader::new()?;
        let path_cstr = CString::new(path.as_ref())?;
        let engine_cstr = CString::new(engine.as_str())?;
        
        let handle = unsafe {
            loader.overdrive_open_with_engine(path_cstr.as_ptr(), engine_cstr.as_ptr())
        };
        
        if handle.is_null() {
            return Err(OverDriveError::DatabaseError("Failed to open database with engine".into()));
        }
        
        Ok(Self { handle, loader })
    }
    
    /// Close database connection
    pub fn close(&mut self) -> Result<()> {
        if !self.handle.is_null() {
            unsafe {
                self.loader.overdrive_close(self.handle);
            }
            self.handle = ptr::null_mut();
        }
        Ok(())
    }
    
    /// Create table (schemaless by default)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// odb.create_table("users")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_table(&mut self, name: &str) -> Result<()> {
        let name_cstr = CString::new(name)?;
        
        unsafe {
            self.loader.overdrive_create_table(self.handle, name_cstr.as_ptr());
        }
        
        Ok(())
    }
    
    /// Insert JSON document
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// let id = odb.insert("users", &json!({
    ///     "name": "Alice",
    ///     "age": 30
    /// }))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn insert(&mut self, table: &str, document: &Value) -> Result<String> {
        let table_cstr = CString::new(table)?;
        let json_str = document.to_string();
        let json_cstr = CString::new(json_str)?;
        
        let result_ptr = unsafe {
            self.loader.overdrive_insert(self.handle, table_cstr.as_ptr(), json_cstr.as_ptr())
        };
        
        if result_ptr.is_null() {
            return Err(OverDriveError::DatabaseError("Insert failed".into()));
        }
        
        let result = unsafe {
            let cstr = CStr::from_ptr(result_ptr);
            let id = cstr.to_string_lossy().into_owned();
            self.loader.overdrive_free_string(result_ptr);
            id
        };
        
        Ok(result)
    }
    
    /// Get document by ID
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # let id = odb.insert("users", &json!({"name": "Alice"}))?;
    /// let user = odb.get("users", &id)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get(&self, table: &str, id: &str) -> Result<Option<Value>> {
        let table_cstr = CString::new(table)?;
        let id_cstr = CString::new(id)?;
        
        let result_ptr = unsafe {
            self.loader.overdrive_get(self.handle, table_cstr.as_ptr(), id_cstr.as_ptr())
        };
        
        if result_ptr.is_null() {
            return Ok(None);
        }
        
        let result = unsafe {
            let cstr = CStr::from_ptr(result_ptr);
            let json_str = cstr.to_string_lossy();
            let value: Value = serde_json::from_str(&json_str)?;
            self.loader.overdrive_free_string(result_ptr);
            value
        };
        
        Ok(Some(result))
    }
    
    /// Update document
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # let id = odb.insert("users", &json!({"name": "Alice", "age": 30}))?;
    /// odb.update("users", &id, &json!({"age": 31}))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn update(&mut self, table: &str, id: &str, updates: &Value) -> Result<()> {
        let table_cstr = CString::new(table)?;
        let id_cstr = CString::new(id)?;
        let json_str = updates.to_string();
        let json_cstr = CString::new(json_str)?;
        
        unsafe {
            self.loader.overdrive_update(
                self.handle,
                table_cstr.as_ptr(),
                id_cstr.as_ptr(),
                json_cstr.as_ptr()
            );
        }
        
        Ok(())
    }
    
    /// Delete document
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # let id = odb.insert("users", &json!({"name": "Alice"}))?;
    /// odb.delete("users", &id)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn delete(&mut self, table: &str, id: &str) -> Result<()> {
        let table_cstr = CString::new(table)?;
        let id_cstr = CString::new(id)?;
        
        unsafe {
            self.loader.overdrive_delete(self.handle, table_cstr.as_ptr(), id_cstr.as_ptr());
        }
        
        Ok(())
    }
    
    /// Execute SQL query
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # odb.insert("users", &json!({"name": "Alice", "age": 30}))?;
    /// let result = odb.query("SELECT * FROM users WHERE age > 25")?;
    /// println!("Found {} users", result.rows.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn query(&self, sql: &str) -> Result<QueryResult> {
        let sql_cstr = CString::new(sql)?;
        
        let result_ptr = unsafe {
            self.loader.overdrive_query(self.handle, sql_cstr.as_ptr())
        };
        
        if result_ptr.is_null() {
            return Err(OverDriveError::QueryError("Query failed".into()));
        }
        
        let result = unsafe {
            let cstr = CStr::from_ptr(result_ptr);
            let json_str = cstr.to_string_lossy();
            let rows: Vec<Value> = serde_json::from_str(&json_str)?;
            self.loader.overdrive_free_string(result_ptr);
            QueryResult { rows }
        };
        
        Ok(result)
    }
    
    /// Execute parameterized SQL query (SQL injection safe)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # odb.insert("users", &json!({"name": "Alice", "age": 30}))?;
    /// let result = odb.query_safe(
    ///     "SELECT * FROM users WHERE age > ?",
    ///     &[json!(25)]
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn query_safe(&self, sql: &str, params: &[Value]) -> Result<QueryResult> {
        let sql_cstr = CString::new(sql)?;
        let params_json = serde_json::to_string(params)?;
        let params_cstr = CString::new(params_json)?;
        
        let result_ptr = unsafe {
            self.loader.overdrive_query_safe(
                self.handle,
                sql_cstr.as_ptr(),
                params_cstr.as_ptr()
            )
        };
        
        if result_ptr.is_null() {
            return Err(OverDriveError::QueryError("Parameterized query failed".into()));
        }
        
        let result = unsafe {
            let cstr = CStr::from_ptr(result_ptr);
            let json_str = cstr.to_string_lossy();
            let rows: Vec<Value> = serde_json::from_str(&json_str)?;
            self.loader.overdrive_free_string(result_ptr);
            QueryResult { rows }
        };
        
        Ok(result)
    }
    
    /// Full-text search
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # odb.insert("users", &json!({"name": "Alice Smith", "bio": "Software engineer"}))?;
    /// let results = odb.search("users", "Alice")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn search(&self, table: &str, text: &str) -> Result<QueryResult> {
        let table_cstr = CString::new(table)?;
        let text_cstr = CString::new(text)?;
        
        let result_ptr = unsafe {
            self.loader.overdrive_search(self.handle, table_cstr.as_ptr(), text_cstr.as_ptr())
        };
        
        if result_ptr.is_null() {
            return Err(OverDriveError::QueryError("Search failed".into()));
        }
        
        let result = unsafe {
            let cstr = CStr::from_ptr(result_ptr);
            let json_str = cstr.to_string_lossy();
            let rows: Vec<Value> = serde_json::from_str(&json_str)?;
            self.loader.overdrive_free_string(result_ptr);
            QueryResult { rows }
        };
        
        Ok(result)
    }
    
    /// Count documents in table
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// # odb.insert("users", &json!({"name": "Alice"}))?;
    /// let count = odb.count("users")?;
    /// println!("Total users: {}", count);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn count(&self, table: &str) -> Result<usize> {
        let table_cstr = CString::new(table)?;
        
        let count = unsafe {
            self.loader.overdrive_count(self.handle, table_cstr.as_ptr())
        };
        
        Ok(count as usize)
    }
    
    /// Begin transaction
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use overdrive::OverDrive;
    /// # use serde_json::json;
    /// # let mut odb = OverDrive::open(":memory:")?;
    /// # odb.create_table("users")?;
    /// let tx = odb.begin_transaction(None)?;
    /// // Perform operations...
    /// tx.commit()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn begin_transaction(&self, options: Option<TransactionOptions>) -> Result<Transaction> {
        // Implementation would create a transaction handle
        // For now, return a placeholder
        Ok(Transaction {
            handle: ptr::null_mut(),
            loader: &self.loader,
        })
    }
}

impl Drop for OverDrive {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Transaction handle
pub struct Transaction<'a> {
    handle: *mut std::ffi::c_void,
    loader: &'a NativeLoader,
}

impl<'a> Transaction<'a> {
    /// Commit transaction
    pub fn commit(self) -> Result<()> {
        // Implementation would commit the transaction
        Ok(())
    }
    
    /// Rollback transaction
    pub fn rollback(self) -> Result<()> {
        // Implementation would rollback the transaction
        Ok(())
    }
}

// Thread safety
unsafe impl Send for OverDrive {}
unsafe impl Sync for OverDrive {}