//! Basic CRUD operations with OverDrive SDK
//!
//! Run: `cargo run --example basic_crud`

use overdrive::OverDriveDB;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OverDrive SDK: Basic CRUD ===\n");

    // Open (or create) a database
    let mut db = OverDriveDB::open("example_crud.odb")?;
    println!("✓ Database opened: {}", db.path());

    // Create a table
    if !db.table_exists("users")? {
        db.create_table("users")?;
        println!("✓ Table 'users' created");
    }

    // INSERT
    let id = db.insert("users", &json!({
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30,
        "tags": ["admin", "developer"]
    }))?;
    println!("✓ Inserted user: {}", id);

    // GET
    if let Some(user) = db.get("users", &id)? {
        println!("✓ Retrieved: {} (age: {})", user["name"], user["age"]);
    }

    // UPDATE
    let updated = db.update("users", &id, &json!({
        "age": 31,
        "last_login": "2026-02-22"
    }))?;
    println!("✓ Updated: {}", updated);

    // Verify update
    let user = db.get("users", &id)?.unwrap();
    println!("  → Age is now: {}", user["age"]);

    // COUNT
    println!("✓ User count: {}", db.count("users")?);

    // DELETE
    let deleted = db.delete("users", &id)?;
    println!("✓ Deleted: {}", deleted);
    println!("✓ User count after delete: {}", db.count("users")?);

    // STATS
    let stats = db.stats()?;
    println!("\n📊 Database stats:");
    println!("  Tables: {}", stats.tables);
    println!("  Records: {}", stats.total_records);
    println!("  File size: {} bytes", stats.file_size_bytes);

    // Clean up
    db.close()?;
    OverDriveDB::destroy("example_crud.odb")?;
    println!("\n✓ Done! Database cleaned up.");

    Ok(())
}
