# Java SDK Guide — OverDrive-DB

## Installation

### Maven

Add the repository and dependency to your `pom.xml`:

```xml
<repositories>
    <repository>
        <id>github</id>
        <url>https://maven.pkg.github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK</url>
    </repository>
</repositories>

<dependencies>
    <dependency>
        <groupId>com.afot</groupId>
        <artifactId>overdrive-db</artifactId>
        <version>1.0.1</version>
    </dependency>
</dependencies>
```

## Quick Start

```java
import com.afot.overdrive.OverDrive;
import java.util.List;
import java.util.Map;

public class App {
    public static void main(String[] args) {
        // Use try-with-resources for automatic closing
        try (OverDrive db = new OverDrive("myapp.odb")) {
            
            // Create a table
            db.createTable("users");
            
            // Insert a document (JSON string)
            String json = "{\"name\":\"Alice\", \"age\":30}";
            String id = db.insert("users", json);
            System.out.println("Inserted: " + id);
            
            // SQL Query
            String results = db.query("SELECT * FROM users");
            System.out.println("Results: " + results);
            
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

## API Reference

### `new OverDrive(String path)`
Opens or creates a database at the given path.

### `db.close()`
Closes the database.

### `db.createTable(String name)`
Creates a new table.

### `db.insert(String table, String json)`
Inserts a JSON document into a table. Returns the document ID.

### `db.query(String sql)`
Executes a SQL query. Returns results as a JSON string.

### `db.search(String table, String text)`
Full-text search across a table.

### `db.count(String table)`
Returns the number of records in a table.

---

#afot #OverDriveDb #JavaSDK #Maven #EmbeddedDB
