// OverDrive-DB Go SDK — Sample Code (v1.4.3)
// Install: go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest
package main

import (
	"fmt"
	"log"

	overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"
)

func main() {
	// ── 1. Open database ──────────────────────────
	db, err := overdrive.Open("myapp.odb")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	// ── 2. Insert documents (table auto-created) ──
	id, _ := db.Insert("users", map[string]any{
		"name": "Alice", "age": 30, "role": "admin",
	})
	fmt.Println("Inserted:", id)

	db.Insert("users", map[string]any{
		"name": "Bob", "age": 25, "role": "user",
	})
	db.Insert("users", map[string]any{
		"name": "Carol", "age": 35, "role": "admin",
	})

	// ── 3. Query with SQL ─────────────────────────
	result, _ := db.Query("SELECT * FROM users WHERE age > 28")
	fmt.Printf("Users over 28: %d\n", len(result.Rows))

	// ── 4. Count ──────────────────────────────────
	count, _ := db.Count("users")
	fmt.Printf("Total users: %d\n", count)

	// ── 5. Get by ID ──────────────────────────────
	doc, _ := db.Get("users", id)
	if doc != nil {
		fmt.Printf("Found: %v\n", doc["name"])
	}

	// ── 6. Version ────────────────────────────────
	fmt.Println("SDK version:", overdrive.Version())

	fmt.Println("\n✅ Done!")
}
