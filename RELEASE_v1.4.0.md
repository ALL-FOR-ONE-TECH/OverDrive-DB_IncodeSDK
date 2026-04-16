# OverDrive-DB InCode SDK v1.4.0 Release Notes

**Release Date:** April 16, 2026  
**Repository:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK

---

## 🎯 What's New

v1.4.0 dramatically simplifies the SDK API while maintaining **100% backward compatibility**.
Every line of v1.3 code continues to work unchanged.

---

## 🚀 Highlights

### Open a database in one line
```python
db = OverDrive.open("myapp.odb")
db.insert("users", {"name": "Alice"})  # table auto-created — no createTable() needed
```

### Password-protected databases
```python
db = OverDrive.open("secure.odb", password="my-secret-pass")
# Keys derived via Argon2id (64MB memory, 3 iterations) — no env vars needed
```

### RAM engine for sub-microsecond caching
```python
cache = OverDrive.open("cache.odb", engine="RAM")
cache.insert("sessions", {"user_id": 123})
cache.snapshot("./backup/cache.odb")   # persist to disk
cache.restore("./backup/cache.odb")    # restore later
usage = cache.memoryUsage()            # {"bytes": ..., "mb": ..., "percent": ...}
```

### File integrity monitoring
```python
report = OverDrive.watchdog("app.odb")
# → WatchdogReport(integrity_status="valid", file_size_bytes=8192, page_count=2, ...)
```

### Transaction callbacks
```python
result = db.transaction(lambda txn: db.insert("orders", {"item": "widget"}))
# auto-commits on success, auto-rolls back on exception
```

### Helper methods
```python
user    = db.findOne("users", "age > 25")
users   = db.findAll("users", order_by="name", limit=10)
count   = db.updateMany("users", "status = 'trial'", {"status": "active"})
deleted = db.deleteMany("logs", "created_at < '2025-01-01'")
n       = db.countWhere("users", "active = 1")
exists  = db.exists("users", "users_1")
```

---

## 📦 Package Updates

### Python (`overdrive-db`)
```bash
pip install overdrive-db==1.4.0
```

### Node.js (`overdrive-db`)
```bash
npm install overdrive-db@1.4.0
```

### Java (`com.afot:overdrive-db`)
```xml
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.4.0</version>
</dependency>
```

### Go
```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.4.0
```

### Rust
```toml
overdrive-db = "1.4.0"
```

---

## 🔧 Native Library Downloads

| Platform | File | Size |
|----------|------|------|
| Windows x64 | `overdrive.dll` | ~3.3 MB |
| Linux x64 | `liboverdrive.so` | ~4.1 MB |
| Linux ARM64 | `liboverdrive-arm64.so` | ~3.9 MB |
| macOS x64 | `liboverdrive.dylib` | ~3.8 MB |
| macOS ARM64 | `liboverdrive-arm64.dylib` | ~3.6 MB |

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/tag/v1.4.0).

---

## 📋 Full Changelog

### Added — All SDKs (Python, Node.js, Java, Go)
- `open()` static method with `password`, `engine`, `autoCreateTables` parameters
- `watchdog()` static method returning `WatchdogReport` with integrity status
- `transaction(callback)` — auto-commit/rollback callback pattern
- `transactionWithRetry()` — exponential backoff retry on conflict
- `findOne()`, `findAll()`, `updateMany()`, `deleteMany()`, `countWhere()`, `exists()` helper methods
- `snapshot()`, `restore()`, `memoryUsage()` RAM engine methods
- Structured error hierarchy: `AuthenticationError`, `TableError`, `QueryError`, `TransactionError`, `IOError`, `FFIError`
- Error codes: `ODB-AUTH-001`, `ODB-TABLE-001`, `ODB-QUERY-001`, `ODB-TXN-001`, `ODB-IO-001`, `ODB-FFI-001`

### Added — Native Library
- Password-based encryption via Argon2id key derivation
- Auto-table creation across all 6 storage engines
- RAM engine FFI: `overdrive_snapshot`, `overdrive_restore`, `overdrive_memory_usage`
- Watchdog FFI: `overdrive_watchdog` (< 100ms for files under 1GB)
- Engine selection FFI: `overdrive_open_with_engine`, `overdrive_create_table_with_engine`

### Added — Build & CI/CD
- Multi-platform build workflow (Windows, Linux x64/ARM64, macOS x64/ARM64)
- Automated sync from private repo to this public SDK repo
- Native binaries embedded in SDK packages

### Added — Documentation
- [Quick Start Guide](docs/quickstart.md)
- [API Reference](docs/api-reference.md)
- [Migration Guide v1.3 → v1.4](docs/migration-v1.3-to-v1.4.md)
- [Security Best Practices](docs/security.md)
- Working examples for Python and Node.js

### Backward Compatibility
- ✅ All v1.3.0 constructors still work
- ✅ All v1.3.0 methods unchanged
- ✅ All v1.3.0 error types preserved
- ✅ 100% test suite compatibility

---

## 🔗 Links

- **Repository:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK
- **Releases:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases
- **npm:** https://www.npmjs.com/package/overdrive-db
- **PyPI:** https://pypi.org/project/overdrive-db/
- **Issues:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues
- **Website:** https://overdrive-db.com

---

## 📞 Support

- **Issues:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues
- **Email:** admin@afot.in
- **Website:** https://overdrive-db.com

---

**Full Changelog:** https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/compare/v1.3.0...v1.4.0
