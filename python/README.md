<p align="center">
  <h1 align="center">⚡ overdrive-db</h1>
  <p align="center">
    <strong>Embeddable hybrid SQL+NoSQL document database for Python</strong><br/>
    Like SQLite, but for JSON. Zero config. No server. ACID transactions.
  </p>
  <p align="center">
    <a href="https://pypi.org/project/overdrive-db/"><img src="https://img.shields.io/pypi/v/overdrive-db?style=flat-square&color=3776ab&logo=python" alt="PyPI"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="license"/></a>
  </p>
</p>

---

## Install

```bash
pip install overdrive-db
```

> **Requires** the native `overdrive.dll` (Windows) / `liboverdrive.so` (Linux) / `liboverdrive.dylib` (macOS) in the `lib/{os}-{arch}/` directory.
> Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

---

## Quick Start

```python
from overdrive import OverdriveDb

odb = OverdriveDb.open("myapp.odb")

# Create table
odb.create_table("users")

# Insert — returns the generated _id
id = odb.insert("users", {"name": "Alice", "age": 30})

# Get by ID
doc = odb.get("users", id)
print(doc)  # {'_id': '...', 'name': 'Alice', 'age': 30}

# SQL Query
rows = odb.query("SELECT * FROM users WHERE age > 25")

# Update
odb.update("users", id, {"age": 31})

# Delete
odb.delete("users", id)

odb.close()
```

**Context manager:**
```python
with OverdriveDb.open("myapp.odb") as odb:
    odb.create_table("users")
    id = odb.insert("users", {"name": "Bob"})
    print(odb.get("users", id))
```

---

## API Reference

### Open / Close

```python
odb = OverdriveDb.open(path)                         # plain open
odb = OverdriveDb.open(path, password="secret")      # encrypted
odb = OverdriveDb.open(path, engine="RAM")           # in-memory

odb.sync()   # flush to disk
odb.close()  # release handle

OverdriveDb.version()  # native library version string
```

### Tables

```python
odb.create_table("users")
odb.drop_table("users")
odb.list_tables()          # → ['users', 'products', ...]
odb.table_exists("users")  # → True / False
```

### CRUD

| Method | Returns | Description |
|--------|---------|-------------|
| `odb.insert(table, doc)` | `str` (\_id) | Insert a document |
| `odb.insert_many(table, docs)` | `list[str]` | Insert multiple documents |
| `odb.get(table, id)` | `dict \| None` | Get document by \_id |
| `odb.update(table, id, patch)` | `bool` | Update document by \_id |
| `odb.delete(table, id)` | `bool` | Delete document by \_id |
| `odb.count(table)` | `int` | Count documents in table |

### Query

```python
# SQL query — returns list of dicts
rows = odb.query("SELECT * FROM users ORDER BY age DESC LIMIT 10")

# Full-text search
results = odb.search("users", "Alice")
```

### Transactions

```python
from overdrive import IsolationLevel

# Callback style (auto-commit / auto-abort)
def transfer(odb):
    odb.insert("logs", {"event": "login"})

odb.transaction(transfer, IsolationLevel.SERIALIZABLE)

# Manual style
txn = odb.begin_transaction(IsolationLevel.READ_COMMITTED)
try:
    odb.insert("logs", {"event": "login"})
    odb.commit_transaction(txn)
except Exception:
    odb.abort_transaction(txn)
    raise
```

**Isolation Levels:**

| Name | Value |
|------|-------|
| `IsolationLevel.READ_UNCOMMITTED` | 0 |
| `IsolationLevel.READ_COMMITTED` | 1 (default) |
| `IsolationLevel.REPEATABLE_READ` | 2 |
| `IsolationLevel.SERIALIZABLE` | 3 |

### Integrity Check

```python
report = odb.verify_integrity()
print(report)  # {'status': 'ok', ...}
```

---

## 6 Storage Engines

```python
odb = OverdriveDb.open("app.odb")                            # Disk (default)
odb = OverdriveDb.open("cache.odb", engine="RAM")            # in-memory
odb = OverdriveDb.open("vecs.odb", engine="Vector")          # embeddings
odb = OverdriveDb.open("ts.odb", engine="Time-Series")       # metrics/IoT
odb = OverdriveDb.open("g.odb", engine="Graph")              # social graphs
odb = OverdriveDb.open("q.odb", engine="Streaming")          # event queue
```

---

## More SDKs

```bash
pip install overdrive-db          # Python  ← you are here
npm install overdrive-db          # Node.js
cargo add overdrive-db            # Rust
go get github.com/ALL-FOR-ONE-TECH/overdrive-db-go@v1.0.0
```

---

## Links

- [GitHub Repository](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)
- [Releases / Native Libraries](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest)
- [License: MIT](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE)
