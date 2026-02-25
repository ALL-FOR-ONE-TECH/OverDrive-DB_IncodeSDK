# OverDrive-DB — Python SDK

**A high-performance, embeddable hybrid SQL+NoSQL document database — built for speed and version control. Like SQLite, but for JSON.**

Use OverDrive-DB in your Python applications whenever you need a fast, serverless database that supports both document queries and relational SQL.

## Install

```bash
pip install overdrive-db
```

After installing, download the native library for your platform from
[GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest)
and place it in your project directory or on your system PATH.

| Platform | File |
|---|---|
| Windows x64 | `overdrive-windows-x64.dll` → rename to `overdrive.dll` |
| Linux x64 | `liboverdrive-linux-x64.so` → rename to `liboverdrive.so` |
| macOS ARM64 | `liboverdrive-macos-arm64.dylib` → rename to `liboverdrive.dylib` |

## Quick Start

```python
from overdrive import OverDrive

# Context manager for automatic cleanup
with OverDrive("myapp.odb") as db:
    db.create_table("users")

    # Insert
    user_id = db.insert("users", {
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30
    })

    # SQL Query
    results = db.query("SELECT * FROM users WHERE age > 25")
    for row in results:
        print(f"  {row['name']} — {row['email']}")

    # Full-text search
    matches = db.search("users", "alice")

    # Batch insert
    db.insert_many("users", [
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35},
    ])

    print(f"Total users: {db.count('users')}")
```

## API

| Method | Description |
|---|---|
| `OverDrive(path)` | Open or create a database |
| `db.close()` | Close the database |
| `db.create_table(name)` | Create a table |
| `db.drop_table(name)` | Drop a table |
| `db.list_tables()` | List all tables |
| `db.insert(table, doc)` | Insert a document, returns `_id` |
| `db.insert_many(table, docs)` | Batch insert |
| `db.get(table, id)` | Get by `_id` |
| `db.update(table, id, updates)` | Update fields |
| `db.delete(table, id)` | Delete by `_id` |
| `db.count(table)` | Count documents |
| `db.query(sql)` | Execute SQL query |
| `db.search(table, text)` | Full-text search |

## Links

- [Full Documentation](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/python-guide.md)
- [GitHub](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK)
- [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases)

## License

MIT / Apache-2.0
