# Security Guide

OverDrive-DB has built-in security features. This guide explains each one and when to use it.

---

## 1. Password-Protected Databases

Encrypt your database with a password. The key is derived using Argon2id (memory-hard, resistant to brute force).

### How It Works

```
Your password → Argon2id (64MB memory, 3 iterations) → 256-bit AES key → AES-256-GCM encryption
```

The salt is stored in the database file header. You only need the password to re-open.

### Python

```python
from overdrive import OverDrive
import os

# ✅ Best practice — load password from environment variable
password = os.environ.get("DB_PASSWORD")
if not password:
    raise ValueError("DB_PASSWORD not set")

# Create encrypted database
db = OverDrive.open("secure.odb", password=password)
db.insert("secrets", {"key": "api_token", "value": "sk-abc123"})
db.close()

# Re-open with same password
db = OverDrive.open("secure.odb", password=password)
secrets = db.findAll("secrets")
db.close()
```

### Node.js

```javascript
const { OverDrive } = require('overdrive-db');

const password = process.env.DB_PASSWORD;
if (!password) throw new Error('DB_PASSWORD not set');

const db = OverDrive.open('secure.odb', { password });
db.insert('secrets', { key: 'api_token', value: 'sk-abc123' });
db.close();
```

### Java

```java
String password = System.getenv("DB_PASSWORD");
if (password == null) throw new RuntimeException("DB_PASSWORD not set");

try (OverDrive db = OverDrive.open("secure.odb",
        new OpenOptions().password(password))) {
    db.insert("secrets", Map.of("key", "api_token", "value", "sk-abc123"));
}
```

### Go

```go
password := os.Getenv("DB_PASSWORD")
if password == "" {
    panic("DB_PASSWORD not set")
}

db, err := overdrive.Open("secure.odb", overdrive.WithPassword(password))
if err != nil { panic(err) }
defer db.Close()
```

### Password Requirements

- Minimum 8 characters (enforced by SDK)
- Recommended: 12+ characters for production
- All UTF-8 characters supported
- Never hardcode passwords in source code

### Wrong Password

```python
from overdrive import OverDrive, AuthenticationError

try:
    db = OverDrive.open("secure.odb", password="wrong-password")
except AuthenticationError as e:
    print(e.code)     # ODB-AUTH-001
    print(e.message)  # "Incorrect password for database 'secure.odb'"
```

---

## 2. Environment Variable Encryption (v1.3 — Still Supported)

Load the encryption key from an environment variable. The key is zeroed from memory after use.

```python
# Set in shell: export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
db = OverDrive.open_encrypted("app.odb", "ODB_KEY")
```

```javascript
// Set: process.env.ODB_KEY = "my-aes-256-key-32-chars-minimum!!!!"
const db = OverDrive.openEncrypted('app.odb', 'ODB_KEY');
```

```java
// Set: export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
OverDrive db = OverDrive.openEncrypted("app.odb", "ODB_KEY");
```

```go
// Set: export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
db, err := overdrive.OpenEncrypted("app.odb", "ODB_KEY")
```

> **Difference from password:** `openEncrypted()` uses the raw key bytes directly. `open(password=...)` derives the key via Argon2id, which is stronger against brute force.

---

## 3. SQL Injection Prevention

**Never** build SQL by concatenating user input. Use `querySafe()` with `?` placeholders.

```python
# ❌ DANGEROUS — SQL injection vulnerable
user_name = request.get("name")  # could be: "'; DROP TABLE users; --"
db.query(f"SELECT * FROM users WHERE name = '{user_name}'")

# ✅ SAFE — parameterized query
results = db.querySafe("SELECT * FROM users WHERE name = ?", user_name)

# Multiple parameters
results = db.querySafe(
    "SELECT * FROM users WHERE age > ? AND city = ?",
    "25", "London"
)
```

```javascript
// Node.js
const results = db.querySafe(
    "SELECT * FROM users WHERE name = ?",
    [userName]
);
```

```java
// Java
List<Map<String, Object>> results = db.querySafe(
    "SELECT * FROM users WHERE name = ?",
    userName
);
```

```go
// Go
results, err := db.QuerySafe(
    "SELECT * FROM users WHERE name = ?",
    userName
)
```

### What `querySafe()` Blocks

- Comment tokens: `--`, `/*`, `*/`
- Dangerous keywords: `DROP`, `TRUNCATE`, `ALTER`, `EXEC`, `EXECUTE`, `UNION`, `XP_`
- Automatic single-quote escaping

---

## 4. Encrypted Backups

Create encrypted backups of your database. The backup inherits the encryption of the source.

```python
db = OverDrive.open("app.odb", password=os.environ["DB_PASSWORD"])

# Backup — syncs to disk first, then copies .odb + .wal
db.backup("backups/app_2026-04-16.odb")

# Verify backup integrity before trusting it
report = OverDrive.watchdog("backups/app_2026-04-16.odb")
if report.integrity_status == "valid":
    print("Backup verified!")
else:
    print(f"Backup corrupted: {report.corruption_details}")
```

```javascript
db.backup('backups/app_2026-04-16.odb');
```

```java
db.backup("backups/app_2026-04-16.odb");
```

```go
db.Backup("backups/app_2026-04-16.odb")
```

**Best practices:**
- Store backups on a separate drive or cloud storage
- Verify backups with `watchdog()` before relying on them
- Rotate backups (keep last 7 days, last 4 weeks, etc.)

---

## 5. WAL Cleanup

The Write-Ahead Log (WAL) file contains recent writes. After a confirmed commit, delete it to prevent stale replay attacks.

```python
txn_id = db.begin_transaction()
db.insert("users", {"name": "Alice"})
db.commit_transaction(txn_id)
db.cleanup_wal()  # Delete the WAL file
```

```javascript
const txnId = db.beginTransaction();
db.insert('users', { name: 'Alice' });
db.commitTransaction(txnId);
db.cleanupWal();
```

```java
long txnId = db.beginTransaction();
db.insert("users", Map.of("name", "Alice"));
db.commitTransaction(txnId);
db.cleanupWal();
```

```go
txn, _ := db.BeginTransaction(overdrive.ReadCommitted)
db.Insert("users", map[string]any{"name": "Alice"})
db.CommitTransaction(txn)
db.CleanupWAL()
```

---

## 6. File Permission Hardening

OverDrive automatically sets restrictive permissions on every `.odb` file when opened:

| Platform | Permission | Effect |
|----------|-----------|--------|
| Windows | `icacls /grant:r` | Only current user has access |
| Linux/macOS | `chmod 600` | Owner read/write only |

This happens automatically — you don't need to do anything.

To manually harden:
```bash
# Linux/macOS
chmod 600 app.odb app.odb.wal

# Windows PowerShell
icacls app.odb /inheritance:r /grant:r "$env:USERNAME:F"
```

---

## 7. Watchdog — File Integrity Monitoring

Check a database file for corruption before opening it. Runs in < 100ms for files under 1GB.

```python
from overdrive import OverDrive

# Check before opening
report = OverDrive.watchdog("app.odb")

print(f"Status: {report.integrity_status}")   # "valid", "corrupted", or "missing"
print(f"Size: {report.file_size_bytes} bytes")
print(f"Pages: {report.page_count}")
print(f"Magic valid: {report.magic_valid}")

if report.integrity_status == "corrupted":
    print(f"Corruption details: {report.corruption_details}")
    # Restore from backup instead of opening
elif report.integrity_status == "missing":
    print("File not found — creating new database")
else:
    db = OverDrive.open("app.odb")
```

```javascript
const report = OverDrive.watchdog('app.odb');
console.log(report.integrityStatus);  // "valid", "corrupted", "missing"
console.log(report.fileSizeBytes);
console.log(report.pageCount);
```

```java
OverDrive.WatchdogReport report = OverDrive.watchdog("app.odb");
System.out.println(report.getIntegrityStatus());
System.out.println(report.getFileSizeBytes());
```

```go
report, err := overdrive.Watchdog("app.odb")
fmt.Println(report.IntegrityStatus)
fmt.Println(report.FileSizeBytes)
```

### WatchdogReport Fields

| Field | Type | Description |
|-------|------|-------------|
| `filePath` | string | Path that was inspected |
| `fileSizeBytes` | number | File size in bytes |
| `lastModified` | number | Unix timestamp of last modification |
| `integrityStatus` | string | `"valid"`, `"corrupted"`, or `"missing"` |
| `corruptionDetails` | string/null | Description if corrupted |
| `pageCount` | number | Number of pages in the file |
| `magicValid` | boolean | Whether the magic number is valid |

---

## 8. Thread-Safe Access

The base `OverDrive` class is not thread-safe. For multi-threaded applications, use the thread-safe wrappers.

### Python — `ThreadSafeOverDrive`

```python
from overdrive import ThreadSafeOverDrive
import threading

db = ThreadSafeOverDrive("app.odb")

def worker(thread_id):
    db.insert("logs", {"thread": thread_id, "event": "started"})
    results = db.query("SELECT * FROM logs")
    print(f"Thread {thread_id}: {len(results)} logs")

threads = [threading.Thread(target=worker, args=(i,)) for i in range(10)]
for t in threads: t.start()
for t in threads: t.join()

db.close()
```

### Node.js — `SharedOverDrive`

```javascript
const { SharedOverDrive } = require('overdrive-db');

const db = new SharedOverDrive('app.odb');

// All methods return Promises — safe for async/await
await db.insert('logs', { event: 'started' });
const results = await db.query('SELECT * FROM logs');
```

### Go — `SafeDB`

```go
db, _ := overdrive.OpenSafe("app.odb")
defer db.Close()

// Uses sync.RWMutex internally
var wg sync.WaitGroup
for i := 0; i < 10; i++ {
    wg.Add(1)
    go func(id int) {
        defer wg.Done()
        db.Insert("logs", map[string]any{"thread": id})
    }(i)
}
wg.Wait()
```

---

## Security Checklist

Before deploying to production:

- [ ] Passwords loaded from environment variables, not hardcoded
- [ ] Using `querySafe()` for any query that includes user input
- [ ] Backups configured and tested
- [ ] Backup integrity verified with `watchdog()`
- [ ] WAL cleanup after sensitive commits
- [ ] Thread-safe wrapper used if multiple goroutines/threads access the DB
- [ ] Database file stored outside the web root (not publicly accessible)
- [ ] File permissions verified (chmod 600 / Windows ACL)

---

## Key Rotation

To change the encryption password:

```python
import os

# 1. Open with old password
old_db = OverDrive.open("app.odb", password=os.environ["OLD_DB_PASSWORD"])

# 2. Export all data
tables = old_db.list_tables()
data = {}
for table in tables:
    data[table] = old_db.findAll(table)
old_db.close()

# 3. Rename old file
os.rename("app.odb", "app.odb.old")

# 4. Create new database with new password
new_db = OverDrive.open("app.odb", password=os.environ["NEW_DB_PASSWORD"])
for table, rows in data.items():
    for row in rows:
        row.pop("_id", None)  # remove old IDs
        new_db.insert(table, row)
new_db.close()

# 5. Verify new database
report = OverDrive.watchdog("app.odb")
assert report.integrity_status == "valid"

# 6. Delete old file
os.remove("app.odb.old")
print("Key rotation complete!")
```
