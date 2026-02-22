# OverDrive InCode SDK — Go

**Embeddable document database — like SQLite for JSON.**

## Install

```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go
```

Then download the native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest) and place it in `lib/` directory.

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go"
)

func main() {
    db, err := overdrive.Open("myapp.odb")
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    db.CreateTable("users")

    id, _ := db.Insert("users", map[string]any{
        "name": "Alice",
        "age":  30,
        "tags": []string{"admin", "developer"},
    })
    fmt.Println("Inserted:", id)

    result, _ := db.Query("SELECT * FROM users WHERE age > 25")
    for _, row := range result.Rows {
        fmt.Printf("  %s (age %v)\n", row["name"], row["age"])
    }

    count, _ := db.Count("users")
    fmt.Println("Total users:", count)
}
```

## API

| Function | Description |
|---|---|
| `overdrive.Open(path)` | Open or create a database |
| `db.Close()` | Close the database |
| `db.CreateTable(name)` | Create a table |
| `db.DropTable(name)` | Drop a table |
| `db.ListTables()` | List all tables |
| `db.TableExists(name)` | Check if table exists |
| `db.Insert(table, doc)` | Insert document, returns `_id` |
| `db.Get(table, id)` | Get by `_id` |
| `db.Update(table, id, updates)` | Update fields |
| `db.Delete(table, id)` | Delete by `_id` |
| `db.Count(table)` | Count documents |
| `db.Query(sql)` | Execute SQL query |
| `db.Search(table, text)` | Full-text search |
| `overdrive.Version()` | Get SDK version |

## License

MIT / Apache-2.0
