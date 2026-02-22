# Architecture

## Overview

OverDrive SDK is an **embeddable database library**. Unlike client-server databases (PostgreSQL, MongoDB), everything runs in-process. There's no separate daemon, no TCP connection, no config files — just a library and a file.

```
┌─────────────────────────────────────────────────────────────┐
│                     Your Application                        │
│                                                             │
│  use overdrive::OverDriveDB;                                │
│  let db = OverDriveDB::open("app.odb")?;                    │
│  db.query("SELECT * FROM users WHERE age > 25")?;           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                   OverDrive SDK Layer                        │
│                                                             │
│  ┌──────────┐  ┌──────────────┐  ┌────────┐  ┌──────────┐  │
│  │OverDrive │  │ Query Engine │  │  C FFI │  │   Error  │  │
│  │   DB     │  │ (SQL Parser) │  │  Layer │  │ Handling │  │
│  │ (lib.rs) │  │              │  │        │  │          │  │
│  └────┬─────┘  └──────┬───────┘  └───┬────┘  └──────────┘  │
│       │               │              │                       │
├───────┴───────────────┴──────────────┴───────────────────────┤
│                  OverDrive Core Engine                        │
│                                                              │
│  ┌─────────┐ ┌─────┐ ┌──────┐ ┌───────┐ ┌──────────────┐   │
│  │ Storage │ │ WAL │ │ MVCC │ │ Index │ │  Full-text   │   │
│  │ (Pages) │ │     │ │      │ │(BTree)│ │   Search     │   │
│  └────┬────┘ └──┬──┘ └──┬───┘ └───┬───┘ └──────┬───────┘   │
│       │         │       │         │             │            │
│  ┌────┴─────────┴───────┴─────────┴─────────────┴────────┐  │
│  │                 Page Manager                           │  │
│  │  ┌──────────┐ ┌───────────┐ ┌───────┐ ┌────────────┐  │  │
│  │  │  Header  │ │  B-Tree   │ │ Data  │ │  Free List │  │  │
│  │  │  Page    │ │  Nodes    │ │ Pages │ │            │  │  │
│  │  └──────────┘ └───────────┘ └───────┘ └────────────┘  │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │                                   │
├──────────────────────────┴───────────────────────────────────┤
│                    Operating System                           │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐   │
│  │              File System (app.odb)                     │   │
│  │              Memory-mapped I/O (mmap)                  │   │
│  └───────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

## Component Details

### SDK Layer (`overdrive-sdk`)

| Component | File | Purpose |
|---|---|---|
| **OverDriveDB** | `lib.rs` | Main API struct — 25+ public methods |
| **Query Engine** | `query_engine.rs` | SQL tokenizer, parser, and executor |
| **FFI Layer** | `ffi.rs` | C-compatible `extern` functions |
| **Error Types** | `result.rs` | User-friendly error enum |

### Core Engine (`overdrive-db`)

| Component | Purpose |
|---|---|
| **B-Tree** | Primary data structure for key-value storage |
| **WAL** | Write-Ahead Log for crash recovery |
| **MVCC** | Multi-Version Concurrency Control |
| **Page Manager** | Fixed-size page allocation and management |
| **Index Manager** | Secondary indexes (B-Tree and Hash) |
| **Full-text Search** | Text tokenization and inverted index |
| **Compression** | zstd compression for data pages |
| **Encryption** | AES-256-GCM at-rest encryption |
| **Cache** | LRU page cache for hot data |

## Data Flow

### Write Path (Insert)

```
db.insert("users", json)
    │
    ├── 1. Serialize JSON → bytes
    ├── 2. Generate UUID v4 for _id
    ├── 3. Write to WAL (crash safety)
    ├── 4. Insert into B-Tree (key = _id)
    ├── 5. Update secondary indexes
    ├── 6. Update full-text index
    └── 7. Return _id
```

### Read Path (Get)

```
db.get("users", id)
    │
    ├── 1. Check page cache (LRU)
    ├── 2. If miss → B-Tree lookup
    ├── 3. Read data page
    ├── 4. Decompress (if zstd enabled)
    ├── 5. Decrypt (if AES enabled)
    ├── 6. Deserialize → JSON
    └── 7. Return Value
```

### Query Path (SQL)

```
db.query("SELECT * FROM users WHERE age > 25")
    │
    ├── 1. Tokenize SQL string
    ├── 2. Parse into query AST
    ├── 3. Identify query type (SELECT)
    ├── 4. Scan table data
    ├── 5. Evaluate WHERE clause per row
    ├── 6. Apply ORDER BY (if present)
    ├── 7. Apply LIMIT/OFFSET
    ├── 8. Project selected columns
    └── 9. Return QueryResult
```

## File Format

The `.odb` database file uses a page-based layout:

```
┌──────────────────┐  Page 0
│   File Header    │  Magic bytes, version, page size
├──────────────────┤  Page 1
│   Table Catalog  │  Table names → root page mappings
├──────────────────┤  Page 2..N
│   B-Tree Nodes   │  Internal + leaf nodes
├──────────────────┤
│   Data Pages     │  Actual JSON documents
├──────────────────┤
│   Index Pages    │  Secondary index data
├──────────────────┤
│   Free List      │  Reusable page numbers
└──────────────────┘
```

## Thread Safety

- The `OverDriveDB` struct is **not `Send + Sync`** — use one instance per thread
- For multi-threaded access, use a `Mutex<OverDriveDB>` or separate instances
- The core engine uses internal locking for page-level concurrency

## Performance Characteristics

| Operation | Complexity | Notes |
|---|---|---|
| Get by ID | O(log n) | B-Tree lookup |
| Insert | O(log n) | B-Tree insert + WAL write |
| Scan (full table) | O(n) | Reads all pages |
| WHERE filter | O(n) | Full scan + filter |
| WHERE with index | O(log n) | Index lookup |
| COUNT | O(n) or O(1) | Depends on implementation |
