# OverDrive-DB — Node.js SDK

**A high-performance, embeddable hybrid SQL+NoSQL document database — built for speed and version control. Like SQLite, but for JSON.**

Use OverDrive-DB in your Node.js applications whenever you need a fast, serverless database that supports both document queries and relational SQL.

## Install

```bash
npm install overdrive-db
```

The native binary will be **automatically downloaded** from GitHub Releases during install.

## Quick Start

```javascript
const { OverDrive } = require('overdrive-db');

const db = new OverDrive('myapp.odb');

// Create table
db.createTable('products');

// Insert
const id = db.insert('products', {
    name: 'Laptop',
    price: 999.99,
    specs: { ram: '16GB', cpu: 'i7' }
});

// SQL Query
const results = db.query('SELECT * FROM products WHERE price > 500');
console.table(results);

// Batch insert
const ids = db.insertMany('products', [
    { name: 'Mouse', price: 29.99 },
    { name: 'Keyboard', price: 79.99 },
]);

// Count
console.log(`Total products: ${db.count('products')}`);

// Clean up
db.close();
```

## API

| Method | Description |
|---|---|
| `new OverDrive(path)` | Open or create a database |
| `db.close()` | Close the database |
| `db.createTable(name)` | Create a table |
| `db.dropTable(name)` | Drop a table |
| `db.listTables()` | List all tables |
| `db.insert(table, doc)` | Insert a document, returns `_id` |
| `db.insertMany(table, docs)` | Batch insert |
| `db.get(table, id)` | Get by `_id` |
| `db.update(table, id, updates)` | Update fields |
| `db.delete(table, id)` | Delete by `_id` |
| `db.count(table)` | Count documents |
| `db.query(sql)` | Execute SQL query |
| `db.queryFull(sql)` | SQL query with full metadata |
| `db.search(table, text)` | Full-text search |
| `OverDrive.version()` | Get SDK version |

## TypeScript

Full TypeScript definitions are included (`index.d.ts`).

```typescript
import { OverDrive } from 'overdrive-db';
const db = new OverDrive('myapp.odb');
```

## Links

- [Full Documentation](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/nodejs-guide.md)
- [GitHub](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK)
- [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases)

## License

MIT / Apache-2.0
