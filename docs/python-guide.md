# Python SDK Guide

## Requirements

- Python 3.7+
- The OverDrive shared library (`overdrive.dll` / `liboverdrive.so` / `liboverdrive.dylib`)

## Installation

### From PyPI (coming soon)

```bash
pip install overdrive-db
```

### From source

```bash
# 1. Build the shared library
cd OverDrive-DB_SDK
cargo build --release

# 2. Copy the library next to the Python module
cp target/release/overdrive.dll python/overdrive/   # Windows
cp target/release/liboverdrive.so python/overdrive/  # Linux
cp target/release/liboverdrive.dylib python/overdrive/  # macOS

# 3. Use it
cd python
python -c "from overdrive import OverDrive; print(OverDrive.version())"
```

## Quick Start

```python
from overdrive import OverDrive

db = OverDrive("myapp.odb")
db.create_table("users")

# Insert
user_id = db.insert("users", {
    "name": "Alice",
    "email": "alice@example.com",
    "age": 30
})
print(f"Created user: {user_id}")

# Query
results = db.query("SELECT * FROM users WHERE age > 25")
for row in results:
    print(f"  {row['name']} — {row['email']}")

db.close()
```

## Context Manager

Use `with` for automatic cleanup:

```python
with OverDrive("myapp.odb") as db:
    db.create_table("logs")
    db.insert("logs", {"message": "App started", "level": "info"})
    # Database closes automatically when leaving the with block
```

## API

### Open / Close

```python
db = OverDrive("path/to/database.odb")  # Open or create
db.close()                               # Close explicitly

# Or with context manager
with OverDrive("path.odb") as db:
    pass  # Auto-close
```

### Tables

```python
db.create_table("users")
db.drop_table("old_table")
tables = db.list_tables()         # ["users", "products"]
exists = db.table_exists("users") # True
```

### Insert

```python
# Single document
doc_id = db.insert("users", {"name": "Alice", "age": 30})

# Multiple documents
ids = db.insert_many("users", [
    {"name": "Bob", "age": 25},
    {"name": "Charlie", "age": 35},
])
```

### Read

```python
# Get by ID
user = db.get("users", doc_id)  # dict or None

# Count
count = db.count("users")  # int
```

### Update

```python
updated = db.update("users", doc_id, {"age": 31})
# True if found and updated, False if not found
```

### Delete

```python
deleted = db.delete("users", doc_id)
# True if found and deleted
```

### SQL Queries

```python
# Simple query — returns list of dicts
results = db.query("SELECT * FROM users WHERE age > 25")

# Full query — returns dict with metadata
result = db.query_full("SELECT COUNT(*) FROM users")
# {"rows": [...], "columns": [...], "rows_affected": 0, ...}
```

### Search

```python
matches = db.search("users", "alice")
# Returns list of matching documents
```

## Error Handling

```python
from overdrive import OverDrive, OverDriveError

try:
    db = OverDrive("myapp.odb")
    db.create_table("users")
    db.create_table("users")  # Error: table already exists
except OverDriveError as e:
    print(f"Database error: {e}")
```

## Full Example

```python
from overdrive import OverDrive

with OverDrive("shop.odb") as db:
    # Setup
    db.create_table("products")

    # Seed data
    db.insert_many("products", [
        {"name": "Laptop", "price": 999.99, "category": "electronics"},
        {"name": "Mouse", "price": 29.99, "category": "electronics"},
        {"name": "Desk", "price": 299.99, "category": "furniture"},
        {"name": "Chair", "price": 199.99, "category": "furniture"},
        {"name": "Pen", "price": 2.99, "category": "office"},
    ])

    # Query by category
    electronics = db.query(
        "SELECT * FROM products WHERE category = 'electronics' ORDER BY price DESC"
    )
    print("Electronics:")
    for item in electronics:
        print(f"  ${item['price']:.2f} — {item['name']}")

    # Full-text search
    matches = db.search("products", "lap")
    print(f"\nSearch for 'lap': {len(matches)} results")

    # Stats
    print(f"\nTotal products: {db.count('products')}")
    print(f"Tables: {db.list_tables()}")
    print(f"SDK version: {OverDrive.version()}")
```
