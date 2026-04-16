# Rust SDK — Complete Guide

**Version:** 1.4.0 | **Requires:** Rust 2021 edition

---

## Installation

```toml
# Cargo.toml
[dependencies]
overdrive-db = "1.4.0"
serde_json = "1.0"
```

> **Important:** The Rust crate dynamically loads the native library at runtime.
> Download `overdrive.dll` / `liboverdrive.so` / `liboverdrive.dylib` from
> [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest)
> and place it in your project directory (next to your binary).

---

## Import

```rust
use overdrive::{OverDriveDB, IsolationLevel, QueryResult, Stats};
use overdrive::shared::SharedDB;
use serde_json::{json, Value};
```

---

## Opening a Database

```rust
// Basic open (auto-creates if not exists)
let mut db = OverDriveDB::open("myapp.odb")?;

// Create new (error if exists)
let mut db = OverDriveDB::create("newapp.odb")?;

// Open existing (error if not found)
let mut db = OverDriveDB::open_existing("app.odb")?;

// Environment variable encryption
// $env:ODB_KEY = "my-aes-256-key-32-chars-minimum!!!!"
let mut db = OverDriveDB::open_encrypted("app.odb", "ODB_KEY")?;

// Always close when done
db.close()?;
```

---

## Table Operations

```rust
// Create table
db.create_table("users")?;

// Drop table
db.drop_table("old_table")?;

// List tables
let tables: Vec<String> = db.list_tables()?;

// Check existence
let exists: bool = db.table_exists("users")?;
```

---

## CRUD Operations

### Insert

```rust
// Single document — returns auto-generated _id
let id = db.insert("users", &json!({
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com",
    "tags": ["admin", "dev"],
    "active": true
}))?;
println!("Inserted: {}", id);  // "users_1"

// Multiple documents
let docs = vec![
    json!({"name": "Bob",   "age": 25}),
    json!({"name": "Carol", "age": 35}),
];
let ids = db.insert_batch("users", &docs)?;
println!("{:?}", ids);  // ["users_2", "users_3"]
```

### Get

```rust
// Get by _id
let user: Option<Value> = db.get("users", "users_1")?;
match user {
    Some(doc) => println!("Found: {}", doc["name"]),
    None => println!("Not found"),
}

// Count all documents
let count: usize = db.count("users")?;
```

### Update

```rust
// Update by _id — only specified fields change
let updated: bool = db.update("users", "users_1", &json!({
    "age": 31,
    "status": "active"
}))?;
// true if found and updated, false if not found
```

### Delete

```rust
// Delete by _id
let deleted: bool = db.delete("users", "users_1")?;
// true if found and deleted, false if not found
```

### Scan

```rust
// Get all documents in a table
let all_users: Vec<Value> = db.scan("users")?;
```

---

## SQL Queries

```rust
// Execute SQL — returns QueryResult
let result: QueryResult = db.query("SELECT * FROM users")?;
let result = db.query("SELECT * FROM users WHERE age > 25")?;
let result = db.query("SELECT * FROM users ORDER BY name DESC LIMIT 10")?;
let result = db.query("SELECT COUNT(*) FROM users")?;

// Access results
println!("Found {} rows in {:.2}ms", result.rows.len(), result.execution_time_ms);
for row in &result.rows {
    println!("  {} — age {}", row["name"], row["age"]);
}

// QueryResult fields
result.rows              // Vec<Value> — result rows
result.columns           // Vec<String> — column names
result.rows_affected     // usize — for INSERT/UPDATE/DELETE
result.execution_time_ms // f64 — query time

// Safe parameterized query (use for user input!)
let result = db.query_safe(
    "SELECT * FROM users WHERE name = ?",
    &[user_input]
)?;
let result = db.query_safe(
    "SELECT * FROM users WHERE age > ? AND city = ?",
    &["25", "London"]
)?;

// Full-text search
let matches: Vec<Value> = db.search("users", "alice")?;
```

---

## Transactions

```rust
use overdrive::IsolationLevel;

// Begin transaction
let txn = db.begin_transaction(IsolationLevel::ReadCommitted)?;

// Do work
db.insert("users", &json!({"name": "Alice"}))?;
db.insert("logs", &json!({"event": "user_created"}))?;

// Commit or abort
db.commit_transaction(&txn)?;
// or: db.abort_transaction(&txn)?;

// Isolation levels
IsolationLevel::ReadUncommitted  // 0
IsolationLevel::ReadCommitted    // 1 (default)
IsolationLevel::RepeatableRead   // 2
IsolationLevel::Serializable     // 3
```

---

## Security

```rust
use overdrive::{OverDriveDB, SecretKey};

// Load key from environment variable (key is zeroed from RAM on drop)
let key = SecretKey::from_env("ODB_KEY")?;
// key bytes are automatically zeroed when `key` is dropped

// Open with encryption
let mut db = OverDriveDB::open_encrypted("app.odb", "ODB_KEY")?;

// Backup
db.backup("backups/app_2026-04-16.odb")?;

// WAL cleanup after commit
let txn = db.begin_transaction(IsolationLevel::ReadCommitted)?;
db.insert("users", &json!({"name": "Alice"}))?;
db.commit_transaction(&txn)?;
db.cleanup_wal()?;

// Safe parameterized query
let result = db.query_safe(
    "SELECT * FROM users WHERE name = ?",
    &[user_input]
)?;
```

---

## Thread-Safe Access

```rust
use overdrive::shared::SharedDB;
use std::sync::Arc;
use std::thread;

// SharedDB wraps OverDriveDB in Arc<Mutex<>>
let shared = SharedDB::open("app.odb")?;

let handles: Vec<_> = (0..10).map(|i| {
    let db = shared.clone();
    thread::spawn(move || {
        db.insert("logs", &json!({"thread": i, "event": "started"})).unwrap();
    })
}).collect();

for h in handles { h.join().unwrap(); }
```

---

## Integrity Verification

```rust
// Verify database integrity
let report = db.verify_integrity()?;

println!("Valid: {}", report.is_valid);
println!("Pages checked: {}", report.pages_checked);
println!("Tables verified: {}", report.tables_verified);
if !report.issues.is_empty() {
    println!("Issues: {:?}", report.issues);
}
```

---

## Statistics

```rust
let stats = db.stats()?;

println!("Tables: {}", stats.tables);
println!("Total records: {}", stats.total_records);
println!("File size: {} bytes", stats.file_size_bytes);
println!("SDK version: {}", stats.sdk_version);
```

---

## Error Handling

```rust
use overdrive::result::{SdkError, SdkResult};

fn open_database(path: &str) -> SdkResult<OverDriveDB> {
    OverDriveDB::open(path)
}

match open_database("app.odb") {
    Ok(db) => println!("Opened!"),
    Err(SdkError::DatabaseNotFound(path)) => println!("Not found: {}", path),
    Err(SdkError::SecurityError(msg)) => println!("Security error: {}", msg),
    Err(SdkError::BackupError(msg)) => println!("Backup error: {}", msg),
    Err(e) => println!("Error: {}", e),
}
```

---

## Complete Example

```rust
use overdrive::OverDriveDB;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = OverDriveDB::open("store.odb")?;

    // Create table
    db.create_table("products")?;

    // Insert products
    let products = vec![
        json!({"name": "Laptop",  "price": 999,  "category": "electronics", "stock": 5}),
        json!({"name": "Mouse",   "price": 29,   "category": "electronics", "stock": 50}),
        json!({"name": "Desk",    "price": 299,  "category": "furniture",   "stock": 10}),
        json!({"name": "Chair",   "price": 199,  "category": "furniture",   "stock": 15}),
    ];
    let ids = db.insert_batch("products", &products)?;
    println!("Inserted {} products", ids.len());

    // Query
    let result = db.query(
        "SELECT * FROM products WHERE category = 'electronics' ORDER BY price DESC"
    )?;
    println!("Electronics ({} items):", result.rows.len());
    for p in &result.rows {
        println!("  ${} — {}", p["price"], p["name"]);
    }

    // Count
    let total = db.count("products")?;
    let count_result = db.query("SELECT COUNT(*) FROM products WHERE price > 100")?;
    println!("Total: {}, Expensive: {:?}", total, count_result.rows[0]);

    // Stats
    let stats = db.stats()?;
    println!("File size: {} bytes", stats.file_size_bytes);

    // Backup
    db.backup("backups/store_backup.odb")?;
    println!("Backup created!");

    db.close()?;
    Ok(())
}
```
