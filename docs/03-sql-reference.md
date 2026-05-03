# SQL Reference

OverDrive has a built-in SQL engine. You write standard SQL, it runs on your JSON documents — no schema required.

---

## SELECT

### Basic Select

```sql
-- All documents in a table
SELECT * FROM users

-- Specific fields
SELECT name, age FROM users

-- With a condition
SELECT * FROM users WHERE age > 25

-- Sorted
SELECT * FROM users ORDER BY name ASC
SELECT * FROM users ORDER BY age DESC

-- Limited
SELECT * FROM users LIMIT 10
SELECT * FROM users LIMIT 10 OFFSET 20

-- Combined
SELECT * FROM users WHERE age > 18 ORDER BY name ASC LIMIT 5
```

### WHERE Operators

| Operator | Example | Meaning |
|----------|---------|---------|
| `=` | `WHERE name = 'Alice'` | Equals |
| `!=` | `WHERE status != 'deleted'` | Not equals |
| `>` | `WHERE age > 25` | Greater than |
| `<` | `WHERE price < 100` | Less than |
| `>=` | `WHERE age >= 18` | Greater or equal |
| `<=` | `WHERE score <= 50` | Less or equal |
| `AND` | `WHERE age > 18 AND active = true` | Both conditions |
| `OR` | `WHERE city = 'London' OR city = 'Paris'` | Either condition |
| `LIKE` | `WHERE name LIKE 'Al%'` | Pattern match (`%` = any chars, `_` = one char) |

### Examples

```python
# Python — all these work
db.query("SELECT * FROM users")
db.query("SELECT * FROM users WHERE age > 25")
db.query("SELECT * FROM users WHERE name = 'Alice'")
db.query("SELECT * FROM users WHERE age > 18 AND active = true")
db.query("SELECT * FROM users WHERE city = 'London' OR city = 'Paris'")
db.query("SELECT * FROM users ORDER BY age DESC LIMIT 10")
db.query("SELECT * FROM users WHERE age > 18 ORDER BY name LIMIT 5 OFFSET 10")
db.query("SELECT * FROM users WHERE name LIKE 'Al%'")
```

---

## Aggregations

```sql
SELECT COUNT(*) FROM users
SELECT COUNT(*) FROM users WHERE active = true
SELECT SUM(price) FROM orders
SELECT AVG(age) FROM users
SELECT MIN(price) FROM products
SELECT MAX(score) FROM results
```

### In Code

```python
# Python
rows = db.query("SELECT COUNT(*) FROM users")
count = rows[0]["COUNT(*)"]  # or rows[0]["count(*)"]

rows = db.query("SELECT AVG(age) FROM users WHERE active = true")
avg_age = rows[0]["AVG(age)"]

rows = db.query("SELECT MIN(price), MAX(price), SUM(price) FROM products")
stats = rows[0]
```

```javascript
// Node.js
const rows = db.query("SELECT COUNT(*) FROM users");
const count = rows[0]["COUNT(*)"];
```

---

## INSERT

```sql
INSERT INTO users VALUES {"name": "Alice", "age": 30}
INSERT INTO products VALUES {"name": "Laptop", "price": 999.99, "in_stock": true}
INSERT INTO logs VALUES {"level": "info", "message": "App started", "ts": 1700000000}
```

> The values must be valid JSON. Use the SDK's `insert()` method for most cases — it handles serialization for you.

---

## UPDATE

```sql
UPDATE users SET {"age": 31} WHERE name = 'Alice'
UPDATE products SET {"price": 899, "on_sale": true} WHERE name = 'Laptop'
UPDATE orders SET {"status": "shipped"} WHERE status = 'pending'
```

> The `SET` clause uses JSON to specify which fields to update. Other fields are unchanged.

---

## DELETE

```sql
DELETE FROM users WHERE name = 'Alice'
DELETE FROM products WHERE price < 10
DELETE FROM logs WHERE level = 'debug'
```

> ⚠️ Always include a `WHERE` clause. Without it, all documents are deleted.

---

## DDL (Table Management)

```sql
-- Create a table
CREATE TABLE users
CREATE TABLE products

-- Drop a table (permanent!)
DROP TABLE old_data

-- List all tables
SHOW TABLES
```

---

## Data Types in WHERE

OverDrive automatically infers types from your JSON:

| JSON Type | SQL Example |
|-----------|-------------|
| String | `WHERE name = 'Alice'` (use single quotes) |
| Integer | `WHERE age > 25` |
| Float | `WHERE price > 9.99` |
| Boolean | `WHERE active = true` or `WHERE active = false` |
| Null | `WHERE email = null` |

---

## Parameterized Queries (Safe for User Input)

**Never** build SQL by concatenating user input. Use `querySafe()` instead:

```python
# ❌ DANGEROUS — SQL injection vulnerable
name = user_input  # could be: "'; DROP TABLE users; --"
db.query(f"SELECT * FROM users WHERE name = '{name}'")

# ✅ SAFE — use querySafe with ? placeholders
results = db.querySafe("SELECT * FROM users WHERE name = ?", name)
results = db.querySafe("SELECT * FROM users WHERE age > ? AND city = ?", "25", "London")
```

```javascript
// Node.js
const results = db.querySafe("SELECT * FROM users WHERE name = ?", [userName]);
```

```java
// Java
List<Map<String, Object>> results = db.querySafe(
    "SELECT * FROM users WHERE name = ?", userName
);
```

```go
// Go
results, err := db.QuerySafe("SELECT * FROM users WHERE name = ?", userName)
```

---

## Full Example

```python
from overdrive import OverDrive

db = OverDrive.open("shop.odb")

# Insert products
db.insert("products", {"name": "Laptop",  "price": 999,  "category": "electronics", "stock": 5})
db.insert("products", {"name": "Mouse",   "price": 29,   "category": "electronics", "stock": 50})
db.insert("products", {"name": "Desk",    "price": 299,  "category": "furniture",   "stock": 10})
db.insert("products", {"name": "Chair",   "price": 199,  "category": "furniture",   "stock": 15})
db.insert("products", {"name": "Pen",     "price": 2,    "category": "office",      "stock": 200})
db.insert("products", {"name": "Notepad", "price": 5,    "category": "office",      "stock": 100})

# All products
all_products = db.query("SELECT * FROM products")
print(f"Total: {len(all_products)}")

# Electronics only, sorted by price
electronics = db.query(
    "SELECT * FROM products WHERE category = 'electronics' ORDER BY price DESC"
)
for p in electronics:
    print(f"  ${p['price']} — {p['name']}")

# Count by category
count = db.query("SELECT COUNT(*) FROM products WHERE category = 'furniture'")
print(f"Furniture items: {count[0]['COUNT(*)']}")

# Average price
avg = db.query("SELECT AVG(price) FROM products")
print(f"Average price: ${avg[0]['AVG(price)']:.2f}")

# Update — apply sale discount
db.query("UPDATE products SET {\"on_sale\": true} WHERE price > 100")

# Delete cheap items
db.query("DELETE FROM products WHERE price < 5")

# Remaining
remaining = db.query("SELECT * FROM products ORDER BY price")
print(f"After cleanup: {len(remaining)} products")

db.close()
```

---

## SQL Quick Reference Card

```
SELECT * FROM table
SELECT col1, col2 FROM table
SELECT * FROM table WHERE condition
SELECT * FROM table WHERE cond1 AND cond2
SELECT * FROM table WHERE cond1 OR cond2
SELECT * FROM table ORDER BY col ASC|DESC
SELECT * FROM table LIMIT n
SELECT * FROM table LIMIT n OFFSET m
SELECT COUNT(*) FROM table
SELECT SUM(col), AVG(col), MIN(col), MAX(col) FROM table
INSERT INTO table VALUES {json}
UPDATE table SET {json} WHERE condition
DELETE FROM table WHERE condition
CREATE TABLE name
DROP TABLE name
SHOW TABLES
```
