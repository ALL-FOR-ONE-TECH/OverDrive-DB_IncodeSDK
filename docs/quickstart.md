# Getting Started with OverDrive SDK

## Installation

Add OverDrive to your Rust project:

```bash
cargo add overdrive-sdk
```

Or add to your `Cargo.toml` manually:

```toml
[dependencies]
overdrive-sdk = "1.0.0"
```

## Your First Database

```rust
use overdrive::OverDriveDB;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open (or create) a database
    let mut db = OverDriveDB::open("todo.odb")?;

    // 2. Create a table
    db.create_table("tasks")?;

    // 3. Insert some documents
    let id1 = db.insert("tasks", &json!({
        "title": "Buy groceries",
        "done": false,
        "priority": "high"
    }))?;

    let id2 = db.insert("tasks", &json!({
        "title": "Write report",
        "done": false,
        "priority": "medium"
    }))?;

    let id3 = db.insert("tasks", &json!({
        "title": "Clean house",
        "done": true,
        "priority": "low"
    }))?;

    // 4. Query with SQL
    let result = db.query("SELECT * FROM tasks WHERE done = false")?;
    println!("Pending tasks: {}", result.rows.len());

    for task in &result.rows {
        println!("  [ ] {} ({})", task["title"], task["priority"]);
    }

    // 5. Update a task
    db.update("tasks", &id1, &json!({"done": true}))?;

    // 6. Count
    println!("Total tasks: {}", db.count("tasks")?);

    // 7. Close
    db.close()?;
    Ok(())
}
```

## Core Concepts

### Documents

OverDrive stores **JSON documents** — no fixed schema required. Each document gets an auto-generated `_id` field.

```rust
// Any valid JSON works
db.insert("users", &json!({
    "name": "Alice",
    "age": 30,
    "address": {
        "city": "Chennai",
        "country": "India"
    },
    "tags": ["admin", "developer"],
    "active": true
}))?;
```

### Tables

Tables are logical collections of documents. Create them on the fly:

```rust
db.create_table("users")?;
db.create_table("products")?;
db.create_table("orders")?;
```

### Queries

Use SQL for complex queries, or direct API methods for simple operations:

```rust
// SQL way
let result = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name")?;

// Direct way
let user = db.get("users", &id)?;      // Get by ID
let all = db.scan("users")?;            // Get all
let count = db.count("users")?;         // Count
```

## What's Next?

- [API Reference](api-reference.md) — Complete method documentation
- [SQL Guide](sql-guide.md) — Supported SQL syntax
- [Python Guide](python-guide.md) — Python SDK usage
- [Node.js Guide](nodejs-guide.md) — Node.js SDK usage
- [C/C++ Guide](c-guide.md) — C FFI integration
- [Architecture](architecture.md) — How OverDrive works internally
