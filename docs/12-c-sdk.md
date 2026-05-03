# C / C++ SDK — Complete Guide

**Version:** 1.4.0

---

## Setup

### 1. Download the Header

The C header is at `c/include/overdrive.h` in the SDK repository.

### 2. Download the Native Library

| Platform | File |
|----------|------|
| Windows x64 | `overdrive.dll` + `overdrive.dll.lib` |
| Linux x64 | `liboverdrive.so` |
| macOS | `liboverdrive.dylib` |

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

### 3. Compile

```bash
# Linux
gcc -o myapp myapp.c -L./lib -loverdrive -Wl,-rpath,./lib

# macOS
clang -o myapp myapp.c -L./lib -loverdrive

# Windows (MSVC)
cl myapp.c /link /LIBPATH:lib overdrive.dll.lib
```

---

## Memory Rules

> **Critical:** Every `char*` returned by `overdrive_*` functions **must** be freed with `overdrive_free_string()`. The `ODB*` handle must be closed with `overdrive_close()`.

```c
// ✅ Correct
char* result = overdrive_query(db, "SELECT * FROM users");
printf("%s\n", result);
overdrive_free_string(result);  // ← must free!

// ❌ Memory leak
char* result = overdrive_query(db, "SELECT * FROM users");
printf("%s\n", result);
// forgot to free!
```

---

## Basic Usage

```c
#include "overdrive.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Open (or create) a database
    ODB* db = overdrive_open("myapp.odb");
    if (!db) {
        printf("Error: %s\n", overdrive_last_error());
        return 1;
    }

    // Create a table
    if (overdrive_create_table(db, "users") != 0) {
        printf("Error: %s\n", overdrive_last_error());
        overdrive_close(db);
        return 1;
    }

    // Insert a document
    char* id = overdrive_insert(db, "users",
        "{\"name\":\"Alice\",\"age\":30,\"email\":\"alice@example.com\"}");
    if (!id) {
        printf("Insert error: %s\n", overdrive_last_error());
    } else {
        printf("Inserted: %s\n", id);
        overdrive_free_string(id);  // ← free the returned string
    }

    // Query
    char* result = overdrive_query(db,
        "SELECT * FROM users WHERE age > 25");
    if (result) {
        printf("Results: %s\n", result);
        overdrive_free_string(result);  // ← free the returned string
    }

    // Get by ID
    char* user = overdrive_get(db, "users", "users_1");
    if (user) {
        printf("User: %s\n", user);
        overdrive_free_string(user);
    }

    // Update
    int updated = overdrive_update(db, "users", "users_1",
        "{\"age\":31,\"status\":\"active\"}");
    printf("Updated: %d\n", updated);  // 1 = success, 0 = not found, -1 = error

    // Delete
    int deleted = overdrive_delete(db, "users", "users_1");
    printf("Deleted: %d\n", deleted);

    // Count
    int count = overdrive_count(db, "users");
    printf("Count: %d\n", count);

    // Full-text search
    char* matches = overdrive_search(db, "users", "alice");
    if (matches) {
        printf("Search results: %s\n", matches);
        overdrive_free_string(matches);
    }

    // Sync to disk
    overdrive_sync(db);

    // Close
    overdrive_close(db);
    return 0;
}
```

---

## Error Handling

```c
// Check return values
int result = overdrive_create_table(db, "users");
if (result != 0) {
    const char* err = overdrive_last_error();
    fprintf(stderr, "Error: %s\n", err);
    // Note: do NOT free overdrive_last_error() — it's managed internally
}

// Check pointer returns
char* id = overdrive_insert(db, "users", "{\"name\":\"Alice\"}");
if (!id) {
    const char* err = overdrive_last_error();
    fprintf(stderr, "Insert failed: %s\n", err);
}
```

---

## Table Operations

```c
// Create table
int result = overdrive_create_table(db, "users");

// Drop table
int result = overdrive_drop_table(db, "users");

// List tables — returns JSON array string
char* tables = overdrive_list_tables(db);
// → ["users", "products", ...]
printf("Tables: %s\n", tables);
overdrive_free_string(tables);

// Check existence
int exists = overdrive_table_exists(db, "users");
// 1 = exists, 0 = not found, -1 = error
```

---

## Transactions

```c
// Begin transaction
// isolation: 0=ReadUncommitted, 1=ReadCommitted, 2=RepeatableRead, 3=Serializable
uint64_t txn_id = overdrive_begin_transaction(db, 1);  // ReadCommitted
if (txn_id == 0) {
    fprintf(stderr, "Transaction error: %s\n", overdrive_last_error());
    return 1;
}

// Do work
overdrive_insert(db, "users", "{\"name\":\"Alice\"}");
overdrive_insert(db, "logs", "{\"event\":\"user_created\"}");

// Commit
int result = overdrive_commit_transaction(db, txn_id);
if (result != 0) {
    fprintf(stderr, "Commit error: %s\n", overdrive_last_error());
    // Abort instead
    overdrive_abort_transaction(db, txn_id);
}

// Or abort
overdrive_abort_transaction(db, txn_id);
```

---

## Integrity Verification

```c
// Verify database integrity
char* report = overdrive_verify_integrity(db);
if (report) {
    printf("Integrity report: %s\n", report);
    // JSON: {"valid": true, "pages_checked": 42, "tables_verified": 3, "issues": []}
    overdrive_free_string(report);
}
```

---

## Statistics

```c
// Get database statistics
char* stats = overdrive_stats(db);
if (stats) {
    printf("Stats: %s\n", stats);
    // JSON: {"tables": 3, "total_records": 150, "file_size_bytes": 8192, ...}
    overdrive_free_string(stats);
}
```

---

## Version

```c
// Get SDK version — do NOT free this pointer
const char* version = overdrive_version();
printf("OverDrive SDK version: %s\n", version);
```

---

## Complete C Example

```c
#include "overdrive.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void check_error(const char* operation) {
    const char* err = overdrive_last_error();
    if (err) {
        fprintf(stderr, "%s failed: %s\n", operation, err);
        exit(1);
    }
}

int main() {
    printf("OverDrive SDK %s\n", overdrive_version());

    // Open database
    ODB* db = overdrive_open("store.odb");
    if (!db) { check_error("open"); }

    // Create table
    if (overdrive_create_table(db, "products") != 0) {
        check_error("create_table");
    }

    // Insert products
    const char* products[] = {
        "{\"name\":\"Laptop\",\"price\":999,\"category\":\"electronics\",\"stock\":5}",
        "{\"name\":\"Mouse\",\"price\":29,\"category\":\"electronics\",\"stock\":50}",
        "{\"name\":\"Desk\",\"price\":299,\"category\":\"furniture\",\"stock\":10}",
        "{\"name\":\"Chair\",\"price\":199,\"category\":\"furniture\",\"stock\":15}",
    };

    for (int i = 0; i < 4; i++) {
        char* id = overdrive_insert(db, "products", products[i]);
        if (id) {
            printf("Inserted: %s\n", id);
            overdrive_free_string(id);
        }
    }

    // Query electronics
    char* result = overdrive_query(db,
        "SELECT * FROM products WHERE category = 'electronics' ORDER BY price DESC");
    if (result) {
        printf("Electronics: %s\n", result);
        overdrive_free_string(result);
    }

    // Count
    int count = overdrive_count(db, "products");
    printf("Total products: %d\n", count);

    // Update with transaction
    uint64_t txn = overdrive_begin_transaction(db, 1);
    overdrive_query(db, "UPDATE products SET {\"on_sale\":true} WHERE price > 100");
    overdrive_commit_transaction(db, txn);
    printf("Sale applied!\n");

    // Stats
    char* stats = overdrive_stats(db);
    if (stats) {
        printf("Stats: %s\n", stats);
        overdrive_free_string(stats);
    }

    // Sync and close
    overdrive_sync(db);
    overdrive_close(db);
    printf("Done!\n");
    return 0;
}
```

---

## C++ Wrapper Example

```cpp
#include "overdrive.h"
#include <string>
#include <stdexcept>
#include <iostream>

class OverDriveDB {
    ODB* db_;

public:
    explicit OverDriveDB(const std::string& path) {
        db_ = overdrive_open(path.c_str());
        if (!db_) {
            throw std::runtime_error(
                std::string("Failed to open: ") + overdrive_last_error()
            );
        }
    }

    ~OverDriveDB() {
        if (db_) overdrive_close(db_);
    }

    // Non-copyable
    OverDriveDB(const OverDriveDB&) = delete;
    OverDriveDB& operator=(const OverDriveDB&) = delete;

    std::string insert(const std::string& table, const std::string& json) {
        char* id = overdrive_insert(db_, table.c_str(), json.c_str());
        if (!id) throw std::runtime_error(overdrive_last_error());
        std::string result(id);
        overdrive_free_string(id);
        return result;
    }

    std::string query(const std::string& sql) {
        char* result = overdrive_query(db_, sql.c_str());
        if (!result) throw std::runtime_error(overdrive_last_error());
        std::string s(result);
        overdrive_free_string(result);
        return s;
    }

    void createTable(const std::string& name) {
        if (overdrive_create_table(db_, name.c_str()) != 0) {
            throw std::runtime_error(overdrive_last_error());
        }
    }
};

int main() {
    try {
        OverDriveDB db("myapp.odb");
        db.createTable("users");

        std::string id = db.insert("users",
            R"({"name":"Alice","age":30})");
        std::cout << "Inserted: " << id << std::endl;

        std::string results = db.query("SELECT * FROM users");
        std::cout << "Results: " << results << std::endl;

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
    return 0;
}
```

---

## FFI Function Reference

| Function | Returns | Description |
|----------|---------|-------------|
| `overdrive_open(path)` | `ODB*` | Open/create database |
| `overdrive_close(db)` | `void` | Close database |
| `overdrive_sync(db)` | `void` | Flush to disk |
| `overdrive_create_table(db, name)` | `int` | 0=ok, -1=error |
| `overdrive_drop_table(db, name)` | `int` | 0=ok, -1=error |
| `overdrive_list_tables(db)` | `char*` | JSON array (free!) |
| `overdrive_table_exists(db, name)` | `int` | 1=yes, 0=no, -1=error |
| `overdrive_insert(db, table, json)` | `char*` | _id string (free!) |
| `overdrive_get(db, table, id)` | `char*` | JSON doc (free!) or NULL |
| `overdrive_update(db, table, id, json)` | `int` | 1=updated, 0=not found, -1=error |
| `overdrive_delete(db, table, id)` | `int` | 1=deleted, 0=not found, -1=error |
| `overdrive_count(db, table)` | `int` | count or -1=error |
| `overdrive_query(db, sql)` | `char*` | JSON result (free!) |
| `overdrive_search(db, table, text)` | `char*` | JSON array (free!) |
| `overdrive_last_error()` | `const char*` | Error message (do NOT free) |
| `overdrive_free_string(ptr)` | `void` | Free returned strings |
| `overdrive_version()` | `const char*` | Version string (do NOT free) |
| `overdrive_begin_transaction(db, isolation)` | `uint64_t` | txn_id or 0=error |
| `overdrive_commit_transaction(db, txn_id)` | `int` | 0=ok, -1=error |
| `overdrive_abort_transaction(db, txn_id)` | `int` | 0=ok, -1=error |
| `overdrive_verify_integrity(db)` | `char*` | JSON report (free!) |
| `overdrive_stats(db)` | `char*` | JSON stats (free!) |
