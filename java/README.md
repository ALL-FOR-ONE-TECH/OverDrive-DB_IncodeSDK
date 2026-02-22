# OverDrive InCode SDK — Java

**Embeddable document database — like SQLite for JSON.**

## Install

### Maven

```xml
<dependency>
    <groupId>com.afot</groupId>
    <artifactId>overdrive-db</artifactId>
    <version>1.0.1</version>
</dependency>
```

### Gradle

```groovy
implementation 'com.afot:overdrive-db:1.0.1'
```

Then download the native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) and place it on your `java.library.path`.

## Quick Start

```java
import com.afot.overdrive.OverDrive;
import java.util.Map;
import java.util.List;

public class Main {
    public static void main(String[] args) {
        // AutoCloseable — try-with-resources
        try (OverDrive db = OverDrive.open("myapp.odb")) {
            db.createTable("users");

            // Insert
            String id = db.insert("users", Map.of(
                "name", "Alice",
                "age", 30,
                "email", "alice@example.com"
            ));
            System.out.println("Inserted: " + id);

            // SQL Query
            List<Map<String, Object>> results =
                db.query("SELECT * FROM users WHERE age > 25");
            for (Map<String, Object> row : results) {
                System.out.printf("  %s — %s%n", row.get("name"), row.get("email"));
            }

            // Count
            System.out.println("Total: " + db.count("users"));
        }
    }
}
```

## API

| Method | Description |
|---|---|
| `OverDrive.open(path)` | Open or create a database |
| `db.close()` | Close (also via try-with-resources) |
| `db.createTable(name)` | Create a table |
| `db.dropTable(name)` | Drop a table |
| `db.listTables()` | List all tables |
| `db.tableExists(name)` | Check if table exists |
| `db.insert(table, doc)` | Insert, returns `_id` |
| `db.insertMany(table, docs)` | Batch insert |
| `db.get(table, id)` | Get by `_id` |
| `db.update(table, id, updates)` | Update fields |
| `db.delete(table, id)` | Delete by `_id` |
| `db.count(table)` | Count docs |
| `db.query(sql)` | SQL query |
| `db.search(table, text)` | Full-text search |
| `OverDrive.version()` | SDK version |

## Requirements

- Java 11+
- JNA 5.14+
- Native library (`.dll` / `.so` / `.dylib`) on library path

## License

MIT / Apache-2.0
