# overdrive-db (Rust)

[![Crates.io](https://img.shields.io/crates/v/overdrive-db?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/overdrive-db)
[![License](https://img.shields.io/crates/l/overdrive-db?style=flat-square)](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)

Zero-config embedded JSON document database with ACID transactions, AES-256 encryption, and 6 storage engines.

## Install

```toml
[dependencies]
overdrive-db = "1.0"
```

## Quick Start

```rust
use overdrive::OverdriveDb;
use serde_json::json;

fn main() {
    let mut odb = OverdriveDb::open("myapp.odb").unwrap();

    odb.create_table("users").unwrap();

    // Insert — takes a &serde_json::Value
    let id = odb.insert("users", &json!({"name": "Alice", "age": 30})).unwrap();

    // Get by _id
    let doc = odb.get("users", &id).unwrap();
    println!("{:?}", doc);

    // SQL Query
    let rows = odb.query("SELECT * FROM users WHERE age > 25").unwrap();
    println!("{} rows", rows.len());

    // Count
    let n = odb.count("users").unwrap();
    println!("Total: {}", n);

    // Update
    odb.update("users", &id, &json!({"age": 31})).unwrap();

    // Delete
    odb.delete("users", &id).unwrap();

    odb.close().unwrap();
}
```

## API Reference

### Open

```rust
// Plain open
let mut odb = OverdriveDb::open("app.odb").unwrap();

// With options (password / engine)
use overdrive::OpenOptions;
let mut odb = OverdriveDb::open_with_options(
    "app.odb",
    OpenOptions::new().password("secret").engine("RAM"),
).unwrap();

// Version
let v = OverdriveDb::version();
```

### Tables

```rust
odb.create_table("users").unwrap();
odb.drop_table("users").unwrap();
let tables: Vec<String> = odb.list_tables().unwrap();
let exists: bool = odb.table_exists("users").unwrap();
```

### CRUD

```rust
// Insert → returns _id String
let id = odb.insert("users", &json!({"name": "Alice"})).unwrap();

// Insert batch → returns Vec<String> of _ids
let ids = odb.insert_batch("users", &[
    json!({"name": "Bob"}),
    json!({"name": "Carol"}),
]).unwrap();

// Get → Option<Value>
let doc = odb.get("users", &id).unwrap();

// Update → bool (true = updated)
let updated = odb.update("users", &id, &json!({"age": 31})).unwrap();

// Delete → bool (true = deleted)
let deleted = odb.delete("users", &id).unwrap();

// Count
let n = odb.count("users").unwrap();
```

### Query

```rust
// SQL — returns Vec<Value>
let rows = odb.query("SELECT * FROM users WHERE age > 25 ORDER BY age LIMIT 10").unwrap();

// Full-text search — returns Vec<Value>
let results = odb.search("users", "Alice").unwrap();
```

### Transactions

```rust
use overdrive::IsolationLevel;

// Callback style (auto-commit / auto-abort)
let result = odb.transaction(IsolationLevel::Serializable, |odb| {
    odb.insert("accounts", &json!({"balance": 1000}))?;
    odb.update("accounts", &some_id, &json!({"balance": 900}))?;
    Ok(())
}).unwrap();

// Manual style
let txn = odb.begin_transaction(IsolationLevel::ReadCommitted).unwrap();
match do_work(&mut odb) {
    Ok(_)  => odb.commit_transaction(&txn).unwrap(),
    Err(e) => { odb.abort_transaction(&txn).unwrap(); }
}
```

**Isolation Levels:**

| Variant | Value |
|---------|-------|
| `IsolationLevel::ReadUncommitted` | 0 |
| `IsolationLevel::ReadCommitted` | 1 (default) |
| `IsolationLevel::RepeatableRead` | 2 |
| `IsolationLevel::Serializable` | 3 |

### Integrity Check

```rust
let report = odb.verify_integrity().unwrap();
println!("{}", report);
```

## 6 Storage Engines

```rust
OverdriveDb::open_with_options("app.odb", OpenOptions::new().engine("Disk"))       // default
OverdriveDb::open_with_options("cache.odb", OpenOptions::new().engine("RAM"))      // in-memory
OverdriveDb::open_with_options("vecs.odb", OpenOptions::new().engine("Vector"))    // embeddings
OverdriveDb::open_with_options("ts.odb", OpenOptions::new().engine("Time-Series")) // metrics/IoT
OverdriveDb::open_with_options("g.odb", OpenOptions::new().engine("Graph"))        // social graphs
OverdriveDb::open_with_options("q.odb", OpenOptions::new().engine("Streaming"))    // event queue
```

## Platform Support

| Platform | Native Library |
|----------|---------------|
| Windows x64 | `overdrive.dll` |
| Linux x64 | `liboverdrive.so` |
| Linux ARM64 | `liboverdrive.so` |
| macOS x64 | `liboverdrive.dylib` |
| macOS ARM64 | `liboverdrive.dylib` |

## Links

- 📦 [crates.io/crates/overdrive-db](https://crates.io/crates/overdrive-db)
- 🐙 [GitHub Repository](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)
- 🌐 [overdrive-db.com](https://overdrive-db.com)
