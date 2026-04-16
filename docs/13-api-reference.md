# API Reference — All Languages

Quick reference for every method across Python, Node.js, Java, and Go.

---

## Database Opening

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Open/create | `OverDrive.open(path, password?, engine?, auto_create_tables?)` | `OverDrive.open(path, {password?, engine?, autoCreateTables?})` | `OverDrive.open(path, new OpenOptions()...)` | `overdrive.Open(path, ...opts)` |
| Env-var encryption | `OverDrive.open_encrypted(path, env_var)` | `OverDrive.openEncrypted(path, envVar)` | `OverDrive.openEncrypted(path, envVar)` | `overdrive.OpenEncrypted(path, envVar)` |
| Close | `db.close()` | `db.close()` | `db.close()` / auto | `db.Close()` |
| Sync | `db.sync()` | `db.sync()` | `db.sync()` | `db.Sync()` |
| Version | `OverDrive.version()` | `OverDrive.version()` | `OverDrive.version()` | `overdrive.Version()` |

### Engine Values
`"Disk"` (default) · `"RAM"` · `"Vector"` · `"Time-Series"` · `"Graph"` · `"Streaming"`

---

## Table Operations

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Create | `db.create_table(name, engine?)` | `db.createTable(name, {engine?})` | `db.createTable(name, opts?)` | `db.CreateTable(name, opts...)` |
| Drop | `db.drop_table(name)` | `db.dropTable(name)` | `db.dropTable(name)` | `db.DropTable(name)` |
| List | `db.list_tables()` | `db.listTables()` | `db.listTables()` | `db.ListTables()` |
| Exists | `db.table_exists(name)` | `db.tableExists(name)` | `db.tableExists(name)` | `db.TableExists(name)` |

---

## CRUD

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Insert | `db.insert(table, doc)` → `str` | `db.insert(table, doc)` → `string` | `db.insert(table, Map)` → `String` | `db.Insert(table, map)` → `(string, error)` |
| Insert many | `db.insert_many(table, docs)` → `[str]` | `db.insertMany(table, docs)` → `[string]` | `db.insertMany(table, List)` → `List<String>` | loop `db.Insert()` |
| Get | `db.get(table, id)` → `dict\|None` | `db.get(table, id)` → `object\|null` | `db.get(table, id)` → `Map\|null` | `db.Get(table, id)` → `(map, error)` |
| Update | `db.update(table, id, updates)` → `bool` | `db.update(table, id, updates)` → `bool` | `db.update(table, id, Map)` → `bool` | `db.Update(table, id, map)` → `(bool, error)` |
| Delete | `db.delete(table, id)` → `bool` | `db.delete(table, id)` → `bool` | `db.delete(table, id)` → `bool` | `db.Delete(table, id)` → `(bool, error)` |
| Count | `db.count(table)` → `int` | `db.count(table)` → `number` | `db.count(table)` → `int` | `db.Count(table)` → `(int, error)` |

---

## Queries

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| SQL query | `db.query(sql)` → `[dict]` | `db.query(sql)` → `[object]` | `db.query(sql)` → `List<Map>` | `db.Query(sql)` → `(*QueryResult, error)` |
| Full query | `db.query_full(sql)` → `dict` | `db.queryFull(sql)` → `object` | `db.queryFull(sql)` → `Map` | included in `QueryResult` |
| Safe query | `db.querySafe(sql, *params)` | `db.querySafe(sql, [params])` | `db.querySafe(sql, ...params)` | `db.QuerySafe(sql, ...params)` |
| Search | `db.search(table, text)` → `[dict]` | `db.search(table, text)` → `[object]` | `db.search(table, text)` → `List<Map>` | `db.Search(table, text)` → `([]map, error)` |

---

## Helper Methods (v1.4)

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Find one | `db.findOne(table, where?)` | `db.findOne(table, where?)` | `db.findOne(table, where)` | `db.FindOne(table, where)` |
| Find all | `db.findAll(table, where?, order_by?, limit?)` | `db.findAll(table, where?, orderBy?, limit?)` | `db.findAll(table, where, orderBy, limit)` | `db.FindAll(table, where, orderBy, limit)` |
| Update many | `db.updateMany(table, where, updates)` → `int` | `db.updateMany(table, where, updates)` → `number` | `db.updateMany(table, where, Map)` → `int` | `db.UpdateMany(table, where, map)` → `(int, error)` |
| Delete many | `db.deleteMany(table, where)` → `int` | `db.deleteMany(table, where)` → `number` | `db.deleteMany(table, where)` → `int` | `db.DeleteMany(table, where)` → `(int, error)` |
| Count where | `db.countWhere(table, where?)` → `int` | `db.countWhere(table, where?)` → `number` | `db.countWhere(table, where)` → `int` | `db.CountWhere(table, where)` → `(int, error)` |
| Exists | `db.exists(table, id)` → `bool` | `db.exists(table, id)` → `bool` | `db.exists(table, id)` → `bool` | `db.Exists(table, id)` → `(bool, error)` |

---

## Transactions

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Callback | `db.transaction(fn, isolation?)` | `db.transaction(fn, isolation?)` | `db.transaction(fn, isolation?)` | `db.Transaction(fn, isolation)` |
| With retry | `db.transaction_with_retry(fn, max_retries?)` | `db.transactionWithRetry(fn, isolation?, maxRetries?)` | `db.transactionWithRetry(fn, isolation, maxRetries)` | `db.TransactionWithRetry(fn, isolation, maxRetries)` |
| Begin | `db.begin_transaction(isolation?)` | `db.beginTransaction(isolation?)` | `db.beginTransaction(isolation?)` | `db.BeginTransaction(isolation)` |
| Commit | `db.commit_transaction(txn_id)` | `db.commitTransaction(txnId)` | `db.commitTransaction(txnId)` | `db.CommitTransaction(txn)` |
| Abort | `db.abort_transaction(txn_id)` | `db.abortTransaction(txnId)` | `db.abortTransaction(txnId)` | `db.AbortTransaction(txn)` |

### Isolation Level Constants

| Level | Python | Node.js | Java | Go |
|-------|--------|---------|------|-----|
| Read Uncommitted | `OverDrive.READ_UNCOMMITTED` (0) | `OverDrive.READ_UNCOMMITTED` (0) | `OverDrive.READ_UNCOMMITTED` (0) | `overdrive.ReadUncommitted` (0) |
| Read Committed | `OverDrive.READ_COMMITTED` (1) | `OverDrive.READ_COMMITTED` (1) | `OverDrive.READ_COMMITTED` (1) | `overdrive.ReadCommitted` (1) |
| Repeatable Read | `OverDrive.REPEATABLE_READ` (2) | `OverDrive.REPEATABLE_READ` (2) | `OverDrive.REPEATABLE_READ` (2) | `overdrive.RepeatableRead` (2) |
| Serializable | `OverDrive.SERIALIZABLE` (3) | `OverDrive.SERIALIZABLE` (3) | `OverDrive.SERIALIZABLE` (3) | `overdrive.Serializable` (3) |

---

## RAM Engine Methods (v1.4)

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Snapshot | `db.snapshot(path)` | `db.snapshot(path)` | `db.snapshot(path)` | `db.Snapshot(path)` |
| Restore | `db.restore(path)` | `db.restore(path)` | `db.restore(path)` | `db.Restore(path)` |
| Memory usage | `db.memoryUsage()` / `db.memory_usage()` | `db.memoryUsage()` | `db.memoryUsage()` | `db.MemoryUsageStats()` |

### MemoryUsage Fields

| Field | Python | Node.js | Java | Go |
|-------|--------|---------|------|-----|
| Bytes used | `usage['bytes']` | `usage.bytes` | `usage.getBytes()` | `usage.Bytes` |
| Megabytes | `usage['mb']` | `usage.mb` | `usage.getMb()` | `usage.Mb` |
| Limit bytes | `usage['limit_bytes']` | `usage.limit_bytes` | `usage.getLimitBytes()` | `usage.LimitBytes` |
| Percent | `usage['percent']` | `usage.percent` | `usage.getPercent()` | `usage.Percent` |

---

## Watchdog (v1.4)

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Check file | `OverDrive.watchdog(path)` | `OverDrive.watchdog(path)` | `OverDrive.watchdog(path)` | `overdrive.Watchdog(path)` |

### WatchdogReport Fields

| Field | Python | Node.js | Java | Go |
|-------|--------|---------|------|-----|
| File path | `report.file_path` | `report.filePath` | `report.getFilePath()` | `report.FilePath` |
| File size | `report.file_size_bytes` | `report.fileSizeBytes` | `report.getFileSizeBytes()` | `report.FileSizeBytes` |
| Last modified | `report.last_modified` | `report.lastModified` | `report.getLastModified()` | `report.LastModified` |
| Status | `report.integrity_status` | `report.integrityStatus` | `report.getIntegrityStatus()` | `report.IntegrityStatus` |
| Corruption | `report.corruption_details` | `report.corruptionDetails` | `report.getCorruptionDetails()` | `report.CorruptionDetails` |
| Page count | `report.page_count` | `report.pageCount` | `report.getPageCount()` | `report.PageCount` |
| Magic valid | `report.magic_valid` | `report.magicValid` | `report.isMagicValid()` | `report.MagicValid` |

---

## Security Methods

| Method | Python | Node.js | Java | Go |
|--------|--------|---------|------|-----|
| Backup | `db.backup(dest)` | `db.backup(dest)` | `db.backup(dest)` | `db.Backup(dest)` |
| Cleanup WAL | `db.cleanup_wal()` | `db.cleanupWal()` | `db.cleanupWal()` | `db.CleanupWAL()` |

---

## Error Classes

| Class | Python | Node.js | Java | Go |
|-------|--------|---------|------|-----|
| Base | `OverDriveError` | `OverDriveError` | `OverDriveException` | `OverDriveError` (interface) |
| Auth | `AuthenticationError` | `AuthenticationError` | `AuthenticationException` | `*AuthenticationError` |
| Table | `TableError` | `TableError` | `TableException` | `*TableError` |
| Query | `QueryError` | `QueryError` | `QueryException` | `*QueryError` |
| Transaction | `TransactionError` | `TransactionError` | `TransactionException` | `*TransactionError` |
| I/O | `OverDriveIOError` | `OverDriveIOError` | `IOError` | `*IOError` |
| FFI | `FFIError` | `FFIError` | `FFIException` | `*FFIError` |

### Error Properties

| Property | Python | Node.js | Java | Go |
|----------|--------|---------|------|-----|
| Code | `e.code` | `e.code` | `e.getCode()` | `e.Code()` |
| Message | `e.message` | `e.message` | `e.getMessage()` | `e.Error()` |
| Context | `e.context` | `e.context` | `e.getContext()` | `e.Context()` |
| Suggestions | `e.suggestions` | `e.suggestions` | `e.getSuggestions()` | `e.Suggestions()` |
| Doc link | `e.doc_link` | `e.docLink` | `e.getDocLink()` | `e.DocLink()` |

---

## Common Error Codes

| Code | Meaning |
|------|---------|
| `ODB-AUTH-001` | Incorrect password |
| `ODB-AUTH-002` | Password too short (< 8 chars) |
| `ODB-TABLE-001` | Table not found |
| `ODB-TABLE-002` | Table already exists |
| `ODB-QUERY-001` | SQL syntax error |
| `ODB-TXN-001` | Transaction deadlock |
| `ODB-TXN-002` | Transaction conflict |
| `ODB-IO-001` | File not found |
| `ODB-IO-002` | Permission denied |
| `ODB-IO-003` | File corrupted |
| `ODB-FFI-001` | Native library not found |
| `ODB-FFI-002` | Null handle |
