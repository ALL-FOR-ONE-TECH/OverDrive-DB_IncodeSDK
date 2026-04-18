# OverDrive InCode SDK — Go v1.4.0

**Embeddable hybrid SQL+NoSQL document database. Like SQLite for JSON.**

> **v1.4.0** — Password encryption, RAM engine, watchdog monitoring, transaction callbacks, helper methods, auto-table creation, structured error codes. 100% backward compatible with v1.3.

---

## Install

```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.4.0
```

Place the native library in `lib/` next to your binary:

| Platform | File |
|----------|------|
| Windows x64 | `overdrive.dll` |
| Linux x64 | `liboverdrive.so` |
| Linux ARM64 | `liboverdrive-arm64.so` |
| macOS x64 | `liboverdrive.dylib` |
| macOS ARM64 | `liboverdrive-arm64.dylib` |

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

---

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"
)

func main() {
    // Open or create — table auto-created on first insert
    db, err := overdrive.Open("myapp.odb")
    if err != nil { log.Fatal(err) }
    defer db.Close()

    // Insert — "users" table auto-created
    id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})
    fmt.Println("Inserted:", id)  // users_1

    // SQL query
    result, _ := db.Query("SELECT * FROM users WHERE age > 25 ORDER BY name")
    for _, row := range result.Rows {
        fmt.Printf("  %s (age %v)\n", row["name"], row["age"])
    }
}
```

---

## v1.4 Features

### Password-Protected Database

```go
import "os"

// Load password from environment variable — never hardcode!
db, err := overdrive.Open("secure.odb",
    overdrive.WithPassword(os.Getenv("DB_PASSWORD")))
if err != nil { log.Fatal(err) }
defer db.Close()
```

### RAM Engine — Sub-Microsecond Reads

```go
// Full RAM database
cache, _ := overdrive.Open("cache.odb", overdrive.WithEngine("RAM"))
defer cache.Close()

cache.Insert("sessions", map[string]any{"userId": 123, "token": "abc"})

// Memory usage
usage, _ := cache.MemoryUsageStats()
fmt.Printf("RAM: %.1f MB (%.1f%%)\n", usage.Mb, usage.Percent)

// Persist to disk
cache.Snapshot("./backup/cache.odb")

// Restore later
cache.Restore("./backup/cache.odb")

// Per-table RAM in a disk database
db, _ := overdrive.Open("app.odb")
db.CreateTable("sessions", overdrive.WithTableEngine("RAM"))  // RAM table
db.Insert("users", map[string]any{"name": "Alice"})           // Disk table
```

### Watchdog — File Integrity

```go
// Check file integrity without opening the database
report, err := overdrive.Watchdog("myapp.odb")
if err != nil { log.Fatal(err) }

fmt.Println(report.IntegrityStatus)  // "valid", "corrupted", "missing"
fmt.Println(report.FileSizeBytes)
fmt.Println(report.PageCount)

if report.IntegrityStatus == "corrupted" {
    fmt.Println("Details:", *report.CorruptionDetails)
}
```

### Transaction Callback Pattern

```go
// Auto-commits on success, auto-rolls back on error
err := db.Transaction(func(txn *overdrive.TransactionHandle) error {
    _, err := db.UpdateMany("accounts", "id = 'alice'",
        map[string]any{"balance": 900})
    if err != nil { return err }
    _, err = db.UpdateMany("accounts", "id = 'bob'",
        map[string]any{"balance": 600})
    return err
}, overdrive.ReadCommitted)

// With retry on conflict
err = db.TransactionWithRetry(func(txn *overdrive.TransactionHandle) error {
    _, err := db.Insert("orders", map[string]any{"item": "widget"})
    return err
}, overdrive.ReadCommitted, 3)
```

### Helper Methods

```go
// findOne — first match or nil
user, _ := db.FindOne("users", "age > 25")

// findAll — all matches with sorting and limit
users, _ := db.FindAll("users", "age > 25", "name ASC", 10)

// updateMany — bulk update, returns count
count, _ := db.UpdateMany("users", "status = 'trial'",
    map[string]any{"status": "active"})
fmt.Printf("Updated %d users\n", count)

// deleteMany — bulk delete, returns count
count, _ = db.DeleteMany("logs", "old = true")

// countWhere — count matching documents
n, _ := db.CountWhere("users", "age > 25")
total, _ := db.CountWhere("users", "")  // count all

// exists — check by _id
found, _ := db.Exists("users", "users_1")
```

### Auto-Table Creation

```go
// Tables are auto-created on first insert (default: true)
db, _ := overdrive.Open("app.odb", overdrive.WithAutoCreateTables(true))
db.Insert("users", map[string]any{"name": "Alice"})  // "users" auto-created

// Disable for strict mode
db2, _ := overdrive.Open("app.odb", overdrive.WithAutoCreateTables(false))
// Must call db2.CreateTable("users") before inserting
```

---

## Security

```go
import "os"

// Password from environment variable
db, _ := overdrive.Open("secure.odb",
    overdrive.WithPassword(os.Getenv("DB_PASSWORD")))

// Environment variable encryption (v1.3 — still works)
// export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
db2, _ := overdrive.OpenEncrypted("app.odb", "ODB_KEY")

// SQL injection prevention
userInput := "Alice'; DROP TABLE users--"
result, err := db.QuerySafe("SELECT * FROM users WHERE name = ?", userInput)
if err != nil {
    fmt.Println("Blocked:", err)  // injection blocked
}

// Encrypted backup
db.Backup("backups/app_2026-04-16.odb")

// WAL cleanup after commit
txn, _ := db.BeginTransaction(overdrive.ReadCommitted)
db.Insert("users", map[string]any{"name": "Alice"})
db.CommitTransaction(txn)
db.CleanupWAL()
```

---

## Error Handling

```go
db, err := overdrive.Open("secure.odb",
    overdrive.WithPassword("wrong"))
if err != nil {
    switch e := err.(type) {
    case *overdrive.AuthenticationError:
        fmt.Printf("Auth error [%s]: %s\n", e.Code(), e.Error())
        fmt.Printf("Suggestions: %v\n", e.Suggestions())
    case *overdrive.FFIError:
        fmt.Printf("Library error [%s]: %s\n", e.Code(), e.Error())
    default:
        if ode, ok := err.(overdrive.OverDriveError); ok {
            fmt.Printf("[%s] %s\n", ode.Code(), ode.Error())
        }
    }
}
```

### Error Types

| Type | Code Prefix | When |
|------|-------------|------|
| `*AuthenticationError` | `ODB-AUTH-*` | Wrong password, password too short |
| `*TableError` | `ODB-TABLE-*` | Table not found, already exists |
| `*QueryError` | `ODB-QUERY-*` | SQL syntax error |
| `*TransactionError` | `ODB-TXN-*` | Deadlock, conflict |
| `*IOError` | `ODB-IO-*` | File not found, corrupted |
| `*FFIError` | `ODB-FFI-*` | Native library not found |

---

## Thread-Safe Access

```go
// SafeDB uses sync.RWMutex — reads parallel, writes exclusive
safeDB, _ := overdrive.OpenSafe("app.odb")
defer safeDB.Close()

var wg sync.WaitGroup
for i := 0; i < 10; i++ {
    wg.Add(1)
    go func(id int) {
        defer wg.Done()
        safeDB.Insert("logs", map[string]any{"thread": id})
    }(i)
}
wg.Wait()
```

---

## Full API Reference

### Opening

```go
overdrive.Open(path, ...OpenOption) (*DB, error)
overdrive.OpenEncrypted(path, keyEnvVar string) (*DB, error)
overdrive.OpenSafe(path, ...OpenOption) (*SafeDB, error)
overdrive.Watchdog(filePath string) (*WatchdogReport, error)
overdrive.Version() string

// Options
overdrive.WithPassword(pwd string) OpenOption
overdrive.WithEngine(engine string) OpenOption      // "Disk"|"RAM"|"Vector"|"Time-Series"|"Graph"|"Streaming"
overdrive.WithAutoCreateTables(bool) OpenOption
```

### Tables & CRUD

```go
db.CreateTable(name string, opts ...TableOption) error
db.CreateTable(name, overdrive.WithTableEngine("RAM"))
db.DropTable(name string) error
db.ListTables() ([]string, error)
db.TableExists(name string) bool

db.Insert(table string, doc map[string]any) (string, error)
db.Get(table, id string) (map[string]any, error)
db.Update(table, id string, updates map[string]any) (bool, error)
db.Delete(table, id string) (bool, error)
db.Count(table string) (int, error)
```

### Query

```go
db.Query(sql string) (*QueryResult, error)
db.QuerySafe(sql string, params ...string) (*QueryResult, error)
db.Search(table, text string) ([]map[string]any, error)
```

### Helper Methods (v1.4)

```go
db.FindOne(table, where string) (map[string]any, error)
db.FindAll(table, where, orderBy string, limit int) ([]map[string]any, error)
db.UpdateMany(table, where string, updates map[string]any) (int, error)
db.DeleteMany(table, where string) (int, error)
db.CountWhere(table, where string) (int, error)
db.Exists(table, id string) (bool, error)
```

### Transactions

```go
db.Transaction(fn func(*TransactionHandle) error, isolation IsolationLevel) error
db.TransactionWithRetry(fn func(*TransactionHandle) error, isolation IsolationLevel, maxRetries int) error
db.BeginTransaction(isolation IsolationLevel) (*TransactionHandle, error)
db.CommitTransaction(txn *TransactionHandle) error
db.AbortTransaction(txn *TransactionHandle) error

// Isolation levels
overdrive.ReadUncommitted  // 0
overdrive.ReadCommitted    // 1 (default)
overdrive.RepeatableRead   // 2
overdrive.Serializable     // 3
```

### RAM Engine (v1.4)

```go
db.Snapshot(path string) error
db.Restore(path string) error
db.MemoryUsageStats() (*MemoryUsage, error)
// MemoryUsage: .Bytes, .Mb, .LimitBytes, .Percent
```

### Security & Maintenance

```go
db.Backup(destPath string) error
db.CleanupWAL() error
db.VerifyIntegrity() (*IntegrityReport, error)
db.Stats() (*StatsResult, error)
db.Close()
db.Sync()
db.Path() string
```

---

## Migration from v1.3

```go
// v1.3 — still works
db, _ := overdrive.Open("app.odb")
db.CreateTable("users")
db.Insert("users", map[string]any{"name": "Alice"})

// v1.4 — recommended
db, _ := overdrive.Open("app.odb")  // auto_create_tables=true by default
db.Insert("users", map[string]any{"name": "Alice"})  // table auto-created

// v1.3 manual transaction — still works
txn, _ := db.BeginTransaction(overdrive.ReadCommitted)
db.Insert("orders", map[string]any{"item": "widget"})
db.CommitTransaction(txn)

// v1.4 callback pattern — recommended
db.Transaction(func(txn *overdrive.TransactionHandle) error {
    _, err := db.Insert("orders", map[string]any{"item": "widget"})
    return err
}, overdrive.ReadCommitted)
```

---

## Links

- **GitHub:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK
- **Releases:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases
- **Issues:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues
- **Docs:** https://all-for-one-tech.github.io/OverDrive-DB_IncodeSDK/guide.html

## License

MIT / Apache-2.0
