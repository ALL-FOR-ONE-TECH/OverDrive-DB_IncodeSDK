# Node.js SDK — Complete Guide

**Version:** 1.4.0 | **Requires:** Node.js 14+

---

## Installation

```bash
npm install overdrive-db
```

---

## Import

```javascript
const {
    OverDrive,
    SharedOverDrive,
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
} = require('overdrive-db');
```

---

## Opening a Database

```javascript
// Basic open (auto-creates if not exists)
const db = OverDrive.open('myapp.odb');

// With password encryption
const db = OverDrive.open('secure.odb', { password: 'my-secret-pass-123' });

// With RAM engine
const db = OverDrive.open('cache.odb', { engine: 'RAM' });

// All options
const db = OverDrive.open('app.odb', {
    password: 'my-secret-pass',     // optional, min 8 chars
    engine: 'Disk',                  // 'Disk'|'RAM'|'Vector'|'Time-Series'|'Graph'|'Streaming'
    autoCreateTables: true           // default true
});

// Legacy constructor (v1.3 — still works)
const db = new OverDrive('myapp.odb');

// Environment variable encryption (v1.3 — still works)
// process.env.ODB_KEY = "my-aes-256-key-32-chars-minimum!!!!"
const db = OverDrive.openEncrypted('app.odb', 'ODB_KEY');

// Always close when done
db.close();
```

---

## Table Operations

```javascript
// Create table (optional — auto-created on first insert)
db.createTable('users');
db.createTable('sessions', { engine: 'RAM' });  // RAM table

// Drop table
db.dropTable('old_table');

// List tables
const tables = db.listTables();  // ['users', 'products', ...]

// Check existence
const exists = db.tableExists('users');  // true or false
```

---

## CRUD Operations

### Insert

```javascript
// Single document — returns auto-generated _id
const id = db.insert('users', {
    name: 'Alice',
    age: 30,
    email: 'alice@example.com',
    tags: ['admin', 'dev'],
    active: true
});
console.log(id);  // "users_1"

// Multiple documents
const ids = db.insertMany('users', [
    { name: 'Bob',   age: 25 },
    { name: 'Carol', age: 35 },
]);
console.log(ids);  // ["users_2", "users_3"]
```

### Get

```javascript
// Get by _id
const user = db.get('users', 'users_1');
// → { _id: 'users_1', name: 'Alice', age: 30, ... }
// → null if not found

// Count all documents
const count = db.count('users');  // number
```

### Update

```javascript
// Update by _id — only specified fields change
const updated = db.update('users', 'users_1', { age: 31, status: 'active' });
// → true if found and updated, false if not found
```

### Delete

```javascript
// Delete by _id
const deleted = db.delete('users', 'users_1');
// → true if found and deleted, false if not found
```

---

## SQL Queries

```javascript
// Execute SQL — returns array of objects
const results = db.query('SELECT * FROM users');
const results = db.query('SELECT * FROM users WHERE age > 25');
const results = db.query('SELECT * FROM users WHERE age > 25 ORDER BY name DESC LIMIT 10');
const results = db.query('SELECT COUNT(*) FROM users');
const results = db.query('SELECT AVG(age), MIN(age), MAX(age) FROM users');

// Full result with metadata
const result = db.queryFull('SELECT * FROM users');
// → { rows: [...], columns: [...], rows_affected: 0 }

// Safe parameterized query (use for user input!)
const results = db.querySafe('SELECT * FROM users WHERE name = ?', [userName]);
const results = db.querySafe('SELECT * FROM users WHERE age > ? AND city = ?', ['25', 'London']);

// Full-text search
const matches = db.search('users', 'alice');  // array of matching documents
```

---

## Helper Methods (v1.4)

```javascript
// findOne — first match or null
const user = db.findOne('users', "age > 25");
const first = db.findOne('users');  // first document, no filter

// findAll — all matches
const users = db.findAll('users');
const users = db.findAll('users', 'age > 25');
const users = db.findAll('users', 'age > 25', 'name ASC');
const users = db.findAll('users', 'age > 25', 'name ASC', 10);

// updateMany — bulk update, returns count of updated docs
const count = db.updateMany('users', "status = 'trial'", { status: 'active' });

// deleteMany — bulk delete, returns count of deleted docs
const count = db.deleteMany('logs', "created_at < '2025-01-01'");

// countWhere — count matching docs
const n = db.countWhere('users', 'age > 25');
const total = db.countWhere('users');  // count all

// exists — check if document exists by _id
const found = db.exists('users', 'users_1');  // true or false
```

---

## Transactions

```javascript
// Callback pattern — sync (recommended — v1.4)
const result = db.transaction((txn) => {
    db.updateMany('accounts', "id = 'alice'", { balance: 900 });
    db.updateMany('accounts', "id = 'bob'",   { balance: 600 });
    return 'transfer complete';
});

// Callback pattern — async (returns Promise)
const result = await db.transaction(async (txn) => {
    const orderId = db.insert('orders', { item: 'widget' });
    await someExternalCall();
    return orderId;
});

// With isolation level
const result = db.transaction(
    (txn) => db.insert('logs', { event: 'test' }),
    OverDrive.SERIALIZABLE
);

// With retry on conflict
const result = await db.transactionWithRetry(
    async (txn) => db.insert('orders', { item: 'widget' }),
    OverDrive.READ_COMMITTED,
    3  // max retries
);

// Context object (no callback)
const txn = db.transaction();  // returns { id, commit(), abort() }
db.insert('users', { name: 'Alice' });
txn.commit();  // or txn.abort()

// Manual (v1.3 — still works)
const txnId = db.beginTransaction();
const txnId = db.beginTransaction(OverDrive.SERIALIZABLE);
db.commitTransaction(txnId);
db.abortTransaction(txnId);

// Isolation level constants
OverDrive.READ_UNCOMMITTED  // 0
OverDrive.READ_COMMITTED    // 1 (default)
OverDrive.REPEATABLE_READ   // 2
OverDrive.SERIALIZABLE      // 3
```

---

## RAM Engine Methods (v1.4)

```javascript
// Snapshot — persist RAM database to disk
db.snapshot('./backup/cache.odb');

// Restore — load snapshot into RAM database
db.restore('./backup/cache.odb');

// Memory usage
const usage = db.memoryUsage();
// → { bytes: 1048576, mb: 1.0, limit_bytes: 2147483648, percent: 0.05 }
console.log(`Using ${usage.mb.toFixed(1)} MB (${usage.percent.toFixed(1)}%)`);
```

---

## Watchdog (v1.4)

```javascript
// Static method — no open database needed
const report = OverDrive.watchdog('app.odb');

// WatchdogReport fields
report.filePath           // string — path inspected
report.fileSizeBytes      // number — file size
report.lastModified       // number — Unix timestamp
report.integrityStatus    // string — "valid", "corrupted", "missing"
report.corruptionDetails  // string|null — details if corrupted
report.pageCount          // number — number of pages
report.magicValid         // boolean — magic number valid

// Usage pattern
if (report.integrityStatus === 'valid') {
    const db = OverDrive.open('app.odb');
} else if (report.integrityStatus === 'corrupted') {
    console.error(`Corrupted: ${report.corruptionDetails}`);
} else {
    console.log('File not found — creating new database');
    const db = OverDrive.open('app.odb');
}
```

---

## Error Handling

```javascript
const { AuthenticationError, TableError, QueryError, OverDriveError } = require('overdrive-db');

try {
    const db = OverDrive.open('secure.odb', { password: 'wrong' });
} catch (e) {
    if (e instanceof AuthenticationError) {
        console.log(e.code);         // "ODB-AUTH-001"
        console.log(e.message);      // "Incorrect password..."
        console.log(e.context);      // "secure.odb"
        console.log(e.suggestions);  // ["Verify you're using the correct password", ...]
        console.log(e.docLink);      // "https://overdrive-db.com/docs/errors/ODB-AUTH-001"
    }
}

try {
    db.query('INVALID SQL !!!');
} catch (e) {
    if (e instanceof QueryError) {
        console.log(`Query error: ${e.code} — ${e.message}`);
    }
}

// Catch all OverDrive errors
try {
    db.insert('users', { name: 'Alice' });
} catch (e) {
    if (e instanceof OverDriveError) {
        console.log(`Database error [${e.code}]: ${e.message}`);
    }
}
```

---

## Security

```javascript
// Password from environment variable
const db = OverDrive.open('secure.odb', { password: process.env.DB_PASSWORD });

// Backup
db.backup('backups/app_2026-04-16.odb');

// WAL cleanup after commit
const txnId = db.beginTransaction();
db.insert('users', { name: 'Alice' });
db.commitTransaction(txnId);
db.cleanupWal();

// Safe queries
const results = db.querySafe('SELECT * FROM users WHERE name = ?', [userInput]);
```

---

## Async-Safe Access

```javascript
const { SharedOverDrive } = require('overdrive-db');

// Promise-based mutex — safe for async/await
const db = new SharedOverDrive('app.odb');

// All methods return Promises
await db.insert('logs', { event: 'started' });
const results = await db.query('SELECT * FROM logs');
await db.close();
```

---

## Complete Example

```javascript
const { OverDrive } = require('overdrive-db');

const db = OverDrive.open('store.odb');

// Insert products (table auto-created)
const products = [
    { name: 'Laptop',  price: 999,  category: 'electronics', stock: 5 },
    { name: 'Mouse',   price: 29,   category: 'electronics', stock: 50 },
    { name: 'Desk',    price: 299,  category: 'furniture',   stock: 10 },
    { name: 'Chair',   price: 199,  category: 'furniture',   stock: 15 },
];
const ids = db.insertMany('products', products);
console.log(`Inserted ${ids.length} products`);

// Query
const electronics = db.findAll('products', "category = 'electronics'", 'price DESC');
console.log('Electronics:', electronics.map(p => p.name));

// Count
const total = db.countWhere('products');
const expensive = db.countWhere('products', 'price > 100');
console.log(`Total: ${total}, Expensive: ${expensive}`);

// Update with transaction
const updated = db.transaction((txn) => {
    const count = db.updateMany('products', 'price > 100', { onSale: true, discount: 10 });
    db.insert('auditLog', { event: 'saleApplied', affected: count });
    return count;
});
console.log(`Applied sale to ${updated} products`);

// Watchdog check
const report = OverDrive.watchdog('store.odb');
console.log(`Database health: ${report.integrityStatus} (${report.fileSizeBytes} bytes)`);

// Backup
db.backup('backups/store_backup.odb');
console.log('Backup created!');

db.close();
```
