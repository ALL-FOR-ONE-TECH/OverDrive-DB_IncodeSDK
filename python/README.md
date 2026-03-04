# OverDrive-DB — Python SDK v1.3.0

**A high-performance, embeddable hybrid SQL+NoSQL document database. Like SQLite, but for JSON.**

> **v1.3.0** — Security hardened: encrypted key from env, parameterized queries, auto `chmod 600`, encrypted backups, WAL cleanup, thread-safe wrapper.

## Install

```bash
pip install overdrive-db==1.3.0
```

Place the native library for your platform in your project directory or on `PATH`:

| Platform | File |
|---|---|
| Windows x64 | `overdrive.dll` |
| Linux x64 | `liboverdrive.so` |
| macOS ARM64 | `liboverdrive.dylib` |

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest).

## Quick Start

```python
from overdrive import OverDrive

# Open — file permissions auto-hardened (chmod 600 / Windows ACL)
with OverDrive("myapp.odb") as db:
    db.create_table("users")
    user_id = db.insert("users", {"name": "Alice", "email": "alice@example.com", "age": 30})

    # ✅ Safe parameterized query — blocks SQL injection
    results = db.query_safe("SELECT * FROM users WHERE age > ?", [25])
    for row in results:
        print(f"  {row['name']} — {row['email']}")

    # Backup + WAL cleanup
    db.backup("backups/app.odb")
    db.cleanup_wal()
```

## Security APIs (v1.3.0)

```python
import os
from overdrive import OverDrive, ThreadSafeOverDrive

# 🔐 Open with encryption key from env (never hardcode!)
# $env:ODB_KEY = "my-secret-32-char-key!!!!" (PowerShell)
# export ODB_KEY="my-secret-32-char-key!!!!"  (bash)
db = OverDrive.open_encrypted("app.odb", "ODB_KEY")

# 🛡️ SQL injection prevention — use query_safe() for user input
user_input = "Alice'; DROP TABLE users--"  # malicious
try:
    db.query_safe("SELECT * FROM users WHERE name = ?", [user_input])
except Exception as e:
    print(f"Blocked: {e}")  # ✅ injection blocked

# 💾 Encrypted backup (sync → copy .odb + .wal → harden perms)
db.backup("backups/app_2026-03-04.odb")

# 🗑️ WAL cleanup after commit
txn_id = db.begin_transaction(db.SERIALIZABLE)
db.insert("users", {"name": "Carol"})
db.commit_transaction(txn_id)
db.cleanup_wal()

# 🧵 Thread-safe access
with ThreadSafeOverDrive("app.odb") as safe_db:
    import threading
    threads = [threading.Thread(target=lambda: safe_db.query("SELECT * FROM users")) for _ in range(4)]
    for t in threads: t.start()
    for t in threads: t.join()
```

## Full API

| Method | Description |
|---|---|
| `OverDrive(path)` | Open database (auto-hardens permissions) |
| `OverDrive.open_encrypted(path, key_env_var)` | 🔐 Open with key from env var |
| `db.close()` | Close the database |
| `db.sync()` | Force flush to disk |
| `db.create_table(name)` | Create a table |
| `db.drop_table(name)` | Drop a table |
| `db.list_tables()` | List all tables |
| `db.table_exists(name)` | Check if table exists |
| `db.insert(table, doc)` | Insert document, returns `_id` |
| `db.insert_many(table, docs)` | Batch insert |
| `db.get(table, id)` | Get by `_id` |
| `db.update(table, id, updates)` | Update fields |
| `db.delete(table, id)` | Delete by `_id` |
| `db.count(table)` | Count documents |
| `db.scan(table)` | Get all documents |
| `db.query(sql)` | Execute SQL query (trusted input only) |
| `db.query_safe(sql, params)` | ✅ Parameterized query (user input safe) |
| `db.search(table, text)` | Full-text search |
| `db.backup(dest_path)` | 💾 Encrypted backup |
| `db.cleanup_wal()` | 🗑️ Delete stale WAL file |
| `db.begin_transaction(isolation)` | Start MVCC transaction |
| `db.commit_transaction(txn_id)` | Commit transaction |
| `db.abort_transaction(txn_id)` | Rollback transaction |
| `db.verify_integrity()` | Check database integrity |
| `db.stats()` | Database statistics |
| `ThreadSafeOverDrive(path)` | 🧵 Thread-safe wrapper |

## Links

- [Full Documentation](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/python-guide.md)
- [GitHub](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK)
- [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases)
- [Security Guide](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/architecture.md#security-model)

## License

MIT / Apache-2.0
