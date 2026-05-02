# OverDrive-DB InCode SDK — Build Plan v1.5.0
> Built directly from OverDrive-DB server FFI layer (src/ffi.rs v1.4.6)

---

## Architecture

```
x:\OverDrive-DB (server)
    └── src/ffi.rs  ← single source of truth
            ↓  cargo build --features ffi --release
    target/release/overdrive_db.dll
            ↓  copy
x:\OverDrive-DB\IncodeSDK\
    lib/
        windows/  overdrive.dll
        linux/    liboverdrive.so
        macos/    liboverdrive.dylib
    rust/
    nodejs/
    go/
    java/
    python/
```

**Key rule**: No network downloads. No pre-built binaries from GitHub.
The `.dll` is always built from the server source → zero version drift.

---

## FFI Functions (from ffi.rs)

### Core
| Symbol | Description |
|---|---|
| `overdrive_open(path)` | Open/create `.odb` file |
| `overdrive_open_with_engine(path, engine, opts_json)` | Open with engine + options |
| `overdrive_close(handle)` | Close and free |
| `overdrive_sync(handle)` | Flush to disk |
| `overdrive_version()` | Static version string |
| `overdrive_free_string(ptr)` | Free any returned string |
| `overdrive_last_error()` | Last error message |
| `overdrive_last_error_json()` | Last error as JSON |

### Tables
| Symbol | Description |
|---|---|
| `overdrive_create_table(handle, name)` | Create table |
| `overdrive_drop_table(handle, name)` | Drop table |
| `overdrive_list_tables(handle)` | → JSON array |
| `overdrive_table_exists(handle, name)` | → 1/0/-1 |

### CRUD ✅ All confirmed working
| Symbol | Description |
|---|---|
| `overdrive_insert(handle, table, json)` | Insert doc → returns `_id` |
| `overdrive_get(handle, table, id)` | Get doc by `_id` → JSON |
| `overdrive_update(handle, table, id, json)` | Update → 1/0/-1 |
| `overdrive_delete(handle, table, id)` | Delete → 1/0/-1 |
| `overdrive_count(handle, table)` | Count docs |

### Transactions (MVCC)
| Symbol | Description |
|---|---|
| `overdrive_begin_transaction(handle, iso)` | Start txn → txn_id |
| `overdrive_commit_transaction(handle, txn_id)` | Commit |
| `overdrive_abort_transaction(handle, txn_id)` | Rollback |

### Other
| Symbol | Description |
|---|---|
| `overdrive_query(handle, sql)` | SQL query (⚠️ SELECT broken — fix needed) |
| `overdrive_search(handle, table, text)` | Full-text search |
| `overdrive_verify_integrity(handle)` | Integrity report JSON |

---

## Repo Structure

```
IncodeSDK/
├── plan.md                      ← this file
├── README.md
├── .gitignore
├── .github/
│   └── workflows/
│       └── ci.yml               ← build dll + run all e2e tests
│
├── scripts/
│   └── build-native.ps1         ← build overdrive.dll from server
│
├── lib/                         ← bundled native binaries
│   ├── windows/overdrive.dll
│   ├── linux/liboverdrive.so
│   └── macos/liboverdrive.dylib
│
├── rust/                        ← Rust SDK
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── lib.rs
│       ├── ffi.rs
│       └── error.rs
│
├── nodejs/                      ← Node.js SDK
│   ├── package.json
│   ├── index.js
│   └── test/e2e.js
│
├── go/                          ← Go SDK
│   ├── go.mod
│   ├── overdrive.go
│   └── overdrive_e2e_test.go
│
├── java/                        ← Java SDK
│   ├── pom.xml
│   └── src/
│       ├── main/java/com/afot/overdrive/
│       │   ├── OverDrive.java
│       │   └── NativeLoader.java
│       └── test/java/com/afot/overdrive/
│           └── OverDriveE2ETest.java
│
└── python/                      ← Python SDK
    ├── pyproject.toml
    └── overdrive/
        ├── __init__.py
        └── _native.py
```

---

## SDK APIs

### Rust
```rust
let mut db = OverDrive::open("app.odb")?;
db.create_table("users")?;
let id = db.insert("users", json!({"name":"Alice"}))?;
let doc = db.get("users", &id)?;
db.update("users", &id, json!({"age": 30}))?;
db.delete("users", &id)?;
let n = db.count("users")?;

// Transaction
let txn = db.begin_transaction(IsolationLevel::ReadCommitted)?;
db.insert("users", json!({"name":"Bob"}))?;
db.commit_transaction(&txn)?;
```

### Node.js
```js
const db = OverDrive.open('app.odb');
db.createTable('users');
const id = db.insert('users', { name: 'Alice' });
const doc = db.get('users', id);
db.update('users', id, { age: 30 });
db.delete('users', id);
const n = db.count('users');
db.close();
```

### Go
```go
db, _ := overdrive.Open("app.odb")
defer db.Close()
db.CreateTable("users")
id, _ := db.Insert("users", map[string]any{"name":"Alice"})
doc, _ := db.Get("users", id)
db.Update("users", id, map[string]any{"age": 30})
db.Delete("users", id)
n, _ := db.Count("users")
```

### Java
```java
try (OverDrive db = OverDrive.open("app.odb")) {
    db.createTable("users");
    String id = db.insert("users", Map.of("name", "Alice"));
    Map<String,Object> doc = db.get("users", id);
    db.update("users", id, Map.of("age", 30));
    db.delete("users", id);
    long n = db.count("users");
}
```

### Python
```python
with OverDrive.open("app.odb") as db:
    db.create_table("users")
    id = db.insert("users", {"name": "Alice"})
    doc = db.get("users", id)
    db.update("users", id, {"age": 30})
    db.delete("users", id)
    n = db.count("users")
```

---

## E2E Tests (same 10 tests for ALL languages)

| # | Test | Verifies |
|---|---|---|
| 1 | `open()` creates `.odb` file | Native lib loads |
| 2 | `insert()` + `get()` roundtrip | CRUD works |
| 3 | `count()` is accurate | 0→3 after 3 inserts |
| 4 | Multiple `get()` calls | Per-doc field accuracy |
| 5 | `update()` changes field | Verified by `get()` |
| 6 | `delete()` removes doc | Count drops, `get()` → null |
| 7 | Data persists after close+reopen | `count()` + `get()` by `_id` |
| 8 | `insert_batch()` | Correct IDs, count matches |
| 9 | `table_exists()` | Correct bool |
| 10 | `version()` | Non-empty, non-"unknown" |

> ⚠️ `overdrive_query` SQL SELECT not tested — returns empty rows (server bug).
> All tests use direct FFI: insert/get/count/update/delete only.

---

## Versions (unified)

| SDK | Version |
|---|---|
| Rust `Cargo.toml` | `1.5.0` |
| Node.js `package.json` | `1.5.0` |
| Go module tag | `v1.5.0` |
| Java `pom.xml` | `1.5.0` |
| Python `pyproject.toml` | `1.5.0` |
| Native lib (overdrive.dll) | Built from server `1.4.6` |

---

## Build Order

```
Step 1: scripts/build-native.ps1
        → builds overdrive.dll from x:\OverDrive-DB --features ffi

Step 2: rust/
        → cargo test --test e2e (10/10 must pass)

Step 3: nodejs/
        → node test/e2e.js (10/10 must pass)

Step 4: go/
        → go test -v -tags e2e . (10/10 must pass)

Step 5: java/
        → mvn test -Dtest=OverDriveE2ETest (10/10 must pass)

Step 6: python/
        → pytest tests/test_e2e.py (10/10 must pass)

Step 7: git tag v1.5.0 && git push origin v1.5.0
```

---

## CI/CD (.github/workflows/ci.yml)

```yaml
name: Build & Test
on: [push, pull_request]
jobs:
  windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: ALL-FOR-ONE-TECH/OverDrive-DB
          path: server
      - run: cd server && cargo build --features ffi --release
      - run: Copy-Item server/target/release/overdrive_db.dll lib/windows/overdrive.dll
      - run: cd rust && cargo test --test e2e -- --nocapture
      - run: node nodejs/test/e2e.js
      - run: cd go && go test -v -tags e2e .
      - run: cd java && mvn test -Dtest=OverDriveE2ETest
      - run: cd python && pip install -e . && pytest tests/
```

---

## Known Issues / Notes

| Issue | Status | Notes |
|---|---|---|
| `overdrive_query` SQL SELECT returns empty | ⚠️ Known bug | Root cause: Shell creates new DB connection, doesn't share handle state |
| Windows file permissions warning | ℹ️ Non-critical | `icacls` fails in some CI environments — safe to ignore |
| `overdrive_version()` returns `1.4.4` (hardcoded) | 🔧 Fix needed | Update `c"1.4.4"` → `c"1.4.6"` in server ffi.rs |

---

## Next Steps

1. **Approve this plan** → start implementation
2. **Fix `overdrive_version()`** in server `ffi.rs` line 656: `c"1.4.4"` → `c"1.4.6"`
3. **Fix `overdrive_query`** SQL SELECT — or document it as unsupported
4. Decide: **which package registries** to publish to (npm, PyPI, crates.io, Maven, Go proxy)
