# C/C++ Integration Guide

## Overview

OverDrive exposes a **C FFI** (Foreign Function Interface) layer, making it usable from any language that can call C functions — including C, C++, Objective-C, Zig, and more.

## Building the Shared Library

```bash
cd OverDrive-DB_SDK
cargo build --release
```

This produces:

| Platform | Output |
|---|---|
| Windows | `target/release/overdrive.dll` |
| Linux | `target/release/liboverdrive.so` |
| macOS | `target/release/liboverdrive.dylib` |

## Header File

The C header (`overdrive.h`) can be generated with cbindgen:

```bash
cbindgen --config cbindgen.toml --crate overdrive-sdk --output include/overdrive.h
```

Or use the API directly:

```c
// Core types
typedef struct ODB ODB;

// Lifecycle
ODB*        overdrive_open(const char* path);
void        overdrive_close(ODB* db);
void        overdrive_sync(ODB* db);

// Tables
int         overdrive_create_table(ODB* db, const char* name);
int         overdrive_drop_table(ODB* db, const char* name);
char*       overdrive_list_tables(ODB* db);
int         overdrive_table_exists(ODB* db, const char* name);

// CRUD
char*       overdrive_insert(ODB* db, const char* table, const char* json);
char*       overdrive_get(ODB* db, const char* table, const char* id);
int         overdrive_update(ODB* db, const char* table, const char* id, const char* json);
int         overdrive_delete(ODB* db, const char* table, const char* id);
int         overdrive_count(ODB* db, const char* table);

// Query
char*       overdrive_query(ODB* db, const char* sql);
char*       overdrive_search(ODB* db, const char* table, const char* text);

// Error handling
const char* overdrive_last_error(void);
void        overdrive_free_string(char* ptr);
const char* overdrive_version(void);
```

## Memory Management Rules

> ⚠️ **Critical:** Failing to follow these rules will cause memory leaks or crashes.

1. **Every `char*` returned by `overdrive_*` functions must be freed** with `overdrive_free_string()`.
2. **The `ODB*` handle must be closed** with `overdrive_close()`.
3. **String parameters** (`const char*`) are borrowed — the library does NOT take ownership.
4. **`overdrive_last_error()` returns a pointer** that is valid until the next API call — do not free it.

## Return Values

| Return Type | Success | Error |
|---|---|---|
| `ODB*` | Non-NULL pointer | NULL (check `overdrive_last_error()`) |
| `char*` | Non-NULL string (free it!) | NULL (check `overdrive_last_error()`) |
| `int` (for create/drop) | `0` | `-1` |
| `int` (for update/delete) | `1` (found) / `0` (not found) | `-1` |
| `int` (for exists) | `1` (yes) / `0` (no) | `-1` |
| `int` (for count) | Count value | `-1` |

## Complete C Example

```c
#include <stdio.h>
#include <stdlib.h>
#include "overdrive.h"

int main() {
    // Open database
    ODB* db = overdrive_open("myapp.odb");
    if (!db) {
        fprintf(stderr, "Error: %s\n", overdrive_last_error());
        return 1;
    }

    printf("SDK version: %s\n", overdrive_version());

    // Create table
    if (overdrive_create_table(db, "users") != 0) {
        fprintf(stderr, "Create table error: %s\n", overdrive_last_error());
    }

    // Insert documents
    char* id1 = overdrive_insert(db, "users",
        "{\"name\":\"Alice\",\"age\":30,\"email\":\"alice@example.com\"}");
    if (id1) {
        printf("Inserted user: %s\n", id1);
    }

    char* id2 = overdrive_insert(db, "users",
        "{\"name\":\"Bob\",\"age\":25,\"email\":\"bob@example.com\"}");

    // Get by ID
    char* user = overdrive_get(db, "users", id1);
    if (user) {
        printf("Got user: %s\n", user);
        overdrive_free_string(user);
    }

    // Query
    char* result = overdrive_query(db,
        "SELECT * FROM users WHERE age > 20 ORDER BY name");
    if (result) {
        printf("Query result: %s\n", result);
        overdrive_free_string(result);
    }

    // Count
    int count = overdrive_count(db, "users");
    printf("Total users: %d\n", count);

    // Update
    int updated = overdrive_update(db, "users", id1,
        "{\"age\":31}");
    printf("Updated: %s\n", updated == 1 ? "yes" : "no");

    // Search
    char* matches = overdrive_search(db, "users", "alice");
    if (matches) {
        printf("Search results: %s\n", matches);
        overdrive_free_string(matches);
    }

    // List tables
    char* tables = overdrive_list_tables(db);
    if (tables) {
        printf("Tables: %s\n", tables);
        overdrive_free_string(tables);
    }

    // Cleanup
    overdrive_free_string(id1);
    overdrive_free_string(id2);
    overdrive_close(db);

    return 0;
}
```

## Compiling

### GCC (Linux)

```bash
gcc -o myapp myapp.c -L./target/release -loverdrive -Wl,-rpath,./target/release
```

### MSVC (Windows)

```bash
cl myapp.c /I include /link target\release\overdrive.dll.lib
```

### CMake

```cmake
add_executable(myapp main.c)
target_link_libraries(myapp ${CMAKE_SOURCE_DIR}/target/release/overdrive.dll)
target_include_directories(myapp PRIVATE ${CMAKE_SOURCE_DIR}/include)
```

## C++ Wrapper (Optional)

```cpp
#include <string>
#include <vector>
#include <stdexcept>
#include "overdrive.h"

class OverDrive {
    ODB* db_;
public:
    OverDrive(const std::string& path) {
        db_ = overdrive_open(path.c_str());
        if (!db_) throw std::runtime_error(overdrive_last_error());
    }

    ~OverDrive() {
        if (db_) overdrive_close(db_);
    }

    void createTable(const std::string& name) {
        if (overdrive_create_table(db_, name.c_str()) != 0)
            throw std::runtime_error(overdrive_last_error());
    }

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

    // ... more methods following the same pattern
};
```
