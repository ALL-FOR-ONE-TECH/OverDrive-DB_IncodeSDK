use overdrive::OverDriveDB;

fn main() {
    println!("=== OverDrive Rust SDK Test ===");

    // Clean up from previous runs
    let _ = std::fs::remove_file("rust_test.odb");
    let _ = std::fs::remove_file("rust_test.odb.wal");

    let mut db = OverDriveDB::open("rust_test.odb").expect("Failed to open DB");
    println!("Version: {}", OverDriveDB::version());

    db.create_table("users").expect("Failed to create table");
    println!("Created table 'users'");

    let id = db.insert("users", &serde_json::json!({
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30
    })).expect("Insert failed");
    println!("Inserted: {}", id);

    let user = db.get("users", &id).expect("Get failed");
    println!("Got: {:?}", user);

    db.update("users", &id, &serde_json::json!({"age": 31})).expect("Update failed");
    println!("Updated age to 31");

    let count = db.count("users").expect("Count failed");
    println!("Count: {}", count);

    let deleted = db.delete("users", &id).expect("Delete failed");
    println!("Deleted: {}", deleted);

    let count2 = db.count("users").expect("Count failed");
    println!("Count after delete: {}", count2);

    db.close().expect("Close failed");

    // Cleanup
    let _ = std::fs::remove_file("rust_test.odb");
    let _ = std::fs::remove_file("rust_test.odb.wal");
    println!("\n=== ALL RUST TESTS PASSED ===");
}
