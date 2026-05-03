# Core Concepts

Understanding these concepts will make everything else click.

---

## 1. The Database File

Your entire database lives in **one `.odb` file**.

```
myapp.odb        ← your database
myapp.odb.wal    ← write-ahead log (auto-managed, auto-deleted)
```

- Copy the file → you have a backup
- Delete the file → database is gone
- Move the file → database moves with it
- The `.wal` file is temporary — it's created during writes and cleaned up automatically

---

## 2. Tables and Documents

OverDrive is **schemaless**. Tables don't have fixed columns. Each document (row) is a JSON object and can have any fields.

```python
# These can all go in the same "users" table — different shapes are fine
db.insert("users", {"name": "Alice", "age": 30})
db.insert("users", {"name": "Bob", "email": "bob@example.com", "premium": True})
db.insert("users", {"name": "Carol", "age": 25, "tags": ["admin", "dev"], "score": 9.5})
```

### Auto-generated `_id`

Every document gets an `_id` automatically:

```
Format: {table_name}_{counter}
Examples: users_1, users_2, orders_1, products_42
```

You can use this `_id` to get, update, or delete a specific document.

---

## 3. Auto-Table Creation

You **never need to call `createTable()`** unless you want to. Tables are created automatically on the first insert.

```python
# This works — "users" table is created automatically
db.insert("users", {"name": "Alice"})

# This also works — "orders" table is created automatically
db.insert("orders", {"item": "Laptop", "qty": 1})
```

To disable this behavior:
```python
db = OverDrive.open("app.odb", auto_create_tables=False)
# Now you must call db.createTable("users") before inserting
```

---

## 4. SQL on JSON

OverDrive has a built-in SQL engine. You write SQL, it runs on your JSON documents.

```python
# SQL works on any JSON field — no schema needed
db.insert("products", {"name": "Laptop", "price": 999, "category": "electronics"})
db.insert("products", {"name": "Desk",   "price": 299, "category": "furniture"})

# Query with SQL
results = db.query("SELECT * FROM products WHERE price > 500 ORDER BY price DESC")
# → [{'_id': 'products_1', 'name': 'Laptop', 'price': 999, ...}]
```

The SQL engine supports: `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `WHERE`, `ORDER BY`, `LIMIT`, `OFFSET`, `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `GROUP BY`.

---

## 5. The 6 Storage Engines

OverDrive has 6 different storage engines. You pick the right one for your use case.

| Engine | Best For | Latency |
|--------|----------|---------|
| `Disk` | General persistent storage | ~1ms |
| `RAM` | Caching, sessions, hot data | <1µs |
| `Vector` | Similarity search, AI embeddings | ~5ms |
| `Time-Series` | Metrics, IoT, logs | ~2ms |
| `Graph` | Social networks, relationships | ~3ms |
| `Streaming` | Event queues, message brokers | ~1ms |

```python
# Choose engine when opening
db = OverDrive.open("cache.odb", engine="RAM")

# Or per-table within a disk database
db = OverDrive.open("app.odb")
db.createTable("sessions", engine="RAM")   # this table in RAM
db.insert("users", {"name": "Alice"})      # this table on disk
```

---

## 6. MVCC Transactions

OverDrive uses **Multi-Version Concurrency Control (MVCC)** — the same technique used by PostgreSQL.

What this means for you:
- Multiple readers never block each other
- Writers don't block readers
- Each transaction sees a consistent snapshot of the data
- Conflicts are detected and reported

### 4 Isolation Levels

| Level | What It Means |
|-------|---------------|
| `READ_UNCOMMITTED` | Can read data from uncommitted transactions (fastest, least safe) |
| `READ_COMMITTED` | Only reads committed data (default — good for most apps) |
| `REPEATABLE_READ` | Same data every time you read within a transaction |
| `SERIALIZABLE` | Full isolation — transactions run as if sequential (safest, slowest) |

---

## 7. The Write-Ahead Log (WAL)

Every write goes to the WAL first, then to the main database file. This ensures:

- **Crash safety** — if your app crashes mid-write, the WAL is replayed on next open
- **Durability** — data is never lost once written to the WAL

The WAL is managed automatically. You don't need to think about it unless you're doing security-sensitive work (see [Security Guide](06-security.md)).

---

## 8. How the SDK Works

The SDK is a **thin wrapper** that loads a prebuilt native library at runtime:

```
Your Python/Node.js/Java/Go code
        ↓
OverDrive SDK (Python/JS/Java/Go wrapper)
        ↓
overdrive.dll / liboverdrive.so / liboverdrive.dylib   ← prebuilt native library
        ↓
app.odb file
```

The native library contains the actual database engine (B-Tree, MVCC, WAL, etc.). The SDK wrappers just call into it via FFI (Foreign Function Interface).

This means:
- The SDK is lightweight — just a wrapper
- The engine is the same across all languages
- Behavior is identical whether you use Python, Node.js, Java, or Go

---

## 9. File Permissions

When you open a database, OverDrive automatically hardens the file permissions:

- **Windows:** `icacls` — only the current user gets access
- **Linux/macOS:** `chmod 600` — owner read/write only

This prevents other users on the same machine from reading your database file.

---

## 10. Error Codes

Every error has a structured code in the format `ODB-{CATEGORY}-{NUMBER}`:

| Category | Prefix | Examples |
|----------|--------|---------|
| Authentication | `ODB-AUTH-` | Wrong password, password too short |
| Table | `ODB-TABLE-` | Table not found, already exists |
| Query | `ODB-QUERY-` | SQL syntax error |
| Transaction | `ODB-TXN-` | Deadlock, conflict |
| I/O | `ODB-IO-` | File not found, permission denied |
| FFI | `ODB-FFI-` | Native library not found |

```python
from overdrive import OverDrive, AuthenticationError

try:
    db = OverDrive.open("secure.odb", password="wrong")
except AuthenticationError as e:
    print(e.code)         # ODB-AUTH-001
    print(e.message)      # "Incorrect password for database 'secure.odb'"
    print(e.suggestions)  # ["Verify you're using the correct password", ...]
```
