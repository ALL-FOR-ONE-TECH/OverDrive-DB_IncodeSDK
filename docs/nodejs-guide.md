# Node.js SDK Guide

## Requirements

- Node.js 14+
- `ffi-napi` and `ref-napi` packages
- The OverDrive shared library (`overdrive.dll` / `liboverdrive.so` / `liboverdrive.dylib`)

## Installation

### From npm (coming soon)

```bash
npm install overdrive-db
```

### From source

```bash
# 1. Build the shared library
cd OverDrive-DB_SDK
cargo build --release

# 2. Copy the library next to the Node.js module
cp target/release/overdrive.dll nodejs/    # Windows
cp target/release/liboverdrive.so nodejs/  # Linux

# 3. Install Node dependencies
cd nodejs
npm install ffi-napi ref-napi

# 4. Test
node -e "const {OverDrive} = require('./index'); console.log(OverDrive.version())"
```

## Quick Start

```javascript
const { OverDrive } = require('overdrive-db');

const db = new OverDrive('myapp.odb');

db.createTable('users');

const id = db.insert('users', {
    name: 'Alice',
    email: 'alice@example.com',
    age: 30
});

console.log(`Created user: ${id}`);

const results = db.query('SELECT * FROM users WHERE age > 25');
console.table(results);

db.close();
```

## API

### Open / Close

```javascript
const db = new OverDrive('path/to/database.odb');
db.close();
```

### Tables

```javascript
db.createTable('users');
db.dropTable('old_table');
const tables = db.listTables();            // ['users', 'products']
const exists = db.tableExists('users');    // true
```

### Insert

```javascript
// Single document
const id = db.insert('users', { name: 'Alice', age: 30 });

// Multiple documents
const ids = db.insertMany('users', [
    { name: 'Bob', age: 25 },
    { name: 'Charlie', age: 35 },
]);
```

### Read

```javascript
const user = db.get('users', id);      // object or null
const count = db.count('users');        // number
```

### Update

```javascript
const updated = db.update('users', id, { age: 31 });
// true if found and updated
```

### Delete

```javascript
const deleted = db.delete('users', id);
// true if found and deleted
```

### SQL Queries

```javascript
// Simple — returns array of objects
const results = db.query('SELECT * FROM users WHERE age > 25');

// Full — returns object with metadata
const result = db.queryFull('SELECT COUNT(*) FROM users');
// { rows: [...], columns: [...], rows_affected: 0 }
```

### Search

```javascript
const matches = db.search('users', 'alice');
```

## Error Handling

```javascript
try {
    const db = new OverDrive('myapp.odb');
    db.createTable('users');
    db.createTable('users'); // throws Error
} catch (err) {
    console.error(`Database error: ${err.message}`);
}
```

## Full Example

```javascript
const { OverDrive } = require('overdrive-db');

const db = new OverDrive('shop.odb');

// Setup
db.createTable('products');

// Seed data
db.insertMany('products', [
    { name: 'Laptop', price: 999.99, category: 'electronics' },
    { name: 'Mouse', price: 29.99, category: 'electronics' },
    { name: 'Desk', price: 299.99, category: 'furniture' },
    { name: 'Chair', price: 199.99, category: 'furniture' },
]);

// Query
const expensive = db.query('SELECT * FROM products WHERE price > 100 ORDER BY price DESC');
console.log('Expensive items:');
console.table(expensive);

// Search
const matches = db.search('products', 'lap');
console.log(`Search results: ${matches.length}`);

// Stats
console.log(`Total products: ${db.count('products')}`);
console.log(`Tables: ${db.listTables()}`);
console.log(`SDK version: ${OverDrive.version()}`);

db.close();
```
