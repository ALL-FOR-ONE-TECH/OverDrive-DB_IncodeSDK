# OverDrive-DB — Node.js SDK v1.3.0

**A high-performance, embeddable hybrid SQL+NoSQL document database. Like SQLite, but for JSON.**

> **v1.3.0** — Security hardened: encrypted key from env, parameterized queries, auto file permission hardening, encrypted backups, WAL cleanup, async-safe wrapper.

## Install

```bash
npm install overdrive-db@1.3.0
```

The native binary is **automatically downloaded** from GitHub Releases during install.

## Quick Start

```javascript
const { OverDrive } = require('overdrive-db');

// Open — file permissions auto-hardened
const db = new OverDrive('myapp.odb');

db.createTable('products');
const id = db.insert('products', { name: 'Laptop', price: 999.99 });

// ✅ Safe parameterized query — blocks SQL injection
const results = db.querySafe('SELECT * FROM products WHERE price > ?', ['500']);
console.table(results);

db.backup('backups/products.odb');
db.cleanupWal();
db.close();
```

## Security APIs (v1.3.0)

```javascript
const { OverDrive, SharedOverDrive } = require('overdrive-db');

// 🔐 Open with encryption key from env (never hardcode!)
// $env:ODB_KEY = "my-secret-32-char-key!!!!"  (PowerShell)
// process.env.ODB_KEY = 'my-secret-key'
const db = OverDrive.openEncrypted('app.odb', 'ODB_KEY');

// 🛡️ SQL injection prevention
const userInput = "Alice'; DROP TABLE users--"; // malicious
try {
    db.querySafe('SELECT * FROM users WHERE name = ?', [userInput]);
} catch (e) {
    console.log('Blocked:', e.message); // ✅ injection blocked
}

// 💾 Encrypted backup
db.backup('backups/app_2026-03-04.odb');

// 🗑️ WAL cleanup after commit
db.cleanupWal();

// 🔄 Async-safe access (serializes concurrent calls)
const shared = new SharedOverDrive('app.odb');
await Promise.all([
    shared.query('SELECT * FROM users'),
    shared.insert('users', { name: 'Bob' }),
]);
```

## Full API

| Method | Description |
|---|---|
| `new OverDrive(path)` | Open database (auto-hardens permissions) |
| `OverDrive.openEncrypted(path, keyEnvVar)` | 🔐 Open with key from env var |
| `db.close()` | Close the database |
| `db.sync()` | Force flush to disk |
| `db.createTable(name)` | Create a table |
| `db.dropTable(name)` | Drop a table |
| `db.listTables()` | List all tables |
| `db.tableExists(name)` | Check if table exists |
| `db.insert(table, doc)` | Insert document, returns `_id` |
| `db.insertMany(table, docs)` | Batch insert |
| `db.get(table, id)` | Get by `_id` |
| `db.update(table, id, updates)` | Update fields |
| `db.delete(table, id)` | Delete by `_id` |
| `db.count(table)` | Count documents |
| `db.query(sql)` | Execute SQL (trusted input only) |
| `db.queryFull(sql)` | SQL with full metadata |
| `db.querySafe(sql, params)` | ✅ Parameterized query (user input safe) |
| `db.search(table, text)` | Full-text search |
| `db.backup(destPath)` | 💾 Encrypted backup |
| `db.cleanupWal()` | 🗑️ Delete stale WAL file |
| `OverDrive.version()` | SDK version |
| `new SharedOverDrive(path)` | 🔄 Async-safe serialized wrapper |

## TypeScript

Full TypeScript definitions included (`index.d.ts`).

```typescript
import { OverDrive, SharedOverDrive } from 'overdrive-db';
const db = new OverDrive('myapp.odb');
const rows = db.querySafe('SELECT * FROM users WHERE name = ?', [userInput]);
```

## Links

- [Full Documentation](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/nodejs-guide.md)
- [GitHub](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK)
- [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases)
- [Security Guide](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/architecture.md#security-model)

## License

MIT / Apache-2.0
