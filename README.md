<p align="center">
  <h1 align="center">⚡ OverDrive-DB — InCode SDK</h1>
  <p align="center">
    <strong>A high-performance, embeddable hybrid SQL+NoSQL document database. Like SQLite, but for JSON.</strong><br/>
    Import the package. Open a file. Query your data. <em>No server needed.</em>
  </p>
  <p align="center">
    <a href="https://crates.io/crates/overdrive-db"><img src="https://img.shields.io/crates/v/overdrive-db?style=flat-square&color=orange&logo=rust" alt="crates.io"/></a>
    <a href="https://www.npmjs.com/package/overdrive-db"><img src="https://img.shields.io/npm/v/overdrive-db?style=flat-square&color=cb3837&logo=npm" alt="npm"/></a>
    <a href="https://pypi.org/project/overdrive-db/"><img src="https://img.shields.io/pypi/v/overdrive-db?style=flat-square&color=3776ab&logo=python" alt="PyPI"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/packages"><img src="https://img.shields.io/badge/maven-central-007ec6?style=flat-square&logo=apache-maven" alt="maven"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat-square" alt="license"/></a>
  </p>
</p>

#afot #OverDriveDb #InCodeSDK #EmbeddedDB

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
| 🛡️ **Security Hardened** | Key from env vars, `query_safe()` injection blocking, auto `chmod 600`, WAL cleanup, thread-safe wrappers |
| 🌍 **Cross-platform** | Windows, Linux, macOS |
| 🔗 **Multi-language** | Rust, Python, Node.js, Java, Go, C/C++ |

---

## Quick Start

### Install

**Option 1 — Download prebuilt binary:**

| Platform | Download |
|---|---|
| Windows x64 | [`overdrive.dll`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) |
| Linux x64 | [`liboverdrive.so`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) |
| macOS ARM64 | [`liboverdrive.dylib`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) |

**Option 2 — Add as Rust dependency:**

```toml
[dependencies]
overdrive-db = "1.3.0"
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
| `db.query_safe(sql, &[params])` | ✅ Parameterized query — safe for user input |

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

### Security APIs (v1.3.0)

| Method | Threat Mitigated | Description |
|---|---|---|
| `OverDriveDB::open_encrypted(path, "ODB_KEY")` | 🔴 Key hardcoding | Reads AES key from env var, zeroed on drop |
| `db.query_safe(sql, &[params])` | 🟠 SQL injection | Parameterized query with injection detection |
| `db.backup(dest_path)` | 🔴 Ransomware/deletion | Encrypted backup copy + permission hardening |
| `db.cleanup_wal()` | 🟡 WAL replay | Deletes stale WAL after commit |
| `SharedDB::open(path)` | 🟠 Race condition | Thread-safe Arc+Mutex wrapper |
| Auto on `open()` | 🟡 Unauthorized access | `chmod 600` / Windows ACL on every open |

```rust
// Set env var (never hardcode!)
// $env:ODB_KEY = "my-aes-256-key-32-chars-min!!!!"
use overdrive::OverDriveDB;
use overdrive::shared::SharedDB;

// 1. Open with key from environment
let mut db = OverDriveDB::open_encrypted("app.odb", "ODB_KEY")?;

// 2. Safe parameterized query — blocks SQL injection
let rows = db.query_safe(
    "SELECT * FROM users WHERE name = ?",
    &[user_input],  // injection attempt is blocked automatically
)?;

// 3. Backup
db.backup("offsite/app_backup.odb")?;

// 4. WAL cleanup after commit
let txn = db.begin_transaction(IsolationLevel::ReadCommitted)?;
db.commit_transaction(&txn)?;
db.cleanup_wal()?;

// 5. Thread-safe access
let shared = SharedDB::open("app.odb")?;
let db2 = shared.clone();
std::thread::spawn(move || db2.query("SELECT * FROM users"));
```

### Statistics

| Method | Description |
|---|---|
| `db.stats()` | Get `Stats { tables, total_records, file_size_bytes, path }` |

---

## Python SDK

```bash
pip install overdrive-db
```

```python
from overdrive import OverDrive, ThreadSafeOverDrive
import os

# Context manager for automatic cleanup
with OverDrive("myapp.odb") as db:  # permissions auto-hardened on open
    db.create_table("users")

    # Insert
    user_id = db.insert("users", {"name": "Alice", "email": "alice@example.com", "age": 30})

    # Safe parameterized query
    results = db.query_safe("SELECT * FROM users WHERE age > ?", [25])
    for row in results:
        print(f"  {row['name']} — {row['email']}")

    # Full-text search
    matches = db.search("users", "alice")

    # Backup + WAL cleanup after commit
    db.backup("backups/app.odb")
    db.cleanup_wal()

# Thread-safe multi-threaded access
os.environ["ODB_KEY"] = "my-secret-key"
db = ThreadSafeOverDrive.open_encrypted("app.odb", "ODB_KEY")
```

---

## Node.js SDK

```bash
npm install overdrive-db
```

```javascript
const { OverDrive, SharedOverDrive } = require('overdrive-db');

const db = new OverDrive('myapp.odb'); // permissions auto-hardened

// Create table & insert
db.createTable('products');
const id = db.insert('products', { name: 'Laptop', price: 999.99 });

// ✅ Safe parameterized query
const results = db.querySafe('SELECT * FROM products WHERE price > ?', ['500']);
console.table(results);

// Backup & WAL cleanup
db.backup('backups/products.odb');
db.cleanupWal();

// Async-safe shared access across async handlers
const shared = new SharedOverDrive('app.odb');
await shared.query('SELECT * FROM products');

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

## Go SDK

```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go
```

```go
import "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go"

func main() {
    // Open (permissions auto-hardened)
    db, _ := overdrive.Open("myapp.odb")
    defer db.Close()

    db.CreateTable("users")
    id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})

    // ✅ Safe parameterized query
    results, _ := db.QuerySafe("SELECT * FROM users WHERE age > ?", "25")

    // Thread-safe access
    safe, _ := overdrive.OpenSafe("app.odb")
    go safe.Query("SELECT * FROM users")

    // Backup + WAL cleanup
    db.Backup("backups/app.odb")
    db.CleanupWAL()
    _ = id
}
```

---

## Java SDK

```xml
<!-- Add to pom.xml -->
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.3.0</version>
</dependency>
```

```java
import com.afot.overdrive.OverDrive;
import com.afot.overdrive.OverDriveSafe;
import java.util.Map;

public class Main {
    public static void main(String[] args) throws Exception {
        // Open with auto permission hardening
        try (OverDrive db = OverDrive.open("myapp.odb")) {
            db.createTable("users");
            String id = db.insert("users", Map.of("name", "Alice", "age", 30));

            // ✅ Safe parameterized query
            var rows = db.querySafe("SELECT * FROM users WHERE name = ?", userInput);

            // Backup + WAL cleanup
            db.backup("backups/app.odb");
            db.cleanupWal();
        }
        // Thread-safe access
        try (OverDriveSafe safe = OverDriveSafe.open("app.odb")) {
            safe.insert("users", Map.of("name", "Bob"));
        }
    }
}
```

---

## Architecture

```
┌─────────────────────────────────────┐
│           Your Application          │
│                                     │
│  db = OverDriveDB::open("app.odb")  │
│  db.query_safe("WHERE name = ?", …) │
│  shared = SharedDB::open(…)         │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│    OverDrive SDK (1.3.0) — Secure   │
│                                     │
│  OverDriveDB  │  QueryEngine  │ FFI │
│  SharedDB     │  query_safe   │     │
│  SecretKey    │  set_perms    │     │
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
         │  app.odb   │  ← AES-256-GCM encrypted
         │  (chmod 600│     owner-only permissions
         └────────────┘
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
├── Cargo.toml           # Crate configuration (v1.3.0)
├── README.md            # This file
├── LICENSE              # MIT License
│
├── src/
│   ├── lib.rs           # OverDriveDB + all security APIs
│   ├── shared.rs        # SharedDB — thread-safe wrapper (NEW v1.3.0)
│   ├── query_engine.rs  # Embedded SQL parser & executor
│   ├── result.rs        # SdkError types (SecurityError, BackupError)
│   └── ffi.rs           # C FFI extern functions
│
├── python/overdrive/
│   └── __init__.py      # Python ctypes wrapper + security APIs
│
├── nodejs/
│   └── index.js         # Node.js ffi-napi wrapper + security APIs
│
├── go/
│   └── overdrive.go     # Go CGo bindings + security APIs + SafeDB
│
├── java/src/.../
│   ├── OverDrive.java   # Java JNA wrapper + security APIs
│   └── OverDriveSafe.java # Thread-safe wrapper (in OverDrive.java)
│
├── docs/
│   ├── quickstart.md    # Getting started guide
│   ├── api-reference.md # Full API documentation
│   ├── sql-guide.md     # SQL syntax reference
│   ├── python-guide.md  # Python SDK guide
│   ├── nodejs-guide.md  # Node.js SDK guide
│   ├── c-guide.md       # C/C++ integration guide
│   └── architecture.md  # Architecture + v1.3.0 Security Model
│
└── examples/
    ├── basic_crud.rs     # Basic CRUD operations
    ├── sql_queries.rs    # SQL query examples
    ├── batch_ops.rs      # Batch operations
    └── secure_open.rs    # 🔐 Security hardening demo (NEW v1.3.0)
```

---

## Links

| Resource | URL |
|---|---|
| 📦 **Download** | [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases) |
| � **API Docs** | [docs/api-reference.md](docs/api-reference.md) |
| � **Quick Start** | [docs/quickstart.md](docs/quickstart.md) |
| 🏠 **Homepage** | [Official Website](https://all-for-one-tech.github.io/OverDrive-DB_SDK/) |
| 📖 **Developer Guide** | [Step-by-Step Guide](https://all-for-one-tech.github.io/OverDrive-DB_SDK/guide.html) |
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
  <a href="https://all-for-one-tech.github.io/OverDrive-DB_SDK/">Official Website</a> •
  <a href="https://all-for-one-tech.github.io/OverDrive-DB_SDK/guide.html">Developer Guide</a>
</p>

---

#afot #OverDriveDb #InCodeSDK #EmbeddedDB #MrV2K #AllForOneTech #EmbeddeDB #HybridDB
