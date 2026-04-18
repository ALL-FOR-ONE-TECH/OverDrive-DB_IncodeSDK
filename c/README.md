# OverDrive InCode SDK — C / C++ (v1.4.0)

**Embeddable hybrid SQL+NoSQL document database. Like SQLite for JSON.**

---

## Install

1. Download the native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest)
2. Copy `include/overdrive.h` to your include path
3. Link against the library

| Platform | Library File |
|----------|-------------|
| Windows x64 | `overdrive.dll` + `overdrive.dll.lib` |
| Linux x64 | `liboverdrive.so` |
| Linux ARM64 | `liboverdrive-arm64.so` |
| macOS x64 | `liboverdrive.dylib` |
| macOS ARM64 | `liboverdrive-arm64.dylib` |

---

## Compile

```bash
# Linux
gcc -o myapp myapp.c -I./include -L./lib -loverdrive -lm -ldl -lpthread

# macOS
clang -o myapp myapp.c -I./include -L./lib -loverdrive

# Windows (MSVC)
cl /I include myapp.c /link /LIBPATH:lib overdrive.lib

# Windows (MinGW)
gcc -o myapp.exe myapp.c -I./include -L./lib -loverdrive
```

### CMake

```cmake
cmake_minimum_required(VERSION 3.16)
project(myapp C)

add_executable(myapp main.c)
target_include_directories(myapp PRIVATE ${CMAKE_SOURCE_DIR}/include)
target_link_directories(myapp PRIVATE ${CMAKE_SOURCE_DIR}/lib)
target_link_libraries(myapp overdrive)
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

**Do NOT free:**
- `overdrive_last_error()` — static string
- `overdrive_get_error_details()` — static string
- `overdrive_version()` — static string

---

## Quick Start

```c
#include "overdrive.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Open or create a database
    ODB* db = overdrive_open("myapp.odb");
    if (!db) {
        printf("Error: %s\n", overdrive_last_error());
        return 1;
    }

    // Create a table
    overdrive_create_table(db, "users");

    // Insert a document — returns auto-generated _id
    char* id = overdrive_insert(db, "users",
        "{\"name\":\"Alice\",\"age\":30,\"email\":\"alice@example.com\"}");
    printf("Inserted: %s\n", id);  // "users_1"
    overdrive_free_string(id);

    // SQL query
    char* result = overdrive_query(db,
        "SELECT * FROM users WHERE age > 25 ORDER BY name");
    printf("Results: %s\n", result);
    overdrive_free_string(result);

    // Get by ID
    char* user = overdrive_get(db, "users", "users_1");
    if (user) {
        printf("User: %s\n", user);
        overdrive_free_string(user);
    }

    // Update
    int updated = overdrive_update(db, "users", "users_1",
        "{\"age\":31,\"status\":\"active\"}");
    printf("Updated: %d\n", updated);  // 1

    // Count
    int count = overdrive_count(db, "users");
    printf("Count: %d\n", count);

    // Sync and close
    overdrive_sync(db);
    overdrive_close(db);
    return 0;
}
```

---

## v1.4 Features

### Password-Protected Database

```c
// Open with password encryption (Argon2id key derivation)
ODB* db = overdrive_open_with_engine("secure.odb", "Disk",
    "{\"password\":\"my-secret-pass-123\",\"auto_create_tables\":true}");
if (!db) {
    printf("Error: %s\n", overdrive_last_error());
    return 1;
}
```

### RAM Engine

```c
// Full RAM database — sub-microsecond reads
ODB* cache = overdrive_open_with_engine("cache.odb", "RAM",
    "{\"auto_create_tables\":true}");

overdrive_insert(cache, "sessions", "{\"user_id\":123,\"token\":\"abc\"}");

// Persist to disk
overdrive_snapshot(cache, "./backup/cache.odb");

// Check memory usage
char* usage = overdrive_memory_usage(cache);
printf("Memory: %s\n", usage);  // {"bytes":N,"mb":N.N,"limit_bytes":N,"percent":N.N}
overdrive_free_string(usage);

// Restore from snapshot
overdrive_restore(cache, "./backup/cache.odb");

overdrive_close(cache);
```

### Watchdog — File Integrity

```c
// Check file integrity without opening the database
char* report = overdrive_watchdog("myapp.odb");
printf("Integrity: %s\n", report);
// {"file_path":"myapp.odb","file_size_bytes":8192,"integrity_status":"valid",...}
overdrive_free_string(report);
```

### Transactions

```c
// Begin transaction (1 = ReadCommitted)
uint64_t txn_id = overdrive_begin_transaction(db, 1);
if (txn_id == 0) {
    printf("Error: %s\n", overdrive_last_error());
    return 1;
}

// Do work
overdrive_insert(db, "orders", "{\"item\":\"widget\",\"qty\":1}");
overdrive_insert(db, "logs", "{\"event\":\"order_created\"}");

// Commit
if (overdrive_commit_transaction(db, txn_id) != 0) {
    printf("Commit error: %s\n", overdrive_last_error());
    overdrive_abort_transaction(db, txn_id);
}
```

### Auto-Table Creation

```c
// Tables are auto-created on first insert by default
ODB* db = overdrive_open("myapp.odb");
// No need to call overdrive_create_table() first!
char* id = overdrive_insert(db, "users", "{\"name\":\"Alice\"}");
overdrive_free_string(id);

// Disable auto-creation for strict mode
overdrive_set_auto_create_tables(db, 0);
```

### Structured Error Codes

```c
ODB* db = overdrive_open_with_engine("secure.odb", "Disk",
    "{\"password\":\"wrong\"}");
if (!db) {
    // Get structured error details
    const char* details = overdrive_get_error_details();
    if (details) {
        printf("Error details: %s\n", details);
        // {"code":"ODB-AUTH-001","message":"Incorrect password","suggestions":[...]}
    } else {
        printf("Error: %s\n", overdrive_last_error());
    }
}
```

---

## Complete Example

```c
#include "overdrive.h"
#include <stdio.h>
#include <stdlib.h>

void check(int result, const char* op) {
    if (result != 0) {
        fprintf(stderr, "%s failed: %s\n", op, overdrive_last_error());
        exit(1);
    }
}

int main() {
    printf("OverDrive SDK %s\n", overdrive_version());

    // Open database
    ODB* db = overdrive_open("store.odb");
    if (!db) { fprintf(stderr, "Open failed: %s\n", overdrive_last_error()); return 1; }

    // Create table
    check(overdrive_create_table(db, "products"), "create_table");

    // Insert products
    const char* products[] = {
        "{\"name\":\"Laptop\",\"price\":999,\"category\":\"electronics\"}",
        "{\"name\":\"Mouse\",\"price\":29,\"category\":\"electronics\"}",
        "{\"name\":\"Desk\",\"price\":299,\"category\":\"furniture\"}",
    };
    for (int i = 0; i < 3; i++) {
        char* id = overdrive_insert(db, "products", products[i]);
        if (id) { printf("Inserted: %s\n", id); overdrive_free_string(id); }
    }

    // Query
    char* result = overdrive_query(db,
        "SELECT * FROM products WHERE price > 100 ORDER BY price DESC");
    if (result) { printf("Expensive: %s\n", result); overdrive_free_string(result); }

    // Count
    printf("Total: %d\n", overdrive_count(db, "products"));

    // Transaction
    uint64_t txn = overdrive_begin_transaction(db, 1);
    overdrive_query(db, "UPDATE products SET {\"on_sale\":true} WHERE price > 100");
    overdrive_commit_transaction(db, txn);

    // Watchdog check
    char* report = overdrive_watchdog("store.odb");
    if (report) { printf("Health: %s\n", report); overdrive_free_string(report); }

    // Stats
    char* stats = overdrive_stats(db);
    if (stats) { printf("Stats: %s\n", stats); overdrive_free_string(stats); }

    overdrive_sync(db);
    overdrive_close(db);
    printf("Done!\n");
    return 0;
}
```

---

## C++ Wrapper

```cpp
#include "overdrive.h"
#include <string>
#include <stdexcept>

class OverDriveDB {
    ODB* db_;
public:
    explicit OverDriveDB(const std::string& path) {
        db_ = overdrive_open(path.c_str());
        if (!db_) throw std::runtime_error(overdrive_last_error());
    }
    ~OverDriveDB() { if (db_) overdrive_close(db_); }

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
        char* r = overdrive_query(db_, sql.c_str());
        if (!r) throw std::runtime_error(overdrive_last_error());
        std::string result(r);
        overdrive_free_string(r);
        return result;
    }

    void createTable(const std::string& name) {
        if (overdrive_create_table(db_, name.c_str()) != 0)
            throw std::runtime_error(overdrive_last_error());
    }
};

int main() {
    try {
        OverDriveDB db("myapp.odb");
        db.createTable("users");
        std::string id = db.insert("users", R"({"name":"Alice","age":30})");
        std::string results = db.query("SELECT * FROM users");
        printf("ID: %s\nResults: %s\n", id.c_str(), results.c_str());
    } catch (const std::exception& e) {
        fprintf(stderr, "Error: %s\n", e.what());
        return 1;
    }
    return 0;
}
```

---

## API Reference

| Function | Returns | Description |
|----------|---------|-------------|
| `overdrive_open(path)` | `ODB*` | Open/create database |
| `overdrive_open_with_engine(path, engine, opts)` | `ODB*` | Open with engine + options (v1.4) |
| `overdrive_close(db)` | `void` | Close database |
| `overdrive_sync(db)` | `void` | Flush to disk |
| `overdrive_version()` | `const char*` | SDK version (do not free) |
| `overdrive_create_table(db, name)` | `int` | 0=ok, -1=error |
| `overdrive_create_table_with_engine(db, name, engine)` | `int` | 0=ok, -1=error (v1.4) |
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
| `overdrive_begin_transaction(db, isolation)` | `uint64_t` | txn_id or 0=error |
| `overdrive_commit_transaction(db, txn_id)` | `int` | 0=ok, -1=error |
| `overdrive_abort_transaction(db, txn_id)` | `int` | 0=ok, -1=error |
| `overdrive_snapshot(db, path)` | `int` | 0=ok, -1=error (v1.4) |
| `overdrive_restore(db, path)` | `int` | 0=ok, -1=error (v1.4) |
| `overdrive_memory_usage(db)` | `char*` | JSON stats (free!) (v1.4) |
| `overdrive_set_memory_limit(db, bytes)` | `int` | 0=ok, -1=error (v1.4) |
| `overdrive_watchdog(path)` | `char*` | JSON report (free!) (v1.4) |
| `overdrive_verify_integrity(db)` | `char*` | JSON report (free!) |
| `overdrive_stats(db)` | `char*` | JSON stats (free!) |
| `overdrive_last_error()` | `const char*` | Error message (do not free) |
| `overdrive_get_error_details()` | `const char*` | JSON error (do not free) (v1.4) |
| `overdrive_free_string(ptr)` | `void` | Free returned strings |
| `overdrive_set_auto_create_tables(db, enabled)` | `int` | 0=ok, -1=error (v1.4) |

---

## Links

- **GitHub:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK
- **Releases:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases
- **Issues:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues

## License

MIT / Apache-2.0
