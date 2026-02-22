# API Reference

> Auto-generated docs: [docs.rs/overdrive-sdk](https://docs.rs/overdrive-sdk)

## `OverDriveDB`

The main entry point for all database operations. Thread-safe within the constraints of the underlying storage engine.

---

### Database Lifecycle

#### `OverDriveDB::open(path: &str) -> SdkResult<Self>`

Open an existing database or create a new one. This is the primary entry point.

```rust
let mut db = OverDriveDB::open("myapp.odb")?;
```

**Arguments:**
- `path` — File path for the database. Can be relative or absolute.

**Returns:** `SdkResult<OverDriveDB>` — The database handle.

**Errors:**
- `SdkError::IoError` — If the path is inaccessible

---

#### `OverDriveDB::create(path: &str) -> SdkResult<Self>`

Create a new database. Returns an error if the file already exists.

```rust
let mut db = OverDriveDB::create("newdb.odb")?;
```

**Errors:**
- `SdkError::DatabaseAlreadyExists` — If the file exists

---

#### `OverDriveDB::open_existing(path: &str) -> SdkResult<Self>`

Open an existing database. Returns an error if the file doesn't exist.

```rust
let mut db = OverDriveDB::open_existing("existing.odb")?;
```

**Errors:**
- `SdkError::DatabaseNotFound` — If the file doesn't exist

---

#### `db.close(self) -> SdkResult<()>`

Close the database, sync all data to disk, and release resources.

```rust
db.close()?;
```

> **Note:** The database also closes automatically when dropped, but calling `close()` explicitly lets you handle errors.

---

#### `db.sync(&self) -> SdkResult<()>`

Force all buffered data to be written to disk immediately.

```rust
db.sync()?;
```

---

#### `OverDriveDB::destroy(path: &str) -> SdkResult<()>`

Delete a database file from disk permanently.

```rust
OverDriveDB::destroy("old.odb")?;
```

> ⚠️ **Warning:** This permanently deletes the database file. There is no undo.

---

#### `db.path() -> &str`

Get the file path of the open database.

#### `OverDriveDB::version() -> &'static str`

Get the SDK version string (e.g., `"1.0.0"`).

---

### Table Management

#### `db.create_table(name: &str) -> SdkResult<()>`

Create a new schemaless table.

```rust
db.create_table("users")?;
db.create_table("products")?;
```

**Errors:**
- `SdkError::TableAlreadyExists` — If the table already exists

---

#### `db.drop_table(name: &str) -> SdkResult<()>`

Drop a table and **all its data** permanently.

```rust
db.drop_table("old_table")?;
```

**Errors:**
- `SdkError::TableNotFound` — If the table doesn't exist

---

#### `db.list_tables() -> SdkResult<Vec<String>>`

List all table names in the database.

```rust
let tables = db.list_tables()?;
for table in &tables {
    println!("Table: {}", table);
}
```

---

#### `db.table_exists(name: &str) -> SdkResult<bool>`

Check if a table exists.

```rust
if db.table_exists("users")? {
    println!("Users table found");
}
```

---

### CRUD Operations

#### `db.insert(table: &str, doc: &Value) -> SdkResult<String>`

Insert a JSON document into a table. Returns the auto-generated `_id`.

```rust
let id = db.insert("users", &json!({
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com"
}))?;
println!("Created user: {}", id);
```

**Arguments:**
- `table` — Target table name
- `doc` — JSON document (any valid `serde_json::Value` object)

**Returns:** The auto-generated `_id` string (UUID v4).

---

#### `db.insert_batch(table: &str, docs: &[Value]) -> SdkResult<Vec<String>>`

Insert multiple documents in a batch. Returns all generated IDs.

```rust
let ids = db.insert_batch("users", &[
    json!({"name": "Bob", "age": 25}),
    json!({"name": "Charlie", "age": 35}),
    json!({"name": "Diana", "age": 28}),
])?;
println!("Inserted {} users", ids.len());
```

---

#### `db.get(table: &str, id: &str) -> SdkResult<Option<Value>>`

Retrieve a document by its `_id`. Returns `None` if not found.

```rust
if let Some(user) = db.get("users", &id)? {
    println!("Name: {}", user["name"]);
    println!("Age: {}", user["age"]);
} else {
    println!("User not found");
}
```

---

#### `db.update(table: &str, id: &str, updates: &Value) -> SdkResult<bool>`

Update specific fields of a document. Returns `true` if the document was found and updated.

```rust
let updated = db.update("users", &id, &json!({
    "age": 31,
    "email": "alice@newmail.com"
}))?;

if updated {
    println!("User updated");
}
```

> **Note:** Only the specified fields are updated; the rest remain unchanged.

---

#### `db.delete(table: &str, id: &str) -> SdkResult<bool>`

Delete a document by `_id`. Returns `true` if found and deleted.

```rust
if db.delete("users", &id)? {
    println!("User deleted");
}
```

---

#### `db.count(table: &str) -> SdkResult<usize>`

Count all documents in a table.

```rust
let n = db.count("users")?;
println!("{} users in database", n);
```

---

#### `db.scan(table: &str) -> SdkResult<Vec<Value>>`

Retrieve all documents from a table (no filtering).

```rust
let all_users = db.scan("users")?;
for user in &all_users {
    println!("{}", user);
}
```

> ⚠️ **Performance:** Avoid on large tables. Use `query()` with `LIMIT` instead.

---

### Query Engine

#### `db.query(sql: &str) -> SdkResult<QueryResult>`

Execute an SQL query against the embedded database engine.

```rust
let result = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name LIMIT 10")?;

println!("Found {} rows in {:.2}ms", result.rows.len(), result.execution_time_ms);

for row in &result.rows {
    println!("  {} — age {}", row["name"], row["age"]);
}
```

**Returns:**

```rust
pub struct QueryResult {
    pub rows: Vec<Value>,          // Result documents
    pub columns: Vec<String>,      // Column names
    pub rows_affected: usize,      // For INSERT/UPDATE/DELETE
    pub execution_time_ms: f64,    // Query timing
}
```

See [SQL Guide](sql-guide.md) for full syntax reference.

---

### Search

#### `db.search(table: &str, text: &str) -> SdkResult<Vec<Value>>`

Full-text search across all string fields in a table.

```rust
let matches = db.search("users", "alice")?;
println!("Found {} matches", matches.len());
```

---

### Indexing

#### `db.create_index(table: &str, column: &str) -> SdkResult<()>`

Create a B-Tree secondary index on a column for faster queries.

```rust
db.create_index("users", "email")?;
db.create_index("users", "age")?;
```

#### `db.drop_index(name: &str) -> SdkResult<bool>`

Drop an index by name. Returns `true` if found and removed.

```rust
db.drop_index("idx_users_email")?;
```

> **Index naming:** Auto-generated as `idx_{table}_{column}`.

---

### Statistics

#### `db.stats() -> SdkResult<Stats>`

Get database statistics.

```rust
let stats = db.stats()?;
println!("Tables: {}", stats.tables);
println!("Records: {}", stats.total_records);
println!("File size: {} bytes", stats.file_size_bytes);
println!("Path: {}", stats.path);
```

---

## Error Types

```rust
pub enum SdkError {
    DatabaseNotFound(String),
    DatabaseAlreadyExists(String),
    DatabaseClosed,
    TableNotFound(String),
    TableAlreadyExists(String),
    KeyNotFound(String),
    QueryError(String),
    SerializationError(String),
    IoError(std::io::Error),
    TransactionError(String),
    ValidationError(String),
    InternalError(String),
}
```

All methods return `SdkResult<T>`, which is `Result<T, SdkError>`.
