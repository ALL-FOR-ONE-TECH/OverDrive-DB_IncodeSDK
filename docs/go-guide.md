# Go SDK Guide — OverDrive-DB

## Installation

```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go
```

## Quick Start

```go
package main

import (
	"fmt"
	"log"
	"github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go"
)

func main() {
	// Open or create a database
	db, err := overdrive.Open("myapp.odb")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	// Create a table
	if err := db.CreateTable("users"); err != nil {
		log.Fatal(err)
	}

	// Insert a document
	user := map[string]any{
		"name":  "Alice",
		"email": "alice@example.com",
		"age":   30,
	}
	id, err := db.Insert("users", user)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Inserted user with ID: %s\n", id)

	// SQL Query
	results, err := db.Query("SELECT * FROM users WHERE age >= 30")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Query results: %v\n", results)
}
```

## API Reference

### `Open(path string)`
Opens an existing database or creates a new one.

### `db.Close()`
Closes the database handle.

### `db.CreateTable(name string)`
Creates a new table in the database.

### `db.Insert(table string, doc map[string]any)`
Inserts a JSON document (represented as a map) into the specified table. Returns the generated ID.

### `db.Query(sql string)`
Executes a SQL query and returns the results as a slice of maps.

### `db.Search(table string, text string)`
Performs a full-text search across the specified table.

### `db.Count(table string)`
Returns the total number of records in a table.

---

#afot #OverDriveDb #GoSDK #EmbeddedDB
