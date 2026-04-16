/**
 * OverDrive-DB — Basic CRUD Example (Node.js)
 */
const { OverDrive } = require('overdrive-db');

const db = OverDrive.open('example.odb');

// INSERT
const id1 = db.insert('users', { name: 'Alice', age: 30, email: 'alice@example.com' });
const id2 = db.insert('users', { name: 'Bob', age: 25, email: 'bob@example.com' });
const id3 = db.insert('users', { name: 'Charlie', age: 35, email: 'charlie@example.com' });
console.log('Inserted:', id1, id2, id3);

// GET
const user = db.get('users', id1);
console.log('Got user:', user);

// UPDATE
db.update('users', id1, { age: 31 });
console.log('Updated Alice age to 31');

// QUERY
const results = db.query('SELECT * FROM users WHERE age > 25 ORDER BY name');
console.log('Users over 25:', results);

// HELPERS
console.log('findOne:', db.findOne('users', "name = 'Alice'"));
console.log('findAll:', db.findAll('users', '', 'age DESC', 10));
console.log('countWhere:', db.countWhere('users', 'age > 25'));
console.log('exists:', db.exists('users', id1));

// DELETE
db.delete('users', id3);
console.log('Deleted', id3);

// BULK
console.log('updateMany:', db.updateMany('users', 'age < 30', { status: 'young' }));
console.log('deleteMany:', db.deleteMany('users', 'age > 100'));

db.close();
console.log('Done!');
