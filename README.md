<p align="center">
  <h1 align="center">⚡ OverDrive InCode SDK</h1>
  <p align="center">
    <strong>Embeddable document database — like SQLite for JSON.</strong><br/>
    Import the package. Open a file. Query your data. <em>No server needed.</em>
  </p>
  <p align="center">
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest"><img src="https://img.shields.io/github/v/release/ALL-FOR-ONE-TECH/OverDrive-DB_SDK?style=flat-square&logo=github&color=orange" alt="release"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases"><img src="https://img.shields.io/github/downloads/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/total?style=flat-square&color=blue" alt="downloads"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat-square" alt="license"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK"><img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blueviolet?style=flat-square" alt="platforms"/></a>
  </p>
</p>

---

## What is OverDrive?

OverDrive is an **embeddable, zero-config document database** for Rust, Python, Node.js, and C/C++. It stores JSON documents, supports SQL queries, full-text search, secondary indexing, and ACID transactions — all within a single library. No external server. No network. Just a file.

| Feature | Description |
|---|---|
| 🗄️ **Zero-config** | Open a file, start querying — no setup, no config files |
| 📄 **JSON Native** | Store, query, and index JSON documents directly |
| 🔍 **SQL Queries** | `SELECT`, `INSERT`, `UPDATE`, `DELETE` with `WHERE`, `ORDER BY`, `LIMIT` |
| 📊 **Aggregations** | `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `GROUP BY` |
| 🔎 **Full-text Search** | Built-in text search across documents |
| 🌳 **B-Tree Indexes** | Secondary indexes for fast lookups |
| 🔒 **ACID Transactions** | Reliable, consistent data operations |
| 🗜️ **Compression** | zstd compression for efficient storage |
| 🔐 **Encryption** | AES-256-GCM at-rest encryption |
| 🌍 **Cross-platform** | Windows, Linux, macOS |
| 🔗 **Multi-language** | Rust, Python, Node.js, C, C++ |

---

## Quick Start

### Install

**Option 1 — Download prebuilt binary:**

| Platform | Download |
|---|---|
| Windows x64 | [`overdrive.dll`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) |
| Linux x64 | [`liboverdrive.so`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) |
| macOS ARM64 | [`liboverdrive.dylib`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) |

**Option 2 — Add as Rust dependency (git):**

```toml
[dependencies]
overdrive-sdk = { git = "https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK.git" }
```

### Use

```rust
use overdrive::OverDriveDB;
use serde_json::json;

fn main() {
    // Open or create a database
    let mut db = OverDriveDB::open("myapp.odb").unwrap();

    // Create a table
    db.create_table("users").unwrap();

    // Insert a document
    let id = db.insert("users", &json!({
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30,
        "tags": ["admin", "developer"]
    })).unwrap();

    println!("Inserted user with ID: {}", id);

    // SQL query
    let result = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name").unwrap();
    println!("Found {} users in {:.2}ms", result.rows.len(), result.execution_time_ms);

    for row in &result.rows {
        println!("  → {} ({})", row["name"], row["email"]);
    }

    // Full-text search
    let matches = db.search("users", "alice").unwrap();
    println!("Search found {} matches", matches.len());

    // Get by ID
    let user = db.get("users", &id).unwrap();
    println!("Got: {:?}", user);

    // Update
    db.update("users", &id, &json!({"age": 31})).unwrap();

    // Count
    let count = db.count("users").unwrap();
    println!("Total users: {}", count);

    // Clean up
    db.close().unwrap();
}
```

**That's it.** No server to install. No docker. No config. Just add the dependency and go.

---

## API Reference

> 📖 Full API docs: [docs/api-reference.md](docs/api-reference.md)

### Database Lifecycle

| Method | Description |
|---|---|
| `OverDriveDB::open(path)` | Open or create a database |
| `OverDriveDB::create(path)` | Create new (error if exists) |
| `OverDriveDB::open_existing(path)` | Open existing (error if not found) |
| `db.close()` | Close and release resources |
| `db.sync()` | Force flush to disk |
| `OverDriveDB::destroy(path)` | Delete database file |
| `OverDriveDB::version()` | Get SDK version |

### Table Management

| Method | Description |
|---|---|
| `db.create_table(name)` | Create a new table |
| `db.drop_table(name)` | Drop a table and all data |
| `db.list_tables()` | List all table names |
| `db.table_exists(name)` | Check if a table exists |

### CRUD Operations

| Method | Description |
|---|---|
| `db.insert(table, &json)` | Insert document, returns `_id` |
| `db.insert_batch(table, &[json])` | Batch insert, returns `Vec<_id>` |
| `db.get(table, id)` | Get document by `_id` |
| `db.update(table, id, &json)` | Update document fields |
| `db.delete(table, id)` | Delete document |
| `db.count(table)` | Count documents |
| `db.scan(table)` | Get all documents |

### SQL Queries

| Method | Description |
|---|---|
| `db.query(sql)` | Execute SQL, returns `QueryResult` |

```sql
-- Supported SQL
SELECT * FROM users WHERE age > 25 ORDER BY name DESC LIMIT 10 OFFSET 5
SELECT COUNT(*), AVG(age) FROM users
INSERT INTO users VALUES {"name": "Bob", "age": 25}
UPDATE users SET {"age": 26} WHERE name = 'Bob'
DELETE FROM users WHERE age < 18
CREATE TABLE products
DROP TABLE products
SHOW TABLES
```

### Search & Indexing

| Method | Description |
|---|---|
| `db.search(table, text)` | Full-text search |
| `db.create_index(table, column)` | Create B-Tree index |
| `db.drop_index(name)` | Drop an index |

### Statistics

| Method | Description |
|---|---|
| `db.stats()` | Get `Stats { tables, total_records, file_size_bytes, path }` |

---

## Python SDK

```bash
pip install overdrive-db   # Coming soon to PyPI
```

```python
from overdrive import OverDrive

# Context manager for automatic cleanup
with OverDrive("myapp.odb") as db:
    db.create_table("users")

    # Insert
    user_id = db.insert("users", {
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30
    })

    # SQL Query
    results = db.query("SELECT * FROM users WHERE age > 25")
    for row in results:
        print(f"  {row['name']} — {row['email']}")

    # Full-text search
    matches = db.search("users", "alice")

    # Batch insert
    db.insert_many("users", [
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35},
    ])

    # Count
    print(f"Total users: {db.count('users')}")
```

---

## Node.js SDK

```bash
npm install overdrive-db   # Coming soon to npm
```

```javascript
const { OverDrive } = require('overdrive-db');

const db = new OverDrive('myapp.odb');

// Create table
db.createTable('products');

// Insert
const id = db.insert('products', {
    name: 'Laptop',
    price: 999.99,
    specs: { ram: '16GB', cpu: 'i7' }
});

// SQL Query
const results = db.query('SELECT * FROM products WHERE price > 500');
console.table(results);

// Batch insert
const ids = db.insertMany('products', [
    { name: 'Mouse', price: 29.99 },
    { name: 'Keyboard', price: 79.99 },
]);

// Count
console.log(`Total products: ${db.count('products')}`);

// Clean up
db.close();
```

---

## C / C++ API

```c
#include "overdrive.h"

int main() {
    // Open database
    ODB* db = overdrive_open("myapp.odb");
    if (!db) {
        printf("Error: %s\n", overdrive_last_error());
        return 1;
    }

    // Create table
    overdrive_create_table(db, "users");

    // Insert
    char* id = overdrive_insert(db, "users",
        "{\"name\":\"Alice\",\"age\":30}");
    printf("Inserted: %s\n", id);
    overdrive_free_string(id);

    // Query
    char* result = overdrive_query(db,
        "SELECT * FROM users WHERE age > 25");
    printf("Results: %s\n", result);
    overdrive_free_string(result);

    // Search
    char* matches = overdrive_search(db, "users", "alice");
    printf("Search: %s\n", matches);
    overdrive_free_string(matches);

    // Close
    overdrive_close(db);
    return 0;
}
```

> **Memory rule**: Every `char*` returned by `overdrive_*` functions must be freed with `overdrive_free_string()`. The `ODB*` handle must be closed with `overdrive_close()`.

---

## Architecture

```
┌─────────────────────────────────────┐
│           Your Application          │
│                                     │
│  db = OverDriveDB::open("app.odb")  │
│  db.insert("users", {...})          │
│  db.query("SELECT * FROM ...")      │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│         OverDrive SDK (1.0.0)       │
│                                     │
│  OverDriveDB  │  QueryEngine  │ FFI │
│  (lib.rs)     │  (SQL parser) │     │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│       OverDrive Engine (0.2.0)      │
│                                     │
│  B-Tree  │  WAL  │  MVCC  │  Index  │
│  Pages   │  zstd │  AES   │  Cache  │
└─────────────────────────────────────┘
               │
         ┌─────▼─────┐
         │  app.odb   │
         │  (file)    │
         └───────────-┘
```

- **No network** — everything runs in-process
- **No server** — just a library and a file
- **No config** — open and go

---

## Comparison

| Feature | OverDrive | SQLite | MongoDB | Redis |
|---|---|---|---|---|
| **Deployment** | Library | Library | Server | Server |
| **Data Model** | JSON Documents | Relational | JSON Documents | Key-Value |
| **SQL Support** | ✅ | ✅ | ❌ | ❌ |
| **Schema** | Optional | Required | Optional | None |
| **Full-text Search** | ✅ Built-in | FTS5 Extension | ✅ | ❌ |
| **Encryption** | ✅ AES-256 | SEE (paid) | ✅ Enterprise | ✅ |
| **Compression** | ✅ zstd | ❌ | ✅ | ❌ |
| **Versioning** | ✅ Git-like | ❌ | ❌ | ❌ |
| **Size** | ~1.4MB | ~1.2MB | ~200MB | ~3MB |

---

## Building from Source

```bash
# Clone
git clone https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK.git
cd OverDrive-DB_SDK

# Build (debug)
cargo build

# Build (release — produces overdrive.dll / liboverdrive.so)
cargo build --release

# Run tests
cargo test --lib

# Run all tests including doc-tests
cargo test
```

### Build Outputs

| Platform | File | Location |
|---|---|---|
| Windows | `overdrive.dll` | `target/release/` |
| Linux | `liboverdrive.so` | `target/release/` |
| macOS | `liboverdrive.dylib` | `target/release/` |

---

## Project Structure

```
OverDrive-DB_SDK/
├── Cargo.toml           # Crate configuration
├── README.md            # This file
├── LICENSE              # MIT License
├── cbindgen.toml        # C header generation config
│
├── src/
│   ├── lib.rs           # OverDriveDB struct + public API
│   ├── query_engine.rs  # Embedded SQL parser & executor
│   ├── result.rs        # SdkError types + From impls
│   └── ffi.rs           # C FFI extern functions
│
├── python/
│   └── overdrive/
│       └── __init__.py  # Python ctypes wrapper
│
├── nodejs/
│   └── index.js         # Node.js ffi-napi wrapper
│
├── docs/
│   ├── quickstart.md    # Getting started guide
│   ├── api-reference.md # Full API documentation
│   ├── sql-guide.md     # SQL syntax reference
│   ├── python-guide.md  # Python SDK guide
│   ├── nodejs-guide.md  # Node.js SDK guide
│   ├── c-guide.md       # C/C++ integration guide
│   └── architecture.md  # How it works internally
│
└── examples/
    ├── basic_crud.rs     # Basic CRUD operations
    ├── sql_queries.rs    # SQL query examples
    └── batch_ops.rs      # Batch operations
```

---

## Links

| Resource | URL |
|---|---|
| 📦 **Download** | [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases) |
| � **API Docs** | [docs/api-reference.md](docs/api-reference.md) |
| � **Quick Start** | [docs/quickstart.md](docs/quickstart.md) |
| 🏠 **Homepage** | [overdrive-db.com](https://overdrive-db.com) |
| 🐛 **Issues** | [GitHub Issues](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/issues) |
| 💬 **Discussions** | [GitHub Discussions](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/discussions) |

---

## License

Licensed under either of:

- **MIT License** ([LICENSE-MIT](LICENSE))
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE))

at your option.

---

<p align="center">
  Built by <a href="https://github.com/ALL-FOR-ONE-TECH"><strong>ALL FOR ONE TECH</strong></a> • 
  <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases">Downloads</a> • 
  <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/api-reference.md">API Docs</a>
</p>
