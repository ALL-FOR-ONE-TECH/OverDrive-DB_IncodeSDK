# OverDrive-DB InCode SDK — Documentation

**Version:** 1.4.0 | **License:** MIT / Apache-2.0

Welcome to the complete documentation for **OverDrive-DB**, an embeddable hybrid SQL+NoSQL document database. No server. No config. Just a file.

---

## What is OverDrive-DB?

OverDrive-DB is a **library you embed directly in your application**. Think of it like SQLite, but built natively for JSON documents. You get:

- SQL queries on JSON data
- 6 storage engines (Disk, RAM, Vector, Time-Series, Graph, Streaming)
- ACID transactions with MVCC
- AES-256-GCM encryption
- Full-text search
- Works in Rust, Python, Node.js, Java, Go, and C/C++

```
Your App  →  OverDrive SDK  →  overdrive.dll/.so/.dylib  →  app.odb file
```

No network. No daemon. No Docker. Just import and go.

---

## Documentation Map

| Document | What You'll Learn |
|----------|-------------------|
| [01 — Quick Start](01-quickstart.md) | Install, open a database, first insert and query |
| [02 — Core Concepts](02-concepts.md) | How OverDrive works under the hood |
| [03 — SQL Reference](03-sql-reference.md) | Every SQL statement with examples |
| [04 — Storage Engines](04-storage-engines.md) | Disk, RAM, Vector, Time-Series, Graph, Streaming |
| [05 — Transactions](05-transactions.md) | MVCC, isolation levels, callback pattern |
| [06 — Security](06-security.md) | Encryption, passwords, SQL injection prevention |
| [07 — Python SDK](07-python-sdk.md) | Complete Python guide with all v1.4 features |
| [08 — Node.js SDK](08-nodejs-sdk.md) | Complete Node.js guide with all v1.4 features |
| [09 — Java SDK](09-java-sdk.md) | Complete Java guide with all v1.4 features |
| [10 — Go SDK](10-go-sdk.md) | Complete Go guide with all v1.4 features |
| [11 — Rust SDK](11-rust-sdk.md) | Complete Rust guide with all v1.4 features |
| [12 — C/C++ SDK](12-c-sdk.md) | C header and FFI usage |
| [13 — API Reference](13-api-reference.md) | Every method, every language, every parameter |
| [14 — Error Handling](14-error-handling.md) | Error codes, hierarchy, retry patterns |
| [15 — Migration Guide](migration-v1.3-to-v1.4.md) | Upgrading from v1.3 |

---

## 30-Second Install

```bash
pip install overdrive-db          # Python
npm install overdrive-db          # Node.js
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.4.0
```

```toml
# Rust — Cargo.toml
overdrive-db = "1.4.0"
```

```xml
<!-- Java — pom.xml -->
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.4.0</version>
</dependency>
```

---

## 3-Line Hello World

```python
from overdrive import OverDrive
db = OverDrive.open("hello.odb")
db.insert("greetings", {"message": "Hello, World!", "from": "OverDrive"})
print(db.query("SELECT * FROM greetings"))
```

Output:
```
[{'_id': 'greetings_1', 'message': 'Hello, World!', 'from': 'OverDrive'}]
```

---

## Key Concepts in 60 Seconds

1. **Database = a single `.odb` file** — copy it, back it up, delete it. That's your whole database.
2. **Tables are schemaless** — insert any JSON shape, no column definitions needed.
3. **`_id` is auto-generated** — every document gets a unique ID like `users_1`, `users_2`.
4. **SQL works on JSON** — `SELECT * FROM users WHERE age > 25` just works.
5. **Auto-table creation** — `db.insert("users", {...})` creates the table if it doesn't exist.
6. **Transactions are ACID** — full MVCC with 4 isolation levels.

---

## Links

- **GitHub:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK
- **Releases:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases
- **npm:** https://www.npmjs.com/package/overdrive-db
- **PyPI:** https://pypi.org/project/overdrive-db/
- **Issues:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues
