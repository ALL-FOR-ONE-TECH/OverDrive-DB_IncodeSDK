//! SQL query examples with OverDrive SDK
//!
//! Run: `cargo run --example sql_queries`

use overdrive::OverDriveDB;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OverDrive SDK: SQL Queries ===\n");

    let mut db = OverDriveDB::open("example_sql.odb")?;

    // Create table and seed data
    db.create_table("products")?;
    db.insert_batch("products", &[
        json!({"name": "Laptop", "price": 999, "category": "electronics", "stock": 15}),
        json!({"name": "Mouse", "price": 29, "category": "electronics", "stock": 200}),
        json!({"name": "Keyboard", "price": 79, "category": "electronics", "stock": 100}),
        json!({"name": "Desk", "price": 299, "category": "furniture", "stock": 30}),
        json!({"name": "Chair", "price": 199, "category": "furniture", "stock": 45}),
        json!({"name": "Pen", "price": 3, "category": "office", "stock": 500}),
        json!({"name": "Notebook", "price": 5, "category": "office", "stock": 300}),
    ])?;
    println!("✓ Seeded 7 products\n");

    // SELECT *
    println!("--- All products ---");
    let result = db.query("SELECT * FROM products")?;
    println!("  {} rows in {:.2}ms\n", result.rows.len(), result.execution_time_ms);

    // WHERE clause
    println!("--- Electronics over $50 ---");
    let result = db.query("SELECT name, price FROM products WHERE price > 50")?;
    for row in &result.rows {
        println!("  ${} — {}", row["price"], row["name"]);
    }

    // ORDER BY
    println!("\n--- All products by price (desc) ---");
    let result = db.query("SELECT name, price FROM products ORDER BY price DESC")?;
    for row in &result.rows {
        println!("  ${} — {}", row["price"], row["name"]);
    }

    // LIMIT + OFFSET
    println!("\n--- Top 3 most expensive ---");
    let result = db.query("SELECT name, price FROM products ORDER BY price DESC LIMIT 3")?;
    for row in &result.rows {
        println!("  ${} — {}", row["price"], row["name"]);
    }

    // Aggregations
    println!("\n--- Aggregations ---");
    let result = db.query("SELECT COUNT(*) FROM products")?;
    println!("  Count: {:?}", result.rows);

    let result = db.query("SELECT AVG(price) FROM products")?;
    println!("  Avg price: {:?}", result.rows);

    let result = db.query("SELECT SUM(price) FROM products")?;
    println!("  Total value: {:?}", result.rows);

    // SHOW TABLES
    println!("\n--- Tables ---");
    let result = db.query("SHOW TABLES")?;
    println!("  {:?}", result.rows);

    // Clean up
    db.close()?;
    OverDriveDB::destroy("example_sql.odb")?;
    println!("\n✓ Done!");

    Ok(())
}
