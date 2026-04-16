# Python SDK — Complete Guide

**Version:** 1.4.0 | **Requires:** Python 3.8+

---

## Installation

```bash
pip install overdrive-db
```

---

## Import

```python
from overdrive import (
    OverDrive,
    WatchdogReport,
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
    ThreadSafeOverDrive,
)
```

---

## Opening a Database

```python
# Basic open (auto-creates if not exists)
db = OverDrive.open("myapp.odb")

# With password encryption
db = OverDrive.open("secure.odb", password="my-secret-pass-123")

# With RAM engine
db = OverDrive.open("cache.odb", engine="RAM")

# All options
db = OverDrive.open(
    "app.odb",
    password="my-secret-pass",     # optional, min 8 chars
    engine="Disk",                  # "Disk"|"RAM"|"Vector"|"Time-Series"|"Graph"|"Streaming"
    auto_create_tables=True         # default True
)

# Legacy constructor (v1.3 — still works)
db = OverDrive("myapp.odb")

# Environment variable encryption (v1.3 — still works)
# export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
db = OverDrive.open_encrypted("app.odb", "ODB_KEY")

# Context manager — auto-closes
with OverDrive.open("app.odb") as db:
    db.insert("users", {"name": "Alice"})
```

---

## Table Operations

```python
# Create table (optional — auto-created on first insert)
db.create_table("users")
db.create_table("sessions", engine="RAM")   # RAM table

# Drop table
db.drop_table("old_table")

# List tables
tables = db.list_tables()  # ["users", "products", ...]

# Check existence
exists = db.table_exists("users")  # True or False
```

---

## CRUD Operations

### Insert

```python
# Single document — returns auto-generated _id
id = db.insert("users", {
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com",
    "tags": ["admin", "dev"],
    "active": True
})
print(id)  # "users_1"

# Multiple documents
ids = db.insert_many("users", [
    {"name": "Bob",   "age": 25},
    {"name": "Carol", "age": 35},
])
print(ids)  # ["users_2", "users_3"]
```

### Get

```python
# Get by _id
user = db.get("users", "users_1")
# → {"_id": "users_1", "name": "Alice", "age": 30, ...}
# → None if not found

# Count all documents
count = db.count("users")  # int
```

### Update

```python
# Update by _id — only specified fields change
updated = db.update("users", "users_1", {"age": 31, "status": "active"})
# → True if found and updated, False if not found
```

### Delete

```python
# Delete by _id
deleted = db.delete("users", "users_1")
# → True if found and deleted, False if not found
```

---

## SQL Queries

```python
# Execute SQL — returns list of dicts
results = db.query("SELECT * FROM users")
results = db.query("SELECT * FROM users WHERE age > 25")
results = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name DESC LIMIT 10")
results = db.query("SELECT COUNT(*) FROM users")
results = db.query("SELECT AVG(age), MIN(age), MAX(age) FROM users")

# Full result with metadata
result = db.query_full("SELECT * FROM users")
# → {"rows": [...], "columns": [...], "rows_affected": 0, "execution_time_ms": 1.2}

# Safe parameterized query (use for user input!)
results = db.querySafe("SELECT * FROM users WHERE name = ?", user_input)
results = db.querySafe("SELECT * FROM users WHERE age > ? AND city = ?", "25", "London")

# Full-text search
matches = db.search("users", "alice")  # list of matching documents
```

---

## Helper Methods (v1.4)

```python
# findOne — first match or None
user = db.findOne("users", "age > 25")
first = db.findOne("users")  # first document, no filter

# findAll — all matches
users = db.findAll("users")
users = db.findAll("users", where="age > 25")
users = db.findAll("users", where="age > 25", order_by="name ASC")
users = db.findAll("users", where="age > 25", order_by="name ASC", limit=10)

# updateMany — bulk update, returns count of updated docs
count = db.updateMany("users", "status = 'trial'", {"status": "active"})

# deleteMany — bulk delete, returns count of deleted docs
count = db.deleteMany("logs", "created_at < '2025-01-01'")

# countWhere — count matching docs
n = db.countWhere("users", "age > 25")
total = db.countWhere("users")  # count all

# exists — check if document exists by _id
found = db.exists("users", "users_1")  # True or False
```

---

## Transactions

```python
# Callback pattern (recommended — v1.4)
result = db.transaction(lambda txn: db.insert("orders", {"item": "widget"}))

def complex_operation(txn):
    db.updateMany("accounts", "id = 'alice'", {"balance": 900})
    db.updateMany("accounts", "id = 'bob'",   {"balance": 600})
    return "done"

result = db.transaction(complex_operation)
result = db.transaction(complex_operation, isolation=OverDrive.SERIALIZABLE)

# With retry on conflict
result = db.transaction_with_retry(complex_operation, max_retries=3)
result = db.transaction_with_retry(
    complex_operation,
    isolation=OverDrive.SERIALIZABLE,
    max_retries=5
)

# Context manager (v1.3 — still works)
with db.transaction() as txn_id:
    db.insert("users", {"name": "Alice"})
    # auto-commits on exit, auto-aborts on exception

# Manual (v1.3 — still works)
txn_id = db.begin_transaction()
txn_id = db.begin_transaction(OverDrive.SERIALIZABLE)
db.commit_transaction(txn_id)
db.abort_transaction(txn_id)

# camelCase aliases (for cross-SDK consistency)
txn_id = db.beginTransaction()
db.commitTransaction(txn_id)
db.abortTransaction(txn_id)

# Isolation level constants
OverDrive.READ_UNCOMMITTED  # 0
OverDrive.READ_COMMITTED    # 1 (default)
OverDrive.REPEATABLE_READ   # 2
OverDrive.SERIALIZABLE      # 3
```

---

## RAM Engine Methods (v1.4)

```python
# Snapshot — persist RAM database to disk
db.snapshot("./backup/cache.odb")

# Restore — load snapshot into RAM database
db.restore("./backup/cache.odb")

# Memory usage
usage = db.memoryUsage()  # or db.memory_usage()
# → {"bytes": 1048576, "mb": 1.0, "limit_bytes": 2147483648, "percent": 0.05}
print(f"Using {usage['mb']:.1f} MB ({usage['percent']:.1f}%)")
```

---

## Watchdog (v1.4)

```python
# Static method — no open database needed
report = OverDrive.watchdog("app.odb")

# WatchdogReport fields
report.file_path          # str — path inspected
report.file_size_bytes    # int — file size
report.last_modified      # int — Unix timestamp
report.integrity_status   # str — "valid", "corrupted", "missing"
report.corruption_details # Optional[str] — details if corrupted
report.page_count         # int — number of pages
report.magic_valid        # bool — magic number valid

# Usage pattern
if report.integrity_status == "valid":
    db = OverDrive.open("app.odb")
elif report.integrity_status == "corrupted":
    print(f"Corrupted: {report.corruption_details}")
    # restore from backup
else:
    print("File not found — creating new database")
    db = OverDrive.open("app.odb")
```

---

## Error Handling

```python
from overdrive import (
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
)

try:
    db = OverDrive.open("secure.odb", password="wrong")
except AuthenticationError as e:
    print(e.code)         # "ODB-AUTH-001"
    print(e.message)      # "Incorrect password..."
    print(e.context)      # "secure.odb"
    print(e.suggestions)  # ["Verify you're using the correct password", ...]
    print(e.doc_link)     # "https://overdrive-db.com/docs/errors/ODB-AUTH-001"

try:
    db.query("INVALID SQL !!!")
except QueryError as e:
    print(f"Query error: {e.code} — {e.message}")

try:
    db.insert("users", {"name": "Alice"})
except TableError as e:
    print(f"Table error: {e}")

# Catch all OverDrive errors
try:
    db.insert("users", {"name": "Alice"})
except OverDriveError as e:
    print(f"Database error [{e.code}]: {e.message}")
```

---

## Security

```python
import os

# Password from environment variable
db = OverDrive.open("secure.odb", password=os.environ["DB_PASSWORD"])

# Backup
db.backup("backups/app_2026-04-16.odb")

# WAL cleanup after commit
txn_id = db.begin_transaction()
db.insert("users", {"name": "Alice"})
db.commit_transaction(txn_id)
db.cleanup_wal()

# Safe queries
results = db.querySafe("SELECT * FROM users WHERE name = ?", user_input)
```

---

## Thread-Safe Access

```python
from overdrive import ThreadSafeOverDrive
import threading

# Thread-safe wrapper
db = ThreadSafeOverDrive("app.odb")

def worker(i):
    db.insert("logs", {"thread": i, "event": "started"})

threads = [threading.Thread(target=worker, args=(i,)) for i in range(10)]
for t in threads: t.start()
for t in threads: t.join()

db.close()
```

---

## Complete Example

```python
from overdrive import OverDrive
import os

# Open database
db = OverDrive.open("store.odb")

# Insert products (table auto-created)
products = [
    {"name": "Laptop",  "price": 999,  "category": "electronics", "stock": 5},
    {"name": "Mouse",   "price": 29,   "category": "electronics", "stock": 50},
    {"name": "Desk",    "price": 299,  "category": "furniture",   "stock": 10},
    {"name": "Chair",   "price": 199,  "category": "furniture",   "stock": 15},
]
ids = db.insert_many("products", products)
print(f"Inserted {len(ids)} products")

# Query
electronics = db.findAll("products", "category = 'electronics'", order_by="price DESC")
print(f"Electronics: {[p['name'] for p in electronics]}")

# Count
total = db.countWhere("products")
expensive = db.countWhere("products", "price > 100")
print(f"Total: {total}, Expensive: {expensive}")

# Update with transaction
def apply_sale(txn):
    count = db.updateMany("products", "price > 100", {"on_sale": True, "discount": 10})
    db.insert("audit_log", {"event": "sale_applied", "affected": count})
    return count

updated = db.transaction(apply_sale)
print(f"Applied sale to {updated} products")

# Watchdog check
report = OverDrive.watchdog("store.odb")
print(f"Database health: {report.integrity_status} ({report.file_size_bytes} bytes)")

# Backup
db.backup("backups/store_backup.odb")
print("Backup created!")

db.close()
```
