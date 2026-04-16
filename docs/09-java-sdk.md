# Java SDK — Complete Guide

**Version:** 1.4.0 | **Requires:** Java 11+

---

## Installation (Maven)

```xml
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.4.0</version>
</dependency>
```

---

## Import

```java
import com.afot.overdrive.OverDrive;
import com.afot.overdrive.OverDriveException;
import com.afot.overdrive.OverDriveException.*;
import java.util.Map;
import java.util.List;
```

---

## Opening a Database

```java
// Basic open (auto-creates if not exists)
OverDrive db = OverDrive.open("myapp.odb");

// With password encryption
OverDrive db = OverDrive.open("secure.odb",
    new OverDrive.OpenOptions().password("my-secret-pass-123"));

// With RAM engine
OverDrive db = OverDrive.open("cache.odb",
    new OverDrive.OpenOptions().engine("RAM"));

// All options — fluent builder
OverDrive db = OverDrive.open("app.odb", new OverDrive.OpenOptions()
    .password("my-secret-pass")     // optional, min 8 chars
    .engine("Disk")                  // "Disk"|"RAM"|"Vector"|"Time-Series"|"Graph"|"Streaming"
    .autoCreateTables(true)          // default true
);

// Environment variable encryption (v1.3 — still works)
// export ODB_KEY="my-aes-256-key-32-chars-minimum!!!!"
OverDrive db = OverDrive.openEncrypted("app.odb", "ODB_KEY");

// try-with-resources — auto-closes
try (OverDrive db = OverDrive.open("myapp.odb")) {
    db.insert("users", Map.of("name", "Alice"));
}  // auto-closes here
```

---

## Table Operations

```java
// Create table (optional — auto-created on first insert)
db.createTable("users");
db.createTable("sessions", new OverDrive.TableOptions().engine("RAM"));

// Drop table
db.dropTable("old_table");

// List tables
List<String> tables = db.listTables();  // ["users", "products", ...]

// Check existence
boolean exists = db.tableExists("users");
```

---

## CRUD Operations

### Insert

```java
// Single document — returns auto-generated _id
String id = db.insert("users", Map.of(
    "name", "Alice",
    "age", 30,
    "email", "alice@example.com",
    "active", true
));
System.out.println(id);  // "users_1"

// Multiple documents
List<String> ids = db.insertMany("users", List.of(
    Map.of("name", "Bob",   "age", 25),
    Map.of("name", "Carol", "age", 35)
));
System.out.println(ids);  // ["users_2", "users_3"]
```

### Get

```java
// Get by _id
Map<String, Object> user = db.get("users", "users_1");
// → {"_id": "users_1", "name": "Alice", "age": 30, ...}
// → null if not found

// Count all documents
int count = db.count("users");
```

### Update

```java
// Update by _id — only specified fields change
boolean updated = db.update("users", "users_1", Map.of("age", 31, "status", "active"));
// → true if found and updated, false if not found
```

### Delete

```java
// Delete by _id
boolean deleted = db.delete("users", "users_1");
// → true if found and deleted, false if not found
```

---

## SQL Queries

```java
// Execute SQL — returns List of Maps
List<Map<String, Object>> results = db.query("SELECT * FROM users");
List<Map<String, Object>> results = db.query("SELECT * FROM users WHERE age > 25");
List<Map<String, Object>> results = db.query(
    "SELECT * FROM users WHERE age > 25 ORDER BY name DESC LIMIT 10"
);

// Full result with metadata
Map<String, Object> result = db.queryFull("SELECT * FROM users");
// → {"rows": [...], "columns": [...], "rows_affected": 0}

// Safe parameterized query (use for user input!)
List<Map<String, Object>> results = db.querySafe(
    "SELECT * FROM users WHERE name = ?", userName
);
List<Map<String, Object>> results = db.querySafe(
    "SELECT * FROM users WHERE age > ? AND city = ?", "25", "London"
);

// Full-text search
List<Map<String, Object>> matches = db.search("users", "alice");
```

---

## Helper Methods (v1.4)

```java
// findOne — first match or null
Map<String, Object> user = db.findOne("users", "age > 25");
Map<String, Object> first = db.findOne("users");  // first document, no filter

// findAll — all matches
List<Map<String, Object>> users = db.findAll("users", null, null, 0);
List<Map<String, Object>> users = db.findAll("users", "age > 25", null, 0);
List<Map<String, Object>> users = db.findAll("users", "age > 25", "name ASC", 0);
List<Map<String, Object>> users = db.findAll("users", "age > 25", "name ASC", 10);

// updateMany — bulk update, returns count
int count = db.updateMany("users", "status = 'trial'", Map.of("status", "active"));

// deleteMany — bulk delete, returns count
int count = db.deleteMany("logs", "created_at < '2025-01-01'");

// countWhere — count matching docs
int n = db.countWhere("users", "age > 25");
int total = db.countWhere("users");  // count all

// exists — check if document exists by _id
boolean found = db.exists("users", "users_1");
```

---

## Transactions

```java
// Callback pattern — generic return type (recommended — v1.4)
String result = db.transaction(txn -> {
    db.updateMany("accounts", "id = 'alice'", Map.of("balance", 900));
    db.updateMany("accounts", "id = 'bob'",   Map.of("balance", 600));
    return "transfer complete";
});

// With isolation level
Integer count = db.transaction(txn -> {
    db.insert("logs", Map.of("event", "test"));
    return db.count("logs");
}, OverDrive.SERIALIZABLE);

// With retry on conflict
String result = db.transactionWithRetry(txn -> {
    db.insert("orders", Map.of("item", "widget"));
    return "done";
}, OverDrive.READ_COMMITTED, 3);

// Manual (v1.3 — still works)
long txnId = db.beginTransaction();
long txnId = db.beginTransaction(OverDrive.SERIALIZABLE);
db.commitTransaction(txnId);
db.abortTransaction(txnId);

// Isolation level constants
OverDrive.READ_UNCOMMITTED  // 0
OverDrive.READ_COMMITTED    // 1 (default)
OverDrive.REPEATABLE_READ   // 2
OverDrive.SERIALIZABLE      // 3
```

---

## RAM Engine Methods (v1.4)

```java
// Snapshot — persist RAM database to disk
db.snapshot("./backup/cache.odb");

// Restore — load snapshot into RAM database
db.restore("./backup/cache.odb");

// Memory usage
OverDrive.MemoryUsage usage = db.memoryUsage();
System.out.printf("Using %.1f MB (%.1f%%)%n", usage.getMb(), usage.getPercent());

// MemoryUsage fields
usage.getBytes()       // long — bytes used
usage.getMb()          // double — megabytes used
usage.getLimitBytes()  // long — memory limit
usage.getPercent()     // double — utilization percentage
```

---

## Watchdog (v1.4)

```java
// Static method — no open database needed
OverDrive.WatchdogReport report = OverDrive.watchdog("app.odb");

// WatchdogReport fields
report.getFilePath()           // String — path inspected
report.getFileSizeBytes()      // long — file size
report.getLastModified()       // long — Unix timestamp
report.getIntegrityStatus()    // String — "valid", "corrupted", "missing"
report.getCorruptionDetails()  // String — details if corrupted (null if valid)
report.getPageCount()          // int — number of pages
report.isMagicValid()          // boolean — magic number valid

// Usage pattern
switch (report.getIntegrityStatus()) {
    case "valid":
        OverDrive db = OverDrive.open("app.odb");
        break;
    case "corrupted":
        System.err.println("Corrupted: " + report.getCorruptionDetails());
        break;
    case "missing":
        System.out.println("File not found — creating new database");
        OverDrive db = OverDrive.open("app.odb");
        break;
}
```

---

## Error Handling

```java
import com.afot.overdrive.OverDriveException;
import com.afot.overdrive.OverDriveException.*;

try {
    OverDrive db = OverDrive.open("secure.odb",
        new OverDrive.OpenOptions().password("wrong"));
} catch (AuthenticationException e) {
    System.out.println(e.getCode());         // "ODB-AUTH-001"
    System.out.println(e.getMessage());      // "Incorrect password..."
    System.out.println(e.getContext());      // "secure.odb"
    System.out.println(e.getSuggestions());  // ["Verify you're using the correct password", ...]
    System.out.println(e.getDocLink());      // "https://overdrive-db.com/docs/errors/ODB-AUTH-001"
}

try {
    db.query("INVALID SQL !!!");
} catch (QueryException e) {
    System.out.println("Query error: " + e.getCode() + " — " + e.getMessage());
}

// Catch all OverDrive errors
try {
    db.insert("users", Map.of("name", "Alice"));
} catch (OverDriveException e) {
    System.out.println("Database error [" + e.getCode() + "]: " + e.getMessage());
}
```

---

## Security

```java
// Password from environment variable
String password = System.getenv("DB_PASSWORD");
if (password == null) throw new RuntimeException("DB_PASSWORD not set");

OverDrive db = OverDrive.open("secure.odb",
    new OverDrive.OpenOptions().password(password));

// Backup
db.backup("backups/app_2026-04-16.odb");

// WAL cleanup after commit
long txnId = db.beginTransaction();
db.insert("users", Map.of("name", "Alice"));
db.commitTransaction(txnId);
db.cleanupWal();

// Safe queries
List<Map<String, Object>> results = db.querySafe(
    "SELECT * FROM users WHERE name = ?", userInput
);
```

---

## Complete Example

```java
import com.afot.overdrive.OverDrive;
import java.util.*;

public class StoreExample {
    public static void main(String[] args) throws Exception {
        try (OverDrive db = OverDrive.open("store.odb")) {

            // Insert products (table auto-created)
            List<Map<String, Object>> products = List.of(
                Map.of("name", "Laptop",  "price", 999,  "category", "electronics", "stock", 5),
                Map.of("name", "Mouse",   "price", 29,   "category", "electronics", "stock", 50),
                Map.of("name", "Desk",    "price", 299,  "category", "furniture",   "stock", 10),
                Map.of("name", "Chair",   "price", 199,  "category", "furniture",   "stock", 15)
            );
            List<String> ids = db.insertMany("products", products);
            System.out.println("Inserted " + ids.size() + " products");

            // Query
            List<Map<String, Object>> electronics = db.findAll(
                "products", "category = 'electronics'", "price DESC", 0
            );
            System.out.println("Electronics: " + electronics.stream()
                .map(p -> (String) p.get("name"))
                .toList());

            // Count
            int total = db.countWhere("products");
            int expensive = db.countWhere("products", "price > 100");
            System.out.printf("Total: %d, Expensive: %d%n", total, expensive);

            // Update with transaction
            int updated = db.transaction(txn -> {
                int count = db.updateMany("products", "price > 100",
                    Map.of("onSale", true, "discount", 10));
                db.insert("auditLog", Map.of("event", "saleApplied", "affected", count));
                return count;
            });
            System.out.println("Applied sale to " + updated + " products");

            // Watchdog check
            OverDrive.WatchdogReport report = OverDrive.watchdog("store.odb");
            System.out.printf("Database health: %s (%d bytes)%n",
                report.getIntegrityStatus(), report.getFileSizeBytes());

            // Backup
            db.backup("backups/store_backup.odb");
            System.out.println("Backup created!");
        }
    }
}
```
