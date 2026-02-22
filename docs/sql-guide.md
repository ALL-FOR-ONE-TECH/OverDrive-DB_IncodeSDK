# SQL Reference Guide

OverDrive includes a built-in SQL parser and executor. Queries run entirely in-process — no server, no network.

## SELECT

```sql
SELECT * FROM users
SELECT name, age FROM users
SELECT * FROM users WHERE age > 25
SELECT * FROM users WHERE name = 'Alice'
SELECT * FROM users WHERE age > 25 AND name = 'Alice'
SELECT * FROM users ORDER BY age ASC
SELECT * FROM users ORDER BY name DESC
SELECT * FROM users LIMIT 10
SELECT * FROM users LIMIT 10 OFFSET 20
SELECT * FROM users WHERE age > 25 ORDER BY name DESC LIMIT 5
```

### Column Selection

```sql
-- All columns
SELECT * FROM users

-- Specific columns
SELECT name, email FROM users
```

### WHERE Clause

Supported operators:

| Operator | Example | Description |
|---|---|---|
| `=` | `WHERE name = 'Alice'` | Equality |
| `!=` | `WHERE status != 'deleted'` | Not equal |
| `>` | `WHERE age > 25` | Greater than |
| `<` | `WHERE price < 100` | Less than |
| `>=` | `WHERE age >= 18` | Greater or equal |
| `<=` | `WHERE score <= 50` | Less or equal |

### Logical Operators

```sql
-- AND: both conditions must be true
SELECT * FROM users WHERE age > 25 AND active = true

-- OR: either condition can be true
SELECT * FROM products WHERE price < 10 OR category = 'sale'
```

### ORDER BY

```sql
-- Ascending (default)
SELECT * FROM users ORDER BY name
SELECT * FROM users ORDER BY name ASC

-- Descending
SELECT * FROM users ORDER BY age DESC
```

### LIMIT and OFFSET

```sql
-- First 10 results
SELECT * FROM users LIMIT 10

-- Skip 20, get next 10 (pagination)
SELECT * FROM users LIMIT 10 OFFSET 20
```

---

## Aggregations

```sql
SELECT COUNT(*) FROM users
SELECT COUNT(*) FROM users WHERE active = true
SELECT AVG(age) FROM users
SELECT SUM(price) FROM products
SELECT MIN(age) FROM users
SELECT MAX(score) FROM results
```

| Function | Description |
|---|---|
| `COUNT(*)` | Count all documents |
| `SUM(column)` | Sum numeric values |
| `AVG(column)` | Average of numeric values |
| `MIN(column)` | Minimum value |
| `MAX(column)` | Maximum value |

---

## INSERT

```sql
INSERT INTO users VALUES {"name": "Alice", "age": 30, "email": "alice@example.com"}
INSERT INTO products VALUES {"name": "Laptop", "price": 999.99}
```

> **Note:** The values must be valid JSON.

---

## UPDATE

```sql
UPDATE users SET {"age": 31} WHERE name = 'Alice'
UPDATE products SET {"price": 899.99, "on_sale": true} WHERE name = 'Laptop'
```

> **Note:** The SET clause uses JSON to specify updated fields.

---

## DELETE

```sql
DELETE FROM users WHERE name = 'Alice'
DELETE FROM products WHERE price < 10
```

---

## DDL (Data Definition)

### CREATE TABLE

```sql
CREATE TABLE users
CREATE TABLE products
CREATE TABLE orders
```

### DROP TABLE

```sql
DROP TABLE old_data
```

> ⚠️ **Warning:** This permanently deletes the table and all its data.

### SHOW TABLES

```sql
SHOW TABLES
```

Returns a list of all table names in the database.

---

## Data Types in WHERE

OverDrive automatically infers types:

| Type | Example |
|---|---|
| String | `WHERE name = 'Alice'` |
| Number (int) | `WHERE age > 25` |
| Number (float) | `WHERE price > 9.99` |
| Boolean | `WHERE active = true` |

---

## Full Example

```rust
use overdrive::OverDriveDB;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = OverDriveDB::open("shop.odb")?;

    // Setup
    db.query("CREATE TABLE products")?;

    // Insert data
    db.query(r#"INSERT INTO products VALUES {"name": "Laptop", "price": 999, "category": "electronics"}"#)?;
    db.query(r#"INSERT INTO products VALUES {"name": "Mouse", "price": 29, "category": "electronics"}"#)?;
    db.query(r#"INSERT INTO products VALUES {"name": "Desk", "price": 299, "category": "furniture"}"#)?;
    db.query(r#"INSERT INTO products VALUES {"name": "Chair", "price": 199, "category": "furniture"}"#)?;

    // Query
    let expensive = db.query("SELECT * FROM products WHERE price > 100 ORDER BY price DESC")?;
    println!("Expensive items: {:?}", expensive.rows);

    // Aggregate
    let stats = db.query("SELECT COUNT(*), AVG(price) FROM products")?;
    println!("Stats: {:?}", stats.rows);

    // Update
    db.query("UPDATE products SET {\"price\": 899} WHERE name = 'Laptop'")?;

    // Delete
    db.query("DELETE FROM products WHERE price < 50")?;

    // Show tables
    let tables = db.query("SHOW TABLES")?;
    println!("Tables: {:?}", tables.rows);

    db.close()?;
    Ok(())
}
```
