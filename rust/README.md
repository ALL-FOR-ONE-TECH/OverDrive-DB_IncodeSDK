# overdrive-db (Rust SDK)

> ⚠️ **Package renamed**: This crate was previously published as `overdrive-sdk`. The canonical crate is now [`overdrive-db`](https://crates.io/crates/overdrive-db).
>
> **Please use:**
> ```toml
> [dependencies]
> overdrive-db = "2.2"
> ```

---

# OverDrive-DB — Embedded Document Database

[![Crates.io](https://img.shields.io/crates/v/overdrive-db)](https://crates.io/crates/overdrive-db)
[![License](https://img.shields.io/crates/l/overdrive-db)](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)
[![GitHub](https://img.shields.io/badge/GitHub-IncodeSDK-blue)](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)

Zero-config embedded JSON document database with ACID transactions, AES-256 encryption, and 6 storage engines — available in Rust, Python, Node.js, Java, and Go.

## Install

```toml
[dependencies]
overdrive-db = "2.2"
```

## Quick Start

```rust
use overdrive::OverdriveDb;

fn main() {
    let odb = OverdriveDb::open("myapp.odb").unwrap();

    // Insert
    odb.insert("users", r#"{"name":"Alice","age":30}"#).unwrap();

    // Query
    let rows = odb.select("users", "age > 25").unwrap();
    println!("{}", rows);

    // Count
    let n = odb.count("users").unwrap();
    println!("Total: {}", n);

    odb.close();
}
```

## Storage Engines

| Engine | Use Case | Latency |
|---|---|---|
| Disk (default) | General-purpose persistent storage | ~1ms |
| RAM | Caching, sessions, leaderboards | <1µs |
| Vector | Similarity search, embeddings | ~5ms |
| Time-Series | Metrics, IoT, logs | ~2ms |
| Graph | Social networks, knowledge graphs | ~3ms |
| Streaming | Event queues, message brokers | ~1ms |

## Features

- ✅ Zero-config — open a file, start querying
- ✅ SQL queries — SELECT, INSERT, UPDATE, DELETE, WHERE, ORDER BY, LIMIT
- ✅ Aggregations — COUNT, SUM, AVG, MIN, MAX, GROUP BY
- ✅ Full-text search
- ✅ B-Tree indexes
- ✅ ACID transactions (MVCC, 4 isolation levels)
- ✅ AES-256-GCM encryption via Argon2id
- ✅ Cross-platform — Windows, Linux x64/ARM64, macOS x64/ARM64

## Platform Support

| Platform | Binary |
|---|---|
| Windows x64 | `overdrive.dll` |
| Linux x64 | `liboverdrive.so` |
| Linux ARM64 | `liboverdrive.so` |
| macOS x64 | `liboverdrive.dylib` |
| macOS ARM64 | `liboverdrive.dylib` |

## Links

- 📦 [crates.io/crates/overdrive-db](https://crates.io/crates/overdrive-db)
- 🐙 [GitHub Repository](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)
- 🌐 [overdrive-db.com](https://overdrive-db.com)
- 📖 [Documentation](https://docs.rs/overdrive-db)
