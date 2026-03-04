# OverDrive InCode SDK — Java v1.3.0

**Embeddable document database — like SQLite for JSON.**

> **v1.3.0** — Security hardened: `openEncrypted`, `querySafe` SQL injection prevention, auto file permission hardening, `backup`, `cleanupWal`, thread-safe `OverDriveSafe` wrapper.

## Install

### Maven

```xml
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.3.0</version>
</dependency>
```

### Gradle

```groovy
implementation 'com.afot:overdrive-db:1.3.0'
```

Place the native library on your `java.library.path`:
- Windows: `overdrive.dll`
- Linux: `liboverdrive.so`
- macOS: `liboverdrive.dylib`

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest).

## Quick Start

```java
import com.afot.overdrive.OverDrive;
import java.util.Map;
import java.util.List;

public class Main {
    public static void main(String[] args) throws Exception {
        // Open — file permissions auto-hardened (Windows ACL / chmod 600)
        try (OverDrive db = OverDrive.open("myapp.odb")) {
            db.createTable("users");

            String id = db.insert("users", Map.of(
                "name", "Alice", "age", 30, "email", "alice@example.com"
            ));
            System.out.println("Inserted: " + id);

            // ✅ Safe parameterized query — blocks SQL injection
            List<Map<String, Object>> results =
                db.querySafe("SELECT * FROM users WHERE age > ?", "25");
            for (var row : results) {
                System.out.printf("  %s — %s%n", row.get("name"), row.get("email"));
            }
        }
    }
}
```

## Security APIs (v1.3.0)

```java
import com.afot.overdrive.OverDrive;
import com.afot.overdrive.OverDriveSafe;

// 🔐 Open with encryption key from env (never hardcode!)
// export ODB_KEY="my-secret-32-char-key!!!!"  (bash)
// $env:ODB_KEY="my-secret-32-char-key!!!!"    (PowerShell)
OverDrive db = OverDrive.openEncrypted("app.odb", "ODB_KEY");

// 🛡️ SQL injection prevention
String userInput = "Alice'; DROP TABLE users--"; // malicious
try {
    db.querySafe("SELECT * FROM users WHERE name = ?", userInput);
} catch (OverDriveException e) {
    System.out.println("Blocked: " + e.getMessage()); // ✅ injection blocked
}

// 💾 Encrypted backup
db.backup("backups/app_2026-03-04.odb");

// 🗑️ WAL cleanup after commit
db.cleanupWal();

// 🧵 Thread-safe access (ReentrantReadWriteLock — reads parallel, writes exclusive)
try (OverDriveSafe safe = OverDriveSafe.open("app.odb")) {
    ExecutorService pool = Executors.newFixedThreadPool(4);
    pool.submit(() -> safe.query("SELECT * FROM users"));             // read lock
    pool.submit(() -> safe.insert("users", Map.of("name", "Bob"))); // write lock
}
```

## Full API

### Core
| Method | Description |
|---|---|
| `OverDrive.open(path)` | Open or create database (auto-hardens permissions) |
| `OverDrive.openEncrypted(path, keyEnvVar)` | 🔐 Open with key from env var |
| `db.close()` | Close (also via try-with-resources) |
| `db.sync()` | Force flush to disk |
| `db.getPath()` | Get database file path |
| `OverDrive.version()` | SDK version |

### Tables & CRUD
| Method | Description |
|---|---|
| `db.createTable(name)` | Create a table |
| `db.dropTable(name)` | Drop a table |
| `db.listTables()` | List all tables |
| `db.tableExists(name)` | Check if table exists |
| `db.insert(table, doc)` | Insert `Map`, returns `_id` |
| `db.insertMany(table, docs)` | Batch insert |
| `db.get(table, id)` | Get by `_id` |
| `db.update(table, id, updates)` | Update fields |
| `db.delete(table, id)` | Delete by `_id` |
| `db.count(table)` | Count documents |

### Query & Security
| Method | Description |
|---|---|
| `db.query(sql)` | Execute SQL (trusted input only) |
| `db.querySafe(sql, params...)` | ✅ Parameterized query (user input safe) |
| `db.search(table, text)` | Full-text search |
| `db.backup(destPath)` | 💾 Encrypted backup |
| `db.cleanupWal()` | 🗑️ Delete stale WAL file |

### Thread-safe Wrapper (`OverDriveSafe`)
| Method | Description |
|---|---|
| `OverDriveSafe.open(path)` | Open as thread-safe |
| `OverDriveSafe.openEncrypted(path, key)` | Encrypted + thread-safe |
| All `OverDrive` methods | Automatically locked (read/write locks) |

## Requirements

- Java 11+
- JNA 5.14+
- Native library on `java.library.path`

## Links

- [Full Documentation](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/)
- [GitHub](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK)
- [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases)
- [Security Guide](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/blob/main/docs/architecture.md#security-model)

## License

MIT / Apache-2.0
