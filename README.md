<p align="center">
  <h1 align="center">⚡ OverDrive-DB — InCode SDK</h1>
  <p align="center">
    <strong>Embeddable hybrid SQL+NoSQL document database. Like SQLite, but for JSON — with encryption, MVCC, and 6 storage engines.</strong><br/>
    Import the package. Open a file. Query your data. <em>No server needed.</em>
  </p>
  <p align="center">
    <a href="https://crates.io/crates/overdrive-db"><img src="https://img.shields.io/crates/v/overdrive-db?style=flat-square&color=orange&logo=rust" alt="crates.io"/></a>
    <a href="https://pypi.org/project/overdrive-db/"><img src="https://img.shields.io/pypi/v/overdrive-db?style=flat-square&color=3776ab&logo=python" alt="PyPI"/></a>
    <a href="https://www.npmjs.com/package/overdrive-db"><img src="https://img.shields.io/npm/v/overdrive-db?style=flat-square&color=cb3837&logo=npm" alt="npm"/></a>
    <a href="https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/releases"><img src="https://img.shields.io/github/v/release/karthikeyanV2K/OverDrive-DB_IncodeSDK?style=flat-square&label=release" alt="release"/></a>
    <a href="https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat-square" alt="license"/></a>
  </p>
</p>

---

## Install

```bash
pip install overdrive-db                # Python
npm install overdrive-db                # Node.js
cargo add overdrive-db                  # Rust (native crate)
go get github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/go@v2.3.0  # Go
```

**Java (Maven):**
```xml
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>2.3.0</version>
</dependency>
```

**C/C++:** Download `overdrive.h` + native library from [GitHub Releases](https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/releases/latest).

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
| **SQL Queries** | `SELECT`, `WHERE`, `ORDER BY`, `LIMIT` with field comparisons |
| **Full-text Search** | Built-in tokenised text search across documents |
| **B-Tree Indexes** | Secondary indexes for fast range lookups |
| **ACID Transactions** | MVCC with 4 isolation levels + deadlock detection |
| **Deferred Sync** | WAL-based crash safety; fsync batched every 1000 ops / 5 s |
| **Change History** | Per-document WAL changelog (`get_history`) |
| **Encryption** | AES-256-GCM via Argon2id key derivation |
| **6 Storage Engines** | Disk, RAM, Vector, Time-Series, Graph, Streaming |
| **Cross-platform** | Windows x64, Linux x64/ARM64, macOS x64/ARM64 |

---

## 6 Storage Engines

| Engine | Use Case | Latency | Open Flag |
|--------|----------|---------|-----------|
| `Disk` (default) | General-purpose persistent storage | ~1 ms | `engine="Disk"` |
| `RAM` | Caching, sessions, leaderboards | < 1 µs | `engine="RAM"` |
| `Vector` | Similarity search, embeddings | ~5 ms | `engine="Vector"` |
| `TimeSeries` | Metrics, IoT, logs | ~2 ms | `engine="TimeSeries"` |
| `Graph` | Social networks, knowledge graphs | ~3 ms | `engine="Graph"` |
| `Streaming` | Event queues, message brokers | ~1 ms | `engine="Streaming"` |

```python
# Open with a specific engine
odb = OverdriveDb.open("cache.odb", engine="RAM")
odb = OverdriveDb.open("events.odb", engine="TimeSeries")
odb = OverdriveDb.open("graph.odb", engine="Graph")
```

---

## API Reference

All SDKs share the same API surface. Method names follow each language's conventions.

### Database Lifecycle

| Python | Node.js | Java | Go | Rust | C |
|--------|---------|------|----|------|---|
| `OverdriveDb.open(path)` | `OverdriveDb.open(path)` | `OverDrive.open(path)` | `overdrive.Open(path)` | `OverdriveDb::open(path)` | `overdrive_open(path)` |
| `odb.close()` | `odb.close()` | `odb.close()` | `odb.Close()` | `odb.close()` | `overdrive_close(odb)` |
| `odb.sync()` | `odb.sync()` | `odb.sync()` | `odb.Sync()` | `odb.sync()` | `overdrive_sync(odb)` |
| `OverdriveDb.version()` | `OverdriveDb.version()` | `OverDrive.version()` | `overdrive.Version()` | `OverdriveDb::version()` | `overdrive_version()` |

### CRUD Operations

| Operation | Python | Node.js | Java | Go | Rust |
|-----------|--------|---------|------|----|------|
| Insert | `odb.insert(table, doc)` | `odb.insert(table, doc)` | `odb.insert(table, doc)` | `odb.Insert(table, doc)` | `odb.insert(table, &doc)` |
| Insert Many | `odb.insert_many(table, docs)` | `odb.insertMany(table, docs)` | `odb.insertMany(table, docs)` | `odb.InsertBatch(table, docs)` | `odb.insert_batch(table, &docs)` |
| Get | `odb.get(table, id)` | `odb.get(table, id)` | `odb.get(table, id)` | `odb.Get(table, id)` | `odb.get(table, id)` |
| Update | `odb.update(table, id, patch)` | `odb.update(table, id, patch)` | `odb.update(table, id, patch)` | `odb.Update(table, id, patch)` | `odb.update(table, id, &patch)` |
| Delete | `odb.delete(table, id)` | `odb.delete(table, id)` | `odb.delete(table, id)` | `odb.Delete(table, id)` | `odb.delete(table, id)` |
| Count | `odb.count(table)` | `odb.count(table)` | `odb.count(table)` | `odb.Count(table)` | `odb.count(table)` |
| Query (SQL) | `odb.query(sql)` | `odb.query(sql)` | `odb.query(sql)` | `odb.Query(sql)` | `odb.query(sql)` |
| Query Safe | `odb.query_safe(sql, params)` | `odb.querySafe(sql, params)` | `odb.querySafe(sql, params)` | `odb.QuerySafe(sql, params)` | `odb.query_safe(sql, &params)` |
| Search | `odb.search(table, text)` | `odb.search(table, text)` | `odb.search(table, text)` | `odb.Search(table, text)` | `odb.search(table, text)` |
| History | `odb.get_history(table, id)` | `odb.getHistory(table, id)` | `odb.getHistory(table, id)` | `odb.GetHistory(table, id)` | `odb.get_history(table, id)` |

### Tables

| Operation | Python | Node.js / Java | Go |
|-----------|--------|----------------|-----|
| Create | `odb.create_table(name)` | `odb.createTable(name)` | `odb.CreateTable(name)` |
| Drop | `odb.drop_table(name)` | `odb.dropTable(name)` | `odb.DropTable(name)` |
| List | `odb.list_tables()` | `odb.listTables()` | `odb.ListTables()` |
| Exists | `odb.table_exists(name)` | `odb.tableExists(name)` | `odb.TableExists(name)` |

### Advanced / Security

| Feature | Python | Node.js | C |
|---------|--------|---------|---|
| Password open | `OverdriveDb.open(path, password=...)` | `OverdriveDb.open(path, {password:...})` | `overdrive_open_with_engine(path, engine, pass)` |
| Engine select | `OverdriveDb.open(path, engine="RAM")` | `OverdriveDb.open(path, {engine:"RAM"})` | `overdrive_open_with_engine(path, engine, "")` |
| Backup | `odb.backup(dest)` | `odb.backup(dest)` | `overdrive_backup(odb, dest)` |
| Cleanup WAL | `odb.cleanup_wal()` | `odb.cleanupWal()` | `overdrive_cleanup_wal(odb)` |
| Verify integrity | `odb.verify_integrity()` | `odb.verifyIntegrity()` | `overdrive_verify_integrity(odb)` |
| Transaction callback | `odb.transaction(fn)` | `odb.transaction(fn)` | — |

### Transactions

All SDKs support MVCC transactions with 4 isolation levels:

| Level | Value | Description |
|-------|-------|-------------|
| Read Uncommitted | 0 | Fastest, least safe |
| Read Committed | 1 | Default |
| Repeatable Read | 2 | Snapshot isolation |
| Serializable | 3 | Full isolation |

```python
# Manual transaction
txn = odb.begin_transaction()
try:
    odb.insert("orders", {"item": "X", "qty": 5})
    odb.commit_transaction(txn)
except:
    odb.abort_transaction(txn)

# Callback style (auto commit/rollback)
odb.transaction(lambda: odb.insert("orders", {"item": "X"}))
```

---

## Engine-Specific APIs

### RAM Engine — Snapshot / Restore

```python
odb = OverdriveDb.open("cache.odb", engine="RAM")
odb.insert("sessions", {"user": "alice", "token": "abc"})
snap = odb.snapshot()      # capture in-memory state
# ... do more work ...
odb.restore(snap)          # revert to snapshot
```

| Python | C |
|--------|---|
| `odb.snapshot()` | `overdrive_snapshot(odb)` |
| `odb.restore(snap)` | `overdrive_restore(odb, snap)` |
| `odb.memory_usage()` | `overdrive_memory_usage(odb)` |
| `odb.set_memory_limit(bytes)` | `overdrive_set_memory_limit(odb, bytes)` |

### Vector Engine — Similarity Search

```python
odb = OverdriveDb.open("vectors.odb", engine="Vector")
odb.create_vector_index("embeddings", dimensions=384)
odb.insert_vector("embeddings", "doc1", [0.1, 0.2, ...])
results = odb.vector_search("embeddings", [0.1, 0.2, ...], top_k=10)
```

| Python | C |
|--------|---|
| `odb.create_vector_index(name, dimensions)` | `overdrive_create_vector_index(odb, name, dims)` |
| `odb.insert_vector(index, id, vector)` | `overdrive_insert_vector(odb, index, id, vec_json)` |
| `odb.vector_search(index, vector, top_k)` | `overdrive_vector_search(odb, index, vec_json, k)` |
| `odb.drop_vector_index(name)` | `overdrive_drop_vector_index(odb, name)` |
| `odb.list_vector_indexes()` | `overdrive_list_vector_indexes(odb)` |

### Time-Series Engine — Metrics & IoT

```python
odb = OverdriveDb.open("metrics.odb", engine="TimeSeries")
odb.create_timeseries("cpu", retention_days=30)
odb.insert_measurement("cpu", timestamp=1700000000, value=87.3,
                        tags={"host": "server1"})
rows = odb.query_timeseries("cpu", start=1699900000, end=1700000000)
stats = odb.aggregate_timeseries("cpu", func="avg", interval="1h")
```

| Python | C |
|--------|---|
| `odb.create_timeseries(name, retention_days)` | `overdrive_create_timeseries(odb, name, days)` |
| `odb.insert_measurement(name, timestamp, value, tags)` | `overdrive_insert_measurement(odb, name, ts, val, tags_json)` |
| `odb.query_timeseries(name, start, end)` | `overdrive_query_timeseries(odb, name, start, end)` |
| `odb.aggregate_timeseries(name, func, interval)` | `overdrive_aggregate_timeseries(odb, name, func, interval)` |
| `odb.drop_timeseries(name)` | `overdrive_drop_timeseries(odb, name)` |
| `odb.list_timeseries()` | `overdrive_list_timeseries(odb)` |

### Graph Engine — Nodes & Edges

```python
odb = OverdriveDb.open("social.odb", engine="Graph")
odb.create_node_type("Person")
odb.create_edge_type("FOLLOWS")
alice = odb.create_node("Person", {"name": "Alice"})
bob   = odb.create_node("Person", {"name": "Bob"})
odb.create_edge("FOLLOWS", alice, bob)
path = odb.shortest_path(alice, bob)
neighbors = odb.graph_traverse(alice, depth=2)
```

| Python | C |
|--------|---|
| `odb.create_node_type(name)` | `overdrive_create_node_type(odb, name, props_json)` |
| `odb.create_edge_type(name)` | `overdrive_create_edge_type(odb, name, props_json)` |
| `odb.create_node(type, props)` | `overdrive_create_node(odb, type, props_json)` |
| `odb.create_edge(type, from_id, to_id)` | `overdrive_create_edge(odb, type, from, to, props_json)` |
| `odb.graph_traverse(node_id, depth)` | `overdrive_graph_traverse(odb, node_id, depth)` |
| `odb.shortest_path(from_id, to_id)` | `overdrive_shortest_path(odb, from_id, to_id)` |
| `odb.list_nodes(type)` | `overdrive_list_nodes(odb, type)` |
| `odb.delete_node(node_id)` | `overdrive_delete_node(odb, node_id)` |

### Streaming Engine — Event Queues

```python
odb = OverdriveDb.open("events.odb", engine="Streaming")
odb.create_topic("orders", partitions=4)
odb.publish("orders", {"event": "placed", "order_id": "X1"})
sub = odb.subscribe("orders", group="workers", partition=0)
msgs = odb.poll(sub, max_messages=100)
odb.commit_offset(sub)
```

| Python | C |
|--------|---|
| `odb.create_topic(name, partitions)` | `overdrive_create_topic(odb, name, partitions)` |
| `odb.publish(topic, message)` | `overdrive_publish(odb, topic, msg_json)` |
| `odb.subscribe(topic, group, partition)` | `overdrive_subscribe(odb, topic, group, partition)` |
| `odb.poll(sub_id, max_messages)` | `overdrive_poll(odb, sub_id, max)` |
| `odb.commit_offset(sub_id)` | `overdrive_commit_offset(odb, sub_id, offset)` |
| `odb.unsubscribe(sub_id)` | `overdrive_unsubscribe(odb, sub_id)` |
| `odb.drop_topic(name)` | `overdrive_drop_topic(odb, name)` |
| `odb.list_topics()` | `overdrive_list_topics(odb)` |

---

## Change History (MVCC)

Every insert and update is recorded in the WAL with a before/after snapshot:

```python
id1 = odb.insert("users", {"name": "Alice", "age": 25})
odb.update("users", id1, {"age": 26})
odb.update("users", id1, {"age": 27, "role": "admin"})

history = odb.get_history("users", id1)
# [
#   {"lsn": 3,  "op": "INSERT", "data": {"age": 25, ...}, "prev_data": None},
#   {"lsn": 12, "op": "UPDATE", "data": {"age": 26, ...}, "prev_data": {"age": 25, ...}},
#   {"lsn": 16, "op": "UPDATE", "data": {"age": 27, "role": "admin"}, "prev_data": {"age": 26, ...}}
# ]
```

> **Note:** The WAL is checkpointed on every `sync()` / `close()`. History is live-session only — for permanent audit logs, back up the `.wal` file before closing.

---

## Security

```python
# Encrypted database (AES-256-GCM, key from Argon2id)
odb = OverdriveDb.open("secret.odb", password="my-passphrase")

# Parameterised query — prevents SQL injection
results = odb.query_safe(
    "SELECT * FROM orders WHERE amount > ?1 AND status = ?2",
    [100, "paid"]
)

# Backup (flush + copy, chmod 600 on Linux/Mac)
odb.backup("backups/myapp-2024-01-01.odb")

# WAL cleanup (removes replay-attack surface after shutdown)
odb.sync()
odb.cleanup_wal()
```

### Error Codes

| Code | Type | When |
|------|------|------|
| `ODB-AUTH-*` | Authentication | Wrong password, key too short |
| `ODB-TABLE-*` | Table | Not found, already exists |
| `ODB-QUERY-*` | Query | SQL syntax error, unbound placeholder |
| `ODB-TXN-*` | Transaction | Deadlock, conflict |
| `ODB-IO-*` | I/O | File not found, corrupted, backup failed |
| `ODB-FFI-*` | FFI | Native library not found |

### C/C++ Memory Rules

Every `char*` returned by `overdrive_*` functions **must** be freed with `overdrive_free_string()`.  
Do **not** free: `overdrive_last_error()`, `overdrive_version()` (static pointers).

---

## Native Library Downloads

| Platform | File | Size |
|----------|------|------|
| Windows x64 | `overdrive.dll` | ~3.4 MB |
| Linux x64 | `liboverdrive.so` | ~4.1 MB |
| Linux ARM64 | `liboverdrive-arm64.so` | ~3.9 MB |
| macOS x64 | `liboverdrive.dylib` | ~3.8 MB |
| macOS ARM64 | `liboverdrive-arm64.dylib` | ~3.6 MB |

Download from [GitHub Releases](https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/releases/latest).

> Python and Rust auto-download the native library on first use. Node.js downloads on `npm install`. Java and Go require manual placement in `lib/`.

---

## Links

| Resource | URL |
|----------|-----|
| GitHub | https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK |
| Releases | https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/releases |
| crates.io | https://crates.io/crates/overdrive-db |
| PyPI | https://pypi.org/project/overdrive-db/ |
| npm | https://www.npmjs.com/package/overdrive-db |
| Issues | https://github.com/karthikeyanV2K/OverDrive-DB_IncodeSDK/issues |

## License

Licensed under either **MIT** or **Apache-2.0**, at your option.

---

<p align="center">
  Built by <a href="https://github.com/karthikeyanV2K"><strong>karthikeyanV2K</strong></a> · <a href="https://overdrive-db.com">overdrive-db.com</a>
</p>
