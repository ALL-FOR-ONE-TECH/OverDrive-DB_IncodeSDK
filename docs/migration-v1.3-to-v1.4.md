# Migration Guide: v1.3 → v1.4

## What's New in v1.4

- **`open()` with password & engine** — Single-line database creation with encryption and engine selection
- **Auto-table creation** — Tables created automatically on first insert (all 6 engines)
- **RAM engine API** — `snapshot()`, `restore()`, `memoryUsage()` for in-memory databases
- **Watchdog function** — File integrity monitoring without opening the database
- **Transaction callback** — `transaction(callback)` with auto-commit/rollback
- **Helper methods** — `findOne()`, `findAll()`, `updateMany()`, `deleteMany()`, `countWhere()`, `exists()`
- **Enhanced errors** — Structured error codes, suggestions, and documentation links

## Backward Compatibility

> [!IMPORTANT]
> **v1.4 is 100% backward compatible.** All v1.3 code works without any changes.

No breaking changes. No deprecations. All existing methods remain unchanged.

---

## Migration Examples

### 1. Constructor → `open()`

**Before (v1.3):**
```python
db = OverDrive("app.odb")
```

**After (v1.4):**
```python
db = OverDrive.open("app.odb")
```

Both still work. `open()` adds engine selection and auto-table creation.

---

### 2. `openEncrypted()` → `open()` with password

**Before (v1.3):**
```python
import os
os.environ["ODB_KEY"] = "my-secret-key-32chars!!"
db = OverDrive.openEncrypted("app.odb", "ODB_KEY")
```

**After (v1.4):**
```python
db = OverDrive.open("app.odb", password="my-secret-key-32chars!!")
```

> [!NOTE]
> `openEncrypted()` still works for environment-variable-based encryption. The new `password` parameter uses Argon2id key derivation for stronger security.

---

### 3. Manual transactions → Callback pattern

**Before (v1.3):**
```python
txn_id = db.begin_transaction()
try:
    db.insert("users", {"name": "Alice"})
    db.insert("logs", {"event": "user_created"})
    db.commit_transaction(txn_id)
except:
    db.abort_transaction(txn_id)
    raise
```

**After (v1.4):**
```python
db.transaction(lambda txn:
    db.insert("users", {"name": "Alice"}) and
    db.insert("logs", {"event": "user_created"})
)
```

Manual `beginTransaction()`, `commitTransaction()`, `abortTransaction()` still work.

---

### 4. Manual table creation → Auto-creation

**Before (v1.3):**
```python
db = OverDrive("app.odb")
db.createTable("users")
db.insert("users", {"name": "Alice"})
```

**After (v1.4):**
```python
db = OverDrive.open("app.odb")
db.insert("users", {"name": "Alice"})  # Table auto-created!
```

Explicit `createTable()` still works and is recommended for production.

---

### 5. New: RAM Engine

```python
# Full RAM database
db = OverDrive.open("cache.odb", engine="RAM")
db.insert("sessions", {"user_id": 123, "token": "abc"})
db.snapshot("backup/cache.odb")       # Persist to disk
db.restore("backup/cache.odb")        # Restore later
usage = db.memoryUsage()              # Check RAM usage
```

---

### 6. New: Watchdog Monitoring

```python
report = OverDrive.watchdog("app.odb")
if report.integrity_status == "corrupted":
    print(f"Database corrupted: {report.corruption_details}")
```

---

### 7. New: Helper Methods

```python
user = db.findOne("users", "name = 'Alice'")
all_users = db.findAll("users", "age > 18", order_by="name", limit=100)
count = db.updateMany("users", "status = 'trial'", {"status": "active"})
deleted = db.deleteMany("logs", "created_at < '2025-01-01'")
active_count = db.countWhere("users", "status = 'active'")
has_user = db.exists("users", "users_1")
```

---

## Node.js Equivalents

| v1.3                                          | v1.4                                                    |
|-----------------------------------------------|---------------------------------------------------------|
| `new OverDrive('app.odb')`                    | `OverDrive.open('app.odb')`                             |
| `OverDrive.openEncrypted('app.odb', 'KEY')`   | `OverDrive.open('app.odb', { password: '...' })`        |
| `db.beginTransaction()` / `commitTransaction()`| `db.transaction(callback)`                              |
| `db.createTable('users')`                      | Auto-created on `db.insert('users', ...)`               |

## Java Equivalents

| v1.3                                          | v1.4                                                    |
|-----------------------------------------------|---------------------------------------------------------|
| `OverDrive.open("app.odb")`                   | `OverDrive.open("app.odb", new OpenOptions())`          |
| `OverDrive.openEncrypted("app.odb", "KEY")`   | `OverDrive.open("app.odb", new OpenOptions().password("..."))` |
| Manual try/catch transaction                   | `db.transaction(txn -> { ... })`                        |

## Go Equivalents

| v1.3                                          | v1.4                                                    |
|-----------------------------------------------|---------------------------------------------------------|
| `overdrive.Open("app.odb")`                   | `overdrive.Open("app.odb", overdrive.WithEngine("RAM"))`|
| `overdrive.OpenEncrypted("app.odb", "KEY")`   | `overdrive.Open("app.odb", overdrive.WithPassword("..."))` |
| Manual BeginTransaction/CommitTransaction      | `db.Transaction(func(txn) error { ... }, isolation)`    |
