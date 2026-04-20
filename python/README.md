<p align="center">
  <h1 align="center">⚡ OverDrive-DB — InCode SDK v1.4.4</h1>
  <p align="center">
    <strong>Embeddable hybrid SQL+NoSQL document database. Like SQLite, but for JSON.</strong><br/>
    Import the package. Open a file. Query your data. <em>No server needed.</em>
  </p>
  <p align="center">
    <a href="https://crates.io/crates/overdrive-db"><img src="https://img.shields.io/crates/v/overdrive-db?style=flat-square&color=orange&logo=rust" alt="crates.io"/></a>
    <a href="https://pypi.org/project/overdrive-db/"><img src="https://img.shields.io/pypi/v/overdrive-db?style=flat-square&color=3776ab&logo=python" alt="PyPI"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/packages"><img src="https://img.shields.io/badge/maven-1.4.4-007ec6?style=flat-square&logo=apache-maven" alt="maven"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat-square" alt="license"/></a>
  </p>
</p>

---

## Install

```bash
pip install overdrive-db                # Python
npm install overdrive-db                # Node.js
cargo add overdrive-db                  # Rust
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.4.4  # Go
```

**Java (Maven):**
```xml
<repositories>
  <repository>
    <id>github-overdrive</id>
    <url>https://maven.pkg.github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK</url>
  </repository>
</repositories>
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.4.4</version>
</dependency>
```

**C/C++:** Download `overdrive.h` + native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

---

## Hello World

### Python
```python
from overdrive import OverDrive

db = OverDrive.open("myapp.odb")
db.insert("users", {"name": "Alice", "age": 30})
print(db.query("SELECT * FROM users"))
db.close()
```

### Node.js
```javascript
const { OverDrive } = require('overdrive-db');

const db = OverDrive.open('myapp.odb');
db.insert('users', { name: 'Alice', age: 30 });
console.log(db.query('SELECT * FROM users'));
db.close();
```

### Java
```java
import com.afot.overdrive.OverDrive;

try (OverDrive db = OverDrive.open("myapp.odb")) {
    db.insert("users", Map.of("name", "Alice", "age", 30));
    System.out.println(db.query("SELECT * FROM users"));
}
```

### Go
```go
db, _ := overdrive.Open("myapp.odb")
defer db.Close()
db.Insert("users", map[string]any{"name": "Alice", "age": 30})
result, _ := db.Query("SELECT * FROM users")
fmt.Println(result.Rows)
```

### Rust
```rust
use overdrive::OverDriveDB;

let mut db = OverDriveDB::open("myapp.odb").unwrap();
db.create_table("users").unwrap();
db.insert("users", &serde_json::json!({"name": "Alice", "age": 30})).unwrap();
let result = db.query("SELECT * FROM users WHERE age > 25").unwrap();
println!("{} rows", result.rows.len());
db.close().unwrap();
```

### C
```c
#include "overdrive.h"

ODB* db = overdrive_open("myapp.odb");
overdrive_create_table(db, "users");
char* id = overdrive_insert(db, "users", "{\"name\":\"Alice\",\"age\":30}");
overdrive_free_string(id);

char* result = overdrive_query(db, "SELECT * FROM users");
printf("%s\n", result);
overdrive_free_string(result);

overdrive_close(db);
```

---

## Features

| Feature | Description |
|---|---|
| **Zero-config** | Open a file, start querying — no setup needed |
| **JSON Native** | Store, query, and index JSON documents |
| **SQL Queries** | `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `WHERE`, `ORDER BY`, `LIMIT` |
| **Aggregations** | `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `GROUP BY` |
| **Full-text Search** | Built-in text search across documents |
| **B-Tree Indexes** | Secondary indexes for fast lookups |
| **ACID Transactions** | MVCC with 4 isolation levels |
| **Encryption** | AES-256-GCM via Argon2id key derivation |
| **RAM Engine** | Sub-microsecond in-memory storage with snapshot/restore |
| **Watchdog** | File integrity monitoring |
| **Cross-platform** | Windows x64, Linux x64/ARM64, macOS x64/ARM64 |

---

## 6 Storage Engines

| Engine | Use Case | Latency |
|--------|----------|---------|
| `Disk` (default) | General-purpose persistent storage | ~1ms |
| `RAM` | Caching, sessions, leaderboards | <1µs |
| `Vector` | Similarity search, embeddings | ~5ms |
| `Time-Series` | Metrics, IoT, logs | ~2ms |
| `Graph` | Social networks, knowledge graphs | ~3ms |
| `Streaming` | Event queues, message brokers | ~1ms |

---

## API Reference

All SDKs share the same API surface. Method names follow each language's conventions.

### Database Lifecycle

| Python | Node.js | Java | Go | Rust | C |
|--------|---------|------|----|------|---|
| `OverDrive.open(path)` | `OverDrive.open(path)` | `OverDrive.open(path)` | `overdrive.Open(path)` | `OverDriveDB::open(path)` | `overdrive_open(path)` |
| `db.close()` | `db.close()` | `db.close()` | `db.Close()` | `db.close()` | `overdrive_close(db)` |
| `db.sync()` | `db.sync()` | `db.sync()` | `db.Sync()` | `db.sync()` | `overdrive_sync(db)` |
| `OverDrive.version()` | `OverDrive.version()` | `OverDrive.version()` | `overdrive.Version()` | `OverDriveDB::version()` | `overdrive_version()` |

### CRUD Operations

| Operation | Python | Node.js / Java | Go | Rust |
|-----------|--------|----------------|-----|------|
| Insert | `db.insert(table, doc)` | `db.insert(table, doc)` | `db.Insert(table, doc)` | `db.insert(table, &doc)` |
| Get | `db.get(table, id)` | `db.get(table, id)` | `db.Get(table, id)` | `db.get(table, id)` |
| Update | `db.update(table, id, updates)` | `db.update(table, id, updates)` | `db.Update(table, id, updates)` | `db.update(table, id, &updates)` |
| Delete | `db.delete(table, id)` | `db.delete(table, id)` | `db.Delete(table, id)` | `db.delete(table, id)` |
| Count | `db.count(table)` | `db.count(table)` | `db.Count(table)` | `db.count(table)` |
| Query | `db.query(sql)` | `db.query(sql)` | `db.Query(sql)` | `db.query(sql)` |
| Safe Query | `db.query_safe(sql, params)` | `db.querySafe(sql, params)` | `db.QuerySafe(sql, params...)` | `db.query_safe(sql, &params)` |
| Search | `db.search(table, text)` | `db.search(table, text)` | `db.Search(table, text)` | `db.search(table, text)` |

### Tables

| Operation | Python | Node.js / Java | Go |
|-----------|--------|----------------|-----|
| Create | `db.create_table(name)` | `db.createTable(name)` | `db.CreateTable(name)` |
| Drop | `db.drop_table(name)` | `db.dropTable(name)` | `db.DropTable(name)` |
| List | `db.list_tables()` | `db.listTables()` | `db.ListTables()` |
| Exists | `db.table_exists(name)` | `db.tableExists(name)` | `db.TableExists(name)` |

### v1.4 Features

| Feature | Python | Node.js | Java | Go |
|---------|--------|---------|------|----|
| Password open | `OverDrive.open(path, password=...)` | `OverDrive.open(path, {password:...})` | `OverDrive.open(path, password)` | `overdrive.Open(path, WithPassword(...))` |
| RAM engine | `OverDrive.open(path, engine="RAM")` | `OverDrive.open(path, {engine:"RAM"})` | `OverDrive.open(path, "RAM")` | `overdrive.Open(path, WithEngine("RAM"))` |
| Watchdog | `OverDrive.watchdog(path)` | `OverDrive.watchdog(path)` | `OverDrive.watchdog(path)` | `overdrive.Watchdog(path)` |
| Transaction callback | `db.transaction(fn)` | `db.transaction(fn)` | `db.transaction(fn)` | `db.Transaction(fn, isolation)` |
| Find one | `db.findOne(table, where)` | `db.findOne(table, where)` | `db.findOne(table, where)` | `db.FindOne(table, where)` |
| Find all | `db.findAll(table, ...)` | `db.findAll(table, ...)` | `db.findAll(table, ...)` | `db.FindAll(table, ...)` |
| Update many | `db.updateMany(table, where, updates)` | `db.updateMany(...)` | `db.updateMany(...)` | `db.UpdateMany(...)` |
| Delete many | `db.deleteMany(table, where)` | `db.deleteMany(...)` | `db.deleteMany(...)` | `db.DeleteMany(...)` |
| Snapshot | `db.snapshot(path)` | `db.snapshot(path)` | `db.snapshot(path)` | `db.Snapshot(path)` |
| Memory usage | `db.memoryUsage()` | `db.memoryUsage()` | `db.memoryUsage()` | `db.MemoryUsageStats()` |

### Transactions

All SDKs support MVCC transactions with 4 isolation levels:

| Level | Value | Description |
|-------|-------|-------------|
| Read Uncommitted | 0 | Fastest, least safe |
| Read Committed | 1 | Default |
| Repeatable Read | 2 | Snapshot isolation |
| Serializable | 3 | Full isolation |

### Security

| Feature | Usage |
|---------|-------|
| Password encryption | `open(path, password=...)` — AES-256-GCM via Argon2id |
| Env var key | `open_encrypted(path, "ODB_KEY")` — key from environment |
| Parameterized queries | `query_safe(sql, params)` — blocks SQL injection |
| Encrypted backup | `backup(dest)` — syncs + copies + hardens permissions |
| WAL cleanup | `cleanup_wal()` — removes replay-attack surface |
| File permissions | Auto `chmod 600` (Linux/Mac) or Windows ACL on open |

### C/C++ Memory Rules

Every `char*` returned by `overdrive_*` functions **must** be freed with `overdrive_free_string()`.
Do **not** free: `overdrive_last_error()`, `overdrive_version()` (static pointers).

### Error Codes

| Code | Type | When |
|------|------|------|
| `ODB-AUTH-*` | Authentication | Wrong password, key too short |
| `ODB-TABLE-*` | Table | Not found, already exists |
| `ODB-QUERY-*` | Query | SQL syntax error |
| `ODB-TXN-*` | Transaction | Deadlock, conflict |
| `ODB-IO-*` | I/O | File not found, corrupted |
| `ODB-FFI-*` | FFI | Native library not found |

---

## Native Library Downloads

| Platform | File | Size |
|----------|------|------|
| Windows x64 | `overdrive.dll` | ~3.3 MB |
| Linux x64 | `liboverdrive.so` | ~4.1 MB |
| Linux ARM64 | `liboverdrive-arm64.so` | ~3.9 MB |
| macOS x64 | `liboverdrive.dylib` | ~3.8 MB |
| macOS ARM64 | `liboverdrive-arm64.dylib` | ~3.6 MB |

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

> Python and Rust auto-download the native library on first use. Node.js downloads on `npm install`. Java and Go require manual placement.

---

## Project Structure

```
OverDrive-DB_IncodeSDK/
├── Cargo.toml              # Rust crate (crates.io)
├── README.md               # This file
├── src/                    # Rust SDK core
│   ├── lib.rs              # OverDriveDB API
│   ├── dynamic.rs          # Runtime native library loader
│   ├── ffi.rs              # C FFI exports (overdrive.dll)
│   ├── shared.rs           # Thread-safe wrapper
│   ├── query_engine.rs     # SQL query types
│   └── result.rs           # Error types
├── python/overdrive/       # Python SDK (ctypes)
├── nodejs/                 # Node.js SDK (koffi + TypeScript)
├── java/src/               # Java SDK (JNA)
├── go/                     # Go SDK (syscall, no CGo on Windows)
├── c/include/overdrive.h   # C/C++ header
├── docs/                   # Full documentation
├── examples/               # Working examples for all languages
└── .github/workflows/      # CI/CD pipelines
```

---

## Links

| Resource | URL |
|----------|-----|
| GitHub | https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK |
| Releases | https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases |
| crates.io | https://crates.io/crates/overdrive-db |
| PyPI | https://pypi.org/project/overdrive-db/ |
| Issues | https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues |
| Website | https://overdrive-db.com |

## License

Licensed under either **MIT** or **Apache-2.0**, at your option.

---

<p align="center">
  Built by <a href="https://github.com/ALL-FOR-ONE-TECH"><strong>ALL FOR ONE TECH</strong></a>
</p>
