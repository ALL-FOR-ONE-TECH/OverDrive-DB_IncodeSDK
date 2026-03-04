# OverDrive InCode SDK — Go v1.3.0

**Embeddable document database — like SQLite for JSON.**

> **v1.3.0** — Security hardened: `OpenEncrypted`, `QuerySafe` SQL injection prevention, auto `chmod 600`, `Backup`, `CleanupWAL`, concurrent-safe `SafeDB`.

## Install

```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go@v1.3.0
```

Place the native library in `lib/` and set your CGo linker path:

```bash
# Download from GitHub Releases for your platform:
# overdrive.dll (Windows) / liboverdrive.so (Linux) / liboverdrive.dylib (macOS)
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go"
)

func main() {
    // Open — file permissions auto-hardened (chmod 600 / Windows ACL)
    db, err := overdrive.Open("myapp.odb")
    if err != nil { log.Fatal(err) }
    defer db.Close()

    db.CreateTable("users")
    id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})
    fmt.Println("Inserted:", id)

    // ✅ Safe parameterized query — blocks SQL injection
    result, _ := db.QuerySafe("SELECT * FROM users WHERE age > ?", "25")
    for _, row := range result.Rows {
        fmt.Printf("  %s (age %v)\n", row["name"], row["age"])
    }
}
```

## Security APIs (v1.3.0)

```go
import (
    "os"
    overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go"
)

// 🔐 Open with encryption key from env (never hardcode!)
// export ODB_KEY="my-secret-32-char-key!!!!"       (bash)
// $env:ODB_KEY="my-secret-32-char-key!!!!"         (PowerShell)
db, err := overdrive.OpenEncrypted("app.odb", "ODB_KEY")

// 🛡️ SQL injection prevention
userInput := "Alice'; DROP TABLE users--"
result, err := db.QuerySafe("SELECT * FROM users WHERE name = ?", userInput)
if err != nil {
    fmt.Println("Blocked:", err) // ✅ injection blocked
}

// 💾 Encrypted backup
db.Backup("backups/app_2026-03-04.odb")

// 🗑️ WAL cleanup after commit
txn, _ := db.BeginTransaction(overdrive.Serializable)
db.CommitTransaction(txn)
db.CleanupWAL()

// 🧵 Concurrent-safe access (RWMutex — reads parallel, writes exclusive)
safe, _ := overdrive.OpenSafe("app.odb")
// or: safe := overdrive.NewSafeDB(db)
go safe.Query("SELECT * FROM users")     // read lock
go safe.Insert("users", map[string]any{"name": "Bob"}) // write lock
```

## Full API

### Core
| Function | Description |
|---|---|
| `overdrive.Open(path)` | Open or create (auto-hardens permissions) |
| `overdrive.OpenEncrypted(path, keyEnvVar)` | 🔐 Open with key from env var |
| `overdrive.OpenSafe(path)` | Open as concurrent-safe `SafeDB` |
| `overdrive.OpenSafeEncrypted(path, keyEnvVar)` | Encrypted + concurrent-safe |
| `overdrive.NewSafeDB(db)` | Wrap existing DB in SafeDB |
| `db.Close()` | Close the database |
| `db.Sync()` | Force flush to disk |
| `db.Path()` | Get database path |

### Tables & CRUD
| Function | Description |
|---|---|
| `db.CreateTable(name)` | Create a table |
| `db.DropTable(name)` | Drop a table |
| `db.ListTables()` | List all tables |
| `db.TableExists(name)` | Check if table exists |
| `db.Insert(table, doc)` | Insert document, returns `_id` |
| `db.Get(table, id)` | Get by `_id` |
| `db.Update(table, id, updates)` | Update fields |
| `db.Delete(table, id)` | Delete by `_id` |
| `db.Count(table)` | Count documents |

### Query & Security
| Function | Description |
|---|---|
| `db.Query(sql)` | Execute SQL (trusted input only) |
| `db.QuerySafe(sql, params...)` | ✅ Parameterized query (user input safe) |
| `db.Search(table, text)` | Full-text search |
| `db.Backup(destPath)` | 💾 Encrypted backup |
| `db.CleanupWAL()` | 🗑️ Delete stale WAL file |

### Transactions
| Function | Description |
|---|---|
| `db.BeginTransaction(isolation)` | Start MVCC transaction |
| `db.CommitTransaction(txn)` | Commit transaction |
| `db.AbortTransaction(txn)` | Rollback transaction |
| `db.VerifyIntegrity()` | Check database integrity |
| `db.Stats()` | Database statistics |

## Isolation Levels

```go
overdrive.ReadUncommitted
overdrive.ReadCommitted
overdrive.RepeatableRead
overdrive.Serializable
```

## Links

- [GitHub](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK)
- [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases)
- [Security Guide](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/architecture.md#security-model)

## License

MIT / Apache-2.0
