# OverDrive-DB InCode SDK — Complete Build Plan v1.5.0

---

## Native Binary Distribution (Best Structure)

### ❌ Old Approach (What We Had)
```
lib/
    windows/overdrive.dll       ← one platform, manual copy
```

### ✅ New Approach — Platform-Arch Directories
```
lib/
    windows-x64/
        overdrive.dll
    linux-x64/
        liboverdrive.so
    linux-arm64/
        liboverdrive.so
    macos-x64/
        liboverdrive.dylib
    macos-arm64/
        liboverdrive.dylib
```

**Why this is better:**
- Auto-detect at runtime: `{os}-{arch}` → correct binary
- CI can build each platform independently and drop into the right folder
- Each SDK uses the same resolution logic: `lib/{os}-{arch}/overdrive.{ext}`
- Works for cross-compilation (build Windows dll on Linux CI)
- Matches what `esbuild`, `@napi-rs`, and `better-sqlite3` all do

### Library Filename per OS
| Platform | Filename | Extension |
|---|---|---|
| Windows x64 | `overdrive.dll` | `.dll` |
| Linux x64 | `liboverdrive.so` | `.so` |
| Linux ARM64 | `liboverdrive.so` | `.so` |
| macOS x64 | `liboverdrive.dylib` | `.dylib` |
| macOS ARM64 | `liboverdrive.dylib` | `.dylib` |

### Runtime Resolution (same logic in ALL SDKs)
```
1. Check env var: OVERDRIVE_LIB_PATH (user override, highest priority)
2. Check: lib/{os}-{arch}/overdrive.{ext}   (bundled, default)
3. Check: executable directory              (for Rust cargo test)
4. Check: system PATH                       (installed globally)
5. FAIL with clear error message
```

---

## Full Feature Set

### Core Features
| Feature | Description |
|---|---|
| Zero-config | Open a file, start querying — no setup needed |
| JSON Native | Store, query, and index JSON documents |
| SQL Queries | SELECT, INSERT, UPDATE, DELETE, WHERE, ORDER BY, LIMIT |
| Aggregations | COUNT, SUM, AVG, MIN, MAX, GROUP BY |
| Full-text Search | Built-in text search across all documents |
| B-Tree Indexes | Secondary indexes for fast lookups |
| ACID Transactions | MVCC with 4 isolation levels |
| Encryption | AES-256-GCM via Argon2id key derivation |
| RAM Engine | Sub-microsecond in-memory storage with snapshot/restore |
| Watchdog | File integrity monitoring |
| Cross-platform | Windows x64, Linux x64/ARM64, macOS x64/ARM64 |

### 6 Storage Engines
| Engine | Use Case | Latency |
|---|---|---|
| Disk (default) | General-purpose persistent storage | ~1ms |
| RAM | Caching, sessions, leaderboards | <1µs |
| Vector | Similarity search, embeddings | ~5ms |
| Time-Series | Metrics, IoT, logs | ~2ms |
| Graph | Social networks, knowledge graphs | ~3ms |
| Streaming | Event queues, message brokers | ~1ms |

---

## Complete API Reference

### Database Lifecycle
| Operation | Rust | Node.js | Java | Go | Python |
|---|---|---|---|---|---|
| Open | `OverDriveDB::open(path)` | `OverDrive.open(path)` | `OverDrive.open(path)` | `Open(path)` | `OverDrive.open(path)` |
| Open (engine) | `open_with_options(path, opts)` | `open(path, {engine})` | `open(path, "RAM")` | `Open(path, WithEngine("RAM"))` | `open(path, engine="RAM")` |
| Open (encrypted) | `open_encrypted(path, "ODB_KEY")` | `openEncrypted(path, "ODB_KEY")` | `openEncrypted(path, "ODB_KEY")` | `OpenEncrypted(path, "ODB_KEY")` | `open_encrypted(path, "ODB_KEY")` |
| Open (password) | `open_with_options(path, pwd)` | `open(path, {password})` | `open(path, password)` | `Open(path, WithPassword(pwd))` | `open(path, password=pwd)` |
| Close | `db.close()` | `db.close()` | `db.close()` | `db.Close()` | `db.close()` |
| Sync | `db.sync()` | `db.sync()` | `db.sync()` | `db.Sync()` | `db.sync()` |
| Version | `OverDriveDB::version()` | `OverDrive.version()` | `OverDrive.version()` | `Version()` | `OverDrive.version()` |
| Watchdog | `watchdog(path)` | `OverDrive.watchdog(path)` | `OverDrive.watchdog(path)` | `Watchdog(path)` | `OverDrive.watchdog(path)` |

### Table Operations
| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Create | `db.create_table(name)` | `db.createTable(name)` | `db.CreateTable(name)` | `db.create_table(name)` |
| Drop | `db.drop_table(name)` | `db.dropTable(name)` | `db.DropTable(name)` | `db.drop_table(name)` |
| List | `db.list_tables()` | `db.listTables()` | `db.ListTables()` | `db.list_tables()` |
| Exists | `db.table_exists(name)` | `db.tableExists(name)` | `db.TableExists(name)` | `db.table_exists(name)` |
| Create (engine) | `db.create_table_with_engine(name, "RAM")` | `db.createTable(name, {engine:"RAM"})` | `db.CreateTable(name, WithTableEngine("RAM"))` | `db.create_table(name, engine="RAM")` |

### CRUD Operations
| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Insert | `db.insert(table, &doc)` | `db.insert(table, doc)` | `db.Insert(table, doc)` | `db.insert(table, doc)` |
| Insert batch | `db.insert_batch(table, &docs)` | `db.insertMany(table, docs)` | `db.InsertBatch(table, docs)` | `db.insert_many(table, docs)` |
| Get | `db.get(table, id)` | `db.get(table, id)` | `db.Get(table, id)` | `db.get(table, id)` |
| Update | `db.update(table, id, &patch)` | `db.update(table, id, patch)` | `db.Update(table, id, patch)` | `db.update(table, id, patch)` |
| Delete | `db.delete(table, id)` | `db.delete(table, id)` | `db.Delete(table, id)` | `db.delete(table, id)` |
| Count | `db.count(table)` | `db.count(table)` | `db.Count(table)` | `db.count(table)` |

### Query & Search
| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| SQL Query | `db.query(sql)` | `db.query(sql)` | `db.Query(sql)` | `db.query(sql)` |
| Safe Query | `db.query_safe(sql, &params)` | `db.querySafe(sql, params)` | `db.QuerySafe(sql, params...)` | `db.query_safe(sql, params)` |
| Search | `db.search(table, text)` | `db.search(table, text)` | `db.Search(table, text)` | `db.search(table, text)` |
| Find one | `db.find_one(table, where)` | `db.findOne(table, where)` | `db.FindOne(table, where)` | `db.find_one(table, where)` |
| Find all | `db.find_all(table, where, order, limit)` | `db.findAll(table, ...)` | `db.FindAll(table, ...)` | `db.find_all(table, ...)` |
| Count where | `db.count_where(table, where)` | `db.countWhere(table, where)` | `db.CountWhere(table, where)` | `db.count_where(table, where)` |
| Exists | `db.exists(table, id)` | `db.exists(table, id)` | `db.Exists(table, id)` | `db.exists(table, id)` |
| Update many | `db.update_many(table, where, patch)` | `db.updateMany(table, where, patch)` | `db.UpdateMany(table, where, patch)` | `db.update_many(table, where, patch)` |
| Delete many | `db.delete_many(table, where)` | `db.deleteMany(table, where)` | `db.DeleteMany(table, where)` | `db.delete_many(table, where)` |

### Transactions (MVCC)
| Level | Value | Description |
|---|---|---|
| Read Uncommitted | 0 | Fastest, least safe |
| Read Committed | 1 | Default |
| Repeatable Read | 2 | Snapshot isolation |
| Serializable | 3 | Full isolation |

| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Begin | `db.begin_transaction(iso)` | `db.beginTransaction(iso)` | `db.BeginTransaction(iso)` | `db.begin_transaction(iso)` |
| Commit | `db.commit_transaction(&txn)` | `db.commitTransaction(id)` | `db.CommitTransaction(id)` | `db.commit_transaction(id)` |
| Abort | `db.abort_transaction(&txn)` | `db.abortTransaction(id)` | `db.AbortTransaction(id)` | `db.abort_transaction(id)` |
| Callback | `db.transaction(\|txn\| {...})` | `db.transaction(fn)` | `db.Transaction(fn, iso)` | `db.transaction(fn)` |
| Retry | `db.transaction_with_retry(fn, n)` | `db.transactionWithRetry(fn, n)` | `db.TransactionWithRetry(fn, n)` | `db.transaction_with_retry(fn, n)` |

### Backup & Integrity
| Operation | Rust | Node.js / Java | Go | Python |
|---|---|---|---|---|
| Backup | `db.backup(dest)` | `db.backup(dest)` | `db.Backup(dest)` | `db.backup(dest)` |
| Snapshot | `db.snapshot(path)` | `db.snapshot(path)` | `db.Snapshot(path)` | `db.snapshot(path)` |
| Restore | `db.restore(path)` | `db.restore(path)` | `db.Restore(path)` | `db.restore(path)` |
| Cleanup WAL | `db.cleanup_wal()` | `db.cleanupWal()` | `db.CleanupWal()` | `db.cleanup_wal()` |
| Memory usage | `db.memory_usage()` | `db.memoryUsage()` | `db.MemoryUsageStats()` | `db.memory_usage()` |

### Security
| Feature | Usage |
|---|---|
| Password encryption | `open(path, password=...)` — AES-256-GCM via Argon2id |
| Env var key | `open_encrypted(path, "ODB_KEY")` — key from environment |
| Parameterized queries | `query_safe(sql, params)` — blocks SQL injection |
| Encrypted backup | `backup(dest)` — syncs + copies + hardens file permissions |
| WAL cleanup | `cleanup_wal()` — removes WAL replay-attack surface |
| File permissions | Auto chmod 600 (Linux/Mac) or Windows ACL on open |

### Error Codes
| Code | Type | When |
|---|---|---|
| `ODB-AUTH-*` | Authentication | Wrong password, key too short |
| `ODB-TABLE-*` | Table | Not found, already exists |
| `ODB-QUERY-*` | Query | SQL syntax error |
| `ODB-TXN-*` | Transaction | Deadlock, conflict |
| `ODB-IO-*` | I/O | File not found, corrupted |
| `ODB-FFI-*` | FFI | Native library not found |

---

## Repo Structure

```
IncodeSDK/
├── plan.md                         ← this file
├── README.md
├── .gitignore
│
├── scripts/
│   ├── build-native.ps1            ← Windows: cargo build --features ffi
│   └── build-native.sh             ← Linux/macOS: cargo build --features ffi
│
├── lib/                            ← native binaries (built by CI, committed)
│   ├── windows-x64/
│   │   └── overdrive.dll
│   ├── linux-x64/
│   │   └── liboverdrive.so
│   ├── linux-arm64/
│   │   └── liboverdrive.so
│   ├── macos-x64/
│   │   └── liboverdrive.dylib
│   └── macos-arm64/
│       └── liboverdrive.dylib
│
├── rust/                           ← Rust SDK (crates.io: overdrive-sdk)
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── lib.rs
│       ├── ffi.rs
│       ├── error.rs
│       └── engines.rs
│
├── nodejs/                         ← Node.js SDK (npm: overdrive-db)
│   ├── package.json
│   ├── index.js
│   ├── lib.js                      ← native loader (platform-arch detection)
│   └── test/
│       ├── basic.js                ← API surface test
│       └── e2e.js                  ← real .odb tests
│
├── go/                             ← Go SDK (module: overdrive-db-go)
│   ├── go.mod
│   ├── overdrive.go
│   ├── overdrive_windows.go
│   ├── overdrive_linux.go
│   ├── overdrive_darwin.go
│   └── overdrive_e2e_test.go
│
├── java/                           ← Java SDK (Maven: com.afot:overdrive-db)
│   ├── pom.xml
│   └── src/
│       ├── main/java/com/afot/overdrive/
│       │   ├── OverDrive.java
│       │   └── NativeLoader.java
│       └── test/java/com/afot/overdrive/
│           └── OverDriveE2ETest.java
│
└── python/                         ← Python SDK (PyPI: overdrive-db)
    ├── pyproject.toml
    └── overdrive/
        ├── __init__.py
        ├── _native.py              ← ctypes bindings
        └── tests/
            └── test_e2e.py
```

---

## Native Library Resolution (all SDKs same logic)

```
Priority 1: $OVERDRIVE_LIB_PATH       (user override)
Priority 2: lib/{os}-{arch}/overdrive.{ext}    (bundled — default)
Priority 3: <executable directory>/overdrive.{ext}    (cargo test path)
Priority 4: PATH / LD_LIBRARY_PATH / DYLD_LIBRARY_PATH (system install)
→ FAIL: clear error "native library not found, set OVERDRIVE_LIB_PATH"
```

| Platform detection | Value |
|---|---|
| `std::env::consts::OS` / `process.platform` / `runtime.GOOS` | `windows`, `linux`, `darwin` |
| `std::env::consts::ARCH` / `process.arch` / `runtime.GOARCH` | `x86_64`→`x64`, `aarch64`→`arm64` |

---

## Build Scripts

### `scripts/build-native.ps1` (Windows)
```powershell
$SERVER = "x:\OverDrive-DB"
Set-Location $SERVER
cargo build --features ffi --release
$OUT = "$PSScriptRoot\..\lib\windows-x64\overdrive.dll"
Copy-Item "target\release\overdrive_db.dll" $OUT -Force
Write-Host "✅ Built overdrive.dll → $OUT"
```

### `scripts/build-native.sh` (Linux/macOS)
```bash
#!/bin/bash
SERVER="$(dirname "$0")/../../OverDrive-DB"
cd "$SERVER"
cargo build --features ffi --release
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m | sed 's/x86_64/x64/' | sed 's/aarch64/arm64/')
OUT="$(dirname "$0")/../lib/${OS}-${ARCH}"
mkdir -p "$OUT"
if [ "$OS" = "darwin" ]; then
  cp target/release/liboverdrive_db.dylib "$OUT/liboverdrive.dylib"
else
  cp target/release/liboverdrive_db.so "$OUT/liboverdrive.so"
fi
echo "✅ Built native lib → $OUT"
```

---

## E2E Tests (10 tests per SDK — same across all languages)

| # | Test | Method Used |
|---|---|---|
| 1 | `open()` creates `.odb` file on disk | `open` |
| 2 | `insert()` + `get()` roundtrip | `insert`, `get` |
| 3 | `count()` accurate (0→3 after 3 inserts) | `count` |
| 4 | Multiple `get()` return correct fields | `get` × 3 |
| 5 | `update()` changes field verified by `get()` | `update`, `get` |
| 6 | `delete()` removes doc, count drops | `delete`, `count`, `get` |
| 7 | Data persists after close + reopen | `close`, `open`, `count`, `get` |
| 8 | `insert_batch()` / `insertMany()` correct | `insert_batch`, `count`, `get` |
| 9 | `table_exists()` returns correct bool | `table_exists` |
| 10 | `version()` returns non-empty string | `version` |

> ⚠️ `overdrive_query` SQL SELECT currently returns empty rows (server bug).
> Tests use direct CRUD FFI only. SQL will be added once fixed.

---

## Versions (unified v1.5.0)

| File | Version Field | Value |
|---|---|---|
| `rust/Cargo.toml` | `version` | `1.5.0` |
| `nodejs/package.json` | `version` | `1.5.0` |
| `go/go.mod` + git tag | `v1.5.0` | — |
| `java/pom.xml` | `<version>` | `1.5.0` |
| `python/pyproject.toml` | `version` | `1.5.0` |
| `lib/*/overdrive.dll` | Built from | server `1.4.6` |

---

## Build Order

```
Step 1  scripts/build-native.ps1          → lib/windows-x64/overdrive.dll
Step 2  rust/   cargo test --test e2e     → 10/10 pass
Step 3  nodejs/ node test/e2e.js          → 10/10 pass
Step 4  go/     go test -v -tags e2e .    → 10/10 pass
Step 5  java/   mvn test                  → 10/10 pass
Step 6  python/ pytest tests/test_e2e.py  → 10/10 pass
Step 7  git tag v1.5.0 && git push        → release
```

---

## Known Issues

| Issue | Severity | Fix |
|---|---|---|
| `overdrive_query` SQL SELECT returns empty | 🔴 High | Root cause: Shell creates new DB connection per query call — doesn't share the open handle's state. Fix in server `ffi.rs` `overdrive_query()` |
| `overdrive_version()` hardcoded `"1.4.4"` | 🟡 Medium | Change `c"1.4.4"` → `c"1.4.6"` in `ffi.rs` line 656 |
| Windows ACL permission warning | 🟢 Low | `icacls` fails when USERNAME has no SID mapping. Non-critical, safe to ignore |
