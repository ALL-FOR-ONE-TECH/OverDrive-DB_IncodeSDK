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
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/packages"><img src="https://img.shields.io/badge/maven-1.4.0-007ec6?style=flat-square&logo=apache-maven" alt="maven"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat-square" alt="license"/></a>
  </p>
</p>

#afot #OverDriveDb #InCodeSDK #EmbeddedDB

---

## What is OverDrive?

OverDrive is an **embeddable, zero-config document database** for Rust, Python, Node.js, Java, Go, and C/C++. It stores JSON documents, supports SQL queries, full-text search, secondary indexing, and ACID transactions — all within a single library. No external server. No network. Just a file.

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
| 🛡️ **Security Hardened** | Key from env vars, `querySafe()` injection blocking, auto `chmod 600`, WAL cleanup, thread-safe wrappers |
| 🚀 **RAM Engine** | Sub-microsecond in-memory storage with snapshot/restore |
| 🔭 **Watchdog** | File integrity monitoring — detect corruption before it matters |
| 🌍 **Cross-platform** | Windows x64, Linux x64/ARM64, macOS x64/ARM64 |
| 🔗 **Multi-language** | Rust, Python, Node.js, Java, Go, C/C++ |

---

## Quick Start

### Install

**Python:**
```bash
pip install overdrive-db
```

**Node.js:**
```bash
npm install overdrive-db
```

**Java (GitHub Packages):**
```xml
<!-- Step 1: Add to ~/.m2/settings.xml (see INSTALL.md for auth setup) -->
<!-- Step 2: Add repository to pom.xml -->
<repositories>
  <repository>
    <id>github-overdrive</id>
    <url>https://maven.pkg.github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK</url>
  </repository>
</repositories>
<!-- Step 3: Add dependency -->
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.4.3</version>
</dependency>
```
> ⚠️ GitHub Packages requires a PAT token for auth. See [INSTALL.md](INSTALL.md#java) for full setup.

**Go (Windows — no CGo/GCC needed ✅):**
```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest
```

**Rust:**
```toml
[dependencies]
overdrive-db = "1.4.3"
```

> **Auto-setup:** `pip install`, `npm install`, and `cargo add` all bundle or auto-download the native library. See [INSTALL.md](INSTALL.md) for full details.

---

## Hello World — 3 Lines

### Python
```python
from overdrive import OverDrive

db = OverDrive.open("myapp.odb")
db.insert("users", {"name": "Alice", "age": 30})  # table auto-created
print(db.query("SELECT * FROM users"))
db.close()
```

### Node.js
```javascript
const { OverDrive } = require('overdrive-db');

const db = OverDrive.open('myapp.odb');
db.insert('users', { name: 'Alice', age: 30 });  // table auto-created
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
let id = db.insert("users", &serde_json::json!({"name": "Alice", "age": 30})).unwrap();
let result = db.query("SELECT * FROM users WHERE age > 25").unwrap();
println!("{} users found in {:.2}ms", result.rows.len(), result.execution_time_ms);
db.close().unwrap();
```

---

## v1.4.0 — What's New

### Password-protected databases
```python
db = OverDrive.open("secure.odb", password="my-secret-pass")
```

### RAM engine for sub-microsecond caching
```python
cache = OverDrive.open("cache.odb", engine="RAM")
cache.insert("sessions", {"user_id": 123, "token": "abc"})
cache.snapshot("./backup/cache.odb")   # persist to disk
usage = cache.memoryUsage()            # {"bytes": ..., "mb": ..., "percent": ...}
```

### File integrity monitoring
```python
report = OverDrive.watchdog("app.odb")
if report.integrity_status == "corrupted":
    print(f"Corruption: {report.corruption_details}")
```

### Transaction callbacks
```python
result = db.transaction(lambda txn:
    db.insert("orders", {"item": "widget"})
)  # auto-commits on success, auto-rolls back on exception
```

### Helper methods
```python
user    = db.findOne("users", "age > 25")
users   = db.findAll("users", order_by="name", limit=10)
count   = db.updateMany("users", "status = 'trial'", {"status": "active"})
deleted = db.deleteMany("logs", "created_at < '2025-01-01'")
n       = db.countWhere("users", "active = 1")
exists  = db.exists("users", "users_1")
```

---

## All 6 Storage Engines

| Engine | Use Case | Latency |
|--------|----------|---------|
| `Disk` (default) | General-purpose persistent storage | ~1ms |
| `RAM` | Caching, sessions, leaderboards | <1µs |
| `Vector` | Similarity search, embeddings | ~5ms |
| `Time-Series` | Metrics, IoT, logs | ~2ms |
| `Graph` | Social networks, knowledge graphs | ~3ms |
| `Streaming` | Event queues, message brokers | ~1ms |

```python
# Select engine on open
db = OverDrive.open("app.odb", engine="RAM")

# Or per-table
db = OverDrive.open("app.odb")
db.createTable("hot_cache", engine="RAM")   # this table in RAM
db.insert("users", {"name": "Alice"})       # this table on disk (auto-created)
```

---

## Security

| Feature | Description |
|---------|-------------|
| `open(path, password=...)` | AES-256-GCM encryption via Argon2id key derivation |
| `openEncrypted(path, "ODB_KEY")` | Key from environment variable — never hardcoded |
| `querySafe(sql, params)` | SQL injection prevention with parameterized queries |
| `backup(dest)` | Encrypted backup + permission hardening |
| `cleanupWal()` | Delete WAL after commit — prevents replay attacks |
| Auto on `open()` | `chmod 600` / Windows ACL on every database file |

```python
# Never hardcode passwords — use environment variables
import os
db = OverDrive.open("secure.odb", password=os.environ["DB_PASSWORD"])

# Or use the env-var method
db = OverDrive.open_encrypted("app.odb", "ODB_KEY")

# Safe parameterized queries
results = db.querySafe("SELECT * FROM users WHERE name = ?", user_input)
```

---

## Downloads

| Platform | File |
|----------|------|
| Windows x64 | [`overdrive.dll`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| Linux x64 | [`liboverdrive.so`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| Linux ARM64 | [`liboverdrive-arm64.so`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| macOS x64 | [`liboverdrive.dylib`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| macOS ARM64 | [`liboverdrive-arm64.dylib`](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |

---

## Documentation

| Resource | Link |
|----------|------|
| 📖 Quick Start | [docs/quickstart.md](docs/quickstart.md) |
| 📚 API Reference | [docs/api-reference.md](docs/api-reference.md) |
| 🔄 Migration Guide | [docs/migration-v1.3-to-v1.4.md](docs/migration-v1.3-to-v1.4.md) |
| 🔐 Security Guide | [docs/security.md](docs/security.md) |
| 🐍 Python Guide | [docs/python-guide.md](docs/python-guide.md) |
| 🟨 Node.js Guide | [docs/nodejs-guide.md](docs/nodejs-guide.md) |
| ☕ Java Guide | [docs/java-guide.md](docs/java-guide.md) |
| 🐹 Go Guide | [docs/go-guide.md](docs/go-guide.md) |
| 💻 Examples | [examples/](examples/) |

---

## Project Structure

```
OverDrive-DB_IncodeSDK/
├── Cargo.toml              # Rust crate (dynamic loader)
├── README.md               # This file
├── LICENSE
│
├── src/
│   ├── lib.rs              # OverDriveDB Rust API
│   ├── dynamic.rs          # Runtime native library loader
│   ├── ffi.rs              # C FFI exports
│   ├── shared.rs           # Thread-safe SharedDB wrapper
│   ├── query_engine.rs     # SQL query types
│   └── result.rs           # Error types
│
├── python/overdrive/
│   └── __init__.py         # Python SDK (ctypes)
│
├── nodejs/
│   ├── index.js            # Node.js SDK (koffi)
│   └── index.d.ts          # TypeScript definitions
│
├── java/src/
│   └── .../OverDrive.java  # Java SDK (JNA)
│
├── go/
│   └── overdrive.go        # Go SDK (CGo)
│
├── c/include/
│   └── overdrive.h         # C/C++ header
│
├── docs/                   # Full documentation
├── examples/               # Working code examples
└── .github/workflows/      # CI/CD pipelines
```

---

## Comparison

| Feature | OverDrive | SQLite | MongoDB | Redis |
|---------|-----------|--------|---------|-------|
| Deployment | Library | Library | Server | Server |
| Data Model | JSON Documents | Relational | JSON Documents | Key-Value |
| SQL Support | ✅ | ✅ | ❌ | ❌ |
| Schema | Optional | Required | Optional | None |
| Full-text Search | ✅ Built-in | FTS5 Extension | ✅ | ❌ |
| Encryption | ✅ AES-256 | SEE (paid) | ✅ Enterprise | ✅ |
| RAM Engine | ✅ | ❌ | ❌ | ✅ |
| Watchdog | ✅ | ❌ | ❌ | ❌ |
| Size | ~3MB | ~1.2MB | ~200MB | ~3MB |

---

## Links

| Resource | URL |
|----------|-----|
| 📦 Releases | [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases) |
| 🐛 Issues | [GitHub Issues](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues) |
| 💬 Discussions | [GitHub Discussions](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/discussions) |
| 🏠 Homepage | [overdrive-db.com](https://overdrive-db.com) |
| 📦 npm | [npmjs.com/package/overdrive-db](https://www.npmjs.com/package/overdrive-db) |
| 🐍 PyPI | [pypi.org/project/overdrive-db](https://pypi.org/project/overdrive-db/) |

---

## License

Licensed under either of:
- **MIT License** ([LICENSE](LICENSE))
- **Apache License 2.0** ([LICENSE](LICENSE))

at your option.

---

<p align="center">
  Built by <a href="https://github.com/ALL-FOR-ONE-TECH"><strong>ALL FOR ONE TECH</strong></a> •
  <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases">Downloads</a> •
  <a href="https://overdrive-db.com">Website</a>
</p>

#afot #OverDriveDb #InCodeSDK #EmbeddedDB #MrV2K #AllForOneTech #HybridDB
