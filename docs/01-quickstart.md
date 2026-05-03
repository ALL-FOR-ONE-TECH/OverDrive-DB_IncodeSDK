# Quick Start Guide

This guide gets you from zero to a working database in under 5 minutes.

---

## Step 1 — Install

Pick your language:

### Python
```bash
pip install overdrive-db
```

### Node.js
```bash
npm install overdrive-db
```

### Java (Maven)
```xml
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.4.0</version>
</dependency>
```

### Go
```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@v1.4.0
```

### Rust
```toml
# Cargo.toml
[dependencies]
overdrive-db = "1.4.0"
```

> **Rust note:** The crate dynamically loads the native library at runtime.
> Download `overdrive.dll` / `liboverdrive.so` / `liboverdrive.dylib` from
> [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest)
> and place it in your project directory.

---

## Step 2 — Open a Database

A database is just a file. If it doesn't exist, it's created automatically.

### Python
```python
from overdrive import OverDrive

db = OverDrive.open("myapp.odb")
print("Database opened!")
db.close()
```

### Node.js
```javascript
const { OverDrive } = require('overdrive-db');

const db = OverDrive.open('myapp.odb');
console.log('Database opened!');
db.close();
```

### Java
```java
import com.afot.overdrive.OverDrive;

try (OverDrive db = OverDrive.open("myapp.odb")) {
    System.out.println("Database opened!");
}  // auto-closes
```

### Go
```go
package main

import (
    "fmt"
    overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"
)

func main() {
    db, err := overdrive.Open("myapp.odb")
    if err != nil { panic(err) }
    defer db.Close()
    fmt.Println("Database opened!")
}
```

### Rust
```rust
use overdrive::OverDriveDB;

fn main() {
    let mut db = OverDriveDB::open("myapp.odb").unwrap();
    println!("Database opened!");
    db.close().unwrap();
}
```

---

## Step 3 — Insert Your First Document

Tables are **created automatically** on first insert. No `CREATE TABLE` needed.

### Python
```python
from overdrive import OverDrive

db = OverDrive.open("myapp.odb")

# Insert a document — table "users" is auto-created
id = db.insert("users", {
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com",
    "tags": ["developer", "admin"]
})

print(f"Inserted with ID: {id}")  # → "users_1"
db.close()
```

### Node.js
```javascript
const { OverDrive } = require('overdrive-db');
const db = OverDrive.open('myapp.odb');

const id = db.insert('users', {
    name: 'Alice',
    age: 30,
    email: 'alice@example.com',
    tags: ['developer', 'admin']
});

console.log(`Inserted with ID: ${id}`);  // → "users_1"
db.close();
```

### Java
```java
import com.afot.overdrive.OverDrive;
import java.util.Map;
import java.util.List;

try (OverDrive db = OverDrive.open("myapp.odb")) {
    String id = db.insert("users", Map.of(
        "name", "Alice",
        "age", 30,
        "email", "alice@example.com"
    ));
    System.out.println("Inserted with ID: " + id);  // → "users_1"
}
```

### Go
```go
db, _ := overdrive.Open("myapp.odb")
defer db.Close()

id, err := db.Insert("users", map[string]any{
    "name":  "Alice",
    "age":   30,
    "email": "alice@example.com",
})
fmt.Printf("Inserted with ID: %s\n", id)  // → "users_1"
```

---

## Step 4 — Query Your Data

Use SQL to query your JSON documents.

### Python
```python
db = OverDrive.open("myapp.odb")

# Insert some data
db.insert("users", {"name": "Alice", "age": 30, "city": "London"})
db.insert("users", {"name": "Bob",   "age": 25, "city": "Paris"})
db.insert("users", {"name": "Carol", "age": 35, "city": "London"})

# Query with SQL
results = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name")

for user in results:
    print(f"  {user['name']} — age {user['age']} — {user['city']}")

# Output:
#   Alice — age 30 — London
#   Carol — age 35 — London

db.close()
```

### Node.js
```javascript
const db = OverDrive.open('myapp.odb');

db.insert('users', { name: 'Alice', age: 30, city: 'London' });
db.insert('users', { name: 'Bob',   age: 25, city: 'Paris' });
db.insert('users', { name: 'Carol', age: 35, city: 'London' });

const results = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name");
results.forEach(u => console.log(`${u.name} — age ${u.age}`));

db.close();
```

---

## Step 5 — Get, Update, Delete

### Python
```python
db = OverDrive.open("myapp.odb")

# Insert
id = db.insert("products", {"name": "Laptop", "price": 999, "stock": 10})

# Get by ID
product = db.get("products", id)
print(product)  # {'_id': 'products_1', 'name': 'Laptop', 'price': 999, 'stock': 10}

# Update — only the fields you specify change
db.update("products", id, {"price": 899, "on_sale": True})

# Verify update
product = db.get("products", id)
print(product["price"])  # 899

# Delete
db.delete("products", id)
print(db.count("products"))  # 0

db.close()
```

---

## Step 6 — Use the Helper Methods

v1.4 adds convenient helpers so you don't have to write SQL for common patterns.

### Python
```python
db = OverDrive.open("myapp.odb")

# Seed data
for i in range(10):
    db.insert("orders", {
        "customer": f"customer_{i}",
        "amount": (i + 1) * 50,
        "status": "pending" if i % 2 == 0 else "shipped"
    })

# findOne — first match
order = db.findOne("orders", "status = 'pending'")
print(order["customer"])  # customer_0

# findAll — all matches with sorting
pending = db.findAll("orders", "status = 'pending'", order_by="amount DESC", limit=3)
print(f"Top 3 pending: {[o['amount'] for o in pending]}")

# countWhere
n = db.countWhere("orders", "status = 'shipped'")
print(f"Shipped: {n}")

# updateMany — bulk update, returns count
updated = db.updateMany("orders", "status = 'pending'", {"status": "processing"})
print(f"Updated {updated} orders")

# deleteMany — bulk delete, returns count
deleted = db.deleteMany("orders", "amount < 100")
print(f"Deleted {deleted} small orders")

# exists — check by ID
first_id = db.findOne("orders")["_id"]
print(db.exists("orders", first_id))  # True
print(db.exists("orders", "orders_999"))  # False

db.close()
```

---

## Step 7 — Transactions

Wrap multiple operations in a transaction. Auto-commits on success, auto-rolls back on error.

### Python
```python
db = OverDrive.open("myapp.odb")

db.insert("accounts", {"id": "acc_alice", "balance": 1000})
db.insert("accounts", {"id": "acc_bob",   "balance": 500})

# Callback pattern — recommended
def transfer(txn):
    db.updateMany("accounts", "id = 'acc_alice'", {"balance": 900})
    db.updateMany("accounts", "id = 'acc_bob'",   {"balance": 600})
    return "transfer complete"

result = db.transaction(transfer)
print(result)  # "transfer complete"

# If an exception is raised inside the callback, it auto-rolls back:
try:
    def bad_transfer(txn):
        db.updateMany("accounts", "id = 'acc_alice'", {"balance": 0})
        raise ValueError("Insufficient funds!")  # ← triggers rollback

    db.transaction(bad_transfer)
except ValueError as e:
    print(f"Rolled back: {e}")
    # Alice's balance is still 900 — not 0

db.close()
```

---

## Complete Example — A Mini Blog

```python
from overdrive import OverDrive
from datetime import datetime

db = OverDrive.open("blog.odb")

# Create some posts (tables auto-created)
db.insert("posts", {
    "title": "Getting Started with OverDrive-DB",
    "body": "OverDrive is an embeddable database...",
    "author": "alice",
    "published": True,
    "views": 0,
    "created_at": "2026-04-16"
})

db.insert("posts", {
    "title": "Advanced SQL Queries",
    "body": "Let's explore the SQL engine...",
    "author": "bob",
    "published": False,
    "views": 0,
    "created_at": "2026-04-17"
})

db.insert("comments", {
    "post_id": "posts_1",
    "author": "carol",
    "text": "Great article!",
    "created_at": "2026-04-16"
})

# Query published posts
published = db.findAll("posts", "published = true", order_by="created_at DESC")
print(f"Published posts: {len(published)}")

# Increment view count
db.updateMany("posts", "title = 'Getting Started with OverDrive-DB'", {"views": 1})

# Full-text search
results = db.query("SELECT * FROM posts")
print(f"Total posts: {len(results)}")

# Stats
print(f"Tables: {db.list_tables()}")
print(f"Post count: {db.count('posts')}")
print(f"Comment count: {db.count('comments')}")

db.close()
print("Done!")
```

---

## What's Next?

- **[02 — Core Concepts](02-concepts.md)** — Understand how OverDrive works
- **[03 — SQL Reference](03-sql-reference.md)** — Full SQL syntax guide
- **[04 — Storage Engines](04-storage-engines.md)** — RAM, Vector, Graph, and more
- **[07 — Python SDK](07-python-sdk.md)** — Full Python API reference
- **[08 — Node.js SDK](08-nodejs-sdk.md)** — Full Node.js API reference
