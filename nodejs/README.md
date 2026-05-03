<p align="center">
  <h1 align="center">⚡ overdrive-db</h1>
  <p align="center">
    <strong>Embeddable hybrid SQL+NoSQL document database for Node.js</strong><br/>
    Like SQLite, but for JSON. Zero config. No server. ACID transactions.
  </p>
  <p align="center">
    <a href="https://www.npmjs.com/package/overdrive-db"><img src="https://img.shields.io/npm/v/overdrive-db?style=flat-square&color=cb3837&logo=npm" alt="npm"/></a>
    <a href="https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="license"/></a>
  </p>
</p>

---

## Install

```bash
npm install overdrive-db
```

> **Requires** the native `overdrive.dll` (Windows) / `liboverdrive.so` (Linux) / `liboverdrive.dylib` (macOS) in the `lib/{os}-{arch}/` directory.
> Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest).

---

## Quick Start

```javascript
const { OverdriveDb } = require('overdrive-db');

const odb = OverdriveDb.open('myapp.odb');

// Create table
odb.createTable('users');

// Insert
const id = odb.insert('users', { name: 'Alice', age: 30 });

// Get by ID
const doc = odb.get('users', id);
console.log(doc); // { _id: '...', name: 'Alice', age: 30 }

// Query with SQL
const rows = odb.query('SELECT * FROM users WHERE age > 25');

// Update
odb.update('users', id, { age: 31 });

// Delete
odb.delete('users', id);

odb.close();
```

---

## API Reference

### Open / Close

```javascript
const odb = OverdriveDb.open(path);                          // plain open
const odb = OverdriveDb.open(path, { password: 'secret' }); // encrypted
const odb = OverdriveDb.open(path, { engine: 'RAM' });       // in-memory

odb.sync();   // flush to disk
odb.close();  // release handle

OverdriveDb.version(); // native library version string
```

### Tables

```javascript
odb.createTable('users');
odb.dropTable('users');
odb.listTables();          // → ['users', 'products', ...]
odb.tableExists('users');  // → true / false
```

### CRUD

| Method | Returns | Description |
|--------|---------|-------------|
| `odb.insert(table, doc)` | `string` (\_id) | Insert a document |
| `odb.insertMany(table, docs)` | `string[]` | Insert multiple documents |
| `odb.get(table, id)` | `object \| null` | Get document by \_id |
| `odb.update(table, id, patch)` | `boolean` | Update document by \_id |
| `odb.delete(table, id)` | `boolean` | Delete document by \_id |
| `odb.count(table)` | `number` | Count documents in table |

### Query

```javascript
// SQL query — returns array of row objects
const rows = odb.query('SELECT * FROM users ORDER BY age DESC LIMIT 10');

// Full-text search
const results = odb.search('users', 'Alice');
```

### Transactions

```javascript
const { IsolationLevel } = require('overdrive-db');

// Callback style (auto-commit / auto-abort)
odb.transaction(() => {
  odb.insert('accounts', { balance: 1000 });
  odb.update('accounts', id, { balance: 900 });
}, IsolationLevel.Serializable);

// Manual style
const txn = odb.beginTransaction(IsolationLevel.ReadCommitted);
try {
  odb.insert('logs', { event: 'login' });
  odb.commitTransaction(txn);
} catch (e) {
  odb.abortTransaction(txn);
}
```

**Isolation Levels:**

| Name | Value |
|------|-------|
| `IsolationLevel.ReadUncommitted` | 0 |
| `IsolationLevel.ReadCommitted` | 1 (default) |
| `IsolationLevel.RepeatableRead` | 2 |
| `IsolationLevel.Serializable` | 3 |

### Integrity Check

```javascript
const report = odb.verifyIntegrity();
console.log(report); // { status: 'ok', ... }
```

---

## 6 Storage Engines

```javascript
const odb = OverdriveDb.open('app.odb', { engine: 'Disk' });       // default
const odb = OverdriveDb.open('cache.odb', { engine: 'RAM' });      // in-memory
const odb = OverdriveDb.open('vecs.odb', { engine: 'Vector' });    // embeddings
const odb = OverdriveDb.open('ts.odb', { engine: 'Time-Series' }); // metrics
const odb = OverdriveDb.open('g.odb', { engine: 'Graph' });        // social graphs
const odb = OverdriveDb.open('q.odb', { engine: 'Streaming' });    // event queue
```

---

## Error Handling

All methods throw an `Error` on failure. The error message includes the native error code:

```javascript
try {
  odb.insert('nonexistent', { key: 'val' });
} catch (e) {
  console.error(e.message); // [overdrive-db] insert failed: ODB-TABLE-001
}
```

---

## More SDKs

OverDrive-DB is available for every major language:

```bash
pip install overdrive-db          # Python
npm install overdrive-db          # Node.js  ← you are here
cargo add overdrive-db            # Rust
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.0.0
```

---

## Links

- [GitHub Repository](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)
- [Native Library Downloads (Releases)](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest)
- [License: MIT](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/blob/main/LICENSE)
