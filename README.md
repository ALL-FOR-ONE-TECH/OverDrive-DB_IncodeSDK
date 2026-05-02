# OverDrive-DB InCode SDK v2.2.0

Zero-config embedded document database with SQL, MVCC transactions, and 6 storage engines — in 5 languages.

[![version](https://img.shields.io/badge/version-2.2.0-blue)](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)
[![Rust](https://img.shields.io/badge/Rust-10%2F10-brightgreen)](rust/)
[![Node.js](https://img.shields.io/badge/Node.js-10%2F10-brightgreen)](nodejs/)
[![Go](https://img.shields.io/badge/Go-10%2F10-brightgreen)](go/)
[![Java](https://img.shields.io/badge/Java-10%2F10-brightgreen)](java/)
[![Python](https://img.shields.io/badge/Python-10%2F10-brightgreen)](python/)

---

## Features

| Feature | Description |
|---|---|
| Zero-config | Open a file, start querying — no setup needed |
| JSON Native | Store, query, and index JSON documents |
| SQL Queries | SELECT, INSERT, UPDATE, DELETE, WHERE, ORDER BY, LIMIT |
| Aggregations | COUNT, SUM, AVG, MIN, MAX, GROUP BY |
| Full-text Search | Built-in text search across documents |
| B-Tree Indexes | Secondary indexes for fast lookups |
| ACID Transactions | MVCC with 4 isolation levels |
| Encryption | AES-256-GCM via Argon2id key derivation |
| RAM Engine | Sub-microsecond in-memory storage |
| Watchdog | File integrity monitoring |
| Cross-platform | Windows x64, Linux x64/ARM64, macOS x64/ARM64 |

## 6 Storage Engines

| Engine | Use Case | Latency |
|---|---|---|
| **Disk** (default) | General-purpose persistent storage | ~1ms |
| **RAM** | Caching, sessions, leaderboards | <1µs |
| **Vector** | Similarity search, embeddings | ~5ms |
| **Time-Series** | Metrics, IoT, logs | ~2ms |
| **Graph** | Social networks, knowledge graphs | ~3ms |
| **Streaming** | Event queues, message brokers | ~1ms |

---

## Quick Start

### Rust
```rust
use overdrive::OverdriveDb;
use serde_json::json;

let mut odb = OverdriveDb::open("app.odb").unwrap();
odb.create_table("users").unwrap();
let id = odb.insert("users", &json!({"name":"Alice","age":30})).unwrap();
let doc = odb.get("users", &id).unwrap().unwrap();
println!("{}", doc["name"]);  // "Alice"
odb.close().ok();
```

### Node.js
```js
const { OverdriveDb } = require('overdrive-db');

const odb = OverdriveDb.open('app.odb');
odb.createTable('users');
const id  = odb.insert('users', { name: 'Alice', age: 30 });
const doc = odb.get('users', id);
console.log(doc.name);  // "Alice"
odb.close();
```

### Go
```go
import overdrive "github.com/ALL-FOR-ONE-TECH/overdrive-db-go"

odb, _ := overdrive.Open("app.odb")
defer odb.Close()
odb.CreateTable("users")
id, _ := odb.Insert("users", map[string]any{"name": "Alice", "age": 30})
doc, _ := odb.Get("users", id)
fmt.Println(doc["name"])  // Alice
```

### Java
```java
try (OverdriveDb odb = OverdriveDb.open("app.odb")) {
    odb.createTable("users");
    String id = odb.insert("users", Map.of("name","Alice","age",30));
    Map<String,Object> doc = odb.get("users", id);
    System.out.println(doc.get("name"));  // Alice
}
```

### Python
```python
from overdrive import OverdriveDb

with OverdriveDb.open("app.odb") as odb:
    odb.create_table("users")
    id = odb.insert("users", {"name": "Alice", "age": 30})
    doc = odb.get("users", id)
    print(doc["name"])  # Alice
```

---

## Full API Reference

### Lifecycle

| Operation | Rust | Node.js | Java | Go | Python |
|---|---|---|---|---|---|
| Open | `OverdriveDb::open(path)` | `OverdriveDb.open(path)` | `OverdriveDb.open(path)` | `Open(path)` | `OverdriveDb.open(path)` |
| Open+Engine | `open_with_options(path,opts)` | `open(path,{engine})` | `open(path,"RAM",null)` | `Open(path,WithEngine("RAM"))` | `open(path,engine="RAM")` |
| Open+Password | `open_with_options(path,opts)` | `open(path,{password})` | `open(path,"Disk",pwd)` | `Open(path,WithPassword(pwd))` | `open(path,password=pwd)` |
| Close | `odb.close()` | `odb.close()` | `odb.close()` | `odb.Close()` | `odb.close()` |
| Sync | `odb.sync()` | `odb.sync()` | `odb.sync()` | `odb.Sync()` | `odb.sync()` |
| Version | `OverdriveDb::version()` | `OverdriveDb.version()` | `OverdriveDb.version()` | `Version()` | `OverdriveDb.version()` |

### Tables

| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Create | `odb.create_table(name)` | `odb.createTable(name)` | `odb.CreateTable(name)` | `odb.create_table(name)` |
| Drop | `odb.drop_table(name)` | `odb.dropTable(name)` | `odb.DropTable(name)` | `odb.drop_table(name)` |
| List | `odb.list_tables()` | `odb.listTables()` | `odb.ListTables()` | `odb.list_tables()` |
| Exists | `odb.table_exists(name)` | `odb.tableExists(name)` | `odb.TableExists(name)` | `odb.table_exists(name)` |

### CRUD

| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Insert | `odb.insert(table, &doc)` | `odb.insert(table, doc)` | `odb.Insert(table, doc)` | `odb.insert(table, doc)` |
| Insert batch | `odb.insert_batch(table, &docs)` | `odb.insertMany(table, docs)` | `odb.InsertBatch(table, docs)` | `odb.insert_many(table, docs)` |
| Get | `odb.get(table, id)` | `odb.get(table, id)` | `odb.Get(table, id)` | `odb.get(table, id)` |
| Update | `odb.update(table, id, &patch)` | `odb.update(table, id, patch)` | `odb.Update(table, id, patch)` | `odb.update(table, id, patch)` |
| Delete | `odb.delete(table, id)` | `odb.delete(table, id)` | `odb.Delete(table, id)` | `odb.delete(table, id)` |
| Count | `odb.count(table)` | `odb.count(table)` | `odb.Count(table)` | `odb.count(table)` |

### Query & Search

| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| SQL Query | `odb.query(sql)` | `odb.query(sql)` | `odb.Query(sql)` | `odb.query(sql)` |
| Search | `odb.search(table, text)` | `odb.search(table, text)` | `odb.Search(table, text)` | `odb.search(table, text)` |

### Transactions

| Level | Value |
|---|---|
| Read Uncommitted | 0 |
| Read Committed | 1 (default) |
| Repeatable Read | 2 |
| Serializable | 3 |

| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Begin | `odb.begin_transaction(iso)` | `odb.beginTransaction(iso)` | `odb.BeginTransaction(iso)` | `odb.begin_transaction(iso)` |
| Commit | `odb.commit_transaction(&txn)` | `odb.commitTransaction(id)` | `odb.CommitTransaction(id)` | `odb.commit_transaction(id)` |
| Abort | `odb.abort_transaction(&txn)` | `odb.abortTransaction(id)` | `odb.AbortTransaction(id)` | `odb.abort_transaction(id)` |
| Callback | `odb.transaction(iso, \|odb\|{})` | `odb.transaction(fn)` | `odb.Transaction(fn, iso)` | `odb.transaction(fn)` |

---

## Error Codes

| Code | Type | When |
|---|---|---|
| `ODB-AUTH-*` | Authentication | Wrong password, key too short |
| `ODB-TABLE-*` | Table | Not found, already exists |
| `ODB-QUERY-*` | Query | SQL syntax error |
| `ODB-TXN-*` | Transaction | Deadlock, conflict |
| `ODB-IO-*` | I/O | File not found, corrupted |
| `ODB-FFI-*` | FFI | Native library not found |

---

## Native Binary Distribution

The native library lives in `lib/{os}-{arch}/`:

```
lib/
  windows-x64/  overdrive.dll       (3.4MB)
  linux-x64/    liboverdrive.so
  linux-arm64/  liboverdrive.so
  macos-x64/    liboverdrive.dylib
  macos-arm64/  liboverdrive.dylib
```

**Override path:** Set `OVERDRIVE_LIB_PATH` env var to load a custom native lib.

**Rebuild from source:**
```powershell
# Windows
.\scripts\build-native.ps1

# Linux / macOS
./scripts/build-native.sh
```

---

## E2E Test Results — v2.2.0

| SDK | Tests | Status |
|---|---|---|
| 🦀 Rust | 10/10 | ✅ PASS |
| 🟢 Node.js | 10/10 | ✅ PASS |
| 🐹 Go | 10/10 | ✅ PASS |
| ☕ Java | 10/10 | ✅ PASS |
| 🐍 Python | 10/10 | ✅ PASS |
| **Total** | **50/50** | ✅ **ALL PASS** |

---

## License

MIT OR Apache-2.0 — © 2026 [AFOT](https://afot.in)
