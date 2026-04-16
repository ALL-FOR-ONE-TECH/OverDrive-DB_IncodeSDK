# Security Best Practices — OverDrive-DB InCode SDK

## Password Selection Guidelines

### Requirements
- **Minimum:** 8 characters (enforced by SDK)
- **Recommended:** 12+ characters for production use
- **Encoding:** Full UTF-8 support

### Best Practices
1. **Use a strong password generator** — avoid dictionary words
2. **Never hardcode passwords** in source code — use environment variables or secrets managers
3. **Use different passwords** for each database file
4. **Document your password policy** for your team

### Example: Using Environment Variables
```python
import os

# Load password from environment
password = os.environ.get("DB_PASSWORD")
if not password:
    raise ValueError("DB_PASSWORD environment variable not set")

db = OverDrive.open("app.odb", password=password)
```

```javascript
const password = process.env.DB_PASSWORD;
if (!password) throw new Error('DB_PASSWORD not set');
const db = OverDrive.open('app.odb', { password });
```

---

## Key Rotation Strategies

### Why Rotate Keys?
- Minimize exposure if a key is compromised
- Comply with security policies (e.g., rotate every 90 days)
- Revoke access for former team members

### How to Rotate

1. **Open with old password:**
   ```python
   old_db = OverDrive.open("app.odb", password="old-password-123")
   ```

2. **Create backup with new password:**
   ```python
   # Export all data
   tables = old_db.listTables()
   data = {}
   for table in tables:
       data[table] = old_db.findAll(table)
   old_db.close()
   ```

3. **Create new database with new password:**
   ```python
   import os
   os.rename("app.odb", "app.odb.old")
   
   new_db = OverDrive.open("app.odb", password="new-password-456")
   for table, rows in data.items():
       for row in rows:
           new_db.insert(table, row)
   new_db.close()
   
   os.remove("app.odb.old")
   ```

---

## Backup Encryption Best Practices

### Always Encrypt Backups
```python
# Backup inherits the encryption of the source database
db.backup("backups/app_2026-04-15.odb")
```

### Store Backups Separately
- Use a different drive, cloud storage, or offsite location
- Never store backups on the same disk as the original

### Verify Backups
```python
# Use watchdog to verify backup integrity
report = OverDrive.watchdog("backups/app_2026-04-15.odb")
assert report.integrity_status == "valid"
```

### Clean Up WAL After Backup
```python
db.backup("backups/app.odb")
db.cleanupWal()  # Prevents stale WAL replay attacks
```

---

## File Permission Hardening

### Automatic Hardening
OverDrive-DB automatically sets restrictive file permissions on all `.odb` files:

| Platform      | Permission       | Description              |
|---------------|------------------|--------------------------|
| Windows       | `icacls /grant:r`| Owner-only full control  |
| Linux/macOS   | `chmod 600`      | Owner read/write only    |

### Manual Hardening
If you need additional hardening:

**Linux/macOS:**
```bash
chmod 600 app.odb app.odb.wal
chown $(whoami) app.odb app.odb.wal
```

**Windows (PowerShell):**
```powershell
icacls app.odb /inheritance:r /grant:r "$env:USERNAME:F"
```

### Directory Permissions
Protect the directory containing your database:
```bash
chmod 700 /path/to/db/
```

---

## SQL Injection Prevention

### Always Use `querySafe()` for User Input

**UNSAFE — Never do this:**
```python
# ⛔ DANGEROUS — SQL injection vulnerable
name = user_input  # Could be: "'; DROP TABLE users; --"
db.query(f"SELECT * FROM users WHERE name = '{name}'")
```

**SAFE — Always do this:**
```python
# ✅ SAFE — parameterized query
results = db.querySafe("SELECT * FROM users WHERE name = ?", user_input)
```

### What `querySafe()` Blocks
- Comment tokens: `--`, `/*`, `*/`
- Dangerous keywords: `DROP`, `TRUNCATE`, `ALTER`, `EXEC`, `EXECUTE`, `UNION`, `XP_`
- Automatic single-quote escaping

### Helper Methods Are Already Safe
The helper methods (`findOne`, `findAll`, etc.) accept WHERE clauses as strings. For user input in WHERE clauses, always sanitize first:

```python
# ✅ SAFE — use querySafe for user input
results = db.querySafe("SELECT * FROM users WHERE email = ?", user_email)

# Or use helper methods with trusted conditions only
admin_users = db.findAll("users", "role = 'admin'")  # Trusted condition
```

---

## Thread Safety

### Single-Threaded Use
The base `OverDrive` class is **not thread-safe**. Use one instance per thread, or use the thread-safe wrappers:

### Thread-Safe Wrappers

**Python:**
```python
from overdrive import SharedOverDrive
db = SharedOverDrive("app.odb")  # Thread-safe with async queue
```

**Node.js:**
```javascript
const { SharedOverDrive } = require('overdrive-db');
const db = new SharedOverDrive('app.odb');  // Promise-based mutex
```

**Java:**
```java
OverDriveSafe db = OverDriveSafe.open("app.odb");  // ReentrantReadWriteLock
```

**Go:**
```go
db, _ := overdrive.OpenSafe("app.odb")  // sync.RWMutex
```

---

## Encryption Details

### Algorithm: AES-256-GCM
- 256-bit key, 96-bit nonce, 128-bit auth tag
- Authenticated encryption — detects tampering

### Key Derivation: Argon2id (password mode)
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4
- Output: 32 bytes (256-bit key)
- Salt: 16 random bytes (stored in file header)

### File Header Layout
```
Offset  Size  Field
------  ----  -----
0       8     Magic number
8       4     Version
12      1     Encryption flag (0=none, 1=env-var, 2=password)
13      16    Salt (for password-based encryption)
29      ...   Existing fields
```

### Encryption Modes
| Mode | Flag | Description |
|------|------|-------------|
| None | 0    | Unencrypted database |
| Environment Variable | 1 | Key loaded from env var via `openEncrypted()` |
| Password | 2 | Key derived via Argon2id from user password |

> [!WARNING]
> Cannot mix modes: a database created with password encryption must always be opened with the same password. Environment-variable and password encryption are not interchangeable.
