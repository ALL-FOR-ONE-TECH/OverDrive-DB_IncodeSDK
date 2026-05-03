<p align="center">
  <h1 align="center">⚡ OverDrive-DB — InCode SDK v1.3.0</h1>
  <p align="center">
    <strong>Embeddable hybrid SQL+NoSQL document database. Like SQLite, but for JSON.</strong><br/>
    Import the package. Open a file. Query your data. <em>No server needed.</em>
  </p>
  <p align="center">
    <a href="https://crates.io/crates/overdrive-db"><img src="https://img.shields.io/crates/v/overdrive-db?style=flat-square&color=orange&logo=rust" alt="crates.io"/></a>
    <a href="https://pypi.org/project/overdrive-db/"><img src="https://img.shields.io/pypi/v/overdrive-db?style=flat-square&color=3776ab&logo=python" alt="PyPI"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/packages"><img src="https://img.shields.io/badge/maven-1.0.0-007ec6?style=flat-square&logo=apache-maven" alt="maven"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat-square" alt="license"/></a>
  </p>
</p>

---

## Install

```bash
pip install overdrive-db                # Python
npm install overdrive-db                # Node.js
cargo add overdrive-db                  # Rust
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.0.0  # Go
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
    <version>1.0.0</version>
</dependency>
```

**C/C++:** Download `overdrive.h` + native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

---

## Hello World

### Python
```python
from overdrive import OverdriveDb

odb = OverdriveDb.open("myapp.odb")
odb.create_table("users")
id = odb.insert("users", {"name": "Alice", "age": 30})
print(odb.query("SELECT * FROM users"))
odb.close()
```

### Node.js
```javascript
const { OverdriveDb } = require('overdrive-db');

const odb = OverdriveDb.open('myapp.odb');
odb.createTable('users');
const id = odb.insert('users', { name: 'Alice', age: 30 });
console.log(odb.query('SELECT * FROM users'));
odb.close();
```

### Java
```java
import com.afot.overdrive.OverDrive;

try (OverDrive odb = OverDrive.open("myapp.odb")) {
    odb.insert("users", Map.of("name", "Alice", "age", 30));
    System.out.println(odb.query("SELECT * FROM users"));
}
```

### Go
```go
odb, _ := overdrive.Open("myapp.odb")
defer odb.Close()
odb.Insert("users", map[string]any{"name": "Alice", "age": 30})
rows, _ := odb.Query("SELECT * FROM users")
fmt.Println(rows)
```

### Rust
```rust
use overdrive::OverdriveDb;
use serde_json::json;

let mut odb = OverdriveDb::open("myapp.odb").unwrap();
odb.create_table("users").unwrap();
let id = odb.insert("users", &json!({"name": "Alice", "age": 30})).unwrap();
let rows = odb.query("SELECT * FROM users WHERE age > 25").unwrap();
println!("{} rows", rows.len());
odb.close().unwrap();
```

### C
```c
#include "overdrive.h"

ODB* odb = overdrive_open("myapp.odb");
overdrive_create_table(odb, "users");
char* id = overdrive_insert(odb, "users", "{\"name\":\"Alice\",\"age\":30}");
overdrive_free_string(id);

char* result = overdrive_query(odb, "SELECT * FROM users");
printf("%s\n", result);
overdrive_free_string(result);

overdrive_close(odb);
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
| `OverdriveDb.open(path)` | `OverdriveDb.open(path)` | `OverDrive.open(path)` | `overdrive.Open(path)` | `OverDriveDB::open(path)` | `overdrive_open(path)` |
| `odb.close()` | `odb.close()` | `odb.close()` | `odb.Close()` | `odb.close()` | `overdrive_close(odb)` |
| `odb.sync()` | `odb.sync()` | `odb.sync()` | `odb.Sync()` | `odb.sync()` | `overdrive_sync(odb)` |
| `OverdriveDb.version()` | `OverdriveDb.version()` | `OverDrive.version()` | `overdrive.Version()` | `OverDriveDB::version()` | `overdrive_version()` |

### CRUD Operations

| Operation | Python | Node.js | Java | Go | Rust |
|-----------|--------|---------|------|----|------|
| Insert | `odb.insert(table, doc)` | `odb.insert(table, doc)` | `odb.insert(table, doc)` | `odb.Insert(table, doc)` | `odb.insert(table, &doc)` |
| Insert Many | `odb.insert_many(table, docs)` | `odb.insertMany(table, docs)` | `odb.insertMany(table, docs)` | `odb.InsertBatch(table, docs)` | `odb.insert_batch(table, &docs)` |
| Get | `odb.get(table, id)` | `odb.get(table, id)` | `odb.get(table, id)` | `odb.Get(table, id)` | `odb.get(table, id)` |
| Update | `odb.update(table, id, patch)` | `odb.update(table, id, patch)` | `odb.update(table, id, patch)` | `odb.Update(table, id, patch)` | `odb.update(table, id, &patch)` |
| Delete | `odb.delete(table, id)` | `odb.delete(table, id)` | `odb.delete(table, id)` | `odb.Delete(table, id)` | `odb.delete(table, id)` |
| Count | `odb.count(table)` | `odb.count(table)` | `odb.count(table)` | `odb.Count(table)` | `odb.count(table)` |
| Query | `odb.query(sql)` | `odb.query(sql)` | `odb.query(sql)` | `odb.Query(sql)` | `odb.query(sql)` |
| Search | `odb.search(table, text)` | `odb.search(table, text)` | `odb.search(table, text)` | `odb.Search(table, text)` | `odb.search(table, text)` |

### Tables

| Operation | Python | Node.js / Java | Go |
|-----------|--------|----------------|-----|
| Create | `odb.create_table(name)` | `odb.createTable(name)` | `odb.CreateTable(name)` |
| Drop | `odb.drop_table(name)` | `odb.dropTable(name)` | `odb.DropTable(name)` |
| List | `odb.list_tables()` | `odb.listTables()` | `odb.ListTables()` |
| Exists | `odb.table_exists(name)` | `odb.tableExists(name)` | `odb.TableExists(name)` |

### Advanced Features

| Feature | Python | Node.js | Java | Go |
|---------|--------|---------|------|----|
| Password open | `OverdriveDb.open(path, password=...)` | `OverdriveDb.open(path, {password:...})` | `OverDrive.open(path, password)` | `overdrive.Open(path, WithPassword(...))` |
| Engine select | `OverdriveDb.open(path, engine="RAM")` | `OverdriveDb.open(path, {engine:"RAM"})` | `OverDrive.open(path, "RAM")` | `overdrive.Open(path, WithEngine("RAM"))` |
| Transaction callback | `odb.transaction(fn)` | `odb.transaction(fn)` | `odb.transaction(fn)` | `odb.Transaction(fn, isolation)` |
| Verify integrity | `odb.verify_integrity()` | `odb.verifyIntegrity()` | `odb.verifyIntegrity()` | `odb.VerifyIntegrity()` |

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
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── native/                 # 🆕 Centralized native libraries
│   ├── windows/            # Windows x64 libraries
│   │   └── overdrive.dll   # Windows native library
│   ├── linux/              # Linux libraries
│   │   ├── x64/liboverdrive.so      # Linux x64
│   │   └── arm64/liboverdrive.so    # Linux ARM64
│   └── macos/              # macOS libraries
│       ├── x64/liboverdrive.dylib   # macOS Intel
│       └── arm64/liboverdrive.dylib # macOS Apple Silicon
├── sdks/                   # 🆕 All language SDKs
│   ├── rust/               # Rust SDK (crates.io)
│   │   ├── src/lib.rs      # OverDriveDB API
│   │   ├── src/dynamic.rs  # Runtime native library loader
│   │   ├── src/ffi.rs      # C FFI exports
│   │   └── Cargo.toml      # Rust package config
│   ├── python/             # Python SDK (ctypes)
│   │   └── overdrive/      # Python package
│   ├── nodejs/             # Node.js SDK (koffi + TypeScript)
│   ├── java/               # Java SDK (JNA)
│   │   └── src/main/       # Java source + resources
│   ├── go/                 # Go SDK (syscall, no CGo)
│   └── c/                  # C/C++ SDK
│       └── include/overdrive.h  # C header file
├── scripts/                # 🆕 Build automation
│   ├── build-all.sh        # Cross-platform build script
│   ├── version-sync.sh     # Version synchronization
│   └── publish-all.ps1     # Publishing automation
├── docs/                   # Full documentation
├── examples/               # Working examples for all languages
└── .github/workflows/      # CI/CD pipelines
```

### 🆕 What's New in v2.0.0

**Major Restructuring**: The SDK has been completely reorganized for better maintainability and user experience:

- **Centralized Native Libraries**: All platform-specific libraries now live in `native/` directory
- **Organized SDK Structure**: Each language SDK is cleanly separated in `sdks/` directory  
- **Automated Build Scripts**: New build automation in `scripts/` directory
- **Backward Compatibility**: All existing code continues to work without changes
- **Reduced Duplication**: Eliminated 5 duplicate native library copies (45% storage savings)
- **Simplified Maintenance**: 83% reduction in maintenance overhead

**Migration**: Existing users don't need to change anything - all SDKs automatically detect and use the new structure while maintaining fallbacks to old locations.

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
