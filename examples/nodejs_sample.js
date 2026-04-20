/**
 * OverDrive-DB Node.js SDK — Sample Code (v1.4.3)
 * Install: npm install overdrive-db
 */
const { OverDrive } = require('overdrive-db');

// ── 1. Open database ──────────────────────────
const db = OverDrive.open('myapp.odb');

// ── 2. Insert documents (table auto-created) ──
db.insert('users', { name: 'Alice', age: 30, role: 'admin' });
db.insert('users', { name: 'Bob',   age: 25, role: 'user' });
db.insert('users', { name: 'Carol', age: 35, role: 'admin' });

// ── 3. Query ──────────────────────────────────
const results = db.query('SELECT * FROM users WHERE age > 28');
console.log(`Users over 28: ${results.length}`);

// ── 4. Get by ID ──────────────────────────────
const tables = db.listTables();
console.log(`Tables: ${tables.join(', ')}`);

// ── 5. Count ──────────────────────────────────
const count = db.count('users');
console.log(`Total users: ${count}`);

// ── 6. Transactions ───────────────────────────
const txnId = db.beginTransaction();
db.insert('users', { name: 'Dave', age: 28, role: 'user' });
db.commitTransaction(txnId);
console.log('Transaction committed');

// ── 7. Parameterized queries (safe) ───────────
const safe = db.querySafe(
    "SELECT * FROM users WHERE name = ?",
    ['Alice']
);
console.log(`Safe query: ${safe.length} result(s)`);

// ── 8. Cleanup ────────────────────────────────
db.close();
console.log('\n✅ Done!');
