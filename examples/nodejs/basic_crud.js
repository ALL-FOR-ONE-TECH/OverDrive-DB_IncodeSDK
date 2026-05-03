/**
 * OverDrive-DB — Basic CRUD Example (Node.js)
 *
 * Demonstrates: insert, get, update, delete, query, and helper methods.
 *
 * Install:
 *   npm install overdrive-db
 *
 * Run:
 *   node basic_crud.js
 */
const { OverDrive } = require('overdrive-db');

// Open (or create) a database.
// Tables are auto-created on first insert — no createTable() needed.
const db = OverDrive.open('basic_crud.odb');

console.log('=== INSERT ===');
// Insert documents — each gets an auto-generated _id like "users_1"
const id1 = db.insert('users', { name: 'Alice',   age: 30, email: 'alice@example.com' });
const id2 = db.insert('users', { name: 'Bob',     age: 25, email: 'bob@example.com' });
const id3 = db.insert('users', { name: 'Charlie', age: 35, email: 'charlie@example.com' });
console.log('Inserted:', id1, id2, id3);

console.log('\n=== GET ===');
// Retrieve a single document by its _id
const user = db.get('users', id1);
console.log('Got:', user);

console.log('\n=== UPDATE ===');
// Update specific fields on a document
db.update('users', id1, { age: 31, status: 'active' });
console.log('Updated Alice:', db.get('users', id1));

console.log('\n=== DELETE ===');
// Delete a document by _id
const deleted = db.delete('users', id3);
console.log(`Deleted ${id3}:`, deleted);
console.log('Remaining count:', db.count('users'));

console.log('\n=== SQL QUERY ===');
// Execute SQL directly — full SELECT/WHERE/ORDER BY support
const results = db.query('SELECT * FROM users WHERE age > 25 ORDER BY name');
results.forEach(row => console.log(`  ${row.name} (age ${row.age})`));

console.log('\n=== HELPER METHODS ===');
// findOne — returns first match or null
const alice = db.findOne('users', "name = 'Alice'");
console.log('findOne:', alice ? alice.name : 'not found');

// findAll — returns all matches with optional sorting and limit
const allUsers = db.findAll('users', '', 'age DESC', 10);
console.log('findAll:', allUsers.map(u => u.name));

// findAll with WHERE clause
const adults = db.findAll('users', 'age >= 30');
console.log('Adults:', adults.map(u => u.name));

// countWhere — count matching documents
const count = db.countWhere('users', 'age > 25');
console.log('countWhere age > 25:', count);

// exists — check if a document exists by _id
console.log(`exists(${id1}):`, db.exists('users', id1));
console.log("exists('users_999'):", db.exists('users', 'users_999'));

console.log('\n=== BULK OPERATIONS ===');
// updateMany — update all matching documents, returns count
const updated = db.updateMany('users', 'age < 30', { tier: 'standard' });
console.log('updateMany: updated', updated, 'users');

// deleteMany — delete all matching documents, returns count
const removed = db.deleteMany('users', "tier = 'standard'");
console.log('deleteMany: removed', removed, 'users');

console.log('\nFinal count:', db.count('users'));

db.close();
console.log('\nDone!');
