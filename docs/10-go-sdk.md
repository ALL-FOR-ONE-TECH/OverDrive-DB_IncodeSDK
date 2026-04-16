# Go SDK — Complete Guide

**Version:** 1.4.0 | **Requires:** Go 1.21+

---

## Installation

```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.4.0
```

> **Note:** The Go SDK uses CGo and requires the native library (`overdrive.dll`/`liboverdrive.so`/`liboverdrive.dylib`) to be in your library path or project directory.

---

## Import

```go
import overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"
```

---

## Opening a Database

```go
// Basic open (auto-creates if not exists)
db, err := overdrive.Open("myapp.odb")
if err != nil { panic(err) }
defer db.Close()

// With password encryption
db, err := overdrive.Open("secure.odb",
    overdrive.WithPassword("my-secret-pass-123"))

// With RAM engine
db, err := overdrive.Open("cache.odb",
    overdrive.WithEngine("RAM"))

// All options
db, err := overdrive.Open("app.odb",
    overdrive.WithPassword("my-secret-pass"),     // optional, min 8 chars
    overdrive.WithEngine("Disk"),                  // "Disk"|"RAM"|"Vector"|"Time-Series"|"Graph"|"Streaming"
    overdrive.WithAutoCreateTables(true),          // default true
)

// Environment variable encryption (v1.3 — still works)
// export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
db, err := overdrive.OpenEncrypted("app.odb", "ODB_KEY")

// Thread-safe wrapper
safeDB, err := overdrive.OpenSafe("app.odb")
defer safeDB.Close()
```

---

## Table Operations

```go
// Create table (optional — auto-created on first insert)
err := db.CreateTable("users")
err := db.CreateTable("sessions", overdrive.WithTableEngine("RAM"))

// Drop table
err := db.DropTable("old_table")

// List tables
tables, err := db.ListTables()  // []string{"users", "products", ...}

// Check existence
exists := db.TableExists("users")  // bool
```

---

## CRUD Operations

### Insert

```go
// Single document — returns auto-generated _id
id, err := db.Insert("users", map[string]any{
    "name":  "Alice",
    "age":   30,
    "email": "alice@example.com",
    "active": true,
})
fmt.Println(id)  // "users_1"

// Multiple documents
ids := make([]string, 0)
for _, doc := range docs {
    id, err := db.Insert("users", doc)
    if err != nil { return err }
    ids = append(ids, id)
}
```

### Get

```go
// Get by _id
user, err := db.Get("users", "users_1")
// → map[string]any{"_id": "users_1", "name": "Alice", ...}
// → nil if not found

// Count all documents
count, err := db.Count("users")  // int
```

### Update

```go
// Update by _id — only specified fields change
updated, err := db.Update("users", "users_1", map[string]any{
    "age":    31,
    "status": "active",
})
// → true if found and updated, false if not found
```

### Delete

```go
// Delete by _id
deleted, err := db.Delete("users", "users_1")
// → true if found and deleted, false if not found
```

---

## SQL Queries

```go
// Execute SQL — returns *QueryResult
result, err := db.Query("SELECT * FROM users")
result, err := db.Query("SELECT * FROM users WHERE age > 25")
result, err := db.Query("SELECT * FROM users WHERE age > 25 ORDER BY name DESC LIMIT 10")
result, err := db.Query("SELECT COUNT(*) FROM users")

// Access rows
for _, row := range result.Rows {
    fmt.Println(row["name"], row["age"])
}

// Safe parameterized query (use for user input!)
result, err := db.QuerySafe("SELECT * FROM users WHERE name = ?", userName)
result, err := db.QuerySafe("SELECT * FROM users WHERE age > ? AND city = ?", "25", "London")

// Full-text search
matches, err := db.Search("users", "alice")  // []map[string]any
```

---

## Helper Methods (v1.4)

```go
// FindOne — first match or nil
user, err := db.FindOne("users", "age > 25")
first, err := db.FindOne("users", "")  // first document, no filter

// FindAll — all matches
users, err := db.FindAll("users", "", "", 0)
users, err := db.FindAll("users", "age > 25", "", 0)
users, err := db.FindAll("users", "age > 25", "name ASC", 0)
users, err := db.FindAll("users", "age > 25", "name ASC", 10)

// UpdateMany — bulk update, returns count
count, err := db.UpdateMany("users", "status = 'trial'", map[string]any{"status": "active"})

// DeleteMany — bulk delete, returns count
count, err := db.DeleteMany("logs", "created_at < '2025-01-01'")

// CountWhere — count matching docs
n, err := db.CountWhere("users", "age > 25")
total, err := db.CountWhere("users", "")  // count all

// Exists — check if document exists by _id
found, err := db.Exists("users", "users_1")  // bool
```

---

## Transactions

```go
// Callback pattern (recommended — v1.4)
err := db.Transaction(func(txn *overdrive.TransactionHandle) error {
    _, err := db.UpdateMany("accounts", "id = 'alice'", map[string]any{"balance": 900})
    if err != nil { return err }
    _, err = db.UpdateMany("accounts", "id = 'bob'", map[string]any{"balance": 600})
    return err
}, overdrive.ReadCommitted)

// With retry on conflict
err := db.TransactionWithRetry(func(txn *overdrive.TransactionHandle) error {
    _, err := db.Insert("orders", map[string]any{"item": "widget"})
    return err
}, overdrive.ReadCommitted, 3)

// Manual (v1.3 — still works)
txn, err := db.BeginTransaction(overdrive.ReadCommitted)
if err != nil { panic(err) }

_, err = db.Insert("users", map[string]any{"name": "Alice"})
if err != nil {
    db.AbortTransaction(txn)
    return err
}
err = db.CommitTransaction(txn)

// Isolation level constants
overdrive.ReadUncommitted  // 0
overdrive.ReadCommitted    // 1 (default)
overdrive.RepeatableRead   // 2
overdrive.Serializable     // 3
```

---

## RAM Engine Methods (v1.4)

```go
// Snapshot — persist RAM database to disk
err := db.Snapshot("./backup/cache.odb")

// Restore — load snapshot into RAM database
err := db.Restore("./backup/cache.odb")

// Memory usage
usage, err := db.MemoryUsageStats()
fmt.Printf("Using %.1f MB (%.1f%%)\n", usage.Mb, usage.Percent)

// MemoryUsage fields
usage.Bytes       // int64 — bytes used
usage.Mb          // float64 — megabytes used
usage.LimitBytes  // int64 — memory limit
usage.Percent     // float64 — utilization percentage
```

---

## Watchdog (v1.4)

```go
// Package-level function — no open database needed
report, err := overdrive.Watchdog("app.odb")
if err != nil { panic(err) }

// WatchdogReport fields
report.FilePath           // string — path inspected
report.FileSizeBytes      // int64 — file size
report.LastModified       // int64 — Unix timestamp
report.IntegrityStatus    // string — "valid", "corrupted", "missing"
report.CorruptionDetails  // *string — details if corrupted (nil if valid)
report.PageCount          // int — number of pages
report.MagicValid         // bool — magic number valid

// Usage pattern
switch report.IntegrityStatus {
case "valid":
    db, err := overdrive.Open("app.odb")
case "corrupted":
    fmt.Fprintf(os.Stderr, "Corrupted: %s\n", *report.CorruptionDetails)
case "missing":
    fmt.Println("File not found — creating new database")
    db, err := overdrive.Open("app.odb")
}
```

---

## Error Handling

```go
import overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"

db, err := overdrive.Open("secure.odb", overdrive.WithPassword("wrong"))
if err != nil {
    // Check error type
    if authErr, ok := err.(*overdrive.AuthenticationError); ok {
        fmt.Println(authErr.Code())         // "ODB-AUTH-001"
        fmt.Println(authErr.Error())        // full message with suggestions
        fmt.Println(authErr.Suggestions())  // []string
        fmt.Println(authErr.DocLink())      // URL
    }

    // Or use the interface
    if ode, ok := err.(overdrive.OverDriveError); ok {
        fmt.Printf("Error [%s]: %s\n", ode.Code(), ode.Error())
    }
}
```

---

## Security

```go
// Password from environment variable
password := os.Getenv("DB_PASSWORD")
if password == "" {
    panic("DB_PASSWORD not set")
}

db, err := overdrive.Open("secure.odb", overdrive.WithPassword(password))
if err != nil { panic(err) }
defer db.Close()

// Backup
err = db.Backup("backups/app_2026-04-16.odb")

// WAL cleanup after commit
txn, _ := db.BeginTransaction(overdrive.ReadCommitted)
db.Insert("users", map[string]any{"name": "Alice"})
db.CommitTransaction(txn)
db.CleanupWAL()

// Safe queries
results, err := db.QuerySafe("SELECT * FROM users WHERE name = ?", userInput)
```

---

## Thread-Safe Access

```go
// SafeDB uses sync.RWMutex internally
safeDB, err := overdrive.OpenSafe("app.odb")
defer safeDB.Close()

var wg sync.WaitGroup
for i := 0; i < 10; i++ {
    wg.Add(1)
    go func(id int) {
        defer wg.Done()
        safeDB.Insert("logs", map[string]any{"thread": id, "event": "started"})
    }(i)
}
wg.Wait()
```

---

## Complete Example

```go
package main

import (
    "fmt"
    "os"
    overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"
)

func main() {
    db, err := overdrive.Open("store.odb")
    if err != nil { panic(err) }
    defer db.Close()

    // Insert products (table auto-created)
    products := []map[string]any{
        {"name": "Laptop",  "price": 999,  "category": "electronics", "stock": 5},
        {"name": "Mouse",   "price": 29,   "category": "electronics", "stock": 50},
        {"name": "Desk",    "price": 299,  "category": "furniture",   "stock": 10},
        {"name": "Chair",   "price": 199,  "category": "furniture",   "stock": 15},
    }
    for _, p := range products {
        id, _ := db.Insert("products", p)
        fmt.Printf("Inserted: %s\n", id)
    }

    // Query
    electronics, _ := db.FindAll("products", "category = 'electronics'", "price DESC", 0)
    fmt.Printf("Electronics: %d items\n", len(electronics))

    // Count
    total, _ := db.CountWhere("products", "")
    expensive, _ := db.CountWhere("products", "price > 100")
    fmt.Printf("Total: %d, Expensive: %d\n", total, expensive)

    // Update with transaction
    err = db.Transaction(func(txn *overdrive.TransactionHandle) error {
        count, err := db.UpdateMany("products", "price > 100",
            map[string]any{"onSale": true, "discount": 10})
        if err != nil { return err }
        _, err = db.Insert("auditLog", map[string]any{"event": "saleApplied", "affected": count})
        fmt.Printf("Applied sale to %d products\n", count)
        return err
    }, overdrive.ReadCommitted)
    if err != nil { panic(err) }

    // Watchdog check
    report, _ := overdrive.Watchdog("store.odb")
    fmt.Printf("Database health: %s (%d bytes)\n",
        report.IntegrityStatus, report.FileSizeBytes)

    // Backup
    db.Backup("backups/store_backup.odb")
    fmt.Println("Backup created!")
}
```
